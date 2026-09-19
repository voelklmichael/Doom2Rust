use crate::d_loop::init_net_game;
use crate::d_loop::register_loop_callbacks;
use crate::d_loop::start_net_game;
use crate::d_loop::{LoopInterface, NetConnectData, NetGameSettings};
use crate::d_main::do_advance_demo;
use crate::d_mode::skill_from_raw;
use crate::d_ticcmd::TicCmd;
use crate::g_game::check_demo_status;
use crate::g_game::g_ticker;
use crate::m_argv::parm_exists;
use crate::w_checksum::checksum;
use crate::w_wad::check_num_for_name;

use crate::d_main::d_process_events;
use crate::doomdef::MAXPLAYERS;
use crate::g_game::g_build_ticcmd;
use crate::game_state::GameState;
use crate::m_menu::m_ticker;
use crate::tables::ANG270;
use crate::tables::ANG90;
fn player_quit_game(state: &mut GameState, player_num: u32) {
    state.g_game.playeringame[player_num as usize] = false;
    state.g_game.players[state.g_game.consoleplayer as usize].message =
        Some(format!("Player {} left the game", player_num + 1));
    if state.g_game.demorecording {
        check_demo_status(state);
    }
}
fn run_tic(state: &mut GameState, cmds: &[TicCmd], ingame: &[bool]) {
    let mut i: u32;
    i = 0;
    while i < MAXPLAYERS as u32 {
        if !state.g_game.demoplayback
            && state.g_game.playeringame[i as usize]
            && !ingame[i as usize]
        {
            player_quit_game(state, i);
        }
        i = i.wrapping_add(1);
    }
    if state.d_main.advancedemo {
        do_advance_demo(state);
    }
    g_ticker(state, cmds);
}
const DOOM_LOOP_INTERFACE: LoopInterface = LoopInterface {
    process_events: Some(d_process_events),
    build_ticcmd: Some(g_build_ticcmd),
    run_tic: Some(run_tic),
    run_menu: Some(m_ticker),
};
fn load_game_settings(state: &mut GameState, settings: &NetGameSettings) {
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
fn save_game_settings(state: &GameState, settings: &mut NetGameSettings) {
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
        (parm_exists(&state.m_argv, "-record") && !parm_exists(&state.m_argv, "-longtics")) as i32;
}
fn init_connect_data(state: &mut GameState, connect_data: &mut NetConnectData) {
    connect_data.max_players = MAXPLAYERS;
    connect_data.drone = false;
    if parm_exists(&state.m_argv, "-left") {
        state.r_main.viewangleoffset = ANG90;
        connect_data.drone = true;
    }
    if parm_exists(&state.m_argv, "-right") {
        state.r_main.viewangleoffset = ANG270 as i32;
        connect_data.drone = true;
    }
    connect_data.gamemode = state.doomstat.gamemode as i32;
    connect_data.gamemission = state.doomstat.gamemission as i32;
    connect_data.lowres_turn =
        (parm_exists(&state.m_argv, "-record") && !parm_exists(&state.m_argv, "-longtics")) as i32;
    connect_data.wad_sha1sum = checksum(&mut state.w_checksum, &state.w_wad);
    connect_data.is_freedoom = check_num_for_name(&state.w_wad, "FREEDOOM").is_some();
}
pub fn connect_net_game(state: &mut GameState) {
    let mut connect_data: NetConnectData = NetConnectData {
        gamemode: 0,
        gamemission: 0,
        lowres_turn: 0,
        drone: false,
        max_players: 0,
        is_freedoom: false,
        wad_sha1sum: [0; 20],
        player_class: 0,
    };
    init_connect_data(state, &mut connect_data);
    state.g_game.netgame = init_net_game(&mut state.d_loop, &mut state.i_system, &connect_data);
    if parm_exists(&state.m_argv, "-solo-net") {
        state.g_game.netgame = true;
    }
}
pub fn check_net_game(state: &mut GameState) {
    let mut settings: NetGameSettings = NetGameSettings {
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
    register_loop_callbacks(&mut state.d_loop, DOOM_LOOP_INTERFACE);
    save_game_settings(state, &mut settings);
    start_net_game(&mut state.d_loop, &mut settings);
    load_game_settings(state, &settings);
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
        if state.g_game.timelimit == 20 && parm_exists(&state.m_argv, "-avg") {
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
