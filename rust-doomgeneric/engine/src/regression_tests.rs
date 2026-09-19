//! Golden-output regression tests.
//!
//! These run the real engine headlessly against the shareware `doom1.wad` and
//! compare hashes of the simulation state and of rendered frames with values
//! recorded in `golden/regression.txt`. They exist so that refactors that must
//! not change behaviour can be checked with a plain `cargo test --release`.
//!
//! The IWAD is not part of the repository: it is read from `$DOOM_IWAD`, or
//! `$HOME/Downloads/doom1.wad`. If neither exists the tests print a note and
//! pass vacuously, so CI without the WAD stays green.
//!
//! Set `UPDATE_GOLDEN=1` to rewrite the golden file from the current output.
//! Only do that when a behaviour change is intended and understood.

use crate::d_event::{event_t, D_PostEvent, EvType};
use crate::d_main::doomgeneric_Tick;
use crate::doomdef::pixel_t;
use crate::doomgeneric::doomgeneric_Create;
use crate::filesystem::MemFileSystem;
use crate::g_game::{G_DoLoadGame, G_DoSaveGame, G_ExitLevel, G_LoadGame, G_SaveGame};
use crate::game_state::{init_game_state, GameState};
use crate::p_saveg::P_SaveGameFile;
use crate::p_setup::SectorId;
use crate::p_tick::P_MobjThinkerIds;
use crate::platform::DoomPlatform;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::fmt::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};

const GOLDEN_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/golden/regression.txt");
/// Generous upper bound on `doomgeneric_Tick` calls for the longest demo.
const MAX_TICK_CALLS: u32 = 200_000;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// A platform with a virtual clock that advances by a millisecond per query, so
/// the engine's wait loops always terminate and every run sees the same timing.
#[derive(Default)]
struct NullPlatform {
    now_ms: u32,
}

impl DoomPlatform for NullPlatform {
    fn init(&mut self, _screen_buffer: *mut pixel_t, _resx: i32, _resy: i32) {}
    fn draw_frame(&mut self) {}
    fn sleep_ms(&mut self, ms: u32) {
        self.now_ms += ms;
    }
    fn get_ticks_ms(&mut self) -> u32 {
        self.now_ms += 1;
        self.now_ms
    }
    fn get_key(&mut self) -> Option<(bool, u8)> {
        None
    }
    fn set_window_title(&mut self, _title: &str) {}
    fn print(&mut self, _message: &str) {}
    fn eprint(&mut self, _message: &str) {}
    fn quit(&mut self) -> ! {
        panic!("quit")
    }
}

fn fnv(mut hash: u64, values: impl IntoIterator<Item = u32>) -> u64 {
    for v in values {
        hash = (hash ^ u64::from(v)).wrapping_mul(FNV_PRIME);
    }
    hash
}

fn fnv_bytes(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(FNV_PRIME)
    })
}

fn iwad_bytes() -> Option<Vec<u8>> {
    let path = std::env::var("DOOM_IWAD").ok().or_else(|| {
        std::env::var("HOME")
            .ok()
            .map(|home| format!("{home}/Downloads/doom1.wad"))
    })?;
    std::fs::read(path).ok()
}

/// A freshly created engine reading the IWAD from memory, plus a handle for
/// the panic the engine raises when a timedemo finishes.
fn start(args: &[&str]) -> Option<&'static mut GameState> {
    let wad = iwad_bytes()?;
    let mut fs = MemFileSystem::default();
    fs.files.insert("doom1.wad".to_string(), wad);
    let state = init_game_state(Box::new(NullPlatform::default()), Box::new(fs));
    let mut argv: Vec<String> = alloc::vec![
        "doom".to_string(),
        "-iwad".to_string(),
        "doom1.wad".to_string()
    ];
    argv.extend(args.iter().map(|a| (*a).to_string()));
    doomgeneric_Create(state, argv);
    Some(state)
}

/// Ticks the engine until it stops with a panic (the end of a timedemo) and
/// returns the panic message. `before_tick` runs ahead of every tick.
fn run_until_exit(state: &mut GameState, mut before_tick: impl FnMut(&mut GameState)) -> String {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        for _ in 0..MAX_TICK_CALLS {
            before_tick(state);
            doomgeneric_Tick(state);
        }
        panic!("runaway: the demo did not finish within {MAX_TICK_CALLS} ticks");
    }));
    let payload = outcome.expect_err("the engine loop never returns normally");
    if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else {
        String::new()
    }
}

