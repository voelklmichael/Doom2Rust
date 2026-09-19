//! `DoomPlatform` for the CoreS3: LCD output, a millisecond clock and the serial console.
//! Input arrives over Wi-Fi (see `net`).

use core_s3::{bsp::CoreS3Display, ui::Label};
use core_s3_protocol::Command;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Rectangle,
};
use esp_hal::{delay::Delay, time::Instant};
use esp_println::print;
use heapless::String;
use rust_doomgeneric::DoomPlatform;

use crate::net;

/// DOOM's native resolution. The engine is run with `-scaling 1`, which draws the 320x200
/// image centred in each (wider) row of its frame buffer.
const DOOM_WIDTH: usize = 320;
const DOOM_HEIGHT: usize = 200;
/// The LCD is 320x240, so the picture is centred vertically with black bars above and below.
const LCD_TOP: i32 = 20;

/// Frame timing, printed over serial every couple of seconds.
#[derive(Default)]
struct FrameStats {
    window_start_us: u64,
    frames: u32,
    blit_us: u64,
}

const STATS_WINDOW_US: u64 = 2_000_000;

fn now_us() -> u64 {
    Instant::now().duration_since_epoch().as_micros() as u64
}

pub struct CoreS3Platform {
    display: CoreS3Display,
    /// Pixels per row of the engine's frame buffer, set in `init`.
    stride: usize,
    /// Shown in the top black bar, which the game never draws over (e.g. the controller address).
    status: String<40>,
    stats: FrameStats,
}

impl CoreS3Platform {
    pub fn new(display: CoreS3Display, status: String<40>) -> Self {
        Self { display, stride: DOOM_WIDTH, status, stats: FrameStats::default() }
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
    fn init(&mut self, resx: i32, _resy: i32) {
        self.stride = resx as usize;
        self.display.clear(Rgb565::BLACK).expect("clear LCD");
        Label { text: &self.status, top_left: Point::new(4, 5), color: Rgb565::CYAN }
            .draw(&mut self.display)
            .expect("draw status");
    }

    fn draw_frame(&mut self, frame: &[u32]) {
        let x0 = (self.stride - DOOM_WIDTH) / 2;
        let pixels = frame
            .chunks_exact(self.stride)
            .take(DOOM_HEIGHT)
            .flat_map(|row| row[x0..x0 + DOOM_WIDTH].iter().map(|&p| to_rgb565(p)));
        let area = Rectangle::new(
            Point::new(0, LCD_TOP),
            Size::new(DOOM_WIDTH as u32, DOOM_HEIGHT as u32),
        );
        let blit_start = now_us();
        self.display.blit_pixels(&area, pixels).expect("blit frame");
        let now = now_us();

        let stats = &mut self.stats;
        if stats.window_start_us == 0 {
            stats.window_start_us = blit_start;
        }
        stats.frames += 1;
        stats.blit_us += now - blit_start;
        let elapsed = now - stats.window_start_us;
        if elapsed >= STATS_WINDOW_US {
            let frames = u64::from(stats.frames);
            print!(
                "[perf] {}.{} fps, blit {} us/frame, everything else {} us/frame\n",
                frames * 1_000_000 / elapsed,
                frames * 10_000_000 / elapsed % 10,
                stats.blit_us / frames,
                (elapsed - stats.blit_us) / frames,
            );
            *stats = FrameStats { window_start_us: now, ..FrameStats::default() };
        }
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
        loop {
            core::hint::spin_loop();
        }
    }
}
