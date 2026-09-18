use crate::am_map::AM_Drawer;
use crate::d_event::D_PopEvent;
use crate::d_event::GameAction;
use crate::d_event::GameScreenState;
use crate::d_iwad::D_FindIWAD;
use crate::d_iwad::D_SaveGameIWADName;
use crate::d_loop::D_StartGameLoop;
use crate::d_loop::NetUpdate;
use crate::d_loop::TryRunTics;
use crate::d_mode::GameMission_t;
use crate::d_mode::GameMode_t;
use crate::d_mode::GameVersion;
use crate::d_mode::{skill_from_raw, SkillType};
use crate::d_net::D_CheckNetGame;
use crate::d_net::D_ConnectNetGame;
use crate::d_player::{PlayerId, PlayerState};
use crate::doomdef::MAXPLAYERS;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::doomdef::TICRATE;
use crate::dummy::drone;
use crate::f_finale::F_Drawer;
use crate::f_wipe::wipe_EndScreen;
use crate::f_wipe::wipe_ScreenWipe;
use crate::f_wipe::wipe_StartScreen;
use crate::fixed_cstr::FixedCStr;
use crate::g_game::G_BeginRecording;
use crate::g_game::G_CheckDemoStatus;
use crate::g_game::G_DeferedPlayDemo;
use crate::g_game::G_InitNew;
use crate::g_game::G_LoadGame;
use crate::g_game::G_RecordDemo;
use crate::g_game::G_Responder;
use crate::g_game::G_TimeDemo;
use crate::game_state::GameState;
use crate::hu_lib::patch_t;
use crate::hu_stuff::HU_Drawer;
use crate::hu_stuff::HU_Erase;
use crate::hu_stuff::HU_Init;
use crate::i_joystick::I_BindJoystickVariables;
use crate::i_sound::I_BindSoundVariables;
use crate::i_sound::I_InitMusic;
use crate::i_sound::I_InitSound;
use crate::i_system::I_AtExit;
use crate::i_system::I_Error;
use crate::i_system::I_PrintBanner;
use crate::i_system::I_PrintDivider;
use crate::i_system::I_PrintStartupBanner;
use crate::i_timer::I_GetTime;
use crate::i_timer::I_Sleep;
use crate::i_video::I_FinishUpdate;
use crate::i_video::I_InitGraphics;
use crate::i_video::I_SetGrabMouseCallback;
use crate::i_video::I_SetPalette;
use crate::i_video::I_SetWindowTitle;
use crate::m_argv::{M_ArgvAtoi, M_CheckParm, M_CheckParmWithArgs};
use crate::m_config::M_BindVariable_int;
use crate::m_config::M_BindVariable_string;
use crate::m_config::M_GetSaveGameDir;
use crate::m_config::M_LoadDefaults;
use crate::m_config::M_SaveDefaults;
use crate::m_config::M_SetConfigDir;
use crate::m_config::M_SetConfigFilenames;
use crate::m_controls::M_BindBaseControls;
use crate::m_controls::M_BindChatControls;
use crate::m_controls::M_BindMapControls;
use crate::m_controls::M_BindMenuControls;
use crate::m_controls::M_BindWeaponControls;
use crate::m_menu::M_Drawer;
use crate::m_menu::M_Init;
use crate::m_menu::M_Responder;
use crate::m_misc::M_StringEndsWith;
use crate::p_saveg::P_SaveGameFile;
use crate::p_setup::P_Init;
use crate::r_draw::R_DrawViewBorder;
use crate::r_draw::R_FillBackScreen;
use crate::r_main::R_ExecuteSetViewSize;
use crate::r_main::R_Init;
use crate::r_main::R_RenderPlayerView;
use crate::s_sound::S_Init;
use crate::s_sound::S_StartMusic;
use crate::s_sound::S_UpdateSounds;
use crate::sounds::{mus_dm2ttl, mus_intro};
use crate::st_stuff::ST_Drawer;
use crate::st_stuff::ST_Init;
use crate::statdump::StatDump;
use crate::stdint_types::byte;
use crate::stdint_types::size_t;
use crate::v_video::V_DrawMouseSpeedBox;
use crate::v_video::V_DrawPatch;
use crate::v_video::V_DrawPatchDirect;
use crate::w_main::W_ParseCommandLine;
use crate::w_wad::W_AddFile;
use crate::w_wad::W_CheckCorrectIWAD;
use crate::w_wad::W_GenerateHashTable;
use crate::w_wad::{W_CacheLumpName, W_CheckNumForName};
use crate::wi_stuff::WI_Drawer;

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
    pub gameversions: [C2RustUnnamed_4; 9],
}

