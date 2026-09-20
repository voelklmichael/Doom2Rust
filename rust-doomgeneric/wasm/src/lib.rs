//! DOOM in the browser: the engine compiled to WebAssembly, with the shareware IWAD linked in.
//!
//! The page (`www/`) runs this in a Web Worker: it makes a [`Doom`], calls [`Doom::tick`] about
//! 35 times a second and forwards key events; frames and sound come back through the
//! `Host` object it hands to [`Doom::new`].
#![deny(unsafe_code)]

mod audio;
mod fs;
mod keys;
mod platform;

use platform::{BrowserPlatform, KeyQueue};
use rust_doomgeneric::{doomgeneric_create, doomgeneric_tick, init_game_state, GameState, Options};
use wasm_bindgen::prelude::*;

/// The shareware IWAD (E1M1-E1M9). See `assets/README.md`.
static WAD: &[u8] = include_bytes!(env!("DOOM_WAD"));
const WAD_NAME: &str = "doom1.wad";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(message: &str);
}

#[wasm_bindgen]
pub struct Doom {
    state: &'static mut GameState,
    keys: KeyQueue,
}

#[wasm_bindgen]
impl Doom {
    /// Starts the game: loads the WAD and everything else the engine needs before its first
    /// frame, which takes a moment.
    #[wasm_bindgen(constructor)]
    pub fn new(host: platform::Host) -> Doom {
        // A panic (the engine's `I_Error`) is an unreachable trap, which says nothing.
        std::panic::set_hook(Box::new(|info| error(&info.to_string())));
        let keys = KeyQueue::default();
        let state = init_game_state(
            Box::new(BrowserPlatform::new(host, KeyQueue::clone(&keys))),
            Box::new(fs::BrowserFs::new(WAD_NAME, WAD)),
        );
        let options = Options {
            iwad: Some(WAD_NAME.to_string()),
            scaling: Some(1),
            ..Options::default()
        };
        doomgeneric_create(state, options);
        Doom { state, keys }
    }

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
