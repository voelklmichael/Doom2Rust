use crate::src::d_loop::D_InitNetGame;
use crate::src::d_loop::D_RegisterLoopCallbacks;
use crate::src::d_loop::D_StartNetGame;
use crate::src::d_loop::{loop_interface_t, net_connect_data_t, net_gamesettings_t};
use crate::src::d_main::D_DoAdvanceDemo;
use crate::src::d_mode::skill_t;
use crate::src::d_player::player_t;
use crate::src::d_ticcmd::ticcmd_t;
use crate::src::doomdef::boolean;
use crate::src::g_game::G_CheckDemoStatus;
use crate::src::g_game::G_Ticker;
use crate::src::m_argv::M_CheckParm;
use crate::src::stdint_types::byte;
use crate::src::w_checksum::W_Checksum;
use crate::src::w_wad::W_CheckNumForName;

use crate::src::d_main::D_ProcessEvents;
use crate::src::doomdef::false_0;
use crate::src::doomdef::true_0;
use crate::src::doomdef::MAXPLAYERS;
use crate::src::g_game::G_BuildTiccmd;
use crate::src::game_state::GameState;
use crate::src::m_menu::M_Ticker;
use crate::src::tables::ANG270;
use crate::src::tables::ANG90;
pub struct DNetState {
    pub netcmds: *mut ticcmd_t,
}

impl DNetState {
    pub const fn new() -> Self {
        DNetState {
            netcmds: ::core::ptr::null::<ticcmd_t>() as *mut ticcmd_t,
        }
    }
}

