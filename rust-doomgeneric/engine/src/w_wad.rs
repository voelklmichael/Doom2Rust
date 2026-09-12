use crate::src::d_iwad::D_SuggestGameName;
use crate::src::d_mode::indetermined;
use crate::src::d_mode::D_GameMissionString;
use crate::src::d_mode::{doom, heretic, hexen, strife, GameMission_t};
use crate::src::doomdef::NULL;
use crate::src::fixed_cstr::FixedCStr;
use crate::src::game_state::GameState;
use crate::src::i_system::I_Error;
use crate::src::m_misc::M_ExtractFileBase;
use crate::src::stdint_types::byte;
use crate::src::stdint_types::size_t;
use crate::src::w_file::wad_file_t;
use crate::src::w_file::W_OpenFile;
use crate::src::w_file::W_Read;
use crate::src::z_zone::Z_ChangeTag2;
use crate::src::z_zone::Z_ChangeUser;
use crate::src::z_zone::Z_Free;
use crate::src::z_zone::Z_Malloc;
use crate::src::z_zone::{PU_CACHE, PU_STATIC};
use crate::src::mem_compat::{memcpy, memset};
use std::alloc::{alloc_zeroed, dealloc, Layout};

pub struct WWadState {
    pub lumpinfo: *mut lumpinfo_t,
    pub numlumps: u32,
    pub lumphash: *mut *mut lumpinfo_t,
}

