use crate::fixed_cstr::FixedCStr;
use crate::game_state::GameState;
use crate::i_system::I_ConsoleStdout;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACBITS;
use crate::p_mobj::mobj_t;
use crate::p_mobj::thinker_t;
use crate::p_mobj::ThinkerFn;
use crate::p_tick::P_ThinkerRaw;
use crate::r_draw::ColumnSource;
use crate::r_defs::lighttable_t;
use crate::stdint_types::byte;
use crate::w_wad::W_CacheLumpNum;
use crate::w_wad::W_LumpLength;
use crate::w_wad::W_LumpNameHash;
use crate::w_wad::{W_CacheLumpName, W_CheckNumForName, W_GetNumForName, W_ReleaseLumpName};

pub struct RDataState {
    pub firstflat: i32,
    pub lastflat: i32,
    pub numflats: i32,
    pub firstpatch: i32,
    pub lastpatch: i32,
    pub numpatches: i32,
    pub firstspritelump: i32,
    pub lastspritelump: i32,
    pub numspritelumps: i32,
    pub numtextures: i32,
    pub textures: Vec<Box<texture_t>>,
    pub textures_hashtable: Vec<Option<TextureId>>,
    pub texturewidthmask: Vec<i32>,
    pub textureheight: Vec<fixed_t>,
    pub texturecompositesize: Vec<i32>,
    pub texturecolumnlump: Vec<Vec<i16>>,
    pub texturecolumnofs: Vec<Vec<u16>>,
    pub texturecomposite: Vec<Option<Box<[u8]>>>,
    pub flattranslation: Vec<i32>,
    pub texturetranslation: Vec<i32>,
    pub spritewidth: Vec<fixed_t>,
    pub spriteoffset: Vec<fixed_t>,
    pub spritetopoffset: Vec<fixed_t>,
    pub colormaps: Vec<lighttable_t>,
    pub flatmemory: i32,
    pub texturememory: i32,
    pub spritememory: i32,
}

