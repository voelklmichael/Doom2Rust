# doomgeneric_wasm

DOOM in the browser: the engine compiled to WebAssembly, with the shareware IWAD linked into the
module. Nothing is downloaded but the page, a script and the 5 MB module.

## Build and run

Needs the `wasm32-unknown-unknown` target, the `wasm-bindgen` command line tool at the version
pinned in `Cargo.toml`, and the IWAD in `assets/` (see `assets/README.md`):

```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.121
wasm/build.sh serve        # then open http://localhost:8000
```

`build.sh` without `serve` only builds into `www/pkg`. `www/` is a static site; serve it with any
web server (opening the file directly does not work, browsers do not load modules or workers from
`file:`).

## How it fits together

* `src/lib.rs` exports `Doom` (make one, call `tick()` about 35 times a second, forward keys with
  `key_event`). `src/platform.rs` is its `DoomPlatform`; `src/fs.rs` its `DoomFileSystem` (the
  WAD, plus files the game writes, such as savegames, in memory: they are gone on reload).
* The game runs in a Web Worker (`www/worker.js`). The engine draws the screen melt in a loop inside
  one tick, so on the page's own thread the picture would freeze for a second. The platform calls
  the page as soon as it has a frame or sound instead of leaving them to be picked up after the
  tick. The page (`www/main.js`) draws the newest frame once per screen refresh, schedules the
  sound with Web Audio and sends the keyboard to the worker.
* Frames use the engine's 320x200 palette-index screen (`draw_indexed_frame`), which the page shows
  at 4:3.

## Not there yet

Mouse and touch input, saving to `localStorage`, the full game's WAD (or picking a WAD), pausing
when the tab is hidden.
