use crate::d_player::PlayerId;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_argv::M_CheckParmWithArgs;
use crate::m_bbox::BoxIndex;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedDiv;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::m_misc::M_StrToInt;
use crate::m_random::P_Random;
use crate::p_inter::P_DamageMobj;
use crate::p_inter::P_TouchSpecialThing;
use crate::p_maputl::intercept_t;
use crate::p_maputl::InterceptTarget;
use crate::p_maputl::P_AproxDistance;
use crate::p_maputl::P_BlockLinesIterator;
use crate::p_maputl::P_BlockThingsIterator;
use crate::p_maputl::P_BoxOnLineSide;
use crate::p_maputl::P_LineOpening;
use crate::p_maputl::P_PathTraverse;
use crate::p_maputl::P_PointOnLineSide;
use crate::p_maputl::P_SetThingPosition;
use crate::p_maputl::P_UnsetThingPosition;
use crate::p_maputl::MAPBLOCKSHIFT;
use crate::p_maputl::PT_ADDLINES;
use crate::p_maputl::PT_ADDTHINGS;

use crate::p_mobj::MobjId;
use crate::p_mobj::MobjType;
use crate::p_mobj::P_RemoveMobj;
use crate::p_mobj::P_SetMobjState;
use crate::p_mobj::P_SpawnBlood;
use crate::p_mobj::P_SpawnMobj;
use crate::p_mobj::P_SpawnPuff;
use crate::p_mobj::P_SubstNullMobj;
use crate::p_mobj::SlopeType;
use crate::p_mobj::StateNum;
use crate::p_mobj::{
    MF_DROPOFF, MF_DROPPED, MF_FLOAT, MF_MISSILE, MF_NOBLOOD, MF_NOCLIP, MF_PICKUP, MF_SHOOTABLE,
    MF_SKULLFLY, MF_SOLID, MF_SPECIAL, MF_TELEPORT,
};
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_setup::SubsectorId;
use crate::p_sight::P_CheckSight;
use crate::p_spec::P_CrossSpecialLine;
use crate::p_spec::P_ShootSpecialLine;
use crate::p_spec::ML_TWOSIDED;
use crate::p_switch::P_UseSpecialLine;
use crate::r_main::R_PointInSubsector;
use crate::r_main::R_PointToAngle2;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::tables::angle_t;
use crate::tables::finecosine;
use crate::tables::finesine;
use crate::tables::ANG180;
use crate::tables::ANGLETOFINESHIFT;

