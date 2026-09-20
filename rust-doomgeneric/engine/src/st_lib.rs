use crate::filesystem::DoomFileSystem;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::st_stuff::ST_Y;
use crate::v_video::cache_patch_num;
use crate::v_video::copy_rect;
use crate::v_video::draw_patch;
use crate::v_video::Screen;
use crate::w_wad::WWadState;
use crate::w_wad::{get_num_for_name, lump_bytes};

// Identifies one of StStuffState's own fixed lump-number arrays -- always
// what a raw `*mut i32` used to point at here (tallnum/shortnum/faces/keys,
// or one weapon's 2-element on/off pair within arms). Resolved back to a
// slice via StStuffState::digit_set.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StDigitSet {
    TallNum,
    ShortNum,
    Faces,
    Arms(usize),
    Keys,
}
#[derive(Copy, Clone)]
pub struct StNumber {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub oldnum: i32,
    pub p: StDigitSet,
    pub data: i32,
}
#[derive(Copy, Clone)]
pub struct StPercent {
    pub n: StNumber,
    pub p: i32,
}
#[derive(Copy, Clone)]
pub struct StMultiIcon {
    pub x: i32,
    pub y: i32,
    pub oldinum: i32,
    pub p: StDigitSet,
    pub data: i32,
}
#[derive(Copy, Clone)]
pub struct StBinIcon {
    pub x: i32,
    pub y: i32,
    pub oldval: bool,
    pub p: i32,
    pub data: i32,
}
pub struct StLibState {
    sttminus: i32,
}

impl Default for StLibState {
    fn default() -> Self {
        Self::new()
    }
}

impl StLibState {
    pub const fn new() -> Self {
        Self { sttminus: -1 }
    }
}

