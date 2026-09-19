# core_s3: DOOM on the M5Stack CoreS3 Lite

Bare-metal (`no_std`, `esp-hal`) firmware for the [CoreS3 Lite](https://docs.m5stack.com/en/core/CoreS3-Lite).

## Milestones

1. **Board bring-up, something on the LCD.** DONE, confirmed on hardware (2026-09-19).
2. **Demo playback**: run the engine with no input, showing the demo. WAD embedded via
   `include_bytes!`. DONE, confirmed on hardware (2026-09-19).
3. **Input over Wi-Fi**: control the game from a PC over TCP. DONE, confirmed on hardware (2026-09-19).

## Running it

```bash
source ~/export-esp.sh          # puts the Xtensa linker on PATH
cd core_s3
cargo run --release             # builds, flashes, opens the serial monitor
```

Wi-Fi input needs credentials: copy `wifi.env.example` to `wifi.env` (gitignored) and fill in
`WIFI_SSID` / `WIFI_PASSWORD`; `build.rs` compiles them into the image (so treat the built `.elf` /
`.bin` as sensitive). Without them the firmware still runs the demos with Wi-Fi off. Only WPA2 (or
WPA/WPA2 mixed) networks work. Then, from the repo root, with the address shown at the top of the LCD:

```bash
cargo run -p core_s3_sender -- 192.168.68.103      # port 7878 unless given as host:port
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

## Milestone 2: how it works, and measured performance

- The engine (`../engine`, unchanged) is a path dependency. It builds for
  `xtensa-esp32s3-none-elf` as is.
- **Heap:** the whole heap is the 8 MB PSRAM, initialised in quad mode explicitly (the LCD uses
  GPIO35-37, the octal-PSRAM pins). `GameState` builds on the stack without overflowing.
- **WAD:** `assets/doom1.wad` (gitignored, shareware, see `assets/README.md`) is embedded with
  `include_bytes!` and served by `EmbeddedWad`, a read-only `DoomFileSystem`. The default espflash
  partition table is too small, so `partitions.csv` gives the app 8 MB (image is 5.2 MB).
- **Video:** the engine runs with `-scaling 1`; `CoreS3Platform::draw_frame` copies the middle
  320 columns of its 640-wide rows to the LCD through the BSP's `blit_pixels`, centred vertically.

**Measured** (`[perf]` line on serial every 2 s): about 3 fps. The LCD transfer is about 58 ms per
frame (the theoretical minimum at 40 MHz is about 26 ms); everything else is 200-280 ms per frame,
so the engine side dominates. The split inside that is not measured yet. Candidates: the engine
allocating and filling a ~512 KB frame in PSRAM every frame, all game data living in PSRAM, and the
renderer itself. Speed was judged good enough for now; profile there first if it needs work.

**Known oddity:** the engine prints `Demo is from a different game version! (read 108, should be
109)` once during the demo loop and then carries on. Not investigated.

## Milestone 3: input over Wi-Fi

**Protocol** (`../core_s3_protocol`, shared by firmware and sender): a plain TCP stream on port 7878
where every byte is one command. Bit 7 = 1 means "released", bits 0-6 are the command code (forward,
turn, strafe, fire, use, run, menu keys, weapons 1-7). Press and release are separate so a held key
stays held exactly as long as the sender says. Unknown codes are ignored.

**Sender** (`../core_s3_sender`): reads the keyboard in a terminal (arrows or WASD, Q/E strafe, Space
fire, F use, 1-7 weapons, Tab map, Enter/Esc/Y/N, R toggles run, Ctrl-C quits). Terminals with the
kitty keyboard protocol report real key releases; others get releases after `--hold-ms` (default 150).
The key mapping, hold tracking and the TCP path are unit tested; the terminal glue is not.

**Firmware layout**
- Core 0: esp-rtos scheduler plus an embassy executor running the Wi-Fi station (reconnects on drop),
  DHCP and the TCP command server (`net.rs`). One controller at a time; keep-alive frees the slot if it
  vanishes, and any keys it held are released on disconnect.
- Core 1: builds and runs the game (`main.rs`), reading events through `get_key()`.
- The controller address is shown on the LCD while starting and stays in the top black bar.

**Things learned the hard way**
- Building the engine state by value (290 KB) overflowed the main thread's stack once esp-rtos and the
  Wi-Fi heap took internal RAM: esp-hal's stack guard shows up as "Unhandled interrupt" in `memset`. The
  game therefore runs on core 1 on a 1 MiB stack allocated from PSRAM.
- The allocator uses the first region that fits, so PSRAM is added to the heap first (the game lands
  there) and the two internal regions second (the radio asks for internal RAM explicitly).
- A board sitting in the panic handler cannot be reflashed: `espflash` hangs at "Connecting...". Hold
  reset ~3 s until the green LED lights to enter download mode.
- `wifi.env` is read by `build.rs` from the crate directory; edit that file, not `wifi.env.example`.

**Not done / ideas:** no on-device key-echo or connection indicator beyond serial logs; the performance
work from milestone 2 still applies (about 3 fps).
