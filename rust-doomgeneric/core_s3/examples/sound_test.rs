//! Speaker test for the CoreS3: plays a rising 8-note scale, then a left-only and a right-only
//! beep, forever, through the same `audio` module the game uses. Run it with
//!
//! ```text
//! source ~/export-esp.sh; cd core_s3; cargo run --release --example sound_test
//! ```
//!
//! The LCD shows the setup result and the current step; the serial log has the amp registers and,
//! every 2 s, how many frames went out. The volume is moderate on purpose (about 15 % of full
//! scale, see `AMPLITUDE`).
#![no_std]
#![no_main]

// The game's own speaker code, so what is tested here is what the game uses.
#[path = "../src/audio.rs"]
mod audio;

use core::fmt::Write as _;

use core_s3::{
    audio::{AudioSource, Tone},
    bsp::CoreS3DisplayResources,
    ui::Label,
    CoreS3,
};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, delay::Delay, time::Instant};
use esp_println::println;
use heapless::String;

esp_bootloader_esp_idf::esp_app_desc!();

/// Peak sample value of the test tones (full scale is 32767).
const AMPLITUDE: i16 = 5000;
/// Tones fade in and out over this many samples (10 ms) so they do not click.
const FADE: u32 = 220;

/// One step of the test sequence. `hz == 0` is silence.
struct Step {
    label: &'static str,
    hz: u16,
    ms: u16,
    left: bool,
    right: bool,
}

const fn note(label: &'static str, hz: u16) -> Step {
    Step { label, hz, ms: 200, left: true, right: true }
}

const fn pause(ms: u16) -> Step {
    Step { label: "", hz: 0, ms, left: false, right: false }
}

/// C major, C4 to C5, then the stereo check.
const STEPS: [Step; 13] = [
    note("C4  262 Hz", 262),
    note("D4  294 Hz", 294),
    note("E4  330 Hz", 330),
    note("F4  349 Hz", 349),
    note("G4  392 Hz", 392),
    note("A4  440 Hz", 440),
    note("B4  494 Hz", 494),
    note("C5  523 Hz", 523),
    pause(500),
    Step { label: "LEFT only 440 Hz", hz: 440, ms: 400, left: true, right: false },
    pause(200),
    Step { label: "RIGHT only 660 Hz", hz: 660, ms: 400, left: false, right: true },
    pause(1500),
];

/// Generates the sequence one frame at a time.
struct Player {
    step: usize,
    tone: Tone,
    index: u32,
    length: u32,
}

impl Player {
    fn new() -> Self {
        let mut player = Self { step: STEPS.len() - 1, tone: Tone::new(0, 0, 1, 0), index: 0, length: 0 };
        player.next_step();
        player
    }

    fn next_step(&mut self) {
        self.step = (self.step + 1) % STEPS.len();
        let step = &STEPS[self.step];
        self.tone = Tone::new(step.hz, step.ms, audio::SAMPLE_RATE, AMPLITUDE);
        self.index = 0;
        self.length = audio::SAMPLE_RATE / 1000 * u32::from(step.ms);
    }

    /// The next `(left, right)` frame, and whether it is the first of a new step.
    fn frame(&mut self) -> ((i16, i16), bool) {
        let mut started = false;
        if self.index >= self.length {
            self.next_step();
            started = true;
        }
        let step = &STEPS[self.step];
        let sample = i32::from(self.tone.next_sample().unwrap_or(0));
        let fade = self.index.min(self.length - self.index).min(FADE) as i32;
        let sample = (sample * fade / FADE as i32) as i16;
        self.index += 1;
        ((if step.left { sample } else { 0 }, if step.right { sample } else { 0 }), started)
    }
}

