use crate::d_iwad::D_SuggestGameName;
use crate::d_mode::GameMode_t;
use crate::d_mode::D_GameMissionString;
use crate::d_mode::GameMission_t;
use crate::fixed_cstr::FixedCStr;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_misc::M_ExtractFileBase;
use crate::stdint_types::byte;
use crate::stdint_types::size_t;
use crate::w_file::wad_file_t;
use crate::w_file::W_OpenFile;
use crate::w_file::W_Read;

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
    pub wad_file: &'static wad_file_t,
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
pub fn W_AddFile(state: &mut GameState, filename: &str) -> Option<&'static wad_file_t> {
    let mut header: wadinfo_t = wadinfo_t {
        identification: FixedCStr([0; 4]),
        numlumps: 0,
        infotableofs: 0,
    };
    let mut length: i32 = 0;
    // Scratch WAD-directory buffer -- built and consumed entirely within
    // this function, so a plain owned Vec replaces the old
    // Z_Malloc-then-Z_Free-at-the-end pair with no lifetime change.
    let fileinfo: Vec<filelump_t>;
    let wad_file = match W_OpenFile(filename) {
        Some(wad_file) => wad_file,
        None => {
            println!(" couldn't open {}", filename);
            return None;
        }
    };
    let is_wad = filename.len() >= 3 && filename[filename.len() - 3..].eq_ignore_ascii_case("wad");
    if !is_wad {
        let mut single = filelump_t {
            filepos: 0 as i32,
            size: wad_file.length as i32,
            name: FixedCStr([0; 8]),
        };
        M_ExtractFileBase(filename, &mut single.name);
        fileinfo = vec![single];
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
    }
    state
        .w_wad
        .lumpinfo
        .extend(fileinfo.into_iter().map(|fi| lumpinfo_t {
            name: fi.name,
            wad_file,
            position: fi.filepos,
            size: fi.size,
            cache: None,
            next: None,
        }));
    state.w_wad.numlumps = state.w_wad.lumpinfo.len() as u32;
    state.w_wad.lumphash = Vec::new();
    Some(wad_file)
}
pub fn W_NumLumps(state: &mut WWadState) -> i32 {
    return state.numlumps as i32;
}
pub fn W_CheckNumForName(state: &mut WWadState, name: &str) -> i32 {
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
pub fn W_GetNumForName(state: &mut WWadState, name: &str) -> i32 {
    let mut i: i32 = 0;
    i = W_CheckNumForName(state, name);
    if i < 0 as i32 {
        I_Error(&format!("W_GetNumForName: {} not found!", name));
    }
    return i;
}
pub fn W_LumpLength(state: &mut WWadState, lump: u32) -> i32 {
    if lump >= state.numlumps {
        I_Error(&format!("W_LumpLength: {} >= numlumps", lump));
    }
    return state.lumpinfo[lump as usize].size;
}
pub fn W_ReadLump(state: &mut WWadState, lump: u32, dest: *mut ::core::ffi::c_void) {
    if lump >= state.numlumps {
        I_Error(&format!("W_ReadLump: {} >= numlumps", lump));
    }
    let l = &state.lumpinfo[lump as usize];
    let c = W_Read(l.wad_file, l.position as u32, dest, l.size as size_t) as i32;
    if c < l.size {
        I_Error(&format!(
            "W_ReadLump: only read {} of {} on lump {}",
            c, l.size, lump
        ));
    }
}
pub fn W_CacheLumpNum(state: &mut GameState, lumpnum: i32) -> *mut ::core::ffi::c_void {
    if lumpnum as u32 >= state.w_wad.numlumps {
        I_Error(&format!("W_CacheLumpNum: {} >= numlumps", lumpnum));
    }
    let lump = &mut state.w_wad.lumpinfo[lumpnum as usize];
    let result: *mut byte = if let Some(cache) = lump.cache.as_mut() {
        cache.as_mut_ptr()
    } else {
        let lumplen = W_LumpLength(&mut state.w_wad, lumpnum as u32);
        let mut buf = vec![0u8; lumplen as usize].into_boxed_slice();
        W_ReadLump(
            &mut state.w_wad,
            lumpnum as u32,
            buf.as_mut_ptr() as *mut ::core::ffi::c_void,
        );
        let lump = &mut state.w_wad.lumpinfo[lumpnum as usize];
        lump.cache = Some(buf);
        lump.cache.as_mut().unwrap().as_mut_ptr()
    };
    result as *mut ::core::ffi::c_void
}
pub fn W_CacheLumpName(state: &mut GameState, name: &str) -> *mut ::core::ffi::c_void {
    let lumpnum = W_GetNumForName(&mut state.w_wad, name);
    W_CacheLumpNum(state, lumpnum)
}
pub fn W_ReleaseLumpNum(state: &mut WWadState, lumpnum: i32) {
    // Releasing a cached lump is a no-op now -- nothing purges cached blocks
    // under memory pressure since the zone allocator was removed entirely;
    // the owned cache buffer just stays cached until process exit either
    // way. Kept as a bounds-checked no-op rather than deleted, matching this
    // function's original validation behavior.
    if lumpnum as u32 >= state.numlumps {
        I_Error(&format!("W_ReleaseLumpNum: {} >= numlumps", lumpnum));
    }
}
pub fn W_ReleaseLumpName(state: &mut WWadState, name: &str) {
    let lumpnum = W_GetNumForName(state, name);
    W_ReleaseLumpNum(state, lumpnum);
}
pub fn W_GenerateHashTable(state: &mut GameState) {
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
pub fn W_CheckCorrectIWAD(state: &mut WWadState, mission: GameMission_t) {
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
