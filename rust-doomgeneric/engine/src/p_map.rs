use crate::d_player::PlayerId;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::m_argv::check_parm_with_args;
use crate::m_argv::MArgvState;
use crate::m_bbox::BBox;
use crate::m_bbox::BoxIndex;
use crate::m_fixed::fixed_div;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::m_misc::str_to_int;
use crate::m_random::p_random;
use crate::p_inter::damage_mobj;
use crate::p_inter::touch_special_thing;
use crate::p_maputl::aprox_distance;
use crate::p_maputl::block_lines_iterator;
use crate::p_maputl::block_things_iterator;
use crate::p_maputl::box_on_line_side;
use crate::p_maputl::line_opening;
use crate::p_maputl::path_traverse;
use crate::p_maputl::point_on_line_side;
use crate::p_maputl::set_thing_position;
use crate::p_maputl::unset_thing_position;
use crate::p_maputl::Intercept;
use crate::p_maputl::InterceptTarget;
use crate::p_maputl::MAPBLOCKSHIFT;
use crate::p_maputl::PT_ADDLINES;
use crate::p_maputl::PT_ADDTHINGS;
use crate::p_mobj::LineFlags;
use crate::p_mobj::MobjFlags;
use crate::p_mobj::PMobjState;
use crate::p_setup::PSetupState;
use crate::platform::DoomPlatform;

use crate::p_mobj::remove_mobj;
use crate::p_mobj::set_mobj_state;
use crate::p_mobj::spawn_blood;
use crate::p_mobj::spawn_mobj;
use crate::p_mobj::spawn_puff;
use crate::p_mobj::subst_null_mobj;
use crate::p_mobj::MobjId;
use crate::p_mobj::MobjType;
use crate::p_mobj::SlopeType;
use crate::p_mobj::StateNum;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_setup::SubsectorId;
use crate::p_sight::check_sight;
use crate::p_spec::cross_special_line;
use crate::p_spec::shoot_special_line;

use crate::p_switch::use_special_line;
use crate::r_main::point_in_subsector;
use crate::r_main::point_to_angle2;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::tables::Angle;
use crate::tables::ANG180;
use crate::tables::ANGLETOFINESHIFT;
use crate::tables::FINECOSINE;
use crate::tables::FINESINE;

pub struct PMapState {
    pub tmbbox: BBox,
    pub tmthing: Option<MobjId>,
    pub tmflags: MobjFlags,
    pub tmx: Fixed,
    pub tmy: Fixed,
    pub floatok: bool,
    pub tmfloorz: Fixed,
    pub tmceilingz: Fixed,
    pub tmdropoffz: Fixed,
    pub ceilingline: Option<LineId>,
    pub spechit: [LineId; 20],
    pub numspechit: i32,
    pub bestslidefrac: Fixed,
    pub secondslidefrac: Fixed,
    pub bestslideline: LineId,
    pub secondslideline: LineId,
    pub slidemo: Option<MobjId>,
    pub tmxmove: Fixed,
    pub tmymove: Fixed,
    pub linetarget: Option<MobjId>,
    pub shootthing: Option<MobjId>,
    pub shootz: Fixed,
    pub la_damage: i32,
    pub attackrange: Fixed,
    pub aimslope: Fixed,
    pub usething: Option<MobjId>,
    pub bombsource: Option<MobjId>,
    pub bombspot: Option<MobjId>,
    pub bombdamage: i32,
    pub crushchange: bool,
    pub nofit: bool,
    pub baseaddr: u32,
}

impl Default for PMapState {
    fn default() -> Self {
        Self::new()
    }
}

impl PMapState {
    pub const fn new() -> Self {
        Self {
            tmbbox: BBox::new([0; 4]),
            tmthing: None,
            tmflags: MobjFlags::empty(),
            tmx: 0,
            tmy: 0,
            floatok: false,
            tmfloorz: 0,
            tmceilingz: 0,
            tmdropoffz: 0,
            ceilingline: None,
            spechit: [LineId(0); 20],
            numspechit: 0,
            bestslidefrac: 0,
            secondslidefrac: 0,
            bestslideline: LineId(0),
            secondslideline: LineId(0),
            slidemo: None,
            tmxmove: 0,
            tmymove: 0,
            linetarget: None,
            shootthing: None,
            shootz: 0,
            la_damage: 0,
            attackrange: 0,
            aimslope: 0,
            usething: None,
            bombsource: None,
            bombspot: None,
            bombdamage: 0,
            crushchange: false,
            nofit: false,
            baseaddr: 0,
        }
    }
}

