use crate::src::d_ticcmd::BT_SPECIAL;
use crate::src::doomdef::TICRATE;
use crate::src::dummy::drone;
use crate::src::dummy::net_client_connected;
use crate::src::game_state::GameState;
use crate::src::i_system::I_AtExit;
use crate::src::i_system::I_Error;
use crate::src::i_timer::I_GetTime;
use crate::src::i_timer::I_GetTimeMS;
use crate::src::i_timer::I_Sleep;
use crate::src::i_video::I_StartTic;
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FRACUNIT;
use crate::src::sha1::sha1_digest_t;
use crate::src::stdint_types::byte;
use crate::src::stdint_types::size_t;
use crate::src::mem_compat::{memcpy, memset};

pub struct DLoopState {
    pub ticdata: [ticcmd_set_t; 128],
    pub maketic: i32,
    pub recvtic: i32,
    pub gametic: i32,
    pub skiptics: i32,
    pub ticdup: i32,
    pub new_sync: bool,
    pub loop_interface: loop_interface_t,
    pub local_playeringame: [bool; 8],
    pub player_class: i32,
    pub lasttime: i32,
    pub frameon: i32,
    pub frameskip: [i32; 4],
    pub oldnettics: i32,
    pub singletics: bool,
    pub try_run_tics_oldentertics: i32,
}

impl DLoopState {
    pub fn new() -> Self {
        DLoopState {
            ticdata: [ticcmd_set_t {
                cmds: [ticcmd_t {
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
                }; 8],
                ingame: [false; 8],
            }; 128],
            maketic: 0,
            recvtic: 0,
            gametic: 0,
            skiptics: 0,
            ticdup: 0,
            new_sync: true,
            loop_interface: loop_interface_t {
                ProcessEvents: None,
                BuildTiccmd: None,
                RunTic: None,
                RunMenu: None,
            },
            local_playeringame: [false; 8],
            player_class: 0,
            lasttime: 0,
            frameon: 0,
            frameskip: [0; 4],
            oldnettics: 0,
            singletics: false,
            try_run_tics_oldentertics: 0,
        }
    }
}

