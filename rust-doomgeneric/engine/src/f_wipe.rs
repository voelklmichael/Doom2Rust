use crate::src::doomdef::NULL;
use crate::src::doomdef::SCREENHEIGHT;
use crate::src::doomdef::SCREENWIDTH;
use crate::src::game_state::GameState;
use crate::src::i_video::I_ReadScreen;
use crate::src::m_random::M_Random;
use crate::src::stdint_types::byte;
use crate::src::stdint_types::size_t;
use crate::src::v_video::V_DrawBlock;
use crate::src::v_video::V_MarkRect;
use crate::src::z_zone::Z_Free;
use crate::src::z_zone::Z_Malloc;
use crate::src::z_zone::PU_STATIC;
use crate::src::mem_compat::memcpy;

pub struct FWipeState {
    pub go: bool,
    pub wipe_scr_start: *mut byte,
    pub wipe_scr_end: *mut byte,
    pub wipe_scr: *mut byte,
    pub y: *mut i32,
}

impl FWipeState {
    pub const fn new() -> Self {
        FWipeState {
            go: false,
            wipe_scr_start: ::core::ptr::null::<byte>() as *mut byte,
            wipe_scr_end: ::core::ptr::null::<byte>() as *mut byte,
            wipe_scr: ::core::ptr::null::<byte>() as *mut byte,
            y: ::core::ptr::null::<i32>() as *mut i32,
        }
    }
}

