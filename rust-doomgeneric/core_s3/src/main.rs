//! DOOM on the M5Stack CoreS3 Lite. Milestone 2: play the shareware attract-mode demos
//! from a WAD embedded in the firmware image, with no input.
#![no_std]
#![no_main]

extern crate alloc;

mod platform;
mod wad_fs;

use alloc::{boxed::Box, string::ToString, vec::Vec};
use core_s3::{
    CoreS3,
    bsp::{CoreS3DisplayParts, CoreS3DisplayResources},
};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    psram::{PsramConfig, PsramMode},
};
use esp_println::println;
use rust_doomgeneric::{doomgeneric_create, doomgeneric_tick, init_game_state};

use platform::CoreS3Platform;
use wad_fs::EmbeddedWad;

esp_bootloader_esp_idf::esp_app_desc!();

/// Shareware IWAD, embedded in flash and read in place. See `assets/README.md`.
static WAD: &[u8] = include_bytes!("../assets/doom1.wad");

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    // The engine's state and frame buffer are far bigger than the internal RAM, so the whole
    // heap lives in the 8 MB PSRAM. The CoreS3's PSRAM is quad SPI; asking for it explicitly
    // keeps esp-hal from probing octal mode on GPIO35-37, which are the LCD pins.
    esp_alloc::psram_allocator!(
        peripherals.PSRAM,
        esp_hal::psram,
        PsramConfig { mode: PsramMode::QuadSpi, ..Default::default() }
    );
    println!("heap: {} bytes free", esp_alloc::HEAP.free());

    let CoreS3DisplayParts { display, internal_i2c: _i2c } =
        CoreS3::init_display(CoreS3DisplayResources {
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

    let state = init_game_state(
        Box::new(CoreS3Platform::new(display)),
        Box::new(EmbeddedWad::new("doom1.wad", WAD)),
    );
    println!("game state initialised, {} bytes free", esp_alloc::HEAP.free());

    let args: Vec<_> = ["doomgeneric", "-iwad", "doom1.wad", "-scaling", "1"]
        .into_iter()
        .map(ToString::to_string)
        .collect();
    doomgeneric_create(state, args);
    println!("doomgeneric_create done, {} bytes free", esp_alloc::HEAP.free());

    loop {
        doomgeneric_tick(state);
    }
}
