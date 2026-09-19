use crate::fixed_cstr::FixedCStr;
use crate::game_state::GameState;
use crate::i_system::console_stdout;
use crate::i_system::error;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::p_tick::mobj_thinker_ids;
use crate::r_defs::LightTable;
use crate::r_draw::ColumnSource;
use crate::w_wad::lump_bytes;
use crate::w_wad::lump_length;
use crate::w_wad::lump_name_hash;
use crate::w_wad::{check_num_for_name, get_num_for_name, lump_bytes_name, release_lump_name};

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
    pub textures: Vec<Texture>,
    pub textures_hashtable: Vec<Option<TextureId>>,
    pub texturewidthmask: Vec<i32>,
    pub textureheight: Vec<Fixed>,
    pub texturecompositesize: Vec<i32>,
    pub texturecolumnlump: Vec<Vec<i16>>,
    pub texturecolumnofs: Vec<Vec<u16>>,
    pub texturecomposite: Vec<Option<Box<[u8]>>>,
    pub flattranslation: Vec<i32>,
    pub texturetranslation: Vec<i32>,
    pub spritewidth: Vec<Fixed>,
    pub spriteoffset: Vec<Fixed>,
    pub spritetopoffset: Vec<Fixed>,
    pub colormaps: Vec<LightTable>,
    pub flatmemory: i32,
    pub texturememory: i32,
    pub spritememory: i32,
}

