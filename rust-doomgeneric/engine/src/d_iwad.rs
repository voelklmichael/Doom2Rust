use crate::d_mode::GameMission;
use crate::d_mode::GameMode;
use crate::filesystem::DoomFileSystem;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::platform::DoomPlatform;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
#[derive(Copy, Clone)]
pub struct Iwad {
    pub name: &'static str,
    pub mission: GameMission,
    pub mode: GameMode,
    pub description: &'static str,
}
pub const FILES_DIR: &str = ".";
pub const DIR_SEPARATOR: char = '/';
pub const DIR_SEPARATOR_S: &str = "/";
static IWADS: [Iwad; 14] = [
    Iwad {
        name: "doom2.wad",
        mission: GameMission::Doom2,
        mode: GameMode::Commercial,
        description: "Doom II",
    },
    Iwad {
        name: "plutonia.wad",
        mission: GameMission::PackPlut,
        mode: GameMode::Commercial,
        description: "Final Doom: Plutonia Experiment",
    },
    Iwad {
        name: "tnt.wad",
        mission: GameMission::PackTnt,
        mode: GameMode::Commercial,
        description: "Final Doom: TNT: Evilution",
    },
    Iwad {
        name: "doom.wad",
        mission: GameMission::Doom,
        mode: GameMode::Retail,
        description: "Doom",
    },
    Iwad {
        name: "doom1.wad",
        mission: GameMission::Doom,
        mode: GameMode::Shareware,
        description: "Doom Shareware",
    },
    Iwad {
        name: "chex.wad",
        mission: GameMission::PackChex,
        mode: GameMode::Shareware,
        description: "Chex Quest",
    },
    Iwad {
        name: "hacx.wad",
        mission: GameMission::PackHacx,
        mode: GameMode::Commercial,
        description: "Hacx",
    },
    Iwad {
        name: "freedm.wad",
        mission: GameMission::Doom2,
        mode: GameMode::Commercial,
        description: "FreeDM",
    },
    Iwad {
        name: "freedoom2.wad",
        mission: GameMission::Doom2,
        mode: GameMode::Commercial,
        description: "Freedoom: Phase 2",
    },
    Iwad {
        name: "freedoom1.wad",
        mission: GameMission::Doom,
        mode: GameMode::Retail,
        description: "Freedoom: Phase 1",
    },
    Iwad {
        name: "heretic.wad",
        mission: GameMission::Heretic,
        mode: GameMode::Retail,
        description: "Heretic",
    },
    Iwad {
        name: "heretic1.wad",
        mission: GameMission::Heretic,
        mode: GameMode::Shareware,
        description: "Heretic Shareware",
    },
    Iwad {
        name: "hexen.wad",
        mission: GameMission::Hexen,
        mode: GameMode::Commercial,
        description: "Hexen",
    },
    Iwad {
        name: "strife1.wad",
        mission: GameMission::Strife,
        mode: GameMode::Commercial,
        description: "Strife",
    },
];

pub struct DIwadState {
    iwad_dirs_built: bool,
    iwad_dirs: Vec<String>,
}

impl Default for DIwadState {
    fn default() -> Self {
        Self::new()
    }
}

impl DIwadState {
    pub const fn new() -> Self {
        Self {
            iwad_dirs_built: false,
            iwad_dirs: Vec::new(),
        }
    }
}

