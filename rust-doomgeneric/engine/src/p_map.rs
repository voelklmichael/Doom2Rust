use crate::d_player::player_t;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_argv::M_CheckParmWithArgs;
use crate::m_bbox::{BOXBOTTOM, BOXLEFT, BOXRIGHT, BOXTOP};
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
use crate::p_mobj::mobj_t;
use crate::p_mobj::MobjId;
use crate::p_mobj::MobjType;
use crate::p_mobj::P_RemoveMobj;
use crate::p_mobj::P_SetMobjState;
use crate::p_mobj::P_SpawnBlood;
use crate::p_mobj::P_SpawnMobj;
use crate::p_mobj::P_SpawnPuff;
use crate::p_mobj::P_SubstNullMobj;
use crate::p_mobj::StateNum;
use crate::p_mobj::{sector_t, SlopeType};
use crate::p_mobj::{
    MF_DROPOFF, MF_DROPPED, MF_FLOAT, MF_MISSILE, MF_NOBLOOD, MF_NOCLIP, MF_PICKUP, MF_SHOOTABLE,
    MF_SKULLFLY, MF_SOLID, MF_SPECIAL, MF_TELEPORT,
};
use crate::p_setup::LineId;
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
use crate::sounds::sfx_noway;
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
pub unsafe fn PIT_StompThing(state: &mut GameState, mut thing_id: MobjId) -> bool {
    let thing = state.p_mobj.mobj_get(thing_id).unwrap();
    let tmthing = state.p_mobj.mobj_get(state.p_map.tmthing.unwrap()).unwrap();
    let mut blockdist: fixed_t = 0;
    if (*thing).flags & MF_SHOOTABLE as i32 == 0 {
        return true;
    }
    blockdist = (*thing).radius + (*tmthing).radius;
    if ((*thing).x - state.p_map.tmx).abs() >= blockdist
        || ((*thing).y - state.p_map.tmy).abs() >= blockdist
    {
        return true;
    }
    if thing == tmthing {
        return true;
    }
    if (*tmthing).player.is_none() && state.g_game.gamemap != 30_i32 {
        return false;
    }
    P_DamageMobj(state, thing, tmthing, tmthing, 10000_i32);
    true
}
pub unsafe fn P_TeleportMove(
    state: &mut GameState,
    mut thing: *mut mobj_t,
    mut x: fixed_t,
    mut y: fixed_t,
) -> bool {
    let mut xl: i32 = 0;
    let mut xh: i32 = 0;
    let mut yl: i32 = 0;
    let mut yh: i32 = 0;
    let mut bx: i32 = 0;
    let mut by: i32 = 0;
    let mut newsubsec: SubsectorId = SubsectorId(0);
    state.p_map.tmthing = Some((*thing).id);
    state.p_map.tmflags = (*thing).flags;
    state.p_map.tmx = x;
    state.p_map.tmy = y;
    state.p_map.tmbbox[BOXTOP as usize] = y + (*thing).radius;
    state.p_map.tmbbox[BOXBOTTOM as usize] = y - (*thing).radius;
    state.p_map.tmbbox[BOXRIGHT as usize] = x + (*thing).radius;
    state.p_map.tmbbox[BOXLEFT as usize] = x - (*thing).radius;
    newsubsec = R_PointInSubsector(state, x, y);
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
    state.p_map.numspechit = 0_i32;
    xl = (state.p_map.tmbbox[BOXLEFT as usize] - state.p_setup.bmaporgx - 32_i32 * FRACUNIT) >> MAPBLOCKSHIFT;
    xh = (state.p_map.tmbbox[BOXRIGHT as usize] - state.p_setup.bmaporgx + 32_i32 * FRACUNIT) >> MAPBLOCKSHIFT;
    yl = (state.p_map.tmbbox[BOXBOTTOM as usize] - state.p_setup.bmaporgy - 32_i32 * FRACUNIT) >> MAPBLOCKSHIFT;
    yh = (state.p_map.tmbbox[BOXTOP as usize] - state.p_setup.bmaporgy + 32_i32 * FRACUNIT) >> MAPBLOCKSHIFT;
    bx = xl;
    while bx <= xh {
        by = yl;
        while by <= yh {
            if !P_BlockThingsIterator(
                state,
                bx,
                by,
                |s, id| unsafe { PIT_StompThing(s, id) },
            ) {
                return false;
            }
            by += 1;
        }
        bx += 1;
    }
    P_UnsetThingPosition(state, &mut *thing);
    (*thing).floorz = state.p_map.tmfloorz;
    (*thing).ceilingz = state.p_map.tmceilingz;
    (*thing).x = x;
    (*thing).y = y;
    P_SetThingPosition(state, &mut *thing);
    true
}
pub unsafe fn PIT_CheckLine(state: &mut GameState, mut ld: LineId) -> bool {
    let ldv = state.p_setup.line(ld);
    if state.p_map.tmbbox[BOXRIGHT as usize] <= ldv.bbox[BOXLEFT as usize]
        || state.p_map.tmbbox[BOXLEFT as usize] >= ldv.bbox[BOXRIGHT as usize]
        || state.p_map.tmbbox[BOXTOP as usize] <= ldv.bbox[BOXBOTTOM as usize]
        || state.p_map.tmbbox[BOXBOTTOM as usize] >= ldv.bbox[BOXTOP as usize]
    {
        return true;
    }
    let tmbbox = state.p_map.tmbbox;
    if P_BoxOnLineSide(state, tmbbox, ld) != -1_i32 {
        return true;
    }
    if ldv.backsector.is_none() {
        return false;
    }
    let tmthing = state.p_mobj.mobj_get(state.p_map.tmthing.unwrap()).unwrap();
    if (*tmthing).flags & MF_MISSILE as i32 == 0 {
        if ldv.flags as i32 & ML_BLOCKING != 0 {
            return false;
        }
        if (*tmthing).player.is_none() && ldv.flags as i32 & ML_BLOCKMONSTERS != 0 {
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
pub unsafe fn PIT_CheckThing(state: &mut GameState, mut thing_id: MobjId) -> bool {
    let thing = state.p_mobj.mobj_get(thing_id).unwrap();
    let tmthing = state.p_mobj.mobj_get(state.p_map.tmthing.unwrap()).unwrap();
    let mut blockdist: fixed_t = 0;
    let mut solid: bool = false;
    let mut damage: i32 = 0;
    if (*thing).flags & (MF_SOLID as i32 | MF_SPECIAL as i32 | MF_SHOOTABLE as i32) == 0 {
        return true;
    }
    blockdist = (*thing).radius + (*tmthing).radius;
    if ((*thing).x - state.p_map.tmx).abs() >= blockdist
        || ((*thing).y - state.p_map.tmy).abs() >= blockdist
    {
        return true;
    }
    if thing == tmthing {
        return true;
    }
    if (*tmthing).flags & MF_SKULLFLY as i32 != 0 {
        damage = (P_Random(&mut state.m_random) % 8_i32 + 1_i32)
            * (*state.info.mobjinfo_mut((*tmthing).type_0)).damage;
        P_DamageMobj(state, thing, tmthing, tmthing, damage);
        (*tmthing).flags &= !(MF_SKULLFLY as i32);
        (*tmthing).momz = 0_i32 as fixed_t;
        (*tmthing).momy = (*tmthing).momz;
        (*tmthing).momx = (*tmthing).momy;
        let spawnstate = (*state.info.mobjinfo_mut((*tmthing).type_0)).spawnstate;
        P_SetMobjState(state, tmthing, spawnstate);
        return false;
    }
    if (*tmthing).flags & MF_MISSILE as i32 != 0 {
        if (*tmthing).z > (*thing).z + (*thing).height {
            return true;
        }
        if (*tmthing).z + (*tmthing).height < (*thing).z {
            return true;
        }
        let tm_target = (*tmthing).target.and_then(|id| state.p_mobj.mobj_get(id));
        if tm_target.is_some()
            && ((*tm_target.unwrap()).type_0 as u32 == (*thing).type_0 as u32
                || (*tm_target.unwrap()).type_0 as u32 == MobjType::MT_KNIGHT as i32 as u32
                    && (*thing).type_0 as u32 == MobjType::MT_BRUISER as i32 as u32
                || (*tm_target.unwrap()).type_0 as u32 == MobjType::MT_BRUISER as i32 as u32
                    && (*thing).type_0 as u32 == MobjType::MT_KNIGHT as i32 as u32)
        {
            if Some(thing) == tm_target {
                return true;
            }
            if (*thing).type_0 as u32 != MobjType::MT_PLAYER as i32 as u32
                && deh_species_infighting == 0
            {
                return false;
            }
        }
        if (*thing).flags & MF_SHOOTABLE as i32 == 0 {
            return (*thing).flags & MF_SOLID as i32 == 0;
        }
        damage = (P_Random(&mut state.m_random) % 8_i32 + 1_i32)
            * (*state.info.mobjinfo_mut((*tmthing).type_0)).damage;
        P_DamageMobj(
            state,
            thing,
            tmthing,
            tm_target.unwrap_or(::core::ptr::null_mut()),
            damage,
        );
        return false;
    }
    if (*thing).flags & MF_SPECIAL as i32 != 0 {
        solid = (*thing).flags & MF_SOLID as i32 != 0;
        if state.p_map.tmflags & MF_PICKUP as i32 != 0 {
            P_TouchSpecialThing(state, thing, tmthing);
        }
        return !solid;
    }
    (*thing).flags & MF_SOLID as i32 == 0
}
pub unsafe fn P_CheckPosition(
    state: &mut GameState,
    mut thing: *mut mobj_t,
    mut x: fixed_t,
    mut y: fixed_t,
) -> bool {
    let mut xl: i32 = 0;
    let mut xh: i32 = 0;
    let mut yl: i32 = 0;
    let mut yh: i32 = 0;
    let mut bx: i32 = 0;
    let mut by: i32 = 0;
    let mut newsubsec: SubsectorId = SubsectorId(0);
    state.p_map.tmthing = Some((*thing).id);
    state.p_map.tmflags = (*thing).flags;
    state.p_map.tmx = x;
    state.p_map.tmy = y;
    state.p_map.tmbbox[BOXTOP as usize] = y + (*thing).radius;
    state.p_map.tmbbox[BOXBOTTOM as usize] = y - (*thing).radius;
    state.p_map.tmbbox[BOXRIGHT as usize] = x + (*thing).radius;
    state.p_map.tmbbox[BOXLEFT as usize] = x - (*thing).radius;
    newsubsec = R_PointInSubsector(state, x, y);
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
    state.p_map.numspechit = 0_i32;
    if state.p_map.tmflags & MF_NOCLIP as i32 != 0 {
        return true;
    }
    xl = (state.p_map.tmbbox[BOXLEFT as usize] - state.p_setup.bmaporgx - 32_i32 * FRACUNIT) >> MAPBLOCKSHIFT;
    xh = (state.p_map.tmbbox[BOXRIGHT as usize] - state.p_setup.bmaporgx + 32_i32 * FRACUNIT) >> MAPBLOCKSHIFT;
    yl = (state.p_map.tmbbox[BOXBOTTOM as usize] - state.p_setup.bmaporgy - 32_i32 * FRACUNIT) >> MAPBLOCKSHIFT;
    yh = (state.p_map.tmbbox[BOXTOP as usize] - state.p_setup.bmaporgy + 32_i32 * FRACUNIT) >> MAPBLOCKSHIFT;
    bx = xl;
    while bx <= xh {
        by = yl;
        while by <= yh {
            if !P_BlockThingsIterator(
                state,
                bx,
                by,
                |s, id| unsafe { PIT_CheckThing(s, id) },
            ) {
                return false;
            }
            by += 1;
        }
        bx += 1;
    }
    xl = (state.p_map.tmbbox[BOXLEFT as usize] - state.p_setup.bmaporgx) >> MAPBLOCKSHIFT;
    xh = (state.p_map.tmbbox[BOXRIGHT as usize] - state.p_setup.bmaporgx) >> MAPBLOCKSHIFT;
    yl = (state.p_map.tmbbox[BOXBOTTOM as usize] - state.p_setup.bmaporgy) >> MAPBLOCKSHIFT;
    yh = (state.p_map.tmbbox[BOXTOP as usize] - state.p_setup.bmaporgy) >> MAPBLOCKSHIFT;
    bx = xl;
    while bx <= xh {
        by = yl;
        while by <= yh {
            if !P_BlockLinesIterator(
                state,
                bx,
                by,
                |s, ld| unsafe { PIT_CheckLine(s, ld) },
            ) {
                return false;
            }
            by += 1;
        }
        bx += 1;
    }
    true
}
pub unsafe fn P_TryMove(
    state: &mut GameState,
    mut thing: *mut mobj_t,
    mut x: fixed_t,
    mut y: fixed_t,
) -> bool {
    let mut oldx: fixed_t = 0;
    let mut oldy: fixed_t = 0;
    let mut side: i32 = 0;
    let mut oldside: i32 = 0;
    let mut ld: LineId;
    state.p_map.floatok = false;
    if !P_CheckPosition(state, thing, x, y) {
        return false;
    }
    if (*thing).flags & MF_NOCLIP as i32 == 0 {
        if state.p_map.tmceilingz - state.p_map.tmfloorz < (*thing).height {
            return false;
        }
        state.p_map.floatok = true;
        if (*thing).flags & MF_TELEPORT as i32 == 0
            && state.p_map.tmceilingz - (*thing).z < (*thing).height
        {
            return false;
        }
        if (*thing).flags & MF_TELEPORT as i32 == 0
            && state.p_map.tmfloorz - (*thing).z > 24_i32 * FRACUNIT
        {
            return false;
        }
        if (*thing).flags & (MF_DROPOFF as i32 | MF_FLOAT as i32) == 0
            && state.p_map.tmfloorz - state.p_map.tmdropoffz > 24_i32 * FRACUNIT
        {
            return false;
        }
    }
    P_UnsetThingPosition(state, &mut *thing);
    oldx = (*thing).x;
    oldy = (*thing).y;
    (*thing).floorz = state.p_map.tmfloorz;
    (*thing).ceilingz = state.p_map.tmceilingz;
    (*thing).x = x;
    (*thing).y = y;
    P_SetThingPosition(state, &mut *thing);
    if (*thing).flags & (MF_TELEPORT as i32 | MF_NOCLIP as i32) == 0 {
        loop {
            let fresh0 = state.p_map.numspechit;
            state.p_map.numspechit -= 1;
            if fresh0 == 0 {
                break;
            }
            ld = state.p_map.spechit[state.p_map.numspechit as usize];
            side = P_PointOnLineSide(state, (*thing).x, (*thing).y, ld);
            oldside = P_PointOnLineSide(state, oldx, oldy, ld);
            if side != oldside && state.p_setup.line(ld).special != 0 {
                P_CrossSpecialLine(state, ld.0 as i32, oldside, thing);
            }
        }
    }
    true
}
pub unsafe fn P_ThingHeightClip(state: &mut GameState, mut thing: *mut mobj_t) -> bool {
    let mut onfloor: bool;
    onfloor = (*thing).z == (*thing).floorz;
    P_CheckPosition(state, thing, (*thing).x, (*thing).y);
    (*thing).floorz = state.p_map.tmfloorz;
    (*thing).ceilingz = state.p_map.tmceilingz;
    if onfloor {
        (*thing).z = (*thing).floorz;
    } else if (*thing).z + (*thing).height > (*thing).ceilingz {
        (*thing).z = (*thing).ceilingz - (*thing).height;
    }
    if (*thing).ceilingz - (*thing).floorz < (*thing).height {
        return false;
    }
    true
}
pub unsafe fn P_HitSlideLine(state: &mut GameState, mut ld: LineId) {
    let mut side: i32 = 0;
    let mut lineangle: angle_t = 0;
    let mut moveangle: angle_t = 0;
    let mut deltaangle: angle_t = 0;
    let mut movelen: fixed_t = 0;
    let mut newlen: fixed_t = 0;
    let ldv = state.p_setup.line(ld);
    if ldv.slopetype == SlopeType::ST_HORIZONTAL {
        state.p_map.tmymove = 0_i32 as fixed_t;
        return;
    }
    if ldv.slopetype == SlopeType::ST_VERTICAL {
        state.p_map.tmxmove = 0_i32 as fixed_t;
        return;
    }
    let slidemo = state.p_mobj.mobj_get(state.p_map.slidemo.unwrap()).unwrap();
    side = P_PointOnLineSide(state, (*slidemo).x, (*slidemo).y, ld);
    lineangle = R_PointToAngle2(state, 0 as fixed_t, 0 as fixed_t, ldv.dx, ldv.dy);
    if side == 1_i32 {
        lineangle = lineangle.wrapping_add(ANG180) as angle_t as angle_t;
    }
    moveangle = R_PointToAngle2(
        state,
        0 as fixed_t,
        0 as fixed_t,
        state.p_map.tmxmove,
        state.p_map.tmymove,
    );
    deltaangle = moveangle.wrapping_sub(lineangle);
    if deltaangle > ANG180 {
        deltaangle = deltaangle.wrapping_add(ANG180) as angle_t as angle_t;
    }
    lineangle >>= ANGLETOFINESHIFT;
    deltaangle >>= ANGLETOFINESHIFT;
    movelen = P_AproxDistance(state.p_map.tmxmove, state.p_map.tmymove);
    newlen = FixedMul(movelen, finecosine[deltaangle as isize]);
    state.p_map.tmxmove = FixedMul(newlen, finecosine[lineangle as isize]);
    state.p_map.tmymove = FixedMul(newlen, finesine[lineangle as usize]);
}
pub unsafe fn PTR_SlideTraverse(state: &mut GameState, in_0: intercept_t) -> bool {
    let li: LineId = match in_0.target {
        InterceptTarget::Line(id) => id,
        InterceptTarget::Thing(_) => I_Error("PTR_SlideTraverse: not a line?"),
    };
    let slidemo = state.p_mobj.mobj_get(state.p_map.slidemo.unwrap()).unwrap();
    if state.p_setup.line(li).flags as i32 & ML_TWOSIDED == 0 {
        if P_PointOnLineSide(state, (*slidemo).x, (*slidemo).y, li) != 0 {
            return true;
        }
    } else {
        P_LineOpening(state, li);
        if state.p_maputl.openrange >= (*slidemo).height {
            if state.p_maputl.opentop - (*slidemo).z >= (*slidemo).height {
                if state.p_maputl.openbottom - (*slidemo).z <= 24_i32 * FRACUNIT {
                    return true;
                }
            }
        }
    }
    if in_0.frac < state.p_map.bestslidefrac {
        state.p_map.secondslidefrac = state.p_map.bestslidefrac;
        state.p_map.secondslideline = state.p_map.bestslideline;
        state.p_map.bestslidefrac = in_0.frac;
        state.p_map.bestslideline = li;
    }
    false
}
pub unsafe fn P_SlideMove(state: &mut GameState, mut mo: *mut mobj_t) {
    let mut leadx: fixed_t = 0;
    let mut leady: fixed_t = 0;
    let mut trailx: fixed_t = 0;
    let mut traily: fixed_t = 0;
    let mut newx: fixed_t = 0;
    let mut newy: fixed_t = 0;
    let mut hitcount: i32 = 0;
    state.p_map.slidemo = Some((*mo).id);
    hitcount = 0_i32;
    loop {
        hitcount += 1;
        if hitcount == 3_i32 {
            break;
        }
        if (*mo).momx > 0_i32 {
            leadx = (*mo).x + (*mo).radius;
            trailx = (*mo).x - (*mo).radius;
        } else {
            leadx = (*mo).x - (*mo).radius;
            trailx = (*mo).x + (*mo).radius;
        }
        if (*mo).momy > 0_i32 {
            leady = (*mo).y + (*mo).radius;
            traily = (*mo).y - (*mo).radius;
        } else {
            leady = (*mo).y - (*mo).radius;
            traily = (*mo).y + (*mo).radius;
        }
        state.p_map.bestslidefrac = (FRACUNIT + 1_i32) as fixed_t;
        P_PathTraverse(
            state,
            leadx,
            leady,
            leadx + (*mo).momx,
            leady + (*mo).momy,
            PT_ADDLINES,
            |s, i| unsafe { PTR_SlideTraverse(s, i) },
        );
        P_PathTraverse(
            state,
            trailx,
            leady,
            trailx + (*mo).momx,
            leady + (*mo).momy,
            PT_ADDLINES,
            |s, i| unsafe { PTR_SlideTraverse(s, i) },
        );
        P_PathTraverse(
            state,
            leadx,
            traily,
            leadx + (*mo).momx,
            traily + (*mo).momy,
            PT_ADDLINES,
            |s, i| unsafe { PTR_SlideTraverse(s, i) },
        );
        if state.p_map.bestslidefrac == FRACUNIT + 1_i32 {
            break;
        }
        state.p_map.bestslidefrac -= 0x800_i32;
        if state.p_map.bestslidefrac > 0_i32 {
            newx = FixedMul((*mo).momx, state.p_map.bestslidefrac);
            newy = FixedMul((*mo).momy, state.p_map.bestslidefrac);
            if !P_TryMove(state, mo, (*mo).x + newx, (*mo).y + newy) {
                break;
            }
        }
        state.p_map.bestslidefrac = (FRACUNIT - (state.p_map.bestslidefrac + 0x800_i32)) as fixed_t;
        if state.p_map.bestslidefrac > FRACUNIT {
            state.p_map.bestslidefrac = FRACUNIT as fixed_t;
        }
        if state.p_map.bestslidefrac <= 0_i32 {
            return;
        }
        state.p_map.tmxmove = FixedMul((*mo).momx, state.p_map.bestslidefrac);
        state.p_map.tmymove = FixedMul((*mo).momy, state.p_map.bestslidefrac);
        P_HitSlideLine(state, state.p_map.bestslideline);
        (*mo).momx = state.p_map.tmxmove;
        (*mo).momy = state.p_map.tmymove;
        if !P_TryMove(
            state,
            mo,
            (*mo).x + state.p_map.tmxmove,
            (*mo).y + state.p_map.tmymove,
        ) {
            continue;
        }
        return;
    }
    if !P_TryMove(state, mo, (*mo).x, (*mo).y + (*mo).momy) {
        P_TryMove(state, mo, (*mo).x + (*mo).momx, (*mo).y);
    }
}
pub unsafe fn PTR_AimTraverse(state: &mut GameState, in_0: intercept_t) -> bool {
    let mut th: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut slope: fixed_t = 0;
    let mut thingtopslope: fixed_t = 0;
    let mut thingbottomslope: fixed_t = 0;
    let mut dist: fixed_t = 0;
    if let InterceptTarget::Line(li) = in_0.target {
        let liv = state.p_setup.line(li);
        if liv.flags as i32 & ML_TWOSIDED == 0 {
            return false;
        }
        P_LineOpening(state, li);
        if state.p_maputl.openbottom >= state.p_maputl.opentop {
            return false;
        }
        dist = FixedMul(state.p_map.attackrange, in_0.frac);
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
    th = match in_0.target {
        InterceptTarget::Thing(id) => state.p_mobj.mobj_get(id).unwrap(),
        InterceptTarget::Line(_) => unreachable!(),
    };
    if Some((*th).id) == state.p_map.shootthing {
        return true;
    }
    if (*th).flags & MF_SHOOTABLE as i32 == 0 {
        return true;
    }
    dist = FixedMul(state.p_map.attackrange, in_0.frac);
    thingtopslope = FixedDiv((*th).z + (*th).height - state.p_map.shootz, dist);
    if thingtopslope < state.p_sight.bottomslope {
        return true;
    }
    thingbottomslope = FixedDiv((*th).z - state.p_map.shootz, dist);
    if thingbottomslope > state.p_sight.topslope {
        return true;
    }
    if thingtopslope > state.p_sight.topslope {
        thingtopslope = state.p_sight.topslope;
    }
    if thingbottomslope < state.p_sight.bottomslope {
        thingbottomslope = state.p_sight.bottomslope;
    }
    state.p_map.aimslope = ((thingtopslope + thingbottomslope) / 2_i32) as fixed_t;
    state.p_map.linetarget = Some((*th).id);
    false
}
pub unsafe fn PTR_ShootTraverse(state: &mut GameState, in_0: intercept_t) -> bool {
    let mut current_block: u64;
    let mut x: fixed_t = 0;
    let mut y: fixed_t = 0;
    let mut z: fixed_t = 0;
    let mut frac: fixed_t = 0;
    let mut th: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut slope: fixed_t = 0;
    let mut dist: fixed_t = 0;
    let mut thingtopslope: fixed_t = 0;
    let mut thingbottomslope: fixed_t = 0;
    let shootthing = state
        .p_mobj
        .mobj_get(state.p_map.shootthing.unwrap())
        .unwrap();
    if let InterceptTarget::Line(li) = in_0.target {
        if state.p_setup.line(li).special != 0 {
            P_ShootSpecialLine(state, shootthing, li);
        }
        if state.p_setup.line(li).flags as i32 & ML_TWOSIDED != 0 {
            P_LineOpening(state, li);
            dist = FixedMul(state.p_map.attackrange, in_0.frac);
            if state.p_setup.line(li).backsector.is_none() {
                slope = FixedDiv(state.p_maputl.openbottom - state.p_map.shootz, dist);
                if slope > state.p_map.aimslope {
                    current_block = 15534775465039326179;
                } else {
                    slope = FixedDiv(state.p_maputl.opentop - state.p_map.shootz, dist);
                    if slope < state.p_map.aimslope {
                        current_block = 15534775465039326179;
                    } else {
                        current_block = 4808432441040389987;
                    }
                }
            } else {
                if state
                    .p_setup
                    .sector_mut(state.p_setup.line(li).frontsector.unwrap())
                    .floorheight
                    != state
                        .p_setup
                        .sector_mut(state.p_setup.line(li).backsector.unwrap())
                        .floorheight
                {
                    slope = FixedDiv(state.p_maputl.openbottom - state.p_map.shootz, dist);
                    if slope > state.p_map.aimslope {
                        current_block = 15534775465039326179;
                    } else {
                        current_block = 12039483399334584727;
                    }
                } else {
                    current_block = 12039483399334584727;
                }
                match current_block {
                    15534775465039326179 => {}
                    _ => {
                        if state
                            .p_setup
                            .sector_mut(state.p_setup.line(li).frontsector.unwrap())
                            .ceilingheight
                            != state
                                .p_setup
                                .sector_mut(state.p_setup.line(li).backsector.unwrap())
                                .ceilingheight
                        {
                            slope = FixedDiv(state.p_maputl.opentop - state.p_map.shootz, dist);
                            if slope < state.p_map.aimslope {
                                current_block = 15534775465039326179;
                            } else {
                                current_block = 4808432441040389987;
                            }
                        } else {
                            current_block = 4808432441040389987;
                        }
                    }
                }
            }
            match current_block {
                15534775465039326179 => {}
                _ => return true,
            }
        }
        frac = in_0.frac - FixedDiv(4 as fixed_t * FRACUNIT, state.p_map.attackrange);
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
        th = match in_0.target {
            InterceptTarget::Thing(id) => state.p_mobj.mobj_get(id).unwrap(),
            InterceptTarget::Line(_) => unreachable!(),
        };
        if th == shootthing {
            return true;
        }
        if (*th).flags & MF_SHOOTABLE as i32 == 0 {
            return true;
        }
        dist = FixedMul(state.p_map.attackrange, in_0.frac);
        thingtopslope = FixedDiv((*th).z + (*th).height - state.p_map.shootz, dist);
        if thingtopslope < state.p_map.aimslope {
            return true;
        }
        thingbottomslope = FixedDiv((*th).z - state.p_map.shootz, dist);
        if thingbottomslope > state.p_map.aimslope {
            return true;
        }
        frac = in_0.frac - FixedDiv(10 as fixed_t * FRACUNIT, state.p_map.attackrange);
        x = state.p_maputl.trace.x + FixedMul(state.p_maputl.trace.dx, frac);
        y = state.p_maputl.trace.y + FixedMul(state.p_maputl.trace.dy, frac);
        z = state.p_map.shootz
            + FixedMul(
                state.p_map.aimslope,
                FixedMul(frac, state.p_map.attackrange),
            );
        if (*th).flags & MF_NOBLOOD as i32 != 0 {
            P_SpawnPuff(state, x, y, z);
        } else {
            P_SpawnBlood(state, x, y, z, state.p_map.la_damage);
        }
        if state.p_map.la_damage != 0 {
            P_DamageMobj(state, th, shootthing, shootthing, state.p_map.la_damage);
        }
        false
    }
}
pub unsafe fn P_AimLineAttack(
    state: &mut GameState,
    mut t1: *mut mobj_t,
    mut angle: angle_t,
    mut distance: fixed_t,
) -> fixed_t {
    let mut x2: fixed_t = 0;
    let mut y2: fixed_t = 0;
    t1 = P_SubstNullMobj(&mut state.p_mobj, t1);
    angle >>= ANGLETOFINESHIFT;
    state.p_map.shootthing = Some((*t1).id);
    x2 = (*t1).x + (distance >> FRACBITS) * finecosine[angle as isize];
    y2 = (*t1).y + (distance >> FRACBITS) * finesine[angle as usize];
    state.p_map.shootz = ((*t1).z + ((*t1).height >> 1_i32) + 8_i32 * FRACUNIT) as fixed_t;
    state.p_sight.topslope = (100_i32 * FRACUNIT / 160_i32) as fixed_t;
    state.p_sight.bottomslope = (-100_i32 * FRACUNIT / 160_i32) as fixed_t;
    state.p_map.attackrange = distance;
    state.p_map.linetarget = None;
    P_PathTraverse(
        state,
        (*t1).x,
        (*t1).y,
        x2,
        y2,
        PT_ADDLINES | PT_ADDTHINGS,
        |s, i| unsafe { PTR_AimTraverse(s, i) },
    );
    if state.p_map.linetarget.is_some() {
        return state.p_map.aimslope;
    }
    0 as fixed_t
}
pub unsafe fn P_LineAttack(
    state: &mut GameState,
    mut t1: *mut mobj_t,
    mut angle: angle_t,
    mut distance: fixed_t,
    mut slope: fixed_t,
    mut damage: i32,
) {
    let mut x2: fixed_t = 0;
    let mut y2: fixed_t = 0;
    angle >>= ANGLETOFINESHIFT;
    state.p_map.shootthing = Some((*t1).id);
    state.p_map.la_damage = damage;
    x2 = (*t1).x + (distance >> FRACBITS) * finecosine[angle as isize];
    y2 = (*t1).y + (distance >> FRACBITS) * finesine[angle as usize];
    state.p_map.shootz = ((*t1).z + ((*t1).height >> 1_i32) + 8_i32 * FRACUNIT) as fixed_t;
    state.p_map.attackrange = distance;
    state.p_map.aimslope = slope;
    P_PathTraverse(
        state,
        (*t1).x,
        (*t1).y,
        x2,
        y2,
        PT_ADDLINES | PT_ADDTHINGS,
        |s, i| unsafe { PTR_ShootTraverse(s, i) },
    );
}
pub unsafe fn PTR_UseTraverse(state: &mut GameState, in_0: intercept_t) -> bool {
    let mut side: i32 = 0;
    let li = match in_0.target {
        InterceptTarget::Line(id) => id,
        InterceptTarget::Thing(_) => unreachable!(),
    };
    let usething = state
        .p_mobj
        .mobj_get(state.p_map.usething.unwrap())
        .unwrap();
    if state.p_setup.line(li).special == 0 {
        P_LineOpening(state, li);
        if state.p_maputl.openrange <= 0_i32 {
            S_StartSound(state, SoundOrigin::Mobj((*(usething)).id), sfx_noway as i32);
            return false;
        }
        return true;
    }
    side = 0_i32;
    if P_PointOnLineSide(state, (*usething).x, (*usething).y, li) == 1_i32 {
        side = 1_i32;
    }
    P_UseSpecialLine(state, usething, li, side);
    false
}
pub unsafe fn P_UseLines(state: &mut GameState, mut player: *mut player_t) {
    let mut angle: i32 = 0;
    let mut x1: fixed_t = 0;
    let mut y1: fixed_t = 0;
    let mut x2: fixed_t = 0;
    let mut y2: fixed_t = 0;
    state.p_map.usething = (*player).mo;
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    angle = ((*player_mo).angle >> ANGLETOFINESHIFT) as i32;
    x1 = (*player_mo).x;
    y1 = (*player_mo).y;
    x2 = x1 + (USERANGE >> FRACBITS) * finecosine[angle as isize];
    y2 = y1 + (USERANGE >> FRACBITS) * finesine[angle as usize];
    P_PathTraverse(
        state,
        x1,
        y1,
        x2,
        y2,
        PT_ADDLINES,
        |s, i| unsafe { PTR_UseTraverse(s, i) },
    );
}
pub unsafe fn PIT_RadiusAttack(state: &mut GameState, mut thing_id: MobjId) -> bool {
    let thing = state.p_mobj.mobj_get(thing_id).unwrap();
    let mut dx: fixed_t = 0;
    let mut dy: fixed_t = 0;
    let mut dist: fixed_t = 0;
    if (*thing).flags & MF_SHOOTABLE as i32 == 0 {
        return true;
    }
    if (*thing).type_0 as u32 == MobjType::MT_CYBORG as i32 as u32
        || (*thing).type_0 as u32 == MobjType::MT_SPIDER as i32 as u32
    {
        return true;
    }
    let bombspot = state
        .p_mobj
        .mobj_get(state.p_map.bombspot.unwrap())
        .unwrap();
    let bombsource = state
        .p_map
        .bombsource
        .and_then(|id| state.p_mobj.mobj_get(id))
        .unwrap_or(::core::ptr::null_mut());
    dx = ((*thing).x - (*bombspot).x).abs() as fixed_t;
    dy = ((*thing).y - (*bombspot).y).abs() as fixed_t;
    dist = if dx > dy { dx } else { dy };
    dist = (dist - (*thing).radius) >> FRACBITS;
    if dist < 0_i32 {
        dist = 0_i32 as fixed_t;
    }
    if dist >= state.p_map.bombdamage {
        return true;
    }
    if P_CheckSight(state, &*thing, &*bombspot) {
        P_DamageMobj(
            state,
            thing,
            bombspot,
            bombsource,
            state.p_map.bombdamage - dist,
        );
    }
    true
}
pub unsafe fn P_RadiusAttack(
    state: &mut GameState,
    mut spot: *mut mobj_t,
    mut source: *mut mobj_t,
    mut damage: i32,
) {
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut xl: i32 = 0;
    let mut xh: i32 = 0;
    let mut yl: i32 = 0;
    let mut yh: i32 = 0;
    let mut dist: fixed_t = 0;
    dist = ((damage + 32_i32 * FRACUNIT) << FRACBITS) as fixed_t;
    yh = ((*spot).y + dist - state.p_setup.bmaporgy) >> MAPBLOCKSHIFT;
    yl = ((*spot).y - dist - state.p_setup.bmaporgy) >> MAPBLOCKSHIFT;
    xh = ((*spot).x + dist - state.p_setup.bmaporgx) >> MAPBLOCKSHIFT;
    xl = ((*spot).x - dist - state.p_setup.bmaporgx) >> MAPBLOCKSHIFT;
    state.p_map.bombspot = Some((*spot).id);
    state.p_map.bombsource = if source.is_null() {
        None
    } else {
        Some((*source).id)
    };
    state.p_map.bombdamage = damage;
    y = yl;
    while y <= yh {
        x = xl;
        while x <= xh {
            P_BlockThingsIterator(
                state,
                x,
                y,
                |s, id| unsafe { PIT_RadiusAttack(s, id) },
            );
            x += 1;
        }
        y += 1;
    }
}
pub unsafe fn PIT_ChangeSector(state: &mut GameState, mut thing_id: MobjId) -> bool {
    let thing = state.p_mobj.mobj_get(thing_id).unwrap();
    let mut mo: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    if P_ThingHeightClip(state, thing) {
        return true;
    }
    if (*thing).health <= 0_i32 {
        P_SetMobjState(state, thing, StateNum::S_GIBS);
        (*thing).flags &= !(MF_SOLID as i32);
        (*thing).height = 0_i32 as fixed_t;
        (*thing).radius = 0_i32 as fixed_t;
        return true;
    }
    if (*thing).flags & MF_DROPPED as i32 != 0 {
        P_RemoveMobj(state, thing);
        return true;
    }
    if (*thing).flags & MF_SHOOTABLE as i32 == 0 {
        return true;
    }
    state.p_map.nofit = true;
    if state.p_map.crushchange && state.p_tick.leveltime & 3_i32 == 0 {
        P_DamageMobj(
            state,
            thing,
            ::core::ptr::null_mut::<mobj_t>(),
            ::core::ptr::null_mut::<mobj_t>(),
            10_i32,
        );
        mo = P_SpawnMobj(
            state,
            (*thing).x,
            (*thing).y,
            (*thing).z + (*thing).height / 2 as fixed_t,
            MobjType::MT_BLOOD,
        );
        (*mo).momx =
            ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 12_i32) as fixed_t;
        (*mo).momy =
            ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 12_i32) as fixed_t;
    }
    true
}
pub unsafe fn P_ChangeSector(
    state: &mut GameState,
    mut sector: *mut sector_t,
    mut crunch: bool,
) -> bool {
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    state.p_map.nofit = false;
    state.p_map.crushchange = crunch;
    x = (*sector).blockbox[BOXLEFT as usize];
    while x <= (*sector).blockbox[BOXRIGHT as usize] {
        y = (*sector).blockbox[BOXBOTTOM as usize];
        while y <= (*sector).blockbox[BOXTOP as usize] {
            P_BlockThingsIterator(
                state,
                x,
                y,
                |s, id| unsafe { PIT_ChangeSector(s, id) },
            );
            y += 1;
        }
        x += 1;
    }
    state.p_map.nofit
}
fn SpechitOverrun(state: &mut GameState, mut ld: LineId) {
    let mut addr: u32 = 0;
    if state.p_map.baseaddr == 0_u32 {
        let mut p: i32 = 0;
        p = M_CheckParmWithArgs(state, "-spechit", 1_i32);
        if p > 0_i32 {
            let mut baseaddr: i32 = 0;
            M_StrToInt(
                state.m_argv.myargv[(p + 1_i32) as usize].to_str().unwrap(),
                &mut baseaddr,
            );
            state.p_map.baseaddr = baseaddr as u32;
        } else {
            state.p_map.baseaddr = DEFAULT_SPECHIT_MAGIC as u32;
        }
    }
    addr = (state.p_map.baseaddr as i64 + ld.0 as i64 * 0x3e_i64) as u32;
    match state.p_map.numspechit {
        9 | 10 | 11 | 12 => {
            state.p_map.tmbbox[(state.p_map.numspechit - 9_i32) as usize] = addr as fixed_t;
        }
        13 => {
            state.p_map.crushchange = addr != 0;
        }
        14 => {
            state.p_map.nofit = addr != 0;
        }
        _ => {
            eprintln!(
                "SpechitOverrun: Warning: unable to emulatean overrun where numspechit={}",
                state.p_map.numspechit,
            );
        }
    };
}
