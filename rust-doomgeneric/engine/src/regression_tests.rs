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

use crate::d_event::{post_event, EvType, Event};
use crate::d_main::doomgeneric_tick;
use crate::doomdef::{Pixel, SCREENHEIGHT, SCREENWIDTH};
use crate::doomgeneric::{doomgeneric_create, DOOMGENERIC_RESX};
use crate::f_finale::cast_ticker;
use crate::filesystem::{read_file, MemFileSystem};
use crate::g_game::{do_load_game, do_save_game, exit_level, g_load_game, g_save_game};
use crate::game_state::{init_game_state, GameState};
use crate::info::StateId;
use crate::p_mobj::MobjFlags;
use crate::p_saveg::save_game_file;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_switch::use_special_line;
use crate::p_tick::mobj_thinker_ids;
use crate::platform::DoomPlatform;
use crate::r_main::set_view_size;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::cell::RefCell;
use std::fmt::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

const GOLDEN_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/golden/regression.txt");
/// Generous upper bound on `doomgeneric_tick` calls for the longest demo.
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
    fn init(&mut self, _resx: i32, _resy: i32) {}
    fn draw_frame(&mut self, _frame: &[Pixel]) {}
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

/// A [`NullPlatform`] that hashes every frame it is given. Whichever way the
/// engine delivers a frame, the hash is of the same thing: the 320 x 200 image
/// as `0x00RRGGBB` pixels. With `indexed` it accepts the engine's indexed
/// frame; without, it takes the scaled 32-bit frame from `draw_frame`.
struct HashingPlatform {
    inner: NullPlatform,
    indexed: bool,
    hashes: Rc<RefCell<Vec<u64>>>,
}

impl DoomPlatform for HashingPlatform {
    fn init(&mut self, resx: i32, resy: i32) {
        self.inner.init(resx, resy);
    }
    fn draw_frame(&mut self, frame: &[Pixel]) {
        assert!(!self.indexed, "an indexed platform must not get draw_frame");
        // Auto-scaling on a 640 x 400 frame doubles every pixel both ways.
        let (width, height) = (SCREENWIDTH as usize, SCREENHEIGHT as usize);
        let stride = DOOMGENERIC_RESX as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                let p = frame[2 * y * stride + 2 * x];
                assert_eq!(p, frame[2 * y * stride + 2 * x + 1]);
                assert_eq!(p, frame[(2 * y + 1) * stride + 2 * x]);
                assert_eq!(p, frame[(2 * y + 1) * stride + 2 * x + 1]);
                pixels.push(p);
            }
        }
        self.hashes.borrow_mut().push(fnv(FNV_OFFSET, pixels));
    }
    fn draw_indexed_frame(&mut self, indices: &[u8], palette: &[Pixel; 256]) -> bool {
        if !self.indexed {
            return false;
        }
        assert_eq!(indices.len(), (SCREENWIDTH * SCREENHEIGHT) as usize);
        let hash = fnv(FNV_OFFSET, indices.iter().map(|&i| palette[usize::from(i)]));
        self.hashes.borrow_mut().push(hash);
        true
    }
    fn sleep_ms(&mut self, ms: u32) {
        self.inner.sleep_ms(ms);
    }
    fn get_ticks_ms(&mut self) -> u32 {
        self.inner.get_ticks_ms()
    }
    fn get_key(&mut self) -> Option<(bool, u8)> {
        self.inner.get_key()
    }
    fn set_window_title(&mut self, title: &str) {
        self.inner.set_window_title(title);
    }
    fn print(&mut self, message: &str) {
        self.inner.print(message);
    }
    fn eprint(&mut self, message: &str) {
        self.inner.eprint(message);
    }
    fn quit(&mut self) -> ! {
        self.inner.quit()
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

pub(crate) fn iwad_bytes() -> Option<Vec<u8>> {
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
    start_with(Box::new(NullPlatform::default()), args)
}

fn start_with(platform: Box<dyn DoomPlatform>, args: &[&str]) -> Option<&'static mut GameState> {
    let wad = iwad_bytes()?;
    let mut fs = MemFileSystem::default();
    fs.files.insert("doom1.wad".to_string(), wad);
    let state = init_game_state(platform, Box::new(fs));
    let mut argv: Vec<String> = alloc::vec![
        "doom".to_string(),
        "-iwad".to_string(),
        "doom1.wad".to_string()
    ];
    argv.extend(args.iter().map(|a| (*a).to_string()));
    doomgeneric_create(state, argv);
    Some(state)
}

