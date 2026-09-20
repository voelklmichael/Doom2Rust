use crate::d_loop::init_net_game;
use crate::d_loop::register_loop_callbacks;
use crate::d_loop::start_net_game;
use crate::d_loop::{LoopInterface, NetConnectData, NetGameSettings};
use crate::d_main::do_advance_demo;
use crate::d_main::DMainState;
use crate::d_mode::skill_from_raw;
use crate::d_player::PlayerId;
use crate::d_ticcmd::TicCmd;
use crate::g_game::check_demo_status;
use crate::g_game::g_ticker;
use crate::g_game::GGameState;
use crate::game_state::Game;
use crate::platform::DoomPlatform;
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
    state.game.g_game.playeringame[player_num as usize] = false;
    state.game.g_game.players[state.game.g_game.consoleplayer].message =
        Some(format!("Player {} left the game", player_num + 1));
    if state.game.g_game.demorecording {
        check_demo_status(state);
    }
}
fn run_tic(state: &mut GameState, cmds: &[TicCmd], ingame: &[bool]) {
    for i in 0..MAXPLAYERS as u32 {
        if !state.game.g_game.demoplayback
            && state.game.g_game.playeringame[i as usize]
            && !ingame[i as usize]
        {
            player_quit_game(state, i);
        }
    }
    if state.game.d_main.advancedemo {
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
fn load_game_settings(
    d_main: &mut DMainState,
    g_game: &mut GGameState,
    platform: &mut dyn DoomPlatform,
    settings: &NetGameSettings,
) {
    g_game.deathmatch = settings.deathmatch;
    d_main.startepisode = settings.episode;
    d_main.startmap = settings.map;
    d_main.startskill = skill_from_raw(settings.skill);
    d_main.startloadgame = settings.loadgame;
    g_game.lowres_turn = settings.lowres_turn != 0;
    d_main.nomonsters = settings.nomonsters != 0;
    d_main.fastparm = settings.fast_monsters != 0;
    d_main.respawnparm = settings.respawn_monsters != 0;
    g_game.timelimit = settings.timelimit;
    g_game.consoleplayer = PlayerId(settings.consoleplayer as u8);
    if g_game.lowres_turn {
        doom_println!(platform,
            "NOTE: Turning resolution is reduced; this is probably because there is a client recording a Vanilla demo."
        );
    }
    for i in 0..MAXPLAYERS as u32 {
        g_game.playeringame[i as usize] = i < settings.num_players as u32;
    }
}
fn save_game_settings(game: &Game, settings: &mut NetGameSettings) {
    settings.deathmatch = game.g_game.deathmatch;
    settings.episode = game.d_main.startepisode;
    settings.map = game.d_main.startmap;
    settings.skill = game.d_main.startskill as i32;
    settings.loadgame = game.d_main.startloadgame;
    settings.gameversion = game.doomstat.gameversion as i32;
    settings.nomonsters = i32::from(game.d_main.nomonsters);
    settings.fast_monsters = i32::from(game.d_main.fastparm);
    settings.respawn_monsters = i32::from(game.d_main.respawnparm);
    settings.timelimit = game.g_game.timelimit;
    settings.lowres_turn = i32::from(game.options.record && !game.options.longtics);
}
fn init_connect_data(state: &mut GameState, connect_data: &mut NetConnectData) {
    connect_data.max_players = MAXPLAYERS;
    connect_data.drone = false;
    if state.game.options.left {
        state.render.r_main.viewangleoffset = ANG90;
        connect_data.drone = true;
    }
    if state.game.options.right {
        state.render.r_main.viewangleoffset = ANG270 as i32;
        connect_data.drone = true;
    }
    connect_data.gamemode = state.game.doomstat.gamemode as i32;
    connect_data.gamemission = state.game.doomstat.gamemission as i32;
    connect_data.lowres_turn = i32::from(state.game.options.record && !state.game.options.longtics);
    connect_data.wad_sha1sum = checksum(&mut state.assets.w_checksum, &state.assets.w_wad);
    connect_data.is_freedoom = check_num_for_name(&state.assets.w_wad, "FREEDOOM").is_some();
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
    state.game.g_game.netgame = init_net_game(
        &mut state.game.d_loop,
        &mut state.io.i_system,
        &connect_data,
    );
    if state.game.options.solo_net {
        state.game.g_game.netgame = true;
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
    if state.game.g_game.netgame {
        state.game.d_main.autostart = true;
    }
    register_loop_callbacks(&mut state.game.d_loop, DOOM_LOOP_INTERFACE);
    save_game_settings(&state.game, &mut settings);
    start_net_game(&mut state.game.d_loop, &mut settings);
    load_game_settings(
        &mut state.game.d_main,
        &mut state.game.g_game,
        &mut *state.io.platform,
        &settings,
    );
    doom_println!(
        state.io.platform,
        "startskill {}  deathmatch: {}  startmap: {}  startepisode: {}",
        state.game.d_main.startskill as i32,
        state.game.g_game.deathmatch,
        state.game.d_main.startmap,
        state.game.d_main.startepisode,
    );
    doom_println!(
        state.io.platform,
        "player {} of {} ({} nodes)",
        state.game.g_game.consoleplayer.as_i32() + 1,
        settings.num_players,
        settings.num_players,
    );
    if state.game.g_game.timelimit > 0 && state.game.g_game.deathmatch != 0 {
        if state.game.g_game.timelimit == 20 && state.game.options.avg {
            doom_println!(
                state.io.platform,
                "Austin Virtual Gaming: Levels will end after 20 minutes"
            );
        } else {
            doom_print!(
                state.io.platform,
                "Levels will end after {} minute",
                state.game.g_game.timelimit
            );
            if state.game.g_game.timelimit > 1 {
                doom_print!(state.io.platform, "s");
            }
            doom_println!(state.io.platform, ".");
        }
    }
}
