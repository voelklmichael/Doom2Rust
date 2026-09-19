//! `DoomPlatform` for the CoreS3: LCD output, a millisecond clock and the serial console.
//! Input arrives over Wi-Fi (see `net`); the LCD is driven from core 0 (see `lcd`).

use core_s3_protocol::Command;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use esp_hal::{delay::Delay, time::Instant};
use esp_println::print;
use rust_doomgeneric::DoomPlatform;

use crate::{audio, lcd, net, sound};

/// Frame timing, printed over serial every couple of seconds.
#[derive(Default)]
struct FrameStats {
    window_start_us: u64,
    frames: u32,
    /// Time spent in `draw_indexed_frame`: waiting for the LCD, then converting the frame.
    present_us: u64,
    /// The part of that spent waiting for the previous frame to finish going out.
    waiting_us: u64,
}

const STATS_WINDOW_US: u64 = 2_000_000;

fn now_us() -> u64 {
    Instant::now().duration_since_epoch().as_micros() as u64
}

#[derive(Default)]
pub struct CoreS3Platform {
    stats: FrameStats,
}

impl CoreS3Platform {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one frame to the timing and prints it once per window. `start` is when the frame came
    /// in, `acquired` when the LCD buffer became free, `end` when the frame was handed over.
    fn record(&mut self, start: u64, acquired: u64, end: u64) {
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
            print!(
                "[perf] {}.{} fps, present {} us/frame (waiting for the LCD {}), everything else {} us/frame\n",
                frames * 1_000_000 / elapsed,
                frames * 10_000_000 / elapsed % 10,
                stats.present_us / frames,
                stats.waiting_us / frames,
                (elapsed - stats.present_us) / frames,
            );
            *stats = FrameStats { window_start_us: end, ..FrameStats::default() };
        }
    }
}

/// Engine pixels are `0x00RRGGBB`.
fn to_rgb565(pixel: u32) -> Rgb565 {
    let [_, r, g, b] = pixel.to_be_bytes();
    Rgb565::new(r >> 3, g >> 2, b >> 3)
}

/// The engine key code for a command: what the engine's default bindings (m_controls.rs) expect.
fn doom_key(command: Command) -> u8 {
    match command {
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
    }
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
        // The speaker runs at a fixed rate; `None` if it did not come up (see `main`).
        sound::ready().then_some(audio::SAMPLE_RATE)
    }

    fn audio_frames_wanted(&mut self) -> usize {
        sound::frames_wanted()
    }

    fn audio_write(&mut self, samples: &[i16]) {
        sound::write(samples);
    }

    fn sleep_ms(&mut self, ms: u32) {
        Delay::new().delay_millis(ms);
    }

    fn get_ticks_ms(&mut self) -> u32 {
        // Wraps after ~49 days, like the engine's own 32-bit tick counters expect.
        Instant::now().duration_since_epoch().as_millis() as u32
    }

    fn get_key(&mut self) -> Option<(bool, u8)> {
        let event = net::next_key_event()?;
        Some((event.pressed, doom_key(event.command)))
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