impl WWadState {
    pub const fn new() -> Self {
        WWadState {
            lumpinfo: ::core::ptr::null::<lumpinfo_t>() as *mut lumpinfo_t,
            numlumps: 0,
            lumphash: ::core::ptr::null::<*mut lumpinfo_t>() as *mut *mut lumpinfo_t,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct lumpinfo_s {
    pub name: FixedCStr<8>,
    pub wad_file: *mut wad_file_t,
    pub position: i32,
    pub size: i32,
    pub cache: *mut ::core::ffi::c_void,
    pub next: Option<u32>,
}
pub type lumpinfo_t = lumpinfo_s;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct filelump_t {
    pub filepos: i32,
    pub size: i32,
    pub name: FixedCStr<8>,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct wadinfo_t {
    pub identification: FixedCStr<4>,
    pub numlumps: i32,
    pub infotableofs: i32,
}
#[derive(Copy, Clone)]
pub struct C2RustUnnamed_0 {
    pub mission: GameMission_t,
    pub lumpname: &'static str,
}
pub const PROGRAM_PREFIX: FixedCStr<12> = FixedCStr(*b"doomgeneric\0");
pub fn W_LumpNameHash(s: &[u8]) -> u32 {
    let mut result: u32 = 5381 as u32;
    for &b in s.iter().take(8) {
        if b == 0 {
            break;
        }
        result = result << 5 as i32 ^ result ^ b.to_ascii_uppercase() as u32;
    }
    return result;
}
unsafe fn ExtendLumpInfo(state: &mut WWadState, mut newnumlumps: i32) {
    let mut newlumpinfo: *mut lumpinfo_t = ::core::ptr::null_mut::<lumpinfo_t>();
    let mut i: u32 = 0;
    let new_layout = match Layout::array::<lumpinfo_t>(newnumlumps as usize) {
        Ok(layout) => layout,
        Err(_) => I_Error("Couldn't realloc lumpinfo"),
    };
    newlumpinfo = alloc_zeroed(new_layout) as *mut lumpinfo_t;
    if newlumpinfo.is_null() {
        I_Error("Couldn't realloc lumpinfo");
    }
    i = 0 as u32;
    while i < state.numlumps && i < newnumlumps as u32 {
        memcpy(
            newlumpinfo.offset(i as isize) as *mut lumpinfo_t as *mut ::core::ffi::c_void,
            state.lumpinfo.offset(i as isize) as *mut lumpinfo_t
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<lumpinfo_t>() as size_t,
        );
        if !(*newlumpinfo.offset(i as isize)).cache.is_null() {
            Z_ChangeUser(
                (*newlumpinfo.offset(i as isize)).cache,
                &raw mut (*newlumpinfo.offset(i as isize)).cache,
            );
        }
        // `.next` is an index into this same array, not an address, so the
        // memcpy above already carried it over correctly -- no recompute
        // needed (unlike when it was a raw pointer into the old allocation).
        i = i.wrapping_add(1);
    }
    if !state.lumpinfo.is_null() {
        let old_layout = Layout::array::<lumpinfo_t>(state.numlumps as usize).unwrap();
        dealloc(state.lumpinfo as *mut u8, old_layout);
    }
    state.lumpinfo = newlumpinfo;
    state.numlumps = newnumlumps as u32;
}
pub unsafe fn W_AddFile(state: &mut GameState, filename: &str) -> *mut wad_file_t {
    let mut header: wadinfo_t = wadinfo_t {
        identification: FixedCStr([0; 4]),
        numlumps: 0,
        infotableofs: 0,
    };
    let mut lump_p: *mut lumpinfo_t = ::core::ptr::null_mut::<lumpinfo_t>();
    let mut i: u32 = 0;
    let mut wad_file: *mut wad_file_t = ::core::ptr::null_mut::<wad_file_t>();
    let mut length: i32 = 0;
    let mut startlump: i32 = 0;
    let mut fileinfo: *mut filelump_t = ::core::ptr::null_mut::<filelump_t>();
    let mut filerover: *mut filelump_t = ::core::ptr::null_mut::<filelump_t>();
    let mut newnumlumps: i32 = 0;
    wad_file = W_OpenFile(state, filename);
    if wad_file.is_null() {
        println!(" couldn't open {}", filename);
        return ::core::ptr::null_mut::<wad_file_t>();
    }
    newnumlumps = state.w_wad.numlumps as i32;
    let is_wad = filename.len() >= 3 && filename[filename.len() - 3..].eq_ignore_ascii_case("wad");
    if !is_wad {
        fileinfo = Z_Malloc(
            &mut state.z_zone,
            ::core::mem::size_of::<filelump_t>() as i32,
            PU_STATIC as i32,
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ) as *mut filelump_t;
        (*fileinfo).filepos = 0 as i32;
        (*fileinfo).size = (*wad_file).length as i32;
        M_ExtractFileBase(filename, &mut (*fileinfo).name);
        newnumlumps += 1;
    } else {
        W_Read(
            wad_file,
            0 as u32,
            &raw mut header as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<wadinfo_t>() as size_t,
        );
        if header.identification.0 != *b"IWAD" {
            if header.identification.0 != *b"PWAD" {
                I_Error(&format!(
                    "Wad file {} doesn't have IWAD or PWAD id\n",
                    filename,
                ));
            }
        }
        header.numlumps = header.numlumps;
        header.infotableofs = header.infotableofs;
        length = (header.numlumps as usize)
            .wrapping_mul(::core::mem::size_of::<filelump_t>() as usize) as i32;
        fileinfo = Z_Malloc(
            &mut state.z_zone,
            length,
            PU_STATIC as i32,
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ) as *mut filelump_t;
        W_Read(
            wad_file,
            header.infotableofs as u32,
            fileinfo as *mut ::core::ffi::c_void,
            length as size_t,
        );
        newnumlumps += header.numlumps;
    }
    startlump = state.w_wad.numlumps as i32;
    ExtendLumpInfo(&mut state.w_wad, newnumlumps);
    lump_p = state
        .w_wad
        .lumpinfo
        .offset(startlump as isize) as *mut lumpinfo_t;
    filerover = fileinfo;
    i = startlump as u32;
    while i < state.w_wad.numlumps {
        (*lump_p).wad_file = wad_file;
        (*lump_p).position = (*filerover).filepos;
        (*lump_p).size = (*filerover).size;
        (*lump_p).cache = NULL;
        (*lump_p).name = (*filerover).name;
        // ExtendLumpInfo zero-fills newly grown slots via alloc_zeroed, which
        // is only a valid `None` for a raw pointer field, not for an
        // `Option<u32>` (no guaranteed all-zero niche) -- so every freshly
        // extended record needs `.next` set explicitly here, before anything
        // can read it. W_GenerateHashTable overwrites it again once a real
        // chain is built.
        (*lump_p).next = None;
        lump_p = lump_p.offset(1);
        filerover = filerover.offset(1);
        i = i.wrapping_add(1);
    }
    Z_Free(
        &mut state.z_zone,
        fileinfo as *mut ::core::ffi::c_void,
    );
    if !state.w_wad.lumphash.is_null() {
        Z_Free(
            &mut state.z_zone,
            state.w_wad.lumphash as *mut ::core::ffi::c_void,
        );
        state.w_wad.lumphash = ::core::ptr::null_mut::<*mut lumpinfo_t>();
    }
    return wad_file;
}
pub unsafe fn W_NumLumps(state: &mut WWadState) -> i32 {
    return state.numlumps as i32;
}
/// Reads up to 8 bytes at `ptr` as a WAD lump name and converts it to an
/// owned `String`, stopping at the first nul (if any). WAD lump names are a
/// fixed 8-byte field with no guaranteed nul terminator, so unlike
/// `CStr::from_ptr` this never reads past the 8th byte; invalid UTF-8 is
/// lossily replaced rather than panicking, since arbitrary WAD/PWAD data is
/// not guaranteed to be valid UTF-8 (or even ASCII).
pub unsafe fn wad_name8_to_string(ptr: *const ::core::ffi::c_char) -> String {
    let bytes = ::core::slice::from_raw_parts(ptr as *const u8, 8);
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(8);
    String::from_utf8_lossy(&bytes[..len]).into_owned()
}
pub unsafe fn W_CheckNumForName(state: &mut WWadState, name: &str) -> i32 {
    let mut lump_p: *mut lumpinfo_t = ::core::ptr::null_mut::<lumpinfo_t>();
    let mut i: i32 = 0;
    if !state.lumphash.is_null() {
        let mut hash: i32 = 0;
        hash = W_LumpNameHash(name.as_bytes()).wrapping_rem(state.numlumps) as i32;
        lump_p = *state.lumphash.offset(hash as isize);
        while !lump_p.is_null() {
            if (*lump_p).name.eq_str_ignore_ascii_case(name) {
                return lump_p.offset_from(state.lumpinfo) as i64 as i32;
            }
            lump_p = match (*lump_p).next {
                Some(idx) => state.lumpinfo.offset(idx as isize),
                None => ::core::ptr::null_mut(),
            };
        }
    } else {
        i = state.numlumps.wrapping_sub(1 as u32) as i32;
        while i >= 0 as i32 {
            if (*state.lumpinfo.offset(i as isize))
                .name
                .eq_str_ignore_ascii_case(name)
            {
                return i;
            }
            i -= 1;
        }
    }
    return -(1 as i32);
}
pub unsafe fn W_GetNumForName(state: &mut WWadState, name: &str) -> i32 {
    let mut i: i32 = 0;
    i = W_CheckNumForName(state, name);
    if i < 0 as i32 {
        I_Error(&format!("W_GetNumForName: {} not found!", name));
    }
    return i;
}
pub unsafe fn W_LumpLength(state: &mut WWadState, mut lump: u32) -> i32 {
    if lump >= state.numlumps {
        I_Error(&format!("W_LumpLength: {} >= numlumps", lump));
    }
    return (*state.lumpinfo.offset(lump as isize)).size;
}
pub unsafe fn W_ReadLump(state: &mut WWadState, mut lump: u32, mut dest: *mut ::core::ffi::c_void) {
    let mut c: i32 = 0;
    let mut l: *mut lumpinfo_t = ::core::ptr::null_mut::<lumpinfo_t>();
    if lump >= state.numlumps {
        I_Error(&format!("W_ReadLump: {} >= numlumps", lump));
    }
    l = state.lumpinfo.offset(lump as isize);
    c = W_Read(
        (*l).wad_file,
        (*l).position as u32,
        dest,
        (*l).size as size_t,
    ) as i32;
    if c < (*l).size {
        I_Error(&format!(
            "W_ReadLump: only read {} of {} on lump {}",
            c,
            (*l).size,
            lump
        ));
    }
}
pub unsafe fn W_CacheLumpNum(
    state: &mut GameState,
    mut lumpnum: i32,
    mut tag: i32,
) -> *mut ::core::ffi::c_void {
    let mut result: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut lump: *mut lumpinfo_t = ::core::ptr::null_mut::<lumpinfo_t>();
    if lumpnum as u32 >= state.w_wad.numlumps {
        I_Error(&format!("W_CacheLumpNum: {} >= numlumps", lumpnum));
    }
    lump = state.w_wad.lumpinfo.offset(lumpnum as isize) as *mut lumpinfo_t;
    if !(*(*lump).wad_file).mapped.is_null() {
        result = (*(*lump).wad_file).mapped.offset((*lump).position as isize);
    } else if !(*lump).cache.is_null() {
        result = (*lump).cache as *mut byte;
        Z_ChangeTag2(
            (*lump).cache,
            tag,
            "w_wad.c",
            410 as i32,
        );
    } else {
        let lumplen = W_LumpLength(&mut state.w_wad, lumpnum as u32);
        (*lump).cache = Z_Malloc(
            &mut state.z_zone,
            lumplen,
            tag,
            &raw mut (*lump).cache as *mut ::core::ffi::c_void,
        );
        W_ReadLump(&mut state.w_wad, lumpnum as u32, (*lump).cache);
        result = (*lump).cache as *mut byte;
    }
    return result as *mut ::core::ffi::c_void;
}
pub unsafe fn W_CacheLumpName(
    state: &mut GameState,
    name: &str,
    mut tag: i32,
) -> *mut ::core::ffi::c_void {
    let lumpnum = W_GetNumForName(&mut state.w_wad, name);
    return W_CacheLumpNum(state, lumpnum, tag);
}
pub unsafe fn W_ReleaseLumpNum(state: &mut WWadState, mut lumpnum: i32) {
    let mut lump: *mut lumpinfo_t = ::core::ptr::null_mut::<lumpinfo_t>();
    if lumpnum as u32 >= state.numlumps {
        I_Error(&format!("W_ReleaseLumpNum: {} >= numlumps", lumpnum));
    }
    lump = state.lumpinfo.offset(lumpnum as isize) as *mut lumpinfo_t;
    if (*(*lump).wad_file).mapped.is_null() {
        Z_ChangeTag2(
            (*lump).cache,
            PU_CACHE as i32,
            "w_wad.c",
            461 as i32,
        );
    }
}
pub unsafe fn W_ReleaseLumpName(state: &mut WWadState, name: &str) {
    let lumpnum = W_GetNumForName(state, name);
    W_ReleaseLumpNum(state, lumpnum);
}
pub unsafe fn W_GenerateHashTable(state: &mut GameState) {
    let mut i: u32 = 0;
    if !state.w_wad.lumphash.is_null() {
        Z_Free(
            &mut state.z_zone,
            state.w_wad.lumphash as *mut ::core::ffi::c_void,
        );
    }
    if state.w_wad.numlumps > 0 as u32 {
        state.w_wad.lumphash = Z_Malloc(
            &mut state.z_zone,
            (::core::mem::size_of::<*mut lumpinfo_t>() as usize)
                .wrapping_mul(state.w_wad.numlumps as usize) as i32,
            PU_STATIC as i32,
            NULL,
        ) as *mut *mut lumpinfo_t;
        memset(
            state.w_wad.lumphash as *mut ::core::ffi::c_void,
            0 as i32,
            (::core::mem::size_of::<*mut lumpinfo_t>() as size_t)
                .wrapping_mul(state.w_wad.numlumps as size_t),
        );
        i = 0 as u32;
        while i < state.w_wad.numlumps {
            let mut hash: u32 = 0;
            hash = W_LumpNameHash(
                (*state.w_wad.lumpinfo.offset(i as isize))
                    .name
                    .as_bytes(),
            )
            .wrapping_rem(state.w_wad.numlumps);
            let old_head = *state.w_wad.lumphash.offset(hash as isize);
            (*state.w_wad.lumpinfo.offset(i as isize)).next = if old_head.is_null() {
                None
            } else {
                Some(old_head.offset_from(state.w_wad.lumpinfo) as i64 as u32)
            };
            let ref mut fresh2 = *state.w_wad.lumphash.offset(hash as isize);
            *fresh2 = state.w_wad.lumpinfo.offset(i as isize) as *mut lumpinfo_t;
            i = i.wrapping_add(1);
        }
    }
}
static unique_lumps: [C2RustUnnamed_0; 4] = [
    C2RustUnnamed_0 {
        mission: doom,
        lumpname: "POSSA1",
    },
    C2RustUnnamed_0 {
        mission: heretic,
        lumpname: "IMPXA1",
    },
    C2RustUnnamed_0 {
        mission: hexen,
        lumpname: "ETTNA1",
    },
    C2RustUnnamed_0 {
        mission: strife,
        lumpname: "AGRDA1",
    },
];
pub unsafe fn W_CheckCorrectIWAD(state: &mut WWadState, mut mission: GameMission_t) {
    let mut i: i32 = 0;
    let mut lumpnum: i32 = 0;
    i = 0 as i32;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_0; 4]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_0>() as usize)
    {
        if mission as u32 != unique_lumps[i as usize].mission as u32 {
            lumpnum = W_CheckNumForName(state, unique_lumps[i as usize].lumpname);
            if lumpnum >= 0 as i32 {
                I_Error(&format!(
                    "\nYou are trying to use a {} IWAD file with the {}{} binary.\nThis isn't going to work.\nYou probably want to use the {}{} binary.",
                    D_SuggestGameName(unique_lumps[i as usize].mission, indetermined),
                    PROGRAM_PREFIX.as_str(),
                    D_GameMissionString(mission),
                    PROGRAM_PREFIX.as_str(),
                    D_GameMissionString(unique_lumps[i as usize].mission),
                ));
            }
        }
        i += 1;
    }
}
