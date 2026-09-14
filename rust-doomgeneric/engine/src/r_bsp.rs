use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_bbox::{BOXBOTTOM, BOXLEFT, BOXRIGHT, BOXTOP};
use crate::m_fixed::fixed_t;
use crate::p_mobj::subsector_t;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_setup::SegId;
use crate::p_setup::SideId;
use crate::p_setup::SubsectorId;
use crate::r_defs::{drawseg_s, drawseg_t, node_t, visplane_t};
use crate::r_main::R_PointOnSide;
use crate::r_main::R_PointToAngle;
use crate::r_plane::R_FindPlane;
use crate::r_segs::R_StoreWallRange;
use crate::r_things::R_AddSprites;
use crate::tables::angle_t;
use crate::tables::ANG180;
use crate::tables::ANG90;
use crate::tables::ANGLETOFINESHIFT;

pub struct RBspState {
    pub curline: SegId,
    pub sidedef: SideId,
    pub linedef: LineId,
    pub frontsector: Option<SectorId>,
    pub backsector: Option<SectorId>,
    pub drawsegs: [drawseg_t; 256],
    pub ds_p: *mut drawseg_t,
    pub newend: *mut cliprange_t,
    pub solidsegs: [cliprange_t; 32],
}

impl RBspState {
    pub const fn new() -> Self {
        RBspState {
            curline: SegId(0),
            sidedef: SideId(0),
            linedef: LineId(0),
            frontsector: None,
            backsector: None,
            drawsegs: [drawseg_s {
                curline: SegId(0),
                x1: 0,
                x2: 0,
                scale1: 0,
                scale2: 0,
                scalestep: 0,
                silhouette: 0,
                bsilheight: 0,
                tsilheight: 0,
                sprtopclip: ::core::ptr::null::<i16>() as *mut i16,
                sprbottomclip: ::core::ptr::null::<i16>() as *mut i16,
                maskedtexturecol: ::core::ptr::null::<i16>() as *mut i16,
            }; 256],
            ds_p: ::core::ptr::null::<drawseg_t>() as *mut drawseg_t,
            newend: ::core::ptr::null::<cliprange_t>() as *mut cliprange_t,
            solidsegs: [cliprange_t { first: 0, last: 0 }; 32],
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct cliprange_t {
    pub first: i32,
    pub last: i32,
}
pub const NF_SUBSECTOR: i32 = 0x8000;
pub unsafe fn R_ClearDrawSegs(state: &mut GameState) {
    state.r_bsp.ds_p = &raw mut state.r_bsp.drawsegs as *mut drawseg_t;
}
pub unsafe fn R_ClipSolidWallSegment(state: &mut GameState, mut first: i32, mut last: i32) {
    let mut current_block: u64;
    let mut next: *mut cliprange_t = ::core::ptr::null_mut::<cliprange_t>();
    let mut start: *mut cliprange_t = ::core::ptr::null_mut::<cliprange_t>();
    start = &raw mut state.r_bsp.solidsegs as *mut cliprange_t;
    while (*start).last < first - 1 as i32 {
        start = start.offset(1);
    }
    if first < (*start).first {
        if last < (*start).first - 1 as i32 {
            R_StoreWallRange(state, first, last);
            next = state.r_bsp.newend;
            state.r_bsp.newend = state.r_bsp.newend.offset(1);
            while next != start {
                *next = *next.offset(-(1 as i32 as isize));
                next = next.offset(-1);
            }
            (*next).first = first;
            (*next).last = last;
            return;
        }
        R_StoreWallRange(state, first, (*start).first - 1 as i32);
        (*start).first = first;
    }
    if last <= (*start).last {
        return;
    }
    next = start;
    loop {
        if !(last >= (*next.offset(1 as i32 as isize)).first - 1 as i32) {
            current_block = 224731115979188411;
            break;
        }
        R_StoreWallRange(
            state,
            (*next).last + 1 as i32,
            (*next.offset(1 as i32 as isize)).first - 1 as i32,
        );
        next = next.offset(1);
        if !(last <= (*next).last) {
            continue;
        }
        (*start).last = (*next).last;
        current_block = 18287538169731164953;
        break;
    }
    match current_block {
        224731115979188411 => {
            R_StoreWallRange(state, (*next).last + 1 as i32, last);
            (*start).last = last;
        }
        _ => {}
    }
    if next == start {
        return;
    }
    loop {
        let fresh0 = next;
        next = next.offset(1);
        if !(fresh0 != state.r_bsp.newend) {
            break;
        }
        start = start.offset(1);
        *start = *next;
    }
    state.r_bsp.newend = start.offset(1 as i32 as isize);
}
pub unsafe fn R_ClipPassWallSegment(state: &mut GameState, mut first: i32, mut last: i32) {
    let mut start: *mut cliprange_t = ::core::ptr::null_mut::<cliprange_t>();
    start = &raw mut state.r_bsp.solidsegs as *mut cliprange_t;
    while (*start).last < first - 1 as i32 {
        start = start.offset(1);
    }
    if first < (*start).first {
        if last < (*start).first - 1 as i32 {
            R_StoreWallRange(state, first, last);
            return;
        }
        R_StoreWallRange(state, first, (*start).first - 1 as i32);
    }
    if last <= (*start).last {
        return;
    }
    while last >= (*start.offset(1 as i32 as isize)).first - 1 as i32 {
        R_StoreWallRange(
            state,
            (*start).last + 1 as i32,
            (*start.offset(1 as i32 as isize)).first - 1 as i32,
        );
        start = start.offset(1);
        if last <= (*start).last {
            return;
        }
    }
    R_StoreWallRange(state, (*start).last + 1 as i32, last);
}
pub unsafe fn R_ClearClipSegs(state: &mut GameState) {
    state.r_bsp.solidsegs[0].first = -(0x7fffffff as i32);
    state.r_bsp.solidsegs[0].last = -(1 as i32);
    state.r_bsp.solidsegs[1].first = state.r_draw.viewwidth;
    state.r_bsp.solidsegs[1].last = 0x7fffffff as i32;
    state.r_bsp.newend =
        (&raw mut state.r_bsp.solidsegs as *mut cliprange_t).offset(2 as i32 as isize);
}
pub unsafe fn R_AddLine(state: &mut GameState, mut line: SegId) {
    let mut x1: i32 = 0;
    let mut x2: i32 = 0;
    let mut angle1: angle_t = 0;
    let mut angle2: angle_t = 0;
    let mut span: angle_t = 0;
    let mut tspan: angle_t = 0;
    state.r_bsp.curline = line;
    let line_v1 = state.p_setup.vertexes[state.p_setup.seg(line).v1.0 as usize];
    let line_v2 = state.p_setup.vertexes[state.p_setup.seg(line).v2.0 as usize];
    angle1 = R_PointToAngle(state, line_v1.x, line_v1.y);
    angle2 = R_PointToAngle(state, line_v2.x, line_v2.y);
    span = angle1.wrapping_sub(angle2);
    if span >= ANG180 {
        return;
    }
    state.r_segs.rw_angle1 = angle1 as i32;
    angle1 = angle1.wrapping_sub(state.r_main.viewangle);
    angle2 = angle2.wrapping_sub(state.r_main.viewangle);
    tspan = angle1.wrapping_add(state.r_main.clipangle);
    if tspan > (2 as angle_t).wrapping_mul(state.r_main.clipangle) {
        tspan = tspan.wrapping_sub((2 as angle_t).wrapping_mul(state.r_main.clipangle));
        if tspan >= span {
            return;
        }
        angle1 = state.r_main.clipangle;
    }
    tspan = state.r_main.clipangle.wrapping_sub(angle2);
    if tspan > (2 as angle_t).wrapping_mul(state.r_main.clipangle) {
        tspan = tspan.wrapping_sub((2 as angle_t).wrapping_mul(state.r_main.clipangle));
        if tspan >= span {
            return;
        }
        angle2 = state.r_main.clipangle.wrapping_neg();
    }
    angle1 = angle1.wrapping_add(ANG90 as angle_t) >> ANGLETOFINESHIFT;
    angle2 = angle2.wrapping_add(ANG90 as angle_t) >> ANGLETOFINESHIFT;
    x1 = state.r_main.viewangletox[angle1 as usize];
    x2 = state.r_main.viewangletox[angle2 as usize];
    if x1 == x2 {
        return;
    }
    state.r_bsp.backsector = state.p_setup.seg(line).backsector;
    if !state.r_bsp.backsector.is_none() {
        if !((*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).ceilingheight
            <= (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).floorheight
            || (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).floorheight
                >= (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingheight)
        {
            if !((*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).ceilingheight
                != (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingheight
                || (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).floorheight
                    != (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).floorheight)
            {
                if (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).ceilingpic as i32
                    == (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingpic
                        as i32
                    && (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).floorpic as i32
                        == (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).floorpic
                            as i32
                    && (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).lightlevel
                        as i32
                        == (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).lightlevel
                            as i32
                    && (*state
                        .p_setup
                        .side_mut(state.p_setup.seg(state.r_bsp.curline).sidedef))
                    .midtexture as i32
                        == 0 as i32
                {
                    return;
                }
            }
            R_ClipPassWallSegment(state, x1, x2 - 1 as i32);
            return;
        }
    }
    R_ClipSolidWallSegment(state, x1, x2 - 1 as i32);
}
#[no_mangle]
pub static checkcoord: [[i32; 4]; 12] = [
    [3 as i32, 0 as i32, 2 as i32, 1 as i32],
    [3 as i32, 0 as i32, 2 as i32, 0 as i32],
    [3 as i32, 1 as i32, 2 as i32, 0 as i32],
    [0 as i32; 4],
    [2 as i32, 0 as i32, 2 as i32, 1 as i32],
    [0 as i32, 0 as i32, 0 as i32, 0 as i32],
    [3 as i32, 1 as i32, 3 as i32, 0 as i32],
    [0 as i32; 4],
    [2 as i32, 0 as i32, 3 as i32, 1 as i32],
    [2 as i32, 1 as i32, 3 as i32, 1 as i32],
    [2 as i32, 1 as i32, 3 as i32, 0 as i32],
    [0; 4],
];
pub unsafe fn R_CheckBBox(state: &mut GameState, mut bspcoord: *mut fixed_t) -> bool {
    let mut boxx: i32 = 0;
    let mut boxy: i32 = 0;
    let mut boxpos: i32 = 0;
    let mut x1: fixed_t = 0;
    let mut y1: fixed_t = 0;
    let mut x2: fixed_t = 0;
    let mut y2: fixed_t = 0;
    let mut angle1: angle_t = 0;
    let mut angle2: angle_t = 0;
    let mut span: angle_t = 0;
    let mut tspan: angle_t = 0;
    let mut start: *mut cliprange_t = ::core::ptr::null_mut::<cliprange_t>();
    let mut sx1: i32 = 0;
    let mut sx2: i32 = 0;
    if state.r_main.viewx <= *bspcoord.offset(BOXLEFT as i32 as isize) {
        boxx = 0 as i32;
    } else if state.r_main.viewx < *bspcoord.offset(BOXRIGHT as i32 as isize) {
        boxx = 1 as i32;
    } else {
        boxx = 2 as i32;
    }
    if state.r_main.viewy >= *bspcoord.offset(BOXTOP as i32 as isize) {
        boxy = 0 as i32;
    } else if state.r_main.viewy > *bspcoord.offset(BOXBOTTOM as i32 as isize) {
        boxy = 1 as i32;
    } else {
        boxy = 2 as i32;
    }
    boxpos = (boxy << 2 as i32) + boxx;
    if boxpos == 5 as i32 {
        return true;
    }
    x1 = *bspcoord.offset(checkcoord[boxpos as usize][0] as isize);
    y1 = *bspcoord.offset(checkcoord[boxpos as usize][1] as isize);
    x2 = *bspcoord.offset(checkcoord[boxpos as usize][2] as isize);
    y2 = *bspcoord.offset(checkcoord[boxpos as usize][3] as isize);
    angle1 = R_PointToAngle(state, x1, y1).wrapping_sub(state.r_main.viewangle);
    angle2 = R_PointToAngle(state, x2, y2).wrapping_sub(state.r_main.viewangle);
    span = angle1.wrapping_sub(angle2);
    if span >= ANG180 {
        return true;
    }
    tspan = angle1.wrapping_add(state.r_main.clipangle);
    if tspan > (2 as angle_t).wrapping_mul(state.r_main.clipangle) {
        tspan = tspan.wrapping_sub((2 as angle_t).wrapping_mul(state.r_main.clipangle));
        if tspan >= span {
            return false;
        }
        angle1 = state.r_main.clipangle;
    }
    tspan = state.r_main.clipangle.wrapping_sub(angle2);
    if tspan > (2 as angle_t).wrapping_mul(state.r_main.clipangle) {
        tspan = tspan.wrapping_sub((2 as angle_t).wrapping_mul(state.r_main.clipangle));
        if tspan >= span {
            return false;
        }
        angle2 = state.r_main.clipangle.wrapping_neg();
    }
    angle1 = angle1.wrapping_add(ANG90 as angle_t) >> ANGLETOFINESHIFT;
    angle2 = angle2.wrapping_add(ANG90 as angle_t) >> ANGLETOFINESHIFT;
    sx1 = state.r_main.viewangletox[angle1 as usize];
    sx2 = state.r_main.viewangletox[angle2 as usize];
    if sx1 == sx2 {
        return false;
    }
    sx2 -= 1;
    start = &raw mut state.r_bsp.solidsegs as *mut cliprange_t;
    while (*start).last < sx2 {
        start = start.offset(1);
    }
    if sx1 >= (*start).first && sx2 <= (*start).last {
        return false;
    }
    return true;
}
pub unsafe fn R_Subsector(state: &mut GameState, mut num: i32) {
    let mut count: i32 = 0;
    let mut line: SegId;
    let mut sub: *mut subsector_t = ::core::ptr::null_mut::<subsector_t>();
    if num >= state.p_setup.numsubsectors {
        I_Error(&format!(
            "R_Subsector: ss {} with numss = {}",
            num, state.p_setup.numsubsectors
        ));
    }
    state.r_main.sscount += 1;
    sub = state.p_setup.subsector_mut(SubsectorId(num as u32));
    state.r_bsp.frontsector = Some((*sub).sector);
    count = (*sub).numlines as i32;
    line = SegId((*sub).firstline as u32);
    let frontsector = state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap());
    if (*frontsector).floorheight < state.r_main.viewz {
        let (floorheight, floorpic, lightlevel) = (
            (*frontsector).floorheight,
            (*frontsector).floorpic as i32,
            (*frontsector).lightlevel as i32,
        );
        state.r_plane.floorplane = R_FindPlane(state, floorheight, floorpic, lightlevel);
    } else {
        state.r_plane.floorplane = ::core::ptr::null_mut::<visplane_t>();
    }
    if (*frontsector).ceilingheight > state.r_main.viewz
        || (*frontsector).ceilingpic as i32 == state.r_sky.skyflatnum
    {
        let (ceilingheight, ceilingpic, lightlevel) = (
            (*frontsector).ceilingheight,
            (*frontsector).ceilingpic as i32,
            (*frontsector).lightlevel as i32,
        );
        state.r_plane.ceilingplane = R_FindPlane(state, ceilingheight, ceilingpic, lightlevel);
    } else {
        state.r_plane.ceilingplane = ::core::ptr::null_mut::<visplane_t>();
    }
    R_AddSprites(state, frontsector);
    loop {
        let fresh1 = count;
        count = count - 1;
        if !(fresh1 != 0) {
            break;
        }
        R_AddLine(state, line);
        line = SegId(line.0 + 1);
    }
}
pub unsafe fn R_RenderBSPNode(state: &mut GameState, mut bspnum: i32) {
    let mut bsp: *mut node_t = ::core::ptr::null_mut::<node_t>();
    let mut side: i32 = 0;
    if bspnum & NF_SUBSECTOR != 0 {
        if bspnum == -(1 as i32) {
            R_Subsector(state, 0 as i32);
        } else {
            R_Subsector(state, bspnum & !NF_SUBSECTOR);
        }
        return;
    }
    bsp = state.p_setup.nodes.as_mut_ptr().offset(bspnum as isize);
    side = R_PointOnSide(state.r_main.viewx, state.r_main.viewy, bsp);
    R_RenderBSPNode(state, (*bsp).children[side as usize] as i32);
    if R_CheckBBox(
        state,
        &raw mut *(&raw mut (*bsp).bbox as *mut [fixed_t; 4]).offset((side ^ 1 as i32) as isize)
            as *mut fixed_t,
    ) {
        R_RenderBSPNode(state, (*bsp).children[(side ^ 1 as i32) as usize] as i32);
    }
}
