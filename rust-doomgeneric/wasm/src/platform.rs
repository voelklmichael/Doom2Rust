//! `DoomPlatform` for a web page. The game runs in a Web Worker (see `www/worker.js`), so that the
//! page never waits for it: the engine draws the screen wipe in a loop inside a single tick, and
//! the frames of that loop have to reach the canvas while it is still running. The platform
//! therefore does not queue anything for the page to fetch; it calls the page (the [`Host`])
//! whenever it has a frame or sound.

use crate::audio::AudioClock;
use crate::fs::Persist;
use crate::keys::doom_key;
use rust_doomgeneric::DoomPlatform;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// The engine's own screen, before any scaling.
const WIDTH: usize = 320;
const HEIGHT: usize = 200;

#[wasm_bindgen]
extern "C" {
    /// The page's side of the platform: a plain object with these methods, passed to
    /// `Doom::new`.
    pub type Host;

    /// A finished frame: `width * height` pixels of RGBA. The bytes are only valid during the
    /// call.
    #[wasm_bindgen(method)]
    fn present(this: &Host, rgba: &[u8], width: u32, height: u32);
    /// Mixed sound: interleaved stereo 16-bit samples at `rate` Hz. Only valid during the call.
    #[wasm_bindgen(method)]
    fn audio(this: &Host, samples: &[i16], rate: u32);
    /// Milliseconds on a clock that only goes forward.
    #[wasm_bindgen(method)]
    fn now(this: &Host) -> f64;
    /// Console output of the game; `error` for diagnostics.
    #[wasm_bindgen(method)]
    fn log(this: &Host, message: &str, error: bool);
    /// The game has finished.
    #[wasm_bindgen(method)]
    fn quit(this: &Host);
    /// The game wrote a file (a savegame, the config): keep it for the next visit. The bytes are
    /// only valid during the call.
    #[wasm_bindgen(method)]
    fn store(this: &Host, path: &str, data: &[u8]);
    /// The game deleted a file it wrote earlier.
    #[wasm_bindgen(method)]
    fn remove(this: &Host, path: &str);
}

impl Persist for Rc<Host> {
    fn store(&self, path: &str, data: &[u8]) {
        Host::store(self, path, data);
    }
    fn remove(&self, path: &str) {
        Host::remove(self, path);
    }
}

/// Key events from the page, in the order they happened: `(pressed, engine key code)`.
pub type KeyQueue = Rc<RefCell<VecDeque<(bool, u8)>>>;

pub fn push_key(keys: &KeyQueue, pressed: bool, code: &str) {
    if let Some(key) = doom_key(code) {
        keys.borrow_mut().push_back((pressed, key));
    }
}

pub struct BrowserPlatform {
    host: Rc<Host>,
    keys: KeyQueue,
    rgba: Vec<u8>,
    audio: Option<AudioClock>,
}

impl BrowserPlatform {
    pub fn new(host: Rc<Host>, keys: KeyQueue) -> Self {
        Self {
            host,
            keys,
            rgba: vec![0; WIDTH * HEIGHT * 4],
            audio: None,
        }
    }
}

/// Fills `rgba` with the pixels `indices` stands for. A palette entry is `0x00RRGGBB`.
fn indexed_to_rgba(indices: &[u8], palette: &[u32; 256], rgba: &mut [u8]) {
    let (pixels, _) = rgba.as_chunks_mut::<4>();
    for (&index, out) in indices.iter().zip(pixels) {
        let [_, r, g, b] = palette[usize::from(index)].to_be_bytes();
        *out = [r, g, b, 0xff];
    }
}

impl DoomPlatform for BrowserPlatform {
    fn init(&mut self, _resx: i32, _resy: i32) {}

    /// Never called: [`draw_indexed_frame`](Self::draw_indexed_frame) takes every frame.
    fn draw_frame(&mut self, _frame: &[u32]) {}

    fn draw_indexed_frame(&mut self, indices: &[u8], palette: &[u32; 256]) -> bool {
        indexed_to_rgba(indices, palette, &mut self.rgba);
        self.host.present(&self.rgba, WIDTH as u32, HEIGHT as u32);
        true
    }

    fn audio_open(&mut self, preferred_rate: u32) -> Option<u32> {
        // Web Audio resamples whatever rate a buffer has, so the engine's own rate is fine.
        self.audio = Some(AudioClock::new(preferred_rate));
        Some(preferred_rate)
    }

    fn audio_frames_wanted(&mut self) -> usize {
        let now = self.host.now();
        self.audio
            .as_mut()
            .map_or(0, |clock| clock.frames_wanted(now))
    }

    fn audio_write(&mut self, samples: &[i16]) {
        if let Some(clock) = self.audio.as_mut() {
            clock.mixed(samples.len() / 2);
            self.host.audio(samples, clock.rate());
        }
    }

    /// The engine waits for the next tic by asking the time again in a loop, so there is nothing to
    /// sleep for: the page paces the ticks (see `www/worker.js`).
    fn sleep_ms(&mut self, _ms: u32) {}

    fn get_ticks_ms(&mut self) -> u32 {
        // The engine only looks at differences, so the clock may wrap.
        // The clock is milliseconds since the page loaded: not negative.
        #[allow(clippy::cast_sign_loss)]
        let ms = self.host.now() as u64;
        ms as u32
    }

    fn get_key(&mut self) -> Option<(bool, u8)> {
        self.keys.borrow_mut().pop_front()
    }

    fn set_window_title(&mut self, _title: &str) {}

    fn print(&mut self, message: &str) {
        self.host.log(message, false);
    }

    fn eprint(&mut self, message: &str) {
        self.host.log(message, true);
    }

    fn quit(&mut self) -> ! {
        self.host.quit();
        // Unwinds into the page's tick call, which has stopped ticking by now.
        wasm_bindgen::throw_str("DOOM has quit")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_entries_become_rgba() {
        let mut palette = [0; 256];
        palette[1] = 0x00ff_8040;
        palette[2] = 0x0001_0203;
        let mut rgba = [0; 12];
        indexed_to_rgba(&[1, 2, 0], &palette, &mut rgba);
        assert_eq!(rgba, [0xff, 0x80, 0x40, 0xff, 1, 2, 3, 0xff, 0, 0, 0, 0xff]);
    }

    #[test]
    fn keys_are_queued_in_order_and_unknown_ones_dropped() {
        let keys = KeyQueue::default();
        push_key(&keys, true, "ControlLeft");
        push_key(&keys, true, "MetaLeft");
        push_key(&keys, false, "ControlLeft");
        let queued: Vec<_> = keys.borrow().iter().copied().collect();
        assert_eq!(queued, [(true, 0xa3), (false, 0xa3)]);
    }
}
