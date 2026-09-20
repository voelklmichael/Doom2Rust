use crate::game_state::GameState;
use crate::m_bbox::BBox;
use crate::m_bbox::BoxIndex;
use crate::m_fixed::fixed_div;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::m_fixed::INT_MAX;
use crate::p_mobj::MobjFlags;
use crate::p_mobj::PMobjState;
use crate::p_pspr::PPsprState;
use crate::p_setup::PSetupState;

use crate::p_mobj::MobjId;
use crate::p_setup::LineId;
use crate::r_main::point_in_subsector;

pub struct PMaputlState {
    pub intercepts_overrun: [InterceptsOverrun; 23],
    pub opentop: Fixed,
    pub openbottom: Fixed,
    pub openrange: Fixed,
    pub lowfloor: Fixed,
    pub intercepts: [Intercept; 189],
    pub intercept_p: usize,
    pub trace: DivLine,
    pub earlyout: bool,
    pub ptflags: i32,
}

impl Default for PMaputlState {
    fn default() -> Self {
        Self::new()
    }
}

impl PMaputlState {
    pub fn new() -> Self {
        Self {
            opentop: Fixed::ZERO,
            openbottom: Fixed::ZERO,
            openrange: Fixed::ZERO,
            lowfloor: Fixed::ZERO,
            intercepts: [Intercept {
                frac: Fixed::ZERO,
                target: InterceptTarget::Line(LineId(0)),
            }; 189],
            intercept_p: 0,
            trace: DivLine {
                x: Fixed::ZERO,
                y: Fixed::ZERO,
                dx: Fixed::ZERO,
                dy: Fixed::ZERO,
            },
            earlyout: false,
            ptflags: 0,
            // Vanilla-intercepts-overrun emulation table: byte-offset ranges
            // paired with the GameState field each range aliases in vanilla's
            // stack layout. See intercepts_memory_overrun().
            intercepts_overrun: [
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::None,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::None,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::None,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::LowFloor,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::OpenBottom,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::OpenTop,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::OpenRange,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::None,
                },
                InterceptsOverrun {
                    len: 120,
                    target: OverrunTarget::None,
                },
                InterceptsOverrun {
                    len: 8,
                    target: OverrunTarget::None,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::BulletSlope,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::None,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::None,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::None,
                },
                InterceptsOverrun {
                    len: 40,
                    target: OverrunTarget::PlayerStarts,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::None,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::BmapWidth,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::None,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::BmapOrgX,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::BmapOrgY,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::None,
                },
                InterceptsOverrun {
                    len: 4,
                    target: OverrunTarget::BmapHeight,
                },
                InterceptsOverrun {
                    len: 0,
                    target: OverrunTarget::None,
                },
            ],
        }
    }
}