/// Hash of the world as a save game captures it: every live mobj, every
/// sector and the console player's vitals.
fn world_summary(state: &mut GameState) -> String {
    let mut mobj_hash = FNV_OFFSET;
    let mut count = 0;
    for id in P_MobjThinkerIds(state) {
        let m = state.p_mobj.mo(id);
        mobj_hash = fnv(
            mobj_hash,
            [
                m.x,
                m.y,
                m.z,
                m.health,
                m.momx,
                m.momy,
                m.kind as i32,
                m.flags,
                m.tics,
                m.movecount,
            ]
            .map(|v| v as u32),
        );
        count += 1;
    }
    let mut sector_hash = FNV_OFFSET;
    for i in 0..state.p_setup.numsectors {
        let s = state.p_setup.sector_mut(SectorId(i as u32));
        sector_hash = fnv(
            sector_hash,
            [s.floorheight, s.ceilingheight, i32::from(s.lightlevel)].map(|v| v as u32),
        );
    }
    let p = &state.g_game.players[state.g_game.consoleplayer as usize];
    let player_hash = fnv(
        FNV_OFFSET,
        [p.health, p.armorpoints, p.viewz, p.readyweapon as i32].map(|v| v as u32),
    );
    format!("mobjs={count} mobjhash={mobj_hash:016x} sechash={sector_hash:016x} playerhash={player_hash:016x}")
}

/// [`world_summary`] plus the random-number indices and the tic count.
fn simulation_summary(state: &mut GameState) -> String {
    format!(
        "{} prnd={} rnd={} gametic={}",
        world_summary(state),
        state.m_random.prndindex,
        state.m_random.rndindex,
        state.d_loop.gametic
    )
}

fn key(state: &mut GameState, k: i32) {
    D_PostEvent(
        &mut state.d_event,
        event_t {
            kind: EvType::ev_keydown,
            data1: k,
            data2: 0,
            data3: 0,
            data4: 0,
        },
    );
}

/// Drives menus, the automap, cheats, intermissions and finales during
/// `-timedemo demo3` so that the frame hashes cover the whole UI.
fn scripted_input(state: &mut GameState) {
    let g = state.d_loop.gametic;
    if g == 50 {
        state.g_game.singledemo = true;
    }
    match g {
        100 | 200 => key(state, 9),
        120 | 620 | 625 | 630 | 635 | 640 => key(state, 0x3d),
        130 => key(state, i32::from(b'g')),
        140 => key(state, i32::from(b'f')),
        150 => key(state, i32::from(b'm')),
        160 | 600 | 605 => key(state, 0x2d),
        400 | 425 | 430 | 460 | 490 | 510 | 540 => key(state, 27),
        405 | 410 => key(state, 0xaf),
        415 => key(state, 13),
        450 => key(state, 0x80 + 0x3b),
        480 => key(state, 0x80 + 0x3c),
        500 => key(state, 0x80 + 0x3e),
        530 => key(state, 0x80 + 0x3d),
        550 | 555 | 560 | 565 => key(state, 0x80 + 0x57),
        650 | 655 => key(state, 0x80 + 0x3f),
        660 => key(state, 0x80 + 0x41),
        670 | 710 => key(state, i32::from(b'n')),
        680 | 690 => key(state, 0x80 + 0x42),
        700 => key(state, 0x80 + 0x44),
        750 => key(state, i32::from(b'i')),
        752 | 754 | 758 | 772 => key(state, i32::from(b'd')),
        756 => key(state, i32::from(b'q')),
        770 => key(state, i32::from(b'i')),
        774 => key(state, i32::from(b'k')),
        776 => key(state, i32::from(b'f')),
        778 => key(state, i32::from(b'a')),
        800 => G_ExitLevel(state),
        950 | 1050 | 1150 | 1250 | 1350 | 1950 | 2050 | 2150 | 2250 | 2350 | 3150 | 3250 | 3350
        | 3450 | 3550 => {
            state.wi_stuff.acceleratestage = 1;
        }
        1800 => {
            state.g_game.gameepisode = 1;
            state.g_game.gamemap = 8;
            G_ExitLevel(state);
        }
        3000 => {
            state.g_game.gameepisode = 3;
            state.g_game.gamemap = 8;
            G_ExitLevel(state);
        }
        3600 => state.f_finale.finalecount = 5000,
        3650 => state.f_finale.finalecount = 300,
        3700 => state.f_finale.finalecount = 700,
        3750 => state.f_finale.finalecount = 1150,
        3800 => state.f_finale.finalecount = 1200,
        _ => {}
    }
}

