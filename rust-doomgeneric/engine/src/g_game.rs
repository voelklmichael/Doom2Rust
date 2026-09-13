use crate::src::am_map::AM_Responder;
use crate::src::am_map::AM_Stop;
use crate::src::am_map::AM_Ticker;
use crate::src::d_event::event_t;
use crate::src::d_event::GameScreenState;
use crate::src::d_event::EvType;
use crate::src::d_event::GameAction;
use crate::src::d_loop::BACKUPTICS;
use crate::src::d_main::D_AdvanceDemo;
use crate::src::d_main::D_PageTicker;
use crate::src::d_mode::GameVersion;
use crate::src::d_mode::GameMode_t;
use crate::src::d_mode::GameMission_t;
use crate::src::d_mode::{skill_from_raw, SkillType};
use crate::src::d_player::PowerType;
use crate::src::d_player::{ammotype_t, NUMAMMO};
use crate::src::d_player::{player_s, player_t, PlayerId, PlayerState};
use crate::src::d_player::weapontype_t;
use crate::src::d_ticcmd::ticcmd_t;
use crate::src::d_ticcmd::{
    BTS_PAUSE, BTS_SAVEGAME, BTS_SAVEMASK, BTS_SAVESHIFT, BT_ATTACK, BT_CHANGE, BT_SPECIAL,
    BT_SPECIALMASK, BT_USE, BT_WEAPONSHIFT,
};
use crate::src::doomdef::boolean;
use crate::src::doomdef::false_0;
use crate::src::doomdef::true_0;
use crate::src::doomdef::MAXPLAYERS;
use crate::src::doomdef::NULL;
use crate::src::doomdef::TICRATE;
use crate::src::doomstat::DoomstatState;
use crate::src::f_finale::F_Responder;
use crate::src::f_finale::F_StartFinale;
use crate::src::f_finale::F_Ticker;
use crate::src::game_state::GameState;
use crate::src::hu_stuff::HU_Responder;
use crate::src::hu_stuff::HU_Ticker;
use crate::src::hu_stuff::HU_dequeueChatChar;
use crate::src::i_system::I_Error;
use crate::src::i_system::I_Quit;
use crate::src::i_timer::I_GetTime;
use crate::src::info::{S_SARG_PAIN2, S_SARG_RUN1};
use crate::src::m_argv::{M_ArgvAtoi, M_CheckParm, M_CheckParmWithArgs};
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FRACBITS;
use crate::src::m_fixed::FRACUNIT;
use crate::src::m_menu::M_StartControlPanel;
use crate::src::m_misc::M_TempFile;
use crate::src::m_misc::M_WriteFile;
use crate::src::m_random::M_ClearRandom;
use crate::src::m_random::P_Random;
use crate::src::p_inter::maxammo;
use crate::src::p_map::P_CheckPosition;
use crate::src::p_mobj::P_RemoveMobj;
use crate::src::p_mobj::P_SpawnMobj;
use crate::src::p_mobj::P_SpawnPlayer;
use crate::src::p_mobj::MF_SHADOW;
use crate::src::p_mobj::mapthing_t;
use crate::src::p_mobj::{mobj_t, pspdef_t};
use crate::src::p_mobj::{MT_BRUISERSHOT, MT_HEADSHOT, MT_TFOG, MT_TROOPSHOT};
use crate::src::p_saveg::P_ArchivePlayers;
use crate::src::p_saveg::P_ArchiveSpecials;
use crate::src::p_saveg::P_ArchiveThinkers;
use crate::src::p_saveg::P_ArchiveWorld;
use crate::src::p_saveg::P_ReadSaveGameEOF;
use crate::src::p_saveg::P_ReadSaveGameHeader;
use crate::src::p_saveg::P_SaveGameFile;
use crate::src::p_saveg::P_TempSaveGameFile;
use crate::src::p_saveg::P_UnArchivePlayers;
use crate::src::p_saveg::P_UnArchiveSpecials;
use crate::src::p_saveg::P_UnArchiveThinkers;
use crate::src::p_saveg::P_UnArchiveWorld;
use crate::src::p_saveg::P_WriteSaveGameEOF;
use crate::src::p_saveg::P_WriteSaveGameHeader;
use crate::src::p_setup::P_SetupLevel;
use crate::src::p_setup::SubsectorId;
use crate::src::p_tick::P_Ticker;
use crate::src::r_data::R_FlatNumForName;
use crate::src::r_data::R_TextureNumForName;
use crate::src::r_draw::R_FillBackScreen;
use crate::src::r_main::R_ExecuteSetViewSize;
use crate::src::r_main::R_PointInSubsector;
use crate::src::s_sound::S_PauseSound;
use crate::src::s_sound::S_ResumeSound;
use crate::src::s_sound::S_StartSound;
use crate::src::sounds::sfx_telept;
use crate::src::st_stuff::ST_Responder;
use crate::src::st_stuff::ST_Ticker;
use crate::src::statdump::StatCopy;
use crate::src::stdint_types::byte;
use crate::src::stdint_types::size_t;
use crate::src::tables::finecosine;
use crate::src::tables::finesine;
use crate::src::tables::finetangent;
use crate::src::tables::ANG45;
use crate::src::tables::ANGLETOFINESHIFT;
use crate::src::v_video::V_ScreenShot;
use crate::src::w_wad::{
    wad_name8_to_string, W_CacheLumpName, W_CheckNumForName, W_ReleaseLumpName,
};
use crate::src::wi_stuff::WI_End;
use crate::src::wi_stuff::WI_Start;
use crate::src::wi_stuff::WI_Ticker;
use crate::src::wi_stuff::{wbplayerstruct_t, wbstartstruct_t};
use crate::src::z_zone::Z_CheckHeap;
use crate::src::z_zone::Z_Free;
use crate::src::z_zone::Z_Malloc;
use crate::src::z_zone::PU_STATIC;
use crate::src::mem_compat::{memcpy, memset};
use std::io::Seek;

pub struct GGameState {
    pub oldgamestate: GameScreenState,
    pub gameaction: GameAction,
    pub gamestate: GameScreenState,
    pub gameskill: SkillType,
    pub respawnmonsters: bool,
    pub gameepisode: i32,
    pub gamemap: i32,
    pub timelimit: i32,
    pub paused: bool,
    pub sendpause: bool,
    pub sendsave: bool,
    pub usergame: bool,
    pub timingdemo: bool,
    pub nodrawers: bool,
    pub starttime: i32,
    pub viewactive: bool,
    pub deathmatch: i32,
    pub netgame: bool,
    pub playeringame: [boolean; 4],
    pub players: [player_t; 4],
    pub turbodetected: [boolean; 4],
    pub consoleplayer: i32,
    pub displayplayer: i32,
    pub levelstarttic: i32,
    pub totalsecret: i32,
    pub totalkills: i32,
    pub totalitems: i32,
    pub demoname: *mut ::core::ffi::c_char,
    pub demorecording: bool,
    pub longtics: bool,
    pub lowres_turn: bool,
    pub demoplayback: bool,
    pub netdemo: bool,
    pub demobuffer: *mut byte,
    pub demo_p: *mut byte,
    pub demoend: *mut byte,
    pub singledemo: bool,
    pub precache: bool,
    pub testcontrols: bool,
    pub testcontrols_mousespeed: i32,
    pub wminfo: wbstartstruct_t,
    pub consistancy: [[byte; 128]; 4],
    pub forwardmove: [fixed_t; 2],
    pub sidemove: [fixed_t; 2],
    pub next_weapon: i32,
    pub gamekeydown: [boolean; 256],
    pub turnheld: i32,
    pub mousearray: [boolean; 9],
    pub mousebuttons: *mut boolean,
    pub mousex: i32,
    pub mousey: i32,
    pub dclicktime: i32,
    pub dclickstate: boolean,
    pub dclicks: i32,
    pub dclicktime2: i32,
    pub dclickstate2: boolean,
    pub dclicks2: i32,
    pub joyxmove: i32,
    pub joyymove: i32,
    pub joystrafemove: i32,
    pub joyarray: [boolean; 21],
    pub joybuttons: *mut boolean,
    pub savegameslot: i32,
    pub savedescription: String,
    pub bodyque: [*mut mobj_t; 32],
    pub bodyqueslot: i32,
    pub vanilla_savegame_limit: i32,
    pub vanilla_demo_limit: i32,
    pub secretexit: bool,
    pub savename: String,
    pub d_skill: SkillType,
    pub d_episode: i32,
    pub d_map: i32,
    pub defdemoname: *mut ::core::ffi::c_char,
    pub g_build_ticcmd_carry: i16,
}

const NEW_PLAYER: player_s = player_s {
    mo: ::core::ptr::null::<mobj_t>() as *mut mobj_t,
    playerstate: PlayerState::PST_LIVE,
    cmd: ticcmd_t {
        forwardmove: 0,
        sidemove: 0,
        angleturn: 0,
        chatchar: 0,
        buttons: 0,
        consistancy: 0,
        buttons2: 0,
        inventory: 0,
        lookfly: 0,
        arti: 0,
    },
    viewz: 0,
    viewheight: 0,
    deltaviewheight: 0,
    bob: 0,
    health: 0,
    armorpoints: 0,
    armortype: 0,
    powers: [0; 6],
    cards: [false; 6],
    backpack: false,
    frags: [0; 4],
    readyweapon: weapontype_t::wp_fist,
    pendingweapon: weapontype_t::wp_fist,
    weaponowned: [false; 9],
    ammo: [0; 4],
    maxammo: [0; 4],
    attackdown: 0,
    usedown: 0,
    cheats: 0,
    refire: 0,
    killcount: 0,
    itemcount: 0,
    secretcount: 0,
    message: None,
    damagecount: 0,
    bonuscount: 0,
    attacker: None,
    extralight: 0,
    fixedcolormap: 0,
    colormap: 0,
    psprites: [pspdef_t {
        state: None,
        tics: 0,
        sx: 0,
        sy: 0,
    }; 2],
    didsecret: false,
};

