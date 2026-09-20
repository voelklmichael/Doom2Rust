//! DOOM in the browser: the engine compiled to WebAssembly. The game data is not part of it: the
//! player drops an IWAD on the page.
//!
//! The page (`www/`) runs this in a Web Worker: it makes a [`Setup`] from the IWAD, hands it the
//! files kept from earlier visits and starts it, which gives a [`Doom`]. It then calls
//! [`Doom::tick`] about 35 times a second and forwards key events; frames, sound and the files
//! the game writes come back through the `Host` object it passes to [`Setup::start`].
#![deny(unsafe_code)]

mod audio;
mod fs;
mod keys;
mod platform;
mod wad;

use platform::{BrowserPlatform, KeyQueue};
use rust_doomgeneric::{doomgeneric_create, doomgeneric_tick, init_game_state, GameState, Options};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(message: &str);
}

/// Which build this is (date and commit, see build.sh), for the page to compare with its own.
#[wasm_bindgen]
pub fn build_version() -> String {
    option_env!("DOOM_BUILD").unwrap_or("unknown").to_string()
}

#[wasm_bindgen]
pub struct Doom {
    state: &'static mut GameState,
    keys: KeyQueue,
}

/// A game that is not running yet: the IWAD, and the files kept from earlier visits.
#[wasm_bindgen]
pub struct Setup {
    wad_name: &'static str,
    wad: Vec<u8>,
    saved: Vec<(String, Vec<u8>)>,
}

#[wasm_bindgen]
impl Setup {
    /// Fails, saying why, if `wad` is not an IWAD the game can run.
    #[wasm_bindgen(constructor)]
    pub fn new(wad: Vec<u8>) -> Result<Setup, JsError> {
        let wad_name = wad::identify(&wad).map_err(|reason| JsError::new(&reason))?;
        Ok(Setup {
            wad_name,
            wad,
            saved: Vec::new(),
        })
    }

    /// What files kept for this IWAD are filed under: savegames must not be offered to another
    /// game, so the page keeps them apart by this.
    pub fn storage_key(&self) -> String {
        format!("{}:{}", self.wad_name, self.wad.len())
    }

    /// A file the game wrote on an earlier visit (see `Host.store`), to be there again.
    pub fn add_file(&mut self, path: &str, data: Vec<u8>) {
        self.saved.push((path.to_string(), data));
    }

    /// Starts the game: loads the WAD and everything else the engine needs before its first
    /// frame, which takes a moment.
    pub fn start(self, host: platform::Host) -> Doom {
        // A panic (the engine's `I_Error`) is an unreachable trap, which says nothing.
        std::panic::set_hook(Box::new(|info| error(&info.to_string())));
        let keys = KeyQueue::default();
        let host = Rc::new(host);
        let state = init_game_state(
            Box::new(BrowserPlatform::new(
                Rc::clone(&host),
                KeyQueue::clone(&keys),
            )),
            Box::new(fs::BrowserFs::new(
                self.wad_name,
                self.wad,
                self.saved,
                host,
            )),
        );
        let options = Options {
            iwad: Some(self.wad_name.to_string()),
            scaling: Some(1),
            ..Options::default()
        };
        doomgeneric_create(state, options);
        Doom { state, keys }
    }
}

#[wasm_bindgen]
impl Doom {
    /// Runs the game until the next tic (1/35 s of game time) is due, and draws it. Call this
    /// again right after the tic comes due; earlier calls just wait for it.
    pub fn tick(&mut self) {
        doomgeneric_tick(self.state);
    }

    /// A key went down or up. `code` is the `KeyboardEvent.code`; keys the game has no use for
    /// are ignored.
    pub fn key_event(&self, pressed: bool, code: &str) {
        platform::push_key(&self.keys, pressed, code);
    }
}
