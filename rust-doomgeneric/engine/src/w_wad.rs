use crate::src::d_iwad::D_SuggestGameName;
use crate::src::d_mode::GameMode_t;
use crate::src::d_mode::D_GameMissionString;
use crate::src::d_mode::GameMission_t;
use crate::src::fixed_cstr::FixedCStr;
use crate::src::game_state::GameState;
use crate::src::i_system::I_Error;
use crate::src::m_misc::M_ExtractFileBase;
use crate::src::stdint_types::byte;
use crate::src::stdint_types::size_t;
use crate::src::w_file::wad_file_t;
use crate::src::w_file::W_OpenFile;
use crate::src::w_file::W_Read;

pub struct WWadState {
    pub lumpinfo: Vec<lumpinfo_t>,
    pub numlumps: u32,
    pub lumphash: Vec<Option<u32>>,
}

impl WWadState {
    pub const fn new() -> Self {
        WWadState {
            lumpinfo: Vec::new(),
            numlumps: 0,
            lumphash: Vec::new(),
        }
    }
}

#[derive(Clone)]
#[repr(C)]
pub struct lumpinfo_s {
    pub name: FixedCStr<8>,
    pub wad_file: *mut wad_file_t,
    pub position: i32,
    pub size: i32,
    pub cache: Option<Box<[u8]>>,
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
    // `cache` is now an owned `Box<[u8]>` (a separate heap allocation, not a
    // zone block whose back-pointer needs fixing up), and `.next` is an
    // index into this same array rather than an address -- so moving each
    // kept entry into the new Vec carries both fields over correctly with
    // no further fixup, unlike when `cache` was a raw zone pointer.
    let keep = state.numlumps.min(newnumlumps as u32) as usize;
    let mut old_lumpinfo = ::core::mem::take(&mut state.lumpinfo).into_iter();
    let mut new_lumpinfo: Vec<lumpinfo_t> = Vec::with_capacity(newnumlumps as usize);
    for _ in 0..keep {
        new_lumpinfo.push(old_lumpinfo.next().unwrap());
    }
    while new_lumpinfo.len() < newnumlumps as usize {
        new_lumpinfo.push(lumpinfo_t {
            name: FixedCStr([0; 8]),
            wad_file: ::core::ptr::null_mut(),
            position: 0,
            size: 0,
            cache: None,
            next: None,
        });
    }
    state.lumpinfo = new_lumpinfo;
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
    // Scratch WAD-directory buffer -- built and consumed entirely within
    // this function, so a plain owned Vec replaces the old
    // Z_Malloc-then-Z_Free-at-the-end pair with no lifetime change.
    let fileinfo: Vec<filelump_t>;
    let mut newnumlumps: i32 = 0;
    wad_file = W_OpenFile(state, filename);
    if wad_file.is_null() {
        println!(" couldn't open {}", filename);
        return ::core::ptr::null_mut::<wad_file_t>();
    }
    newnumlumps = state.w_wad.numlumps as i32;
    let is_wad = filename.len() >= 3 && filename[filename.len() - 3..].eq_ignore_ascii_case("wad");
    if !is_wad {
        let mut single = filelump_t {
            filepos: 0 as i32,
            size: (*wad_file).length as i32,
            name: FixedCStr([0; 8]),
        };
        M_ExtractFileBase(filename, &mut single.name);
        fileinfo = vec![single];
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
        let mut buf = vec![
            filelump_t {
                filepos: 0,
                size: 0,
                name: FixedCStr([0; 8]),
            };
            header.numlumps as usize
        ];
        W_Read(
            wad_file,
            header.infotableofs as u32,
            buf.as_mut_ptr() as *mut ::core::ffi::c_void,
            length as size_t,
        );
        fileinfo = buf;
        newnumlumps += header.numlumps;
    }
    startlump = state.w_wad.numlumps as i32;
    ExtendLumpInfo(&mut state.w_wad, newnumlumps);
    lump_p = state.w_wad.lumpinfo.as_mut_ptr().offset(startlump as isize);
    i = startlump as u32;
    let mut fi: usize = 0;
    while i < state.w_wad.numlumps {
        (*lump_p).wad_file = wad_file;
        (*lump_p).position = fileinfo[fi].filepos;
        (*lump_p).size = fileinfo[fi].size;
        (*lump_p).cache = None;
        (*lump_p).name = fileinfo[fi].name;
        // ExtendLumpInfo already initializes every freshly grown slot's
        // `.next` to None, but W_GenerateHashTable overwrites it again once a
        // real chain is built, so no need to touch it here.
        lump_p = lump_p.offset(1);
        fi += 1;
        i = i.wrapping_add(1);
    }
    state.w_wad.lumphash = Vec::new();
    return wad_file;
}
pub fn W_NumLumps(state: &mut WWadState) -> i32 {
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
    let mut i: i32 = 0;
    if !state.lumphash.is_empty() {
        let mut hash: u32 = 0;
        hash = W_LumpNameHash(name.as_bytes()).wrapping_rem(state.numlumps);
        let mut cur = state.lumphash[hash as usize];
        while let Some(idx) = cur {
            if state.lumpinfo[idx as usize].name.eq_str_ignore_ascii_case(name) {
                return idx as i32;
            }
            cur = state.lumpinfo[idx as usize].next;
        }
    } else {
        i = state.numlumps.wrapping_sub(1 as u32) as i32;
        while i >= 0 as i32 {
            if state.lumpinfo[i as usize]
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
    return state.lumpinfo[lump as usize].size;
}
pub unsafe fn W_ReadLump(state: &mut WWadState, mut lump: u32, mut dest: *mut ::core::ffi::c_void) {
    let mut c: i32 = 0;
    let mut l: *mut lumpinfo_t = ::core::ptr::null_mut::<lumpinfo_t>();
    if lump >= state.numlumps {
        I_Error(&format!("W_ReadLump: {} >= numlumps", lump));
    }
    l = state.lumpinfo.as_mut_ptr().offset(lump as isize);
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
    _tag: i32,
) -> *mut ::core::ffi::c_void {
    let mut result: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut lump: *mut lumpinfo_t = ::core::ptr::null_mut::<lumpinfo_t>();
    if lumpnum as u32 >= state.w_wad.numlumps {
        I_Error(&format!("W_CacheLumpNum: {} >= numlumps", lumpnum));
    }
    lump = state.w_wad.lumpinfo.as_mut_ptr().offset(lumpnum as isize);
    if !(*(*lump).wad_file).mapped.is_null() {
        result = (*(*lump).wad_file).mapped.offset((*lump).position as isize);
    } else if let Some(cache) = (*lump).cache.as_mut() {
        result = cache.as_mut_ptr();
    } else {
        let lumplen = W_LumpLength(&mut state.w_wad, lumpnum as u32);
        let mut buf = vec![0u8; lumplen as usize].into_boxed_slice();
        W_ReadLump(
            &mut state.w_wad,
            lumpnum as u32,
            buf.as_mut_ptr() as *mut ::core::ffi::c_void,
        );
        // `lump` was computed before this call; W_ReadLump only takes
        // `&mut WWadState` and never touches `lumpinfo`'s length, so the
        // Vec's backing store can't have moved underneath this pointer.
        (*lump).cache = Some(buf);
        result = (*lump).cache.as_mut().unwrap().as_mut_ptr();
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
    // Demoting a cached lump's tag back to PU_CACHE is inert now -- nothing
    // purges cached blocks under memory pressure since the zone allocator
    // moved to std::alloc (see docs/known-deviations.md); the owned cache
    // buffer just stays cached until process exit either way. Kept as a
    // bounds-checked no-op rather than deleted, matching this function's
    // original validation behavior.
    if lumpnum as u32 >= state.numlumps {
        I_Error(&format!("W_ReleaseLumpNum: {} >= numlumps", lumpnum));
    }
}
pub unsafe fn W_ReleaseLumpName(state: &mut WWadState, name: &str) {
    let lumpnum = W_GetNumForName(state, name);
    W_ReleaseLumpNum(state, lumpnum);
}
pub unsafe fn W_GenerateHashTable(state: &mut GameState) {
    let mut i: u32 = 0;
    state.w_wad.lumphash = Vec::new();
    if state.w_wad.numlumps > 0 as u32 {
        state.w_wad.lumphash = vec![None; state.w_wad.numlumps as usize];
        i = 0 as u32;
        while i < state.w_wad.numlumps {
            let mut hash: u32 = 0;
            hash = W_LumpNameHash(state.w_wad.lumpinfo[i as usize].name.as_bytes())
                .wrapping_rem(state.w_wad.numlumps);
            let old_head = state.w_wad.lumphash[hash as usize];
            state.w_wad.lumpinfo[i as usize].next = old_head;
            state.w_wad.lumphash[hash as usize] = Some(i);
            i = i.wrapping_add(1);
        }
    }
}
static unique_lumps: [C2RustUnnamed_0; 4] = [
    C2RustUnnamed_0 {
        mission: GameMission_t::doom,
        lumpname: "POSSA1",
    },
    C2RustUnnamed_0 {
        mission: GameMission_t::heretic,
        lumpname: "IMPXA1",
    },
    C2RustUnnamed_0 {
        mission: GameMission_t::hexen,
        lumpname: "ETTNA1",
    },
    C2RustUnnamed_0 {
        mission: GameMission_t::strife,
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
                    D_SuggestGameName(unique_lumps[i as usize].mission, GameMode_t::indetermined),
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