impl RDataState {
    pub const fn new() -> Self {
        RDataState {
            firstflat: 0,
            lastflat: 0,
            numflats: 0,
            firstpatch: 0,
            lastpatch: 0,
            numpatches: 0,
            firstspritelump: 0,
            lastspritelump: 0,
            numspritelumps: 0,
            numtextures: 0,
            textures: Vec::new(),
            textures_hashtable: Vec::new(),
            texturewidthmask: Vec::new(),
            textureheight: Vec::new(),
            texturecompositesize: Vec::new(),
            texturecolumnlump: Vec::new(),
            texturecolumnofs: Vec::new(),
            texturecomposite: Vec::new(),
            flattranslation: Vec::new(),
            texturetranslation: Vec::new(),
            spritewidth: Vec::new(),
            spriteoffset: Vec::new(),
            spritetopoffset: Vec::new(),
            colormaps: Vec::new(),
            flatmemory: 0,
            texturememory: 0,
            spritememory: 0,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct post_t {
    pub topdelta: byte,
    pub length: byte,
}
pub type column_t = post_t;
pub type texture_t = texture_s;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TextureId(pub u32);

// No longer Copy/Clone: `patches` owns a Vec instead of being a C flexible
// array member -- confirmed nothing copies a texture_t by value anywhere,
// only ever accessed through the owning Vec<Box<texture_t>> in
// RDataState.textures or a raw pointer derived from it.
pub struct texture_s {
    pub name: FixedCStr<8>,
    pub width: i16,
    pub height: i16,
    pub index: i32,
    pub next: Option<TextureId>,
    pub patchcount: i16,
    pub patches: Vec<texpatch_t>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct texpatch_t {
    pub originx: i16,
    pub originy: i16,
    pub patch: i32,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct mappatch_t {
    pub originx: i16,
    pub originy: i16,
    pub patch: i16,
    pub stepdir: i16,
    pub colormap: i16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct maptexture_t {
    pub name: FixedCStr<8>,
    pub masked: i32,
    pub width: i16,
    pub height: i16,
    pub obsolete: i32,
    pub patchcount: i16,
    pub patches: [mappatch_t; 1],
}
// `patch` is a byte slice starting at a post-stream (immediately after a
// column's `columnofs` lookup, before the first post's 4-byte header);
// walks it exactly like the original pointer version but with a cursor
// index instead of pointer arithmetic, so an out-of-bounds/corrupt post
// panics instead of reading adjacent memory.
pub fn R_DrawColumnInCache(patch: &[u8], cache: &mut [u8], originy: i32, cacheheight: i32) {
    let mut count: i32 = 0;
    let mut position: i32 = 0;
    let mut cursor: usize = 0;
    while patch[cursor] as i32 != 0xff_i32 {
        let topdelta = patch[cursor] as i32;
        let length = patch[cursor + 1] as i32;
        let source = &patch[cursor + 3..cursor + 3 + length as usize];
        count = length;
        position = originy + topdelta;
        if position < 0_i32 {
            count += position;
            position = 0_i32;
        }
        if position + count > cacheheight {
            count = cacheheight - position;
        }
        if count > 0_i32 {
            cache[position as usize..(position + count) as usize]
                .copy_from_slice(&source[..count as usize]);
        }
        cursor += length as usize + 4;
    }
}
pub fn R_GenerateComposite(state: &mut GameState, texnum: i32) {
    let mut x: i32;
    let mut x1: i32;
    let mut x2: i32;
    let mut i: i32;
    // Built locally and only stored into texturecomposite once fully drawn,
    // unlike the old Z_Malloc user-backpointer trick which wrote the
    // (still-empty) allocation into that slot immediately -- safe here
    // since nothing re-enters this slot mid-loop (R_DrawColumnInCache is a
    // plain column-copy routine, no recursion back into R_GetColumn).
    //
    // Padded by 128 bytes past the exact composite size for the same reason
    // as W_CacheLumpNum's cache buffer: r_draw.rs's column readers mask with
    // `& 127` (matching vanilla's R_DrawColumn verbatim), which can read up
    // to 127 bytes past a short column's real data when that column sits
    // near the end of the buffer. Vanilla's zone allocator happened to leave
    // slack there; an exactly-sized Vec doesn't, so it panics instead.
    let mut block: Vec<u8> =
        vec![0u8; state.r_data.texturecompositesize[texnum as usize] as usize + 128];
    let texture_patchcount = state.r_data.textures[texnum as usize].patchcount as i32;
    let texture_width = state.r_data.textures[texnum as usize].width as i32;
    let texture_height = state.r_data.textures[texnum as usize].height as i32;
    i = 0_i32;
    while i < texture_patchcount {
        let tex_patch = state.r_data.textures[texnum as usize].patches[i as usize];
        // `realpatch` is the raw picture-format lump ("patch_t": width:i16,
        // height:i16, leftoffset:i16, topoffset:i16, then `width` many i32
        // columnofs entries) -- decoded field-by-field below instead of via
        // pointer-cast, same reasoning as R_InitTextures's maptexture_t.
        let realpatch_len = W_LumpLength(&mut state.w_wad, tex_patch.patch as u32) as usize;
        let realpatch_ptr = W_CacheLumpNum(state, tex_patch.patch) as *const u8;
        let realpatch = unsafe { ::core::slice::from_raw_parts(realpatch_ptr, realpatch_len) };
        let realpatch_width = i16::from_le_bytes(realpatch[0..2].try_into().unwrap()) as i32;
        x1 = tex_patch.originx as i32;
        x2 = x1 + realpatch_width;
        if x1 < 0_i32 {
            x = 0_i32;
        } else {
            x = x1;
        }
        if x2 > texture_width {
            x2 = texture_width;
        }
        while x < x2 {
            if (state.r_data.texturecolumnlump[texnum as usize][x as usize] as i32) < 0_i32 {
                let colofs_off = (8 + (x - x1) * 4) as usize;
                let columnofs =
                    i32::from_le_bytes(realpatch[colofs_off..colofs_off + 4].try_into().unwrap());
                let patchcol = &realpatch[columnofs as usize..];
                let cache_off = state.r_data.texturecolumnofs[texnum as usize][x as usize] as usize;
                R_DrawColumnInCache(
                    patchcol,
                    &mut block[cache_off..],
                    tex_patch.originy as i32,
                    texture_height,
                );
            }
            x += 1;
        }
        i += 1;
    }
    state.r_data.texturecomposite[texnum as usize] = Some(block.into_boxed_slice());
}
pub fn R_GenerateLookup(state: &mut GameState, texnum: i32) {
    let mut patchcount: Vec<byte>;
    let mut x: i32;
    let mut x1: i32;
    let mut x2: i32;
    let mut i: i32;
    state.r_data.texturecomposite[texnum as usize] = None;
    state.r_data.texturecompositesize[texnum as usize] = 0_i32;
    let texture_patchcount = state.r_data.textures[texnum as usize].patchcount as i32;
    let texture_width = state.r_data.textures[texnum as usize].width as i32;
    let texture_height = state.r_data.textures[texnum as usize].height as i32;
    patchcount = vec![0u8; texture_width as usize];
    i = 0_i32;
    while i < texture_patchcount {
        let tex_patch = state.r_data.textures[texnum as usize].patches[i as usize];
        let realpatch_len = W_LumpLength(&mut state.w_wad, tex_patch.patch as u32) as usize;
        let realpatch_ptr = W_CacheLumpNum(state, tex_patch.patch) as *const u8;
        let realpatch = unsafe { ::core::slice::from_raw_parts(realpatch_ptr, realpatch_len) };
        let realpatch_width = i16::from_le_bytes(realpatch[0..2].try_into().unwrap()) as i32;
        x1 = tex_patch.originx as i32;
        x2 = x1 + realpatch_width;
        if x1 < 0_i32 {
            x = 0_i32;
        } else {
            x = x1;
        }
        if x2 > texture_width {
            x2 = texture_width;
        }
        while x < x2 {
            patchcount[x as usize] = patchcount[x as usize].wrapping_add(1);
            state.r_data.texturecolumnlump[texnum as usize][x as usize] = tex_patch.patch as i16;
            let colofs_off = (8 + (x - x1) * 4) as usize;
            let columnofs =
                i32::from_le_bytes(realpatch[colofs_off..colofs_off + 4].try_into().unwrap());
            state.r_data.texturecolumnofs[texnum as usize][x as usize] = (columnofs + 3_i32) as u16;
            x += 1;
        }
        i += 1;
    }
    x = 0_i32;
    while x < texture_width {
        if patchcount[x as usize] == 0 {
            println!(
                "R_GenerateLookup: column without a patch ({})",
                state.r_data.textures[texnum as usize].name.as_str(),
            );
            return;
        }
        if patchcount[x as usize] as i32 > 1_i32 {
            state.r_data.texturecolumnlump[texnum as usize][x as usize] = -1_i32 as i16;
            state.r_data.texturecolumnofs[texnum as usize][x as usize] =
                state.r_data.texturecompositesize[texnum as usize] as u16;
            if state.r_data.texturecompositesize[texnum as usize] > 0x10000_i32 - texture_height {
                I_Error(&format!("R_GenerateLookup: texture {} is >64k", texnum));
            }
            state.r_data.texturecompositesize[texnum as usize] += texture_height;
        }
        x += 1;
    }
}
pub unsafe fn R_GetColumn(state: &mut GameState, mut tex: i32, mut col: i32) -> ColumnSource {
    let mut lump: i32 = 0;
    let mut ofs: i32 = 0;
    col &= state.r_data.texturewidthmask[tex as usize];
    lump = state.r_data.texturecolumnlump[tex as usize][col as usize] as i32;
    ofs = state.r_data.texturecolumnofs[tex as usize][col as usize] as i32;
    if lump > 0_i32 {
        W_CacheLumpNum(state, lump);
        return ColumnSource::Lump {
            lump,
            offset: ofs as usize,
        };
    }
    if state.r_data.texturecomposite[tex as usize].is_none() {
        R_GenerateComposite(state, tex);
    }
    return ColumnSource::Composite {
        tex,
        offset: ofs as usize,
    };
}
fn GenerateTextureHashTable(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut key: i32 = 0;
    state.r_data.textures_hashtable = vec![None; state.r_data.numtextures as usize];
    i = 0_i32;
    while i < state.r_data.numtextures {
        state.r_data.textures[i as usize].index = i;
        state.r_data.textures[i as usize].next = None;
        key = W_LumpNameHash(state.r_data.textures[i as usize].name.as_bytes())
            .wrapping_rem(state.r_data.numtextures as u32) as i32;
        // Walk to the end of the bucket's chain, appending there (matches
        // the original pointer-to-pointer "rover" trick's tail-append order).
        match state.r_data.textures_hashtable[key as usize] {
            None => {
                state.r_data.textures_hashtable[key as usize] = Some(TextureId(i as u32));
            }
            Some(mut cursor) => {
                loop {
                    match state.r_data.textures[cursor.0 as usize].next {
                        Some(next) => cursor = next,
                        None => break,
                    }
                }
                state.r_data.textures[cursor.0 as usize].next = Some(TextureId(i as u32));
            }
        }
        i += 1;
    }
}
pub unsafe fn R_InitTextures(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut patchlookup: Vec<i32>;
    let mut nummappatches: i32 = 0;
    let mut offset: i32 = 0;
    let mut maxoff: i32 = 0;
    let mut maxoff2: i32 = 0;
    let mut numtextures1: i32 = 0;
    let mut numtextures2: i32 = 0;
    let mut temp1: i32 = 0;
    let mut temp2: i32 = 0;
    let mut temp3: i32 = 0;
    // PNAMES/TEXTURE1/TEXTURE2 are on-disk WAD lumps (raw bytes, not typed
    // Rust structs), so each is captured as a plain byte buffer once here
    // and decoded field-by-field with explicit little-endian reads below --
    // strictly more correct than the old pointer-cast version, which only
    // produced right answers on a little-endian host, and confines the two
    // unavoidable raw-pointer operations (the cache lookup and the
    // raw-parts slice construction) to one place per lump instead of
    // scattering pointer arithmetic through the whole parse.
    let pnames_lump = W_GetNumForName(&mut state.w_wad, "PNAMES") as u32;
    let pnames_len = W_LumpLength(&mut state.w_wad, pnames_lump) as usize;
    let pnames_ptr = W_CacheLumpName(state, "PNAMES") as *const u8;
    let pnames = ::core::slice::from_raw_parts(pnames_ptr, pnames_len).to_vec();
    nummappatches = i32::from_le_bytes(pnames[0..4].try_into().unwrap());
    patchlookup = vec![0i32; nummappatches as usize];
    i = 0_i32;
    while i < nummappatches {
        let name_off = 4 + (i * 8_i32) as usize;
        let patch_name = FixedCStr::<8>::from_bytes(&pnames[name_off..name_off + 8])
            .as_str()
            .into_owned();
        patchlookup[i as usize] = W_CheckNumForName(&mut state.w_wad, &patch_name);
        i += 1;
    }
    W_ReleaseLumpName(&mut state.w_wad, "PNAMES");
    let texture1_lump = W_GetNumForName(&mut state.w_wad, "TEXTURE1") as u32;
    maxoff = W_LumpLength(&mut state.w_wad, texture1_lump);
    let maptex1_ptr = W_CacheLumpName(state, "TEXTURE1") as *const u8;
    let maptex1 = ::core::slice::from_raw_parts(maptex1_ptr, maxoff as usize).to_vec();
    numtextures1 = i32::from_le_bytes(maptex1[0..4].try_into().unwrap());
    let maptex2 = if W_CheckNumForName(&mut state.w_wad, "TEXTURE2") != -1_i32 {
        let texture2_lump = W_GetNumForName(&mut state.w_wad, "TEXTURE2") as u32;
        maxoff2 = W_LumpLength(&mut state.w_wad, texture2_lump);
        let maptex2_ptr = W_CacheLumpName(state, "TEXTURE2") as *const u8;
        numtextures2 = i32::from_le_bytes(
            ::core::slice::from_raw_parts(maptex2_ptr, 4)
                .try_into()
                .unwrap(),
        );
        Some(::core::slice::from_raw_parts(maptex2_ptr, maxoff2 as usize).to_vec())
    } else {
        numtextures2 = 0_i32;
        maxoff2 = 0_i32;
        None
    };
    state.r_data.numtextures = numtextures1 + numtextures2;
    // textures/texturecolumnlump/texturecolumnofs are built via push() in
    // the loop below instead of pre-sized-then-indexed -- the loop always
    // assigns index i on iteration i, strictly in order, so there's no need
    // for a placeholder value (unlike a raw Z_Malloc'd null pointer, an
    // owned Vec<Box<texture_t>>/Vec<Vec<_>> has no cheap "empty" placeholder
    // worth inventing just to pre-size).
    state.r_data.textures = Vec::with_capacity(state.r_data.numtextures as usize);
    state.r_data.texturecolumnlump = Vec::with_capacity(state.r_data.numtextures as usize);
    state.r_data.texturecolumnofs = Vec::with_capacity(state.r_data.numtextures as usize);
    state.r_data.texturecomposite = vec![None; state.r_data.numtextures as usize];
    state.r_data.texturecompositesize = vec![0_i32; state.r_data.numtextures as usize];
    state.r_data.texturewidthmask = vec![0_i32; state.r_data.numtextures as usize];
    state.r_data.textureheight = vec![0 as fixed_t; state.r_data.numtextures as usize];
    temp1 = W_GetNumForName(&mut state.w_wad, "S_START");
    temp2 = W_GetNumForName(&mut state.w_wad, "S_END") - 1_i32;
    temp3 = (temp2 - temp1 + 63_i32) / 64_i32 + (state.r_data.numtextures + 63_i32) / 64_i32;
    if I_ConsoleStdout() {
        print!("[");
        i = 0_i32;
        while i < temp3 + 9_i32 {
            print!(" ");
            i += 1;
        }
        print!("]");
        i = 0_i32;
        while i < temp3 + 10_i32 {
            print!("\x08");
            i += 1;
        }
    }
    let mut current_maptex: &Vec<u8> = &maptex1;
    let mut dir_index: i32 = 0;
    i = 0_i32;
    while i < state.r_data.numtextures {
        if i & 63_i32 == 0 {
            print!(".");
        }
        if i == numtextures1 {
            current_maptex = maptex2.as_ref().unwrap();
            maxoff = maxoff2;
            dir_index = 0;
        }
        let dir_off = 4 + (dir_index * 4) as usize;
        offset = i32::from_le_bytes(current_maptex[dir_off..dir_off + 4].try_into().unwrap());
        if offset > maxoff {
            I_Error("R_InitTextures: bad texture directory");
        }
        // maptexture_t's on-disk layout: name[8], masked:i32 (unused),
        // width:i16, height:i16, columndirectory:i32 (unused/obsolete),
        // patchcount:i16 -- a fixed 22-byte header, followed immediately by
        // `patchcount` 10-byte mappatch_t entries (the C flexible-array-
        // member tail the old code reached via `&raw mut (*mtexture).patches`).
        let mt = &current_maptex[offset as usize..];
        let mt_name = FixedCStr::<8>::from_bytes(&mt[0..8]);
        let mt_width = i16::from_le_bytes(mt[12..14].try_into().unwrap());
        let mt_height = i16::from_le_bytes(mt[14..16].try_into().unwrap());
        let mt_patchcount = i16::from_le_bytes(mt[20..22].try_into().unwrap());
        // patches is built directly as a Vec (pushed patchcount times below)
        // instead of over-allocating size_of::<texture_t>() +
        // size_of::<texpatch_t>()*(patchcount-1) raw bytes for a C flexible
        // array member tail.
        let mut patches: Vec<texpatch_t> = Vec::with_capacity(mt_patchcount.max(0) as usize);
        j = 0_i32;
        while j < mt_patchcount as i32 {
            let p_off = 22 + (j * 10) as usize;
            let p = &mt[p_off..p_off + 10];
            let p_originx = i16::from_le_bytes(p[0..2].try_into().unwrap());
            let p_originy = i16::from_le_bytes(p[2..4].try_into().unwrap());
            let p_patch = i16::from_le_bytes(p[4..6].try_into().unwrap());
            let patch_entry = texpatch_t {
                originx: p_originx,
                originy: p_originy,
                patch: patchlookup[p_patch as usize],
            };
            if patch_entry.patch == -1_i32 {
                I_Error(&format!(
                    "R_InitTextures: Missing patch in texture {}",
                    mt_name.as_str(),
                ));
            }
            patches.push(patch_entry);
            j += 1;
        }
        state.r_data.textures.push(Box::new(texture_t {
            name: mt_name,
            width: mt_width,
            height: mt_height,
            index: 0,
            next: None,
            patchcount: mt_patchcount,
            patches,
        }));
        let texture_width = state.r_data.textures[i as usize].width;
        let texture_height = state.r_data.textures[i as usize].height;
        state
            .r_data
            .texturecolumnlump
            .push(vec![0i16; texture_width as usize]);
        state
            .r_data
            .texturecolumnofs
            .push(vec![0u16; texture_width as usize]);
        j = 1_i32;
        while j * 2_i32 <= texture_width as i32 {
            j <<= 1_i32;
        }
        state.r_data.texturewidthmask[i as usize] = j - 1_i32;
        state.r_data.textureheight[i as usize] = ((texture_height as i32) << FRACBITS) as fixed_t;
        i += 1;
        dir_index += 1;
    }
    W_ReleaseLumpName(&mut state.w_wad, "TEXTURE1");
    if maptex2.is_some() {
        W_ReleaseLumpName(&mut state.w_wad, "TEXTURE2");
    }
    i = 0_i32;
    while i < state.r_data.numtextures {
        R_GenerateLookup(state, i);
        i += 1;
    }
    state.r_data.texturetranslation = vec![0_i32; (state.r_data.numtextures + 1_i32) as usize];
    i = 0_i32;
    while i < state.r_data.numtextures {
        state.r_data.texturetranslation[i as usize] = i;
        i += 1;
    }
    GenerateTextureHashTable(state);
}
pub fn R_InitFlats(state: &mut GameState) {
    let mut i: i32 = 0;
    state.r_data.firstflat = W_GetNumForName(&mut state.w_wad, "F_START") + 1_i32;
    state.r_data.lastflat = W_GetNumForName(&mut state.w_wad, "F_END") - 1_i32;
    state.r_data.numflats = state.r_data.lastflat - state.r_data.firstflat + 1_i32;
    state.r_data.flattranslation = vec![0_i32; (state.r_data.numflats + 1_i32) as usize];
    i = 0_i32;
    while i < state.r_data.numflats {
        state.r_data.flattranslation[i as usize] = i;
        i += 1;
    }
}
pub fn R_InitSpriteLumps(state: &mut GameState) {
    let mut i: i32 = 0;
    state.r_data.firstspritelump = W_GetNumForName(&mut state.w_wad, "S_START") + 1_i32;
    state.r_data.lastspritelump = W_GetNumForName(&mut state.w_wad, "S_END") - 1_i32;
    state.r_data.numspritelumps =
        state.r_data.lastspritelump - state.r_data.firstspritelump + 1_i32;
    state.r_data.spritewidth = vec![0 as fixed_t; state.r_data.numspritelumps as usize];
    state.r_data.spriteoffset = vec![0 as fixed_t; state.r_data.numspritelumps as usize];
    state.r_data.spritetopoffset = vec![0 as fixed_t; state.r_data.numspritelumps as usize];
    i = 0_i32;
    while i < state.r_data.numspritelumps {
        if i & 63_i32 == 0 {
            print!(".");
        }
        // Only the fixed 8-byte patch_t header (width/height/leftoffset/
        // topoffset) is needed here -- decode those fields explicitly from
        // the raw lump bytes instead of reinterpreting via pointer cast, so
        // this loop body doesn't need to know the lump's true length (the
        // header is always present regardless of `width`, unlike
        // `columnofs`, which is a true flexible-array tail elsewhere).
        let lump_ptr = W_CacheLumpNum(state, state.r_data.firstspritelump + i) as *const u8;
        let header = unsafe { ::core::slice::from_raw_parts(lump_ptr, 8) };
        let width = i16::from_le_bytes(header[0..2].try_into().unwrap());
        let leftoffset = i16::from_le_bytes(header[4..6].try_into().unwrap());
        let topoffset = i16::from_le_bytes(header[6..8].try_into().unwrap());
        state.r_data.spritewidth[i as usize] = ((width as i32) << FRACBITS) as fixed_t;
        state.r_data.spriteoffset[i as usize] = ((leftoffset as i32) << FRACBITS) as fixed_t;
        state.r_data.spritetopoffset[i as usize] = ((topoffset as i32) << FRACBITS) as fixed_t;
        i += 1;
    }
}
pub fn R_InitColormaps(state: &mut GameState) {
    let mut lump: i32 = 0;
    lump = W_GetNumForName(&mut state.w_wad, "COLORMAP");
    let lump_ptr = W_CacheLumpNum(state, lump) as *const lighttable_t;
    let lumplen = W_LumpLength(&mut state.w_wad, lump as u32) as usize;
    state.r_data.colormaps = unsafe { ::core::slice::from_raw_parts(lump_ptr, lumplen) }.to_vec();
}
pub fn R_InitData(state: &mut GameState) {
    unsafe { R_InitTextures(state) };
    print!(".");
    R_InitFlats(state);
    print!(".");
    R_InitSpriteLumps(state);
    print!(".");
    R_InitColormaps(state);
}
pub fn R_FlatNumForName(state: &mut GameState, name: &str) -> i32 {
    let mut i: i32 = 0;
    i = W_CheckNumForName(&mut state.w_wad, name);
    if i == -1_i32 {
        I_Error(&format!("R_FlatNumForName: {} not found", name));
    }
    i - state.r_data.firstflat
}
pub fn R_CheckTextureNumForName(state: &RDataState, name: &str) -> i32 {
    let mut key: i32 = 0;
    if name.as_bytes().first() == Some(&b'-') {
        return 0_i32;
    }
    key = W_LumpNameHash(name.as_bytes()).wrapping_rem(state.numtextures as u32) as i32;
    let mut cursor = state.textures_hashtable[key as usize];
    while let Some(id) = cursor {
        let texture = &state.textures[id.0 as usize];
        if texture.name.eq_bytes_ignore_ascii_case(name.as_bytes()) {
            return texture.index;
        }
        cursor = texture.next;
    }
    -1_i32
}
pub fn R_TextureNumForName(state: &mut RDataState, name: &str) -> i32 {
    let mut i: i32 = 0;
    i = R_CheckTextureNumForName(state, name);
    if i == -1_i32 {
        I_Error(&format!("R_TextureNumForName: {} not found", name));
    }
    i
}
pub unsafe fn R_PrecacheLevel(state: &mut GameState) {
    let mut flatpresent: Vec<u8>;
    let mut texturepresent: Vec<u8>;
    let mut spritepresent: Vec<u8>;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut k: i32 = 0;
    let mut lump: i32 = 0;
    let mut th: *mut thinker_t = ::core::ptr::null_mut::<thinker_t>();
    if state.g_game.demoplayback {
        return;
    }
    flatpresent = vec![0u8; state.r_data.numflats as usize];
    i = 0_i32;
    while i < state.p_setup.numsectors {
        flatpresent[state.p_setup.sectors[i as usize].floorpic as usize] = 1_u8;
        flatpresent[state.p_setup.sectors[i as usize].ceilingpic as usize] = 1_u8;
        i += 1;
    }
    state.r_data.flatmemory = 0_i32;
    i = 0_i32;
    while i < state.r_data.numflats {
        if flatpresent[i as usize] != 0 {
            lump = state.r_data.firstflat + i;
            state.r_data.flatmemory += state.w_wad.lumpinfo[lump as usize].size;
            W_CacheLumpNum(state, lump);
        }
        i += 1;
    }
    texturepresent = vec![0u8; state.r_data.numtextures as usize];
    i = 0_i32;
    while i < state.p_setup.numsides {
        texturepresent[state.p_setup.sides[i as usize].toptexture as usize] = 1_u8;
        texturepresent[state.p_setup.sides[i as usize].midtexture as usize] = 1_u8;
        texturepresent[state.p_setup.sides[i as usize].bottomtexture as usize] = 1_u8;
        i += 1;
    }
    texturepresent[state.r_sky.skytexture as usize] = 1_u8;
    state.r_data.texturememory = 0_i32;
    i = 0_i32;
    while i < state.r_data.numtextures {
        if texturepresent[i as usize] != 0 {
            let patchcount = state.r_data.textures[i as usize].patchcount as i32;
            j = 0_i32;
            while j < patchcount {
                lump = state.r_data.textures[i as usize].patches[j as usize].patch;
                state.r_data.texturememory += state.w_wad.lumpinfo[lump as usize].size;
                W_CacheLumpNum(state, lump);
                j += 1;
            }
        }
        i += 1;
    }
    spritepresent = vec![0u8; state.r_things.numsprites as usize];
    let mut cursor = state.p_tick.head();
    while let Some(id) = cursor {
        th = P_ThinkerRaw(state, id);
        if matches!((*th).function, ThinkerFn::Mobj(_)) {
            spritepresent[(*(th as *mut mobj_t)).sprite as usize] = 1_u8;
        }
        cursor = state.p_tick.next(id);
    }
    state.r_data.spritememory = 0_i32;
    i = 0_i32;
    while i < state.r_things.numsprites {
        if spritepresent[i as usize] != 0 {
            j = 0_i32;
            while j < state.r_things.sprites[i as usize].numframes {
                k = 0_i32;
                while k < 8_i32 {
                    lump = state.r_data.firstspritelump
                        + state.r_things.sprites[i as usize].spriteframes[j as usize].lump
                            [k as usize] as i32;
                    state.r_data.spritememory += state.w_wad.lumpinfo[lump as usize].size;
                    W_CacheLumpNum(state, lump);
                    k += 1;
                }
                j += 1;
            }
        }
        i += 1;
    }
}
