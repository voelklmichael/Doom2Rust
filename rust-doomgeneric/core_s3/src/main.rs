//! DOOM on the M5Stack CoreS3 Lite, controlled over Wi-Fi.
//!
//! Core 0 runs the esp-rtos scheduler, Wi-Fi, the TCP command server (`net`) and the LCD (`lcd`).
//! Core 1 builds and runs the game. Before the game starts, core 0 shows the board's IP address on
//! the LCD so the controller (`core_s3_sender`) knows where to connect.
#![no_std]
#![no_main]

extern crate alloc;

mod lcd;
mod net;
mod platform;
mod wad_fs;

use alloc::{boxed::Box, string::ToString, vec::Vec};
use core::{cell::RefCell, fmt::Write as _};
use core_s3::{
    bsp, devices,
    display::{BusConfig, Display, DisplayGeometry, PanelConfig},
    ui::Label,
    CoreS3,
};
use core_s3_protocol::DEFAULT_PORT;
use embassy_executor::Spawner;
use embassy_time::{with_timeout, Duration, Timer};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    i2c::master::{Config as I2cConfig, I2c},
    interrupt::software::SoftwareInterruptControl,
    psram::{PsramConfig, PsramMode, SpiRamFreq},
    ram,
    system::Stack,
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::println;
use heapless::String;
use rust_doomgeneric::{doomgeneric_create, doomgeneric_tick, init_game_state};

use platform::CoreS3Platform;
use wad_fs::EmbeddedWad;

esp_bootloader_esp_idf::esp_app_desc!();

/// Shareware IWAD, embedded in flash and read in place. See `assets/README.md`.
static WAD: &[u8] = include_bytes!("../assets/doom1.wad");

/// Stack of the game's thread on core 1. `init_game_state` builds the engine's state (hundreds of
/// KB) by value before boxing it, so the stack has to be far bigger than internal RAM can spare.
const GAME_STACK_SIZE: usize = 1024 * 1024;

/// A stack for core 1, allocated from the heap (which puts it in PSRAM).
fn game_stack() -> &'static mut Stack<GAME_STACK_SIZE> {
    let stack = Box::<Stack<GAME_STACK_SIZE>>::new_uninit();
    // SAFETY: `Stack` consists of nothing but `MaybeUninit` bytes, so it is always initialised.
    Box::leak(unsafe { stack.assume_init() })
}

const WAIT_FOR_ADDRESS: Duration = Duration::from_secs(20);
const SHOW_ADDRESS: Duration = Duration::from_secs(3);

