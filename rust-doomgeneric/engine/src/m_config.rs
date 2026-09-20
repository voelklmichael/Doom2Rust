use crate::filesystem::DoomFileSystem;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::options::Options;
use crate::platform::DoomPlatform;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DefaultType {
    Int,
    IntHex,
    String,
    Float,
    Key,
}
/// Where a bound configuration variable lives: an accessor that projects the
/// variable out of the game state.
type StrAccessor = Rc<dyn Fn(&mut GameState) -> &mut Option<&'static str>>;
// The bound accessors are stored but nothing loads or saves the defaults yet.
#[allow(dead_code)]
#[derive(Clone)]
pub enum DefaultLocation {
    Int(Rc<dyn Fn(&mut GameState) -> &mut i32>),
    Str(StrAccessor),
}
pub struct ConfigVariable {
    pub name: &'static str,
    pub location: Option<DefaultLocation>,
    pub kind: DefaultType,
    pub bound: bool,
}
pub struct ConfigVariableCollection {
    pub defaults: Vec<ConfigVariable>,
    pub filename: String,
}
pub const DIR_SEPARATOR_S: &str = "/";
pub struct MConfigState {
    configdir: String,
    default_main_config: &'static str,
    default_extra_config: &'static str,
    doom_defaults: ConfigVariableCollection,
    extra_defaults: ConfigVariableCollection,
}

/// The variables of the main configuration file, with their types.
const DOOM_DEFAULTS: [(&str, DefaultType); 76] = [
    ("mouse_sensitivity", DefaultType::Int),
    ("sfx_volume", DefaultType::Int),
    ("music_volume", DefaultType::Int),
    ("show_talk", DefaultType::Int),
    ("voice_volume", DefaultType::Int),
    ("show_messages", DefaultType::Int),
    ("key_right", DefaultType::Key),
    ("key_left", DefaultType::Key),
    ("key_up", DefaultType::Key),
    ("key_down", DefaultType::Key),
    ("key_strafeleft", DefaultType::Key),
    ("key_straferight", DefaultType::Key),
    ("key_useHealth", DefaultType::Key),
    ("key_jump", DefaultType::Key),
    ("key_flyup", DefaultType::Key),
    ("key_flydown", DefaultType::Key),
    ("key_flycenter", DefaultType::Key),
    ("key_lookup", DefaultType::Key),
    ("key_lookdown", DefaultType::Key),
    ("key_lookcenter", DefaultType::Key),
    ("key_invquery", DefaultType::Key),
    ("key_mission", DefaultType::Key),
    ("key_invPop", DefaultType::Key),
    ("key_invKey", DefaultType::Key),
    ("key_invHome", DefaultType::Key),
    ("key_invEnd", DefaultType::Key),
    ("key_invleft", DefaultType::Key),
    ("key_invright", DefaultType::Key),
    ("key_invLeft", DefaultType::Key),
    ("key_invRight", DefaultType::Key),
    ("key_useartifact", DefaultType::Key),
    ("key_invUse", DefaultType::Key),
    ("key_invDrop", DefaultType::Key),
    ("key_lookUp", DefaultType::Key),
    ("key_lookDown", DefaultType::Key),
    ("key_fire", DefaultType::Key),
    ("key_use", DefaultType::Key),
    ("key_strafe", DefaultType::Key),
    ("key_speed", DefaultType::Key),
    ("use_mouse", DefaultType::Int),
    ("mouseb_fire", DefaultType::Int),
    ("mouseb_strafe", DefaultType::Int),
    ("mouseb_forward", DefaultType::Int),
    ("mouseb_jump", DefaultType::Int),
    ("use_joystick", DefaultType::Int),
    ("joyb_fire", DefaultType::Int),
    ("joyb_strafe", DefaultType::Int),
    ("joyb_use", DefaultType::Int),
    ("joyb_speed", DefaultType::Int),
    ("joyb_jump", DefaultType::Int),
    ("screenblocks", DefaultType::Int),
    ("screensize", DefaultType::Int),
    ("detaillevel", DefaultType::Int),
    ("snd_channels", DefaultType::Int),
    ("snd_musicdevice", DefaultType::Int),
    ("snd_sfxdevice", DefaultType::Int),
    ("snd_sbport", DefaultType::Int),
    ("snd_sbirq", DefaultType::Int),
    ("snd_sbdma", DefaultType::Int),
    ("snd_mport", DefaultType::Int),
    ("usegamma", DefaultType::Int),
    ("savedir", DefaultType::String),
    ("messageson", DefaultType::Int),
    ("back_flat", DefaultType::String),
    ("nickname", DefaultType::String),
    ("chatmacro0", DefaultType::String),
    ("chatmacro1", DefaultType::String),
    ("chatmacro2", DefaultType::String),
    ("chatmacro3", DefaultType::String),
    ("chatmacro4", DefaultType::String),
    ("chatmacro5", DefaultType::String),
    ("chatmacro6", DefaultType::String),
    ("chatmacro7", DefaultType::String),
    ("chatmacro8", DefaultType::String),
    ("chatmacro9", DefaultType::String),
    ("comport", DefaultType::Int),
];

