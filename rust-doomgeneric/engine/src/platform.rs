use crate::doomdef::pixel_t;

pub trait DoomPlatform {
    /// Called once before the game loop starts. `screen_buffer` points to
    /// `resx * resy` `pixel_t`s, owned by the engine for the process lifetime.
    fn init(&mut self, screen_buffer: *mut pixel_t, resx: i32, resy: i32);
    /// Called once per frame, after the engine has rendered into `screen_buffer`.
    fn draw_frame(&mut self);
    fn sleep_ms(&mut self, ms: u32);
    fn get_ticks_ms(&mut self) -> u32;
    /// Pops one queued key event, if any: `(pressed, keycode)`.
    fn get_key(&mut self) -> Option<(bool, u8)>;
    fn set_window_title(&mut self, title: &str);
}
