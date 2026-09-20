//! `DoomPlatform` for the CoreS3: LCD output, a millisecond clock and the serial console.
//! Input arrives over Wi-Fi (see `net`); the LCD is driven from core 0 (see `lcd`).

use core::sync::atomic::{AtomicU32, Ordering};

use core_s3_protocol::{Command, KeyEvent, PollGate};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use esp_hal::{delay::Delay, time::Instant};
use esp_println::print;
use rust_doomgeneric::{DoomPlatform, MusicCommand};

use crate::{lcd, music, net, sound};

/// Frame timing, printed over serial every couple of seconds.
#[derive(Default)]
struct FrameStats {
    window_start_us: u64,
    frames: u32,
    /// Time spent in `draw_indexed_frame`: waiting for the LCD, then converting the frame.
    present_us: u64,
    /// The part of that spent waiting for the previous frame to finish going out.
    waiting_us: u64,
    /// Time the engine spent mixing sound and handing it over (`audio_frames_wanted` to the end
    /// of each `audio_write`); part of "everything else".
    audio_us: u64,
}

const STATS_WINDOW_US: u64 = 2_000_000;
/// How often a new frame rate is published for the LCD bar and the web page (shorter than the
/// `[perf]` window, which stays at 2 s so the serial log and its benchmark recipe do not change).
const FPS_WINDOW_US: u64 = 1_000_000;

/// The latest frame rate, for the LCD bar and the web controller (both on core 0): the number of
/// tenths of a frame per second in the low 16 bits and, in the high 16, a count that goes up with
/// every sample (so a repeat of the same rate still counts as new, and a stalled game shows as no
/// change). 0 = nothing measured yet. One relaxed store per second; the game does no
/// formatting or allocation for it.
static FPS_SAMPLE: AtomicU32 = AtomicU32::new(0);

/// The latest sample: `(counter, fps in tenths)`, or `None` before the first one. A different
/// counter than last time means a new sample.
pub fn fps_sample() -> Option<(u16, u16)> {
    match FPS_SAMPLE.load(Ordering::Relaxed) {
        0 => None,
        sample => Some(((sample >> 16) as u16, sample as u16)),
    }
}

fn publish_fps(tenths: u64) {
    let counter = (FPS_SAMPLE.load(Ordering::Relaxed) >> 16).wrapping_add(1) & 0xffff;
    // Counter 0 is skipped on wrap-around so a sample is never mistaken for "nothing yet".
    let counter = if counter == 0 { 1 } else { counter };
    FPS_SAMPLE.store(
        counter << 16 | tenths.min(u64::from(u16::MAX)) as u32,
        Ordering::Relaxed,
    );
}

fn now_us() -> u64 {
    Instant::now().duration_since_epoch().as_micros() as u64
}

/// Frames counted towards the next published frame rate.
#[derive(Default)]
struct FpsWindow {
    start_us: u64,
    frames: u32,
}

/// Tenths of a frame per second for `frames` in `elapsed_us`.
fn fps_tenths(frames: u32, elapsed_us: u64) -> u64 {
    u64::from(frames) * 10_000_000 / elapsed_us
}

#[derive(Default)]
pub struct CoreS3Platform {
    stats: FrameStats,
    fps_window: FpsWindow,
    /// When the engine last started on a chunk of sound (see `FrameStats::audio_us`).
    audio_mark_us: u64,
    /// Keeps a tap's release from reaching the game in the same poll as its press.
    input_gate: PollGate,
}

impl CoreS3Platform {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one frame to the timing and prints it once per window. `start` is when the frame came
    /// in, `acquired` when the LCD buffer became free, `end` when the frame was handed over.
    fn record(&mut self, start: u64, acquired: u64, end: u64) {
        let window = &mut self.fps_window;
        if window.start_us == 0 {
            window.start_us = start;
        }
        window.frames += 1;
        let elapsed = end - window.start_us;
        if elapsed >= FPS_WINDOW_US {
            publish_fps(fps_tenths(window.frames, elapsed));
            *window = FpsWindow {
                start_us: end,
                frames: 0,
            };
        }

        let stats = &mut self.stats;
        if stats.window_start_us == 0 {
            stats.window_start_us = start;
        }
        stats.frames += 1;
        stats.present_us += end - start;
        stats.waiting_us += acquired - start;
        let elapsed = end - stats.window_start_us;
        if elapsed >= STATS_WINDOW_US {
            let frames = u64::from(stats.frames);
            let tenths = fps_tenths(stats.frames, elapsed);
            print!(
                "[perf] {}.{} fps, present {} us/frame (waiting for the LCD {}), everything else {} us/frame (of it sound {})\n",
                tenths / 10,
                tenths % 10,
                stats.present_us / frames,
                stats.waiting_us / frames,
                (elapsed - stats.present_us) / frames,
                stats.audio_us / frames,
            );
            *stats = FrameStats {
                window_start_us: end,
                ..FrameStats::default()
            };
        }
    }
}

