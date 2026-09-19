# core_s3: DOOM on the M5Stack CoreS3 Lite

Bare-metal (`no_std`, `esp-hal`) firmware for the [CoreS3 Lite](https://docs.m5stack.com/en/core/CoreS3-Lite).

## Milestones

1. **Colour cycle**: boot, show one solid colour on the LCD, change it every second.
2. **Demo playback**: run the engine with no input, showing the demo. WAD embedded via `include_bytes!`.

## Hardware facts (from the M5 docs; re-check against the schematic before relying on them)

| | |
|---|---|
| MCU | ESP32-S3, Xtensa LX7 dual core, 240 MHz |
| Memory | 16 MB flash, 8 MB PSRAM |
| LCD | 2.0" IPS, 320x240, ILI9342C, SPI |
| LCD pins | MOSI = GPIO37, SCK = GPIO36, CS = GPIO3, DC = GPIO35 |
| LCD reset | **AW9523B** pin P1_1 (not a GPIO) |
| LCD backlight | powered by **AXP2101** DLDO1 (not a GPIO, no PWM pin) |
| Internal I2C | SDA = GPIO12, SCL = GPIO11 |
| I2C addresses | AXP2101 = 0x34, AW9523B = 0x58 (touch 0x38, IMU 0x69, RTC 0x51, ...) |
| Download mode | hold reset ~3 s until the green LED lights, then release |

The important consequence: **nothing shows on the LCD until we have talked to the power
chip over I2C.** The backlight and the reset line both sit behind I2C devices, so "single
colour" is really "I2C bring-up + SPI display bring-up".

## Decisions made in this skeleton

- **Bare metal `esp-hal`, not ESP-IDF `std`.** `rust_doomgeneric` is already `#![no_std]` +
  `alloc`, and `doomgeneric_fs` abstracts file access, so no libc/newlib layer is needed.
- **Standalone workspace** (`[workspace]` in `core_s3/Cargo.toml`). The crate needs the `esp`
  toolchain and a non-host target; keeping it out of the root workspace means host
  `cargo build/test --workspace` is unaffected. It has its own `Cargo.lock`.
- **Own register writes for AXP2101/AW9523B**, no driver crates. We need about a dozen
  register writes; the available crates are small and unproven.
- **`mipidsi` + `embedded-hal-bus` for the LCD** in milestone 1 (it has an `ILI9342C` model).
  Milestone 2 may drop to raw `set_pixels`/DMA for the framebuffer blit.
- Pinned to `esp-hal` 1.2.x (needs Rust >= 1.95; the `esp` toolchain must be at least that).

## Status of the skeleton

Written: `Cargo.toml` (dependency resolution verified via `cargo generate-lockfile`),
`rust-toolchain.toml`, `.cargo/config.toml`, and a `main.rs` that only prints a heartbeat.
**Not yet compiled**: the `esp` toolchain is not installed on this machine, and Xtensa
support is not in upstream rustc, so neither the dependency features nor the `esp-hal` API
calls have been type-checked. Stage 0 fixes that.

## Milestone 1 plan

### Stage 0: host setup (one-off, needs your OK, installs into `~/.cargo` / `~/.rustup`)

- `cargo install espup espflash` (espup 0.17.x, espflash 4.6.x)
- `espup install --targets esp32s3` (installs the `esp` rustup toolchain + Xtensa GCC)
- `source ~/export-esp.sh` in each shell that builds (puts the Xtensa linker on `PATH`)
- Serial access: you are **not** in the `dialout` group (`id` shows adm/sudo/plugdev/...), so
  `/dev/ttyACM0` will be denied. `sudo usermod -aG dialout $USER`, then log out/in.
- Optional: `cargo install esp-generate`, generate a throwaway `esp32s3` project and diff its
  `Cargo.toml`/`main.rs`/`config.toml` against ours. It is the authoritative reference for
  the exact esp-hal 1.2 boilerplate.
- **Exit:** `cd core_s3 && cargo build --release` succeeds.

### Stage A: toolchain and flashing (skeleton already contains this)

- Connect the board, enter download mode if needed, `cargo run --release`.
- **Exit:** the serial monitor shows `core_s3 alive: N s`, counting once per second.
- Likely snags: board not in download mode on first flash; USB re-enumerates after reset so
  the monitor needs to reattach; app descriptor / bootloader mismatch (`esp_app_desc!`).

### Stage B: power and I2C bring-up (`src/board.rs`)

- I2C0 at 400 kHz on GPIO12 (SDA) / GPIO11 (SCL), blocking driver.
- Probe both devices and log the ID registers as a wiring sanity check (AXP2101 chip ID at
  reg 0x03, AW9523B ID at reg 0x10; expected values to be confirmed from the datasheets).
- AXP2101: set the LDO voltages and enable bits for the LCD/backlight rails (DLDO1 for the
  backlight, plus the other rails M5 enables at boot).
- AW9523B: configure port 1 as push-pull outputs, drive P1_1 (LCD_RST) low then high with
  delays.
- **Register values: port them from M5's own init code** (M5Unified `Power_Class` /
  `M5GFX` CoreS3 board setup, or the `M5CoreS3` library), not from memory. Keep each write
  as a named constant with a comment saying what it does.
