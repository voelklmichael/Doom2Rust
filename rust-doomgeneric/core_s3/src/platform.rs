//! `DoomPlatform` for the CoreS3: LCD output, a millisecond clock and the serial console.
//! There is no input yet, so the engine just plays its attract-mode demos.

use core_s3::bsp::CoreS3Display;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Rectangle,
};
use esp_hal::{delay::Delay, time::Instant};
use esp_println::print;
use rust_doomgeneric::DoomPlatform;

/// DOOM's native resolution. The engine is run with `-scaling 1`, which draws the 320x200
/// image centred in each (wider) row of its frame buffer.
const DOOM_WIDTH: usize = 320;
const DOOM_HEIGHT: usize = 200;
/// The LCD is 320x240, so the picture is centred vertically with black bars above and below.
const LCD_TOP: i32 = 20;

pub struct CoreS3Platform {
    display: CoreS3Display,
    /// Pixels per row of the engine's frame buffer, set in `init`.
    stride: usize,
}

impl CoreS3Platform {
    pub fn new(display: CoreS3Display) -> Self {
        Self { display, stride: DOOM_WIDTH }
    }
}

/// Engine pixels are `0x00RRGGBB`.
fn to_rgb565(pixel: u32) -> Rgb565 {
    let [_, r, g, b] = pixel.to_be_bytes();
    Rgb565::new(r >> 3, g >> 2, b >> 3)
}

impl DoomPlatform for CoreS3Platform {
    fn init(&mut self, resx: i32, _resy: i32) {
        self.stride = resx as usize;
        self.display.clear(Rgb565::BLACK).expect("clear LCD");
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
        self.display.blit_pixels(&area, pixels).expect("blit frame");
    }

    fn sleep_ms(&mut self, ms: u32) {
        Delay::new().delay_millis(ms);
    }

    fn get_ticks_ms(&mut self) -> u32 {
        // Wraps after ~49 days, like the engine's own 32-bit tick counters expect.
        Instant::now().duration_since_epoch().as_millis() as u32
    }

    fn get_key(&mut self) -> Option<(bool, u8)> {
        None
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
