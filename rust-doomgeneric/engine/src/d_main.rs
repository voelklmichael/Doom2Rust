use crate::am_map::am_drawer;
use crate::d_event::pop_event;
use crate::d_event::GameAction;
use crate::d_event::GameScreenState;
use crate::d_iwad::find_iwad;
use crate::d_iwad::save_game_iwadname;
use crate::d_loop::net_update;
use crate::d_loop::start_game_loop;
use crate::d_loop::try_run_tics;
use crate::d_mode::GameMission;
use crate::d_mode::GameMode;
use crate::d_mode::GameVersion;
use crate::d_mode::SkillType;
use crate::d_net::check_net_game;
use crate::d_net::connect_net_game;
use crate::d_player::PlayerState;
use crate::doomdef::MAXPLAYERS;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::doomdef::TICRATE;
use crate::doomstat::DoomstatState;
use crate::f_finale::f_drawer;
use crate::f_wipe::wipe_end_screen;
use crate::f_wipe::wipe_screen_wipe;
use crate::f_wipe::wipe_start_screen;
use crate::filesystem::DoomFileSystem;
use crate::fixed_cstr::FixedCStr;
use crate::g_game::begin_recording;
use crate::g_game::check_demo_status;
use crate::g_game::defered_play_demo;
use crate::g_game::g_load_game;
use crate::g_game::g_responder;
use crate::g_game::init_new;
use crate::g_game::record_demo;
use crate::g_game::time_demo;
use crate::g_game::GGameState;
use crate::game_state::GameState;
use crate::hu_stuff::erase;
use crate::hu_stuff::hu_drawer;
use crate::hu_stuff::hu_init;
use crate::i_joystick::bind_joystick_variables;
use crate::i_sound::bind_sound_variables;
use crate::i_sound::init_music;
use crate::i_sound::init_sound;
use crate::i_system::at_exit;
use crate::i_system::error;
use crate::i_system::print_banner;
use crate::i_system::print_divider;
use crate::i_system::print_startup_banner;
use crate::i_timer::get_time;
use crate::i_timer::sleep;
use crate::i_video::finish_update;
use crate::i_video::i_set_window_title;
use crate::i_video::init_graphics;
use crate::i_video::set_grab_mouse_callback;
use crate::i_video::set_palette;
use crate::m_config::bind_variable_int;
use crate::m_config::bind_variable_string;
use crate::m_config::get_save_game_dir;
use crate::m_config::load_defaults;
use crate::m_config::save_defaults;
use crate::m_config::set_config_dir;
use crate::m_config::set_config_filenames;
use crate::m_config::MConfigState;
use crate::m_controls::bind_base_controls;
use crate::m_controls::bind_chat_controls;
use crate::m_controls::bind_map_controls;
use crate::m_controls::bind_menu_controls;
use crate::m_controls::bind_weapon_controls;
use crate::m_controls::MControlsState;
use crate::m_menu::m_drawer;
use crate::m_menu::m_init;
use crate::m_menu::m_responder;
use crate::m_misc::string_ends_with;
use crate::p_saveg::save_game_file;
use crate::p_setup::p_init;
use crate::platform::DoomPlatform;
use crate::r_draw::draw_view_border;
use crate::r_draw::fill_back_screen;
use crate::r_main::execute_set_view_size;
use crate::r_main::r_init;
use crate::r_main::render_player_view;
use crate::s_sound::s_init;
use crate::s_sound::start_music;
use crate::s_sound::update_sounds;
use crate::sounds::MusicName;
use crate::st_stuff::st_drawer;
use crate::st_stuff::st_init;
use crate::statdump::stat_dump;
use crate::v_video::cache_patch_name;
use crate::v_video::Screen;
use crate::w_wad::WWadState;
use alloc::string::String;

use crate::v_video::draw_mouse_speed_box;
use crate::v_video::draw_patch;
use crate::v_video::draw_patch_direct;
use crate::w_main::parse_command_line;
use crate::w_wad::check_correct_iwad;
use crate::w_wad::generate_hash_table;
use crate::w_wad::w_add_file;
use crate::w_wad::{check_num_for_name, lump_bytes_name};
use crate::wi_stuff::wi_drawer;

pub struct DMainState {
    pub savegamedir: String,
    pub iwadfile: String,
    pub devparm: bool,
    pub nomonsters: bool,
    pub respawnparm: bool,
    pub fastparm: bool,
    pub startskill: SkillType,
    pub startepisode: i32,
    pub startmap: i32,
    pub autostart: bool,
    pub startloadgame: i32,
    pub advancedemo: bool,
    pub storedemo: bool,
    pub bfgedition: bool,
    pub main_loop_started: bool,
    pub wadfile: [::core::ffi::c_char; 1024],
    pub mapdir: [::core::ffi::c_char; 1024],
    pub show_endoom: i32,
    pub wipegamestate: GameScreenState,
    pub d_display_viewactivestate: bool,
    pub d_display_menuactivestate: bool,
    pub d_display_inhelpscreensstate: bool,
    pub d_display_fullscreen: bool,
    pub d_display_oldgamestate: GameScreenState,
    pub d_display_borderdrawcount: i32,
    pub demosequence: i32,
    pub pagetic: i32,
    pub pagename: &'static str,
    pub gameversions: [GameVersionInfo; 9],
}

