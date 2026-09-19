# core_s3: DOOM on the M5Stack CoreS3 Lite

Bare-metal (`no_std`, `esp-hal`) firmware for the [CoreS3 Lite](https://docs.m5stack.com/en/core/CoreS3-Lite).

## Milestones

1. **Board bring-up, something on the LCD.** DONE, confirmed on hardware (2026-09-19).
2. **Demo playback**: run the engine with no input, showing the demo. WAD embedded via
   `include_bytes!`. DONE, confirmed on hardware (2026-09-19).
3. **Input over Wi-Fi**: control the game from a PC over TCP. DONE, confirmed on hardware (2026-09-19).
4. **Sound effects** through the speaker (see "Sound" below), with a standalone speaker test:
   `cargo run --release --example sound_test`. DONE on hardware (2026-09-19).

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
- **layouts**: keys are remembered by their *position* (`KeyboardEvent.code`), like games do, so WASD
  stays a cluster on Neo2 or AZERTY, and are labelled with what they print on the user's keyboard.
  A browser only tells the page its layout on secure pages (Chromium, `navigator.keyboard`), which the
  board's plain `http://` page is not, so labels are learned as keys are pressed or bound (keys not yet
  seen show the US name, in italics). Arrows, Enter, Esc, Tab and Backspace are remembered by name, so
  Neo2's cursor layer (an arrow produced by another key) works. The text box in Keys mode identifies
  keys exactly like the key list (by the key event, nothing is typed into the box); input that only
  arrives as characters (a phone keyboard, a paste) is matched against what keys are known to print;
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

## Sound (sound effects; music is not on the device yet)

**Status:** builds, flashes, runs; the serial log shows real audio flowing (see below). Whether it
sounds right was judged by the owner's ears: effects play, and were too loud at first (see Volume).

**Hardware.** Speaker amp AW88298 (I2C 0x36, reset line = AW9523B P0_2, released by the BSP's power
init), fed by I2S1 on DMA channel 1 (the LCD uses channel 0): BCLK = GPIO34, WS = GPIO33, DOUT = GPIO13,
Philips I2S, 16-bit slots. The amp is an I2S slave that derives its clock from BCLK, so no MCLK
(GPIO0) is needed. Enabling the amp needs a running BCLK first, so the I2S is started (sending
silence), then the amp is reset and configured (M5Unified's register sequence, with the rate bucket
for 11025 Hz). After that its status register changes from 0x4000 to 0x0311 (clock/PLL locked, as
far as one can tell without the datasheet). The amp's volume register stays at "full".

**Code.** `src/audio.rs` (I2S ring + amp; shared with the example), `src/sound.rs` (queue and the
pump task on core 0, mute), `../core_s3_audio` (lock-free frame queue, host tested),
`examples/sound_test.rs`, the three audio hooks in `src/platform.rs`. The game (core 1) mixes into a
queue and never waits; a task on core 0 moves audio into the DMA ring one 512-byte chunk at a time and
pads with silence when the game is late (a level load), so a stall is a gap, not a replay of old audio.

**Rate, ring, memory.** 11025 Hz, sent as stereo with both channels equal (one speaker; the engine's
left/right panning becomes a level difference). 11025 Hz is what the sound effects are recorded at, so
the engine does no resampling. The ring is three 128-frame chunks (1.5 KB, 35 ms); the queue holds
1024 frames (4 KB): about 6 KB of internal RAM in all (the frame buffer for the LCD is 128 KB). Delay from game to speaker is about 60-80 ms.
esp-hal's I2S driver always builds 4092-byte descriptors, except that a circular buffer of at most
8184 bytes is cut into exactly three, so three chunks is the finest ring possible; asking for smaller
descriptors with `dma_buffers!` is silently ignored (an early version did that: free-space counts made
no sense).

**Volume and mute.**
- `VOLUME_256THS` in `src/sound.rs` (currently 32, i.e. -12 dB below an earlier default that was "very
  distracting"; 256 would be the engine's own level) is applied to the samples. Change it and rebuild.
  The in-game sound volume slider (Options) works on top of it; its default (8 of 15) is the same as
  vanilla DOOM.
- Mute at run time, no reflash: the **Sound on/off** button (or the M key) on the web controller, or
  `M` in `core_s3_sender`. It is a new protocol command (`Command::ToggleSound`, code 22) that the
  firmware handles itself; muted, the game keeps mixing and the ring is fed silence, so nothing else
  changes. The serial log says `[audio] muted` / `unmuted`. Seen working on the device: a browser
  joined the board's network, opened the controller and pressed the button, the log printed
  `[audio] muted`, and from then on the pump fed silence only (the fps is the same either way). The
  sender's `M` is only covered by host tests.
- Build time: `SOUND=off cargo build --release` leaves the speaker unset up completely, `SOUND=muted`
  starts muted.

**Measured** (`[perf]`, same attract-mode demo, first 22 windows of 2 s, mean): built with
`SOUND=off` 29.0 fps, with sound 28.0 fps (-1.0 fps, about 3.5%). Title-screen windows are unchanged
(33.1 vs 32.9); in the demo's gameplay windows sound costs 1 to 1.9 fps (e.g. 28.9 -> 27.0; the busiest
window 26.0 -> 23.4 in one run, 24.9 in another). Running muted costs the same, so it is not the pump
or the copy into the ring. The `[perf]` line now ends with `(of it sound N)`: the engine's mixing plus
handing the audio over is 0.5-0.8 ms per frame; the rest of the loss (about 1.5 ms per frame) is not
profiled: most likely the engine's own per-tic bookkeeping of sound channels, which now stay alive for
the length of each sound (with no mixer they were dropped at once). The mixer in
`engine/src/sfx_mixer.rs` divides by 255 twice for every voice and output frame; doing that division
once per output frame would likely recover part of the 0.5-0.8 ms (not tried: the mixer is also being
changed by the music work).

**Open.** The pump has to run at least every 35 ms or the ring runs dry and esp-hal's bookkeeping
forces a restart of the transfer (a short glitch, counted in the `DMA restarts` log field; one at
startup is normal, later ones would be a sign of a stalled core 0). The longest gap seen is 3 ms.
Music (OPL synthesis) on the device is untried; it would need a higher rate (the amp supports up to
96 kHz) and its own CPU budget.
