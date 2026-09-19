use crate::d_loop::D_InitNetGame;
use crate::d_loop::D_RegisterLoopCallbacks;
use crate::d_loop::D_StartNetGame;
use crate::d_loop::{loop_interface_t, net_connect_data_t, net_gamesettings_t};
use crate::d_main::D_DoAdvanceDemo;
use crate::d_mode::skill_from_raw;
use crate::d_ticcmd::ticcmd_t;
use crate::g_game::G_CheckDemoStatus;
use crate::g_game::G_Ticker;
use crate::m_argv::M_CheckParm;
use crate::w_checksum::W_Checksum;
use crate::w_wad::W_CheckNumForName;

use crate::d_main::D_ProcessEvents;
use crate::doomdef::MAXPLAYERS;
use crate::g_game::G_BuildTiccmd;
use crate::game_state::GameState;
use crate::m_menu::M_Ticker;
use crate::tables::ANG270;
use crate::tables::ANG90;
fn PlayerQuitGame(state: &mut GameState, player_num: u32) {
    state.g_game.playeringame[player_num as usize] = false;
    state.g_game.players[state.g_game.consoleplayer as usize].message =
        Some(format!("Player {} left the game", player_num + 1));
    if state.g_game.demorecording {
        G_CheckDemoStatus(state);
    }
}
fn RunTic(state: &mut GameState, cmds: &[ticcmd_t], ingame: &[bool]) {
    let mut i: u32;
    i = 0;
    while i < MAXPLAYERS as u32 {
        if !state.g_game.demoplayback
            && state.g_game.playeringame[i as usize]
            && !ingame[i as usize]
        {
            PlayerQuitGame(state, i);
        }
        i = i.wrapping_add(1);
    }
    if state.d_main.advancedemo {
        D_DoAdvanceDemo(state);
    }
    G_Ticker(state, cmds);
}
const DOOM_LOOP_INTERFACE: loop_interface_t = loop_interface_t {
    ProcessEvents: Some(D_ProcessEvents),
    BuildTiccmd: Some(G_BuildTiccmd),
    RunTic: Some(RunTic),
    RunMenu: Some(M_Ticker),
};
fn LoadGameSettings(state: &mut GameState, settings: &mut net_gamesettings_t) {
    let mut i: u32;
    state.g_game.deathmatch = settings.deathmatch;
    state.d_main.startepisode = settings.episode;
    state.d_main.startmap = settings.map;
    state.d_main.startskill = skill_from_raw(settings.skill);
    state.d_main.startloadgame = settings.loadgame;
    state.g_game.lowres_turn = settings.lowres_turn != 0;
    state.d_main.nomonsters = settings.nomonsters != 0;
    state.d_main.fastparm = settings.fast_monsters != 0;
    state.d_main.respawnparm = settings.respawn_monsters != 0;
    state.g_game.timelimit = settings.timelimit;
    state.g_game.consoleplayer = settings.consoleplayer;
    if state.g_game.lowres_turn {
        doom_println!(state.platform,
            "NOTE: Turning resolution is reduced; this is probably because there is a client recording a Vanilla demo."
        );
    }
    i = 0;
    while i < MAXPLAYERS as u32 {
        state.g_game.playeringame[i as usize] = i < settings.num_players as u32;
        i = i.wrapping_add(1);
    }
}
fn SaveGameSettings(state: &mut GameState, settings: &mut net_gamesettings_t) {
    settings.deathmatch = state.g_game.deathmatch;
    settings.episode = state.d_main.startepisode;
    settings.map = state.d_main.startmap;
    settings.skill = state.d_main.startskill as i32;
    settings.loadgame = state.d_main.startloadgame;
    settings.gameversion = state.doomstat.gameversion as i32;
    settings.nomonsters = state.d_main.nomonsters as i32;
    settings.fast_monsters = state.d_main.fastparm as i32;
    settings.respawn_monsters = state.d_main.respawnparm as i32;
    settings.timelimit = state.g_game.timelimit;
    settings.lowres_turn =
        (M_CheckParm(state, "-record") > 0 && M_CheckParm(state, "-longtics") == 0) as i32;
}
fn InitConnectData(state: &mut GameState, connect_data: &mut net_connect_data_t) {
    connect_data.max_players = MAXPLAYERS;
    connect_data.drone = false;
    if M_CheckParm(state, "-left") > 0 {
        state.r_main.viewangleoffset = ANG90;
        connect_data.drone = true;
    }
    if M_CheckParm(state, "-right") > 0 {
        state.r_main.viewangleoffset = ANG270 as i32;
        connect_data.drone = true;
    }
    connect_data.gamemode = state.doomstat.gamemode as i32;
    connect_data.gamemission = state.doomstat.gamemission as i32;
    connect_data.lowres_turn =
        (M_CheckParm(state, "-record") > 0 && M_CheckParm(state, "-longtics") == 0) as i32;
    connect_data.wad_sha1sum = W_Checksum(state);
    connect_data.is_freedoom = W_CheckNumForName(&mut state.w_wad, "FREEDOOM") >= 0;
}
pub fn D_ConnectNetGame(state: &mut GameState) {
    let mut connect_data: net_connect_data_t = net_connect_data_t {
        gamemode: 0,
        gamemission: 0,
        lowres_turn: 0,
        drone: false,
        max_players: 0,
        is_freedoom: false,
        wad_sha1sum: [0; 20],
        player_class: 0,
    };
    InitConnectData(state, &mut connect_data);
    state.g_game.netgame = D_InitNetGame(state, &mut connect_data);
    if M_CheckParm(state, "-solo-net") > 0 {
        state.g_game.netgame = true;
    }
}
pub fn D_CheckNetGame(state: &mut GameState) {
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
        num_players: 0,
        consoleplayer: 0,
        player_classes: [0; 8],
    };
    if state.g_game.netgame {
        state.d_main.autostart = true;
    }
    D_RegisterLoopCallbacks(state, DOOM_LOOP_INTERFACE);
    SaveGameSettings(state, &mut settings);
    D_StartNetGame(state, &mut settings);
    LoadGameSettings(state, &mut settings);
    doom_println!(
        state.platform,
        "startskill {}  deathmatch: {}  startmap: {}  startepisode: {}",
        state.d_main.startskill as i32,
        state.g_game.deathmatch,
        state.d_main.startmap,
        state.d_main.startepisode,
    );
    doom_println!(
        state.platform,
        "player {} of {} ({} nodes)",
        state.g_game.consoleplayer + 1,
        settings.num_players,
        settings.num_players,
    );
    if state.g_game.timelimit > 0 && state.g_game.deathmatch != 0 {
        if state.g_game.timelimit == 20 && M_CheckParm(state, "-avg") != 0 {
            doom_println!(
                state.platform,
                "Austin Virtual Gaming: Levels will end after 20 minutes"
            );
        } else {
            doom_print!(
                state.platform,
                "Levels will end after {} minute",
                state.g_game.timelimit
            );
            if state.g_game.timelimit > 1 {
                doom_print!(state.platform, "s");
            }
            doom_println!(state.platform, ".");
        }
    }
}