impl Default for DMainState {
    fn default() -> Self {
        Self {
            savegamedir: String::new(),
            iwadfile: String::new(),
            devparm: false,
            nomonsters: false,
            respawnparm: false,
            fastparm: false,
            startskill: SkillType::Baby,
            startepisode: 0,
            startmap: 0,
            autostart: false,
            startloadgame: 0,
            advancedemo: false,
            storedemo: false,
            bfgedition: false,
            main_loop_started: false,
            wadfile: [0; 1024],
            mapdir: [0; 1024],
            show_endoom: 1,
            wipegamestate: GameScreenState::Demoscreen,
            d_display_viewactivestate: false,
            d_display_menuactivestate: false,
            d_display_inhelpscreensstate: false,
            d_display_fullscreen: false,
            d_display_oldgamestate: GameScreenState::Wipped,
            d_display_borderdrawcount: 0,
            demosequence: 0,
            pagetic: 0,
            pagename: "",
            gameversions: [
                GameVersionInfo {
                    description: "Doom 1.666",
                    options: "1.666",
                    version: GameVersion::Doom1666,
                },
                GameVersionInfo {
                    description: "Doom 1.7/1.7a",
                    options: "1.7",
                    version: GameVersion::Doom17,
                },
                GameVersionInfo {
                    description: "Doom 1.8",
                    options: "1.8",
                    version: GameVersion::Doom18,
                },
                GameVersionInfo {
                    description: "Doom 1.9",
                    options: "1.9",
                    version: GameVersion::Doom19,
                },
                GameVersionInfo {
                    description: "Hacx",
                    options: "hacx",
                    version: GameVersion::Hacx,
                },
                GameVersionInfo {
                    description: "Ultimate Doom",
                    options: "ultimate",
                    version: GameVersion::Ultimate,
                },
                GameVersionInfo {
                    description: "Final Doom",
                    options: "final",
                    version: GameVersion::Final,
                },
                GameVersionInfo {
                    description: "Final Doom (alt)",
                    options: "final2",
                    version: GameVersion::Final2,
                },
                GameVersionInfo {
                    description: "Chex Quest",
                    options: "chex",
                    version: GameVersion::Chex,
                },
            ],
        }
    }
}

#[derive(Copy, Clone)]
pub struct MissionPack {
    pub name: &'static str,
    pub mission: GameMission,
}
#[derive(Copy, Clone)]
pub struct GameVersionInfo {
    pub description: &'static str,
    pub options: &'static str,
    pub version: GameVersion,
}
pub const PACKAGE_STRING: FixedCStr<17> = FixedCStr(*b"Doom Generic 0.1\0");
pub const D_DEVSTR: FixedCStr<22> = FixedCStr(*b"Development mode ON.\n\0");
pub const HUSTR_KEYGREEN: i32 = 'g' as i32;
pub const HUSTR_KEYINDIGO: i32 = 'i' as i32;
pub const HUSTR_KEYBROWN: i32 = 'b' as i32;
pub const HUSTR_KEYRED: i32 = 'r' as i32;
pub fn d_process_events(state: &mut GameState) {
    if state.game.d_main.storedemo {
        return;
    }
    while let Some(ev) = pop_event(&mut state.game.d_event) {
        if m_responder(state, &ev) {
            continue;
        }
        g_responder(state, ev);
    }
}
/// Draws what the current game state shows: the level with the automap or status bar, the intermission,
/// the finale or the demo page.
fn draw_game_screen(state: &mut GameState, wipe: bool) {
    let mut redrawsbar: bool = false;
    match state.game.g_game.gamestate {
        GameScreenState::Level => {
            if state.game.d_loop.gametic != 0 {
                if state.ui.am_map.automapactive {
                    am_drawer(state);
                }
                if wipe
                    || state.render.r_draw.viewheight != 200
                        && state.game.d_main.d_display_fullscreen
                {
                    redrawsbar = true;
                }
                if state.game.d_main.d_display_inhelpscreensstate && !state.ui.m_menu.inhelpscreens
                {
                    redrawsbar = true;
                }
                let fullscreen = state.render.r_draw.viewheight == 200;
                st_drawer(state, fullscreen, redrawsbar);
                state.game.d_main.d_display_fullscreen = state.render.r_draw.viewheight == 200;
            }
        }
        GameScreenState::Intermission => {
            wi_drawer(state);
        }
        GameScreenState::Finale => {
            f_drawer(state);
        }
        GameScreenState::Demoscreen => {
            page_drawer(state);
        }
        GameScreenState::Wipped => {}
    }
}

/// Redraws the border of a reduced view window for a few frames after anything disturbed it.
fn redraw_view_border_if_needed(state: &mut GameState) {
    if state.game.g_game.gamestate == GameScreenState::Level
        && !state.ui.am_map.automapactive
        && state.render.r_draw.scaledviewwidth != 320
    {
        if state.ui.m_menu.menuactive
            || state.game.d_main.d_display_menuactivestate
            || !state.game.d_main.d_display_viewactivestate
        {
            state.game.d_main.d_display_borderdrawcount = 3;
        }
        if state.game.d_main.d_display_borderdrawcount != 0 {
            draw_view_border(
                &mut state.io.i_video,
                &state.render.r_draw,
                &mut state.io.v_video,
            );
            state.game.d_main.d_display_borderdrawcount -= 1;
        }
    }
}

