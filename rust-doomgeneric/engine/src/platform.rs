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
    /// Writes `message` to the console (stdout), exactly as given: the engine
    /// adds its own newlines.
    fn print(&mut self, message: &str);
    /// Like `print`, but for diagnostics (stderr).
    fn eprint(&mut self, message: &str);
    /// Terminates the program successfully; called when the engine has
    /// finished running its exit handlers.
    fn quit(&mut self) -> !;
}
