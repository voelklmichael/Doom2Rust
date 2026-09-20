use crate::am_map::am_responder;
use crate::am_map::am_stop;
use crate::am_map::am_ticker;
use crate::d_event::EvType;
use crate::d_event::Event;
use crate::d_event::GameAction;
use crate::d_event::GameScreenState;
use crate::d_loop::DLoopState;
use crate::d_loop::BACKUPTICS;
use crate::d_main::advance_demo;
use crate::d_main::page_ticker;
use crate::d_mode::GameMission;
use crate::d_mode::GameMode;
use crate::d_mode::GameVersion;
use crate::d_mode::{skill_from_raw, SkillType};
use crate::d_player::CheatFlags;
use crate::d_player::PerPlayer;
use crate::d_player::PowerType;
use crate::d_player::WeaponType;
use crate::d_player::{AmmoType, NUMAMMO};
use crate::d_player::{Player, PlayerId, PlayerState};
use crate::d_ticcmd::TicCmd;
use crate::d_ticcmd::{
    BTS_PAUSE, BTS_SAVEGAME, BTS_SAVEMASK, BTS_SAVESHIFT, BT_ATTACK, BT_CHANGE, BT_SPECIAL,
    BT_SPECIALMASK, BT_USE, BT_WEAPONSHIFT,
};
use crate::doomdef::MAXPLAYERS;
use crate::doomdef::TICRATE;
use crate::doomstat::DoomstatState;
use crate::f_finale::f_responder;
use crate::f_finale::f_start_finale;
use crate::f_finale::f_ticker;
use crate::filesystem::read_file;
use crate::fixed_cstr::FixedCStr;
use crate::game_state::Game;
use crate::game_state::GameState;
use crate::hu_stuff::dequeue_chat_char;
use crate::hu_stuff::hu_responder;
use crate::hu_stuff::hu_ticker;
use crate::hu_stuff::PLAYER_NAMES;
use crate::i_system::error;
use crate::i_system::i_quit;
use crate::i_timer::get_time;
use crate::m_argv::MArgvState;
use crate::m_argv::{argv_atoi, check_parm_with_args, parm_exists};
use crate::m_controls::MControlsState;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::m_menu::start_control_panel;
use crate::m_random::clear_random;
use crate::m_random::p_random;
use crate::p_mobj::MobjFlags;
use crate::p_mobj::PMobjState;
use crate::w_wad::WWadState;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::p_inter::MAXAMMO;
use crate::p_map::check_position;
use crate::p_mobj::remove_mobj;
use crate::p_mobj::spawn_mobj;
use crate::p_mobj::spawn_player;
use crate::p_mobj::MapThing;
use crate::p_mobj::MobjId;
use crate::p_mobj::MobjType;
use crate::p_mobj::PspDef;
use crate::p_mobj::StateNum;

use crate::p_saveg::archive_players;
use crate::p_saveg::archive_specials;
use crate::p_saveg::archive_thinkers;
use crate::p_saveg::archive_world;
use crate::p_saveg::read_save_game_eof;
use crate::p_saveg::read_save_game_header;
use crate::p_saveg::report_save_game_read_error;
use crate::p_saveg::save_game_file;
use crate::p_saveg::temp_save_game_file;
use crate::p_saveg::un_archive_players;
use crate::p_saveg::un_archive_specials;
use crate::p_saveg::un_archive_thinkers;
use crate::p_saveg::un_archive_world;
use crate::p_saveg::write_save_game_eof;
use crate::p_saveg::write_save_game_header;
use crate::p_setup::setup_level;

use crate::p_tick::p_ticker;
use crate::r_data::flat_num_for_name;
use crate::r_data::texture_num_for_name;
use crate::r_draw::fill_back_screen;
use crate::r_main::execute_set_view_size;
use crate::r_main::point_in_subsector;
use crate::s_sound::pause_sound;
use crate::s_sound::resume_sound;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::st_stuff::st_responder;
use crate::st_stuff::st_ticker;
use crate::statdump::stat_copy;

use crate::tables::ANG45;
use crate::tables::ANGLETOFINESHIFT;
use crate::tables::FINECOSINE;
use crate::tables::FINESINE;
use crate::tables::FINETANGENT;
use crate::v_video::v_screen_shot;
use crate::w_wad::{
    check_num_for_name, get_num_for_name, lump_bytes, lump_length, release_lump_name,
};
use crate::wi_stuff::wi_end;
use crate::wi_stuff::wi_start;
use crate::wi_stuff::wi_ticker;
use crate::wi_stuff::{WbPlayerStruct, WbStartStruct};

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
    pub playeringame: PerPlayer<bool>,
    pub players: PerPlayer<Player>,
    pub turbodetected: [bool; 4],
    pub consoleplayer: PlayerId,
    pub displayplayer: PlayerId,
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
    pub demobuffer: Vec<u8>,
    pub demo_p: usize,
    pub demoend: usize,
    pub singledemo: bool,
    pub precache: bool,
    pub testcontrols: bool,
    pub testcontrols_mousespeed: i32,
    pub wminfo: WbStartStruct,
    pub consistancy: [[u8; 128]; 4],
    pub forwardmove: [Fixed; 2],
    pub sidemove: [Fixed; 2],
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