pub use crate::src::d_ticcmd::ticcmd_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct net_connect_data_t {
    pub gamemode: i32,
    pub gamemission: i32,
    pub lowres_turn: i32,
    pub drone: i32,
    pub max_players: i32,
    pub is_freedoom: i32,
    pub wad_sha1sum: sha1_digest_t,
    pub deh_sha1sum: sha1_digest_t,
    pub player_class: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct net_gamesettings_t {
    pub ticdup: i32,
    pub extratics: i32,
    pub deathmatch: i32,
    pub episode: i32,
    pub nomonsters: i32,
    pub fast_monsters: i32,
    pub respawn_monsters: i32,
    pub map: i32,
    pub skill: i32,
    pub gameversion: i32,
    pub lowres_turn: i32,
    pub new_sync: i32,
    pub timelimit: i32,
    pub loadgame: i32,
    pub random: i32,
    pub num_players: i32,
    pub consoleplayer: i32,
    pub player_classes: [i32; 8],
}
pub type netgame_startup_callback_t = Option<unsafe fn(i32, i32) -> bool>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct loop_interface_t {
    pub ProcessEvents: Option<unsafe fn(&mut GameState) -> ()>,
    pub BuildTiccmd: Option<unsafe fn(&mut GameState, *mut ticcmd_t, i32) -> ()>,
    pub RunTic: Option<unsafe fn(&mut GameState, *mut ticcmd_t, *mut bool) -> ()>,
    pub RunMenu: Option<unsafe fn(&mut GameState) -> ()>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ticcmd_set_t {
    pub cmds: [ticcmd_t; 8],
    pub ingame: [bool; 8],
}
pub const NET_MAXPLAYERS: i32 = 8;
pub const BACKUPTICS: i32 = 128;
static localplayer: i32 = 0;
#[no_mangle]
pub static offsetms: fixed_t = 0;
fn GetAdjustedTime(state: &mut GameState) -> i32 {
    let mut time_ms: i32 = 0;
    time_ms = I_GetTimeMS(state);
    if state.d_loop.new_sync {
        time_ms += offsetms as i32 / FRACUNIT;
    }
    return time_ms * TICRATE / 1000 as i32;
}
unsafe fn BuildNewTic(state: &mut GameState) -> bool {
    let mut gameticdiv: i32 = 0;
    let mut cmd: ticcmd_t = ticcmd_t {
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
    gameticdiv = state.d_loop.gametic / state.d_loop.ticdup;
    I_StartTic(state);
    let process_events = state.d_loop.loop_interface
        .ProcessEvents
        .expect("non-null function pointer");
    process_events(state);
    let run_menu = state.d_loop.loop_interface
        .RunMenu
        .expect("non-null function pointer");
    run_menu(state);
    if drone {
        return false;
    }
    if state.d_loop.new_sync {
        if !net_client_connected && state.d_loop.maketic - gameticdiv > 2 as i32 {
            return false;
        }
        if state.d_loop.maketic - gameticdiv > 8 as i32 {
            return false;
        }
    } else if state.d_loop.maketic - gameticdiv >= 5 as i32 {
        return false;
    }
    memset(
        &raw mut cmd as *mut ::core::ffi::c_void,
        0 as i32,
        ::core::mem::size_of::<ticcmd_t>() as size_t,
    );
    let build_ticcmd = state.d_loop.loop_interface
        .BuildTiccmd
        .expect("non-null function pointer");
    let maketic = state.d_loop.maketic;
    build_ticcmd(state, &raw mut cmd, maketic);
    state.d_loop.ticdata[(state.d_loop.maketic % BACKUPTICS) as usize].cmds[localplayer as usize] =
        cmd;
    state.d_loop.ticdata[(state.d_loop.maketic % BACKUPTICS) as usize].ingame
        [localplayer as usize] = true;
    state.d_loop.maketic += 1;
    return true;
}
pub unsafe fn NetUpdate(state: &mut GameState) {
    let mut nowtime: i32 = 0;
    let mut newtics: i32 = 0;
    let mut i: i32 = 0;
    if state.d_loop.singletics {
        return;
    }
    nowtime = GetAdjustedTime(state) / state.d_loop.ticdup;
    newtics = nowtime - state.d_loop.lasttime;
    state.d_loop.lasttime = nowtime;
    if state.d_loop.skiptics <= newtics {
        newtics -= state.d_loop.skiptics;
        state.d_loop.skiptics = 0 as i32;
    } else {
        state.d_loop.skiptics -= newtics;
        newtics = 0 as i32;
    }
    i = 0 as i32;
    while i < newtics {
        if !BuildNewTic(state) {
            break;
        }
        i += 1;
    }
}
fn D_Disconnected() {
    if drone {
        I_Error("Disconnected from server in drone mode.");
    }
    println!("Disconnected from server.");
}
pub unsafe fn D_ReceiveTic(
    state: &mut DLoopState,
    mut ticcmds: *mut ticcmd_t,
    mut players_mask: *mut bool,
) {
    let mut i: i32 = 0;
    if ticcmds.is_null() && players_mask.is_null() {
        D_Disconnected();
        return;
    }
    i = 0 as i32;
    while i < NET_MAXPLAYERS {
        if !(!drone && i == localplayer) {
            state.ticdata[(state.recvtic % BACKUPTICS) as usize].cmds[i as usize] =
                *ticcmds.offset(i as isize);
            state.ticdata[(state.recvtic % BACKUPTICS) as usize].ingame[i as usize] =
                *players_mask.offset(i as isize);
        }
        i += 1;
    }
    state.recvtic += 1;
}
pub fn D_StartGameLoop(state: &mut GameState) {
    state.d_loop.lasttime = GetAdjustedTime(state) / state.d_loop.ticdup;
}
pub unsafe fn D_StartNetGame(
    state: &mut GameState,
    mut settings: *mut net_gamesettings_t,
) {
    (*settings).consoleplayer = 0 as i32;
    (*settings).num_players = 1 as i32;
    (*settings).player_classes[0 as usize] = state.d_loop.player_class;
    (*settings).new_sync = 0 as i32;
    (*settings).extratics = 1 as i32;
    (*settings).ticdup = 1 as i32;
    state.d_loop.ticdup = (*settings).ticdup;
    state.d_loop.new_sync = (*settings).new_sync != 0;
}
pub unsafe fn D_InitNetGame(
    state: &mut GameState,
    mut connect_data: *mut net_connect_data_t,
) -> bool {
    let mut result: bool = false;
    I_AtExit(&mut state.i_system, Some(D_QuitNetGame as unsafe fn(&mut GameState) -> ()), true);
    state.d_loop.player_class = (*connect_data).player_class;
    return result;
}
pub fn D_QuitNetGame(_state: &mut GameState) {}
fn GetLowTic(state: &mut GameState) -> i32 {
    let mut lowtic: i32 = 0;
    lowtic = state.d_loop.maketic;
    return lowtic;
}
fn OldNetSync(state: &mut GameState) {
    let mut i: u32 = 0;
    let mut keyplayer: i32 = -(1 as i32);
    state.d_loop.frameon += 1;
    i = 0 as u32;
    while i < NET_MAXPLAYERS as u32 {
        if state.d_loop.local_playeringame[i as usize] {
            keyplayer = i as i32;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    if keyplayer < 0 as i32 {
        return;
    }
    if !(localplayer == keyplayer) {
        if state.d_loop.maketic <= state.d_loop.recvtic {
            state.d_loop.lasttime -= 1;
        }
        state.d_loop.frameskip[(state.d_loop.frameon & 3 as i32) as usize] =
            (state.d_loop.oldnettics > state.d_loop.recvtic) as i32;
        state.d_loop.oldnettics = state.d_loop.maketic;
        if state.d_loop.frameskip[0 as usize] != 0
            && state.d_loop.frameskip[1 as usize] != 0
            && state.d_loop.frameskip[2 as usize] != 0
            && state.d_loop.frameskip[3 as usize] != 0
        {
            state.d_loop.skiptics = 1 as i32;
        }
    }
}
fn PlayersInGame(state: &mut GameState) -> bool {
    let mut result: bool = false;
    let mut i: u32 = 0;
    if net_client_connected {
        i = 0 as u32;
        while i < NET_MAXPLAYERS as u32 {
            result = result || state.d_loop.local_playeringame[i as usize];
            i = i.wrapping_add(1);
        }
    }
    if !drone {
        result = true;
    }
    return result;
}
unsafe fn TicdupSquash(mut set: *mut ticcmd_set_t) {
    let mut cmd: *mut ticcmd_t = ::core::ptr::null_mut::<ticcmd_t>();
    let mut i: u32 = 0;
    i = 0 as u32;
    while i < NET_MAXPLAYERS as u32 {
        cmd = (&raw mut (*set).cmds as *mut ticcmd_t).offset(i as isize) as *mut ticcmd_t;
        (*cmd).chatchar = 0 as byte;
        if (*cmd).buttons as i32 & BT_SPECIAL as i32 != 0 {
            (*cmd).buttons = 0 as byte;
        }
        i = i.wrapping_add(1);
    }
}
unsafe fn SinglePlayerClear(mut set: *mut ticcmd_set_t) {
    let mut i: u32 = 0;
    i = 0 as u32;
    while i < NET_MAXPLAYERS as u32 {
        if i != localplayer as u32 {
            (*set).ingame[i as usize] = false;
        }
        i = i.wrapping_add(1);
    }
}
pub unsafe fn TryRunTics(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut lowtic: i32 = 0;
    let mut entertic: i32 = 0;
    let mut realtics: i32 = 0;
    let mut availabletics: i32 = 0;
    let mut counts: i32 = 0;
    entertic = I_GetTime(state) / state.d_loop.ticdup;
    realtics = entertic - state.d_loop.try_run_tics_oldentertics;
    state.d_loop.try_run_tics_oldentertics = entertic;
    if state.d_loop.singletics {
        BuildNewTic(state);
    } else {
        NetUpdate(state);
    }
    lowtic = GetLowTic(state);
    availabletics = lowtic - state.d_loop.gametic / state.d_loop.ticdup;
    if state.d_loop.new_sync {
        counts = availabletics;
    } else {
        if realtics < availabletics - 1 as i32 {
            counts = realtics + 1 as i32;
        } else if realtics < availabletics {
            counts = realtics;
        } else {
            counts = availabletics;
        }
        if counts < 1 as i32 {
            counts = 1 as i32;
        }
        if net_client_connected {
            OldNetSync(state);
        }
    }
    if counts < 1 as i32 {
        counts = 1 as i32;
    }
    while !PlayersInGame(state) || lowtic < state.d_loop.gametic / state.d_loop.ticdup + counts {
        NetUpdate(state);
        lowtic = GetLowTic(state);
        if lowtic < state.d_loop.gametic / state.d_loop.ticdup {
            I_Error("TryRunTics: lowtic < gametic");
        }
        if I_GetTime(state) / state.d_loop.ticdup - entertic > 0 as i32 {
            return;
        }
        I_Sleep(state, 1 as i32);
    }
    loop {
        let fresh0 = counts;
        counts = counts - 1;
        if !(fresh0 != 0) {
            break;
        }
        let mut set: *mut ticcmd_set_t = ::core::ptr::null_mut::<ticcmd_set_t>();
        if !PlayersInGame(state) {
            return;
        }
        set = (&raw mut state.d_loop.ticdata as *mut ticcmd_set_t)
            .offset((state.d_loop.gametic / state.d_loop.ticdup % BACKUPTICS) as isize)
            as *mut ticcmd_set_t;
        if !net_client_connected {
            SinglePlayerClear(set);
        }
        i = 0 as i32;
        while i < state.d_loop.ticdup {
            if state.d_loop.gametic / state.d_loop.ticdup > lowtic {
                I_Error("gametic>lowtic");
            }
            memcpy(
                &raw mut state.d_loop.local_playeringame as *mut bool
                    as *mut ::core::ffi::c_void,
                &raw mut (*set).ingame as *mut bool as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[bool; 8]>() as size_t,
            );
            let run_tic = state.d_loop.loop_interface
                .RunTic
                .expect("non-null function pointer");
            run_tic(
                state,
                &raw mut (*set).cmds as *mut ticcmd_t,
                &raw mut (*set).ingame as *mut bool,
            );
            state.d_loop.gametic += 1;
            TicdupSquash(set);
            i += 1;
        }
        NetUpdate(state);
    }
}
pub fn D_RegisterLoopCallbacks(state: &mut GameState, i: loop_interface_t) {
    state.d_loop.loop_interface = i;
}
