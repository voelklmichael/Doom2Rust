use crate::src::am_map::{AM_MSGENTERED, AM_MSGEXITED, AM_MSGHEADER};
use crate::src::d_event::event_t;
use crate::src::d_event::EvType;
use crate::src::d_items::{weaponinfo, weaponinfo_t};
use crate::src::d_mode::{commercial, registered, retail, shareware};
use crate::src::d_mode::{doom, doom2, pack_chex, pack_hacx};
use crate::src::d_mode::{sk_nightmare, GameVersion};
use crate::src::d_player::player_t;
use crate::src::d_player::{am_noammo, NUMAMMO};
use crate::src::d_player::{pw_invulnerability, pw_ironfeet, pw_strength};
use crate::src::d_player::{wp_chainsaw, NUMWEAPONS};
use crate::src::d_player::{CF_GODMODE, CF_NOCLIP};
use crate::src::doomdef::true_0;
use crate::src::doomdef::MAXPLAYERS;
use crate::src::doomdef::SCREENHEIGHT;
use crate::src::doomdef::SCREENWIDTH;
use crate::src::doomdef::TICRATE;
use crate::src::g_game::G_DeferedInitNew;
use crate::src::game_state::GameState;
use crate::src::hu_lib::patch_t;
use crate::src::i_video::I_SetPalette;
use crate::src::m_cheat::cheatseq_t;
use crate::src::m_cheat::cht_CheckCheat;
use crate::src::m_cheat::cht_GetParam;
use crate::src::m_random::M_Random;
use crate::src::p_inter::P_GivePower;
use crate::src::p_inter::NUMCARDS;
use crate::src::r_main::R_PointToAngle2;
use crate::src::s_sound::S_ChangeMusic;
use crate::src::sounds::{mus_e1m1, mus_runnin};
use crate::src::st_lib::STlib_init;
use crate::src::st_lib::STlib_initBinIcon;
use crate::src::st_lib::STlib_initMultIcon;
use crate::src::st_lib::STlib_initNum;
use crate::src::st_lib::STlib_initPercent;
use crate::src::st_lib::STlib_updateBinIcon;
use crate::src::st_lib::STlib_updateMultIcon;
use crate::src::st_lib::STlib_updateNum;
use crate::src::st_lib::STlib_updatePercent;
use crate::src::st_lib::{st_binicon_t, st_multicon_t, st_number_t, st_percent_t};
use crate::src::stdint_types::byte;
use crate::src::stdint_types::size_t;
use crate::src::tables::angle_t;
use crate::src::tables::ANG180;
use crate::src::tables::ANG45;
use crate::src::v_video::V_CopyRect;
use crate::src::v_video::V_DrawPatch;
use crate::src::v_video::V_RestoreBuffer;
use crate::src::v_video::V_UseBuffer;
use crate::src::w_wad::W_CacheLumpNum;
use crate::src::w_wad::{W_CacheLumpName, W_GetNumForName, W_ReleaseLumpName};
use crate::src::z_zone::Z_Malloc;
use crate::src::z_zone::{PU_CACHE, PU_STATIC};

pub struct StStuffState {
    pub st_backing_screen: *mut byte,
    pub plyr: *mut player_t,
    pub st_firsttime: bool,
    pub lu_palette: i32,
    pub st_clock: u32,
    pub st_msgcounter: i32,
    pub st_chatstate: st_chatstateenum_t,
    pub st_gamestate: st_stateenum_t,
    pub st_statusbaron: bool,
    pub st_chat: bool,
    pub st_oldchat: bool,
    pub st_cursoron: bool,
    pub st_notdeathmatch: bool,
    pub st_armson: bool,
    pub st_fragson: bool,
    pub sbar: *mut patch_t,
    pub tallnum: [*mut patch_t; 10],
    pub tallpercent: *mut patch_t,
    pub shortnum: [*mut patch_t; 10],
    pub keys: [*mut patch_t; 6],
    pub faces: [*mut patch_t; 42],
    pub faceback: *mut patch_t,
    pub armsbg: *mut patch_t,
    pub arms: [[*mut patch_t; 2]; 6],
    pub w_ready: st_number_t,
    pub w_frags: st_number_t,
    pub w_health: st_percent_t,
    pub w_armsbg: st_binicon_t,
    pub w_arms_owned: [i32; 6],
    pub w_arms: [st_multicon_t; 6],
    pub w_faces: st_multicon_t,
    pub w_keyboxes: [st_multicon_t; 3],
    pub w_armor: st_percent_t,
    pub w_ammo: [st_number_t; 4],
    pub w_maxammo: [st_number_t; 4],
    pub st_fragscount: i32,
    pub st_oldhealth: i32,
    pub oldweaponsowned: [bool; 9],
    pub st_facecount: i32,
    pub st_faceindex: i32,
    pub keyboxes: [i32; 3],
    pub st_randomnumber: i32,
    pub cheat_mus: cheatseq_t,
    pub cheat_god: cheatseq_t,
    pub cheat_ammo: cheatseq_t,
    pub cheat_ammonokey: cheatseq_t,
    pub cheat_noclip: cheatseq_t,
    pub cheat_commercial_noclip: cheatseq_t,
    pub cheat_powerup: [cheatseq_t; 7],
    pub cheat_choppers: cheatseq_t,
    pub cheat_clev: cheatseq_t,
    pub cheat_mypos: cheatseq_t,
    pub st_calcpainoffset_lastcalc: i32,
    pub st_calcpainoffset_oldhealth: i32,
    pub st_updatefacewidget_lastattackdown: i32,
    pub st_updatefacewidget_priority: i32,
    pub st_updatewidgets_largeammo: i32,
    pub st_palette: i32,
    pub st_stopped: bool,
}