impl DMainState {
    pub const fn new() -> Self {
        DMainState {
            savegamedir: String::new(),
            iwadfile: String::new(),
            devparm: false,
            nomonsters: false,
            respawnparm: false,
            fastparm: false,
            startskill: SkillType::sk_baby,
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
            wipegamestate: GameScreenState::GS_DEMOSCREEN,
            d_display_viewactivestate: false,
            d_display_menuactivestate: false,
            d_display_inhelpscreensstate: false,
            d_display_fullscreen: false,
            d_display_oldgamestate: GameScreenState::GS_WIPPED,
            d_display_borderdrawcount: 0,
            demosequence: 0,
            pagetic: 0,
            pagename: "",
            gameversions: [
                C2RustUnnamed_4 {
                    description: "Doom 1.666",
                    cmdline: "1.666",
                    version: GameVersion::doom_1_666,
                },
                C2RustUnnamed_4 {
                    description: "Doom 1.7/1.7a",
                    cmdline: "1.7",
                    version: GameVersion::doom_1_7,
                },
                C2RustUnnamed_4 {
                    description: "Doom 1.8",
                    cmdline: "1.8",
                    version: GameVersion::doom_1_8,
                },
                C2RustUnnamed_4 {
                    description: "Doom 1.9",
                    cmdline: "1.9",
                    version: GameVersion::doom_1_9,
                },
                C2RustUnnamed_4 {
                    description: "Hacx",
                    cmdline: "hacx",
                    version: GameVersion::hacx,
                },
                C2RustUnnamed_4 {
                    description: "Ultimate Doom",
                    cmdline: "ultimate",
                    version: GameVersion::ultimate,
                },
                C2RustUnnamed_4 {
                    description: "Final Doom",
                    cmdline: "final",
                    version: GameVersion::r#final,
                },
                C2RustUnnamed_4 {
                    description: "Final Doom (alt)",
                    cmdline: "final2",
                    version: GameVersion::final2,
                },
                C2RustUnnamed_4 {
                    description: "Chex Quest",
                    cmdline: "chex",
                    version: GameVersion::chex,
                },
            ],
        }
    }
}

