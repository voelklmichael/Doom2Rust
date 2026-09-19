use crate::game_state::GameState;
use crate::i_system::error;
use crate::m_bbox::BoxIndex;
use crate::m_fixed::Fixed;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_setup::SegId;
use crate::p_setup::SideId;
use crate::p_setup::SubsectorId;
use crate::r_defs::DrawSeg;
use crate::r_draw::RDrawState;
use crate::r_main::point_on_side;
use crate::r_main::point_to_angle;
use crate::r_main::RMainState;
use crate::r_plane::find_plane;
use crate::r_segs::store_wall_range;
use crate::r_things::add_sprites;
use crate::tables::Angle;
use crate::tables::ANG180;
use crate::tables::ANG90;
use crate::tables::ANGLETOFINESHIFT;

pub struct RBspState {
    pub curline: SegId,
    pub sidedef: SideId,
    pub linedef: LineId,
    pub frontsector: Option<SectorId>,
    pub backsector: Option<SectorId>,
    pub drawsegs: [DrawSeg; 256],
    pub ds_p: usize,
    pub newend: usize,
    pub solidsegs: [ClipRange; 32],
}

impl Default for RBspState {
    fn default() -> Self {
        Self::new()
    }
}

impl RBspState {
    pub const fn new() -> Self {
        Self {
            curline: SegId(0),
            sidedef: SideId(0),
            linedef: LineId(0),
            frontsector: None,
            backsector: None,
            drawsegs: [DrawSeg {
                curline: SegId(0),
                x1: 0,
                x2: 0,
                scale1: 0,
                scale2: 0,
                scalestep: 0,
                silhouette: 0,
                bsilheight: 0,
                tsilheight: 0,
                sprtopclip: None,
                sprbottomclip: None,
                maskedtexturecol: None,
            }; 256],
            ds_p: 0,
            newend: 0,
            solidsegs: [ClipRange { first: 0, last: 0 }; 32],
        }
    }
}

