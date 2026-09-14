use crate::d_mode::GameMode_t;
use crate::d_mode::GameMission_t;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_argv::M_CheckParmWithArgs;
use crate::m_misc::M_FileExists;
#[derive(Copy, Clone)]
pub struct iwad_t {
    pub name: &'static str,
    pub mission: GameMission_t,
    pub mode: GameMode_t,
    pub description: &'static str,
}
pub const FILES_DIR: &str = ".";
pub const DIR_SEPARATOR: char = '/';
pub const DIR_SEPARATOR_S: &str = "/";
static iwads: [iwad_t; 14] = [
    iwad_t {
        name: "doom2.wad",
        mission: GameMission_t::doom2,
        mode: GameMode_t::commercial,
        description: "Doom II",
    },
    iwad_t {
        name: "plutonia.wad",
        mission: GameMission_t::pack_plut,
        mode: GameMode_t::commercial,
        description: "Final Doom: Plutonia Experiment",
    },
    iwad_t {
        name: "tnt.wad",
        mission: GameMission_t::pack_tnt,
        mode: GameMode_t::commercial,
        description: "Final Doom: TNT: Evilution",
    },
    iwad_t {
        name: "doom.wad",
        mission: GameMission_t::doom,
        mode: GameMode_t::retail,
        description: "Doom",
    },
    iwad_t {
        name: "doom1.wad",
        mission: GameMission_t::doom,
        mode: GameMode_t::shareware,
        description: "Doom Shareware",
    },
    iwad_t {
        name: "chex.wad",
        mission: GameMission_t::pack_chex,
        mode: GameMode_t::shareware,
        description: "Chex Quest",
    },
    iwad_t {
        name: "hacx.wad",
        mission: GameMission_t::pack_hacx,
        mode: GameMode_t::commercial,
        description: "Hacx",
    },
    iwad_t {
        name: "freedm.wad",
        mission: GameMission_t::doom2,
        mode: GameMode_t::commercial,
        description: "FreeDM",
    },
    iwad_t {
        name: "freedoom2.wad",
        mission: GameMission_t::doom2,
        mode: GameMode_t::commercial,
        description: "Freedoom: Phase 2",
    },
    iwad_t {
        name: "freedoom1.wad",
        mission: GameMission_t::doom,
        mode: GameMode_t::retail,
        description: "Freedoom: Phase 1",
    },
    iwad_t {
        name: "heretic.wad",
        mission: GameMission_t::heretic,
        mode: GameMode_t::retail,
        description: "Heretic",
    },
    iwad_t {
        name: "heretic1.wad",
        mission: GameMission_t::heretic,
        mode: GameMode_t::shareware,
        description: "Heretic Shareware",
    },
    iwad_t {
        name: "hexen.wad",
        mission: GameMission_t::hexen,
        mode: GameMode_t::commercial,
        description: "Hexen",
    },
    iwad_t {
        name: "strife1.wad",
        mission: GameMission_t::strife,
        mode: GameMode_t::commercial,
        description: "Strife",
    },
];
pub const MAX_IWAD_DIRS: i32 = 128;

pub struct DIwadState {
    iwad_dirs_built: bool,
    iwad_dirs: Vec<String>,
}

impl DIwadState {
    pub const fn new() -> Self {
        DIwadState {
            iwad_dirs_built: false,
            iwad_dirs: Vec::new(),
        }
    }
}

