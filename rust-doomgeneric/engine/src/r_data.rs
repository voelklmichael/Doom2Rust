use crate::fixed_cstr::FixedCStr;
use crate::game_state::GameState;
use crate::hu_lib::patch_t;
use crate::i_system::I_ConsoleStdout;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACBITS;
use crate::p_mobj::mobj_t;
use crate::p_mobj::thinker_t;
use crate::p_mobj::ThinkerFn;
use crate::r_defs::lighttable_t;
use crate::r_defs::spriteframe_t;
use crate::stdint_types::byte;
use crate::stdint_types::size_t;
use crate::w_wad::W_CacheLumpNum;
use crate::w_wad::W_LumpLength;
use crate::w_wad::W_LumpNameHash;
use crate::w_wad::{
    wad_name8_to_string, W_CacheLumpName, W_CheckNumForName, W_GetNumForName, W_ReleaseLumpName,
};
use crate::mem_compat::memcpy;

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
pub unsafe fn R_DrawColumnInCache(
    mut patch: *mut column_t,
    mut cache: *mut byte,
    mut originy: i32,
    mut cacheheight: i32,
) {
    let mut count: i32 = 0;
    let mut position: i32 = 0;
    let mut source: *mut byte = ::core::ptr::null_mut::<byte>();
    while (*patch).topdelta as i32 != 0xff as i32 {
        source = (patch as *mut byte).offset(3 as i32 as isize);
        count = (*patch).length as i32;
        position = originy + (*patch).topdelta as i32;
        if position < 0 as i32 {
            count += position;
            position = 0 as i32;
        }
        if position + count > cacheheight {
            count = cacheheight - position;
        }
        if count > 0 as i32 {
            memcpy(
                cache.offset(position as isize) as *mut ::core::ffi::c_void,
                source as *const ::core::ffi::c_void,
                count as size_t,
            );
        }
        patch = (patch as *mut byte)
            .offset((*patch).length as i32 as isize)
            .offset(4 as i32 as isize) as *mut column_t;
    }
}
pub unsafe fn R_GenerateComposite(state: &mut GameState, mut texnum: i32) {
    let mut texture: *mut texture_t = ::core::ptr::null_mut::<texture_t>();
    let mut patch: *mut texpatch_t = ::core::ptr::null_mut::<texpatch_t>();
    let mut realpatch: *mut patch_t = ::core::ptr::null_mut::<patch_t>();
    let mut x: i32 = 0;
    let mut x1: i32 = 0;
    let mut x2: i32 = 0;
    let mut i: i32 = 0;
    let mut patchcol: *mut column_t = ::core::ptr::null_mut::<column_t>();
    let mut collump: *mut i16 = ::core::ptr::null_mut::<i16>();
    let mut colofs: *mut u16 = ::core::ptr::null_mut::<u16>();
    texture = state.r_data.textures[texnum as usize].as_mut() as *mut texture_t;
    // Built locally and only stored into texturecomposite once fully drawn,
    // unlike the old Z_Malloc user-backpointer trick which wrote the
    // (still-empty) allocation into that slot immediately -- safe here
    // since nothing re-enters this slot mid-loop (R_DrawColumnInCache is a
    // plain column-copy routine, no recursion back into R_GetColumn).
    let mut block: Vec<u8> =
        vec![0u8; state.r_data.texturecompositesize[texnum as usize] as usize];
    collump = state.r_data.texturecolumnlump[texnum as usize].as_mut_ptr();
    colofs = state.r_data.texturecolumnofs[texnum as usize].as_mut_ptr();
    patch = (*texture).patches.as_mut_ptr();
    i = 0 as i32;
    patch = (*texture).patches.as_mut_ptr();
    while i < (*texture).patchcount as i32 {
        realpatch = W_CacheLumpNum(state, (*patch).patch) as *mut patch_t;
        x1 = (*patch).originx as i32;
        x2 = x1 + (*realpatch).width as i32;
        if x1 < 0 as i32 {
            x = 0 as i32;
        } else {
            x = x1;
        }
        if x2 > (*texture).width as i32 {
            x2 = (*texture).width as i32;
        }
        while x < x2 {
            if !(*collump.offset(x as isize) as i32 >= 0 as i32) {
                patchcol = (realpatch as *mut byte).offset(
                    *(&raw const (*realpatch).columnofs as *const i32).offset((x - x1) as isize)
                        as isize,
                ) as *mut column_t;
                R_DrawColumnInCache(
                    patchcol,
                    block
                        .as_mut_ptr()
                        .offset(*colofs.offset(x as isize) as i32 as isize),
                    (*patch).originy as i32,
                    (*texture).height as i32,
                );
            }
            x += 1;
        }
        i += 1;
        patch = patch.offset(1);
    }
    state.r_data.texturecomposite[texnum as usize] = Some(block.into_boxed_slice());
}
pub unsafe fn R_GenerateLookup(state: &mut GameState, mut texnum: i32) {
    let mut texture: *mut texture_t = ::core::ptr::null_mut::<texture_t>();
    let mut patchcount: Vec<byte> = Vec::new();
    let mut patch: *mut texpatch_t = ::core::ptr::null_mut::<texpatch_t>();
    let mut realpatch: *mut patch_t = ::core::ptr::null_mut::<patch_t>();
    let mut x: i32 = 0;
    let mut x1: i32 = 0;
    let mut x2: i32 = 0;
    let mut i: i32 = 0;
    let mut collump: *mut i16 = ::core::ptr::null_mut::<i16>();
    let mut colofs: *mut u16 = ::core::ptr::null_mut::<u16>();
    texture = state.r_data.textures[texnum as usize].as_mut() as *mut texture_t;
    state.r_data.texturecomposite[texnum as usize] = None;
    state.r_data.texturecompositesize[texnum as usize] = 0 as i32;
    collump = state.r_data.texturecolumnlump[texnum as usize].as_mut_ptr();
    colofs = state.r_data.texturecolumnofs[texnum as usize].as_mut_ptr();
    patchcount = vec![0u8; (*texture).width as usize];
    patch = (*texture).patches.as_mut_ptr();
    i = 0 as i32;
    patch = (*texture).patches.as_mut_ptr();
    while i < (*texture).patchcount as i32 {
        realpatch = W_CacheLumpNum(state, (*patch).patch) as *mut patch_t;
        x1 = (*patch).originx as i32;
        x2 = x1 + (*realpatch).width as i32;
        if x1 < 0 as i32 {
            x = 0 as i32;
        } else {
            x = x1;
        }
        if x2 > (*texture).width as i32 {
            x2 = (*texture).width as i32;
        }
        while x < x2 {
            patchcount[x as usize] = patchcount[x as usize].wrapping_add(1);
            *collump.offset(x as isize) = (*patch).patch as i16;
            *colofs.offset(x as isize) = (*(&raw const (*realpatch).columnofs as *const i32)
                .offset((x - x1) as isize)
                + 3 as i32) as u16;
            x += 1;
        }
        i += 1;
        patch = patch.offset(1);
    }
    x = 0 as i32;
    while x < (*texture).width as i32 {
        if patchcount[x as usize] == 0 {
            println!(
                "R_GenerateLookup: column without a patch ({})",
                (*texture).name.as_str(),
            );
            return;
        }
        if patchcount[x as usize] as i32 > 1 as i32 {
            *collump.offset(x as isize) = -(1 as i32) as i16;
            *colofs.offset(x as isize) = state.r_data.texturecompositesize[texnum as usize] as u16;
            if state.r_data.texturecompositesize[texnum as usize]
                > 0x10000 as i32 - (*texture).height as i32
            {
                I_Error(&format!("R_GenerateLookup: texture {} is >64k", texnum));
            }
            state.r_data.texturecompositesize[texnum as usize] += (*texture).height as i32;
        }
        x += 1;
    }
}
pub unsafe fn R_GetColumn(state: &mut GameState, mut tex: i32, mut col: i32) -> *mut byte {
    let mut lump: i32 = 0;
    let mut ofs: i32 = 0;
    col &= state.r_data.texturewidthmask[tex as usize];
    lump = state.r_data.texturecolumnlump[tex as usize][col as usize] as i32;
    ofs = state.r_data.texturecolumnofs[tex as usize][col as usize] as i32;
    if lump > 0 as i32 {
        return (W_CacheLumpNum(state, lump) as *mut byte).offset(ofs as isize);
    }
    if state.r_data.texturecomposite[tex as usize].is_none() {
        R_GenerateComposite(state, tex);
    }
    return state.r_data.texturecomposite[tex as usize]
        .as_mut()
        .unwrap()
        .as_mut_ptr()
        .offset(ofs as isize);
}
unsafe fn GenerateTextureHashTable(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut key: i32 = 0;
    state.r_data.textures_hashtable = vec![None; state.r_data.numtextures as usize];
    i = 0 as i32;
    while i < state.r_data.numtextures {
        (*state.r_data.textures[i as usize]).index = i;
        (*state.r_data.textures[i as usize]).next = None;
        key = W_LumpNameHash((*state.r_data.textures[i as usize]).name.as_bytes())
            .wrapping_rem(state.r_data.numtextures as u32) as i32;
        // Walk to the end of the bucket's chain, appending there (matches
        // the original pointer-to-pointer "rover" trick's tail-append order).
        match state.r_data.textures_hashtable[key as usize] {
            None => {
                state.r_data.textures_hashtable[key as usize] = Some(TextureId(i as u32));
            }
            Some(mut cursor) => {
                loop {
                    match (*state.r_data.textures[cursor.0 as usize]).next {
                        Some(next) => cursor = next,
                        None => break,
                    }
                }
                (*state.r_data.textures[cursor.0 as usize]).next = Some(TextureId(i as u32));
            }
        }
        i += 1;
    }
}
pub unsafe fn R_InitTextures(state: &mut GameState) {
    let mut mtexture: *mut maptexture_t = ::core::ptr::null_mut::<maptexture_t>();
    let mut texture: *mut texture_t = ::core::ptr::null_mut::<texture_t>();
    let mut mpatch: *mut mappatch_t = ::core::ptr::null_mut::<mappatch_t>();
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut maptex: *mut i32 = ::core::ptr::null_mut::<i32>();
    let mut maptex2: *mut i32 = ::core::ptr::null_mut::<i32>();
    let mut maptex1: *mut i32 = ::core::ptr::null_mut::<i32>();
    let mut names: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut name_p: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut patchlookup: Vec<i32>;
    let mut nummappatches: i32 = 0;
    let mut offset: i32 = 0;
    let mut maxoff: i32 = 0;
    let mut maxoff2: i32 = 0;
    let mut numtextures1: i32 = 0;
    let mut numtextures2: i32 = 0;
    let mut directory: *mut i32 = ::core::ptr::null_mut::<i32>();
    let mut temp1: i32 = 0;
    let mut temp2: i32 = 0;
    let mut temp3: i32 = 0;
    names = W_CacheLumpName(state, "PNAMES") as *mut u8;
    nummappatches = *(names as *mut i32);
    name_p = names.offset(4 as i32 as isize);
    patchlookup = vec![0i32; nummappatches as usize];
    i = 0 as i32;
    while i < nummappatches {
        let patch_name = wad_name8_to_string(
            name_p.offset((i * 8 as i32) as isize) as *const ::core::ffi::c_char,
        );
        patchlookup[i as usize] = W_CheckNumForName(&mut state.w_wad, &patch_name);
        i += 1;
    }
    W_ReleaseLumpName(&mut state.w_wad, "PNAMES");
    maptex1 = W_CacheLumpName(state, "TEXTURE1") as *mut i32;
    maptex = maptex1;
    numtextures1 = *maptex;
    let texture1_lump = W_GetNumForName(&mut state.w_wad, "TEXTURE1") as u32;
    maxoff = W_LumpLength(&mut state.w_wad, texture1_lump);
    directory = maptex.offset(1 as i32 as isize);
    if W_CheckNumForName(&mut state.w_wad, "TEXTURE2") != -(1 as i32) {
        maptex2 = W_CacheLumpName(state, "TEXTURE2") as *mut i32;
        numtextures2 = *maptex2;
        let texture2_lump = W_GetNumForName(&mut state.w_wad, "TEXTURE2") as u32;
        maxoff2 = W_LumpLength(&mut state.w_wad, texture2_lump);
    } else {
        maptex2 = ::core::ptr::null_mut::<i32>();
        numtextures2 = 0 as i32;
        maxoff2 = 0 as i32;
    }
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
    state.r_data.texturecompositesize = vec![0 as i32; state.r_data.numtextures as usize];
    state.r_data.texturewidthmask = vec![0 as i32; state.r_data.numtextures as usize];
    state.r_data.textureheight = vec![0 as fixed_t; state.r_data.numtextures as usize];
    temp1 = W_GetNumForName(&mut state.w_wad, "S_START");
    temp2 = W_GetNumForName(&mut state.w_wad, "S_END") - 1 as i32;
    temp3 = (temp2 - temp1 + 63 as i32) / 64 as i32
        + (state.r_data.numtextures + 63 as i32) / 64 as i32;
    if I_ConsoleStdout() {
        print!("[");
        i = 0 as i32;
        while i < temp3 + 9 as i32 {
            print!(" ");
            i += 1;
        }
        print!("]");
        i = 0 as i32;
        while i < temp3 + 10 as i32 {
            print!("\x08");
            i += 1;
        }
    }
    i = 0 as i32;
    while i < state.r_data.numtextures {
        if i & 63 as i32 == 0 {
            print!(".");
        }
        if i == numtextures1 {
            maptex = maptex2;
            maxoff = maxoff2;
            directory = maptex.offset(1 as i32 as isize);
        }
        offset = *directory;
        if offset > maxoff {
            I_Error("R_InitTextures: bad texture directory");
        }
        mtexture = (maptex as *mut byte).offset(offset as isize) as *mut maptexture_t;
        // patches is built directly as a Vec (pushed patchcount times below)
        // instead of over-allocating size_of::<texture_t>() +
        // size_of::<texpatch_t>()*(patchcount-1) raw bytes for a C flexible
        // array member tail.
        let mut patches: Vec<texpatch_t> = Vec::with_capacity((*mtexture).patchcount.max(0) as usize);
        mpatch = (&raw mut (*mtexture).patches as *mut mappatch_t).offset(0 as i32 as isize)
            as *mut mappatch_t;
        j = 0 as i32;
        while j < (*mtexture).patchcount as i32 {
            let patch_entry = texpatch_t {
                originx: (*mpatch).originx,
                originy: (*mpatch).originy,
                patch: patchlookup[(*mpatch).patch as usize],
            };
            if patch_entry.patch == -(1 as i32) {
                I_Error(&format!(
                    "R_InitTextures: Missing patch in texture {}",
                    (*mtexture).name.as_str(),
                ));
            }
            patches.push(patch_entry);
            j += 1;
            mpatch = mpatch.offset(1);
        }
        state.r_data.textures.push(Box::new(texture_t {
            name: (*mtexture).name,
            width: (*mtexture).width,
            height: (*mtexture).height,
            index: 0,
            next: None,
            patchcount: (*mtexture).patchcount,
            patches,
        }));
        texture = state.r_data.textures[i as usize].as_mut() as *mut texture_t;
        state.r_data.texturecolumnlump.push(vec![0i16; (*texture).width as usize]);
        state.r_data.texturecolumnofs.push(vec![0u16; (*texture).width as usize]);
        j = 1 as i32;
        while j * 2 as i32 <= (*texture).width as i32 {
            j <<= 1 as i32;
        }
        state.r_data.texturewidthmask[i as usize] = j - 1 as i32;
        state.r_data.textureheight[i as usize] = (((*texture).height as i32) << FRACBITS) as fixed_t;
        i += 1;
        directory = directory.offset(1);
    }
    W_ReleaseLumpName(&mut state.w_wad, "TEXTURE1");
    if !maptex2.is_null() {
        W_ReleaseLumpName(&mut state.w_wad, "TEXTURE2");
    }
    i = 0 as i32;
    while i < state.r_data.numtextures {
        R_GenerateLookup(state, i);
        i += 1;
    }
    state.r_data.texturetranslation = vec![0 as i32; (state.r_data.numtextures + 1 as i32) as usize];
    i = 0 as i32;
    while i < state.r_data.numtextures {
        state.r_data.texturetranslation[i as usize] = i;
        i += 1;
    }
    GenerateTextureHashTable(state);
}
pub fn R_InitFlats(state: &mut GameState) {
    let mut i: i32 = 0;
    state.r_data.firstflat = W_GetNumForName(&mut state.w_wad, "F_START") + 1 as i32;
    state.r_data.lastflat = W_GetNumForName(&mut state.w_wad, "F_END") - 1 as i32;
    state.r_data.numflats = state.r_data.lastflat - state.r_data.firstflat + 1 as i32;
    state.r_data.flattranslation = vec![0 as i32; (state.r_data.numflats + 1 as i32) as usize];
    i = 0 as i32;
    while i < state.r_data.numflats {
        state.r_data.flattranslation[i as usize] = i;
        i += 1;
    }
}
pub unsafe fn R_InitSpriteLumps(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut patch: *mut patch_t = ::core::ptr::null_mut::<patch_t>();
    state.r_data.firstspritelump = W_GetNumForName(&mut state.w_wad, "S_START") + 1 as i32;
    state.r_data.lastspritelump = W_GetNumForName(&mut state.w_wad, "S_END") - 1 as i32;
    state.r_data.numspritelumps =
        state.r_data.lastspritelump - state.r_data.firstspritelump + 1 as i32;
    state.r_data.spritewidth = vec![0 as fixed_t; state.r_data.numspritelumps as usize];
    state.r_data.spriteoffset = vec![0 as fixed_t; state.r_data.numspritelumps as usize];
    state.r_data.spritetopoffset = vec![0 as fixed_t; state.r_data.numspritelumps as usize];
    i = 0 as i32;
    while i < state.r_data.numspritelumps {
        if i & 63 as i32 == 0 {
            print!(".");
        }
        patch = W_CacheLumpNum(state, state.r_data.firstspritelump + i) as *mut patch_t;
        state.r_data.spritewidth[i as usize] = (((*patch).width as i32) << FRACBITS) as fixed_t;
        state.r_data.spriteoffset[i as usize] =
            (((*patch).leftoffset as i32) << FRACBITS) as fixed_t;
        state.r_data.spritetopoffset[i as usize] =
            (((*patch).topoffset as i32) << FRACBITS) as fixed_t;
        i += 1;
    }
}
pub unsafe fn R_InitColormaps(state: &mut GameState) {
    let mut lump: i32 = 0;
    lump = W_GetNumForName(&mut state.w_wad, "COLORMAP");
    let lump_ptr = W_CacheLumpNum(state, lump) as *const lighttable_t;
    let lumplen = W_LumpLength(&mut state.w_wad, lump as u32) as usize;
    state.r_data.colormaps = ::core::slice::from_raw_parts(lump_ptr, lumplen).to_vec();
}
pub fn R_InitData(state: &mut GameState) {
    unsafe { R_InitTextures(state) };
    print!(".");
    R_InitFlats(state);
    print!(".");
    unsafe { R_InitSpriteLumps(state) };
    print!(".");
    unsafe { R_InitColormaps(state) };
}
pub fn R_FlatNumForName(state: &mut GameState, name: &str) -> i32 {
    let mut i: i32 = 0;
    i = W_CheckNumForName(&mut state.w_wad, name);
    if i == -(1 as i32) {
        I_Error(&format!("R_FlatNumForName: {} not found", name));
    }
    return i - state.r_data.firstflat;
}
pub unsafe fn R_CheckTextureNumForName(state: &mut RDataState, name: &str) -> i32 {
    let mut key: i32 = 0;
    if name.as_bytes().first() == Some(&b'-') {
        return 0 as i32;
    }
    key = W_LumpNameHash(name.as_bytes()).wrapping_rem(state.numtextures as u32) as i32;
    let mut cursor = state.textures_hashtable[key as usize];
    while let Some(id) = cursor {
        let texture = state.textures[id.0 as usize].as_mut() as *mut texture_t;
        if (*texture).name.eq_bytes_ignore_ascii_case(name.as_bytes()) {
            return (*texture).index;
        }
        cursor = (*texture).next;
    }
    return -(1 as i32);
}
pub fn R_TextureNumForName(state: &mut RDataState, name: &str) -> i32 {
    let mut i: i32 = 0;
    i = unsafe { R_CheckTextureNumForName(state, name) };
    if i == -(1 as i32) {
        I_Error(&format!("R_TextureNumForName: {} not found", name));
    }
    return i;
}
pub unsafe fn R_PrecacheLevel(state: &mut GameState) {
    let mut flatpresent: Vec<u8>;
    let mut texturepresent: Vec<u8>;
    let mut spritepresent: Vec<u8>;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut k: i32 = 0;
    let mut lump: i32 = 0;
    let mut texture: *mut texture_t = ::core::ptr::null_mut::<texture_t>();
    let mut th: *mut thinker_t = ::core::ptr::null_mut::<thinker_t>();
    let mut sf: *mut spriteframe_t = ::core::ptr::null_mut::<spriteframe_t>();
    if state.g_game.demoplayback {
        return;
    }
    flatpresent = vec![0u8; state.r_data.numflats as usize];
    i = 0 as i32;
    while i < state.p_setup.numsectors {
        flatpresent[state.p_setup.sectors[i as usize].floorpic as usize] = 1 as u8;
        flatpresent[state.p_setup.sectors[i as usize].ceilingpic as usize] = 1 as u8;
        i += 1;
    }
    state.r_data.flatmemory = 0 as i32;
    i = 0 as i32;
    while i < state.r_data.numflats {
        if flatpresent[i as usize] != 0 {
            lump = state.r_data.firstflat + i;
            state.r_data.flatmemory += state.w_wad.lumpinfo[lump as usize].size;
            W_CacheLumpNum(state, lump);
        }
        i += 1;
    }
    texturepresent = vec![0u8; state.r_data.numtextures as usize];
    i = 0 as i32;
    while i < state.p_setup.numsides {
        texturepresent[state.p_setup.sides[i as usize].toptexture as usize] = 1 as u8;
        texturepresent[state.p_setup.sides[i as usize].midtexture as usize] = 1 as u8;
        texturepresent[state.p_setup.sides[i as usize].bottomtexture as usize] = 1 as u8;
        i += 1;
    }
    texturepresent[state.r_sky.skytexture as usize] = 1 as u8;
    state.r_data.texturememory = 0 as i32;
    i = 0 as i32;
    while i < state.r_data.numtextures {
        if texturepresent[i as usize] != 0 {
            texture = state.r_data.textures[i as usize].as_mut() as *mut texture_t;
            j = 0 as i32;
            while j < (*texture).patchcount as i32 {
                lump = (*texture).patches[j as usize].patch;
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
        th = state.p_tick.raw(id);
        if matches!((*th).function, ThinkerFn::Mobj(_)) {
            spritepresent[(*(th as *mut mobj_t)).sprite as usize] = 1 as u8;
        }
        cursor = state.p_tick.next(id);
    }
    state.r_data.spritememory = 0 as i32;
    i = 0 as i32;
    while i < state.r_things.numsprites {
        if spritepresent[i as usize] != 0 {
            j = 0 as i32;
            while j < state.r_things.sprites[i as usize].numframes {
                sf = &raw mut state.r_things.sprites[i as usize].spriteframes[j as usize]
                    as *mut spriteframe_t;
                k = 0 as i32;
                while k < 8 as i32 {
                    lump = state.r_data.firstspritelump + (*sf).lump[k as usize] as i32;
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
