use crate::src::game_state::GameState;
use crate::src::i_system::I_Error;
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FixedDiv;
use crate::src::m_fixed::FixedMul;
use crate::src::m_fixed::FRACBITS;
use crate::src::p_maputl::divline_t;
use crate::src::p_mobj::mobj_t;
use crate::src::p_mobj::{line_t, sector_t, subsector_t, vertex_t};
use crate::src::p_setup::SubsectorId;
use crate::src::p_spec::ML_TWOSIDED;
use crate::src::r_bsp::NF_SUBSECTOR;
use crate::src::r_defs::{node_t, seg_t};

pub struct PSightState {
    sightzstart: fixed_t,
    pub topslope: fixed_t,
    pub bottomslope: fixed_t,
    strace: divline_t,
    t2x: fixed_t,
    t2y: fixed_t,
    sightcounts: [i32; 2],
}

impl PSightState {
    pub const fn new() -> Self {
        PSightState {
            sightzstart: 0,
            topslope: 0,
            bottomslope: 0,
            strace: divline_t {
                x: 0,
                y: 0,
                dx: 0,
                dy: 0,
            },
            t2x: 0,
            t2y: 0,
            sightcounts: [0; 2],
        }
    }
}
pub unsafe fn P_DivlineSide(mut x: fixed_t, mut y: fixed_t, mut node: *mut divline_t) -> i32 {
    let mut dx: fixed_t = 0;
    let mut dy: fixed_t = 0;
    let mut left: fixed_t = 0;
    let mut right: fixed_t = 0;
    if (*node).dx == 0 {
        if x == (*node).x {
            return 2 as i32;
        }
        if x <= (*node).x {
            return ((*node).dy > 0 as i32) as i32;
        }
        return ((*node).dy < 0 as i32) as i32;
    }
    if (*node).dy == 0 {
        if x == (*node).y {
            return 2 as i32;
        }
        if y <= (*node).y {
            return ((*node).dx < 0 as i32) as i32;
        }
        return ((*node).dx > 0 as i32) as i32;
    }
    dx = x - (*node).x;
    dy = y - (*node).y;
    left = ((*node).dy >> FRACBITS) * (dx >> FRACBITS);
    right = (dy >> FRACBITS) * ((*node).dx >> FRACBITS);
    if right < left {
        return 0 as i32;
    }
    if left == right {
        return 2 as i32;
    }
    return 1 as i32;
}
pub unsafe fn P_InterceptVector2(mut v2: *mut divline_t, mut v1: *mut divline_t) -> fixed_t {
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
pub unsafe fn P_CrossSubsector(state: &mut GameState, mut num: i32) -> bool {
    let mut seg: *mut seg_t = ::core::ptr::null_mut::<seg_t>();
    let mut line: *mut line_t = ::core::ptr::null_mut::<line_t>();
    let mut s1: i32 = 0;
    let mut s2: i32 = 0;
    let mut count: i32 = 0;
    let mut sub: *mut subsector_t = ::core::ptr::null_mut::<subsector_t>();
    let mut front: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut back: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut opentop: fixed_t = 0;
    let mut openbottom: fixed_t = 0;
    let mut divl: divline_t = divline_t {
        x: 0,
        y: 0,
        dx: 0,
        dy: 0,
    };
    let mut v1: *mut vertex_t = ::core::ptr::null_mut::<vertex_t>();
    let mut v2: *mut vertex_t = ::core::ptr::null_mut::<vertex_t>();
    let mut frac: fixed_t = 0;
    let mut slope: fixed_t = 0;
    if num >= state.p_setup.numsubsectors {
        I_Error(&format!(
            "P_CrossSubsector: ss {} with numss = {}",
            num, state.p_setup.numsubsectors
        ));
    }
    sub = state.p_setup.subsector_mut(SubsectorId(num as u32));
    count = (*sub).numlines as i32;
    seg = state.p_setup.segs.as_mut_ptr().offset((*sub).firstline as isize);
    while count != 0 {
        line = state.p_setup.line_mut((*seg).linedef);
        if !((*line).validcount == state.r_main.validcount) {
            (*line).validcount = state.r_main.validcount;
            v1 = state.p_setup.vertex_mut((*line).v1);
            v2 = state.p_setup.vertex_mut((*line).v2);
            s1 = P_DivlineSide((*v1).x, (*v1).y, &raw mut state.p_sight.strace);
            s2 = P_DivlineSide((*v2).x, (*v2).y, &raw mut state.p_sight.strace);
            if !(s1 == s2) {
                divl.x = (*v1).x;
                divl.y = (*v1).y;
                divl.dx = (*v2).x - (*v1).x;
                divl.dy = (*v2).y - (*v1).y;
                s1 = P_DivlineSide(
                    state.p_sight.strace.x,
                    state.p_sight.strace.y,
                    &raw mut divl,
                );
                s2 = P_DivlineSide(state.p_sight.t2x, state.p_sight.t2y, &raw mut divl);
                if !(s1 == s2) {
                    if (*line).backsector.is_none() {
                        return false;
                    }
                    if (*line).flags as i32 & ML_TWOSIDED == 0 {
                        return false;
                    }
                    front = state.p_setup.sector_mut((*seg).frontsector.unwrap());
                    back = state.p_setup.sector_mut((*seg).backsector.unwrap());
                    if !((*front).floorheight == (*back).floorheight
                        && (*front).ceilingheight == (*back).ceilingheight)
                    {
                        if (*front).ceilingheight < (*back).ceilingheight {
                            opentop = (*front).ceilingheight;
                        } else {
                            opentop = (*back).ceilingheight;
                        }
                        if (*front).floorheight > (*back).floorheight {
                            openbottom = (*front).floorheight;
                        } else {
                            openbottom = (*back).floorheight;
                        }
                        if openbottom >= opentop {
                            return false;
                        }
                        frac = P_InterceptVector2(&raw mut state.p_sight.strace, &raw mut divl);
                        if (*front).floorheight != (*back).floorheight {
                            slope = FixedDiv(openbottom - state.p_sight.sightzstart, frac);
                            if slope > state.p_sight.bottomslope {
                                state.p_sight.bottomslope = slope;
                            }
                        }
                        if (*front).ceilingheight != (*back).ceilingheight {
                            slope = FixedDiv(opentop - state.p_sight.sightzstart, frac);
                            if slope < state.p_sight.topslope {
                                state.p_sight.topslope = slope;
                            }
                        }
                        if state.p_sight.topslope <= state.p_sight.bottomslope {
                            return false;
                        }
                    }
                }
            }
        }
        seg = seg.offset(1);
        count -= 1;
    }
    return true;
}
pub unsafe fn P_CrossBSPNode(state: &mut GameState, mut bspnum: i32) -> bool {
    let mut bsp: *mut node_t = ::core::ptr::null_mut::<node_t>();
    let mut side: i32 = 0;
    if bspnum & NF_SUBSECTOR != 0 {
        if bspnum == -(1 as i32) {
            return P_CrossSubsector(state, 0 as i32);
        } else {
            return P_CrossSubsector(state, bspnum & !NF_SUBSECTOR);
        }
    }
    bsp = state.p_setup.nodes.as_mut_ptr().offset(bspnum as isize);
    side = P_DivlineSide(
        state.p_sight.strace.x,
        state.p_sight.strace.y,
        bsp as *mut divline_t,
    );
    if side == 2 as i32 {
        side = 0 as i32;
    }
    if !P_CrossBSPNode(state, (*bsp).children[side as usize] as i32) {
        return false;
    }
    if side == P_DivlineSide(state.p_sight.t2x, state.p_sight.t2y, bsp as *mut divline_t) {
        return true;
    }
    return P_CrossBSPNode(state, (*bsp).children[(side ^ 1 as i32) as usize] as i32);
}
pub unsafe fn P_CheckSight(
    state: &mut GameState,
    mut t1: *mut mobj_t,
    mut t2: *mut mobj_t,
) -> bool {
    let mut s1: i32 = 0;
    let mut s2: i32 = 0;
    let mut pnum: i32 = 0;
    let mut bytenum: i32 = 0;
    let mut bitnum: i32 = 0;
    s1 = state.p_setup.subsectors[(*t1).subsector.0 as usize].sector.0 as i32;
    s2 = state.p_setup.subsectors[(*t2).subsector.0 as usize].sector.0 as i32;
    pnum = s1 * state.p_setup.numsectors + s2;
    bytenum = pnum >> 3 as i32;
    bitnum = (1 as i32) << (pnum & 7 as i32);
    if state.p_setup.rejectmatrix[bytenum as usize] as i32 & bitnum != 0 {
        state.p_sight.sightcounts[0 as i32 as usize] += 1;
        return false;
    }
    state.p_sight.sightcounts[1 as i32 as usize] += 1;
    state.r_main.validcount += 1;
    state.p_sight.sightzstart = (*t1).z + (*t1).height - ((*t1).height >> 2 as i32);
    state.p_sight.topslope = (*t2).z + (*t2).height - state.p_sight.sightzstart;
    state.p_sight.bottomslope = (*t2).z - state.p_sight.sightzstart;
    state.p_sight.strace.x = (*t1).x;
    state.p_sight.strace.y = (*t1).y;
    state.p_sight.t2x = (*t2).x;
    state.p_sight.t2y = (*t2).y;
    state.p_sight.strace.dx = (*t2).x - (*t1).x;
    state.p_sight.strace.dy = (*t2).y - (*t1).y;
    return P_CrossBSPNode(state, state.p_setup.numnodes - 1 as i32);
}