pub const DEH_DEFAULT_SPECIES_INFIGHTING: i32 = 0;
pub const DEH_SPECIES_INFIGHTING: i32 = DEH_DEFAULT_SPECIES_INFIGHTING;
pub const USERANGE: i32 = 64 * FRACUNIT;
pub const MAXSPECIALCROSS_ORIGINAL: i32 = 8;
pub const DEFAULT_SPECHIT_MAGIC: i32 = 0x1c09c98;
pub fn stomp_thing(state: &mut GameState, thing_id: MobjId) -> bool {
    let thing = thing_id;
    let tmthing = state.world.p_map.tmthing.unwrap();

    if !state
        .world
        .p_mobj
        .mo(thing)
        .flags
        .contains(MobjFlags::SHOOTABLE)
    {
        return true;
    }
    let blockdist: Fixed =
        state.world.p_mobj.mo(thing).radius + state.world.p_mobj.mo(tmthing).radius;
    if (state.world.p_mobj.mo(thing).x - state.world.p_map.tmx).abs() >= blockdist
        || (state.world.p_mobj.mo(thing).y - state.world.p_map.tmy).abs() >= blockdist
    {
        return true;
    }
    if thing == tmthing {
        return true;
    }
    if state.world.p_mobj.mo(tmthing).player.is_none() && state.game.g_game.gamemap != 30 {
        return false;
    }
    damage_mobj(state, thing, Some(tmthing), Some(tmthing), 10000);
    true
}
pub fn teleport_move(state: &mut GameState, thing: MobjId, x: Fixed, y: Fixed) -> bool {
    state.world.p_map.tmthing = Some(thing);
    state.world.p_map.tmflags = state.world.p_mobj.mo(thing).flags;
    state.world.p_map.tmx = x;
    state.world.p_map.tmy = y;
    state.world.p_map.tmbbox[BoxIndex::Top] = y + state.world.p_mobj.mo(thing).radius;
    state.world.p_map.tmbbox[BoxIndex::Bottom] = y - state.world.p_mobj.mo(thing).radius;
    state.world.p_map.tmbbox[BoxIndex::Right] = x + state.world.p_mobj.mo(thing).radius;
    state.world.p_map.tmbbox[BoxIndex::Left] = x - state.world.p_mobj.mo(thing).radius;
    let newsubsec: SubsectorId = point_in_subsector(&state.world.p_setup, x, y);
    state.world.p_map.ceilingline = None;
    state.world.p_map.tmdropoffz = state
        .world
        .p_setup
        .sector_mut(state.world.p_setup.subsectors[newsubsec.0 as usize].sector)
        .floorheight;
    state.world.p_map.tmfloorz = state.world.p_map.tmdropoffz;
    state.world.p_map.tmceilingz = state
        .world
        .p_setup
        .sector_mut(state.world.p_setup.subsectors[newsubsec.0 as usize].sector)
        .ceilingheight;
    state.world.p_setup.validcount += 1;
    state.world.p_map.numspechit = 0;
    let xl: i32 =
        (state.world.p_map.tmbbox[BoxIndex::Left] - state.world.p_setup.bmaporgx - 32 * FRACUNIT)
            >> MAPBLOCKSHIFT;
    let xh: i32 = (state.world.p_map.tmbbox[BoxIndex::Right] - state.world.p_setup.bmaporgx
        + 32 * FRACUNIT)
        >> MAPBLOCKSHIFT;
    let yl: i32 =
        (state.world.p_map.tmbbox[BoxIndex::Bottom] - state.world.p_setup.bmaporgy - 32 * FRACUNIT)
            >> MAPBLOCKSHIFT;
    let yh: i32 = (state.world.p_map.tmbbox[BoxIndex::Top] - state.world.p_setup.bmaporgy
        + 32 * FRACUNIT)
        >> MAPBLOCKSHIFT;
    for bx in xl..=xh {
        for by in yl..=yh {
            if !block_things_iterator(state, bx, by, stomp_thing) {
                return false;
            }
        }
    }
    unset_thing_position(&mut state.world.p_mobj, &mut state.world.p_setup, thing);
    state.world.p_mobj.mo_mut(thing).floorz = state.world.p_map.tmfloorz;
    state.world.p_mobj.mo_mut(thing).ceilingz = state.world.p_map.tmceilingz;
    state.world.p_mobj.mo_mut(thing).x = x;
    state.world.p_mobj.mo_mut(thing).y = y;
    set_thing_position(&mut state.world.p_mobj, &mut state.world.p_setup, thing);
    true
}
pub fn check_line(state: &mut GameState, ld: LineId) -> bool {
    let ldv = state.world.p_setup.line(ld);
    if state.world.p_map.tmbbox[BoxIndex::Right] <= ldv.bbox[BoxIndex::Left]
        || state.world.p_map.tmbbox[BoxIndex::Left] >= ldv.bbox[BoxIndex::Right]
        || state.world.p_map.tmbbox[BoxIndex::Top] <= ldv.bbox[BoxIndex::Bottom]
        || state.world.p_map.tmbbox[BoxIndex::Bottom] >= ldv.bbox[BoxIndex::Top]
    {
        return true;
    }
    let tmbbox = state.world.p_map.tmbbox;
    if box_on_line_side(&state.world.p_setup, tmbbox, ld) != -1 {
        return true;
    }
    if ldv.backsector.is_none() {
        return false;
    }
    let tmthing = state.world.p_map.tmthing.unwrap();
    if !state
        .world
        .p_mobj
        .mo(tmthing)
        .flags
        .contains(MobjFlags::MISSILE)
    {
        if ldv.flags.contains(LineFlags::BLOCKING) {
            return false;
        }
        if state.world.p_mobj.mo(tmthing).player.is_none()
            && ldv.flags.contains(LineFlags::BLOCKMONSTERS)
        {
            return false;
        }
    }
    line_opening(&mut state.world.p_maputl, &mut state.world.p_setup, ld);
    if state.world.p_maputl.opentop < state.world.p_map.tmceilingz {
        state.world.p_map.tmceilingz = state.world.p_maputl.opentop;
        state.world.p_map.ceilingline = Some(ld);
    }
    if state.world.p_maputl.openbottom > state.world.p_map.tmfloorz {
        state.world.p_map.tmfloorz = state.world.p_maputl.openbottom;
    }
    if state.world.p_maputl.lowfloor < state.world.p_map.tmdropoffz {
        state.world.p_map.tmdropoffz = state.world.p_maputl.lowfloor;
    }
    if ldv.special != 0 {
        state.world.p_map.spechit[state.world.p_map.numspechit as usize] = ld;
        state.world.p_map.numspechit += 1;
        if state.world.p_map.numspechit > MAXSPECIALCROSS_ORIGINAL {
            spechit_overrun(
                &state.game.m_argv,
                &mut state.world.p_map,
                &mut *state.io.platform,
                ld,
            );
        }
    }
    true
}
pub fn check_thing(state: &mut GameState, thing_id: MobjId) -> bool {
    let thing = thing_id;
    let tmthing = state.world.p_map.tmthing.unwrap();

    let damage: i32;
    if !state
        .world
        .p_mobj
        .mo(thing)
        .flags
        .intersects(MobjFlags::SOLID | MobjFlags::SPECIAL | MobjFlags::SHOOTABLE)
    {
        return true;
    }
    let blockdist: Fixed =
        state.world.p_mobj.mo(thing).radius + state.world.p_mobj.mo(tmthing).radius;
    if (state.world.p_mobj.mo(thing).x - state.world.p_map.tmx).abs() >= blockdist
        || (state.world.p_mobj.mo(thing).y - state.world.p_map.tmy).abs() >= blockdist
    {
        return true;
    }
    if thing == tmthing {
        return true;
    }
    if state
        .world
        .p_mobj
        .mo(tmthing)
        .flags
        .contains(MobjFlags::SKULLFLY)
    {
        damage = (p_random(&mut state.world.m_random) % 8 + 1)
            * state
                .assets
                .info
                .mobjinfo_mut(state.world.p_mobj.mo(tmthing).kind)
                .damage;
        damage_mobj(state, thing, Some(tmthing), Some(tmthing), damage);
        state.world.p_mobj.mo_mut(tmthing).flags &= !MobjFlags::SKULLFLY;
        state.world.p_mobj.mo_mut(tmthing).momz = 0;
        state.world.p_mobj.mo_mut(tmthing).momy = state.world.p_mobj.mo(tmthing).momz;
        state.world.p_mobj.mo_mut(tmthing).momx = state.world.p_mobj.mo(tmthing).momy;
        let spawnstate = state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(tmthing).kind)
            .spawnstate;
        set_mobj_state(state, tmthing, spawnstate);
        return false;
    }
    if state
        .world
        .p_mobj
        .mo(tmthing)
        .flags
        .contains(MobjFlags::MISSILE)
    {
        if state.world.p_mobj.mo(tmthing).z
            > state.world.p_mobj.mo(thing).z + state.world.p_mobj.mo(thing).height
        {
            return true;
        }
        if state.world.p_mobj.mo(tmthing).z + state.world.p_mobj.mo(tmthing).height
            < state.world.p_mobj.mo(thing).z
        {
            return true;
        }
        let tm_target = state
            .world
            .p_mobj
            .mo(tmthing)
            .target
            .filter(|&id| state.world.p_mobj.is_live(id));
        if tm_target.is_some()
            && (state.world.p_mobj.mo(tm_target.unwrap()).kind as u32
                == state.world.p_mobj.mo(thing).kind as u32
                || state.world.p_mobj.mo(tm_target.unwrap()).kind as u32
                    == MobjType::Knight as i32 as u32
                    && state.world.p_mobj.mo(thing).kind as u32 == MobjType::Bruiser as i32 as u32
                || state.world.p_mobj.mo(tm_target.unwrap()).kind as u32
                    == MobjType::Bruiser as i32 as u32
                    && state.world.p_mobj.mo(thing).kind as u32 == MobjType::Knight as i32 as u32)
        {
            if Some(thing) == tm_target {
                return true;
            }
            if state.world.p_mobj.mo(thing).kind as u32 != MobjType::Player as i32 as u32
                && DEH_SPECIES_INFIGHTING == 0
            {
                return false;
            }
        }
        if !state
            .world
            .p_mobj
            .mo(thing)
            .flags
            .contains(MobjFlags::SHOOTABLE)
        {
            return !state
                .world
                .p_mobj
                .mo(thing)
                .flags
                .contains(MobjFlags::SOLID);
        }
        damage = (p_random(&mut state.world.m_random) % 8 + 1)
            * state
                .assets
                .info
                .mobjinfo_mut(state.world.p_mobj.mo(tmthing).kind)
                .damage;
        damage_mobj(state, thing, Some(tmthing), tm_target, damage);
        return false;
    }
    if state
        .world
        .p_mobj
        .mo(thing)
        .flags
        .contains(MobjFlags::SPECIAL)
    {
        let solid: bool = state
            .world
            .p_mobj
            .mo(thing)
            .flags
            .contains(MobjFlags::SOLID);
        if state.world.p_map.tmflags.contains(MobjFlags::PICKUP) {
            touch_special_thing(state, thing, tmthing);
        }
        return !solid;
    }
    !state
        .world
        .p_mobj
        .mo(thing)
        .flags
        .contains(MobjFlags::SOLID)
}
pub fn check_position(state: &mut GameState, thing: MobjId, x: Fixed, y: Fixed) -> bool {
    state.world.p_map.tmthing = Some(thing);
    state.world.p_map.tmflags = state.world.p_mobj.mo(thing).flags;
    state.world.p_map.tmx = x;
    state.world.p_map.tmy = y;
    state.world.p_map.tmbbox[BoxIndex::Top] = y + state.world.p_mobj.mo(thing).radius;
    state.world.p_map.tmbbox[BoxIndex::Bottom] = y - state.world.p_mobj.mo(thing).radius;
    state.world.p_map.tmbbox[BoxIndex::Right] = x + state.world.p_mobj.mo(thing).radius;
    state.world.p_map.tmbbox[BoxIndex::Left] = x - state.world.p_mobj.mo(thing).radius;
    let newsubsec: SubsectorId = point_in_subsector(&state.world.p_setup, x, y);
    state.world.p_map.ceilingline = None;
    state.world.p_map.tmdropoffz = state
        .world
        .p_setup
        .sector_mut(state.world.p_setup.subsectors[newsubsec.0 as usize].sector)
        .floorheight;
    state.world.p_map.tmfloorz = state.world.p_map.tmdropoffz;
    state.world.p_map.tmceilingz = state
        .world
        .p_setup
        .sector_mut(state.world.p_setup.subsectors[newsubsec.0 as usize].sector)
        .ceilingheight;
    state.world.p_setup.validcount += 1;
    state.world.p_map.numspechit = 0;
    if state.world.p_map.tmflags.contains(MobjFlags::NOCLIP) {
        return true;
    }
    let mut xl: i32 =
        (state.world.p_map.tmbbox[BoxIndex::Left] - state.world.p_setup.bmaporgx - 32 * FRACUNIT)
            >> MAPBLOCKSHIFT;
    let mut xh: i32 = (state.world.p_map.tmbbox[BoxIndex::Right] - state.world.p_setup.bmaporgx
        + 32 * FRACUNIT)
        >> MAPBLOCKSHIFT;
    let mut yl: i32 =
        (state.world.p_map.tmbbox[BoxIndex::Bottom] - state.world.p_setup.bmaporgy - 32 * FRACUNIT)
            >> MAPBLOCKSHIFT;
    let mut yh: i32 = (state.world.p_map.tmbbox[BoxIndex::Top] - state.world.p_setup.bmaporgy
        + 32 * FRACUNIT)
        >> MAPBLOCKSHIFT;
    for bx in xl..=xh {
        for by in yl..=yh {
            if !block_things_iterator(state, bx, by, check_thing) {
                return false;
            }
        }
    }
    xl = (state.world.p_map.tmbbox[BoxIndex::Left] - state.world.p_setup.bmaporgx) >> MAPBLOCKSHIFT;
    xh =
        (state.world.p_map.tmbbox[BoxIndex::Right] - state.world.p_setup.bmaporgx) >> MAPBLOCKSHIFT;
    yl = (state.world.p_map.tmbbox[BoxIndex::Bottom] - state.world.p_setup.bmaporgy)
        >> MAPBLOCKSHIFT;
    yh = (state.world.p_map.tmbbox[BoxIndex::Top] - state.world.p_setup.bmaporgy) >> MAPBLOCKSHIFT;
    for bx in xl..=xh {
        for by in yl..=yh {
            if !block_lines_iterator(state, bx, by, check_line) {
                return false;
            }
        }
    }
    true
}
pub fn try_move(state: &mut GameState, thing: MobjId, x: Fixed, y: Fixed) -> bool {
    state.world.p_map.floatok = false;
    if !check_position(state, thing, x, y) {
        return false;
    }
    if !state
        .world
        .p_mobj
        .mo(thing)
        .flags
        .contains(MobjFlags::NOCLIP)
    {
        if state.world.p_map.tmceilingz - state.world.p_map.tmfloorz
            < state.world.p_mobj.mo(thing).height
        {
            return false;
        }
        state.world.p_map.floatok = true;
        if !state
            .world
            .p_mobj
            .mo(thing)
            .flags
            .contains(MobjFlags::TELEPORT)
            && state.world.p_map.tmceilingz - state.world.p_mobj.mo(thing).z
                < state.world.p_mobj.mo(thing).height
        {
            return false;
        }
        if !state
            .world
            .p_mobj
            .mo(thing)
            .flags
            .contains(MobjFlags::TELEPORT)
            && state.world.p_map.tmfloorz - state.world.p_mobj.mo(thing).z > 24 * FRACUNIT
        {
            return false;
        }
        if !state
            .world
            .p_mobj
            .mo(thing)
            .flags
            .intersects(MobjFlags::DROPOFF | MobjFlags::FLOAT)
            && state.world.p_map.tmfloorz - state.world.p_map.tmdropoffz > 24 * FRACUNIT
        {
            return false;
        }
    }
    unset_thing_position(&mut state.world.p_mobj, &mut state.world.p_setup, thing);
    let oldx: Fixed = state.world.p_mobj.mo(thing).x;
    let oldy: Fixed = state.world.p_mobj.mo(thing).y;
    state.world.p_mobj.mo_mut(thing).floorz = state.world.p_map.tmfloorz;
    state.world.p_mobj.mo_mut(thing).ceilingz = state.world.p_map.tmceilingz;
    state.world.p_mobj.mo_mut(thing).x = x;
    state.world.p_mobj.mo_mut(thing).y = y;
    set_thing_position(&mut state.world.p_mobj, &mut state.world.p_setup, thing);
    if !state
        .world
        .p_mobj
        .mo(thing)
        .flags
        .intersects(MobjFlags::TELEPORT | MobjFlags::NOCLIP)
    {
        while state.world.p_map.numspechit > 0 {
            state.world.p_map.numspechit -= 1;
            let ld: LineId = state.world.p_map.spechit[state.world.p_map.numspechit as usize];
            let side: i32 = point_on_line_side(
                &state.world.p_setup,
                state.world.p_mobj.mo(thing).x,
                state.world.p_mobj.mo(thing).y,
                ld,
            );
            let oldside: i32 = point_on_line_side(&state.world.p_setup, oldx, oldy, ld);
            if side != oldside && state.world.p_setup.line(ld).special != 0 {
                cross_special_line(state, ld.0 as i32, oldside, thing);
            }
        }
    }
    true
}
pub fn thing_height_clip(state: &mut GameState, thing: MobjId) -> bool {
    let (z, floorz, x, y) = {
        let t = state.world.p_mobj.mo(thing);
        (t.z, t.floorz, t.x, t.y)
    };
    let onfloor = z == floorz;
    check_position(state, thing, x, y);
    let (tmfloorz, tmceilingz) = (state.world.p_map.tmfloorz, state.world.p_map.tmceilingz);
    let t = state.world.p_mobj.mo_mut(thing);
    t.floorz = tmfloorz;
    t.ceilingz = tmceilingz;
    if onfloor {
        t.z = t.floorz;
    } else if t.z + t.height > t.ceilingz {
        t.z = t.ceilingz - t.height;
    }
    if t.ceilingz - t.floorz < t.height {
        return false;
    }
    true
}
pub fn hit_slide_line(
    p_map: &mut PMapState,
    p_mobj: &PMobjState,
    p_setup: &PSetupState,
    ld: LineId,
) {
    let ldv = p_setup.line(ld);
    if ldv.slopetype == SlopeType::Horizontal {
        p_map.tmymove = 0;
        return;
    }
    if ldv.slopetype == SlopeType::Vertical {
        p_map.tmxmove = 0;
        return;
    }
    let slidemo = p_map.slidemo.unwrap();
    let side: i32 = point_on_line_side(p_setup, p_mobj.mo(slidemo).x, p_mobj.mo(slidemo).y, ld);
    let mut lineangle: Angle = point_to_angle2(0, 0, ldv.dx, ldv.dy);
    if side == 1 {
        lineangle = lineangle.wrapping_add(ANG180) as Angle as Angle;
    }
    let moveangle: Angle = point_to_angle2(0, 0, p_map.tmxmove, p_map.tmymove);
    let mut deltaangle: Angle = moveangle.wrapping_sub(lineangle);
    if deltaangle > ANG180 {
        deltaangle = deltaangle.wrapping_add(ANG180) as Angle as Angle;
    }
    lineangle >>= ANGLETOFINESHIFT;
    deltaangle >>= ANGLETOFINESHIFT;
    let movelen: Fixed = aprox_distance(p_map.tmxmove, p_map.tmymove);
    let newlen: Fixed = fixed_mul(movelen, FINECOSINE[deltaangle as usize]);
    p_map.tmxmove = fixed_mul(newlen, FINECOSINE[lineangle as usize]);
    p_map.tmymove = fixed_mul(newlen, FINESINE[lineangle as usize]);
}
pub fn slide_traverse(state: &mut GameState, intercept: Intercept) -> bool {
    let li: LineId = match intercept.target {
        InterceptTarget::Line(id) => id,
        InterceptTarget::Thing(_) => error("PTR_SlideTraverse: not a line?"),
    };
    let slidemo = state.world.p_map.slidemo.unwrap();
    if state
        .world
        .p_setup
        .line(li)
        .flags
        .contains(LineFlags::TWOSIDED)
    {
        line_opening(&mut state.world.p_maputl, &mut state.world.p_setup, li);
        if state.world.p_maputl.openrange >= state.world.p_mobj.mo(slidemo).height
            && state.world.p_maputl.opentop - state.world.p_mobj.mo(slidemo).z
                >= state.world.p_mobj.mo(slidemo).height
            && state.world.p_maputl.openbottom - state.world.p_mobj.mo(slidemo).z <= 24 * FRACUNIT
        {
            return true;
        }
    } else if point_on_line_side(
        &state.world.p_setup,
        state.world.p_mobj.mo(slidemo).x,
        state.world.p_mobj.mo(slidemo).y,
        li,
    ) != 0
    {
        return true;
    }
    if intercept.frac < state.world.p_map.bestslidefrac {
        state.world.p_map.secondslidefrac = state.world.p_map.bestslidefrac;
        state.world.p_map.secondslideline = state.world.p_map.bestslideline;
        state.world.p_map.bestslidefrac = intercept.frac;
        state.world.p_map.bestslideline = li;
    }
    false
}
pub fn slide_move(state: &mut GameState, mo: MobjId) {
    state.world.p_map.slidemo = Some(mo);
    let mut hitcount: i32 = 0;
    loop {
        hitcount += 1;
        if hitcount == 3 {
            break;
        }
        let (mx, my, momx, momy, radius) = {
            let m = state.world.p_mobj.mo(mo);
            (m.x, m.y, m.momx, m.momy, m.radius)
        };
        let (leadx, trailx) = if momx > 0 {
            (mx + radius, mx - radius)
        } else {
            (mx - radius, mx + radius)
        };
        let (leady, traily) = if momy > 0 {
            (my + radius, my - radius)
        } else {
            (my - radius, my + radius)
        };
        state.world.p_map.bestslidefrac = (FRACUNIT + 1) as Fixed;
        path_traverse(
            state,
            leadx,
            leady,
            leadx + momx,
            leady + momy,
            PT_ADDLINES,
            slide_traverse,
        );
        path_traverse(
            state,
            trailx,
            leady,
            trailx + momx,
            leady + momy,
            PT_ADDLINES,
            slide_traverse,
        );
        path_traverse(
            state,
            leadx,
            traily,
            leadx + momx,
            traily + momy,
            PT_ADDLINES,
            slide_traverse,
        );
        if state.world.p_map.bestslidefrac == FRACUNIT + 1 {
            break;
        }
        state.world.p_map.bestslidefrac -= 0x800;
        if state.world.p_map.bestslidefrac > 0 {
            let newx = fixed_mul(momx, state.world.p_map.bestslidefrac);
            let newy = fixed_mul(momy, state.world.p_map.bestslidefrac);
            if !try_move(state, mo, mx + newx, my + newy) {
                break;
            }
        }
        state.world.p_map.bestslidefrac =
            (FRACUNIT - (state.world.p_map.bestslidefrac + 0x800)) as Fixed;
        if state.world.p_map.bestslidefrac > FRACUNIT {
            state.world.p_map.bestslidefrac = FRACUNIT as Fixed;
        }
        if state.world.p_map.bestslidefrac <= 0 {
            return;
        }
        state.world.p_map.tmxmove = fixed_mul(momx, state.world.p_map.bestslidefrac);
        state.world.p_map.tmymove = fixed_mul(momy, state.world.p_map.bestslidefrac);
        let bestslideline = state.world.p_map.bestslideline;
        hit_slide_line(
            &mut state.world.p_map,
            &state.world.p_mobj,
            &state.world.p_setup,
            bestslideline,
        );
        let (tmxmove, tmymove) = (state.world.p_map.tmxmove, state.world.p_map.tmymove);
        {
            let m = state.world.p_mobj.mo_mut(mo);
            m.momx = tmxmove;
            m.momy = tmymove;
        }
        let (cx, cy) = {
            let m = state.world.p_mobj.mo(mo);
            (m.x, m.y)
        };
        if !try_move(state, mo, cx + tmxmove, cy + tmymove) {
            continue;
        }
        return;
    }
    let (cx, cy, cmomx, cmomy) = {
        let m = state.world.p_mobj.mo(mo);
        (m.x, m.y, m.momx, m.momy)
    };
    if !try_move(state, mo, cx, cy + cmomy) {
        let (cx, cy) = {
            let m = state.world.p_mobj.mo(mo);
            (m.x, m.y)
        };
        try_move(state, mo, cx + cmomx, cy);
    }
}
pub fn aim_traverse(state: &mut GameState, intercept: Intercept) -> bool {
    let dist: Fixed;
    if let InterceptTarget::Line(li) = intercept.target {
        let mut slope: Fixed;

        let liv = state.world.p_setup.line(li);
        if !liv.flags.contains(LineFlags::TWOSIDED) {
            return false;
        }
        line_opening(&mut state.world.p_maputl, &mut state.world.p_setup, li);
        if state.world.p_maputl.openbottom >= state.world.p_maputl.opentop {
            return false;
        }
        dist = fixed_mul(state.world.p_map.attackrange, intercept.frac);
        if liv.backsector.is_none()
            || state
                .world
                .p_setup
                .sector_mut(liv.frontsector.unwrap())
                .floorheight
                != state
                    .world
                    .p_setup
                    .sector_mut(liv.backsector.unwrap())
                    .floorheight
        {
            slope = fixed_div(
                state.world.p_maputl.openbottom - state.world.p_map.shootz,
                dist,
            );
            if slope > state.world.p_sight.bottomslope {
                state.world.p_sight.bottomslope = slope;
            }
        }
        if liv.backsector.is_none()
            || state
                .world
                .p_setup
                .sector_mut(liv.frontsector.unwrap())
                .ceilingheight
                != state
                    .world
                    .p_setup
                    .sector_mut(liv.backsector.unwrap())
                    .ceilingheight
        {
            slope = fixed_div(
                state.world.p_maputl.opentop - state.world.p_map.shootz,
                dist,
            );
            if slope < state.world.p_sight.topslope {
                state.world.p_sight.topslope = slope;
            }
        }
        if state.world.p_sight.topslope <= state.world.p_sight.bottomslope {
            return false;
        }
        return true;
    }
    let th = match intercept.target {
        InterceptTarget::Thing(id) => id,
        InterceptTarget::Line(_) => unreachable!(),
    };
    if Some(th) == state.world.p_map.shootthing {
        return true;
    }
    if !state
        .world
        .p_mobj
        .mo(th)
        .flags
        .contains(MobjFlags::SHOOTABLE)
    {
        return true;
    }
    dist = fixed_mul(state.world.p_map.attackrange, intercept.frac);
    let mut thingtopslope: Fixed = fixed_div(
        state.world.p_mobj.mo(th).z + state.world.p_mobj.mo(th).height - state.world.p_map.shootz,
        dist,
    );
    if thingtopslope < state.world.p_sight.bottomslope {
        return true;
    }
    let mut thingbottomslope: Fixed =
        fixed_div(state.world.p_mobj.mo(th).z - state.world.p_map.shootz, dist);
    if thingbottomslope > state.world.p_sight.topslope {
        return true;
    }
    if thingtopslope > state.world.p_sight.topslope {
        thingtopslope = state.world.p_sight.topslope;
    }
    if thingbottomslope < state.world.p_sight.bottomslope {
        thingbottomslope = state.world.p_sight.bottomslope;
    }
    state.world.p_map.aimslope = i32::midpoint(thingtopslope, thingbottomslope) as Fixed;
    state.world.p_map.linetarget = Some(th);
    false
}
pub fn shoot_traverse(state: &mut GameState, intercept: Intercept) -> bool {
    let shootthing = state.world.p_map.shootthing.unwrap();
    if let InterceptTarget::Line(li) = intercept.target {
        if state.world.p_setup.line(li).special != 0 {
            shoot_special_line(state, shootthing, li);
        }
        if state
            .world
            .p_setup
            .line(li)
            .flags
            .contains(LineFlags::TWOSIDED)
        {
            line_opening(&mut state.world.p_maputl, &mut state.world.p_setup, li);
            let dist = fixed_mul(state.world.p_map.attackrange, intercept.frac);
            // A missing back side (emulated) leaves both openings to check.
            let (check_floor, check_ceiling) = match state.world.p_setup.line(li).backsector {
                None => (true, true),
                Some(back) => {
                    let front = state.world.p_setup.line(li).frontsector.unwrap();
                    let (front_floor, front_ceiling) = {
                        let s = state.world.p_setup.sector_mut(front);
                        (s.floorheight, s.ceilingheight)
                    };
                    let (back_floor, back_ceiling) = {
                        let s = state.world.p_setup.sector_mut(back);
                        (s.floorheight, s.ceilingheight)
                    };
                    (front_floor != back_floor, front_ceiling != back_ceiling)
                }
            };
            let hits_line = (check_floor
                && fixed_div(
                    state.world.p_maputl.openbottom - state.world.p_map.shootz,
                    dist,
                ) > state.world.p_map.aimslope)
                || (check_ceiling
                    && fixed_div(
                        state.world.p_maputl.opentop - state.world.p_map.shootz,
                        dist,
                    ) < state.world.p_map.aimslope);
            if !hits_line {
                // The shot passes through.
                return true;
            }
        }
        let frac: Fixed = intercept.frac - fixed_div(4 * FRACUNIT, state.world.p_map.attackrange);
        let x: Fixed =
            state.world.p_maputl.trace.x + fixed_mul(state.world.p_maputl.trace.dx, frac);
        let y: Fixed =
            state.world.p_maputl.trace.y + fixed_mul(state.world.p_maputl.trace.dy, frac);
        let z: Fixed = state.world.p_map.shootz
            + fixed_mul(
                state.world.p_map.aimslope,
                fixed_mul(frac, state.world.p_map.attackrange),
            );
        if i32::from(
            state
                .world
                .p_setup
                .sector_mut(state.world.p_setup.line(li).frontsector.unwrap())
                .ceilingpic,
        ) == state.render.r_sky.skyflatnum
        {
            if z > state
                .world
                .p_setup
                .sector_mut(state.world.p_setup.line(li).frontsector.unwrap())
                .ceilingheight
            {
                return false;
            }
            if state.world.p_setup.line(li).backsector.is_some()
                && i32::from(
                    state
                        .world
                        .p_setup
                        .sector_mut(state.world.p_setup.line(li).backsector.unwrap())
                        .ceilingpic,
                ) == state.render.r_sky.skyflatnum
            {
                return false;
            }
        }
        spawn_puff(state, x, y, z);
        false
    } else {
        let th = match intercept.target {
            InterceptTarget::Thing(id) => id,
            InterceptTarget::Line(_) => unreachable!(),
        };
        if th == shootthing {
            return true;
        }
        if !state
            .world
            .p_mobj
            .mo(th)
            .flags
            .contains(MobjFlags::SHOOTABLE)
        {
            return true;
        }
        let dist: Fixed = fixed_mul(state.world.p_map.attackrange, intercept.frac);
        let thingtopslope: Fixed = fixed_div(
            state.world.p_mobj.mo(th).z + state.world.p_mobj.mo(th).height
                - state.world.p_map.shootz,
            dist,
        );
        if thingtopslope < state.world.p_map.aimslope {
            return true;
        }
        let thingbottomslope: Fixed =
            fixed_div(state.world.p_mobj.mo(th).z - state.world.p_map.shootz, dist);
        if thingbottomslope > state.world.p_map.aimslope {
            return true;
        }
        let frac: Fixed = intercept.frac - fixed_div(10 * FRACUNIT, state.world.p_map.attackrange);
        let x: Fixed =
            state.world.p_maputl.trace.x + fixed_mul(state.world.p_maputl.trace.dx, frac);
        let y: Fixed =
            state.world.p_maputl.trace.y + fixed_mul(state.world.p_maputl.trace.dy, frac);
        let z: Fixed = state.world.p_map.shootz
            + fixed_mul(
                state.world.p_map.aimslope,
                fixed_mul(frac, state.world.p_map.attackrange),
            );
        if state.world.p_mobj.mo(th).flags.contains(MobjFlags::NOBLOOD) {
            spawn_puff(state, x, y, z);
        } else {
            spawn_blood(state, x, y, z, state.world.p_map.la_damage);
        }
        if state.world.p_map.la_damage != 0 {
            damage_mobj(
                state,
                th,
                Some(shootthing),
                Some(shootthing),
                state.world.p_map.la_damage,
            );
        }
        false
    }
}
pub fn aim_line_attack(
    state: &mut GameState,
    t1: Option<MobjId>,
    angle: Angle,
    distance: Fixed,
) -> Fixed {
    let t1 = subst_null_mobj(&mut state.world.p_mobj, t1);
    let angle = angle >> ANGLETOFINESHIFT;
    let (x1, y1, z1, height1) = {
        let m = state.world.p_mobj.mo(t1);
        (m.x, m.y, m.z, m.height)
    };
    state.world.p_map.shootthing = Some(t1);
    let x2 = x1 + (distance >> FRACBITS) * FINECOSINE[angle as usize];
    let y2 = y1 + (distance >> FRACBITS) * FINESINE[angle as usize];
    state.world.p_map.shootz = (z1 + (height1 >> 1) + 8 * FRACUNIT) as Fixed;
    state.world.p_sight.topslope = (100 * FRACUNIT / 160) as Fixed;
    state.world.p_sight.bottomslope = (-100 * FRACUNIT / 160) as Fixed;
    state.world.p_map.attackrange = distance;
    state.world.p_map.linetarget = None;
    path_traverse(
        state,
        x1,
        y1,
        x2,
        y2,
        PT_ADDLINES | PT_ADDTHINGS,
        aim_traverse,
    );
    if state.world.p_map.linetarget.is_some() {
        return state.world.p_map.aimslope;
    }
    0
}
pub fn line_attack(
    state: &mut GameState,
    t1: MobjId,
    angle: Angle,
    distance: Fixed,
    slope: Fixed,
    damage: i32,
) {
    let angle = angle >> ANGLETOFINESHIFT;
    let (x1, y1, z1, height1) = {
        let m = state.world.p_mobj.mo(t1);
        (m.x, m.y, m.z, m.height)
    };
    state.world.p_map.shootthing = Some(t1);
    state.world.p_map.la_damage = damage;
    let x2 = x1 + (distance >> FRACBITS) * FINECOSINE[angle as usize];
    let y2 = y1 + (distance >> FRACBITS) * FINESINE[angle as usize];
    state.world.p_map.shootz = (z1 + (height1 >> 1) + 8 * FRACUNIT) as Fixed;
    state.world.p_map.attackrange = distance;
    state.world.p_map.aimslope = slope;
    path_traverse(
        state,
        x1,
        y1,
        x2,
        y2,
        PT_ADDLINES | PT_ADDTHINGS,
        shoot_traverse,
    );
}
pub fn use_traverse(state: &mut GameState, intercept: Intercept) -> bool {
    let li = match intercept.target {
        InterceptTarget::Line(id) => id,
        InterceptTarget::Thing(_) => unreachable!(),
    };
    let usething = state.world.p_map.usething.unwrap();
    if state.world.p_setup.line(li).special == 0 {
        line_opening(&mut state.world.p_maputl, &mut state.world.p_setup, li);
        if state.world.p_maputl.openrange <= 0 {
            s_start_sound(state, SoundOrigin::Mobj(usething), SfxName::Noway);
            return false;
        }
        return true;
    }
    let mut side: i32 = 0;
    let (use_x, use_y) = {
        let u = state.world.p_mobj.mo(usething);
        (u.x, u.y)
    };
    if point_on_line_side(&state.world.p_setup, use_x, use_y, li) == 1 {
        side = 1;
    }
    use_special_line(state, usething, li, side);
    false
}
pub fn use_lines(state: &mut GameState, player: PlayerId) {
    let player_mo = state.game.g_game.player_mut(player).mo;
    state.world.p_map.usething = player_mo;
    let player_mo = player_mo.unwrap();
    let (angle, x1, y1) = {
        let m = state.world.p_mobj.mo(player_mo);
        ((m.angle >> ANGLETOFINESHIFT) as i32, m.x, m.y)
    };
    let x2 = x1 + (USERANGE >> FRACBITS) * FINECOSINE[angle as usize];
    let y2 = y1 + (USERANGE >> FRACBITS) * FINESINE[angle as usize];
    path_traverse(state, x1, y1, x2, y2, PT_ADDLINES, use_traverse);
}
pub fn pit_radius_attack(state: &mut GameState, thing_id: MobjId) -> bool {
    let thing = thing_id;

    if !state
        .world
        .p_mobj
        .mo(thing)
        .flags
        .contains(MobjFlags::SHOOTABLE)
    {
        return true;
    }
    if state.world.p_mobj.mo(thing).kind as u32 == MobjType::Cyborg as i32 as u32
        || state.world.p_mobj.mo(thing).kind as u32 == MobjType::Spider as i32 as u32
    {
        return true;
    }
    let bombspot = state.world.p_map.bombspot.unwrap();
    let bombsource = state
        .world
        .p_map
        .bombsource
        .filter(|&id| state.world.p_mobj.is_live(id));
    let dx: Fixed =
        (state.world.p_mobj.mo(thing).x - state.world.p_mobj.mo(bombspot).x).abs() as Fixed;
    let dy: Fixed =
        (state.world.p_mobj.mo(thing).y - state.world.p_mobj.mo(bombspot).y).abs() as Fixed;
    let mut dist: Fixed = if dx > dy { dx } else { dy };
    dist = (dist - state.world.p_mobj.mo(thing).radius) >> FRACBITS;
    if dist < 0 {
        dist = 0;
    }
    if dist >= state.world.p_map.bombdamage {
        return true;
    }
    if check_sight(state, thing, bombspot) {
        damage_mobj(
            state,
            thing,
            Some(bombspot),
            bombsource,
            state.world.p_map.bombdamage - dist,
        );
    }
    true
}
pub fn p_radius_attack(state: &mut GameState, spot: MobjId, source: Option<MobjId>, damage: i32) {
    let (spot_x, spot_y) = {
        let s = state.world.p_mobj.mo(spot);
        (s.x, s.y)
    };
    let dist: Fixed = ((damage + 32 * FRACUNIT) << FRACBITS) as Fixed;
    let yh = (spot_y + dist - state.world.p_setup.bmaporgy) >> MAPBLOCKSHIFT;
    let yl = (spot_y - dist - state.world.p_setup.bmaporgy) >> MAPBLOCKSHIFT;
    let xh = (spot_x + dist - state.world.p_setup.bmaporgx) >> MAPBLOCKSHIFT;
    let xl = (spot_x - dist - state.world.p_setup.bmaporgx) >> MAPBLOCKSHIFT;
    state.world.p_map.bombspot = Some(spot);
    state.world.p_map.bombsource = source;
    state.world.p_map.bombdamage = damage;
    for y in yl..=yh {
        for x in xl..=xh {
            block_things_iterator(state, x, y, pit_radius_attack);
        }
    }
}
pub fn pit_change_sector(state: &mut GameState, thing: MobjId) -> bool {
    if thing_height_clip(state, thing) {
        return true;
    }
    let (health, flags) = {
        let t = state.world.p_mobj.mo(thing);
        (t.health, t.flags)
    };
    if health <= 0 {
        set_mobj_state(state, thing, StateNum::Gibs);
        let t = state.world.p_mobj.mo_mut(thing);
        t.flags &= !MobjFlags::SOLID;
        t.height = 0;
        t.radius = 0;
        return true;
    }
    if flags.contains(MobjFlags::DROPPED) {
        remove_mobj(state, thing);
        return true;
    }
    if !flags.contains(MobjFlags::SHOOTABLE) {
        return true;
    }
    state.world.p_map.nofit = true;
    if state.world.p_map.crushchange && state.world.p_tick.leveltime & 3 == 0 {
        damage_mobj(state, thing, None, None, 10);
        let (x, y, z, height) = {
            let t = state.world.p_mobj.mo(thing);
            (t.x, t.y, t.z, t.height)
        };
        let mo = spawn_mobj(state, x, y, z + height / 2, MobjType::Blood);
        let momx = ((p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random))
            << 12) as Fixed;
        let momy = ((p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random))
            << 12) as Fixed;
        let m = state.world.p_mobj.mo_mut(mo);
        m.momx = momx;
        m.momy = momy;
    }
    true
}
pub fn p_change_sector(state: &mut GameState, sector: SectorId, crunch: bool) -> bool {
    state.world.p_map.nofit = false;
    state.world.p_map.crushchange = crunch;
    let blockbox = state.world.p_setup.sector_mut(sector).blockbox;
    for x in blockbox[BoxIndex::Left]..=blockbox[BoxIndex::Right] {
        for y in blockbox[BoxIndex::Bottom]..=blockbox[BoxIndex::Top] {
            block_things_iterator(state, x, y, pit_change_sector);
        }
    }
    state.world.p_map.nofit
}
fn spechit_overrun(
    m_argv: &MArgvState,
    p_map: &mut PMapState,
    platform: &mut dyn DoomPlatform,
    ld: LineId,
) {
    if p_map.baseaddr == 0 {
        if let Some(p) = check_parm_with_args(m_argv, "-spechit", 1) {
            let mut baseaddr: i32 = 0;
            str_to_int(m_argv.myargv[p + 1].as_str(), &mut baseaddr);
            p_map.baseaddr = baseaddr as u32;
        } else {
            p_map.baseaddr = DEFAULT_SPECHIT_MAGIC as u32;
        }
    }
    let addr: u32 = (i64::from(p_map.baseaddr) + i64::from(ld.0) * 0x3e) as u32;
    match p_map.numspechit {
        9..=12 => {
            p_map.tmbbox[(p_map.numspechit - 9) as usize] = addr as Fixed;
        }
        13 => {
            p_map.crushchange = addr != 0;
        }
        14 => {
            p_map.nofit = addr != 0;
        }
        _ => {
            doom_eprintln!(
                platform,
                "SpechitOverrun: Warning: unable to emulatean overrun where numspechit={}",
                p_map.numspechit,
            );
        }
    }
}
