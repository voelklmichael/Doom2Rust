use crate::d_main::doom_main;
use crate::doomdef::Pixel;
use crate::game_state::GameState;
use crate::options::Options;

pub const DOOMGENERIC_RESX: i32 = 640;
pub const DOOMGENERIC_RESY: i32 = 400;
/// Starts the game with the given options.
pub fn doomgeneric_create(state: &mut GameState, options: Options) {
    state.game.options = options;
    state.io.i_video.dg_screen_buffer =
        vec![0 as Pixel; (DOOMGENERIC_RESX * DOOMGENERIC_RESY) as usize];
    state.io.platform.init(DOOMGENERIC_RESX, DOOMGENERIC_RESY);
    doom_main(state);
}
