use crate::d_ticcmd::BT_SPECIAL;
use crate::doomdef::TICRATE;
use crate::dummy::DRONE;
use crate::dummy::NET_CLIENT_CONNECTED;
use crate::game_state::GameState;
use crate::i_system::at_exit;
use crate::i_system::error;
use crate::i_system::ISystemState;
use crate::i_timer::get_time;
use crate::i_timer::get_time_ms;
use crate::i_timer::sleep;
use crate::i_timer::ITimerState;
use crate::i_video::start_tic;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACUNIT;
use crate::platform::DoomPlatform;
use crate::w_checksum::Sha1Digest;

pub struct DLoopState {
    pub ticdata: [TicCmdSet; 128],
    pub maketic: i32,
    pub recvtic: i32,
    pub gametic: i32,
    pub skiptics: i32,
    pub ticdup: i32,
    pub new_sync: bool,
    pub loop_interface: LoopInterface,
    pub local_playeringame: [bool; 8],
    pub player_class: i32,
    pub lasttime: i32,
    pub frameon: i32,
    pub frameskip: [i32; 4],
    pub oldnettics: i32,
    pub singletics: bool,
    pub try_run_tics_oldentertics: i32,
}

impl Default for DLoopState {
    fn default() -> Self {
        Self::new()
    }
}