/// Engine pixels are `0x00RRGGBB`.
fn to_rgb565(pixel: u32) -> Rgb565 {
    let [_, r, g, b] = pixel.to_be_bytes();
    Rgb565::new(r >> 3, g >> 2, b >> 3)
}

/// The engine key code for a command: what the engine's default bindings (m_controls.rs) expect.
/// `None` for a command that is not a game key.
fn doom_key(command: Command) -> Option<u8> {
    Some(match command {
        Command::Forward => 0xad,
        Command::Backward => 0xaf,
        Command::TurnLeft => 0xac,
        Command::TurnRight => 0xae,
        Command::StrafeLeft => 0xa0,
        Command::StrafeRight => 0xa1,
        Command::Fire => 0xa3,
        Command::Use => 0xa2,
        Command::Run => 0x80 + 0x36, // KEY_RSHIFT
        Command::Enter => 13,
        Command::Escape => 27,
        Command::Map => 9, // Tab
        Command::Yes => b'y',
        Command::No => b'n',
        Command::Weapon1 => b'1',
        Command::Weapon2 => b'2',
        Command::Weapon3 => b'3',
        Command::Weapon4 => b'4',
        Command::Weapon5 => b'5',
        Command::Weapon6 => b'6',
        Command::Weapon7 => b'7',
        Command::Backspace => 0x7f, // KEY_BACKSPACE
        // A typed character is the key itself; the engine sees letters in lower case.
        Command::Char(character) => character.to_ascii_lowercase(),
        // The firmware's own (the speaker), see `get_key`.
        Command::ToggleSound => return None,
    })
}

impl DoomPlatform for CoreS3Platform {
    fn init(&mut self, _resx: i32, _resy: i32) {}

    fn draw_frame(&mut self, _frame: &[u32]) {
        // Only reached if `draw_indexed_frame` declines, which it never does.
    }

    fn draw_indexed_frame(&mut self, indices: &[u8], palette: &[u32; 256]) -> bool {
        // The engine's 320x200 screen goes through a colour table into a buffer that core 0 then
        // streams to the LCD by DMA, while the engine moves on to the next frame.
        let start = now_us();
        let colors = palette.map(|pixel| to_rgb565(pixel).into_storage().to_be_bytes());
        let mut frame = lcd::acquire_frame();
        let acquired = now_us();
        frame.fill(indices, &colors);
        lcd::submit_frame(frame);
        self.record(start, acquired, now_us());
        true
    }

    fn audio_open(&mut self, _preferred_rate: u32) -> Option<u32> {
        // The engine mixes the effects at their own rate; `sound` brings them to the speaker's.
        // `None` if the speaker did not come up (see `main`).
        sound::ready().then_some(sound::SFX_RATE)
    }

    fn audio_frames_wanted(&mut self) -> usize {
        self.audio_mark_us = now_us();
        sound::frames_wanted()
    }

    fn audio_write(&mut self, samples: &[i16]) {
        sound::write(samples);
        let now = now_us();
        self.stats.audio_us += now - self.audio_mark_us;
        self.audio_mark_us = now;
    }

    fn music_open(&mut self, genmidi: &[u8]) -> bool {
        // The synthesizer runs on core 0 (see `music`).
        music::open(genmidi)
    }

    fn music_command(&mut self, command: MusicCommand<'_>) {
        music::command(command);
    }

    fn sleep_ms(&mut self, ms: u32) {
        Delay::new().delay_millis(ms);
    }

    fn get_ticks_ms(&mut self) -> u32 {
        // Wraps after ~49 days, like the engine's own 32-bit tick counters expect.
        Instant::now().duration_since_epoch().as_millis() as u32
    }

    fn get_key(&mut self) -> Option<(bool, u8)> {
        // The firmware's own keys (the sound toggle) are dealt with here; the game never sees them.
        let mut source = || -> Option<KeyEvent> {
            loop {
                let event = net::next_key_event()?;
                if doom_key(event.command).is_some() {
                    return Some(event);
                }
                if event.pressed {
                    sound::toggle_mute();
                }
            }
        };
        let event = self.input_gate.next(&mut source)?;
        doom_key(event.command).map(|key| (event.pressed, key))
    }

    fn set_window_title(&mut self, _title: &str) {}

    fn print(&mut self, message: &str) {
        print!("{message}");
    }

    fn eprint(&mut self, message: &str) {
        print!("{message}");
    }

    fn quit(&mut self) -> ! {
        // The engine has finished (Quit Game in its menu): switch the board off.
        crate::power::power_off();
        // Still here: the power chip left us powered, so all that is left is to wait for a reset.
        loop {
            core::hint::spin_loop();
        }
    }
}