/// Ticks the engine until it stops with a panic (the end of a timedemo) and
/// returns the panic message. `before_tick` runs ahead of every tick.
fn run_until_exit(state: &mut GameState, mut before_tick: impl FnMut(&mut GameState)) -> String {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        for _ in 0..MAX_TICK_CALLS {
            before_tick(state);
            doomgeneric_tick(state);
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
    for id in mobj_thinker_ids(&state.p_mobj, &state.p_tick) {
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
                m.flags.bits(),
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
    post_event(
        &mut state.d_event,
        Event {
            kind: EvType::Keydown,
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
        800 => exit_level(&mut state.g_game),
        950 | 1050 | 1150 | 1250 | 1350 | 1950 | 2050 | 2150 | 2250 | 2350 | 3150 | 3250 | 3350
        | 3450 | 3550 => {
            state.wi_stuff.acceleratestage = true;
        }
        1800 => {
            state.g_game.gameepisode = 1;
            state.g_game.gamemap = 8;
            exit_level(&mut state.g_game);
        }
        3000 => {
            state.g_game.gameepisode = 3;
            state.g_game.gamemap = 8;
            exit_level(&mut state.g_game);
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
                fnv_bytes(&state.i_video.i_video_buffer)
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

/// `frame` lines for demo1 at the given detail level (0 = high, 1 = low) with
/// every monster made invisible or colour-translated in turn. The plain demos
/// take neither the fuzz nor the translated column drawers, and never leave
/// high detail, so this is what covers those drawing paths.
fn effect_frame_lines(detail: i32) -> Option<Vec<String>> {
    let state = start(&["-timedemo", "demo1"])?;
    let mut lines = Vec::new();
    let message = run_until_exit(state, |state| {
        let g = state.d_loop.gametic;
        if g == 10 {
            let blocks = state.m_menu.screenblocks;
            set_view_size(&mut state.r_main, blocks, detail);
        }
        let ids = mobj_thinker_ids(&state.p_mobj, &state.p_tick);
        for (n, id) in ids.into_iter().enumerate() {
            let flags = &mut state.p_mobj.mo_mut(id).flags;
            if flags.contains(MobjFlags::COUNTKILL) {
                *flags |= match n % 3 {
                    0 => MobjFlags::SHADOW,
                    class => {
                        MobjFlags::from_bits_retain((class as i32) << MobjFlags::TRANSLATION_SHIFT)
                    }
                };
            }
        }
        if g % 5 == 0 {
            lines.push(format!(
                "fx{detail} {g:05} {:016x}",
                fnv_bytes(&state.i_video.i_video_buffer)
            ));
        }
    });
    lines.push(format!("fx{detail} end {message}"));
    Some(lines)
}

/// Hash of the Doom II cast-call sequence (monster, animation state, attack
/// phase and timing after every tic). No demo reaches the cast, so it is
/// driven directly.
fn cast_trace() -> Option<String> {
    let state = start(&[])?;
    // What start_cast sets up, minus the Doom II music that doom1.wad lacks.
    let first = state.f_finale.castorder[0].kind;
    let see = StateId(state.info.mobjinfo[first as usize].seestate as u32);
    state.f_finale.castnum = 0;
    state.f_finale.caststate = Some(see);
    state.f_finale.casttics = state.info.state_mut(see).tics;
    state.f_finale.castdeath = false;
    state.f_finale.castframes = 0;
    state.f_finale.castonmelee = 0;
    state.f_finale.castattacking = false;
    let mut hash = FNV_OFFSET;
    for _ in 0..6000 {
        cast_ticker(state);
        let f = &state.f_finale;
        hash = fnv(
            hash,
            [
                f.castnum,
                f.casttics,
                f.caststate.map_or(-1, |s| s.0 as i32),
                i32::from(f.castdeath),
                f.castframes,
                f.castonmelee,
                i32::from(f.castattacking),
            ]
            .map(|v| v as u32),
        );
    }
    Some(format!("{hash:016x} castnum={}", state.f_finale.castnum))
}

/// Uses the first two-sided line of E1M1 with every line special in turn, from the player and
/// from a monster, on both sides, and hashes each outcome and the resulting
/// world. Covers every arm of `use_special_line`.
fn use_special_line_trace() -> Option<String> {
    let state = start_e1m1()?;
    let (save_path, _) = save_slot(state, 0);
    let line = (0..state.p_setup.numlines)
        .map(|i| LineId(i as u32))
        .find(|&l| state.p_setup.line(l).backsector.is_some())?;
    // A tag carried by only a few sectors, as real maps use, so that one
    // activation does not affect every untagged sector.
    let tag = (0..state.p_setup.numsectors)
        .map(|i| state.p_setup.sector_mut(SectorId(i as u32)).tag)
        .find(|&t| t != 0)?;
    let mut hash = FNV_OFFSET;
    for special in 0..=145i16 {
        for side in 0..=1 {
            for player_uses in [true, false] {
                // Every case starts from the same saved world.
                g_load_game(&mut state.g_game, &save_path);
                do_load_game(state);
                let actor = if player_uses {
                    state.g_game.players[0].mo.unwrap()
                } else {
                    mobj_thinker_ids(&state.p_mobj, &state.p_tick)
                        .into_iter()
                        .find(|&id| {
                            let m = state.p_mobj.mo(id);
                            m.player.is_none() && m.flags.contains(MobjFlags::COUNTKILL)
                        })?
                };
                state.p_setup.line_mut(line).special = special;
                state.p_setup.line_mut(line).tag = tag;
                let used = use_special_line(state, actor, line, side);
                // Let any mover that was started run for a few tics, and
                // note what the use did to the line itself (a switch flips
                // its textures and clears once-only specials).
                for _ in 0..8 {
                    crate::p_tick::run_thinkers(state);
                }
                let sidenum = state.p_setup.line(line).sidenum[0] as usize;
                let side_textures = {
                    let s = &state.p_setup.sides[sidenum];
                    [s.toptexture, s.midtexture, s.bottomtexture].map(|t| t as i32)
                };
                let line_special = state.p_setup.line(line).special;
                let world = world_summary(state);
                hash = fnv(
                    hash,
                    [
                        u32::from(used),
                        i32::from(line_special) as u32,
                        side_textures[0] as u32,
                        side_textures[1] as u32,
                        side_textures[2] as u32,
                        fnv_bytes(world.as_bytes()) as u32,
                    ],
                );
                state.g_game.gameaction = crate::d_event::GameAction::Nothing;
            }
        }
    }
    Some(format!("{hash:016x}"))
}

fn actual_output() -> Option<String> {
    let mut out = String::new();
    for demo in ["demo1", "demo2", "demo3"] {
        writeln!(out, "sim {demo} {}", demo_summary(demo)?).unwrap();
    }
    writeln!(out, "cast {}", cast_trace()?).unwrap();
    writeln!(out, "usespecial {}", use_special_line_trace()?).unwrap();
    let state = start_e1m1()?;
    let (_, save) = save_slot(state, 0);
    writeln!(
        out,
        "save e1m1@350 len={} hash={:016x}",
        save.len(),
        fnv_bytes(&save)
    )
    .unwrap();
    for line in ui_frame_lines()? {
        writeln!(out, "{line}").unwrap();
    }
    for detail in [0, 1] {
        for line in effect_frame_lines(detail)? {
            writeln!(out, "{line}").unwrap();
        }
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

/// Plays 350 tics of E1M1 and returns the engine, ready to save.
fn start_e1m1() -> Option<&'static mut GameState> {
    let state = start(&["-warp", "1", "1", "-skill", "3"])?;
    while state.d_loop.gametic < 350 {
        doomgeneric_tick(state);
    }
    Some(state)
}

/// Writes save slot `slot` directly, not through the deferred `sendsave`
/// request (which would trigger a second save on the next tick), and returns
/// the file's path and contents.
fn save_slot(state: &mut GameState, slot: i32) -> (String, Vec<u8>) {
    g_save_game(&mut state.g_game, slot, "roundtrip");
    state.g_game.sendsave = false;
    do_save_game(state);
    let path = save_game_file(&state.d_main, slot);
    let bytes =
        read_file(&mut *state.fs, &path).unwrap_or_else(|| panic!("no save file at {path}"));
    (path, bytes)
}

/// Saving and loading restores the exact world, and saving again writes the
/// same bytes: nothing process-specific (such as addresses) is in the file.
#[test]
fn save_game_round_trips() {
    let Some(state) = start_e1m1() else {
        std::eprintln!("skipping: no IWAD (set DOOM_IWAD or put doom1.wad in ~/Downloads)");
        return;
    };
    let before = world_summary(state);
    let (path, first) = save_slot(state, 0);

    // Let the world move on, then restore it from the file.
    for _ in 0..40 {
        doomgeneric_tick(state);
    }
    assert_ne!(
        before,
        world_summary(state),
        "the world should have changed"
    );
    g_load_game(&mut state.g_game, &path);
    do_load_game(state);
    assert_eq!(before, world_summary(state));

    let (_, second) = save_slot(state, 1);
    assert!(first == second, "re-saving a loaded game changed the file");
}

/// Two independent engines saving at the same point write identical files.
#[test]
fn save_files_are_reproducible() {
    let (Some(a), Some(b)) = (start_e1m1(), start_e1m1()) else {
        std::eprintln!("skipping: no IWAD (set DOOM_IWAD or put doom1.wad in ~/Downloads)");
        return;
    };
    assert!(save_slot(a, 0).1 == save_slot(b, 0).1);
}

/// The frame a platform gets through `draw_indexed_frame` is the same image
/// as the one the engine scales and converts for `draw_frame`, on every frame
/// of a run that covers menus, the automap, intermissions, finales and wipes.
#[test]
fn indexed_frames_match_scaled_frames() {
    let run = |indexed: bool| -> Option<Vec<u64>> {
        let hashes = Rc::new(RefCell::new(Vec::new()));
        let platform = HashingPlatform {
            inner: NullPlatform::default(),
            indexed,
            hashes: Rc::clone(&hashes),
        };
        let state = start_with(Box::new(platform), &["-timedemo", "demo3"])?;
        let message = run_until_exit(state, scripted_input);
        assert!(message.starts_with("timed "), "unexpected exit: {message}");
        Some(hashes.take())
    };
    let (Some(scaled), Some(indexed)) = (run(false), run(true)) else {
        std::eprintln!("skipping: no IWAD (set DOOM_IWAD or put doom1.wad in ~/Downloads)");
        return;
    };
    assert!(scaled.len() > 1000, "only {} frames", scaled.len());
    assert_eq!(scaled.len(), indexed.len());
    for (n, (a, b)) in scaled.iter().zip(&indexed).enumerate() {
        assert_eq!(a, b, "frame {n} differs");
    }
}

/// A [`NullPlatform`] with a sound card: 11025 Hz, and every tick call wants a
/// 35th of a second of audio, which it records.
struct AudioPlatform {
    inner: NullPlatform,
    opened: Rc<RefCell<bool>>,
    samples: Rc<RefCell<Vec<i16>>>,
    /// `Some`: this platform plays the music itself and logs what the engine asks of it.
    music: Option<Rc<RefCell<Vec<std::string::String>>>>,
}

impl DoomPlatform for AudioPlatform {
    fn init(&mut self, resx: i32, resy: i32) {
        self.inner.init(resx, resy);
    }
    fn draw_frame(&mut self, frame: &[Pixel]) {
        self.inner.draw_frame(frame);
    }
    fn audio_open(&mut self, preferred_rate: u32) -> Option<u32> {
        assert_eq!(preferred_rate, 44100, "snd_samplerate default");
        *self.opened.borrow_mut() = true;
        Some(11025)
    }
    fn audio_frames_wanted(&mut self) -> usize {
        11025 / 35
    }
    fn audio_write(&mut self, samples: &[i16]) {
        self.samples.borrow_mut().extend_from_slice(samples);
    }
    fn music_open(&mut self, genmidi: &[u8]) -> bool {
        let Some(log) = &self.music else { return false };
        log.borrow_mut()
            .push(std::format!("open {}", genmidi.starts_with(b"#OPL_II#")));
        true
    }
    fn music_command(&mut self, command: crate::MusicCommand<'_>) {
        let log = self.music.as_ref().expect("music_open returned false");
        let text = match command {
            crate::MusicCommand::Register(data) => {
                std::format!("register mus={}", data.starts_with(b"MUS\x1a"))
            }
            other => std::format!("{other:?}"),
        };
        log.borrow_mut().push(text);
    }
    fn sleep_ms(&mut self, ms: u32) {
        self.inner.sleep_ms(ms);
    }
    fn get_ticks_ms(&mut self) -> u32 {
        self.inner.get_ticks_ms()
    }
    fn get_key(&mut self) -> Option<(bool, u8)> {
        self.inner.get_key()
    }
    fn set_window_title(&mut self, title: &str) {
        self.inner.set_window_title(title);
    }
    fn print(&mut self, message: &str) {
        self.inner.print(message);
    }
    fn eprint(&mut self, message: &str) {
        self.inner.eprint(message);
    }
    fn quit(&mut self) -> ! {
        self.inner.quit()
    }
}

/// Runs the first `ticks` tick calls of demo1 on an [`AudioPlatform`], with the
/// extra command line arguments, and returns whether the audio device was
/// opened and everything the engine wrote to it.
fn demo_audio(args: &[&str], ticks: u32) -> Option<(bool, Vec<i16>)> {
    demo_audio_music(args, ticks, None)
}

/// Like [`demo_audio`], for a platform that plays the music itself if `music`
/// is given (it collects the requests).
fn demo_audio_music(
    args: &[&str],
    ticks: u32,
    music: Option<Rc<RefCell<Vec<std::string::String>>>>,
) -> Option<(bool, Vec<i16>)> {
    let opened = Rc::new(RefCell::new(false));
    let samples = Rc::new(RefCell::new(Vec::new()));
    let platform = AudioPlatform {
        inner: NullPlatform::default(),
        opened: Rc::clone(&opened),
        samples: Rc::clone(&samples),
        music,
    };
    let mut argv = alloc::vec!["-timedemo", "demo1"];
    argv.extend_from_slice(args);
    let state = start_with(Box::new(platform), &argv)?;
    for _ in 0..ticks {
        doomgeneric_tick(state);
    }
    let opened = *opened.borrow();
    Some((opened, samples.take()))
}

/// The sound effects a demo triggers come out of the audio device: audible,
/// stereo, and in whole batches of what the platform asked for.
#[test]
fn sound_effects_reach_the_audio_output() {
    let Some((opened, samples)) = demo_audio(&[], 1500) else {
        std::eprintln!("skipping: no IWAD (set DOOM_IWAD or put doom1.wad in ~/Downloads)");
        return;
    };
    assert!(opened);
    // A tick call feeds the device once, or a few more times while the engine
    // waits for a screen wipe.
    let per_call = 2 * (11025 / 35);
    assert_eq!(samples.len() % per_call, 0);
    assert!(samples.len() >= 1500 * per_call);
    let peak = samples.iter().map(|s| i32::from(s.unsigned_abs())).max();
    assert!(peak > Some(1000), "demo1 is silent: peak {peak:?}");
    let panned = samples.chunks_exact(2).any(|f| f[0] != f[1]);
    assert!(panned, "every sound came out dead centre");
    let sounding = samples.chunks_exact(2).filter(|f| f[0] != 0 || f[1] != 0);
    assert!(sounding.count() > 11025, "less than a second of sound");
}

/// `-nosound` never opens the audio device.
#[test]
fn nosound_leaves_the_audio_device_closed() {
    let Some((opened, samples)) = demo_audio(&["-nosound"], 200) else {
        std::eprintln!("skipping: no IWAD (set DOOM_IWAD or put doom1.wad in ~/Downloads)");
        return;
    };
    assert!(!opened);
    assert!(samples.is_empty());
}

/// A platform that takes the music over gets the GENMIDI bank once, then every song request in
/// order, and the engine mixes no music of its own into the stream.
#[test]
fn a_platform_can_take_the_music_over() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let Some((_, samples)) = demo_audio_music(&[], 400, Some(Rc::clone(&log))) else {
        std::eprintln!("skipping: no IWAD (set DOOM_IWAD or put doom1.wad in ~/Downloads)");
        return;
    };
    let log = log.take();
    assert_eq!(
        log.first().map(String::as_str),
        Some("open true"),
        "{log:?}"
    );
    assert_eq!(
        log.iter().filter(|line| line.starts_with("open")).count(),
        1
    );
    let register = log.iter().position(|line| line == "register mus=true");
    let play = log.iter().position(|line| line.starts_with("Play"));
    assert!(
        register.is_some() && play > register,
        "song not registered then played: {log:?}"
    );
    assert!(log.iter().any(|line| line.starts_with("Volume")), "{log:?}");

    // The same run with the engine's own player mixes music into the stream, so it is louder
    // and different from the one without.
    let Some((_, with_engine_music)) = demo_audio_music(&[], 400, None) else {
        return;
    };
    assert_ne!(
        samples, with_engine_music,
        "the engine still mixed music of its own"
    );
}
