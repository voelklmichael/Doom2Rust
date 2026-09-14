use crate::d_main::D_DoomMain;
use crate::doomdef::pixel_t;
use crate::game_state::GameState;
use crate::m_argv::M_FindResponseFile;

pub const DOOMGENERIC_RESX: i32 = 640;
pub const DOOMGENERIC_RESY: i32 = 400;
pub unsafe fn doomgeneric_Create(state: &mut GameState, args: Vec<String>) {
    state.m_argv.myargv = args
        .into_iter()
        .map(|arg| ::std::ffi::CString::new(arg).expect("argument contains a nul byte"))
        .collect();
    M_FindResponseFile(state);
    state.i_video.dg_screen_buffer = vec![0 as pixel_t; (DOOMGENERIC_RESX * DOOMGENERIC_RESY) as usize];
    state.platform.init(
        state.i_video.dg_screen_buffer.as_mut_ptr(),
        DOOMGENERIC_RESX,
        DOOMGENERIC_RESY,
    );
    D_DoomMain(state);
}