fn add_iwad_dir(state: &mut DIwadState, dir: &str) {
    state.iwad_dirs.push(dir.to_string());
}
fn dir_is_file(path: &str, filename: &str) -> bool {
    path.len() > filename.len()
        && path.as_bytes()[path.len() - filename.len() - 1] == DIR_SEPARATOR as u8
        && path[path.len() - filename.len()..].eq_ignore_ascii_case(filename)
}
fn check_directory_has_iwad(
    fs: &dyn DoomFileSystem,
    platform: &mut dyn DoomPlatform,
    dir: &str,
    iwadname: &str,
) -> Option<String> {
    if dir_is_file(dir, iwadname) && fs.exists(dir) {
        return Some(dir.to_string());
    }
    let filename = if dir == "." {
        iwadname.to_string()
    } else {
        format!("{dir}{DIR_SEPARATOR_S}{iwadname}")
    };
    doom_println!(platform, "Trying IWAD file:{}", filename);
    if fs.exists(&filename) {
        Some(filename)
    } else {
        None
    }
}
fn search_directory_for_iwad(
    fs: &dyn DoomFileSystem,
    platform: &mut dyn DoomPlatform,
    dir: &str,
    mask: i32,
    mission: &mut GameMission,
) -> Option<String> {
    for iwad in &IWADS {
        if 1 << iwad.mission as i32 & mask == 0 {
            continue;
        }
        if let Some(filename) = check_directory_has_iwad(fs, platform, dir, iwad.name) {
            *mission = iwad.mission;
            return Some(filename);
        }
    }
    None
}
fn identify_iwad_by_name(name: &str, mask: i32) -> GameMission {
    let name = match name.rfind(DIR_SEPARATOR) {
        Some(pos) => &name[pos + 1..],
        None => name,
    };
    for iwad in &IWADS {
        if 1 << iwad.mission as i32 & mask == 0 {
            continue;
        }
        if name.eq_ignore_ascii_case(iwad.name) {
            return iwad.mission;
        }
    }
    GameMission::None
}
fn build_iwad_dir_list(state: &mut DIwadState) {
    add_iwad_dir(state, FILES_DIR);
    state.iwad_dirs_built = true;
}
pub fn find_wadby_name(
    state: &mut DIwadState,
    fs: &dyn DoomFileSystem,
    name: &str,
) -> Option<String> {
    if fs.exists(name) {
        return Some(name.to_string());
    }
    build_iwad_dir_list(state);
    for dir in &state.iwad_dirs {
        if dir_is_file(dir, name) && fs.exists(dir) {
            return Some(dir.clone());
        }
        let path = format!("{dir}{DIR_SEPARATOR_S}{name}");
        if fs.exists(&path) {
            return Some(path);
        }
    }
    None
}
pub fn try_find_wadby_name(
    state: &mut DIwadState,
    fs: &dyn DoomFileSystem,
    filename: &str,
) -> String {
    find_wadby_name(state, fs, filename).unwrap_or_else(|| filename.to_string())
}
pub fn find_iwad(state: &mut GameState, mask: i32, mission: &mut GameMission) -> String {
    if let Some(iwadfile) = state.game.options.iwad.clone() {
        let result = find_wadby_name(&mut state.game.d_iwad, &*state.assets.fs, &iwadfile);
        let Some(result) = result else {
            error(&format!("IWAD file '{iwadfile}' not found!"));
        };
        *mission = identify_iwad_by_name(&result, mask);
        result
    } else {
        doom_println!(
            state.io.platform,
            "-iwad not specified, trying a few iwad names"
        );
        build_iwad_dir_list(&mut state.game.d_iwad);
        for dir in &state.game.d_iwad.iwad_dirs {
            if let Some(found) = search_directory_for_iwad(
                &*state.assets.fs,
                &mut *state.io.platform,
                dir,
                mask,
                mission,
            ) {
                return found;
            }
        }
        String::new()
    }
}
pub fn save_game_iwadname(gamemission: GameMission) -> &'static str {
    for iwad in &IWADS {
        if gamemission == iwad.mission {
            return iwad.name;
        }
    }
    "unknown.wad"
}
pub fn suggest_game_name(mission: GameMission, mode: GameMode) -> &'static str {
    for iwad in &IWADS {
        if iwad.mission == mission && (mode == GameMode::Indetermined || iwad.mode == mode) {
            return iwad.description;
        }
    }
    "Unknown game?"
}