impl GGameState {
    pub const fn new() -> Self {
        GGameState {
            oldgamestate: GameScreenState::GS_LEVEL,
            gameaction: GameAction::ga_nothing,
            gamestate: GameScreenState::GS_LEVEL,
            gameskill: SkillType::sk_baby,
            respawnmonsters: false,
            gameepisode: 0,
            gamemap: 0,
            timelimit: 0,
            paused: false,
            sendpause: false,
            sendsave: false,
            usergame: false,
            timingdemo: false,
            nodrawers: false,
            starttime: 0,
            viewactive: false,
            deathmatch: 0,
            netgame: false,
            playeringame: [0; 4],
            players: [NEW_PLAYER, NEW_PLAYER, NEW_PLAYER, NEW_PLAYER],
            turbodetected: [0; 4],
            consoleplayer: 0,
            displayplayer: 0,
            levelstarttic: 0,
            totalsecret: 0,
            totalkills: 0,
            totalitems: 0,
            demoname: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
            demorecording: false,
            longtics: false,
            lowres_turn: false,
            demoplayback: false,
            netdemo: false,
            demobuffer: ::core::ptr::null::<byte>() as *mut byte,
            demo_p: ::core::ptr::null::<byte>() as *mut byte,
            demoend: ::core::ptr::null::<byte>() as *mut byte,
            singledemo: false,
            precache: true,
            testcontrols: false,
            testcontrols_mousespeed: 0,
            wminfo: wbstartstruct_t {
                epsd: 0,
                didsecret: false,
                last: 0,
                next: 0,
                maxkills: 0,
                maxitems: 0,
                maxsecret: 0,
                maxfrags: 0,
                partime: 0,
                pnum: 0,
                plyr: [wbplayerstruct_t {
                    in_0: false,
                    skills: 0,
                    sitems: 0,
                    ssecret: 0,
                    stime: 0,
                    frags: [0; 4],
                    score: 0,
                }; 4],
            },
            consistancy: [[0; 128]; 4],
            forwardmove: [0x19 as i32, 0x32 as i32],
            sidemove: [0x18 as i32, 0x28 as i32],
            next_weapon: 0,
            gamekeydown: [0; 256],
            turnheld: 0,
            mousearray: [0; 9],
            mousebuttons: ::core::ptr::null::<boolean>() as *mut boolean,
            mousex: 0,
            mousey: 0,
            dclicktime: 0,
            dclickstate: 0,
            dclicks: 0,
            dclicktime2: 0,
            dclickstate2: 0,
            dclicks2: 0,
            joyxmove: 0,
            joyymove: 0,
            joystrafemove: 0,
            joyarray: [0; 21],
            joybuttons: ::core::ptr::null::<boolean>() as *mut boolean,
            savegameslot: 0,
            savedescription: String::new(),
            bodyque: [::core::ptr::null::<mobj_t>() as *mut mobj_t; 32],
            bodyqueslot: 0,
            vanilla_savegame_limit: 1,
            vanilla_demo_limit: 1,
            secretexit: false,
            savename: String::new(),
            d_skill: SkillType::sk_baby,
            d_episode: 0,
            d_map: 0,
            defdemoname: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
            g_build_ticcmd_carry: 0,
        }
    }

    // joyarray[0]/mousearray[0] are unused (mirrors the original C, where
    // index 0 is reserved); joybuttons/mousebuttons alias the rest of the
    // array. Self-referential, so must run once the struct is at its final,
    // permanently-stable address -- see game_state::finish_init.
    pub fn fixup_button_pointers(&mut self) {
        self.joybuttons = (&raw mut self.joyarray as *mut boolean).wrapping_offset(1);
        self.mousebuttons = (&raw mut self.mousearray as *mut boolean).wrapping_offset(1);
    }