impl StStuffState {
    pub const fn new() -> Self {
        StStuffState {
            st_backing_screen: ::core::ptr::null::<byte>() as *mut byte,
            plyr: ::core::ptr::null::<player_t>() as *mut player_t,
            st_firsttime: false,
            lu_palette: 0,
            st_clock: 0,
            st_msgcounter: 0,
            st_chatstate: StartChatState,
            st_gamestate: AutomapState,
            st_statusbaron: false,
            st_chat: false,
            st_oldchat: false,
            st_cursoron: false,
            st_notdeathmatch: false,
            st_armson: false,
            st_fragson: false,
            sbar: ::core::ptr::null::<patch_t>() as *mut patch_t,
            tallnum: [::core::ptr::null::<patch_t>() as *mut patch_t; 10],
            tallpercent: ::core::ptr::null::<patch_t>() as *mut patch_t,
            shortnum: [::core::ptr::null::<patch_t>() as *mut patch_t; 10],
            keys: [::core::ptr::null::<patch_t>() as *mut patch_t; 6],
            faces: [::core::ptr::null::<patch_t>() as *mut patch_t; 42],
            faceback: ::core::ptr::null::<patch_t>() as *mut patch_t,
            armsbg: ::core::ptr::null::<patch_t>() as *mut patch_t,
            arms: [[::core::ptr::null::<patch_t>() as *mut patch_t; 2]; 6],
            w_ready: st_number_t {
                x: 0,
                y: 0,
                width: 0,
                oldnum: 0,
                num: ::core::ptr::null::<i32>() as *mut i32,
                on: ::core::ptr::null::<bool>() as *mut bool,
                p: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
                data: 0,
            },
            w_frags: st_number_t {
                x: 0,
                y: 0,
                width: 0,
                oldnum: 0,
                num: ::core::ptr::null::<i32>() as *mut i32,
                on: ::core::ptr::null::<bool>() as *mut bool,
                p: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
                data: 0,
            },
            w_health: st_percent_t {
                n: st_number_t {
                    x: 0,
                    y: 0,
                    width: 0,
                    oldnum: 0,
                    num: ::core::ptr::null::<i32>() as *mut i32,
                    on: ::core::ptr::null::<bool>() as *mut bool,
                    p: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
                    data: 0,
                },
                p: ::core::ptr::null::<patch_t>() as *mut patch_t,
            },
            w_armsbg: st_binicon_t {
                x: 0,
                y: 0,
                oldval: false,
                val: ::core::ptr::null::<bool>() as *mut bool,
                on: ::core::ptr::null::<bool>() as *mut bool,
                p: ::core::ptr::null::<patch_t>() as *mut patch_t,
                data: 0,
            },
            w_arms_owned: [0; 6],
            w_arms: [st_multicon_t {
                x: 0,
                y: 0,
                oldinum: 0,
                inum: ::core::ptr::null::<i32>() as *mut i32,
                on: ::core::ptr::null::<bool>() as *mut bool,
                p: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
                data: 0,
            }; 6],
            w_faces: st_multicon_t {
                x: 0,
                y: 0,
                oldinum: 0,
                inum: ::core::ptr::null::<i32>() as *mut i32,
                on: ::core::ptr::null::<bool>() as *mut bool,
                p: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
                data: 0,
            },
            w_keyboxes: [st_multicon_t {
                x: 0,
                y: 0,
                oldinum: 0,
                inum: ::core::ptr::null::<i32>() as *mut i32,
                on: ::core::ptr::null::<bool>() as *mut bool,
                p: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
                data: 0,
            }; 3],
            w_armor: st_percent_t {
                n: st_number_t {
                    x: 0,
                    y: 0,
                    width: 0,
                    oldnum: 0,
                    num: ::core::ptr::null::<i32>() as *mut i32,
                    on: ::core::ptr::null::<bool>() as *mut bool,
                    p: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
                    data: 0,
                },
                p: ::core::ptr::null::<patch_t>() as *mut patch_t,
            },
            w_ammo: [st_number_t {
                x: 0,
                y: 0,
                width: 0,
                oldnum: 0,
                num: ::core::ptr::null::<i32>() as *mut i32,
                on: ::core::ptr::null::<bool>() as *mut bool,
                p: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
                data: 0,
            }; 4],
            w_maxammo: [st_number_t {
                x: 0,
                y: 0,
                width: 0,
                oldnum: 0,
                num: ::core::ptr::null::<i32>() as *mut i32,
                on: ::core::ptr::null::<bool>() as *mut bool,
                p: ::core::ptr::null::<*mut patch_t>() as *mut *mut patch_t,
                data: 0,
            }; 4],
            st_fragscount: 0,
            st_oldhealth: -1,
            oldweaponsowned: [false; 9],
            st_facecount: 0,
            st_faceindex: 0,
            keyboxes: [0; 3],
            st_randomnumber: 0,
            cheat_mus: cheatseq_t {
                sequence: [0; 25],
                sequence_len: 0,
                parameter_chars: 0,
                chars_read: 0,
                param_chars_read: 0,
                parameter_buf: [0; 5],
            },
            cheat_god: cheatseq_t {
                sequence: [0; 25],
                sequence_len: 0,
                parameter_chars: 0,
                chars_read: 0,
                param_chars_read: 0,
                parameter_buf: [0; 5],
            },
            cheat_ammo: cheatseq_t {
                sequence: [0; 25],
                sequence_len: 0,
                parameter_chars: 0,
                chars_read: 0,
                param_chars_read: 0,
                parameter_buf: [0; 5],
            },
            cheat_ammonokey: cheatseq_t {
                sequence: [0; 25],
                sequence_len: 0,
                parameter_chars: 0,
                chars_read: 0,
                param_chars_read: 0,
                parameter_buf: [0; 5],
            },
            cheat_noclip: cheatseq_t {
                sequence: [0; 25],
                sequence_len: 0,
                parameter_chars: 0,
                chars_read: 0,
                param_chars_read: 0,
                parameter_buf: [0; 5],
            },
            cheat_commercial_noclip: cheatseq_t {
                sequence: [0; 25],
                sequence_len: 0,
                parameter_chars: 0,
                chars_read: 0,
                param_chars_read: 0,
                parameter_buf: [0; 5],
            },
            cheat_powerup: [cheatseq_t {
                sequence: [0; 25],
                sequence_len: 0,
                parameter_chars: 0,
                chars_read: 0,
                param_chars_read: 0,
                parameter_buf: [0; 5],
            }; 7],
            cheat_choppers: cheatseq_t {
                sequence: [0; 25],
                sequence_len: 0,
                parameter_chars: 0,
                chars_read: 0,
                param_chars_read: 0,
                parameter_buf: [0; 5],
            },
            cheat_clev: cheatseq_t {
                sequence: [0; 25],
                sequence_len: 0,
                parameter_chars: 0,
                chars_read: 0,
                param_chars_read: 0,
                parameter_buf: [0; 5],
            },
            cheat_mypos: cheatseq_t {
                sequence: [0; 25],
                sequence_len: 0,
                parameter_chars: 0,
                chars_read: 0,
                param_chars_read: 0,
                parameter_buf: [0; 5],
            },
            st_calcpainoffset_lastcalc: 0,
            st_calcpainoffset_oldhealth: -1,
            st_updatefacewidget_lastattackdown: -1,
            st_updatefacewidget_priority: 0,
            st_updatewidgets_largeammo: 1994,
            st_palette: 0,
            st_stopped: true,
        }
    }
}

