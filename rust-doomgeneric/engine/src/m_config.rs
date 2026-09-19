use crate::filesystem::DoomFileSystem;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_argv::M_CheckParmWithArgs;
use crate::platform::DoomPlatform;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DefaultType {
    DEFAULT_INT = 0,
    DEFAULT_INT_HEX = 1,
    DEFAULT_STRING = 2,
    DEFAULT_FLOAT = 3,
    DEFAULT_KEY = 4,
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
pub struct default_t {
    pub name: &'static str,
    pub location: Option<DefaultLocation>,
    pub kind: DefaultType,
    pub bound: bool,
}
pub struct default_collection_t {
    pub defaults: Vec<default_t>,
    pub filename: String,
}
pub const DIR_SEPARATOR_S: &str = "/";
pub struct MConfigState {
    configdir: String,
    default_main_config: &'static str,
    default_extra_config: &'static str,
    doom_defaults: default_collection_t,
    extra_defaults: default_collection_t,
}

impl Default for MConfigState {
    fn default() -> Self {
        Self::new()
    }
}

impl MConfigState {
    pub fn new() -> Self {
        let doom_defaults_list = vec![
            default_t {
                name: "mouse_sensitivity",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "sfx_volume",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "music_volume",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "show_talk",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "voice_volume",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "show_messages",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "key_right",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_left",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_up",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_down",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_strafeleft",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_straferight",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_useHealth",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_jump",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_flyup",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_flydown",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_flycenter",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_lookup",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_lookdown",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_lookcenter",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_invquery",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_mission",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_invPop",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_invKey",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_invHome",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_invEnd",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_invleft",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_invright",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_invLeft",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_invRight",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_useartifact",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_invUse",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_invDrop",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_lookUp",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_lookDown",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_fire",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_use",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_strafe",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_speed",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "use_mouse",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "mouseb_fire",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "mouseb_strafe",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "mouseb_forward",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "mouseb_jump",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "use_joystick",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joyb_fire",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joyb_strafe",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joyb_use",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joyb_speed",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joyb_jump",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "screenblocks",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "screensize",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "detaillevel",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "snd_channels",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "snd_musicdevice",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "snd_sfxdevice",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "snd_sbport",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "snd_sbirq",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "snd_sbdma",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "snd_mport",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "usegamma",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "savedir",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "messageson",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "back_flat",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "nickname",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "chatmacro0",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "chatmacro1",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "chatmacro2",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "chatmacro3",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "chatmacro4",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "chatmacro5",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "chatmacro6",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "chatmacro7",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "chatmacro8",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "chatmacro9",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "comport",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
        ];
        let extra_defaults_list = vec![
            default_t {
                name: "graphical_startup",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "autoadjust_video_settings",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "fullscreen",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "aspect_ratio_correct",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "startup_delay",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "screen_width",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "screen_height",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "screen_bpp",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "grabmouse",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "novert",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "mouse_acceleration",
                location: None,
                kind: DefaultType::DEFAULT_FLOAT,
                bound: false,
            },
            default_t {
                name: "mouse_threshold",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "snd_samplerate",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "snd_cachesize",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "snd_maxslicetime_ms",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "snd_musiccmd",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "opl_io_port",
                location: None,
                kind: DefaultType::DEFAULT_INT_HEX,
                bound: false,
            },
            default_t {
                name: "show_endoom",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "png_screenshots",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "vanilla_savegame_limit",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "vanilla_demo_limit",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "vanilla_keyboard_mapping",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "video_driver",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "window_position",
                location: None,
                kind: DefaultType::DEFAULT_STRING,
                bound: false,
            },
            default_t {
                name: "joystick_index",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_x_axis",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_x_invert",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_y_axis",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_y_invert",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_strafe_axis",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_strafe_invert",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button0",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button1",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button2",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button3",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button4",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button5",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button6",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button7",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button8",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button9",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joyb_strafeleft",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joyb_straferight",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joyb_menu_activate",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joyb_prevweapon",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "joyb_nextweapon",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "mouseb_strafeleft",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "mouseb_straferight",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "mouseb_use",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "mouseb_backward",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "mouseb_prevweapon",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "mouseb_nextweapon",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "dclick_use",
                location: None,
                kind: DefaultType::DEFAULT_INT,
                bound: false,
            },
            default_t {
                name: "key_pause",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_activate",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_up",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_down",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_left",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_right",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_back",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_forward",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_confirm",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_abort",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_help",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_save",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_load",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_volume",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_detail",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_qsave",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_endgame",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_messages",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_qload",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_quit",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_gamma",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_spy",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_incscreen",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_decscreen",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_menu_screenshot",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_map_toggle",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_map_north",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_map_south",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_map_east",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_map_west",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_map_zoomin",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_map_zoomout",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_map_maxzoom",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_map_follow",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_map_grid",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_map_mark",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_map_clearmark",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_weapon1",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_weapon2",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_weapon3",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_weapon4",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_weapon5",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_weapon6",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_weapon7",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_weapon8",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_prevweapon",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_nextweapon",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_arti_all",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_arti_health",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_arti_poisonbag",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_arti_blastradius",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_arti_teleport",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_arti_teleportother",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_arti_egg",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_arti_invulnerability",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_message_refresh",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_demo_quit",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_multi_msg",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer1",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer2",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer3",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer4",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer5",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer6",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer7",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer8",
                location: None,
                kind: DefaultType::DEFAULT_KEY,
                bound: false,
            },
        ];
        MConfigState {
            configdir: String::new(),
            default_main_config: "",
            default_extra_config: "",
            doom_defaults: default_collection_t {
                defaults: doom_defaults_list,
                filename: String::new(),
            },
            extra_defaults: default_collection_t {
                defaults: extra_defaults_list,
                filename: String::new(),
            },
        }
    }
}

fn SearchCollection<'a>(
    collection: &'a mut default_collection_t,
    name: &str,
) -> Option<&'a mut default_t> {
    collection
        .defaults
        .iter_mut()
        .find(|entry| entry.name == name)
}
pub fn M_SetConfigFilenames(
    state: &mut MConfigState,
    main_config: &'static str,
    extra_config: &'static str,
) {
    state.default_main_config = main_config;
    state.default_extra_config = extra_config;
}
pub fn M_SaveDefaults(_state: &mut GameState) {}
pub fn M_LoadDefaults(state: &mut GameState) {
    let mut i: i32;
    i = M_CheckParmWithArgs(state, "-config", 1);
    if i != 0 {
        state.m_config.doom_defaults.filename =
            state.m_argv.myargv[(i + 1) as usize].as_str().to_string();
        doom_println!(
            state.platform,
            "\tdefault file: {}",
            state.m_config.doom_defaults.filename,
        );
    } else {
        state.m_config.doom_defaults.filename = format!(
            "{}{}",
            state.m_config.configdir, state.m_config.default_main_config
        );
    }
    doom_println!(
        state.platform,
        "saving config in {}",
        state.m_config.doom_defaults.filename
    );
    i = M_CheckParmWithArgs(state, "-extraconfig", 1);
    if i != 0 {
        state.m_config.extra_defaults.filename =
            state.m_argv.myargv[(i + 1) as usize].as_str().to_string();
        doom_println!(
            state.platform,
            "        extra configuration file: {}",
            state.m_config.extra_defaults.filename,
        );
    } else {
        state.m_config.extra_defaults.filename = format!(
            "{}{}",
            state.m_config.configdir, state.m_config.default_extra_config
        );
    }
}
fn GetDefaultForName<'a>(state: &'a mut MConfigState, name: &str) -> &'a mut default_t {
    let mut result = SearchCollection(&mut state.doom_defaults, name);
    if result.is_none() {
        result = SearchCollection(&mut state.extra_defaults, name);
    }
    if let Some(result) = result {
        result
    } else {
        I_Error(&format!("Unknown configuration variable: '{}'", name));
    }
}
pub fn M_BindVariable_int(
    state: &mut MConfigState,
    name: &str,
    location: impl Fn(&mut GameState) -> &mut i32 + 'static,
) {
    let variable = GetDefaultForName(state, name);
    match variable.kind {
        DefaultType::DEFAULT_INT | DefaultType::DEFAULT_INT_HEX | DefaultType::DEFAULT_KEY => {}
        _ => I_Error(&format!(
            "M_BindVariable_int: '{}' is not an int/key variable",
            name
        )),
    }
    variable.location = Some(DefaultLocation::Int(Rc::new(location)));
    variable.bound = true;
}
pub fn M_BindVariable_string(
    state: &mut MConfigState,
    name: &str,
    location: impl Fn(&mut GameState) -> &mut Option<&'static str> + 'static,
) {
    let variable = GetDefaultForName(state, name);
    if variable.kind != DefaultType::DEFAULT_STRING {
        I_Error(&format!(
            "M_BindVariable_string: '{}' is not a string variable",
            name
        ));
    }
    variable.location = Some(DefaultLocation::Str(Rc::new(location)));
    variable.bound = true;
}
fn GetDefaultConfigDir() -> String {
    ".".to_string()
}
pub fn M_SetConfigDir(
    state: &mut MConfigState,
    fs: &mut dyn DoomFileSystem,
    platform: &mut dyn DoomPlatform,
    dir: Option<&str>,
) {
    if let Some(dir) = dir {
        state.configdir = dir.to_string();
    } else {
        state.configdir = GetDefaultConfigDir();
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
pub fn M_GetSaveGameDir(
    state: &mut MConfigState,
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
