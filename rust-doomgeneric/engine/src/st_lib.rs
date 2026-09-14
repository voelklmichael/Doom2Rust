use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::st_stuff::ST_Y;
use crate::v_video::V_CachePatchNum;
use crate::v_video::V_CopyRect;
use crate::v_video::V_DrawPatch;
use crate::w_wad::{W_CacheLumpNum, W_GetNumForName};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct st_number_t {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub oldnum: i32,
    pub num: *mut i32,
    pub on: *mut bool,
    pub p: *mut i32,
    pub data: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct st_percent_t {
    pub n: st_number_t,
    pub p: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct st_multicon_t {
    pub x: i32,
    pub y: i32,
    pub oldinum: i32,
    pub inum: *mut i32,
    pub on: *mut bool,
    pub p: *mut i32,
    pub data: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct st_binicon_t {
    pub x: i32,
    pub y: i32,
    pub oldval: bool,
    pub val: *mut bool,
    pub on: *mut bool,
    pub p: i32,
    pub data: i32,
}
pub struct StLibState {
    sttminus: i32,
}

impl StLibState {
    pub const fn new() -> Self {
        StLibState { sttminus: -1 }
    }
}

pub unsafe fn STlib_init(state: &mut GameState) {
    let lumpnum = W_GetNumForName(&mut state.w_wad, "STTMINUS");
    W_CacheLumpNum(state, lumpnum);
    state.st_lib.sttminus = lumpnum;
}
pub unsafe fn STlib_initNum(
    mut n: *mut st_number_t,
    mut x: i32,
    mut y: i32,
    mut pl: *mut i32,
    mut num: *mut i32,
    mut on: *mut bool,
    mut width: i32,
) {
    (*n).x = x;
    (*n).y = y;
    (*n).oldnum = 0 as i32;
    (*n).width = width;
    (*n).num = num;
    (*n).on = on;
    (*n).p = pl;
}
pub unsafe fn STlib_drawNum(state: &mut GameState, mut n: *mut st_number_t) {
    let mut numdigits: i32 = (*n).width;
    let mut num: i32 = *(*n).num;
    let zero_patch = V_CachePatchNum(state, *(*n).p.offset(0 as i32 as isize));
    let mut w: i32 = (*zero_patch).width as i32;
    let mut h: i32 = (*zero_patch).height as i32;
    let mut x: i32 = (*n).x;
    let mut neg: i32 = 0;
    (*n).oldnum = *(*n).num;
    neg = (num < 0 as i32) as i32;
    if neg != 0 {
        if numdigits == 2 as i32 && num < -(9 as i32) {
            num = -(9 as i32);
        } else if numdigits == 3 as i32 && num < -(99 as i32) {
            num = -(99 as i32);
        }
        num = -num;
    }
    x = (*n).x - numdigits * w;
    if (*n).y - ST_Y < 0 as i32 {
        I_Error("drawNum: n->y - ST_Y < 0");
    }
    let st_backing_screen = state.st_stuff.st_backing_screen.as_mut_ptr();
    V_CopyRect(state,
        x,
        (*n).y - ST_Y,
        st_backing_screen,
        w * numdigits,
        h,
        x,
        (*n).y,
    );
    if num == 1994 as i32 {
        return;
    }
    x = (*n).x;
    if num == 0 {
        V_DrawPatch(state,
            x - w,
            (*n).y,
            zero_patch,
        );
    }
    while num != 0 && {
        let fresh0 = numdigits;
        numdigits = numdigits - 1;
        fresh0 != 0
    } {
        x -= w;
        let digit_patch = V_CachePatchNum(state, *(*n).p.offset((num % 10 as i32) as isize));
        V_DrawPatch(state,
            x,
            (*n).y,
            digit_patch,
        );
        num /= 10 as i32;
    }
    if neg != 0 {
        let patch = V_CachePatchNum(state, state.st_lib.sttminus);
        V_DrawPatch(state,
            x - 8 as i32,
            (*n).y,
            patch,
        );
    }
}
pub unsafe fn STlib_updateNum(state: &mut GameState, mut n: *mut st_number_t) {
    if *(*n).on {
        STlib_drawNum(state, n);
    }
}
pub unsafe fn STlib_initPercent(
    mut p: *mut st_percent_t,
    mut x: i32,
    mut y: i32,
    mut pl: *mut i32,
    mut num: *mut i32,
    mut on: *mut bool,
    mut percent: i32,
) {
    STlib_initNum(&raw mut (*p).n, x, y, pl, num, on, 3 as i32);
    (*p).p = percent;
}
pub unsafe fn STlib_updatePercent(
    state: &mut GameState,
    mut per: *mut st_percent_t,
    mut refresh: i32,
) {
    if refresh != 0 && *(*per).n.on {
        let patch = V_CachePatchNum(state, (*per).p);
        V_DrawPatch(state, (*per).n.x, (*per).n.y, patch);
    }
    STlib_updateNum(state, &raw mut (*per).n);
}
pub unsafe fn STlib_initMultIcon(
    mut i: *mut st_multicon_t,
    mut x: i32,
    mut y: i32,
    mut il: *mut i32,
    mut inum: *mut i32,
    mut on: *mut bool,
) {
    (*i).x = x;
    (*i).y = y;
    (*i).oldinum = -(1 as i32);
    (*i).inum = inum;
    (*i).on = on;
    (*i).p = il;
}
pub unsafe fn STlib_updateMultIcon(
    state: &mut GameState,
    mut mi: *mut st_multicon_t,
    mut refresh: bool,
) {
    let mut w: i32 = 0;
    let mut h: i32 = 0;
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    if *(*mi).on && ((*mi).oldinum != *(*mi).inum || refresh) && *(*mi).inum != -(1 as i32) {
        if (*mi).oldinum != -(1 as i32) {
            let old_patch = V_CachePatchNum(state, *(*mi).p.offset((*mi).oldinum as isize));
            x = (*mi).x - (*old_patch).leftoffset as i32;
            y = (*mi).y - (*old_patch).topoffset as i32;
            w = (*old_patch).width as i32;
            h = (*old_patch).height as i32;
            if y - ST_Y < 0 as i32 {
                I_Error("updateMultIcon: y - ST_Y < 0");
            }
            let st_backing_screen = state.st_stuff.st_backing_screen.as_mut_ptr();
            V_CopyRect(state,
                x,
                y - ST_Y,
                st_backing_screen,
                w,
                h,
                x,
                y,
            );
        }
        let new_patch = V_CachePatchNum(state, *(*mi).p.offset(*(*mi).inum as isize));
        V_DrawPatch(state,
            (*mi).x,
            (*mi).y,
            new_patch,
        );
        (*mi).oldinum = *(*mi).inum;
    }
}
pub unsafe fn STlib_initBinIcon(
    mut b: *mut st_binicon_t,
    mut x: i32,
    mut y: i32,
    mut i: i32,
    mut val: *mut bool,
    mut on: *mut bool,
) {
    (*b).x = x;
    (*b).y = y;
    (*b).oldval = false;
    (*b).val = val;
    (*b).on = on;
    (*b).p = i;
}
pub unsafe fn STlib_updateBinIcon(
    state: &mut GameState,
    mut bi: *mut st_binicon_t,
    mut refresh: bool,
) {
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut w: i32 = 0;
    let mut h: i32 = 0;
    if *(*bi).on && ((*bi).oldval != *(*bi).val || refresh) {
        let patch = V_CachePatchNum(state, (*bi).p);
        x = (*bi).x - (*patch).leftoffset as i32;
        y = (*bi).y - (*patch).topoffset as i32;
        w = (*patch).width as i32;
        h = (*patch).height as i32;
        if y - ST_Y < 0 as i32 {
            I_Error("updateBinIcon: y - ST_Y < 0");
        }
        if *(*bi).val {
            V_DrawPatch(state, (*bi).x, (*bi).y, patch);
        } else {
            let st_backing_screen = state.st_stuff.st_backing_screen.as_mut_ptr();
            V_CopyRect(state,
                x,
                y - ST_Y,
                st_backing_screen,
                w,
                h,
                x,
                y,
            );
        }
        (*bi).oldval = *(*bi).val;
    }
}
