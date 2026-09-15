use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::i_video::I_ReadScreen;
use crate::m_random::M_Random;
use crate::mem_compat::memcpy;
use crate::stdint_types::byte;
use crate::stdint_types::size_t;
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

pub unsafe fn wipe_shittyColMajorXform(mut array: *mut i16, mut width: i32, mut height: i32) {
    let mut x: i32 = 0;
    let mut y_0: i32 = 0;
    let mut dest: Vec<i16> = vec![0i16; (width * height) as usize];
    y_0 = 0_i32;
    while y_0 < height {
        x = 0_i32;
        while x < width {
            dest[(x * height + y_0) as usize] = *array.offset((y_0 * width + x) as isize);
            x += 1;
        }
        y_0 += 1;
    }
    memcpy(
        array as *mut ::core::ffi::c_void,
        dest.as_ptr() as *const ::core::ffi::c_void,
        (width * height * 2_i32) as size_t,
    );
}
pub unsafe fn wipe_initColorXForm(
    state: &mut GameState,
    mut width: i32,
    mut height: i32,
    _ticks: i32,
) -> i32 {
    memcpy(
        state.i_video.I_VideoBuffer.as_mut_ptr() as *mut ::core::ffi::c_void,
        state.f_wipe.wipe_scr_start.as_ptr() as *const ::core::ffi::c_void,
        (width * height) as size_t,
    );
    0_i32
}
pub unsafe fn wipe_doColorXForm(
    state: &mut GameState,
    mut width: i32,
    mut height: i32,
    mut ticks: i32,
) -> i32 {
    let mut changed: bool;
    let mut w: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut e: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut newval: i32 = 0;
    changed = false;
    let wipe_scr = state.i_video.I_VideoBuffer.as_mut_ptr();
    w = wipe_scr;
    e = state.f_wipe.wipe_scr_end.as_mut_ptr();
    while w != wipe_scr.offset((width * height) as isize) {
        if *w as i32 != *e as i32 {
            if *w as i32 > *e as i32 {
                newval = *w as i32 - ticks;
                if newval < *e as i32 {
                    *w = *e;
                } else {
                    *w = newval as byte;
                }
                changed = true;
            } else if (*w as i32) < *e as i32 {
                newval = *w as i32 + ticks;
                if newval > *e as i32 {
                    *w = *e;
                } else {
                    *w = newval as byte;
                }
                changed = true;
            }
        }
        w = w.offset(1);
        e = e.offset(1);
    }
    (!changed) as i32
}
pub fn wipe_exitColorXForm(_state: &mut GameState, _width: i32, _height: i32, _ticks: i32) -> i32 {
    0_i32
}
pub unsafe fn wipe_initMelt(
    state: &mut GameState,
    mut width: i32,
    mut height: i32,
    _ticks: i32,
) -> i32 {
    let mut i: i32 = 0;
    let mut r: i32 = 0;
    memcpy(
        state.i_video.I_VideoBuffer.as_mut_ptr() as *mut ::core::ffi::c_void,
        state.f_wipe.wipe_scr_start.as_ptr() as *const ::core::ffi::c_void,
        (width * height) as size_t,
    );
    let wipe_scr_start = state.f_wipe.wipe_scr_start.as_mut_ptr() as *mut i16;
    wipe_shittyColMajorXform(wipe_scr_start, width / 2_i32, height);
    let wipe_scr_end = state.f_wipe.wipe_scr_end.as_mut_ptr() as *mut i16;
    wipe_shittyColMajorXform(wipe_scr_end, width / 2_i32, height);
    state.f_wipe.y = vec![0i32; width as usize];
    state.f_wipe.y[0] = -(M_Random(&mut state.m_random) % 16_i32);
    i = 1_i32;
    while i < width {
        r = M_Random(&mut state.m_random) % 3_i32 - 1_i32;
        state.f_wipe.y[i as usize] = state.f_wipe.y[(i - 1_i32) as usize] + r;
        if state.f_wipe.y[i as usize] > 0_i32 {
            state.f_wipe.y[i as usize] = 0_i32;
        } else if state.f_wipe.y[i as usize] == -16_i32 {
            state.f_wipe.y[i as usize] = -15_i32;
        }
        i += 1;
    }
    0_i32
}
pub unsafe fn wipe_doMelt(
    state: &mut GameState,
    mut width: i32,
    mut height: i32,
    mut ticks: i32,
) -> i32 {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut dy: i32 = 0;
    let mut idx: i32 = 0;
    let mut s: *mut i16 = ::core::ptr::null_mut::<i16>();
    let mut d: *mut i16 = ::core::ptr::null_mut::<i16>();
    let mut done: bool = true;
    width /= 2_i32;
    loop {
        let fresh0 = ticks;
        ticks -= 1;
        if fresh0 == 0 {
            break;
        }
        i = 0_i32;
        while i < width {
            if state.f_wipe.y[i as usize] < 0_i32 {
                state.f_wipe.y[i as usize] += 1;
                done = false;
            } else if state.f_wipe.y[i as usize] < height {
                dy = if state.f_wipe.y[i as usize] < 16_i32 {
                    state.f_wipe.y[i as usize] + 1_i32
                } else {
                    8_i32
                };
                if state.f_wipe.y[i as usize] + dy >= height {
                    dy = height - state.f_wipe.y[i as usize];
                }
                s = (state.f_wipe.wipe_scr_end.as_mut_ptr() as *mut i16)
                    .offset((i * height + state.f_wipe.y[i as usize]) as isize);
                d = (state.i_video.I_VideoBuffer.as_mut_ptr() as *mut i16)
                    .offset((state.f_wipe.y[i as usize] * width + i) as isize);
                idx = 0_i32;
                j = dy;
                while j != 0 {
                    let fresh2 = s;
                    s = s.offset(1);
                    *d.offset(idx as isize) = *fresh2;
                    idx += width;
                    j -= 1;
                }
                state.f_wipe.y[i as usize] += dy;
                s = (state.f_wipe.wipe_scr_start.as_mut_ptr() as *mut i16)
                    .offset((i * height) as isize);
                d = (state.i_video.I_VideoBuffer.as_mut_ptr() as *mut i16)
                    .offset((state.f_wipe.y[i as usize] * width + i) as isize);
                idx = 0_i32;
                j = height - state.f_wipe.y[i as usize];
                while j != 0 {
                    let fresh3 = s;
                    s = s.offset(1);
                    *d.offset(idx as isize) = *fresh3;
                    idx += width;
                    j -= 1;
                }
                done = false;
            }
            i += 1;
        }
    }
    done as i32
}
pub fn wipe_exitMelt(state: &mut GameState, _width: i32, _height: i32, _ticks: i32) -> i32 {
    state.f_wipe.y = Vec::new();
    state.f_wipe.wipe_scr_start = Vec::new();
    state.f_wipe.wipe_scr_end = Vec::new();
    0_i32
}
pub fn wipe_StartScreen(state: &mut GameState) -> i32 {
    state.f_wipe.wipe_scr_start = vec![0u8; (SCREENWIDTH * SCREENHEIGHT) as usize];
    let wipe_scr_start = state.f_wipe.wipe_scr_start.as_mut_ptr();
    unsafe { I_ReadScreen(state, wipe_scr_start) };
    0_i32
}
pub fn wipe_EndScreen(
    state: &mut GameState,
    mut x: i32,
    mut y_0: i32,
    mut width: i32,
    mut height: i32,
) -> i32 {
    state.f_wipe.wipe_scr_end = vec![0u8; (SCREENWIDTH * SCREENHEIGHT) as usize];
    let wipe_scr_end = state.f_wipe.wipe_scr_end.as_mut_ptr();
    unsafe { I_ReadScreen(state, wipe_scr_end) };
    let wipe_scr_start = state.f_wipe.wipe_scr_start.as_mut_ptr();
    unsafe { V_DrawBlock(state, x, y_0, width, height, wipe_scr_start) };
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
    let wipes: [Option<unsafe fn(&mut GameState, i32, i32, i32) -> i32>; 6] = [
        Some(wipe_initColorXForm as unsafe fn(&mut GameState, i32, i32, i32) -> i32),
        Some(wipe_doColorXForm as unsafe fn(&mut GameState, i32, i32, i32) -> i32),
        Some(wipe_exitColorXForm as unsafe fn(&mut GameState, i32, i32, i32) -> i32),
        Some(wipe_initMelt as unsafe fn(&mut GameState, i32, i32, i32) -> i32),
        Some(wipe_doMelt as unsafe fn(&mut GameState, i32, i32, i32) -> i32),
        Some(wipe_exitMelt as unsafe fn(&mut GameState, i32, i32, i32) -> i32),
    ];
    if !state.f_wipe.go {
        state.f_wipe.go = true;
        let init_fn = wipes[(wipeno * 3_i32) as usize].expect("non-null function pointer");
        unsafe { init_fn(state, width, height, ticks) };
    }
    V_MarkRect(state, 0_i32, 0_i32, width, height);
    let do_fn = wipes[(wipeno * 3_i32 + 1_i32) as usize].expect("non-null function pointer");
    rc = unsafe { do_fn(state, width, height, ticks) };
    if rc != 0 {
        state.f_wipe.go = false;
        let exit_fn = wipes[(wipeno * 3_i32 + 2_i32) as usize].expect("non-null function pointer");
        unsafe { exit_fn(state, width, height, ticks) };
    }
    (!state.f_wipe.go) as i32
}