/// The variables of the extra (game-specific) configuration file.
const EXTRA_DEFAULTS: [(&str, DefaultType); 119] = [
    ("graphical_startup", DefaultType::Int),
    ("autoadjust_video_settings", DefaultType::Int),
    ("fullscreen", DefaultType::Int),
    ("aspect_ratio_correct", DefaultType::Int),
    ("startup_delay", DefaultType::Int),
    ("screen_width", DefaultType::Int),
    ("screen_height", DefaultType::Int),
    ("screen_bpp", DefaultType::Int),
    ("grabmouse", DefaultType::Int),
    ("novert", DefaultType::Int),
    ("mouse_acceleration", DefaultType::Float),
    ("mouse_threshold", DefaultType::Int),
    ("snd_samplerate", DefaultType::Int),
    ("snd_cachesize", DefaultType::Int),
    ("snd_maxslicetime_ms", DefaultType::Int),
    ("snd_musiccmd", DefaultType::String),
    ("opl_io_port", DefaultType::IntHex),
    ("show_endoom", DefaultType::Int),
    ("png_screenshots", DefaultType::Int),
    ("vanilla_savegame_limit", DefaultType::Int),
    ("vanilla_demo_limit", DefaultType::Int),
    ("vanilla_keyboard_mapping", DefaultType::Int),
    ("video_driver", DefaultType::String),
    ("window_position", DefaultType::String),
    ("joystick_index", DefaultType::Int),
    ("joystick_x_axis", DefaultType::Int),
    ("joystick_x_invert", DefaultType::Int),
    ("joystick_y_axis", DefaultType::Int),
    ("joystick_y_invert", DefaultType::Int),
    ("joystick_strafe_axis", DefaultType::Int),
    ("joystick_strafe_invert", DefaultType::Int),
    ("joystick_physical_button0", DefaultType::Int),
    ("joystick_physical_button1", DefaultType::Int),
    ("joystick_physical_button2", DefaultType::Int),
    ("joystick_physical_button3", DefaultType::Int),
    ("joystick_physical_button4", DefaultType::Int),
    ("joystick_physical_button5", DefaultType::Int),
    ("joystick_physical_button6", DefaultType::Int),
    ("joystick_physical_button7", DefaultType::Int),
    ("joystick_physical_button8", DefaultType::Int),
    ("joystick_physical_button9", DefaultType::Int),
    ("joyb_strafeleft", DefaultType::Int),
    ("joyb_straferight", DefaultType::Int),
    ("joyb_menu_activate", DefaultType::Int),
    ("joyb_prevweapon", DefaultType::Int),
    ("joyb_nextweapon", DefaultType::Int),
    ("mouseb_strafeleft", DefaultType::Int),
    ("mouseb_straferight", DefaultType::Int),
    ("mouseb_use", DefaultType::Int),
    ("mouseb_backward", DefaultType::Int),
    ("mouseb_prevweapon", DefaultType::Int),
    ("mouseb_nextweapon", DefaultType::Int),
    ("dclick_use", DefaultType::Int),
    ("key_pause", DefaultType::Key),
    ("key_menu_activate", DefaultType::Key),
    ("key_menu_up", DefaultType::Key),
    ("key_menu_down", DefaultType::Key),
    ("key_menu_left", DefaultType::Key),
    ("key_menu_right", DefaultType::Key),
    ("key_menu_back", DefaultType::Key),
    ("key_menu_forward", DefaultType::Key),
    ("key_menu_confirm", DefaultType::Key),
    ("key_menu_abort", DefaultType::Key),
    ("key_menu_help", DefaultType::Key),
    ("key_menu_save", DefaultType::Key),
    ("key_menu_load", DefaultType::Key),
    ("key_menu_volume", DefaultType::Key),
    ("key_menu_detail", DefaultType::Key),
    ("key_menu_qsave", DefaultType::Key),
    ("key_menu_endgame", DefaultType::Key),
    ("key_menu_messages", DefaultType::Key),
    ("key_menu_qload", DefaultType::Key),
    ("key_menu_quit", DefaultType::Key),
    ("key_menu_gamma", DefaultType::Key),
    ("key_spy", DefaultType::Key),
    ("key_menu_incscreen", DefaultType::Key),
    ("key_menu_decscreen", DefaultType::Key),
    ("key_menu_screenshot", DefaultType::Key),
    ("key_map_toggle", DefaultType::Key),
    ("key_map_north", DefaultType::Key),
    ("key_map_south", DefaultType::Key),
    ("key_map_east", DefaultType::Key),
    ("key_map_west", DefaultType::Key),
    ("key_map_zoomin", DefaultType::Key),
    ("key_map_zoomout", DefaultType::Key),
    ("key_map_maxzoom", DefaultType::Key),
    ("key_map_follow", DefaultType::Key),
    ("key_map_grid", DefaultType::Key),
    ("key_map_mark", DefaultType::Key),
    ("key_map_clearmark", DefaultType::Key),
    ("key_weapon1", DefaultType::Key),
    ("key_weapon2", DefaultType::Key),
    ("key_weapon3", DefaultType::Key),
    ("key_weapon4", DefaultType::Key),
    ("key_weapon5", DefaultType::Key),
    ("key_weapon6", DefaultType::Key),
    ("key_weapon7", DefaultType::Key),
    ("key_weapon8", DefaultType::Key),
    ("key_prevweapon", DefaultType::Key),
    ("key_nextweapon", DefaultType::Key),
    ("key_arti_all", DefaultType::Key),
    ("key_arti_health", DefaultType::Key),
    ("key_arti_poisonbag", DefaultType::Key),
    ("key_arti_blastradius", DefaultType::Key),
    ("key_arti_teleport", DefaultType::Key),
    ("key_arti_teleportother", DefaultType::Key),
    ("key_arti_egg", DefaultType::Key),
    ("key_arti_invulnerability", DefaultType::Key),
    ("key_message_refresh", DefaultType::Key),
    ("key_demo_quit", DefaultType::Key),
    ("key_multi_msg", DefaultType::Key),
    ("key_multi_msgplayer1", DefaultType::Key),
    ("key_multi_msgplayer2", DefaultType::Key),
    ("key_multi_msgplayer3", DefaultType::Key),
    ("key_multi_msgplayer4", DefaultType::Key),
    ("key_multi_msgplayer5", DefaultType::Key),
    ("key_multi_msgplayer6", DefaultType::Key),
    ("key_multi_msgplayer7", DefaultType::Key),
    ("key_multi_msgplayer8", DefaultType::Key),
];