#[derive(Copy, Clone)]
pub struct DivLine {
    pub x: Fixed,
    pub y: Fixed,
    pub dx: Fixed,
    pub dy: Fixed,
}
#[derive(Copy, Clone)]
pub enum InterceptTarget {
    Line(LineId),
    Thing(MobjId),
}
#[derive(Copy, Clone)]
pub struct Intercept {
    pub frac: Fixed,
    pub target: InterceptTarget,
}
/// Which `GameState` field a vanilla-intercepts-overrun table entry
/// (mis)writes into -- see `intercepts_memory_overrun()`.
#[derive(Copy, Clone)]
pub enum OverrunTarget {
    None,
    LowFloor,
    OpenBottom,
    OpenTop,
    OpenRange,
    BulletSlope,
    PlayerStarts,
    BmapWidth,
    BmapOrgX,
    BmapOrgY,
    BmapHeight,
}
#[derive(Copy, Clone)]
pub struct InterceptsOverrun {
    pub len: i32,
    pub target: OverrunTarget,
}
pub const MAPBLOCKUNITS: i32 = 128;
pub const MAPBLOCKSIZE: Fixed = Fixed::from_int(MAPBLOCKUNITS);
pub const MAPBLOCKSHIFT: u32 = FRACBITS + 7;
pub const MAPBTOFRAC: u32 = MAPBLOCKSHIFT - FRACBITS;
pub const MAXINTERCEPTS_ORIGINAL: i32 = 128;
pub const PT_ADDLINES: i32 = 1;
pub const PT_ADDTHINGS: i32 = 2;
pub const PT_EARLYOUT: i32 = 4;
pub fn aprox_distance(mut dx: Fixed, mut dy: Fixed) -> Fixed {
    dx = dx.abs();
    dy = dy.abs();
    if dx < dy {
        return dx + dy - (dx >> 1);
    }
    dx + dy - (dy >> 1)
}
pub fn point_on_line_side(p_setup: &PSetupState, x: Fixed, y: Fixed, line: LineId) -> i32 {
    let line = p_setup.line(line);
    let line_v1 = p_setup.vertexes[line.v1.0 as usize];
    if line.dx == Fixed::ZERO {
        if x <= line_v1.x {
            return i32::from(line.dy > Fixed::ZERO);
        }
        return i32::from(line.dy < Fixed::ZERO);
    }
    if line.dy == Fixed::ZERO {
        if y <= line_v1.y {
            return i32::from(line.dx < Fixed::ZERO);
        }
        return i32::from(line.dx > Fixed::ZERO);
    }
    let dx: Fixed = x - line_v1.x;
    let dy: Fixed = y - line_v1.y;
    let left: Fixed = fixed_mul(line.dy >> FRACBITS, dx);
    let right: Fixed = fixed_mul(dy, line.dx >> FRACBITS);
    if right < left {
        return 0;
    }
    1
}
pub fn box_on_line_side(p_setup: &PSetupState, tmbox: BBox, ld: LineId) -> i32 {
    let mut p1: i32 = 0;
    let mut p2: i32 = 0;
    let ldv = p_setup.line(ld);
    match ldv.slopetype as u32 {
        0 => {
            let ld_v1 = p_setup.vertexes[ldv.v1.0 as usize];
            p1 = i32::from(tmbox[BoxIndex::Top] > ld_v1.y);
            p2 = i32::from(tmbox[BoxIndex::Bottom] > ld_v1.y);
            if ldv.dx < Fixed::ZERO {
                p1 ^= 1;
                p2 ^= 1;
            }
        }
        1 => {
            let ld_v1 = p_setup.vertexes[ldv.v1.0 as usize];
            p1 = i32::from(tmbox[BoxIndex::Right] < ld_v1.x);
            p2 = i32::from(tmbox[BoxIndex::Left] < ld_v1.x);
            if ldv.dy < Fixed::ZERO {
                p1 ^= 1;
                p2 ^= 1;
            }
        }
        2 => {
            p1 = point_on_line_side(p_setup, tmbox[BoxIndex::Left], tmbox[BoxIndex::Top], ld);
            p2 = point_on_line_side(p_setup, tmbox[BoxIndex::Right], tmbox[BoxIndex::Bottom], ld);
        }
        3 => {
            p1 = point_on_line_side(p_setup, tmbox[BoxIndex::Right], tmbox[BoxIndex::Top], ld);
            p2 = point_on_line_side(p_setup, tmbox[BoxIndex::Left], tmbox[BoxIndex::Bottom], ld);
        }
        _ => {}
    }
    if p1 == p2 {
        return p1;
    }
    -1
}
pub fn point_on_divline_side(x: Fixed, y: Fixed, line: &DivLine) -> i32 {
    if line.dx == Fixed::ZERO {
        if x <= line.x {
            return i32::from(line.dy > Fixed::ZERO);
        }
        return i32::from(line.dy < Fixed::ZERO);
    }
    if line.dy == Fixed::ZERO {
        if y <= line.y {
            return i32::from(line.dx < Fixed::ZERO);
        }
        return i32::from(line.dx > Fixed::ZERO);
    }
    let dx = x - line.x;
    let dy = y - line.y;
    if (line.dy ^ line.dx ^ dx ^ dy).to_bits() as u32 & 0x80000000 != 0 {
        if (line.dy ^ dx).to_bits() as u32 & 0x80000000 != 0 {
            return 1;
        }
        return 0;
    }
    let left = fixed_mul(line.dy >> 8, dx >> 8);
    let right = fixed_mul(dy >> 8, line.dx >> 8);
    if right < left {
        return 0;
    }
    1
}
pub fn make_divline(p_setup: &PSetupState, li: LineId) -> DivLine {
    let li = p_setup.line(li);
    let li_v1 = p_setup.vertexes[li.v1.0 as usize];
    DivLine {
        x: li_v1.x,
        y: li_v1.y,
        dx: li.dx,
        dy: li.dy,
    }
}
pub fn intercept_vector(v2: &DivLine, v1: &DivLine) -> Fixed {
    let den = fixed_mul(v1.dy >> 8, v2.dx) - fixed_mul(v1.dx >> 8, v2.dy);
    if den == Fixed::ZERO {
        return Fixed::ZERO;
    }
    let num = fixed_mul((v1.x - v2.x) >> 8, v1.dy) + fixed_mul((v2.y - v1.y) >> 8, v1.dx);
    fixed_div(num, den)
}
pub fn line_opening(p_maputl: &mut PMaputlState, p_setup: &mut PSetupState, linedef: LineId) {
    let linedefv = p_setup.line(linedef);
    if i32::from(linedefv.sidenum[1]) == -1 {
        p_maputl.openrange = Fixed::ZERO;
        return;
    }
    let (front_floor, front_ceiling) = {
        let front = p_setup.sector_mut(linedefv.frontsector.unwrap());
        (front.floorheight, front.ceilingheight)
    };
    let (back_floor, back_ceiling) = {
        let back = p_setup.sector_mut(linedefv.backsector.unwrap());
        (back.floorheight, back.ceilingheight)
    };
    p_maputl.opentop = front_ceiling.min(back_ceiling);
    if front_floor > back_floor {
        p_maputl.openbottom = front_floor;
        p_maputl.lowfloor = back_floor;
    } else {
        p_maputl.openbottom = back_floor;
        p_maputl.lowfloor = front_floor;
    }
    p_maputl.openrange = p_maputl.opentop - p_maputl.openbottom;
}
pub fn unset_thing_position(p_mobj: &mut PMobjState, p_setup: &mut PSetupState, thing: MobjId) {
    let (flags, snext, sprev, subsector, bnext, bprev, x, y) = {
        let t = p_mobj.mo(thing);
        (
            t.flags,
            t.snext,
            t.sprev,
            t.subsector,
            t.bnext,
            t.bprev,
            t.x,
            t.y,
        )
    };
    if !flags.contains(MobjFlags::NOSECTOR) {
        if let Some(id) = snext {
            p_mobj
                .mobj_mut(id)
                .expect("sector-list snext neighbor is always live")
                .sprev = sprev;
        }
        if let Some(id) = sprev {
            p_mobj
                .mobj_mut(id)
                .expect("sector-list sprev neighbor is always live")
                .snext = snext;
        } else {
            let sector = p_setup.subsectors[subsector.0 as usize].sector;
            p_setup.sector_mut(sector).thinglist = snext;
        }
    }
    if !flags.contains(MobjFlags::NOBLOCKMAP) {
        if let Some(id) = bnext {
            p_mobj
                .mobj_mut(id)
                .expect("blockmap-list bnext neighbor is always live")
                .bprev = bprev;
        }
        if let Some(id) = bprev {
            p_mobj
                .mobj_mut(id)
                .expect("blockmap-list bprev neighbor is always live")
                .bnext = bnext;
        } else {
            let blockx = (x - p_setup.bmaporgx).to_block();
            let blocky = (y - p_setup.bmaporgy).to_block();
            if blockx >= 0
                && blockx < (Fixed(p_setup.bmapwidth)).to_bits()
                && blocky >= 0
                && blocky < (Fixed(p_setup.bmapheight)).to_bits()
            {
                p_setup.blocklinks[(blocky * p_setup.bmapwidth + blockx) as usize] = bnext;
            }
        }
    }
}
pub fn set_thing_position(p_mobj: &mut PMobjState, p_setup: &mut PSetupState, thing: MobjId) {
    let (x, y, flags) = {
        let t = p_mobj.mo(thing);
        (t.x, t.y, t.flags)
    };
    let ss = point_in_subsector(p_setup, x, y);
    p_mobj.mo_mut(thing).subsector = ss;
    if !flags.contains(MobjFlags::NOSECTOR) {
        let sector = p_setup.subsectors[ss.0 as usize].sector;
        let old_head = p_setup.sector_mut(sector).thinglist;
        {
            let t = p_mobj.mo_mut(thing);
            t.sprev = None;
            t.snext = old_head;
        }
        if let Some(head_id) = old_head {
            p_mobj
                .mobj_mut(head_id)
                .expect("sector thinglist head is always live")
                .sprev = Some(thing);
        }
        p_setup.sector_mut(sector).thinglist = Some(thing);
    }
    if !flags.contains(MobjFlags::NOBLOCKMAP) {
        let blockx = (x - p_setup.bmaporgx).to_block();
        let blocky = (y - p_setup.bmaporgy).to_block();
        if blockx >= 0
            && blockx < (Fixed(p_setup.bmapwidth)).to_bits()
            && blocky >= 0
            && blocky < (Fixed(p_setup.bmapheight)).to_bits()
        {
            let idx = (blocky * p_setup.bmapwidth + blockx) as usize;
            let old_head = p_setup.blocklinks[idx];
            {
                let t = p_mobj.mo_mut(thing);
                t.bprev = None;
                t.bnext = old_head;
            }
            if let Some(head_id) = old_head {
                p_mobj
                    .mobj_mut(head_id)
                    .expect("blockmap-list head is always live")
                    .bprev = Some(thing);
            }
            p_setup.blocklinks[idx] = Some(thing);
        } else {
            let t = p_mobj.mo_mut(thing);
            t.bprev = None;
            t.bnext = None;
        }
    }
}
pub fn block_lines_iterator<F: FnMut(&mut GameState, LineId) -> bool>(
    state: &mut GameState,
    x: i32,
    y: i32,
    mut func: F,
) -> bool {
    if x < 0 || y < 0 || x >= state.world.p_setup.bmapwidth || y >= state.world.p_setup.bmapheight {
        return true;
    }
    let offset = y * state.world.p_setup.bmapwidth + x;
    let mut list = i32::from(state.world.p_setup.blockmaplump[(4 + offset) as usize]) as usize;
    while i32::from(state.world.p_setup.blockmaplump[list]) != -1 {
        let ld = LineId(state.world.p_setup.blockmaplump[list] as u32);
        if state.world.p_setup.line(ld).validcount != state.world.p_setup.validcount {
            state.world.p_setup.line_mut(ld).validcount = state.world.p_setup.validcount;
            if !func(state, ld) {
                return false;
            }
        }
        list += 1;
    }
    true
}
pub fn block_things_iterator<F: FnMut(&mut GameState, MobjId) -> bool>(
    state: &mut GameState,
    x: i32,
    y: i32,
    mut func: F,
) -> bool {
    if x < 0 || y < 0 || x >= state.world.p_setup.bmapwidth || y >= state.world.p_setup.bmapheight {
        return true;
    }
    let mut cursor =
        state.world.p_setup.blocklinks[(y * state.world.p_setup.bmapwidth + x) as usize];
    while let Some(id) = cursor {
        state
            .world
            .p_mobj
            .mobj_ref(id)
            .expect("blockmap-list entry is always live");
        if !func(state, id) {
            return false;
        }
        // Read after the callback on purpose (as vanilla does): the callback
        // may have removed this mobj, whose bnext is still its old successor.
        cursor = state
            .world
            .p_mobj
            .mobj_ref(id)
            .expect("blockmap-list entry survives its own callback")
            .bnext;
    }
    true
}
pub fn add_line_intercepts(state: &mut GameState, ld: LineId) -> bool {
    let ldv = state.world.p_setup.line(ld);
    let trace = state.world.p_maputl.trace;
    let (s1, s2);
    if trace.dx > FRACUNIT * 16
        || trace.dy > FRACUNIT * 16
        || trace.dx < -FRACUNIT * 16
        || trace.dy < -FRACUNIT * 16
    {
        let ld_v1 = state.world.p_setup.vertexes[ldv.v1.0 as usize];
        let ld_v2 = state.world.p_setup.vertexes[ldv.v2.0 as usize];
        s1 = point_on_divline_side(ld_v1.x, ld_v1.y, &trace);
        s2 = point_on_divline_side(ld_v2.x, ld_v2.y, &trace);
    } else {
        s1 = point_on_line_side(&state.world.p_setup, trace.x, trace.y, ld);
        s2 = point_on_line_side(
            &state.world.p_setup,
            trace.x + trace.dx,
            trace.y + trace.dy,
            ld,
        );
    }
    if s1 == s2 {
        return true;
    }
    let dl = make_divline(&state.world.p_setup, ld);
    let frac = intercept_vector(&trace, &dl);
    if frac < Fixed::ZERO {
        return true;
    }
    if state.world.p_maputl.earlyout && frac < FRACUNIT && ldv.backsector.is_none() {
        return false;
    }
    let idx = state.world.p_maputl.intercept_p;
    state.world.p_maputl.intercepts[idx].frac = frac;
    state.world.p_maputl.intercepts[idx].target = InterceptTarget::Line(ld);
    let num_intercepts = idx as i32;
    let intercept = state.world.p_maputl.intercepts[idx];
    intercepts_overrun(state, num_intercepts, intercept);
    state.world.p_maputl.intercept_p += 1;
    true
}
pub fn add_thing_intercepts(state: &mut GameState, thing_id: MobjId) -> bool {
    let (thing_x, thing_y, thing_radius) = {
        let thing = state.world.p_mobj.mobj_ref(thing_id).unwrap();
        (thing.x, thing.y, thing.radius)
    };
    let trace = state.world.p_maputl.trace;
    let tracepositive = trace.dx ^ trace.dy > Fixed::ZERO;
    let (x1, y1, x2, y2);
    if tracepositive {
        x1 = thing_x - thing_radius;
        y1 = thing_y + thing_radius;
        x2 = thing_x + thing_radius;
        y2 = thing_y - thing_radius;
    } else {
        x1 = thing_x - thing_radius;
        y1 = thing_y - thing_radius;
        x2 = thing_x + thing_radius;
        y2 = thing_y + thing_radius;
    }
    let s1 = point_on_divline_side(x1, y1, &trace);
    let s2 = point_on_divline_side(x2, y2, &trace);
    if s1 == s2 {
        return true;
    }
    let dl = DivLine {
        x: x1,
        y: y1,
        dx: x2 - x1,
        dy: y2 - y1,
    };
    let frac = intercept_vector(&trace, &dl);
    if frac < Fixed::ZERO {
        return true;
    }
    let idx = state.world.p_maputl.intercept_p;
    state.world.p_maputl.intercepts[idx].frac = frac;
    state.world.p_maputl.intercepts[idx].target = InterceptTarget::Thing(thing_id);
    let num_intercepts = idx as i32;
    let intercept = state.world.p_maputl.intercepts[idx];
    intercepts_overrun(state, num_intercepts, intercept);
    state.world.p_maputl.intercept_p += 1;
    true
}
pub fn traverse_intercepts<F: FnMut(&mut GameState, Intercept) -> bool>(
    state: &mut GameState,
    mut func: F,
    maxfrac: Fixed,
) -> bool {
    let count = state.world.p_maputl.intercept_p as i32;
    let mut in_idx = 0_usize;
    for _ in 0..count {
        let mut dist = INT_MAX;
        for scan_idx in 0..state.world.p_maputl.intercept_p {
            if state.world.p_maputl.intercepts[scan_idx].frac < Fixed(dist) {
                dist = (state.world.p_maputl.intercepts[scan_idx].frac).to_bits();
                in_idx = scan_idx;
            }
        }
        if dist > maxfrac.to_bits() {
            return true;
        }
        let intercept = state.world.p_maputl.intercepts[in_idx];
        if !func(state, intercept) {
            return false;
        }
        state.world.p_maputl.intercepts[in_idx].frac = Fixed(INT_MAX);
    }
    true
}
fn intercepts_memory_overrun(
    p_maputl: &mut PMaputlState,
    p_pspr: &mut PPsprState,
    p_setup: &mut PSetupState,
    location: i32,
    value: Fixed,
) {
    let mut i = 0;
    let mut offset = 0;
    while p_maputl.intercepts_overrun[i as usize].len != 0 {
        let entry_len = p_maputl.intercepts_overrun[i as usize].len;
        if offset + entry_len > location {
            let index = location - offset;
            match p_maputl.intercepts_overrun[i as usize].target {
                OverrunTarget::None => {}
                OverrunTarget::LowFloor => p_maputl.lowfloor = value,
                OverrunTarget::OpenBottom => p_maputl.openbottom = value,
                OverrunTarget::OpenTop => p_maputl.opentop = value,
                OverrunTarget::OpenRange => p_maputl.openrange = value,
                OverrunTarget::BulletSlope => p_pspr.bulletslope = value,
                OverrunTarget::BmapWidth => p_setup.bmapwidth = value.to_bits(),
                OverrunTarget::BmapOrgX => p_setup.bmaporgx = value,
                OverrunTarget::BmapOrgY => p_setup.bmaporgy = value,
                OverrunTarget::BmapHeight => p_setup.bmapheight = value.to_bits(),
                OverrunTarget::PlayerStarts => {
                    // `MapThing` is 5 i16 fields (10 bytes); `index` here is
                    // a 16-bit-word offset into the flattened [MapThing; 4].
                    let word = index / 2;
                    let mt_idx = (word / 5) as usize;
                    let field_idx = word % 5;
                    let lo = (value & 0xffff).to_bits() as i16;
                    let hi = (value >> 16 & 0xffff).to_bits() as i16;
                    if let Some(mt) = p_setup.playerstarts.get_mut(mt_idx) {
                        match field_idx {
                            0 => mt.x = lo,
                            1 => mt.y = lo,
                            2 => mt.angle = lo,
                            3 => mt.kind = lo,
                            4 => mt.options = lo,
                            _ => unreachable!(),
                        }
                        // Vanilla writes both 16-bit halves of `value` as one
                        // 32-bit store; mirror that by also patching the next
                        // field with the high half, when there is one.
                        let next_mt_idx = (word + 1) / 5;
                        let next_field_idx = (word + 1) % 5;
                        if let Some(next_mt) = p_setup.playerstarts.get_mut(next_mt_idx as usize) {
                            match next_field_idx {
                                0 => next_mt.x = hi,
                                1 => next_mt.y = hi,
                                2 => next_mt.angle = hi,
                                3 => next_mt.kind = hi,
                                4 => next_mt.options = hi,
                                _ => unreachable!(),
                            }
                        }
                    }
                }
            }
            break;
        }
        offset += entry_len;
        i += 1;
    }
}
fn intercepts_overrun(state: &mut GameState, num_intercepts: i32, intercept: Intercept) {
    if num_intercepts <= MAXINTERCEPTS_ORIGINAL {
        return;
    }
    let location = (num_intercepts - MAXINTERCEPTS_ORIGINAL - 1) * 12;
    // Vanilla's overrun corrupts adjacent memory with the raw in-memory
    // representation of `isaline`/`d` (a bool then a pointer-sized union);
    // since this is an index now rather than a real heap address, the value
    // plugged in here was already not byte-for-byte vanilla-compatible (this
    // build's heap addresses never matched vanilla's either) -- substituting
    // the index preserves "some plausible distinguishing value" without
    // pretending to reproduce the exact original corruption.
    let (isaline, target_value) = match intercept.target {
        InterceptTarget::Line(id) => (true, id.0 as i32),
        InterceptTarget::Thing(id) => (false, id.raw_index() as i32),
    };
    intercepts_memory_overrun(
        &mut state.world.p_maputl,
        &mut state.world.p_pspr,
        &mut state.world.p_setup,
        location,
        intercept.frac,
    );
    intercepts_memory_overrun(
        &mut state.world.p_maputl,
        &mut state.world.p_pspr,
        &mut state.world.p_setup,
        location + 4,
        Fixed(i32::from(isaline)),
    );
    intercepts_memory_overrun(
        &mut state.world.p_maputl,
        &mut state.world.p_pspr,
        &mut state.world.p_setup,
        location + 8,
        Fixed(target_value),
    );
}
pub fn path_traverse<F: FnMut(&mut GameState, Intercept) -> bool>(
    state: &mut GameState,
    mut x1: Fixed,
    mut y1: Fixed,
    mut x2: Fixed,
    mut y2: Fixed,
    flags: i32,
    trav: F,
) -> bool {
    state.world.p_maputl.earlyout = (flags & PT_EARLYOUT) != 0;
    state.world.p_setup.validcount += 1;
    state.world.p_maputl.intercept_p = 0;
    if (x1 - state.world.p_setup.bmaporgx) & (MAPBLOCKSIZE - Fixed(1)).to_bits() == Fixed::ZERO {
        x1 += FRACUNIT;
    }
    if (y1 - state.world.p_setup.bmaporgy) & (MAPBLOCKSIZE - Fixed(1)).to_bits() == Fixed::ZERO {
        y1 += FRACUNIT;
    }
    state.world.p_maputl.trace.x = x1;
    state.world.p_maputl.trace.y = y1;
    state.world.p_maputl.trace.dx = x2 - x1;
    state.world.p_maputl.trace.dy = y2 - y1;
    x1 -= state.world.p_setup.bmaporgx;
    y1 -= state.world.p_setup.bmaporgy;
    let xt1: i32 = x1.to_block();
    let yt1: i32 = y1.to_block();
    x2 -= state.world.p_setup.bmaporgx;
    y2 -= state.world.p_setup.bmaporgy;
    let xt2: i32 = x2.to_block();
    let yt2: i32 = y2.to_block();
    let (mapxstep, partial, ystep): (i32, Fixed, Fixed) = if xt2 > xt1 {
        (
            1,
            (FRACUNIT - (x1 >> MAPBTOFRAC & (FRACUNIT - Fixed(1)).to_bits())),
            fixed_div(y2 - y1, (x2 - x1).abs()),
        )
    } else if xt2 < xt1 {
        (
            -1,
            (x1 >> MAPBTOFRAC & (FRACUNIT - Fixed(1)).to_bits()),
            fixed_div(y2 - y1, (x2 - x1).abs()),
        )
    } else {
        (0, FRACUNIT, (256 * FRACUNIT))
    };
    let mut yintercept: Fixed = (y1 >> MAPBTOFRAC) + fixed_mul(partial, ystep);
    let (mapystep, partial, xstep): (i32, Fixed, Fixed) = if yt2 > yt1 {
        (
            1,
            (FRACUNIT - (y1 >> MAPBTOFRAC & (FRACUNIT - Fixed(1)).to_bits())),
            fixed_div(x2 - x1, (y2 - y1).abs()),
        )
    } else if yt2 < yt1 {
        (
            -1,
            (y1 >> MAPBTOFRAC & (FRACUNIT - Fixed(1)).to_bits()),
            fixed_div(x2 - x1, (y2 - y1).abs()),
        )
    } else {
        (0, FRACUNIT, (256 * FRACUNIT))
    };
    let mut xintercept: Fixed = (x1 >> MAPBTOFRAC) + fixed_mul(partial, xstep);
    let mut mapx: i32 = xt1;
    let mut mapy: i32 = yt1;
    for _ in 0..64 {
        if flags & PT_ADDLINES != 0 && !block_lines_iterator(state, mapx, mapy, add_line_intercepts)
        {
            return false;
        }
        if flags & PT_ADDTHINGS != 0
            && !block_things_iterator(state, mapx, mapy, add_thing_intercepts)
        {
            return false;
        }
        if mapx == xt2 && mapy == yt2 {
            break;
        }
        if yintercept >> FRACBITS == Fixed(mapy) {
            yintercept += ystep;
            mapx += mapxstep;
        } else if xintercept >> FRACBITS == Fixed(mapx) {
            xintercept += xstep;
            mapy += mapystep;
        }
    }
    traverse_intercepts(state, trav, FRACUNIT)
}
