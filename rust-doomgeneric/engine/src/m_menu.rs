use crate::d_event::{event_t, GameScreenState};
use crate::d_main::D_StartTitle;
use crate::dstrings::{doom1_endmsg, doom2_endmsg};
use crate::i_system::I_Error;

use crate::w_wad::W_LumpBytesName;

use crate::d_event::EvType;
use crate::d_mode::GameMission_t;
use crate::d_mode::GameMode_t;
use crate::d_mode::{skill_from_raw, GameVersion};
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::fixed_cstr::FixedCStr;
use crate::g_game::G_DeferedInitNew;
use crate::g_game::G_LoadGame;
use crate::g_game::G_SaveGame;
use crate::g_game::G_ScreenShot;
use crate::game_state::GameState;
use crate::hu_stuff::HU_FONTSIZE;
use crate::hu_stuff::HU_FONTSTART;
use crate::i_system::I_Quit;
use crate::i_timer::I_GetTime;
use crate::i_video::I_SetPalette;
use crate::m_controls::KEY_BACKSPACE;
use crate::m_controls::KEY_CAPSLOCK;
use crate::m_controls::KEY_ENTER;
use crate::m_controls::KEY_ESCAPE;
use crate::m_controls::KEY_PAUSE;
use crate::m_controls::KEY_SCRLCK;
use crate::p_saveg::P_SaveGameFile;
use crate::r_main::R_SetViewSize;
use crate::s_sound::S_SetMusicVolume;
use crate::s_sound::S_SetSfxVolume;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::v_video::Screen;
use crate::v_video::V_CachePatchName;

use crate::v_video::V_CachePatchNum;
use crate::v_video::V_DrawPatchDirect;

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
    pub MainDef: menu_t,
    pub EpiDef: menu_t,
    pub NewDef: menu_t,
    pub OptionsDef: menu_t,
    pub ReadDef1: menu_t,
    pub ReadDef2: menu_t,
    pub SoundDef: menu_t,
    pub LoadDef: menu_t,
    pub SaveDef: menu_t,
}

impl Default for MMenuDefsHolder {
    fn default() -> Self {
        Self::new()
    }
}

impl MMenuDefsHolder {
    pub fn new() -> Self {
        MMenuDefsHolder {
            MainDef: menu_s {
                numitems: main_end as i16,
                prevMenu: None,
                items: vec![
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_NGAME\0\0\0"),
                        routine: Some(M_NewGame as fn(&mut GameState, i32)),
                        alphaKey: b'n',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_OPTION\0\0"),
                        routine: Some(M_Options as fn(&mut GameState, i32)),
                        alphaKey: b'o',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_LOADG\0\0\0"),
                        routine: Some(M_LoadGame as fn(&mut GameState, i32)),
                        alphaKey: b'l',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_SAVEG\0\0\0"),
                        routine: Some(M_SaveGame as fn(&mut GameState, i32)),
                        alphaKey: b's',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_RDTHIS\0\0"),
                        routine: Some(M_ReadThis as fn(&mut GameState, i32)),
                        alphaKey: b'r',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_QUITG\0\0\0"),
                        routine: Some(M_QuitDOOM as fn(&mut GameState, i32)),
                        alphaKey: b'q',
                    },
                ],
                routine: Some(M_DrawMainMenu as fn(&mut GameState) -> ()),
                x: 97_i16,
                y: 64_i16,
                lastOn: 0_i16,
            },
            EpiDef: menu_s {
                numitems: ep_end as i16,
                prevMenu: Some(MenuId::Main),
                items: vec![
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_EPI1\0\0\0\0"),
                        routine: Some(M_Episode as fn(&mut GameState, i32)),
                        alphaKey: b'k',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_EPI2\0\0\0\0"),
                        routine: Some(M_Episode as fn(&mut GameState, i32)),
                        alphaKey: b't',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_EPI3\0\0\0\0"),
                        routine: Some(M_Episode as fn(&mut GameState, i32)),
                        alphaKey: b'i',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_EPI4\0\0\0\0"),
                        routine: Some(M_Episode as fn(&mut GameState, i32)),
                        alphaKey: b't',
                    },
                ],
                routine: Some(M_DrawEpisode as fn(&mut GameState) -> ()),
                x: 48_i16,
                y: 63_i16,
                lastOn: EpisodeMenu::ep1 as i32 as i16,
            },
            NewDef: menu_s {
                numitems: newg_end as i16,
                prevMenu: Some(MenuId::Epi),
                items: vec![
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_JKILL\0\0\0"),
                        routine: Some(M_ChooseSkill as fn(&mut GameState, i32)),
                        alphaKey: b'i',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_ROUGH\0\0\0"),
                        routine: Some(M_ChooseSkill as fn(&mut GameState, i32)),
                        alphaKey: b'h',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_HURT\0\0\0\0"),
                        routine: Some(M_ChooseSkill as fn(&mut GameState, i32)),
                        alphaKey: b'h',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_ULTRA\0\0\0"),
                        routine: Some(M_ChooseSkill as fn(&mut GameState, i32)),
                        alphaKey: b'u',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_NMARE\0\0\0"),
                        routine: Some(M_ChooseSkill as fn(&mut GameState, i32)),
                        alphaKey: b'n',
                    },
                ],
                routine: Some(M_DrawNewGame as fn(&mut GameState) -> ()),
                x: 48_i16,
                y: 63_i16,
                lastOn: NewGameMenu::hurtme as i32 as i16,
            },
            OptionsDef: menu_s {
                numitems: opt_end as i16,
                prevMenu: Some(MenuId::Main),
                items: vec![
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_ENDGAM\0\0"),
                        routine: Some(M_EndGame as fn(&mut GameState, i32)),
                        alphaKey: b'e',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_MESSG\0\0\0"),
                        routine: Some(M_ChangeMessages as fn(&mut GameState, i32)),
                        alphaKey: b'm',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_DETAIL\0\0"),
                        routine: Some(M_ChangeDetail as fn(&mut GameState, i32)),
                        alphaKey: b'g',
                    },
                    menuitem_t {
                        status: 2_i16,
                        name: FixedCStr(*b"M_SCRNSZ\0\0"),
                        routine: Some(M_SizeDisplay as fn(&mut GameState, i32)),
                        alphaKey: b's',
                    },
                    menuitem_t {
                        status: -1_i32 as i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: None,
                        alphaKey: b'\0',
                    },
                    menuitem_t {
                        status: 2_i16,
                        name: FixedCStr(*b"M_MSENS\0\0\0"),
                        routine: Some(M_ChangeSensitivity as fn(&mut GameState, i32)),
                        alphaKey: b'm',
                    },
                    menuitem_t {
                        status: -1_i32 as i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: None,
                        alphaKey: b'\0',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"M_SVOL\0\0\0\0"),
                        routine: Some(M_Sound as fn(&mut GameState, i32)),
                        alphaKey: b's',
                    },
                ],
                routine: Some(M_DrawOptions as fn(&mut GameState) -> ()),
                x: 60_i16,
                y: 37_i16,
                lastOn: 0_i16,
            },
            ReadDef1: menu_s {
                numitems: read1_end as i16,
                prevMenu: Some(MenuId::Main),
                items: vec![menuitem_t {
                    status: 1_i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: Some(M_ReadThis2 as fn(&mut GameState, i32)),
                    alphaKey: 0_u8,
                }],
                routine: Some(M_DrawReadThis1 as fn(&mut GameState) -> ()),
                x: 280_i16,
                y: 185_i16,
                lastOn: 0_i16,
            },
            ReadDef2: menu_s {
                numitems: read2_end as i16,
                prevMenu: Some(MenuId::Read1),
                items: vec![menuitem_t {
                    status: 1_i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: Some(M_FinishReadThis as fn(&mut GameState, i32)),
                    alphaKey: 0_u8,
                }],
                routine: Some(M_DrawReadThis2 as fn(&mut GameState) -> ()),
                x: 330_i16,
                y: 175_i16,
                lastOn: 0_i16,
            },
            SoundDef: menu_s {
                numitems: sound_end as i16,
                prevMenu: Some(MenuId::Options),
                items: vec![
                    menuitem_t {
                        status: 2_i16,
                        name: FixedCStr(*b"M_SFXVOL\0\0"),
                        routine: Some(M_SfxVol as fn(&mut GameState, i32)),
                        alphaKey: b's',
                    },
                    menuitem_t {
                        status: -1_i32 as i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: None,
                        alphaKey: b'\0',
                    },
                    menuitem_t {
                        status: 2_i16,
                        name: FixedCStr(*b"M_MUSVOL\0\0"),
                        routine: Some(M_MusicVol as fn(&mut GameState, i32)),
                        alphaKey: b'm',
                    },
                    menuitem_t {
                        status: -1_i32 as i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: None,
                        alphaKey: b'\0',
                    },
                ],
                routine: Some(M_DrawSound as fn(&mut GameState) -> ()),
                x: 80_i16,
                y: 64_i16,
                lastOn: 0_i16,
            },
            LoadDef: menu_s {
                numitems: load_end as i16,
                prevMenu: Some(MenuId::Main),
                items: vec![
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(M_LoadSelect as fn(&mut GameState, i32)),
                        alphaKey: b'1',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(M_LoadSelect as fn(&mut GameState, i32)),
                        alphaKey: b'2',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(M_LoadSelect as fn(&mut GameState, i32)),
                        alphaKey: b'3',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(M_LoadSelect as fn(&mut GameState, i32)),
                        alphaKey: b'4',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(M_LoadSelect as fn(&mut GameState, i32)),
                        alphaKey: b'5',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(M_LoadSelect as fn(&mut GameState, i32)),
                        alphaKey: b'6',
                    },
                ],
                routine: Some(M_DrawLoad as fn(&mut GameState) -> ()),
                x: 80_i16,
                y: 54_i16,
                lastOn: 0_i16,
            },
            SaveDef: menu_s {
                numitems: load_end as i16,
                prevMenu: Some(MenuId::Main),
                items: vec![
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(M_SaveSelect as fn(&mut GameState, i32)),
                        alphaKey: b'1',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(M_SaveSelect as fn(&mut GameState, i32)),
                        alphaKey: b'2',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(M_SaveSelect as fn(&mut GameState, i32)),
                        alphaKey: b'3',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(M_SaveSelect as fn(&mut GameState, i32)),
                        alphaKey: b'4',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(M_SaveSelect as fn(&mut GameState, i32)),
                        alphaKey: b'5',
                    },
                    menuitem_t {
                        status: 1_i16,
                        name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                        routine: Some(M_SaveSelect as fn(&mut GameState, i32)),
                        alphaKey: b'6',
                    },
                ],
                routine: Some(M_DrawSave as fn(&mut GameState) -> ()),
                x: 80_i16,
                y: 54_i16,
                lastOn: 0_i16,
            },
        }
    }
}

