
use crate::game_state::GameState;
use crate::v_video::Screen;
use crate::i_video::I_ReadScreen;
use crate::m_random::M_Random;
use crate::stdint_types::byte;
use crate::v_video::V_DrawBlock;
use crate::v_video::V_MarkRect;

pub struct FWipeState {
    pub go: bool,
    pub wipe_scr_start: Vec<byte>,
    pub wipe_scr_end: Vec<byte>,
    pub y: Vec<i32>,
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

fn wipe_shittyColMajorXform(array: &mut [byte], width: i32, height: i32) {
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
fn wipe_initColorXForm(state: &mut GameState, width: i32, height: i32, _ticks: i32) -> i32 {
    let n = (width * height) as usize;
    state.i_video.I_VideoBuffer[..n].copy_from_slice(&state.f_wipe.wipe_scr_start[..n]);
    0_i32
}
fn wipe_doColorXForm(state: &mut GameState, width: i32, height: i32, ticks: i32) -> i32 {
    let mut changed = false;
    let n = (width * height) as usize;
    let end = &state.f_wipe.wipe_scr_end[..n];
    for (w, &e) in state.i_video.I_VideoBuffer[..n].iter_mut().zip(end) {
        if *w > e {
            let newval = *w as i32 - ticks;
            *w = if newval < e as i32 { e } else { newval as byte };
            changed = true;
        } else if *w < e {
            let newval = *w as i32 + ticks;
            *w = if newval > e as i32 { e } else { newval as byte };
            changed = true;
        }
    }
    (!changed) as i32
}
fn wipe_exitColorXForm(_state: &mut GameState, _width: i32, _height: i32, _ticks: i32) -> i32 {
    0_i32
}
fn wipe_initMelt(state: &mut GameState, width: i32, height: i32, _ticks: i32) -> i32 {
    let n = (width * height) as usize;
    state.i_video.I_VideoBuffer[..n].copy_from_slice(&state.f_wipe.wipe_scr_start[..n]);
    wipe_shittyColMajorXform(&mut state.f_wipe.wipe_scr_start, width / 2_i32, height);
    wipe_shittyColMajorXform(&mut state.f_wipe.wipe_scr_end, width / 2_i32, height);
    state.f_wipe.y = vec![0i32; width as usize];
    state.f_wipe.y[0] = -(M_Random(&mut state.m_random) % 16_i32);
    for i in 1..width as usize {
        let r = M_Random(&mut state.m_random) % 3_i32 - 1_i32;
        state.f_wipe.y[i] = state.f_wipe.y[i - 1] + r;
        if state.f_wipe.y[i] > 0_i32 {
            state.f_wipe.y[i] = 0_i32;
        } else if state.f_wipe.y[i] == -16_i32 {
            state.f_wipe.y[i] = -15_i32;
        }
    }
    0_i32
}
fn wipe_doMelt(state: &mut GameState, width: i32, height: i32, mut ticks: i32) -> i32 {
    let mut done = true;
    let width = (width / 2_i32) as usize;
    let height_words = height as usize;
    let video = &mut state.i_video.I_VideoBuffer;
    let scr_start = &state.f_wipe.wipe_scr_start;
    let scr_end = &state.f_wipe.wipe_scr_end;
    let ys = &mut state.f_wipe.y;
    while ticks > 0 {
        ticks -= 1;
        for i in 0..width {
            if ys[i] < 0_i32 {
                ys[i] += 1;
                done = false;
            } else if ys[i] < height {
                let mut dy = if ys[i] < 16_i32 { ys[i] + 1_i32 } else { 8_i32 };
                if ys[i] + dy >= height {
                    dy = height - ys[i];
                }
                let src = i * height_words + ys[i] as usize;
                let mut dst = ys[i] as usize * width + i;
                for k in 0..dy as usize {
                    let so = 2 * (src + k);
                    let d = 2 * dst;
                    video[d..d + 2].copy_from_slice(&scr_end[so..so + 2]);
                    dst += width;
                }
                ys[i] += dy;
                let src = i * height_words;
                let mut dst = ys[i] as usize * width + i;
                for k in 0..(height - ys[i]) as usize {
                    let so = 2 * (src + k);
                    let d = 2 * dst;
                    video[d..d + 2].copy_from_slice(&scr_start[so..so + 2]);
                    dst += width;
                }
                done = false;
            }
        }
    }
    done as i32
}
fn wipe_exitMelt(state: &mut GameState, _width: i32, _height: i32, _ticks: i32) -> i32 {
    state.f_wipe.y = Vec::new();
    state.f_wipe.wipe_scr_start = Vec::new();
    state.f_wipe.wipe_scr_end = Vec::new();
    0_i32
}
pub fn wipe_StartScreen(state: &mut GameState) -> i32 {
    state.f_wipe.wipe_scr_start = I_ReadScreen(state);
    0_i32
}
pub fn wipe_EndScreen(
    state: &mut GameState,
    mut x: i32,
    mut y_0: i32,
    mut width: i32,
    mut height: i32,
) -> i32 {
    state.f_wipe.wipe_scr_end = I_ReadScreen(state);
    let wipe_scr_start = std::mem::take(&mut state.f_wipe.wipe_scr_start);
    V_DrawBlock(state, Screen::Video, x, y_0, width, height, &wipe_scr_start);
    state.f_wipe.wipe_scr_start = wipe_scr_start;
    0_i32
}
pub fn wipe_ScreenWipe(
    state: &mut GameState,
    mut wipeno: i32,
    mut width: i32,
    mut height: i32,
    mut ticks: i32,
) -> i32 {
    let mut rc: i32 = 0;
    let wipes: [fn(&mut GameState, i32, i32, i32) -> i32; 6] = [
        wipe_initColorXForm,
        wipe_doColorXForm,
        wipe_exitColorXForm,
        wipe_initMelt,
        wipe_doMelt,
        wipe_exitMelt,
    ];
    if !state.f_wipe.go {
        state.f_wipe.go = true;
        let init_fn = wipes[(wipeno * 3_i32) as usize];
        init_fn(state, width, height, ticks);
    }
    let dest_screen = Screen::Video;
    V_MarkRect(state, dest_screen, 0_i32, 0_i32, width, height);
    let do_fn = wipes[(wipeno * 3_i32 + 1_i32) as usize];
    rc = do_fn(state, width, height, ticks);
    if rc != 0 {
        state.f_wipe.go = false;
        let exit_fn = wipes[(wipeno * 3_i32 + 2_i32) as usize];
        exit_fn(state, width, height, ticks);
    }
    (!state.f_wipe.go) as i32
}
