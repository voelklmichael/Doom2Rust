use crate::filesystem::DoomFileSystem;
use crate::fixed_cstr::FixedCStr;
use crate::game_state::GameState;
use crate::i_system::console_stdout;
use crate::i_system::error;
use crate::m_fixed::Fixed;
use crate::w_wad::LumpNum;
use crate::w_wad::WWadState;
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
    pub firstflat: LumpNum,
    pub lastflat: LumpNum,
    pub numflats: i32,
    pub firstpatch: i32,
    pub lastpatch: i32,
    pub numpatches: i32,
    pub firstspritelump: LumpNum,
    pub lastspritelump: LumpNum,
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
        Self {
            firstflat: LumpNum(0),
            lastflat: LumpNum(0),
            numflats: 0,
            firstpatch: 0,
            lastpatch: 0,
            numpatches: 0,
            firstspritelump: LumpNum(0),
            lastspritelump: LumpNum(0),
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
    pub patch: LumpNum,
}
// `patch` is a byte slice starting at a post-stream (immediately after a
// column's `columnofs` lookup, before the first post's 4-byte header);
// walks it exactly like the original pointer version but with a cursor
// index instead of pointer arithmetic, so an out-of-bounds/corrupt post
// panics instead of reading adjacent memory.
pub fn draw_column_in_cache(patch: &[u8], cache: &mut [u8], originy: i32, cacheheight: i32) {
    let mut cursor: usize = 0;
    while i32::from(patch[cursor]) != 0xff {
        let topdelta = i32::from(patch[cursor]);
        let length = i32::from(patch[cursor + 1]);
        let source = &patch[cursor + 3..cursor + 3 + length as usize];
        let mut count: i32 = length;
        let mut position: i32 = originy + topdelta;
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
pub fn generate_composite(
    fs: &dyn DoomFileSystem,
    r_data: &mut RDataState,
    w_wad: &mut WWadState,
    texnum: i32,
) {
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
    let mut block: Vec<u8> = vec![0u8; r_data.texturecompositesize[texnum as usize] as usize + 128];
    let texture_patchcount = i32::from(r_data.textures[texnum as usize].patchcount);
    let texture_width = i32::from(r_data.textures[texnum as usize].width);
    let texture_height = i32::from(r_data.textures[texnum as usize].height);
    for i in 0..texture_patchcount as usize {
        let tex_patch = r_data.textures[texnum as usize].patches[i];
        // `realpatch` is the raw picture-format lump ("patch_t": width:i16,
        // height:i16, leftoffset:i16, topoffset:i16, then `width` many i32
        // columnofs entries) -- decoded field-by-field below instead of via
        // pointer-cast, same reasoning as init_textures's maptexture_t.
        let realpatch_len = lump_length(w_wad, tex_patch.patch) as usize;
        let realpatch_lump = lump_bytes(fs, w_wad, tex_patch.patch);
        let realpatch = &realpatch_lump[..realpatch_len];
        let realpatch_width = i32::from(i16::from_le_bytes(realpatch[0..2].try_into().unwrap()));
        let x1: i32 = i32::from(tex_patch.originx);
        let x2: i32 = (x1 + realpatch_width).min(texture_width);
        for x in x1.max(0)..x2 {
            if i32::from(r_data.texturecolumnlump[texnum as usize][x as usize]) < 0 {
                let colofs_off = (8 + (x - x1) * 4) as usize;
                let columnofs =
                    i32::from_le_bytes(realpatch[colofs_off..colofs_off + 4].try_into().unwrap());
                let patchcol = &realpatch[columnofs as usize..];
                let cache_off = r_data.texturecolumnofs[texnum as usize][x as usize] as usize;
                draw_column_in_cache(
                    patchcol,
                    &mut block[cache_off..],
                    i32::from(tex_patch.originy),
                    texture_height,
                );
            }
        }
    }
    r_data.texturecomposite[texnum as usize] = Some(block.into_boxed_slice());
}
pub fn generate_lookup(state: &mut GameState, texnum: i32) {
    state.render.r_data.texturecomposite[texnum as usize] = None;
    state.render.r_data.texturecompositesize[texnum as usize] = 0;
    let texture_patchcount = i32::from(state.render.r_data.textures[texnum as usize].patchcount);
    let texture_width = i32::from(state.render.r_data.textures[texnum as usize].width);
    let texture_height = i32::from(state.render.r_data.textures[texnum as usize].height);
    let mut patchcount: Vec<u8> = vec![0u8; texture_width as usize];
    for i in 0..texture_patchcount as usize {
        let tex_patch = state.render.r_data.textures[texnum as usize].patches[i];
        let realpatch_len = lump_length(&state.assets.w_wad, tex_patch.patch) as usize;
        let realpatch_lump =
            lump_bytes(&*state.assets.fs, &mut state.assets.w_wad, tex_patch.patch);
        let realpatch = &realpatch_lump[..realpatch_len];
        let realpatch_width = i32::from(i16::from_le_bytes(realpatch[0..2].try_into().unwrap()));
        let x1: i32 = i32::from(tex_patch.originx);
        let mut x2: i32 = x1 + realpatch_width;
        if x2 > texture_width {
            x2 = texture_width;
        }
        for x in x1.max(0)..x2 {
            patchcount[x as usize] = patchcount[x as usize].wrapping_add(1);
            state.render.r_data.texturecolumnlump[texnum as usize][x as usize] =
                tex_patch.patch.0 as i16;
            let colofs_off = (8 + (x - x1) * 4) as usize;
            let columnofs =
                i32::from_le_bytes(realpatch[colofs_off..colofs_off + 4].try_into().unwrap());
            state.render.r_data.texturecolumnofs[texnum as usize][x as usize] =
                (columnofs + 3) as u16;
        }
    }
    for (x, &count) in patchcount.iter().enumerate().take(texture_width as usize) {
        if count == 0 {
            doom_println!(
                state.io.platform,
                "R_GenerateLookup: column without a patch ({})",
                state.render.r_data.textures[texnum as usize].name.as_str(),
            );
            return;
        }
        if count > 1 {
            state.render.r_data.texturecolumnlump[texnum as usize][x] = -1_i16;
            state.render.r_data.texturecolumnofs[texnum as usize][x] =
                state.render.r_data.texturecompositesize[texnum as usize] as u16;
            if state.render.r_data.texturecompositesize[texnum as usize] > 0x10000 - texture_height
            {
                error(&format!("R_GenerateLookup: texture {texnum} is >64k"));
            }
            state.render.r_data.texturecompositesize[texnum as usize] += texture_height;
        }
    }
}
pub fn get_column(
    fs: &dyn DoomFileSystem,
    r_data: &mut RDataState,
    w_wad: &mut WWadState,
    tex: i32,
    mut col: i32,
) -> ColumnSource {
    col &= r_data.texturewidthmask[tex as usize];
    let lump = r_data.texturecolumnlump[tex as usize][col as usize];
    let ofs: i32 = i32::from(r_data.texturecolumnofs[tex as usize][col as usize]);
    if lump > 0 {
        let lump = LumpNum(lump as u32);
        lump_bytes(fs, w_wad, lump);
        return ColumnSource::Lump {
            lump,
            offset: ofs as usize,
        };
    }
    if r_data.texturecomposite[tex as usize].is_none() {
        generate_composite(fs, r_data, w_wad, tex);
    }
    ColumnSource::Composite {
        tex,
        offset: ofs as usize,
    }
}
fn generate_texture_hash_table(r_data: &mut RDataState) {
    r_data.textures_hashtable = vec![None; r_data.numtextures as usize];
    for i in 0..r_data.numtextures {
        r_data.textures[i as usize].index = i;
        r_data.textures[i as usize].next = None;
        let key: usize = lump_name_hash(r_data.textures[i as usize].name.as_bytes())
            .wrapping_rem(r_data.numtextures as u32) as i32 as usize;
        // Walk to the end of the bucket's chain, appending there (matches
        // the original pointer-to-pointer "rover" trick's tail-append order).
        match r_data.textures_hashtable[key] {
            None => {
                r_data.textures_hashtable[key] = Some(TextureId(i as u32));
            }
            Some(mut cursor) => {
                while let Some(next) = r_data.textures[cursor.0 as usize].next {
                    cursor = next;
                }
                r_data.textures[cursor.0 as usize].next = Some(TextureId(i as u32));
            }
        }
    }
}
pub fn init_textures(state: &mut GameState) {
    // PNAMES/TEXTURE1/TEXTURE2 are on-disk WAD lumps (raw bytes, not typed
    // Rust structs), so each is captured as a plain byte buffer once here
    // and decoded field-by-field with explicit little-endian reads below --
    // strictly more correct than the old pointer-cast version, which only
    // produced right answers on a little-endian host, and confines the two
    // unavoidable raw-pointer operations (the cache lookup and the
    // raw-parts slice construction) to one place per lump instead of
    // scattering pointer arithmetic through the whole parse.
    let pnames_lump = get_num_for_name(&state.assets.w_wad, "PNAMES");
    let pnames_len = lump_length(&state.assets.w_wad, pnames_lump) as usize;
    let pnames = lump_bytes_name(&*state.assets.fs, &mut state.assets.w_wad, "PNAMES")
        [..pnames_len]
        .to_vec();
    let nummappatches: i32 = i32::from_le_bytes(pnames[0..4].try_into().unwrap());
    let mut patchlookup: Vec<Option<LumpNum>> = vec![None; nummappatches as usize];
    for i in 0..nummappatches {
        let name_off = 4 + (i * 8) as usize;
        let patch_name = FixedCStr::<8>::from_bytes(&pnames[name_off..name_off + 8])
            .as_str()
            .into_owned();
        patchlookup[i as usize] = check_num_for_name(&state.assets.w_wad, &patch_name);
    }
    release_lump_name(&state.assets.w_wad, "PNAMES");
    let texture1_lump = get_num_for_name(&state.assets.w_wad, "TEXTURE1");
    let mut maxoff: i32 = lump_length(&state.assets.w_wad, texture1_lump);
    let maptex1 = lump_bytes_name(&*state.assets.fs, &mut state.assets.w_wad, "TEXTURE1")
        [..maxoff as usize]
        .to_vec();
    let numtextures1: i32 = i32::from_le_bytes(maptex1[0..4].try_into().unwrap());
    let (maptex2, maxoff2, numtextures2) =
        if check_num_for_name(&state.assets.w_wad, "TEXTURE2").is_none() {
            (None, 0, 0)
        } else {
            let texture2_lump = get_num_for_name(&state.assets.w_wad, "TEXTURE2");
            let maxoff2 = lump_length(&state.assets.w_wad, texture2_lump);
            let maptex2 = lump_bytes_name(&*state.assets.fs, &mut state.assets.w_wad, "TEXTURE2")
                [..maxoff2 as usize]
                .to_vec();
            let numtextures2 = i32::from_le_bytes(maptex2[0..4].try_into().unwrap());
            (Some(maptex2), maxoff2, numtextures2)
        };
    state.render.r_data.numtextures = numtextures1 + numtextures2;
    // textures/texturecolumnlump/texturecolumnofs are built via push() in
    // the loop below instead of pre-sized-then-indexed -- the loop always
    // assigns index i on iteration i, strictly in order, so there's no need
    // for a placeholder value (unlike a raw Z_Malloc'd null pointer, an
    // owned Vec<Texture>/Vec<Vec<_>> has no cheap "empty" placeholder
    // worth inventing just to pre-size).
    state.render.r_data.textures = Vec::with_capacity(state.render.r_data.numtextures as usize);
    state.render.r_data.texturecolumnlump =
        Vec::with_capacity(state.render.r_data.numtextures as usize);
    state.render.r_data.texturecolumnofs =
        Vec::with_capacity(state.render.r_data.numtextures as usize);
    state.render.r_data.texturecomposite = vec![None; state.render.r_data.numtextures as usize];
    state.render.r_data.texturecompositesize = vec![0; state.render.r_data.numtextures as usize];
    state.render.r_data.texturewidthmask = vec![0; state.render.r_data.numtextures as usize];
    state.render.r_data.textureheight = vec![Fixed::ZERO; state.render.r_data.numtextures as usize];
    let temp1 = get_num_for_name(&state.assets.w_wad, "S_START");
    let temp2 = get_num_for_name(&state.assets.w_wad, "S_END") - 1;
    let temp3: i32 = (temp2 - temp1 + 63) / 64 + (state.render.r_data.numtextures + 63) / 64;
    if console_stdout() {
        doom_print!(state.io.platform, "[");
        for _ in 0..temp3 + 9 {
            doom_print!(state.io.platform, " ");
        }
        doom_print!(state.io.platform, "]");
        for _ in 0..temp3 + 10 {
            doom_print!(state.io.platform, "\x08");
        }
    }
    let mut current_maptex: &Vec<u8> = &maptex1;
    let mut dir_index: i32 = 0;
    for i in 0..state.render.r_data.numtextures {
        if i & 63 == 0 {
            doom_print!(state.io.platform, ".");
        }
        if i == numtextures1 {
            current_maptex = maptex2.as_ref().unwrap();
            maxoff = maxoff2;
            dir_index = 0;
        }
        let dir_off = 4 + (dir_index * 4) as usize;
        let offset: i32 =
            i32::from_le_bytes(current_maptex[dir_off..dir_off + 4].try_into().unwrap());
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
        for j in 0..i32::from(mt_patchcount) {
            let p_off = 22 + (j * 10) as usize;
            let p = &mt[p_off..p_off + 10];
            let p_originx = i16::from_le_bytes(p[0..2].try_into().unwrap());
            let p_originy = i16::from_le_bytes(p[2..4].try_into().unwrap());
            let p_patch = i16::from_le_bytes(p[4..6].try_into().unwrap());
            let Some(patch) = patchlookup[p_patch as usize] else {
                error(&format!(
                    "R_InitTextures: Missing patch in texture {}",
                    mt_name.as_str(),
                ));
            };
            patches.push(TexPatch {
                originx: p_originx,
                originy: p_originy,
                patch,
            });
        }
        state.render.r_data.textures.push(Texture {
            name: mt_name,
            width: mt_width,
            height: mt_height,
            index: 0,
            next: None,
            patchcount: mt_patchcount,
            patches,
        });
        let texture_width = state.render.r_data.textures[i as usize].width;
        let texture_height = state.render.r_data.textures[i as usize].height;
        state
            .render
            .r_data
            .texturecolumnlump
            .push(vec![0i16; texture_width as usize]);
        state
            .render
            .r_data
            .texturecolumnofs
            .push(vec![0u16; texture_width as usize]);
        let mut j: i32 = 1;
        while j * 2 <= i32::from(texture_width) {
            j <<= 1;
        }
        state.render.r_data.texturewidthmask[i as usize] = j - 1;
        state.render.r_data.textureheight[i as usize] = Fixed::from_int(i32::from(texture_height));
        dir_index += 1;
    }
    release_lump_name(&state.assets.w_wad, "TEXTURE1");
    if maptex2.is_some() {
        release_lump_name(&state.assets.w_wad, "TEXTURE2");
    }
    for i in 0..state.render.r_data.numtextures {
        generate_lookup(state, i);
    }
    state.render.r_data.texturetranslation =
        vec![0; (state.render.r_data.numtextures + 1) as usize];
    for i in 0..state.render.r_data.numtextures {
        state.render.r_data.texturetranslation[i as usize] = i;
    }
    generate_texture_hash_table(&mut state.render.r_data);
}
pub fn init_flats(r_data: &mut RDataState, w_wad: &WWadState) {
    r_data.firstflat = get_num_for_name(w_wad, "F_START") + 1;
    r_data.lastflat = get_num_for_name(w_wad, "F_END") - 1;
    r_data.numflats = r_data.lastflat - r_data.firstflat + 1;
    r_data.flattranslation = vec![0; (r_data.numflats + 1) as usize];
    for i in 0..r_data.numflats {
        r_data.flattranslation[i as usize] = i;
    }
}
pub fn init_sprite_lumps(state: &mut GameState) {
    state.render.r_data.firstspritelump = get_num_for_name(&state.assets.w_wad, "S_START") + 1;
    state.render.r_data.lastspritelump = get_num_for_name(&state.assets.w_wad, "S_END") - 1;
    state.render.r_data.numspritelumps =
        state.render.r_data.lastspritelump - state.render.r_data.firstspritelump + 1;
    state.render.r_data.spritewidth =
        vec![Fixed::ZERO; state.render.r_data.numspritelumps as usize];
    state.render.r_data.spriteoffset =
        vec![Fixed::ZERO; state.render.r_data.numspritelumps as usize];
    state.render.r_data.spritetopoffset =
        vec![Fixed::ZERO; state.render.r_data.numspritelumps as usize];
    for i in 0..state.render.r_data.numspritelumps {
        if i & 63 == 0 {
            doom_print!(state.io.platform, ".");
        }
        // Only the fixed 8-byte patch_t header (width/height/leftoffset/
        // topoffset) is needed here -- decode those fields explicitly from
        // the raw lump bytes instead of reinterpreting via pointer cast, so
        // this loop body doesn't need to know the lump's true length (the
        // header is always present regardless of `width`, unlike
        // `columnofs`, which is a true flexible-array tail elsewhere).
        let lump = lump_bytes(
            &*state.assets.fs,
            &mut state.assets.w_wad,
            state.render.r_data.firstspritelump + i,
        );
        let header = &lump[..8];
        let width = i16::from_le_bytes(header[0..2].try_into().unwrap());
        let leftoffset = i16::from_le_bytes(header[4..6].try_into().unwrap());
        let topoffset = i16::from_le_bytes(header[6..8].try_into().unwrap());
        state.render.r_data.spritewidth[i as usize] = Fixed::from_int(i32::from(width));
        state.render.r_data.spriteoffset[i as usize] = Fixed::from_int(i32::from(leftoffset));
        state.render.r_data.spritetopoffset[i as usize] = Fixed::from_int(i32::from(topoffset));
    }
}
pub fn init_colormaps(fs: &dyn DoomFileSystem, r_data: &mut RDataState, w_wad: &mut WWadState) {
    let lump = get_num_for_name(w_wad, "COLORMAP");
    let lumplen = lump_length(w_wad, lump) as usize;
    r_data.colormaps = lump_bytes(fs, w_wad, lump)[..lumplen].to_vec();
}
pub fn r_init_data(state: &mut GameState) {
    init_textures(state);
    doom_print!(state.io.platform, ".");
    init_flats(&mut state.render.r_data, &state.assets.w_wad);
    doom_print!(state.io.platform, ".");
    init_sprite_lumps(state);
    doom_print!(state.io.platform, ".");
    init_colormaps(
        &*state.assets.fs,
        &mut state.render.r_data,
        &mut state.assets.w_wad,
    );
}
pub fn flat_num_for_name(r_data: &RDataState, w_wad: &WWadState, name: &str) -> i32 {
    let i = check_num_for_name(w_wad, name)
        .unwrap_or_else(|| error(&format!("R_FlatNumForName: {name} not found")));
    i - r_data.firstflat
}
/// The number of the texture called `name`, if there is one (`-` means "no
/// texture" and is texture 0).
pub fn check_texture_num_for_name(state: &RDataState, name: &str) -> Option<i32> {
    if name.as_bytes().first() == Some(&b'-') {
        return Some(0);
    }
    let key: i32 = lump_name_hash(name.as_bytes()).wrapping_rem(state.numtextures as u32) as i32;
    let mut cursor = state.textures_hashtable[key as usize];
    while let Some(id) = cursor {
        let texture = &state.textures[id.0 as usize];
        if texture.name.eq_bytes_ignore_ascii_case(name.as_bytes()) {
            return Some(texture.index);
        }
        cursor = texture.next;
    }
    None
}
pub fn texture_num_for_name(state: &RDataState, name: &str) -> i32 {
    check_texture_num_for_name(state, name)
        .unwrap_or_else(|| error(&format!("R_TextureNumForName: {name} not found")))
}
pub fn precache_level(state: &mut GameState) {
    if state.game.g_game.demoplayback {
        return;
    }
    let mut flatpresent: Vec<u8> = vec![0u8; state.render.r_data.numflats as usize];
    for i in 0..(state.world.p_setup.numsectors as usize) {
        flatpresent[state.world.p_setup.sectors[i].floorpic as usize] = 1;
        flatpresent[state.world.p_setup.sectors[i].ceilingpic as usize] = 1;
    }
    state.render.r_data.flatmemory = 0;
    for i in 0..state.render.r_data.numflats {
        if flatpresent[i as usize] != 0 {
            let lump = state.render.r_data.firstflat + i;
            state.render.r_data.flatmemory += state.assets.w_wad.lumpinfo[lump.index()].size;
            lump_bytes(&*state.assets.fs, &mut state.assets.w_wad, lump);
        }
    }
    let mut texturepresent: Vec<u8> = vec![0u8; state.render.r_data.numtextures as usize];
    for i in 0..(state.world.p_setup.numsides as usize) {
        texturepresent[state.world.p_setup.sides[i].toptexture as usize] = 1;
        texturepresent[state.world.p_setup.sides[i].midtexture as usize] = 1;
        texturepresent[state.world.p_setup.sides[i].bottomtexture as usize] = 1;
    }
    texturepresent[state.render.r_sky.skytexture as usize] = 1;
    state.render.r_data.texturememory = 0;
    for (i, &present) in texturepresent.iter().enumerate() {
        if present != 0 {
            let patchcount = i32::from(state.render.r_data.textures[i].patchcount);
            for j in 0..patchcount as usize {
                let lump = state.render.r_data.textures[i].patches[j].patch;
                state.render.r_data.texturememory += state.assets.w_wad.lumpinfo[lump.index()].size;
                lump_bytes(&*state.assets.fs, &mut state.assets.w_wad, lump);
            }
        }
    }
    let mut spritepresent: Vec<u8> = vec![0u8; state.render.r_things.numsprites as usize];
    for mobj_id in mobj_thinker_ids(&state.world.p_mobj, &state.world.p_tick) {
        spritepresent[state.world.p_mobj.mo(mobj_id).sprite as usize] = 1;
    }
    state.render.r_data.spritememory = 0;
    for (i, &present) in spritepresent.iter().enumerate() {
        if present != 0 {
            for j in 0..(state.render.r_things.sprites[i].numframes) as usize {
                for k in 0..8 {
                    let lump = state.render.r_data.firstspritelump
                        + i32::from(state.render.r_things.sprites[i].spriteframes[j].lump[k]);
                    state.render.r_data.spritememory +=
                        state.assets.w_wad.lumpinfo[lump.index()].size;
                    lump_bytes(&*state.assets.fs, &mut state.assets.w_wad, lump);
                }
            }
        }
    }
}