pub struct MMenuState {
    pub defs: MMenuDefsHolder,
    pub mouseSensitivity: i32,
    pub showMessages: i32,
    pub detailLevel: i32,
    pub screenblocks: i32,
    pub screenSize: i32,
    pub quickSaveSlot: i32,
    pub messageToPrint: i32,
    pub messageString: String,
    pub messx: i32,
    pub messy: i32,
    pub messageLastMenuActive: i32,
    pub messageNeedsInput: bool,
    pub messageRoutine: Option<fn(&mut GameState, i32)>,
    /// True while the pending message is the quit confirmation from `M_QuitDOOM`.
    pub messageIsQuitPrompt: bool,
    pub saveStringEnter: i32,
    pub saveSlot: i32,
    pub saveCharIndex: i32,
    pub saveOldString: String,
    pub inhelpscreens: bool,
    pub menuactive: bool,
    pub savegamestrings: [String; 10],
    pub itemOn: i16,
    pub skullAnimCounter: i16,
    pub whichSkull: i16,
    pub currentMenu: MenuId,
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
        MMenuState {
            defs: MMenuDefsHolder::new(),
            mouseSensitivity: 5,
            showMessages: 1,
            detailLevel: 0,
            screenblocks: 10,
            screenSize: 0,
            quickSaveSlot: 0,
            messageToPrint: 0,
            messageString: String::new(),
            messx: 0,
            messy: 0,
            messageLastMenuActive: 0,
            messageNeedsInput: false,
            messageRoutine: None,
            messageIsQuitPrompt: false,
            saveStringEnter: 0,
            saveSlot: 0,
            saveCharIndex: 0,
            saveOldString: String::new(),
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
            itemOn: 0,
            skullAnimCounter: 0,
            whichSkull: 0,
            currentMenu: MenuId::Main,
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

    pub fn def(&self, id: MenuId) -> &menu_t {
        match id {
            MenuId::Main => &self.defs.MainDef,
            MenuId::Epi => &self.defs.EpiDef,
            MenuId::New => &self.defs.NewDef,
            MenuId::Options => &self.defs.OptionsDef,
            MenuId::Read1 => &self.defs.ReadDef1,
            MenuId::Read2 => &self.defs.ReadDef2,
            MenuId::Sound => &self.defs.SoundDef,
            MenuId::Load => &self.defs.LoadDef,
            MenuId::Save => &self.defs.SaveDef,
        }
    }

    pub fn def_mut(&mut self, id: MenuId) -> &mut menu_t {
        match id {
            MenuId::Main => &mut self.defs.MainDef,
            MenuId::Epi => &mut self.defs.EpiDef,
            MenuId::New => &mut self.defs.NewDef,
            MenuId::Options => &mut self.defs.OptionsDef,
            MenuId::Read1 => &mut self.defs.ReadDef1,
            MenuId::Read2 => &mut self.defs.ReadDef2,
            MenuId::Sound => &mut self.defs.SoundDef,
            MenuId::Load => &mut self.defs.LoadDef,
            MenuId::Save => &mut self.defs.SaveDef,
        }
    }

    pub fn current(&self) -> &menu_t {
        self.def(self.currentMenu)
    }

    pub fn current_mut(&mut self) -> &mut menu_t {
        self.def_mut(self.currentMenu)
    }
}

#[derive(Copy, Clone)]
pub struct menuitem_t {
    pub status: i16,
    pub name: FixedCStr<10>,
    pub routine: Option<fn(&mut GameState, i32)>,
    pub alphaKey: u8,
}
pub type menu_t = menu_s;
#[derive(Clone)]
pub struct menu_s {
    pub numitems: i16,
    pub prevMenu: Option<MenuId>,
    pub items: Vec<menuitem_t>,
    pub routine: Option<fn(&mut GameState) -> ()>,
    pub x: i16,
    pub y: i16,
    pub lastOn: i16,
}
pub const read2_end: i32 = 1;
pub const read1_end: i32 = 1;
pub const load_end: i32 = 6;
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; discriminants must stay
pub enum OptionsMenu {
    endgame = 0,
    messages = 1,
    detail = 2,
    scrnsize = 3,
    option_empty1 = 4,
    mousesens = 5,
    option_empty2 = 6,
    soundvol = 7,
}
pub const opt_end: i32 = 8;
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; discriminants must stay
pub enum SoundMenu {
    sfx_vol = 0,
    sfx_empty1 = 1,
    music_vol = 2,
    sfx_empty2 = 3,
}
pub const sound_end: i32 = 4;
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; discriminants must stay
pub enum EpisodeMenu {
    ep1 = 0,
    ep2 = 1,
    ep3 = 2,
    ep4 = 3,
}
pub const ep_end: i32 = 4;
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; discriminants must stay
pub enum NewGameMenu {
    killthings = 0,
    toorough = 1,
    hurtme = 2,
    violence = 3,
    nightmare = 4,
}
pub const newg_end: i32 = 5;
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; discriminants must stay
pub enum MainMenu {
    newgame = 0,
    options = 1,
    loadgame = 2,
    savegame = 3,
    readthis = 4,
    quitdoom = 5,
}
pub const main_end: i32 = 6;
pub const KEY_NUMLOCK: i32 = 0x80 + 0x45_i32;
pub const GAMMALVL0: &str = "Gamma correction OFF\0";
pub const GAMMALVL1: &str = "Gamma correction level 1\0";
pub const GAMMALVL2: &str = "Gamma correction level 2\0";
pub const GAMMALVL3: &str = "Gamma correction level 3\0";
pub const GAMMALVL4: &str = "Gamma correction level 4\0";
pub const EMPTYSTRING: &str = "empty slot\0";
pub const NUM_QUITMESSAGES: i32 = 8;
pub const SAVESTRINGSIZE: i32 = 24;
pub static gammamsg: [&str; 5] = [GAMMALVL0, GAMMALVL1, GAMMALVL2, GAMMALVL3, GAMMALVL4];
pub const SKULLXOFF: i32 = -32_i32;
pub const LINEHEIGHT: i32 = 16;
pub static skullName: [&str; 2] = ["M_SKULL1", "M_SKULL2"];
pub fn M_ReadSaveStrings(state: &mut GameState) {
    let mut i: i32;
    i = 0_i32;
    while i < load_end {
        let savegame_file = P_SaveGameFile(state, i);
        match std::fs::File::open(&savegame_file) {
            Err(_) => {
                state.m_menu.savegamestrings[i as usize] =
                    EMPTYSTRING.trim_end_matches('\0').to_string();
                state.m_menu.defs.LoadDef.items[i as usize].status = 0_i16;
            }
            Ok(mut handle) => {
                let mut buf: [u8; SAVESTRINGSIZE as usize] = [0; SAVESTRINGSIZE as usize];
                let _ = std::io::Read::read(&mut handle, &mut buf);
                let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
                state.m_menu.savegamestrings[i as usize] =
                    String::from_utf8_lossy(&buf[..len]).into_owned();
                state.m_menu.defs.LoadDef.items[i as usize].status = 1_i16;
            }
        }
        i += 1;
    }
}
pub fn M_DrawLoad(state: &mut GameState) {
    let mut i: i32;
    let __wcache890_24 = V_CachePatchName(state, "M_LOADG");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, 72_i32, 28_i32, &__wcache890_24);
    i = 0_i32;
    while i < load_end {
        let loaddef_x = state.m_menu.defs.LoadDef.x as i32;
        let loaddef_y = state.m_menu.defs.LoadDef.y as i32 + LINEHEIGHT * i;
        M_DrawSaveLoadBorder(state, loaddef_x, loaddef_y);
        let savestr = state.m_menu.savegamestrings[i as usize].clone();
        M_WriteText(state, loaddef_x, loaddef_y, &savestr);
        i += 1;
    }
}
pub fn M_DrawSaveLoadBorder(state: &mut GameState, mut x: i32, y: i32) {
    let mut i: i32;
    let __wcache908_23 = V_CachePatchName(state, "M_LSLEFT");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, x - 8_i32, y + 7_i32, &__wcache908_23);
    i = 0_i32;
    while i < 24_i32 {
        let __wcache916_22 = V_CachePatchName(state, "M_LSCNTR");
        let dest_screen = Screen::Video;
        V_DrawPatchDirect(state, dest_screen, x, y + 7_i32, &__wcache916_22);
        x += 8_i32;
        i += 1;
    }
    let __wcache925_21 = V_CachePatchName(state, "M_LSRGHT");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, x, y + 7_i32, &__wcache925_21);
}
pub fn M_LoadSelect(state: &mut GameState, choice: i32) {
    let savegame_file = P_SaveGameFile(state, choice);
    G_LoadGame(state, &savegame_file);
    M_ClearMenus(state);
}
pub fn M_LoadGame(state: &mut GameState, _choice: i32) {
    if state.g_game.netgame {
        M_StartMessage(
            state,
            "you can't do load while in a net game!\n\npress a key.",
            None,
            false,
        );
        return;
    }
    let menudef = MenuId::Load;
    M_SetupNextMenu(state, menudef);
    M_ReadSaveStrings(state);
}
pub fn M_DrawSave(state: &mut GameState) {
    let mut i: i32;
    let __wcache961_20 = V_CachePatchName(state, "M_SAVEG");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, 72_i32, 28_i32, &__wcache961_20);
    i = 0_i32;
    while i < load_end {
        let loaddef_x = state.m_menu.defs.LoadDef.x as i32;
        let loaddef_y = state.m_menu.defs.LoadDef.y as i32 + LINEHEIGHT * i;
        M_DrawSaveLoadBorder(state, loaddef_x, loaddef_y);
        let savestr = state.m_menu.savegamestrings[i as usize].clone();
        M_WriteText(state, loaddef_x, loaddef_y, &savestr);
        i += 1;
    }
    if state.m_menu.saveStringEnter != 0 {
        let savestr = state.m_menu.savegamestrings[state.m_menu.saveSlot as usize].clone();
        i = M_StringWidth(state, &savestr);
        let text_x = state.m_menu.defs.LoadDef.x as i32 + i;
        let text_y = state.m_menu.defs.LoadDef.y as i32 + LINEHEIGHT * state.m_menu.saveSlot;
        M_WriteText(state, text_x, text_y, "_");
    }
}
pub fn M_DoSave(state: &mut GameState, slot: i32) {
    let savegame_name = state.m_menu.savegamestrings[slot as usize].clone();
    G_SaveGame(state, slot, &savegame_name);
    M_ClearMenus(state);
    if state.m_menu.quickSaveSlot == -2_i32 {
        state.m_menu.quickSaveSlot = slot;
    }
}
pub fn M_SaveSelect(state: &mut GameState, choice: i32) {
    state.m_menu.saveStringEnter = 1_i32;
    state.m_menu.saveSlot = choice;
    state.m_menu.saveOldString = state.m_menu.savegamestrings[choice as usize].clone();
    if state.m_menu.savegamestrings[choice as usize] == EMPTYSTRING.trim_end_matches('\0') {
        state.m_menu.savegamestrings[choice as usize].clear();
    }
    state.m_menu.saveCharIndex = state.m_menu.savegamestrings[choice as usize].len() as i32;
}
pub fn M_SaveGame(state: &mut GameState, _choice: i32) {
    if !state.g_game.usergame {
        M_StartMessage(
            state,
            "you can't save if you aren't playing!\n\npress a key.",
            None,
            false,
        );
        return;
    }
    if state.g_game.gamestate != GameScreenState::GS_LEVEL {
        return;
    }
    let menudef = MenuId::Save;
    M_SetupNextMenu(state, menudef);
    M_ReadSaveStrings(state);
}
pub fn M_QuickSaveResponse(state: &mut GameState, key: i32) {
    if key == state.m_controls.key_menu_confirm {
        let quick_save_slot = state.m_menu.quickSaveSlot;
        M_DoSave(state, quick_save_slot);
        S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchx as i32);
    }
}
pub fn M_QuickSave(state: &mut GameState) {
    if !state.g_game.usergame {
        S_StartSound(state, SoundOrigin::None, SfxName::sfx_oof as i32);
        return;
    }
    if state.g_game.gamestate != GameScreenState::GS_LEVEL {
        return;
    }
    if state.m_menu.quickSaveSlot < 0_i32 {
        M_StartControlPanel(state);
        M_ReadSaveStrings(state);
        let menudef = MenuId::Save;
        M_SetupNextMenu(state, menudef);
        state.m_menu.quickSaveSlot = -2_i32;
        return;
    }
    let msg = format!(
        "quicksave over your game named\n\n'{}'?\n\npress y or n.",
        state.m_menu.savegamestrings[state.m_menu.quickSaveSlot as usize],
    );
    let routine = Some(M_QuickSaveResponse as fn(&mut GameState, i32));
    M_StartMessage(state, &msg, routine, true);
}
pub fn M_QuickLoadResponse(state: &mut GameState, key: i32) {
    if key == state.m_controls.key_menu_confirm {
        let quick_save_slot = state.m_menu.quickSaveSlot;
        M_LoadSelect(state, quick_save_slot);
        S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchx as i32);
    }
}
pub fn M_QuickLoad(state: &mut GameState) {
    if state.g_game.netgame {
        M_StartMessage(
            state,
            "you can't quickload during a netgame!\n\npress a key.",
            None,
            false,
        );
        return;
    }
    if state.m_menu.quickSaveSlot < 0_i32 {
        M_StartMessage(
            state,
            "you haven't picked a quicksave slot yet!\n\npress a key.",
            None,
            false,
        );
        return;
    }
    let msg = format!(
        "do you want to quickload the game named\n\n'{}'?\n\npress y or n.",
        state.m_menu.savegamestrings[state.m_menu.quickSaveSlot as usize],
    );
    let routine = Some(M_QuickLoadResponse as fn(&mut GameState, i32));
    M_StartMessage(state, &msg, routine, true);
}
pub fn M_DrawReadThis1(state: &mut GameState) {
    let lumpname: &str;
    let mut skullx: i32 = 330_i32;
    let mut skully: i32 = 175_i32;
    state.m_menu.inhelpscreens = true;
    match state.doomstat.gameversion as u32 {
        1..=5 => {
            if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
                lumpname = "HELP";
                skullx = 330_i32;
                skully = 165_i32;
            } else {
                lumpname = "HELP2";
                skullx = 280_i32;
                skully = 185_i32;
            }
        }
        6 | 9 => {
            lumpname = "HELP1";
        }
        7 | 8 => {
            lumpname = "HELP";
        }
        _ => {
            I_Error("Unhandled game version");
        }
    }
    let __wcache1158_19 = V_CachePatchName(state, lumpname);
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, 0_i32, 0_i32, &__wcache1158_19);
    state.m_menu.defs.ReadDef1.x = skullx as i16;
    state.m_menu.defs.ReadDef1.y = skully as i16;
}
pub fn M_DrawReadThis2(state: &mut GameState) {
    state.m_menu.inhelpscreens = true;
    let __wcache1170_18 = V_CachePatchName(state, "HELP1");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, 0_i32, 0_i32, &__wcache1170_18);
}
pub fn M_DrawSound(state: &mut GameState) {
    let __wcache1179_17 = V_CachePatchName(state, "M_SVOL");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, 60_i32, 38_i32, &__wcache1179_17);
    let (x, y, vol) = (
        state.m_menu.defs.SoundDef.x as i32,
        state.m_menu.defs.SoundDef.y as i32 + LINEHEIGHT * (SoundMenu::sfx_vol as i32 + 1_i32),
        state.s_sound.sfxVolume,
    );
    M_DrawThermo(state, x, y, 16_i32, vol);
    let (x, y, vol) = (
        state.m_menu.defs.SoundDef.x as i32,
        state.m_menu.defs.SoundDef.y as i32 + LINEHEIGHT * (SoundMenu::music_vol as i32 + 1_i32),
        state.s_sound.musicVolume,
    );
    M_DrawThermo(state, x, y, 16_i32, vol);
}
pub fn M_Sound(state: &mut GameState, _choice: i32) {
    let menudef = MenuId::Sound;
    M_SetupNextMenu(state, menudef);
}
pub fn M_SfxVol(state: &mut GameState, choice: i32) {
    match choice {
        0 => {
            if state.s_sound.sfxVolume != 0 {
                state.s_sound.sfxVolume -= 1;
            }
        }
        1 if state.s_sound.sfxVolume < 15_i32 => {
            state.s_sound.sfxVolume += 1;
        }
        _ => {}
    }
    let sfx_volume = state.s_sound.sfxVolume * 8_i32;
    S_SetSfxVolume(state, sfx_volume);
}
pub fn M_MusicVol(state: &mut GameState, choice: i32) {
    match choice {
        0 => {
            if state.s_sound.musicVolume != 0 {
                state.s_sound.musicVolume -= 1;
            }
        }
        1 if state.s_sound.musicVolume < 15_i32 => {
            state.s_sound.musicVolume += 1;
        }
        _ => {}
    }
    let music_volume = state.s_sound.musicVolume * 8_i32;
    S_SetMusicVolume(state, music_volume);
}
pub fn M_DrawMainMenu(state: &mut GameState) {
    let __wcache1241_16 = V_CachePatchName(state, "M_DOOM");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, 94_i32, 2_i32, &__wcache1241_16);
}
pub fn M_DrawNewGame(state: &mut GameState) {
    let __wcache1250_15 = V_CachePatchName(state, "M_NEWG");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, 96_i32, 14_i32, &__wcache1250_15);
    let __wcache1256_14 = V_CachePatchName(state, "M_SKILL");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, 54_i32, 38_i32, &__wcache1256_14);
}
pub fn M_NewGame(state: &mut GameState, _choice: i32) {
    if state.g_game.netgame && !state.g_game.demoplayback {
        M_StartMessage(
            state,
            "you can't start a new game\nwhile in a network game.\n\npress a key.",
            None,
            false,
        );
        return;
    }
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32
        || state.doomstat.gameversion == GameVersion::chex
    {
        let menudef = MenuId::New;
        M_SetupNextMenu(state, menudef);
    } else {
        let menudef = MenuId::Epi;
        M_SetupNextMenu(state, menudef);
    };
}
pub fn M_DrawEpisode(state: &mut GameState) {
    let __wcache1286_13 = V_CachePatchName(state, "M_EPISOD");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, 54_i32, 38_i32, &__wcache1286_13);
}
pub fn M_VerifyNightmare(state: &mut GameState, key: i32) {
    if key != state.m_controls.key_menu_confirm {
        return;
    }
    G_DeferedInitNew(
        state,
        skill_from_raw(NewGameMenu::nightmare as i32),
        state.m_menu.epi + 1_i32,
        1_i32,
    );
    M_ClearMenus(state);
}
pub fn M_ChooseSkill(state: &mut GameState, choice: i32) {
    if choice == NewGameMenu::nightmare as i32 {
        M_StartMessage(
            state,
            "are you sure? this skill level\nisn't even remotely fair.\n\npress y or n.",
            Some(M_VerifyNightmare as fn(&mut GameState, i32)),
            true,
        );
        return;
    }
    G_DeferedInitNew(
        state,
        skill_from_raw(choice),
        state.m_menu.epi + 1_i32,
        1_i32,
    );
    M_ClearMenus(state);
}
pub fn M_Episode(state: &mut GameState, mut choice: i32) {
    if state.doomstat.gamemode as u32 == GameMode_t::shareware as i32 as u32 && choice != 0 {
        M_StartMessage(
            state,
            "this is the shareware version of doom.\n\n\
                        you need to order the entire trilogy.\n\n\
                        press a key.",
            None,
            false,
        );
        let menudef = MenuId::Read1;
        M_SetupNextMenu(state, menudef);
        return;
    }
    if state.doomstat.gamemode as u32 == GameMode_t::registered as i32 as u32 && choice > 2_i32 {
        eprintln!("M_Episode: 4th episode requires UltimateDOOM");
        choice = 0_i32;
    }
    state.m_menu.epi = choice;
    let menudef = MenuId::New;
    M_SetupNextMenu(state, menudef);
}
static DETAIL_NAMES: [&str; 2] = ["M_GDHIGH", "M_GDLOW"];
static MSG_NAMES: [&str; 2] = ["M_MSGOFF", "M_MSGON"];
pub fn M_DrawOptions(state: &mut GameState) {
    let __wcache1358_12 = V_CachePatchName(state, "M_OPTTTL");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, 108_i32, 15_i32, &__wcache1358_12);
    let __wcache1364_11 = V_CachePatchName(state, DETAIL_NAMES[state.m_menu.detailLevel as usize]);
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(
        state,
        dest_screen,
        state.m_menu.defs.OptionsDef.x as i32 + 175_i32,
        state.m_menu.defs.OptionsDef.y as i32 + LINEHEIGHT * OptionsMenu::detail as i32,
        &__wcache1364_11,
    );
    let __wcache1373_10 = V_CachePatchName(state, MSG_NAMES[state.m_menu.showMessages as usize]);
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(
        state,
        dest_screen,
        state.m_menu.defs.OptionsDef.x as i32 + 120_i32,
        state.m_menu.defs.OptionsDef.y as i32 + LINEHEIGHT * OptionsMenu::messages as i32,
        &__wcache1373_10,
    );
    let (x, y, sens) = (
        state.m_menu.defs.OptionsDef.x as i32,
        state.m_menu.defs.OptionsDef.y as i32
            + LINEHEIGHT * (OptionsMenu::mousesens as i32 + 1_i32),
        state.m_menu.mouseSensitivity,
    );
    M_DrawThermo(state, x, y, 10_i32, sens);
    let (x, y, sz) = (
        state.m_menu.defs.OptionsDef.x as i32,
        state.m_menu.defs.OptionsDef.y as i32 + LINEHEIGHT * (OptionsMenu::scrnsize as i32 + 1_i32),
        state.m_menu.screenSize,
    );
    M_DrawThermo(state, x, y, 9_i32, sz);
}
pub fn M_Options(state: &mut GameState, _choice: i32) {
    let menudef = MenuId::Options;
    M_SetupNextMenu(state, menudef);
}
pub fn M_ChangeMessages(state: &mut GameState, _choice: i32) {
    state.m_menu.showMessages = 1_i32 - state.m_menu.showMessages;
    if state.m_menu.showMessages == 0 {
        state.g_game.players[state.g_game.consoleplayer as usize].message =
            Some("Messages OFF".to_string());
    } else {
        state.g_game.players[state.g_game.consoleplayer as usize].message =
            Some("Messages ON".to_string());
    }
    state.hu_stuff.message_dontfuckwithme = true;
}
pub fn M_EndGameResponse(state: &mut GameState, key: i32) {
    if key != state.m_controls.key_menu_confirm {
        return;
    }
    let item_on = state.m_menu.itemOn;
    state.m_menu.current_mut().lastOn = item_on;
    M_ClearMenus(state);
    D_StartTitle(state);
}
pub fn M_EndGame(state: &mut GameState, _choice: i32) {
    if !state.g_game.usergame {
        S_StartSound(state, SoundOrigin::None, SfxName::sfx_oof as i32);
        return;
    }
    if state.g_game.netgame {
        M_StartMessage(
            state,
            "you can't end a netgame!\n\npress a key.",
            None,
            false,
        );
        return;
    }
    M_StartMessage(
        state,
        "are you sure you want to end the game?\n\npress y or n.",
        Some(M_EndGameResponse as fn(&mut GameState, i32)),
        true,
    );
}
pub fn M_ReadThis(state: &mut GameState, _choice: i32) {
    let menudef = MenuId::Read1;
    M_SetupNextMenu(state, menudef);
}
pub fn M_ReadThis2(state: &mut GameState, _choice: i32) {
    if state.doomstat.gameversion.below_1_9()
        && state.doomstat.gamemode as u32 != GameMode_t::commercial as i32 as u32
    {
        let menudef = MenuId::Read2;
        M_SetupNextMenu(state, menudef);
    } else {
        M_FinishReadThis(state, 0_i32);
    };
}
pub fn M_FinishReadThis(state: &mut GameState, _choice: i32) {
    let menudef = MenuId::Main;
    M_SetupNextMenu(state, menudef);
}
pub static quitsounds: [i32; 8] = [
    SfxName::sfx_pldeth as i32,
    SfxName::sfx_dmpain as i32,
    SfxName::sfx_popain as i32,
    SfxName::sfx_slop as i32,
    SfxName::sfx_telept as i32,
    SfxName::sfx_posit1 as i32,
    SfxName::sfx_posit3 as i32,
    SfxName::sfx_sgtatk as i32,
];
pub static quitsounds2: [i32; 8] = [
    SfxName::sfx_vilact as i32,
    SfxName::sfx_getpow as i32,
    SfxName::sfx_boscub as i32,
    SfxName::sfx_slop as i32,
    SfxName::sfx_skeswg as i32,
    SfxName::sfx_kntdth as i32,
    SfxName::sfx_bspact as i32,
    SfxName::sfx_sgtatk as i32,
];
pub fn M_QuitResponse(state: &mut GameState, key: i32) {
    if key != state.m_controls.key_menu_confirm {
        return;
    }
    if !state.g_game.netgame {
        if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
            S_StartSound(
                state,
                SoundOrigin::None,
                quitsounds2[(state.d_loop.gametic >> 2_i32 & 7_i32) as usize],
            );
        } else {
            S_StartSound(
                state,
                SoundOrigin::None,
                quitsounds[(state.d_loop.gametic >> 2_i32 & 7_i32) as usize],
            );
        }
    }
    I_Quit(state);
}
fn M_SelectEndMessage(state: &mut GameState) -> &'static str {
    let endmsg: &'static [&'static str; 8] =
        if (if state.doomstat.gamemission as u32 == GameMission_t::pack_chex as i32 as u32 {
            GameMission_t::doom as i32 as u32
        } else if state.doomstat.gamemission as u32 == GameMission_t::pack_hacx as i32 as u32 {
            GameMission_t::doom2 as i32 as u32
        } else {
            state.doomstat.gamemission as u32
        }) == GameMission_t::doom as i32 as u32
        {
            &doom1_endmsg
        } else {
            &doom2_endmsg
        };
    endmsg[(state.d_loop.gametic % NUM_QUITMESSAGES) as usize]
}
pub fn M_QuitDOOM(state: &mut GameState, _choice: i32) {
    let msg = format!("{}\n\n(press y to quit to dos.)", M_SelectEndMessage(state));
    let routine = Some(M_QuitResponse as fn(&mut GameState, i32));
    M_StartMessage(state, &msg, routine, true);
    state.m_menu.messageIsQuitPrompt = true;
}
pub fn M_ChangeSensitivity(state: &mut GameState, choice: i32) {
    match choice {
        0 => {
            if state.m_menu.mouseSensitivity != 0 {
                state.m_menu.mouseSensitivity -= 1;
            }
        }
        1 if state.m_menu.mouseSensitivity < 9_i32 => {
            state.m_menu.mouseSensitivity += 1;
        }
        _ => {}
    };
}
pub fn M_ChangeDetail(state: &mut GameState, _choice: i32) {
    state.m_menu.detailLevel = 1_i32 - state.m_menu.detailLevel;
    let (screenblocks, detail_level) = (state.m_menu.screenblocks, state.m_menu.detailLevel);
    R_SetViewSize(state, screenblocks, detail_level);
    if state.m_menu.detailLevel == 0 {
        state.g_game.players[state.g_game.consoleplayer as usize].message =
            Some("High detail".to_string());
    } else {
        state.g_game.players[state.g_game.consoleplayer as usize].message =
            Some("Low detail".to_string());
    };
}
pub fn M_SizeDisplay(state: &mut GameState, choice: i32) {
    match choice {
        0 => {
            if state.m_menu.screenSize > 0_i32 {
                state.m_menu.screenblocks -= 1;
                state.m_menu.screenSize -= 1;
            }
        }
        1 if state.m_menu.screenSize < 8_i32 => {
            state.m_menu.screenblocks += 1;
            state.m_menu.screenSize += 1;
        }
        _ => {}
    }
    let (screenblocks, detail_level) = (state.m_menu.screenblocks, state.m_menu.detailLevel);
    R_SetViewSize(state, screenblocks, detail_level);
}
pub fn M_DrawThermo(
    state: &mut GameState,
    x: i32,
    y: i32,
    thermWidth: i32,
    thermDot: i32,
) {
    let mut xx: i32;
    let mut i: i32;
    xx = x;
    let __wcache1619_9 = V_CachePatchName(state, "M_THERML");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, xx, y, &__wcache1619_9);
    xx += 8_i32;
    i = 0_i32;
    while i < thermWidth {
        let __wcache1628_8 = V_CachePatchName(state, "M_THERMM");
        let dest_screen = Screen::Video;
        V_DrawPatchDirect(state, dest_screen, xx, y, &__wcache1628_8);
        xx += 8_i32;
        i += 1;
    }
    let __wcache1637_7 = V_CachePatchName(state, "M_THERMR");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(state, dest_screen, xx, y, &__wcache1637_7);
    let __wcache1643_6 = V_CachePatchName(state, "M_THERMO");
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(
        state,
        dest_screen,
        x + 8_i32 + thermDot * 8_i32,
        y,
        &__wcache1643_6,
    );
}
pub fn M_StartMessage(
    state: &mut GameState,
    string: &str,
    routine: Option<fn(&mut GameState, i32)>,
    input: bool,
) {
    state.m_menu.messageLastMenuActive = state.m_menu.menuactive as i32;
    state.m_menu.messageToPrint = 1_i32;
    state.m_menu.messageString = string.to_string();
    state.m_menu.messageRoutine = routine;
    state.m_menu.messageIsQuitPrompt = false;
    state.m_menu.messageNeedsInput = input;
    state.m_menu.menuactive = true;
}
pub fn M_StringWidth(state: &mut GameState, string: &str) -> i32 {
    let mut w: i32 = 0_i32;
    let mut c: i32;
    for b in string.bytes() {
        c = b.to_ascii_uppercase() as i32 - HU_FONTSTART;
        if !(0_i32..HU_FONTSIZE).contains(&c) {
            w += 4_i32;
        } else {
            let font_patch = V_CachePatchNum(state, state.hu_stuff.hu_font[c as usize]);
            w += font_patch.width();
        }
    }
    w
}
pub fn M_StringHeight(state: &mut GameState, string: &str) -> i32 {
    let mut h: i32;
    let height: i32 = V_CachePatchNum(state, state.hu_stuff.hu_font[0]).height();
    h = height;
    for b in string.bytes() {
        if b == b'\n' {
            h += height;
        }
    }
    h
}
pub fn M_WriteText(state: &mut GameState, x: i32, y: i32, string: &str) {
    let mut w: i32;
    let mut c: i32;
    let mut cx: i32;
    let mut cy: i32;
    cx = x;
    cy = y;
    'outer: for b in string.bytes() {
        c = b as i32;
        if c == '\n' as i32 {
            cx = x;
            cy += 12_i32;
        } else {
            c = (c as u8).to_ascii_uppercase() as i32 - HU_FONTSTART;
            if !(0_i32..HU_FONTSIZE).contains(&c) {
                cx += 4_i32;
            } else {
                let font_patch = V_CachePatchNum(state, state.hu_stuff.hu_font[c as usize]);
                w = font_patch.width();
                if cx + w > SCREENWIDTH {
                    break 'outer;
                }
                let dest_screen = Screen::Video;
                V_DrawPatchDirect(state, dest_screen, cx, cy, &font_patch);
                cx += w;
            }
        }
    }
}
fn IsNullKey(key: i32) -> bool {
    key == KEY_PAUSE || key == KEY_CAPSLOCK || key == KEY_SCRLCK || key == KEY_NUMLOCK
}
pub fn M_Responder(state: &mut GameState, ev: &mut event_t) -> bool {
    let mut ch: i32;
    let mut key: i32;
    let mut i: i32;
    if state.g_game.testcontrols {
        if ev.type_0 == EvType::ev_quit
            || ev.type_0 == EvType::ev_keydown
                && (ev.data1 == state.m_controls.key_menu_activate
                    || ev.data1 == state.m_controls.key_menu_quit)
        {
            I_Quit(state);
            return true;
        }
        return false;
    }
    if ev.type_0 == EvType::ev_quit {
        if state.m_menu.menuactive
            && state.m_menu.messageToPrint != 0
            && state.m_menu.messageIsQuitPrompt
        {
            let key_menu_confirm = state.m_controls.key_menu_confirm;
            M_QuitResponse(state, key_menu_confirm);
        } else {
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
            M_QuitDOOM(state, 0_i32);
        }
        return true;
    }
    ch = 0_i32;
    key = -1_i32;
    if ev.type_0 == EvType::ev_joystick && state.m_menu.responder_joywait < I_GetTime(state) {
        if ev.data3 < 0_i32 {
            key = state.m_controls.key_menu_up;
            state.m_menu.responder_joywait = I_GetTime(state) + 5_i32;
        } else if ev.data3 > 0_i32 {
            key = state.m_controls.key_menu_down;
            state.m_menu.responder_joywait = I_GetTime(state) + 5_i32;
        }
        if ev.data2 < 0_i32 {
            key = state.m_controls.key_menu_left;
            state.m_menu.responder_joywait = I_GetTime(state) + 2_i32;
        } else if ev.data2 > 0_i32 {
            key = state.m_controls.key_menu_right;
            state.m_menu.responder_joywait = I_GetTime(state) + 2_i32;
        }
        if ev.data1 & 1_i32 != 0 {
            key = state.m_controls.key_menu_forward;
            state.m_menu.responder_joywait = I_GetTime(state) + 5_i32;
        }
        if ev.data1 & 2_i32 != 0 {
            key = state.m_controls.key_menu_back;
            state.m_menu.responder_joywait = I_GetTime(state) + 5_i32;
        }
        if state.m_controls.joybmenu >= 0_i32
            && ev.data1 & 1_i32 << state.m_controls.joybmenu != 0_i32
        {
            key = state.m_controls.key_menu_activate;
            state.m_menu.responder_joywait = I_GetTime(state) + 5_i32;
        }
    } else if ev.type_0 == EvType::ev_mouse && state.m_menu.responder_mousewait < I_GetTime(state) {
        state.m_menu.responder_mousey += ev.data3;
        if state.m_menu.responder_mousey < state.m_menu.responder_lasty - 30_i32 {
            key = state.m_controls.key_menu_down;
            state.m_menu.responder_mousewait = I_GetTime(state) + 5_i32;
            state.m_menu.responder_lasty -= 30_i32;
            state.m_menu.responder_mousey = state.m_menu.responder_lasty;
        } else if state.m_menu.responder_mousey > state.m_menu.responder_lasty + 30_i32 {
            key = state.m_controls.key_menu_up;
            state.m_menu.responder_mousewait = I_GetTime(state) + 5_i32;
            state.m_menu.responder_lasty += 30_i32;
            state.m_menu.responder_mousey = state.m_menu.responder_lasty;
        }
        state.m_menu.responder_mousex += ev.data2;
        if state.m_menu.responder_mousex < state.m_menu.responder_lastx - 30_i32 {
            key = state.m_controls.key_menu_left;
            state.m_menu.responder_mousewait = I_GetTime(state) + 5_i32;
            state.m_menu.responder_lastx -= 30_i32;
            state.m_menu.responder_mousex = state.m_menu.responder_lastx;
        } else if state.m_menu.responder_mousex > state.m_menu.responder_lastx + 30_i32 {
            key = state.m_controls.key_menu_right;
            state.m_menu.responder_mousewait = I_GetTime(state) + 5_i32;
            state.m_menu.responder_lastx += 30_i32;
            state.m_menu.responder_mousex = state.m_menu.responder_lastx;
        }
        if ev.data1 & 1_i32 != 0 {
            key = state.m_controls.key_menu_forward;
            state.m_menu.responder_mousewait = I_GetTime(state) + 15_i32;
        }
        if ev.data1 & 2_i32 != 0 {
            key = state.m_controls.key_menu_back;
            state.m_menu.responder_mousewait = I_GetTime(state) + 15_i32;
        }
    } else if ev.type_0 == EvType::ev_keydown {
        key = ev.data1;
        ch = ev.data2;
    }
    if key == -1_i32 {
        return false;
    }
    if state.m_menu.saveStringEnter != 0 {
        match key {
            KEY_BACKSPACE => {
                if state.m_menu.saveCharIndex > 0_i32 {
                    state.m_menu.saveCharIndex -= 1;
                    state.m_menu.savegamestrings[state.m_menu.saveSlot as usize]
                        .truncate(state.m_menu.saveCharIndex as usize);
                }
            }
            KEY_ESCAPE => {
                state.m_menu.saveStringEnter = 0_i32;
                state.m_menu.savegamestrings[state.m_menu.saveSlot as usize] =
                    state.m_menu.saveOldString.clone();
            }
            KEY_ENTER => {
                state.m_menu.saveStringEnter = 0_i32;
                if !state.m_menu.savegamestrings[state.m_menu.saveSlot as usize].is_empty() {
                    let save_slot = state.m_menu.saveSlot;
                    M_DoSave(state, save_slot);
                }
            }
            _ => {
                if state.i_input.vanilla_keyboard_mapping != 0 {
                    ch = key;
                }
                ch = (ch as u8).to_ascii_uppercase() as i32;
                if !(ch != ' ' as i32
                    && (ch - HU_FONTSTART < 0_i32 || ch - HU_FONTSTART >= HU_FONTSIZE))
                {
                    let savestr =
                        state.m_menu.savegamestrings[state.m_menu.saveSlot as usize].clone();
                    if (32_i32..=127_i32).contains(&ch)
                        && state.m_menu.saveCharIndex < SAVESTRINGSIZE - 1_i32
                        && M_StringWidth(state, &savestr) < (SAVESTRINGSIZE - 2_i32) * 8_i32
                    {
                        state.m_menu.saveCharIndex += 1;
                        state.m_menu.savegamestrings[state.m_menu.saveSlot as usize]
                            .push(ch as u8 as char);
                    }
                }
            }
        }
        return true;
    }
    if state.m_menu.messageToPrint != 0 {
        if state.m_menu.messageNeedsInput
            && key != ' ' as i32
            && key != KEY_ESCAPE
            && key != state.m_controls.key_menu_confirm
            && key != state.m_controls.key_menu_abort
        {
            return false;
        }
        state.m_menu.menuactive = state.m_menu.messageLastMenuActive != 0;
        state.m_menu.messageToPrint = 0_i32;
        if state.m_menu.messageRoutine.is_some() {
            state
                .m_menu
                .messageRoutine
                .expect("non-null function pointer")(state, key);
        }
        state.m_menu.menuactive = false;
        S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchx as i32);
        return true;
    }
    if state.d_main.devparm && key == state.m_controls.key_menu_help
        || key != 0_i32 && key == state.m_controls.key_menu_screenshot
    {
        G_ScreenShot(state);
        return true;
    }
    if !state.m_menu.menuactive {
        if key == state.m_controls.key_menu_decscreen {
            if state.am_map.automapactive || state.hu_stuff.chat_on {
                return false;
            }
            M_SizeDisplay(state, 0_i32);
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_stnmov as i32);
            return true;
        } else if key == state.m_controls.key_menu_incscreen {
            if state.am_map.automapactive || state.hu_stuff.chat_on {
                return false;
            }
            M_SizeDisplay(state, 1_i32);
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_stnmov as i32);
            return true;
        } else if key == state.m_controls.key_menu_help {
            M_StartControlPanel(state);
            if state.doomstat.gamemode as u32 == GameMode_t::retail as i32 as u32 {
                state.m_menu.currentMenu = MenuId::Read2;
            } else {
                state.m_menu.currentMenu = MenuId::Read1;
            }
            state.m_menu.itemOn = 0_i16;
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
            return true;
        } else if key == state.m_controls.key_menu_save {
            M_StartControlPanel(state);
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
            M_SaveGame(state, 0_i32);
            return true;
        } else if key == state.m_controls.key_menu_load {
            M_StartControlPanel(state);
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
            M_LoadGame(state, 0_i32);
            return true;
        } else if key == state.m_controls.key_menu_volume {
            M_StartControlPanel(state);
            state.m_menu.currentMenu = MenuId::Sound;
            state.m_menu.itemOn = SoundMenu::sfx_vol as i32 as i16;
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
            return true;
        } else if key == state.m_controls.key_menu_detail {
            M_ChangeDetail(state, 0_i32);
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
            return true;
        } else if key == state.m_controls.key_menu_qsave {
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
            M_QuickSave(state);
            return true;
        } else if key == state.m_controls.key_menu_endgame {
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
            M_EndGame(state, 0_i32);
            return true;
        } else if key == state.m_controls.key_menu_messages {
            M_ChangeMessages(state, 0_i32);
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
            return true;
        } else if key == state.m_controls.key_menu_qload {
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
            M_QuickLoad(state);
            return true;
        } else if key == state.m_controls.key_menu_quit {
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
            M_QuitDOOM(state, 0_i32);
            return true;
        } else if key == state.m_controls.key_menu_gamma {
            state.i_video.usegamma += 1;
            if state.i_video.usegamma > 4_i32 {
                state.i_video.usegamma = 0_i32;
            }
            state.g_game.players[state.g_game.consoleplayer as usize].message =
                Some(gammamsg[state.i_video.usegamma as usize].to_string());
            let pal = W_LumpBytesName(state, "PLAYPAL");
            I_SetPalette(state, &pal[..768]);
            return true;
        }
    }
    if !state.m_menu.menuactive {
        if key == state.m_controls.key_menu_activate {
            M_StartControlPanel(state);
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
            return true;
        }
        return false;
    }
    if key == state.m_controls.key_menu_down {
        loop {
            if state.m_menu.itemOn as i32 + 1_i32 > state.m_menu.current().numitems as i32 - 1_i32 {
                state.m_menu.itemOn = 0_i16;
            } else {
                state.m_menu.itemOn += 1;
            }
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_pstop as i32);
            if state.m_menu.current().items[state.m_menu.itemOn as usize].status as i32 != -1_i32 {
                break;
            }
        }
        return true;
    } else if key == state.m_controls.key_menu_up {
        loop {
            if state.m_menu.itemOn == 0 {
                state.m_menu.itemOn = (state.m_menu.current().numitems as i32 - 1_i32) as i16;
            } else {
                state.m_menu.itemOn -= 1;
            }
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_pstop as i32);
            if state.m_menu.current().items[state.m_menu.itemOn as usize].status as i32 != -1_i32 {
                break;
            }
        }
        return true;
    } else if key == state.m_controls.key_menu_left {
        let item = state.m_menu.current().items[state.m_menu.itemOn as usize];
        if let Some(routine) = item.routine.filter(|_| item.status as i32 == 2_i32) {
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_stnmov as i32);
            routine(state, 0_i32);
        }
        return true;
    } else if key == state.m_controls.key_menu_right {
        let item = state.m_menu.current().items[state.m_menu.itemOn as usize];
        if let Some(routine) = item.routine.filter(|_| item.status as i32 == 2_i32) {
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_stnmov as i32);
            routine(state, 1_i32);
        }
        return true;
    } else if key == state.m_controls.key_menu_forward {
        let item = state.m_menu.current().items[state.m_menu.itemOn as usize];
        if let Some(routine) = item.routine.filter(|_| item.status as i32 != 0) {
            let item_on = state.m_menu.itemOn;
            state.m_menu.current_mut().lastOn = item_on;
            if item.status as i32 == 2_i32 {
                routine(state, 1_i32);
                S_StartSound(state, SoundOrigin::None, SfxName::sfx_stnmov as i32);
            } else {
                let item_on = state.m_menu.itemOn as i32;
                routine(state, item_on);
                S_StartSound(state, SoundOrigin::None, SfxName::sfx_pistol as i32);
            }
        }
        return true;
    } else if key == state.m_controls.key_menu_activate {
        let item_on = state.m_menu.itemOn;
        state.m_menu.current_mut().lastOn = item_on;
        M_ClearMenus(state);
        S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchx as i32);
        return true;
    } else if key == state.m_controls.key_menu_back {
        let item_on = state.m_menu.itemOn;
        state.m_menu.current_mut().lastOn = item_on;
        if let Some(prev) = state.m_menu.current().prevMenu {
            state.m_menu.currentMenu = prev;
            state.m_menu.itemOn = state.m_menu.current().lastOn;
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_swtchn as i32);
        }
        return true;
    } else if ch != 0_i32 || IsNullKey(key) {
        i = state.m_menu.itemOn as i32 + 1_i32;
        while i < state.m_menu.current().numitems as i32 {
            if state.m_menu.current().items[i as usize].alphaKey as i32 == ch {
                state.m_menu.itemOn = i as i16;
                S_StartSound(state, SoundOrigin::None, SfxName::sfx_pstop as i32);
                return true;
            }
            i += 1;
        }
        i = 0_i32;
        while i <= state.m_menu.itemOn as i32 {
            if state.m_menu.current().items[i as usize].alphaKey as i32 == ch {
                state.m_menu.itemOn = i as i16;
                S_StartSound(state, SoundOrigin::None, SfxName::sfx_pstop as i32);
                return true;
            }
            i += 1;
        }
    }
    false
}
pub fn M_StartControlPanel(state: &mut GameState) {
    if state.m_menu.menuactive {
        return;
    }
    state.m_menu.menuactive = true;
    state.m_menu.currentMenu = MenuId::Main;
    state.m_menu.itemOn = state.m_menu.current().lastOn;
}
pub fn M_Drawer(state: &mut GameState) {
    let mut i: u32;
    
    state.m_menu.inhelpscreens = false;
    if state.m_menu.messageToPrint != 0 {
        let message_string = state.m_menu.messageString.clone();
        state.m_menu.drawer_y =
            (SCREENHEIGHT / 2_i32 - M_StringHeight(state, &message_string) / 2_i32) as i16;
        for line in message_string.split('\n') {
            let line = if line.len() > 79 { &line[..79] } else { line };
            state.m_menu.drawer_x =
                (SCREENWIDTH / 2_i32 - M_StringWidth(state, line) / 2_i32) as i16;
            M_WriteText(
                state,
                state.m_menu.drawer_x as i32,
                state.m_menu.drawer_y as i32,
                line,
            );
            state.m_menu.drawer_y = (state.m_menu.drawer_y as i32
                + V_CachePatchNum(state, state.hu_stuff.hu_font[0]).height())
                as i16;
        }
        return;
    }
    if !state.m_menu.menuactive {
        return;
    }
    let draw_routine = state.m_menu.current().routine;
    if let Some(draw_routine) = draw_routine {
        draw_routine(state);
    }
    state.m_menu.drawer_x = state.m_menu.current().x;
    state.m_menu.drawer_y = state.m_menu.current().y;
    let max: u32 = state.m_menu.current().numitems as u32;
    i = 0_u32;
    while i < max {
        let item_name = state.m_menu.current().items[i as usize].name;
        if !item_name.is_empty() {
            let __wcache2221_2 = V_CachePatchName(state, &item_name.as_str());
            let dest_screen = Screen::Video;
            V_DrawPatchDirect(
                state,
                dest_screen,
                state.m_menu.drawer_x as i32,
                state.m_menu.drawer_y as i32,
                &__wcache2221_2,
            );
        }
        state.m_menu.drawer_y = (state.m_menu.drawer_y as i32 + LINEHEIGHT) as i16;
        i = i.wrapping_add(1);
    }
    let __wcache2231_1 = V_CachePatchName(state, skullName[state.m_menu.whichSkull as usize]);
    let dest_screen = Screen::Video;
    V_DrawPatchDirect(
        state,
        dest_screen,
        state.m_menu.drawer_x as i32 + SKULLXOFF,
        state.m_menu.current().y as i32 - 5_i32 + state.m_menu.itemOn as i32 * LINEHEIGHT,
        &__wcache2231_1,
    );
}
pub fn M_ClearMenus(state: &mut GameState) {
    state.m_menu.menuactive = false;
}
pub fn M_SetupNextMenu(state: &mut GameState, menudef: MenuId) {
    state.m_menu.currentMenu = menudef;
    state.m_menu.itemOn = state.m_menu.current().lastOn;
}
pub fn M_Ticker(state: &mut GameState) {
    state.m_menu.skullAnimCounter -= 1;
    if state.m_menu.skullAnimCounter as i32 <= 0_i32 {
        state.m_menu.whichSkull = (state.m_menu.whichSkull as i32 ^ 1_i32) as i16;
        state.m_menu.skullAnimCounter = 8_i16;
    }
}
pub fn M_Init(state: &mut GameState) {
    state.m_menu.currentMenu = MenuId::Main;
    state.m_menu.menuactive = false;
    state.m_menu.itemOn = state.m_menu.current().lastOn;
    state.m_menu.whichSkull = 0_i16;
    state.m_menu.skullAnimCounter = 10_i16;
    state.m_menu.screenSize = state.m_menu.screenblocks - 3_i32;
    state.m_menu.messageToPrint = 0_i32;
    state.m_menu.messageString = String::new();
    state.m_menu.messageLastMenuActive = state.m_menu.menuactive as i32;
    state.m_menu.quickSaveSlot = -1_i32;
    match state.doomstat.gamemode as u32 {
        2 => {
            state.m_menu.defs.MainDef.items[MainMenu::readthis as usize] =
                state.m_menu.defs.MainDef.items[MainMenu::quitdoom as usize];
            state.m_menu.defs.MainDef.numitems -= 1;
            state.m_menu.defs.MainDef.y = (state.m_menu.defs.MainDef.y as i32 + 8_i32) as i16;
            state.m_menu.defs.NewDef.prevMenu = Some(MenuId::Main);
        }
        0 => {}
        _ => {}
    }
    if !state.doomstat.gameversion.is_ultimate_or_higher() {
        state.m_menu.defs.EpiDef.numitems -= 1;
    }
}
