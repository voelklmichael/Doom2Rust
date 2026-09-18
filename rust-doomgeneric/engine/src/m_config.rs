use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_argv::M_CheckParmWithArgs;
use crate::m_controls::KEY_BACKSPACE;
use crate::m_controls::KEY_CAPSLOCK;
use crate::m_controls::KEY_DEL;
use crate::m_controls::KEY_DOWNARROW;
use crate::m_controls::KEY_END;
use crate::m_controls::KEY_F1;
use crate::m_controls::KEY_F10;
use crate::m_controls::KEY_F11;
use crate::m_controls::KEY_F12;
use crate::m_controls::KEY_F2;
use crate::m_controls::KEY_F3;
use crate::m_controls::KEY_F4;
use crate::m_controls::KEY_F5;
use crate::m_controls::KEY_F6;
use crate::m_controls::KEY_F7;
use crate::m_controls::KEY_F8;
use crate::m_controls::KEY_F9;
use crate::m_controls::KEY_HOME;
use crate::m_controls::KEY_INS;
use crate::m_controls::KEY_LEFTARROW;
use crate::m_controls::KEY_MINUS;
use crate::m_controls::KEY_PAUSE;
use crate::m_controls::KEY_PGDN;
use crate::m_controls::KEY_PGUP;
use crate::m_controls::KEY_RALT;
use crate::m_controls::KEY_RIGHTARROW;
use crate::m_controls::KEY_RSHIFT;
use crate::m_controls::KEY_SCRLCK;
use crate::m_controls::KEY_UPARROW;
use crate::m_misc::M_MakeDirectory;
use crate::m_misc::M_StrToInt;
use std::rc::Rc;

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
#[derive(Clone)]
pub enum DefaultLocation {
    Int(Rc<dyn Fn(&mut GameState) -> &mut i32>),
    Float(Rc<dyn Fn(&mut GameState) -> &mut f32>),
    Str(StrAccessor),
}
pub struct default_t {
    pub name: &'static str,
    pub location: Option<DefaultLocation>,
    pub type_0: DefaultType,
    pub untranslated: i32,
    pub original_translated: i32,
    pub bound: bool,
}
pub struct default_collection_t {
    pub defaults: Vec<default_t>,
    pub numdefaults: i32,
    pub filename: String,
}
pub const DIR_SEPARATOR_S: &str = "/";
pub const KEY_RCTRL: i32 = 0x80 + 0x1d_i32;
pub const KEY_PRTSCR: i32 = 0x80 + 0x59_i32;
pub const KEYP_5: i32 = '5' as i32;
pub const KEYP_PLUS: i32 = '+' as i32;
pub const KEYP_MULTIPLY: i32 = '*' as i32;
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
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "sfx_volume",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "music_volume",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "show_talk",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "voice_volume",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "show_messages",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_right",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_left",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_up",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_down",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_strafeleft",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_straferight",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_useHealth",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_jump",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_flyup",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_flydown",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_flycenter",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_lookup",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_lookdown",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_lookcenter",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_invquery",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_mission",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_invPop",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_invKey",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_invHome",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_invEnd",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_invleft",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_invright",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_invLeft",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_invRight",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_useartifact",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_invUse",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_invDrop",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_lookUp",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_lookDown",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_fire",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_use",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_strafe",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_speed",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "use_mouse",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "mouseb_fire",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "mouseb_strafe",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "mouseb_forward",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "mouseb_jump",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "use_joystick",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joyb_fire",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joyb_strafe",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joyb_use",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joyb_speed",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joyb_jump",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "screenblocks",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "screensize",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "detaillevel",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "snd_channels",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "snd_musicdevice",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "snd_sfxdevice",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "snd_sbport",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "snd_sbirq",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "snd_sbdma",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "snd_mport",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "usegamma",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "savedir",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "messageson",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "back_flat",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "nickname",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "chatmacro0",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "chatmacro1",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "chatmacro2",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "chatmacro3",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "chatmacro4",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "chatmacro5",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "chatmacro6",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "chatmacro7",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "chatmacro8",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "chatmacro9",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "comport",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
        ];
        let doom_defaults_len = doom_defaults_list.len() as i32;
        let extra_defaults_list = vec![
            default_t {
                name: "graphical_startup",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "autoadjust_video_settings",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "fullscreen",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "aspect_ratio_correct",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "startup_delay",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "screen_width",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "screen_height",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "screen_bpp",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "grabmouse",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "novert",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "mouse_acceleration",
                location: None,
                type_0: DefaultType::DEFAULT_FLOAT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "mouse_threshold",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "snd_samplerate",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "snd_cachesize",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "snd_maxslicetime_ms",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "snd_musiccmd",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "opl_io_port",
                location: None,
                type_0: DefaultType::DEFAULT_INT_HEX,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "show_endoom",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "png_screenshots",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "vanilla_savegame_limit",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "vanilla_demo_limit",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "vanilla_keyboard_mapping",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "video_driver",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "window_position",
                location: None,
                type_0: DefaultType::DEFAULT_STRING,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_index",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_x_axis",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_x_invert",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_y_axis",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_y_invert",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_strafe_axis",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_strafe_invert",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button0",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button1",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button2",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button3",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button4",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button5",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button6",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button7",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button8",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joystick_physical_button9",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joyb_strafeleft",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joyb_straferight",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joyb_menu_activate",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joyb_prevweapon",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "joyb_nextweapon",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "mouseb_strafeleft",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "mouseb_straferight",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "mouseb_use",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "mouseb_backward",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "mouseb_prevweapon",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "mouseb_nextweapon",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "dclick_use",
                location: None,
                type_0: DefaultType::DEFAULT_INT,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_pause",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_activate",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_up",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_down",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_left",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_right",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_back",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_forward",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_confirm",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_abort",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_help",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_save",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_load",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_volume",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_detail",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_qsave",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_endgame",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_messages",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_qload",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_quit",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_gamma",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_spy",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_incscreen",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_decscreen",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_menu_screenshot",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_map_toggle",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_map_north",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_map_south",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_map_east",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_map_west",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_map_zoomin",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_map_zoomout",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_map_maxzoom",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_map_follow",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_map_grid",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_map_mark",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_map_clearmark",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_weapon1",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_weapon2",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_weapon3",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_weapon4",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_weapon5",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_weapon6",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_weapon7",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_weapon8",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_prevweapon",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_nextweapon",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_arti_all",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_arti_health",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_arti_poisonbag",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_arti_blastradius",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_arti_teleport",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_arti_teleportother",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_arti_egg",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_arti_invulnerability",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_message_refresh",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_demo_quit",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_multi_msg",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer1",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer2",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer3",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer4",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer5",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer6",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer7",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
            default_t {
                name: "key_multi_msgplayer8",
                location: None,
                type_0: DefaultType::DEFAULT_KEY,
                untranslated: 0_i32,
                original_translated: 0_i32,
                bound: false,
            },
        ];
        let extra_defaults_len = extra_defaults_list.len() as i32;
        MConfigState {
            configdir: String::new(),
            default_main_config: "",
            default_extra_config: "",
            doom_defaults: default_collection_t {
                numdefaults: doom_defaults_len,
                defaults: doom_defaults_list,
                filename: String::new(),
            },
            extra_defaults: default_collection_t {
                numdefaults: extra_defaults_len,
                defaults: extra_defaults_list,
                filename: String::new(),
            },
        }
    }
}