/// The PAUSE patch over the view while the game is paused.
fn draw_pause_patch(state: &mut GameState) {
    if state.game.g_game.paused {
        let y: i32 = if state.ui.am_map.automapactive {
            4
        } else {
            state.render.r_draw.viewwindowy + 4
        };
        let __wcache429_2 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "M_PAUSE");
        let dest_screen = Screen::Video;
        draw_patch_direct(
            state,
            dest_screen,
            state.render.r_draw.viewwindowx + (state.render.r_draw.scaledviewwidth - 68) / 2,
            y,
            &__wcache429_2,
        );
    }
}

/// Melts the old screen into the new one (a blocking loop of wipe steps).
fn run_screen_wipe(state: &mut GameState) {
    wipe_end_screen(state, 0, 0, SCREENWIDTH, SCREENHEIGHT);
    let mut wipestart: i32 = get_time(&mut state.io.i_timer, &mut *state.io.platform) - 1;
    loop {
        let (nowtime, tics): (i32, i32) = loop {
            let nowtime = get_time(&mut state.io.i_timer, &mut *state.io.platform);
            let tics = nowtime - wipestart;
            sleep(&mut *state.io.platform, 1);
            if tics > 0 {
                break (nowtime, tics);
            }
        };
        wipestart = nowtime;
        let done: bool = wipe_screen_wipe(state, SCREENWIDTH, SCREENHEIGHT, tics);
        m_drawer(state);
        finish_update(&mut state.io.i_video, &mut *state.io.platform);
        if done {
            break;
        }
    }
}

