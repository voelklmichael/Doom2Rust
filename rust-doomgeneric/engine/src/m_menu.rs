use crate::d_event::{Event, GameScreenState};
use crate::d_loop::DLoopState;
use crate::d_main::start_title;
use crate::d_main::DMainState;
use crate::doomstat::DoomstatState;
use crate::dstrings::{DOOM1_ENDMSG, DOOM2_ENDMSG};
use crate::filesystem::DoomFileSystem;
use crate::g_game::GGameState;
use crate::hu_stuff::HuStuffState;
use crate::i_system::error;
use crate::w_wad::WWadState;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::w_wad::lump_bytes_name;

use crate::d_event::EvType;
use crate::d_mode::GameMission;
use crate::d_mode::GameMode;
use crate::d_mode::{skill_from_raw, GameVersion};
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::fixed_cstr::FixedCStr;
use crate::g_game::defered_init_new;
use crate::g_game::g_load_game;
use crate::g_game::g_save_game;
use crate::g_game::g_screen_shot;
use crate::game_state::GameState;
use crate::hu_stuff::HU_FONTSIZE;
use crate::hu_stuff::HU_FONTSTART;
use crate::i_system::i_quit;
use crate::i_timer::get_time;
use crate::i_video::set_palette;
use crate::m_controls::KEY_BACKSPACE;
use crate::m_controls::KEY_CAPSLOCK;
use crate::m_controls::KEY_ENTER;
use crate::m_controls::KEY_ESCAPE;
use crate::m_controls::KEY_PAUSE;
use crate::m_controls::KEY_SCRLCK;
use crate::p_saveg::save_game_file;
use crate::r_main::set_view_size;
use crate::s_sound::s_set_music_volume;
use crate::s_sound::s_start_sound;
use crate::s_sound::set_sfx_volume;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::v_video::cache_patch_name;
use crate::v_video::Screen;

use crate::v_video::cache_patch_num;
use crate::v_video::draw_patch_direct;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MenuId {
    Main,
    Epi,
    New,
    Options,
    Read1,
    Read2,
    Sound,
    Load,
    Save,
}

pub struct MMenuDefsHolder {
    pub main_def: Menu,
    pub epi_def: Menu,
    pub new_def: Menu,
    pub options_def: Menu,
    pub read_def1: Menu,
    pub read_def2: Menu,
    pub sound_def: Menu,
    pub load_def: Menu,
    pub save_def: Menu,
}

impl Default for MMenuDefsHolder {
    fn default() -> Self {
        Self::new()
    }
}

impl MMenuDefsHolder {
    // Kept out of line: `GameState::new` inlines every state constructor, and once
    // `init_game_state` passes 256 KB the Xtensa linker fails ("dangerous relocation:
    // l32r: literal target out of range") building the firmware. These are the biggest.
    #[inline(never)]
    pub fn new() -> Self {
        Self {
            main_def: Menu {
                numitems: MAIN_END as i16,
                prev_menu: None,
                items: vec![
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_NGAME\0\0\0"),
                        routine: Some(new_game as fn(&mut GameState, i32)),
                        alpha_key: b'n',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_OPTION\0\0"),
                        routine: Some(options as fn(&mut GameState, i32)),
                        alpha_key: b'o',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_LOADG\0\0\0"),
                        routine: Some(m_load_game as fn(&mut GameState, i32)),
                        alpha_key: b'l',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_SAVEG\0\0\0"),
                        routine: Some(m_save_game as fn(&mut GameState, i32)),
                        alpha_key: b's',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_RDTHIS\0\0"),
                        routine: Some(read_this as fn(&mut GameState, i32)),
                        alpha_key: b'r',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_QUITG\0\0\0"),
                        routine: Some(quit_doom as fn(&mut GameState, i32)),
                        alpha_key: b'q',
                    },
                ],
                routine: Some(draw_main_menu as fn(&mut GameState) -> ()),
                x: 97,
                y: 64,
                last_on: 0,
            },
            epi_def: Menu {
                numitems: EP_END as i16,
                prev_menu: Some(MenuId::Main),
                items: vec![
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_EPI1\0\0\0\0"),
                        routine: Some(m_episode as fn(&mut GameState, i32)),
                        alpha_key: b'k',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_EPI2\0\0\0\0"),
                        routine: Some(m_episode as fn(&mut GameState, i32)),
                        alpha_key: b't',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_EPI3\0\0\0\0"),
                        routine: Some(m_episode as fn(&mut GameState, i32)),
                        alpha_key: b'i',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_EPI4\0\0\0\0"),
                        routine: Some(m_episode as fn(&mut GameState, i32)),
                        alpha_key: b't',
                    },
                ],
                routine: Some(draw_episode as fn(&mut GameState) -> ()),
                x: 48,
                y: 63,
                last_on: EpisodeMenu::Ep1 as i32 as i16,
            },
            new_def: Menu {
                numitems: NEWG_END as i16,
                prev_menu: Some(MenuId::Epi),
                items: vec![
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_JKILL\0\0\0"),
                        routine: Some(choose_skill as fn(&mut GameState, i32)),
                        alpha_key: b'i',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_ROUGH\0\0\0"),
                        routine: Some(choose_skill as fn(&mut GameState, i32)),
                        alpha_key: b'h',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_HURT\0\0\0\0"),
                        routine: Some(choose_skill as fn(&mut GameState, i32)),
                        alpha_key: b'h',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_ULTRA\0\0\0"),
                        routine: Some(choose_skill as fn(&mut GameState, i32)),
                        alpha_key: b'u',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_NMARE\0\0\0"),
                        routine: Some(choose_skill as fn(&mut GameState, i32)),
                        alpha_key: b'n',
                    },
                ],
                routine: Some(draw_new_game as fn(&mut GameState) -> ()),
                x: 48,
                y: 63,
                last_on: NewGameMenu::Hurtme as i32 as i16,
            },
            options_def: Menu {
                numitems: OPT_END as i16,
                prev_menu: Some(MenuId::Main),
                items: vec![
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_ENDGAM\0\0"),
                        routine: Some(end_game as fn(&mut GameState, i32)),
                        alpha_key: b'e',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_MESSG\0\0\0"),
                        routine: Some(change_messages as fn(&mut GameState, i32)),
                        alpha_key: b'm',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_DETAIL\0\0"),
                        routine: Some(change_detail as fn(&mut GameState, i32)),
                        alpha_key: b'g',
                    },
                    MenuItem {
                        status: 2,
                        name: FixedCStr(*b"M_SCRNSZ\0\0"),
                        routine: Some(size_display as fn(&mut GameState, i32)),
                        alpha_key: b's',
                    },
                    MenuItem {
                        status: -1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: None,
                        alpha_key: b'\0',
                    },
                    MenuItem {
                        status: 2,
                        name: FixedCStr(*b"M_MSENS\0\0\0"),
                        routine: Some(change_sensitivity as fn(&mut GameState, i32)),
                        alpha_key: b'm',
                    },
                    MenuItem {
                        status: -1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: None,
                        alpha_key: b'\0',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"M_SVOL\0\0\0\0"),
                        routine: Some(m_sound as fn(&mut GameState, i32)),
                        alpha_key: b's',
                    },
                ],
                routine: Some(draw_options as fn(&mut GameState) -> ()),
                x: 60,
                y: 37,
                last_on: 0,
            },
            read_def1: Menu {
                numitems: READ1_END as i16,
                prev_menu: Some(MenuId::Main),
                items: vec![MenuItem {
                    status: 1,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: Some(read_this2 as fn(&mut GameState, i32)),
                    alpha_key: 0,
                }],
                routine: Some(draw_read_this1 as fn(&mut GameState) -> ()),
                x: 280,
                y: 185,
                last_on: 0,
            },
            read_def2: Menu {
                numitems: READ2_END as i16,
                prev_menu: Some(MenuId::Read1),
                items: vec![MenuItem {
                    status: 1,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: Some(finish_read_this as fn(&mut GameState, i32)),
                    alpha_key: 0,
                }],
                routine: Some(draw_read_this2 as fn(&mut GameState) -> ()),
                x: 330,
                y: 175,
                last_on: 0,
            },
            sound_def: Menu {
                numitems: SOUND_END as i16,
                prev_menu: Some(MenuId::Options),
                items: vec![
                    MenuItem {
                        status: 2,
                        name: FixedCStr(*b"M_SFXVOL\0\0"),
                        routine: Some(sfx_vol as fn(&mut GameState, i32)),
                        alpha_key: b's',
                    },
                    MenuItem {
                        status: -1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: None,
                        alpha_key: b'\0',
                    },
                    MenuItem {
                        status: 2,
                        name: FixedCStr(*b"M_MUSVOL\0\0"),
                        routine: Some(music_vol as fn(&mut GameState, i32)),
                        alpha_key: b'm',
                    },
                    MenuItem {
                        status: -1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: None,
                        alpha_key: b'\0',
                    },
                ],
                routine: Some(draw_sound as fn(&mut GameState) -> ()),
                x: 80,
                y: 64,
                last_on: 0,
            },
            load_def: Menu {
                numitems: LOAD_END as i16,
                prev_menu: Some(MenuId::Main),
                items: vec![
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(load_select as fn(&mut GameState, i32)),
                        alpha_key: b'1',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(load_select as fn(&mut GameState, i32)),
                        alpha_key: b'2',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(load_select as fn(&mut GameState, i32)),
                        alpha_key: b'3',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(load_select as fn(&mut GameState, i32)),
                        alpha_key: b'4',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(load_select as fn(&mut GameState, i32)),
                        alpha_key: b'5',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(load_select as fn(&mut GameState, i32)),
                        alpha_key: b'6',
                    },
                ],
                routine: Some(draw_load as fn(&mut GameState) -> ()),
                x: 80,
                y: 54,
                last_on: 0,
            },
            save_def: Menu {
                numitems: LOAD_END as i16,
                prev_menu: Some(MenuId::Main),
                items: vec![
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(save_select as fn(&mut GameState, i32)),
                        alpha_key: b'1',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(save_select as fn(&mut GameState, i32)),
                        alpha_key: b'2',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(save_select as fn(&mut GameState, i32)),
                        alpha_key: b'3',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(save_select as fn(&mut GameState, i32)),
                        alpha_key: b'4',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(save_select as fn(&mut GameState, i32)),
                        alpha_key: b'5',
                    },
                    MenuItem {
                        status: 1,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(save_select as fn(&mut GameState, i32)),
                        alpha_key: b'6',
                    },
                ],
                routine: Some(draw_save as fn(&mut GameState) -> ()),
                x: 80,
                y: 54,
                last_on: 0,
            },
        }
    }
}

