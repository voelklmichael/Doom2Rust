use crate::game_state::GameState;
use crate::i_system::error;
use crate::m_fixed::fixed_div;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::p_maputl::DivLine;
use crate::p_mobj::LineFlags;
use crate::p_mobj::MobjId;
use crate::p_setup::SubsectorId;

use crate::r_bsp::NF_SUBSECTOR;

pub struct PSightState {
    sightzstart: Fixed,
    pub topslope: Fixed,
    pub bottomslope: Fixed,
    strace: DivLine,
    t2x: Fixed,
    t2y: Fixed,
    sightcounts: [i32; 2],
}

impl Default for PSightState {
    fn default() -> Self {
        Self::new()
    }
}

impl PSightState {
    pub const fn new() -> Self {
        Self {
            sightzstart: 0,
            topslope: 0,
            bottomslope: 0,
            strace: DivLine {
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
pub fn divline_side(x: Fixed, y: Fixed, node: &DivLine) -> i32 {
    if node.dx == 0 {
        if x == node.x {
            return 2;
        }
        if x <= node.x {
            return (node.dy > 0) as i32;
        }
        return (node.dy < 0) as i32;
    }
    if node.dy == 0 {
        if x == node.y {
            return 2;
        }
        if y <= node.y {
            return (node.dx < 0) as i32;
        }
        return (node.dx > 0) as i32;
    }
    let dx = x - node.x;
    let dy = y - node.y;
    let left = (node.dy >> FRACBITS) * (dx >> FRACBITS);
    let right = (dy >> FRACBITS) * (node.dx >> FRACBITS);
    if right < left {
        return 0;
    }
    if left == right {
        return 2;
    }
    1
}
pub fn intercept_vector2(v2: &DivLine, v1: &DivLine) -> Fixed {
    let den = fixed_mul(v1.dy >> 8, v2.dx) - fixed_mul(v1.dx >> 8, v2.dy);
    if den == 0 {
        return 0;
    }
    let num = fixed_mul((v1.x - v2.x) >> 8, v1.dy) + fixed_mul((v2.y - v1.y) >> 8, v1.dx);
    fixed_div(num, den)
}
pub fn cross_subsector(state: &mut GameState, num: i32) -> bool {
    if num >= state.p_setup.numsubsectors {
        error(&format!(
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
        if divline_side(v1.x, v1.y, &strace) == divline_side(v2.x, v2.y, &strace) {
            continue;
        }
        let divl = DivLine {
            x: v1.x,
            y: v1.y,
            dx: v2.x - v1.x,
            dy: v2.y - v1.y,
        };
        if divline_side(strace.x, strace.y, &divl) == divline_side(t2x, t2y, &divl) {
            continue;
        }
        if !has_back {
            return false;
        }
        if !flags.contains(LineFlags::TWOSIDED) {
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
        let frac = intercept_vector2(&strace, &divl);
        if front_floor != back_floor {
            let slope = fixed_div(openbottom - state.p_sight.sightzstart, frac);
            if slope > state.p_sight.bottomslope {
                state.p_sight.bottomslope = slope;
            }
        }
        if front_ceiling != back_ceiling {
            let slope = fixed_div(opentop - state.p_sight.sightzstart, frac);
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
pub fn cross_bspnode(state: &mut GameState, bspnum: i32) -> bool {
    if bspnum & NF_SUBSECTOR != 0 {
        if bspnum == -1 {
            return cross_subsector(state, 0);
        }
        return cross_subsector(state, bspnum & !NF_SUBSECTOR);
    }
    let bsp = &state.p_setup.nodes[bspnum as usize];
    let divl = DivLine {
        x: bsp.x,
        y: bsp.y,
        dx: bsp.dx,
        dy: bsp.dy,
    };
    let children = bsp.children;
    let mut side = divline_side(state.p_sight.strace.x, state.p_sight.strace.y, &divl);
    if side == 2 {
        side = 0;
    }
    if !cross_bspnode(state, children[side as usize] as i32) {
        return false;
    }
    if side == divline_side(state.p_sight.t2x, state.p_sight.t2y, &divl) {
        return true;
    }
    cross_bspnode(state, children[(side ^ 1) as usize] as i32)
}
pub fn check_sight(state: &mut GameState, t1: MobjId, t2: MobjId) -> bool {
    let (t1_subsector, t1_x, t1_y, t1_z, t1_height) = {
        let m = state.p_mobj.mo(t1);
        (m.subsector, m.x, m.y, m.z, m.height)
    };
    let (t2_subsector, t2_x, t2_y, t2_z, t2_height) = {
        let m = state.p_mobj.mo(t2);
        (m.subsector, m.x, m.y, m.z, m.height)
    };
    let s1 = state.p_setup.subsectors[t1_subsector.0 as usize].sector.0 as i32;
    let s2 = state.p_setup.subsectors[t2_subsector.0 as usize].sector.0 as i32;
    let pnum = s1 * state.p_setup.numsectors + s2;
    let bytenum = pnum >> 3;
    let bitnum = 1 << (pnum & 7);
    if state.p_setup.rejectmatrix[bytenum as usize] as i32 & bitnum != 0 {
        state.p_sight.sightcounts[0] += 1;
        return false;
    }
    state.p_sight.sightcounts[1] += 1;
    state.r_main.validcount += 1;
    state.p_sight.sightzstart = t1_z + t1_height - (t1_height >> 2);
    state.p_sight.topslope = t2_z + t2_height - state.p_sight.sightzstart;
    state.p_sight.bottomslope = t2_z - state.p_sight.sightzstart;
    state.p_sight.strace.x = t1_x;
    state.p_sight.strace.y = t1_y;
    state.p_sight.t2x = t2_x;
    state.p_sight.t2y = t2_y;
    state.p_sight.strace.dx = t2_x - t1_x;
    state.p_sight.strace.dy = t2_y - t1_y;
    cross_bspnode(state, state.p_setup.numnodes - 1)
}