pub fn display(state: &mut GameState) {
    if state.game.g_game.nodrawers {
        return;
    }
    if state.render.r_main.setsizeneeded {
        execute_set_view_size(&mut state.render);
        state.game.d_main.d_display_oldgamestate = GameScreenState::Wipped;
        state.game.d_main.d_display_borderdrawcount = 3;
    }
    let wipe: bool = state.game.g_game.gamestate != state.game.d_main.wipegamestate;
    if wipe {
        wipe_start_screen(&mut state.ui.f_wipe, &state.io.i_video);
    }
    if state.game.g_game.gamestate == GameScreenState::Level && state.game.d_loop.gametic != 0 {
        erase(state);
    }
    draw_game_screen(state, wipe);
    if state.game.g_game.gamestate == GameScreenState::Level
        && !state.ui.am_map.automapactive
        && state.game.d_loop.gametic != 0
    {
        render_player_view(state, state.game.g_game.displayplayer);
    }
    if state.game.g_game.gamestate == GameScreenState::Level && state.game.d_loop.gametic != 0 {
        hu_drawer(state);
    }
    if state.game.g_game.gamestate != state.game.d_main.d_display_oldgamestate
        && state.game.g_game.gamestate != GameScreenState::Level
    {
        let pal = lump_bytes_name(&*state.assets.fs, &mut state.assets.w_wad, "PLAYPAL");
        set_palette(&mut state.io.i_video, &pal[..768]);
    }
    if state.game.g_game.gamestate == GameScreenState::Level
        && state.game.d_main.d_display_oldgamestate != GameScreenState::Level
    {
        state.game.d_main.d_display_viewactivestate = false;
        fill_back_screen(state);
    }
    redraw_view_border_if_needed(state);
    if state.game.g_game.testcontrols {
        draw_mouse_speed_box(
            &mut state.io.i_video,
            &mut *state.io.platform,
            state.game.g_game.testcontrols_mousespeed,
        );
    }
    state.game.d_main.d_display_menuactivestate = state.ui.m_menu.menuactive;
    state.game.d_main.d_display_viewactivestate = state.game.g_game.viewactive;
    state.game.d_main.d_display_inhelpscreensstate = state.ui.m_menu.inhelpscreens;
    state.game.d_main.wipegamestate = state.game.g_game.gamestate;
    state.game.d_main.d_display_oldgamestate = state.game.d_main.wipegamestate;
    draw_pause_patch(state);
    m_drawer(state);
    net_update(state);
    if !wipe {
        finish_update(&mut state.io.i_video, &mut *state.io.platform);
        return;
    }
    run_screen_wipe(state);
}
pub fn bind_variables(m_config: &mut MConfigState, m_controls: &mut MControlsState) {
    bind_joystick_variables(m_config);
    bind_sound_variables(m_config);
    bind_base_controls(m_config);
    bind_weapon_controls(m_config);
    bind_map_controls(m_config);
    bind_menu_controls(m_config);
    bind_chat_controls(m_config, MAXPLAYERS);
    m_controls.key_multi_msgplayer[0] = HUSTR_KEYGREEN;
    m_controls.key_multi_msgplayer[1] = HUSTR_KEYINDIGO;
    m_controls.key_multi_msgplayer[2] = HUSTR_KEYBROWN;
    m_controls.key_multi_msgplayer[3] = HUSTR_KEYRED;
    bind_variable_int(m_config, "mouse_sensitivity", |s| {
        &mut s.ui.m_menu.mouse_sensitivity
    });
    bind_variable_int(m_config, "sfx_volume", |s| &mut s.audio.s_sound.sfx_volume);
    bind_variable_int(m_config, "music_volume", |s| {
        &mut s.audio.s_sound.music_volume
    });
    bind_variable_int(m_config, "show_messages", |s| {
        &mut s.ui.m_menu.show_messages
    });
    bind_variable_int(m_config, "screenblocks", |s| &mut s.ui.m_menu.screenblocks);
    bind_variable_int(m_config, "detaillevel", |s| &mut s.ui.m_menu.detail_level);
    bind_variable_int(m_config, "snd_channels", |s| {
        &mut s.audio.s_sound.snd_channels
    });
    bind_variable_int(m_config, "vanilla_savegame_limit", |s| {
        &mut s.game.g_game.vanilla_savegame_limit
    });
    bind_variable_int(m_config, "vanilla_demo_limit", |s| {
        &mut s.game.g_game.vanilla_demo_limit
    });
    bind_variable_int(m_config, "show_endoom", |s| &mut s.game.d_main.show_endoom);
    for i in 0..10 {
        let name = format!("chatmacro{i}");
        bind_variable_string(m_config, &name, move |s| &mut s.ui.hu_stuff.chat_macros[i]);
    }
}
pub fn doomgeneric_tick(state: &mut GameState) {
    try_run_tics(state);
    let listener_id = state.game.g_game.players[state.game.g_game.consoleplayer].mo;
    update_sounds(state, listener_id);
    if state.io.i_video.screenvisible {
        display(state);
    }
}
pub fn doom_loop(state: &mut GameState) {
    if state.game.d_main.bfgedition
        && (state.game.g_game.demorecording
            || state.game.g_game.gameaction == GameAction::PlayDemo
            || state.game.g_game.netgame)
    {
        doom_println!(state.io.platform,
            " WARNING: You are playing using one of the Doom Classic\n IWAD files shipped with the Doom 3: BFG Edition. These are\n known to be incompatible with the regular IWAD files and\n may cause demos and network games to get out of sync."
        );
    }
    if state.game.g_game.demorecording {
        begin_recording(&mut state.game);
    }
    state.game.d_main.main_loop_started = true;
    try_run_tics(state);
    i_set_window_title(&mut *state.io.platform, state.game.doomstat.gamedescription);
    set_grab_mouse_callback();
    init_graphics(
        &mut state.io.i_video,
        &state.game.options,
        &mut *state.io.platform,
    );
    execute_set_view_size(&mut state.render);
    start_game_loop(
        &mut state.game.d_loop,
        &mut state.io.i_timer,
        &mut *state.io.platform,
    );
    if state.game.g_game.testcontrols {
        state.game.d_main.wipegamestate = state.game.g_game.gamestate;
    }
    doomgeneric_tick(state);
}
pub fn page_ticker(d_main: &mut DMainState) {
    d_main.pagetic -= 1;
    if d_main.pagetic < 0 {
        advance_demo(d_main);
    }
}
pub fn page_drawer(state: &mut GameState) {
    let __wcache609_1 = cache_patch_name(
        &*state.assets.fs,
        &mut state.assets.w_wad,
        state.game.d_main.pagename,
    );
    let dest_screen = Screen::Video;
    draw_patch(state, dest_screen, 0, 0, &__wcache609_1);
}
pub fn advance_demo(d_main: &mut DMainState) {
    d_main.advancedemo = true;
}
pub fn do_advance_demo(state: &mut GameState) {
    state.game.g_game.players[state.game.g_game.consoleplayer].playerstate = PlayerState::Live;
    state.game.d_main.advancedemo = false;
    state.game.g_game.usergame = false;
    state.game.g_game.paused = false;
    state.game.g_game.gameaction = GameAction::Nothing;
    if [GameVersion::Ultimate, GameVersion::Final].contains(&state.game.doomstat.gameversion) {
        state.game.d_main.demosequence = (state.game.d_main.demosequence + 1) % 7;
    } else {
        state.game.d_main.demosequence = (state.game.d_main.demosequence + 1) % 6;
    }
    match state.game.d_main.demosequence {
        0 => {
            if state.game.doomstat.gamemode == GameMode::Commercial {
                state.game.d_main.pagetic = TICRATE * 11;
            } else {
                state.game.d_main.pagetic = 170;
            }
            state.game.g_game.gamestate = GameScreenState::Demoscreen;
            state.game.d_main.pagename = "TITLEPIC";
            if state.game.doomstat.gamemode == GameMode::Commercial {
                start_music(state, MusicName::Dm2ttl as i32);
            } else {
                start_music(state, MusicName::Intro as i32);
            }
        }
        1 => {
            defered_play_demo(&mut state.game.g_game, FixedCStr::new("demo1"));
        }
        2 => {
            state.game.d_main.pagetic = 200;
            state.game.g_game.gamestate = GameScreenState::Demoscreen;
            state.game.d_main.pagename = "CREDIT";
        }
        3 => {
            defered_play_demo(&mut state.game.g_game, FixedCStr::new("demo2"));
        }
        4 => {
            state.game.g_game.gamestate = GameScreenState::Demoscreen;
            if state.game.doomstat.gamemode == GameMode::Commercial {
                state.game.d_main.pagetic = TICRATE * 11;
                state.game.d_main.pagename = "TITLEPIC";
                start_music(state, MusicName::Dm2ttl as i32);
            } else {
                state.game.d_main.pagetic = 200;
                if state.game.doomstat.gamemode == GameMode::Retail {
                    state.game.d_main.pagename = "CREDIT";
                } else {
                    state.game.d_main.pagename = "HELP2";
                }
            }
        }
        5 => {
            defered_play_demo(&mut state.game.g_game, FixedCStr::new("demo3"));
        }
        6 => {
            defered_play_demo(&mut state.game.g_game, FixedCStr::new("demo4"));
        }
        _ => {}
    }
    if state.game.d_main.bfgedition
        && state.game.d_main.pagename.eq_ignore_ascii_case("TITLEPIC")
        && check_num_for_name(&state.assets.w_wad, "titlepic").is_none()
    {
        state.game.d_main.pagename = "INTERPIC";
    }
}
pub fn start_title(d_main: &mut DMainState, g_game: &mut GGameState) {
    g_game.gameaction = GameAction::Nothing;
    d_main.demosequence = -1;
    advance_demo(d_main);
}
fn set_mission_for_pack_name(
    doomstat: &mut DoomstatState,
    platform: &mut dyn DoomPlatform,
    pack_name: &str,
) {
    const PACKS: [MissionPack; 3] = [
        MissionPack {
            name: "doom2",
            mission: GameMission::Doom2,
        },
        MissionPack {
            name: "tnt",
            mission: GameMission::PackTnt,
        },
        MissionPack {
            name: "plutonia",
            mission: GameMission::PackPlut,
        },
    ];
    for pack in &PACKS {
        if pack_name.eq_ignore_ascii_case(pack.name) {
            doomstat.gamemission = pack.mission;
            return;
        }
    }
    doom_println!(platform, "Valid mission packs are:");
    for pack in &PACKS {
        doom_println!(platform, "\t{}", pack.name);
    }
    error(&format!("Unknown mission pack name: {pack_name}"));
}
pub fn identify_version(state: &mut GameState) {
    if state.game.doomstat.gamemission == GameMission::None {
        for i in 0..state.assets.w_wad.numlumps {
            if state.assets.w_wad.lumpinfo[i as usize]
                .name
                .eq_str_ignore_ascii_case("MAP01")
            {
                state.game.doomstat.gamemission = GameMission::Doom2;
                break;
            } else if state.assets.w_wad.lumpinfo[i as usize]
                .name
                .eq_str_ignore_ascii_case("E1M1")
            {
                state.game.doomstat.gamemission = GameMission::Doom;
                break;
            }
        }
        if state.game.doomstat.gamemission == GameMission::None {
            error("Unknown or invalid IWAD file.");
        }
    }
    if state.game.doomstat.gamemission.base() == GameMission::Doom {
        if check_num_for_name(&state.assets.w_wad, "E4M1").is_some() {
            state.game.doomstat.gamemode = GameMode::Retail;
        } else if check_num_for_name(&state.assets.w_wad, "E3M1").is_some() {
            state.game.doomstat.gamemode = GameMode::Registered;
        } else {
            state.game.doomstat.gamemode = GameMode::Shareware;
        }
    } else {
        state.game.doomstat.gamemode = GameMode::Commercial;
        if let Some(pack_name) = state.game.options.pack.clone() {
            set_mission_for_pack_name(
                &mut state.game.doomstat,
                &mut *state.io.platform,
                &pack_name,
            );
        }
    }
}
pub fn set_game_description(doomstat: &mut DoomstatState, w_wad: &WWadState) {
    let is_freedoom: bool = check_num_for_name(w_wad, "FREEDOOM").is_some();
    let is_freedm: bool = check_num_for_name(w_wad, "FREEDM").is_some();
    doomstat.gamedescription = "Unknown";
    if doomstat.gamemission.base() == GameMission::Doom {
        if is_freedoom {
            doomstat.gamedescription = "Freedoom: Phase 1";
        } else if doomstat.gamemode == GameMode::Retail {
            doomstat.gamedescription = "The Ultimate DOOM";
        } else if doomstat.gamemode == GameMode::Registered {
            doomstat.gamedescription = "DOOM Registered";
        } else if doomstat.gamemode == GameMode::Shareware {
            doomstat.gamedescription = "DOOM Shareware";
        }
    } else if is_freedoom {
        if is_freedm {
            doomstat.gamedescription = "FreeDM";
        } else {
            doomstat.gamedescription = "Freedoom: Phase 2";
        }
    } else if doomstat.gamemission.base() == GameMission::Doom2 {
        doomstat.gamedescription = "DOOM 2: Hell on Earth";
    } else if doomstat.gamemission.base() == GameMission::PackPlut {
        doomstat.gamedescription = "DOOM 2: Plutonia Experiment";
    } else if doomstat.gamemission.base() == GameMission::PackTnt {
        doomstat.gamedescription = "DOOM 2: TNT - Evilution";
    }
}
fn d_add_file(
    fs: &mut dyn DoomFileSystem,
    platform: &mut dyn DoomPlatform,
    w_wad: &mut WWadState,
    filename: &str,
) -> bool {
    doom_println!(platform, " adding {}", filename);
    w_add_file(&mut *fs, &mut *platform, w_wad, filename).is_some()
}
fn init_game_version(state: &mut GameState) {
    if let Some(name) = state.game.options.gameversion.clone() {
        let arg = name.as_bytes();
        let found = state
            .game
            .d_main
            .gameversions
            .iter()
            .find(|gv| gv.options.as_bytes() == arg);
        if let Some(gv) = found {
            state.game.doomstat.gameversion = gv.version;
        } else {
            doom_println!(state.io.platform, "Supported game versions:");
            for gv in &state.game.d_main.gameversions {
                doom_println!(state.io.platform, "\t{} ({})", gv.options, gv.description);
            }
            error(&format!("Unknown game version '{name}'"));
        }
    } else if state.game.doomstat.gamemission == GameMission::PackChex {
        state.game.doomstat.gameversion = GameVersion::Chex;
    } else if state.game.doomstat.gamemission == GameMission::PackHacx {
        state.game.doomstat.gameversion = GameVersion::Hacx;
    } else if state.game.doomstat.gamemode == GameMode::Shareware
        || state.game.doomstat.gamemode == GameMode::Registered
    {
        state.game.doomstat.gameversion = GameVersion::Doom19;
    } else if state.game.doomstat.gamemode == GameMode::Retail {
        state.game.doomstat.gameversion = GameVersion::Ultimate;
    } else if state.game.doomstat.gamemode == GameMode::Commercial {
        if state.game.doomstat.gamemission == GameMission::Doom2 {
            state.game.doomstat.gameversion = GameVersion::Doom19;
        } else {
            state.game.doomstat.gameversion = GameVersion::Final;
        }
    }
    if state.game.doomstat.gameversion == GameVersion::Ultimate
        && state.game.doomstat.gamemode == GameMode::Retail
    {
        state.game.doomstat.gamemode = GameMode::Registered;
    }
    if (state.game.doomstat.gameversion as u32) < GameVersion::Final as u32
        && state.game.doomstat.gamemode == GameMode::Commercial
        && (state.game.doomstat.gamemission == GameMission::PackTnt
            || state.game.doomstat.gamemission == GameMission::PackPlut)
    {
        state.game.doomstat.gamemission = GameMission::Doom2;
    }
}
pub fn print_game_version(
    d_main: &DMainState,
    doomstat: &DoomstatState,
    platform: &mut dyn DoomPlatform,
) {
    if let Some(gv) = d_main
        .gameversions
        .iter()
        .find(|gv| gv.version == doomstat.gameversion)
    {
        doom_println!(
            platform,
            "Emulating the behavior of the '{}' executable.",
            gv.description
        );
    }
}
fn endoom(state: &mut GameState) {
    if state.game.d_main.show_endoom == 0
        || !state.game.d_main.main_loop_started
        || state.io.i_video.screensaver_mode
        || state.game.options.testcontrols
    {
        return;
    }
    state.io.platform.quit();
}
fn quit_check_demo_status(state: &mut GameState) {
    check_demo_status(state);
}
/// -turbo scales the walking and strafing speeds.
fn apply_turbo(state: &mut GameState) {
    if let Some(percent) = state.game.options.turbo {
        let scale: i32 = percent.unwrap_or(200).clamp(10, 400);
        doom_println!(state.io.platform, "turbo scale: {}%", scale);
        state.game.g_game.forwardmove[0] = state.game.g_game.forwardmove[0] * scale / 100;
        state.game.g_game.forwardmove[1] = state.game.g_game.forwardmove[1] * scale / 100;
        state.game.g_game.sidemove[0] = state.game.g_game.sidemove[0] * scale / 100;
        state.game.g_game.sidemove[1] = state.game.g_game.sidemove[1] * scale / 100;
    }
}