impl Default for RDataState {
    fn default() -> Self {
        Self::new()
    }
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TextureId(pub u32);

// No longer Copy/Clone: `patches` owns a Vec instead of being a C flexible
// array member -- confirmed nothing copies a Texture by value anywhere,
// only ever accessed through the owning Vec<Texture> in
// RDataState.textures or a raw pointer derived from it.
pub struct Texture {
    pub name: FixedCStr<8>,
    pub width: i16,
    pub height: i16,
    pub index: i32,
    pub next: Option<TextureId>,
    pub patchcount: i16,
    pub patches: Vec<TexPatch>,
}
#[derive(Copy, Clone)]
pub struct TexPatch {
    pub originx: i16,
    pub originy: i16,
    pub patch: i32,
}
// `patch` is a byte slice starting at a post-stream (immediately after a
// column's `columnofs` lookup, before the first post's 4-byte header);
// walks it exactly like the original pointer version but with a cursor
// index instead of pointer arithmetic, so an out-of-bounds/corrupt post
// panics instead of reading adjacent memory.
pub fn draw_column_in_cache(patch: &[u8], cache: &mut [u8], originy: i32, cacheheight: i32) {
    let mut count: i32;
    let mut position: i32;
    let mut cursor: usize = 0;
    while patch[cursor] as i32 != 0xff {
        let topdelta = patch[cursor] as i32;
        let length = patch[cursor + 1] as i32;
        let source = &patch[cursor + 3..cursor + 3 + length as usize];
        count = length;
        position = originy + topdelta;
        if position < 0 {
            count += position;
            position = 0;
        }
        if position + count > cacheheight {
            count = cacheheight - position;
        }
        if count > 0 {
            cache[position as usize..(position + count) as usize]
                .copy_from_slice(&source[..count as usize]);
        }
        cursor += length as usize + 4;
    }
}
pub fn generate_composite(state: &mut GameState, texnum: i32) {
    let mut x: i32;
    let mut x1: i32;
    let mut x2: i32;
    // Built locally and only stored into texturecomposite once fully drawn,
    // unlike the old Z_Malloc user-backpointer trick which wrote the
    // (still-empty) allocation into that slot immediately -- safe here
    // since nothing re-enters this slot mid-loop (draw_column_in_cache is a
    // plain column-copy routine, no recursion back into get_column).
    //
    // Padded by 128 bytes past the exact composite size for the same reason
    // as lump_bytes's cache buffer: r_draw.rs's column readers mask with
    // `& 127` (matching vanilla's draw_column verbatim), which can read up
    // to 127 bytes past a short column's real data when that column sits
    // near the end of the buffer. Vanilla's zone allocator happened to leave
    // slack there; an exactly-sized Vec doesn't, so it panics instead.
    let mut block: Vec<u8> =
        vec![0u8; state.r_data.texturecompositesize[texnum as usize] as usize + 128];
    let texture_patchcount = state.r_data.textures[texnum as usize].patchcount as i32;
    let texture_width = state.r_data.textures[texnum as usize].width as i32;
    let texture_height = state.r_data.textures[texnum as usize].height as i32;
    for i in 0..texture_patchcount {
        let tex_patch = state.r_data.textures[texnum as usize].patches[i as usize];
        // `realpatch` is the raw picture-format lump ("patch_t": width:i16,
        // height:i16, leftoffset:i16, topoffset:i16, then `width` many i32
        // columnofs entries) -- decoded field-by-field below instead of via
        // pointer-cast, same reasoning as init_textures's maptexture_t.
        let realpatch_len = lump_length(&mut state.w_wad, tex_patch.patch as u32) as usize;
        let realpatch_lump = lump_bytes(state, tex_patch.patch);
        let realpatch = &realpatch_lump[..realpatch_len];
        let realpatch_width = i16::from_le_bytes(realpatch[0..2].try_into().unwrap()) as i32;
        x1 = tex_patch.originx as i32;
        x2 = x1 + realpatch_width;
        if x1 < 0 {
            x = 0;
        } else {
            x = x1;
        }
        if x2 > texture_width {
            x2 = texture_width;
        }
        while x < x2 {
            if (state.r_data.texturecolumnlump[texnum as usize][x as usize] as i32) < 0 {
                let colofs_off = (8 + (x - x1) * 4) as usize;
                let columnofs =
                    i32::from_le_bytes(realpatch[colofs_off..colofs_off + 4].try_into().unwrap());
                let patchcol = &realpatch[columnofs as usize..];
                let cache_off = state.r_data.texturecolumnofs[texnum as usize][x as usize] as usize;
                draw_column_in_cache(
                    patchcol,
                    &mut block[cache_off..],
                    tex_patch.originy as i32,
                    texture_height,
                );
            }
            x += 1;
        }
    }
    state.r_data.texturecomposite[texnum as usize] = Some(block.into_boxed_slice());
}
pub fn generate_lookup(state: &mut GameState, texnum: i32) {
    let mut patchcount: Vec<u8>;
    let mut x: i32;
    let mut x1: i32;
    let mut x2: i32;
    state.r_data.texturecomposite[texnum as usize] = None;
    state.r_data.texturecompositesize[texnum as usize] = 0;
    let texture_patchcount = state.r_data.textures[texnum as usize].patchcount as i32;
    let texture_width = state.r_data.textures[texnum as usize].width as i32;
    let texture_height = state.r_data.textures[texnum as usize].height as i32;
    patchcount = vec![0u8; texture_width as usize];
    for i in 0..texture_patchcount {
        let tex_patch = state.r_data.textures[texnum as usize].patches[i as usize];
        let realpatch_len = lump_length(&mut state.w_wad, tex_patch.patch as u32) as usize;
        let realpatch_lump = lump_bytes(state, tex_patch.patch);
        let realpatch = &realpatch_lump[..realpatch_len];
        let realpatch_width = i16::from_le_bytes(realpatch[0..2].try_into().unwrap()) as i32;
        x1 = tex_patch.originx as i32;
        x2 = x1 + realpatch_width;
        if x1 < 0 {
            x = 0;
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
            state.r_data.texturecolumnofs[texnum as usize][x as usize] = (columnofs + 3) as u16;
            x += 1;
        }
    }
    for x in 0..texture_width {
        if patchcount[x as usize] == 0 {
            doom_println!(
                state.platform,
                "R_GenerateLookup: column without a patch ({})",
                state.r_data.textures[texnum as usize].name.as_str(),
            );
            return;
        }
        if patchcount[x as usize] as i32 > 1 {
            state.r_data.texturecolumnlump[texnum as usize][x as usize] = -1_i16;
            state.r_data.texturecolumnofs[texnum as usize][x as usize] =
                state.r_data.texturecompositesize[texnum as usize] as u16;
            if state.r_data.texturecompositesize[texnum as usize] > 0x10000 - texture_height {
                error(&format!("R_GenerateLookup: texture {} is >64k", texnum));
            }
            state.r_data.texturecompositesize[texnum as usize] += texture_height;
        }
    }
}
pub fn get_column(state: &mut GameState, tex: i32, mut col: i32) -> ColumnSource {
    col &= state.r_data.texturewidthmask[tex as usize];
    let lump: i32 = state.r_data.texturecolumnlump[tex as usize][col as usize] as i32;
    let ofs: i32 = state.r_data.texturecolumnofs[tex as usize][col as usize] as i32;
    if lump > 0 {
        lump_bytes(state, lump);
        return ColumnSource::Lump {
            lump,
            offset: ofs as usize,
        };
    }
    if state.r_data.texturecomposite[tex as usize].is_none() {
        generate_composite(state, tex);
    }
    ColumnSource::Composite {
        tex,
        offset: ofs as usize,
    }
}
fn generate_texture_hash_table(state: &mut GameState) {
    let mut key: i32;
    state.r_data.textures_hashtable = vec![None; state.r_data.numtextures as usize];
    for i in 0..state.r_data.numtextures {
        state.r_data.textures[i as usize].index = i;
        state.r_data.textures[i as usize].next = None;
        key = lump_name_hash(state.r_data.textures[i as usize].name.as_bytes())
            .wrapping_rem(state.r_data.numtextures as u32) as i32;
        // Walk to the end of the bucket's chain, appending there (matches
        // the original pointer-to-pointer "rover" trick's tail-append order).
        match state.r_data.textures_hashtable[key as usize] {
            None => {
                state.r_data.textures_hashtable[key as usize] = Some(TextureId(i as u32));
            }
            Some(mut cursor) => {
                while let Some(next) = state.r_data.textures[cursor.0 as usize].next {
                    cursor = next;
                }
                state.r_data.textures[cursor.0 as usize].next = Some(TextureId(i as u32));
            }
        }
    }
}
pub fn init_textures(state: &mut GameState) {
    let mut i: i32;
    let mut j: i32;
    let mut patchlookup: Vec<i32>;

    let mut offset: i32;
    let mut maxoff: i32;
    let maxoff2: i32;

    let numtextures2: i32;

    // PNAMES/TEXTURE1/TEXTURE2 are on-disk WAD lumps (raw bytes, not typed
    // Rust structs), so each is captured as a plain byte buffer once here
    // and decoded field-by-field with explicit little-endian reads below --
    // strictly more correct than the old pointer-cast version, which only
    // produced right answers on a little-endian host, and confines the two
    // unavoidable raw-pointer operations (the cache lookup and the
    // raw-parts slice construction) to one place per lump instead of
    // scattering pointer arithmetic through the whole parse.
    let pnames_lump = get_num_for_name(&mut state.w_wad, "PNAMES") as u32;
    let pnames_len = lump_length(&mut state.w_wad, pnames_lump) as usize;
    let pnames = lump_bytes_name(state, "PNAMES")[..pnames_len].to_vec();
    let nummappatches: i32 = i32::from_le_bytes(pnames[0..4].try_into().unwrap());
    patchlookup = vec![0i32; nummappatches as usize];
    for i in 0..nummappatches {
        let name_off = 4 + (i * 8) as usize;
        let patch_name = FixedCStr::<8>::from_bytes(&pnames[name_off..name_off + 8])
            .as_str()
            .into_owned();
        patchlookup[i as usize] = check_num_for_name(&mut state.w_wad, &patch_name);
    }
    release_lump_name(&mut state.w_wad, "PNAMES");
    let texture1_lump = get_num_for_name(&mut state.w_wad, "TEXTURE1") as u32;
    maxoff = lump_length(&mut state.w_wad, texture1_lump);
    let maptex1 = lump_bytes_name(state, "TEXTURE1")[..maxoff as usize].to_vec();
    let numtextures1: i32 = i32::from_le_bytes(maptex1[0..4].try_into().unwrap());
    let maptex2 = if check_num_for_name(&mut state.w_wad, "TEXTURE2") != -1 {
        let texture2_lump = get_num_for_name(&mut state.w_wad, "TEXTURE2") as u32;
        maxoff2 = lump_length(&mut state.w_wad, texture2_lump);
        let maptex2 = lump_bytes_name(state, "TEXTURE2")[..maxoff2 as usize].to_vec();
        numtextures2 = i32::from_le_bytes(maptex2[0..4].try_into().unwrap());
        Some(maptex2)
    } else {
        numtextures2 = 0;
        maxoff2 = 0;
        None
    };
    state.r_data.numtextures = numtextures1 + numtextures2;
    // textures/texturecolumnlump/texturecolumnofs are built via push() in
    // the loop below instead of pre-sized-then-indexed -- the loop always
    // assigns index i on iteration i, strictly in order, so there's no need
    // for a placeholder value (unlike a raw Z_Malloc'd null pointer, an
    // owned Vec<Texture>/Vec<Vec<_>> has no cheap "empty" placeholder
    // worth inventing just to pre-size).
    state.r_data.textures = Vec::with_capacity(state.r_data.numtextures as usize);
    state.r_data.texturecolumnlump = Vec::with_capacity(state.r_data.numtextures as usize);
    state.r_data.texturecolumnofs = Vec::with_capacity(state.r_data.numtextures as usize);
    state.r_data.texturecomposite = vec![None; state.r_data.numtextures as usize];
    state.r_data.texturecompositesize = vec![0; state.r_data.numtextures as usize];
    state.r_data.texturewidthmask = vec![0; state.r_data.numtextures as usize];
    state.r_data.textureheight = vec![0; state.r_data.numtextures as usize];
    let temp1: i32 = get_num_for_name(&mut state.w_wad, "S_START");
    let temp2: i32 = get_num_for_name(&mut state.w_wad, "S_END") - 1;
    let temp3: i32 = (temp2 - temp1 + 63) / 64 + (state.r_data.numtextures + 63) / 64;
    if console_stdout() {
        doom_print!(state.platform, "[");
        for _ in 0..temp3 + 9 {
            doom_print!(state.platform, " ");
        }
        doom_print!(state.platform, "]");
        for _ in 0..temp3 + 10 {
            doom_print!(state.platform, "\x08");
        }
    }
    let mut current_maptex: &Vec<u8> = &maptex1;
    let mut dir_index: i32 = 0;
    i = 0;
    while i < state.r_data.numtextures {
        if i & 63 == 0 {
            doom_print!(state.platform, ".");
        }
        if i == numtextures1 {
            current_maptex = maptex2.as_ref().unwrap();
            maxoff = maxoff2;
            dir_index = 0;
        }
        let dir_off = 4 + (dir_index * 4) as usize;
        offset = i32::from_le_bytes(current_maptex[dir_off..dir_off + 4].try_into().unwrap());
        if offset > maxoff {
            error("R_InitTextures: bad texture directory");
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
        // instead of over-allocating size_of::<Texture>() +
        // size_of::<TexPatch>()*(patchcount-1) raw bytes for a C flexible
        // array member tail.
        let mut patches: Vec<TexPatch> = Vec::with_capacity(mt_patchcount.max(0) as usize);
        for j in 0..mt_patchcount as i32 {
            let p_off = 22 + (j * 10) as usize;
            let p = &mt[p_off..p_off + 10];
            let p_originx = i16::from_le_bytes(p[0..2].try_into().unwrap());
            let p_originy = i16::from_le_bytes(p[2..4].try_into().unwrap());
            let p_patch = i16::from_le_bytes(p[4..6].try_into().unwrap());
            let patch_entry = TexPatch {
                originx: p_originx,
                originy: p_originy,
                patch: patchlookup[p_patch as usize],
            };
            if patch_entry.patch == -1 {
                error(&format!(
                    "R_InitTextures: Missing patch in texture {}",
                    mt_name.as_str(),
                ));
            }
            patches.push(patch_entry);
        }
        state.r_data.textures.push(Texture {
            name: mt_name,
            width: mt_width,
            height: mt_height,
            index: 0,
            next: None,
            patchcount: mt_patchcount,
            patches,
        });
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
        j = 1;
        while j * 2 <= texture_width as i32 {
            j <<= 1;
        }
        state.r_data.texturewidthmask[i as usize] = j - 1;
        state.r_data.textureheight[i as usize] = ((texture_height as i32) << FRACBITS) as Fixed;
        i += 1;
        dir_index += 1;
    }
    release_lump_name(&mut state.w_wad, "TEXTURE1");
    if maptex2.is_some() {
        release_lump_name(&mut state.w_wad, "TEXTURE2");
    }
    for i in 0..state.r_data.numtextures {
        generate_lookup(state, i);
    }
    state.r_data.texturetranslation = vec![0; (state.r_data.numtextures + 1) as usize];
    for i in 0..state.r_data.numtextures {
        state.r_data.texturetranslation[i as usize] = i;
    }
    generate_texture_hash_table(state);
}
pub fn init_flats(state: &mut GameState) {
    state.r_data.firstflat = get_num_for_name(&mut state.w_wad, "F_START") + 1;
    state.r_data.lastflat = get_num_for_name(&mut state.w_wad, "F_END") - 1;
    state.r_data.numflats = state.r_data.lastflat - state.r_data.firstflat + 1;
    state.r_data.flattranslation = vec![0; (state.r_data.numflats + 1) as usize];
    for i in 0..state.r_data.numflats {
        state.r_data.flattranslation[i as usize] = i;
    }
}
pub fn init_sprite_lumps(state: &mut GameState) {
    state.r_data.firstspritelump = get_num_for_name(&mut state.w_wad, "S_START") + 1;
    state.r_data.lastspritelump = get_num_for_name(&mut state.w_wad, "S_END") - 1;
    state.r_data.numspritelumps = state.r_data.lastspritelump - state.r_data.firstspritelump + 1;
    state.r_data.spritewidth = vec![0; state.r_data.numspritelumps as usize];
    state.r_data.spriteoffset = vec![0; state.r_data.numspritelumps as usize];
    state.r_data.spritetopoffset = vec![0; state.r_data.numspritelumps as usize];
    for i in 0..state.r_data.numspritelumps {
        if i & 63 == 0 {
            doom_print!(state.platform, ".");
        }
        // Only the fixed 8-byte patch_t header (width/height/leftoffset/
        // topoffset) is needed here -- decode those fields explicitly from
        // the raw lump bytes instead of reinterpreting via pointer cast, so
        // this loop body doesn't need to know the lump's true length (the
        // header is always present regardless of `width`, unlike
        // `columnofs`, which is a true flexible-array tail elsewhere).
        let lump = lump_bytes(state, state.r_data.firstspritelump + i);
        let header = &lump[..8];
        let width = i16::from_le_bytes(header[0..2].try_into().unwrap());
        let leftoffset = i16::from_le_bytes(header[4..6].try_into().unwrap());
        let topoffset = i16::from_le_bytes(header[6..8].try_into().unwrap());
        state.r_data.spritewidth[i as usize] = ((width as i32) << FRACBITS) as Fixed;
        state.r_data.spriteoffset[i as usize] = ((leftoffset as i32) << FRACBITS) as Fixed;
        state.r_data.spritetopoffset[i as usize] = ((topoffset as i32) << FRACBITS) as Fixed;
    }
}
pub fn init_colormaps(state: &mut GameState) {
    let lump: i32 = get_num_for_name(&mut state.w_wad, "COLORMAP");
    let lumplen = lump_length(&mut state.w_wad, lump as u32) as usize;
    state.r_data.colormaps = lump_bytes(state, lump)[..lumplen].to_vec();
}
pub fn r_init_data(state: &mut GameState) {
    init_textures(state);
    doom_print!(state.platform, ".");
    init_flats(state);
    doom_print!(state.platform, ".");
    init_sprite_lumps(state);
    doom_print!(state.platform, ".");
    init_colormaps(state);
}
pub fn flat_num_for_name(state: &mut GameState, name: &str) -> i32 {
    let i: i32 = check_num_for_name(&mut state.w_wad, name);
    if i == -1 {
        error(&format!("R_FlatNumForName: {} not found", name));
    }
    i - state.r_data.firstflat
}
pub fn check_texture_num_for_name(state: &RDataState, name: &str) -> i32 {
    if name.as_bytes().first() == Some(&b'-') {
        return 0;
    }
    let key: i32 = lump_name_hash(name.as_bytes()).wrapping_rem(state.numtextures as u32) as i32;
    let mut cursor = state.textures_hashtable[key as usize];
    while let Some(id) = cursor {
        let texture = &state.textures[id.0 as usize];
        if texture.name.eq_bytes_ignore_ascii_case(name.as_bytes()) {
            return texture.index;
        }
        cursor = texture.next;
    }
    -1
}
pub fn texture_num_for_name(state: &mut RDataState, name: &str) -> i32 {
    let i: i32 = check_texture_num_for_name(state, name);
    if i == -1 {
        error(&format!("R_TextureNumForName: {} not found", name));
    }
    i
}
pub fn precache_level(state: &mut GameState) {
    let mut flatpresent: Vec<u8>;
    let mut texturepresent: Vec<u8>;
    let mut spritepresent: Vec<u8>;
    let mut lump: i32;
    if state.g_game.demoplayback {
        return;
    }
    flatpresent = vec![0u8; state.r_data.numflats as usize];
    for i in 0..(state.p_setup.numsectors as usize) {
        flatpresent[state.p_setup.sectors[i].floorpic as usize] = 1;
        flatpresent[state.p_setup.sectors[i].ceilingpic as usize] = 1;
    }
    state.r_data.flatmemory = 0;
    for i in 0..state.r_data.numflats {
        if flatpresent[i as usize] != 0 {
            lump = state.r_data.firstflat + i;
            state.r_data.flatmemory += state.w_wad.lumpinfo[lump as usize].size;
            lump_bytes(state, lump);
        }
    }
    texturepresent = vec![0u8; state.r_data.numtextures as usize];
    for i in 0..(state.p_setup.numsides as usize) {
        texturepresent[state.p_setup.sides[i].toptexture as usize] = 1;
        texturepresent[state.p_setup.sides[i].midtexture as usize] = 1;
        texturepresent[state.p_setup.sides[i].bottomtexture as usize] = 1;
    }
    texturepresent[state.r_sky.skytexture as usize] = 1;
    state.r_data.texturememory = 0;
    for (i, &present) in texturepresent.iter().enumerate() {
        if present != 0 {
            let patchcount = state.r_data.textures[i].patchcount as i32;
            for j in 0..patchcount {
                lump = state.r_data.textures[i].patches[j as usize].patch;
                state.r_data.texturememory += state.w_wad.lumpinfo[lump as usize].size;
                lump_bytes(state, lump);
            }
        }
    }
    spritepresent = vec![0u8; state.r_things.numsprites as usize];
    for mobj_id in mobj_thinker_ids(state) {
        spritepresent[state.p_mobj.mo(mobj_id).sprite as usize] = 1;
    }
    state.r_data.spritememory = 0;
    for (i, &present) in spritepresent.iter().enumerate() {
        if present != 0 {
            for j in 0..state.r_things.sprites[i].numframes {
                for k in 0..8 {
                    lump = state.r_data.firstspritelump
                        + state.r_things.sprites[i].spriteframes[j as usize].lump[k] as i32;
                    state.r_data.spritememory += state.w_wad.lumpinfo[lump as usize].size;
                    lump_bytes(state, lump);
                }
            }
        }
    }
}
