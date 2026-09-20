use crate::game_state::GameState;
use crate::i_system::error;
use crate::m_bbox::BBox;
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
                scale1: Fixed::ZERO,
                scale2: Fixed::ZERO,
                scalestep: Fixed::ZERO,
                silhouette: 0,
                bsilheight: Fixed::ZERO,
                tsilheight: Fixed::ZERO,
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
    while state.render.r_bsp.solidsegs[start].last < first - 1 {
        start += 1;
    }
    if first < state.render.r_bsp.solidsegs[start].first {
        if last < state.render.r_bsp.solidsegs[start].first - 1 {
            store_wall_range(state, first, last);
            let mut next = state.render.r_bsp.newend;
            state.render.r_bsp.newend += 1;
            while next != start {
                state.render.r_bsp.solidsegs[next] = state.render.r_bsp.solidsegs[next - 1];
                next -= 1;
            }
            state.render.r_bsp.solidsegs[next].first = first;
            state.render.r_bsp.solidsegs[next].last = last;
            return;
        }
        let start_first = state.render.r_bsp.solidsegs[start].first;
        store_wall_range(state, first, start_first - 1);
        state.render.r_bsp.solidsegs[start].first = first;
    }
    if last <= state.render.r_bsp.solidsegs[start].last {
        return;
    }
    let mut next = start;
    let reached_end_of_gap = loop {
        if last < state.render.r_bsp.solidsegs[next + 1].first - 1 {
            break true;
        }
        let (from, to) = (
            state.render.r_bsp.solidsegs[next].last + 1,
            state.render.r_bsp.solidsegs[next + 1].first - 1,
        );
        store_wall_range(state, from, to);
        next += 1;
        if last > state.render.r_bsp.solidsegs[next].last {
            continue;
        }
        state.render.r_bsp.solidsegs[start].last = state.render.r_bsp.solidsegs[next].last;
        break false;
    };
    if reached_end_of_gap {
        let from = state.render.r_bsp.solidsegs[next].last + 1;
        store_wall_range(state, from, last);
        state.render.r_bsp.solidsegs[start].last = last;
    }
    if next == start {
        return;
    }
    while next != state.render.r_bsp.newend {
        next += 1;
        start += 1;
        state.render.r_bsp.solidsegs[start] = state.render.r_bsp.solidsegs[next];
    }
    state.render.r_bsp.newend = start + 1;
}
pub fn clip_pass_wall_segment(state: &mut GameState, first: i32, last: i32) {
    let mut start: usize = 0;
    while state.render.r_bsp.solidsegs[start].last < first - 1 {
        start += 1;
    }
    if first < state.render.r_bsp.solidsegs[start].first {
        if last < state.render.r_bsp.solidsegs[start].first - 1 {
            store_wall_range(state, first, last);
            return;
        }
        let start_first = state.render.r_bsp.solidsegs[start].first;
        store_wall_range(state, first, start_first - 1);
    }
    if last <= state.render.r_bsp.solidsegs[start].last {
        return;
    }
    while last >= state.render.r_bsp.solidsegs[start + 1].first - 1 {
        let (from, to) = (
            state.render.r_bsp.solidsegs[start].last + 1,
            state.render.r_bsp.solidsegs[start + 1].first - 1,
        );
        store_wall_range(state, from, to);
        start += 1;
        if last <= state.render.r_bsp.solidsegs[start].last {
            return;
        }
    }
    let from = state.render.r_bsp.solidsegs[start].last + 1;
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
    state.render.r_bsp.curline = line;
    let line_v1 = state.world.p_setup.vertexes[state.world.p_setup.seg(line).v1.0 as usize];
    let line_v2 = state.world.p_setup.vertexes[state.world.p_setup.seg(line).v2.0 as usize];
    let mut angle1: Angle = point_to_angle(&state.render.r_main, line_v1.x, line_v1.y);
    let mut angle2: Angle = point_to_angle(&state.render.r_main, line_v2.x, line_v2.y);
    let span: Angle = angle1 - angle2;
    if span >= ANG180 {
        return;
    }
    state.render.r_segs.rw_angle1 = angle1;
    angle1 -= state.render.r_main.viewangle;
    angle2 -= state.render.r_main.viewangle;
    let mut tspan: Angle = angle1 + state.render.r_main.clipangle;
    if tspan > state.render.r_main.clipangle * 2 {
        tspan -= state.render.r_main.clipangle * 2;
        if tspan >= span {
            return;
        }
        angle1 = state.render.r_main.clipangle;
    }
    tspan = state.render.r_main.clipangle - angle2;
    if tspan > state.render.r_main.clipangle * 2 {
        tspan -= state.render.r_main.clipangle * 2;
        if tspan >= span {
            return;
        }
        angle2 = -state.render.r_main.clipangle;
    }
    let angle1 = (angle1 + ANG90).fine();
    let angle2 = (angle2 + ANG90).fine();
    let x1: i32 = state.render.r_main.viewangletox[angle1];
    let x2: i32 = state.render.r_main.viewangletox[angle2];
    if x1 == x2 {
        return;
    }
    state.render.r_bsp.backsector = state.world.p_setup.seg(line).backsector;
    if state.render.r_bsp.backsector.is_some()
        && !(state
            .world
            .p_setup
            .sector(state.render.r_bsp.back())
            .ceilingheight
            <= state
                .world
                .p_setup
                .sector(state.render.r_bsp.front())
                .floorheight
            || state
                .world
                .p_setup
                .sector(state.render.r_bsp.back())
                .floorheight
                >= state
                    .world
                    .p_setup
                    .sector(state.render.r_bsp.front())
                    .ceilingheight)
    {
        if !(state
            .world
            .p_setup
            .sector(state.render.r_bsp.back())
            .ceilingheight
            != state
                .world
                .p_setup
                .sector(state.render.r_bsp.front())
                .ceilingheight
            || state
                .world
                .p_setup
                .sector(state.render.r_bsp.back())
                .floorheight
                != state
                    .world
                    .p_setup
                    .sector(state.render.r_bsp.front())
                    .floorheight)
            && i32::from(
                state
                    .world
                    .p_setup
                    .sector(state.render.r_bsp.back())
                    .ceilingpic,
            ) == i32::from(
                state
                    .world
                    .p_setup
                    .sector(state.render.r_bsp.front())
                    .ceilingpic,
            )
            && i32::from(
                state
                    .world
                    .p_setup
                    .sector(state.render.r_bsp.back())
                    .floorpic,
            ) == i32::from(
                state
                    .world
                    .p_setup
                    .sector(state.render.r_bsp.front())
                    .floorpic,
            )
            && i32::from(
                state
                    .world
                    .p_setup
                    .sector(state.render.r_bsp.back())
                    .lightlevel,
            ) == i32::from(
                state
                    .world
                    .p_setup
                    .sector(state.render.r_bsp.front())
                    .lightlevel,
            )
            && i32::from(
                state
                    .world
                    .p_setup
                    .side_mut(state.world.p_setup.seg(state.render.r_bsp.curline).sidedef)
                    .midtexture,
            ) == 0
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
pub fn check_bbox(r_bsp: &RBspState, r_main: &RMainState, bspcoord: BBox) -> bool {
    let boxx: i32 = if r_main.viewx <= bspcoord[BoxIndex::Left] {
        0
    } else if r_main.viewx < bspcoord[BoxIndex::Right] {
        1
    } else {
        2
    };
    let boxy: i32 = if r_main.viewy >= bspcoord[BoxIndex::Top] {
        0
    } else if r_main.viewy > bspcoord[BoxIndex::Bottom] {
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
    let mut angle1: Angle = point_to_angle(r_main, x1, y1) - r_main.viewangle;
    let mut angle2: Angle = point_to_angle(r_main, x2, y2) - r_main.viewangle;
    let span: Angle = angle1 - angle2;
    if span >= ANG180 {
        return true;
    }
    let mut tspan: Angle = angle1 + r_main.clipangle;
    if tspan > r_main.clipangle * 2 {
        tspan -= r_main.clipangle * 2;
        if tspan >= span {
            return false;
        }
        angle1 = r_main.clipangle;
    }
    tspan = r_main.clipangle - angle2;
    if tspan > r_main.clipangle * 2 {
        tspan -= r_main.clipangle * 2;
        if tspan >= span {
            return false;
        }
        angle2 = -r_main.clipangle;
    }
    let angle1 = (angle1 + ANG90).fine();
    let angle2 = (angle2 + ANG90).fine();
    let sx1: i32 = r_main.viewangletox[angle1];
    let mut sx2: i32 = r_main.viewangletox[angle2];
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
    if num >= state.world.p_setup.numsubsectors {
        error(&format!(
            "R_Subsector: ss {} with numss = {}",
            num, state.world.p_setup.numsubsectors
        ));
    }
    state.render.r_main.sscount += 1;
    let sub = state.world.p_setup.subsector(SubsectorId(num as u32));
    state.render.r_bsp.frontsector = Some(sub.sector);
    let count: i32 = i32::from(sub.numlines);
    let mut line: SegId = SegId(sub.firstline as u32);
    let frontsector_id = state.render.r_bsp.front();
    let frontsector = state.world.p_setup.sector(frontsector_id);
    let (floorheight, floorpic, ceilingheight, ceilingpic, lightlevel) = (
        frontsector.floorheight,
        i32::from(frontsector.floorpic),
        frontsector.ceilingheight,
        i32::from(frontsector.ceilingpic),
        i32::from(frontsector.lightlevel),
    );
    if floorheight < state.render.r_main.viewz {
        state.render.r_plane.floorplane = Some(find_plane(
            &mut state.render.r_plane,
            &state.render.r_sky,
            floorheight,
            floorpic,
            lightlevel,
        ));
    } else {
        state.render.r_plane.floorplane = None;
    }
    if ceilingheight > state.render.r_main.viewz || ceilingpic == state.render.r_sky.skyflatnum {
        state.render.r_plane.ceilingplane = Some(find_plane(
            &mut state.render.r_plane,
            &state.render.r_sky,
            ceilingheight,
            ceilingpic,
            lightlevel,
        ));
    } else {
        state.render.r_plane.ceilingplane = None;
    }
    add_sprites(state, frontsector_id);
    for _ in 0..count {
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
    let bsp = state.world.p_setup.nodes[bspnum as usize];
    let side: i32 = point_on_side(state.render.r_main.viewx, state.render.r_main.viewy, &bsp);
    render_bspnode(state, i32::from(bsp.children[side as usize]));
    if check_bbox(
        &state.render.r_bsp,
        &state.render.r_main,
        bsp.bbox[(side ^ 1) as usize],
    ) {
        render_bspnode(state, i32::from(bsp.children[(side ^ 1) as usize]));
    }
}

impl RBspState {
    /// The sector in front of the line being processed (always set while walls are drawn).
    pub fn front(&self) -> SectorId {
        self.frontsector.expect("no current front sector")
    }

    /// The sector behind the line being processed (set for two-sided lines).
    pub fn back(&self) -> SectorId {
        self.backsector.expect("no current back sector")
    }
}
