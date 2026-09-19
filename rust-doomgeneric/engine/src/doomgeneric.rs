use crate::d_main::doom_main;
use crate::doomdef::Pixel;
use crate::game_state::GameState;
use alloc::string::String;
use alloc::vec::Vec;

pub const DOOMGENERIC_RESX: i32 = 640;
pub const DOOMGENERIC_RESY: i32 = 400;
pub fn doomgeneric_create(state: &mut GameState, args: Vec<String>) {
    state.m_argv.myargv = args;
    state.i_video.dg_screen_buffer =
        vec![0 as Pixel; (DOOMGENERIC_RESX * DOOMGENERIC_RESY) as usize];
    state.platform.init(DOOMGENERIC_RESX, DOOMGENERIC_RESY);
    doom_main(state);
}