const NEW_PLAYER: Player = Player {
    mo: None,
    playerstate: PlayerState::Live,
    cmd: TicCmd {
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
    readyweapon: WeaponType::Fist,
    pendingweapon: WeaponType::Fist,
    weaponowned: [false; 9],
    ammo: [0; 4],
    maxammo: [0; 4],
    attackdown: false,
    usedown: false,
    cheats: CheatFlags::empty(),
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
    psprites: [PspDef {
        state: None,
        tics: 0,
        sx: 0,
        sy: 0,
    }; 2],
    didsecret: false,
};

impl Default for GGameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GGameState {
    pub const fn new() -> Self {
        Self {
            oldgamestate: GameScreenState::Level,
            gameaction: GameAction::Nothing,
            gamestate: GameScreenState::Level,
            gameskill: SkillType::Baby,
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
            playeringame: PerPlayer([false; 4]),
            players: PerPlayer([NEW_PLAYER, NEW_PLAYER, NEW_PLAYER, NEW_PLAYER]),
            turbodetected: [false; 4],
            consoleplayer: PlayerId(0),
            displayplayer: PlayerId(0),
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
            wminfo: WbStartStruct {
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
                plyr: [WbPlayerStruct {
                    intercept: false,
                    skills: 0,
                    sitems: 0,
                    ssecret: 0,
                    stime: 0,
                    frags: [0; 4],
                    score: 0,
                }; 4],
            },
            consistancy: [[0; 128]; 4],
            forwardmove: [0x19, 0x32],
            sidemove: [0x18, 0x28],
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
            d_skill: SkillType::Baby,
            d_episode: 0,
            d_map: 0,
            defdemoname: FixedCStr::from_array([0; 8]),
            g_build_ticcmd_carry: 0,
        }
    }

    pub fn player_mut(&mut self, id: PlayerId) -> &mut Player {
        &mut self.players[id]
    }

    fn demo_read_byte(&mut self) -> u8 {
        let b = self.demobuffer[self.demo_p];
        self.demo_p += 1;
        b
    }

    fn demo_write_byte(&mut self, b: u8) {
        self.demobuffer[self.demo_p] = b;
        self.demo_p += 1;
    }
}

#[derive(Copy, Clone)]
pub struct WeaponOrder {
    pub weapon: WeaponType,
    pub weapon_num: WeaponType,
}
pub const DEH_DEFAULT_INITIAL_HEALTH: i32 = 100;
pub const DEH_DEFAULT_INITIAL_BULLETS: i32 = 50;
pub const DEH_INITIAL_HEALTH: i32 = DEH_DEFAULT_INITIAL_HEALTH;
pub const DEH_INITIAL_BULLETS: i32 = DEH_DEFAULT_INITIAL_BULLETS;
pub const DOOM_191_VERSION: i32 = 111;
pub const SAVEGAMESIZE: i32 = 0x2c000;
pub const TURBOTHRESHOLD: i32 = 0x32;
pub static ANGLETURN: [Fixed; 3] = [640, 1280, 320];
static WEAPON_ORDER_TABLE: [WeaponOrder; 9] = [
    WeaponOrder {
        weapon: WeaponType::Fist,
        weapon_num: WeaponType::Fist,
    },
    WeaponOrder {
        weapon: WeaponType::Chainsaw,
        weapon_num: WeaponType::Fist,
    },
    WeaponOrder {
        weapon: WeaponType::Pistol,
        weapon_num: WeaponType::Pistol,
    },
    WeaponOrder {
        weapon: WeaponType::Shotgun,
        weapon_num: WeaponType::Shotgun,
    },
    WeaponOrder {
        weapon: WeaponType::Supershotgun,
        weapon_num: WeaponType::Shotgun,
    },
    WeaponOrder {
        weapon: WeaponType::Chaingun,
        weapon_num: WeaponType::Chaingun,
    },
    WeaponOrder {
        weapon: WeaponType::Missile,
        weapon_num: WeaponType::Missile,
    },
    WeaponOrder {
        weapon: WeaponType::Plasma,
        weapon_num: WeaponType::Plasma,
    },
    WeaponOrder {
        weapon: WeaponType::Bfg,
        weapon_num: WeaponType::Bfg,
    },
];
pub const SLOWTURNTICS: i32 = 6;
pub const NUMKEYS: i32 = 256;
pub const MAX_JOY_BUTTONS: i32 = 20;
pub const BODYQUESIZE: i32 = 32;
fn weapon_selectable(doomstat: &DoomstatState, g_game: &GGameState, weapon: WeaponType) -> bool {
    if weapon as u32 == WeaponType::Supershotgun as u32
        && (if doomstat.gamemission as u32 == GameMission::PackChex as u32 {
            GameMission::Doom as u32
        } else if doomstat.gamemission as u32 == GameMission::PackHacx as u32 {
            GameMission::Doom2 as u32
        } else {
            doomstat.gamemission as u32
        }) == GameMission::Doom as u32
    {
        return false;
    }
    if (weapon as u32 == WeaponType::Plasma as u32 || weapon as u32 == WeaponType::Bfg as u32)
        && doomstat.gamemission as u32 == GameMission::Doom as u32
        && doomstat.gamemode as u32 == GameMode::Shareware as u32
    {
        return false;
    }
    if !g_game.players[g_game.consoleplayer].weaponowned[weapon as usize] {
        return false;
    }
    if weapon as u32 == WeaponType::Fist as u32
        && g_game.players[g_game.consoleplayer].weaponowned[WeaponType::Chainsaw as usize]
        && g_game.players[g_game.consoleplayer].powers[PowerType::Strength as usize] == 0
    {
        return false;
    }
    true
}
fn g_next_weapon(doomstat: &DoomstatState, g_game: &GGameState, direction: i32) -> i32 {
    let mut i: i32;
    let weapon: WeaponType = if g_game.players[g_game.consoleplayer].pendingweapon as u32
        == WeaponType::Nochange as u32
    {
        g_game.players[g_game.consoleplayer].readyweapon
    } else {
        g_game.players[g_game.consoleplayer].pendingweapon
    };
    i = 0;
    while (i as usize)
        < ::core::mem::size_of::<[WeaponOrder; 9]>()
            .wrapping_div(::core::mem::size_of::<WeaponOrder>())
    {
        if WEAPON_ORDER_TABLE[i as usize].weapon as u32 == weapon as u32 {
            break;
        }
        i += 1;
    }
    let start_i: i32 = i;
    loop {
        i += direction;
        i = (i as usize)
            .wrapping_add(
                ::core::mem::size_of::<[WeaponOrder; 9]>()
                    .wrapping_div(::core::mem::size_of::<WeaponOrder>()),
            )
            .wrapping_rem(
                ::core::mem::size_of::<[WeaponOrder; 9]>()
                    .wrapping_div(::core::mem::size_of::<WeaponOrder>()),
            ) as i32;
        if !(i != start_i
            && !weapon_selectable(doomstat, g_game, WEAPON_ORDER_TABLE[i as usize].weapon))
        {
            break;
        }
    }
    WEAPON_ORDER_TABLE[i as usize].weapon_num as i32
}
pub fn g_build_ticcmd(state: &mut GameState, cmd: &mut TicCmd, maketic: i32) {
    let mut i: i32;

    let bstrafe: bool;

    let mut forward: i32;
    let mut side: i32;
    *cmd = TicCmd {
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
    cmd.consistancy = state.game.g_game.consistancy[state.game.g_game.consoleplayer.slot()]
        [(maketic % BACKUPTICS) as usize];
    let strafe: bool = state.game.g_game.gamekeydown[state.game.m_controls.key_strafe as usize]
        || state.game.g_game.mousearray[(state.game.m_controls.mousebstrafe + 1) as usize]
        || state.game.g_game.joyarray[(state.game.m_controls.joybstrafe + 1) as usize];
    let speed: i32 = (state.game.m_controls.key_speed >= NUMKEYS
        || state.game.m_controls.joybspeed >= MAX_JOY_BUTTONS
        || state.game.g_game.gamekeydown[state.game.m_controls.key_speed as usize]
        || state.game.g_game.joyarray[(state.game.m_controls.joybspeed + 1) as usize])
        as i32;
    side = 0;
    forward = side;
    if state.game.g_game.joyxmove != 0
        || state.game.g_game.gamekeydown[state.game.m_controls.key_right as usize]
        || state.game.g_game.gamekeydown[state.game.m_controls.key_left as usize]
    {
        state.game.g_game.turnheld += state.game.d_loop.ticdup;
    } else {
        state.game.g_game.turnheld = 0;
    }
    let tspeed: i32 = if state.game.g_game.turnheld < SLOWTURNTICS {
        2
    } else {
        speed
    };
    if strafe {
        if state.game.g_game.gamekeydown[state.game.m_controls.key_right as usize] {
            side += state.game.g_game.sidemove[speed as usize];
        }
        if state.game.g_game.gamekeydown[state.game.m_controls.key_left as usize] {
            side -= state.game.g_game.sidemove[speed as usize];
        }
        if state.game.g_game.joyxmove > 0 {
            side += state.game.g_game.sidemove[speed as usize];
        }
        if state.game.g_game.joyxmove < 0 {
            side -= state.game.g_game.sidemove[speed as usize];
        }
    } else {
        if state.game.g_game.gamekeydown[state.game.m_controls.key_right as usize] {
            cmd.angleturn = (cmd.angleturn as i32 - ANGLETURN[tspeed as usize]) as i16;
        }
        if state.game.g_game.gamekeydown[state.game.m_controls.key_left as usize] {
            cmd.angleturn = (cmd.angleturn as i32 + ANGLETURN[tspeed as usize]) as i16;
        }
        if state.game.g_game.joyxmove > 0 {
            cmd.angleturn = (cmd.angleturn as i32 - ANGLETURN[tspeed as usize]) as i16;
        }
        if state.game.g_game.joyxmove < 0 {
            cmd.angleturn = (cmd.angleturn as i32 + ANGLETURN[tspeed as usize]) as i16;
        }
    }
    if state.game.g_game.gamekeydown[state.game.m_controls.key_up as usize] {
        forward += state.game.g_game.forwardmove[speed as usize];
    }
    if state.game.g_game.gamekeydown[state.game.m_controls.key_down as usize] {
        forward -= state.game.g_game.forwardmove[speed as usize];
    }
    if state.game.g_game.joyymove < 0 {
        forward += state.game.g_game.forwardmove[speed as usize];
    }
    if state.game.g_game.joyymove > 0 {
        forward -= state.game.g_game.forwardmove[speed as usize];
    }
    if state.game.g_game.gamekeydown[state.game.m_controls.key_strafeleft as usize]
        || state.game.g_game.joyarray[(state.game.m_controls.joybstrafeleft + 1) as usize]
        || state.game.g_game.mousearray[(state.game.m_controls.mousebstrafeleft + 1) as usize]
        || state.game.g_game.joystrafemove < 0
    {
        side -= state.game.g_game.sidemove[speed as usize];
    }
    if state.game.g_game.gamekeydown[state.game.m_controls.key_straferight as usize]
        || state.game.g_game.joyarray[(state.game.m_controls.joybstraferight + 1) as usize]
        || state.game.g_game.mousearray[(state.game.m_controls.mousebstraferight + 1) as usize]
        || state.game.g_game.joystrafemove > 0
    {
        side += state.game.g_game.sidemove[speed as usize];
    }
    cmd.chatchar = dequeue_chat_char(&mut state.ui.hu_stuff);
    if state.game.g_game.gamekeydown[state.game.m_controls.key_fire as usize]
        || state.game.g_game.mousearray[(state.game.m_controls.mousebfire + 1) as usize]
        || state.game.g_game.joyarray[(state.game.m_controls.joybfire + 1) as usize]
    {
        cmd.buttons = (cmd.buttons as i32 | BT_ATTACK) as u8;
    }
    if state.game.g_game.gamekeydown[state.game.m_controls.key_use as usize]
        || state.game.g_game.joyarray[(state.game.m_controls.joybuse + 1) as usize]
        || state.game.g_game.mousearray[(state.game.m_controls.mousebuse + 1) as usize]
    {
        cmd.buttons = (cmd.buttons as i32 | BT_USE) as u8;
        state.game.g_game.dclicks = 0;
    }
    if state.game.g_game.gamestate == GameScreenState::Level && state.game.g_game.next_weapon != 0 {
        let next_weapon = state.game.g_game.next_weapon;
        i = g_next_weapon(&state.game.doomstat, &state.game.g_game, next_weapon);
        cmd.buttons = (cmd.buttons as i32 | BT_CHANGE) as u8;
        cmd.buttons = (cmd.buttons as i32 | i << BT_WEAPONSHIFT) as u8;
    } else {
        let weapon_keys = state.game.m_controls.weapon_keys();
        i = 0;
        while (i as usize) < weapon_keys.len() {
            let key: i32 = weapon_keys[i as usize];
            if state.game.g_game.gamekeydown[key as usize] {
                cmd.buttons = (cmd.buttons as i32 | BT_CHANGE) as u8;
                cmd.buttons = (cmd.buttons as i32 | i << BT_WEAPONSHIFT) as u8;
                break;
            }
            i += 1;
        }
    }
    state.game.g_game.next_weapon = 0;
    if state.game.g_game.mousearray[(state.game.m_controls.mousebforward + 1) as usize] {
        forward += state.game.g_game.forwardmove[speed as usize];
    }
    if state.game.g_game.mousearray[(state.game.m_controls.mousebbackward + 1) as usize] {
        forward -= state.game.g_game.forwardmove[speed as usize];
    }
    if state.game.m_controls.dclick_use != 0 {
        if state.game.g_game.mousearray[(state.game.m_controls.mousebforward + 1) as usize]
            != state.game.g_game.dclickstate
            && state.game.g_game.dclicktime > 1
        {
            state.game.g_game.dclickstate =
                state.game.g_game.mousearray[(state.game.m_controls.mousebforward + 1) as usize];
            if state.game.g_game.dclickstate {
                state.game.g_game.dclicks += 1;
            }
            if state.game.g_game.dclicks == 2 {
                cmd.buttons = (cmd.buttons as i32 | BT_USE) as u8;
                state.game.g_game.dclicks = 0;
            } else {
                state.game.g_game.dclicktime = 0;
            }
        } else {
            state.game.g_game.dclicktime += state.game.d_loop.ticdup;
            if state.game.g_game.dclicktime > 20 {
                state.game.g_game.dclicks = 0;
                state.game.g_game.dclickstate = false;
            }
        }
        bstrafe = state.game.g_game.mousearray[(state.game.m_controls.mousebstrafe + 1) as usize]
            || state.game.g_game.joyarray[(state.game.m_controls.joybstrafe + 1) as usize];
        if bstrafe != state.game.g_game.dclickstate2 && state.game.g_game.dclicktime2 > 1 {
            state.game.g_game.dclickstate2 = bstrafe;
            if state.game.g_game.dclickstate2 {
                state.game.g_game.dclicks2 += 1;
            }
            if state.game.g_game.dclicks2 == 2 {
                cmd.buttons = (cmd.buttons as i32 | BT_USE) as u8;
                state.game.g_game.dclicks2 = 0;
            } else {
                state.game.g_game.dclicktime2 = 0;
            }
        } else {
            state.game.g_game.dclicktime2 += state.game.d_loop.ticdup;
            if state.game.g_game.dclicktime2 > 20 {
                state.game.g_game.dclicks2 = 0;
                state.game.g_game.dclickstate2 = false;
            }
        }
    }
    forward += state.game.g_game.mousey;
    if strafe {
        side += state.game.g_game.mousex * 2;
    } else {
        cmd.angleturn = (cmd.angleturn as i32 - state.game.g_game.mousex * 0x8) as i16;
    }
    if state.game.g_game.mousex == 0 {
        state.game.g_game.testcontrols_mousespeed = 0;
    }
    state.game.g_game.mousey = 0;
    state.game.g_game.mousex = state.game.g_game.mousey;
    if forward > state.game.g_game.forwardmove[1] {
        forward = state.game.g_game.forwardmove[1];
    } else if forward < -state.game.g_game.forwardmove[1] {
        forward = -state.game.g_game.forwardmove[1];
    }
    if side > state.game.g_game.forwardmove[1] {
        side = state.game.g_game.forwardmove[1];
    } else if side < -state.game.g_game.forwardmove[1] {
        side = -state.game.g_game.forwardmove[1];
    }
    cmd.forwardmove = (cmd.forwardmove as i32 + forward) as i8;
    cmd.sidemove = (cmd.sidemove as i32 + side) as i8;
    if state.game.g_game.sendpause {
        state.game.g_game.sendpause = false;
        cmd.buttons = (BT_SPECIAL | BTS_PAUSE) as u8;
    }
    if state.game.g_game.sendsave {
        state.game.g_game.sendsave = false;
        cmd.buttons =
            (BT_SPECIAL | BTS_SAVEGAME | state.game.g_game.savegameslot << BTS_SAVESHIFT) as u8;
    }
    if state.game.g_game.lowres_turn {
        let desired_angleturn: i16 =
            (cmd.angleturn as i32 + state.game.g_game.g_build_ticcmd_carry as i32) as i16;
        cmd.angleturn = ((desired_angleturn as i32 + 128) & 0xff00) as i16;
        state.game.g_game.g_build_ticcmd_carry =
            (desired_angleturn as i32 - cmd.angleturn as i32) as i16;
    }
}
pub fn do_load_level(state: &mut GameState) {
    state.render.r_sky.skyflatnum =
        flat_num_for_name(&state.render.r_data, &state.assets.w_wad, "F_SKY1");
    if state.game.doomstat.gamemode as u32 == GameMode::Commercial as u32
        && [GameVersion::Final2, GameVersion::Chex].contains(&state.game.doomstat.gameversion)
    {
        let skytexturename: &str = if state.game.g_game.gamemap < 12 {
            "SKY1"
        } else if state.game.g_game.gamemap < 21 {
            "SKY2"
        } else {
            "SKY3"
        };
        state.render.r_sky.skytexture = texture_num_for_name(&state.render.r_data, skytexturename);
    }
    state.game.g_game.levelstarttic = state.game.d_loop.gametic;
    if state.game.d_main.wipegamestate == GameScreenState::Level {
        state.game.d_main.wipegamestate = GameScreenState::Wipped;
    }
    state.game.g_game.gamestate = GameScreenState::Level;
    for i in 0..(MAXPLAYERS as usize) {
        state.game.g_game.turbodetected[i] = false;
        if state.game.g_game.playeringame[i]
            && state.game.g_game.players[i].playerstate == PlayerState::Dead
        {
            state.game.g_game.players[i].playerstate = PlayerState::Reborn;
        }
        state.game.g_game.players[i].frags = [0; 4];
    }
    setup_level(
        state,
        state.game.g_game.gameepisode,
        state.game.g_game.gamemap,
    );
    state.game.g_game.displayplayer = state.game.g_game.consoleplayer;
    state.game.g_game.gameaction = GameAction::Nothing;
    state.game.g_game.gamekeydown = [false; 256];
    state.game.g_game.joystrafemove = 0;
    state.game.g_game.joyymove = state.game.g_game.joystrafemove;
    state.game.g_game.joyxmove = state.game.g_game.joyymove;
    state.game.g_game.mousey = 0;
    state.game.g_game.mousex = state.game.g_game.mousey;
    state.game.g_game.paused = false;
    state.game.g_game.sendsave = state.game.g_game.paused;
    state.game.g_game.sendpause = state.game.g_game.sendsave;
    state.game.g_game.mousearray = [false; 9];
    state.game.g_game.joyarray = [false; 21];
    if state.game.g_game.testcontrols {
        state.game.g_game.players[state.game.g_game.consoleplayer].message =
            Some("Press escape to quit.".to_string());
    }
}
fn set_joy_buttons(g_game: &mut GGameState, m_controls: &MControlsState, buttons_mask: u32) {
    for i in 0..MAX_JOY_BUTTONS {
        let button_on: i32 = (buttons_mask & (1 << i) as u32 != 0) as i32;
        if !g_game.joyarray[(i + 1) as usize] && button_on != 0 {
            if i == m_controls.joybprevweapon {
                g_game.next_weapon = -1;
            } else if i == m_controls.joybnextweapon {
                g_game.next_weapon = 1;
            }
        }
        g_game.joyarray[(i + 1) as usize] = button_on != 0;
    }
}
fn set_mouse_buttons(g_game: &mut GGameState, m_controls: &MControlsState, buttons_mask: u32) {
    for i in 0..MAX_MOUSE_BUTTONS {
        let button_on: u32 = (buttons_mask & (1 << i) as u32 != 0) as u32;
        if !g_game.mousearray[(i + 1) as usize] && button_on != 0 {
            if i == m_controls.mousebprevweapon {
                g_game.next_weapon = -1;
            } else if i == m_controls.mousebnextweapon {
                g_game.next_weapon = 1;
            }
        }
        g_game.mousearray[(i + 1) as usize] = button_on != 0;
    }
}
pub fn g_responder(state: &mut GameState, ev: Event) -> bool {
    if state.game.g_game.gamestate == GameScreenState::Level
        && ev.kind == EvType::Keydown
        && ev.data1 == state.game.m_controls.key_spy
        && (state.game.g_game.singledemo || state.game.g_game.deathmatch == 0)
    {
        loop {
            state.game.g_game.displayplayer = state.game.g_game.displayplayer.next_wrapping();
            if state.game.g_game.playeringame[state.game.g_game.displayplayer]
                || state.game.g_game.displayplayer == state.game.g_game.consoleplayer
            {
                break;
            }
        }
        return true;
    }
    if state.game.g_game.gameaction == GameAction::Nothing
        && !state.game.g_game.singledemo
        && (state.game.g_game.demoplayback
            || state.game.g_game.gamestate == GameScreenState::Demoscreen)
    {
        if ev.kind == EvType::Keydown
            || ev.kind == EvType::Mouse && ev.data1 != 0
            || ev.kind == EvType::Joystick && ev.data1 != 0
        {
            start_control_panel(&mut state.ui.m_menu);
            return true;
        }
        return false;
    }
    if state.game.g_game.gamestate == GameScreenState::Level {
        if hu_responder(
            &mut state.game.g_game,
            &mut state.ui.hu_stuff,
            &state.game.m_controls,
            &ev,
        ) {
            return true;
        }
        if st_responder(state, &ev) {
            return true;
        }
        if am_responder(state, &ev) {
            return true;
        }
    }
    if state.game.g_game.gamestate == GameScreenState::Finale && f_responder(state, &ev) {
        return true;
    }
    if state.game.g_game.testcontrols && ev.kind == EvType::Mouse {
        state.game.g_game.testcontrols_mousespeed = (ev.data2).abs();
    }
    if ev.kind == EvType::Keydown && ev.data1 == state.game.m_controls.key_prevweapon {
        state.game.g_game.next_weapon = -1;
    } else if ev.kind == EvType::Keydown && ev.data1 == state.game.m_controls.key_nextweapon {
        state.game.g_game.next_weapon = 1;
    }
    match ev.kind as u32 {
        0 => {
            if ev.data1 == state.game.m_controls.key_pause {
                state.game.g_game.sendpause = true;
            } else if ev.data1 < NUMKEYS {
                state.game.g_game.gamekeydown[ev.data1 as usize] = true;
            }
            return true;
        }
        1 => {
            if ev.data1 < NUMKEYS {
                state.game.g_game.gamekeydown[ev.data1 as usize] = false;
            }
            return false;
        }
        2 => {
            set_mouse_buttons(
                &mut state.game.g_game,
                &state.game.m_controls,
                ev.data1 as u32,
            );
            state.game.g_game.mousex = ev.data2 * (state.ui.m_menu.mouse_sensitivity + 5) / 10;
            state.game.g_game.mousey = ev.data3 * (state.ui.m_menu.mouse_sensitivity + 5) / 10;
            return true;
        }
        3 => {
            set_joy_buttons(
                &mut state.game.g_game,
                &state.game.m_controls,
                ev.data1 as u32,
            );
            state.game.g_game.joyxmove = ev.data2;
            state.game.g_game.joyymove = ev.data3;
            state.game.g_game.joystrafemove = ev.data4;
            return true;
        }
        _ => {}
    }
    false
}
pub fn g_ticker(state: &mut GameState, netcmds: &[TicCmd]) {
    for i in 0..MAXPLAYERS {
        if state.game.g_game.playeringame[i as usize]
            && state.game.g_game.players[i as usize].playerstate == PlayerState::Reborn
        {
            do_reborn(state, i);
        }
    }
    while state.game.g_game.gameaction != GameAction::Nothing {
        match state.game.g_game.gameaction {
            GameAction::LoadLevel => {
                do_load_level(state);
            }
            GameAction::NewGame => {
                do_new_game(state);
            }
            GameAction::LoadGame => {
                do_load_game(state);
            }
            GameAction::SaveGame => {
                do_save_game(state);
            }
            GameAction::PlayDemo => {
                do_play_demo(state);
            }
            GameAction::Completed => {
                do_completed(state);
            }
            GameAction::Victory => {
                f_start_finale(state);
            }
            GameAction::WorldDone => {
                do_world_done(state);
            }
            GameAction::Screenshot => {
                v_screen_shot(
                    &mut *state.assets.fs,
                    &state.io.i_video,
                    &mut state.assets.w_wad,
                );
                state.game.g_game.players[state.game.g_game.consoleplayer].message =
                    Some("screen shot".to_string());
                state.game.g_game.gameaction = GameAction::Nothing;
            }
            GameAction::Nothing => {}
        }
    }
    let buf: i32 = state.game.d_loop.gametic / state.game.d_loop.ticdup % BACKUPTICS;
    for i in 0..MAXPLAYERS {
        if state.game.g_game.playeringame[i as usize] {
            state.game.g_game.players[i as usize].cmd = netcmds[i as usize];
            if state.game.g_game.demoplayback {
                read_demo_ticcmd(state, i as usize);
            }
            if state.game.g_game.demorecording {
                write_demo_ticcmd(state, i as usize);
            }
            if state.game.g_game.players[i as usize].cmd.forwardmove as i32 > TURBOTHRESHOLD {
                state.game.g_game.turbodetected[i as usize] = true;
            }
            if state.game.d_loop.gametic & 31 == 0
                && (state.game.d_loop.gametic >> 5) % MAXPLAYERS == i
                && state.game.g_game.turbodetected[i as usize]
            {
                let player_name = PLAYER_NAMES[i as usize];
                state.game.g_game.players[state.game.g_game.consoleplayer].message =
                    Some(format!("{player_name} is turbo!"));
                state.game.g_game.turbodetected[i as usize] = false;
            }
            if state.game.g_game.netgame
                && !state.game.g_game.netdemo
                && state.game.d_loop.gametic % state.game.d_loop.ticdup == 0
            {
                if state.game.d_loop.gametic > BACKUPTICS
                    && state.game.g_game.consistancy[i as usize][buf as usize] as i32
                        != state.game.g_game.players[i as usize].cmd.consistancy as i32
                {
                    error(&format!(
                        "consistency failure ({} should be {})",
                        state.game.g_game.players[i as usize].cmd.consistancy as i32,
                        state.game.g_game.consistancy[i as usize][buf as usize] as i32,
                    ));
                }
                if let Some(mo_id) = state.game.g_game.players[i as usize].mo {
                    state.game.g_game.consistancy[i as usize][buf as usize] =
                        state.world.p_mobj.mo(mo_id).x as u8;
                } else {
                    state.game.g_game.consistancy[i as usize][buf as usize] =
                        state.world.m_random.rndindex;
                }
            }
        }
    }
    for i in 0..(MAXPLAYERS as usize) {
        if state.game.g_game.playeringame[i]
            && state.game.g_game.players[i].cmd.buttons as i32 & BT_SPECIAL != 0
        {
            match state.game.g_game.players[i].cmd.buttons as i32 & BT_SPECIALMASK {
                1 => {
                    state.game.g_game.paused = !state.game.g_game.paused;
                    if state.game.g_game.paused {
                        pause_sound(
                            &mut state.audio.i_sound,
                            &mut *state.io.platform,
                            &mut state.audio.s_sound,
                        );
                    } else {
                        resume_sound(
                            &mut state.audio.i_sound,
                            &mut *state.io.platform,
                            &mut state.audio.s_sound,
                        );
                    }
                }
                2 => {
                    if state.game.g_game.savedescription.is_empty() {
                        state.game.g_game.savedescription = "NET GAME".to_string();
                    }
                    state.game.g_game.savegameslot =
                        (state.game.g_game.players[i].cmd.buttons as i32 & BTS_SAVEMASK)
                            >> BTS_SAVESHIFT;
                    state.game.g_game.gameaction = GameAction::SaveGame;
                }
                _ => {}
            }
        }
    }
    if state.game.g_game.oldgamestate == GameScreenState::Intermission
        && state.game.g_game.gamestate != GameScreenState::Intermission
    {
        wi_end(state);
    }
    state.game.g_game.oldgamestate = state.game.g_game.gamestate;
    match state.game.g_game.gamestate {
        GameScreenState::Level => {
            p_ticker(state);
            st_ticker(state);
            am_ticker(
                &mut state.ui.am_map,
                &mut state.game.g_game,
                &state.world.p_mobj,
            );
            hu_ticker(state);
        }
        GameScreenState::Intermission => {
            wi_ticker(state);
        }
        GameScreenState::Finale => {
            f_ticker(state);
        }
        GameScreenState::Demoscreen => {
            page_ticker(&mut state.game.d_main);
        }
        GameScreenState::Wipped => {}
    }
}
pub fn player_finish_level(g_game: &mut GGameState, p_mobj: &mut PMobjState, player: i32) {
    let p = &mut g_game.players[player as usize];
    p.powers = [0; 6];
    p.cards = [false; 6];
    let mo_id = p.mo.unwrap();
    p.extralight = 0;
    p.fixedcolormap = 0;
    p.damagecount = 0;
    p.bonuscount = 0;
    p_mobj.mo_mut(mo_id).flags &= !MobjFlags::SHADOW;
}
pub fn player_reborn(state: &mut GGameState, player: i32) {
    let old = &state.players[player as usize];
    let (frags, killcount, itemcount, secretcount) =
        (old.frags, old.killcount, old.itemcount, old.secretcount);
    let p = &mut state.players[player as usize];
    *p = NEW_PLAYER;
    p.frags = frags;
    p.killcount = killcount;
    p.itemcount = itemcount;
    p.secretcount = secretcount;
    p.attackdown = true;
    p.usedown = p.attackdown;
    p.playerstate = PlayerState::Live;
    p.health = DEH_INITIAL_HEALTH;
    p.pendingweapon = WeaponType::Pistol;
    p.readyweapon = p.pendingweapon;
    p.weaponowned[WeaponType::Fist as usize] = true;
    p.weaponowned[WeaponType::Pistol as usize] = true;
    p.ammo[AmmoType::Clip as usize] = DEH_INITIAL_BULLETS;
    p.maxammo[..NUMAMMO as usize].copy_from_slice(&MAXAMMO[..NUMAMMO as usize]);
}
pub fn check_spot(state: &mut GameState, playernum: i32, mthing: &MapThing) -> bool {
    if state.game.g_game.players[playernum as usize].mo.is_none() {
        for i in 0..playernum as usize {
            let other_mo = state
                .world
                .p_mobj
                .mo(state.game.g_game.players[i].mo.unwrap());
            if other_mo.x == (mthing.x as i32) << FRACBITS
                && other_mo.y == (mthing.y as i32) << FRACBITS
            {
                return false;
            }
        }
        return true;
    }
    let x: Fixed = ((mthing.x as i32) << FRACBITS) as Fixed;
    let y: Fixed = ((mthing.y as i32) << FRACBITS) as Fixed;
    let player_mo_id = state.game.g_game.players[playernum as usize].mo.unwrap();
    if !check_position(state, player_mo_id, x, y) {
        return false;
    }
    if state.game.g_game.bodyqueslot >= BODYQUESIZE {
        let old_id = state.game.g_game.bodyque
            [(state.game.g_game.bodyqueslot % BODYQUESIZE) as usize]
            .unwrap();
        if state.world.p_mobj.is_live(old_id) {
            remove_mobj(state, old_id);
        }
    }
    state.game.g_game.bodyque[(state.game.g_game.bodyqueslot % BODYQUESIZE) as usize] =
        Some(player_mo_id);
    state.game.g_game.bodyqueslot += 1;
    let ss = point_in_subsector(&state.world.p_setup, x, y);
    let xa: Fixed;
    let ya: Fixed;
    let an: i32 = (ANG45 >> ANGLETOFINESHIFT) * (mthing.angle as i32 / 45);
    match an {
        4096 => {
            xa = FINETANGENT[2048];
            ya = FINETANGENT[0];
        }
        5120 => {
            xa = FINETANGENT[3072];
            ya = FINETANGENT[1024];
        }
        6144 => {
            xa = FINESINE[0];
            ya = FINETANGENT[2048];
        }
        7168 => {
            xa = FINESINE[1024];
            ya = FINETANGENT[3072];
        }
        0 | 1024 | 2048 | 3072 => {
            xa = FINECOSINE[an as usize];
            ya = FINESINE[an as usize];
        }
        _ => {
            error(&format!("G_CheckSpot: unexpected angle {an}\n"));
        }
    }
    let floorheight = state
        .world
        .p_setup
        .sector_mut(state.world.p_setup.subsectors[ss.0 as usize].sector)
        .floorheight;
    let mo = spawn_mobj(state, x + 20 * xa, y + 20 * ya, floorheight, MobjType::Tfog);
    if state.game.g_game.players[state.game.g_game.consoleplayer].viewz != 1 {
        s_start_sound(state, SoundOrigin::Mobj(mo), SfxName::Telept as i32);
    }
    true
}
pub fn death_match_spawn_player(state: &mut GameState, playernum: i32) {
    let selections = state.world.p_setup.deathmatch_p as i32;
    if selections < 4 {
        error(&format!("Only {selections} deathmatch spots, 4 required"));
    }
    for _ in 0..20 {
        let i = (p_random(&mut state.world.m_random) % selections) as usize;
        let dm_spot = state.world.p_setup.deathmatchstarts[i];
        if check_spot(state, playernum, &dm_spot) {
            state.world.p_setup.deathmatchstarts[i].kind = (playernum + 1) as i16;
            let dm_spot = state.world.p_setup.deathmatchstarts[i];
            spawn_player(state, dm_spot);
            return;
        }
    }
    let spot = state.world.p_setup.playerstarts[playernum as usize];
    spawn_player(state, spot);
}
pub fn do_reborn(state: &mut GameState, playernum: i32) {
    if state.game.g_game.netgame {
        let player_mo_id = state.game.g_game.players[playernum as usize].mo.unwrap();
        state.world.p_mobj.mo_mut(player_mo_id).player = None;
        if state.game.g_game.deathmatch != 0 {
            death_match_spawn_player(state, playernum);
            return;
        }
        let spot = state.world.p_setup.playerstarts[playernum as usize];
        if check_spot(state, playernum, &spot) {
            let spot = state.world.p_setup.playerstarts[playernum as usize];
            spawn_player(state, spot);
            return;
        }
        for i in 0..MAXPLAYERS as usize {
            let spot = state.world.p_setup.playerstarts[i];
            if check_spot(state, playernum, &spot) {
                state.world.p_setup.playerstarts[i].kind = (playernum + 1) as i16;
                let spot = state.world.p_setup.playerstarts[i];
                spawn_player(state, spot);
                state.world.p_setup.playerstarts[i].kind = (i as i32 + 1) as i16;
                return;
            }
        }
        let spot = state.world.p_setup.playerstarts[playernum as usize];
        spawn_player(state, spot);
    } else {
        state.game.g_game.gameaction = GameAction::LoadLevel;
    }
}
pub fn g_screen_shot(g_game: &mut GGameState) {
    g_game.gameaction = GameAction::Screenshot;
}
pub static PARS: [[i32; 10]; 4] = [
    [0; 10],
    [0, 30, 75, 120, 90, 165, 180, 180, 30, 165],
    [0, 90, 90, 90, 120, 90, 360, 240, 30, 170],
    [0, 90, 45, 90, 150, 90, 90, 165, 30, 135],
];
pub static CPARS: [i32; 32] = [
    30, 90, 120, 120, 90, 150, 120, 120, 270, 90, 210, 150, 150, 150, 210, 150, 420, 150, 210, 150,
    240, 150, 180, 150, 150, 300, 330, 420, 300, 180, 120, 30,
];
pub fn exit_level(g_game: &mut GGameState) {
    g_game.secretexit = false;
    g_game.gameaction = GameAction::Completed;
}
pub fn secret_exit_level(doomstat: &DoomstatState, g_game: &mut GGameState, w_wad: &WWadState) {
    g_game.secretexit = !(doomstat.gamemode as u32 == GameMode::Commercial as u32
        && check_num_for_name(w_wad, "map31").is_none());
    g_game.gameaction = GameAction::Completed;
}
pub fn do_completed(state: &mut GameState) {
    state.game.g_game.gameaction = GameAction::Nothing;
    for i in 0..MAXPLAYERS {
        if state.game.g_game.playeringame[i as usize] {
            player_finish_level(&mut state.game.g_game, &mut state.world.p_mobj, i);
        }
    }
    if state.ui.am_map.automapactive {
        am_stop(state);
    }
    if state.game.doomstat.gamemode as u32 != GameMode::Commercial as u32 {
        if state.game.doomstat.gameversion == GameVersion::Chex {
            if state.game.g_game.gamemap == 5 {
                state.game.g_game.gameaction = GameAction::Victory;
                return;
            }
        } else {
            match state.game.g_game.gamemap {
                8 => {
                    state.game.g_game.gameaction = GameAction::Victory;
                    return;
                }
                9 => {
                    for i in 0..(MAXPLAYERS as usize) {
                        state.game.g_game.players[i].didsecret = true;
                    }
                }
                _ => {}
            }
        }
    }
    if state.game.g_game.gamemap == 8
        && state.game.doomstat.gamemode as u32 != GameMode::Commercial as u32
    {
        state.game.g_game.gameaction = GameAction::Victory;
        return;
    }
    if state.game.g_game.gamemap == 9
        && state.game.doomstat.gamemode as u32 != GameMode::Commercial as u32
    {
        for i in 0..(MAXPLAYERS as usize) {
            state.game.g_game.players[i].didsecret = true;
        }
    }
    state.game.g_game.wminfo.didsecret =
        state.game.g_game.players[state.game.g_game.consoleplayer].didsecret;
    state.game.g_game.wminfo.epsd = state.game.g_game.gameepisode - 1;
    state.game.g_game.wminfo.last = state.game.g_game.gamemap - 1;
    if state.game.doomstat.gamemode as u32 == GameMode::Commercial as u32 {
        if state.game.g_game.secretexit {
            match state.game.g_game.gamemap {
                15 => {
                    state.game.g_game.wminfo.next = 30;
                }
                31 => {
                    state.game.g_game.wminfo.next = 31;
                }
                _ => {}
            }
        } else {
            match state.game.g_game.gamemap {
                31 | 32 => {
                    state.game.g_game.wminfo.next = 15;
                }
                _ => {
                    state.game.g_game.wminfo.next = state.game.g_game.gamemap;
                }
            }
        }
    } else if state.game.g_game.secretexit {
        state.game.g_game.wminfo.next = 8;
    } else if state.game.g_game.gamemap == 9 {
        match state.game.g_game.gameepisode {
            1 => {
                state.game.g_game.wminfo.next = 3;
            }
            2 => {
                state.game.g_game.wminfo.next = 5;
            }
            3 => {
                state.game.g_game.wminfo.next = 6;
            }
            4 => {
                state.game.g_game.wminfo.next = 2;
            }
            _ => {}
        }
    } else {
        state.game.g_game.wminfo.next = state.game.g_game.gamemap;
    }
    state.game.g_game.wminfo.maxkills = state.game.g_game.totalkills;
    state.game.g_game.wminfo.maxitems = state.game.g_game.totalitems;
    state.game.g_game.wminfo.maxsecret = state.game.g_game.totalsecret;
    state.game.g_game.wminfo.maxfrags = 0;
    if state.game.doomstat.gamemode as u32 == GameMode::Commercial as u32 {
        state.game.g_game.wminfo.partime =
            TICRATE * CPARS[(state.game.g_game.gamemap - 1) as usize];
    } else if state.game.g_game.gameepisode < 4 {
        state.game.g_game.wminfo.partime = TICRATE
            * PARS[state.game.g_game.gameepisode as usize][state.game.g_game.gamemap as usize];
    } else {
        state.game.g_game.wminfo.partime = TICRATE * CPARS[state.game.g_game.gamemap as usize];
    }
    state.game.g_game.wminfo.pnum = state.game.g_game.consoleplayer.as_i32();
    for i in 0..(MAXPLAYERS as usize) {
        state.game.g_game.wminfo.plyr[i].intercept = state.game.g_game.playeringame[i];
        state.game.g_game.wminfo.plyr[i].skills = state.game.g_game.players[i].killcount;
        state.game.g_game.wminfo.plyr[i].sitems = state.game.g_game.players[i].itemcount;
        state.game.g_game.wminfo.plyr[i].ssecret = state.game.g_game.players[i].secretcount;
        state.game.g_game.wminfo.plyr[i].stime = state.world.p_tick.leveltime;
        state.game.g_game.wminfo.plyr[i].frags = state.game.g_game.players[i].frags;
    }
    state.game.g_game.gamestate = GameScreenState::Intermission;
    state.game.g_game.viewactive = false;
    state.ui.am_map.automapactive = false;
    stat_copy(
        &state.game.g_game,
        &state.game.m_argv,
        &mut state.ui.statdump,
    );
    wi_start(state);
}
pub fn world_done(state: &mut GameState) {
    state.game.g_game.gameaction = GameAction::WorldDone;
    if state.game.g_game.secretexit {
        state.game.g_game.players[state.game.g_game.consoleplayer].didsecret = true;
    }
    if state.game.doomstat.gamemode as u32 == GameMode::Commercial as u32 {
        let start_finale = match state.game.g_game.gamemap {
            15 | 31 => state.game.g_game.secretexit,
            6 | 11 | 20 | 30 => true,
            _ => false,
        };
        if start_finale {
            f_start_finale(state);
        }
    }
}
pub fn do_world_done(state: &mut GameState) {
    state.game.g_game.gamestate = GameScreenState::Level;
    state.game.g_game.gamemap = state.game.g_game.wminfo.next + 1;
    do_load_level(state);
    state.game.g_game.gameaction = GameAction::Nothing;
    state.game.g_game.viewactive = true;
}
pub fn g_load_game(g_game: &mut GGameState, name: &str) {
    g_game.savename = name.to_string();
    g_game.gameaction = GameAction::LoadGame;
}
pub fn do_load_game(state: &mut GameState) {
    state.game.g_game.gameaction = GameAction::Nothing;
    let Some(image) = read_file(&mut *state.assets.fs, &state.game.g_game.savename) else {
        return;
    };
    state.world.p_saveg.save_buffer = image;
    state.world.p_saveg.save_pos = 0;
    state.world.p_saveg.savegame_error = false;
    if !read_save_game_header(state) {
        report_save_game_read_error(&state.world.p_saveg, &mut *state.io.platform);
        state.world.p_saveg.save_buffer = Vec::new();
        return;
    }
    let savedleveltime: i32 = state.world.p_tick.leveltime;
    let (skill, episode, map) = (
        state.game.g_game.gameskill,
        state.game.g_game.gameepisode,
        state.game.g_game.gamemap,
    );
    init_new(state, skill, episode, map);
    state.world.p_tick.leveltime = savedleveltime;
    un_archive_players(&mut state.game.g_game, &mut state.world.p_saveg);
    un_archive_world(&mut state.world.p_saveg, &mut state.world.p_setup);
    un_archive_thinkers(state);
    un_archive_specials(&mut state.world);
    if !read_save_game_eof(&mut state.world.p_saveg) {
        report_save_game_read_error(&state.world.p_saveg, &mut *state.io.platform);
        error("Bad savegame");
    }
    report_save_game_read_error(&state.world.p_saveg, &mut *state.io.platform);
    state.world.p_saveg.save_buffer = Vec::new();
    if state.render.r_main.setsizeneeded {
        execute_set_view_size(&mut state.render);
    }
    fill_back_screen(state);
}
pub fn g_save_game(g_game: &mut GGameState, slot: i32, description: &str) {
    g_game.savegameslot = slot;
    g_game.savedescription = description.to_string();
    g_game.sendsave = true;
}
pub fn do_save_game(state: &mut GameState) {
    let temp_savegame_file = temp_save_game_file(&state.game.d_main, &mut state.world.p_saveg);
    let savegame_file = save_game_file(&state.game.d_main, state.game.g_game.savegameslot);
    state.world.p_saveg.save_buffer = Vec::new();
    state.world.p_saveg.save_pos = 0;
    state.world.p_saveg.savegame_error = false;
    let savedescription = state.game.g_game.savedescription.clone();
    write_save_game_header(state, &savedescription);
    archive_players(&state.game.g_game, &mut state.world.p_saveg);
    archive_world(&mut state.world.p_saveg, &mut state.world.p_setup);
    archive_thinkers(&mut state.world);
    archive_specials(&mut state.world);
    write_save_game_eof(&mut state.world.p_saveg);
    if state.game.g_game.vanilla_savegame_limit != 0
        && state.world.p_saveg.save_buffer.len() > SAVEGAMESIZE as usize
    {
        error("Savegame buffer overrun");
    }
    let image = core::mem::take(&mut state.world.p_saveg.save_buffer);
    if !state.assets.fs.write_file(&temp_savegame_file, &image) {
        let recovery_file = state.assets.fs.temp_path("recovery.dsg");
        if !state.assets.fs.write_file(&recovery_file, &image) {
            error(&format!(
                "Failed to open either '{temp_savegame_file}' or '{recovery_file}' to write savegame.",
            ));
        }
        error(&format!(
            "Failed to open savegame file '{temp_savegame_file}' for writing.\nBut your game has been saved to '{recovery_file}' for recovery.",
        ));
    }
    state.assets.fs.remove_file(&savegame_file);
    state.assets.fs.rename(&temp_savegame_file, &savegame_file);
    state.game.g_game.gameaction = GameAction::Nothing;
    state.game.g_game.savedescription.clear();
    state.game.g_game.players[state.game.g_game.consoleplayer].message =
        Some("game saved.".to_string());
    fill_back_screen(state);
}
pub fn defered_init_new(g_game: &mut GGameState, skill: SkillType, episode: i32, map: i32) {
    g_game.d_skill = skill;
    g_game.d_episode = episode;
    g_game.d_map = map;
    g_game.gameaction = GameAction::NewGame;
}
pub fn do_new_game(state: &mut GameState) {
    state.game.g_game.demoplayback = false;
    state.game.g_game.netdemo = false;
    state.game.g_game.netgame = false;
    state.game.g_game.deathmatch = 0;
    state.game.g_game.playeringame[3] = false;
    state.game.g_game.playeringame[2] = state.game.g_game.playeringame[3];
    state.game.g_game.playeringame[1] = state.game.g_game.playeringame[2];
    state.game.d_main.respawnparm = false;
    state.game.d_main.fastparm = false;
    state.game.d_main.nomonsters = false;
    state.game.g_game.consoleplayer = PlayerId(0);
    let (d_skill, d_episode, d_map) = (
        state.game.g_game.d_skill,
        state.game.g_game.d_episode,
        state.game.g_game.d_map,
    );
    init_new(state, d_skill, d_episode, d_map);
    state.game.g_game.gameaction = GameAction::Nothing;
}
pub fn init_new(state: &mut GameState, mut skill: SkillType, mut episode: i32, mut map: i32) {
    let skytexturename: &str;
    if state.game.g_game.paused {
        state.game.g_game.paused = false;
        resume_sound(
            &mut state.audio.i_sound,
            &mut *state.io.platform,
            &mut state.audio.s_sound,
        );
    }
    if skill > SkillType::Nightmare {
        skill = SkillType::Nightmare;
    }
    if state.game.doomstat.gameversion.is_ultimate_or_higher() {
        if episode == 0 {
            episode = 4;
        }
    } else {
        episode = episode.clamp(1, 3);
    }
    if episode > 1 && state.game.doomstat.gamemode as u32 == GameMode::Shareware as u32 {
        episode = 1;
    }
    if map < 1 {
        map = 1;
    }
    if map > 9 && state.game.doomstat.gamemode as u32 != GameMode::Commercial as u32 {
        map = 9;
    }
    clear_random(&mut state.world.m_random);
    state.game.g_game.respawnmonsters =
        skill == SkillType::Nightmare || state.game.d_main.respawnparm;
    if state.game.d_main.fastparm
        || skill == SkillType::Nightmare && state.game.g_game.gameskill != SkillType::Nightmare
    {
        for i in StateNum::SargRun1 as i32..=StateNum::SargPain2 as i32 {
            state.assets.info.states[i as usize].tics >>= 1;
        }
        state.assets.info.mobjinfo[MobjType::Bruisershot as usize].speed = 20 * FRACUNIT;
        state.assets.info.mobjinfo[MobjType::Headshot as usize].speed = 20 * FRACUNIT;
        state.assets.info.mobjinfo[MobjType::Troopshot as usize].speed = 20 * FRACUNIT;
    } else if skill != SkillType::Nightmare && state.game.g_game.gameskill == SkillType::Nightmare {
        for i in StateNum::SargRun1 as i32..=StateNum::SargPain2 as i32 {
            state.assets.info.states[i as usize].tics <<= 1;
        }
        state.assets.info.mobjinfo[MobjType::Bruisershot as usize].speed = 15 * FRACUNIT;
        state.assets.info.mobjinfo[MobjType::Headshot as usize].speed = 10 * FRACUNIT;
        state.assets.info.mobjinfo[MobjType::Troopshot as usize].speed = 10 * FRACUNIT;
    }
    for i in 0..(MAXPLAYERS as usize) {
        state.game.g_game.players[i].playerstate = PlayerState::Reborn;
    }
    state.game.g_game.usergame = true;
    state.game.g_game.paused = false;
    state.game.g_game.demoplayback = false;
    state.ui.am_map.automapactive = false;
    state.game.g_game.viewactive = true;
    state.game.g_game.gameepisode = episode;
    state.game.g_game.gamemap = map;
    state.game.g_game.gameskill = skill;
    state.game.g_game.viewactive = true;
    if state.game.doomstat.gamemode as u32 == GameMode::Commercial as u32 {
        if state.game.g_game.gamemap < 12 {
            skytexturename = "SKY1";
        } else if state.game.g_game.gamemap < 21 {
            skytexturename = "SKY2";
        } else {
            skytexturename = "SKY3";
        }
    } else {
        match state.game.g_game.gameepisode {
            2 => {
                skytexturename = "SKY2";
            }
            3 => {
                skytexturename = "SKY3";
            }
            4 => {
                skytexturename = "SKY4";
            }
            _ => {
                skytexturename = "SKY1";
            }
        }
    }
    state.render.r_sky.skytexture = texture_num_for_name(&state.render.r_data, skytexturename);
    do_load_level(state);
}
pub const DEMOMARKER: i32 = 0x80;
pub fn read_demo_ticcmd(state: &mut GameState, player_num: usize) {
    if state.game.g_game.demobuffer[state.game.g_game.demo_p] as i32 == DEMOMARKER {
        check_demo_status(state);
        return;
    }
    let forwardmove = state.game.g_game.demo_read_byte() as i8;
    let sidemove = state.game.g_game.demo_read_byte() as i8;

    let new_angleturn = if state.game.g_game.longtics {
        let lo = state.game.g_game.demo_read_byte() as i16;
        let hi = state.game.g_game.demo_read_byte();
        (lo as i32 | (hi as i32) << 8) as i16
    } else {
        let hi = state.game.g_game.demo_read_byte();
        ((hi as i32) << 8) as i16
    };
    let buttons = state.game.g_game.demo_read_byte();
    let cmd = &mut state.game.g_game.players[player_num].cmd;
    cmd.forwardmove = forwardmove;
    cmd.sidemove = sidemove;
    cmd.angleturn = new_angleturn;
    cmd.buttons = buttons;
}
fn increase_demo_buffer(g_game: &mut GGameState) {
    let new_length = g_game.demoend * 2_usize;
    g_game.demobuffer.resize(new_length, 0);
    g_game.demoend = new_length;
}
pub fn write_demo_ticcmd(state: &mut GameState, player_num: usize) {
    if state.game.g_game.gamekeydown[state.game.m_controls.key_demo_quit as usize] {
        check_demo_status(state);
    }
    let demo_start = state.game.g_game.demo_p;
    let cmd = state.game.g_game.players[player_num].cmd;
    state.game.g_game.demo_write_byte(cmd.forwardmove as u8);
    state.game.g_game.demo_write_byte(cmd.sidemove as u8);
    if state.game.g_game.longtics {
        state
            .game
            .g_game
            .demo_write_byte((cmd.angleturn as i32 & 0xff) as u8);
        state
            .game
            .g_game
            .demo_write_byte((cmd.angleturn as i32 >> 8 & 0xff) as u8);
    } else {
        state
            .game
            .g_game
            .demo_write_byte((cmd.angleturn as i32 >> 8) as u8);
    }
    state.game.g_game.demo_write_byte(cmd.buttons);
    state.game.g_game.demo_p = demo_start;
    if state.game.g_game.demo_p > state.game.g_game.demoend.saturating_sub(16) {
        if state.game.g_game.vanilla_demo_limit != 0 {
            check_demo_status(state);
            return;
        }
        increase_demo_buffer(&mut state.game.g_game);
    }
    read_demo_ticcmd(state, player_num);
}
pub fn record_demo(g_game: &mut GGameState, m_argv: &MArgvState, name: &str) {
    let mut maxsize: i32;
    g_game.usergame = false;
    g_game.demoname = format!("{name}.lmp");
    maxsize = 0x20000;
    if let Some(i) = check_parm_with_args(m_argv, "-maxdemo", 1) {
        maxsize = argv_atoi(&m_argv.myargv[i + 1]) * 1024;
    }
    g_game.demobuffer = vec![0u8; maxsize as usize];
    g_game.demoend = maxsize as usize;
    g_game.demorecording = true;
}
pub fn vanilla_version_code(state: &DoomstatState) -> i32 {
    match state.gameversion as u32 {
        0 => {
            error("Doom 1.2 does not have a version code!");
        }
        1 => {}
        2 => return 107,
        3 => return 108,
        _ => return 109,
    }
    106
}
pub fn begin_recording(game: &mut Game) {
    game.g_game.longtics = parm_exists(&game.m_argv, "-longtics");
    game.g_game.lowres_turn = !game.g_game.longtics;
    game.g_game.demo_p = 0;
    if game.g_game.longtics {
        game.g_game.demo_write_byte(DOOM_191_VERSION as u8);
    } else {
        let version = vanilla_version_code(&game.doomstat) as u8;
        game.g_game.demo_write_byte(version);
    }
    game.g_game.demo_write_byte(game.g_game.gameskill as u8);
    game.g_game.demo_write_byte(game.g_game.gameepisode as u8);
    game.g_game.demo_write_byte(game.g_game.gamemap as u8);
    game.g_game.demo_write_byte(game.g_game.deathmatch as u8);
    game.g_game.demo_write_byte(game.d_main.respawnparm as u8);
    game.g_game.demo_write_byte(game.d_main.fastparm as u8);
    game.g_game.demo_write_byte(game.d_main.nomonsters as u8);
    game.g_game.demo_write_byte(game.g_game.consoleplayer.0);
    for i in 0..(MAXPLAYERS as usize) {
        let b = game.g_game.playeringame[i] as u8;
        game.g_game.demo_write_byte(b);
    }
}
pub fn defered_play_demo(g_game: &mut GGameState, name: FixedCStr<8>) {
    g_game.defdemoname = name;
    g_game.gameaction = GameAction::PlayDemo;
}
fn demo_version_description(_state: &mut GameState, version: i32) -> String {
    match version {
        104 => return "v1.4".to_string(),
        105 => return "v1.5".to_string(),
        106 => return "v1.6/v1.666".to_string(),
        107 => return "v1.7/v1.7a".to_string(),
        108 => return "v1.8".to_string(),
        109 => return "v1.9".to_string(),
        _ => {}
    }
    if (0..=4).contains(&version) {
        "v1.0/v1.1/v1.2".to_string()
    } else {
        format!("{}.{} (unknown)", version / 100, version % 100)
    }
}
pub fn do_play_demo(state: &mut GameState) {
    state.game.g_game.gameaction = GameAction::Nothing;
    let demo_lumpname = state.game.g_game.defdemoname.as_str().into_owned();
    let demo_lumpnum = get_num_for_name(&state.assets.w_wad, &demo_lumpname);
    let demo_lumplen = lump_length(&state.assets.w_wad, demo_lumpnum as u32) as usize;
    state.game.g_game.demobuffer =
        lump_bytes(&*state.assets.fs, &mut state.assets.w_wad, demo_lumpnum)[..demo_lumplen]
            .to_vec();
    state.game.g_game.demo_p = 0;
    let demoversion: i32 = state.game.g_game.demo_read_byte() as i32;
    if demoversion == vanilla_version_code(&state.game.doomstat) {
        state.game.g_game.longtics = false;
    } else if demoversion == DOOM_191_VERSION {
        state.game.g_game.longtics = true;
    } else {
        doom_println!(state.io.platform,
            "Demo is from a different game version!\n(read {}, should be {})\n\n*** You may need to upgrade your version of Doom to v1.9. ***\n    See: https://www.doomworld.com/classicdoom/info/patches.php\n    This appears to be {}.",
            demoversion,
            vanilla_version_code(&state.game.doomstat),
            demo_version_description(state, demoversion),
        );
    }
    let skill: SkillType = skill_from_raw(state.game.g_game.demo_read_byte() as i32);
    let episode: i32 = state.game.g_game.demo_read_byte() as i32;
    let map: i32 = state.game.g_game.demo_read_byte() as i32;
    state.game.g_game.deathmatch = state.game.g_game.demo_read_byte() as i32;
    state.game.d_main.respawnparm = state.game.g_game.demo_read_byte() != 0;
    state.game.d_main.fastparm = state.game.g_game.demo_read_byte() != 0;
    state.game.d_main.nomonsters = state.game.g_game.demo_read_byte() != 0;
    state.game.g_game.consoleplayer = PlayerId(state.game.g_game.demo_read_byte());
    for i in 0..(MAXPLAYERS as usize) {
        state.game.g_game.playeringame[i] = state.game.g_game.demo_read_byte() != 0;
    }
    if state.game.g_game.playeringame[1]
        || parm_exists(&state.game.m_argv, "-solo-net")
        || parm_exists(&state.game.m_argv, "-netdemo")
    {
        state.game.g_game.netgame = true;
        state.game.g_game.netdemo = true;
    }
    state.game.g_game.precache = false;
    init_new(state, skill, episode, map);
    state.game.g_game.precache = true;
    state.game.g_game.starttime = get_time(&mut state.io.i_timer, &mut *state.io.platform);
    state.game.g_game.usergame = false;
    state.game.g_game.demoplayback = true;
}
pub fn time_demo(
    d_loop: &mut DLoopState,
    g_game: &mut GGameState,
    m_argv: &MArgvState,
    name: FixedCStr<8>,
) {
    g_game.nodrawers = parm_exists(m_argv, "-nodraw");
    g_game.timingdemo = true;
    d_loop.singletics = true;
    g_game.defdemoname = name;
    g_game.gameaction = GameAction::PlayDemo;
}
pub fn check_demo_status(state: &mut GameState) -> bool {
    let endtime: i32;
    if state.game.g_game.timingdemo {
        endtime = get_time(&mut state.io.i_timer, &mut *state.io.platform);
        let realtics: i32 = endtime - state.game.g_game.starttime;
        let fps: f32 = state.game.d_loop.gametic as f32 * TICRATE as f32 / realtics as f32;
        state.game.g_game.timingdemo = false;
        state.game.g_game.demoplayback = false;
        error(&format!(
            "timed {} gametics in {} realtics ({:.6} fps)",
            state.game.d_loop.gametic, realtics, fps as f64,
        ));
    }
    if state.game.g_game.demoplayback {
        release_lump_name(&state.assets.w_wad, &state.game.g_game.defdemoname.as_str());
        state.game.g_game.demoplayback = false;
        state.game.g_game.netdemo = false;
        state.game.g_game.netgame = false;
        state.game.g_game.deathmatch = 0;
        state.game.g_game.playeringame[3] = false;
        state.game.g_game.playeringame[2] = state.game.g_game.playeringame[3];
        state.game.g_game.playeringame[1] = state.game.g_game.playeringame[2];
        state.game.d_main.respawnparm = false;
        state.game.d_main.fastparm = false;
        state.game.d_main.nomonsters = false;
        state.game.g_game.consoleplayer = PlayerId(0);
        if state.game.g_game.singledemo {
            i_quit(state);
        } else {
            advance_demo(&mut state.game.d_main);
        }
        return true;
    }
    if state.game.g_game.demorecording {
        state.game.g_game.demo_write_byte(DEMOMARKER as u8);
        let demo_len = state.game.g_game.demo_p;
        state.assets.fs.write_file(
            &state.game.g_game.demoname,
            &state.game.g_game.demobuffer[..demo_len],
        );
        state.game.g_game.demobuffer = Vec::new();
        state.game.g_game.demorecording = false;
        error(&format!("Demo {} recorded", state.game.g_game.demoname));
    }
    false
}
pub const MAX_MOUSE_BUTTONS: i32 = 8;
