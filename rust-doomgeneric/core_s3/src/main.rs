//! M5Stack CoreS3 Lite firmware. Milestone 1 stage A: prove the toolchain and
//! flashing work by printing a heartbeat over USB-Serial-JTAG.
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{clock::CpuClock, delay::Delay, main};
use esp_println::println;

// Descriptor the ESP-IDF second-stage bootloader expects at the start of the image.
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let _peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let delay = Delay::new();
    let mut seconds = 0u32;
    loop {
        println!("core_s3 alive: {seconds}s");
        seconds += 1;
        delay.delay_millis(1000);
    }
}
