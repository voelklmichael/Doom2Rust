# core_s3: DOOM on the M5Stack CoreS3 Lite

Bare-metal (`no_std`, `esp-hal`) firmware for the [CoreS3 Lite](https://docs.m5stack.com/en/core/CoreS3-Lite).

## Milestones

1. **Board bring-up, something on the LCD.** DONE, confirmed on hardware (2026-09-19).
2. **Demo playback**: run the engine with no input, showing the demo. WAD embedded via
   `include_bytes!`.

## Running it

```bash
source ~/export-esp.sh          # puts the Xtensa linker on PATH
cd core_s3
cargo run --release             # builds, flashes, opens the serial monitor
```

One-off host setup: `cargo install espup espflash --locked`, `espup install --targets esp32s3`,
and membership of the `dialout` group. If the board is not detected, hold reset ~3 s until the
green LED lights, then release. The "LOAD segment with RWX permissions" linker warning is
harmless.

## How the crate is set up

- **Standalone workspace** (`[workspace]` in `Cargo.toml`, own `Cargo.lock`). It needs the `esp`
  toolchain and a non-host target, so host `cargo build/test --workspace` is unaffected.
- **Bare-metal `esp-hal`, not ESP-IDF `std` and not embassy.** `rust_doomgeneric` is `#![no_std]` +
  `alloc` and `doomgeneric_fs` abstracts file access. The engine is a blocking single-threaded loop,
  so an async executor buys nothing.
- **Board support from the `core-s3` crate**, a git dependency pinned to tag `v0.5.0` of
  [anapeksha/core-s3-rs](https://github.com/anapeksha/core-s3-rs). It does the I2C, power-chip and
  LCD bring-up. `main.rs` is currently its `display_widgets` example, verbatim.
- **Version pins must match the BSP.** `esp-hal` is a `links` crate, so `esp-hal`,
  `esp-println`, `esp-backtrace` and `esp-bootloader-esp-idf` are pinned to exactly the versions
  `core-s3` v0.5.0 uses (see the comment in `Cargo.toml`). Bump them together with the BSP tag.

## Hardware facts (M5 docs and M5GFX source)

| | |
|---|---|
| MCU | ESP32-S3, Xtensa LX7 dual core, 240 MHz |
| Memory | 16 MB flash, 8 MB PSRAM |
| LCD | 2.0" IPS, 320x240, ILI9342C **or** ILI9342E (told apart via the FT6336U touch firmware ID) |
| LCD pins | MOSI = GPIO37, SCK = GPIO36, CS = GPIO3, DC = GPIO35 (shared with SD MISO) |
| LCD reset | AW9523B pin P1_1, over I2C (0x58) |
| LCD backlight | AXP2101 DLDO1 rail, over I2C (0x34) |
| Internal I2C | SDA = GPIO12, SCL = GPIO11 |
| SD card | shares the LCD SPI bus, CS = GPIO4 |

## Milestone 2: open questions to settle before writing the blit

These are risks found while planning; each needs a quick check on the real board or in the code.

- **Does the engine compile for `xtensa-esp32s3-none-elf`?** It is `#![no_std]` + `alloc` but has
  never been built for this target. First step: add it as a path dependency and build, then fix
  whatever breaks (`build-std` already includes `alloc`).
- **Heap.** The engine's allocations go well past internal RAM. Use `esp-alloc` with the 8 MB PSRAM
  added as a heap region. The LCD uses GPIO35-37, which are the octal-PSRAM pins on some ESP32-S3
  parts, so PSRAM on this board must be quad mode. Confirm before writing PSRAM init.
- **WAD in flash.** `include_bytes!` of a WAD (shareware is about 4 MB) will not fit the default
  ~1 MB app partition. Needs a custom partition table for the 16 MB flash and
  `espflash --flash-size 16mb`. The data is memory-mapped, so no copy is needed; implement a
  read-only in-memory `DoomFileSystem` over a `&'static [u8]`.
- **Blit path.** DOOM renders 320x200 8-bit palettised; the LCD is 320x240 RGB565. Convert through
  the palette in chunks and stream to the LCD, first with the BSP's display API, later with SPI DMA
  if the frame rate needs it.
- **Time and stack.** The engine needs a millisecond tick (`SystemTimer`) and a much larger stack
  than the esp-hal default.
- **No input.** Milestone 2 runs the built-in demo loop, so no event source is needed yet.
