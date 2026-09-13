use crate::src::d_event::{event_t, GameScreenState};
use crate::src::d_main::D_StartTitle;
use crate::src::dstrings::{doom1_endmsg, doom2_endmsg};
use crate::src::hu_lib::patch_t;
use crate::src::i_system::I_Error;
use crate::src::w_wad::W_CacheLumpName;

use crate::src::d_event::EvType;
use crate::src::d_mode::{commercial, registered, retail, shareware};
use crate::src::d_mode::{doom, doom2, pack_chex, pack_hacx};
use crate::src::d_mode::{skill_from_raw, GameVersion};
use crate::src::doomdef::NULL;
use crate::src::doomdef::SCREENHEIGHT;
use crate::src::doomdef::SCREENWIDTH;
use crate::src::fixed_cstr::FixedCStr;
use crate::src::g_game::G_DeferedInitNew;
use crate::src::g_game::G_LoadGame;
use crate::src::g_game::G_SaveGame;
use crate::src::g_game::G_ScreenShot;
use crate::src::game_state::GameState;
use crate::src::hu_stuff::HU_FONTSIZE;
use crate::src::hu_stuff::HU_FONTSTART;
use crate::src::i_system::I_Quit;
use crate::src::i_timer::I_GetTime;
use crate::src::i_video::I_SetPalette;
use crate::src::m_controls::KEY_BACKSPACE;
use crate::src::m_controls::KEY_CAPSLOCK;
use crate::src::m_controls::KEY_ENTER;
use crate::src::m_controls::KEY_ESCAPE;
use crate::src::m_controls::KEY_PAUSE;
use crate::src::m_controls::KEY_SCRLCK;
use crate::src::p_saveg::P_SaveGameFile;
use crate::src::r_main::R_SetViewSize;
use crate::src::s_sound::S_SetMusicVolume;
use crate::src::s_sound::S_SetSfxVolume;
use crate::src::s_sound::S_StartSound;
use crate::src::sounds::{
    sfx_boscub, sfx_bspact, sfx_dmpain, sfx_getpow, sfx_kntdth, sfx_oof, sfx_pistol, sfx_pldeth,
    sfx_popain, sfx_posit1, sfx_posit3, sfx_pstop, sfx_sgtatk, sfx_skeswg, sfx_slop, sfx_stnmov,
    sfx_swtchn, sfx_swtchx, sfx_telept, sfx_vilact,
};
use crate::src::stdint_types::byte;
use crate::src::v_video::V_DrawPatchDirect;
use crate::src::z_zone::PU_CACHE;

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

impl MMenuDefsHolder {
    pub fn new() -> Self {
        MMenuDefsHolder {
            MainDef: menu_s {
                numitems: main_end as i32 as i16,
                prevMenu: ::core::ptr::null::<menu_s>() as *mut menu_s,
                menuitems: ::core::ptr::null_mut::<menuitem_t>(),
                routine: None,
                x: 97 as i16,
                y: 64 as i16,
                lastOn: 0 as i16,
            },
            EpiDef: menu_s {
                numitems: ep_end as i32 as i16,
                prevMenu: ::core::ptr::null_mut::<menu_s>(),
                menuitems: ::core::ptr::null_mut::<menuitem_t>(),
                routine: None,
                x: 48 as i16,
                y: 63 as i16,
                lastOn: ep1 as i32 as i16,
            },
            NewDef: menu_s {
                numitems: newg_end as i32 as i16,
                prevMenu: ::core::ptr::null_mut::<menu_s>(),
                menuitems: ::core::ptr::null_mut::<menuitem_t>(),
                routine: None,
                x: 48 as i16,
                y: 63 as i16,
                lastOn: hurtme as i32 as i16,
            },
            OptionsDef: menu_s {
                numitems: opt_end as i32 as i16,
                prevMenu: ::core::ptr::null_mut::<menu_s>(),
                menuitems: ::core::ptr::null_mut::<menuitem_t>(),
                routine: None,
                x: 60 as i16,
                y: 37 as i16,
                lastOn: 0 as i16,
            },
            ReadDef1: menu_s {
                numitems: read1_end as i32 as i16,
                prevMenu: ::core::ptr::null_mut::<menu_s>(),
                menuitems: ::core::ptr::null_mut::<menuitem_t>(),
                routine: None,
                x: 280 as i16,
                y: 185 as i16,
                lastOn: 0 as i16,
            },
            ReadDef2: menu_s {
                numitems: read2_end as i32 as i16,
                prevMenu: ::core::ptr::null_mut::<menu_s>(),
                menuitems: ::core::ptr::null_mut::<menuitem_t>(),
                routine: None,
                x: 330 as i16,
                y: 175 as i16,
                lastOn: 0 as i16,
            },
            SoundDef: menu_s {
                numitems: sound_end as i32 as i16,
                prevMenu: ::core::ptr::null_mut::<menu_s>(),
                menuitems: ::core::ptr::null_mut::<menuitem_t>(),
                routine: None,
                x: 80 as i16,
                y: 64 as i16,
                lastOn: 0 as i16,
            },
            LoadDef: menu_s {
                numitems: load_end as i32 as i16,
                prevMenu: ::core::ptr::null_mut::<menu_s>(),
                menuitems: ::core::ptr::null_mut::<menuitem_t>(),
                routine: None,
                x: 80 as i16,
                y: 54 as i16,
                lastOn: 0 as i16,
            },
            SaveDef: menu_s {
                numitems: load_end as i32 as i16,
                prevMenu: ::core::ptr::null_mut::<menu_s>(),
                menuitems: ::core::ptr::null_mut::<menuitem_t>(),
                routine: None,
                x: 80 as i16,
                y: 54 as i16,
                lastOn: 0 as i16,
            },
        }
    }
}

pub struct MMenuMenusHolder {
    pub MainMenu: [menuitem_t; 6],
    pub EpisodeMenu: [menuitem_t; 4],
    pub NewGameMenu: [menuitem_t; 5],
    pub OptionsMenu: [menuitem_t; 8],
    pub ReadMenu1: [menuitem_t; 1],
    pub ReadMenu2: [menuitem_t; 1],
    pub SoundMenu: [menuitem_t; 4],
    pub LoadMenu: [menuitem_t; 6],
    pub SaveMenu: [menuitem_t; 6],
}

impl MMenuMenusHolder {
    pub fn new() -> Self {
        MMenuMenusHolder {
            MainMenu: [
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_NGAME\0\0\0"),
                    routine: None,
                    alphaKey: 'n' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_OPTION\0\0"),
                    routine: None,
                    alphaKey: 'o' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_LOADG\0\0\0"),
                    routine: None,
                    alphaKey: 'l' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_SAVEG\0\0\0"),
                    routine: None,
                    alphaKey: 's' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_RDTHIS\0\0"),
                    routine: None,
                    alphaKey: 'r' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_QUITG\0\0\0"),
                    routine: None,
                    alphaKey: 'q' as u8,
                },
            ],
            EpisodeMenu: [
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_EPI1\0\0\0\0"),
                    routine: None,
                    alphaKey: 'k' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_EPI2\0\0\0\0"),
                    routine: None,
                    alphaKey: 't' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_EPI3\0\0\0\0"),
                    routine: None,
                    alphaKey: 'i' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_EPI4\0\0\0\0"),
                    routine: None,
                    alphaKey: 't' as u8,
                },
            ],
            NewGameMenu: [
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_JKILL\0\0\0"),
                    routine: None,
                    alphaKey: 'i' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_ROUGH\0\0\0"),
                    routine: None,
                    alphaKey: 'h' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_HURT\0\0\0\0"),
                    routine: None,
                    alphaKey: 'h' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_ULTRA\0\0\0"),
                    routine: None,
                    alphaKey: 'u' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_NMARE\0\0\0"),
                    routine: None,
                    alphaKey: 'n' as u8,
                },
            ],
            OptionsMenu: [
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_ENDGAM\0\0"),
                    routine: None,
                    alphaKey: 'e' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_MESSG\0\0\0"),
                    routine: None,
                    alphaKey: 'm' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_DETAIL\0\0"),
                    routine: None,
                    alphaKey: 'g' as u8,
                },
                menuitem_t {
                    status: 2 as i16,
                    name: FixedCStr(*b"M_SCRNSZ\0\0"),
                    routine: None,
                    alphaKey: 's' as u8,
                },
                menuitem_t {
                    status: -(1 as i32) as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '\0' as u8,
                },
                menuitem_t {
                    status: 2 as i16,
                    name: FixedCStr(*b"M_MSENS\0\0\0"),
                    routine: None,
                    alphaKey: 'm' as u8,
                },
                menuitem_t {
                    status: -(1 as i32) as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '\0' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"M_SVOL\0\0\0\0"),
                    routine: None,
                    alphaKey: 's' as u8,
                },
            ],
            ReadMenu1: [menuitem_t {
                status: 1 as i16,
                name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                routine: None,
                alphaKey: 0 as u8,
            }],
            ReadMenu2: [menuitem_t {
                status: 1 as i16,
                name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                routine: None,
                alphaKey: 0 as u8,
            }],
            SoundMenu: [
                menuitem_t {
                    status: 2 as i16,
                    name: FixedCStr(*b"M_SFXVOL\0\0"),
                    routine: None,
                    alphaKey: 's' as u8,
                },
                menuitem_t {
                    status: -(1 as i32) as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '\0' as u8,
                },
                menuitem_t {
                    status: 2 as i16,
                    name: FixedCStr(*b"M_MUSVOL\0\0"),
                    routine: None,
                    alphaKey: 'm' as u8,
                },
                menuitem_t {
                    status: -(1 as i32) as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '\0' as u8,
                },
            ],
            LoadMenu: [
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '1' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '2' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '3' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '4' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '5' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '6' as u8,
                },
            ],
            SaveMenu: [
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '1' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '2' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '3' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '4' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '5' as u8,
                },
                menuitem_t {
                    status: 1 as i16,
                    name: FixedCStr(*b"\0\0\0\0\0\0\0\0\0\0"),
                    routine: None,
                    alphaKey: '6' as u8,
                },
            ],
        }
    }
}

