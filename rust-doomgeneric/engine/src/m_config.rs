use crate::filesystem::DoomFileSystem;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::m_argv::check_parm_with_args;
use crate::m_argv::MArgvState;
use crate::platform::DoomPlatform;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DefaultType {
    Int = 0,
    IntHex = 1,
    String = 2,
    Float = 3,
    Key = 4,
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

impl Default for MConfigState {
    fn default() -> Self {
        Self::new()
    }
}

impl MConfigState {
    // Kept out of line: `GameState::new` inlines every state constructor, and once
    // `init_game_state` passes 256 KB the Xtensa linker fails ("dangerous relocation:
    // l32r: literal target out of range") building the firmware. These are the biggest.
    #[inline(never)]
    pub fn new() -> Self {
        let doom_defaults_list = vec![
            ConfigVariable {
                name: "mouse_sensitivity",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "sfx_volume",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "music_volume",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "show_talk",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "voice_volume",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "show_messages",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "key_right",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_left",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_up",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_down",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_strafeleft",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_straferight",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_useHealth",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_jump",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_flyup",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_flydown",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_flycenter",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_lookup",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_lookdown",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_lookcenter",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_invquery",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_mission",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_invPop",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_invKey",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_invHome",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_invEnd",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_invleft",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_invright",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_invLeft",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_invRight",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_useartifact",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_invUse",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_invDrop",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_lookUp",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_lookDown",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_fire",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_use",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_strafe",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_speed",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "use_mouse",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "mouseb_fire",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "mouseb_strafe",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "mouseb_forward",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "mouseb_jump",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "use_joystick",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joyb_fire",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joyb_strafe",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joyb_use",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joyb_speed",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joyb_jump",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "screenblocks",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "screensize",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "detaillevel",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "snd_channels",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "snd_musicdevice",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "snd_sfxdevice",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "snd_sbport",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "snd_sbirq",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "snd_sbdma",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "snd_mport",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "usegamma",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "savedir",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "messageson",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "back_flat",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "nickname",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "chatmacro0",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "chatmacro1",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "chatmacro2",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "chatmacro3",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "chatmacro4",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "chatmacro5",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "chatmacro6",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "chatmacro7",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "chatmacro8",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "chatmacro9",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "comport",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
        ];
        let extra_defaults_list = vec![
            ConfigVariable {
                name: "graphical_startup",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "autoadjust_video_settings",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "fullscreen",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "aspect_ratio_correct",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "startup_delay",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "screen_width",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "screen_height",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "screen_bpp",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "grabmouse",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "novert",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "mouse_acceleration",
                location: None,
                kind: DefaultType::Float,
                bound: false,
            },
            ConfigVariable {
                name: "mouse_threshold",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "snd_samplerate",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "snd_cachesize",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "snd_maxslicetime_ms",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "snd_musiccmd",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "opl_io_port",
                location: None,
                kind: DefaultType::IntHex,
                bound: false,
            },
            ConfigVariable {
                name: "show_endoom",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "png_screenshots",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "vanilla_savegame_limit",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "vanilla_demo_limit",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "vanilla_keyboard_mapping",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "video_driver",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "window_position",
                location: None,
                kind: DefaultType::String,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_index",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_x_axis",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_x_invert",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_y_axis",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_y_invert",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_strafe_axis",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_strafe_invert",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_physical_button0",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_physical_button1",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_physical_button2",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_physical_button3",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_physical_button4",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_physical_button5",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_physical_button6",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_physical_button7",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_physical_button8",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joystick_physical_button9",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joyb_strafeleft",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joyb_straferight",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joyb_menu_activate",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joyb_prevweapon",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "joyb_nextweapon",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "mouseb_strafeleft",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "mouseb_straferight",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "mouseb_use",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "mouseb_backward",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "mouseb_prevweapon",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "mouseb_nextweapon",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "dclick_use",
                location: None,
                kind: DefaultType::Int,
                bound: false,
            },
            ConfigVariable {
                name: "key_pause",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_activate",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_up",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_down",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_left",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_right",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_back",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_forward",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_confirm",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_abort",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_help",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_save",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_load",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_volume",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_detail",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_qsave",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_endgame",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_messages",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_qload",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_quit",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_gamma",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_spy",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_incscreen",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_decscreen",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_menu_screenshot",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_map_toggle",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_map_north",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_map_south",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_map_east",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_map_west",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_map_zoomin",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_map_zoomout",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_map_maxzoom",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_map_follow",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_map_grid",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_map_mark",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_map_clearmark",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_weapon1",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_weapon2",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_weapon3",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_weapon4",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_weapon5",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_weapon6",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_weapon7",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_weapon8",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_prevweapon",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_nextweapon",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_arti_all",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_arti_health",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_arti_poisonbag",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_arti_blastradius",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_arti_teleport",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_arti_teleportother",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_arti_egg",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_arti_invulnerability",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_message_refresh",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_demo_quit",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_multi_msg",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_multi_msgplayer1",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_multi_msgplayer2",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_multi_msgplayer3",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_multi_msgplayer4",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_multi_msgplayer5",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_multi_msgplayer6",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_multi_msgplayer7",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
            ConfigVariable {
                name: "key_multi_msgplayer8",
                location: None,
                kind: DefaultType::Key,
                bound: false,
            },
        ];
        Self {
            configdir: String::new(),
            default_main_config: "",
            default_extra_config: "",
            doom_defaults: ConfigVariableCollection {
                defaults: doom_defaults_list,
                filename: String::new(),
            },
            extra_defaults: ConfigVariableCollection {
                defaults: extra_defaults_list,
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
    m_argv: &MArgvState,
    m_config: &mut MConfigState,
    platform: &mut dyn DoomPlatform,
) {
    if let Some(i) = check_parm_with_args(m_argv, "-config", 1) {
        m_config.doom_defaults.filename = m_argv.myargv[i + 1].as_str().to_string();
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
    if let Some(i) = check_parm_with_args(m_argv, "-extraconfig", 1) {
        m_config.extra_defaults.filename = m_argv.myargv[i + 1].as_str().to_string();
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
    let savegamedir;
    if state.configdir.is_empty() {
        savegamedir = String::new();
    } else {
        savegamedir = format!("{}{}.savegame/", state.configdir, DIR_SEPARATOR_S);
        fs.create_dir(&savegamedir);
        doom_println!(platform, "Using {} for savegames", savegamedir);
    }
    savegamedir
}