fn file_exists(path: &str) -> bool {
    M_FileExists(path)
}
fn add_iwad_dir(state: &mut DIwadState, dir: &str) {
    state.iwad_dirs.push(dir.to_string());
}
fn dir_is_file(path: &str, filename: &str) -> bool {
    path.len() >= filename.len() + 1
        && path.as_bytes()[path.len() - filename.len() - 1] == DIR_SEPARATOR as u8
        && path[path.len() - filename.len()..].eq_ignore_ascii_case(filename)
}
fn check_directory_has_iwad(dir: &str, iwadname: &str) -> Option<String> {
    if dir_is_file(dir, iwadname) && file_exists(dir) {
        return Some(dir.to_string());
    }
    let filename = if dir == "." {
        iwadname.to_string()
    } else {
        format!("{}{}{}", dir, DIR_SEPARATOR_S, iwadname)
    };
    println!("Trying IWAD file:{}", filename);
    if file_exists(&filename) {
        Some(filename)
    } else {
        None
    }
}
fn search_directory_for_iwad(
    dir: &str,
    mask: i32,
    mission: *mut GameMission_t,
) -> Option<String> {
    for iwad in iwads.iter() {
        if (1 as i32) << iwad.mission as i32 & mask == 0 as i32 {
            continue;
        }
        if let Some(filename) = check_directory_has_iwad(dir, iwad.name) {
            unsafe {
                *mission = iwad.mission;
            }
            return Some(filename);
        }
    }
    None
}
fn identify_iwad_by_name(name: &str, mask: i32) -> GameMission_t {
    let name = match name.rfind(DIR_SEPARATOR) {
        Some(pos) => &name[pos + 1..],
        None => name,
    };
    for iwad in iwads.iter() {
        if (1 as i32) << iwad.mission as i32 & mask == 0 as i32 {
            continue;
        }
        if name.eq_ignore_ascii_case(iwad.name) {
            return iwad.mission;
        }
    }
    GameMission_t::none
}
fn build_iwad_dir_list(state: &mut DIwadState) {
    add_iwad_dir(state, FILES_DIR);
    state.iwad_dirs_built = true;
}
pub fn D_FindWADByName(state: &mut DIwadState, name: &str) -> Option<String> {
    if file_exists(name) {
        return Some(name.to_string());
    }
    build_iwad_dir_list(state);
    for dir in state.iwad_dirs.iter() {
        if dir_is_file(dir, name) && file_exists(dir) {
            return Some(dir.clone());
        }
        let path = format!("{}{}{}", dir, DIR_SEPARATOR_S, name);
        if file_exists(&path) {
            return Some(path);
        }
    }
    None
}
pub fn D_TryFindWADByName(state: &mut DIwadState, filename: &str) -> String {
    D_FindWADByName(state, filename).unwrap_or_else(|| filename.to_string())
}
pub fn D_FindIWAD(state: &mut GameState, mask: i32, mission: *mut GameMission_t) -> String {
    let iwadparm = M_CheckParmWithArgs(state, "-iwad", 1 as i32);
    if iwadparm != 0 {
        let iwadfile = state.m_argv.myargv[(iwadparm + 1 as i32) as usize]
            .to_str()
            .unwrap()
            .to_string();
        let result = D_FindWADByName(&mut state.d_iwad, &iwadfile);
        let Some(result) = result else {
            I_Error(&format!("IWAD file '{}' not found!", iwadfile));
        };
        unsafe {
            *mission = identify_iwad_by_name(&result, mask);
        }
        result
    } else {
        println!("-iwad not specified, trying a few iwad names");
        build_iwad_dir_list(&mut state.d_iwad);
        for dir in state.d_iwad.iwad_dirs.iter() {
            if let Some(found) = search_directory_for_iwad(dir, mask, mission) {
                return found;
            }
        }
        String::new()
    }
}
pub unsafe fn D_FindAllIWADs(state: &mut DIwadState, mut mask: i32) -> *mut *const iwad_t {
    let mut result: Vec<*const iwad_t> = Vec::new();
    for iwad in iwads.iter() {
        if (1 as i32) << iwad.mission as i32 & mask == 0 as i32 {
            continue;
        }
        if D_FindWADByName(state, iwad.name).is_some() {
            result.push(iwad as *const iwad_t);
        }
    }
    result.push(::core::ptr::null());
    Box::leak(result.into_boxed_slice()).as_mut_ptr()
}
pub fn D_SaveGameIWADName(gamemission: GameMission_t) -> &'static str {
    for iwad in iwads.iter() {
        if gamemission == iwad.mission {
            return iwad.name;
        }
    }
    "unknown.wad"
}
pub fn D_SuggestIWADName(
    mut mission: GameMission_t,
    mut mode: GameMode_t,
) -> *mut ::core::ffi::c_char {
    for iwad in iwads.iter() {
        if iwad.mission == mission && iwad.mode == mode {
            return ::std::ffi::CString::new(iwad.name).unwrap().into_raw();
        }
    }
    ::std::ffi::CString::new("unknown.wad").unwrap().into_raw()
}
pub fn D_SuggestGameName(mission: GameMission_t, mode: GameMode_t) -> &'static str {
    for iwad in iwads.iter() {
        if iwad.mission == mission && (mode == GameMode_t::indetermined || iwad.mode == mode) {
            return iwad.description;
        }
    }
    "Unknown game?"
}