pub unsafe fn wipe_shittyColMajorXform(
    state: &mut GameState,
    mut array: *mut i16,
    mut width: i32,
    mut height: i32,
) {
    let mut x: i32 = 0;
    let mut y_0: i32 = 0;
    let mut dest: *mut i16 = ::core::ptr::null_mut::<i16>();
    dest = Z_Malloc(
        &mut state.z_zone,
        width * height * 2 as i32,
        PU_STATIC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut i16;
    y_0 = 0 as i32;
    while y_0 < height {
        x = 0 as i32;
        while x < width {
            *dest.offset((x * height + y_0) as isize) = *array.offset((y_0 * width + x) as isize);
            x += 1;
        }
        y_0 += 1;
    }
    memcpy(
        array as *mut ::core::ffi::c_void,
        dest as *const ::core::ffi::c_void,
        (width * height * 2 as i32) as size_t,
    );
    Z_Free(&mut state.z_zone, dest as *mut ::core::ffi::c_void);
}
pub unsafe fn wipe_initColorXForm(
    state: &mut GameState,
    mut width: i32,
    mut height: i32,
    _ticks: i32,
) -> i32 {
    memcpy(
        state.f_wipe.wipe_scr as *mut ::core::ffi::c_void,
        state.f_wipe.wipe_scr_start as *const ::core::ffi::c_void,
        (width * height) as size_t,
    );
    return 0 as i32;
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
    w = state.f_wipe.wipe_scr;
    e = state.f_wipe.wipe_scr_end;
    while w != state.f_wipe.wipe_scr.offset((width * height) as isize) {
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
    return (!changed) as i32;
}
pub fn wipe_exitColorXForm(
    _state: &mut GameState,
    _width: i32,
    _height: i32,
    _ticks: i32,
) -> i32 {
    return 0 as i32;
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
        state.f_wipe.wipe_scr as *mut ::core::ffi::c_void,
        state.f_wipe.wipe_scr_start as *const ::core::ffi::c_void,
        (width * height) as size_t,
    );
    let wipe_scr_start = state.f_wipe.wipe_scr_start as *mut i16;
    wipe_shittyColMajorXform(state, wipe_scr_start, width / 2 as i32, height);
    let wipe_scr_end = state.f_wipe.wipe_scr_end as *mut i16;
    wipe_shittyColMajorXform(state, wipe_scr_end, width / 2 as i32, height);
    state.f_wipe.y = Z_Malloc(
        &mut state.z_zone,
        (width as usize).wrapping_mul(::core::mem::size_of::<i32>() as usize) as i32,
        PU_STATIC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut i32;
    *state.f_wipe.y.offset(0 as i32 as isize) = -(M_Random(&mut state.m_random) % 16 as i32);
    i = 1 as i32;
    while i < width {
        r = M_Random(&mut state.m_random) % 3 as i32 - 1 as i32;
        *state.f_wipe.y.offset(i as isize) = *state.f_wipe.y.offset((i - 1 as i32) as isize) + r;
        if *state.f_wipe.y.offset(i as isize) > 0 as i32 {
            *state.f_wipe.y.offset(i as isize) = 0 as i32;
        } else if *state.f_wipe.y.offset(i as isize) == -(16 as i32) {
            *state.f_wipe.y.offset(i as isize) = -(15 as i32);
        }
        i += 1;
    }
    return 0 as i32;
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
    width /= 2 as i32;
    loop {
        let fresh0 = ticks;
        ticks = ticks - 1;
        if !(fresh0 != 0) {
            break;
        }
        i = 0 as i32;
        while i < width {
            if *state.f_wipe.y.offset(i as isize) < 0 as i32 {
                let ref mut fresh1 = *state.f_wipe.y.offset(i as isize);
                *fresh1 += 1;
                done = false;
            } else if *state.f_wipe.y.offset(i as isize) < height {
                dy = if *state.f_wipe.y.offset(i as isize) < 16 as i32 {
                    *state.f_wipe.y.offset(i as isize) + 1 as i32
                } else {
                    8 as i32
                };
                if *state.f_wipe.y.offset(i as isize) + dy >= height {
                    dy = height - *state.f_wipe.y.offset(i as isize);
                }
                s = (state.f_wipe.wipe_scr_end as *mut i16)
                    .offset((i * height + *state.f_wipe.y.offset(i as isize)) as isize)
                    as *mut i16;
                d = (state.f_wipe.wipe_scr as *mut i16)
                    .offset((*state.f_wipe.y.offset(i as isize) * width + i) as isize)
                    as *mut i16;
                idx = 0 as i32;
                j = dy;
                while j != 0 {
                    let fresh2 = s;
                    s = s.offset(1);
                    *d.offset(idx as isize) = *fresh2;
                    idx += width;
                    j -= 1;
                }
                *state.f_wipe.y.offset(i as isize) += dy;
                s = (state.f_wipe.wipe_scr_start as *mut i16).offset((i * height) as isize)
                    as *mut i16;
                d = (state.f_wipe.wipe_scr as *mut i16)
                    .offset((*state.f_wipe.y.offset(i as isize) * width + i) as isize)
                    as *mut i16;
                idx = 0 as i32;
                j = height - *state.f_wipe.y.offset(i as isize);
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
    return done as i32;
}
pub unsafe fn wipe_exitMelt(
    state: &mut GameState,
    _width: i32,
    _height: i32,
    _ticks: i32,
) -> i32 {
    Z_Free(
        &mut state.z_zone,
        state.f_wipe.y as *mut ::core::ffi::c_void,
    );
    Z_Free(
        &mut state.z_zone,
        state.f_wipe.wipe_scr_start as *mut ::core::ffi::c_void,
    );
    Z_Free(
        &mut state.z_zone,
        state.f_wipe.wipe_scr_end as *mut ::core::ffi::c_void,
    );
    return 0 as i32;
}
pub unsafe fn wipe_StartScreen(state: &mut GameState) -> i32 {
    state.f_wipe.wipe_scr_start = Z_Malloc(
        &mut state.z_zone,
        SCREENWIDTH * SCREENHEIGHT,
        PU_STATIC as i32,
        NULL,
    ) as *mut byte;
    I_ReadScreen(state, state.f_wipe.wipe_scr_start);
    return 0 as i32;
}
pub unsafe fn wipe_EndScreen(
    state: &mut GameState,
    mut x: i32,
    mut y_0: i32,
    mut width: i32,
    mut height: i32,
) -> i32 {
    state.f_wipe.wipe_scr_end = Z_Malloc(
        &mut state.z_zone,
        SCREENWIDTH * SCREENHEIGHT,
        PU_STATIC as i32,
        NULL,
    ) as *mut byte;
    I_ReadScreen(state, state.f_wipe.wipe_scr_end);
    V_DrawBlock(state,
        x,
        y_0,
        width,
        height,
        state.f_wipe.wipe_scr_start,
    );
    return 0 as i32;
}
pub unsafe fn wipe_ScreenWipe(
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
        state.f_wipe.wipe_scr = state.i_video.I_VideoBuffer;
        let init_fn = wipes[(wipeno * 3 as i32) as usize].expect("non-null function pointer");
        init_fn(state, width, height, ticks);
    }
    V_MarkRect(state, 0 as i32, 0 as i32, width, height);
    let do_fn = wipes[(wipeno * 3 as i32 + 1 as i32) as usize].expect("non-null function pointer");
    rc = do_fn(state, width, height, ticks);
    if rc != 0 {
        state.f_wipe.go = false;
        let exit_fn =
            wipes[(wipeno * 3 as i32 + 2 as i32) as usize].expect("non-null function pointer");
        exit_fn(state, width, height, ticks);
    }
    return (!state.f_wipe.go) as i32;
}