    pub fn player_mut(&mut self, id: PlayerId) -> *mut player_t {
        &mut self.players[id.0 as usize] as *mut player_t
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_5 {
    pub weapon: weapontype_t,
    pub weapon_num: weapontype_t,
}
pub const DEH_DEFAULT_INITIAL_HEALTH: i32 = 100;
pub const DEH_DEFAULT_INITIAL_BULLETS: i32 = 50;
pub const deh_initial_health: i32 = DEH_DEFAULT_INITIAL_HEALTH;
pub const deh_initial_bullets: i32 = DEH_DEFAULT_INITIAL_BULLETS;
pub const DOOM_191_VERSION: i32 = 111;
pub const SAVEGAMESIZE: i32 = 0x2c000;
pub const TURBOTHRESHOLD: i32 = 0x32;
#[no_mangle]
pub static angleturn: [fixed_t; 3] = [640 as i32, 1280 as i32, 320 as i32];
static weapon_order_table: [C2RustUnnamed_5; 9] = [
    C2RustUnnamed_5 {
        weapon: weapontype_t::wp_fist,
        weapon_num: weapontype_t::wp_fist,
    },
    C2RustUnnamed_5 {
        weapon: weapontype_t::wp_chainsaw,
        weapon_num: weapontype_t::wp_fist,
    },
    C2RustUnnamed_5 {
        weapon: weapontype_t::wp_pistol,
        weapon_num: weapontype_t::wp_pistol,
    },
    C2RustUnnamed_5 {
        weapon: weapontype_t::wp_shotgun,
        weapon_num: weapontype_t::wp_shotgun,
    },
    C2RustUnnamed_5 {
        weapon: weapontype_t::wp_supershotgun,
        weapon_num: weapontype_t::wp_shotgun,
    },
    C2RustUnnamed_5 {
        weapon: weapontype_t::wp_chaingun,
        weapon_num: weapontype_t::wp_chaingun,
    },
    C2RustUnnamed_5 {
        weapon: weapontype_t::wp_missile,
        weapon_num: weapontype_t::wp_missile,
    },
    C2RustUnnamed_5 {
        weapon: weapontype_t::wp_plasma,
        weapon_num: weapontype_t::wp_plasma,
    },
    C2RustUnnamed_5 {
        weapon: weapontype_t::wp_bfg,
        weapon_num: weapontype_t::wp_bfg,
    },
];
pub const SLOWTURNTICS: i32 = 6;
pub const NUMKEYS: i32 = 256;
pub const MAX_JOY_BUTTONS: i32 = 20;
pub const BODYQUESIZE: i32 = 32;
pub unsafe fn G_CmdChecksum(mut cmd: *mut ticcmd_t) -> i32 {
    let mut i: size_t = 0;
    let mut sum: i32 = 0 as i32;
    i = 0 as size_t;
    while i
        < (::core::mem::size_of::<ticcmd_t>() as usize)
            .wrapping_div(4 as usize)
            .wrapping_sub(1 as usize)
    {
        sum += *(cmd as *mut i32).offset(i as isize);
        i = i.wrapping_add(1);
    }
    return sum;
}
fn WeaponSelectable(state: &mut GameState, mut weapon: weapontype_t) -> bool {
    if weapon as u32 == weapontype_t::wp_supershotgun as u32
        && (if state.doomstat.gamemission as u32 == GameMission_t::pack_chex as u32 {
            GameMission_t::doom as u32
        } else {
            if state.doomstat.gamemission as u32 == GameMission_t::pack_hacx as u32 {
                GameMission_t::doom2 as u32
            } else {
                state.doomstat.gamemission as u32
            }
        }) == GameMission_t::doom as u32
    {
        return false;
    }
    if (weapon as u32 == weapontype_t::wp_plasma as u32 || weapon as u32 == weapontype_t::wp_bfg as u32)
        && state.doomstat.gamemission as u32 == GameMission_t::doom as u32
        && state.doomstat.gamemode as u32 == GameMode_t::shareware as u32
    {
        return false;
    }
    if !state.g_game.players[state.g_game.consoleplayer as usize].weaponowned[weapon as usize] {
        return false;
    }
    if weapon as u32 == weapontype_t::wp_fist as u32
        && state.g_game.players[state.g_game.consoleplayer as usize].weaponowned
            [weapontype_t::wp_chainsaw as i32 as usize]
        && state.g_game.players[state.g_game.consoleplayer as usize].powers
            [PowerType::pw_strength as i32 as usize]
            == 0
    {
        return false;
    }
    return true;
}
fn G_NextWeapon(state: &mut GameState, mut direction: i32) -> i32 {
    let mut weapon: weapontype_t = weapontype_t::wp_fist;
    let mut start_i: i32 = 0;
    let mut i: i32 = 0;
    if state.g_game.players[state.g_game.consoleplayer as usize].pendingweapon as u32
        == weapontype_t::wp_nochange as u32
    {
        weapon = state.g_game.players[state.g_game.consoleplayer as usize].readyweapon;
    } else {
        weapon = state.g_game.players[state.g_game.consoleplayer as usize].pendingweapon;
    }
    i = 0 as i32;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_5; 9]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_5>() as usize)
    {
        if weapon_order_table[i as usize].weapon as u32 == weapon as u32 {
            break;
        }
        i += 1;
    }
    start_i = i;
    loop {
        i += direction;
        i = (i as usize)
            .wrapping_add(
                (::core::mem::size_of::<[C2RustUnnamed_5; 9]>() as usize)
                    .wrapping_div(::core::mem::size_of::<C2RustUnnamed_5>() as usize),
            )
            .wrapping_rem(
                (::core::mem::size_of::<[C2RustUnnamed_5; 9]>() as usize)
                    .wrapping_div(::core::mem::size_of::<C2RustUnnamed_5>() as usize),
            ) as i32;
        if !(i != start_i && !WeaponSelectable(state, weapon_order_table[i as usize].weapon)) {
            break;
        }
    }
    return weapon_order_table[i as usize].weapon_num as i32;
}
pub unsafe fn G_BuildTiccmd(state: &mut GameState, mut cmd: *mut ticcmd_t, mut maketic: i32) {
    let mut i: i32 = 0;
    let mut strafe: bool = false;
    let mut bstrafe: boolean = 0;
    let mut speed: i32 = 0;
    let mut tspeed: i32 = 0;
    let mut forward: i32 = 0;
    let mut side: i32 = 0;
    memset(
        cmd as *mut ::core::ffi::c_void,
        0 as i32,
        ::core::mem::size_of::<ticcmd_t>() as size_t,
    );
    (*cmd).consistancy = state.g_game.consistancy[state.g_game.consoleplayer as usize]
        [(maketic % BACKUPTICS) as usize];
    strafe = state.g_game.gamekeydown[state.m_controls.key_strafe as usize] != 0
        || *state
            .g_game
            .mousebuttons
            .offset(state.m_controls.mousebstrafe as isize)
            != 0
        || *state
            .g_game
            .joybuttons
            .offset(state.m_controls.joybstrafe as isize)
            != 0;
    speed = (state.m_controls.key_speed >= NUMKEYS
        || state.m_controls.joybspeed >= MAX_JOY_BUTTONS
        || state.g_game.gamekeydown[state.m_controls.key_speed as usize] != 0
        || *state
            .g_game
            .joybuttons
            .offset(state.m_controls.joybspeed as isize)
            != 0) as i32;
    side = 0 as i32;
    forward = side;
    if state.g_game.joyxmove < 0 as i32
        || state.g_game.joyxmove > 0 as i32
        || state.g_game.gamekeydown[state.m_controls.key_right as usize] != 0
        || state.g_game.gamekeydown[state.m_controls.key_left as usize] != 0
    {
        state.g_game.turnheld += state.d_loop.ticdup;
    } else {
        state.g_game.turnheld = 0 as i32;
    }
    if state.g_game.turnheld < SLOWTURNTICS {
        tspeed = 2 as i32;
    } else {
        tspeed = speed;
    }
    if strafe {
        if state.g_game.gamekeydown[state.m_controls.key_right as usize] != 0 {
            side += state.g_game.sidemove[speed as usize] as i32;
        }
        if state.g_game.gamekeydown[state.m_controls.key_left as usize] != 0 {
            side -= state.g_game.sidemove[speed as usize] as i32;
        }
        if state.g_game.joyxmove > 0 as i32 {
            side += state.g_game.sidemove[speed as usize] as i32;
        }
        if state.g_game.joyxmove < 0 as i32 {
            side -= state.g_game.sidemove[speed as usize] as i32;
        }
    } else {
        if state.g_game.gamekeydown[state.m_controls.key_right as usize] != 0 {
            (*cmd).angleturn = ((*cmd).angleturn as i32 - angleturn[tspeed as usize] as i32) as i16;
        }
        if state.g_game.gamekeydown[state.m_controls.key_left as usize] != 0 {
            (*cmd).angleturn = ((*cmd).angleturn as i32 + angleturn[tspeed as usize] as i32) as i16;
        }
        if state.g_game.joyxmove > 0 as i32 {
            (*cmd).angleturn = ((*cmd).angleturn as i32 - angleturn[tspeed as usize] as i32) as i16;
        }
        if state.g_game.joyxmove < 0 as i32 {
            (*cmd).angleturn = ((*cmd).angleturn as i32 + angleturn[tspeed as usize] as i32) as i16;
        }
    }
    if state.g_game.gamekeydown[state.m_controls.key_up as usize] != 0 {
        forward += state.g_game.forwardmove[speed as usize] as i32;
    }
    if state.g_game.gamekeydown[state.m_controls.key_down as usize] != 0 {
        forward -= state.g_game.forwardmove[speed as usize] as i32;
    }
    if state.g_game.joyymove < 0 as i32 {
        forward += state.g_game.forwardmove[speed as usize] as i32;
    }
    if state.g_game.joyymove > 0 as i32 {
        forward -= state.g_game.forwardmove[speed as usize] as i32;
    }
    if state.g_game.gamekeydown[state.m_controls.key_strafeleft as usize] != 0
        || *state
            .g_game
            .joybuttons
            .offset(state.m_controls.joybstrafeleft as isize)
            != 0
        || *state
            .g_game
            .mousebuttons
            .offset(state.m_controls.mousebstrafeleft as isize)
            != 0
        || state.g_game.joystrafemove < 0 as i32
    {
        side -= state.g_game.sidemove[speed as usize] as i32;
    }
    if state.g_game.gamekeydown[state.m_controls.key_straferight as usize] != 0
        || *state
            .g_game
            .joybuttons
            .offset(state.m_controls.joybstraferight as isize)
            != 0
        || *state
            .g_game
            .mousebuttons
            .offset(state.m_controls.mousebstraferight as isize)
            != 0
        || state.g_game.joystrafemove > 0 as i32
    {
        side += state.g_game.sidemove[speed as usize] as i32;
    }
    (*cmd).chatchar = HU_dequeueChatChar(&mut state.hu_stuff) as byte;
    if state.g_game.gamekeydown[state.m_controls.key_fire as usize] != 0
        || *state
            .g_game
            .mousebuttons
            .offset(state.m_controls.mousebfire as isize)
            != 0
        || *state
            .g_game
            .joybuttons
            .offset(state.m_controls.joybfire as isize)
            != 0
    {
        (*cmd).buttons = ((*cmd).buttons as i32 | BT_ATTACK as i32) as byte;
    }
    if state.g_game.gamekeydown[state.m_controls.key_use as usize] != 0
        || *state
            .g_game
            .joybuttons
            .offset(state.m_controls.joybuse as isize)
            != 0
        || *state
            .g_game
            .mousebuttons
            .offset(state.m_controls.mousebuse as isize)
            != 0
    {
        (*cmd).buttons = ((*cmd).buttons as i32 | BT_USE as i32) as byte;
        state.g_game.dclicks = 0 as i32;
    }
    if state.g_game.gamestate == GameScreenState::GS_LEVEL && state.g_game.next_weapon != 0 as i32 {
        let next_weapon = state.g_game.next_weapon;
        i = G_NextWeapon(state, next_weapon);
        (*cmd).buttons = ((*cmd).buttons as i32 | BT_CHANGE as i32) as byte;
        (*cmd).buttons = ((*cmd).buttons as i32 | i << BT_WEAPONSHIFT as i32) as byte;
    } else {
        i = 0 as i32;
        while (i as usize)
            < (::core::mem::size_of::<[*mut i32; 8]>() as usize)
                .wrapping_div(::core::mem::size_of::<*mut i32>() as usize)
        {
            let mut key: i32 = *state.m_controls.weapon_keys[i as usize];
            if state.g_game.gamekeydown[key as usize] != 0 {
                (*cmd).buttons = ((*cmd).buttons as i32 | BT_CHANGE as i32) as byte;
                (*cmd).buttons = ((*cmd).buttons as i32 | i << BT_WEAPONSHIFT as i32) as byte;
                break;
            } else {
                i += 1;
            }
        }
    }
    state.g_game.next_weapon = 0 as i32;
    if *state
        .g_game
        .mousebuttons
        .offset(state.m_controls.mousebforward as isize)
        != 0
    {
        forward += state.g_game.forwardmove[speed as usize] as i32;
    }
    if *state
        .g_game
        .mousebuttons
        .offset(state.m_controls.mousebbackward as isize)
        != 0
    {
        forward -= state.g_game.forwardmove[speed as usize] as i32;
    }
    if state.m_controls.dclick_use != 0 {
        if *state
            .g_game
            .mousebuttons
            .offset(state.m_controls.mousebforward as isize)
            != state.g_game.dclickstate
            && state.g_game.dclicktime > 1 as i32
        {
            state.g_game.dclickstate = *state
                .g_game
                .mousebuttons
                .offset(state.m_controls.mousebforward as isize);
            if state.g_game.dclickstate != 0 {
                state.g_game.dclicks += 1;
            }
            if state.g_game.dclicks == 2 as i32 {
                (*cmd).buttons = ((*cmd).buttons as i32 | BT_USE as i32) as byte;
                state.g_game.dclicks = 0 as i32;
            } else {
                state.g_game.dclicktime = 0 as i32;
            }
        } else {
            state.g_game.dclicktime += state.d_loop.ticdup;
            if state.g_game.dclicktime > 20 as i32 {
                state.g_game.dclicks = 0 as i32;
                state.g_game.dclickstate = 0 as boolean;
            }
        }
        bstrafe = (*state
            .g_game
            .mousebuttons
            .offset(state.m_controls.mousebstrafe as isize)
            != 0
            || *state
                .g_game
                .joybuttons
                .offset(state.m_controls.joybstrafe as isize)
                != 0) as i32 as boolean;
        if bstrafe != state.g_game.dclickstate2 && state.g_game.dclicktime2 > 1 as i32 {
            state.g_game.dclickstate2 = bstrafe;
            if state.g_game.dclickstate2 != 0 {
                state.g_game.dclicks2 += 1;
            }
            if state.g_game.dclicks2 == 2 as i32 {
                (*cmd).buttons = ((*cmd).buttons as i32 | BT_USE as i32) as byte;
                state.g_game.dclicks2 = 0 as i32;
            } else {
                state.g_game.dclicktime2 = 0 as i32;
            }
        } else {
            state.g_game.dclicktime2 += state.d_loop.ticdup;
            if state.g_game.dclicktime2 > 20 as i32 {
                state.g_game.dclicks2 = 0 as i32;
                state.g_game.dclickstate2 = 0 as boolean;
            }
        }
    }
    forward += state.g_game.mousey;
    if strafe {
        side += state.g_game.mousex * 2 as i32;
    } else {
        (*cmd).angleturn = ((*cmd).angleturn as i32 - state.g_game.mousex * 0x8 as i32) as i16;
    }
    if state.g_game.mousex == 0 as i32 {
        state.g_game.testcontrols_mousespeed = 0 as i32;
    }
    state.g_game.mousey = 0 as i32;
    state.g_game.mousex = state.g_game.mousey;
    if forward > state.g_game.forwardmove[1 as i32 as usize] {
        forward = state.g_game.forwardmove[1 as i32 as usize] as i32;
    } else if forward < -state.g_game.forwardmove[1 as i32 as usize] {
        forward = -state.g_game.forwardmove[1 as i32 as usize] as i32;
    }
    if side > state.g_game.forwardmove[1 as i32 as usize] {
        side = state.g_game.forwardmove[1 as i32 as usize] as i32;
    } else if side < -state.g_game.forwardmove[1 as i32 as usize] {
        side = -state.g_game.forwardmove[1 as i32 as usize] as i32;
    }
    (*cmd).forwardmove = ((*cmd).forwardmove as i32 + forward) as i8;
    (*cmd).sidemove = ((*cmd).sidemove as i32 + side) as i8;
    if state.g_game.sendpause {
        state.g_game.sendpause = false;
        (*cmd).buttons = (BT_SPECIAL as i32 | BTS_PAUSE as i32) as byte;
    }
    if state.g_game.sendsave {
        state.g_game.sendsave = false;
        (*cmd).buttons = (BT_SPECIAL as i32
            | BTS_SAVEGAME as i32
            | state.g_game.savegameslot << BTS_SAVESHIFT as i32) as byte;
    }
    if state.g_game.lowres_turn {
        let mut desired_angleturn: i16 = 0;
        desired_angleturn =
            ((*cmd).angleturn as i32 + state.g_game.g_build_ticcmd_carry as i32) as i16;
        (*cmd).angleturn = (desired_angleturn as i32 + 128 as i32 & 0xff00 as i32) as i16;
        state.g_game.g_build_ticcmd_carry =
            (desired_angleturn as i32 - (*cmd).angleturn as i32) as i16;
    }
}
pub unsafe fn G_DoLoadLevel(state: &mut GameState) {
    let mut i: i32 = 0;
    state.r_sky.skyflatnum = R_FlatNumForName(state, "F_SKY1");
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as u32
        && [GameVersion::final2, GameVersion::chex].contains(&state.doomstat.gameversion)
    {
        let skytexturename: &str;
        if state.g_game.gamemap < 12 as i32 {
            skytexturename = "SKY1";
        } else if state.g_game.gamemap < 21 as i32 {
            skytexturename = "SKY2";
        } else {
            skytexturename = "SKY3";
        }
        state.r_sky.skytexture = R_TextureNumForName(&mut state.r_data, skytexturename);
    }
    state.g_game.levelstarttic = state.d_loop.gametic;
    if state.d_main.wipegamestate == GameScreenState::GS_LEVEL {
        state.d_main.wipegamestate = GameScreenState::GS_WIPPED;
    }
    state.g_game.gamestate = GameScreenState::GS_LEVEL;
    i = 0 as i32;
    while i < MAXPLAYERS {
        state.g_game.turbodetected[i as usize] = false_0 as boolean;
        if state.g_game.playeringame[i as usize] != 0
            && state.g_game.players[i as usize].playerstate == PlayerState::PST_DEAD
        {
            state.g_game.players[i as usize].playerstate = PlayerState::PST_REBORN;
        }
        memset(
            &raw mut (*(&raw mut state.g_game.players as *mut player_t).offset(i as isize)).frags
                as *mut i32 as *mut ::core::ffi::c_void,
            0 as i32,
            ::core::mem::size_of::<[i32; 4]>() as size_t,
        );
        i += 1;
    }
    P_SetupLevel(
        state,
        state.g_game.gameepisode,
        state.g_game.gamemap,
    );
    state.g_game.displayplayer = state.g_game.consoleplayer;
    state.g_game.gameaction = GameAction::ga_nothing;
    Z_CheckHeap(&mut state.z_zone);
    memset(
        &raw mut state.g_game.gamekeydown as *mut boolean as *mut ::core::ffi::c_void,
        0 as i32,
        ::core::mem::size_of::<[boolean; 256]>() as size_t,
    );
    state.g_game.joystrafemove = 0 as i32;
    state.g_game.joyymove = state.g_game.joystrafemove;
    state.g_game.joyxmove = state.g_game.joyymove;
    state.g_game.mousey = 0 as i32;
    state.g_game.mousex = state.g_game.mousey;
    state.g_game.paused = false;
    state.g_game.sendsave = state.g_game.paused;
    state.g_game.sendpause = state.g_game.sendsave;
    memset(
        &raw mut state.g_game.mousearray as *mut boolean as *mut ::core::ffi::c_void,
        0 as i32,
        ::core::mem::size_of::<[boolean; 9]>() as size_t,
    );
    memset(
        &raw mut state.g_game.joyarray as *mut boolean as *mut ::core::ffi::c_void,
        0 as i32,
        ::core::mem::size_of::<[boolean; 21]>() as size_t,
    );
    if state.g_game.testcontrols {
        state.g_game.players[state.g_game.consoleplayer as usize].message =
            Some("Press escape to quit.".to_string());
    }
}
unsafe fn SetJoyButtons(state: &mut GameState, mut buttons_mask: u32) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < MAX_JOY_BUTTONS {
        let mut button_on: i32 = (buttons_mask & ((1 as i32) << i) as u32 != 0 as u32) as i32;
        if *state.g_game.joybuttons.offset(i as isize) == 0 && button_on != 0 {
            if i == state.m_controls.joybprevweapon {
                state.g_game.next_weapon = -(1 as i32);
            } else if i == state.m_controls.joybnextweapon {
                state.g_game.next_weapon = 1 as i32;
            }
        }
        *state.g_game.joybuttons.offset(i as isize) = button_on as boolean;
        i += 1;
    }
}
unsafe fn SetMouseButtons(state: &mut GameState, mut buttons_mask: u32) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < MAX_MOUSE_BUTTONS {
        let mut button_on: u32 = (buttons_mask & ((1 as i32) << i) as u32 != 0 as u32) as u32;
        if *state.g_game.mousebuttons.offset(i as isize) == 0 && button_on != 0 {
            if i == state.m_controls.mousebprevweapon {
                state.g_game.next_weapon = -(1 as i32);
            } else if i == state.m_controls.mousebnextweapon {
                state.g_game.next_weapon = 1 as i32;
            }
        }
        *state.g_game.mousebuttons.offset(i as isize) = button_on as boolean;
        i += 1;
    }
}
pub unsafe fn G_Responder(state: &mut GameState, mut ev: event_t) -> bool {
    if state.g_game.gamestate == GameScreenState::GS_LEVEL
        && ev.type_0 == EvType::ev_keydown
        && ev.data1 == state.m_controls.key_spy
        && (state.g_game.singledemo || state.g_game.deathmatch == 0)
    {
        loop {
            state.g_game.displayplayer += 1;
            if state.g_game.displayplayer == MAXPLAYERS {
                state.g_game.displayplayer = 0 as i32;
            }
            if !(state.g_game.playeringame[state.g_game.displayplayer as usize] == 0
                && state.g_game.displayplayer != state.g_game.consoleplayer)
            {
                break;
            }
        }
        return true;
    }
    if state.g_game.gameaction == GameAction::ga_nothing
        && !state.g_game.singledemo
        && (state.g_game.demoplayback || state.g_game.gamestate == GameScreenState::GS_DEMOSCREEN)
    {
        if ev.type_0 == EvType::ev_keydown
            || ev.type_0 == EvType::ev_mouse && ev.data1 != 0
            || ev.type_0 == EvType::ev_joystick && ev.data1 != 0
        {
            M_StartControlPanel(state);
            return true;
        }
        return false;
    }
    if state.g_game.gamestate == GameScreenState::GS_LEVEL {
        if HU_Responder(state, &ev) {
            return true;
        }
        if ST_Responder(state, &ev) {
            return true;
        }
        if AM_Responder(state, &ev) {
            return true;
        }
    }
    if state.g_game.gamestate == GameScreenState::GS_FINALE {
        if F_Responder(state, &ev) {
            return true;
        }
    }
    if state.g_game.testcontrols && ev.type_0 == EvType::ev_mouse {
        state.g_game.testcontrols_mousespeed = (ev.data2).abs();
    }
    if ev.type_0 == EvType::ev_keydown && ev.data1 == state.m_controls.key_prevweapon {
        state.g_game.next_weapon = -1;
    } else if ev.type_0 == EvType::ev_keydown && ev.data1 == state.m_controls.key_nextweapon {
        state.g_game.next_weapon = 1;
    }
    match ev.type_0 as u32 {
        0 => {
            if ev.data1 == state.m_controls.key_pause {
                state.g_game.sendpause = true;
            } else if ev.data1 < NUMKEYS {
                state.g_game.gamekeydown[ev.data1 as usize] = true_0 as boolean;
            }
            return true;
        }
        1 => {
            if ev.data1 < NUMKEYS {
                state.g_game.gamekeydown[ev.data1 as usize] = false_0 as boolean;
            }
            return false;
        }
        2 => {
            SetMouseButtons(state, ev.data1 as u32);
            state.g_game.mousex = ev.data2 * (state.m_menu.mouseSensitivity + 5 as i32) / 10 as i32;
            state.g_game.mousey = ev.data3 * (state.m_menu.mouseSensitivity + 5 as i32) / 10 as i32;
            return true;
        }
        3 => {
            SetJoyButtons(state, ev.data1 as u32);
            state.g_game.joyxmove = ev.data2;
            state.g_game.joyymove = ev.data3;
            state.g_game.joystrafemove = ev.data4;
            return true;
        }
        _ => {}
    }
    return false;
}
pub unsafe fn G_Ticker(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut buf: i32 = 0;
    let mut cmd: *mut ticcmd_t = ::core::ptr::null_mut::<ticcmd_t>();
    i = 0 as i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] != 0
            && state.g_game.players[i as usize].playerstate == PlayerState::PST_REBORN
        {
            G_DoReborn(state, i);
        }
        i += 1;
    }
    while state.g_game.gameaction != GameAction::ga_nothing {
        match state.g_game.gameaction {
            GameAction::ga_loadlevel => {
                G_DoLoadLevel(state);
            }
            GameAction::ga_newgame => {
                G_DoNewGame(state);
            }
            GameAction::ga_loadgame => {
                G_DoLoadGame(state);
            }
            GameAction::ga_savegame => {
                G_DoSaveGame(state);
            }
            GameAction::ga_playdemo => {
                G_DoPlayDemo(state);
            }
            GameAction::ga_completed => {
                G_DoCompleted(state);
            }
            GameAction::ga_victory => {
                F_StartFinale(state);
            }
            GameAction::ga_worlddone => {
                G_DoWorldDone(state);
            }
            GameAction::ga_screenshot => {
                V_ScreenShot(state);
                state.g_game.players[state.g_game.consoleplayer as usize].message =
                    Some("screen shot".to_string());
                state.g_game.gameaction = GameAction::ga_nothing;
            }
            GameAction::ga_nothing => {}
        }
    }
    buf = state.d_loop.gametic / state.d_loop.ticdup % BACKUPTICS;
    i = 0 as i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] != 0 {
            cmd =
                &raw mut (*(&raw mut state.g_game.players as *mut player_t).offset(i as isize)).cmd;
            memcpy(
                cmd as *mut ::core::ffi::c_void,
                state.d_net.netcmds.offset(i as isize) as *mut ticcmd_t
                    as *const ::core::ffi::c_void,
                ::core::mem::size_of::<ticcmd_t>() as size_t,
            );
            if state.g_game.demoplayback {
                G_ReadDemoTiccmd(state, cmd);
            }
            if state.g_game.demorecording {
                G_WriteDemoTiccmd(state, cmd);
            }
            if (*cmd).forwardmove as i32 > TURBOTHRESHOLD {
                state.g_game.turbodetected[i as usize] = true_0 as boolean;
            }
            if state.d_loop.gametic & 31 as i32 == 0 as i32
                && (state.d_loop.gametic >> 5 as i32) % MAXPLAYERS == i
                && state.g_game.turbodetected[i as usize] != 0
            {
                let player_name =
                    ::std::ffi::CStr::from_ptr(state.hu_stuff.player_names[i as usize])
                        .to_string_lossy()
                        .into_owned();
                state.g_game.players[state.g_game.consoleplayer as usize].message =
                    Some(format!("{} is turbo!", player_name));
                state.g_game.turbodetected[i as usize] = false_0 as boolean;
            }
            if state.g_game.netgame
                && !state.g_game.netdemo
                && state.d_loop.gametic % state.d_loop.ticdup == 0
            {
                if state.d_loop.gametic > BACKUPTICS
                    && state.g_game.consistancy[i as usize][buf as usize] as i32
                        != (*cmd).consistancy as i32
                {
                    I_Error(&format!(
                        "consistency failure ({} should be {})",
                        (*cmd).consistancy as i32,
                        state.g_game.consistancy[i as usize][buf as usize] as i32,
                    ));
                }
                if !state.g_game.players[i as usize].mo.is_null() {
                    state.g_game.consistancy[i as usize][buf as usize] =
                        (*state.g_game.players[i as usize].mo).x as byte;
                } else {
                    state.g_game.consistancy[i as usize][buf as usize] =
                        state.m_random.rndindex as byte;
                }
            }
        }
        i += 1;
    }
    i = 0 as i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] != 0 {
            if state.g_game.players[i as usize].cmd.buttons as i32 & BT_SPECIAL as i32 != 0 {
                match state.g_game.players[i as usize].cmd.buttons as i32 & BT_SPECIALMASK as i32 {
                    1 => {
                        state.g_game.paused = !state.g_game.paused;
                        if state.g_game.paused {
                            S_PauseSound(state);
                        } else {
                            S_ResumeSound(state);
                        }
                    }
                    2 => {
                        if state.g_game.savedescription.is_empty() {
                            state.g_game.savedescription = "NET GAME".to_string();
                        }
                        state.g_game.savegameslot = (state.g_game.players[i as usize].cmd.buttons
                            as i32
                            & BTS_SAVEMASK as i32)
                            >> BTS_SAVESHIFT as i32;
                        state.g_game.gameaction = GameAction::ga_savegame;
                    }
                    _ => {}
                }
            }
        }
        i += 1;
    }
    if state.g_game.oldgamestate as u32 == GameScreenState::GS_INTERMISSION as u32
        && state.g_game.gamestate as u32 != GameScreenState::GS_INTERMISSION as u32
    {
        WI_End(state);
    }
    state.g_game.oldgamestate = state.g_game.gamestate;
    match state.g_game.gamestate as u32 {
        0 => {
            P_Ticker(state);
            ST_Ticker(state);
            AM_Ticker(state);
            HU_Ticker(state);
        }
        1 => {
            WI_Ticker(state);
        }
        2 => {
            F_Ticker(state);
        }
        3 => {
            D_PageTicker(state);
        }
        _ => {}
    };
}
pub unsafe fn G_InitPlayer(state: &mut GGameState, mut player: i32) {
    G_PlayerReborn(state, player);
}
pub unsafe fn G_PlayerFinishLevel(state: &mut GGameState, mut player: i32) {
    let mut p: *mut player_t = ::core::ptr::null_mut::<player_t>();
    p = (&raw mut state.players as *mut player_t).offset(player as isize) as *mut player_t;
    memset(
        &raw mut (*p).powers as *mut i32 as *mut ::core::ffi::c_void,
        0 as i32,
        ::core::mem::size_of::<[i32; 6]>() as size_t,
    );
    memset(
        &raw mut (*p).cards as *mut bool as *mut ::core::ffi::c_void,
        0 as i32,
        ::core::mem::size_of::<[bool; 6]>() as size_t,
    );
    (*(*p).mo).flags &= !(MF_SHADOW as i32);
    (*p).extralight = 0 as i32;
    (*p).fixedcolormap = 0 as i32;
    (*p).damagecount = 0 as i32;
    (*p).bonuscount = 0 as i32;
}
pub unsafe fn G_PlayerReborn(state: &mut GGameState, mut player: i32) {
    let mut p: *mut player_t = ::core::ptr::null_mut::<player_t>();
    let mut i: i32 = 0;
    let mut frags: [i32; 4] = [0; 4];
    let mut killcount: i32 = 0;
    let mut itemcount: i32 = 0;
    let mut secretcount: i32 = 0;
    memcpy(
        &raw mut frags as *mut i32 as *mut ::core::ffi::c_void,
        &raw mut (*(&raw mut state.players as *mut player_t).offset(player as isize)).frags
            as *mut i32 as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[i32; 4]>() as size_t,
    );
    killcount = state.players[player as usize].killcount;
    itemcount = state.players[player as usize].itemcount;
    secretcount = state.players[player as usize].secretcount;
    p = (&raw mut state.players as *mut player_t).offset(player as isize) as *mut player_t;
    memset(
        p as *mut ::core::ffi::c_void,
        0 as i32,
        ::core::mem::size_of::<player_t>() as size_t,
    );
    ::core::ptr::write(&raw mut (*p).message, None);
    memcpy(
        &raw mut (*(&raw mut state.players as *mut player_t).offset(player as isize)).frags
            as *mut i32 as *mut ::core::ffi::c_void,
        &raw mut frags as *mut i32 as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[i32; 4]>() as size_t,
    );
    state.players[player as usize].killcount = killcount;
    state.players[player as usize].itemcount = itemcount;
    state.players[player as usize].secretcount = secretcount;
    (*p).attackdown = true_0;
    (*p).usedown = (*p).attackdown;
    (*p).playerstate = PlayerState::PST_LIVE;
    (*p).health = deh_initial_health;
    (*p).pendingweapon = weapontype_t::wp_pistol;
    (*p).readyweapon = (*p).pendingweapon;
    (*p).weaponowned[weapontype_t::wp_fist as i32 as usize] = true;
    (*p).weaponowned[weapontype_t::wp_pistol as i32 as usize] = true;
    (*p).ammo[ammotype_t::am_clip as i32 as usize] = deh_initial_bullets;
    i = 0 as i32;
    while i < NUMAMMO as i32 {
        (*p).maxammo[i as usize] = maxammo[i as usize];
        i += 1;
    }
}
pub unsafe fn G_CheckSpot(
    state: &mut GameState,
    mut playernum: i32,
    mut mthing: *mut mapthing_t,
) -> bool {
    let mut x: fixed_t = 0;
    let mut y: fixed_t = 0;
    let mut ss: SubsectorId = SubsectorId(0);
    let mut mo: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut i: i32 = 0;
    if state.g_game.players[playernum as usize].mo.is_null() {
        i = 0 as i32;
        while i < playernum {
            if (*state.g_game.players[i as usize].mo).x == ((*mthing).x as i32) << FRACBITS
                && (*state.g_game.players[i as usize].mo).y == ((*mthing).y as i32) << FRACBITS
            {
                return false;
            }
            i += 1;
        }
        return true;
    }
    x = (((*mthing).x as i32) << FRACBITS) as fixed_t;
    y = (((*mthing).y as i32) << FRACBITS) as fixed_t;
    if !P_CheckPosition(state, state.g_game.players[playernum as usize].mo, x, y) {
        return false;
    }
    if state.g_game.bodyqueslot >= BODYQUESIZE {
        P_RemoveMobj(
            state,
            state.g_game.bodyque[(state.g_game.bodyqueslot % BODYQUESIZE) as usize],
        );
    }
    state.g_game.bodyque[(state.g_game.bodyqueslot % BODYQUESIZE) as usize] =
        state.g_game.players[playernum as usize].mo;
    state.g_game.bodyqueslot += 1;
    ss = R_PointInSubsector(state, x, y);
    let mut xa: fixed_t = 0;
    let mut ya: fixed_t = 0;
    let mut an: i32 = 0;
    an = (ANG45 >> ANGLETOFINESHIFT) * ((*mthing).angle as i32 / 45 as i32);
    match an {
        4096 => {
            xa = finetangent[2048 as i32 as usize];
            ya = finetangent[0 as i32 as usize];
        }
        5120 => {
            xa = finetangent[3072 as i32 as usize];
            ya = finetangent[1024 as i32 as usize];
        }
        6144 => {
            xa = finesine[0 as i32 as usize];
            ya = finetangent[2048 as i32 as usize];
        }
        7168 => {
            xa = finesine[1024 as i32 as usize];
            ya = finetangent[3072 as i32 as usize];
        }
        0 | 1024 | 2048 | 3072 => {
            xa = finecosine[an as isize];
            ya = finesine[an as usize];
        }
        _ => {
            I_Error(&format!("G_CheckSpot: unexpected angle {}\n", an));
        }
    }
    let floorheight =
        (*state.p_setup.sector_mut(state.p_setup.subsectors[ss.0 as usize].sector)).floorheight;
    mo = P_SpawnMobj(
        state,
        x + 20 as fixed_t * xa,
        y + 20 as fixed_t * ya,
        floorheight,
        MT_TFOG,
    );
    if state.g_game.players[state.g_game.consoleplayer as usize].viewz != 1 as i32 {
        S_StartSound(
            state,
            mo as *mut ::core::ffi::c_void,
            sfx_telept as i32,
        );
    }
    return true;
}
pub unsafe fn G_DeathMatchSpawnPlayer(state: &mut GameState, mut playernum: i32) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut selections: i32 = 0;
    selections = state
        .p_setup
        .deathmatch_p
        .offset_from(&raw mut state.p_setup.deathmatchstarts as *mut mapthing_t)
        as i64 as i32;
    if selections < 4 as i32 {
        I_Error(&format!("Only {} deathmatch spots, 4 required", selections));
    }
    j = 0 as i32;
    while j < 20 as i32 {
        i = P_Random(&mut state.m_random) % selections;
        let dm_spot = (&raw mut state.p_setup.deathmatchstarts as *mut mapthing_t)
            .offset(i as isize) as *mut mapthing_t;
        if G_CheckSpot(state, playernum, dm_spot) {
            state.p_setup.deathmatchstarts[i as usize].type_0 = (playernum + 1 as i32) as i16;
            let dm_spot = (&raw mut state.p_setup.deathmatchstarts as *mut mapthing_t)
                .offset(i as isize) as *mut mapthing_t;
            P_SpawnPlayer(state, dm_spot);
            return;
        }
        j += 1;
    }
    let spot = (&raw mut state.p_setup.playerstarts as *mut mapthing_t).offset(playernum as isize)
        as *mut mapthing_t;
    P_SpawnPlayer(state, spot);
}
pub unsafe fn G_DoReborn(state: &mut GameState, mut playernum: i32) {
    let mut i: i32 = 0;
    if !state.g_game.netgame {
        state.g_game.gameaction = GameAction::ga_loadlevel;
    } else {
        (*state.g_game.players[playernum as usize].mo).player = None;
        if state.g_game.deathmatch != 0 {
            G_DeathMatchSpawnPlayer(state, playernum);
            return;
        }
        let spot = (&raw mut state.p_setup.playerstarts as *mut mapthing_t)
            .offset(playernum as isize) as *mut mapthing_t;
        if G_CheckSpot(state, playernum, spot) {
            let spot = (&raw mut state.p_setup.playerstarts as *mut mapthing_t)
                .offset(playernum as isize) as *mut mapthing_t;
            P_SpawnPlayer(state, spot);
            return;
        }
        i = 0 as i32;
        while i < MAXPLAYERS {
            let spot = (&raw mut state.p_setup.playerstarts as *mut mapthing_t).offset(i as isize)
                as *mut mapthing_t;
            if G_CheckSpot(state, playernum, spot) {
                state.p_setup.playerstarts[i as usize].type_0 = (playernum + 1 as i32) as i16;
                let spot = (&raw mut state.p_setup.playerstarts as *mut mapthing_t)
                    .offset(i as isize) as *mut mapthing_t;
                P_SpawnPlayer(state, spot);
                state.p_setup.playerstarts[i as usize].type_0 = (i + 1 as i32) as i16;
                return;
            }
            i += 1;
        }
        let spot = (&raw mut state.p_setup.playerstarts as *mut mapthing_t)
            .offset(playernum as isize) as *mut mapthing_t;
        P_SpawnPlayer(state, spot);
    };
}
pub fn G_ScreenShot(state: &mut GameState) {
    state.g_game.gameaction = GameAction::ga_screenshot;
}
#[no_mangle]
pub static pars: [[i32; 10]; 4] = [
    [0 as i32; 10],
    [
        0 as i32, 30 as i32, 75 as i32, 120 as i32, 90 as i32, 165 as i32, 180 as i32, 180 as i32,
        30 as i32, 165 as i32,
    ],
    [
        0 as i32, 90 as i32, 90 as i32, 90 as i32, 120 as i32, 90 as i32, 360 as i32, 240 as i32,
        30 as i32, 170 as i32,
    ],
    [
        0 as i32, 90 as i32, 45 as i32, 90 as i32, 150 as i32, 90 as i32, 90 as i32, 165 as i32,
        30 as i32, 135 as i32,
    ],
];
#[no_mangle]
pub static cpars: [i32; 32] = [
    30 as i32, 90 as i32, 120 as i32, 120 as i32, 90 as i32, 150 as i32, 120 as i32, 120 as i32,
    270 as i32, 90 as i32, 210 as i32, 150 as i32, 150 as i32, 150 as i32, 210 as i32, 150 as i32,
    420 as i32, 150 as i32, 210 as i32, 150 as i32, 240 as i32, 150 as i32, 180 as i32, 150 as i32,
    150 as i32, 300 as i32, 330 as i32, 420 as i32, 300 as i32, 180 as i32, 120 as i32, 30 as i32,
];
pub fn G_ExitLevel(state: &mut GameState) {
    state.g_game.secretexit = false;
    state.g_game.gameaction = GameAction::ga_completed;
}
pub unsafe fn G_SecretExitLevel(state: &mut GameState) {
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as u32
        && W_CheckNumForName(&mut state.w_wad, "map31") < 0 as i32
    {
        state.g_game.secretexit = false;
    } else {
        state.g_game.secretexit = true;
    }
    state.g_game.gameaction = GameAction::ga_completed;
}
pub unsafe fn G_DoCompleted(state: &mut GameState) {
    let mut i: i32 = 0;
    state.g_game.gameaction = GameAction::ga_nothing;
    i = 0 as i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] != 0 {
            G_PlayerFinishLevel(&mut state.g_game, i);
        }
        i += 1;
    }
    if state.am_map.automapactive {
        AM_Stop(state);
    }
    if state.doomstat.gamemode as u32 != GameMode_t::commercial as u32 {
        if state.doomstat.gameversion == GameVersion::chex {
            if state.g_game.gamemap == 5 as i32 {
                state.g_game.gameaction = GameAction::ga_victory;
                return;
            }
        } else {
            match state.g_game.gamemap {
                8 => {
                    state.g_game.gameaction = GameAction::ga_victory;
                    return;
                }
                9 => {
                    i = 0 as i32;
                    while i < MAXPLAYERS {
                        state.g_game.players[i as usize].didsecret = true;
                        i += 1;
                    }
                }
                _ => {}
            }
        }
    }
    if state.g_game.gamemap == 8 as i32 && state.doomstat.gamemode as u32 != GameMode_t::commercial as u32 {
        state.g_game.gameaction = GameAction::ga_victory;
        return;
    }
    if state.g_game.gamemap == 9 as i32 && state.doomstat.gamemode as u32 != GameMode_t::commercial as u32 {
        i = 0 as i32;
        while i < MAXPLAYERS {
            state.g_game.players[i as usize].didsecret = true;
            i += 1;
        }
    }
    state.g_game.wminfo.didsecret =
        state.g_game.players[state.g_game.consoleplayer as usize].didsecret;
    state.g_game.wminfo.epsd = state.g_game.gameepisode - 1 as i32;
    state.g_game.wminfo.last = state.g_game.gamemap - 1 as i32;
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as u32 {
        if state.g_game.secretexit {
            match state.g_game.gamemap {
                15 => {
                    state.g_game.wminfo.next = 30 as i32;
                }
                31 => {
                    state.g_game.wminfo.next = 31 as i32;
                }
                _ => {}
            }
        } else {
            match state.g_game.gamemap {
                31 | 32 => {
                    state.g_game.wminfo.next = 15 as i32;
                }
                _ => {
                    state.g_game.wminfo.next = state.g_game.gamemap;
                }
            }
        }
    } else if state.g_game.secretexit {
        state.g_game.wminfo.next = 8 as i32;
    } else if state.g_game.gamemap == 9 as i32 {
        match state.g_game.gameepisode {
            1 => {
                state.g_game.wminfo.next = 3 as i32;
            }
            2 => {
                state.g_game.wminfo.next = 5 as i32;
            }
            3 => {
                state.g_game.wminfo.next = 6 as i32;
            }
            4 => {
                state.g_game.wminfo.next = 2 as i32;
            }
            _ => {}
        }
    } else {
        state.g_game.wminfo.next = state.g_game.gamemap;
    }
    state.g_game.wminfo.maxkills = state.g_game.totalkills;
    state.g_game.wminfo.maxitems = state.g_game.totalitems;
    state.g_game.wminfo.maxsecret = state.g_game.totalsecret;
    state.g_game.wminfo.maxfrags = 0 as i32;
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as u32 {
        state.g_game.wminfo.partime = TICRATE * cpars[(state.g_game.gamemap - 1 as i32) as usize];
    } else if state.g_game.gameepisode < 4 as i32 {
        state.g_game.wminfo.partime =
            TICRATE * pars[state.g_game.gameepisode as usize][state.g_game.gamemap as usize];
    } else {
        state.g_game.wminfo.partime = TICRATE * cpars[state.g_game.gamemap as usize];
    }
    state.g_game.wminfo.pnum = state.g_game.consoleplayer;
    i = 0 as i32;
    while i < MAXPLAYERS {
        state.g_game.wminfo.plyr[i as usize].in_0 = state.g_game.playeringame[i as usize] != 0;
        state.g_game.wminfo.plyr[i as usize].skills = state.g_game.players[i as usize].killcount;
        state.g_game.wminfo.plyr[i as usize].sitems = state.g_game.players[i as usize].itemcount;
        state.g_game.wminfo.plyr[i as usize].ssecret = state.g_game.players[i as usize].secretcount;
        state.g_game.wminfo.plyr[i as usize].stime = state.p_tick.leveltime;
        memcpy(
            &raw mut (*(&raw mut state.g_game.wminfo.plyr as *mut wbplayerstruct_t)
                .offset(i as isize))
            .frags as *mut i32 as *mut ::core::ffi::c_void,
            &raw mut (*(&raw mut state.g_game.players as *mut player_t).offset(i as isize)).frags
                as *mut i32 as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[i32; 4]>() as size_t,
        );
        i += 1;
    }
    state.g_game.gamestate = GameScreenState::GS_INTERMISSION;
    state.g_game.viewactive = false;
    state.am_map.automapactive = false;
    let wminfo = &raw mut state.g_game.wminfo;
    StatCopy(state, wminfo);
    WI_Start(state, wminfo);
}
pub unsafe fn G_WorldDone(state: &mut GameState) {
    state.g_game.gameaction = GameAction::ga_worlddone;
    if state.g_game.secretexit {
        state.g_game.players[state.g_game.consoleplayer as usize].didsecret = true;
    }
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as u32 {
        let mut current_block_3: u64;
        match state.g_game.gamemap {
            15 | 31 => {
                if !state.g_game.secretexit {
                    current_block_3 = 6937071982253665452;
                } else {
                    current_block_3 = 9744923308842414524;
                }
            }
            6 | 11 | 20 | 30 => {
                current_block_3 = 9744923308842414524;
            }
            _ => {
                current_block_3 = 6937071982253665452;
            }
        }
        match current_block_3 {
            9744923308842414524 => {
                F_StartFinale(state);
            }
            _ => {}
        }
    }
}
pub unsafe fn G_DoWorldDone(state: &mut GameState) {
    state.g_game.gamestate = GameScreenState::GS_LEVEL;
    state.g_game.gamemap = state.g_game.wminfo.next + 1 as i32;
    G_DoLoadLevel(state);
    state.g_game.gameaction = GameAction::ga_nothing;
    state.g_game.viewactive = true;
}
pub fn G_LoadGame(state: &mut GameState, name: &str) {
    state.g_game.savename = name.to_string();
    state.g_game.gameaction = GameAction::ga_loadgame;
}
pub unsafe fn G_DoLoadGame(state: &mut GameState) {
    let mut savedleveltime: i32 = 0;
    state.g_game.gameaction = GameAction::ga_nothing;
    state.p_saveg.save_stream = std::fs::File::open(&state.g_game.savename).ok();
    if state.p_saveg.save_stream.is_none() {
        return;
    }
    state.p_saveg.savegame_error = false;
    if !P_ReadSaveGameHeader(state) {
        state.p_saveg.save_stream = None;
        return;
    }
    savedleveltime = state.p_tick.leveltime;
    let (skill, episode, map) = (
        state.g_game.gameskill,
        state.g_game.gameepisode,
        state.g_game.gamemap,
    );
    G_InitNew(state, skill, episode, map);
    state.p_tick.leveltime = savedleveltime;
    P_UnArchivePlayers(state);
    P_UnArchiveWorld(state);
    P_UnArchiveThinkers(state);
    P_UnArchiveSpecials(state);
    if !P_ReadSaveGameEOF(state) {
        I_Error("Bad savegame");
    }
    state.p_saveg.save_stream = None;
    if state.r_main.setsizeneeded {
        R_ExecuteSetViewSize(state);
    }
    R_FillBackScreen(state);
}
pub fn G_SaveGame(
    state: &mut GameState,
    mut slot: i32,
    description: &str,
) {
    state.g_game.savegameslot = slot;
    state.g_game.savedescription = description.to_string();
    state.g_game.sendsave = true;
}
pub unsafe fn G_DoSaveGame(state: &mut GameState) {
    let mut recovery_savegame_file: Option<String> = None;
    let temp_savegame_file = P_TempSaveGameFile(state);
    let savegame_file = P_SaveGameFile(state, state.g_game.savegameslot);
    state.p_saveg.save_stream = std::fs::File::create(&temp_savegame_file).ok();
    if state.p_saveg.save_stream.is_none() {
        let recovery_file = M_TempFile("recovery.dsg");
        state.p_saveg.save_stream = std::fs::File::create(&recovery_file).ok();
        if state.p_saveg.save_stream.is_none() {
            I_Error(&format!(
                "Failed to open either '{}' or '{}' to write savegame.",
                temp_savegame_file, recovery_file,
            ));
        }
        recovery_savegame_file = Some(recovery_file);
    }
    state.p_saveg.savegame_error = false;
    let savedescription = state.g_game.savedescription.clone();
    P_WriteSaveGameHeader(state, &savedescription);
    P_ArchivePlayers(state);
    P_ArchiveWorld(state);
    P_ArchiveThinkers(state);
    P_ArchiveSpecials(state);
    P_WriteSaveGameEOF(state);
    if state.g_game.vanilla_savegame_limit != 0
        && state
            .p_saveg
            .save_stream
            .as_mut()
            .unwrap()
            .stream_position()
            .unwrap_or(0)
            > SAVEGAMESIZE as u64
    {
        I_Error("Savegame buffer overrun");
    }
    state.p_saveg.save_stream = None;
    if let Some(recovery_file) = &recovery_savegame_file {
        I_Error(&format!(
            "Failed to open savegame file '{}' for writing.\nBut your game has been saved to '{}' for recovery.",
            temp_savegame_file, recovery_file,
        ));
    }
    let _ = std::fs::remove_file(&savegame_file);
    let _ = std::fs::rename(&temp_savegame_file, &savegame_file);
    state.g_game.gameaction = GameAction::ga_nothing;
    state.g_game.savedescription.clear();
    state.g_game.players[state.g_game.consoleplayer as usize].message =
        Some("game saved.".to_string());
    R_FillBackScreen(state);
}
pub fn G_DeferedInitNew(
    state: &mut GameState,
    mut skill: SkillType,
    mut episode: i32,
    mut map: i32,
) {
    state.g_game.d_skill = skill;
    state.g_game.d_episode = episode;
    state.g_game.d_map = map;
    state.g_game.gameaction = GameAction::ga_newgame;
}
pub unsafe fn G_DoNewGame(state: &mut GameState) {
    state.g_game.demoplayback = false;
    state.g_game.netdemo = false;
    state.g_game.netgame = false;
    state.g_game.deathmatch = false_0;
    state.g_game.playeringame[3 as i32 as usize] = 0 as boolean;
    state.g_game.playeringame[2 as i32 as usize] = state.g_game.playeringame[3 as i32 as usize];
    state.g_game.playeringame[1 as i32 as usize] = state.g_game.playeringame[2 as i32 as usize];
    state.d_main.respawnparm = false;
    state.d_main.fastparm = false;
    state.d_main.nomonsters = false;
    state.g_game.consoleplayer = 0 as i32;
    let (d_skill, d_episode, d_map) = (
        state.g_game.d_skill,
        state.g_game.d_episode,
        state.g_game.d_map,
    );
    G_InitNew(state, d_skill, d_episode, d_map);
    state.g_game.gameaction = GameAction::ga_nothing;
}
pub unsafe fn G_InitNew(state: &mut GameState, mut skill: SkillType, mut episode: i32, mut map: i32) {
    let skytexturename: &str;
    let mut i: i32 = 0;
    if state.g_game.paused {
        state.g_game.paused = false;
        S_ResumeSound(state);
    }
    if skill > SkillType::sk_nightmare {
        skill = SkillType::sk_nightmare;
    }
    if state.doomstat.gameversion.is_ultimate_or_higher() {
        if episode == 0 as i32 {
            episode = 4 as i32;
        }
    } else {
        if episode < 1 as i32 {
            episode = 1 as i32;
        }
        if episode > 3 as i32 {
            episode = 3 as i32;
        }
    }
    if episode > 1 as i32 && state.doomstat.gamemode as u32 == GameMode_t::shareware as u32 {
        episode = 1 as i32;
    }
    if map < 1 as i32 {
        map = 1 as i32;
    }
    if map > 9 as i32 && state.doomstat.gamemode as u32 != GameMode_t::commercial as u32 {
        map = 9 as i32;
    }
    M_ClearRandom(&mut state.m_random);
    if skill == SkillType::sk_nightmare || state.d_main.respawnparm {
        state.g_game.respawnmonsters = true;
    } else {
        state.g_game.respawnmonsters = false;
    }
    if state.d_main.fastparm
        || skill == SkillType::sk_nightmare
            && state.g_game.gameskill != SkillType::sk_nightmare
    {
        i = S_SARG_RUN1 as i32;
        while i <= S_SARG_PAIN2 as i32 {
            state.info.states[i as usize].tics >>= 1 as i32;
            i += 1;
        }
        state.info.mobjinfo[MT_BRUISERSHOT as i32 as usize].speed = 20 as i32 * FRACUNIT;
        state.info.mobjinfo[MT_HEADSHOT as i32 as usize].speed = 20 as i32 * FRACUNIT;
        state.info.mobjinfo[MT_TROOPSHOT as i32 as usize].speed = 20 as i32 * FRACUNIT;
    } else if skill != SkillType::sk_nightmare
        && state.g_game.gameskill == SkillType::sk_nightmare
    {
        i = S_SARG_RUN1 as i32;
        while i <= S_SARG_PAIN2 as i32 {
            state.info.states[i as usize].tics <<= 1 as i32;
            i += 1;
        }
        state.info.mobjinfo[MT_BRUISERSHOT as i32 as usize].speed = 15 as i32 * FRACUNIT;
        state.info.mobjinfo[MT_HEADSHOT as i32 as usize].speed = 10 as i32 * FRACUNIT;
        state.info.mobjinfo[MT_TROOPSHOT as i32 as usize].speed = 10 as i32 * FRACUNIT;
    }
    i = 0 as i32;
    while i < MAXPLAYERS {
        state.g_game.players[i as usize].playerstate = PlayerState::PST_REBORN;
        i += 1;
    }
    state.g_game.usergame = true;
    state.g_game.paused = false;
    state.g_game.demoplayback = false;
    state.am_map.automapactive = false;
    state.g_game.viewactive = true;
    state.g_game.gameepisode = episode;
    state.g_game.gamemap = map;
    state.g_game.gameskill = skill;
    state.g_game.viewactive = true;
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as u32 {
        if state.g_game.gamemap < 12 as i32 {
            skytexturename = "SKY1";
        } else if state.g_game.gamemap < 21 as i32 {
            skytexturename = "SKY2";
        } else {
            skytexturename = "SKY3";
        }
    } else {
        match state.g_game.gameepisode {
            2 => {
                skytexturename = "SKY2";
            }
            3 => {
                skytexturename = "SKY3";
            }
            4 => {
                skytexturename = "SKY4";
            }
            1 | _ => {
                skytexturename = "SKY1";
            }
        }
    }
    state.r_sky.skytexture = R_TextureNumForName(&mut state.r_data, skytexturename);
    G_DoLoadLevel(state);
}
pub const DEMOMARKER: i32 = 0x80;
pub unsafe fn G_ReadDemoTiccmd(state: &mut GameState, mut cmd: *mut ticcmd_t) {
    if *state.g_game.demo_p as i32 == DEMOMARKER {
        G_CheckDemoStatus(state);
        return;
    }
    let fresh18 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    (*cmd).forwardmove = *fresh18 as i8;
    let fresh19 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    (*cmd).sidemove = *fresh19 as i8;
    if state.g_game.longtics {
        let fresh20 = state.g_game.demo_p;
        state.g_game.demo_p = state.g_game.demo_p.offset(1);
        (*cmd).angleturn = *fresh20 as i16;
        let fresh21 = state.g_game.demo_p;
        state.g_game.demo_p = state.g_game.demo_p.offset(1);
        (*cmd).angleturn = ((*cmd).angleturn as i32 | (*fresh21 as i32) << 8 as i32) as i16;
    } else {
        let fresh22 = state.g_game.demo_p;
        state.g_game.demo_p = state.g_game.demo_p.offset(1);
        (*cmd).angleturn = ((*fresh22 as u8 as i32) << 8 as i32) as i16;
    }
    let fresh23 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    (*cmd).buttons = *fresh23 as u8 as byte;
}
unsafe fn IncreaseDemoBuffer(state: &mut GameState) {
    let mut current_length: i32 = 0;
    let mut new_demobuffer: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut new_demop: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut new_length: i32 = 0;
    current_length = state.g_game.demoend.offset_from(state.g_game.demobuffer) as i64 as i32;
    new_length = current_length * 2 as i32;
    new_demobuffer = Z_Malloc(
        &mut state.z_zone,
        new_length,
        PU_STATIC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut byte;
    new_demop = new_demobuffer
        .offset(state.g_game.demo_p.offset_from(state.g_game.demobuffer) as i64 as isize);
    memcpy(
        new_demobuffer as *mut ::core::ffi::c_void,
        state.g_game.demobuffer as *const ::core::ffi::c_void,
        current_length as size_t,
    );
    Z_Free(
        &mut state.z_zone,
        state.g_game.demobuffer as *mut ::core::ffi::c_void,
    );
    state.g_game.demobuffer = new_demobuffer;
    state.g_game.demo_p = new_demop;
    state.g_game.demoend = state.g_game.demobuffer.offset(new_length as isize);
}
pub unsafe fn G_WriteDemoTiccmd(state: &mut GameState, mut cmd: *mut ticcmd_t) {
    let mut demo_start: *mut byte = ::core::ptr::null_mut::<byte>();
    if state.g_game.gamekeydown[state.m_controls.key_demo_quit as usize] != 0 {
        G_CheckDemoStatus(state);
    }
    demo_start = state.g_game.demo_p;
    let fresh12 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    *fresh12 = (*cmd).forwardmove as byte;
    let fresh13 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    *fresh13 = (*cmd).sidemove as byte;
    if state.g_game.longtics {
        let fresh14 = state.g_game.demo_p;
        state.g_game.demo_p = state.g_game.demo_p.offset(1);
        *fresh14 = ((*cmd).angleturn as i32 & 0xff as i32) as byte;
        let fresh15 = state.g_game.demo_p;
        state.g_game.demo_p = state.g_game.demo_p.offset(1);
        *fresh15 = ((*cmd).angleturn as i32 >> 8 as i32 & 0xff as i32) as byte;
    } else {
        let fresh16 = state.g_game.demo_p;
        state.g_game.demo_p = state.g_game.demo_p.offset(1);
        *fresh16 = ((*cmd).angleturn as i32 >> 8 as i32) as byte;
    }
    let fresh17 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    *fresh17 = (*cmd).buttons;
    state.g_game.demo_p = demo_start;
    if state.g_game.demo_p > state.g_game.demoend.offset(-(16 as i32 as isize)) {
        if state.g_game.vanilla_demo_limit != 0 {
            G_CheckDemoStatus(state);
            return;
        } else {
            IncreaseDemoBuffer(state);
        }
    }
    G_ReadDemoTiccmd(state, cmd);
}
pub unsafe fn G_RecordDemo(state: &mut GameState, name: &str) {
    let mut i: i32 = 0;
    let mut maxsize: i32 = 0;
    state.g_game.usergame = false;
    let demoname_full = format!("{}.lmp", name);
    let demoname_bytes = demoname_full.as_bytes();
    let demoname_size = (demoname_bytes.len() + 1) as size_t;
    state.g_game.demoname = Z_Malloc(
        &mut state.z_zone,
        demoname_size as i32,
        PU_STATIC as i32,
        NULL,
    ) as *mut ::core::ffi::c_char;
    ::core::ptr::copy_nonoverlapping(
        demoname_bytes.as_ptr(),
        state.g_game.demoname as *mut u8,
        demoname_bytes.len(),
    );
    *(state.g_game.demoname as *mut u8).add(demoname_bytes.len()) = 0;
    maxsize = 0x20000 as i32;
    i = M_CheckParmWithArgs(state, "-maxdemo", 1 as i32);
    if i != 0 {
        maxsize =
            M_ArgvAtoi(&state.m_argv.myargv[(i + 1 as i32) as usize])
                * 1024 as i32;
    }
    state.g_game.demobuffer =
        Z_Malloc(&mut state.z_zone, maxsize, PU_STATIC as i32, NULL) as *mut byte;
    state.g_game.demoend = state.g_game.demobuffer.offset(maxsize as isize);
    state.g_game.demorecording = true;
}
pub fn G_VanillaVersionCode(state: &mut DoomstatState) -> i32 {
    match state.gameversion as u32 {
        0 => {
            I_Error("Doom 1.2 does not have a version code!");
        }
        1 => {}
        2 => return 107 as i32,
        3 => return 108 as i32,
        4 | _ => return 109 as i32,
    }
    return 106 as i32;
}
pub unsafe fn G_BeginRecording(state: &mut GameState) {
    let mut i: i32 = 0;
    state.g_game.longtics = M_CheckParm(state, "-longtics") != 0 as i32;
    state.g_game.lowres_turn = !state.g_game.longtics;
    state.g_game.demo_p = state.g_game.demobuffer;
    if state.g_game.longtics {
        let fresh0 = state.g_game.demo_p;
        state.g_game.demo_p = state.g_game.demo_p.offset(1);
        *fresh0 = DOOM_191_VERSION as byte;
    } else {
        let fresh1 = state.g_game.demo_p;
        state.g_game.demo_p = state.g_game.demo_p.offset(1);
        *fresh1 = G_VanillaVersionCode(&mut state.doomstat) as byte;
    }
    let fresh2 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    *fresh2 = state.g_game.gameskill as byte;
    let fresh3 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    *fresh3 = state.g_game.gameepisode as byte;
    let fresh4 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    *fresh4 = state.g_game.gamemap as byte;
    let fresh5 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    *fresh5 = state.g_game.deathmatch as byte;
    let fresh6 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    *fresh6 = state.d_main.respawnparm as byte;
    let fresh7 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    *fresh7 = state.d_main.fastparm as byte;
    let fresh8 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    *fresh8 = state.d_main.nomonsters as byte;
    let fresh9 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    *fresh9 = state.g_game.consoleplayer as byte;
    i = 0 as i32;
    while i < MAXPLAYERS {
        let fresh10 = state.g_game.demo_p;
        state.g_game.demo_p = state.g_game.demo_p.offset(1);
        *fresh10 = state.g_game.playeringame[i as usize] as byte;
        i += 1;
    }
}
pub unsafe fn G_DeferedPlayDemo(state: &mut GameState, mut name: *mut ::core::ffi::c_char) {
    state.g_game.defdemoname = name;
    state.g_game.gameaction = GameAction::ga_playdemo;
}
fn DemoVersionDescription(_state: &mut GameState, version: i32) -> String {
    match version {
        104 => return "v1.4".to_string(),
        105 => return "v1.5".to_string(),
        106 => return "v1.6/v1.666".to_string(),
        107 => return "v1.7/v1.7a".to_string(),
        108 => return "v1.8".to_string(),
        109 => return "v1.9".to_string(),
        _ => {}
    }
    if version >= 0 as i32 && version <= 4 as i32 {
        "v1.0/v1.1/v1.2".to_string()
    } else {
        format!("{}.{} (unknown)", version / 100 as i32, version % 100 as i32)
    }
}
pub unsafe fn G_DoPlayDemo(state: &mut GameState) {
    let mut skill: SkillType = SkillType::sk_baby;
    let mut i: i32 = 0;
    let mut episode: i32 = 0;
    let mut map: i32 = 0;
    let mut demoversion: i32 = 0;
    state.g_game.gameaction = GameAction::ga_nothing;
    state.g_game.demo_p = W_CacheLumpName(state, 
        &wad_name8_to_string(state.g_game.defdemoname),
        PU_STATIC as i32,
    ) as *mut byte;
    state.g_game.demobuffer = state.g_game.demo_p;
    let fresh24 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    demoversion = *fresh24 as i32;
    if demoversion == G_VanillaVersionCode(&mut state.doomstat) {
        state.g_game.longtics = false;
    } else if demoversion == DOOM_191_VERSION {
        state.g_game.longtics = true;
    } else {
        println!(
            "Demo is from a different game version!\n(read {}, should be {})\n\n*** You may need to upgrade your version of Doom to v1.9. ***\n    See: https://www.doomworld.com/classicdoom/info/patches.php\n    This appears to be {}.",
            demoversion,
            G_VanillaVersionCode(&mut state.doomstat),
            DemoVersionDescription(state, demoversion),
        );
    }
    let fresh25 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    skill = skill_from_raw(*fresh25 as i32);
    let fresh26 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    episode = *fresh26 as i32;
    let fresh27 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    map = *fresh27 as i32;
    let fresh28 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    state.g_game.deathmatch = *fresh28 as i32;
    let fresh29 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    state.d_main.respawnparm = *fresh29 != 0;
    let fresh30 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    state.d_main.fastparm = *fresh30 != 0;
    let fresh31 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    state.d_main.nomonsters = *fresh31 != 0;
    let fresh32 = state.g_game.demo_p;
    state.g_game.demo_p = state.g_game.demo_p.offset(1);
    state.g_game.consoleplayer = *fresh32 as i32;
    i = 0 as i32;
    while i < MAXPLAYERS {
        let fresh33 = state.g_game.demo_p;
        state.g_game.demo_p = state.g_game.demo_p.offset(1);
        state.g_game.playeringame[i as usize] = *fresh33 as boolean;
        i += 1;
    }
    if state.g_game.playeringame[1 as i32 as usize] != 0
        || M_CheckParm(state, "-solo-net") > 0 as i32
        || M_CheckParm(state, "-netdemo") > 0 as i32
    {
        state.g_game.netgame = true;
        state.g_game.netdemo = true;
    }
    state.g_game.precache = false;
    G_InitNew(state, skill, episode, map);
    state.g_game.precache = true;
    state.g_game.starttime = I_GetTime(state);
    state.g_game.usergame = false;
    state.g_game.demoplayback = true;
}
pub unsafe fn G_TimeDemo(state: &mut GameState, mut name: *mut ::core::ffi::c_char) {
    state.g_game.nodrawers = M_CheckParm(state, "-nodraw") != 0;
    state.g_game.timingdemo = true;
    state.d_loop.singletics = true;
    state.g_game.defdemoname = name;
    state.g_game.gameaction = GameAction::ga_playdemo;
}
#[no_mangle]
pub unsafe fn G_CheckDemoStatus(state: &mut GameState) -> boolean {
    let mut endtime: i32 = 0;
    if state.g_game.timingdemo {
        let mut fps: f32 = 0.;
        let mut realtics: i32 = 0;
        endtime = I_GetTime(state);
        realtics = endtime - state.g_game.starttime;
        fps = state.d_loop.gametic as f32 * TICRATE as f32 / realtics as f32;
        state.g_game.timingdemo = false;
        state.g_game.demoplayback = false;
        I_Error(&format!(
            "timed {} gametics in {} realtics ({:.6} fps)",
            state.d_loop.gametic,
            realtics,
            fps as f64,
        ));
    }
    if state.g_game.demoplayback {
        W_ReleaseLumpName(
            &mut state.w_wad,
            &wad_name8_to_string(state.g_game.defdemoname),
        );
        state.g_game.demoplayback = false;
        state.g_game.netdemo = false;
        state.g_game.netgame = false;
        state.g_game.deathmatch = false_0;
        state.g_game.playeringame[3 as i32 as usize] = 0 as boolean;
        state.g_game.playeringame[2 as i32 as usize] =
            state.g_game.playeringame[3 as i32 as usize];
        state.g_game.playeringame[1 as i32 as usize] =
            state.g_game.playeringame[2 as i32 as usize];
        state.d_main.respawnparm = false;
        state.d_main.fastparm = false;
        state.d_main.nomonsters = false;
        state.g_game.consoleplayer = 0 as i32;
        if state.g_game.singledemo {
            I_Quit(state);
        } else {
            D_AdvanceDemo(state);
        }
        return true_0 as boolean;
    }
    if state.g_game.demorecording {
        let fresh11 = state.g_game.demo_p;
        state.g_game.demo_p = state.g_game.demo_p.offset(1);
        *fresh11 = DEMOMARKER as byte;
        let demo_len = state.g_game.demo_p.offset_from(state.g_game.demobuffer) as usize;
        M_WriteFile(
            &::std::ffi::CStr::from_ptr(state.g_game.demoname).to_string_lossy(),
            ::core::slice::from_raw_parts(state.g_game.demobuffer, demo_len),
        );
        Z_Free(
            &mut state.z_zone,
            state.g_game.demobuffer as *mut ::core::ffi::c_void,
        );
        state.g_game.demorecording = false;
        I_Error(&format!(
            "Demo {} recorded",
            ::std::ffi::CStr::from_ptr(state.g_game.demoname)
                .to_str()
                .unwrap(),
        ));
    }
    return false_0 as boolean;
}
pub const MAX_MOUSE_BUTTONS: i32 = 8;
