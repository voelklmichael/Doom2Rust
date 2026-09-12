use crate::src::doomdef::boolean;
use crate::src::doomdef::false_0;
use crate::src::doomdef::true_0;
use crate::src::doomdef::NULL;
use crate::src::game_state::GameState;
use crate::src::m_bbox::{BOXBOTTOM, BOXLEFT, BOXRIGHT, BOXTOP};
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FixedDiv;
use crate::src::m_fixed::FixedMul;
use crate::src::m_fixed::FRACBITS;
use crate::src::m_fixed::FRACUNIT;
use crate::src::m_fixed::INT_MAX;
use crate::src::p_mobj::{line_t, mapthing_t, sector_t};
use crate::src::p_mobj::{mobj_s, mobj_t};
use crate::src::p_mobj::{MobjId, MF_NOBLOCKMAP, MF_NOSECTOR};
use crate::src::p_setup::SubsectorId;
use crate::src::r_main::R_PointInSubsector;

pub struct PMaputlState {
    pub intercepts_overrun: [intercepts_overrun_t; 23],
    pub opentop: fixed_t,
    pub openbottom: fixed_t,
    pub openrange: fixed_t,
    pub lowfloor: fixed_t,
    pub intercepts: [intercept_t; 189],
    pub intercept_p: *mut intercept_t,
    pub trace: divline_t,
    pub earlyout: bool,
    pub ptflags: i32,
}