impl DLoopState {
    pub fn new() -> Self {
        Self {
            ticdata: [TicCmdSet {
                cmds: [TicCmd {
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
            loop_interface: LoopInterface {
                process_events: None,
                build_ticcmd: None,
                run_tic: None,
                run_menu: None,
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

pub use crate::d_ticcmd::TicCmd;
#[derive(Copy, Clone)]
pub struct NetConnectData {
    pub gamemode: i32,
    pub gamemission: i32,
    pub lowres_turn: i32,
    pub drone: bool,
    pub max_players: i32,
    pub is_freedoom: bool,
    pub wad_sha1sum: Sha1Digest,
    pub player_class: i32,
}
#[derive(Copy, Clone)]
pub struct NetGameSettings {
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
    pub num_players: i32,
    pub consoleplayer: i32,
    pub player_classes: [i32; 8],
}
type RunTicFn = fn(&mut GameState, &[TicCmd], &[bool]);
#[derive(Copy, Clone)]
pub struct LoopInterface {
    pub process_events: Option<fn(&mut GameState)>,
    pub build_ticcmd: Option<fn(&mut GameState, &mut TicCmd, i32)>,
    pub run_tic: Option<RunTicFn>,
    pub run_menu: Option<fn(&mut GameState)>,
}
#[derive(Copy, Clone)]
pub struct TicCmdSet {
    pub cmds: [TicCmd; 8],
    pub ingame: [bool; 8],
}
pub const NET_MAXPLAYERS: i32 = 8;
pub const BACKUPTICS: i32 = 128;
static LOCALPLAYER: i32 = 0;
pub static OFFSETMS: Fixed = 0;
fn get_adjusted_time(
    d_loop: &DLoopState,
    i_timer: &mut ITimerState,
    platform: &mut dyn DoomPlatform,
) -> i32 {
    let mut time_ms: i32 = get_time_ms(i_timer, &mut *platform);
    if d_loop.new_sync {
        time_ms += OFFSETMS / FRACUNIT;
    }
    time_ms * TICRATE / 1000
}
fn build_new_tic(state: &mut GameState) -> bool {
    let mut cmd: TicCmd = TicCmd {
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
    let gameticdiv: i32 = state.game.d_loop.gametic / state.game.d_loop.ticdup;
    start_tic(
        &mut state.game.d_event,
        &mut state.io.i_input,
        &mut *state.io.platform,
    );
    let process_events = state
        .game
        .d_loop
        .loop_interface
        .process_events
        .expect("non-null function pointer");
    process_events(state);
    let run_menu = state
        .game
        .d_loop
        .loop_interface
        .run_menu
        .expect("non-null function pointer");
    run_menu(state);
    if DRONE {
        return false;
    }
    if state.game.d_loop.new_sync {
        if !NET_CLIENT_CONNECTED && state.game.d_loop.maketic - gameticdiv > 2 {
            return false;
        }
        if state.game.d_loop.maketic - gameticdiv > 8 {
            return false;
        }
    } else if state.game.d_loop.maketic - gameticdiv >= 5 {
        return false;
    }
    let build_ticcmd = state
        .game
        .d_loop
        .loop_interface
        .build_ticcmd
        .expect("non-null function pointer");
    let maketic = state.game.d_loop.maketic;
    build_ticcmd(state, &mut cmd, maketic);
    state.game.d_loop.ticdata[(state.game.d_loop.maketic % BACKUPTICS) as usize].cmds
        [LOCALPLAYER as usize] = cmd;
    state.game.d_loop.ticdata[(state.game.d_loop.maketic % BACKUPTICS) as usize].ingame
        [LOCALPLAYER as usize] = true;
    state.game.d_loop.maketic += 1;
    true
}
pub fn net_update(state: &mut GameState) {
    if state.game.d_loop.singletics {
        return;
    }
    let nowtime: i32 = get_adjusted_time(
        &state.game.d_loop,
        &mut state.io.i_timer,
        &mut *state.io.platform,
    ) / state.game.d_loop.ticdup;
    let mut newtics: i32 = nowtime - state.game.d_loop.lasttime;
    state.game.d_loop.lasttime = nowtime;
    if state.game.d_loop.skiptics <= newtics {
        newtics -= state.game.d_loop.skiptics;
        state.game.d_loop.skiptics = 0;
    } else {
        state.game.d_loop.skiptics -= newtics;
        newtics = 0;
    }
    for _ in 0..newtics {
        if !build_new_tic(state) {
            break;
        }
    }
}
pub fn start_game_loop(
    d_loop: &mut DLoopState,
    i_timer: &mut ITimerState,
    platform: &mut dyn DoomPlatform,
) {
    d_loop.lasttime = get_adjusted_time(d_loop, i_timer, &mut *platform) / d_loop.ticdup;
}
pub fn start_net_game(d_loop: &mut DLoopState, settings: &mut NetGameSettings) {
    settings.consoleplayer = 0;
    settings.num_players = 1;
    settings.player_classes[0] = d_loop.player_class;
    settings.new_sync = 0;
    settings.extratics = 1;
    settings.ticdup = 1;
    d_loop.ticdup = settings.ticdup;
    d_loop.new_sync = settings.new_sync != 0;
}
pub fn init_net_game(
    d_loop: &mut DLoopState,
    i_system: &mut ISystemState,
    connect_data: &NetConnectData,
) -> bool {
    let result: bool = false;
    at_exit(
        i_system,
        Some(quit_net_game as fn(&mut GameState) -> ()),
        true,
    );
    d_loop.player_class = connect_data.player_class;
    result
}
pub fn quit_net_game(_state: &mut GameState) {}
fn get_low_tic(d_loop: &DLoopState) -> i32 {
    let lowtic: i32 = d_loop.maketic;
    lowtic
}
fn old_net_sync(d_loop: &mut DLoopState) {
    let mut keyplayer: i32 = -1;
    d_loop.frameon += 1;
    let mut i: u32 = 0;
    while i < NET_MAXPLAYERS as u32 {
        if d_loop.local_playeringame[i as usize] {
            keyplayer = i as i32;
            break;
        }
        i = i.wrapping_add(1);
    }
    if keyplayer < 0 {
        return;
    }
    if LOCALPLAYER != keyplayer {
        if d_loop.maketic <= d_loop.recvtic {
            d_loop.lasttime -= 1;
        }
        d_loop.frameskip[(d_loop.frameon & 3) as usize] =
            (d_loop.oldnettics > d_loop.recvtic) as i32;
        d_loop.oldnettics = d_loop.maketic;
        if d_loop.frameskip[0] != 0
            && d_loop.frameskip[1] != 0
            && d_loop.frameskip[2] != 0
            && d_loop.frameskip[3] != 0
        {
            d_loop.skiptics = 1;
        }
    }
}
fn players_in_game(d_loop: &DLoopState) -> bool {
    let mut result: bool = false;
    let mut i: u32;
    if NET_CLIENT_CONNECTED {
        i = 0;
        while i < NET_MAXPLAYERS as u32 {
            result = result || d_loop.local_playeringame[i as usize];
            i = i.wrapping_add(1);
        }
    }
    if !DRONE {
        result = true;
    }
    result
}
fn ticdup_squash(set: &mut TicCmdSet) {
    let mut i: u32 = 0;
    while i < NET_MAXPLAYERS as u32 {
        let cmd = &mut set.cmds[i as usize];
        cmd.chatchar = 0_u8;
        if cmd.buttons as i32 & BT_SPECIAL != 0 {
            cmd.buttons = 0_u8;
        }
        i = i.wrapping_add(1);
    }
}
fn single_player_clear(set: &mut TicCmdSet) {
    let mut i: u32 = 0;
    while i < NET_MAXPLAYERS as u32 {
        if i != LOCALPLAYER as u32 {
            set.ingame[i as usize] = false;
        }
        i = i.wrapping_add(1);
    }
}
pub fn try_run_tics(state: &mut GameState) {
    let mut counts: i32;
    let entertic: i32 =
        get_time(&mut state.io.i_timer, &mut *state.io.platform) / state.game.d_loop.ticdup;
    let realtics: i32 = entertic - state.game.d_loop.try_run_tics_oldentertics;
    state.game.d_loop.try_run_tics_oldentertics = entertic;
    if state.game.d_loop.singletics {
        build_new_tic(state);
    } else {
        net_update(state);
    }
    let mut lowtic: i32 = get_low_tic(&state.game.d_loop);
    let availabletics: i32 = lowtic - state.game.d_loop.gametic / state.game.d_loop.ticdup;
    if state.game.d_loop.new_sync {
        counts = availabletics;
    } else {
        if realtics < availabletics - 1 {
            counts = realtics + 1;
        } else if realtics < availabletics {
            counts = realtics;
        } else {
            counts = availabletics;
        }
        if counts < 1 {
            counts = 1;
        }
        if NET_CLIENT_CONNECTED {
            old_net_sync(&mut state.game.d_loop);
        }
    }
    if counts < 1 {
        counts = 1;
    }
    while !players_in_game(&state.game.d_loop)
        || lowtic < state.game.d_loop.gametic / state.game.d_loop.ticdup + counts
    {
        net_update(state);
        lowtic = get_low_tic(&state.game.d_loop);
        if lowtic < state.game.d_loop.gametic / state.game.d_loop.ticdup {
            error("TryRunTics: lowtic < gametic");
        }
        if get_time(&mut state.io.i_timer, &mut *state.io.platform) / state.game.d_loop.ticdup
            - entertic
            > 0
        {
            return;
        }
        sleep(&mut *state.io.platform, 1);
    }
    loop {
        let fresh0 = counts;
        counts -= 1;
        if fresh0 == 0 {
            break;
        }
        if !players_in_game(&state.game.d_loop) {
            return;
        }
        let set_index =
            (state.game.d_loop.gametic / state.game.d_loop.ticdup % BACKUPTICS) as usize;
        let mut set = state.game.d_loop.ticdata[set_index];
        if !NET_CLIENT_CONNECTED {
            single_player_clear(&mut set);
        }
        for _ in 0..state.game.d_loop.ticdup {
            if state.game.d_loop.gametic / state.game.d_loop.ticdup > lowtic {
                error("gametic>lowtic");
            }
            state.game.d_loop.local_playeringame = set.ingame;
            let run_tic = state
                .game
                .d_loop
                .loop_interface
                .run_tic
                .expect("non-null function pointer");
            run_tic(state, &set.cmds, &set.ingame);
            state.game.d_loop.gametic += 1;
            ticdup_squash(&mut set);
        }
        state.game.d_loop.ticdata[set_index] = set;
        net_update(state);
    }
}
pub fn register_loop_callbacks(d_loop: &mut DLoopState, i: LoopInterface) {
    d_loop.loop_interface = i;
}
