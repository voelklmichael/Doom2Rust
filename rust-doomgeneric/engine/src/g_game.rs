use crate::am_map::AM_Responder;
use crate::am_map::AM_Stop;
use crate::am_map::AM_Ticker;
use crate::d_event::event_t;
use crate::d_event::EvType;
use crate::d_event::GameAction;
use crate::d_event::GameScreenState;
use crate::d_loop::BACKUPTICS;
use crate::d_main::D_AdvanceDemo;
use crate::d_main::D_PageTicker;
use crate::d_mode::GameMission_t;
use crate::d_mode::GameMode_t;
use crate::d_mode::GameVersion;
use crate::d_mode::{skill_from_raw, SkillType};
use crate::d_player::weapontype_t;
use crate::d_player::PowerType;
use crate::d_player::{ammotype_t, NUMAMMO};
use crate::d_player::{player_s, player_t, PlayerId, PlayerState};
use crate::d_ticcmd::ticcmd_t;
use crate::d_ticcmd::{
    BTS_PAUSE, BTS_SAVEGAME, BTS_SAVEMASK, BTS_SAVESHIFT, BT_ATTACK, BT_CHANGE, BT_SPECIAL,
    BT_SPECIALMASK, BT_USE, BT_WEAPONSHIFT,
};
use crate::doomdef::false_0;
use crate::doomdef::true_0;
use crate::doomdef::MAXPLAYERS;
use crate::doomdef::TICRATE;
use crate::doomstat::DoomstatState;
use crate::f_finale::F_Responder;
use crate::f_finale::F_StartFinale;
use crate::f_finale::F_Ticker;
use crate::fixed_cstr::FixedCStr;
use crate::game_state::GameState;
use crate::hu_stuff::HU_Responder;
use crate::hu_stuff::HU_Ticker;
use crate::hu_stuff::HU_dequeueChatChar;
use crate::hu_stuff::PLAYER_NAMES;
use crate::i_system::I_Error;
use crate::i_system::I_Quit;
use crate::i_timer::I_GetTime;
use crate::m_argv::{M_ArgvAtoi, M_CheckParm, M_CheckParmWithArgs};
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::m_menu::M_StartControlPanel;
use crate::m_misc::M_TempFile;
use crate::m_misc::M_WriteFile;
use crate::m_random::M_ClearRandom;
use crate::m_random::P_Random;

use crate::p_inter::maxammo;
use crate::p_map::P_CheckPosition;
use crate::p_mobj::mapthing_t;
use crate::p_mobj::MobjId;
use crate::p_mobj::MobjType;
use crate::p_mobj::P_RemoveMobj;
use crate::p_mobj::P_SpawnMobj;
use crate::p_mobj::P_SpawnPlayer;
use crate::p_mobj::StateNum;
use crate::p_mobj::MF_SHADOW;
use crate::p_mobj::{pspdef_t};
use crate::p_saveg::P_ArchivePlayers;
use crate::p_saveg::P_ArchiveSpecials;
use crate::p_saveg::P_ArchiveThinkers;
use crate::p_saveg::P_ArchiveWorld;
use crate::p_saveg::P_ReadSaveGameEOF;
use crate::p_saveg::P_ReadSaveGameHeader;
use crate::p_saveg::P_SaveGameFile;
use crate::p_saveg::P_TempSaveGameFile;
use crate::p_saveg::P_UnArchivePlayers;
use crate::p_saveg::P_UnArchiveSpecials;
use crate::p_saveg::P_UnArchiveThinkers;
use crate::p_saveg::P_UnArchiveWorld;
use crate::p_saveg::P_WriteSaveGameEOF;
use crate::p_saveg::P_WriteSaveGameHeader;
use crate::p_setup::P_SetupLevel;

use crate::p_tick::P_Ticker;
use crate::r_data::R_FlatNumForName;
use crate::r_data::R_TextureNumForName;
use crate::r_draw::R_FillBackScreen;
use crate::r_main::R_ExecuteSetViewSize;
use crate::r_main::R_PointInSubsector;
use crate::s_sound::S_PauseSound;
use crate::s_sound::S_ResumeSound;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::sfx_telept;
use crate::st_stuff::ST_Responder;
use crate::st_stuff::ST_Ticker;
use crate::statdump::StatCopy;
use crate::stdint_types::byte;

use crate::tables::finecosine;
use crate::tables::finesine;
use crate::tables::finetangent;
use crate::tables::ANG45;
use crate::tables::ANGLETOFINESHIFT;
use crate::v_video::V_ScreenShot;
use crate::w_wad::{
    W_CheckNumForName, W_GetNumForName, W_LumpBytes, W_LumpLength, W_ReleaseLumpName,
};
use crate::wi_stuff::WI_End;
use crate::wi_stuff::WI_Start;
use crate::wi_stuff::WI_Ticker;
use crate::wi_stuff::{wbplayerstruct_t, wbstartstruct_t};
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
    pub playeringame: [bool; 4],
    pub players: [player_t; 4],
    pub turbodetected: [bool; 4],
    pub consoleplayer: i32,
    pub displayplayer: i32,
    pub levelstarttic: i32,
    pub totalsecret: i32,
    pub totalkills: i32,
    pub totalitems: i32,
    pub demoname: String,
    pub demorecording: bool,
    pub longtics: bool,
    pub lowres_turn: bool,
    pub demoplayback: bool,
    pub netdemo: bool,
    pub demobuffer: Vec<byte>,
    pub demo_p: usize,
    pub demoend: usize,
    pub singledemo: bool,
    pub precache: bool,
    pub testcontrols: bool,
    pub testcontrols_mousespeed: i32,
    pub wminfo: wbstartstruct_t,
    pub consistancy: [[byte; 128]; 4],
    pub forwardmove: [fixed_t; 2],
    pub sidemove: [fixed_t; 2],
    pub next_weapon: i32,
    pub gamekeydown: [bool; 256],
    pub turnheld: i32,
    pub mousearray: [bool; 9],
    pub mousex: i32,
    pub mousey: i32,
    pub dclicktime: i32,
    pub dclickstate: bool,
    pub dclicks: i32,
    pub dclicktime2: i32,
    pub dclickstate2: bool,
    pub dclicks2: i32,
    pub joyxmove: i32,
    pub joyymove: i32,
    pub joystrafemove: i32,
    pub joyarray: [bool; 21],
    pub savegameslot: i32,
    pub savedescription: String,
    pub bodyque: [Option<MobjId>; 32],
    pub bodyqueslot: i32,
    pub vanilla_savegame_limit: i32,
    pub vanilla_demo_limit: i32,
    pub secretexit: bool,
    pub savename: String,
    pub d_skill: SkillType,
    pub d_episode: i32,
    pub d_map: i32,
    pub defdemoname: FixedCStr<8>,
    pub g_build_ticcmd_carry: i16,
}