/// The demo named by -playdemo or -timedemo: added from a .lmp file when there is one. Returns the
/// lump name to play.
fn add_demo_file(state: &mut GameState) -> FixedCStr<8> {
    let mut demolumpname: FixedCStr<8> = FixedCStr::from_array([0; 8]);
    let demo_arg = state
        .game
        .options
        .playdemo
        .clone()
        .or_else(|| state.game.options.timedemo.clone());
    if let Some(arg) = demo_arg {
        let file: String = if string_ends_with(&arg, ".lmp") {
            arg.clone()
        } else {
            format!("{arg}.lmp")
        };
        if d_add_file(
            &mut *state.assets.fs,
            &mut *state.io.platform,
            &mut state.assets.w_wad,
            &file,
        ) {
            demolumpname = state.assets.w_wad.lumpinfo
                [state.assets.w_wad.numlumps.wrapping_sub(1) as usize]
                .name;
        } else {
            demolumpname = FixedCStr::from_bytes(arg.as_bytes());
        }
        doom_println!(state.io.platform, "Playing demo {}.", file);
    }
    demolumpname
}

/// A modified game (`-file`) cannot be the shareware version, and must have all of the registered
/// version's levels and sprites when it says it is that.
fn check_registered_files(state: &GameState) {
    if state.game.doomstat.modifiedgame {
        let name: [FixedCStr<8>; 23] = [
            FixedCStr(*b"e2m1\0\0\0\0"),
            FixedCStr(*b"e2m2\0\0\0\0"),
            FixedCStr(*b"e2m3\0\0\0\0"),
            FixedCStr(*b"e2m4\0\0\0\0"),
            FixedCStr(*b"e2m5\0\0\0\0"),
            FixedCStr(*b"e2m6\0\0\0\0"),
            FixedCStr(*b"e2m7\0\0\0\0"),
            FixedCStr(*b"e2m8\0\0\0\0"),
            FixedCStr(*b"e2m9\0\0\0\0"),
            FixedCStr(*b"e3m1\0\0\0\0"),
            FixedCStr(*b"e3m3\0\0\0\0"),
            FixedCStr(*b"e3m3\0\0\0\0"),
            FixedCStr(*b"e3m4\0\0\0\0"),
            FixedCStr(*b"e3m5\0\0\0\0"),
            FixedCStr(*b"e3m6\0\0\0\0"),
            FixedCStr(*b"e3m7\0\0\0\0"),
            FixedCStr(*b"e3m8\0\0\0\0"),
            FixedCStr(*b"e3m9\0\0\0\0"),
            FixedCStr(*b"dphoof\0\0"),
            FixedCStr(*b"bfgga0\0\0"),
            FixedCStr(*b"heada1\0\0"),
            FixedCStr(*b"cybra1\0\0"),
            FixedCStr(*b"spida1d1"),
        ];
        if state.game.doomstat.gamemode == GameMode::Shareware {
            error("\nYou cannot -file with the shareware version. Register!");
        }
        if state.game.doomstat.gamemode == GameMode::Registered {
            for lump_name in &name {
                if check_num_for_name(&state.assets.w_wad, &lump_name.as_str()).is_none() {
                    error("\nThis is not the registered version.");
                }
            }
        }
    }
}

