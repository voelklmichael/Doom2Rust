# doom_v5

DOOM in Rust, for three platforms: a Linux desktop (X11), a web browser (WebAssembly), and the
[M5Stack CoreS3 Lite](https://docs.m5stack.com/en/core/CoreS3-Lite) (ESP32-S3 microcontroller,
320x240 LCD, controlled over Wi-Fi).

The Rust code is a port of [doomgeneric](https://github.com/ozkl/doomgeneric), a small
platform-independent version of the original DOOM source. It began as a mechanical `c2rust`
translation and was then reworked by hand into safe, idiomatic Rust. The engine has no `unsafe`
code (`deny(unsafe_code)`) and is `#![no_std]` + `alloc`, which is what lets it run on the
microcontroller and in WebAssembly. Where its behaviour differs from the C original on purpose, it is listed in
[docs/known-deviations.md](docs/known-deviations.md).

You need a DOOM IWAD (game data). It is not included; the freely available shareware
`doom1.wad` works. On Linux you pass its path, in the browser you drop the file onto the page, and on
the CoreS3 it is built into the firmware.

## Layout

| Path | What it is |
|---|---|
| `doomgeneric/` | The original C sources, kept for reference and comparison |
| `rust-doomgeneric/engine` | The game engine (`rust_doomgeneric`), platform independent |
| `rust-doomgeneric/fs` | File-access abstraction the engine uses (`doomgeneric_fs`) |
| `rust-doomgeneric/x11` | Linux front end: X11 window, sound through `aplay`/`paplay` |
| `rust-doomgeneric/wasm` | Browser front end: WebAssembly module, page and Web Worker (`doomgeneric_wasm`) |
| `rust-doomgeneric/core_s3` | CoreS3 Lite firmware (bare-metal `esp-hal`) |
| `rust-doomgeneric/core_s3_*` | Host and firmware helpers: input protocol, keyboard sender, DHCP server, audio, WebSocket |
| `rust-doomgeneric/third_party/oplon` | Local copy of the OPL2 music synthesizer |
| `docs/` | Design notes and history |

## Build and run on Linux (X11)

Requirements: a Rust toolchain (stable, via [rustup](https://rustup.rs)), the X11 development
headers, and a running X11 session. Sound is optional and uses `aplay` (ALSA) or `paplay`
(PulseAudio) if installed.

```bash
# Debian / Ubuntu
sudo apt install build-essential libx11-dev

cd rust-doomgeneric
cargo run --release -p doomgeneric_xlib -- -iwad /path/to/doom1.wad
```

Use `--release`: a debug build is far too slow, and it panics on the integer overflows that DOOM's
fixed-point math relies on. The binary ends up in `rust-doomgeneric/target/release/doomgeneric_xlib`.

Sound is on by default. Pass `-nosound` (or set `DOOM_AUDIO=off`) to silence a run.

Run the tests with `cargo test --release` from `rust-doomgeneric/`.

## Build and run in the browser (WebAssembly)

The game also runs in a browser, from a static web page that contains no game data: you drop your
IWAD onto the game window (or pick it with the button), and the page remembers it. Saved games are
kept in the browser and survive a reload, and the game pauses while its tab is hidden. Sound works;
mouse and touch input do not yet.

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.121   # has to match the wasm-bindgen crate
rust-doomgeneric/wasm/build.sh serve               # then open http://localhost:8000
```

`build.sh` without `serve` only builds the site into `rust-doomgeneric/wasm/www`, which any web
server can host (browsers do not run it from `file:`). Controls: arrows move, Ctrl fires, Space
opens doors, Shift runs, Alt with the arrows (or `,` and `.`) strafes, `1`-`7` pick weapons, Tab
opens the map, Esc the menu; double-click the game window for full screen.

**Publishing on GitHub Pages.** `.github/workflows/pages.yml` builds the site and deploys it on
every push to `main` that touches the engine or the `wasm` crate. It needs one setting, once:
in the GitHub repository, **Settings > Pages > Build and deployment > Source: GitHub Actions**
(the repository has to be public, unless you have a paid plan). The site is then at
`https://<user>.github.io/<repo>/`. More in
[rust-doomgeneric/wasm/README.md](rust-doomgeneric/wasm/README.md).

## Build and flash the CoreS3 Lite

The firmware is its own Cargo workspace (`rust-doomgeneric/core_s3`). It targets
`xtensa-esp32s3-none-elf`, which needs Espressif's Rust toolchain, so a normal
`cargo build --workspace` on the host does not touch it.

**One-time setup**

```bash
cargo install espup espflash --locked
espup install --targets esp32s3       # installs the `esp` toolchain, writes ~/export-esp.sh
sudo usermod -aG dialout "$USER"      # serial port access; log out and in afterwards
```

The first build downloads the board support crate from GitHub, so it needs network access.

**Game data.** Put the shareware IWAD at `rust-doomgeneric/core_s3/assets/doom1.wad`. It is
embedded into the firmware image and is gitignored. The app partition is 8 MB, so the full
commercial WAD (about 11 MB) does not fit. See `core_s3/assets/README.md`.

**Wi-Fi (optional).** By default the board creates its own open network called `CoreS3-DOOM`
(address `192.168.4.1`). To have it join your network instead, copy `core_s3/wifi.env.example` to
`core_s3/wifi.env` and fill in `WIFI_SSID` and `WIFI_PASSWORD`. Only WPA2 networks work. The
credentials are compiled into the firmware image, so do not share the built `.elf` or `.bin`.

**Build, flash and open the serial monitor** (board connected over USB):

```bash
source ~/export-esp.sh        # puts the Xtensa linker on PATH; needed in every new shell
cd rust-doomgeneric/core_s3
cargo run --release
```

If the board is not detected, hold the reset button for about 3 seconds until the green LED lights,
then release it. The linker warning "LOAD segment with RWX permissions" is harmless. To check the
speaker on its own, run `cargo run --release --example sound_test`.

**Controlling the game.** Once running, the LCD shows the network to join and the address to
connect to. There are two ways to play:

- **In a browser** (a phone works too): open `http://<address shown on the LCD>/`. The page has
  on-screen buttons and rebindable keyboard keys, and needs nothing installed.
- **From a terminal on the PC**, using the keyboard sender:

  ```bash
  cd rust-doomgeneric
  cargo run -p core_s3_sender -- 192.168.4.1        # or the address on the LCD; port 7878
  ```

  Arrows or WASD move, Q/E strafe, Space fires, F uses, 1-7 select weapons, Tab opens the map,
  and Ctrl-C quits.

More detail (hardware, performance, protocol, sound) is in
[rust-doomgeneric/core_s3/PLAN.md](rust-doomgeneric/core_s3/PLAN.md) and
[SPEEDUP.md](rust-doomgeneric/core_s3/SPEEDUP.md).

## The C version

The original C build is described in [docs/build-and-run-c.md](docs/build-and-run-c.md). The
upstream project's own README is [doomgeneric/README.md](doomgeneric/README.md).

## License

GNU General Public License v2, like the DOOM source it derives from. See
[doomgeneric/LICENSE](doomgeneric/LICENSE).