const NEW_PLAYER: player_s = player_s {
    mo: None,
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
            playeringame: [false; 4],
            players: [NEW_PLAYER, NEW_PLAYER, NEW_PLAYER, NEW_PLAYER],
            turbodetected: [false; 4],
            consoleplayer: 0,
            displayplayer: 0,
            levelstarttic: 0,
            totalsecret: 0,
            totalkills: 0,
            totalitems: 0,
            demoname: String::new(),
            demorecording: false,
            longtics: false,
            lowres_turn: false,
            demoplayback: false,
            netdemo: false,
            demobuffer: Vec::new(),
            demo_p: 0,
            demoend: 0,
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
            forwardmove: [0x19_i32, 0x32_i32],
            sidemove: [0x18_i32, 0x28_i32],
            next_weapon: 0,
            gamekeydown: [false; 256],
            turnheld: 0,
            mousearray: [false; 9],
            mousex: 0,
            mousey: 0,
            dclicktime: 0,
            dclickstate: false,
            dclicks: 0,
            dclicktime2: 0,
            dclickstate2: false,
            dclicks2: 0,
            joyxmove: 0,
            joyymove: 0,
            joystrafemove: 0,
            joyarray: [false; 21],
            savegameslot: 0,
            savedescription: String::new(),
            bodyque: [None; 32],
            bodyqueslot: 0,
            vanilla_savegame_limit: 1,
            vanilla_demo_limit: 1,
            secretexit: false,
            savename: String::new(),
            d_skill: SkillType::sk_baby,
            d_episode: 0,
            d_map: 0,
            defdemoname: FixedCStr::from_array([0; 8]),
            g_build_ticcmd_carry: 0,
        }
    }

    pub fn player_mut(&mut self, id: PlayerId) -> &mut player_t {
        &mut self.players[id.0 as usize]
    }

    fn demo_read_byte(&mut self) -> byte {
        let b = self.demobuffer[self.demo_p];
        self.demo_p += 1;
        b
    }

    fn demo_write_byte(&mut self, b: byte) {
        self.demobuffer[self.demo_p] = b;
        self.demo_p += 1;
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
pub static angleturn: [fixed_t; 3] = [640_i32, 1280_i32, 320_i32];
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
fn WeaponSelectable(state: &mut GameState, mut weapon: weapontype_t) -> bool {
    if weapon as u32 == weapontype_t::wp_supershotgun as u32
        && (if state.doomstat.gamemission as u32 == GameMission_t::pack_chex as u32 {
            GameMission_t::doom as u32
        } else if state.doomstat.gamemission as u32 == GameMission_t::pack_hacx as u32 {
            GameMission_t::doom2 as u32
        } else {
            state.doomstat.gamemission as u32
        }) == GameMission_t::doom as u32
    {
        return false;
    }
    if (weapon as u32 == weapontype_t::wp_plasma as u32
        || weapon as u32 == weapontype_t::wp_bfg as u32)
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
            [weapontype_t::wp_chainsaw as usize]
        && state.g_game.players[state.g_game.consoleplayer as usize].powers
            [PowerType::pw_strength as usize]
            == 0
    {
        return false;
    }
    true
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
    i = 0_i32;
    while (i as usize)
        < ::core::mem::size_of::<[C2RustUnnamed_5; 9]>()
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_5>())
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
                ::core::mem::size_of::<[C2RustUnnamed_5; 9]>()
                    .wrapping_div(::core::mem::size_of::<C2RustUnnamed_5>()),
            )
            .wrapping_rem(
                ::core::mem::size_of::<[C2RustUnnamed_5; 9]>()
                    .wrapping_div(::core::mem::size_of::<C2RustUnnamed_5>()),
            ) as i32;
        if !(i != start_i && !WeaponSelectable(state, weapon_order_table[i as usize].weapon)) {
            break;
        }
    }
    weapon_order_table[i as usize].weapon_num as i32
}
pub fn G_BuildTiccmd(state: &mut GameState, cmd: &mut ticcmd_t, mut maketic: i32) {
    let mut i: i32 = 0;
    let mut strafe: bool = false;
    let mut bstrafe: bool = false;
    let mut speed: i32 = 0;
    let mut tspeed: i32 = 0;
    let mut forward: i32 = 0;
    let mut side: i32 = 0;
    *cmd = ticcmd_t {
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
    };
    cmd.consistancy = state.g_game.consistancy[state.g_game.consoleplayer as usize]
        [(maketic % BACKUPTICS) as usize];
    strafe = state.g_game.gamekeydown[state.m_controls.key_strafe as usize]
        || state.g_game.mousearray[(state.m_controls.mousebstrafe + 1) as usize]
        || state.g_game.joyarray[(state.m_controls.joybstrafe + 1) as usize];
    speed = (state.m_controls.key_speed >= NUMKEYS
        || state.m_controls.joybspeed >= MAX_JOY_BUTTONS
        || state.g_game.gamekeydown[state.m_controls.key_speed as usize]
        || state.g_game.joyarray[(state.m_controls.joybspeed + 1) as usize]) as i32;
    side = 0_i32;
    forward = side;
    if state.g_game.joyxmove != 0_i32
        || state.g_game.gamekeydown[state.m_controls.key_right as usize]
        || state.g_game.gamekeydown[state.m_controls.key_left as usize]
    {
        state.g_game.turnheld += state.d_loop.ticdup;
    } else {
        state.g_game.turnheld = 0_i32;
    }
    if state.g_game.turnheld < SLOWTURNTICS {
        tspeed = 2_i32;
    } else {
        tspeed = speed;
    }
    if strafe {
        if state.g_game.gamekeydown[state.m_controls.key_right as usize] {
            side += state.g_game.sidemove[speed as usize];
        }
        if state.g_game.gamekeydown[state.m_controls.key_left as usize] {
            side -= state.g_game.sidemove[speed as usize];
        }
        if state.g_game.joyxmove > 0_i32 {
            side += state.g_game.sidemove[speed as usize];
        }
        if state.g_game.joyxmove < 0_i32 {
            side -= state.g_game.sidemove[speed as usize];
        }
    } else {
        if state.g_game.gamekeydown[state.m_controls.key_right as usize] {
            cmd.angleturn = (cmd.angleturn as i32 - angleturn[tspeed as usize]) as i16;
        }
        if state.g_game.gamekeydown[state.m_controls.key_left as usize] {
            cmd.angleturn = (cmd.angleturn as i32 + angleturn[tspeed as usize]) as i16;
        }
        if state.g_game.joyxmove > 0_i32 {
            cmd.angleturn = (cmd.angleturn as i32 - angleturn[tspeed as usize]) as i16;
        }
        if state.g_game.joyxmove < 0_i32 {
            cmd.angleturn = (cmd.angleturn as i32 + angleturn[tspeed as usize]) as i16;
        }
    }
    if state.g_game.gamekeydown[state.m_controls.key_up as usize] {
        forward += state.g_game.forwardmove[speed as usize];
    }
    if state.g_game.gamekeydown[state.m_controls.key_down as usize] {
        forward -= state.g_game.forwardmove[speed as usize];
    }
    if state.g_game.joyymove < 0_i32 {
        forward += state.g_game.forwardmove[speed as usize];
    }
    if state.g_game.joyymove > 0_i32 {
        forward -= state.g_game.forwardmove[speed as usize];
    }
    if state.g_game.gamekeydown[state.m_controls.key_strafeleft as usize]
        || state.g_game.joyarray[(state.m_controls.joybstrafeleft + 1) as usize]
        || state.g_game.mousearray[(state.m_controls.mousebstrafeleft + 1) as usize]
        || state.g_game.joystrafemove < 0_i32
    {
        side -= state.g_game.sidemove[speed as usize];
    }
    if state.g_game.gamekeydown[state.m_controls.key_straferight as usize]
        || state.g_game.joyarray[(state.m_controls.joybstraferight + 1) as usize]
        || state.g_game.mousearray[(state.m_controls.mousebstraferight + 1) as usize]
        || state.g_game.joystrafemove > 0_i32
    {
        side += state.g_game.sidemove[speed as usize];
    }
    cmd.chatchar = HU_dequeueChatChar(&mut state.hu_stuff) as byte;
    if state.g_game.gamekeydown[state.m_controls.key_fire as usize]
        || state.g_game.mousearray[(state.m_controls.mousebfire + 1) as usize]
        || state.g_game.joyarray[(state.m_controls.joybfire + 1) as usize]
    {
        cmd.buttons = (cmd.buttons as i32 | BT_ATTACK as i32) as byte;
    }
    if state.g_game.gamekeydown[state.m_controls.key_use as usize]
        || state.g_game.joyarray[(state.m_controls.joybuse + 1) as usize]
        || state.g_game.mousearray[(state.m_controls.mousebuse + 1) as usize]
    {
        cmd.buttons = (cmd.buttons as i32 | BT_USE as i32) as byte;
        state.g_game.dclicks = 0_i32;
    }
    if state.g_game.gamestate == GameScreenState::GS_LEVEL && state.g_game.next_weapon != 0_i32 {
        let next_weapon = state.g_game.next_weapon;
        i = G_NextWeapon(state, next_weapon);
        cmd.buttons = (cmd.buttons as i32 | BT_CHANGE as i32) as byte;
        cmd.buttons = (cmd.buttons as i32 | i << BT_WEAPONSHIFT as i32) as byte;
    } else {
        let weapon_keys = state.m_controls.weapon_keys();
        i = 0_i32;
        while (i as usize) < weapon_keys.len() {
            let key: i32 = weapon_keys[i as usize];
            if state.g_game.gamekeydown[key as usize] {
                cmd.buttons = (cmd.buttons as i32 | BT_CHANGE as i32) as byte;
                cmd.buttons = (cmd.buttons as i32 | i << BT_WEAPONSHIFT as i32) as byte;
                break;
            } else {
                i += 1;
            }
        }
    }
    state.g_game.next_weapon = 0_i32;
    if state.g_game.mousearray[(state.m_controls.mousebforward + 1) as usize] {
        forward += state.g_game.forwardmove[speed as usize];
    }
    if state.g_game.mousearray[(state.m_controls.mousebbackward + 1) as usize] {
        forward -= state.g_game.forwardmove[speed as usize];
    }
    if state.m_controls.dclick_use != 0 {
        if state.g_game.mousearray[(state.m_controls.mousebforward + 1) as usize]
            != state.g_game.dclickstate
            && state.g_game.dclicktime > 1_i32
        {
            state.g_game.dclickstate =
                state.g_game.mousearray[(state.m_controls.mousebforward + 1) as usize];
            if state.g_game.dclickstate {
                state.g_game.dclicks += 1;
            }
            if state.g_game.dclicks == 2_i32 {
                cmd.buttons = (cmd.buttons as i32 | BT_USE as i32) as byte;
                state.g_game.dclicks = 0_i32;
            } else {
                state.g_game.dclicktime = 0_i32;
            }
        } else {
            state.g_game.dclicktime += state.d_loop.ticdup;
            if state.g_game.dclicktime > 20_i32 {
                state.g_game.dclicks = 0_i32;
                state.g_game.dclickstate = false;
            }
        }
        bstrafe = state.g_game.mousearray[(state.m_controls.mousebstrafe + 1) as usize]
            || state.g_game.joyarray[(state.m_controls.joybstrafe + 1) as usize];
        if bstrafe != state.g_game.dclickstate2 && state.g_game.dclicktime2 > 1_i32 {
            state.g_game.dclickstate2 = bstrafe;
            if state.g_game.dclickstate2 {
                state.g_game.dclicks2 += 1;
            }
            if state.g_game.dclicks2 == 2_i32 {
                cmd.buttons = (cmd.buttons as i32 | BT_USE as i32) as byte;
                state.g_game.dclicks2 = 0_i32;
            } else {
                state.g_game.dclicktime2 = 0_i32;
            }
        } else {
            state.g_game.dclicktime2 += state.d_loop.ticdup;
            if state.g_game.dclicktime2 > 20_i32 {
                state.g_game.dclicks2 = 0_i32;
                state.g_game.dclickstate2 = false;
            }
        }
    }
    forward += state.g_game.mousey;
    if strafe {
        side += state.g_game.mousex * 2_i32;
    } else {
        cmd.angleturn = (cmd.angleturn as i32 - state.g_game.mousex * 0x8_i32) as i16;
    }
    if state.g_game.mousex == 0_i32 {
        state.g_game.testcontrols_mousespeed = 0_i32;
    }
    state.g_game.mousey = 0_i32;
    state.g_game.mousex = state.g_game.mousey;
    if forward > state.g_game.forwardmove[1] {
        forward = state.g_game.forwardmove[1];
    } else if forward < -state.g_game.forwardmove[1] {
        forward = -state.g_game.forwardmove[1];
    }
    if side > state.g_game.forwardmove[1] {
        side = state.g_game.forwardmove[1];
    } else if side < -state.g_game.forwardmove[1] {
        side = -state.g_game.forwardmove[1];
    }
    cmd.forwardmove = (cmd.forwardmove as i32 + forward) as i8;
    cmd.sidemove = (cmd.sidemove as i32 + side) as i8;
    if state.g_game.sendpause {
        state.g_game.sendpause = false;
        cmd.buttons = (BT_SPECIAL as i32 | BTS_PAUSE as i32) as byte;
    }
    if state.g_game.sendsave {
        state.g_game.sendsave = false;
        cmd.buttons = (BT_SPECIAL as i32
            | BTS_SAVEGAME as i32
            | state.g_game.savegameslot << BTS_SAVESHIFT as i32) as byte;
    }
    if state.g_game.lowres_turn {
        let mut desired_angleturn: i16 = 0;
        desired_angleturn =
            (cmd.angleturn as i32 + state.g_game.g_build_ticcmd_carry as i32) as i16;
        cmd.angleturn = ((desired_angleturn as i32 + 128_i32) & 0xff00_i32) as i16;
        state.g_game.g_build_ticcmd_carry =
            (desired_angleturn as i32 - cmd.angleturn as i32) as i16;
    }
}
pub fn G_DoLoadLevel(state: &mut GameState) {
    let mut i: i32 = 0;
    state.r_sky.skyflatnum = R_FlatNumForName(state, "F_SKY1");
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as u32
        && [GameVersion::final2, GameVersion::chex].contains(&state.doomstat.gameversion)
    {
        let skytexturename: &str;
        if state.g_game.gamemap < 12_i32 {
            skytexturename = "SKY1";
        } else if state.g_game.gamemap < 21_i32 {
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
    i = 0_i32;
    while i < MAXPLAYERS {
        state.g_game.turbodetected[i as usize] = false;
        if state.g_game.playeringame[i as usize]
            && state.g_game.players[i as usize].playerstate == PlayerState::PST_DEAD
        {
            state.g_game.players[i as usize].playerstate = PlayerState::PST_REBORN;
        }
        state.g_game.players[i as usize].frags = [0; 4];
        i += 1;
    }
    P_SetupLevel(state, state.g_game.gameepisode, state.g_game.gamemap);
    state.g_game.displayplayer = state.g_game.consoleplayer;
    state.g_game.gameaction = GameAction::ga_nothing;
    state.g_game.gamekeydown = [false; 256];
    state.g_game.joystrafemove = 0_i32;
    state.g_game.joyymove = state.g_game.joystrafemove;
    state.g_game.joyxmove = state.g_game.joyymove;
    state.g_game.mousey = 0_i32;
    state.g_game.mousex = state.g_game.mousey;
    state.g_game.paused = false;
    state.g_game.sendsave = state.g_game.paused;
    state.g_game.sendpause = state.g_game.sendsave;
    state.g_game.mousearray = [false; 9];
    state.g_game.joyarray = [false; 21];
    if state.g_game.testcontrols {
        state.g_game.players[state.g_game.consoleplayer as usize].message =
            Some("Press escape to quit.".to_string());
    }
}
fn SetJoyButtons(state: &mut GameState, mut buttons_mask: u32) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < MAX_JOY_BUTTONS {
        let mut button_on: i32 = (buttons_mask & (1_i32 << i) as u32 != 0_u32) as i32;
        if !state.g_game.joyarray[(i + 1) as usize] && button_on != 0 {
            if i == state.m_controls.joybprevweapon {
                state.g_game.next_weapon = -1_i32;
            } else if i == state.m_controls.joybnextweapon {
                state.g_game.next_weapon = 1_i32;
            }
        }
        state.g_game.joyarray[(i + 1) as usize] = button_on != 0;
        i += 1;
    }
}
fn SetMouseButtons(state: &mut GameState, mut buttons_mask: u32) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < MAX_MOUSE_BUTTONS {
        let mut button_on: u32 = (buttons_mask & (1_i32 << i) as u32 != 0_u32) as u32;
        if !state.g_game.mousearray[(i + 1) as usize] && button_on != 0 {
            if i == state.m_controls.mousebprevweapon {
                state.g_game.next_weapon = -1_i32;
            } else if i == state.m_controls.mousebnextweapon {
                state.g_game.next_weapon = 1_i32;
            }
        }
        state.g_game.mousearray[(i + 1) as usize] = button_on != 0;
        i += 1;
    }
}
pub fn G_Responder(state: &mut GameState, mut ev: event_t) -> bool {
    if state.g_game.gamestate == GameScreenState::GS_LEVEL
        && ev.type_0 == EvType::ev_keydown
        && ev.data1 == state.m_controls.key_spy
        && (state.g_game.singledemo || state.g_game.deathmatch == 0)
    {
        loop {
            state.g_game.displayplayer += 1;
            if state.g_game.displayplayer == MAXPLAYERS {
                state.g_game.displayplayer = 0_i32;
            }
            if !(!state.g_game.playeringame[state.g_game.displayplayer as usize]
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
    if state.g_game.gamestate == GameScreenState::GS_FINALE && F_Responder(state, &ev) {
        return true;
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
                state.g_game.gamekeydown[ev.data1 as usize] = true;
            }
            return true;
        }
        1 => {
            if ev.data1 < NUMKEYS {
                state.g_game.gamekeydown[ev.data1 as usize] = false;
            }
            return false;
        }
        2 => {
            SetMouseButtons(state, ev.data1 as u32);
            state.g_game.mousex = ev.data2 * (state.m_menu.mouseSensitivity + 5_i32) / 10_i32;
            state.g_game.mousey = ev.data3 * (state.m_menu.mouseSensitivity + 5_i32) / 10_i32;
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
    false
}
pub fn G_Ticker(state: &mut GameState, netcmds: &[ticcmd_t]) {
    let mut i: i32 = 0;
    let mut buf: i32 = 0;
    i = 0_i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize]
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
    i = 0_i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            state.g_game.players[i as usize].cmd = netcmds[i as usize];
            if state.g_game.demoplayback {
                G_ReadDemoTiccmd(state, i as usize);
            }
            if state.g_game.demorecording {
                G_WriteDemoTiccmd(state, i as usize);
            }
            if state.g_game.players[i as usize].cmd.forwardmove as i32 > TURBOTHRESHOLD {
                state.g_game.turbodetected[i as usize] = true;
            }
            if state.d_loop.gametic & 31_i32 == 0_i32
                && (state.d_loop.gametic >> 5_i32) % MAXPLAYERS == i
                && state.g_game.turbodetected[i as usize]
            {
                let player_name = PLAYER_NAMES[i as usize];
                state.g_game.players[state.g_game.consoleplayer as usize].message =
                    Some(format!("{} is turbo!", player_name));
                state.g_game.turbodetected[i as usize] = false;
            }
            if state.g_game.netgame
                && !state.g_game.netdemo
                && state.d_loop.gametic % state.d_loop.ticdup == 0
            {
                if state.d_loop.gametic > BACKUPTICS
                    && state.g_game.consistancy[i as usize][buf as usize] as i32
                        != state.g_game.players[i as usize].cmd.consistancy as i32
                {
                    I_Error(&format!(
                        "consistency failure ({} should be {})",
                        state.g_game.players[i as usize].cmd.consistancy as i32,
                        state.g_game.consistancy[i as usize][buf as usize] as i32,
                    ));
                }
                if let Some(mo_id) = state.g_game.players[i as usize].mo {
                    state.g_game.consistancy[i as usize][buf as usize] =
                        state.p_mobj.mo(mo_id).x as byte;
                } else {
                    state.g_game.consistancy[i as usize][buf as usize] =
                        state.m_random.rndindex as byte;
                }
            }
        }
        i += 1;
    }
    i = 0_i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize]
            && state.g_game.players[i as usize].cmd.buttons as i32 & BT_SPECIAL as i32 != 0
        {
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
                    state.g_game.savegameslot =
                        (state.g_game.players[i as usize].cmd.buttons as i32 & BTS_SAVEMASK as i32)
                            >> BTS_SAVESHIFT as i32;
                    state.g_game.gameaction = GameAction::ga_savegame;
                }
                _ => {}
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
pub fn G_InitPlayer(state: &mut GGameState, player: i32) {
    G_PlayerReborn(state, player);
}
pub fn G_PlayerFinishLevel(state: &mut GameState, player: i32) {
    let p = &mut state.g_game.players[player as usize];
    p.powers = [0; 6];
    p.cards = [false; 6];
    let mo_id = p.mo.unwrap();
    p.extralight = 0_i32;
    p.fixedcolormap = 0_i32;
    p.damagecount = 0_i32;
    p.bonuscount = 0_i32;
    state.p_mobj.mo_mut(mo_id).flags &= !(MF_SHADOW as i32);
}
pub fn G_PlayerReborn(state: &mut GGameState, player: i32) {
    let old = &state.players[player as usize];
    let (frags, killcount, itemcount, secretcount) =
        (old.frags, old.killcount, old.itemcount, old.secretcount);
    let p = &mut state.players[player as usize];
    *p = NEW_PLAYER;
    p.frags = frags;
    p.killcount = killcount;
    p.itemcount = itemcount;
    p.secretcount = secretcount;
    p.attackdown = true_0;
    p.usedown = p.attackdown;
    p.playerstate = PlayerState::PST_LIVE;
    p.health = deh_initial_health;
    p.pendingweapon = weapontype_t::wp_pistol;
    p.readyweapon = p.pendingweapon;
    p.weaponowned[weapontype_t::wp_fist as usize] = true;
    p.weaponowned[weapontype_t::wp_pistol as usize] = true;
    p.ammo[ammotype_t::am_clip as usize] = deh_initial_bullets;
    for i in 0..NUMAMMO as usize {
        p.maxammo[i] = maxammo[i];
    }
}
pub fn G_CheckSpot(state: &mut GameState, playernum: i32, mthing: &mapthing_t) -> bool {
    if state.g_game.players[playernum as usize].mo.is_none() {
        for i in 0..playernum {
            let other_mo = state
                .p_mobj
                .mo(state.g_game.players[i as usize].mo.unwrap());
            if other_mo.x == (mthing.x as i32) << FRACBITS
                && other_mo.y == (mthing.y as i32) << FRACBITS
            {
                return false;
            }
        }
        return true;
    }
    let x: fixed_t = ((mthing.x as i32) << FRACBITS) as fixed_t;
    let y: fixed_t = ((mthing.y as i32) << FRACBITS) as fixed_t;
    let player_mo_id = state.g_game.players[playernum as usize].mo.unwrap();
    if !P_CheckPosition(state, player_mo_id, x, y) {
        return false;
    }
    if state.g_game.bodyqueslot >= BODYQUESIZE {
        let old_id =
            state.g_game.bodyque[(state.g_game.bodyqueslot % BODYQUESIZE) as usize].unwrap();
        if state.p_mobj.is_live(old_id) {
            P_RemoveMobj(state, old_id);
        }
    }
    state.g_game.bodyque[(state.g_game.bodyqueslot % BODYQUESIZE) as usize] = Some(player_mo_id);
    state.g_game.bodyqueslot += 1;
    let ss = R_PointInSubsector(state, x, y);
    let xa: fixed_t;
    let ya: fixed_t;
    let an: i32 = (ANG45 >> ANGLETOFINESHIFT) * (mthing.angle as i32 / 45_i32);
    match an {
        4096 => {
            xa = finetangent[2048];
            ya = finetangent[0];
        }
        5120 => {
            xa = finetangent[3072];
            ya = finetangent[1024];
        }
        6144 => {
            xa = finesine[0];
            ya = finetangent[2048];
        }
        7168 => {
            xa = finesine[1024];
            ya = finetangent[3072];
        }
        0 | 1024 | 2048 | 3072 => {
            xa = finecosine[an as isize];
            ya = finesine[an as usize];
        }
        _ => {
            I_Error(&format!("G_CheckSpot: unexpected angle {}\n", an));
        }
    }
    let floorheight = state
        .p_setup
        .sector_mut(state.p_setup.subsectors[ss.0 as usize].sector)
        .floorheight;
    let mo = P_SpawnMobj(
        state,
        x + 20 as fixed_t * xa,
        y + 20 as fixed_t * ya,
        floorheight,
        MobjType::MT_TFOG,
    );
    if state.g_game.players[state.g_game.consoleplayer as usize].viewz != 1_i32 {
        S_StartSound(state, SoundOrigin::Mobj(mo), sfx_telept as i32);
    }
    true
}
pub fn G_DeathMatchSpawnPlayer(state: &mut GameState, playernum: i32) {
    let selections = state.p_setup.deathmatch_p as i32;
    if selections < 4_i32 {
        I_Error(&format!("Only {} deathmatch spots, 4 required", selections));
    }
    for _ in 0..20 {
        let i = (P_Random(&mut state.m_random) % selections) as usize;
        let dm_spot = state.p_setup.deathmatchstarts[i];
        if G_CheckSpot(state, playernum, &dm_spot) {
            state.p_setup.deathmatchstarts[i].type_0 = (playernum + 1_i32) as i16;
            let dm_spot = state.p_setup.deathmatchstarts[i];
            P_SpawnPlayer(state, dm_spot);
            return;
        }
    }
    let spot = state.p_setup.playerstarts[playernum as usize];
    P_SpawnPlayer(state, spot);
}
pub fn G_DoReborn(state: &mut GameState, playernum: i32) {
    if !state.g_game.netgame {
        state.g_game.gameaction = GameAction::ga_loadlevel;
    } else {
        let player_mo_id = state.g_game.players[playernum as usize].mo.unwrap();
        state.p_mobj.mo_mut(player_mo_id).player = None;
        if state.g_game.deathmatch != 0 {
            G_DeathMatchSpawnPlayer(state, playernum);
            return;
        }
        let spot = state.p_setup.playerstarts[playernum as usize];
        if G_CheckSpot(state, playernum, &spot) {
            let spot = state.p_setup.playerstarts[playernum as usize];
            P_SpawnPlayer(state, spot);
            return;
        }
        for i in 0..MAXPLAYERS as usize {
            let spot = state.p_setup.playerstarts[i];
            if G_CheckSpot(state, playernum, &spot) {
                state.p_setup.playerstarts[i].type_0 = (playernum + 1_i32) as i16;
                let spot = state.p_setup.playerstarts[i];
                P_SpawnPlayer(state, spot);
                state.p_setup.playerstarts[i].type_0 = (i as i32 + 1_i32) as i16;
                return;
            }
        }
        let spot = state.p_setup.playerstarts[playernum as usize];
        P_SpawnPlayer(state, spot);
    };
}
pub fn G_ScreenShot(state: &mut GameState) {
    state.g_game.gameaction = GameAction::ga_screenshot;
}
pub static pars: [[i32; 10]; 4] = [
    [0_i32; 10],
    [
        0_i32, 30_i32, 75_i32, 120_i32, 90_i32, 165_i32, 180_i32, 180_i32, 30_i32, 165_i32,
    ],
    [
        0_i32, 90_i32, 90_i32, 90_i32, 120_i32, 90_i32, 360_i32, 240_i32, 30_i32, 170_i32,
    ],
    [
        0_i32, 90_i32, 45_i32, 90_i32, 150_i32, 90_i32, 90_i32, 165_i32, 30_i32, 135_i32,
    ],
];
pub static cpars: [i32; 32] = [
    30_i32, 90_i32, 120_i32, 120_i32, 90_i32, 150_i32, 120_i32, 120_i32, 270_i32, 90_i32, 210_i32,
    150_i32, 150_i32, 150_i32, 210_i32, 150_i32, 420_i32, 150_i32, 210_i32, 150_i32, 240_i32,
    150_i32, 180_i32, 150_i32, 150_i32, 300_i32, 330_i32, 420_i32, 300_i32, 180_i32, 120_i32,
    30_i32,
];
pub fn G_ExitLevel(state: &mut GameState) {
    state.g_game.secretexit = false;
    state.g_game.gameaction = GameAction::ga_completed;
}
pub fn G_SecretExitLevel(state: &mut GameState) {
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as u32
        && W_CheckNumForName(&mut state.w_wad, "map31") < 0_i32
    {
        state.g_game.secretexit = false;
    } else {
        state.g_game.secretexit = true;
    }
    state.g_game.gameaction = GameAction::ga_completed;
}
pub fn G_DoCompleted(state: &mut GameState) {
    let mut i: i32 = 0;
    state.g_game.gameaction = GameAction::ga_nothing;
    i = 0_i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            G_PlayerFinishLevel(state, i);
        }
        i += 1;
    }
    if state.am_map.automapactive {
        AM_Stop(state);
    }
    if state.doomstat.gamemode as u32 != GameMode_t::commercial as u32 {
        if state.doomstat.gameversion == GameVersion::chex {
            if state.g_game.gamemap == 5_i32 {
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
                    i = 0_i32;
                    while i < MAXPLAYERS {
                        state.g_game.players[i as usize].didsecret = true;
                        i += 1;
                    }
                }
                _ => {}
            }
        }
    }
    if state.g_game.gamemap == 8_i32
        && state.doomstat.gamemode as u32 != GameMode_t::commercial as u32
    {
        state.g_game.gameaction = GameAction::ga_victory;
        return;
    }
    if state.g_game.gamemap == 9_i32
        && state.doomstat.gamemode as u32 != GameMode_t::commercial as u32
    {
        i = 0_i32;
        while i < MAXPLAYERS {
            state.g_game.players[i as usize].didsecret = true;
            i += 1;
        }
    }
    state.g_game.wminfo.didsecret =
        state.g_game.players[state.g_game.consoleplayer as usize].didsecret;
    state.g_game.wminfo.epsd = state.g_game.gameepisode - 1_i32;
    state.g_game.wminfo.last = state.g_game.gamemap - 1_i32;
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as u32 {
        if state.g_game.secretexit {
            match state.g_game.gamemap {
                15 => {
                    state.g_game.wminfo.next = 30_i32;
                }
                31 => {
                    state.g_game.wminfo.next = 31_i32;
                }
                _ => {}
            }
        } else {
            match state.g_game.gamemap {
                31 | 32 => {
                    state.g_game.wminfo.next = 15_i32;
                }
                _ => {
                    state.g_game.wminfo.next = state.g_game.gamemap;
                }
            }
        }
    } else if state.g_game.secretexit {
        state.g_game.wminfo.next = 8_i32;
    } else if state.g_game.gamemap == 9_i32 {
        match state.g_game.gameepisode {
            1 => {
                state.g_game.wminfo.next = 3_i32;
            }
            2 => {
                state.g_game.wminfo.next = 5_i32;
            }
            3 => {
                state.g_game.wminfo.next = 6_i32;
            }
            4 => {
                state.g_game.wminfo.next = 2_i32;
            }
            _ => {}
        }
    } else {
        state.g_game.wminfo.next = state.g_game.gamemap;
    }
    state.g_game.wminfo.maxkills = state.g_game.totalkills;
    state.g_game.wminfo.maxitems = state.g_game.totalitems;
    state.g_game.wminfo.maxsecret = state.g_game.totalsecret;
    state.g_game.wminfo.maxfrags = 0_i32;
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as u32 {
        state.g_game.wminfo.partime = TICRATE * cpars[(state.g_game.gamemap - 1_i32) as usize];
    } else if state.g_game.gameepisode < 4_i32 {
        state.g_game.wminfo.partime =
            TICRATE * pars[state.g_game.gameepisode as usize][state.g_game.gamemap as usize];
    } else {
        state.g_game.wminfo.partime = TICRATE * cpars[state.g_game.gamemap as usize];
    }
    state.g_game.wminfo.pnum = state.g_game.consoleplayer;
    i = 0_i32;
    while i < MAXPLAYERS {
        state.g_game.wminfo.plyr[i as usize].in_0 = state.g_game.playeringame[i as usize];
        state.g_game.wminfo.plyr[i as usize].skills = state.g_game.players[i as usize].killcount;
        state.g_game.wminfo.plyr[i as usize].sitems = state.g_game.players[i as usize].itemcount;
        state.g_game.wminfo.plyr[i as usize].ssecret = state.g_game.players[i as usize].secretcount;
        state.g_game.wminfo.plyr[i as usize].stime = state.p_tick.leveltime;
        state.g_game.wminfo.plyr[i as usize].frags = state.g_game.players[i as usize].frags;
        i += 1;
    }
    state.g_game.gamestate = GameScreenState::GS_INTERMISSION;
    state.g_game.viewactive = false;
    state.am_map.automapactive = false;
    StatCopy(state);
    WI_Start(state);
}
pub fn G_WorldDone(state: &mut GameState) {
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
pub fn G_DoWorldDone(state: &mut GameState) {
    state.g_game.gamestate = GameScreenState::GS_LEVEL;
    state.g_game.gamemap = state.g_game.wminfo.next + 1_i32;
    G_DoLoadLevel(state);
    state.g_game.gameaction = GameAction::ga_nothing;
    state.g_game.viewactive = true;
}
pub fn G_LoadGame(state: &mut GameState, name: &str) {
    state.g_game.savename = name.to_string();
    state.g_game.gameaction = GameAction::ga_loadgame;
}
pub fn G_DoLoadGame(state: &mut GameState) {
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
pub fn G_SaveGame(state: &mut GameState, mut slot: i32, description: &str) {
    state.g_game.savegameslot = slot;
    state.g_game.savedescription = description.to_string();
    state.g_game.sendsave = true;
}
pub fn G_DoSaveGame(state: &mut GameState) {
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
pub fn G_DoNewGame(state: &mut GameState) {
    state.g_game.demoplayback = false;
    state.g_game.netdemo = false;
    state.g_game.netgame = false;
    state.g_game.deathmatch = false_0;
    state.g_game.playeringame[3] = false;
    state.g_game.playeringame[2] = state.g_game.playeringame[3];
    state.g_game.playeringame[1] = state.g_game.playeringame[2];
    state.d_main.respawnparm = false;
    state.d_main.fastparm = false;
    state.d_main.nomonsters = false;
    state.g_game.consoleplayer = 0_i32;
    let (d_skill, d_episode, d_map) = (
        state.g_game.d_skill,
        state.g_game.d_episode,
        state.g_game.d_map,
    );
    G_InitNew(state, d_skill, d_episode, d_map);
    state.g_game.gameaction = GameAction::ga_nothing;
}
pub fn G_InitNew(state: &mut GameState, mut skill: SkillType, mut episode: i32, mut map: i32) {
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
        if episode == 0_i32 {
            episode = 4_i32;
        }
    } else {
        if episode < 1_i32 {
            episode = 1_i32;
        }
        if episode > 3_i32 {
            episode = 3_i32;
        }
    }
    if episode > 1_i32 && state.doomstat.gamemode as u32 == GameMode_t::shareware as u32 {
        episode = 1_i32;
    }
    if map < 1_i32 {
        map = 1_i32;
    }
    if map > 9_i32 && state.doomstat.gamemode as u32 != GameMode_t::commercial as u32 {
        map = 9_i32;
    }
    M_ClearRandom(&mut state.m_random);
    if skill == SkillType::sk_nightmare || state.d_main.respawnparm {
        state.g_game.respawnmonsters = true;
    } else {
        state.g_game.respawnmonsters = false;
    }
    if state.d_main.fastparm
        || skill == SkillType::sk_nightmare && state.g_game.gameskill != SkillType::sk_nightmare
    {
        i = StateNum::S_SARG_RUN1 as i32;
        while i <= StateNum::S_SARG_PAIN2 as i32 {
            state.info.states[i as usize].tics >>= 1_i32;
            i += 1;
        }
        state.info.mobjinfo[MobjType::MT_BRUISERSHOT as usize].speed = 20_i32 * FRACUNIT;
        state.info.mobjinfo[MobjType::MT_HEADSHOT as usize].speed = 20_i32 * FRACUNIT;
        state.info.mobjinfo[MobjType::MT_TROOPSHOT as usize].speed = 20_i32 * FRACUNIT;
    } else if skill != SkillType::sk_nightmare && state.g_game.gameskill == SkillType::sk_nightmare
    {
        i = StateNum::S_SARG_RUN1 as i32;
        while i <= StateNum::S_SARG_PAIN2 as i32 {
            state.info.states[i as usize].tics <<= 1_i32;
            i += 1;
        }
        state.info.mobjinfo[MobjType::MT_BRUISERSHOT as usize].speed = 15_i32 * FRACUNIT;
        state.info.mobjinfo[MobjType::MT_HEADSHOT as usize].speed = 10_i32 * FRACUNIT;
        state.info.mobjinfo[MobjType::MT_TROOPSHOT as usize].speed = 10_i32 * FRACUNIT;
    }
    i = 0_i32;
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
        if state.g_game.gamemap < 12_i32 {
            skytexturename = "SKY1";
        } else if state.g_game.gamemap < 21_i32 {
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
pub fn G_ReadDemoTiccmd(state: &mut GameState, player_num: usize) {
    if state.g_game.demobuffer[state.g_game.demo_p] as i32 == DEMOMARKER {
        G_CheckDemoStatus(state);
        return;
    }
    let forwardmove = state.g_game.demo_read_byte() as i8;
    let sidemove = state.g_game.demo_read_byte() as i8;
    let new_angleturn;
    if state.g_game.longtics {
        let lo = state.g_game.demo_read_byte() as i16;
        let hi = state.g_game.demo_read_byte();
        new_angleturn = (lo as i32 | (hi as i32) << 8_i32) as i16;
    } else {
        let hi = state.g_game.demo_read_byte();
        new_angleturn = ((hi as i32) << 8_i32) as i16;
    }
    let buttons = state.g_game.demo_read_byte() as byte;
    let cmd = &mut state.g_game.players[player_num].cmd;
    cmd.forwardmove = forwardmove;
    cmd.sidemove = sidemove;
    cmd.angleturn = new_angleturn;
    cmd.buttons = buttons;
}
fn IncreaseDemoBuffer(state: &mut GameState) {
    let new_length = state.g_game.demoend * 2_usize;
    state.g_game.demobuffer.resize(new_length, 0);
    state.g_game.demoend = new_length;
}
pub fn G_WriteDemoTiccmd(state: &mut GameState, player_num: usize) {
    if state.g_game.gamekeydown[state.m_controls.key_demo_quit as usize] {
        G_CheckDemoStatus(state);
    }
    let demo_start = state.g_game.demo_p;
    let cmd = state.g_game.players[player_num].cmd;
    state.g_game.demo_write_byte(cmd.forwardmove as byte);
    state.g_game.demo_write_byte(cmd.sidemove as byte);
    if state.g_game.longtics {
        state
            .g_game
            .demo_write_byte((cmd.angleturn as i32 & 0xff_i32) as byte);
        state
            .g_game
            .demo_write_byte((cmd.angleturn as i32 >> 8_i32 & 0xff_i32) as byte);
    } else {
        state
            .g_game
            .demo_write_byte((cmd.angleturn as i32 >> 8_i32) as byte);
    }
    state.g_game.demo_write_byte(cmd.buttons);
    state.g_game.demo_p = demo_start;
    if state.g_game.demo_p > state.g_game.demoend.saturating_sub(16) {
        if state.g_game.vanilla_demo_limit != 0 {
            G_CheckDemoStatus(state);
            return;
        } else {
            IncreaseDemoBuffer(state);
        }
    }
    G_ReadDemoTiccmd(state, player_num);
}
pub fn G_RecordDemo(state: &mut GameState, name: &str) {
    let mut i: i32 = 0;
    let mut maxsize: i32 = 0;
    state.g_game.usergame = false;
    state.g_game.demoname = format!("{}.lmp", name);
    maxsize = 0x20000_i32;
    i = M_CheckParmWithArgs(state, "-maxdemo", 1_i32);
    if i != 0 {
        maxsize = M_ArgvAtoi(&state.m_argv.myargv[(i + 1_i32) as usize]) * 1024_i32;
    }
    state.g_game.demobuffer = vec![0u8; maxsize as usize];
    state.g_game.demoend = maxsize as usize;
    state.g_game.demorecording = true;
}
pub fn G_VanillaVersionCode(state: &mut DoomstatState) -> i32 {
    match state.gameversion as u32 {
        0 => {
            I_Error("Doom 1.2 does not have a version code!");
        }
        1 => {}
        2 => return 107_i32,
        3 => return 108_i32,
        4 | _ => return 109_i32,
    }
    106_i32
}
pub fn G_BeginRecording(state: &mut GameState) {
    let mut i: i32 = 0;
    state.g_game.longtics = M_CheckParm(state, "-longtics") != 0_i32;
    state.g_game.lowres_turn = !state.g_game.longtics;
    state.g_game.demo_p = 0;
    if state.g_game.longtics {
        state.g_game.demo_write_byte(DOOM_191_VERSION as byte);
    } else {
        let version = G_VanillaVersionCode(&mut state.doomstat) as byte;
        state.g_game.demo_write_byte(version);
    }
    state.g_game.demo_write_byte(state.g_game.gameskill as byte);
    state
        .g_game
        .demo_write_byte(state.g_game.gameepisode as byte);
    state.g_game.demo_write_byte(state.g_game.gamemap as byte);
    state
        .g_game
        .demo_write_byte(state.g_game.deathmatch as byte);
    state
        .g_game
        .demo_write_byte(state.d_main.respawnparm as byte);
    state.g_game.demo_write_byte(state.d_main.fastparm as byte);
    state
        .g_game
        .demo_write_byte(state.d_main.nomonsters as byte);
    state
        .g_game
        .demo_write_byte(state.g_game.consoleplayer as byte);
    i = 0_i32;
    while i < MAXPLAYERS {
        let b = state.g_game.playeringame[i as usize] as byte;
        state.g_game.demo_write_byte(b);
        i += 1;
    }
}
pub fn G_DeferedPlayDemo(state: &mut GameState, name: FixedCStr<8>) {
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
    if (0_i32..=4_i32).contains(&version) {
        "v1.0/v1.1/v1.2".to_string()
    } else {
        format!("{}.{} (unknown)", version / 100_i32, version % 100_i32)
    }
}
pub fn G_DoPlayDemo(state: &mut GameState) {
    let mut skill: SkillType = SkillType::sk_baby;
    let mut i: i32 = 0;
    let mut episode: i32 = 0;
    let mut map: i32 = 0;
    let mut demoversion: i32 = 0;
    state.g_game.gameaction = GameAction::ga_nothing;
    let demo_lumpname = state.g_game.defdemoname.as_str().into_owned();
    let demo_lumpnum = W_GetNumForName(&mut state.w_wad, &demo_lumpname);
    let demo_lumplen = W_LumpLength(&mut state.w_wad, demo_lumpnum as u32) as usize;
    state.g_game.demobuffer = W_LumpBytes(state, demo_lumpnum)[..demo_lumplen].to_vec();
    state.g_game.demo_p = 0;
    demoversion = state.g_game.demo_read_byte() as i32;
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
    skill = skill_from_raw(state.g_game.demo_read_byte() as i32);
    episode = state.g_game.demo_read_byte() as i32;
    map = state.g_game.demo_read_byte() as i32;
    state.g_game.deathmatch = state.g_game.demo_read_byte() as i32;
    state.d_main.respawnparm = state.g_game.demo_read_byte() != 0;
    state.d_main.fastparm = state.g_game.demo_read_byte() != 0;
    state.d_main.nomonsters = state.g_game.demo_read_byte() != 0;
    state.g_game.consoleplayer = state.g_game.demo_read_byte() as i32;
    i = 0_i32;
    while i < MAXPLAYERS {
        state.g_game.playeringame[i as usize] = state.g_game.demo_read_byte() != 0;
        i += 1;
    }
    if state.g_game.playeringame[1]
        || M_CheckParm(state, "-solo-net") > 0_i32
        || M_CheckParm(state, "-netdemo") > 0_i32
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
pub fn G_TimeDemo(state: &mut GameState, name: FixedCStr<8>) {
    state.g_game.nodrawers = M_CheckParm(state, "-nodraw") != 0;
    state.g_game.timingdemo = true;
    state.d_loop.singletics = true;
    state.g_game.defdemoname = name;
    state.g_game.gameaction = GameAction::ga_playdemo;
}
pub fn G_CheckDemoStatus(state: &mut GameState) -> bool {
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
            state.d_loop.gametic, realtics, fps as f64,
        ));
    }
    if state.g_game.demoplayback {
        W_ReleaseLumpName(&mut state.w_wad, &state.g_game.defdemoname.as_str());
        state.g_game.demoplayback = false;
        state.g_game.netdemo = false;
        state.g_game.netgame = false;
        state.g_game.deathmatch = false_0;
        state.g_game.playeringame[3] = false;
        state.g_game.playeringame[2] = state.g_game.playeringame[3];
        state.g_game.playeringame[1] = state.g_game.playeringame[2];
        state.d_main.respawnparm = false;
        state.d_main.fastparm = false;
        state.d_main.nomonsters = false;
        state.g_game.consoleplayer = 0_i32;
        if state.g_game.singledemo {
            I_Quit(state);
        } else {
            D_AdvanceDemo(state);
        }
        return true;
    }
    if state.g_game.demorecording {
        state.g_game.demo_write_byte(DEMOMARKER as byte);
        let demo_len = state.g_game.demo_p;
        M_WriteFile(&state.g_game.demoname, &state.g_game.demobuffer[..demo_len]);
        state.g_game.demobuffer = Vec::new();
        state.g_game.demorecording = false;
        I_Error(&format!("Demo {} recorded", state.g_game.demoname));
    }
    false
}
pub const MAX_MOUSE_BUTTONS: i32 = 8;