pub type st_stateenum_t = u32;
pub const FirstPersonState: st_stateenum_t = 1;
pub const AutomapState: st_stateenum_t = 0;
pub type st_chatstateenum_t = u32;
pub const GetChatState: st_chatstateenum_t = 2;
pub const WaitDestState: st_chatstateenum_t = 1;
pub const StartChatState: st_chatstateenum_t = 0;
pub type load_callback_t = Option<unsafe fn(&mut GameState, &str, *mut *mut patch_t) -> ()>;
pub const DEH_DEFAULT_GOD_MODE_HEALTH: i32 = 100;
pub const DEH_DEFAULT_IDFA_ARMOR: i32 = 200;
pub const DEH_DEFAULT_IDFA_ARMOR_CLASS: i32 = 2;
pub const DEH_DEFAULT_IDKFA_ARMOR: i32 = 200;
pub const DEH_DEFAULT_IDKFA_ARMOR_CLASS: i32 = 2;
pub const deh_god_mode_health: i32 = DEH_DEFAULT_GOD_MODE_HEALTH;
pub const deh_idfa_armor: i32 = DEH_DEFAULT_IDFA_ARMOR;
pub const deh_idfa_armor_class: i32 = DEH_DEFAULT_IDFA_ARMOR_CLASS;
pub const deh_idkfa_armor: i32 = DEH_DEFAULT_IDKFA_ARMOR;
pub const deh_idkfa_armor_class: i32 = DEH_DEFAULT_IDKFA_ARMOR_CLASS;
pub const ST_HEIGHT: i32 = 32;
pub const ST_WIDTH: i32 = SCREENWIDTH;
pub const ST_Y: i32 = SCREENHEIGHT - ST_HEIGHT;
pub const STARTREDPALS: i32 = 1;
pub const STARTBONUSPALS: i32 = 9;
pub const NUMREDPALS: i32 = 8;
pub const NUMBONUSPALS: i32 = 4;
pub const RADIATIONPAL: i32 = 13;
pub const ST_X: i32 = 0;
pub const ST_FX: i32 = 143;
pub const ST_NUMPAINFACES: i32 = 5;
pub const ST_NUMSTRAIGHTFACES: i32 = 3;
pub const ST_NUMTURNFACES: i32 = 2;
pub const ST_NUMSPECIALFACES: i32 = 3;
pub const ST_FACESTRIDE: i32 = ST_NUMSTRAIGHTFACES + ST_NUMTURNFACES + ST_NUMSPECIALFACES;
pub const ST_TURNOFFSET: i32 = 3;
pub const ST_OUCHOFFSET: i32 = ST_TURNOFFSET + ST_NUMTURNFACES;
pub const ST_EVILGRINOFFSET: i32 = ST_OUCHOFFSET + 1 as i32;
pub const ST_RAMPAGEOFFSET: i32 = ST_EVILGRINOFFSET + 1 as i32;
pub const ST_GODFACE: i32 = ST_NUMPAINFACES * ST_FACESTRIDE;
pub const ST_DEADFACE: i32 = ST_GODFACE + 1 as i32;
pub const ST_FACESX: i32 = 143;
pub const ST_FACESY: i32 = 168;
pub const ST_EVILGRINCOUNT: i32 = 2 * TICRATE;
pub const ST_STRAIGHTFACECOUNT: i32 = TICRATE / 2 as i32;
pub const ST_TURNCOUNT: i32 = 1 * TICRATE;
pub const ST_RAMPAGEDELAY: i32 = 2 * TICRATE;
pub const ST_MUCHPAIN: i32 = 20;
pub const ST_AMMOWIDTH: i32 = 3;
pub const ST_AMMOX: i32 = 44;
pub const ST_AMMOY: i32 = 171;
pub const ST_HEALTHX: i32 = 90;
pub const ST_HEALTHY: i32 = 171;
pub const ST_ARMSX: i32 = 111;
pub const ST_ARMSY: i32 = 172;
pub const ST_ARMSBGX: i32 = 104;
pub const ST_ARMSBGY: i32 = 168;
pub const ST_ARMSXSPACE: i32 = 12;
pub const ST_ARMSYSPACE: i32 = 10;
pub const ST_FRAGSX: i32 = 138;
pub const ST_FRAGSY: i32 = 171;
pub const ST_FRAGSWIDTH: i32 = 2;
pub const ST_ARMORX: i32 = 221;
pub const ST_ARMORY: i32 = 171;
pub const ST_KEY0X: i32 = 239;
pub const ST_KEY0Y: i32 = 171;
pub const ST_KEY1X: i32 = 239;
pub const ST_KEY1Y: i32 = 181;
pub const ST_KEY2X: i32 = 239;
pub const ST_KEY2Y: i32 = 191;
pub const ST_AMMO0WIDTH: i32 = 3;
pub const ST_AMMO0X: i32 = 288;
pub const ST_AMMO0Y: i32 = 173;
pub const ST_AMMO1WIDTH: i32 = ST_AMMO0WIDTH;
pub const ST_AMMO1X: i32 = 288;
pub const ST_AMMO1Y: i32 = 179;
pub const ST_AMMO2WIDTH: i32 = ST_AMMO0WIDTH;
pub const ST_AMMO2X: i32 = 288;
pub const ST_AMMO2Y: i32 = 191;
pub const ST_AMMO3WIDTH: i32 = ST_AMMO0WIDTH;
pub const ST_AMMO3X: i32 = 288;
pub const ST_AMMO3Y: i32 = 185;
pub const ST_MAXAMMO0WIDTH: i32 = 3;
pub const ST_MAXAMMO0X: i32 = 314;
pub const ST_MAXAMMO0Y: i32 = 173;
pub const ST_MAXAMMO1WIDTH: i32 = ST_MAXAMMO0WIDTH;
pub const ST_MAXAMMO1X: i32 = 314;
pub const ST_MAXAMMO1Y: i32 = 179;
pub const ST_MAXAMMO2WIDTH: i32 = ST_MAXAMMO0WIDTH;
pub const ST_MAXAMMO2X: i32 = 314;
pub const ST_MAXAMMO2Y: i32 = 191;
pub const ST_MAXAMMO3WIDTH: i32 = ST_MAXAMMO0WIDTH;
pub const ST_MAXAMMO3X: i32 = 314;
pub const ST_MAXAMMO3Y: i32 = 185;
pub unsafe fn ST_refreshBackground(state: &mut GameState) {
    if state.st_stuff.st_statusbaron {
        V_UseBuffer(&mut state.v_video, state.st_stuff.st_backing_screen);
        V_DrawPatch(state, ST_X, 0 as i32, state.st_stuff.sbar);
        if state.g_game.netgame {
            V_DrawPatch(state, ST_FX, 0 as i32, state.st_stuff.faceback);
        }
        V_RestoreBuffer(state);
        V_CopyRect(state,
            ST_X,
            0 as i32,
            state.st_stuff.st_backing_screen,
            ST_WIDTH,
            ST_HEIGHT,
            ST_X,
            ST_Y,
        );
    }
}
pub unsafe fn ST_Responder(state: &mut GameState, mut ev: &event_t) -> bool {
    let mut i: i32 = 0;
    if (*ev).type_0 == EvType::ev_keyup
        && (*ev).data1 as u32 & 0xffff0000 as u32 == AM_MSGHEADER as u32
    {
        match (*ev).data1 {
            AM_MSGENTERED => {
                state.st_stuff.st_gamestate = AutomapState;
                state.st_stuff.st_firsttime = true;
            }
            AM_MSGEXITED => {
                state.st_stuff.st_gamestate = FirstPersonState;
            }
            _ => {}
        }
    } else if (*ev).type_0 == EvType::ev_keydown {
        if !state.g_game.netgame && state.g_game.gameskill as i32 != sk_nightmare as i32 {
            if cht_CheckCheat(
                &raw mut state.st_stuff.cheat_god,
                (*ev).data2 as ::core::ffi::c_char,
            ) != 0
            {
                (*state.st_stuff.plyr).cheats ^= CF_GODMODE as i32;
                if (*state.st_stuff.plyr).cheats & CF_GODMODE as i32 != 0 {
                    if !(*state.st_stuff.plyr).mo.is_null() {
                        (*(*state.st_stuff.plyr).mo).health = 100 as i32;
                    }
                    (*state.st_stuff.plyr).health = deh_god_mode_health;
                    (*state.st_stuff.plyr).message = Some("Degreelessness Mode On".to_string());
                } else {
                    (*state.st_stuff.plyr).message = Some("Degreelessness Mode Off".to_string());
                }
            } else if cht_CheckCheat(
                &raw mut state.st_stuff.cheat_ammonokey,
                (*ev).data2 as ::core::ffi::c_char,
            ) != 0
            {
                (*state.st_stuff.plyr).armorpoints = deh_idfa_armor;
                (*state.st_stuff.plyr).armortype = deh_idfa_armor_class;
                i = 0 as i32;
                while i < NUMWEAPONS as i32 {
                    (*state.st_stuff.plyr).weaponowned[i as usize] = true;
                    i += 1;
                }
                i = 0 as i32;
                while i < NUMAMMO as i32 {
                    (*state.st_stuff.plyr).ammo[i as usize] =
                        (*state.st_stuff.plyr).maxammo[i as usize];
                    i += 1;
                }
                (*state.st_stuff.plyr).message = Some("Ammo (no keys) Added".to_string());
            } else if cht_CheckCheat(
                &raw mut state.st_stuff.cheat_ammo,
                (*ev).data2 as ::core::ffi::c_char,
            ) != 0
            {
                (*state.st_stuff.plyr).armorpoints = deh_idkfa_armor;
                (*state.st_stuff.plyr).armortype = deh_idkfa_armor_class;
                i = 0 as i32;
                while i < NUMWEAPONS as i32 {
                    (*state.st_stuff.plyr).weaponowned[i as usize] = true;
                    i += 1;
                }
                i = 0 as i32;
                while i < NUMAMMO as i32 {
                    (*state.st_stuff.plyr).ammo[i as usize] =
                        (*state.st_stuff.plyr).maxammo[i as usize];
                    i += 1;
                }
                i = 0 as i32;
                while i < NUMCARDS as i32 {
                    (*state.st_stuff.plyr).cards[i as usize] = true;
                    i += 1;
                }
                (*state.st_stuff.plyr).message = Some("Very Happy Ammo Added".to_string());
            } else if cht_CheckCheat(
                &raw mut state.st_stuff.cheat_mus,
                (*ev).data2 as ::core::ffi::c_char,
            ) != 0
            {
                let mut buf: [::core::ffi::c_char; 3] = [0; 3];
                let mut musnum: i32 = 0;
                (*state.st_stuff.plyr).message = Some("Music Change".to_string());
                cht_GetParam(
                    &raw mut state.st_stuff.cheat_mus,
                    &raw mut buf as *mut ::core::ffi::c_char,
                );
                if state.doomstat.gamemode as u32 == commercial as i32 as u32
                    || !state.doomstat.gameversion.is_ultimate_or_higher()
                {
                    musnum = mus_runnin as i32
                        + (buf[0 as i32 as usize] as i32 - '0' as i32) * 10 as i32
                        + buf[1 as i32 as usize] as i32
                        - '0' as i32
                        - 1 as i32;
                    if (buf[0 as i32 as usize] as i32 - '0' as i32) * 10 as i32
                        + buf[1 as i32 as usize] as i32
                        - '0' as i32
                        > 35 as i32
                    {
                        (*state.st_stuff.plyr).message = Some("IMPOSSIBLE SELECTION".to_string());
                    } else {
                        S_ChangeMusic(state, musnum, 1 as i32);
                    }
                } else {
                    musnum = mus_e1m1 as i32
                        + (buf[0 as i32 as usize] as i32 - '1' as i32) * 9 as i32
                        + (buf[1 as i32 as usize] as i32 - '1' as i32);
                    if (buf[0 as i32 as usize] as i32 - '1' as i32) * 9 as i32
                        + buf[1 as i32 as usize] as i32
                        - '1' as i32
                        > 31 as i32
                    {
                        (*state.st_stuff.plyr).message = Some("IMPOSSIBLE SELECTION".to_string());
                    } else {
                        S_ChangeMusic(state, musnum, 1 as i32);
                    }
                }
            } else if (if state.doomstat.gamemission as u32 == pack_chex as i32 as u32 {
                doom as i32 as u32
            } else {
                if state.doomstat.gamemission as u32 == pack_hacx as i32 as u32 {
                    doom2 as i32 as u32
                } else {
                    state.doomstat.gamemission as u32
                }
            }) == doom as i32 as u32
                && cht_CheckCheat(
                    &raw mut state.st_stuff.cheat_noclip,
                    (*ev).data2 as ::core::ffi::c_char,
                ) != 0
                || (if state.doomstat.gamemission as u32 == pack_chex as i32 as u32 {
                    doom as i32 as u32
                } else {
                    if state.doomstat.gamemission as u32 == pack_hacx as i32 as u32 {
                        doom2 as i32 as u32
                    } else {
                        state.doomstat.gamemission as u32
                    }
                }) != doom as i32 as u32
                    && cht_CheckCheat(
                        &raw mut state.st_stuff.cheat_commercial_noclip,
                        (*ev).data2 as ::core::ffi::c_char,
                    ) != 0
            {
                (*state.st_stuff.plyr).cheats ^= CF_NOCLIP as i32;
                if (*state.st_stuff.plyr).cheats & CF_NOCLIP as i32 != 0 {
                    (*state.st_stuff.plyr).message = Some("No Clipping Mode ON".to_string());
                } else {
                    (*state.st_stuff.plyr).message = Some("No Clipping Mode OFF".to_string());
                }
            }
            i = 0 as i32;
            while i < 6 as i32 {
                if cht_CheckCheat(
                    (&raw mut state.st_stuff.cheat_powerup as *mut cheatseq_t).offset(i as isize)
                        as *mut cheatseq_t,
                    (*ev).data2 as ::core::ffi::c_char,
                ) != 0
                {
                    if (*state.st_stuff.plyr).powers[i as usize] == 0 {
                        P_GivePower(state.st_stuff.plyr, i);
                    } else if i != pw_strength as i32 {
                        (*state.st_stuff.plyr).powers[i as usize] = 1 as i32;
                    } else {
                        (*state.st_stuff.plyr).powers[i as usize] = 0 as i32;
                    }
                    (*state.st_stuff.plyr).message = Some("Power-up Toggled".to_string());
                }
                i += 1;
            }
            if cht_CheckCheat(
                (&raw mut state.st_stuff.cheat_powerup as *mut cheatseq_t).offset(6 as i32 as isize)
                    as *mut cheatseq_t,
                (*ev).data2 as ::core::ffi::c_char,
            ) != 0
            {
                (*state.st_stuff.plyr).message = Some("inVuln, Str, Inviso, Rad, Allmap, or Lite-amp".to_string());
            } else if cht_CheckCheat(
                &raw mut state.st_stuff.cheat_choppers,
                (*ev).data2 as ::core::ffi::c_char,
            ) != 0
            {
                (*state.st_stuff.plyr).weaponowned[wp_chainsaw as i32 as usize] = true;
                (*state.st_stuff.plyr).powers[pw_invulnerability as i32 as usize] = true_0;
                (*state.st_stuff.plyr).message = Some("... doesn't suck - GM".to_string());
            } else if cht_CheckCheat(
                &raw mut state.st_stuff.cheat_mypos,
                (*ev).data2 as ::core::ffi::c_char,
            ) != 0
            {
                (*state.st_stuff.plyr).message = Some(format!(
                    "ang=0x{:x};x,y=(0x{:x},0x{:x})",
                    (*state.g_game.players[state.g_game.consoleplayer as usize].mo).angle,
                    (*state.g_game.players[state.g_game.consoleplayer as usize].mo).x,
                    (*state.g_game.players[state.g_game.consoleplayer as usize].mo).y,
                ));
            }
        }
        if !state.g_game.netgame
            && cht_CheckCheat(
                &raw mut state.st_stuff.cheat_clev,
                (*ev).data2 as ::core::ffi::c_char,
            ) != 0
        {
            let mut buf_1: [::core::ffi::c_char; 3] = [0; 3];
            let mut epsd: i32 = 0;
            let mut map: i32 = 0;
            cht_GetParam(
                &raw mut state.st_stuff.cheat_clev,
                &raw mut buf_1 as *mut ::core::ffi::c_char,
            );
            if state.doomstat.gamemode as u32 == commercial as i32 as u32 {
                epsd = 1 as i32;
                map = (buf_1[0 as i32 as usize] as i32 - '0' as i32) * 10 as i32
                    + buf_1[1 as i32 as usize] as i32
                    - '0' as i32;
            } else {
                epsd = buf_1[0 as i32 as usize] as i32 - '0' as i32;
                map = buf_1[1 as i32 as usize] as i32 - '0' as i32;
            }
            if state.doomstat.gameversion == GameVersion::chex {
                epsd = 1 as i32;
            }
            if epsd < 1 as i32 {
                return false;
            }
            if map < 1 as i32 {
                return false;
            }
            if state.doomstat.gamemode as u32 == retail as i32 as u32
                && (epsd > 4 as i32 || map > 9 as i32)
            {
                return false;
            }
            if state.doomstat.gamemode as u32 == registered as i32 as u32
                && (epsd > 3 as i32 || map > 9 as i32)
            {
                return false;
            }
            if state.doomstat.gamemode as u32 == shareware as i32 as u32
                && (epsd > 1 as i32 || map > 9 as i32)
            {
                return false;
            }
            if state.doomstat.gamemode as u32 == commercial as i32 as u32
                && (epsd > 1 as i32 || map > 40 as i32)
            {
                return false;
            }
            (*state.st_stuff.plyr).message = Some("Changing Level...".to_string());
            let gameskill = state.g_game.gameskill;
            G_DeferedInitNew(state, gameskill, epsd, map);
        }
    }
    return false;
}
pub unsafe fn ST_calcPainOffset(state: &mut GameState) -> i32 {
    let mut health: i32 = 0;
    health = if (*state.st_stuff.plyr).health > 100 as i32 {
        100 as i32
    } else {
        (*state.st_stuff.plyr).health
    };
    if health != state.st_stuff.st_calcpainoffset_oldhealth {
        state.st_stuff.st_calcpainoffset_lastcalc =
            ST_FACESTRIDE * ((100 as i32 - health) * ST_NUMPAINFACES / 101 as i32);
        state.st_stuff.st_calcpainoffset_oldhealth = health;
    }
    return state.st_stuff.st_calcpainoffset_lastcalc;
}
pub unsafe fn ST_updateFaceWidget(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut badguyangle: angle_t = 0;
    let mut diffang: angle_t = 0;
    let mut doevilgrin: bool = false;
    if state.st_stuff.st_updatefacewidget_priority < 10 as i32 {
        if (*state.st_stuff.plyr).health == 0 {
            state.st_stuff.st_updatefacewidget_priority = 9 as i32;
            state.st_stuff.st_faceindex = ST_DEADFACE;
            state.st_stuff.st_facecount = 1 as i32;
        }
    }
    if state.st_stuff.st_updatefacewidget_priority < 9 as i32 {
        if (*state.st_stuff.plyr).bonuscount != 0 {
            doevilgrin = false;
            i = 0 as i32;
            while i < NUMWEAPONS as i32 {
                if state.st_stuff.oldweaponsowned[i as usize]
                    != (*state.st_stuff.plyr).weaponowned[i as usize]
                {
                    doevilgrin = true;
                    state.st_stuff.oldweaponsowned[i as usize] =
                        (*state.st_stuff.plyr).weaponowned[i as usize];
                }
                i += 1;
            }
            if doevilgrin {
                state.st_stuff.st_updatefacewidget_priority = 8 as i32;
                state.st_stuff.st_facecount = ST_EVILGRINCOUNT;
                state.st_stuff.st_faceindex = ST_calcPainOffset(state) + ST_EVILGRINOFFSET;
            }
        }
    }
    if state.st_stuff.st_updatefacewidget_priority < 8 as i32 {
        let plyr_attacker = (*state.st_stuff.plyr)
            .attacker
            .and_then(|id| state.p_mobj.mobj_get(id));
        if (*state.st_stuff.plyr).damagecount != 0
            && plyr_attacker.is_some()
            && plyr_attacker != Some((*state.st_stuff.plyr).mo)
        {
            state.st_stuff.st_updatefacewidget_priority = 7 as i32;
            if (*state.st_stuff.plyr).health - state.st_stuff.st_oldhealth > ST_MUCHPAIN {
                state.st_stuff.st_facecount = ST_TURNCOUNT;
                state.st_stuff.st_faceindex = ST_calcPainOffset(state) + ST_OUCHOFFSET;
            } else {
                let plyr_attacker = plyr_attacker.unwrap();
                let (plyr_mo_x, plyr_mo_y) = (
                    (*(*state.st_stuff.plyr).mo).x,
                    (*(*state.st_stuff.plyr).mo).y,
                );
                badguyangle = R_PointToAngle2(
                    state,
                    plyr_mo_x,
                    plyr_mo_y,
                    (*plyr_attacker).x,
                    (*plyr_attacker).y,
                );
                if badguyangle > (*(*state.st_stuff.plyr).mo).angle {
                    diffang = badguyangle.wrapping_sub((*(*state.st_stuff.plyr).mo).angle);
                    i = (diffang > ANG180) as i32;
                } else {
                    diffang = (*(*state.st_stuff.plyr).mo).angle.wrapping_sub(badguyangle);
                    i = (diffang <= ANG180) as i32;
                }
                state.st_stuff.st_facecount = ST_TURNCOUNT;
                state.st_stuff.st_faceindex = ST_calcPainOffset(state);
                if diffang < ANG45 as angle_t {
                    state.st_stuff.st_faceindex += ST_RAMPAGEOFFSET;
                } else if i != 0 {
                    state.st_stuff.st_faceindex += ST_TURNOFFSET;
                } else {
                    state.st_stuff.st_faceindex += ST_TURNOFFSET + 1 as i32;
                }
            }
        }
    }
    if state.st_stuff.st_updatefacewidget_priority < 7 as i32 {
        if (*state.st_stuff.plyr).damagecount != 0 {
            if (*state.st_stuff.plyr).health - state.st_stuff.st_oldhealth > ST_MUCHPAIN {
                state.st_stuff.st_updatefacewidget_priority = 7 as i32;
                state.st_stuff.st_facecount = ST_TURNCOUNT;
                state.st_stuff.st_faceindex = ST_calcPainOffset(state) + ST_OUCHOFFSET;
            } else {
                state.st_stuff.st_updatefacewidget_priority = 6 as i32;
                state.st_stuff.st_facecount = ST_TURNCOUNT;
                state.st_stuff.st_faceindex = ST_calcPainOffset(state) + ST_RAMPAGEOFFSET;
            }
        }
    }
    if state.st_stuff.st_updatefacewidget_priority < 6 as i32 {
        if (*state.st_stuff.plyr).attackdown != 0 {
            if state.st_stuff.st_updatefacewidget_lastattackdown == -(1 as i32) {
                state.st_stuff.st_updatefacewidget_lastattackdown = ST_RAMPAGEDELAY;
            } else {
                state.st_stuff.st_updatefacewidget_lastattackdown -= 1;
                if state.st_stuff.st_updatefacewidget_lastattackdown == 0 {
                    state.st_stuff.st_updatefacewidget_priority = 5 as i32;
                    state.st_stuff.st_faceindex = ST_calcPainOffset(state) + ST_RAMPAGEOFFSET;
                    state.st_stuff.st_facecount = 1 as i32;
                    state.st_stuff.st_updatefacewidget_lastattackdown = 1 as i32;
                }
            }
        } else {
            state.st_stuff.st_updatefacewidget_lastattackdown = -(1 as i32);
        }
    }
    if state.st_stuff.st_updatefacewidget_priority < 5 as i32 {
        if (*state.st_stuff.plyr).cheats & CF_GODMODE as i32 != 0
            || (*state.st_stuff.plyr).powers[pw_invulnerability as i32 as usize] != 0
        {
            state.st_stuff.st_updatefacewidget_priority = 4 as i32;
            state.st_stuff.st_faceindex = ST_GODFACE;
            state.st_stuff.st_facecount = 1 as i32;
        }
    }
    if state.st_stuff.st_facecount == 0 {
        state.st_stuff.st_faceindex =
            ST_calcPainOffset(state) + state.st_stuff.st_randomnumber % 3 as i32;
        state.st_stuff.st_facecount = ST_STRAIGHTFACECOUNT;
        state.st_stuff.st_updatefacewidget_priority = 0 as i32;
    }
    state.st_stuff.st_facecount -= 1;
}
pub unsafe fn ST_updateWidgets(state: &mut GameState) {
    let mut i: i32 = 0;
    if weaponinfo[(*state.st_stuff.plyr).readyweapon as usize].ammo as u32
        == am_noammo as i32 as u32
    {
        state.st_stuff.w_ready.num = &raw mut state.st_stuff.st_updatewidgets_largeammo;
    } else {
        state.st_stuff.w_ready.num = (&raw mut (*state.st_stuff.plyr).ammo as *mut i32).offset(
            (*(&raw const weaponinfo as *mut weaponinfo_t)
                .offset((*state.st_stuff.plyr).readyweapon as isize))
            .ammo as isize,
        ) as *mut i32;
    }
    state.st_stuff.w_ready.data = (*state.st_stuff.plyr).readyweapon as i32;
    i = 0 as i32;
    while i < 6 as i32 {
        state.st_stuff.w_arms_owned[i as usize] =
            (*state.st_stuff.plyr).weaponowned[(i + 1 as i32) as usize] as i32;
        i += 1;
    }
    i = 0 as i32;
    while i < 3 as i32 {
        state.st_stuff.keyboxes[i as usize] = if (*state.st_stuff.plyr).cards[i as usize] {
            i
        } else {
            -(1 as i32)
        };
        if (*state.st_stuff.plyr).cards[(i + 3 as i32) as usize] {
            state.st_stuff.keyboxes[i as usize] = i + 3 as i32;
        }
        i += 1;
    }
    ST_updateFaceWidget(state);
    state.st_stuff.st_notdeathmatch = state.g_game.deathmatch == 0;
    state.st_stuff.st_armson = state.st_stuff.st_statusbaron && state.g_game.deathmatch == 0;
    state.st_stuff.st_fragson = state.g_game.deathmatch != 0 && state.st_stuff.st_statusbaron;
    state.st_stuff.st_fragscount = 0 as i32;
    i = 0 as i32;
    while i < MAXPLAYERS {
        if i != state.g_game.consoleplayer {
            state.st_stuff.st_fragscount += (*state.st_stuff.plyr).frags[i as usize];
        } else {
            state.st_stuff.st_fragscount -= (*state.st_stuff.plyr).frags[i as usize];
        }
        i += 1;
    }
    state.st_stuff.st_msgcounter -= 1;
    if state.st_stuff.st_msgcounter == 0 {
        state.st_stuff.st_chat = state.st_stuff.st_oldchat;
    }
}
pub unsafe fn ST_Ticker(state: &mut GameState) {
    state.st_stuff.st_clock = state.st_stuff.st_clock.wrapping_add(1);
    state.st_stuff.st_randomnumber = M_Random(&mut state.m_random);
    ST_updateWidgets(state);
    state.st_stuff.st_oldhealth = (*state.st_stuff.plyr).health;
}
pub unsafe fn ST_doPaletteStuff(state: &mut GameState) {
    let mut palette: i32 = 0;
    let mut pal: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut cnt: i32 = 0;
    let mut bzc: i32 = 0;
    cnt = (*state.st_stuff.plyr).damagecount;
    if (*state.st_stuff.plyr).powers[pw_strength as i32 as usize] != 0 {
        bzc = 12 as i32 - ((*state.st_stuff.plyr).powers[pw_strength as i32 as usize] >> 6 as i32);
        if bzc > cnt {
            cnt = bzc;
        }
    }
    if cnt != 0 {
        palette = cnt + 7 as i32 >> 3 as i32;
        if palette >= NUMREDPALS {
            palette = NUMREDPALS - 1 as i32;
        }
        palette += STARTREDPALS;
    } else if (*state.st_stuff.plyr).bonuscount != 0 {
        palette = (*state.st_stuff.plyr).bonuscount + 7 as i32 >> 3 as i32;
        if palette >= NUMBONUSPALS {
            palette = NUMBONUSPALS - 1 as i32;
        }
        palette += STARTBONUSPALS;
    } else if (*state.st_stuff.plyr).powers[pw_ironfeet as i32 as usize] > 4 as i32 * 32 as i32
        || (*state.st_stuff.plyr).powers[pw_ironfeet as i32 as usize] & 8 as i32 != 0
    {
        palette = RADIATIONPAL;
    } else {
        palette = 0 as i32;
    }
    if state.doomstat.gameversion == GameVersion::chex
        && palette >= STARTREDPALS
        && palette < STARTREDPALS + NUMREDPALS
    {
        palette = RADIATIONPAL;
    }
    if palette != state.st_stuff.st_palette {
        state.st_stuff.st_palette = palette;
        pal = (W_CacheLumpNum(state, state.st_stuff.lu_palette, PU_CACHE as i32) as *mut byte)
            .offset((palette * 768 as i32) as isize);
        I_SetPalette(state, pal);
    }
}
pub unsafe fn ST_drawWidgets(state: &mut GameState, mut refresh: bool) {
    let mut i: i32 = 0;
    state.st_stuff.st_armson = state.st_stuff.st_statusbaron && state.g_game.deathmatch == 0;
    state.st_stuff.st_fragson = state.g_game.deathmatch != 0 && state.st_stuff.st_statusbaron;
    let w_ready = &raw mut state.st_stuff.w_ready;
    STlib_updateNum(state, w_ready);
    i = 0 as i32;
    while i < 4 as i32 {
        let w_ammo = (&raw mut state.st_stuff.w_ammo as *mut st_number_t).offset(i as isize)
            as *mut st_number_t;
        STlib_updateNum(state, w_ammo);
        let w_maxammo = (&raw mut state.st_stuff.w_maxammo as *mut st_number_t).offset(i as isize)
            as *mut st_number_t;
        STlib_updateNum(state, w_maxammo);
        i += 1;
    }
    let w_health = &raw mut state.st_stuff.w_health;
    STlib_updatePercent(state, w_health, refresh as i32);
    let w_armor = &raw mut state.st_stuff.w_armor;
    STlib_updatePercent(state, w_armor, refresh as i32);
    let w_armsbg = &raw mut state.st_stuff.w_armsbg;
    STlib_updateBinIcon(state, w_armsbg, refresh);
    i = 0 as i32;
    while i < 6 as i32 {
        let w_arms = (&raw mut state.st_stuff.w_arms as *mut st_multicon_t).offset(i as isize)
            as *mut st_multicon_t;
        STlib_updateMultIcon(state, w_arms, refresh);
        i += 1;
    }
    let w_faces = &raw mut state.st_stuff.w_faces;
    STlib_updateMultIcon(state, w_faces, refresh);
    i = 0 as i32;
    while i < 3 as i32 {
        let w_keyboxes = (&raw mut state.st_stuff.w_keyboxes as *mut st_multicon_t)
            .offset(i as isize) as *mut st_multicon_t;
        STlib_updateMultIcon(state, w_keyboxes, refresh);
        i += 1;
    }
    let w_frags = &raw mut state.st_stuff.w_frags;
    STlib_updateNum(state, w_frags);
}
pub unsafe fn ST_doRefresh(state: &mut GameState) {
    state.st_stuff.st_firsttime = false;
    ST_refreshBackground(state);
    ST_drawWidgets(state, true);
}
pub unsafe fn ST_diffDraw(state: &mut GameState) {
    ST_drawWidgets(state, false);
}
pub unsafe fn ST_Drawer(state: &mut GameState, mut fullscreen: bool, mut refresh: bool) {
    state.st_stuff.st_statusbaron = !fullscreen || state.am_map.automapactive;
    state.st_stuff.st_firsttime = state.st_stuff.st_firsttime || refresh;
    ST_doPaletteStuff(state);
    if state.st_stuff.st_firsttime {
        ST_doRefresh(state);
    } else {
        ST_diffDraw(state);
    };
}
unsafe fn ST_loadUnloadGraphics(state: &mut GameState, mut callback: load_callback_t) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut facenum: i32 = 0;
    i = 0 as i32;
    while i < 10 as i32 {
        let cb_ptr = (&raw mut state.st_stuff.tallnum as *mut *mut patch_t).offset(i as isize)
                as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, &format!("STTNUM{}", i,), cb_ptr);
        let cb_ptr = (&raw mut state.st_stuff.shortnum as *mut *mut patch_t).offset(i as isize)
                as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, &format!("STYSNUM{}", i,), cb_ptr);
        i += 1;
    }
    let cb_ptr = &raw mut state.st_stuff.tallpercent;
    callback.expect("non-null function pointer")(state, "STTPRCNT", cb_ptr);
    i = 0 as i32;
    while i < NUMCARDS as i32 {
        let cb_ptr = (&raw mut state.st_stuff.keys as *mut *mut patch_t).offset(i as isize)
                as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, &format!("STKEYS{}", i,), cb_ptr);
        i += 1;
    }
    let cb_ptr = &raw mut state.st_stuff.armsbg;
    callback.expect("non-null function pointer")(state, "STARMS", cb_ptr);
    i = 0 as i32;
    while i < 6 as i32 {
        let cb_ptr = (&raw mut *(&raw mut state.st_stuff.arms as *mut [*mut patch_t; 2]).offset(i as isize)
                as *mut *mut patch_t)
                .offset(0 as i32 as isize) as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, &format!("STGNUM{}", i + 2 as i32,), cb_ptr);
        state.st_stuff.arms[i as usize][1 as i32 as usize] =
            state.st_stuff.shortnum[(i + 2 as i32) as usize];
        i += 1;
    }
    let cb_ptr = &raw mut state.st_stuff.faceback;
    callback.expect("non-null function pointer")(state, &format!("STFB{}", state.g_game.consoleplayer,), cb_ptr);
    let cb_ptr = &raw mut state.st_stuff.sbar;
    callback.expect("non-null function pointer")(state, "STBAR", cb_ptr);
    facenum = 0 as i32;
    i = 0 as i32;
    while i < ST_NUMPAINFACES {
        j = 0 as i32;
        while j < ST_NUMSTRAIGHTFACES {
            let cb_ptr = (&raw mut state.st_stuff.faces as *mut *mut patch_t).offset(facenum as isize)
                    as *mut *mut patch_t;
            callback.expect("non-null function pointer")(state, &format!("STFST{}{}", i,
                j,), cb_ptr);
            facenum += 1;
            j += 1;
        }
        let cb_ptr = (&raw mut state.st_stuff.faces as *mut *mut patch_t).offset(facenum as isize)
                as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, &format!("STFTR{}0", i,), cb_ptr);
        facenum += 1;
        let cb_ptr = (&raw mut state.st_stuff.faces as *mut *mut patch_t).offset(facenum as isize)
                as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, &format!("STFTL{}0", i,), cb_ptr);
        facenum += 1;
        let cb_ptr = (&raw mut state.st_stuff.faces as *mut *mut patch_t).offset(facenum as isize)
                as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, &format!("STFOUCH{}", i,), cb_ptr);
        facenum += 1;
        let cb_ptr = (&raw mut state.st_stuff.faces as *mut *mut patch_t).offset(facenum as isize)
                as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, &format!("STFEVL{}", i,), cb_ptr);
        facenum += 1;
        let cb_ptr = (&raw mut state.st_stuff.faces as *mut *mut patch_t).offset(facenum as isize)
                as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, &format!("STFKILL{}", i,), cb_ptr);
        facenum += 1;
        i += 1;
    }
    let cb_ptr = (&raw mut state.st_stuff.faces as *mut *mut patch_t).offset(facenum as isize)
            as *mut *mut patch_t;
    callback.expect("non-null function pointer")(state, "STFGOD0", cb_ptr);
    facenum += 1;
    let cb_ptr = (&raw mut state.st_stuff.faces as *mut *mut patch_t).offset(facenum as isize)
            as *mut *mut patch_t;
    callback.expect("non-null function pointer")(state, "STFDEAD0", cb_ptr);
    facenum += 1;
}
unsafe fn ST_loadCallback(state: &mut GameState, lumpname: &str, variable: *mut *mut patch_t) {
    *variable = W_CacheLumpName(state, lumpname, PU_STATIC as i32) as *mut patch_t;
}
pub unsafe fn ST_loadGraphics(state: &mut GameState) {
    ST_loadUnloadGraphics(
        state,
        Some(ST_loadCallback as unsafe fn(&mut GameState, &str, *mut *mut patch_t) -> ()),
    );
}
pub unsafe fn ST_loadData(state: &mut GameState) {
    state.st_stuff.lu_palette = W_GetNumForName(&mut state.w_wad, "PLAYPAL");
    ST_loadGraphics(state);
}
unsafe fn ST_unloadCallback(state: &mut GameState, lumpname: &str, variable: *mut *mut patch_t) {
    W_ReleaseLumpName(&mut state.w_wad, lumpname);
    *variable = ::core::ptr::null_mut::<patch_t>();
}
pub unsafe fn ST_unloadGraphics(state: &mut GameState) {
    ST_loadUnloadGraphics(
        state,
        Some(ST_unloadCallback as unsafe fn(&mut GameState, &str, *mut *mut patch_t) -> ()),
    );
}
pub unsafe fn ST_unloadData(state: &mut GameState) {
    ST_unloadGraphics(state);
}
pub unsafe fn ST_initData(state: &mut GameState) {
    let mut i: i32 = 0;
    state.st_stuff.st_firsttime = true;
    state.st_stuff.plyr = (&raw mut state.g_game.players as *mut player_t)
        .offset(state.g_game.consoleplayer as isize) as *mut player_t;
    state.st_stuff.st_clock = 0 as u32;
    state.st_stuff.st_chatstate = StartChatState;
    state.st_stuff.st_gamestate = FirstPersonState;
    state.st_stuff.st_statusbaron = true;
    state.st_stuff.st_chat = false;
    state.st_stuff.st_oldchat = state.st_stuff.st_chat;
    state.st_stuff.st_cursoron = false;
    state.st_stuff.st_faceindex = 0 as i32;
    state.st_stuff.st_palette = -(1 as i32);
    state.st_stuff.st_oldhealth = -(1 as i32);
    i = 0 as i32;
    while i < NUMWEAPONS as i32 {
        state.st_stuff.oldweaponsowned[i as usize] = (*state.st_stuff.plyr).weaponowned[i as usize];
        i += 1;
    }
    i = 0 as i32;
    while i < 3 as i32 {
        state.st_stuff.keyboxes[i as usize] = -(1 as i32);
        i += 1;
    }
    STlib_init(state);
}
pub unsafe fn ST_createWidgets(state: &mut GameState) {
    let mut i: i32 = 0;
    STlib_initNum(
        &raw mut state.st_stuff.w_ready,
        ST_AMMOX,
        ST_AMMOY,
        &raw mut state.st_stuff.tallnum as *mut *mut patch_t,
        (&raw mut (*state.st_stuff.plyr).ammo as *mut i32).offset(
            (*(&raw const weaponinfo as *mut weaponinfo_t)
                .offset((*state.st_stuff.plyr).readyweapon as isize))
            .ammo as isize,
        ) as *mut i32,
        &raw mut state.st_stuff.st_statusbaron,
        ST_AMMOWIDTH,
    );
    state.st_stuff.w_ready.data = (*state.st_stuff.plyr).readyweapon as i32;
    STlib_initPercent(
        &raw mut state.st_stuff.w_health,
        ST_HEALTHX,
        ST_HEALTHY,
        &raw mut state.st_stuff.tallnum as *mut *mut patch_t,
        &raw mut (*state.st_stuff.plyr).health,
        &raw mut state.st_stuff.st_statusbaron,
        state.st_stuff.tallpercent,
    );
    STlib_initBinIcon(
        &raw mut state.st_stuff.w_armsbg,
        ST_ARMSBGX,
        ST_ARMSBGY,
        state.st_stuff.armsbg,
        &raw mut state.st_stuff.st_notdeathmatch,
        &raw mut state.st_stuff.st_statusbaron,
    );
    i = 0 as i32;
    while i < 6 as i32 {
        STlib_initMultIcon(
            (&raw mut state.st_stuff.w_arms as *mut st_multicon_t).offset(i as isize)
                as *mut st_multicon_t,
            ST_ARMSX + i % 3 as i32 * ST_ARMSXSPACE,
            ST_ARMSY + i / 3 as i32 * ST_ARMSYSPACE,
            &raw mut *(&raw mut state.st_stuff.arms as *mut [*mut patch_t; 2]).offset(i as isize)
                as *mut *mut patch_t,
            (&raw mut state.st_stuff.w_arms_owned as *mut i32).offset(i as isize),
            &raw mut state.st_stuff.st_armson,
        );
        i += 1;
    }
    STlib_initNum(
        &raw mut state.st_stuff.w_frags,
        ST_FRAGSX,
        ST_FRAGSY,
        &raw mut state.st_stuff.tallnum as *mut *mut patch_t,
        &raw mut state.st_stuff.st_fragscount,
        &raw mut state.st_stuff.st_fragson,
        ST_FRAGSWIDTH,
    );
    STlib_initMultIcon(
        &raw mut state.st_stuff.w_faces,
        ST_FACESX,
        ST_FACESY,
        &raw mut state.st_stuff.faces as *mut *mut patch_t,
        &raw mut state.st_stuff.st_faceindex,
        &raw mut state.st_stuff.st_statusbaron,
    );
    STlib_initPercent(
        &raw mut state.st_stuff.w_armor,
        ST_ARMORX,
        ST_ARMORY,
        &raw mut state.st_stuff.tallnum as *mut *mut patch_t,
        &raw mut (*state.st_stuff.plyr).armorpoints,
        &raw mut state.st_stuff.st_statusbaron,
        state.st_stuff.tallpercent,
    );
    STlib_initMultIcon(
        (&raw mut state.st_stuff.w_keyboxes as *mut st_multicon_t).offset(0 as i32 as isize)
            as *mut st_multicon_t,
        ST_KEY0X,
        ST_KEY0Y,
        &raw mut state.st_stuff.keys as *mut *mut patch_t,
        (&raw mut state.st_stuff.keyboxes as *mut i32).offset(0 as i32 as isize) as *mut i32,
        &raw mut state.st_stuff.st_statusbaron,
    );
    STlib_initMultIcon(
        (&raw mut state.st_stuff.w_keyboxes as *mut st_multicon_t).offset(1 as i32 as isize)
            as *mut st_multicon_t,
        ST_KEY1X,
        ST_KEY1Y,
        &raw mut state.st_stuff.keys as *mut *mut patch_t,
        (&raw mut state.st_stuff.keyboxes as *mut i32).offset(1 as i32 as isize) as *mut i32,
        &raw mut state.st_stuff.st_statusbaron,
    );
    STlib_initMultIcon(
        (&raw mut state.st_stuff.w_keyboxes as *mut st_multicon_t).offset(2 as i32 as isize)
            as *mut st_multicon_t,
        ST_KEY2X,
        ST_KEY2Y,
        &raw mut state.st_stuff.keys as *mut *mut patch_t,
        (&raw mut state.st_stuff.keyboxes as *mut i32).offset(2 as i32 as isize) as *mut i32,
        &raw mut state.st_stuff.st_statusbaron,
    );
    STlib_initNum(
        (&raw mut state.st_stuff.w_ammo as *mut st_number_t).offset(0 as i32 as isize)
            as *mut st_number_t,
        ST_AMMO0X,
        ST_AMMO0Y,
        &raw mut state.st_stuff.shortnum as *mut *mut patch_t,
        (&raw mut (*state.st_stuff.plyr).ammo as *mut i32).offset(0 as i32 as isize) as *mut i32,
        &raw mut state.st_stuff.st_statusbaron,
        ST_AMMO0WIDTH,
    );
    STlib_initNum(
        (&raw mut state.st_stuff.w_ammo as *mut st_number_t).offset(1 as i32 as isize)
            as *mut st_number_t,
        ST_AMMO1X,
        ST_AMMO1Y,
        &raw mut state.st_stuff.shortnum as *mut *mut patch_t,
        (&raw mut (*state.st_stuff.plyr).ammo as *mut i32).offset(1 as i32 as isize) as *mut i32,
        &raw mut state.st_stuff.st_statusbaron,
        ST_AMMO1WIDTH,
    );
    STlib_initNum(
        (&raw mut state.st_stuff.w_ammo as *mut st_number_t).offset(2 as i32 as isize)
            as *mut st_number_t,
        ST_AMMO2X,
        ST_AMMO2Y,
        &raw mut state.st_stuff.shortnum as *mut *mut patch_t,
        (&raw mut (*state.st_stuff.plyr).ammo as *mut i32).offset(2 as i32 as isize) as *mut i32,
        &raw mut state.st_stuff.st_statusbaron,
        ST_AMMO2WIDTH,
    );
    STlib_initNum(
        (&raw mut state.st_stuff.w_ammo as *mut st_number_t).offset(3 as i32 as isize)
            as *mut st_number_t,
        ST_AMMO3X,
        ST_AMMO3Y,
        &raw mut state.st_stuff.shortnum as *mut *mut patch_t,
        (&raw mut (*state.st_stuff.plyr).ammo as *mut i32).offset(3 as i32 as isize) as *mut i32,
        &raw mut state.st_stuff.st_statusbaron,
        ST_AMMO3WIDTH,
    );
    STlib_initNum(
        (&raw mut state.st_stuff.w_maxammo as *mut st_number_t).offset(0 as i32 as isize)
            as *mut st_number_t,
        ST_MAXAMMO0X,
        ST_MAXAMMO0Y,
        &raw mut state.st_stuff.shortnum as *mut *mut patch_t,
        (&raw mut (*state.st_stuff.plyr).maxammo as *mut i32).offset(0 as i32 as isize) as *mut i32,
        &raw mut state.st_stuff.st_statusbaron,
        ST_MAXAMMO0WIDTH,
    );
    STlib_initNum(
        (&raw mut state.st_stuff.w_maxammo as *mut st_number_t).offset(1 as i32 as isize)
            as *mut st_number_t,
        ST_MAXAMMO1X,
        ST_MAXAMMO1Y,
        &raw mut state.st_stuff.shortnum as *mut *mut patch_t,
        (&raw mut (*state.st_stuff.plyr).maxammo as *mut i32).offset(1 as i32 as isize) as *mut i32,
        &raw mut state.st_stuff.st_statusbaron,
        ST_MAXAMMO1WIDTH,
    );
    STlib_initNum(
        (&raw mut state.st_stuff.w_maxammo as *mut st_number_t).offset(2 as i32 as isize)
            as *mut st_number_t,
        ST_MAXAMMO2X,
        ST_MAXAMMO2Y,
        &raw mut state.st_stuff.shortnum as *mut *mut patch_t,
        (&raw mut (*state.st_stuff.plyr).maxammo as *mut i32).offset(2 as i32 as isize) as *mut i32,
        &raw mut state.st_stuff.st_statusbaron,
        ST_MAXAMMO2WIDTH,
    );
    STlib_initNum(
        (&raw mut state.st_stuff.w_maxammo as *mut st_number_t).offset(3 as i32 as isize)
            as *mut st_number_t,
        ST_MAXAMMO3X,
        ST_MAXAMMO3Y,
        &raw mut state.st_stuff.shortnum as *mut *mut patch_t,
        (&raw mut (*state.st_stuff.plyr).maxammo as *mut i32).offset(3 as i32 as isize) as *mut i32,
        &raw mut state.st_stuff.st_statusbaron,
        ST_MAXAMMO3WIDTH,
    );
}
pub unsafe fn ST_Start(state: &mut GameState) {
    if !state.st_stuff.st_stopped {
        ST_Stop(state);
    }
    ST_initData(state);
    ST_createWidgets(state);
    state.st_stuff.st_stopped = false;
}
pub unsafe fn ST_Stop(state: &mut GameState) {
    if state.st_stuff.st_stopped {
        return;
    }
    let __wcache1480_1 = W_CacheLumpNum(state, state.st_stuff.lu_palette, PU_CACHE as i32) as *mut byte;
    I_SetPalette(
        state,
        __wcache1480_1,
    );
    state.st_stuff.st_stopped = true;
}
pub unsafe fn ST_Init(state: &mut GameState) {
    ST_loadData(state);
    state.st_stuff.st_backing_screen = Z_Malloc(
        &mut state.z_zone,
        ST_WIDTH * ST_HEIGHT,
        PU_STATIC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut byte;
}
pub unsafe fn fixup_cheat_sequences(state: &mut GameState) {
    state.st_stuff.cheat_clev = cheatseq_t {
        sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
            *b"idclev\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 7]>() as size_t)
            .wrapping_sub(1 as size_t),
        parameter_chars: 2 as i32,
        chars_read: 0 as size_t,
        param_chars_read: 0 as i32,
        parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"\0\0\0\0\0"),
    };
    state.st_stuff.cheat_mypos = cheatseq_t {
        sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
            *b"idmypos\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 8]>() as size_t)
            .wrapping_sub(1 as size_t),
        parameter_chars: 0 as i32,
        chars_read: 0 as size_t,
        param_chars_read: 0 as i32,
        parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"\0\0\0\0\0"),
    };
    state.st_stuff.cheat_choppers = cheatseq_t {
        sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
            *b"idchoppers\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 11]>() as size_t)
            .wrapping_sub(1 as size_t),
        parameter_chars: 0 as i32,
        chars_read: 0 as size_t,
        param_chars_read: 0 as i32,
        parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"\0\0\0\0\0"),
    };
    state.st_stuff.cheat_powerup = [
        cheatseq_t {
            sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
                *b"idbeholdv\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 10]>() as size_t)
                .wrapping_sub(1 as size_t),
            parameter_chars: 0 as i32,
            chars_read: 0 as size_t,
            param_chars_read: 0 as i32,
            parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(
                *b"\0\0\0\0\0",
            ),
        },
        cheatseq_t {
            sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
                *b"idbeholds\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 10]>() as size_t)
                .wrapping_sub(1 as size_t),
            parameter_chars: 0 as i32,
            chars_read: 0 as size_t,
            param_chars_read: 0 as i32,
            parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(
                *b"\0\0\0\0\0",
            ),
        },
        cheatseq_t {
            sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
                *b"idbeholdi\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 10]>() as size_t)
                .wrapping_sub(1 as size_t),
            parameter_chars: 0 as i32,
            chars_read: 0 as size_t,
            param_chars_read: 0 as i32,
            parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(
                *b"\0\0\0\0\0",
            ),
        },
        cheatseq_t {
            sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
                *b"idbeholdr\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 10]>() as size_t)
                .wrapping_sub(1 as size_t),
            parameter_chars: 0 as i32,
            chars_read: 0 as size_t,
            param_chars_read: 0 as i32,
            parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(
                *b"\0\0\0\0\0",
            ),
        },
        cheatseq_t {
            sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
                *b"idbeholda\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 10]>() as size_t)
                .wrapping_sub(1 as size_t),
            parameter_chars: 0 as i32,
            chars_read: 0 as size_t,
            param_chars_read: 0 as i32,
            parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(
                *b"\0\0\0\0\0",
            ),
        },
        cheatseq_t {
            sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
                *b"idbeholdl\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 10]>() as size_t)
                .wrapping_sub(1 as size_t),
            parameter_chars: 0 as i32,
            chars_read: 0 as size_t,
            param_chars_read: 0 as i32,
            parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(
                *b"\0\0\0\0\0",
            ),
        },
        cheatseq_t {
            sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
                *b"idbehold\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 9]>() as size_t)
                .wrapping_sub(1 as size_t),
            parameter_chars: 0 as i32,
            chars_read: 0 as size_t,
            param_chars_read: 0 as i32,
            parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(
                *b"\0\0\0\0\0",
            ),
        },
    ];
    state.st_stuff.cheat_commercial_noclip = cheatseq_t {
        sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
            *b"idclip\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 7]>() as size_t)
            .wrapping_sub(1 as size_t),
        parameter_chars: 0 as i32,
        chars_read: 0 as size_t,
        param_chars_read: 0 as i32,
        parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"\0\0\0\0\0"),
    };
    state.st_stuff.cheat_noclip = cheatseq_t {
        sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
            *b"idspispopd\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 11]>() as size_t)
            .wrapping_sub(1 as size_t),
        parameter_chars: 0 as i32,
        chars_read: 0 as size_t,
        param_chars_read: 0 as i32,
        parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"\0\0\0\0\0"),
    };
    state.st_stuff.cheat_mus = cheatseq_t {
        sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
            *b"idmus\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 6]>() as size_t)
            .wrapping_sub(1 as size_t),
        parameter_chars: 2 as i32,
        chars_read: 0 as size_t,
        param_chars_read: 0 as i32,
        parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"\0\0\0\0\0"),
    };
    state.st_stuff.cheat_ammo = cheatseq_t {
        sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
            *b"idkfa\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 6]>() as size_t)
            .wrapping_sub(1 as size_t),
        parameter_chars: 0 as i32,
        chars_read: 0 as size_t,
        param_chars_read: 0 as i32,
        parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"\0\0\0\0\0"),
    };
    state.st_stuff.cheat_ammonokey = cheatseq_t {
        sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
            *b"idfa\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 5]>() as size_t)
            .wrapping_sub(1 as size_t),
        parameter_chars: 0 as i32,
        chars_read: 0 as size_t,
        param_chars_read: 0 as i32,
        parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"\0\0\0\0\0"),
    };
    state.st_stuff.cheat_god = cheatseq_t {
        sequence: ::core::mem::transmute::<[u8; 25], [::core::ffi::c_char; 25]>(
            *b"iddqd\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
        sequence_len: (::core::mem::size_of::<[::core::ffi::c_char; 6]>() as size_t)
            .wrapping_sub(1 as size_t),
        parameter_chars: 0 as i32,
        chars_read: 0 as size_t,
        param_chars_read: 0 as i32,
        parameter_buf: ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"\0\0\0\0\0"),
    };
}
