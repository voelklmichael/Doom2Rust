use crate::game_state::GameState;
use crate::i_video::I_ReadScreen;
use crate::m_random::M_Random;
use crate::v_video::Screen;
use crate::v_video::V_DrawBlock;
use crate::v_video::V_MarkRect;
use alloc::vec::Vec;

pub struct FWipeState {
    pub go: bool,
    pub wipe_scr_start: Vec<u8>,
    pub wipe_scr_end: Vec<u8>,
    pub y: Vec<i32>,
}

impl Default for FWipeState {
    fn default() -> Self {
        Self::new()
    }
}

impl FWipeState {
    pub const fn new() -> Self {
        FWipeState {
            go: false,
            wipe_scr_start: Vec::new(),
            wipe_scr_end: Vec::new(),
            y: Vec::new(),
        }
    }
}

fn wipe_shittyColMajorXform(array: &mut [u8], width: i32, height: i32) {
    let (width, height) = (width as usize, height as usize);
    let mut dest = vec![0u8; width * height * 2];
    for y in 0..height {
        for x in 0..width {
            let src = 2 * (y * width + x);
            let dst = 2 * (x * height + y);
            dest[dst..dst + 2].copy_from_slice(&array[src..src + 2]);
        }
    }
    array[..width * height * 2].copy_from_slice(&dest);
}
fn wipe_initMelt(state: &mut GameState, width: i32, height: i32) {
    let n = (width * height) as usize;
    state.i_video.I_VideoBuffer[..n].copy_from_slice(&state.f_wipe.wipe_scr_start[..n]);
    wipe_shittyColMajorXform(&mut state.f_wipe.wipe_scr_start, width / 2, height);
    wipe_shittyColMajorXform(&mut state.f_wipe.wipe_scr_end, width / 2, height);
    state.f_wipe.y = vec![0i32; width as usize];
    state.f_wipe.y[0] = -(M_Random(&mut state.m_random) % 16);
    for i in 1..width as usize {
        let r = M_Random(&mut state.m_random) % 3 - 1;
        state.f_wipe.y[i] = state.f_wipe.y[i - 1] + r;
        if state.f_wipe.y[i] > 0 {
            state.f_wipe.y[i] = 0;
        } else if state.f_wipe.y[i] == -16 {
            state.f_wipe.y[i] = -15;
        }
    }
}
/// Advances the melt by `ticks`; returns whether every column has finished.
fn wipe_doMelt(state: &mut GameState, width: i32, height: i32, mut ticks: i32) -> bool {
    let mut done = true;
    let width = (width / 2) as usize;
    let height_words = height as usize;
    let video = &mut state.i_video.I_VideoBuffer;
    let scr_start = &state.f_wipe.wipe_scr_start;
    let scr_end = &state.f_wipe.wipe_scr_end;
    let ys = &mut state.f_wipe.y;
    while ticks > 0 {
        ticks -= 1;
        for (i, y) in ys.iter_mut().enumerate().take(width) {
            if *y < 0 {
                *y += 1;
                done = false;
            } else if *y < height {
                let mut dy = if *y < 16 { *y + 1 } else { 8 };
                if *y + dy >= height {
                    dy = height - *y;
                }
                let src = i * height_words + *y as usize;
                let mut dst = *y as usize * width + i;
                for k in 0..dy as usize {
                    let so = 2 * (src + k);
                    let d = 2 * dst;
                    video[d..d + 2].copy_from_slice(&scr_end[so..so + 2]);
                    dst += width;
                }
                *y += dy;
                let src = i * height_words;
                let mut dst = *y as usize * width + i;
                for k in 0..(height - *y) as usize {
                    let so = 2 * (src + k);
                    let d = 2 * dst;
                    video[d..d + 2].copy_from_slice(&scr_start[so..so + 2]);
                    dst += width;
                }
                done = false;
            }
        }
    }
    done
}
fn wipe_exitMelt(state: &mut GameState) {
    state.f_wipe.y = Vec::new();
    state.f_wipe.wipe_scr_start = Vec::new();
    state.f_wipe.wipe_scr_end = Vec::new();
}
pub fn wipe_StartScreen(state: &mut GameState) {
    state.f_wipe.wipe_scr_start = I_ReadScreen(state);
}
pub fn wipe_EndScreen(state: &mut GameState, x: i32, y: i32, width: i32, height: i32) {
    state.f_wipe.wipe_scr_end = I_ReadScreen(state);
    let wipe_scr_start = core::mem::take(&mut state.f_wipe.wipe_scr_start);
    V_DrawBlock(state, Screen::Video, x, y, width, height, &wipe_scr_start);
    state.f_wipe.wipe_scr_start = wipe_scr_start;
}
/// Runs one step of the screen melt; returns `true` once it has finished.
pub fn wipe_ScreenWipe(state: &mut GameState, width: i32, height: i32, ticks: i32) -> bool {
    if !state.f_wipe.go {
        state.f_wipe.go = true;
        wipe_initMelt(state, width, height);
    }
    V_MarkRect(state, Screen::Video, 0, 0, width, height);
    if wipe_doMelt(state, width, height, ticks) {
        state.f_wipe.go = false;
        wipe_exitMelt(state);
    }
    !state.f_wipe.go
}
