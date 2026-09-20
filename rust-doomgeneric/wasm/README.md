# doomgeneric_wasm

DOOM in the browser: the engine compiled to WebAssembly. The page is a static site of about 1 MB
with **no game data in it**: you drop a DOOM IWAD (`doom1.wad`, `doom.wad`, `doom2.wad`, ...) onto
the game window, or pick one with the button. The shareware `doom1.wad` works, and so does the WAD
of a game you own. The game starts as soon as the file is chosen. The file is not uploaded
anywhere; it is read by your browser and remembered in its storage, so it only has to be dropped
once (on later visits the page says "Click to play").

Saved games (and the game's config) are kept in the browser too, per game, so they are still there
after a reload. The game stops, clock and sound included, while its tab is hidden.

## Build and run

Needs the `wasm32-unknown-unknown` target and the `wasm-bindgen` command line tool at the version
pinned in `Cargo.toml`:

```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.121
wasm/build.sh serve        # then open http://localhost:8000
```

`build.sh` without `serve` only builds into `www/pkg`. `www/` is a static site; serve it with any
web server (opening the file directly does not work: browsers do not load modules or workers from
`file:`).

## Publishing on GitHub Pages

`.github/workflows/pages.yml` builds this crate and deploys `www/` on every push to `main` that
touches the engine or this crate (and on demand from the Actions tab). Once, in the repository:
**Settings > Pages > Build and deployment > Source: GitHub Actions**. The site is then at
`https://<user>.github.io/<repo>/`. Everything the page loads is a relative path, so it works under
that sub-path.

## How it fits together

* `src/lib.rs` exports `Setup` (checks the IWAD, takes the files kept from earlier visits, and
  starts the game) and `Doom` (the running game: call `tick()` about 35 times a second, forward
  keys with `key_event`). `src/platform.rs` is its `DoomPlatform` and `src/fs.rs` its
  `DoomFileSystem`: the IWAD, plus the files the game writes, which are reported to the page as
  they change. `src/wad.rs` tells which game an IWAD is from its levels, not from its file name
  (the engine looks at the name), so a download called `Doom (1).WAD` works.
* The game runs in a Web Worker (`www/worker.js`). The engine draws the screen melt in a loop inside
  one tick, so on the page's own thread the picture would freeze for a second. The platform calls
  the page as soon as it has a frame or sound instead of leaving them to be picked up after the
  tick. The page (`www/main.js`) draws the newest frame once per screen refresh, schedules the
  sound with Web Audio and sends the keyboard to the worker.
* A browser holds sound back until the player has pressed a key or clicked, and dropping a file is
  neither. The game therefore starts silent in that case, and the page lets the sound through at the
  next key press or click.
* `www/storage.js` keeps the WAD and the game's files in IndexedDB. Without it (a private window)
  the game still runs; it just remembers nothing.
* To pause, the worker stops ticking and takes the paused time off the clock it gives the game, so
  the game continues where it stopped instead of running the missed time at once.
* Frames use the engine's 320x200 palette-index screen (`draw_indexed_frame`), which the page shows
  at 4:3.

## Not there yet

Mouse and touch input, several WADs at once (a PWAD on top of an IWAD), and a way to manage or
delete the saved games.
