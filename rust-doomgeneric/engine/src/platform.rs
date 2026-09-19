use crate::doomdef::Pixel;

pub trait DoomPlatform {
    /// Called once before the game loop starts. Every frame later passed to
    /// [`draw_frame`](Self::draw_frame) is `resx * resy` pixels.
    fn init(&mut self, resx: i32, resy: i32);
    /// Called once per frame with the finished frame, row by row. The slice is
    /// only valid for the duration of the call.
    fn draw_frame(&mut self, frame: &[Pixel]);
    /// Optional fast path, offered before [`draw_frame`](Self::draw_frame) each frame.
    ///
    /// `indices` is the engine's own 8-bit screen, `SCREENWIDTH * SCREENHEIGHT`
    /// (320 x 200) palette indices in row-major order, with no scaling applied
    /// (`-scaling` and `-gfxmode` do not affect it). `palette` maps an index to a
    /// `0x00RRGGBB` pixel with gamma already applied. Both are only valid for
    /// the duration of the call.
    ///
    /// Return `true` if the frame was drawn: the engine then skips building the
    /// scaled 32-bit frame that `draw_frame` needs, which is expensive on slow
    /// hardware. The default declines, so `draw_frame` is used.
    fn draw_indexed_frame(&mut self, _indices: &[u8], _palette: &[Pixel; 256]) -> bool {
        false
    }
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