#[derive(Copy, Clone)]
pub struct ClipRange {
    pub first: i32,
    pub last: i32,
}
pub const NF_SUBSECTOR: i32 = 0x8000;
pub fn clear_draw_segs(r_bsp: &mut RBspState) {
    r_bsp.ds_p = 0;
}
pub fn clip_solid_wall_segment(state: &mut GameState, first: i32, last: i32) {
    let mut start: usize = 0;
    while state.r_bsp.solidsegs[start].last < first - 1 {
        start += 1;
    }
    if first < state.r_bsp.solidsegs[start].first {
        if last < state.r_bsp.solidsegs[start].first - 1 {
            store_wall_range(state, first, last);
            let mut next = state.r_bsp.newend;
            state.r_bsp.newend += 1;
            while next != start {
                state.r_bsp.solidsegs[next] = state.r_bsp.solidsegs[next - 1];
                next -= 1;
            }
            state.r_bsp.solidsegs[next].first = first;
            state.r_bsp.solidsegs[next].last = last;
            return;
        }
        let start_first = state.r_bsp.solidsegs[start].first;
        store_wall_range(state, first, start_first - 1);
        state.r_bsp.solidsegs[start].first = first;
    }
    if last <= state.r_bsp.solidsegs[start].last {
        return;
    }
    let mut next = start;
    let reached_end_of_gap = loop {
        if last < state.r_bsp.solidsegs[next + 1].first - 1 {
            break true;
        }
        let (from, to) = (
            state.r_bsp.solidsegs[next].last + 1,
            state.r_bsp.solidsegs[next + 1].first - 1,
        );
        store_wall_range(state, from, to);
        next += 1;
        if last > state.r_bsp.solidsegs[next].last {
            continue;
        }
        state.r_bsp.solidsegs[start].last = state.r_bsp.solidsegs[next].last;
        break false;
    };
    if reached_end_of_gap {
        let from = state.r_bsp.solidsegs[next].last + 1;
        store_wall_range(state, from, last);
        state.r_bsp.solidsegs[start].last = last;
    }
    if next == start {
        return;
    }
    loop {
        let fresh0 = next;
        next += 1;
        if fresh0 == state.r_bsp.newend {
            break;
        }
        start += 1;
        state.r_bsp.solidsegs[start] = state.r_bsp.solidsegs[next];
    }
    state.r_bsp.newend = start + 1;
}
pub fn clip_pass_wall_segment(state: &mut GameState, first: i32, last: i32) {
    let mut start: usize = 0;
    while state.r_bsp.solidsegs[start].last < first - 1 {
        start += 1;
    }
    if first < state.r_bsp.solidsegs[start].first {
        if last < state.r_bsp.solidsegs[start].first - 1 {
            store_wall_range(state, first, last);
            return;
        }
        let start_first = state.r_bsp.solidsegs[start].first;
        store_wall_range(state, first, start_first - 1);
    }
    if last <= state.r_bsp.solidsegs[start].last {
        return;
    }
    while last >= state.r_bsp.solidsegs[start + 1].first - 1 {
        let (from, to) = (
            state.r_bsp.solidsegs[start].last + 1,
            state.r_bsp.solidsegs[start + 1].first - 1,
        );
        store_wall_range(state, from, to);
        start += 1;
        if last <= state.r_bsp.solidsegs[start].last {
            return;
        }
    }
    let from = state.r_bsp.solidsegs[start].last + 1;
    store_wall_range(state, from, last);
}
pub fn clear_clip_segs(r_bsp: &mut RBspState, r_draw: &RDrawState) {
    r_bsp.solidsegs[0].first = -0x7fffffff;
    r_bsp.solidsegs[0].last = -1;
    r_bsp.solidsegs[1].first = r_draw.viewwidth;
    r_bsp.solidsegs[1].last = 0x7fffffff;
    r_bsp.newend = 2;
}
pub fn add_line(state: &mut GameState, line: SegId) {
    let mut angle1: Angle;
    let mut angle2: Angle;

    let mut tspan: Angle;
    state.r_bsp.curline = line;
    let line_v1 = state.p_setup.vertexes[state.p_setup.seg(line).v1.0 as usize];
    let line_v2 = state.p_setup.vertexes[state.p_setup.seg(line).v2.0 as usize];
    angle1 = point_to_angle(&state.r_main, line_v1.x, line_v1.y);
    angle2 = point_to_angle(&state.r_main, line_v2.x, line_v2.y);
    let span: Angle = angle1.wrapping_sub(angle2);
    if span >= ANG180 {
        return;
    }
    state.r_segs.rw_angle1 = angle1 as i32;
    angle1 = angle1.wrapping_sub(state.r_main.viewangle);
    angle2 = angle2.wrapping_sub(state.r_main.viewangle);
    tspan = angle1.wrapping_add(state.r_main.clipangle);
    if tspan > (2 as Angle).wrapping_mul(state.r_main.clipangle) {
        tspan = tspan.wrapping_sub((2 as Angle).wrapping_mul(state.r_main.clipangle));
        if tspan >= span {
            return;
        }
        angle1 = state.r_main.clipangle;
    }
    tspan = state.r_main.clipangle.wrapping_sub(angle2);
    if tspan > (2 as Angle).wrapping_mul(state.r_main.clipangle) {
        tspan = tspan.wrapping_sub((2 as Angle).wrapping_mul(state.r_main.clipangle));
        if tspan >= span {
            return;
        }
        angle2 = state.r_main.clipangle.wrapping_neg();
    }
    angle1 = angle1.wrapping_add(ANG90 as Angle) >> ANGLETOFINESHIFT;
    angle2 = angle2.wrapping_add(ANG90 as Angle) >> ANGLETOFINESHIFT;
    let x1: i32 = state.r_main.viewangletox[angle1 as usize];
    let x2: i32 = state.r_main.viewangletox[angle2 as usize];
    if x1 == x2 {
        return;
    }
    state.r_bsp.backsector = state.p_setup.seg(line).backsector;
    if state.r_bsp.backsector.is_some()
        && !(state
            .p_setup
            .sector_mut(state.r_bsp.backsector.unwrap())
            .ceilingheight
            <= state
                .p_setup
                .sector_mut(state.r_bsp.frontsector.unwrap())
                .floorheight
            || state
                .p_setup
                .sector_mut(state.r_bsp.backsector.unwrap())
                .floorheight
                >= state
                    .p_setup
                    .sector_mut(state.r_bsp.frontsector.unwrap())
                    .ceilingheight)
    {
        if !(state
            .p_setup
            .sector_mut(state.r_bsp.backsector.unwrap())
            .ceilingheight
            != state
                .p_setup
                .sector_mut(state.r_bsp.frontsector.unwrap())
                .ceilingheight
            || state
                .p_setup
                .sector_mut(state.r_bsp.backsector.unwrap())
                .floorheight
                != state
                    .p_setup
                    .sector_mut(state.r_bsp.frontsector.unwrap())
                    .floorheight)
            && state
                .p_setup
                .sector_mut(state.r_bsp.backsector.unwrap())
                .ceilingpic as i32
                == state
                    .p_setup
                    .sector_mut(state.r_bsp.frontsector.unwrap())
                    .ceilingpic as i32
            && state
                .p_setup
                .sector_mut(state.r_bsp.backsector.unwrap())
                .floorpic as i32
                == state
                    .p_setup
                    .sector_mut(state.r_bsp.frontsector.unwrap())
                    .floorpic as i32
            && state
                .p_setup
                .sector_mut(state.r_bsp.backsector.unwrap())
                .lightlevel as i32
                == state
                    .p_setup
                    .sector_mut(state.r_bsp.frontsector.unwrap())
                    .lightlevel as i32
            && state
                .p_setup
                .side_mut(state.p_setup.seg(state.r_bsp.curline).sidedef)
                .midtexture as i32
                == 0
        {
            return;
        }
        clip_pass_wall_segment(state, x1, x2 - 1);
        return;
    }
    clip_solid_wall_segment(state, x1, x2 - 1);
}
pub static CHECKCOORD: [[i32; 4]; 12] = [
    [3, 0, 2, 1],
    [3, 0, 2, 0],
    [3, 1, 2, 0],
    [0; 4],
    [2, 0, 2, 1],
    [0, 0, 0, 0],
    [3, 1, 3, 0],
    [0; 4],
    [2, 0, 3, 1],
    [2, 1, 3, 1],
    [2, 1, 3, 0],
    [0; 4],
];
pub fn check_bbox(r_bsp: &RBspState, r_main: &RMainState, bspcoord: [Fixed; 4]) -> bool {
    let mut angle1: Angle;
    let mut angle2: Angle;

    let mut tspan: Angle;

    let mut sx2: i32;
    let boxx: i32 = if r_main.viewx <= bspcoord[BoxIndex::Left as usize] {
        0
    } else if r_main.viewx < bspcoord[BoxIndex::Right as usize] {
        1
    } else {
        2
    };
    let boxy: i32 = if r_main.viewy >= bspcoord[BoxIndex::Top as usize] {
        0
    } else if r_main.viewy > bspcoord[BoxIndex::Bottom as usize] {
        1
    } else {
        2
    };
    let boxpos: i32 = (boxy << 2) + boxx;
    if boxpos == 5 {
        return true;
    }
    let x1: Fixed = bspcoord[CHECKCOORD[boxpos as usize][0] as usize];
    let y1: Fixed = bspcoord[CHECKCOORD[boxpos as usize][1] as usize];
    let x2: Fixed = bspcoord[CHECKCOORD[boxpos as usize][2] as usize];
    let y2: Fixed = bspcoord[CHECKCOORD[boxpos as usize][3] as usize];
    angle1 = point_to_angle(r_main, x1, y1).wrapping_sub(r_main.viewangle);
    angle2 = point_to_angle(r_main, x2, y2).wrapping_sub(r_main.viewangle);
    let span: Angle = angle1.wrapping_sub(angle2);
    if span >= ANG180 {
        return true;
    }
    tspan = angle1.wrapping_add(r_main.clipangle);
    if tspan > (2 as Angle).wrapping_mul(r_main.clipangle) {
        tspan = tspan.wrapping_sub((2 as Angle).wrapping_mul(r_main.clipangle));
        if tspan >= span {
            return false;
        }
        angle1 = r_main.clipangle;
    }
    tspan = r_main.clipangle.wrapping_sub(angle2);
    if tspan > (2 as Angle).wrapping_mul(r_main.clipangle) {
        tspan = tspan.wrapping_sub((2 as Angle).wrapping_mul(r_main.clipangle));
        if tspan >= span {
            return false;
        }
        angle2 = r_main.clipangle.wrapping_neg();
    }
    angle1 = angle1.wrapping_add(ANG90 as Angle) >> ANGLETOFINESHIFT;
    angle2 = angle2.wrapping_add(ANG90 as Angle) >> ANGLETOFINESHIFT;
    let sx1: i32 = r_main.viewangletox[angle1 as usize];
    sx2 = r_main.viewangletox[angle2 as usize];
    if sx1 == sx2 {
        return false;
    }
    sx2 -= 1;
    let mut start: usize = 0;
    while r_bsp.solidsegs[start].last < sx2 {
        start += 1;
    }
    if sx1 >= r_bsp.solidsegs[start].first && sx2 <= r_bsp.solidsegs[start].last {
        return false;
    }
    true
}
pub fn r_subsector(state: &mut GameState, num: i32) {
    let mut count: i32;
    let mut line: SegId;
    if num >= state.p_setup.numsubsectors {
        error(&format!(
            "R_Subsector: ss {} with numss = {}",
            num, state.p_setup.numsubsectors
        ));
    }
    state.r_main.sscount += 1;
    let sub = state.p_setup.subsector(SubsectorId(num as u32));
    state.r_bsp.frontsector = Some(sub.sector);
    count = sub.numlines as i32;
    line = SegId(sub.firstline as u32);
    let frontsector_id = state.r_bsp.frontsector.unwrap();
    let frontsector = state.p_setup.sector_mut(frontsector_id);
    let (floorheight, floorpic, ceilingheight, ceilingpic, lightlevel) = (
        frontsector.floorheight,
        frontsector.floorpic as i32,
        frontsector.ceilingheight,
        frontsector.ceilingpic as i32,
        frontsector.lightlevel as i32,
    );
    if floorheight < state.r_main.viewz {
        state.r_plane.floorplane = Some(find_plane(
            &mut state.r_plane,
            &state.r_sky,
            floorheight,
            floorpic,
            lightlevel,
        ));
    } else {
        state.r_plane.floorplane = None;
    }
    if ceilingheight > state.r_main.viewz || ceilingpic == state.r_sky.skyflatnum {
        state.r_plane.ceilingplane = Some(find_plane(
            &mut state.r_plane,
            &state.r_sky,
            ceilingheight,
            ceilingpic,
            lightlevel,
        ));
    } else {
        state.r_plane.ceilingplane = None;
    }
    add_sprites(state, frontsector_id);
    loop {
        let fresh1 = count;
        count -= 1;
        if fresh1 == 0 {
            break;
        }
        add_line(state, line);
        line = SegId(line.0 + 1);
    }
}
pub fn render_bspnode(state: &mut GameState, bspnum: i32) {
    if bspnum & NF_SUBSECTOR != 0 {
        if bspnum == -1 {
            r_subsector(state, 0);
        } else {
            r_subsector(state, bspnum & !NF_SUBSECTOR);
        }
        return;
    }
    let bsp = state.p_setup.nodes[bspnum as usize];
    let side: i32 = point_on_side(state.r_main.viewx, state.r_main.viewy, &bsp);
    render_bspnode(state, bsp.children[side as usize] as i32);
    if check_bbox(&state.r_bsp, &state.r_main, bsp.bbox[(side ^ 1) as usize]) {
        render_bspnode(state, bsp.children[(side ^ 1) as usize] as i32);
    }
}