/// Warns about WADs with modified sprites or flats, and about Freedoom.
fn print_wad_warnings(state: &mut GameState) {
    if check_num_for_name(&state.assets.w_wad, "SS_START").is_some()
        || check_num_for_name(&state.assets.w_wad, "FF_END").is_some()
    {
        print_divider(&mut *state.io.platform);
        doom_println!(state.io.platform,
            " WARNING: The loaded WAD file contains modified sprites or\n floor textures.  You may want to use the '-merge' command\n line option instead of '-file'."
        );
    }
    print_startup_banner(&mut *state.io.platform, state.game.doomstat.gamedescription);
    if check_num_for_name(&state.assets.w_wad, "FREEDOOM").is_some()
        && check_num_for_name(&state.assets.w_wad, "FREEDM").is_none()
    {
        doom_println!(state.io.platform,
            " WARNING: You are playing using one of the Freedoom IWAD\n files, which might not work in this port. See this page\n for more information on how to play using Freedoom:\n   http://www.chocolate-doom.org/wiki/index.php/Freedoom"
        );
        print_divider(&mut *state.io.platform);
    }
}

/// The skill, episode, map, time limit and test-controls options that decide how the game starts.
fn read_start_options(state: &mut GameState) {
    state.game.d_main.startskill = SkillType::Medium;
    state.game.d_main.startepisode = 1;
    state.game.d_main.startmap = 1;
    state.game.d_main.autostart = false;
    if let Some(level) = state.game.options.skill {
        let Some(skill) = SkillType::from_raw(level - 1) else {
            error(&format!(
                "-skill {level}: there is no such skill level (1 to 5)"
            ));
        };
        state.game.d_main.startskill = skill;
        state.game.d_main.autostart = true;
    }
    if let Some(episode) = state.game.options.episode {
        state.game.d_main.startepisode = episode;
        state.game.d_main.startmap = 1;
        state.game.d_main.autostart = true;
    }
    state.game.g_game.timelimit = 0;
    if let Some(minutes) = state.game.options.timer {
        state.game.g_game.timelimit = minutes;
    }
    if state.game.options.avg {
        state.game.g_game.timelimit = 20;
    }
    if let Some(warp) = state.game.options.warp {
        if state.game.doomstat.gamemode == GameMode::Commercial {
            state.game.d_main.startmap = warp.map_number;
        } else {
            state.game.d_main.startepisode = warp.episode;
            state.game.d_main.startmap = warp.episode_map;
        }
        state.game.d_main.autostart = true;
    }
    if state.game.options.testcontrols {
        state.game.d_main.startepisode = 1;
        state.game.d_main.startmap = 1;
        state.game.d_main.autostart = true;
        state.game.g_game.testcontrols = true;
    }
    state.game.d_main.startloadgame = state.game.options.loadgame.unwrap_or(-1);
}