fn demo_summary(demo: &str) -> Option<String> {
    let state = start(&["-timedemo", demo])?;
    let message = run_until_exit(state, |_| {});
    assert!(message.starts_with("timed "), "unexpected exit: {message}");
    Some(simulation_summary(state))
}

/// One line per checkpoint: `frame <tic> <hash of the 8-bit frame>` every 5
/// tics and `dg <tic> <hash of the platform framebuffer>` every 100.
fn ui_frame_lines() -> Option<Vec<String>> {
    let state = start(&["-timedemo", "demo3"])?;
    let mut lines = Vec::new();
    let message = run_until_exit(state, |state| {
        scripted_input(state);
        let g = state.d_loop.gametic;
        if g % 5 == 0 {
            lines.push(format!(
                "frame {g:05} {:016x}",
                fnv_bytes(&state.i_video.I_VideoBuffer)
            ));
            if g % 100 == 0 {
                let bytes: Vec<u8> = state
                    .i_video
                    .dg_screen_buffer
                    .iter()
                    .flat_map(|p| p.to_le_bytes())
                    .collect();
                lines.push(format!("dg {g:05} {:016x}", fnv_bytes(&bytes)));
            }
        }
    });
    assert!(message.starts_with("timed "), "unexpected exit: {message}");
    Some(lines)
}

fn actual_output() -> Option<String> {
    let mut out = String::new();
    for demo in ["demo1", "demo2", "demo3"] {
        writeln!(out, "sim {demo} {}", demo_summary(demo)?).unwrap();
    }
    for line in ui_frame_lines()? {
        writeln!(out, "{line}").unwrap();
    }
    Some(out)
}

#[test]
fn simulation_and_frames_match_golden() {
    let Some(actual) = actual_output() else {
        std::eprintln!("skipping: no IWAD (set DOOM_IWAD or put doom1.wad in ~/Downloads)");
        return;
    };
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::create_dir_all(concat!(env!("CARGO_MANIFEST_DIR"), "/golden")).unwrap();
        std::fs::write(GOLDEN_PATH, &actual).unwrap();
        return;
    }
    let golden = std::fs::read_to_string(GOLDEN_PATH)
        .expect("golden/regression.txt (run with UPDATE_GOLDEN=1)");
    for (n, (want, got)) in golden.lines().zip(actual.lines()).enumerate() {
        assert_eq!(want, got, "first difference at golden line {}", n + 1);
    }
    assert_eq!(
        golden.lines().count(),
        actual.lines().count(),
        "different number of checkpoints"
    );
}

/// Saves a game 350 tics into E1M1 and loads it back: the world must come
/// back exactly as it was written.
#[test]
fn save_game_round_trips() {
    let Some(state) = start(&["-warp", "1", "1", "-skill", "3"]) else {
        std::eprintln!("skipping: no IWAD (set DOOM_IWAD or put doom1.wad in ~/Downloads)");
        return;
    };
    while state.d_loop.gametic < 350 {
        doomgeneric_Tick(state);
    }
    let before = world_summary(state);
    // Save directly, not through the deferred `sendsave` request, which would
    // trigger a second save on the next tick.
    G_SaveGame(state, 0, "roundtrip");
    state.g_game.sendsave = false;
    G_DoSaveGame(state);
    let path = P_SaveGameFile(state, 0);
    assert!(state.fs.exists(&path), "no save file at {path}");

    // Let the world move on, then restore it from the file.
    for _ in 0..40 {
        doomgeneric_Tick(state);
    }
    assert_ne!(
        before,
        world_summary(state),
        "the world should have changed"
    );
    G_LoadGame(state, &path);
    G_DoLoadGame(state);
    assert_eq!(before, world_summary(state));
}