fn show<D>(display: &mut D, lines: &[&str])
where
    D: DrawTarget<Color = Rgb565>,
    D::Error: core::fmt::Debug,
{
    display.clear(Rgb565::BLACK).expect("clear LCD");
    for (line, y) in lines.iter().zip((30..).step_by(30)) {
        Label { text: line, top_left: Point::new(20, y), color: Rgb565::CYAN }
            .draw(display)
            .expect("draw text");
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    // The game's state and frame buffer are far bigger than internal RAM, so most of the heap is
    // the 8 MB PSRAM. It goes in first: the allocator uses the first region that fits, so the
    // game lands in PSRAM while the Wi-Fi driver, which asks for internal RAM explicitly, gets
    // the two internal regions. The PSRAM is quad SPI; asking for that explicitly keeps esp-hal
    // from probing octal mode on GPIO35-37, which are the LCD pins.
    esp_alloc::psram_allocator!(
        peripherals.PSRAM,
        esp_hal::psram,
        PsramConfig {
            mode: PsramMode::QuadSpi,
            ram_frequency: SpiRamFreq::Freq80m,
            ..Default::default()
        }
    );
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);

    // The scheduler must be running before the radio is initialised.
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let software_interrupts = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, software_interrupts.software_interrupt0);

    // Panel power, reset and backlight come from the BSP; the SPI side is our own (`lcd`), because
    // the BSP's display initialiser cannot hand the bus over to DMA.
    let mut i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_hz(bsp::INTERNAL_I2C_HZ)),
    )
    .expect("I2C")
    .with_sda(peripherals.GPIO12)
    .with_scl(peripherals.GPIO11);
    CoreS3::init_core_s3_power(&mut i2c).expect("LCD power");
    let lcd = RefCell::new(lcd::Lcd::new(
        peripherals.SPI2,
        peripherals.GPIO36,
        peripherals.GPIO37,
        peripherals.GPIO3,
        peripherals.GPIO35,
        peripherals.DMA_CH0,
    ));
    lcd::init_frames();
    // The TF-card slot shares the bus; keep its chip select high.
    let sd_cs = Output::new(peripherals.GPIO4, Level::High, OutputConfig::default());
    let mut display = Display::new(
        lcd::LcdSpi(&lcd),
        lcd::LcdDc(&lcd),
        sd_cs,
        BusConfig { write_hz: lcd::SPI_HZ },
        PanelConfig {
            invert_colors: true,
            geometry: DisplayGeometry {
                width: devices::display::WIDTH,
                height: devices::display::HEIGHT,
                offset_x: 0,
                offset_y: 0,
            },
        },
    );
    display.init(&mut Delay::new()).expect("display");

    show(&mut display, &["CoreS3 DOOM", "starting Wi-Fi..."]);
    // What the game keeps showing in its top bar once it is running.
    let mut status = String::<40>::new();
    match net::start(spawner, peripherals.WIFI) {
        None => {
            show(&mut display, &["CoreS3 DOOM", "Wi-Fi not configured", "no network input"]);
            let _ = write!(status, "no Wi-Fi");
        }
        Some(stack) => match with_timeout(WAIT_FOR_ADDRESS, stack.wait_config_up()).await {
            Ok(()) => {
                let mut address = String::<40>::new();
                if let Some(config) = stack.config_v4() {
                    let _ = write!(address, "{}", config.address.address());
                }
                let mut port = String::<40>::new();
                let _ = write!(port, "port {DEFAULT_PORT}");
                println!("wifi: address {address}, controller port {DEFAULT_PORT}");
                show(&mut display, &["CoreS3 DOOM", "controller address:", &address, &port]);
                let _ = write!(status, "{address}:{DEFAULT_PORT}");
                Timer::after(SHOW_ADDRESS).await;
            }
            Err(_) => {
                println!("wifi: no address after {WAIT_FOR_ADDRESS:?}; starting the game anyway");
                show(&mut display, &["CoreS3 DOOM", "no Wi-Fi yet", "still retrying"]);
                let _ = write!(status, "no Wi-Fi yet");
                Timer::after(SHOW_ADDRESS).await;
            }
        },
    }

    // What the game keeps showing: a black screen with the status in its top bar, which the game
    // never draws over.
    display.clear(Rgb565::BLACK).expect("clear LCD");
    Label { text: &status, top_left: Point::new(4, 5), color: Rgb565::CYAN }
        .draw(&mut display)
        .expect("draw status");

    // The game runs on core 1, so the network never waits for a frame and the game never waits
    // for the radio. It builds its own state there, on the big stack.
    esp_rtos::start_second_core(
        peripherals.CPU_CTRL,
        software_interrupts.software_interrupt1,
        game_stack(),
        move || {
            let state = init_game_state(
                Box::new(CoreS3Platform::new()),
                Box::new(EmbeddedWad::new("doom1.wad", WAD)),
            );
            println!(
                "game state built: {} bytes, {} bytes of heap free",
                core::mem::size_of_val(&*state),
                esp_alloc::HEAP.free()
            );
            let args: Vec<_> = ["doomgeneric", "-iwad", "doom1.wad", "-scaling", "1"]
                .into_iter()
                .map(ToString::to_string)
                .collect();
            doomgeneric_create(state, args);
            loop {
                doomgeneric_tick(state);
            }
        },
    );

    // Core 0 streams the finished frames to the LCD; the network tasks keep running on this
    // executor in between.
    lcd::run_pump(&lcd).await

}