/// Starts the menu, renderer, play loop, sound, network, heads-up display and status bar.
fn init_game_subsystems(state: &mut GameState) {
    doom_println!(state.io.platform, "M_Init: Init miscellaneous info.");
    m_init(&state.game.doomstat, &mut state.ui.m_menu);
    doom_print!(state.io.platform, "R_Init: Init DOOM refresh daemon - ");
    r_init(state);
    doom_println!(state.io.platform);
    doom_println!(state.io.platform, "P_Init: Init Playloop state.");
    p_init(state);
    doom_println!(state.io.platform, "S_Init: Setting up sound.");
    s_init(
        state,
        state.audio.s_sound.sfx_volume * 8,
        state.audio.s_sound.music_volume * 8,
    );
    doom_println!(
        state.io.platform,
        "D_CheckNetGame: Checking network game status."
    );
    check_net_game(state);
    print_game_version(
        &state.game.d_main,
        &state.game.doomstat,
        &mut *state.io.platform,
    );
    doom_println!(state.io.platform, "HU_Init: Setting up heads up display.");
    hu_init(
        &*state.assets.fs,
        &mut state.ui.hu_stuff,
        &mut state.assets.w_wad,
    );
    doom_println!(state.io.platform, "ST_Init: Init status bar.");
    st_init(state);
}

/// The monster, respawn, speed, developer and deathmatch options.
fn read_game_options(state: &mut GameState) {
    state.game.d_main.nomonsters = state.game.options.nomonsters;
    state.game.d_main.respawnparm = state.game.options.respawn;
    state.game.d_main.fastparm = state.game.options.fast;
    state.game.d_main.devparm = state.game.options.devparm;
    if state.game.options.deathmatch {
        state.game.g_game.deathmatch = 1;
    }
    if state.game.options.altdeath {
        state.game.g_game.deathmatch = 2;
    }
    if state.game.d_main.devparm {
        doom_print!(state.io.platform, "{}", D_DEVSTR.as_str());
    }
}