pub struct PMapState {
    pub tmbbox: [fixed_t; 4],
    pub tmthing: Option<MobjId>,
    pub tmflags: i32,
    pub tmx: fixed_t,
    pub tmy: fixed_t,
    pub floatok: bool,
    pub tmfloorz: fixed_t,
    pub tmceilingz: fixed_t,
    pub tmdropoffz: fixed_t,
    pub ceilingline: Option<LineId>,
    pub spechit: [LineId; 20],
    pub numspechit: i32,
    pub bestslidefrac: fixed_t,
    pub secondslidefrac: fixed_t,
    pub bestslideline: LineId,
    pub secondslideline: LineId,
    pub slidemo: Option<MobjId>,
    pub tmxmove: fixed_t,
    pub tmymove: fixed_t,
    pub linetarget: Option<MobjId>,
    pub shootthing: Option<MobjId>,
    pub shootz: fixed_t,
    pub la_damage: i32,
    pub attackrange: fixed_t,
    pub aimslope: fixed_t,
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
        PMapState {
            tmbbox: [0; 4],
            tmthing: None,
            tmflags: 0,
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
pub const deh_species_infighting: i32 = DEH_DEFAULT_SPECIES_INFIGHTING;
pub const ML_BLOCKING: i32 = 1;
pub const ML_BLOCKMONSTERS: i32 = 2;
pub const USERANGE: i32 = 64 * FRACUNIT;
pub const MAXSPECIALCROSS_ORIGINAL: i32 = 8;
pub const DEFAULT_SPECHIT_MAGIC: i32 = 0x1c09c98;
pub fn PIT_StompThing(state: &mut GameState, thing_id: MobjId) -> bool {
    let thing = thing_id;
    let tmthing = state.p_map.tmthing.unwrap();

    if state.p_mobj.mo(thing).flags & MF_SHOOTABLE == 0 {
        return true;
    }
    let blockdist: fixed_t = state.p_mobj.mo(thing).radius + state.p_mobj.mo(tmthing).radius;
    if (state.p_mobj.mo(thing).x - state.p_map.tmx).abs() >= blockdist
        || (state.p_mobj.mo(thing).y - state.p_map.tmy).abs() >= blockdist
    {
        return true;
    }
    if thing == tmthing {
        return true;
    }
    if state.p_mobj.mo(tmthing).player.is_none() && state.g_game.gamemap != 30 {
        return false;
    }
    P_DamageMobj(state, thing, Some(tmthing), Some(tmthing), 10000);
    true
}
pub fn P_TeleportMove(state: &mut GameState, thing: MobjId, x: fixed_t, y: fixed_t) -> bool {
    state.p_map.tmthing = Some(thing);
    state.p_map.tmflags = state.p_mobj.mo(thing).flags;
    state.p_map.tmx = x;
    state.p_map.tmy = y;
    state.p_map.tmbbox[BoxIndex::Top as usize] = y + state.p_mobj.mo(thing).radius;
    state.p_map.tmbbox[BoxIndex::Bottom as usize] = y - state.p_mobj.mo(thing).radius;
    state.p_map.tmbbox[BoxIndex::Right as usize] = x + state.p_mobj.mo(thing).radius;
    state.p_map.tmbbox[BoxIndex::Left as usize] = x - state.p_mobj.mo(thing).radius;
    let newsubsec: SubsectorId = R_PointInSubsector(state, x, y);
    state.p_map.ceilingline = None;
    state.p_map.tmdropoffz = state
        .p_setup
        .sector_mut(state.p_setup.subsectors[newsubsec.0 as usize].sector)
        .floorheight;
    state.p_map.tmfloorz = state.p_map.tmdropoffz;
    state.p_map.tmceilingz = state
        .p_setup
        .sector_mut(state.p_setup.subsectors[newsubsec.0 as usize].sector)
        .ceilingheight;
    state.r_main.validcount += 1;
    state.p_map.numspechit = 0;
    let xl: i32 =
        (state.p_map.tmbbox[BoxIndex::Left as usize] - state.p_setup.bmaporgx - 32 * FRACUNIT)
            >> MAPBLOCKSHIFT;
    let xh: i32 = (state.p_map.tmbbox[BoxIndex::Right as usize] - state.p_setup.bmaporgx
        + 32 * FRACUNIT)
        >> MAPBLOCKSHIFT;
    let yl: i32 =
        (state.p_map.tmbbox[BoxIndex::Bottom as usize] - state.p_setup.bmaporgy - 32 * FRACUNIT)
            >> MAPBLOCKSHIFT;
    let yh: i32 = (state.p_map.tmbbox[BoxIndex::Top as usize] - state.p_setup.bmaporgy
        + 32 * FRACUNIT)
        >> MAPBLOCKSHIFT;
    for bx in xl..=xh {
        for by in yl..=yh {
            if !P_BlockThingsIterator(state, bx, by, PIT_StompThing) {
                return false;
            }
        }
    }
    P_UnsetThingPosition(state, thing);
    state.p_mobj.mo_mut(thing).floorz = state.p_map.tmfloorz;
    state.p_mobj.mo_mut(thing).ceilingz = state.p_map.tmceilingz;
    state.p_mobj.mo_mut(thing).x = x;
    state.p_mobj.mo_mut(thing).y = y;
    P_SetThingPosition(state, thing);
    true
}
pub fn PIT_CheckLine(state: &mut GameState, ld: LineId) -> bool {
    let ldv = state.p_setup.line(ld);
    if state.p_map.tmbbox[BoxIndex::Right as usize] <= ldv.bbox[BoxIndex::Left as usize]
        || state.p_map.tmbbox[BoxIndex::Left as usize] >= ldv.bbox[BoxIndex::Right as usize]
        || state.p_map.tmbbox[BoxIndex::Top as usize] <= ldv.bbox[BoxIndex::Bottom as usize]
        || state.p_map.tmbbox[BoxIndex::Bottom as usize] >= ldv.bbox[BoxIndex::Top as usize]
    {
        return true;
    }
    let tmbbox = state.p_map.tmbbox;
    if P_BoxOnLineSide(state, tmbbox, ld) != -1 {
        return true;
    }
    if ldv.backsector.is_none() {
        return false;
    }
    let tmthing = state.p_map.tmthing.unwrap();
    if state.p_mobj.mo(tmthing).flags & MF_MISSILE == 0 {
        if ldv.flags as i32 & ML_BLOCKING != 0 {
            return false;
        }
        if state.p_mobj.mo(tmthing).player.is_none() && ldv.flags as i32 & ML_BLOCKMONSTERS != 0 {
            return false;
        }
    }
    P_LineOpening(state, ld);
    if state.p_maputl.opentop < state.p_map.tmceilingz {
        state.p_map.tmceilingz = state.p_maputl.opentop;
        state.p_map.ceilingline = Some(ld);
    }
    if state.p_maputl.openbottom > state.p_map.tmfloorz {
        state.p_map.tmfloorz = state.p_maputl.openbottom;
    }
    if state.p_maputl.lowfloor < state.p_map.tmdropoffz {
        state.p_map.tmdropoffz = state.p_maputl.lowfloor;
    }
    if ldv.special != 0 {
        state.p_map.spechit[state.p_map.numspechit as usize] = ld;
        state.p_map.numspechit += 1;
        if state.p_map.numspechit > MAXSPECIALCROSS_ORIGINAL {
            SpechitOverrun(state, ld);
        }
    }
    true
}
pub fn PIT_CheckThing(state: &mut GameState, thing_id: MobjId) -> bool {
    let thing = thing_id;
    let tmthing = state.p_map.tmthing.unwrap();

    let solid: bool;
    let damage: i32;
    if state.p_mobj.mo(thing).flags & (MF_SOLID | MF_SPECIAL | MF_SHOOTABLE) == 0 {
        return true;
    }
    let blockdist: fixed_t = state.p_mobj.mo(thing).radius + state.p_mobj.mo(tmthing).radius;
    if (state.p_mobj.mo(thing).x - state.p_map.tmx).abs() >= blockdist
        || (state.p_mobj.mo(thing).y - state.p_map.tmy).abs() >= blockdist
    {
        return true;
    }
    if thing == tmthing {
        return true;
    }
    if state.p_mobj.mo(tmthing).flags & MF_SKULLFLY != 0 {
        damage = (P_Random(&mut state.m_random) % 8 + 1)
            * state
                .info
                .mobjinfo_mut(state.p_mobj.mo(tmthing).kind)
                .damage;
        P_DamageMobj(state, thing, Some(tmthing), Some(tmthing), damage);
        state.p_mobj.mo_mut(tmthing).flags &= !MF_SKULLFLY;
        state.p_mobj.mo_mut(tmthing).momz = 0;
        state.p_mobj.mo_mut(tmthing).momy = state.p_mobj.mo(tmthing).momz;
        state.p_mobj.mo_mut(tmthing).momx = state.p_mobj.mo(tmthing).momy;
        let spawnstate = state
            .info
            .mobjinfo_mut(state.p_mobj.mo(tmthing).kind)
            .spawnstate;
        P_SetMobjState(state, tmthing, spawnstate);
        return false;
    }
    if state.p_mobj.mo(tmthing).flags & MF_MISSILE != 0 {
        if state.p_mobj.mo(tmthing).z > state.p_mobj.mo(thing).z + state.p_mobj.mo(thing).height {
            return true;
        }
        if state.p_mobj.mo(tmthing).z + state.p_mobj.mo(tmthing).height < state.p_mobj.mo(thing).z {
            return true;
        }
        let tm_target = state
            .p_mobj
            .mo(tmthing)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        if tm_target.is_some()
            && (state.p_mobj.mo(tm_target.unwrap()).kind as u32
                == state.p_mobj.mo(thing).kind as u32
                || state.p_mobj.mo(tm_target.unwrap()).kind as u32
                    == MobjType::MT_KNIGHT as i32 as u32
                    && state.p_mobj.mo(thing).kind as u32 == MobjType::MT_BRUISER as i32 as u32
                || state.p_mobj.mo(tm_target.unwrap()).kind as u32
                    == MobjType::MT_BRUISER as i32 as u32
                    && state.p_mobj.mo(thing).kind as u32 == MobjType::MT_KNIGHT as i32 as u32)
        {
            if Some(thing) == tm_target {
                return true;
            }
            if state.p_mobj.mo(thing).kind as u32 != MobjType::MT_PLAYER as i32 as u32
                && deh_species_infighting == 0
            {
                return false;
            }
        }
        if state.p_mobj.mo(thing).flags & MF_SHOOTABLE == 0 {
            return state.p_mobj.mo(thing).flags & MF_SOLID == 0;
        }
        damage = (P_Random(&mut state.m_random) % 8 + 1)
            * state
                .info
                .mobjinfo_mut(state.p_mobj.mo(tmthing).kind)
                .damage;
        P_DamageMobj(state, thing, Some(tmthing), tm_target, damage);
        return false;
    }
    if state.p_mobj.mo(thing).flags & MF_SPECIAL != 0 {
        solid = state.p_mobj.mo(thing).flags & MF_SOLID != 0;
        if state.p_map.tmflags & MF_PICKUP != 0 {
            P_TouchSpecialThing(state, thing, tmthing);
        }
        return !solid;
    }
    state.p_mobj.mo(thing).flags & MF_SOLID == 0
}
pub fn P_CheckPosition(state: &mut GameState, thing: MobjId, x: fixed_t, y: fixed_t) -> bool {
    let mut xl: i32;
    let mut xh: i32;
    let mut yl: i32;
    let mut yh: i32;

    state.p_map.tmthing = Some(thing);
    state.p_map.tmflags = state.p_mobj.mo(thing).flags;
    state.p_map.tmx = x;
    state.p_map.tmy = y;
    state.p_map.tmbbox[BoxIndex::Top as usize] = y + state.p_mobj.mo(thing).radius;
    state.p_map.tmbbox[BoxIndex::Bottom as usize] = y - state.p_mobj.mo(thing).radius;
    state.p_map.tmbbox[BoxIndex::Right as usize] = x + state.p_mobj.mo(thing).radius;
    state.p_map.tmbbox[BoxIndex::Left as usize] = x - state.p_mobj.mo(thing).radius;
    let newsubsec: SubsectorId = R_PointInSubsector(state, x, y);
    state.p_map.ceilingline = None;
    state.p_map.tmdropoffz = state
        .p_setup
        .sector_mut(state.p_setup.subsectors[newsubsec.0 as usize].sector)
        .floorheight;
    state.p_map.tmfloorz = state.p_map.tmdropoffz;
    state.p_map.tmceilingz = state
        .p_setup
        .sector_mut(state.p_setup.subsectors[newsubsec.0 as usize].sector)
        .ceilingheight;
    state.r_main.validcount += 1;
    state.p_map.numspechit = 0;
    if state.p_map.tmflags & MF_NOCLIP != 0 {
        return true;
    }
    xl = (state.p_map.tmbbox[BoxIndex::Left as usize] - state.p_setup.bmaporgx - 32 * FRACUNIT)
        >> MAPBLOCKSHIFT;
    xh = (state.p_map.tmbbox[BoxIndex::Right as usize] - state.p_setup.bmaporgx + 32 * FRACUNIT)
        >> MAPBLOCKSHIFT;
    yl = (state.p_map.tmbbox[BoxIndex::Bottom as usize] - state.p_setup.bmaporgy - 32 * FRACUNIT)
        >> MAPBLOCKSHIFT;
    yh = (state.p_map.tmbbox[BoxIndex::Top as usize] - state.p_setup.bmaporgy + 32 * FRACUNIT)
        >> MAPBLOCKSHIFT;
    for bx in xl..=xh {
        for by in yl..=yh {
            if !P_BlockThingsIterator(state, bx, by, PIT_CheckThing) {
                return false;
            }
        }
    }
    xl = (state.p_map.tmbbox[BoxIndex::Left as usize] - state.p_setup.bmaporgx) >> MAPBLOCKSHIFT;
    xh = (state.p_map.tmbbox[BoxIndex::Right as usize] - state.p_setup.bmaporgx) >> MAPBLOCKSHIFT;
    yl = (state.p_map.tmbbox[BoxIndex::Bottom as usize] - state.p_setup.bmaporgy) >> MAPBLOCKSHIFT;
    yh = (state.p_map.tmbbox[BoxIndex::Top as usize] - state.p_setup.bmaporgy) >> MAPBLOCKSHIFT;
    for bx in xl..=xh {
        for by in yl..=yh {
            if !P_BlockLinesIterator(state, bx, by, PIT_CheckLine) {
                return false;
            }
        }
    }
    true
}
pub fn P_TryMove(state: &mut GameState, thing: MobjId, x: fixed_t, y: fixed_t) -> bool {
    let mut side: i32;
    let mut oldside: i32;
    let mut ld: LineId;
    state.p_map.floatok = false;
    if !P_CheckPosition(state, thing, x, y) {
        return false;
    }
    if state.p_mobj.mo(thing).flags & MF_NOCLIP == 0 {
        if state.p_map.tmceilingz - state.p_map.tmfloorz < state.p_mobj.mo(thing).height {
            return false;
        }
        state.p_map.floatok = true;
        if state.p_mobj.mo(thing).flags & MF_TELEPORT == 0
            && state.p_map.tmceilingz - state.p_mobj.mo(thing).z < state.p_mobj.mo(thing).height
        {
            return false;
        }
        if state.p_mobj.mo(thing).flags & MF_TELEPORT == 0
            && state.p_map.tmfloorz - state.p_mobj.mo(thing).z > 24 * FRACUNIT
        {
            return false;
        }
        if state.p_mobj.mo(thing).flags & (MF_DROPOFF | MF_FLOAT) == 0
            && state.p_map.tmfloorz - state.p_map.tmdropoffz > 24 * FRACUNIT
        {
            return false;
        }
    }
    P_UnsetThingPosition(state, thing);
    let oldx: fixed_t = state.p_mobj.mo(thing).x;
    let oldy: fixed_t = state.p_mobj.mo(thing).y;
    state.p_mobj.mo_mut(thing).floorz = state.p_map.tmfloorz;
    state.p_mobj.mo_mut(thing).ceilingz = state.p_map.tmceilingz;
    state.p_mobj.mo_mut(thing).x = x;
    state.p_mobj.mo_mut(thing).y = y;
    P_SetThingPosition(state, thing);
    if state.p_mobj.mo(thing).flags & (MF_TELEPORT | MF_NOCLIP) == 0 {
        loop {
            let fresh0 = state.p_map.numspechit;
            state.p_map.numspechit -= 1;
            if fresh0 == 0 {
                break;
            }
            ld = state.p_map.spechit[state.p_map.numspechit as usize];
            side = P_PointOnLineSide(
                state,
                state.p_mobj.mo(thing).x,
                state.p_mobj.mo(thing).y,
                ld,
            );
            oldside = P_PointOnLineSide(state, oldx, oldy, ld);
            if side != oldside && state.p_setup.line(ld).special != 0 {
                P_CrossSpecialLine(state, ld.0 as i32, oldside, thing);
            }
        }
    }
    true
}
pub fn P_ThingHeightClip(state: &mut GameState, thing: MobjId) -> bool {
    let (z, floorz, x, y) = {
        let t = state.p_mobj.mo(thing);
        (t.z, t.floorz, t.x, t.y)
    };
    let onfloor = z == floorz;
    P_CheckPosition(state, thing, x, y);
    let (tmfloorz, tmceilingz) = (state.p_map.tmfloorz, state.p_map.tmceilingz);
    let t = state.p_mobj.mo_mut(thing);
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
pub fn P_HitSlideLine(state: &mut GameState, ld: LineId) {
    let mut lineangle: angle_t;

    let mut deltaangle: angle_t;

    let ldv = state.p_setup.line(ld);
    if ldv.slopetype == SlopeType::ST_HORIZONTAL {
        state.p_map.tmymove = 0;
        return;
    }
    if ldv.slopetype == SlopeType::ST_VERTICAL {
        state.p_map.tmxmove = 0;
        return;
    }
    let slidemo = state.p_map.slidemo.unwrap();
    let side: i32 = P_PointOnLineSide(
        state,
        state.p_mobj.mo(slidemo).x,
        state.p_mobj.mo(slidemo).y,
        ld,
    );
    lineangle = R_PointToAngle2(state, 0, 0, ldv.dx, ldv.dy);
    if side == 1 {
        lineangle = lineangle.wrapping_add(ANG180) as angle_t as angle_t;
    }
    let moveangle: angle_t = R_PointToAngle2(state, 0, 0, state.p_map.tmxmove, state.p_map.tmymove);
    deltaangle = moveangle.wrapping_sub(lineangle);
    if deltaangle > ANG180 {
        deltaangle = deltaangle.wrapping_add(ANG180) as angle_t as angle_t;
    }
    lineangle >>= ANGLETOFINESHIFT;
    deltaangle >>= ANGLETOFINESHIFT;
    let movelen: fixed_t = P_AproxDistance(state.p_map.tmxmove, state.p_map.tmymove);
    let newlen: fixed_t = FixedMul(movelen, finecosine[deltaangle as usize]);
    state.p_map.tmxmove = FixedMul(newlen, finecosine[lineangle as usize]);
    state.p_map.tmymove = FixedMul(newlen, finesine[lineangle as usize]);
}
pub fn PTR_SlideTraverse(state: &mut GameState, intercept: intercept_t) -> bool {
    let li: LineId = match intercept.target {
        InterceptTarget::Line(id) => id,
        InterceptTarget::Thing(_) => I_Error("PTR_SlideTraverse: not a line?"),
    };
    let slidemo = state.p_map.slidemo.unwrap();
    if state.p_setup.line(li).flags as i32 & ML_TWOSIDED == 0 {
        if P_PointOnLineSide(
            state,
            state.p_mobj.mo(slidemo).x,
            state.p_mobj.mo(slidemo).y,
            li,
        ) != 0
        {
            return true;
        }
    } else {
        P_LineOpening(state, li);
        if state.p_maputl.openrange >= state.p_mobj.mo(slidemo).height
            && state.p_maputl.opentop - state.p_mobj.mo(slidemo).z
                >= state.p_mobj.mo(slidemo).height
            && state.p_maputl.openbottom - state.p_mobj.mo(slidemo).z <= 24 * FRACUNIT
        {
            return true;
        }
    }
    if intercept.frac < state.p_map.bestslidefrac {
        state.p_map.secondslidefrac = state.p_map.bestslidefrac;
        state.p_map.secondslideline = state.p_map.bestslideline;
        state.p_map.bestslidefrac = intercept.frac;
        state.p_map.bestslideline = li;
    }
    false
}
pub fn P_SlideMove(state: &mut GameState, mo: MobjId) {
    state.p_map.slidemo = Some(mo);
    let mut hitcount: i32 = 0;
    loop {
        hitcount += 1;
        if hitcount == 3 {
            break;
        }
        let (mx, my, momx, momy, radius) = {
            let m = state.p_mobj.mo(mo);
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
        state.p_map.bestslidefrac = (FRACUNIT + 1) as fixed_t;
        P_PathTraverse(
            state,
            leadx,
            leady,
            leadx + momx,
            leady + momy,
            PT_ADDLINES,
            PTR_SlideTraverse,
        );
        P_PathTraverse(
            state,
            trailx,
            leady,
            trailx + momx,
            leady + momy,
            PT_ADDLINES,
            PTR_SlideTraverse,
        );
        P_PathTraverse(
            state,
            leadx,
            traily,
            leadx + momx,
            traily + momy,
            PT_ADDLINES,
            PTR_SlideTraverse,
        );
        if state.p_map.bestslidefrac == FRACUNIT + 1 {
            break;
        }
        state.p_map.bestslidefrac -= 0x800;
        if state.p_map.bestslidefrac > 0 {
            let newx = FixedMul(momx, state.p_map.bestslidefrac);
            let newy = FixedMul(momy, state.p_map.bestslidefrac);
            if !P_TryMove(state, mo, mx + newx, my + newy) {
                break;
            }
        }
        state.p_map.bestslidefrac = (FRACUNIT - (state.p_map.bestslidefrac + 0x800)) as fixed_t;
        if state.p_map.bestslidefrac > FRACUNIT {
            state.p_map.bestslidefrac = FRACUNIT as fixed_t;
        }
        if state.p_map.bestslidefrac <= 0 {
            return;
        }
        state.p_map.tmxmove = FixedMul(momx, state.p_map.bestslidefrac);
        state.p_map.tmymove = FixedMul(momy, state.p_map.bestslidefrac);
        P_HitSlideLine(state, state.p_map.bestslideline);
        let (tmxmove, tmymove) = (state.p_map.tmxmove, state.p_map.tmymove);
        {
            let m = state.p_mobj.mo_mut(mo);
            m.momx = tmxmove;
            m.momy = tmymove;
        }
        let (cx, cy) = {
            let m = state.p_mobj.mo(mo);
            (m.x, m.y)
        };
        if !P_TryMove(state, mo, cx + tmxmove, cy + tmymove) {
            continue;
        }
        return;
    }
    let (cx, cy, cmomx, cmomy) = {
        let m = state.p_mobj.mo(mo);
        (m.x, m.y, m.momx, m.momy)
    };
    if !P_TryMove(state, mo, cx, cy + cmomy) {
        let (cx, cy) = {
            let m = state.p_mobj.mo(mo);
            (m.x, m.y)
        };
        P_TryMove(state, mo, cx + cmomx, cy);
    }
}
pub fn PTR_AimTraverse(state: &mut GameState, intercept: intercept_t) -> bool {
    let mut slope: fixed_t;
    let mut thingtopslope: fixed_t;
    let mut thingbottomslope: fixed_t;
    let dist: fixed_t;
    if let InterceptTarget::Line(li) = intercept.target {
        let liv = state.p_setup.line(li);
        if liv.flags as i32 & ML_TWOSIDED == 0 {
            return false;
        }
        P_LineOpening(state, li);
        if state.p_maputl.openbottom >= state.p_maputl.opentop {
            return false;
        }
        dist = FixedMul(state.p_map.attackrange, intercept.frac);
        if liv.backsector.is_none()
            || state
                .p_setup
                .sector_mut(liv.frontsector.unwrap())
                .floorheight
                != state
                    .p_setup
                    .sector_mut(liv.backsector.unwrap())
                    .floorheight
        {
            slope = FixedDiv(state.p_maputl.openbottom - state.p_map.shootz, dist);
            if slope > state.p_sight.bottomslope {
                state.p_sight.bottomslope = slope;
            }
        }
        if liv.backsector.is_none()
            || state
                .p_setup
                .sector_mut(liv.frontsector.unwrap())
                .ceilingheight
                != state
                    .p_setup
                    .sector_mut(liv.backsector.unwrap())
                    .ceilingheight
        {
            slope = FixedDiv(state.p_maputl.opentop - state.p_map.shootz, dist);
            if slope < state.p_sight.topslope {
                state.p_sight.topslope = slope;
            }
        }
        if state.p_sight.topslope <= state.p_sight.bottomslope {
            return false;
        }
        return true;
    }
    let th = match intercept.target {
        InterceptTarget::Thing(id) => id,
        InterceptTarget::Line(_) => unreachable!(),
    };
    if Some(th) == state.p_map.shootthing {
        return true;
    }
    if state.p_mobj.mo(th).flags & MF_SHOOTABLE == 0 {
        return true;
    }
    dist = FixedMul(state.p_map.attackrange, intercept.frac);
    thingtopslope = FixedDiv(
        state.p_mobj.mo(th).z + state.p_mobj.mo(th).height - state.p_map.shootz,
        dist,
    );
    if thingtopslope < state.p_sight.bottomslope {
        return true;
    }
    thingbottomslope = FixedDiv(state.p_mobj.mo(th).z - state.p_map.shootz, dist);
    if thingbottomslope > state.p_sight.topslope {
        return true;
    }
    if thingtopslope > state.p_sight.topslope {
        thingtopslope = state.p_sight.topslope;
    }
    if thingbottomslope < state.p_sight.bottomslope {
        thingbottomslope = state.p_sight.bottomslope;
    }
    state.p_map.aimslope = ((thingtopslope + thingbottomslope) / 2) as fixed_t;
    state.p_map.linetarget = Some(th);
    false
}
pub fn PTR_ShootTraverse(state: &mut GameState, intercept: intercept_t) -> bool {
    let x: fixed_t;
    let y: fixed_t;
    let z: fixed_t;
    let frac: fixed_t;
    let dist: fixed_t;
    let thingtopslope: fixed_t;
    let thingbottomslope: fixed_t;
    let shootthing = state.p_map.shootthing.unwrap();
    if let InterceptTarget::Line(li) = intercept.target {
        if state.p_setup.line(li).special != 0 {
            P_ShootSpecialLine(state, shootthing, li);
        }
        if state.p_setup.line(li).flags as i32 & ML_TWOSIDED != 0 {
            P_LineOpening(state, li);
            let dist = FixedMul(state.p_map.attackrange, intercept.frac);
            // A missing back side (emulated) leaves both openings to check.
            let (check_floor, check_ceiling) = match state.p_setup.line(li).backsector {
                None => (true, true),
                Some(back) => {
                    let front = state.p_setup.line(li).frontsector.unwrap();
                    let (front_floor, front_ceiling) = {
                        let s = state.p_setup.sector_mut(front);
                        (s.floorheight, s.ceilingheight)
                    };
                    let (back_floor, back_ceiling) = {
                        let s = state.p_setup.sector_mut(back);
                        (s.floorheight, s.ceilingheight)
                    };
                    (front_floor != back_floor, front_ceiling != back_ceiling)
                }
            };
            let hits_line = (check_floor
                && FixedDiv(state.p_maputl.openbottom - state.p_map.shootz, dist)
                    > state.p_map.aimslope)
                || (check_ceiling
                    && FixedDiv(state.p_maputl.opentop - state.p_map.shootz, dist)
                        < state.p_map.aimslope);
            if !hits_line {
                // The shot passes through.
                return true;
            }
        }
        frac = intercept.frac - FixedDiv(4 * FRACUNIT, state.p_map.attackrange);
        x = state.p_maputl.trace.x + FixedMul(state.p_maputl.trace.dx, frac);
        y = state.p_maputl.trace.y + FixedMul(state.p_maputl.trace.dy, frac);
        z = state.p_map.shootz
            + FixedMul(
                state.p_map.aimslope,
                FixedMul(frac, state.p_map.attackrange),
            );
        if state
            .p_setup
            .sector_mut(state.p_setup.line(li).frontsector.unwrap())
            .ceilingpic as i32
            == state.r_sky.skyflatnum
        {
            if z > state
                .p_setup
                .sector_mut(state.p_setup.line(li).frontsector.unwrap())
                .ceilingheight
            {
                return false;
            }
            if state.p_setup.line(li).backsector.is_some()
                && state
                    .p_setup
                    .sector_mut(state.p_setup.line(li).backsector.unwrap())
                    .ceilingpic as i32
                    == state.r_sky.skyflatnum
            {
                return false;
            }
        }
        P_SpawnPuff(state, x, y, z);
        false
    } else {
        let th = match intercept.target {
            InterceptTarget::Thing(id) => id,
            InterceptTarget::Line(_) => unreachable!(),
        };
        if th == shootthing {
            return true;
        }
        if state.p_mobj.mo(th).flags & MF_SHOOTABLE == 0 {
            return true;
        }
        dist = FixedMul(state.p_map.attackrange, intercept.frac);
        thingtopslope = FixedDiv(
            state.p_mobj.mo(th).z + state.p_mobj.mo(th).height - state.p_map.shootz,
            dist,
        );
        if thingtopslope < state.p_map.aimslope {
            return true;
        }
        thingbottomslope = FixedDiv(state.p_mobj.mo(th).z - state.p_map.shootz, dist);
        if thingbottomslope > state.p_map.aimslope {
            return true;
        }
        frac = intercept.frac - FixedDiv(10 * FRACUNIT, state.p_map.attackrange);
        x = state.p_maputl.trace.x + FixedMul(state.p_maputl.trace.dx, frac);
        y = state.p_maputl.trace.y + FixedMul(state.p_maputl.trace.dy, frac);
        z = state.p_map.shootz
            + FixedMul(
                state.p_map.aimslope,
                FixedMul(frac, state.p_map.attackrange),
            );
        if state.p_mobj.mo(th).flags & MF_NOBLOOD != 0 {
            P_SpawnPuff(state, x, y, z);
        } else {
            P_SpawnBlood(state, x, y, z, state.p_map.la_damage);
        }
        if state.p_map.la_damage != 0 {
            P_DamageMobj(
                state,
                th,
                Some(shootthing),
                Some(shootthing),
                state.p_map.la_damage,
            );
        }
        false
    }
}
pub fn P_AimLineAttack(
    state: &mut GameState,
    t1: Option<MobjId>,
    angle: angle_t,
    distance: fixed_t,
) -> fixed_t {
    let t1 = P_SubstNullMobj(&mut state.p_mobj, t1);
    let angle = angle >> ANGLETOFINESHIFT;
    let (x1, y1, z1, height1) = {
        let m = state.p_mobj.mo(t1);
        (m.x, m.y, m.z, m.height)
    };
    state.p_map.shootthing = Some(t1);
    let x2 = x1 + (distance >> FRACBITS) * finecosine[angle as usize];
    let y2 = y1 + (distance >> FRACBITS) * finesine[angle as usize];
    state.p_map.shootz = (z1 + (height1 >> 1) + 8 * FRACUNIT) as fixed_t;
    state.p_sight.topslope = (100 * FRACUNIT / 160) as fixed_t;
    state.p_sight.bottomslope = (-100 * FRACUNIT / 160) as fixed_t;
    state.p_map.attackrange = distance;
    state.p_map.linetarget = None;
    P_PathTraverse(
        state,
        x1,
        y1,
        x2,
        y2,
        PT_ADDLINES | PT_ADDTHINGS,
        PTR_AimTraverse,
    );
    if state.p_map.linetarget.is_some() {
        return state.p_map.aimslope;
    }
    0
}
pub fn P_LineAttack(
    state: &mut GameState,
    t1: MobjId,
    angle: angle_t,
    distance: fixed_t,
    slope: fixed_t,
    damage: i32,
) {
    let angle = angle >> ANGLETOFINESHIFT;
    let (x1, y1, z1, height1) = {
        let m = state.p_mobj.mo(t1);
        (m.x, m.y, m.z, m.height)
    };
    state.p_map.shootthing = Some(t1);
    state.p_map.la_damage = damage;
    let x2 = x1 + (distance >> FRACBITS) * finecosine[angle as usize];
    let y2 = y1 + (distance >> FRACBITS) * finesine[angle as usize];
    state.p_map.shootz = (z1 + (height1 >> 1) + 8 * FRACUNIT) as fixed_t;
    state.p_map.attackrange = distance;
    state.p_map.aimslope = slope;
    P_PathTraverse(
        state,
        x1,
        y1,
        x2,
        y2,
        PT_ADDLINES | PT_ADDTHINGS,
        PTR_ShootTraverse,
    );
}
pub fn PTR_UseTraverse(state: &mut GameState, intercept: intercept_t) -> bool {
    let mut side: i32;
    let li = match intercept.target {
        InterceptTarget::Line(id) => id,
        InterceptTarget::Thing(_) => unreachable!(),
    };
    let usething = state.p_map.usething.unwrap();
    if state.p_setup.line(li).special == 0 {
        P_LineOpening(state, li);
        if state.p_maputl.openrange <= 0 {
            S_StartSound(
                state,
                SoundOrigin::Mobj(usething),
                SfxName::sfx_noway as i32,
            );
            return false;
        }
        return true;
    }
    side = 0;
    let (use_x, use_y) = {
        let u = state.p_mobj.mo(usething);
        (u.x, u.y)
    };
    if P_PointOnLineSide(state, use_x, use_y, li) == 1 {
        side = 1;
    }
    P_UseSpecialLine(state, usething, li, side);
    false
}
pub fn P_UseLines(state: &mut GameState, player: PlayerId) {
    let player_mo = state.g_game.player_mut(player).mo;
    state.p_map.usething = player_mo;
    let player_mo = player_mo.unwrap();
    let (angle, x1, y1) = {
        let m = state.p_mobj.mo(player_mo);
        ((m.angle >> ANGLETOFINESHIFT) as i32, m.x, m.y)
    };
    let x2 = x1 + (USERANGE >> FRACBITS) * finecosine[angle as usize];
    let y2 = y1 + (USERANGE >> FRACBITS) * finesine[angle as usize];
    P_PathTraverse(state, x1, y1, x2, y2, PT_ADDLINES, PTR_UseTraverse);
}
pub fn PIT_RadiusAttack(state: &mut GameState, thing_id: MobjId) -> bool {
    let thing = thing_id;

    let mut dist: fixed_t;
    if state.p_mobj.mo(thing).flags & MF_SHOOTABLE == 0 {
        return true;
    }
    if state.p_mobj.mo(thing).kind as u32 == MobjType::MT_CYBORG as i32 as u32
        || state.p_mobj.mo(thing).kind as u32 == MobjType::MT_SPIDER as i32 as u32
    {
        return true;
    }
    let bombspot = state.p_map.bombspot.unwrap();
    let bombsource = state
        .p_map
        .bombsource
        .filter(|&id| state.p_mobj.is_live(id));
    let dx: fixed_t = (state.p_mobj.mo(thing).x - state.p_mobj.mo(bombspot).x).abs() as fixed_t;
    let dy: fixed_t = (state.p_mobj.mo(thing).y - state.p_mobj.mo(bombspot).y).abs() as fixed_t;
    dist = if dx > dy { dx } else { dy };
    dist = (dist - state.p_mobj.mo(thing).radius) >> FRACBITS;
    if dist < 0 {
        dist = 0;
    }
    if dist >= state.p_map.bombdamage {
        return true;
    }
    if P_CheckSight(state, thing, bombspot) {
        P_DamageMobj(
            state,
            thing,
            Some(bombspot),
            bombsource,
            state.p_map.bombdamage - dist,
        );
    }
    true
}
pub fn P_RadiusAttack(state: &mut GameState, spot: MobjId, source: Option<MobjId>, damage: i32) {
    let (spot_x, spot_y) = {
        let s = state.p_mobj.mo(spot);
        (s.x, s.y)
    };
    let dist: fixed_t = ((damage + 32 * FRACUNIT) << FRACBITS) as fixed_t;
    let yh = (spot_y + dist - state.p_setup.bmaporgy) >> MAPBLOCKSHIFT;
    let yl = (spot_y - dist - state.p_setup.bmaporgy) >> MAPBLOCKSHIFT;
    let xh = (spot_x + dist - state.p_setup.bmaporgx) >> MAPBLOCKSHIFT;
    let xl = (spot_x - dist - state.p_setup.bmaporgx) >> MAPBLOCKSHIFT;
    state.p_map.bombspot = Some(spot);
    state.p_map.bombsource = source;
    state.p_map.bombdamage = damage;
    for y in yl..=yh {
        for x in xl..=xh {
            P_BlockThingsIterator(state, x, y, PIT_RadiusAttack);
        }
    }
}
pub fn PIT_ChangeSector(state: &mut GameState, thing: MobjId) -> bool {
    if P_ThingHeightClip(state, thing) {
        return true;
    }
    let (health, flags) = {
        let t = state.p_mobj.mo(thing);
        (t.health, t.flags)
    };
    if health <= 0 {
        P_SetMobjState(state, thing, StateNum::S_GIBS);
        let t = state.p_mobj.mo_mut(thing);
        t.flags &= !MF_SOLID;
        t.height = 0;
        t.radius = 0;
        return true;
    }
    if flags & MF_DROPPED != 0 {
        P_RemoveMobj(state, thing);
        return true;
    }
    if flags & MF_SHOOTABLE == 0 {
        return true;
    }
    state.p_map.nofit = true;
    if state.p_map.crushchange && state.p_tick.leveltime & 3 == 0 {
        P_DamageMobj(state, thing, None, None, 10);
        let (x, y, z, height) = {
            let t = state.p_mobj.mo(thing);
            (t.x, t.y, t.z, t.height)
        };
        let mo = P_SpawnMobj(state, x, y, z + height / 2, MobjType::MT_BLOOD);
        let momx =
            ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 12) as fixed_t;
        let momy =
            ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 12) as fixed_t;
        let m = state.p_mobj.mo_mut(mo);
        m.momx = momx;
        m.momy = momy;
    }
    true
}
pub fn P_ChangeSector(state: &mut GameState, sector: SectorId, crunch: bool) -> bool {
    state.p_map.nofit = false;
    state.p_map.crushchange = crunch;
    let blockbox = state.p_setup.sector_mut(sector).blockbox;
    for x in blockbox[BoxIndex::Left as usize]..=blockbox[BoxIndex::Right as usize] {
        for y in blockbox[BoxIndex::Bottom as usize]..=blockbox[BoxIndex::Top as usize] {
            P_BlockThingsIterator(state, x, y, PIT_ChangeSector);
        }
    }
    state.p_map.nofit
}
fn SpechitOverrun(state: &mut GameState, ld: LineId) {
    if state.p_map.baseaddr == 0 {
        let p: i32 = M_CheckParmWithArgs(state, "-spechit", 1);
        if p > 0 {
            let mut baseaddr: i32 = 0;
            M_StrToInt(
                state.m_argv.myargv[(p + 1) as usize].as_str(),
                &mut baseaddr,
            );
            state.p_map.baseaddr = baseaddr as u32;
        } else {
            state.p_map.baseaddr = DEFAULT_SPECHIT_MAGIC as u32;
        }
    }
    let addr: u32 = (state.p_map.baseaddr as i64 + ld.0 as i64 * 0x3e) as u32;
    match state.p_map.numspechit {
        9..=12 => {
            state.p_map.tmbbox[(state.p_map.numspechit - 9) as usize] = addr as fixed_t;
        }
        13 => {
            state.p_map.crushchange = addr != 0;
        }
        14 => {
            state.p_map.nofit = addr != 0;
        }
        _ => {
            doom_eprintln!(
                state.platform,
                "SpechitOverrun: Warning: unable to emulatean overrun where numspechit={}",
                state.p_map.numspechit,
            );
        }
    };
}