- **Exit:** log shows both chip IDs; the backlight turns on (the panel glows white or shows
  noise, since it has no content yet).

### Stage C: SPI display (`src/display.rs`)

- SPI2 at 40 MHz (raise to 80 MHz later) on SCK=36, MOSI=37, CS=3, DC=35, no MISO.
  Wrap in `embedded_hal_bus::spi::ExclusiveDevice` and feed to `mipidsi::interface::SpiInterface`.
- `mipidsi::Builder::new(models::ILI9342CRgb565, di)`, no reset pin (already toggled in
  stage B), landscape orientation. IPS panels usually need `ColorInversion::Inverted` and
  possibly BGR order; the colour test below tells us which.
- **Exit:** the whole screen fills red once, edge to edge, no garbage rows/columns.
- Likely snags: wrong colour inversion/order, 320x240 offset or rotation, DC/MOSI pin
  mix-up. Note the SD card shares this SPI bus (CS = GPIO4); leave it deselected.

### Stage D: colour cycle (`src/main.rs`)

- Loop over red, green, blue, yellow, cyan, magenta, white, black; `fill_solid` full screen,
  wait 1 s. A blocking delay is fine here.
- Red/green/blue in that order is the acceptance test: it exposes swapped channels, missing
  inversion and byte-order errors in one go. A full-screen fill is about 150 KB, roughly
  30 ms at 40 MHz, so the 1 s cadence is not disturbed.
- **Exit (milestone 1 done):** the screen changes colour every second, colours are correct,
  and it survives a power cycle (boots without the serial monitor attached).

### Suggested commits

1. skeleton + stage A (this commit)
2. stage B (`board.rs`, I2C bring-up)
3. stage C (`display.rs`, one fill)
4. stage D (colour cycle), milestone 1 complete

## Things milestone 1 should not paint us into (milestone 2 look-ahead)

- **Partition table.** `include_bytes!` of a WAD (shareware is about 4 MB) will not fit the
  default ~1 MB app partition. Milestone 2 needs a custom table for the 16 MB flash and
  `espflash --flash-size 16mb`.
- **PSRAM.** The engine's static/heap needs go well past internal RAM. The LCD uses
  GPIO35-37, which are the octal-PSRAM pins on some ESP32-S3 parts, so this board's PSRAM
  must be quad mode. Confirm that before writing PSRAM init.
- **WAD access.** `include_bytes!` data is memory-mapped from flash, so no copy is needed.
  Implement a read-only in-memory `DoomFileSystem` over a `&'static [u8]`.
- **Blit path.** DOOM renders 320x200 8-bit palettised; the LCD is 320x240 RGB565. Convert
  through the palette in chunks and stream via SPI DMA rather than a per-pixel iterator.
- **Timing/stack.** The engine needs a millisecond tick (`SystemTimer`) and a much larger
  stack than the esp-hal default. Check whether the engine crate compiles for
  `xtensa-esp32s3-none-elf` with `build-std` early; it is `no_std`, but untested there.