/// Loads the configuration file and registers saving it on exit.
fn load_config(state: &mut GameState) {
    doom_println!(state.io.platform, "V_Init: allocate screens.");
    doom_println!(state.io.platform, "M_LoadDefaults: Load system defaults.");
    set_config_filenames(
        &mut state.game.m_config,
        "default.cfg",
        "doomgenericdoom.cfg",
    );
    bind_variables(&mut state.game.m_config, &mut state.game.m_controls);
    load_defaults(
        &state.game.options,
        &mut state.game.m_config,
        &mut *state.io.platform,
    );
    at_exit(
        &mut state.io.i_system,
        Some(save_defaults as fn(&mut GameState) -> ()),
        false,
    );
}

/// Finds the IWAD, adds it and works out which game and version it is.
fn load_iwad(state: &mut GameState) {
    let mut gamemission_out = state.game.doomstat.gamemission;
    state.game.d_main.iwadfile = find_iwad(
        state,
        1 << GameMission::Doom as i32
            | 1 << GameMission::Doom2 as i32
            | 1 << GameMission::PackTnt as i32
            | 1 << GameMission::PackPlut as i32
            | 1 << GameMission::PackChex as i32
            | 1 << GameMission::PackHacx as i32,
        &mut gamemission_out,
    );
    state.game.doomstat.gamemission = gamemission_out;
    if state.game.d_main.iwadfile.is_empty() {
        error(
            "Game mode indeterminate.  No IWAD file was found.  Try\nspecifying one with the '-iwad' command line parameter.\n",
        );
    }
    state.game.doomstat.modifiedgame = false;
    doom_println!(state.io.platform, "W_Init: Init WADfiles.");
    let iwadfile = state.game.d_main.iwadfile.clone();
    d_add_file(
        &mut *state.assets.fs,
        &mut *state.io.platform,
        &mut state.assets.w_wad,
        &iwadfile,
    );
    check_correct_iwad(&state.assets.w_wad, GameMission::Doom);
    identify_version(state);
    init_game_version(state);
    if check_num_for_name(&state.assets.w_wad, "dmenupic").is_some() {
        doom_println!(
            state.io.platform,
            "BFG Edition: Using workarounds as needed."
        );
        state.game.d_main.bfgedition = true;
    }
}

pub fn doom_main(state: &mut GameState) {
    at_exit(
        &mut state.io.i_system,
        Some(endoom as fn(&mut GameState) -> ()),
        false,
    );
    print_banner(&mut *state.io.platform, &PACKAGE_STRING.as_str());
    read_game_options(state);
    set_config_dir(
        &mut state.game.m_config,
        &mut *state.assets.fs,
        &mut *state.io.platform,
        None,
    );
    apply_turbo(state);
    load_config(state);
    load_iwad(state);
    let modifiedgame = parse_command_line(state);
    state.game.doomstat.modifiedgame = modifiedgame;
    let demolumpname = add_demo_file(state);
    at_exit(
        &mut state.io.i_system,
        Some(quit_check_demo_status as fn(&mut GameState) -> ()),
        true,
    );
    generate_hash_table(&mut state.assets.w_wad);
    set_game_description(&mut state.game.doomstat, &state.assets.w_wad);
    state.game.d_main.savegamedir = get_save_game_dir(
        &state.game.m_config,
        &mut *state.assets.fs,
        &mut *state.io.platform,
        save_game_iwadname(state.game.doomstat.gamemission),
    );
    check_registered_files(state);
    print_wad_warnings(state);
    doom_println!(state.io.platform, "I_Init: Setting up machine state.");
    init_sound(
        &mut state.audio.i_sound,
        &state.io.i_video,
        &state.game.options,
        &mut *state.io.platform,
        true,
    );
    init_music(state);
    connect_net_game(state);
    read_start_options(state);
    init_game_subsystems(state);
    if state.game.doomstat.gamemode == GameMode::Commercial
        && check_num_for_name(&state.assets.w_wad, "map01").is_none()
    {
        state.game.d_main.storedemo = true;
    }
    if state.game.options.statdump_file.is_some() {
        at_exit(
            &mut state.io.i_system,
            Some(stat_dump as fn(&mut GameState) -> ()),
            true,
        );
        doom_println!(state.io.platform, "External statistics registered.");
    }
    if let Some(record_name) = state.game.options.record_file.clone() {
        record_demo(&mut state.game.g_game, &state.game.options, &record_name);
        state.game.d_main.autostart = true;
    }
    if state.game.options.playdemo.is_some() {
        state.game.g_game.singledemo = true;
        defered_play_demo(&mut state.game.g_game, demolumpname);
        doom_loop(state);
        return;
    }
    if state.game.options.timedemo.is_some() {
        time_demo(
            &mut state.game.d_loop,
            &mut state.game.g_game,
            &state.game.options,
            demolumpname,
        );
        doom_loop(state);
        return;
    }
    if state.game.d_main.startloadgame >= 0 {
        let savegame_file = save_game_file(&state.game.d_main, state.game.d_main.startloadgame);
        g_load_game(&mut state.game.g_game, &savegame_file);
    }
    if state.game.g_game.gameaction != GameAction::LoadGame {
        if state.game.d_main.autostart || state.game.g_game.netgame {
            let (startskill, startepisode, startmap) = (
                state.game.d_main.startskill,
                state.game.d_main.startepisode,
                state.game.d_main.startmap,
            );
            init_new(state, startskill, startepisode, startmap);
        } else {
            start_title(&mut state.game.d_main, &mut state.game.g_game);
        }
    }
    doom_loop(state);
}