fn SearchCollection<'a>(
    mut collection: &'a mut default_collection_t,
    name: &str,
) -> Option<&'a mut default_t> {
    collection
        .defaults
        .iter_mut()
        .find(|entry| entry.name == name)
}
static scantokey: [i32; 128] = [
    0_i32,
    27_i32,
    '1' as i32,
    '2' as i32,
    '3' as i32,
    '4' as i32,
    '5' as i32,
    '6' as i32,
    '7' as i32,
    '8' as i32,
    '9' as i32,
    '0' as i32,
    '-' as i32,
    '=' as i32,
    KEY_BACKSPACE,
    9_i32,
    'q' as i32,
    'w' as i32,
    'e' as i32,
    'r' as i32,
    't' as i32,
    'y' as i32,
    'u' as i32,
    'i' as i32,
    'o' as i32,
    'p' as i32,
    '[' as i32,
    ']' as i32,
    13_i32,
    KEY_RCTRL,
    'a' as i32,
    's' as i32,
    'd' as i32,
    'f' as i32,
    'g' as i32,
    'h' as i32,
    'j' as i32,
    'k' as i32,
    'l' as i32,
    ';' as i32,
    '\'' as i32,
    '`' as i32,
    KEY_RSHIFT,
    '\\' as i32,
    'z' as i32,
    'x' as i32,
    'c' as i32,
    'v' as i32,
    'b' as i32,
    'n' as i32,
    'm' as i32,
    ',' as i32,
    '.' as i32,
    '/' as i32,
    KEY_RSHIFT,
    KEYP_MULTIPLY,
    KEY_RALT,
    ' ' as i32,
    KEY_CAPSLOCK,
    KEY_F1,
    KEY_F2,
    KEY_F3,
    KEY_F4,
    KEY_F5,
    KEY_F6,
    KEY_F7,
    KEY_F8,
    KEY_F9,
    KEY_F10,
    KEY_PAUSE,
    KEY_SCRLCK,
    KEY_HOME,
    KEY_UPARROW,
    KEY_PGUP,
    KEY_MINUS,
    KEY_LEFTARROW,
    KEYP_5,
    KEY_RIGHTARROW,
    KEYP_PLUS,
    KEY_END,
    KEY_DOWNARROW,
    KEY_PGDN,
    KEY_INS,
    KEY_DEL,
    0_i32,
    0_i32,
    0_i32,
    KEY_F11,
    KEY_F12,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    0_i32,
    KEY_PRTSCR,
    0_i32,
];
fn ParseIntParameter(strparm: &str) -> i32 {
    let mut parm: i32 = 0;
    M_StrToInt(strparm, &mut parm);
    parm
}
fn SetVariable(state: &mut GameState, name: &str, value: &str) {
    let def = GetDefaultForName(&mut state.m_config, name);
    match def.type_0 {
        DefaultType::DEFAULT_STRING => {
            if let Some(DefaultLocation::Str(loc)) = def.location.clone() {
                *loc(state) = Some(Box::leak(value.to_string().into_boxed_str()));
            }
        }
        DefaultType::DEFAULT_INT | DefaultType::DEFAULT_INT_HEX => {
            if let Some(DefaultLocation::Int(loc)) = def.location.clone() {
                *loc(state) = ParseIntParameter(value);
            }
        }
        DefaultType::DEFAULT_KEY => {
            let mut intparm = ParseIntParameter(value);
            def.untranslated = intparm;
            if (0_i32..128_i32).contains(&intparm) {
                intparm = scantokey[intparm as usize];
            } else {
                intparm = 0_i32;
            }
            def.original_translated = intparm;
            if let Some(DefaultLocation::Int(loc)) = def.location.clone() {
                *loc(state) = intparm;
            }
        }
        DefaultType::DEFAULT_FLOAT => {
            if let Some(DefaultLocation::Float(loc)) = def.location.clone() {
                *loc(state) = value.trim().parse::<f64>().unwrap_or(0.0) as f32;
            }
        }
    };
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
pub fn M_SaveDefaultsAlternate(state: &mut GameState, main_0: &str, extra: &str) {
    let orig_main = state.m_config.doom_defaults.filename.clone();
    let orig_extra = state.m_config.extra_defaults.filename.clone();
    state.m_config.doom_defaults.filename = main_0.to_string();
    state.m_config.extra_defaults.filename = extra.to_string();
    M_SaveDefaults(state);
    state.m_config.doom_defaults.filename = orig_main;
    state.m_config.extra_defaults.filename = orig_extra;
}
pub fn M_LoadDefaults(state: &mut GameState) {
    let mut i: i32 = 0;
    i = M_CheckParmWithArgs(state, "-config", 1_i32);
    if i != 0 {
        state.m_config.doom_defaults.filename = state.m_argv.myargv[(i + 1_i32) as usize]
            .to_str()
            .unwrap()
            .to_string();
        println!("\tdefault file: {}", state.m_config.doom_defaults.filename,);
    } else {
        state.m_config.doom_defaults.filename = format!(
            "{}{}",
            state.m_config.configdir, state.m_config.default_main_config
        );
    }
    println!("saving config in {}", state.m_config.doom_defaults.filename);
    i = M_CheckParmWithArgs(state, "-extraconfig", 1_i32);
    if i != 0 {
        state.m_config.extra_defaults.filename = state.m_argv.myargv[(i + 1_i32) as usize]
            .to_str()
            .unwrap()
            .to_string();
        println!(
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
    match variable.type_0 {
        DefaultType::DEFAULT_INT | DefaultType::DEFAULT_INT_HEX | DefaultType::DEFAULT_KEY => {}
        _ => I_Error(&format!(
            "M_BindVariable_int: '{}' is not an int/key variable",
            name
        )),
    }
    variable.location = Some(DefaultLocation::Int(Rc::new(location)));
    variable.bound = true;
}
pub fn M_BindVariable_f32(
    state: &mut MConfigState,
    name: &str,
    location: impl Fn(&mut GameState) -> &mut f32 + 'static,
) {
    let variable = GetDefaultForName(state, name);
    if variable.type_0 != DefaultType::DEFAULT_FLOAT {
        I_Error(&format!(
            "M_BindVariable_f32: '{}' is not a float variable",
            name
        ));
    }
    variable.location = Some(DefaultLocation::Float(Rc::new(location)));
    variable.bound = true;
}
pub fn M_BindVariable_string(
    state: &mut MConfigState,
    name: &str,
    location: impl Fn(&mut GameState) -> &mut Option<&'static str> + 'static,
) {
    let variable = GetDefaultForName(state, name);
    if variable.type_0 != DefaultType::DEFAULT_STRING {
        I_Error(&format!(
            "M_BindVariable_string: '{}' is not a string variable",
            name
        ));
    }
    variable.location = Some(DefaultLocation::Str(Rc::new(location)));
    variable.bound = true;
}
pub fn M_SetVariable(state: &mut GameState, name: &str, value: &str) -> bool {
    if !GetDefaultForName(&mut state.m_config, name).bound {
        return false;
    }
    SetVariable(state, name, value);
    true
}
pub fn M_GetIntVariable(state: &mut GameState, name: &str) -> i32 {
    let variable = GetDefaultForName(&mut state.m_config, name);
    if !variable.bound
        || (variable.type_0 != DefaultType::DEFAULT_INT
            && variable.type_0 != DefaultType::DEFAULT_INT_HEX)
    {
        return 0_i32;
    }
    match variable.location.clone() {
        Some(DefaultLocation::Int(loc)) => *loc(state),
        _ => 0_i32,
    }
}
pub fn M_GetStrVariable(state: &mut GameState, name: &str) -> Option<&'static str> {
    let variable = GetDefaultForName(&mut state.m_config, name);
    if !variable.bound || variable.type_0 != DefaultType::DEFAULT_STRING {
        return None;
    }
    match variable.location.clone() {
        Some(DefaultLocation::Str(loc)) => *loc(state),
        _ => None,
    }
}
pub fn M_GetFloatVariable(state: &mut GameState, name: &str) -> f32 {
    let variable = GetDefaultForName(&mut state.m_config, name);
    if !variable.bound || variable.type_0 != DefaultType::DEFAULT_FLOAT {
        return 0_i32 as f32;
    }
    match variable.location.clone() {
        Some(DefaultLocation::Float(loc)) => *loc(state),
        _ => 0_i32 as f32,
    }
}
fn GetDefaultConfigDir() -> String {
    ".".to_string()
}
pub fn M_SetConfigDir(state: &mut MConfigState, dir: Option<&str>) {
    if let Some(dir) = dir {
        state.configdir = dir.to_string();
    } else {
        state.configdir = GetDefaultConfigDir();
    }
    if !state.configdir.is_empty() {
        println!("Using {} for configuration and saves", state.configdir);
    }
    M_MakeDirectory(&state.configdir);
}
pub fn M_GetSaveGameDir(state: &mut MConfigState, _iwadname: &'static str) -> String {
    let savegamedir;
    if state.configdir.is_empty() {
        savegamedir = String::new();
    } else {
        savegamedir = format!("{}{}.savegame/", state.configdir, DIR_SEPARATOR_S);
        M_MakeDirectory(&savegamedir);
        println!("Using {} for savegames", savegamedir);
    }
    savegamedir
}