impl PMaputlState {
    pub fn new() -> Self {
        PMaputlState {
            opentop: 0,
            openbottom: 0,
            openrange: 0,
            lowfloor: 0,
            intercepts: [intercept_t {
                frac: 0,
                isaline: false,
                d: C2RustUnnamed_1 {
                    thing: ::core::ptr::null::<mobj_t>() as *mut mobj_t,
                },
            }; 189],
            intercept_p: ::core::ptr::null::<intercept_t>() as *mut intercept_t,
            trace: divline_t {
                x: 0,
                y: 0,
                dx: 0,
                dy: 0,
            },
            earlyout: false,
            ptflags: 0,
            intercepts_overrun: {
                [
                    intercepts_overrun_t {
                        len: 4 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        // Patched once, after GameState reaches its final 'static address,
                        // by fixup_intercepts_overrun() -- see init_game_state()'s call site.
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        // Patched once, after GameState reaches its final 'static address,
                        // by fixup_intercepts_overrun() -- see init_game_state()'s call site.
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        // Patched once, after GameState reaches its final 'static address,
                        // by fixup_intercepts_overrun() -- see init_game_state()'s call site.
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        // Patched once, after GameState reaches its final 'static address,
                        // by fixup_intercepts_overrun() -- see init_game_state()'s call site.
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 120 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 8 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        // Patched once, after GameState reaches its final 'static address,
                        // by fixup_intercepts_overrun() -- see init_game_state()'s call site.
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 40 as i32,
                        // Patched once, after GameState reaches its final 'static address,
                        // by fixup_intercepts_overrun() -- see init_game_state()'s call site.
                        addr: NULL,
                        int16_array: true,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        // Patched once, after GameState reaches its final 'static address,
                        // by fixup_intercepts_overrun() -- see init_game_state()'s call site.
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        // Patched once, after GameState reaches its final 'static address,
                        // by fixup_intercepts_overrun() -- see init_game_state()'s call site.
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        // Patched once, after GameState reaches its final 'static address,
                        // by fixup_intercepts_overrun() -- see init_game_state()'s call site.
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 4 as i32,
                        // Patched once, after GameState reaches its final 'static address,
                        // by fixup_intercepts_overrun() -- see init_game_state()'s call site.
                        addr: NULL,
                        int16_array: false,
                    },
                    intercepts_overrun_t {
                        len: 0 as i32,
                        addr: NULL,
                        int16_array: false,
                    },
                ]
            },
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct divline_t {
    pub x: fixed_t,
    pub y: fixed_t,
    pub dx: fixed_t,
    pub dy: fixed_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct intercept_t {
    pub frac: fixed_t,
    pub isaline: bool,
    pub d: C2RustUnnamed_1,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub thing: *mut mobj_t,
    pub line: *mut line_t,
}
pub type traverser_t = Option<unsafe fn(&mut GameState, *mut intercept_t) -> boolean>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct intercepts_overrun_t {
    pub len: i32,
    pub addr: *mut ::core::ffi::c_void,
    pub int16_array: bool,
}
pub const MAPBLOCKUNITS: i32 = 128;
pub const MAPBLOCKSIZE: i32 = MAPBLOCKUNITS * FRACUNIT;
pub const MAPBLOCKSHIFT: i32 = FRACBITS + 7 as i32;
pub const MAPBTOFRAC: i32 = MAPBLOCKSHIFT - FRACBITS;
pub const MAXINTERCEPTS_ORIGINAL: i32 = 128;
pub const PT_ADDLINES: i32 = 1;
pub const PT_ADDTHINGS: i32 = 2;
pub const PT_EARLYOUT: i32 = 4;
pub unsafe fn P_AproxDistance(mut dx: fixed_t, mut dy: fixed_t) -> fixed_t {
    dx = (dx as i32).abs() as fixed_t;
    dy = (dy as i32).abs() as fixed_t;
    if dx < dy {
        return dx + dy - (dx >> 1 as i32);
    }
    return dx + dy - (dy >> 1 as i32);
}
pub unsafe fn P_PointOnLineSide(
    state: &mut GameState,
    mut x: fixed_t,
    mut y: fixed_t,
    mut line: *mut line_t,
) -> i32 {
    let mut dx: fixed_t = 0;
    let mut dy: fixed_t = 0;
    let mut left: fixed_t = 0;
    let mut right: fixed_t = 0;
    let line_v1 = state.p_setup.vertexes[(*line).v1.0 as usize];
    if (*line).dx == 0 {
        if x <= line_v1.x {
            return ((*line).dy > 0 as i32) as i32;
        }
        return ((*line).dy < 0 as i32) as i32;
    }
    if (*line).dy == 0 {
        if y <= line_v1.y {
            return ((*line).dx < 0 as i32) as i32;
        }
        return ((*line).dx > 0 as i32) as i32;
    }
    dx = x - line_v1.x;
    dy = y - line_v1.y;
    left = FixedMul((*line).dy >> FRACBITS, dx);
    right = FixedMul(dy, (*line).dx >> FRACBITS);
    if right < left {
        return 0 as i32;
    }
    return 1 as i32;
}
pub unsafe fn P_BoxOnLineSide(
    state: &mut GameState,
    mut tmbox: *mut fixed_t,
    mut ld: *mut line_t,
) -> i32 {
    let mut p1: i32 = 0 as i32;
    let mut p2: i32 = 0 as i32;
    match (*ld).slopetype as u32 {
        0 => {
            let ld_v1 = state.p_setup.vertexes[(*ld).v1.0 as usize];
            p1 = (*tmbox.offset(BOXTOP as i32 as isize) > ld_v1.y) as i32;
            p2 = (*tmbox.offset(BOXBOTTOM as i32 as isize) > ld_v1.y) as i32;
            if (*ld).dx < 0 as i32 {
                p1 ^= 1 as i32;
                p2 ^= 1 as i32;
            }
        }
        1 => {
            let ld_v1 = state.p_setup.vertexes[(*ld).v1.0 as usize];
            p1 = (*tmbox.offset(BOXRIGHT as i32 as isize) < ld_v1.x) as i32;
            p2 = (*tmbox.offset(BOXLEFT as i32 as isize) < ld_v1.x) as i32;
            if (*ld).dy < 0 as i32 {
                p1 ^= 1 as i32;
                p2 ^= 1 as i32;
            }
        }
        2 => {
            p1 = P_PointOnLineSide(
                state,
                *tmbox.offset(BOXLEFT as i32 as isize),
                *tmbox.offset(BOXTOP as i32 as isize),
                ld,
            );
            p2 = P_PointOnLineSide(
                state,
                *tmbox.offset(BOXRIGHT as i32 as isize),
                *tmbox.offset(BOXBOTTOM as i32 as isize),
                ld,
            );
        }
        3 => {
            p1 = P_PointOnLineSide(
                state,
                *tmbox.offset(BOXRIGHT as i32 as isize),
                *tmbox.offset(BOXTOP as i32 as isize),
                ld,
            );
            p2 = P_PointOnLineSide(
                state,
                *tmbox.offset(BOXLEFT as i32 as isize),
                *tmbox.offset(BOXBOTTOM as i32 as isize),
                ld,
            );
        }
        _ => {}
    }
    if p1 == p2 {
        return p1;
    }
    return -(1 as i32);
}
pub unsafe fn P_PointOnDivlineSide(
    mut x: fixed_t,
    mut y: fixed_t,
    mut line: *mut divline_t,
) -> i32 {
    let mut dx: fixed_t = 0;
    let mut dy: fixed_t = 0;
    let mut left: fixed_t = 0;
    let mut right: fixed_t = 0;
    if (*line).dx == 0 {
        if x <= (*line).x {
            return ((*line).dy > 0 as i32) as i32;
        }
        return ((*line).dy < 0 as i32) as i32;
    }
    if (*line).dy == 0 {
        if y <= (*line).y {
            return ((*line).dx < 0 as i32) as i32;
        }
        return ((*line).dx > 0 as i32) as i32;
    }
    dx = x - (*line).x;
    dy = y - (*line).y;
    if ((*line).dy ^ (*line).dx ^ dx ^ dy) as u32 & 0x80000000 as u32 != 0 {
        if ((*line).dy ^ dx) as u32 & 0x80000000 as u32 != 0 {
            return 1 as i32;
        }
        return 0 as i32;
    }
    left = FixedMul((*line).dy >> 8 as i32, dx >> 8 as i32);
    right = FixedMul(dy >> 8 as i32, (*line).dx >> 8 as i32);
    if right < left {
        return 0 as i32;
    }
    return 1 as i32;
}
pub unsafe fn P_MakeDivline(state: &mut GameState, mut li: *mut line_t, mut dl: *mut divline_t) {
    let li_v1 = state.p_setup.vertexes[(*li).v1.0 as usize];
    (*dl).x = li_v1.x;
    (*dl).y = li_v1.y;
    (*dl).dx = (*li).dx;
    (*dl).dy = (*li).dy;
}
pub unsafe fn P_InterceptVector(mut v2: *mut divline_t, mut v1: *mut divline_t) -> fixed_t {
    let mut frac: fixed_t = 0;
    let mut num: fixed_t = 0;
    let mut den: fixed_t = 0;
    den = FixedMul((*v1).dy >> 8 as i32, (*v2).dx) - FixedMul((*v1).dx >> 8 as i32, (*v2).dy);
    if den == 0 as i32 {
        return 0 as fixed_t;
    }
    num = FixedMul((*v1).x - (*v2).x >> 8 as i32, (*v1).dy)
        + FixedMul((*v2).y - (*v1).y >> 8 as i32, (*v1).dx);
    frac = FixedDiv(num, den);
    return frac;
}
pub unsafe fn P_LineOpening(state: &mut GameState, mut linedef: *mut line_t) {
    let mut front: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut back: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    if (*linedef).sidenum[1 as i32 as usize] as i32 == -(1 as i32) {
        state.p_maputl.openrange = 0 as i32 as fixed_t;
        return;
    }
    front = state.p_setup.sector_mut((*linedef).frontsector.unwrap());
    back = state.p_setup.sector_mut((*linedef).backsector.unwrap());
    if (*front).ceilingheight < (*back).ceilingheight {
        state.p_maputl.opentop = (*front).ceilingheight;
    } else {
        state.p_maputl.opentop = (*back).ceilingheight;
    }
    if (*front).floorheight > (*back).floorheight {
        state.p_maputl.openbottom = (*front).floorheight;
        state.p_maputl.lowfloor = (*back).floorheight;
    } else {
        state.p_maputl.openbottom = (*back).floorheight;
        state.p_maputl.lowfloor = (*front).floorheight;
    }
    state.p_maputl.openrange = state.p_maputl.opentop - state.p_maputl.openbottom;
}
pub unsafe fn P_UnsetThingPosition(state: &mut GameState, mut thing: *mut mobj_t) {
    let mut blockx: i32 = 0;
    let mut blocky: i32 = 0;
    if (*thing).flags & MF_NOSECTOR as i32 == 0 {
        if let Some(id) = (*thing).snext {
            let next = state
                .p_mobj
                .mobj_get(id)
                .expect("sector-list snext neighbor is always live");
            (*next).sprev = (*thing).sprev;
        }
        if let Some(id) = (*thing).sprev {
            let prev = state
                .p_mobj
                .mobj_get(id)
                .expect("sector-list sprev neighbor is always live");
            (*prev).snext = (*thing).snext;
        } else {
            (*state
                .p_setup
                .sector_mut(state.p_setup.subsectors[(*thing).subsector.0 as usize].sector))
            .thinglist = (*thing).snext;
        }
    }
    if (*thing).flags & MF_NOBLOCKMAP as i32 == 0 {
        if !(*thing).bnext.is_null() {
            (*(*thing).bnext).bprev = (*thing).bprev;
        }
        if !(*thing).bprev.is_null() {
            (*(*thing).bprev).bnext = (*thing).bnext;
        } else {
            blockx = ((*thing).x - state.p_setup.bmaporgx >> MAPBLOCKSHIFT) as i32;
            blocky = ((*thing).y - state.p_setup.bmaporgy >> MAPBLOCKSHIFT) as i32;
            if blockx >= 0 as i32
                && blockx < state.p_setup.bmapwidth
                && blocky >= 0 as i32
                && blocky < state.p_setup.bmapheight
            {
                let ref mut fresh1 = *state
                    .p_setup
                    .blocklinks
                    .as_mut_ptr()
                    .offset((blocky * state.p_setup.bmapwidth + blockx) as isize);
                *fresh1 = (*thing).bnext as *mut mobj_t;
            }
        }
    }
}
pub unsafe fn P_SetThingPosition(state: &mut GameState, mut thing: *mut mobj_t) {
    let mut ss: SubsectorId = SubsectorId(0);
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut blockx: i32 = 0;
    let mut blocky: i32 = 0;
    let mut link: *mut *mut mobj_t = ::core::ptr::null_mut::<*mut mobj_t>();
    ss = R_PointInSubsector(state, (*thing).x, (*thing).y);
    (*thing).subsector = ss;
    if (*thing).flags & MF_NOSECTOR as i32 == 0 {
        sec = state.p_setup.sector_mut(state.p_setup.subsectors[ss.0 as usize].sector);
        (*thing).sprev = None;
        (*thing).snext = (*sec).thinglist;
        if let Some(head_id) = (*sec).thinglist {
            let head = state
                .p_mobj
                .mobj_get(head_id)
                .expect("sector thinglist head is always live");
            (*head).sprev = Some((*thing).id);
        }
        (*sec).thinglist = Some((*thing).id);
    }
    if (*thing).flags & MF_NOBLOCKMAP as i32 == 0 {
        blockx = ((*thing).x - state.p_setup.bmaporgx >> MAPBLOCKSHIFT) as i32;
        blocky = ((*thing).y - state.p_setup.bmaporgy >> MAPBLOCKSHIFT) as i32;
        if blockx >= 0 as i32
            && blockx < state.p_setup.bmapwidth
            && blocky >= 0 as i32
            && blocky < state.p_setup.bmapheight
        {
            link = state
                .p_setup
                .blocklinks
                .as_mut_ptr()
                .offset((blocky * state.p_setup.bmapwidth + blockx) as isize);
            (*thing).bprev = ::core::ptr::null_mut::<mobj_s>();
            (*thing).bnext = *link as *mut mobj_s;
            if !(*link).is_null() {
                (**link).bprev = thing as *mut mobj_s;
            }
            *link = thing;
        } else {
            (*thing).bprev = ::core::ptr::null_mut::<mobj_s>();
            (*thing).bnext = (*thing).bprev;
        }
    }
}
pub unsafe fn P_BlockLinesIterator(
    state: &mut GameState,
    mut x: i32,
    mut y: i32,
    mut func: Option<unsafe fn(&mut GameState, *mut line_t) -> boolean>,
) -> bool {
    let mut offset: i32 = 0;
    let mut list: *mut i16 = ::core::ptr::null_mut::<i16>();
    let mut ld: *mut line_t = ::core::ptr::null_mut::<line_t>();
    if x < 0 as i32 || y < 0 as i32 || x >= state.p_setup.bmapwidth || y >= state.p_setup.bmapheight
    {
        return true;
    }
    offset = y * state.p_setup.bmapwidth + x;
    offset = *state.p_setup.blockmap.offset(offset as isize) as i32;
    list = state.p_setup.blockmaplump.as_mut_ptr().offset(offset as isize);
    while *list as i32 != -(1 as i32) {
        ld = state.p_setup.lines.as_mut_ptr().offset(*list as isize);
        if !((*ld).validcount == state.r_main.validcount) {
            (*ld).validcount = state.r_main.validcount;
            if func.expect("non-null function pointer")(state, ld) == 0 {
                return false;
            }
        }
        list = list.offset(1);
    }
    return true;
}
pub unsafe fn P_BlockThingsIterator(
    state: &mut GameState,
    mut x: i32,
    mut y: i32,
    mut func: Option<unsafe fn(&mut GameState, MobjId) -> boolean>,
) -> bool {
    let mut mobj: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    if x < 0 as i32 || y < 0 as i32 || x >= state.p_setup.bmapwidth || y >= state.p_setup.bmapheight
    {
        return true;
    }
    mobj = *state
        .p_setup
        .blocklinks
        .as_mut_ptr()
        .offset((y * state.p_setup.bmapwidth + x) as isize);
    while !mobj.is_null() {
        if func.expect("non-null function pointer")(state, (*mobj).id) == 0 {
            return false;
        }
        mobj = (*mobj).bnext as *mut mobj_t;
    }
    return true;
}
#[no_mangle]
pub unsafe fn PIT_AddLineIntercepts(
    state: &mut GameState,
    mut ld: *mut line_t,
) -> boolean {
    let mut s1: i32 = 0;
    let mut s2: i32 = 0;
    let mut frac: fixed_t = 0;
    let mut dl: divline_t = divline_t {
        x: 0,
        y: 0,
        dx: 0,
        dy: 0,
    };
    if state.p_maputl.trace.dx > FRACUNIT * 16 as i32
        || state.p_maputl.trace.dy > FRACUNIT * 16 as i32
        || state.p_maputl.trace.dx < -FRACUNIT * 16 as i32
        || state.p_maputl.trace.dy < -FRACUNIT * 16 as i32
    {
        let ld_v1 = state.p_setup.vertexes[(*ld).v1.0 as usize];
        let ld_v2 = state.p_setup.vertexes[(*ld).v2.0 as usize];
        s1 = P_PointOnDivlineSide(ld_v1.x, ld_v1.y, &raw mut state.p_maputl.trace);
        s2 = P_PointOnDivlineSide(ld_v2.x, ld_v2.y, &raw mut state.p_maputl.trace);
    } else {
        s1 = P_PointOnLineSide(state, state.p_maputl.trace.x, state.p_maputl.trace.y, ld);
        s2 = P_PointOnLineSide(
            state,
            state.p_maputl.trace.x + state.p_maputl.trace.dx,
            state.p_maputl.trace.y + state.p_maputl.trace.dy,
            ld,
        );
    }
    if s1 == s2 {
        return true_0 as boolean;
    }
    P_MakeDivline(state, ld, &raw mut dl);
    frac = P_InterceptVector(&raw mut state.p_maputl.trace, &raw mut dl);
    if frac < 0 as i32 {
        return true_0 as boolean;
    }
    if state.p_maputl.earlyout && frac < FRACUNIT && (*ld).backsector.is_none() {
        return false_0 as boolean;
    }
    (*state.p_maputl.intercept_p).frac = frac;
    (*state.p_maputl.intercept_p).isaline = true;
    (*state.p_maputl.intercept_p).d.line = ld;
    let num_intercepts = state
        .p_maputl
        .intercept_p
        .offset_from(&raw mut state.p_maputl.intercepts as *mut intercept_t)
        as i64 as i32;
    let intercept_p = state.p_maputl.intercept_p;
    InterceptsOverrun(state, num_intercepts, intercept_p);
    state.p_maputl.intercept_p = state.p_maputl.intercept_p.offset(1);
    return true_0 as boolean;
}
#[no_mangle]
pub unsafe fn PIT_AddThingIntercepts(
    state: &mut GameState,
    mut thing_id: MobjId,
) -> boolean {
    let thing = state.p_mobj.mobj_get(thing_id).unwrap();
    let mut x1: fixed_t = 0;
    let mut y1: fixed_t = 0;
    let mut x2: fixed_t = 0;
    let mut y2: fixed_t = 0;
    let mut s1: i32 = 0;
    let mut s2: i32 = 0;
    let mut tracepositive: bool = false;
    let mut dl: divline_t = divline_t {
        x: 0,
        y: 0,
        dx: 0,
        dy: 0,
    };
    let mut frac: fixed_t = 0;
    tracepositive = state.p_maputl.trace.dx ^ state.p_maputl.trace.dy > 0 as i32;
    if tracepositive {
        x1 = (*thing).x - (*thing).radius;
        y1 = (*thing).y + (*thing).radius;
        x2 = (*thing).x + (*thing).radius;
        y2 = (*thing).y - (*thing).radius;
    } else {
        x1 = (*thing).x - (*thing).radius;
        y1 = (*thing).y - (*thing).radius;
        x2 = (*thing).x + (*thing).radius;
        y2 = (*thing).y + (*thing).radius;
    }
    s1 = P_PointOnDivlineSide(x1, y1, &raw mut state.p_maputl.trace);
    s2 = P_PointOnDivlineSide(x2, y2, &raw mut state.p_maputl.trace);
    if s1 == s2 {
        return true_0 as boolean;
    }
    dl.x = x1;
    dl.y = y1;
    dl.dx = x2 - x1;
    dl.dy = y2 - y1;
    frac = P_InterceptVector(&raw mut state.p_maputl.trace, &raw mut dl);
    if frac < 0 as i32 {
        return true_0 as boolean;
    }
    (*state.p_maputl.intercept_p).frac = frac;
    (*state.p_maputl.intercept_p).isaline = false;
    (*state.p_maputl.intercept_p).d.thing = thing;
    let num_intercepts = state
        .p_maputl
        .intercept_p
        .offset_from(&raw mut state.p_maputl.intercepts as *mut intercept_t)
        as i64 as i32;
    let intercept_p = state.p_maputl.intercept_p;
    InterceptsOverrun(state, num_intercepts, intercept_p);
    state.p_maputl.intercept_p = state.p_maputl.intercept_p.offset(1);
    return true_0 as boolean;
}
pub unsafe fn P_TraverseIntercepts(
    state: &mut GameState,
    mut func: traverser_t,
    mut maxfrac: fixed_t,
) -> bool {
    let mut count: i32 = 0;
    let mut dist: fixed_t = 0;
    let mut scan: *mut intercept_t = ::core::ptr::null_mut::<intercept_t>();
    let mut in_0: *mut intercept_t = ::core::ptr::null_mut::<intercept_t>();
    count = state
        .p_maputl
        .intercept_p
        .offset_from(&raw mut state.p_maputl.intercepts as *mut intercept_t) as i64
        as i32;
    in_0 = ::core::ptr::null_mut::<intercept_t>();
    loop {
        let fresh0 = count;
        count = count - 1;
        if !(fresh0 != 0) {
            break;
        }
        dist = INT_MAX as fixed_t;
        scan = &raw mut state.p_maputl.intercepts as *mut intercept_t;
        while scan < state.p_maputl.intercept_p {
            if (*scan).frac < dist {
                dist = (*scan).frac;
                in_0 = scan;
            }
            scan = scan.offset(1);
        }
        if dist > maxfrac {
            return true;
        }
        if func.expect("non-null function pointer")(state, in_0) == 0 {
            return false;
        }
        (*in_0).frac = INT_MAX as fixed_t;
    }
    return true;
}
// Must run only after `gs` is at its final, permanently-stable 'static
// address (see init_game_state()) -- this table's `bulletslope` entry records
// an address for the vanilla-intercepts-overrun memory-corruption
// emulation to write through later; that address must be bulletslope's
// real final location, not a temporary one that could move.
pub unsafe fn fixup_intercepts_overrun(gs: &mut GameState) {
    gs.p_maputl.intercepts_overrun[3].addr =
        &raw mut gs.p_maputl.lowfloor as *mut fixed_t as *mut ::core::ffi::c_void;
    gs.p_maputl.intercepts_overrun[4].addr =
        &raw mut gs.p_maputl.openbottom as *mut fixed_t as *mut ::core::ffi::c_void;
    gs.p_maputl.intercepts_overrun[5].addr =
        &raw mut gs.p_maputl.opentop as *mut fixed_t as *mut ::core::ffi::c_void;
    gs.p_maputl.intercepts_overrun[6].addr =
        &raw mut gs.p_maputl.openrange as *mut fixed_t as *mut ::core::ffi::c_void;
    gs.p_maputl.intercepts_overrun[10].addr =
        &raw mut gs.p_pspr.bulletslope as *mut fixed_t as *mut ::core::ffi::c_void;
    gs.p_maputl.intercepts_overrun[14].addr =
        &raw mut gs.p_setup.playerstarts as *mut [mapthing_t; 4] as *mut ::core::ffi::c_void;
    gs.p_maputl.intercepts_overrun[16].addr =
        &raw mut gs.p_setup.bmapwidth as *mut i32 as *mut ::core::ffi::c_void;
    gs.p_maputl.intercepts_overrun[18].addr =
        &raw mut gs.p_setup.bmaporgx as *mut fixed_t as *mut ::core::ffi::c_void;
    gs.p_maputl.intercepts_overrun[19].addr =
        &raw mut gs.p_setup.bmaporgy as *mut fixed_t as *mut ::core::ffi::c_void;
    gs.p_maputl.intercepts_overrun[21].addr =
        &raw mut gs.p_setup.bmapheight as *mut i32 as *mut ::core::ffi::c_void;
}
unsafe fn InterceptsMemoryOverrun(state: &mut GameState, mut location: i32, mut value: i32) {
    let mut i: i32 = 0;
    let mut offset: i32 = 0;
    let mut index: i32 = 0;
    let mut addr: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    i = 0 as i32;
    offset = 0 as i32;
    while state.p_maputl.intercepts_overrun[i as usize].len != 0 as i32 {
        if offset + state.p_maputl.intercepts_overrun[i as usize].len > location {
            addr = state.p_maputl.intercepts_overrun[i as usize].addr;
            if !addr.is_null() {
                if state.p_maputl.intercepts_overrun[i as usize].int16_array {
                    index = (location - offset) / 2 as i32;
                    *(addr as *mut i16).offset(index as isize) = (value & 0xffff as i32) as i16;
                    *(addr as *mut i16).offset((index + 1 as i32) as isize) =
                        (value >> 16 as i32 & 0xffff as i32) as i16;
                } else {
                    index = (location - offset) / 4 as i32;
                    *(addr as *mut i32).offset(index as isize) = value;
                }
            }
            break;
        } else {
            offset += state.p_maputl.intercepts_overrun[i as usize].len;
            i += 1;
        }
    }
}
unsafe fn InterceptsOverrun(
    state: &mut GameState,
    mut num_intercepts: i32,
    mut intercept: *mut intercept_t,
) {
    let mut location: i32 = 0;
    if num_intercepts <= MAXINTERCEPTS_ORIGINAL {
        return;
    }
    location = (num_intercepts - MAXINTERCEPTS_ORIGINAL - 1 as i32) * 12 as i32;
    InterceptsMemoryOverrun(state, location, (*intercept).frac as i32);
    InterceptsMemoryOverrun(state, location + 4 as i32, (*intercept).isaline as i32);
    InterceptsMemoryOverrun(state, location + 8 as i32, (*intercept).d.thing as i32);
}
pub unsafe fn P_PathTraverse(
    state: &mut GameState,
    mut x1: fixed_t,
    mut y1: fixed_t,
    mut x2: fixed_t,
    mut y2: fixed_t,
    mut flags: i32,
    mut trav: Option<unsafe fn(&mut GameState, *mut intercept_t) -> boolean>,
) -> bool {
    let mut xt1: fixed_t = 0;
    let mut yt1: fixed_t = 0;
    let mut xt2: fixed_t = 0;
    let mut yt2: fixed_t = 0;
    let mut xstep: fixed_t = 0;
    let mut ystep: fixed_t = 0;
    let mut partial: fixed_t = 0;
    let mut xintercept: fixed_t = 0;
    let mut yintercept: fixed_t = 0;
    let mut mapx: i32 = 0;
    let mut mapy: i32 = 0;
    let mut mapxstep: i32 = 0;
    let mut mapystep: i32 = 0;
    let mut count: i32 = 0;
    state.p_maputl.earlyout = (flags & PT_EARLYOUT) != 0;
    state.r_main.validcount += 1;
    state.p_maputl.intercept_p = &raw mut state.p_maputl.intercepts as *mut intercept_t;
    if x1 as i32 - state.p_setup.bmaporgx as i32 & MAPBLOCKSIZE - 1 as i32 == 0 as i32 {
        x1 += FRACUNIT;
    }
    if y1 as i32 - state.p_setup.bmaporgy as i32 & MAPBLOCKSIZE - 1 as i32 == 0 as i32 {
        y1 += FRACUNIT;
    }
    state.p_maputl.trace.x = x1;
    state.p_maputl.trace.y = y1;
    state.p_maputl.trace.dx = x2 - x1;
    state.p_maputl.trace.dy = y2 - y1;
    x1 -= state.p_setup.bmaporgx;
    y1 -= state.p_setup.bmaporgy;
    xt1 = x1 >> MAPBLOCKSHIFT;
    yt1 = y1 >> MAPBLOCKSHIFT;
    x2 -= state.p_setup.bmaporgx;
    y2 -= state.p_setup.bmaporgy;
    xt2 = x2 >> MAPBLOCKSHIFT;
    yt2 = y2 >> MAPBLOCKSHIFT;
    if xt2 > xt1 {
        mapxstep = 1 as i32;
        partial = (FRACUNIT - (x1 as i32 >> MAPBTOFRAC & FRACUNIT - 1 as i32)) as fixed_t;
        ystep = FixedDiv(y2 - y1, (x2 as i32 - x1 as i32).abs() as fixed_t);
    } else if xt2 < xt1 {
        mapxstep = -(1 as i32);
        partial = (x1 as i32 >> MAPBTOFRAC & FRACUNIT - 1 as i32) as fixed_t;
        ystep = FixedDiv(y2 - y1, (x2 as i32 - x1 as i32).abs() as fixed_t);
    } else {
        mapxstep = 0 as i32;
        partial = FRACUNIT as fixed_t;
        ystep = (256 as i32 * FRACUNIT) as fixed_t;
    }
    yintercept = (y1 >> MAPBTOFRAC) + FixedMul(partial, ystep);
    if yt2 > yt1 {
        mapystep = 1 as i32;
        partial = (FRACUNIT - (y1 as i32 >> MAPBTOFRAC & FRACUNIT - 1 as i32)) as fixed_t;
        xstep = FixedDiv(x2 - x1, (y2 as i32 - y1 as i32).abs() as fixed_t);
    } else if yt2 < yt1 {
        mapystep = -(1 as i32);
        partial = (y1 as i32 >> MAPBTOFRAC & FRACUNIT - 1 as i32) as fixed_t;
        xstep = FixedDiv(x2 - x1, (y2 as i32 - y1 as i32).abs() as fixed_t);
    } else {
        mapystep = 0 as i32;
        partial = FRACUNIT as fixed_t;
        xstep = (256 as i32 * FRACUNIT) as fixed_t;
    }
    xintercept = (x1 >> MAPBTOFRAC) + FixedMul(partial, xstep);
    mapx = xt1 as i32;
    mapy = yt1 as i32;
    count = 0 as i32;
    while count < 64 as i32 {
        if flags & PT_ADDLINES != 0 {
            if !P_BlockLinesIterator(
                state,
                mapx,
                mapy,
                Some(
                    PIT_AddLineIntercepts
                        as unsafe fn(&mut GameState, *mut line_t) -> boolean,
                ),
            ) {
                return false;
            }
        }
        if flags & PT_ADDTHINGS != 0 {
            if !P_BlockThingsIterator(
                state,
                mapx,
                mapy,
                Some(
                    PIT_AddThingIntercepts
                        as unsafe fn(&mut GameState, MobjId) -> boolean,
                ),
            ) {
                return false;
            }
        }
        if mapx == xt2 && mapy == yt2 {
            break;
        }
        if yintercept >> FRACBITS == mapy {
            yintercept += ystep;
            mapx += mapxstep;
        } else if xintercept >> FRACBITS == mapx {
            xintercept += xstep;
            mapy += mapystep;
        }
        count += 1;
    }
    return P_TraverseIntercepts(state, trav as traverser_t, FRACUNIT);
}