pub struct MMenuState {
    pub menus: MMenuMenusHolder,
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
    pub messageRoutine: Option<unsafe fn(&mut GameState, i32) -> ()>,
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
    pub currentMenu: *mut menu_t,
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

impl MMenuState {
    pub fn new() -> Self {
        MMenuState {
            menus: MMenuMenusHolder::new(),
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
            currentMenu: ::core::ptr::null::<menu_t>() as *mut menu_t,
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

    // Each menu_t's `menuitems` points at this same struct's own XxxMenu
    // array, and most also `prevMenu`-link to a sibling menu_t -- both are
    // only known once this value is at its final, permanently-stable
    // 'static address (inside GameState, behind Box::leak). Called once
    // from `init_game_state`'s `finish_init`, same pattern as
    // `sounds::fixup_self_links`/
    // `p_maputl::fixup_intercepts_overrun`/`m_controls::fixup_weapon_keys`.
    pub fn fixup_menu_routines(&mut self) {
        self.defs.MainDef.routine =
            Some(M_DrawMainMenu as unsafe fn(&mut GameState) -> ());
        self.defs.EpiDef.routine =
            Some(M_DrawEpisode as unsafe fn(&mut GameState) -> ());
        self.defs.NewDef.routine =
            Some(M_DrawNewGame as unsafe fn(&mut GameState) -> ());
        self.defs.OptionsDef.routine =
            Some(M_DrawOptions as unsafe fn(&mut GameState) -> ());
        self.defs.ReadDef1.routine =
            Some(M_DrawReadThis1 as unsafe fn(&mut GameState) -> ());
        self.defs.ReadDef2.routine =
            Some(M_DrawReadThis2 as unsafe fn(&mut GameState) -> ());
        self.defs.SoundDef.routine =
            Some(M_DrawSound as unsafe fn(&mut GameState) -> ());
        self.defs.LoadDef.routine = Some(M_DrawLoad as unsafe fn(&mut GameState) -> ());
        self.defs.SaveDef.routine = Some(M_DrawSave as unsafe fn(&mut GameState) -> ());
        self.menus.MainMenu[0].routine =
            Some(M_NewGame as unsafe fn(&mut GameState, i32) -> ());
        self.menus.MainMenu[1].routine =
            Some(M_Options as unsafe fn(&mut GameState, i32) -> ());
        self.menus.MainMenu[2].routine =
            Some(M_LoadGame as unsafe fn(&mut GameState, i32) -> ());
        self.menus.MainMenu[3].routine =
            Some(M_SaveGame as unsafe fn(&mut GameState, i32) -> ());
        self.menus.MainMenu[4].routine =
            Some(M_ReadThis as unsafe fn(&mut GameState, i32) -> ());
        self.menus.MainMenu[5].routine =
            Some(M_QuitDOOM as unsafe fn(&mut GameState, i32) -> ());
        self.menus.EpisodeMenu[0].routine =
            Some(M_Episode as unsafe fn(&mut GameState, i32) -> ());
        self.menus.EpisodeMenu[1].routine =
            Some(M_Episode as unsafe fn(&mut GameState, i32) -> ());
        self.menus.EpisodeMenu[2].routine =
            Some(M_Episode as unsafe fn(&mut GameState, i32) -> ());
        self.menus.EpisodeMenu[3].routine =
            Some(M_Episode as unsafe fn(&mut GameState, i32) -> ());
        self.menus.NewGameMenu[0].routine =
            Some(M_ChooseSkill as unsafe fn(&mut GameState, i32) -> ());
        self.menus.NewGameMenu[1].routine =
            Some(M_ChooseSkill as unsafe fn(&mut GameState, i32) -> ());
        self.menus.NewGameMenu[2].routine =
            Some(M_ChooseSkill as unsafe fn(&mut GameState, i32) -> ());
        self.menus.NewGameMenu[3].routine =
            Some(M_ChooseSkill as unsafe fn(&mut GameState, i32) -> ());
        self.menus.NewGameMenu[4].routine =
            Some(M_ChooseSkill as unsafe fn(&mut GameState, i32) -> ());
        self.menus.OptionsMenu[0].routine =
            Some(M_EndGame as unsafe fn(&mut GameState, i32) -> ());
        self.menus.OptionsMenu[1].routine =
            Some(M_ChangeMessages as unsafe fn(&mut GameState, i32) -> ());
        self.menus.OptionsMenu[2].routine =
            Some(M_ChangeDetail as unsafe fn(&mut GameState, i32) -> ());
        self.menus.OptionsMenu[3].routine =
            Some(M_SizeDisplay as unsafe fn(&mut GameState, i32) -> ());
        self.menus.OptionsMenu[5].routine =
            Some(M_ChangeSensitivity as unsafe fn(&mut GameState, i32) -> ());
        self.menus.OptionsMenu[7].routine =
            Some(M_Sound as unsafe fn(&mut GameState, i32) -> ());
        self.menus.ReadMenu1[0].routine =
            Some(M_ReadThis2 as unsafe fn(&mut GameState, i32) -> ());
        self.menus.ReadMenu2[0].routine =
            Some(M_FinishReadThis as unsafe fn(&mut GameState, i32) -> ());
        self.menus.SoundMenu[0].routine =
            Some(M_SfxVol as unsafe fn(&mut GameState, i32) -> ());
        self.menus.SoundMenu[2].routine =
            Some(M_MusicVol as unsafe fn(&mut GameState, i32) -> ());
        self.menus.LoadMenu[0].routine =
            Some(M_LoadSelect as unsafe fn(&mut GameState, i32) -> ());
        self.menus.LoadMenu[1].routine =
            Some(M_LoadSelect as unsafe fn(&mut GameState, i32) -> ());
        self.menus.LoadMenu[2].routine =
            Some(M_LoadSelect as unsafe fn(&mut GameState, i32) -> ());
        self.menus.LoadMenu[3].routine =
            Some(M_LoadSelect as unsafe fn(&mut GameState, i32) -> ());
        self.menus.LoadMenu[4].routine =
            Some(M_LoadSelect as unsafe fn(&mut GameState, i32) -> ());
        self.menus.LoadMenu[5].routine =
            Some(M_LoadSelect as unsafe fn(&mut GameState, i32) -> ());
        self.menus.SaveMenu[0].routine =
            Some(M_SaveSelect as unsafe fn(&mut GameState, i32) -> ());
        self.menus.SaveMenu[1].routine =
            Some(M_SaveSelect as unsafe fn(&mut GameState, i32) -> ());
        self.menus.SaveMenu[2].routine =
            Some(M_SaveSelect as unsafe fn(&mut GameState, i32) -> ());
        self.menus.SaveMenu[3].routine =
            Some(M_SaveSelect as unsafe fn(&mut GameState, i32) -> ());
        self.menus.SaveMenu[4].routine =
            Some(M_SaveSelect as unsafe fn(&mut GameState, i32) -> ());
        self.menus.SaveMenu[5].routine =
            Some(M_SaveSelect as unsafe fn(&mut GameState, i32) -> ());
    }
    pub fn fixup_menu_links(&mut self) {
        self.defs.MainDef.menuitems = &raw mut self.menus.MainMenu as *mut menuitem_t;
        self.defs.EpiDef.menuitems = &raw mut self.menus.EpisodeMenu as *mut menuitem_t;
        self.defs.EpiDef.prevMenu = &raw mut self.defs.MainDef as *mut menu_s;
        self.defs.NewDef.menuitems = &raw mut self.menus.NewGameMenu as *mut menuitem_t;
        self.defs.NewDef.prevMenu = &raw mut self.defs.EpiDef as *mut menu_s;
        self.defs.OptionsDef.menuitems = &raw mut self.menus.OptionsMenu as *mut menuitem_t;
        self.defs.OptionsDef.prevMenu = &raw mut self.defs.MainDef as *mut menu_s;
        self.defs.ReadDef1.menuitems = &raw mut self.menus.ReadMenu1 as *mut menuitem_t;
        self.defs.ReadDef1.prevMenu = &raw mut self.defs.MainDef as *mut menu_s;
        self.defs.ReadDef2.menuitems = &raw mut self.menus.ReadMenu2 as *mut menuitem_t;
        self.defs.ReadDef2.prevMenu = &raw mut self.defs.ReadDef1 as *mut menu_s;
        self.defs.SoundDef.menuitems = &raw mut self.menus.SoundMenu as *mut menuitem_t;
        self.defs.SoundDef.prevMenu = &raw mut self.defs.OptionsDef as *mut menu_s;
        self.defs.LoadDef.menuitems = &raw mut self.menus.LoadMenu as *mut menuitem_t;
        self.defs.LoadDef.prevMenu = &raw mut self.defs.MainDef as *mut menu_s;
        self.defs.SaveDef.menuitems = &raw mut self.menus.SaveMenu as *mut menuitem_t;
        self.defs.SaveDef.prevMenu = &raw mut self.defs.MainDef as *mut menu_s;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct menuitem_t {
    pub status: i16,
    pub name: FixedCStr<10>,
    pub routine: Option<unsafe fn(&mut GameState, i32) -> ()>,
    pub alphaKey: u8,
}
pub type menu_t = menu_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menu_s {
    pub numitems: i16,
    pub prevMenu: *mut menu_s,
    pub menuitems: *mut menuitem_t,
    pub routine: Option<unsafe fn(&mut GameState) -> ()>,
    pub x: i16,
    pub y: i16,
    pub lastOn: i16,
}
pub const read2_end: C2RustUnnamed_6 = 1;
pub const read1_end: C2RustUnnamed_5 = 1;
pub const load_end: C2RustUnnamed_8 = 6;
pub const scrnsize: C2RustUnnamed_4 = 3;
pub const mousesens: C2RustUnnamed_4 = 5;
pub const messages: C2RustUnnamed_4 = 1;
pub const detail: C2RustUnnamed_4 = 2;
pub const music_vol: C2RustUnnamed_7 = 2;
pub const sfx_vol: C2RustUnnamed_7 = 0;
pub const sound_end: C2RustUnnamed_7 = 4;
pub const opt_end: C2RustUnnamed_4 = 8;
pub const ep1: C2RustUnnamed_2 = 0;
pub const hurtme: C2RustUnnamed_3 = 2;
pub const nightmare: C2RustUnnamed_3 = 4;
pub const newg_end: C2RustUnnamed_3 = 5;
pub const ep_end: C2RustUnnamed_2 = 4;
pub const main_end: C2RustUnnamed_1 = 6;
pub const quitdoom: C2RustUnnamed_1 = 5;
pub const readthis: C2RustUnnamed_1 = 4;
pub type C2RustUnnamed_1 = u32;
pub const savegame: C2RustUnnamed_1 = 3;
pub const loadgame: C2RustUnnamed_1 = 2;
pub const options: C2RustUnnamed_1 = 1;
pub const newgame: C2RustUnnamed_1 = 0;
pub type C2RustUnnamed_2 = u32;
pub const ep4: C2RustUnnamed_2 = 3;
pub const ep3: C2RustUnnamed_2 = 2;
pub const ep2: C2RustUnnamed_2 = 1;
pub type C2RustUnnamed_3 = u32;
pub const violence: C2RustUnnamed_3 = 3;
pub const toorough: C2RustUnnamed_3 = 1;
pub const killthings: C2RustUnnamed_3 = 0;
pub type C2RustUnnamed_4 = u32;
pub const soundvol: C2RustUnnamed_4 = 7;
pub const option_empty2: C2RustUnnamed_4 = 6;
pub const option_empty1: C2RustUnnamed_4 = 4;
pub const endgame: C2RustUnnamed_4 = 0;
pub type C2RustUnnamed_5 = u32;
pub const rdthsempty1: C2RustUnnamed_5 = 0;
pub type C2RustUnnamed_6 = u32;
pub const rdthsempty2: C2RustUnnamed_6 = 0;
pub type C2RustUnnamed_7 = u32;
pub const sfx_empty2: C2RustUnnamed_7 = 3;
pub const sfx_empty1: C2RustUnnamed_7 = 1;
pub type C2RustUnnamed_8 = u32;
pub const load6: C2RustUnnamed_8 = 5;
pub const load5: C2RustUnnamed_8 = 4;
pub const load4: C2RustUnnamed_8 = 3;
pub const load3: C2RustUnnamed_8 = 2;
pub const load2: C2RustUnnamed_8 = 1;
pub const load1: C2RustUnnamed_8 = 0;
pub const KEY_NUMLOCK: i32 = 0x80 + 0x45 as i32;
pub const GAMMALVL0: &str = "Gamma correction OFF\0";
pub const GAMMALVL1: &str = "Gamma correction level 1\0";
pub const GAMMALVL2: &str = "Gamma correction level 2\0";
pub const GAMMALVL3: &str = "Gamma correction level 3\0";
pub const GAMMALVL4: &str = "Gamma correction level 4\0";
pub const EMPTYSTRING: &str = "empty slot\0";
pub const NUM_QUITMESSAGES: i32 = 8;
pub const SAVESTRINGSIZE: i32 = 24;
pub static gammamsg: [&str; 5] = [GAMMALVL0, GAMMALVL1, GAMMALVL2, GAMMALVL3, GAMMALVL4];
pub const SKULLXOFF: i32 = -(32 as i32);
pub const LINEHEIGHT: i32 = 16;
#[no_mangle]
pub static skullName: [&str; 2] = ["M_SKULL1", "M_SKULL2"];
#[no_mangle]
pub static main_e: C2RustUnnamed_1 = newgame;
#[no_mangle]
pub static episodes_e: C2RustUnnamed_2 = ep1;
#[no_mangle]
pub static newgame_e: C2RustUnnamed_3 = killthings;
#[no_mangle]
pub static options_e: C2RustUnnamed_4 = endgame;
#[no_mangle]
pub static read_e: C2RustUnnamed_5 = rdthsempty1;
#[no_mangle]
pub static read_e2: C2RustUnnamed_6 = rdthsempty2;
#[no_mangle]
pub static sound_e: C2RustUnnamed_7 = sfx_vol;
#[no_mangle]
pub static load_e: C2RustUnnamed_8 = load1;
pub fn M_ReadSaveStrings(state: &mut GameState) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < load_end as i32 {
        let savegame_file = P_SaveGameFile(state, i);
        match std::fs::File::open(&savegame_file) {
            Err(_) => {
                state.m_menu.savegamestrings[i as usize] =
                    EMPTYSTRING.trim_end_matches('\0').to_string();
                state.m_menu.menus.LoadMenu[i as usize].status = 0 as i16;
            }
            Ok(mut handle) => {
                let mut buf: [u8; SAVESTRINGSIZE as usize] = [0; SAVESTRINGSIZE as usize];
                let _ = std::io::Read::read(&mut handle, &mut buf);
                let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
                state.m_menu.savegamestrings[i as usize] =
                    String::from_utf8_lossy(&buf[..len]).into_owned();
                state.m_menu.menus.LoadMenu[i as usize].status = 1 as i16;
            }
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe fn M_DrawLoad(state: &mut GameState) {
    let mut i: i32 = 0;
    let __wcache890_24 = W_CacheLumpName(state, "M_LOADG", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, 72 as i32, 28 as i32, __wcache890_24);
    i = 0 as i32;
    while i < load_end as i32 {
        let loaddef_x = state.m_menu.defs.LoadDef.x as i32;
        let loaddef_y = state.m_menu.defs.LoadDef.y as i32 + LINEHEIGHT * i;
        M_DrawSaveLoadBorder(state, loaddef_x, loaddef_y);
        let savestr = state.m_menu.savegamestrings[i as usize].clone();
        M_WriteText(state, loaddef_x, loaddef_y, &savestr);
        i += 1;
    }
}
pub unsafe fn M_DrawSaveLoadBorder(state: &mut GameState, mut x: i32, mut y: i32) {
    let mut i: i32 = 0;
    let __wcache908_23 = W_CacheLumpName(state, "M_LSLEFT", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, x - 8 as i32, y + 7 as i32, __wcache908_23);
    i = 0 as i32;
    while i < 24 as i32 {
        let __wcache916_22 = W_CacheLumpName(state, "M_LSCNTR", PU_CACHE as i32) as *mut patch_t;
        V_DrawPatchDirect(state, x, y + 7 as i32, __wcache916_22);
        x += 8 as i32;
        i += 1;
    }
    let __wcache925_21 = W_CacheLumpName(state, "M_LSRGHT", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, x, y + 7 as i32, __wcache925_21);
}
#[no_mangle]
pub fn M_LoadSelect(state: &mut GameState, choice: i32) {
    let savegame_file = P_SaveGameFile(state, choice);
    G_LoadGame(state, &savegame_file);
    M_ClearMenus(state);
}
#[no_mangle]
pub unsafe fn M_LoadGame(state: &mut GameState, _choice: i32) {
    if state.g_game.netgame {
        M_StartMessage(
            state,
            "you can't do load while in a net game!\n\npress a key.",
            None,
            false,
        );
        return;
    }
    let menudef = &raw mut state.m_menu.defs.LoadDef;
    M_SetupNextMenu(state, menudef);
    M_ReadSaveStrings(state);
}
#[no_mangle]
pub unsafe fn M_DrawSave(state: &mut GameState) {
    let mut i: i32 = 0;
    let __wcache961_20 = W_CacheLumpName(state, "M_SAVEG", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, 72 as i32, 28 as i32, __wcache961_20);
    i = 0 as i32;
    while i < load_end as i32 {
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
pub fn M_DoSave(state: &mut GameState, mut slot: i32) {
    let savegame_name = state.m_menu.savegamestrings[slot as usize].clone();
    G_SaveGame(state, slot, &savegame_name);
    M_ClearMenus(state);
    if state.m_menu.quickSaveSlot == -(2 as i32) {
        state.m_menu.quickSaveSlot = slot;
    }
}
#[no_mangle]
pub fn M_SaveSelect(state: &mut GameState, mut choice: i32) {
    state.m_menu.saveStringEnter = 1 as i32;
    state.m_menu.saveSlot = choice;
    state.m_menu.saveOldString = state.m_menu.savegamestrings[choice as usize].clone();
    if state.m_menu.savegamestrings[choice as usize] == EMPTYSTRING.trim_end_matches('\0') {
        state.m_menu.savegamestrings[choice as usize].clear();
    }
    state.m_menu.saveCharIndex = state.m_menu.savegamestrings[choice as usize].len() as i32;
}
#[no_mangle]
pub unsafe fn M_SaveGame(state: &mut GameState, _choice: i32) {
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
    let menudef = &raw mut state.m_menu.defs.SaveDef;
    M_SetupNextMenu(state, menudef);
    M_ReadSaveStrings(state);
}
#[no_mangle]
pub unsafe fn M_QuickSaveResponse(state: &mut GameState, mut key: i32) {
    if key == state.m_controls.key_menu_confirm {
        let quick_save_slot = state.m_menu.quickSaveSlot;
        M_DoSave(state, quick_save_slot);
        S_StartSound(state, NULL, sfx_swtchx as i32);
    }
}
pub unsafe fn M_QuickSave(state: &mut GameState) {
    if !state.g_game.usergame {
        S_StartSound(state, NULL, sfx_oof as i32);
        return;
    }
    if state.g_game.gamestate != GameScreenState::GS_LEVEL {
        return;
    }
    if state.m_menu.quickSaveSlot < 0 as i32 {
        M_StartControlPanel(state);
        M_ReadSaveStrings(state);
        let menudef = &raw mut state.m_menu.defs.SaveDef;
        M_SetupNextMenu(state, menudef);
        state.m_menu.quickSaveSlot = -(2 as i32);
        return;
    }
    let msg = format!(
        "quicksave over your game named\n\n'{}'?\n\npress y or n.",
        state.m_menu.savegamestrings[state.m_menu.quickSaveSlot as usize],
    );
    let routine = Some(M_QuickSaveResponse as unsafe fn(&mut GameState, i32) -> ());
    M_StartMessage(state, &msg, routine, true);
}
#[no_mangle]
pub unsafe fn M_QuickLoadResponse(state: &mut GameState, mut key: i32) {
    if key == state.m_controls.key_menu_confirm {
        let quick_save_slot = state.m_menu.quickSaveSlot;
        M_LoadSelect(state, quick_save_slot);
        S_StartSound(state, NULL, sfx_swtchx as i32);
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
    if state.m_menu.quickSaveSlot < 0 as i32 {
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
    let routine = Some(M_QuickLoadResponse as unsafe fn(&mut GameState, i32) -> ());
    M_StartMessage(state, &msg, routine, true);
}
#[no_mangle]
pub unsafe fn M_DrawReadThis1(state: &mut GameState) {
    let mut lumpname: &str = "CREDIT";
    let mut skullx: i32 = 330 as i32;
    let mut skully: i32 = 175 as i32;
    state.m_menu.inhelpscreens = true;
    match state.doomstat.gameversion as u32 {
        1 | 2 | 3 | 4 | 5 => {
            if state.doomstat.gamemode as u32 == commercial as i32 as u32 {
                lumpname = "HELP";
                skullx = 330 as i32;
                skully = 165 as i32;
            } else {
                lumpname = "HELP2";
                skullx = 280 as i32;
                skully = 185 as i32;
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
    let __wcache1158_19 = W_CacheLumpName(state, lumpname, PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, 0 as i32, 0 as i32, __wcache1158_19);
    state.m_menu.defs.ReadDef1.x = skullx as i16;
    state.m_menu.defs.ReadDef1.y = skully as i16;
}
#[no_mangle]
pub unsafe fn M_DrawReadThis2(state: &mut GameState) {
    state.m_menu.inhelpscreens = true;
    let __wcache1170_18 = W_CacheLumpName(state, "HELP1", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, 0 as i32, 0 as i32, __wcache1170_18);
}
#[no_mangle]
pub unsafe fn M_DrawSound(state: &mut GameState) {
    let __wcache1179_17 = W_CacheLumpName(state, "M_SVOL", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, 60 as i32, 38 as i32, __wcache1179_17);
    let (x, y, vol) = (
        state.m_menu.defs.SoundDef.x as i32,
        state.m_menu.defs.SoundDef.y as i32 + LINEHEIGHT * (sfx_vol as i32 + 1 as i32),
        state.s_sound.sfxVolume,
    );
    M_DrawThermo(state, x, y, 16 as i32, vol);
    let (x, y, vol) = (
        state.m_menu.defs.SoundDef.x as i32,
        state.m_menu.defs.SoundDef.y as i32 + LINEHEIGHT * (music_vol as i32 + 1 as i32),
        state.s_sound.musicVolume,
    );
    M_DrawThermo(state, x, y, 16 as i32, vol);
}
#[no_mangle]
pub unsafe fn M_Sound(state: &mut GameState, _choice: i32) {
    let menudef = &raw mut state.m_menu.defs.SoundDef;
    M_SetupNextMenu(state, menudef);
}
#[no_mangle]
pub fn M_SfxVol(state: &mut GameState, mut choice: i32) {
    match choice {
        0 => {
            if state.s_sound.sfxVolume != 0 {
                state.s_sound.sfxVolume -= 1;
            }
        }
        1 => {
            if state.s_sound.sfxVolume < 15 as i32 {
                state.s_sound.sfxVolume += 1;
            }
        }
        _ => {}
    }
    let sfx_volume = state.s_sound.sfxVolume * 8 as i32;
    S_SetSfxVolume(state, sfx_volume);
}
#[no_mangle]
pub unsafe fn M_MusicVol(state: &mut GameState, mut choice: i32) {
    match choice {
        0 => {
            if state.s_sound.musicVolume != 0 {
                state.s_sound.musicVolume -= 1;
            }
        }
        1 => {
            if state.s_sound.musicVolume < 15 as i32 {
                state.s_sound.musicVolume += 1;
            }
        }
        _ => {}
    }
    let music_volume = state.s_sound.musicVolume * 8 as i32;
    S_SetMusicVolume(state, music_volume);
}
#[no_mangle]
pub unsafe fn M_DrawMainMenu(state: &mut GameState) {
    let __wcache1241_16 = W_CacheLumpName(state, "M_DOOM", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, 94 as i32, 2 as i32, __wcache1241_16);
}
#[no_mangle]
pub unsafe fn M_DrawNewGame(state: &mut GameState) {
    let __wcache1250_15 = W_CacheLumpName(state, "M_NEWG", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, 96 as i32, 14 as i32, __wcache1250_15);
    let __wcache1256_14 = W_CacheLumpName(state, "M_SKILL", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, 54 as i32, 38 as i32, __wcache1256_14);
}
#[no_mangle]
pub unsafe fn M_NewGame(state: &mut GameState, _choice: i32) {
    if state.g_game.netgame && !state.g_game.demoplayback {
        M_StartMessage(
            state,
            "you can't start a new game\nwhile in a network game.\n\npress a key.",
            None,
            false,
        );
        return;
    }
    if state.doomstat.gamemode as u32 == commercial as i32 as u32
        || state.doomstat.gameversion == GameVersion::chex
    {
        let menudef = &raw mut state.m_menu.defs.NewDef;
        M_SetupNextMenu(state, menudef);
    } else {
        let menudef = &raw mut state.m_menu.defs.EpiDef;
        M_SetupNextMenu(state, menudef);
    };
}
#[no_mangle]
pub unsafe fn M_DrawEpisode(state: &mut GameState) {
    let __wcache1286_13 = W_CacheLumpName(state, "M_EPISOD", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, 54 as i32, 38 as i32, __wcache1286_13);
}
#[no_mangle]
pub fn M_VerifyNightmare(state: &mut GameState, mut key: i32) {
    if key != state.m_controls.key_menu_confirm {
        return;
    }
    G_DeferedInitNew(
        state,
        skill_from_raw(nightmare as i32),
        state.m_menu.epi + 1 as i32,
        1 as i32,
    );
    M_ClearMenus(state);
}
#[no_mangle]
pub fn M_ChooseSkill(state: &mut GameState, mut choice: i32) {
    if choice == nightmare as i32 {
        M_StartMessage(
            state,
            "are you sure? this skill level\nisn't even remotely fair.\n\npress y or n.",
            Some(M_VerifyNightmare as unsafe fn(&mut GameState, i32) -> ()),
            true,
        );
        return;
    }
    G_DeferedInitNew(
        state,
        skill_from_raw(choice),
        state.m_menu.epi + 1 as i32,
        1 as i32,
    );
    M_ClearMenus(state);
}
#[no_mangle]
pub unsafe fn M_Episode(state: &mut GameState, mut choice: i32) {
    if state.doomstat.gamemode as u32 == shareware as i32 as u32 && choice != 0 {
        M_StartMessage(state, 
            "this is the shareware version of doom.\n\nyou need to order the entire trilogy.\n\npress a key.",
            None,
            false,
        );
        let menudef = &raw mut state.m_menu.defs.ReadDef1;
        M_SetupNextMenu(state, menudef);
        return;
    }
    if state.doomstat.gamemode as u32 == registered as i32 as u32 && choice > 2 as i32 {
        eprintln!("M_Episode: 4th episode requires UltimateDOOM");
        choice = 0 as i32;
    }
    state.m_menu.epi = choice;
    let menudef = &raw mut state.m_menu.defs.NewDef;
    M_SetupNextMenu(state, menudef);
}
static detailNames: [&str; 2] = ["M_GDHIGH", "M_GDLOW"];
static msgNames: [&str; 2] = ["M_MSGOFF", "M_MSGON"];
#[no_mangle]
pub unsafe fn M_DrawOptions(state: &mut GameState) {
    let __wcache1358_12 = W_CacheLumpName(state, "M_OPTTTL", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, 108 as i32, 15 as i32, __wcache1358_12);
    let __wcache1364_11 = W_CacheLumpName(
        state,
        detailNames[state.m_menu.detailLevel as usize],
        PU_CACHE as i32,
    ) as *mut patch_t;
    V_DrawPatchDirect(
        state,
        state.m_menu.defs.OptionsDef.x as i32 + 175 as i32,
        state.m_menu.defs.OptionsDef.y as i32 + LINEHEIGHT * detail as i32,
        __wcache1364_11,
    );
    let __wcache1373_10 = W_CacheLumpName(
        state,
        msgNames[state.m_menu.showMessages as usize],
        PU_CACHE as i32,
    ) as *mut patch_t;
    V_DrawPatchDirect(
        state,
        state.m_menu.defs.OptionsDef.x as i32 + 120 as i32,
        state.m_menu.defs.OptionsDef.y as i32 + LINEHEIGHT * messages as i32,
        __wcache1373_10,
    );
    let (x, y, sens) = (
        state.m_menu.defs.OptionsDef.x as i32,
        state.m_menu.defs.OptionsDef.y as i32 + LINEHEIGHT * (mousesens as i32 + 1 as i32),
        state.m_menu.mouseSensitivity,
    );
    M_DrawThermo(state, x, y, 10 as i32, sens);
    let (x, y, sz) = (
        state.m_menu.defs.OptionsDef.x as i32,
        state.m_menu.defs.OptionsDef.y as i32 + LINEHEIGHT * (scrnsize as i32 + 1 as i32),
        state.m_menu.screenSize,
    );
    M_DrawThermo(state, x, y, 9 as i32, sz);
}
#[no_mangle]
pub unsafe fn M_Options(state: &mut GameState, _choice: i32) {
    let menudef = &raw mut state.m_menu.defs.OptionsDef;
    M_SetupNextMenu(state, menudef);
}
#[no_mangle]
pub fn M_ChangeMessages(state: &mut GameState, _choice: i32) {
    state.m_menu.showMessages = 1 as i32 - state.m_menu.showMessages;
    if state.m_menu.showMessages == 0 {
        state.g_game.players[state.g_game.consoleplayer as usize].message =
            Some("Messages OFF".to_string());
    } else {
        state.g_game.players[state.g_game.consoleplayer as usize].message =
            Some("Messages ON".to_string());
    }
    state.hu_stuff.message_dontfuckwithme = true;
}
#[no_mangle]
pub unsafe fn M_EndGameResponse(state: &mut GameState, mut key: i32) {
    if key != state.m_controls.key_menu_confirm {
        return;
    }
    (*state.m_menu.currentMenu).lastOn = state.m_menu.itemOn;
    M_ClearMenus(state);
    D_StartTitle(state);
}
#[no_mangle]
pub unsafe fn M_EndGame(state: &mut GameState, _choice: i32) {
    if !state.g_game.usergame {
        S_StartSound(state, NULL, sfx_oof as i32);
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
        Some(M_EndGameResponse as unsafe fn(&mut GameState, i32) -> ()),
        true,
    );
}
#[no_mangle]
pub unsafe fn M_ReadThis(state: &mut GameState, _choice: i32) {
    let menudef = &raw mut state.m_menu.defs.ReadDef1;
    M_SetupNextMenu(state, menudef);
}
#[no_mangle]
pub unsafe fn M_ReadThis2(state: &mut GameState, _choice: i32) {
    if state.doomstat.gameversion.below_1_9()
        && state.doomstat.gamemode as u32 != commercial as i32 as u32
    {
        let menudef = &raw mut state.m_menu.defs.ReadDef2;
        M_SetupNextMenu(state, menudef);
    } else {
        M_FinishReadThis(state, 0 as i32);
    };
}
#[no_mangle]
pub unsafe fn M_FinishReadThis(state: &mut GameState, _choice: i32) {
    let menudef = &raw mut state.m_menu.defs.MainDef;
    M_SetupNextMenu(state, menudef);
}
#[no_mangle]
pub static quitsounds: [i32; 8] = [
    sfx_pldeth as i32,
    sfx_dmpain as i32,
    sfx_popain as i32,
    sfx_slop as i32,
    sfx_telept as i32,
    sfx_posit1 as i32,
    sfx_posit3 as i32,
    sfx_sgtatk as i32,
];
#[no_mangle]
pub static quitsounds2: [i32; 8] = [
    sfx_vilact as i32,
    sfx_getpow as i32,
    sfx_boscub as i32,
    sfx_slop as i32,
    sfx_skeswg as i32,
    sfx_kntdth as i32,
    sfx_bspact as i32,
    sfx_sgtatk as i32,
];
#[no_mangle]
pub unsafe fn M_QuitResponse(state: &mut GameState, mut key: i32) {
    if key != state.m_controls.key_menu_confirm {
        return;
    }
    if !state.g_game.netgame {
        if state.doomstat.gamemode as u32 == commercial as i32 as u32 {
            S_StartSound(
                state,
                NULL,
                quitsounds2[(state.d_loop.gametic >> 2 as i32 & 7 as i32) as usize],
            );
        } else {
            S_StartSound(
                state,
                NULL,
                quitsounds[(state.d_loop.gametic >> 2 as i32 & 7 as i32) as usize],
            );
        }
    }
    I_Quit(state);
}
fn M_SelectEndMessage(state: &mut GameState) -> &'static str {
    let endmsg: &'static [&'static str; 8] =
        if (if state.doomstat.gamemission as u32 == pack_chex as i32 as u32 {
            doom as i32 as u32
        } else {
            if state.doomstat.gamemission as u32 == pack_hacx as i32 as u32 {
                doom2 as i32 as u32
            } else {
                state.doomstat.gamemission as u32
            }
        }) == doom as i32 as u32
        {
            &doom1_endmsg
        } else {
            &doom2_endmsg
        };
    endmsg[(state.d_loop.gametic % NUM_QUITMESSAGES) as usize]
}
#[no_mangle]
pub fn M_QuitDOOM(state: &mut GameState, _choice: i32) {
    let msg = format!(
        "{}\n\n(press y to quit to dos.)",
        M_SelectEndMessage(state)
    );
    let routine = Some(M_QuitResponse as unsafe fn(&mut GameState, i32) -> ());
    M_StartMessage(state, &msg, routine, true);
}
#[no_mangle]
pub fn M_ChangeSensitivity(state: &mut GameState, mut choice: i32) {
    match choice {
        0 => {
            if state.m_menu.mouseSensitivity != 0 {
                state.m_menu.mouseSensitivity -= 1;
            }
        }
        1 => {
            if state.m_menu.mouseSensitivity < 9 as i32 {
                state.m_menu.mouseSensitivity += 1;
            }
        }
        _ => {}
    };
}
#[no_mangle]
pub fn M_ChangeDetail(state: &mut GameState, _choice: i32) {
    state.m_menu.detailLevel = 1 as i32 - state.m_menu.detailLevel;
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
#[no_mangle]
pub fn M_SizeDisplay(state: &mut GameState, mut choice: i32) {
    match choice {
        0 => {
            if state.m_menu.screenSize > 0 as i32 {
                state.m_menu.screenblocks -= 1;
                state.m_menu.screenSize -= 1;
            }
        }
        1 => {
            if state.m_menu.screenSize < 8 as i32 {
                state.m_menu.screenblocks += 1;
                state.m_menu.screenSize += 1;
            }
        }
        _ => {}
    }
    let (screenblocks, detail_level) = (state.m_menu.screenblocks, state.m_menu.detailLevel);
    R_SetViewSize(state, screenblocks, detail_level);
}
pub unsafe fn M_DrawThermo(
    state: &mut GameState,
    mut x: i32,
    mut y: i32,
    mut thermWidth: i32,
    mut thermDot: i32,
) {
    let mut xx: i32 = 0;
    let mut i: i32 = 0;
    xx = x;
    let __wcache1619_9 = W_CacheLumpName(state, "M_THERML", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, xx, y, __wcache1619_9);
    xx += 8 as i32;
    i = 0 as i32;
    while i < thermWidth {
        let __wcache1628_8 = W_CacheLumpName(state, "M_THERMM", PU_CACHE as i32) as *mut patch_t;
        V_DrawPatchDirect(state, xx, y, __wcache1628_8);
        xx += 8 as i32;
        i += 1;
    }
    let __wcache1637_7 = W_CacheLumpName(state, "M_THERMR", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, xx, y, __wcache1637_7);
    let __wcache1643_6 = W_CacheLumpName(state, "M_THERMO", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(state, x + 8 as i32 + thermDot * 8 as i32, y, __wcache1643_6);
}
pub unsafe fn M_DrawEmptyCell(state: &mut GameState, mut menu: *mut menu_t, mut item: i32) {
    let __wcache1651_5 = W_CacheLumpName(state, "M_CELL1", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(
        state,
        (*menu).x as i32 - 10 as i32,
        (*menu).y as i32 + item * LINEHEIGHT - 1 as i32,
        __wcache1651_5,
    );
}
pub unsafe fn M_DrawSelCell(state: &mut GameState, mut menu: *mut menu_t, mut item: i32) {
    let __wcache1659_4 = W_CacheLumpName(state, "M_CELL2", PU_CACHE as i32) as *mut patch_t;
    V_DrawPatchDirect(
        state,
        (*menu).x as i32 - 10 as i32,
        (*menu).y as i32 + item * LINEHEIGHT - 1 as i32,
        __wcache1659_4,
    );
}
pub fn M_StartMessage(
    state: &mut GameState,
    string: &str,
    routine: Option<unsafe fn(&mut GameState, i32) -> ()>,
    mut input: bool,
) {
    state.m_menu.messageLastMenuActive = state.m_menu.menuactive as i32;
    state.m_menu.messageToPrint = 1 as i32;
    state.m_menu.messageString = string.to_string();
    state.m_menu.messageRoutine = routine;
    state.m_menu.messageNeedsInput = input;
    state.m_menu.menuactive = true;
}
pub fn M_StopMessage(state: &mut GameState) {
    state.m_menu.menuactive = state.m_menu.messageLastMenuActive != 0;
    state.m_menu.messageToPrint = 0 as i32;
}
pub unsafe fn M_StringWidth(state: &mut GameState, string: &str) -> i32 {
    let mut w: i32 = 0 as i32;
    let mut c: i32 = 0;
    for b in string.bytes() {
        c = b.to_ascii_uppercase() as i32 - HU_FONTSTART;
        if c < 0 as i32 || c >= HU_FONTSIZE {
            w += 4 as i32;
        } else {
            w += (*state.hu_stuff.hu_font[c as usize]).width as i32;
        }
    }
    return w;
}
pub unsafe fn M_StringHeight(state: &mut GameState, string: &str) -> i32 {
    let mut h: i32 = 0;
    let height: i32 = (*state.hu_stuff.hu_font[0 as i32 as usize]).height as i32;
    h = height;
    for b in string.bytes() {
        if b == b'\n' {
            h += height;
        }
    }
    return h;
}
pub unsafe fn M_WriteText(state: &mut GameState, x: i32, y: i32, string: &str) {
    let mut w: i32 = 0;
    let mut c: i32 = 0;
    let mut cx: i32 = 0;
    let mut cy: i32 = 0;
    cx = x;
    cy = y;
    'outer: for b in string.bytes() {
        c = b as i32;
        if c == '\n' as i32 {
            cx = x;
            cy += 12 as i32;
        } else {
            c = (c as u8).to_ascii_uppercase() as i32 - HU_FONTSTART;
            if c < 0 as i32 || c >= HU_FONTSIZE {
                cx += 4 as i32;
            } else {
                w = (*state.hu_stuff.hu_font[c as usize]).width as i32;
                if cx + w > SCREENWIDTH {
                    break 'outer;
                }
                V_DrawPatchDirect(state, cx, cy, state.hu_stuff.hu_font[c as usize]);
                cx += w;
            }
        }
    }
}
fn IsNullKey(mut key: i32) -> bool {
    return key == KEY_PAUSE || key == KEY_CAPSLOCK || key == KEY_SCRLCK || key == KEY_NUMLOCK;
}
pub unsafe fn M_Responder(state: &mut GameState, ev: &mut event_t) -> bool {
    let mut ch: i32 = 0;
    let mut key: i32 = 0;
    let mut i: i32 = 0;
    if state.g_game.testcontrols {
        if (*ev).type_0 == EvType::ev_quit
            || (*ev).type_0 == EvType::ev_keydown
                && ((*ev).data1 == state.m_controls.key_menu_activate
                    || (*ev).data1 == state.m_controls.key_menu_quit)
        {
            I_Quit(state);
            return true;
        }
        return false;
    }
    if (*ev).type_0 == EvType::ev_quit {
        if state.m_menu.menuactive
            && state.m_menu.messageToPrint != 0
            && state.m_menu.messageRoutine
                == Some(M_QuitResponse as unsafe fn(&mut GameState, i32) -> ())
        {
            let key_menu_confirm = state.m_controls.key_menu_confirm;
            M_QuitResponse(state, key_menu_confirm);
        } else {
            S_StartSound(state, NULL, sfx_swtchn as i32);
            M_QuitDOOM(state, 0 as i32);
        }
        return true;
    }
    ch = 0 as i32;
    key = -(1 as i32);
    if (*ev).type_0 == EvType::ev_joystick
        && state.m_menu.responder_joywait < I_GetTime(state)
    {
        if (*ev).data3 < 0 as i32 {
            key = state.m_controls.key_menu_up;
            state.m_menu.responder_joywait = I_GetTime(state) + 5 as i32;
        } else if (*ev).data3 > 0 as i32 {
            key = state.m_controls.key_menu_down;
            state.m_menu.responder_joywait = I_GetTime(state) + 5 as i32;
        }
        if (*ev).data2 < 0 as i32 {
            key = state.m_controls.key_menu_left;
            state.m_menu.responder_joywait = I_GetTime(state) + 2 as i32;
        } else if (*ev).data2 > 0 as i32 {
            key = state.m_controls.key_menu_right;
            state.m_menu.responder_joywait = I_GetTime(state) + 2 as i32;
        }
        if (*ev).data1 & 1 as i32 != 0 {
            key = state.m_controls.key_menu_forward;
            state.m_menu.responder_joywait = I_GetTime(state) + 5 as i32;
        }
        if (*ev).data1 & 2 as i32 != 0 {
            key = state.m_controls.key_menu_back;
            state.m_menu.responder_joywait = I_GetTime(state) + 5 as i32;
        }
        if state.m_controls.joybmenu >= 0 as i32
            && (*ev).data1 & (1 as i32) << state.m_controls.joybmenu != 0 as i32
        {
            key = state.m_controls.key_menu_activate;
            state.m_menu.responder_joywait = I_GetTime(state) + 5 as i32;
        }
    } else if (*ev).type_0 == EvType::ev_mouse
        && state.m_menu.responder_mousewait < I_GetTime(state)
    {
        state.m_menu.responder_mousey += (*ev).data3;
        if state.m_menu.responder_mousey < state.m_menu.responder_lasty - 30 as i32 {
            key = state.m_controls.key_menu_down;
            state.m_menu.responder_mousewait = I_GetTime(state) + 5 as i32;
            state.m_menu.responder_lasty -= 30 as i32;
            state.m_menu.responder_mousey = state.m_menu.responder_lasty;
        } else if state.m_menu.responder_mousey > state.m_menu.responder_lasty + 30 as i32 {
            key = state.m_controls.key_menu_up;
            state.m_menu.responder_mousewait = I_GetTime(state) + 5 as i32;
            state.m_menu.responder_lasty += 30 as i32;
            state.m_menu.responder_mousey = state.m_menu.responder_lasty;
        }
        state.m_menu.responder_mousex += (*ev).data2;
        if state.m_menu.responder_mousex < state.m_menu.responder_lastx - 30 as i32 {
            key = state.m_controls.key_menu_left;
            state.m_menu.responder_mousewait = I_GetTime(state) + 5 as i32;
            state.m_menu.responder_lastx -= 30 as i32;
            state.m_menu.responder_mousex = state.m_menu.responder_lastx;
        } else if state.m_menu.responder_mousex > state.m_menu.responder_lastx + 30 as i32 {
            key = state.m_controls.key_menu_right;
            state.m_menu.responder_mousewait = I_GetTime(state) + 5 as i32;
            state.m_menu.responder_lastx += 30 as i32;
            state.m_menu.responder_mousex = state.m_menu.responder_lastx;
        }
        if (*ev).data1 & 1 as i32 != 0 {
            key = state.m_controls.key_menu_forward;
            state.m_menu.responder_mousewait = I_GetTime(state) + 15 as i32;
        }
        if (*ev).data1 & 2 as i32 != 0 {
            key = state.m_controls.key_menu_back;
            state.m_menu.responder_mousewait = I_GetTime(state) + 15 as i32;
        }
    } else if (*ev).type_0 == EvType::ev_keydown {
        key = (*ev).data1;
        ch = (*ev).data2;
    }
    if key == -(1 as i32) {
        return false;
    }
    if state.m_menu.saveStringEnter != 0 {
        match key {
            KEY_BACKSPACE => {
                if state.m_menu.saveCharIndex > 0 as i32 {
                    state.m_menu.saveCharIndex -= 1;
                    state.m_menu.savegamestrings[state.m_menu.saveSlot as usize]
                        .truncate(state.m_menu.saveCharIndex as usize);
                }
            }
            KEY_ESCAPE => {
                state.m_menu.saveStringEnter = 0 as i32;
                state.m_menu.savegamestrings[state.m_menu.saveSlot as usize] =
                    state.m_menu.saveOldString.clone();
            }
            KEY_ENTER => {
                state.m_menu.saveStringEnter = 0 as i32;
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
                    && (ch - HU_FONTSTART < 0 as i32 || ch - HU_FONTSTART >= HU_FONTSIZE))
                {
                    let savestr =
                        state.m_menu.savegamestrings[state.m_menu.saveSlot as usize].clone();
                    if ch >= 32 as i32
                        && ch <= 127 as i32
                        && state.m_menu.saveCharIndex < SAVESTRINGSIZE - 1 as i32
                        && M_StringWidth(state, &savestr) < (SAVESTRINGSIZE - 2 as i32) * 8 as i32
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
        if state.m_menu.messageNeedsInput {
            if key != ' ' as i32
                && key != KEY_ESCAPE
                && key != state.m_controls.key_menu_confirm
                && key != state.m_controls.key_menu_abort
            {
                return false;
            }
        }
        state.m_menu.menuactive = state.m_menu.messageLastMenuActive != 0;
        state.m_menu.messageToPrint = 0 as i32;
        if state.m_menu.messageRoutine.is_some() {
            state
                .m_menu
                .messageRoutine
                .expect("non-null function pointer")(state, key);
        }
        state.m_menu.menuactive = false;
        S_StartSound(state, NULL, sfx_swtchx as i32);
        return true;
    }
    if state.d_main.devparm && key == state.m_controls.key_menu_help
        || key != 0 as i32 && key == state.m_controls.key_menu_screenshot
    {
        G_ScreenShot(state);
        return true;
    }
    if !state.m_menu.menuactive {
        if key == state.m_controls.key_menu_decscreen {
            if state.am_map.automapactive || state.hu_stuff.chat_on {
                return false;
            }
            M_SizeDisplay(state, 0 as i32);
            S_StartSound(state, NULL, sfx_stnmov as i32);
            return true;
        } else if key == state.m_controls.key_menu_incscreen {
            if state.am_map.automapactive || state.hu_stuff.chat_on {
                return false;
            }
            M_SizeDisplay(state, 1 as i32);
            S_StartSound(state, NULL, sfx_stnmov as i32);
            return true;
        } else if key == state.m_controls.key_menu_help {
            M_StartControlPanel(state);
            if state.doomstat.gamemode as u32 == retail as i32 as u32 {
                state.m_menu.currentMenu = &raw mut state.m_menu.defs.ReadDef2;
            } else {
                state.m_menu.currentMenu = &raw mut state.m_menu.defs.ReadDef1;
            }
            state.m_menu.itemOn = 0 as i16;
            S_StartSound(state, NULL, sfx_swtchn as i32);
            return true;
        } else if key == state.m_controls.key_menu_save {
            M_StartControlPanel(state);
            S_StartSound(state, NULL, sfx_swtchn as i32);
            M_SaveGame(state, 0 as i32);
            return true;
        } else if key == state.m_controls.key_menu_load {
            M_StartControlPanel(state);
            S_StartSound(state, NULL, sfx_swtchn as i32);
            M_LoadGame(state, 0 as i32);
            return true;
        } else if key == state.m_controls.key_menu_volume {
            M_StartControlPanel(state);
            state.m_menu.currentMenu = &raw mut state.m_menu.defs.SoundDef;
            state.m_menu.itemOn = sfx_vol as i32 as i16;
            S_StartSound(state, NULL, sfx_swtchn as i32);
            return true;
        } else if key == state.m_controls.key_menu_detail {
            M_ChangeDetail(state, 0 as i32);
            S_StartSound(state, NULL, sfx_swtchn as i32);
            return true;
        } else if key == state.m_controls.key_menu_qsave {
            S_StartSound(state, NULL, sfx_swtchn as i32);
            M_QuickSave(state);
            return true;
        } else if key == state.m_controls.key_menu_endgame {
            S_StartSound(state, NULL, sfx_swtchn as i32);
            M_EndGame(state, 0 as i32);
            return true;
        } else if key == state.m_controls.key_menu_messages {
            M_ChangeMessages(state, 0 as i32);
            S_StartSound(state, NULL, sfx_swtchn as i32);
            return true;
        } else if key == state.m_controls.key_menu_qload {
            S_StartSound(state, NULL, sfx_swtchn as i32);
            M_QuickLoad(state);
            return true;
        } else if key == state.m_controls.key_menu_quit {
            S_StartSound(state, NULL, sfx_swtchn as i32);
            M_QuitDOOM(state, 0 as i32);
            return true;
        } else if key == state.m_controls.key_menu_gamma {
            state.i_video.usegamma += 1;
            if state.i_video.usegamma > 4 as i32 {
                state.i_video.usegamma = 0 as i32;
            }
            state.g_game.players[state.g_game.consoleplayer as usize].message =
                Some(gammamsg[state.i_video.usegamma as usize].to_string());
            let __wcache2009_3 = W_CacheLumpName(state, "PLAYPAL", PU_CACHE as i32) as *mut byte;
            I_SetPalette(state, __wcache2009_3);
            return true;
        }
    }
    if !state.m_menu.menuactive {
        if key == state.m_controls.key_menu_activate {
            M_StartControlPanel(state);
            S_StartSound(state, NULL, sfx_swtchn as i32);
            return true;
        }
        return false;
    }
    if key == state.m_controls.key_menu_down {
        loop {
            if state.m_menu.itemOn as i32 + 1 as i32
                > (*state.m_menu.currentMenu).numitems as i32 - 1 as i32
            {
                state.m_menu.itemOn = 0 as i16;
            } else {
                state.m_menu.itemOn += 1;
            }
            S_StartSound(state, NULL, sfx_pstop as i32);
            if !((*(*state.m_menu.currentMenu)
                .menuitems
                .offset(state.m_menu.itemOn as isize))
            .status as i32
                == -(1 as i32))
            {
                break;
            }
        }
        return true;
    } else if key == state.m_controls.key_menu_up {
        loop {
            if state.m_menu.itemOn == 0 {
                state.m_menu.itemOn =
                    ((*state.m_menu.currentMenu).numitems as i32 - 1 as i32) as i16;
            } else {
                state.m_menu.itemOn -= 1;
            }
            S_StartSound(state, NULL, sfx_pstop as i32);
            if !((*(*state.m_menu.currentMenu)
                .menuitems
                .offset(state.m_menu.itemOn as isize))
            .status as i32
                == -(1 as i32))
            {
                break;
            }
        }
        return true;
    } else if key == state.m_controls.key_menu_left {
        if (*(*state.m_menu.currentMenu)
            .menuitems
            .offset(state.m_menu.itemOn as isize))
        .routine
        .is_some()
            && (*(*state.m_menu.currentMenu)
                .menuitems
                .offset(state.m_menu.itemOn as isize))
            .status as i32
                == 2 as i32
        {
            S_StartSound(state, NULL, sfx_stnmov as i32);
            let item = (*state.m_menu.currentMenu)
                .menuitems
                .offset(state.m_menu.itemOn as isize);
            (*item).routine.expect("non-null function pointer")(state, 0 as i32);
        }
        return true;
    } else if key == state.m_controls.key_menu_right {
        if (*(*state.m_menu.currentMenu)
            .menuitems
            .offset(state.m_menu.itemOn as isize))
        .routine
        .is_some()
            && (*(*state.m_menu.currentMenu)
                .menuitems
                .offset(state.m_menu.itemOn as isize))
            .status as i32
                == 2 as i32
        {
            S_StartSound(state, NULL, sfx_stnmov as i32);
            let item = (*state.m_menu.currentMenu)
                .menuitems
                .offset(state.m_menu.itemOn as isize);
            (*item).routine.expect("non-null function pointer")(state, 1 as i32);
        }
        return true;
    } else if key == state.m_controls.key_menu_forward {
        if (*(*state.m_menu.currentMenu)
            .menuitems
            .offset(state.m_menu.itemOn as isize))
        .routine
        .is_some()
            && (*(*state.m_menu.currentMenu)
                .menuitems
                .offset(state.m_menu.itemOn as isize))
            .status as i32
                != 0
        {
            (*state.m_menu.currentMenu).lastOn = state.m_menu.itemOn;
            if (*(*state.m_menu.currentMenu)
                .menuitems
                .offset(state.m_menu.itemOn as isize))
            .status as i32
                == 2 as i32
            {
                let item = (*state.m_menu.currentMenu)
                    .menuitems
                    .offset(state.m_menu.itemOn as isize);
                (*item).routine.expect("non-null function pointer")(state, 1 as i32);
                S_StartSound(state, NULL, sfx_stnmov as i32);
            } else {
                let item = (*state.m_menu.currentMenu)
                    .menuitems
                    .offset(state.m_menu.itemOn as isize);
                let item_on = state.m_menu.itemOn as i32;
                (*item).routine.expect("non-null function pointer")(state, item_on);
                S_StartSound(state, NULL, sfx_pistol as i32);
            }
        }
        return true;
    } else if key == state.m_controls.key_menu_activate {
        (*state.m_menu.currentMenu).lastOn = state.m_menu.itemOn;
        M_ClearMenus(state);
        S_StartSound(state, NULL, sfx_swtchx as i32);
        return true;
    } else if key == state.m_controls.key_menu_back {
        (*state.m_menu.currentMenu).lastOn = state.m_menu.itemOn;
        if !(*state.m_menu.currentMenu).prevMenu.is_null() {
            state.m_menu.currentMenu = (*state.m_menu.currentMenu).prevMenu as *mut menu_t;
            state.m_menu.itemOn = (*state.m_menu.currentMenu).lastOn;
            S_StartSound(state, NULL, sfx_swtchn as i32);
        }
        return true;
    } else if ch != 0 as i32 || IsNullKey(key) {
        i = state.m_menu.itemOn as i32 + 1 as i32;
        while i < (*state.m_menu.currentMenu).numitems as i32 {
            if (*(*state.m_menu.currentMenu).menuitems.offset(i as isize)).alphaKey as i32 == ch {
                state.m_menu.itemOn = i as i16;
                S_StartSound(state, NULL, sfx_pstop as i32);
                return true;
            }
            i += 1;
        }
        i = 0 as i32;
        while i <= state.m_menu.itemOn as i32 {
            if (*(*state.m_menu.currentMenu).menuitems.offset(i as isize)).alphaKey as i32 == ch {
                state.m_menu.itemOn = i as i16;
                S_StartSound(state, NULL, sfx_pstop as i32);
                return true;
            }
            i += 1;
        }
    }
    return false;
}
pub unsafe fn M_StartControlPanel(state: &mut GameState) {
    if state.m_menu.menuactive {
        return;
    }
    state.m_menu.menuactive = true;
    state.m_menu.currentMenu = &raw mut state.m_menu.defs.MainDef;
    state.m_menu.itemOn = (*state.m_menu.currentMenu).lastOn;
}
pub unsafe fn M_Drawer(state: &mut GameState) {
    let mut i: u32 = 0;
    let mut max: u32 = 0;
    state.m_menu.inhelpscreens = false;
    if state.m_menu.messageToPrint != 0 {
        let message_string = state.m_menu.messageString.clone();
        state.m_menu.drawer_y =
            (SCREENHEIGHT / 2 as i32 - M_StringHeight(state, &message_string) / 2 as i32) as i16;
        for line in message_string.split('\n') {
            let line = if line.len() > 79 { &line[..79] } else { line };
            state.m_menu.drawer_x =
                (SCREENWIDTH / 2 as i32 - M_StringWidth(state, line) / 2 as i32) as i16;
            M_WriteText(
                state,
                state.m_menu.drawer_x as i32,
                state.m_menu.drawer_y as i32,
                line,
            );
            state.m_menu.drawer_y = (state.m_menu.drawer_y as i32
                + (*state.hu_stuff.hu_font[0 as i32 as usize]).height as i32)
                as i16;
        }
        return;
    }
    if !state.m_menu.menuactive {
        return;
    }
    if (*state.m_menu.currentMenu).routine.is_some() {
        (*state.m_menu.currentMenu)
            .routine
            .expect("non-null function pointer")(state);
    }
    state.m_menu.drawer_x = (*state.m_menu.currentMenu).x;
    state.m_menu.drawer_y = (*state.m_menu.currentMenu).y;
    max = (*state.m_menu.currentMenu).numitems as u32;
    i = 0 as u32;
    while i < max {
        let item_name = (*(*state.m_menu.currentMenu).menuitems.offset(i as isize)).name;
        if !item_name.is_empty() {
            let __wcache2221_2 =
                W_CacheLumpName(state, &item_name.as_str(), PU_CACHE as i32) as *mut patch_t;
            V_DrawPatchDirect(
                state,
                state.m_menu.drawer_x as i32,
                state.m_menu.drawer_y as i32,
                __wcache2221_2,
            );
        }
        state.m_menu.drawer_y = (state.m_menu.drawer_y as i32 + LINEHEIGHT) as i16;
        i = i.wrapping_add(1);
    }
    let __wcache2231_1 = W_CacheLumpName(
        state,
        skullName[state.m_menu.whichSkull as usize],
        PU_CACHE as i32,
    ) as *mut patch_t;
    V_DrawPatchDirect(
        state,
        state.m_menu.drawer_x as i32 + SKULLXOFF,
        (*state.m_menu.currentMenu).y as i32 - 5 as i32 + state.m_menu.itemOn as i32 * LINEHEIGHT,
        __wcache2231_1,
    );
}
pub fn M_ClearMenus(state: &mut GameState) {
    state.m_menu.menuactive = false;
}
pub unsafe fn M_SetupNextMenu(state: &mut GameState, mut menudef: *mut menu_t) {
    state.m_menu.currentMenu = menudef;
    state.m_menu.itemOn = (*state.m_menu.currentMenu).lastOn;
}
pub fn M_Ticker(state: &mut GameState) {
    state.m_menu.skullAnimCounter -= 1;
    if state.m_menu.skullAnimCounter as i32 <= 0 as i32 {
        state.m_menu.whichSkull = (state.m_menu.whichSkull as i32 ^ 1 as i32) as i16;
        state.m_menu.skullAnimCounter = 8 as i16;
    }
}
pub unsafe fn M_Init(state: &mut GameState) {
    state.m_menu.currentMenu = &raw mut state.m_menu.defs.MainDef;
    state.m_menu.menuactive = false;
    state.m_menu.itemOn = (*state.m_menu.currentMenu).lastOn;
    state.m_menu.whichSkull = 0 as i16;
    state.m_menu.skullAnimCounter = 10 as i16;
    state.m_menu.screenSize = state.m_menu.screenblocks - 3 as i32;
    state.m_menu.messageToPrint = 0 as i32;
    state.m_menu.messageString = String::new();
    state.m_menu.messageLastMenuActive = state.m_menu.menuactive as i32;
    state.m_menu.quickSaveSlot = -(1 as i32);
    match state.doomstat.gamemode as u32 {
        2 => {
            state.m_menu.menus.MainMenu[readthis as i32 as usize] =
                state.m_menu.menus.MainMenu[quitdoom as i32 as usize];
            state.m_menu.defs.MainDef.numitems -= 1;
            state.m_menu.defs.MainDef.y = (state.m_menu.defs.MainDef.y as i32 + 8 as i32) as i16;
            state.m_menu.defs.NewDef.prevMenu = &raw mut state.m_menu.defs.MainDef as *mut menu_s;
        }
        0 => {}
        1 | 3 | _ => {}
    }
    if !state.doomstat.gameversion.is_ultimate_or_higher() {
        state.m_menu.defs.EpiDef.numitems -= 1;
    }
}