pub struct MMenuState {
    pub defs: MMenuDefsHolder,
    pub mouse_sensitivity: i32,
    pub show_messages: i32,
    pub detail_level: i32,
    pub screenblocks: i32,
    pub screen_size: i32,
    pub quick_save_slot: i32,
    pub message_to_print: bool,
    pub message_string: String,
    pub messx: i32,
    pub messy: i32,
    pub message_last_menu_active: i32,
    pub message_needs_input: bool,
    pub message_routine: Option<fn(&mut GameState, i32)>,
    /// True while the pending message is the quit confirmation from `quit_doom`.
    pub message_is_quit_prompt: bool,
    pub save_string_enter: bool,
    pub save_slot: i32,
    pub save_char_index: i32,
    pub save_old_string: String,
    pub inhelpscreens: bool,
    pub menuactive: bool,
    pub savegamestrings: [String; 10],
    pub item_on: i16,
    pub skull_anim_counter: i16,
    pub which_skull: i16,
    pub current_menu: MenuId,
    pub epi: i32,
    pub responder_joywait: i32,
    pub responder_mousewait: i32,
    pub responder_mousey: i32,
    pub responder_lasty: i32,
    pub responder_mousex: i32,
    pub responder_lastx: i32,
    pub drawer_x: i16,
    pub drawer_y: i16,
}

impl Default for MMenuState {
    fn default() -> Self {
        Self::new()
    }
}

impl MMenuState {
    pub fn new() -> Self {
        Self {
            defs: MMenuDefsHolder::new(),
            mouse_sensitivity: 5,
            show_messages: 1,
            detail_level: 0,
            screenblocks: 10,
            screen_size: 0,
            quick_save_slot: 0,
            message_to_print: false,
            message_string: String::new(),
            messx: 0,
            messy: 0,
            message_last_menu_active: 0,
            message_needs_input: false,
            message_routine: None,
            message_is_quit_prompt: false,
            save_string_enter: false,
            save_slot: 0,
            save_char_index: 0,
            save_old_string: String::new(),
            inhelpscreens: false,
            menuactive: false,
            savegamestrings: [
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ],
            item_on: 0,
            skull_anim_counter: 0,
            which_skull: 0,
            current_menu: MenuId::Main,
            epi: 0,
            responder_joywait: 0,
            responder_mousewait: 0,
            responder_mousey: 0,
            responder_lasty: 0,
            responder_mousex: 0,
            responder_lastx: 0,
            drawer_x: 0,
            drawer_y: 0,
        }
    }

    pub fn def(&self, id: MenuId) -> &Menu {
        match id {
            MenuId::Main => &self.defs.main_def,
            MenuId::Epi => &self.defs.epi_def,
            MenuId::New => &self.defs.new_def,
            MenuId::Options => &self.defs.options_def,
            MenuId::Read1 => &self.defs.read_def1,
            MenuId::Read2 => &self.defs.read_def2,
            MenuId::Sound => &self.defs.sound_def,
            MenuId::Load => &self.defs.load_def,
            MenuId::Save => &self.defs.save_def,
        }
    }

    pub fn def_mut(&mut self, id: MenuId) -> &mut Menu {
        match id {
            MenuId::Main => &mut self.defs.main_def,
            MenuId::Epi => &mut self.defs.epi_def,
            MenuId::New => &mut self.defs.new_def,
            MenuId::Options => &mut self.defs.options_def,
            MenuId::Read1 => &mut self.defs.read_def1,
            MenuId::Read2 => &mut self.defs.read_def2,
            MenuId::Sound => &mut self.defs.sound_def,
            MenuId::Load => &mut self.defs.load_def,
            MenuId::Save => &mut self.defs.save_def,
        }
    }

    pub fn current(&self) -> &Menu {
        self.def(self.current_menu)
    }

    pub fn current_mut(&mut self) -> &mut Menu {
        self.def_mut(self.current_menu)
    }
}