unsafe fn PlayerQuitGame(state: &mut GameState, mut player: *mut player_t) {
    let mut player_num: u32 = 0;
    player_num = player.offset_from(&raw mut state.g_game.players as *mut player_t) as i64 as u32;
    state.g_game.playeringame[player_num as usize] = false_0 as boolean;
    state.g_game.players[state.g_game.consoleplayer as usize].message =
        Some(format!("Player {} left the game", player_num + 1));
    if state.g_game.demorecording {
        G_CheckDemoStatus(state);
    }
}
unsafe fn RunTic(state: &mut GameState, mut cmds: *mut ticcmd_t, mut ingame: *mut boolean) {
    let mut i: u32 = 0;
    i = 0 as u32;
    while i < MAXPLAYERS as u32 {
        if !state.g_game.demoplayback
            && state.g_game.playeringame[i as usize] != 0
            && *ingame.offset(i as isize) == 0
        {
            let quitter = (&raw mut state.g_game.players as *mut player_t).offset(i as isize)
                as *mut player_t;
            PlayerQuitGame(state, quitter);
        }
        i = i.wrapping_add(1);
    }
    state.d_net.netcmds = cmds;
    if state.d_main.advancedemo {
        D_DoAdvanceDemo(state);
    }
    G_Ticker(state);
}
const DOOM_LOOP_INTERFACE: loop_interface_t = loop_interface_t {
    ProcessEvents: Some(D_ProcessEvents as unsafe fn(&mut GameState) -> ()),
    BuildTiccmd: Some(G_BuildTiccmd as unsafe fn(&mut GameState, *mut ticcmd_t, i32) -> ()),
    RunTic: Some(RunTic as unsafe fn(&mut GameState, *mut ticcmd_t, *mut boolean) -> ()),
    RunMenu: Some(M_Ticker as unsafe fn(&mut GameState) -> ()),
};
unsafe fn LoadGameSettings(state: &mut GameState, mut settings: *mut net_gamesettings_t) {
    let mut i: u32 = 0;
    state.g_game.deathmatch = (*settings).deathmatch;
    state.d_main.startepisode = (*settings).episode;
    state.d_main.startmap = (*settings).map;
    state.d_main.startskill = (*settings).skill as skill_t;
    state.d_main.startloadgame = (*settings).loadgame;
    state.g_game.lowres_turn = (*settings).lowres_turn != 0;
    state.d_main.nomonsters = (*settings).nomonsters != 0;
    state.d_main.fastparm = (*settings).fast_monsters != 0;
    state.d_main.respawnparm = (*settings).respawn_monsters != 0;
    state.g_game.timelimit = (*settings).timelimit;
    state.g_game.consoleplayer = (*settings).consoleplayer;
    if state.g_game.lowres_turn {
        println!(
            "NOTE: Turning resolution is reduced; this is probably because there is a client recording a Vanilla demo."
        );
    }
    i = 0 as u32;
    while i < MAXPLAYERS as u32 {
        state.g_game.playeringame[i as usize] =
            (i < (*settings).num_players as u32) as i32 as boolean;
        i = i.wrapping_add(1);
    }
}
unsafe fn SaveGameSettings(state: &mut GameState, mut settings: *mut net_gamesettings_t) {
    (*settings).deathmatch = state.g_game.deathmatch;
    (*settings).episode = state.d_main.startepisode;
    (*settings).map = state.d_main.startmap;
    (*settings).skill = state.d_main.startskill as i32;
    (*settings).loadgame = state.d_main.startloadgame;
    (*settings).gameversion = state.doomstat.gameversion as i32;
    (*settings).nomonsters = state.d_main.nomonsters as i32;
    (*settings).fast_monsters = state.d_main.fastparm as i32;
    (*settings).respawn_monsters = state.d_main.respawnparm as i32;
    (*settings).timelimit = state.g_game.timelimit;
    (*settings).lowres_turn = (M_CheckParm(state, "-record") > 0 as i32
        && M_CheckParm(state, "-longtics") == 0 as i32) as i32;
}
unsafe fn InitConnectData(state: &mut GameState, mut connect_data: *mut net_connect_data_t) {
    (*connect_data).max_players = MAXPLAYERS;
    (*connect_data).drone = false_0;
    if M_CheckParm(state, "-left") > 0 as i32 {
        state.r_main.viewangleoffset = ANG90;
        (*connect_data).drone = true_0;
    }
    if M_CheckParm(state, "-right") > 0 as i32 {
        state.r_main.viewangleoffset = ANG270 as i32;
        (*connect_data).drone = true_0;
    }
    (*connect_data).gamemode = state.doomstat.gamemode as i32;
    (*connect_data).gamemission = state.doomstat.gamemission as i32;
    (*connect_data).lowres_turn = (M_CheckParm(state, "-record") > 0 as i32
        && M_CheckParm(state, "-longtics") == 0 as i32) as i32;
    W_Checksum(state, &raw mut (*connect_data).wad_sha1sum as *mut byte);
    (*connect_data).is_freedoom = (W_CheckNumForName(&mut state.w_wad, "FREEDOOM") >= 0 as i32) as i32;
}
pub unsafe fn D_ConnectNetGame(state: &mut GameState) {
    let mut connect_data: net_connect_data_t = net_connect_data_t {
        gamemode: 0,
        gamemission: 0,
        lowres_turn: 0,
        drone: 0,
        max_players: 0,
        is_freedoom: 0,
        wad_sha1sum: [0; 20],
        deh_sha1sum: [0; 20],
        player_class: 0,
    };
    InitConnectData(state, &raw mut connect_data);
    state.g_game.netgame = D_InitNetGame(state, &raw mut connect_data);
    if M_CheckParm(state, "-solo-net") > 0 as i32 {
        state.g_game.netgame = true;
    }
}
pub unsafe fn D_CheckNetGame(state: &mut GameState) {
    let mut settings: net_gamesettings_t = net_gamesettings_t {
        ticdup: 0,
        extratics: 0,
        deathmatch: 0,
        episode: 0,
        nomonsters: 0,
        fast_monsters: 0,
        respawn_monsters: 0,
        map: 0,
        skill: 0,
        gameversion: 0,
        lowres_turn: 0,
        new_sync: 0,
        timelimit: 0,
        loadgame: 0,
        random: 0,
        num_players: 0,
        consoleplayer: 0,
        player_classes: [0; 8],
    };
    if state.g_game.netgame {
        state.d_main.autostart = true;
    }
    D_RegisterLoopCallbacks(state, DOOM_LOOP_INTERFACE);
    SaveGameSettings(state, &raw mut settings);
    D_StartNetGame(state, &raw mut settings);
    LoadGameSettings(state, &raw mut settings);
    println!(
        "startskill {}  deathmatch: {}  startmap: {}  startepisode: {}",
        state.d_main.startskill as i32,
        state.g_game.deathmatch,
        state.d_main.startmap,
        state.d_main.startepisode,
    );
    println!(
        "player {} of {} ({} nodes)",
        state.g_game.consoleplayer + 1 as i32,
        settings.num_players,
        settings.num_players,
    );
    if state.g_game.timelimit > 0 as i32 && state.g_game.deathmatch != 0 {
        if state.g_game.timelimit == 20 as i32 && M_CheckParm(state, "-avg") != 0 {
            println!("Austin Virtual Gaming: Levels will end after 20 minutes");
        } else {
            print!("Levels will end after {} minute", state.g_game.timelimit);
            if state.g_game.timelimit > 1 as i32 {
                print!("s");
            }
            println!(".");
        }
    }
}