fn line<D: DrawTarget<Color = Rgb565>>(display: &mut D, y: i32, color: Rgb565, text: &str) {
    // Wipe the row first: text is drawn by its baseline, 10 rows tall.
    let _ = Rectangle::new(Point::new(0, y - 12), Size::new(320, 16))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(display);
    let _ = Label { text, top_left: Point::new(10, y), color }.draw(display);
}

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    // The USB serial port only shows what is printed after a monitor attached (and `cargo run`
    // needs a moment to attach), so wait a little before the interesting logs.
    Delay::new().delay_millis(3000);
    println!("[sound_test] start");

    // Panel and rail power come from the BSP; that also releases the amp's reset line.
    let mut parts = CoreS3::init_display(CoreS3DisplayResources {
        i2c0: peripherals.I2C0,
        i2c_sda: peripherals.GPIO12,
        i2c_scl: peripherals.GPIO11,
        spi2: peripherals.SPI2,
        lcd_sclk: peripherals.GPIO36,
        lcd_mosi: peripherals.GPIO37,
        lcd_dc: peripherals.GPIO35,
        lcd_cs: peripherals.GPIO3,
        tf_card_cs: peripherals.GPIO4,
    })
    .expect("display");
    let display = &mut parts.display;
    let mut i2c = parts.internal_i2c;
    let _ = display.clear(Rgb565::BLACK);
    line(display, 25, Rgb565::CYAN, "CoreS3 sound test");

    // I2S first (the amp wants a clock), then the amp.
    let opened = audio::Speaker::open(
        peripherals.I2S1,
        peripherals.DMA_CH1,
        peripherals.GPIO34,
        peripherals.GPIO33,
        peripherals.GPIO13,
    );
    let mut speaker = match opened {
        Ok(speaker) => {
            println!("[sound_test] I2S1 + DMA_CH1 running at {} Hz, ring {} frames", audio::SAMPLE_RATE, audio::RING_FRAMES);
            line(display, 50, Rgb565::GREEN, "I2S: running, 22050 Hz");
            speaker
        }
        Err(error) => {
            println!("[sound_test] I2S FAILED: {error}");
            line(display, 50, Rgb565::RED, "I2S: FAILED");
            loop {
                core::hint::spin_loop();
            }
        }
    };
    let delay = Delay::new();
    delay.delay_millis(20); // let the amp see a few clock edges
    match audio::init_amp(&mut i2c) {
        Ok(()) => {
            println!("[sound_test] amp init OK");
            line(display, 75, Rgb565::GREEN, "amp: init OK");
        }
        Err(error) => {
            println!("[sound_test] amp init FAILED: {error:?}");
            line(display, 75, Rgb565::RED, "amp: init FAILED (I2C)");
        }
    }
    line(display, 100, Rgb565::WHITE, "you should hear: 8 rising notes,");
    line(display, 115, Rgb565::WHITE, "then LEFT beep, then RIGHT beep");

    let mut player = Player::new();
    let mut frames_sent: u32 = 0;
    let mut next_report = Instant::now().duration_since_epoch().as_millis() + 2000;
    let mut chunk = [0i16; 2 * audio::CHUNK_FRAMES];
    line(display, 150, Rgb565::YELLOW, STEPS[player.step].label);
    loop {
        // Refill the ring one chunk at a time whenever the DMA has freed one.
        if speaker.free_frames() >= audio::CHUNK_FRAMES {
            let mut new_step = None;
            for frame in chunk.chunks_exact_mut(2) {
                let ((left, right), started) = player.frame();
                frame.copy_from_slice(&[left, right]);
                if started {
                    new_step = Some(player.step);
                }
            }
            if speaker.push_chunk(&chunk) {
                frames_sent = frames_sent.wrapping_add(audio::CHUNK_FRAMES as u32);
            }
            if let Some(step) = new_step {
                line(display, 150, Rgb565::YELLOW, STEPS[step].label);
            }
        } else {
            delay.delay_millis(1);
        }

        let now = Instant::now().duration_since_epoch().as_millis();
        if now >= next_report {
            next_report = now + 2000;
            let mut status = String::<48>::new();
            let _ = write!(status, "frames sent {frames_sent}, DMA restarts {}", speaker.restarts);
            println!("[sound_test] {status}");
            audio::log_amp(&mut i2c, "running");
            line(display, 190, Rgb565::CYAN, &status);
        }
    }
}
