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

The controller connects over Wi-Fi in one of two ways, chosen when the firmware is built:

- **The board's own network (no setup, no password).** With no `wifi.env` (or empty values) the board
  makes an open network called `CoreS3-DOOM` at `192.168.4.1` and runs a small DHCP server. Join it
  from the PC (the PC has no internet while it is joined, unless it has a second connection), then,
  from the repo root:

  ```bash
  cargo run -p core_s3_sender -- 192.168.4.1
  ```

- **Your own network.** Copy `wifi.env.example` to `wifi.env` (gitignored) and fill in `WIFI_SSID` /
  `WIFI_PASSWORD`; `build.rs` compiles them into the image (so treat the built `.elf` / `.bin` as
  sensitive). Only WPA2 (or WPA/WPA2 mixed) networks work. Use the address shown at the top of the LCD:

  ```bash
  cargo run -p core_s3_sender -- 192.168.68.103      # port 7878 unless given as host:port
  ```

Either way the LCD shows what to join and where to connect, and keeps the address in its top bar.

Or skip the sender and open the **web controller** in any browser (a phone works too): `http://<the
address on the LCD>/` (`http://192.168.4.1/` on the board's own network). It needs nothing installed.

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
- **Video:** originally the engine ran with `-scaling 1` and `draw_frame` copied the middle 320 columns
  of its 640-wide rows through the BSP's `blit_pixels`. That path is gone: the engine now hands over its
  320x200 indexed screen (`draw_indexed_frame`) and core 0 sends it to the LCD by DMA (`lcd.rs`, see
  `SPEEDUP.md`).

**Measured at the time of milestone 2** (before the work in `SPEEDUP.md`, which reaches about 20 fps): about 3 fps. The LCD transfer is about 58 ms per
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
fire, F use, 1-7 weapons, Tab map, Enter/Esc/Y/N, R toggles run, Ctrl-C quits) and prints these as a
keyboard drawing (`KEY_MAP`, checked against the bindings by a test) when it connects. Terminals with the
kitty keyboard protocol report real key releases; others get releases after `--hold-ms` (default 150).
The key mapping, hold tracking and the TCP path are unit tested; the terminal glue is not.

**Web controller** (`src/web.rs`, `assets/controller.html`, `../core_s3_ws`): the board serves one page
on port 80 and the page sends key events back over a WebSocket on `/ws`. Every byte of a message is
one protocol event, so it feeds the same queue as the TCP port. Three tasks listen, so one is always
free (a task serving a page or holding a WebSocket is not listening). The page has

- **buttons** for every action, held for as long as they are pressed (mouse or touch; several at once),
  and a sticky Run toggle. A press shorter than 120 ms is stretched to that, because the game only looks
  at held keys once per tic and a quick click would be missed;
- an editable **key list**: click + on an action and press a key to bind it, x to remove one, Reset to
  go back. It is kept in the browser (`localStorage`), and the keyboard works anywhere on the page;
- a **text box** whose input is sent immediately, in two modes. *Keys* (the default): each typed key
  does its binding, so `w` is forward and space is fire; holding a key repeats it, which keeps the
  action going, and Enter/arrows/Shift and the like act as real holds; a pasted run such as `wwwd`
  plays one key per 0.2 s. *Text*: each character goes to the game as typed (cheat codes such as
  `iddqd`, save names), which needs its own mode because letters like `d` and `q` are also bindings.

To make typing possible the protocol grew: codes 32-126 are typed characters (the code is the ASCII
value) and code 21 is Backspace; old bytes keep their meaning. `core_s3_ws` is a small `no_std` crate
(request parsing, the RFC 6455 handshake, a byte-at-a-time frame decoder) tested on the host with the
RFC's own examples. The page was tested in headless Chromium against the board (clicks, holds, keys,
typing in both modes, rebinding, reload, blur), and the server with a separate WebSocket client (two
connections at once, ping/pong, dropped connections).

**The board's own network** (`net.rs`, `../core_s3_dhcp`): with no credentials the firmware starts
esp-radio in access-point mode (open, channel 1, up to 4 clients, SSID `CoreS3-DOOM`) with the static
address `192.168.4.1/24`, and answers DHCP itself. `core_s3_dhcp` is a small `no_std` server (unit
tested on the host: offer, ack, nak, one address per MAC, full pool, junk input); it hands out
`192.168.4.2` upward and offers no gateway or DNS, so a laptop does not route anything else over it.
Checked on hardware: the network appears in a scan, open, on channel 1. Joining it and connecting the
sender was left to the user (the development laptop has one Wi-Fi adapter and would lose its
connection).

**Firmware layout**
- Core 0: esp-rtos scheduler plus an embassy executor running the Wi-Fi station (reconnects on drop)
  or the access point, DHCP, the TCP command server (`net.rs`), the web controller (`web.rs`), and the LCD pump that streams finished frames by DMA
  (`lcd.rs`). One controller at a time; keep-alive frees the slot if it
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

**Quitting switches the board off.** In the game, Esc, Up (it wraps to Quit Game), Enter, `y` make the
engine call the platform's `quit()`, which now writes the AXP2101 soft power-off bit through the BSP
(`src/power.rs`). Tested with USB connected: the board switched off (its USB port disappeared and stayed
away). To start it again press the power button or replug USB (the button is per M5's docs, not tried).
If the write ever fails or the chip leaves the board powered, `quit()` falls back to spinning, which
needs a reset.

**Not done / ideas:** no on-device key-echo or connection indicator beyond serial logs; the performance
work from milestone 2 still applies (about 3 fps).