/// A configuration variable that is not (yet) bound to a place in the game state.
fn unbound_variables(table: &[(&'static str, DefaultType)]) -> Vec<ConfigVariable> {
    table
        .iter()
        .map(|&(name, kind)| ConfigVariable {
            name,
            location: None,
            kind,
            bound: false,
        })
        .collect()
}

impl Default for MConfigState {
    // Kept out of line: `GameState::new` inlines every state constructor, and once
    // `init_game_state` passes 256 KB the Xtensa linker fails ("dangerous relocation:
    // l32r: literal target out of range") building the firmware. (They were the biggest
    // constructors before their tables became compact rows; not re-measured on the firmware.)
    #[inline(never)]
    fn default() -> Self {
        Self {
            configdir: String::new(),
            default_main_config: "",
            default_extra_config: "",
            doom_defaults: ConfigVariableCollection {
                defaults: unbound_variables(&DOOM_DEFAULTS),
                filename: String::new(),
            },
            extra_defaults: ConfigVariableCollection {
                defaults: unbound_variables(&EXTRA_DEFAULTS),
                filename: String::new(),
            },
        }
    }
}

fn search_collection<'a>(
    collection: &'a mut ConfigVariableCollection,
    name: &str,
) -> Option<&'a mut ConfigVariable> {
    collection
        .defaults
        .iter_mut()
        .find(|entry| entry.name == name)
}
pub fn set_config_filenames(
    state: &mut MConfigState,
    main_config: &'static str,
    extra_config: &'static str,
) {
    state.default_main_config = main_config;
    state.default_extra_config = extra_config;
}
pub fn save_defaults(_state: &mut GameState) {}
pub fn load_defaults(
    options: &Options,
    m_config: &mut MConfigState,
    platform: &mut dyn DoomPlatform,
) {
    if let Some(file) = &options.config {
        m_config.doom_defaults.filename = file.clone();
        doom_println!(
            platform,
            "\tdefault file: {}",
            m_config.doom_defaults.filename,
        );
    } else {
        m_config.doom_defaults.filename =
            format!("{}{}", m_config.configdir, m_config.default_main_config);
    }
    doom_println!(
        platform,
        "saving config in {}",
        m_config.doom_defaults.filename
    );
    if let Some(file) = &options.extraconfig {
        m_config.extra_defaults.filename = file.clone();
        doom_println!(
            platform,
            "        extra configuration file: {}",
            m_config.extra_defaults.filename,
        );
    } else {
        m_config.extra_defaults.filename =
            format!("{}{}", m_config.configdir, m_config.default_extra_config);
    }
}
fn get_default_for_name<'a>(state: &'a mut MConfigState, name: &str) -> &'a mut ConfigVariable {
    let mut result = search_collection(&mut state.doom_defaults, name);
    if result.is_none() {
        result = search_collection(&mut state.extra_defaults, name);
    }
    if let Some(result) = result {
        result
    } else {
        error(&format!("Unknown configuration variable: '{name}'"));
    }
}
pub fn bind_variable_int(
    state: &mut MConfigState,
    name: &str,
    location: impl Fn(&mut GameState) -> &mut i32 + 'static,
) {
    let variable = get_default_for_name(state, name);
    match variable.kind {
        DefaultType::Int | DefaultType::IntHex | DefaultType::Key => {}
        _ => error(&format!(
            "M_BindVariable_int: '{name}' is not an int/key variable"
        )),
    }
    variable.location = Some(DefaultLocation::Int(Rc::new(location)));
    variable.bound = true;
}
pub fn bind_variable_string(
    state: &mut MConfigState,
    name: &str,
    location: impl Fn(&mut GameState) -> &mut Option<&'static str> + 'static,
) {
    let variable = get_default_for_name(state, name);
    if variable.kind != DefaultType::String {
        error(&format!(
            "M_BindVariable_string: '{name}' is not a string variable"
        ));
    }
    variable.location = Some(DefaultLocation::Str(Rc::new(location)));
    variable.bound = true;
}
fn get_default_config_dir() -> String {
    ".".to_string()
}
pub fn set_config_dir(
    state: &mut MConfigState,
    fs: &mut dyn DoomFileSystem,
    platform: &mut dyn DoomPlatform,
    dir: Option<&str>,
) {
    if let Some(dir) = dir {
        state.configdir = dir.to_string();
    } else {
        state.configdir = get_default_config_dir();
    }
    if !state.configdir.is_empty() {
        doom_println!(
            platform,
            "Using {} for configuration and saves",
            state.configdir
        );
    }
    fs.create_dir(&state.configdir);
}
pub fn get_save_game_dir(
    state: &MConfigState,
    fs: &mut dyn DoomFileSystem,
    platform: &mut dyn DoomPlatform,
    _iwadname: &'static str,
) -> String {
    if state.configdir.is_empty() {
        String::new()
    } else {
        let savegamedir = format!("{}{}.savegame/", state.configdir, DIR_SEPARATOR_S);
        fs.create_dir(&savegamedir);
        doom_println!(platform, "Using {} for savegames", savegamedir);
        savegamedir
    }
}
