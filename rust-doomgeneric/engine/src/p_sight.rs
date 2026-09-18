use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedDiv;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FRACBITS;
use crate::p_maputl::divline_t;
use crate::p_mobj::mobj_t;
use crate::p_setup::SubsectorId;
use crate::p_spec::ML_TWOSIDED;
use crate::r_bsp::NF_SUBSECTOR;

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
pub fn P_DivlineSide(x: fixed_t, y: fixed_t, node: &divline_t) -> i32 {
    if node.dx == 0 {
        if x == node.x {
            return 2_i32;
        }
        if x <= node.x {
            return (node.dy > 0_i32) as i32;
        }
        return (node.dy < 0_i32) as i32;
    }
    if node.dy == 0 {
        if x == node.y {
            return 2_i32;
        }
        if y <= node.y {
            return (node.dx < 0_i32) as i32;
        }
        return (node.dx > 0_i32) as i32;
    }
    let dx = x - node.x;
    let dy = y - node.y;
    let left = (node.dy >> FRACBITS) * (dx >> FRACBITS);
    let right = (dy >> FRACBITS) * (node.dx >> FRACBITS);
    if right < left {
        return 0_i32;
    }
    if left == right {
        return 2_i32;
    }
    1_i32
}
pub fn P_InterceptVector2(v2: &divline_t, v1: &divline_t) -> fixed_t {
    let den = FixedMul(v1.dy >> 8_i32, v2.dx) - FixedMul(v1.dx >> 8_i32, v2.dy);
    if den == 0_i32 {
        return 0 as fixed_t;
    }
    let num = FixedMul((v1.x - v2.x) >> 8_i32, v1.dy) + FixedMul((v2.y - v1.y) >> 8_i32, v1.dx);
    FixedDiv(num, den)
}
pub fn P_CrossSubsector(state: &mut GameState, num: i32) -> bool {
    if num >= state.p_setup.numsubsectors {
        I_Error(&format!(
            "P_CrossSubsector: ss {} with numss = {}",
            num, state.p_setup.numsubsectors
        ));
    }
    let sub = state.p_setup.subsector(SubsectorId(num as u32));
    let strace = state.p_sight.strace;
    let (t2x, t2y) = (state.p_sight.t2x, state.p_sight.t2y);
    let first = sub.firstline as usize;
    for seg_index in first..first + sub.numlines as usize {
        let seg = state.p_setup.segs[seg_index];
        let line = state.p_setup.line_mut(seg.linedef);
        if line.validcount == state.r_main.validcount {
            continue;
        }
        line.validcount = state.r_main.validcount;
        let (v1_id, v2_id, has_back, flags) =
            (line.v1, line.v2, line.backsector.is_some(), line.flags);
        let v1 = state.p_setup.vertex(v1_id);
        let v2 = state.p_setup.vertex(v2_id);
        if P_DivlineSide(v1.x, v1.y, &strace) == P_DivlineSide(v2.x, v2.y, &strace) {
            continue;
        }
        let divl = divline_t {
            x: v1.x,
            y: v1.y,
            dx: v2.x - v1.x,
            dy: v2.y - v1.y,
        };
        if P_DivlineSide(strace.x, strace.y, &divl) == P_DivlineSide(t2x, t2y, &divl) {
            continue;
        }
        if !has_back {
            return false;
        }
        if flags as i32 & ML_TWOSIDED == 0 {
            return false;
        }
        let (front_floor, front_ceiling) = {
            let front = state.p_setup.sector_mut(seg.frontsector.unwrap());
            (front.floorheight, front.ceilingheight)
        };
        let (back_floor, back_ceiling) = {
            let back = state.p_setup.sector_mut(seg.backsector.unwrap());
            (back.floorheight, back.ceilingheight)
        };
        if front_floor == back_floor && front_ceiling == back_ceiling {
            continue;
        }
        let opentop = front_ceiling.min(back_ceiling);
        let openbottom = front_floor.max(back_floor);
        if openbottom >= opentop {
            return false;
        }
        let frac = P_InterceptVector2(&strace, &divl);
        if front_floor != back_floor {
            let slope = FixedDiv(openbottom - state.p_sight.sightzstart, frac);
            if slope > state.p_sight.bottomslope {
                state.p_sight.bottomslope = slope;
            }
        }
        if front_ceiling != back_ceiling {
            let slope = FixedDiv(opentop - state.p_sight.sightzstart, frac);
            if slope < state.p_sight.topslope {
                state.p_sight.topslope = slope;
            }
        }
        if state.p_sight.topslope <= state.p_sight.bottomslope {
            return false;
        }
    }
    true
}
pub fn P_CrossBSPNode(state: &mut GameState, bspnum: i32) -> bool {
    if bspnum & NF_SUBSECTOR != 0 {
        if bspnum == -1_i32 {
            return P_CrossSubsector(state, 0_i32);
        } else {
            return P_CrossSubsector(state, bspnum & !NF_SUBSECTOR);
        }
    }
    let bsp = &state.p_setup.nodes[bspnum as usize];
    let divl = divline_t {
        x: bsp.x,
        y: bsp.y,
        dx: bsp.dx,
        dy: bsp.dy,
    };
    let children = bsp.children;
    let mut side = P_DivlineSide(state.p_sight.strace.x, state.p_sight.strace.y, &divl);
    if side == 2_i32 {
        side = 0_i32;
    }
    if !P_CrossBSPNode(state, children[side as usize] as i32) {
        return false;
    }
    if side == P_DivlineSide(state.p_sight.t2x, state.p_sight.t2y, &divl) {
        return true;
    }
    P_CrossBSPNode(state, children[(side ^ 1_i32) as usize] as i32)
}
pub fn P_CheckSight(state: &mut GameState, t1: &mobj_t, t2: &mobj_t) -> bool {
    let (t1_subsector, t1_x, t1_y, t1_z, t1_height) =
        (t1.subsector, t1.x, t1.y, t1.z, t1.height);
    let (t2_subsector, t2_x, t2_y, t2_z, t2_height) =
        (t2.subsector, t2.x, t2.y, t2.z, t2.height);
    let s1 = state.p_setup.subsectors[t1_subsector.0 as usize].sector.0 as i32;
    let s2 = state.p_setup.subsectors[t2_subsector.0 as usize].sector.0 as i32;
    let pnum = s1 * state.p_setup.numsectors + s2;
    let bytenum = pnum >> 3_i32;
    let bitnum = 1_i32 << (pnum & 7_i32);
    if state.p_setup.rejectmatrix[bytenum as usize] as i32 & bitnum != 0 {
        state.p_sight.sightcounts[0] += 1;
        return false;
    }
    state.p_sight.sightcounts[1] += 1;
    state.r_main.validcount += 1;
    state.p_sight.sightzstart = t1_z + t1_height - (t1_height >> 2_i32);
    state.p_sight.topslope = t2_z + t2_height - state.p_sight.sightzstart;
    state.p_sight.bottomslope = t2_z - state.p_sight.sightzstart;
    state.p_sight.strace.x = t1_x;
    state.p_sight.strace.y = t1_y;
    state.p_sight.t2x = t2_x;
    state.p_sight.t2y = t2_y;
    state.p_sight.strace.dx = t2_x - t1_x;
    state.p_sight.strace.dy = t2_y - t1_y;
    P_CrossBSPNode(state, state.p_setup.numnodes - 1_i32)
}