#[derive(Copy, Clone)]
pub struct MenuItem {
    pub status: i16,
    pub name: FixedCStr<10>,
    pub routine: Option<fn(&mut GameState, i32)>,
    pub alpha_key: u8,
}
#[derive(Clone)]
pub struct Menu {
    pub numitems: i16,
    pub prev_menu: Option<MenuId>,
    pub items: Vec<MenuItem>,
    pub routine: Option<fn(&mut GameState) -> ()>,
    pub x: i16,
    pub y: i16,
    pub last_on: i16,
}
pub const READ2_END: i32 = 1;
pub const READ1_END: i32 = 1;
pub const LOAD_END: i32 = 6;
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; discriminants must stay
pub enum OptionsMenu {
    Endgame = 0,
    Messages = 1,
    Detail = 2,
    Scrnsize = 3,
    OptionEmpty1 = 4,
    Mousesens = 5,
    OptionEmpty2 = 6,
    Soundvol = 7,
}
pub const OPT_END: i32 = 8;
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; discriminants must stay
pub enum SoundMenu {
    SfxVol = 0,
    SfxEmpty1 = 1,
    MusicVol = 2,
    SfxEmpty2 = 3,
}
pub const SOUND_END: i32 = 4;
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; discriminants must stay
pub enum EpisodeMenu {
    Ep1 = 0,
    Ep2 = 1,
    Ep3 = 2,
    Ep4 = 3,
}
pub const EP_END: i32 = 4;
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; discriminants must stay
pub enum NewGameMenu {
    Killthings = 0,
    Toorough = 1,
    Hurtme = 2,
    Violence = 3,
    Nightmare = 4,
}
pub const NEWG_END: i32 = 5;
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; discriminants must stay
pub enum MainMenu {
    Newgame = 0,
    Options = 1,
    Loadgame = 2,
    Savegame = 3,
    Readthis = 4,
    Quitdoom = 5,
}
pub const MAIN_END: i32 = 6;
pub const KEY_NUMLOCK: i32 = 0x80 + 0x45;
pub const GAMMALVL0: &str = "Gamma correction OFF\0";
pub const GAMMALVL1: &str = "Gamma correction level 1\0";
pub const GAMMALVL2: &str = "Gamma correction level 2\0";
pub const GAMMALVL3: &str = "Gamma correction level 3\0";
pub const GAMMALVL4: &str = "Gamma correction level 4\0";
pub const EMPTYSTRING: &str = "empty slot\0";
pub const NUM_QUITMESSAGES: i32 = 8;
pub const SAVESTRINGSIZE: i32 = 24;
pub static GAMMAMSG: [&str; 5] = [GAMMALVL0, GAMMALVL1, GAMMALVL2, GAMMALVL3, GAMMALVL4];
pub const SKULLXOFF: i32 = -32;
pub const LINEHEIGHT: i32 = 16;
pub static SKULL_NAME: [&str; 2] = ["M_SKULL1", "M_SKULL2"];
pub fn read_save_strings(
    d_main: &DMainState,
    fs: &mut dyn DoomFileSystem,
    m_menu: &mut MMenuState,
) {
    for i in 0..LOAD_END {
        let savegame_file = save_game_file(d_main, i);
        match fs.open(&savegame_file) {
            None => {
                m_menu.savegamestrings[i as usize] = EMPTYSTRING.trim_end_matches('\0').to_string();
                m_menu.defs.load_def.items[i as usize].status = 0;
            }
            Some(handle) => {
                let mut buf: [u8; SAVESTRINGSIZE as usize] = [0; SAVESTRINGSIZE as usize];
                fs.read_at(handle, 0, &mut buf);
                fs.close(handle);
                let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
                m_menu.savegamestrings[i as usize] =
                    String::from_utf8_lossy(&buf[..len]).into_owned();
                m_menu.defs.load_def.items[i as usize].status = 1;
            }
        }
    }
}
pub fn draw_load(state: &mut GameState) {
    let __wcache890_24 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_LOADG");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, 72, 28, &__wcache890_24);
    for i in 0..LOAD_END {
        let loaddef_x = state.ui.m_menu.defs.load_def.x as i32;
        let loaddef_y = state.ui.m_menu.defs.load_def.y as i32 + LINEHEIGHT * i;
        draw_save_load_border(state, loaddef_x, loaddef_y);
        let savestr = state.ui.m_menu.savegamestrings[i as usize].clone();
        write_text(state, loaddef_x, loaddef_y, &savestr);
    }
}
pub fn draw_save_load_border(state: &mut GameState, mut x: i32, y: i32) {
    let __wcache908_23 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_LSLEFT");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, x - 8, y + 7, &__wcache908_23);
    for _ in 0..24 {
        let __wcache916_22 =
            cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_LSCNTR");
        let dest_screen = Screen::Video;
        draw_patch_direct(state, dest_screen, x, y + 7, &__wcache916_22);
        x += 8;
    }
    let __wcache925_21 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_LSRGHT");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, x, y + 7, &__wcache925_21);
}
pub fn load_select(state: &mut GameState, choice: i32) {
    let savegame_file = save_game_file(&state.game.d_main, choice);
    g_load_game(&mut state.game.g_game, &savegame_file);
    clear_menus(&mut state.ui.m_menu);
}
pub fn m_load_game(state: &mut GameState, _choice: i32) {
    if state.game.g_game.netgame {
        start_message(
            &mut state.ui.m_menu,
            "you can't do load while in a net game!\n\npress a key.",
            None,
            false,
        );
        return;
    }
    let menudef = MenuId::Load;
    setup_next_menu(&mut state.ui.m_menu, menudef);
    read_save_strings(
        &state.game.d_main,
        &mut *state.assets.fs,
        &mut state.ui.m_menu,
    );
}
pub fn draw_save(state: &mut GameState) {
    let i: i32;
    let __wcache961_20 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_SAVEG");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, 72, 28, &__wcache961_20);
    for i in 0..LOAD_END {
        let loaddef_x = state.ui.m_menu.defs.load_def.x as i32;
        let loaddef_y = state.ui.m_menu.defs.load_def.y as i32 + LINEHEIGHT * i;
        draw_save_load_border(state, loaddef_x, loaddef_y);
        let savestr = state.ui.m_menu.savegamestrings[i as usize].clone();
        write_text(state, loaddef_x, loaddef_y, &savestr);
    }
    if state.ui.m_menu.save_string_enter {
        let savestr = state.ui.m_menu.savegamestrings[state.ui.m_menu.save_slot as usize].clone();
        i = string_width(
            &*state.assets.fs,
            &state.ui.hu_stuff,
            &mut state.assets.w_wad,
            &savestr,
        );
        let text_x = state.ui.m_menu.defs.load_def.x as i32 + i;
        let text_y =
            state.ui.m_menu.defs.load_def.y as i32 + LINEHEIGHT * state.ui.m_menu.save_slot;
        write_text(state, text_x, text_y, "_");
    }
}
pub fn do_save(g_game: &mut GGameState, m_menu: &mut MMenuState, slot: i32) {
    let savegame_name = m_menu.savegamestrings[slot as usize].clone();
    g_save_game(g_game, slot, &savegame_name);
    clear_menus(m_menu);
    if m_menu.quick_save_slot == -2 {
        m_menu.quick_save_slot = slot;
    }
}
pub fn save_select(state: &mut GameState, choice: i32) {
    state.ui.m_menu.save_string_enter = true;
    state.ui.m_menu.save_slot = choice;
    state.ui.m_menu.save_old_string = state.ui.m_menu.savegamestrings[choice as usize].clone();
    if state.ui.m_menu.savegamestrings[choice as usize] == EMPTYSTRING.trim_end_matches('\0') {
        state.ui.m_menu.savegamestrings[choice as usize].clear();
    }
    state.ui.m_menu.save_char_index = state.ui.m_menu.savegamestrings[choice as usize].len() as i32;
}
pub fn m_save_game(state: &mut GameState, _choice: i32) {
    if !state.game.g_game.usergame {
        start_message(
            &mut state.ui.m_menu,
            "you can't save if you aren't playing!\n\npress a key.",
            None,
            false,
        );
        return;
    }
    if state.game.g_game.gamestate != GameScreenState::Level {
        return;
    }
    let menudef = MenuId::Save;
    setup_next_menu(&mut state.ui.m_menu, menudef);
    read_save_strings(
        &state.game.d_main,
        &mut *state.assets.fs,
        &mut state.ui.m_menu,
    );
}
pub fn quick_save_response(state: &mut GameState, key: i32) {
    if key == state.game.m_controls.key_menu_confirm {
        let quick_save_slot = state.ui.m_menu.quick_save_slot;
        do_save(
            &mut state.game.g_game,
            &mut state.ui.m_menu,
            quick_save_slot,
        );
        s_start_sound(state, SoundOrigin::None, SfxName::Swtchx);
    }
}
pub fn quick_save(state: &mut GameState) {
    if !state.game.g_game.usergame {
        s_start_sound(state, SoundOrigin::None, SfxName::Oof);
        return;
    }
    if state.game.g_game.gamestate != GameScreenState::Level {
        return;
    }
    if state.ui.m_menu.quick_save_slot < 0 {
        start_control_panel(&mut state.ui.m_menu);
        read_save_strings(
            &state.game.d_main,
            &mut *state.assets.fs,
            &mut state.ui.m_menu,
        );
        let menudef = MenuId::Save;
        setup_next_menu(&mut state.ui.m_menu, menudef);
        state.ui.m_menu.quick_save_slot = -2;
        return;
    }
    let msg = format!(
        "quicksave over your game named\n\n'{}'?\n\npress y or n.",
        state.ui.m_menu.savegamestrings[state.ui.m_menu.quick_save_slot as usize],
    );
    let routine = Some(quick_save_response as fn(&mut GameState, i32));
    start_message(&mut state.ui.m_menu, &msg, routine, true);
}
pub fn quick_load_response(state: &mut GameState, key: i32) {
    if key == state.game.m_controls.key_menu_confirm {
        let quick_save_slot = state.ui.m_menu.quick_save_slot;
        load_select(state, quick_save_slot);
        s_start_sound(state, SoundOrigin::None, SfxName::Swtchx);
    }
}
pub fn quick_load(g_game: &GGameState, m_menu: &mut MMenuState) {
    if g_game.netgame {
        start_message(
            m_menu,
            "you can't quickload during a netgame!\n\npress a key.",
            None,
            false,
        );
        return;
    }
    if m_menu.quick_save_slot < 0 {
        start_message(
            m_menu,
            "you haven't picked a quicksave slot yet!\n\npress a key.",
            None,
            false,
        );
        return;
    }
    let msg = format!(
        "do you want to quickload the game named\n\n'{}'?\n\npress y or n.",
        m_menu.savegamestrings[m_menu.quick_save_slot as usize],
    );
    let routine = Some(quick_load_response as fn(&mut GameState, i32));
    start_message(m_menu, &msg, routine, true);
}
pub fn draw_read_this1(state: &mut GameState) {
    let lumpname: &str;
    let mut skullx: i32 = 330;
    let mut skully: i32 = 175;
    state.ui.m_menu.inhelpscreens = true;
    match state.game.doomstat.gameversion as u32 {
        1..=5 => {
            if state.game.doomstat.gamemode == GameMode::Commercial {
                lumpname = "HELP";
                skullx = 330;
                skully = 165;
            } else {
                lumpname = "HELP2";
                skullx = 280;
                skully = 185;
            }
        }
        6 | 9 => {
            lumpname = "HELP1";
        }
        7 | 8 => {
            lumpname = "HELP";
        }
        _ => {
            error("Unhandled game version");
        }
    }
    let __wcache1158_19 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, lumpname);
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, 0, 0, &__wcache1158_19);
    state.ui.m_menu.defs.read_def1.x = skullx as i16;
    state.ui.m_menu.defs.read_def1.y = skully as i16;
}
pub fn draw_read_this2(state: &mut GameState) {
    state.ui.m_menu.inhelpscreens = true;
    let __wcache1170_18 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "HELP1");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, 0, 0, &__wcache1170_18);
}
pub fn draw_sound(state: &mut GameState) {
    let __wcache1179_17 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_SVOL");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, 60, 38, &__wcache1179_17);
    let (x, y, vol) = (
        state.ui.m_menu.defs.sound_def.x as i32,
        state.ui.m_menu.defs.sound_def.y as i32 + LINEHEIGHT * (SoundMenu::SfxVol as i32 + 1),
        state.audio.s_sound.sfx_volume,
    );
    draw_thermo(state, x, y, 16, vol);
    let (x, y, vol) = (
        state.ui.m_menu.defs.sound_def.x as i32,
        state.ui.m_menu.defs.sound_def.y as i32 + LINEHEIGHT * (SoundMenu::MusicVol as i32 + 1),
        state.audio.s_sound.music_volume,
    );
    draw_thermo(state, x, y, 16, vol);
}
pub fn m_sound(state: &mut GameState, _choice: i32) {
    let menudef = MenuId::Sound;
    setup_next_menu(&mut state.ui.m_menu, menudef);
}
pub fn sfx_vol(state: &mut GameState, choice: i32) {
    match choice {
        0 => {
            if state.audio.s_sound.sfx_volume != 0 {
                state.audio.s_sound.sfx_volume -= 1;
            }
        }
        1 if state.audio.s_sound.sfx_volume < 15 => {
            state.audio.s_sound.sfx_volume += 1;
        }
        _ => {}
    }
    let sfx_volume = state.audio.s_sound.sfx_volume * 8;
    set_sfx_volume(&mut state.audio.s_sound, sfx_volume);
}
pub fn music_vol(state: &mut GameState, choice: i32) {
    match choice {
        0 => {
            if state.audio.s_sound.music_volume != 0 {
                state.audio.s_sound.music_volume -= 1;
            }
        }
        1 if state.audio.s_sound.music_volume < 15 => {
            state.audio.s_sound.music_volume += 1;
        }
        _ => {}
    }
    let music_volume = state.audio.s_sound.music_volume * 8;
    s_set_music_volume(
        &mut state.audio.i_sound,
        &mut *state.io.platform,
        music_volume,
    );
}
pub fn draw_main_menu(state: &mut GameState) {
    let __wcache1241_16 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_DOOM");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, 94, 2, &__wcache1241_16);
}
pub fn draw_new_game(state: &mut GameState) {
    let __wcache1250_15 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_NEWG");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, 96, 14, &__wcache1250_15);
    let __wcache1256_14 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_SKILL");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, 54, 38, &__wcache1256_14);
}
pub fn new_game(state: &mut GameState, _choice: i32) {
    if state.game.g_game.netgame && !state.game.g_game.demoplayback {
        start_message(
            &mut state.ui.m_menu,
            "you can't start a new game\nwhile in a network game.\n\npress a key.",
            None,
            false,
        );
        return;
    }
    if state.game.doomstat.gamemode == GameMode::Commercial
        || state.game.doomstat.gameversion == GameVersion::Chex
    {
        let menudef = MenuId::New;
        setup_next_menu(&mut state.ui.m_menu, menudef);
    } else {
        let menudef = MenuId::Epi;
        setup_next_menu(&mut state.ui.m_menu, menudef);
    }
}
pub fn draw_episode(state: &mut GameState) {
    let __wcache1286_13 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_EPISOD");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, 54, 38, &__wcache1286_13);
}
pub fn verify_nightmare(state: &mut GameState, key: i32) {
    if key != state.game.m_controls.key_menu_confirm {
        return;
    }
    defered_init_new(
        &mut state.game.g_game,
        skill_from_raw(NewGameMenu::Nightmare as i32),
        state.ui.m_menu.epi + 1,
        1,
    );
    clear_menus(&mut state.ui.m_menu);
}
pub fn choose_skill(state: &mut GameState, choice: i32) {
    if choice == NewGameMenu::Nightmare as i32 {
        start_message(
            &mut state.ui.m_menu,
            "are you sure? this skill level\nisn't even remotely fair.\n\npress y or n.",
            Some(verify_nightmare as fn(&mut GameState, i32)),
            true,
        );
        return;
    }
    defered_init_new(
        &mut state.game.g_game,
        skill_from_raw(choice),
        state.ui.m_menu.epi + 1,
        1,
    );
    clear_menus(&mut state.ui.m_menu);
}
pub fn m_episode(state: &mut GameState, mut choice: i32) {
    if state.game.doomstat.gamemode == GameMode::Shareware && choice != 0 {
        start_message(
            &mut state.ui.m_menu,
            "this is the shareware version of doom.\n\n\
                        you need to order the entire trilogy.\n\n\
                        press a key.",
            None,
            false,
        );
        let menudef = MenuId::Read1;
        setup_next_menu(&mut state.ui.m_menu, menudef);
        return;
    }
    if state.game.doomstat.gamemode == GameMode::Registered && choice > 2 {
        doom_eprintln!(
            state.io.platform,
            "M_Episode: 4th episode requires UltimateDOOM"
        );
        choice = 0;
    }
    state.ui.m_menu.epi = choice;
    let menudef = MenuId::New;
    setup_next_menu(&mut state.ui.m_menu, menudef);
}
static DETAIL_NAMES: [&str; 2] = ["M_GDHIGH", "M_GDLOW"];
static MSG_NAMES: [&str; 2] = ["M_MSGOFF", "M_MSGON"];
pub fn draw_options(state: &mut GameState) {
    let __wcache1358_12 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_OPTTTL");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, 108, 15, &__wcache1358_12);
    let __wcache1364_11 = cache_patch_name(
        &*state.assets.fs,
        &mut state.assets.w_wad,
        DETAIL_NAMES[state.ui.m_menu.detail_level as usize],
    );
    let dest_screen = Screen::Video;
    draw_patch_direct(
        state,
        dest_screen,
        state.ui.m_menu.defs.options_def.x as i32 + 175,
        state.ui.m_menu.defs.options_def.y as i32 + LINEHEIGHT * OptionsMenu::Detail as i32,
        &__wcache1364_11,
    );
    let __wcache1373_10 = cache_patch_name(
        &*state.assets.fs,
        &mut state.assets.w_wad,
        MSG_NAMES[state.ui.m_menu.show_messages as usize],
    );
    let dest_screen = Screen::Video;
    draw_patch_direct(
        state,
        dest_screen,
        state.ui.m_menu.defs.options_def.x as i32 + 120,
        state.ui.m_menu.defs.options_def.y as i32 + LINEHEIGHT * OptionsMenu::Messages as i32,
        &__wcache1373_10,
    );
    let (x, y, sens) = (
        state.ui.m_menu.defs.options_def.x as i32,
        state.ui.m_menu.defs.options_def.y as i32
            + LINEHEIGHT * (OptionsMenu::Mousesens as i32 + 1),
        state.ui.m_menu.mouse_sensitivity,
    );
    draw_thermo(state, x, y, 10, sens);
    let (x, y, sz) = (
        state.ui.m_menu.defs.options_def.x as i32,
        state.ui.m_menu.defs.options_def.y as i32 + LINEHEIGHT * (OptionsMenu::Scrnsize as i32 + 1),
        state.ui.m_menu.screen_size,
    );
    draw_thermo(state, x, y, 9, sz);
}
pub fn options(state: &mut GameState, _choice: i32) {
    let menudef = MenuId::Options;
    setup_next_menu(&mut state.ui.m_menu, menudef);
}
pub fn change_messages(state: &mut GameState, _choice: i32) {
    state.ui.m_menu.show_messages = 1 - state.ui.m_menu.show_messages;
    if state.ui.m_menu.show_messages == 0 {
        state.game.g_game.players[state.game.g_game.consoleplayer].message =
            Some("Messages OFF".to_string());
    } else {
        state.game.g_game.players[state.game.g_game.consoleplayer].message =
            Some("Messages ON".to_string());
    }
    state.ui.hu_stuff.message_dontfuckwithme = true;
}
pub fn end_game_response(state: &mut GameState, key: i32) {
    if key != state.game.m_controls.key_menu_confirm {
        return;
    }
    let item_on = state.ui.m_menu.item_on;
    state.ui.m_menu.current_mut().last_on = item_on;
    clear_menus(&mut state.ui.m_menu);
    start_title(&mut state.game.d_main, &mut state.game.g_game);
}
pub fn end_game(state: &mut GameState, _choice: i32) {
    if !state.game.g_game.usergame {
        s_start_sound(state, SoundOrigin::None, SfxName::Oof);
        return;
    }
    if state.game.g_game.netgame {
        start_message(
            &mut state.ui.m_menu,
            "you can't end a netgame!\n\npress a key.",
            None,
            false,
        );
        return;
    }
    start_message(
        &mut state.ui.m_menu,
        "are you sure you want to end the game?\n\npress y or n.",
        Some(end_game_response as fn(&mut GameState, i32)),
        true,
    );
}
pub fn read_this(state: &mut GameState, _choice: i32) {
    let menudef = MenuId::Read1;
    setup_next_menu(&mut state.ui.m_menu, menudef);
}
pub fn read_this2(state: &mut GameState, _choice: i32) {
    if state.game.doomstat.gameversion.below_1_9()
        && state.game.doomstat.gamemode != GameMode::Commercial
    {
        let menudef = MenuId::Read2;
        setup_next_menu(&mut state.ui.m_menu, menudef);
    } else {
        finish_read_this(state, 0);
    }
}
pub fn finish_read_this(state: &mut GameState, _choice: i32) {
    let menudef = MenuId::Main;
    setup_next_menu(&mut state.ui.m_menu, menudef);
}
pub static QUITSOUNDS: [SfxName; 8] = [
    SfxName::Pldeth,
    SfxName::Dmpain,
    SfxName::Popain,
    SfxName::Slop,
    SfxName::Telept,
    SfxName::Posit1,
    SfxName::Posit3,
    SfxName::Sgtatk,
];
pub static QUITSOUNDS2: [SfxName; 8] = [
    SfxName::Vilact,
    SfxName::Getpow,
    SfxName::Boscub,
    SfxName::Slop,
    SfxName::Skeswg,
    SfxName::Kntdth,
    SfxName::Bspact,
    SfxName::Sgtatk,
];
pub fn quit_response(state: &mut GameState, key: i32) {
    if key != state.game.m_controls.key_menu_confirm {
        return;
    }
    if !state.game.g_game.netgame {
        if state.game.doomstat.gamemode == GameMode::Commercial {
            s_start_sound(
                state,
                SoundOrigin::None,
                QUITSOUNDS2[(state.game.d_loop.gametic >> 2 & 7) as usize],
            );
        } else {
            s_start_sound(
                state,
                SoundOrigin::None,
                QUITSOUNDS[(state.game.d_loop.gametic >> 2 & 7) as usize],
            );
        }
    }
    i_quit(state);
}
fn select_end_message(d_loop: &DLoopState, doomstat: &DoomstatState) -> &'static str {
    let endmsg: &'static [&'static str; 8] = if doomstat.gamemission.base() == GameMission::Doom {
        &DOOM1_ENDMSG
    } else {
        &DOOM2_ENDMSG
    };
    endmsg[(d_loop.gametic % NUM_QUITMESSAGES) as usize]
}
pub fn quit_doom(state: &mut GameState, _choice: i32) {
    let msg = format!(
        "{}\n\n(press y to quit to dos.)",
        select_end_message(&state.game.d_loop, &state.game.doomstat)
    );
    let routine = Some(quit_response as fn(&mut GameState, i32));
    start_message(&mut state.ui.m_menu, &msg, routine, true);
    state.ui.m_menu.message_is_quit_prompt = true;
}
pub fn change_sensitivity(state: &mut GameState, choice: i32) {
    match choice {
        0 => {
            if state.ui.m_menu.mouse_sensitivity != 0 {
                state.ui.m_menu.mouse_sensitivity -= 1;
            }
        }
        1 if state.ui.m_menu.mouse_sensitivity < 9 => {
            state.ui.m_menu.mouse_sensitivity += 1;
        }
        _ => {}
    }
}
pub fn change_detail(state: &mut GameState, _choice: i32) {
    state.ui.m_menu.detail_level = 1 - state.ui.m_menu.detail_level;
    let (screenblocks, detail_level) = (state.ui.m_menu.screenblocks, state.ui.m_menu.detail_level);
    set_view_size(&mut state.render.r_main, screenblocks, detail_level);
    if state.ui.m_menu.detail_level == 0 {
        state.game.g_game.players[state.game.g_game.consoleplayer].message =
            Some("High detail".to_string());
    } else {
        state.game.g_game.players[state.game.g_game.consoleplayer].message =
            Some("Low detail".to_string());
    }
}
pub fn size_display(state: &mut GameState, choice: i32) {
    match choice {
        0 => {
            if state.ui.m_menu.screen_size > 0 {
                state.ui.m_menu.screenblocks -= 1;
                state.ui.m_menu.screen_size -= 1;
            }
        }
        1 if state.ui.m_menu.screen_size < 8 => {
            state.ui.m_menu.screenblocks += 1;
            state.ui.m_menu.screen_size += 1;
        }
        _ => {}
    }
    let (screenblocks, detail_level) = (state.ui.m_menu.screenblocks, state.ui.m_menu.detail_level);
    set_view_size(&mut state.render.r_main, screenblocks, detail_level);
}
pub fn draw_thermo(state: &mut GameState, x: i32, y: i32, therm_width: i32, therm_dot: i32) {
    let mut xx: i32 = x;
    let __wcache1619_9 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_THERML");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, xx, y, &__wcache1619_9);
    xx += 8;
    for _ in 0..therm_width {
        let __wcache1628_8 =
            cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_THERMM");
        let dest_screen = Screen::Video;
        draw_patch_direct(state, dest_screen, xx, y, &__wcache1628_8);
        xx += 8;
    }
    let __wcache1637_7 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_THERMR");
    let dest_screen = Screen::Video;
    draw_patch_direct(state, dest_screen, xx, y, &__wcache1637_7);
    let __wcache1643_6 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_THERMO");
    let dest_screen = Screen::Video;
    draw_patch_direct(
        state,
        dest_screen,
        x + 8 + therm_dot * 8,
        y,
        &__wcache1643_6,
    );
}
pub fn start_message(
    m_menu: &mut MMenuState,
    string: &str,
    routine: Option<fn(&mut GameState, i32)>,
    input: bool,
) {
    m_menu.message_last_menu_active = m_menu.menuactive as i32;
    m_menu.message_to_print = true;
    m_menu.message_string = string.to_string();
    m_menu.message_routine = routine;
    m_menu.message_is_quit_prompt = false;
    m_menu.message_needs_input = input;
    m_menu.menuactive = true;
}
pub fn string_width(
    fs: &dyn DoomFileSystem,
    hu_stuff: &HuStuffState,
    w_wad: &mut WWadState,
    string: &str,
) -> i32 {
    let mut w: i32 = 0;
    let mut c: i32;
    for b in string.bytes() {
        c = b.to_ascii_uppercase() as i32 - HU_FONTSTART;
        if (0..HU_FONTSIZE).contains(&c) {
            let font_patch = cache_patch_num(fs, w_wad, hu_stuff.hu_font[c as usize]);
            w += font_patch.width();
        } else {
            w += 4;
        }
    }
    w
}
pub fn string_height(
    fs: &dyn DoomFileSystem,
    hu_stuff: &HuStuffState,
    w_wad: &mut WWadState,
    string: &str,
) -> i32 {
    let height: i32 = cache_patch_num(fs, w_wad, hu_stuff.hu_font[0]).height();
    let mut h: i32 = height;
    for b in string.bytes() {
        if b == b'\n' {
            h += height;
        }
    }
    h
}
pub fn write_text(state: &mut GameState, x: i32, y: i32, string: &str) {
    let mut w: i32;
    let mut c: i32;
    let mut cx: i32 = x;
    let mut cy: i32 = y;
    'outer: for b in string.bytes() {
        c = b as i32;
        if c == '\n' as i32 {
            cx = x;
            cy += 12;
        } else {
            c = (c as u8).to_ascii_uppercase() as i32 - HU_FONTSTART;
            if (0..HU_FONTSIZE).contains(&c) {
                let font_patch = cache_patch_num(
                    &*state.assets.fs,
                    &mut state.assets.w_wad,
                    state.ui.hu_stuff.hu_font[c as usize],
                );
                w = font_patch.width();
                if cx + w > SCREENWIDTH {
                    break 'outer;
                }
                let dest_screen = Screen::Video;
                draw_patch_direct(state, dest_screen, cx, cy, &font_patch);
                cx += w;
            } else {
                cx += 4;
            }
        }
    }
}
fn is_null_key(key: i32) -> bool {
    key == KEY_PAUSE || key == KEY_CAPSLOCK || key == KEY_SCRLCK || key == KEY_NUMLOCK
}
pub fn m_responder(state: &mut GameState, ev: &Event) -> bool {
    let mut i: i32;
    if state.game.g_game.testcontrols {
        if ev.kind == EvType::Quit
            || ev.kind == EvType::Keydown
                && (ev.data1 == state.game.m_controls.key_menu_activate
                    || ev.data1 == state.game.m_controls.key_menu_quit)
        {
            i_quit(state);
            return true;
        }
        return false;
    }
    if ev.kind == EvType::Quit {
        if state.ui.m_menu.menuactive
            && state.ui.m_menu.message_to_print
            && state.ui.m_menu.message_is_quit_prompt
        {
            let key_menu_confirm = state.game.m_controls.key_menu_confirm;
            quit_response(state, key_menu_confirm);
        } else {
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
            quit_doom(state, 0);
        }
        return true;
    }
    let mut ch: i32 = 0;
    let mut key: i32 = -1;
    if ev.kind == EvType::Joystick
        && state.ui.m_menu.responder_joywait
            < get_time(&mut state.io.i_timer, &mut *state.io.platform)
    {
        if ev.data3 < 0 {
            key = state.game.m_controls.key_menu_up;
            state.ui.m_menu.responder_joywait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 5;
        } else if ev.data3 > 0 {
            key = state.game.m_controls.key_menu_down;
            state.ui.m_menu.responder_joywait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 5;
        }
        if ev.data2 < 0 {
            key = state.game.m_controls.key_menu_left;
            state.ui.m_menu.responder_joywait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 2;
        } else if ev.data2 > 0 {
            key = state.game.m_controls.key_menu_right;
            state.ui.m_menu.responder_joywait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 2;
        }
        if ev.data1 & 1 != 0 {
            key = state.game.m_controls.key_menu_forward;
            state.ui.m_menu.responder_joywait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 5;
        }
        if ev.data1 & 2 != 0 {
            key = state.game.m_controls.key_menu_back;
            state.ui.m_menu.responder_joywait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 5;
        }
        if state.game.m_controls.joybmenu >= 0
            && ev.data1 & 1 << state.game.m_controls.joybmenu != 0
        {
            key = state.game.m_controls.key_menu_activate;
            state.ui.m_menu.responder_joywait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 5;
        }
    } else if ev.kind == EvType::Mouse
        && state.ui.m_menu.responder_mousewait
            < get_time(&mut state.io.i_timer, &mut *state.io.platform)
    {
        state.ui.m_menu.responder_mousey += ev.data3;
        if state.ui.m_menu.responder_mousey < state.ui.m_menu.responder_lasty - 30 {
            key = state.game.m_controls.key_menu_down;
            state.ui.m_menu.responder_mousewait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 5;
            state.ui.m_menu.responder_lasty -= 30;
            state.ui.m_menu.responder_mousey = state.ui.m_menu.responder_lasty;
        } else if state.ui.m_menu.responder_mousey > state.ui.m_menu.responder_lasty + 30 {
            key = state.game.m_controls.key_menu_up;
            state.ui.m_menu.responder_mousewait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 5;
            state.ui.m_menu.responder_lasty += 30;
            state.ui.m_menu.responder_mousey = state.ui.m_menu.responder_lasty;
        }
        state.ui.m_menu.responder_mousex += ev.data2;
        if state.ui.m_menu.responder_mousex < state.ui.m_menu.responder_lastx - 30 {
            key = state.game.m_controls.key_menu_left;
            state.ui.m_menu.responder_mousewait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 5;
            state.ui.m_menu.responder_lastx -= 30;
            state.ui.m_menu.responder_mousex = state.ui.m_menu.responder_lastx;
        } else if state.ui.m_menu.responder_mousex > state.ui.m_menu.responder_lastx + 30 {
            key = state.game.m_controls.key_menu_right;
            state.ui.m_menu.responder_mousewait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 5;
            state.ui.m_menu.responder_lastx += 30;
            state.ui.m_menu.responder_mousex = state.ui.m_menu.responder_lastx;
        }
        if ev.data1 & 1 != 0 {
            key = state.game.m_controls.key_menu_forward;
            state.ui.m_menu.responder_mousewait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 15;
        }
        if ev.data1 & 2 != 0 {
            key = state.game.m_controls.key_menu_back;
            state.ui.m_menu.responder_mousewait =
                get_time(&mut state.io.i_timer, &mut *state.io.platform) + 15;
        }
    } else if ev.kind == EvType::Keydown {
        key = ev.data1;
        ch = ev.data2;
    }
    if key == -1 {
        return false;
    }
    if state.ui.m_menu.save_string_enter {
        match key {
            KEY_BACKSPACE => {
                if state.ui.m_menu.save_char_index > 0 {
                    state.ui.m_menu.save_char_index -= 1;
                    state.ui.m_menu.savegamestrings[state.ui.m_menu.save_slot as usize]
                        .truncate(state.ui.m_menu.save_char_index as usize);
                }
            }
            KEY_ESCAPE => {
                state.ui.m_menu.save_string_enter = false;
                state.ui.m_menu.savegamestrings[state.ui.m_menu.save_slot as usize] =
                    state.ui.m_menu.save_old_string.clone();
            }
            KEY_ENTER => {
                state.ui.m_menu.save_string_enter = false;
                if !state.ui.m_menu.savegamestrings[state.ui.m_menu.save_slot as usize].is_empty() {
                    let save_slot = state.ui.m_menu.save_slot;
                    do_save(&mut state.game.g_game, &mut state.ui.m_menu, save_slot);
                }
            }
            _ => {
                if state.io.i_input.vanilla_keyboard_mapping != 0 {
                    ch = key;
                }
                ch = (ch as u8).to_ascii_uppercase() as i32;
                if !(ch != ' ' as i32
                    && (ch - HU_FONTSTART < 0 || ch - HU_FONTSTART >= HU_FONTSIZE))
                {
                    let savestr =
                        state.ui.m_menu.savegamestrings[state.ui.m_menu.save_slot as usize].clone();
                    if (32..=127).contains(&ch)
                        && state.ui.m_menu.save_char_index < SAVESTRINGSIZE - 1
                        && string_width(
                            &*state.assets.fs,
                            &state.ui.hu_stuff,
                            &mut state.assets.w_wad,
                            &savestr,
                        ) < (SAVESTRINGSIZE - 2) * 8
                    {
                        state.ui.m_menu.save_char_index += 1;
                        state.ui.m_menu.savegamestrings[state.ui.m_menu.save_slot as usize]
                            .push(ch as u8 as char);
                    }
                }
            }
        }
        return true;
    }
    if state.ui.m_menu.message_to_print {
        if state.ui.m_menu.message_needs_input
            && key != ' ' as i32
            && key != KEY_ESCAPE
            && key != state.game.m_controls.key_menu_confirm
            && key != state.game.m_controls.key_menu_abort
        {
            return false;
        }
        state.ui.m_menu.menuactive = state.ui.m_menu.message_last_menu_active != 0;
        state.ui.m_menu.message_to_print = false;
        if state.ui.m_menu.message_routine.is_some() {
            state
                .ui
                .m_menu
                .message_routine
                .expect("non-null function pointer")(state, key);
        }
        state.ui.m_menu.menuactive = false;
        s_start_sound(state, SoundOrigin::None, SfxName::Swtchx);
        return true;
    }
    if state.game.d_main.devparm && key == state.game.m_controls.key_menu_help
        || key != 0 && key == state.game.m_controls.key_menu_screenshot
    {
        g_screen_shot(&mut state.game.g_game);
        return true;
    }
    if !state.ui.m_menu.menuactive {
        if key == state.game.m_controls.key_menu_decscreen {
            if state.ui.am_map.automapactive || state.ui.hu_stuff.chat_on {
                return false;
            }
            size_display(state, 0);
            s_start_sound(state, SoundOrigin::None, SfxName::Stnmov);
            return true;
        } else if key == state.game.m_controls.key_menu_incscreen {
            if state.ui.am_map.automapactive || state.ui.hu_stuff.chat_on {
                return false;
            }
            size_display(state, 1);
            s_start_sound(state, SoundOrigin::None, SfxName::Stnmov);
            return true;
        } else if key == state.game.m_controls.key_menu_help {
            start_control_panel(&mut state.ui.m_menu);
            if state.game.doomstat.gamemode == GameMode::Retail {
                state.ui.m_menu.current_menu = MenuId::Read2;
            } else {
                state.ui.m_menu.current_menu = MenuId::Read1;
            }
            state.ui.m_menu.item_on = 0;
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
            return true;
        } else if key == state.game.m_controls.key_menu_save {
            start_control_panel(&mut state.ui.m_menu);
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
            m_save_game(state, 0);
            return true;
        } else if key == state.game.m_controls.key_menu_load {
            start_control_panel(&mut state.ui.m_menu);
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
            m_load_game(state, 0);
            return true;
        } else if key == state.game.m_controls.key_menu_volume {
            start_control_panel(&mut state.ui.m_menu);
            state.ui.m_menu.current_menu = MenuId::Sound;
            state.ui.m_menu.item_on = SoundMenu::SfxVol as i32 as i16;
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
            return true;
        } else if key == state.game.m_controls.key_menu_detail {
            change_detail(state, 0);
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
            return true;
        } else if key == state.game.m_controls.key_menu_qsave {
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
            quick_save(state);
            return true;
        } else if key == state.game.m_controls.key_menu_endgame {
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
            end_game(state, 0);
            return true;
        } else if key == state.game.m_controls.key_menu_messages {
            change_messages(state, 0);
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
            return true;
        } else if key == state.game.m_controls.key_menu_qload {
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
            quick_load(&state.game.g_game, &mut state.ui.m_menu);
            return true;
        } else if key == state.game.m_controls.key_menu_quit {
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
            quit_doom(state, 0);
            return true;
        } else if key == state.game.m_controls.key_menu_gamma {
            state.io.i_video.usegamma += 1;
            if state.io.i_video.usegamma > 4 {
                state.io.i_video.usegamma = 0;
            }
            state.game.g_game.players[state.game.g_game.consoleplayer].message =
                Some(GAMMAMSG[state.io.i_video.usegamma as usize].to_string());
            let pal = lump_bytes_name(&*state.assets.fs, &mut state.assets.w_wad, "PLAYPAL");
            set_palette(&mut state.io.i_video, &pal[..768]);
            return true;
        }
    }
    if !state.ui.m_menu.menuactive {
        if key == state.game.m_controls.key_menu_activate {
            start_control_panel(&mut state.ui.m_menu);
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
            return true;
        }
        return false;
    }
    if key == state.game.m_controls.key_menu_down {
        loop {
            if state.ui.m_menu.item_on as i32 + 1 > state.ui.m_menu.current().numitems as i32 - 1 {
                state.ui.m_menu.item_on = 0;
            } else {
                state.ui.m_menu.item_on += 1;
            }
            s_start_sound(state, SoundOrigin::None, SfxName::Pstop);
            if state.ui.m_menu.current().items[state.ui.m_menu.item_on as usize].status as i32 != -1
            {
                break;
            }
        }
        return true;
    } else if key == state.game.m_controls.key_menu_up {
        loop {
            if state.ui.m_menu.item_on == 0 {
                state.ui.m_menu.item_on = (state.ui.m_menu.current().numitems as i32 - 1) as i16;
            } else {
                state.ui.m_menu.item_on -= 1;
            }
            s_start_sound(state, SoundOrigin::None, SfxName::Pstop);
            if state.ui.m_menu.current().items[state.ui.m_menu.item_on as usize].status as i32 != -1
            {
                break;
            }
        }
        return true;
    } else if key == state.game.m_controls.key_menu_left {
        let item = state.ui.m_menu.current().items[state.ui.m_menu.item_on as usize];
        if let Some(routine) = item.routine.filter(|_| item.status as i32 == 2) {
            s_start_sound(state, SoundOrigin::None, SfxName::Stnmov);
            routine(state, 0);
        }
        return true;
    } else if key == state.game.m_controls.key_menu_right {
        let item = state.ui.m_menu.current().items[state.ui.m_menu.item_on as usize];
        if let Some(routine) = item.routine.filter(|_| item.status as i32 == 2) {
            s_start_sound(state, SoundOrigin::None, SfxName::Stnmov);
            routine(state, 1);
        }
        return true;
    } else if key == state.game.m_controls.key_menu_forward {
        let item = state.ui.m_menu.current().items[state.ui.m_menu.item_on as usize];
        if let Some(routine) = item.routine.filter(|_| item.status as i32 != 0) {
            let item_on = state.ui.m_menu.item_on;
            state.ui.m_menu.current_mut().last_on = item_on;
            if item.status as i32 == 2 {
                routine(state, 1);
                s_start_sound(state, SoundOrigin::None, SfxName::Stnmov);
            } else {
                let item_on = state.ui.m_menu.item_on as i32;
                routine(state, item_on);
                s_start_sound(state, SoundOrigin::None, SfxName::Pistol);
            }
        }
        return true;
    } else if key == state.game.m_controls.key_menu_activate {
        let item_on = state.ui.m_menu.item_on;
        state.ui.m_menu.current_mut().last_on = item_on;
        clear_menus(&mut state.ui.m_menu);
        s_start_sound(state, SoundOrigin::None, SfxName::Swtchx);
        return true;
    } else if key == state.game.m_controls.key_menu_back {
        let item_on = state.ui.m_menu.item_on;
        state.ui.m_menu.current_mut().last_on = item_on;
        if let Some(prev) = state.ui.m_menu.current().prev_menu {
            state.ui.m_menu.current_menu = prev;
            state.ui.m_menu.item_on = state.ui.m_menu.current().last_on;
            s_start_sound(state, SoundOrigin::None, SfxName::Swtchn);
        }
        return true;
    } else if ch != 0 || is_null_key(key) {
        for i in state.ui.m_menu.item_on as i32 + 1..state.ui.m_menu.current().numitems as i32 {
            if state.ui.m_menu.current().items[i as usize].alpha_key as i32 == ch {
                state.ui.m_menu.item_on = i as i16;
                s_start_sound(state, SoundOrigin::None, SfxName::Pstop);
                return true;
            }
        }
        i = 0;
        while i <= state.ui.m_menu.item_on as i32 {
            if state.ui.m_menu.current().items[i as usize].alpha_key as i32 == ch {
                state.ui.m_menu.item_on = i as i16;
                s_start_sound(state, SoundOrigin::None, SfxName::Pstop);
                return true;
            }
            i += 1;
        }
    }
    false
}
pub fn start_control_panel(m_menu: &mut MMenuState) {
    if m_menu.menuactive {
        return;
    }
    m_menu.menuactive = true;
    m_menu.current_menu = MenuId::Main;
    m_menu.item_on = m_menu.current().last_on;
}
pub fn m_drawer(state: &mut GameState) {
    state.ui.m_menu.inhelpscreens = false;
    if state.ui.m_menu.message_to_print {
        let message_string = state.ui.m_menu.message_string.clone();
        state.ui.m_menu.drawer_y = (SCREENHEIGHT / 2
            - string_height(
                &*state.assets.fs,
                &state.ui.hu_stuff,
                &mut state.assets.w_wad,
                &message_string,
            ) / 2) as i16;
        for line in message_string.split('\n') {
            let line = if line.len() > 79 { &line[..79] } else { line };
            state.ui.m_menu.drawer_x = (SCREENWIDTH / 2
                - string_width(
                    &*state.assets.fs,
                    &state.ui.hu_stuff,
                    &mut state.assets.w_wad,
                    line,
                ) / 2) as i16;
            write_text(
                state,
                state.ui.m_menu.drawer_x as i32,
                state.ui.m_menu.drawer_y as i32,
                line,
            );
            state.ui.m_menu.drawer_y = (state.ui.m_menu.drawer_y as i32
                + cache_patch_num(
                    &*state.assets.fs,
                    &mut state.assets.w_wad,
                    state.ui.hu_stuff.hu_font[0],
                )
                .height()) as i16;
        }
        return;
    }
    if !state.ui.m_menu.menuactive {
        return;
    }
    let draw_routine = state.ui.m_menu.current().routine;
    if let Some(draw_routine) = draw_routine {
        draw_routine(state);
    }
    state.ui.m_menu.drawer_x = state.ui.m_menu.current().x;
    state.ui.m_menu.drawer_y = state.ui.m_menu.current().y;
    let max: u32 = state.ui.m_menu.current().numitems as u32;
    let mut i: u32 = 0;
    while i < max {
        let item_name = state.ui.m_menu.current().items[i as usize].name;
        if !item_name.is_empty() {
            let __wcache2221_2 = cache_patch_name(
                &*state.assets.fs,
                &mut state.assets.w_wad,
                &item_name.as_str(),
            );
            let dest_screen = Screen::Video;
            draw_patch_direct(
                state,
                dest_screen,
                state.ui.m_menu.drawer_x as i32,
                state.ui.m_menu.drawer_y as i32,
                &__wcache2221_2,
            );
        }
        state.ui.m_menu.drawer_y = (state.ui.m_menu.drawer_y as i32 + LINEHEIGHT) as i16;
        i = i.wrapping_add(1);
    }
    let __wcache2231_1 = cache_patch_name(
        &*state.assets.fs,
        &mut state.assets.w_wad,
        SKULL_NAME[state.ui.m_menu.which_skull as usize],
    );
    let dest_screen = Screen::Video;
    draw_patch_direct(
        state,
        dest_screen,
        state.ui.m_menu.drawer_x as i32 + SKULLXOFF,
        state.ui.m_menu.current().y as i32 - 5 + state.ui.m_menu.item_on as i32 * LINEHEIGHT,
        &__wcache2231_1,
    );
}
pub fn clear_menus(m_menu: &mut MMenuState) {
    m_menu.menuactive = false;
}
pub fn setup_next_menu(m_menu: &mut MMenuState, menudef: MenuId) {
    m_menu.current_menu = menudef;
    m_menu.item_on = m_menu.current().last_on;
}
pub fn m_ticker(state: &mut GameState) {
    state.ui.m_menu.skull_anim_counter -= 1;
    if state.ui.m_menu.skull_anim_counter as i32 <= 0 {
        state.ui.m_menu.which_skull = (state.ui.m_menu.which_skull as i32 ^ 1) as i16;
        state.ui.m_menu.skull_anim_counter = 8;
    }
}
pub fn m_init(doomstat: &DoomstatState, m_menu: &mut MMenuState) {
    m_menu.current_menu = MenuId::Main;
    m_menu.menuactive = false;
    m_menu.item_on = m_menu.current().last_on;
    m_menu.which_skull = 0;
    m_menu.skull_anim_counter = 10;
    m_menu.screen_size = m_menu.screenblocks - 3;
    m_menu.message_to_print = false;
    m_menu.message_string = String::new();
    m_menu.message_last_menu_active = m_menu.menuactive as i32;
    m_menu.quick_save_slot = -1;
    if doomstat.gamemode == GameMode::Commercial {
        m_menu.defs.main_def.items[MainMenu::Readthis as usize] =
            m_menu.defs.main_def.items[MainMenu::Quitdoom as usize];
        m_menu.defs.main_def.numitems -= 1;
        m_menu.defs.main_def.y = (m_menu.defs.main_def.y as i32 + 8) as i16;
        m_menu.defs.new_def.prev_menu = Some(MenuId::Main);
    }
    if !doomstat.gameversion.is_ultimate_or_higher() {
        m_menu.defs.epi_def.numitems -= 1;
    }
}