pub type C2RustUnnamed_2 = u32;
pub const wipe_NUMWIPES: C2RustUnnamed_2 = 2;
pub const wipe_Melt: C2RustUnnamed_2 = 1;
pub const wipe_ColorXForm: C2RustUnnamed_2 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_3 {
    pub name: &'static str,
    pub mission: GameMission_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_4 {
    pub description: &'static str,
    pub cmdline: &'static str,
    pub version: GameVersion,
}
pub const PACKAGE_STRING: FixedCStr<17> = FixedCStr(*b"Doom Generic 0.1\0");
pub const D_DEVSTR: FixedCStr<22> = FixedCStr(*b"Development mode ON.\n\0");
pub const HUSTR_KEYGREEN: i32 = 'g' as i32;
pub const HUSTR_KEYINDIGO: i32 = 'i' as i32;
pub const HUSTR_KEYBROWN: i32 = 'b' as i32;
pub const HUSTR_KEYRED: i32 = 'r' as i32;
pub fn D_ProcessEvents(state: &mut GameState) {
    if state.d_main.storedemo {
        return;
    }
    loop {
        let Some(mut ev) = D_PopEvent(&mut state.d_event) else {
            break;
        };
        if unsafe { M_Responder(state, &mut ev) } {
            continue;
        }
        G_Responder(state, ev);
    }
}
pub unsafe fn D_Display(state: &mut GameState) {
    let mut nowtime: i32 = 0;
    let mut tics: i32 = 0;
    let mut wipestart: i32 = 0;
    let mut y: i32 = 0;
    let mut done: bool = false;
    let mut wipe: bool = false;
    let mut redrawsbar: bool = false;
    if state.g_game.nodrawers {
        return;
    }
    redrawsbar = false;
    if state.r_main.setsizeneeded {
        R_ExecuteSetViewSize(state);
        state.d_main.d_display_oldgamestate = GameScreenState::GS_WIPPED;
        state.d_main.d_display_borderdrawcount = 3_i32;
    }
    if state.g_game.gamestate != state.d_main.wipegamestate {
        wipe = true;
        wipe_StartScreen(state);
    } else {
        wipe = false;
    }
    if state.g_game.gamestate == GameScreenState::GS_LEVEL && state.d_loop.gametic != 0 {
        HU_Erase(state);
    }
    match state.g_game.gamestate as u32 {
        0 => {
            if state.d_loop.gametic != 0 {
                if state.am_map.automapactive {
                    AM_Drawer(state);
                }
                if wipe || state.r_draw.viewheight != 200_i32 && state.d_main.d_display_fullscreen {
                    redrawsbar = true;
                }
                if state.d_main.d_display_inhelpscreensstate && !state.m_menu.inhelpscreens {
                    redrawsbar = true;
                }
                let fullscreen = state.r_draw.viewheight == 200_i32;
                ST_Drawer(state, fullscreen, redrawsbar);
                state.d_main.d_display_fullscreen = state.r_draw.viewheight == 200_i32;
            }
        }
        1 => {
            WI_Drawer(state);
        }
        2 => {
            F_Drawer(state);
        }
        3 => {
            D_PageDrawer(state);
        }
        _ => {}
    }
    if state.g_game.gamestate == GameScreenState::GS_LEVEL
        && !state.am_map.automapactive
        && state.d_loop.gametic != 0
    {
        R_RenderPlayerView(state, PlayerId(state.g_game.displayplayer as u8));
    }
    if state.g_game.gamestate == GameScreenState::GS_LEVEL && state.d_loop.gametic != 0 {
        HU_Drawer(state);
    }
    if state.g_game.gamestate as u32 != state.d_main.d_display_oldgamestate as u32
        && state.g_game.gamestate != GameScreenState::GS_LEVEL
    {
        let __wcache387_3 = W_CacheLumpName(state, "PLAYPAL") as *mut byte;
        I_SetPalette(state, __wcache387_3);
    }
    if state.g_game.gamestate == GameScreenState::GS_LEVEL
        && state.d_main.d_display_oldgamestate != GameScreenState::GS_LEVEL
    {
        state.d_main.d_display_viewactivestate = false;
        R_FillBackScreen(state);
    }
    if state.g_game.gamestate == GameScreenState::GS_LEVEL
        && !state.am_map.automapactive
        && state.r_draw.scaledviewwidth != 320_i32
    {
        if state.m_menu.menuactive
            || state.d_main.d_display_menuactivestate
            || !state.d_main.d_display_viewactivestate
        {
            state.d_main.d_display_borderdrawcount = 3_i32;
        }
        if state.d_main.d_display_borderdrawcount != 0 {
            R_DrawViewBorder(state);
            state.d_main.d_display_borderdrawcount -= 1;
        }
    }
    if state.g_game.testcontrols {
        V_DrawMouseSpeedBox(&mut state.i_video, state.g_game.testcontrols_mousespeed);
    }
    state.d_main.d_display_menuactivestate = state.m_menu.menuactive;
    state.d_main.d_display_viewactivestate = state.g_game.viewactive;
    state.d_main.d_display_inhelpscreensstate = state.m_menu.inhelpscreens;
    state.d_main.wipegamestate = state.g_game.gamestate;
    state.d_main.d_display_oldgamestate = state.d_main.wipegamestate;
    if state.g_game.paused {
        if state.am_map.automapactive {
            y = 4_i32;
        } else {
            y = state.r_draw.viewwindowy + 4_i32;
        }
        let __wcache429_2 = W_CacheLumpName(state, "M_PAUSE") as *mut patch_t;
        let dest_screen = state.i_video.I_VideoBuffer.as_mut_ptr();
        V_DrawPatchDirect(
            state,
            dest_screen,
            state.r_draw.viewwindowx + (state.r_draw.scaledviewwidth - 68_i32) / 2_i32,
            y,
            __wcache429_2,
        );
    }
    M_Drawer(state);
    NetUpdate(state);
    if !wipe {
        I_FinishUpdate(state);
        return;
    }
    wipe_EndScreen(state, 0_i32, 0_i32, SCREENWIDTH, SCREENHEIGHT);
    wipestart = I_GetTime(state) - 1_i32;
    loop {
        loop {
            nowtime = I_GetTime(state);
            tics = nowtime - wipestart;
            I_Sleep(state, 1_i32);
            if tics > 0_i32 {
                break;
            }
        }
        wipestart = nowtime;
        done = wipe_ScreenWipe(state, wipe_Melt as i32, SCREENWIDTH, SCREENHEIGHT, tics) != 0;
        M_Drawer(state);
        I_FinishUpdate(state);
        if done {
            break;
        }
    }
}
pub fn D_BindVariables(state: &mut GameState) {
    let mut i: i32 = 0;
    I_BindJoystickVariables(state);
    I_BindSoundVariables(state);
    M_BindBaseControls(state);
    M_BindWeaponControls(state);
    M_BindMapControls(state);
    M_BindMenuControls(state);
    M_BindChatControls(state, MAXPLAYERS as u32);
    state.m_controls.key_multi_msgplayer[0] = HUSTR_KEYGREEN;
    state.m_controls.key_multi_msgplayer[1] = HUSTR_KEYINDIGO;
    state.m_controls.key_multi_msgplayer[2] = HUSTR_KEYBROWN;
    state.m_controls.key_multi_msgplayer[3] = HUSTR_KEYRED;
    M_BindVariable_int(
        &mut state.m_config,
        "mouse_sensitivity",
        &mut state.m_menu.mouseSensitivity,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "sfx_volume",
        &mut state.s_sound.sfxVolume,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "music_volume",
        &mut state.s_sound.musicVolume,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "show_messages",
        &mut state.m_menu.showMessages,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "screenblocks",
        &mut state.m_menu.screenblocks,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "detaillevel",
        &mut state.m_menu.detailLevel,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "snd_channels",
        &mut state.s_sound.snd_channels,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "vanilla_savegame_limit",
        &mut state.g_game.vanilla_savegame_limit,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "vanilla_demo_limit",
        &mut state.g_game.vanilla_demo_limit,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "show_endoom",
        &mut state.d_main.show_endoom,
    );
    i = 0_i32;
    while i < 10_i32 {
        let name = format!("chatmacro{}", i);
        M_BindVariable_string(
            &mut state.m_config,
            &name,
            &mut state.hu_stuff.chat_macros[i as usize],
        );
        i += 1;
    }
}
pub fn D_GrabMouseCallback(state: &mut GameState) -> bool {
    if drone {
        return false;
    }
    if state.m_menu.menuactive || state.g_game.paused {
        return false;
    }
    state.g_game.gamestate == GameScreenState::GS_LEVEL
        && !state.g_game.demoplayback
        && !state.d_main.advancedemo
}
pub fn doomgeneric_Tick(state: &mut GameState) {
    unsafe { TryRunTics(state) };
    let listener_id = state.g_game.players[state.g_game.consoleplayer as usize].mo;
    let listener_mo = match listener_id {
        Some(id) => state.p_mobj.mobj_get(id).unwrap(),
        None => ::core::ptr::null_mut(),
    };
    unsafe { S_UpdateSounds(state, listener_mo) };
    if state.i_video.screenvisible {
        unsafe { D_Display(state) };
    }
}
pub fn D_DoomLoop(state: &mut GameState) {
    if state.d_main.bfgedition
        && (state.g_game.demorecording
            || state.g_game.gameaction == GameAction::ga_playdemo
            || state.g_game.netgame)
    {
        println!(
            " WARNING: You are playing using one of the Doom Classic\n IWAD files shipped with the Doom 3: BFG Edition. These are\n known to be incompatible with the regular IWAD files and\n may cause demos and network games to get out of sync."
        );
    }
    if state.g_game.demorecording {
        unsafe { G_BeginRecording(state) };
    }
    state.d_main.main_loop_started = true;
    unsafe { TryRunTics(state) };
    I_SetWindowTitle(state, state.doomstat.gamedescription);
    I_SetGrabMouseCallback();
    unsafe { I_InitGraphics(state) };
    unsafe { R_ExecuteSetViewSize(state) };
    D_StartGameLoop(state);
    if state.g_game.testcontrols {
        state.d_main.wipegamestate = state.g_game.gamestate;
    }
    doomgeneric_Tick(state);
}
pub fn D_PageTicker(state: &mut GameState) {
    state.d_main.pagetic -= 1;
    if state.d_main.pagetic < 0_i32 {
        D_AdvanceDemo(state);
    }
}
pub unsafe fn D_PageDrawer(state: &mut GameState) {
    let __wcache609_1 = W_CacheLumpName(state, state.d_main.pagename) as *mut patch_t;
    let dest_screen = state.i_video.I_VideoBuffer.as_mut_ptr();
    V_DrawPatch(state, dest_screen, 0_i32, 0_i32, __wcache609_1);
}
pub fn D_AdvanceDemo(state: &mut GameState) {
    state.d_main.advancedemo = true;
}
pub fn D_DoAdvanceDemo(state: &mut GameState) {
    state.g_game.players[state.g_game.consoleplayer as usize].playerstate = PlayerState::PST_LIVE;
    state.d_main.advancedemo = false;
    state.g_game.usergame = false;
    state.g_game.paused = false;
    state.g_game.gameaction = GameAction::ga_nothing;
    if [GameVersion::ultimate, GameVersion::r#final].contains(&state.doomstat.gameversion) {
        state.d_main.demosequence = (state.d_main.demosequence + 1_i32) % 7_i32;
    } else {
        state.d_main.demosequence = (state.d_main.demosequence + 1_i32) % 6_i32;
    }
    match state.d_main.demosequence {
        0 => {
            if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
                state.d_main.pagetic = TICRATE * 11_i32;
            } else {
                state.d_main.pagetic = 170_i32;
            }
            state.g_game.gamestate = GameScreenState::GS_DEMOSCREEN;
            state.d_main.pagename = "TITLEPIC";
            if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
                S_StartMusic(state, mus_dm2ttl as i32);
            } else {
                S_StartMusic(state, mus_intro as i32);
            }
        }
        1 => {
            G_DeferedPlayDemo(state, FixedCStr::new("demo1"));
        }
        2 => {
            state.d_main.pagetic = 200_i32;
            state.g_game.gamestate = GameScreenState::GS_DEMOSCREEN;
            state.d_main.pagename = "CREDIT";
        }
        3 => {
            G_DeferedPlayDemo(state, FixedCStr::new("demo2"));
        }
        4 => {
            state.g_game.gamestate = GameScreenState::GS_DEMOSCREEN;
            if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
                state.d_main.pagetic = TICRATE * 11_i32;
                state.d_main.pagename = "TITLEPIC";
                S_StartMusic(state, mus_dm2ttl as i32);
            } else {
                state.d_main.pagetic = 200_i32;
                if state.doomstat.gamemode as u32 == GameMode_t::retail as i32 as u32 {
                    state.d_main.pagename = "CREDIT";
                } else {
                    state.d_main.pagename = "HELP2";
                }
            }
        }
        5 => {
            G_DeferedPlayDemo(state, FixedCStr::new("demo3"));
        }
        6 => {
            G_DeferedPlayDemo(state, FixedCStr::new("demo4"));
        }
        _ => {}
    }
    if state.d_main.bfgedition
        && state.d_main.pagename.eq_ignore_ascii_case("TITLEPIC")
        && W_CheckNumForName(&mut state.w_wad, "titlepic") < 0_i32
    {
        state.d_main.pagename = "INTERPIC";
    }
}
pub fn D_StartTitle(state: &mut GameState) {
    state.g_game.gameaction = GameAction::ga_nothing;
    state.d_main.demosequence = -1_i32;
    D_AdvanceDemo(state);
}
fn SetMissionForPackName(state: &mut GameState, pack_name: &str) {
    const packs: [C2RustUnnamed_3; 3] = [
        C2RustUnnamed_3 {
            name: "doom2",
            mission: GameMission_t::doom2,
        },
        C2RustUnnamed_3 {
            name: "tnt",
            mission: GameMission_t::pack_tnt,
        },
        C2RustUnnamed_3 {
            name: "plutonia",
            mission: GameMission_t::pack_plut,
        },
    ];
    for pack in &packs {
        if pack_name.eq_ignore_ascii_case(pack.name) {
            state.doomstat.gamemission = pack.mission;
            return;
        }
    }
    println!("Valid mission packs are:");
    for pack in &packs {
        println!("\t{}", pack.name);
    }
    I_Error(&format!("Unknown mission pack name: {}", pack_name));
}
pub fn D_IdentifyVersion(state: &mut GameState) {
    if state.doomstat.gamemission as u32 == GameMission_t::none as i32 as u32 {
        let mut i: u32 = 0;
        i = 0_u32;
        while i < state.w_wad.numlumps {
            if state.w_wad.lumpinfo[i as usize]
                .name
                .eq_str_ignore_ascii_case("MAP01")
            {
                state.doomstat.gamemission = GameMission_t::doom2;
                break;
            } else if state.w_wad.lumpinfo[i as usize]
                .name
                .eq_str_ignore_ascii_case("E1M1")
            {
                state.doomstat.gamemission = GameMission_t::doom;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        if state.doomstat.gamemission as u32 == GameMission_t::none as i32 as u32 {
            I_Error("Unknown or invalid IWAD file.");
        }
    }
    if (if state.doomstat.gamemission as u32 == GameMission_t::pack_chex as i32 as u32 {
        GameMission_t::doom as i32 as u32
    } else if state.doomstat.gamemission as u32 == GameMission_t::pack_hacx as i32 as u32 {
        GameMission_t::doom2 as i32 as u32
    } else {
        state.doomstat.gamemission as u32
    }) == GameMission_t::doom as i32 as u32
    {
        if W_CheckNumForName(&mut state.w_wad, "E4M1") > 0_i32 {
            state.doomstat.gamemode = GameMode_t::retail;
        } else if W_CheckNumForName(&mut state.w_wad, "E3M1") > 0_i32 {
            state.doomstat.gamemode = GameMode_t::registered;
        } else {
            state.doomstat.gamemode = GameMode_t::shareware;
        }
    } else {
        let mut p: i32 = 0;
        state.doomstat.gamemode = GameMode_t::commercial;
        p = M_CheckParmWithArgs(state, "-pack", 1_i32);
        if p > 0_i32 {
            let pack_name = state.m_argv.myargv[(p + 1_i32) as usize]
                .to_str()
                .unwrap()
                .to_string();
            SetMissionForPackName(state, &pack_name);
        }
    };
}
pub fn D_SetGameDescription(state: &mut GameState) {
    let mut is_freedoom: bool = W_CheckNumForName(&mut state.w_wad, "FREEDOOM") >= 0_i32;
    let mut is_freedm: bool = W_CheckNumForName(&mut state.w_wad, "FREEDM") >= 0_i32;
    state.doomstat.gamedescription = "Unknown";
    if (if state.doomstat.gamemission as u32 == GameMission_t::pack_chex as i32 as u32 {
        GameMission_t::doom as i32 as u32
    } else if state.doomstat.gamemission as u32 == GameMission_t::pack_hacx as i32 as u32 {
        GameMission_t::doom2 as i32 as u32
    } else {
        state.doomstat.gamemission as u32
    }) == GameMission_t::doom as i32 as u32
    {
        if is_freedoom {
            state.doomstat.gamedescription = "Freedoom: Phase 1";
        } else if state.doomstat.gamemode as u32 == GameMode_t::retail as i32 as u32 {
            state.doomstat.gamedescription = "The Ultimate DOOM";
        } else if state.doomstat.gamemode as u32 == GameMode_t::registered as i32 as u32 {
            state.doomstat.gamedescription = "DOOM Registered";
        } else if state.doomstat.gamemode as u32 == GameMode_t::shareware as i32 as u32 {
            state.doomstat.gamedescription = "DOOM Shareware";
        }
    } else if is_freedoom {
        if is_freedm {
            state.doomstat.gamedescription = "FreeDM";
        } else {
            state.doomstat.gamedescription = "Freedoom: Phase 2";
        }
    } else if (if state.doomstat.gamemission as u32 == GameMission_t::pack_chex as i32 as u32 {
        GameMission_t::doom as i32 as u32
    } else if state.doomstat.gamemission as u32 == GameMission_t::pack_hacx as i32 as u32 {
        GameMission_t::doom2 as i32 as u32
    } else {
        state.doomstat.gamemission as u32
    }) == GameMission_t::doom2 as i32 as u32
    {
        state.doomstat.gamedescription = "DOOM 2: Hell on Earth";
    } else if (if state.doomstat.gamemission as u32 == GameMission_t::pack_chex as i32 as u32 {
        GameMission_t::doom as i32 as u32
    } else if state.doomstat.gamemission as u32 == GameMission_t::pack_hacx as i32 as u32 {
        GameMission_t::doom2 as i32 as u32
    } else {
        state.doomstat.gamemission as u32
    }) == GameMission_t::pack_plut as i32 as u32
    {
        state.doomstat.gamedescription = "DOOM 2: Plutonia Experiment";
    } else if (if state.doomstat.gamemission as u32 == GameMission_t::pack_chex as i32 as u32 {
        GameMission_t::doom as i32 as u32
    } else if state.doomstat.gamemission as u32 == GameMission_t::pack_hacx as i32 as u32 {
        GameMission_t::doom2 as i32 as u32
    } else {
        state.doomstat.gamemission as u32
    }) == GameMission_t::pack_tnt as i32 as u32
    {
        state.doomstat.gamedescription = "DOOM 2: TNT - Evilution";
    }
}
pub static title: [::core::ffi::c_char; 128] = [0; 128];
fn D_AddFile(state: &mut GameState, filename: &str) -> bool {
    println!(" adding {}", filename);
    W_AddFile(state, filename).is_some()
}
static copyright_banners: [&str; 3] = [
    "===========================================================================\nATTENTION:  This version of DOOM has been modified.  If you would like to\nget a copy of the original game, call 1-800-IDGAMES or see the readme file.\n        You will not receive technical support for modified games.\n                      press enter to continue\n===========================================================================\n",
    "===========================================================================\n                 Commercial product - do not distribute!\n         Please report software piracy to the SPA: 1-800-388-PIR8\n===========================================================================\n",
    "===========================================================================\n                                Shareware!\n===========================================================================\n",
];
pub fn PrintDehackedBanners() {
    let mut i: size_t = 0;
    i = 0 as size_t;
    while i < copyright_banners.len() as size_t {
        let deh_s_str: &str = copyright_banners[i];
        if deh_s_str != copyright_banners[i] {
            print!("{}", deh_s_str);
            if !deh_s_str.ends_with('\n') {
                println!();
            }
        }
        i = i.wrapping_add(1);
    }
}
fn InitGameVersion(state: &mut GameState) {
    let mut p: i32 = 0;
    p = M_CheckParmWithArgs(state, "-gameversion", 1_i32);
    if p != 0 {
        let arg = state.m_argv.myargv[(p + 1_i32) as usize].as_bytes();
        let found = state
            .d_main
            .gameversions
            .iter()
            .find(|gv| gv.cmdline.as_bytes() == arg);
        if let Some(gv) = found {
            state.doomstat.gameversion = gv.version;
        } else {
            println!("Supported game versions:");
            for gv in &state.d_main.gameversions {
                println!("\t{} ({})", gv.cmdline, gv.description);
            }
            I_Error(&format!(
                "Unknown game version '{}'",
                state.m_argv.myargv[(p + 1_i32) as usize].to_str().unwrap(),
            ));
        }
    } else if state.doomstat.gamemission as u32 == GameMission_t::pack_chex as i32 as u32 {
        state.doomstat.gameversion = GameVersion::chex;
    } else if state.doomstat.gamemission as u32 == GameMission_t::pack_hacx as i32 as u32 {
        state.doomstat.gameversion = GameVersion::hacx;
    } else if state.doomstat.gamemode as u32 == GameMode_t::shareware as i32 as u32
        || state.doomstat.gamemode as u32 == GameMode_t::registered as i32 as u32
    {
        state.doomstat.gameversion = GameVersion::doom_1_9;
    } else if state.doomstat.gamemode as u32 == GameMode_t::retail as i32 as u32 {
        state.doomstat.gameversion = GameVersion::ultimate;
    } else if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        if state.doomstat.gamemission as u32 == GameMission_t::doom2 as i32 as u32 {
            state.doomstat.gameversion = GameVersion::doom_1_9;
        } else {
            state.doomstat.gameversion = GameVersion::r#final;
        }
    }
    if state.doomstat.gameversion == GameVersion::ultimate
        && state.doomstat.gamemode as u32 == GameMode_t::retail as i32 as u32
    {
        state.doomstat.gamemode = GameMode_t::registered;
    }
    if (state.doomstat.gameversion as u32) < GameVersion::r#final as u32
        && state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32
        && (state.doomstat.gamemission as u32 == GameMission_t::pack_tnt as i32 as u32
            || state.doomstat.gamemission as u32 == GameMission_t::pack_plut as i32 as u32)
    {
        state.doomstat.gamemission = GameMission_t::doom2;
    }
}
pub fn PrintGameVersion(state: &mut GameState) {
    if let Some(gv) = state
        .d_main
        .gameversions
        .iter()
        .find(|gv| gv.version == state.doomstat.gameversion)
    {
        println!(
            "Emulating the behavior of the '{}' executable.",
            gv.description
        );
    }
}
fn D_Endoom(state: &mut GameState) {
    if state.d_main.show_endoom == 0
        || !state.d_main.main_loop_started
        || state.i_video.screensaver_mode
        || M_CheckParm(state, "-testcontrols") > 0
    {
        return;
    }
    std::process::exit(0);
}
fn D_QuitCheckDemoStatus(state: &mut GameState) {
    unsafe { G_CheckDemoStatus(state) };
}
pub unsafe fn D_DoomMain(state: &mut GameState) {
    let mut p: i32 = 0;
    let mut file: String = String::new();
    let mut demolumpname: FixedCStr<8> = FixedCStr::from_array([0; 8]);
    I_AtExit(
        &mut state.i_system,
        Some(D_Endoom as fn(&mut GameState) -> ()),
        false,
    );
    I_PrintBanner(&PACKAGE_STRING.as_str());
    state.d_main.nomonsters = M_CheckParm(state, "-nomonsters") != 0;
    state.d_main.respawnparm = M_CheckParm(state, "-respawn") != 0;
    state.d_main.fastparm = M_CheckParm(state, "-fast") != 0;
    state.d_main.devparm = M_CheckParm(state, "-devparm") != 0;
    if M_CheckParm(state, "-deathmatch") != 0 {
        state.g_game.deathmatch = 1_i32;
    }
    if M_CheckParm(state, "-altdeath") != 0 {
        state.g_game.deathmatch = 2_i32;
    }
    if state.d_main.devparm {
        print!("{}", D_DEVSTR.as_str());
    }
    M_SetConfigDir(&mut state.m_config, None);
    p = M_CheckParm(state, "-turbo");
    if p != 0 {
        let mut scale: i32 = 200_i32;
        if p < state.m_argv.myargv.len() as i32 - 1_i32 {
            scale = M_ArgvAtoi(&state.m_argv.myargv[(p + 1_i32) as usize]);
        }
        if scale < 10_i32 {
            scale = 10_i32;
        }
        if scale > 400_i32 {
            scale = 400_i32;
        }
        println!("turbo scale: {}%", scale);
        state.g_game.forwardmove[0] = state.g_game.forwardmove[0] * scale / 100_i32;
        state.g_game.forwardmove[1] = state.g_game.forwardmove[1] * scale / 100_i32;
        state.g_game.sidemove[0] = state.g_game.sidemove[0] * scale / 100_i32;
        state.g_game.sidemove[1] = state.g_game.sidemove[1] * scale / 100_i32;
    }
    println!("V_Init: allocate screens.");
    println!("M_LoadDefaults: Load system defaults.");
    M_SetConfigFilenames(&mut state.m_config, "default.cfg", "doomgenericdoom.cfg");
    D_BindVariables(state);
    M_LoadDefaults(state);
    I_AtExit(
        &mut state.i_system,
        Some(M_SaveDefaults as fn(&mut GameState) -> ()),
        false,
    );
    let mut gamemission_out = state.doomstat.gamemission;
    state.d_main.iwadfile = D_FindIWAD(
        state,
        1_i32 << GameMission_t::doom as i32
            | 1_i32 << GameMission_t::doom2 as i32
            | 1_i32 << GameMission_t::pack_tnt as i32
            | 1_i32 << GameMission_t::pack_plut as i32
            | 1_i32 << GameMission_t::pack_chex as i32
            | 1_i32 << GameMission_t::pack_hacx as i32,
        &mut gamemission_out,
    );
    state.doomstat.gamemission = gamemission_out;
    if state.d_main.iwadfile.is_empty() {
        I_Error(
            "Game mode indeterminate.  No IWAD file was found.  Try\nspecifying one with the '-iwad' command line parameter.\n",
        );
    }
    state.doomstat.modifiedgame = false;
    println!("W_Init: Init WADfiles.");
    let iwadfile = state.d_main.iwadfile.clone();
    D_AddFile(state, &iwadfile);
    W_CheckCorrectIWAD(&mut state.w_wad, GameMission_t::doom);
    D_IdentifyVersion(state);
    InitGameVersion(state);
    if W_CheckNumForName(&mut state.w_wad, "dmenupic") >= 0_i32 {
        println!("BFG Edition: Using workarounds as needed.");
        state.d_main.bfgedition = true;
    }
    let modifiedgame = W_ParseCommandLine(state);
    state.doomstat.modifiedgame = modifiedgame;
    p = M_CheckParmWithArgs(state, "-playdemo", 1_i32);
    if p == 0 {
        p = M_CheckParmWithArgs(state, "-timedemo", 1_i32);
    }
    if p != 0 {
        let arg = state.m_argv.myargv[(p + 1_i32) as usize].to_str().unwrap();
        if M_StringEndsWith(arg, ".lmp") {
            file = arg.to_string();
        } else {
            file = format!("{}.lmp", arg);
        }
        if D_AddFile(state, &file) {
            demolumpname =
                state.w_wad.lumpinfo[state.w_wad.numlumps.wrapping_sub(1_u32) as usize].name;
        } else {
            let src_bytes = state.m_argv.myargv[(p + 1_i32) as usize].as_bytes();
            demolumpname = FixedCStr::from_bytes(src_bytes);
        }
        println!("Playing demo {}.", file);
    }
    I_AtExit(
        &mut state.i_system,
        Some(D_QuitCheckDemoStatus as fn(&mut GameState) -> ()),
        true,
    );
    W_GenerateHashTable(state);
    D_SetGameDescription(state);
    state.d_main.savegamedir = M_GetSaveGameDir(
        &mut state.m_config,
        D_SaveGameIWADName(state.doomstat.gamemission),
    );
    if state.doomstat.modifiedgame {
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
        let mut i: i32 = 0;
        if state.doomstat.gamemode as u32 == GameMode_t::shareware as i32 as u32 {
            I_Error("\nYou cannot -file with the shareware version. Register!");
        }
        if state.doomstat.gamemode as u32 == GameMode_t::registered as i32 as u32 {
            i = 0_i32;
            while i < 23_i32 {
                if W_CheckNumForName(&mut state.w_wad, &name[i as usize].as_str()) < 0_i32 {
                    I_Error("\nThis is not the registered version.");
                }
                i += 1;
            }
        }
    }
    if W_CheckNumForName(&mut state.w_wad, "SS_START") >= 0_i32
        || W_CheckNumForName(&mut state.w_wad, "FF_END") >= 0_i32
    {
        I_PrintDivider();
        println!(
            " WARNING: The loaded WAD file contains modified sprites or\n floor textures.  You may want to use the '-merge' command\n line option instead of '-file'."
        );
    }
    I_PrintStartupBanner(state.doomstat.gamedescription);
    PrintDehackedBanners();
    if W_CheckNumForName(&mut state.w_wad, "FREEDOOM") >= 0_i32
        && W_CheckNumForName(&mut state.w_wad, "FREEDM") < 0_i32
    {
        println!(
            " WARNING: You are playing using one of the Freedoom IWAD\n files, which might not work in this port. See this page\n for more information on how to play using Freedoom:\n   http://www.chocolate-doom.org/wiki/index.php/Freedoom"
        );
        I_PrintDivider();
    }
    println!("I_Init: Setting up machine state.");
    I_InitSound(state, true);
    I_InitMusic(&mut state.i_sound);
    D_ConnectNetGame(state);
    state.d_main.startskill = SkillType::sk_medium;
    state.d_main.startepisode = 1_i32;
    state.d_main.startmap = 1_i32;
    state.d_main.autostart = false;
    p = M_CheckParmWithArgs(state, "-skill", 1_i32);
    if p != 0 {
        state.d_main.startskill = skill_from_raw(
            state.m_argv.myargv[(p + 1_i32) as usize]
                .as_bytes()
                .first()
                .copied()
                .unwrap_or(0) as i32
                - '1' as i32,
        );
        state.d_main.autostart = true;
    }
    p = M_CheckParmWithArgs(state, "-episode", 1_i32);
    if p != 0 {
        state.d_main.startepisode = state.m_argv.myargv[(p + 1_i32) as usize]
            .as_bytes()
            .first()
            .copied()
            .unwrap_or(0) as i32
            - '0' as i32;
        state.d_main.startmap = 1_i32;
        state.d_main.autostart = true;
    }
    state.g_game.timelimit = 0_i32;
    p = M_CheckParmWithArgs(state, "-timer", 1_i32);
    if p != 0 {
        state.g_game.timelimit = M_ArgvAtoi(&state.m_argv.myargv[(p + 1_i32) as usize]);
    }
    p = M_CheckParm(state, "-avg");
    if p != 0 {
        state.g_game.timelimit = 20_i32;
    }
    p = M_CheckParmWithArgs(state, "-warp", 1_i32);
    if p != 0 {
        if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
            state.d_main.startmap = M_ArgvAtoi(&state.m_argv.myargv[(p + 1_i32) as usize]);
        } else {
            state.d_main.startepisode = state.m_argv.myargv[(p + 1_i32) as usize]
                .as_bytes()
                .first()
                .copied()
                .unwrap_or(0) as i32
                - '0' as i32;
            if (p + 2_i32) < state.m_argv.myargv.len() as i32 {
                state.d_main.startmap = state.m_argv.myargv[(p + 2_i32) as usize]
                    .as_bytes()
                    .first()
                    .copied()
                    .unwrap_or(0) as i32
                    - '0' as i32;
            } else {
                state.d_main.startmap = 1_i32;
            }
        }
        state.d_main.autostart = true;
    }
    p = M_CheckParm(state, "-testcontrols");
    if p > 0_i32 {
        state.d_main.startepisode = 1_i32;
        state.d_main.startmap = 1_i32;
        state.d_main.autostart = true;
        state.g_game.testcontrols = true;
    }
    p = M_CheckParmWithArgs(state, "-loadgame", 1_i32);
    if p != 0 {
        state.d_main.startloadgame = M_ArgvAtoi(&state.m_argv.myargv[(p + 1_i32) as usize]);
    } else {
        state.d_main.startloadgame = -1_i32;
    }
    println!("M_Init: Init miscellaneous info.");
    M_Init(state);
    print!("R_Init: Init DOOM refresh daemon - ");
    R_Init(state);
    println!();
    println!("P_Init: Init Playloop state.");
    P_Init(state);
    println!("S_Init: Setting up sound.");
    S_Init(
        state,
        state.s_sound.sfxVolume * 8_i32,
        state.s_sound.musicVolume * 8_i32,
    );
    println!("D_CheckNetGame: Checking network game status.");
    D_CheckNetGame(state);
    PrintGameVersion(state);
    println!("HU_Init: Setting up heads up display.");
    HU_Init(state);
    println!("ST_Init: Init status bar.");
    ST_Init(state);
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32
        && W_CheckNumForName(&mut state.w_wad, "map01") < 0_i32
    {
        state.d_main.storedemo = true;
    }
    if M_CheckParmWithArgs(state, "-statdump", 1_i32) != 0 {
        I_AtExit(
            &mut state.i_system,
            Some(StatDump as fn(&mut GameState) -> ()),
            true,
        );
        println!("External statistics registered.");
    }
    p = M_CheckParmWithArgs(state, "-record", 1_i32);
    if p != 0 {
        let record_name = state.m_argv.myargv[(p + 1_i32) as usize]
            .to_str()
            .unwrap()
            .to_string();
        G_RecordDemo(state, &record_name);
        state.d_main.autostart = true;
    }
    p = M_CheckParmWithArgs(state, "-playdemo", 1_i32);
    if p != 0 {
        state.g_game.singledemo = true;
        G_DeferedPlayDemo(state, demolumpname);
        D_DoomLoop(state);
        return;
    }
    p = M_CheckParmWithArgs(state, "-timedemo", 1_i32);
    if p != 0 {
        G_TimeDemo(state, demolumpname);
        D_DoomLoop(state);
        return;
    }
    if state.d_main.startloadgame >= 0_i32 {
        let savegame_file = P_SaveGameFile(state, state.d_main.startloadgame);
        G_LoadGame(state, &savegame_file);
    }
    if state.g_game.gameaction != GameAction::ga_loadgame {
        if state.d_main.autostart || state.g_game.netgame {
            let (startskill, startepisode, startmap) = (
                state.d_main.startskill,
                state.d_main.startepisode,
                state.d_main.startmap,
            );
            G_InitNew(state, startskill, startepisode, startmap);
        } else {
            D_StartTitle(state);
        }
    }
    D_DoomLoop(state);
}