pub fn stlib_init(fs: &dyn DoomFileSystem, st_lib: &mut StLibState, w_wad: &mut WWadState) {
    let lumpnum = get_num_for_name(w_wad, "STTMINUS");
    lump_bytes(fs, w_wad, lumpnum);
    st_lib.sttminus = lumpnum;
}
pub fn stlib_init_num(n: &mut StNumber, x: i32, y: i32, pl: StDigitSet, width: i32) {
    n.x = x;
    n.y = y;
    n.oldnum = 0;
    n.width = width;
    n.p = pl;
}
pub fn stlib_draw_num(state: &mut GameState, n: &mut StNumber, mut num: i32) {
    let mut numdigits: i32 = n.width;
    let zero_lump = state.ui.st_stuff.digit_set(n.p)[0];
    let zero_patch = cache_patch_num(&*state.assets.fs, &mut state.assets.w_wad, zero_lump);
    let w: i32 = zero_patch.width();
    let h: i32 = zero_patch.height();

    n.oldnum = num;
    let neg: i32 = (num < 0) as i32;
    if neg != 0 {
        if numdigits == 2 && num < -9 {
            num = -9;
        } else if numdigits == 3 && num < -99 {
            num = -99;
        }
        num = -num;
    }
    let mut x: i32 = n.x - numdigits * w;
    if n.y - ST_Y < 0 {
        error("drawNum: n->y - ST_Y < 0");
    }
    let st_backing_screen = Screen::StatusBar;
    let dest_screen = Screen::Video;
    copy_rect(
        state,
        dest_screen,
        x,
        n.y - ST_Y,
        st_backing_screen,
        w * numdigits,
        h,
        x,
        n.y,
    );
    if num == 1994 {
        return;
    }
    x = n.x;
    if num == 0 {
        let dest_screen = Screen::Video;
        draw_patch(state, dest_screen, x - w, n.y, &zero_patch);
    }
    while num != 0 && {
        let fresh0 = numdigits;
        numdigits -= 1;
        fresh0 != 0
    } {
        x -= w;
        let digit_lump = state.ui.st_stuff.digit_set(n.p)[(num % 10) as usize];
        let digit_patch = cache_patch_num(&*state.assets.fs, &mut state.assets.w_wad, digit_lump);
        let dest_screen = Screen::Video;
        draw_patch(state, dest_screen, x, n.y, &digit_patch);
        num /= 10;
    }
    if neg != 0 {
        let patch = cache_patch_num(
            &*state.assets.fs,
            &mut state.assets.w_wad,
            state.ui.st_lib.sttminus,
        );
        let dest_screen = Screen::Video;
        draw_patch(state, dest_screen, x - 8, n.y, &patch);
    }
}
pub fn stlib_update_num(state: &mut GameState, n: &mut StNumber, num: i32, on: bool) {
    if on {
        stlib_draw_num(state, n, num);
    }
}
pub fn stlib_init_percent(p: &mut StPercent, x: i32, y: i32, pl: StDigitSet, percent: i32) {
    stlib_init_num(&mut p.n, x, y, pl, 3);
    p.p = percent;
}
pub fn stlib_update_percent(
    state: &mut GameState,
    per: &mut StPercent,
    num: i32,
    on: bool,
    refresh: i32,
) {
    if refresh != 0 && on {
        let patch = cache_patch_num(&*state.assets.fs, &mut state.assets.w_wad, per.p);
        let dest_screen = Screen::Video;
        draw_patch(state, dest_screen, per.n.x, per.n.y, &patch);
    }
    stlib_update_num(state, &mut per.n, num, on);
}
pub fn stlib_init_mult_icon(i: &mut StMultiIcon, x: i32, y: i32, il: StDigitSet) {
    i.x = x;
    i.y = y;
    i.oldinum = -1;
    i.p = il;
}
pub fn stlib_update_mult_icon(
    state: &mut GameState,
    mi: &mut StMultiIcon,
    inum: i32,
    on: bool,
    refresh: bool,
) {
    let w: i32;
    let h: i32;
    let x: i32;
    let y: i32;
    if on && (mi.oldinum != inum || refresh) && inum != -1 {
        if mi.oldinum != -1 {
            let old_lump = state.ui.st_stuff.digit_set(mi.p)[mi.oldinum as usize];
            let old_patch = cache_patch_num(&*state.assets.fs, &mut state.assets.w_wad, old_lump);
            x = mi.x - old_patch.leftoffset();
            y = mi.y - old_patch.topoffset();
            w = old_patch.width();
            h = old_patch.height();
            if y - ST_Y < 0 {
                error("updateMultIcon: y - ST_Y < 0");
            }
            let st_backing_screen = Screen::StatusBar;
            let dest_screen = Screen::Video;
            copy_rect(
                state,
                dest_screen,
                x,
                y - ST_Y,
                st_backing_screen,
                w,
                h,
                x,
                y,
            );
        }
        let new_lump = state.ui.st_stuff.digit_set(mi.p)[inum as usize];
        let new_patch = cache_patch_num(&*state.assets.fs, &mut state.assets.w_wad, new_lump);
        let dest_screen = Screen::Video;
        draw_patch(state, dest_screen, mi.x, mi.y, &new_patch);
        mi.oldinum = inum;
    }
}
pub fn stlib_init_bin_icon(b: &mut StBinIcon, x: i32, y: i32, i: i32) {
    b.x = x;
    b.y = y;
    b.oldval = false;
    b.p = i;
}
pub fn stlib_update_bin_icon(
    state: &mut GameState,
    bi: &mut StBinIcon,
    val: bool,
    on: bool,
    refresh: bool,
) {
    let x: i32;
    let y: i32;
    let w: i32;
    let h: i32;
    if on && (bi.oldval != val || refresh) {
        let patch = cache_patch_num(&*state.assets.fs, &mut state.assets.w_wad, bi.p);
        x = bi.x - patch.leftoffset();
        y = bi.y - patch.topoffset();
        w = patch.width();
        h = patch.height();
        if y - ST_Y < 0 {
            error("updateBinIcon: y - ST_Y < 0");
        }
        if val {
            let dest_screen = Screen::Video;
            draw_patch(state, dest_screen, bi.x, bi.y, &patch);
        } else {
            let st_backing_screen = Screen::StatusBar;
            let dest_screen = Screen::Video;
            copy_rect(
                state,
                dest_screen,
                x,
                y - ST_Y,
                st_backing_screen,
                w,
                h,
                x,
                y,
            );
        }
        bi.oldval = val;
    }
}
