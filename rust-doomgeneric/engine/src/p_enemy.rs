use crate::src::d_mode::commercial;
use crate::src::d_mode::{sk_easy, sk_nightmare};
use crate::src::d_player::player_t;
use crate::src::doomdef::boolean;
use crate::src::g_game::G_ExitLevel;
use crate::src::i_system::I_Error;
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FixedMul;
use crate::src::m_random::P_Random;
use crate::src::p_doors::EV_DoDoor;
use crate::src::p_doors::{vld_blazeOpen, vld_open};
use crate::src::p_floor::EV_DoFloor;
use crate::src::p_floor::{lowerFloorToLowest, raiseToTexture};
use crate::src::p_inter::P_DamageMobj;
use crate::src::p_map::P_AimLineAttack;
use crate::src::p_map::P_CheckPosition;
use crate::src::p_map::P_LineAttack;
use crate::src::p_map::P_RadiusAttack;
use crate::src::p_map::P_TeleportMove;
use crate::src::p_map::P_TryMove;
use crate::src::p_maputl::P_AproxDistance;
use crate::src::p_maputl::P_BlockThingsIterator;
use crate::src::p_maputl::P_LineOpening;
use crate::src::p_maputl::P_SetThingPosition;
use crate::src::p_maputl::P_UnsetThingPosition;
use crate::src::p_mobj::MobjId;
use crate::src::p_mobj::P_RemoveMobj;
use crate::src::p_mobj::P_SetMobjState;
use crate::src::p_mobj::P_SpawnMissile;
use crate::src::p_mobj::P_SpawnMobj;
use crate::src::p_mobj::P_SpawnPuff;
use crate::src::p_mobj::P_SubstNullMobj;
use crate::src::p_mobj::{
    line_s, line_t, mobjinfo_t, sector_t, thinker_t, ST_HORIZONTAL,
};
use crate::src::p_mobj::{mobj_t, pspdef_t};
use crate::src::p_mobj::{
    mobjtype_t, MT_ARACHPLAZ, MT_BABY, MT_BOSSTARGET, MT_BRUISER, MT_BRUISERSHOT, MT_CYBORG,
    MT_FATSHOT, MT_FATSO, MT_FIRE, MT_HEAD, MT_HEADSHOT, MT_KNIGHT, MT_PAIN, MT_PLAYER, MT_ROCKET,
    MT_SERGEANT, MT_SHADOWS, MT_SKULL, MT_SMOKE, MT_SPAWNFIRE, MT_SPAWNSHOT, MT_SPIDER, MT_TRACER,
    MT_TROOP, MT_TROOPSHOT, MT_UNDEAD, MT_VILE,
};
use crate::src::p_mobj::{statenum_t, ThinkerFn};
use crate::src::p_mobj::{
    MF_AMBUSH, MF_CORPSE, MF_FLOAT, MF_INFLOAT, MF_JUSTATTACKED, MF_JUSTHIT, MF_SHADOW,
    MF_SHOOTABLE, MF_SKULLFLY, MF_SOLID,
};
use crate::src::p_setup::SectorId;
use crate::src::p_setup::VertexId;
use crate::src::p_sight::P_CheckSight;
use crate::src::p_switch::P_UseSpecialLine;
use crate::src::r_main::R_PointToAngle2;
use crate::src::s_sound::S_StartSound;
use crate::src::sounds::{
    sfx_barexp, sfx_bgdth1, sfx_bgsit1, sfx_boscub, sfx_bosdth, sfx_bospit, sfx_bospn, sfx_bossit,
    sfx_bspwlk, sfx_claw, sfx_dbcls, sfx_dbload, sfx_dbopn, sfx_flame, sfx_flamst, sfx_hoof,
    sfx_manatk, sfx_metal, sfx_pdiehi, sfx_pistol, sfx_pldeth, sfx_podth1, sfx_posit1, sfx_shotgn,
    sfx_skepch, sfx_skeswg, sfx_slop, sfx_telept, sfx_vilatk,
};
use crate::src::tables::angle_t;
use crate::src::tables::finecosine;
use crate::src::tables::finesine;

use crate::src::doomdef::false_0;
use crate::src::doomdef::true_0;
use crate::src::doomdef::MAXPLAYERS;
use crate::src::doomdef::NULL;
use crate::src::game_state::GameState;
use crate::src::info::{S_BRAINEXPLODE1, S_NULL, S_VILE_HEAL1};
use crate::src::m_fixed::FRACUNIT;
use crate::src::p_maputl::MAPBLOCKSHIFT;
use crate::src::p_mobj::FLOATSPEED;
use crate::src::p_pspr::A_ReFire;
use crate::src::p_spec::ML_TWOSIDED;
use crate::src::tables::ANG180;
use crate::src::tables::ANG270;
use crate::src::tables::ANG90;
use crate::src::tables::ANGLETOFINESHIFT;

pub struct PEnemyState {
    pub soundtarget: *mut mobj_t,
    pub corpsehit: *mut mobj_t,
    pub vileobj: *mut mobj_t,
    pub viletryx: fixed_t,
    pub viletryy: fixed_t,
    pub braintargets: [*mut mobj_t; 32],
    pub numbraintargets: i32,
    pub braintargeton: i32,
    pub easy: i32,
}

impl PEnemyState {
    pub const fn new() -> Self {
        PEnemyState {
            soundtarget: ::core::ptr::null::<mobj_t>() as *mut mobj_t,
            corpsehit: ::core::ptr::null::<mobj_t>() as *mut mobj_t,
            vileobj: ::core::ptr::null::<mobj_t>() as *mut mobj_t,
            viletryx: 0,
            viletryy: 0,
            braintargets: [::core::ptr::null::<mobj_t>() as *mut mobj_t; 32],
            numbraintargets: 0,
            braintargeton: 0,
            easy: 0,
        }
    }
}

pub type dirtype_t = u32;
pub const NUMDIRS: dirtype_t = 9;
pub const DI_NODIR: dirtype_t = 8;
pub const DI_SOUTHEAST: dirtype_t = 7;
pub const DI_SOUTH: dirtype_t = 6;
pub const DI_SOUTHWEST: dirtype_t = 5;
pub const DI_WEST: dirtype_t = 4;
pub const DI_NORTHWEST: dirtype_t = 3;
pub const DI_NORTH: dirtype_t = 2;
pub const DI_NORTHEAST: dirtype_t = 1;
pub const DI_EAST: dirtype_t = 0;
pub const ML_SOUNDBLOCK: i32 = 64;
pub const MELEERANGE: i32 = 64 * FRACUNIT;
pub const MISSILERANGE: i32 = 32 * 64 as i32 * FRACUNIT;
#[no_mangle]
pub static opposite: [dirtype_t; 9] = [
    DI_WEST,
    DI_SOUTHWEST,
    DI_SOUTH,
    DI_SOUTHEAST,
    DI_EAST,
    DI_NORTHEAST,
    DI_NORTH,
    DI_NORTHWEST,
    DI_NODIR,
];
#[no_mangle]
pub static diags: [dirtype_t; 4] = [DI_NORTHWEST, DI_NORTHEAST, DI_SOUTHWEST, DI_SOUTHEAST];
pub unsafe fn P_RecursiveSound(
    state: &mut GameState,
    mut sec: *mut sector_t,
    mut soundblocks: i32,
) {
    let mut i: i32 = 0;
    let mut check: *mut line_t = ::core::ptr::null_mut::<line_t>();
    let mut other: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let sec_id = SectorId(sec.offset_from(state.p_setup.sectors.as_mut_ptr()) as i64 as u32);
    if (*sec).validcount == state.r_main.validcount
        && (*sec).soundtraversed <= soundblocks + 1 as i32
    {
        return;
    }
    (*sec).validcount = state.r_main.validcount;
    (*sec).soundtraversed = soundblocks + 1 as i32;
    let soundtarget_ptr = state.p_enemy.soundtarget;
    (*sec).soundtarget = if soundtarget_ptr.is_null() {
        None
    } else {
        Some((*soundtarget_ptr).id)
    };
    i = 0 as i32;
    while i < (*sec).linecount {
        check = *(*sec).lines.offset(i as isize) as *mut line_t;
        if !((*check).flags as i32 & ML_TWOSIDED == 0) {
            P_LineOpening(state, check);
            if !(state.p_maputl.openrange <= 0 as i32) {
                let other_id = if state.p_setup.sides[(*check).sidenum[0 as i32 as usize] as usize]
                    .sector
                    == sec_id
                {
                    state.p_setup.sides[(*check).sidenum[1 as i32 as usize] as usize].sector
                } else {
                    state.p_setup.sides[(*check).sidenum[0 as i32 as usize] as usize].sector
                };
                other = state.p_setup.sector_mut(other_id);
                if (*check).flags as i32 & ML_SOUNDBLOCK != 0 {
                    if soundblocks == 0 {
                        P_RecursiveSound(state, other, 1 as i32);
                    }
                } else {
                    P_RecursiveSound(state, other, soundblocks);
                }
            }
        }
        i += 1;
    }
}
pub unsafe fn P_NoiseAlert(
    state: &mut GameState,
    mut target: *mut mobj_t,
    mut emmiter: *mut mobj_t,
) {
    state.p_enemy.soundtarget = target;
    state.r_main.validcount += 1;
    let sec = state
        .p_setup
        .sector_mut(state.p_setup.subsectors[(*emmiter).subsector.0 as usize].sector);
    P_RecursiveSound(state, sec, 0 as i32);
}
pub unsafe fn P_CheckMeleeRange(state: &mut GameState, mut actor: *mut mobj_t) -> bool {
    let mut dist: fixed_t = 0;
    let pl = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(pl) => pl,
        None => return false,
    };
    dist = P_AproxDistance((*pl).x - (*actor).x, (*pl).y - (*actor).y);
    if dist >= MELEERANGE - 20 as i32 * FRACUNIT + (*state.info.mobjinfo_mut((*pl).type_0)).radius {
        return false;
    }
    if !P_CheckSight(state, actor, pl) {
        return false;
    }
    return true;
}
pub unsafe fn P_CheckMissileRange(state: &mut GameState, mut actor: *mut mobj_t) -> bool {
    let mut dist: fixed_t = 0;
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => return false,
    };
    if !P_CheckSight(state, actor, target) {
        return false;
    }
    if (*actor).flags & MF_JUSTHIT as i32 != 0 {
        (*actor).flags &= !(MF_JUSTHIT as i32);
        return true;
    }
    if (*actor).reactiontime != 0 {
        return false;
    }
    dist = (P_AproxDistance((*actor).x - (*target).x, (*actor).y - (*target).y) as i32
        - 64 as i32 * FRACUNIT) as fixed_t;
    if (*state.info.mobjinfo_mut((*actor).type_0)).meleestate == 0 {
        dist -= 128 as i32 * FRACUNIT;
    }
    dist >>= 16 as i32;
    if (*actor).type_0 as u32 == MT_VILE as i32 as u32 {
        if dist > 14 as i32 * 64 as i32 {
            return false;
        }
    }
    if (*actor).type_0 as u32 == MT_UNDEAD as i32 as u32 {
        if dist < 196 as i32 {
            return false;
        }
        dist >>= 1 as i32;
    }
    if (*actor).type_0 as u32 == MT_CYBORG as i32 as u32
        || (*actor).type_0 as u32 == MT_SPIDER as i32 as u32
        || (*actor).type_0 as u32 == MT_SKULL as i32 as u32
    {
        dist >>= 1 as i32;
    }
    if dist > 200 as i32 {
        dist = 200 as i32 as fixed_t;
    }
    if (*actor).type_0 as u32 == MT_CYBORG as i32 as u32 && dist > 160 as i32 {
        dist = 160 as i32 as fixed_t;
    }
    if P_Random(&mut state.m_random) < dist {
        return false;
    }
    return true;
}
#[no_mangle]
pub static xspeed: [fixed_t; 8] = [
    FRACUNIT,
    47000 as i32,
    0 as i32,
    -(47000 as i32),
    -FRACUNIT,
    -(47000 as i32),
    0 as i32,
    47000 as i32,
];
#[no_mangle]
pub static yspeed: [fixed_t; 8] = [
    0 as i32,
    47000 as i32,
    FRACUNIT,
    47000 as i32,
    0 as i32,
    -(47000 as i32),
    -FRACUNIT,
    -(47000 as i32),
];
pub unsafe fn P_Move(state: &mut GameState, mut actor: *mut mobj_t) -> bool {
    let mut tryx: fixed_t = 0;
    let mut tryy: fixed_t = 0;
    let mut ld: *mut line_t = ::core::ptr::null_mut::<line_t>();
    let mut try_ok: bool;
    let mut good: bool;
    if (*actor).movedir == DI_NODIR as i32 {
        return false;
    }
    if (*actor).movedir as u32 >= 8 as u32 {
        I_Error("Weird actor->movedir!");
    }
    tryx = (*actor).x + (*state.info.mobjinfo_mut((*actor).type_0)).speed as fixed_t * xspeed[(*actor).movedir as usize];
    tryy = (*actor).y + (*state.info.mobjinfo_mut((*actor).type_0)).speed as fixed_t * yspeed[(*actor).movedir as usize];
    try_ok = P_TryMove(state, actor, tryx, tryy);
    if !try_ok {
        if (*actor).flags & MF_FLOAT as i32 != 0 && state.p_map.floatok {
            if (*actor).z < state.p_map.tmfloorz {
                (*actor).z += FLOATSPEED;
            } else {
                (*actor).z -= FLOATSPEED;
            }
            (*actor).flags |= MF_INFLOAT as i32;
            return true;
        }
        if state.p_map.numspechit == 0 {
            return false;
        }
        (*actor).movedir = DI_NODIR as i32;
        good = false;
        loop {
            let fresh0 = state.p_map.numspechit;
            state.p_map.numspechit = state.p_map.numspechit - 1;
            if !(fresh0 != 0) {
                break;
            }
            ld = state.p_map.spechit[state.p_map.numspechit as usize];
            if P_UseSpecialLine(state, actor, ld, 0 as i32) {
                good = true;
            }
        }
        return good;
    } else {
        (*actor).flags &= !(MF_INFLOAT as i32);
    }
    if (*actor).flags & MF_FLOAT as i32 == 0 {
        (*actor).z = (*actor).floorz;
    }
    return true;
}
pub unsafe fn P_TryWalk(state: &mut GameState, mut actor: *mut mobj_t) -> bool {
    if !P_Move(state, actor) {
        return false;
    }
    (*actor).movecount = P_Random(&mut state.m_random) & 15 as i32;
    return true;
}
pub unsafe fn P_NewChaseDir(state: &mut GameState, mut actor: *mut mobj_t) {
    let mut deltax: fixed_t = 0;
    let mut deltay: fixed_t = 0;
    let mut d: [dirtype_t; 3] = [DI_EAST; 3];
    let mut tdir: i32 = 0;
    let mut olddir: dirtype_t = DI_EAST;
    let mut turnaround: dirtype_t = DI_EAST;
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => {
            I_Error("P_NewChaseDir: called with no target");
        }
    };
    olddir = (*actor).movedir as dirtype_t;
    turnaround = opposite[olddir as usize];
    deltax = (*target).x - (*actor).x;
    deltay = (*target).y - (*actor).y;
    if deltax > 10 as i32 * FRACUNIT {
        d[1 as i32 as usize] = DI_EAST;
    } else if deltax < -(10 as i32) * FRACUNIT {
        d[1 as i32 as usize] = DI_WEST;
    } else {
        d[1 as i32 as usize] = DI_NODIR;
    }
    if deltay < -(10 as i32) * FRACUNIT {
        d[2 as i32 as usize] = DI_SOUTH;
    } else if deltay > 10 as i32 * FRACUNIT {
        d[2 as i32 as usize] = DI_NORTH;
    } else {
        d[2 as i32 as usize] = DI_NODIR;
    }
    if d[1 as i32 as usize] as u32 != DI_NODIR as i32 as u32
        && d[2 as i32 as usize] as u32 != DI_NODIR as i32 as u32
    {
        (*actor).movedir = diags
            [((((deltay < 0 as i32) as i32) << 1 as i32) + (deltax > 0 as i32) as i32) as usize]
            as i32;
        if (*actor).movedir != turnaround as i32 && P_TryWalk(state, actor) {
            return;
        }
    }
    if P_Random(&mut state.m_random) > 200 as i32 || (deltay as i32).abs() > (deltax as i32).abs() {
        tdir = d[1 as i32 as usize] as i32;
        d[1 as i32 as usize] = d[2 as i32 as usize];
        d[2 as i32 as usize] = tdir as dirtype_t;
    }
    if d[1 as i32 as usize] as u32 == turnaround as u32 {
        d[1 as i32 as usize] = DI_NODIR;
    }
    if d[2 as i32 as usize] as u32 == turnaround as u32 {
        d[2 as i32 as usize] = DI_NODIR;
    }
    if d[1 as i32 as usize] as u32 != DI_NODIR as i32 as u32 {
        (*actor).movedir = d[1 as i32 as usize] as i32;
        if P_TryWalk(state, actor) {
            return;
        }
    }
    if d[2 as i32 as usize] as u32 != DI_NODIR as i32 as u32 {
        (*actor).movedir = d[2 as i32 as usize] as i32;
        if P_TryWalk(state, actor) {
            return;
        }
    }
    if olddir as u32 != DI_NODIR as i32 as u32 {
        (*actor).movedir = olddir as i32;
        if P_TryWalk(state, actor) {
            return;
        }
    }
    if P_Random(&mut state.m_random) & 1 as i32 != 0 {
        tdir = DI_EAST as i32;
        while tdir <= DI_SOUTHEAST as i32 {
            if tdir != turnaround as i32 {
                (*actor).movedir = tdir;
                if P_TryWalk(state, actor) {
                    return;
                }
            }
            tdir += 1;
        }
    } else {
        tdir = DI_SOUTHEAST as i32;
        while tdir != DI_EAST as i32 - 1 as i32 {
            if tdir != turnaround as i32 {
                (*actor).movedir = tdir;
                if P_TryWalk(state, actor) {
                    return;
                }
            }
            tdir -= 1;
        }
    }
    if turnaround as u32 != DI_NODIR as i32 as u32 {
        (*actor).movedir = turnaround as i32;
        if P_TryWalk(state, actor) {
            return;
        }
    }
    (*actor).movedir = DI_NODIR as i32;
}
pub unsafe fn P_LookForPlayers(
    state: &mut GameState,
    mut actor: *mut mobj_t,
    mut allaround: bool,
) -> bool {
    let mut c: i32 = 0;
    let mut stop: i32 = 0;
    let mut player: *mut player_t = ::core::ptr::null_mut::<player_t>();
    let mut an: angle_t = 0;
    let mut dist: fixed_t = 0;
    c = 0 as i32;
    stop = (*actor).lastlook - 1 as i32 & 3 as i32;
    let mut current_block_9: u64;
    loop {
        if !(state.g_game.playeringame[(*actor).lastlook as usize] == 0) {
            let fresh1 = c;
            c = c + 1;
            if fresh1 == 2 as i32 || (*actor).lastlook == stop {
                return false;
            }
            player = (&raw mut state.g_game.players as *mut player_t)
                .offset((*actor).lastlook as isize) as *mut player_t;
            if !((*player).health <= 0 as i32) {
                if P_CheckSight(state, actor, (*player).mo) {
                    if !allaround {
                        an = R_PointToAngle2(
                            state,
                            (*actor).x,
                            (*actor).y,
                            (*(*player).mo).x,
                            (*(*player).mo).y,
                        )
                        .wrapping_sub((*actor).angle);
                        if an > ANG90 as angle_t && an < ANG270 {
                            dist = P_AproxDistance(
                                (*(*player).mo).x - (*actor).x,
                                (*(*player).mo).y - (*actor).y,
                            );
                            if dist > MELEERANGE {
                                current_block_9 = 4644295000439058019;
                            } else {
                                current_block_9 = 8236137900636309791;
                            }
                        } else {
                            current_block_9 = 8236137900636309791;
                        }
                    } else {
                        current_block_9 = 8236137900636309791;
                    }
                    match current_block_9 {
                        4644295000439058019 => {}
                        _ => {
                            (*actor).target = Some((*(*player).mo).id);
                            return true;
                        }
                    }
                }
            }
        }
        (*actor).lastlook = (*actor).lastlook + 1 as i32 & 3 as i32;
    }
}
pub unsafe fn A_KeenDie(state: &mut GameState, id: MobjId) {
    let mo = state.p_mobj.mobj_get(id).unwrap();
    let mut th: *mut thinker_t = ::core::ptr::null_mut::<thinker_t>();
    let mut mo2: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut junk: line_t = line_s {
        v1: VertexId(0),
        v2: VertexId(0),
        dx: 0,
        dy: 0,
        flags: 0,
        special: 0,
        tag: 0,
        sidenum: [0; 2],
        bbox: [0; 4],
        slopetype: ST_HORIZONTAL,
        frontsector: None,
        backsector: None,
        validcount: 0,
    };
    A_Fall(state, (*mo).id);
    let mut cursor = state.p_tick.head();
    while let Some(id) = cursor {
        th = state.p_tick.raw(id);
        if matches!((*th).function, ThinkerFn::Mobj(_)) {
            mo2 = th as *mut mobj_t;
            if mo2 != mo && (*mo2).type_0 as u32 == (*mo).type_0 as u32 && (*mo2).health > 0 as i32
            {
                return;
            }
        }
        cursor = state.p_tick.next(id);
    }
    junk.tag = 666 as i16;
    EV_DoDoor(state, &raw mut junk, vld_open);
}
pub unsafe fn A_Look(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut current_block: u64;
    let mut targ: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    (*actor).threshold = 0 as i32;
    targ = (*state
        .p_setup
        .sector_mut(state.p_setup.subsectors[(*actor).subsector.0 as usize].sector))
    .soundtarget
        .and_then(|id| state.p_mobj.mobj_get(id))
        .unwrap_or(::core::ptr::null_mut());
    if !targ.is_null() && (*targ).flags & MF_SHOOTABLE as i32 != 0 {
        (*actor).target = Some((*targ).id);
        if (*actor).flags & MF_AMBUSH as i32 != 0 {
            if P_CheckSight(state, actor, targ) {
                current_block = 10571674169298881693;
            } else {
                current_block = 15619007995458559411;
            }
        } else {
            current_block = 10571674169298881693;
        }
    } else {
        current_block = 15619007995458559411;
    }
    match current_block {
        15619007995458559411 => {
            if !P_LookForPlayers(state, actor, false) {
                return;
            }
        }
        _ => {}
    }
    if (*state.info.mobjinfo_mut((*actor).type_0)).seesound != 0 {
        let mut sound: i32 = 0;
        match (*state.info.mobjinfo_mut((*actor).type_0)).seesound {
            36 | 37 | 38 => {
                sound = sfx_posit1 as i32 + P_Random(&mut state.m_random) % 3 as i32;
            }
            39 | 40 => {
                sound = sfx_bgsit1 as i32 + P_Random(&mut state.m_random) % 2 as i32;
            }
            _ => {
                sound = (*state.info.mobjinfo_mut((*actor).type_0)).seesound;
            }
        }
        if (*actor).type_0 as u32 == MT_SPIDER as i32 as u32
            || (*actor).type_0 as u32 == MT_CYBORG as i32 as u32
        {
            S_StartSound(state, NULL, sound);
        } else {
            S_StartSound(state, actor as *mut ::core::ffi::c_void, sound);
        }
    }
    let seestate = (*state.info.mobjinfo_mut((*actor).type_0)).seestate as statenum_t;
    P_SetMobjState(state, actor, seestate);
}
pub unsafe fn A_Chase(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut delta: i32 = 0;
    if (*actor).reactiontime != 0 {
        (*actor).reactiontime -= 1;
    }
    let target = (*actor).target.and_then(|id| state.p_mobj.mobj_get(id));
    if (*actor).threshold != 0 {
        if target.is_none() || (*target.unwrap()).health <= 0 as i32 {
            (*actor).threshold = 0 as i32;
        } else {
            (*actor).threshold -= 1;
        }
    }
    if (*actor).movedir < 8 as i32 {
        (*actor).angle &= ((7 as i32) << 29 as i32) as angle_t;
        delta = (*actor)
            .angle
            .wrapping_sub(((*actor).movedir << 29 as i32) as angle_t) as i32;
        if delta > 0 as i32 {
            (*actor).angle = (*actor).angle.wrapping_sub((ANG90 / 2 as i32) as angle_t);
        } else if delta < 0 as i32 {
            (*actor).angle = (*actor).angle.wrapping_add((ANG90 / 2 as i32) as angle_t);
        }
    }
    if target.is_none() || (*target.unwrap()).flags & MF_SHOOTABLE as i32 == 0 {
        if P_LookForPlayers(state, actor, true) {
            return;
        }
        let spawnstate = (*state.info.mobjinfo_mut((*actor).type_0)).spawnstate as statenum_t;
        P_SetMobjState(state, actor, spawnstate);
        return;
    }
    if (*actor).flags & MF_JUSTATTACKED as i32 != 0 {
        (*actor).flags &= !(MF_JUSTATTACKED as i32);
        if state.g_game.gameskill as i32 != sk_nightmare as i32 && !state.d_main.fastparm {
            P_NewChaseDir(state, actor);
        }
        return;
    }
    let actor_info = state.info.mobjinfo_mut((*actor).type_0);
    if (*actor_info).meleestate != 0 && P_CheckMeleeRange(state, actor) {
        let attacksound = (*state.info.mobjinfo_mut((*actor).type_0)).attacksound;
        if attacksound != 0 {
            S_StartSound(state, actor as *mut ::core::ffi::c_void, attacksound);
        }
        let meleestate = (*state.info.mobjinfo_mut((*actor).type_0)).meleestate as statenum_t;
        P_SetMobjState(state, actor, meleestate);
        return;
    }
    if (*state.info.mobjinfo_mut((*actor).type_0)).missilestate != 0 {
        if !((state.g_game.gameskill as i32) < sk_nightmare as i32
            && !state.d_main.fastparm
            && (*actor).movecount != 0)
        {
            if P_CheckMissileRange(state, actor) {
                let missilestate = (*state.info.mobjinfo_mut((*actor).type_0)).missilestate as statenum_t;
                P_SetMobjState(state, actor, missilestate);
                (*actor).flags |= MF_JUSTATTACKED as i32;
                return;
            }
        }
    }
    if state.g_game.netgame
        && (*actor).threshold == 0
        && !P_CheckSight(state, actor, target.unwrap())
    {
        if P_LookForPlayers(state, actor, true) {
            return;
        }
    }
    (*actor).movecount -= 1;
    if (*actor).movecount < 0 as i32 || !P_Move(state, actor) {
        P_NewChaseDir(state, actor);
    }
    let activesound = (*state.info.mobjinfo_mut((*actor).type_0)).activesound;
    if activesound != 0 && P_Random(&mut state.m_random) < 3 as i32 {
        S_StartSound(state, actor as *mut ::core::ffi::c_void, activesound);
    }
}
pub unsafe fn A_FaceTarget(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => return,
    };
    (*actor).flags &= !(MF_AMBUSH as i32);
    (*actor).angle = R_PointToAngle2(state, (*actor).x, (*actor).y, (*target).x, (*target).y);
    if (*target).flags & MF_SHADOW as i32 != 0 {
        (*actor).angle = (*actor).angle.wrapping_add(
            (P_Random(&mut state.m_random) - P_Random(&mut state.m_random) << 21 as i32) as angle_t,
        );
    }
}
pub unsafe fn A_PosAttack(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut angle: i32 = 0;
    let mut damage: i32 = 0;
    let mut slope: i32 = 0;
    if (*actor).target.is_none() {
        return;
    }
    A_FaceTarget(state, (*actor).id);
    angle = (*actor).angle as i32;
    slope = P_AimLineAttack(state, actor, angle as angle_t, MISSILERANGE) as i32;
    S_StartSound(
        state,
        actor as *mut ::core::ffi::c_void,
        sfx_pistol as i32,
    );
    angle += P_Random(&mut state.m_random) - P_Random(&mut state.m_random) << 20 as i32;
    damage = (P_Random(&mut state.m_random) % 5 as i32 + 1 as i32) * 3 as i32;
    P_LineAttack(
        state,
        actor,
        angle as angle_t,
        MISSILERANGE,
        slope as fixed_t,
        damage,
    );
}
pub unsafe fn A_SPosAttack(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut i: i32 = 0;
    let mut angle: i32 = 0;
    let mut bangle: i32 = 0;
    let mut damage: i32 = 0;
    let mut slope: i32 = 0;
    if (*actor).target.is_none() {
        return;
    }
    S_StartSound(
        state,
        actor as *mut ::core::ffi::c_void,
        sfx_shotgn as i32,
    );
    A_FaceTarget(state, (*actor).id);
    bangle = (*actor).angle as i32;
    slope = P_AimLineAttack(state, actor, bangle as angle_t, MISSILERANGE) as i32;
    i = 0 as i32;
    while i < 3 as i32 {
        angle =
            bangle + (P_Random(&mut state.m_random) - P_Random(&mut state.m_random) << 20 as i32);
        damage = (P_Random(&mut state.m_random) % 5 as i32 + 1 as i32) * 3 as i32;
        P_LineAttack(
            state,
            actor,
            angle as angle_t,
            MISSILERANGE,
            slope as fixed_t,
            damage,
        );
        i += 1;
    }
}
pub unsafe fn A_CPosAttack(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut angle: i32 = 0;
    let mut bangle: i32 = 0;
    let mut damage: i32 = 0;
    let mut slope: i32 = 0;
    if (*actor).target.is_none() {
        return;
    }
    S_StartSound(
        state,
        actor as *mut ::core::ffi::c_void,
        sfx_shotgn as i32,
    );
    A_FaceTarget(state, (*actor).id);
    bangle = (*actor).angle as i32;
    slope = P_AimLineAttack(state, actor, bangle as angle_t, MISSILERANGE) as i32;
    angle = bangle + (P_Random(&mut state.m_random) - P_Random(&mut state.m_random) << 20 as i32);
    damage = (P_Random(&mut state.m_random) % 5 as i32 + 1 as i32) * 3 as i32;
    P_LineAttack(
        state,
        actor,
        angle as angle_t,
        MISSILERANGE,
        slope as fixed_t,
        damage,
    );
}
pub unsafe fn A_CPosRefire(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    A_FaceTarget(state, (*actor).id);
    if P_Random(&mut state.m_random) < 40 as i32 {
        return;
    }
    let target = (*actor).target.and_then(|id| state.p_mobj.mobj_get(id));
    if target.is_none()
        || (*target.unwrap()).health <= 0 as i32
        || !P_CheckSight(state, actor, target.unwrap())
    {
        let seestate = (*state.info.mobjinfo_mut((*actor).type_0)).seestate as statenum_t;
        P_SetMobjState(state, actor, seestate);
    }
}
pub unsafe fn A_SpidRefire(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    A_FaceTarget(state, (*actor).id);
    if P_Random(&mut state.m_random) < 10 as i32 {
        return;
    }
    let target = (*actor).target.and_then(|id| state.p_mobj.mobj_get(id));
    if target.is_none()
        || (*target.unwrap()).health <= 0 as i32
        || !P_CheckSight(state, actor, target.unwrap())
    {
        let seestate = (*state.info.mobjinfo_mut((*actor).type_0)).seestate as statenum_t;
        P_SetMobjState(state, actor, seestate);
    }
}
pub unsafe fn A_BspiAttack(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => return,
    };
    A_FaceTarget(state, (*actor).id);
    P_SpawnMissile(state, actor, target, MT_ARACHPLAZ);
}
pub unsafe fn A_TroopAttack(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut damage: i32 = 0;
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => return,
    };
    A_FaceTarget(state, (*actor).id);
    if P_CheckMeleeRange(state, actor) {
        S_StartSound(
            state,
            actor as *mut ::core::ffi::c_void,
            sfx_claw as i32,
        );
        damage = (P_Random(&mut state.m_random) % 8 as i32 + 1 as i32) * 3 as i32;
        P_DamageMobj(state, target, actor, actor, damage);
        return;
    }
    P_SpawnMissile(state, actor, target, MT_TROOPSHOT);
}
pub unsafe fn A_SargAttack(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut damage: i32 = 0;
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => return,
    };
    A_FaceTarget(state, (*actor).id);
    if P_CheckMeleeRange(state, actor) {
        damage = (P_Random(&mut state.m_random) % 10 as i32 + 1 as i32) * 4 as i32;
        P_DamageMobj(state, target, actor, actor, damage);
    }
}
pub unsafe fn A_HeadAttack(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut damage: i32 = 0;
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => return,
    };
    A_FaceTarget(state, (*actor).id);
    if P_CheckMeleeRange(state, actor) {
        damage = (P_Random(&mut state.m_random) % 6 as i32 + 1 as i32) * 10 as i32;
        P_DamageMobj(state, target, actor, actor, damage);
        return;
    }
    P_SpawnMissile(state, actor, target, MT_HEADSHOT);
}
pub unsafe fn A_CyberAttack(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => return,
    };
    A_FaceTarget(state, (*actor).id);
    P_SpawnMissile(state, actor, target, MT_ROCKET);
}
pub unsafe fn A_BruisAttack(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut damage: i32 = 0;
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => return,
    };
    if P_CheckMeleeRange(state, actor) {
        S_StartSound(
            state,
            actor as *mut ::core::ffi::c_void,
            sfx_claw as i32,
        );
        damage = (P_Random(&mut state.m_random) % 8 as i32 + 1 as i32) * 10 as i32;
        P_DamageMobj(state, target, actor, actor, damage);
        return;
    }
    P_SpawnMissile(state, actor, target, MT_BRUISERSHOT);
}
pub unsafe fn A_SkelMissile(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut mo: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => return,
    };
    A_FaceTarget(state, (*actor).id);
    (*actor).z += 16 as i32 * FRACUNIT;
    mo = P_SpawnMissile(state, actor, target, MT_TRACER);
    (*actor).z -= 16 as i32 * FRACUNIT;
    (*mo).x += (*mo).momx;
    (*mo).y += (*mo).momy;
    (*mo).tracer = (*actor).target;
}
#[no_mangle]
pub static TRACEANGLE: i32 = 0xc000000;
pub unsafe fn A_Tracer(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut exact: angle_t = 0;
    let mut dist: fixed_t = 0;
    let mut slope: fixed_t = 0;
    let mut dest: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut th: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    if state.d_loop.gametic & 3 as i32 != 0 {
        return;
    }
    P_SpawnPuff(state, (*actor).x, (*actor).y, (*actor).z);
    th = P_SpawnMobj(
        state,
        (*actor).x - (*actor).momx,
        (*actor).y - (*actor).momy,
        (*actor).z,
        MT_SMOKE,
    );
    (*th).momz = FRACUNIT as fixed_t;
    (*th).tics -= P_Random(&mut state.m_random) & 3 as i32;
    if (*th).tics < 1 as i32 {
        (*th).tics = 1 as i32;
    }
    dest = (*actor)
        .tracer
        .and_then(|id| state.p_mobj.mobj_get(id))
        .unwrap_or(::core::ptr::null_mut());
    if dest.is_null() || (*dest).health <= 0 as i32 {
        return;
    }
    exact = R_PointToAngle2(state, (*actor).x, (*actor).y, (*dest).x, (*dest).y);
    if exact != (*actor).angle {
        if exact.wrapping_sub((*actor).angle) > 0x80000000 as u32 {
            (*actor).angle = (*actor).angle.wrapping_sub(TRACEANGLE as angle_t);
            if exact.wrapping_sub((*actor).angle) < 0x80000000 as u32 {
                (*actor).angle = exact;
            }
        } else {
            (*actor).angle = (*actor).angle.wrapping_add(TRACEANGLE as angle_t);
            if exact.wrapping_sub((*actor).angle) > 0x80000000 as u32 {
                (*actor).angle = exact;
            }
        }
    }
    exact = (*actor).angle >> ANGLETOFINESHIFT;
    (*actor).momx = FixedMul(
        (*state.info.mobjinfo_mut((*actor).type_0)).speed as fixed_t,
        finecosine[exact as isize],
    );
    (*actor).momy = FixedMul((*state.info.mobjinfo_mut((*actor).type_0)).speed as fixed_t, finesine[exact as usize]);
    dist = P_AproxDistance((*dest).x - (*actor).x, (*dest).y - (*actor).y);
    dist = (dist as i32 / (*state.info.mobjinfo_mut((*actor).type_0)).speed) as fixed_t;
    if dist < 1 as i32 {
        dist = 1 as i32 as fixed_t;
    }
    slope = ((*dest).z + 40 as fixed_t * FRACUNIT - (*actor).z) / dist;
    if slope < (*actor).momz {
        (*actor).momz -= FRACUNIT / 8 as i32;
    } else {
        (*actor).momz += FRACUNIT / 8 as i32;
    };
}
pub unsafe fn A_SkelWhoosh(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    if (*actor).target.is_none() {
        return;
    }
    A_FaceTarget(state, (*actor).id);
    S_StartSound(
        state,
        actor as *mut ::core::ffi::c_void,
        sfx_skeswg as i32,
    );
}
pub unsafe fn A_SkelFist(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut damage: i32 = 0;
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => return,
    };
    A_FaceTarget(state, (*actor).id);
    if P_CheckMeleeRange(state, actor) {
        damage = (P_Random(&mut state.m_random) % 10 as i32 + 1 as i32) * 6 as i32;
        S_StartSound(
            state,
            actor as *mut ::core::ffi::c_void,
            sfx_skepch as i32,
        );
        P_DamageMobj(state, target, actor, actor, damage);
    }
}
#[no_mangle]
pub unsafe fn PIT_VileCheck(state: &mut GameState, mut thing_id: MobjId) -> boolean {
    let thing = state.p_mobj.mobj_get(thing_id).unwrap();
    let mut maxdist: i32 = 0;
    let mut check: bool = false;
    if (*thing).flags & MF_CORPSE as i32 == 0 {
        return true_0 as boolean;
    }
    if (*thing).tics != -(1 as i32) {
        return true_0 as boolean;
    }
    if (*state.info.mobjinfo_mut((*thing).type_0)).raisestate == S_NULL as i32 {
        return true_0 as boolean;
    }
    maxdist = (*state.info.mobjinfo_mut((*thing).type_0)).radius + state.info.mobjinfo[MT_VILE as i32 as usize].radius;
    if ((*thing).x as i32 - state.p_enemy.viletryx as i32).abs() > maxdist
        || ((*thing).y as i32 - state.p_enemy.viletryy as i32).abs() > maxdist
    {
        return true_0 as boolean;
    }
    state.p_enemy.corpsehit = thing;
    (*state.p_enemy.corpsehit).momy = 0 as i32 as fixed_t;
    (*state.p_enemy.corpsehit).momx = (*state.p_enemy.corpsehit).momy;
    (*state.p_enemy.corpsehit).height <<= 2 as i32;
    check = P_CheckPosition(
        state,
        state.p_enemy.corpsehit,
        (*state.p_enemy.corpsehit).x,
        (*state.p_enemy.corpsehit).y,
    );
    (*state.p_enemy.corpsehit).height >>= 2 as i32;
    if !check {
        return true_0 as boolean;
    }
    return false_0 as boolean;
}
pub unsafe fn A_VileChase(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut xl: i32 = 0;
    let mut xh: i32 = 0;
    let mut yl: i32 = 0;
    let mut yh: i32 = 0;
    let mut bx: i32 = 0;
    let mut by: i32 = 0;
    let mut info: *mut mobjinfo_t = ::core::ptr::null_mut::<mobjinfo_t>();
    let mut temp: Option<MobjId> = None;
    if (*actor).movedir != DI_NODIR as i32 {
        state.p_enemy.viletryx =
            (*actor).x + (*state.info.mobjinfo_mut((*actor).type_0)).speed as fixed_t * xspeed[(*actor).movedir as usize];
        state.p_enemy.viletryy =
            (*actor).y + (*state.info.mobjinfo_mut((*actor).type_0)).speed as fixed_t * yspeed[(*actor).movedir as usize];
        xl = state.p_enemy.viletryx as i32
            - state.p_setup.bmaporgx as i32
            - 32 as i32 * FRACUNIT * 2 as i32
            >> MAPBLOCKSHIFT;
        xh = state.p_enemy.viletryx as i32 - state.p_setup.bmaporgx as i32
            + 32 as i32 * FRACUNIT * 2 as i32
            >> MAPBLOCKSHIFT;
        yl = state.p_enemy.viletryy as i32
            - state.p_setup.bmaporgy as i32
            - 32 as i32 * FRACUNIT * 2 as i32
            >> MAPBLOCKSHIFT;
        yh = state.p_enemy.viletryy as i32 - state.p_setup.bmaporgy as i32
            + 32 as i32 * FRACUNIT * 2 as i32
            >> MAPBLOCKSHIFT;
        state.p_enemy.vileobj = actor;
        bx = xl;
        while bx <= xh {
            by = yl;
            while by <= yh {
                if !P_BlockThingsIterator(
                    state,
                    bx,
                    by,
                    Some(PIT_VileCheck as unsafe fn(&mut GameState, MobjId) -> boolean),
                ) {
                    temp = (*actor).target;
                    (*actor).target = Some((*state.p_enemy.corpsehit).id);
                    A_FaceTarget(state, (*actor).id);
                    (*actor).target = temp;
                    P_SetMobjState(state, actor, S_VILE_HEAL1);
                    S_StartSound(
                        state,
                        state.p_enemy.corpsehit as *mut ::core::ffi::c_void,
                        sfx_slop as i32,
                    );
                    info = state.info.mobjinfo_mut((*state.p_enemy.corpsehit).type_0);
                    P_SetMobjState(
                        state,
                        state.p_enemy.corpsehit,
                        (*info).raisestate as statenum_t,
                    );
                    (*state.p_enemy.corpsehit).height <<= 2 as i32;
                    (*state.p_enemy.corpsehit).flags = (*info).flags;
                    (*state.p_enemy.corpsehit).health = (*info).spawnhealth;
                    (*state.p_enemy.corpsehit).target = None;
                    return;
                }
                by += 1;
            }
            bx += 1;
        }
    }
    A_Chase(state, (*actor).id);
}
pub unsafe fn A_VileStart(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    S_StartSound(
        state,
        actor as *mut ::core::ffi::c_void,
        sfx_vilatk as i32,
    );
}
pub unsafe fn A_StartFire(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    S_StartSound(
        state,
        actor as *mut ::core::ffi::c_void,
        sfx_flamst as i32,
    );
    A_Fire(state, (*actor).id);
}
pub unsafe fn A_FireCrackle(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    S_StartSound(
        state,
        actor as *mut ::core::ffi::c_void,
        sfx_flame as i32,
    );
    A_Fire(state, (*actor).id);
}
pub unsafe fn A_Fire(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut dest: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut target: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut an: u32 = 0;
    dest = (*actor)
        .tracer
        .and_then(|id| state.p_mobj.mobj_get(id))
        .unwrap_or(::core::ptr::null_mut());
    if dest.is_null() {
        return;
    }
    let target_subst = (*actor)
        .target
        .and_then(|id| state.p_mobj.mobj_get(id))
        .unwrap_or(::core::ptr::null_mut());
    target = P_SubstNullMobj(&mut state.p_mobj, target_subst);
    if !P_CheckSight(state, target, dest) {
        return;
    }
    an = ((*dest).angle >> ANGLETOFINESHIFT) as u32;
    P_UnsetThingPosition(state, actor);
    (*actor).x = (*dest).x + FixedMul(24 as fixed_t * FRACUNIT, finecosine[an as isize]);
    (*actor).y = (*dest).y + FixedMul(24 as fixed_t * FRACUNIT, finesine[an as usize]);
    (*actor).z = (*dest).z;
    P_SetThingPosition(state, actor);
}
pub unsafe fn A_VileTarget(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut fog: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => return,
    };
    A_FaceTarget(state, (*actor).id);
    fog = P_SpawnMobj(state, (*target).x, (*target).x, (*target).z, MT_FIRE);
    (*actor).tracer = Some((*fog).id);
    (*fog).target = Some((*actor).id);
    (*fog).tracer = (*actor).target;
    A_Fire(state, (*fog).id);
}
pub unsafe fn A_VileAttack(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut fire: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut an: i32 = 0;
    let target = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(target) => target,
        None => return,
    };
    A_FaceTarget(state, (*actor).id);
    if !P_CheckSight(state, actor, target) {
        return;
    }
    S_StartSound(
        state,
        actor as *mut ::core::ffi::c_void,
        sfx_barexp as i32,
    );
    P_DamageMobj(state, target, actor, actor, 20 as i32);
    (*target).momz = (1000 as i32 * FRACUNIT / (*state.info.mobjinfo_mut((*target).type_0)).mass) as fixed_t;
    an = ((*actor).angle >> ANGLETOFINESHIFT) as i32;
    fire = (*actor)
        .tracer
        .and_then(|id| state.p_mobj.mobj_get(id))
        .unwrap_or(::core::ptr::null_mut());
    if fire.is_null() {
        return;
    }
    (*fire).x = (*target).x - FixedMul(24 as fixed_t * FRACUNIT, finecosine[an as isize]);
    (*fire).y = (*target).y - FixedMul(24 as fixed_t * FRACUNIT, finesine[an as usize]);
    P_RadiusAttack(state, fire, actor, 70 as i32);
}
pub const FATSPREAD: i32 = ANG90 / 8 as i32;
pub unsafe fn A_FatRaise(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    A_FaceTarget(state, (*actor).id);
    S_StartSound(
        state,
        actor as *mut ::core::ffi::c_void,
        sfx_manatk as i32,
    );
}
pub unsafe fn A_FatAttack1(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut mo: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut target: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut an: i32 = 0;
    A_FaceTarget(state, (*actor).id);
    (*actor).angle = (*actor).angle.wrapping_add(FATSPREAD as angle_t);
    let target_subst = (*actor)
        .target
        .and_then(|id| state.p_mobj.mobj_get(id))
        .unwrap_or(::core::ptr::null_mut());
    target = P_SubstNullMobj(&mut state.p_mobj, target_subst);
    P_SpawnMissile(state, actor, target, MT_FATSHOT);
    mo = P_SpawnMissile(state, actor, target, MT_FATSHOT);
    (*mo).angle = (*mo).angle.wrapping_add(FATSPREAD as angle_t);
    an = ((*mo).angle >> ANGLETOFINESHIFT) as i32;
    (*mo).momx = FixedMul((*state.info.mobjinfo_mut((*mo).type_0)).speed as fixed_t, finecosine[an as isize]);
    (*mo).momy = FixedMul((*state.info.mobjinfo_mut((*mo).type_0)).speed as fixed_t, finesine[an as usize]);
}
pub unsafe fn A_FatAttack2(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut mo: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut target: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut an: i32 = 0;
    A_FaceTarget(state, (*actor).id);
    (*actor).angle = (*actor).angle.wrapping_sub(FATSPREAD as angle_t);
    let target_subst = (*actor)
        .target
        .and_then(|id| state.p_mobj.mobj_get(id))
        .unwrap_or(::core::ptr::null_mut());
    target = P_SubstNullMobj(&mut state.p_mobj, target_subst);
    P_SpawnMissile(state, actor, target, MT_FATSHOT);
    mo = P_SpawnMissile(state, actor, target, MT_FATSHOT);
    (*mo).angle = (*mo).angle.wrapping_sub((FATSPREAD * 2 as i32) as angle_t);
    an = ((*mo).angle >> ANGLETOFINESHIFT) as i32;
    (*mo).momx = FixedMul((*state.info.mobjinfo_mut((*mo).type_0)).speed as fixed_t, finecosine[an as isize]);
    (*mo).momy = FixedMul((*state.info.mobjinfo_mut((*mo).type_0)).speed as fixed_t, finesine[an as usize]);
}
pub unsafe fn A_FatAttack3(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut mo: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut target: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut an: i32 = 0;
    A_FaceTarget(state, (*actor).id);
    let target_subst = (*actor)
        .target
        .and_then(|id| state.p_mobj.mobj_get(id))
        .unwrap_or(::core::ptr::null_mut());
    target = P_SubstNullMobj(&mut state.p_mobj, target_subst);
    mo = P_SpawnMissile(state, actor, target, MT_FATSHOT);
    (*mo).angle = (*mo).angle.wrapping_sub((FATSPREAD / 2 as i32) as angle_t);
    an = ((*mo).angle >> ANGLETOFINESHIFT) as i32;
    (*mo).momx = FixedMul((*state.info.mobjinfo_mut((*mo).type_0)).speed as fixed_t, finecosine[an as isize]);
    (*mo).momy = FixedMul((*state.info.mobjinfo_mut((*mo).type_0)).speed as fixed_t, finesine[an as usize]);
    mo = P_SpawnMissile(state, actor, target, MT_FATSHOT);
    (*mo).angle = (*mo).angle.wrapping_add((FATSPREAD / 2 as i32) as angle_t);
    an = ((*mo).angle >> ANGLETOFINESHIFT) as i32;
    (*mo).momx = FixedMul((*state.info.mobjinfo_mut((*mo).type_0)).speed as fixed_t, finecosine[an as isize]);
    (*mo).momy = FixedMul((*state.info.mobjinfo_mut((*mo).type_0)).speed as fixed_t, finesine[an as usize]);
}
pub const SKULLSPEED: i32 = 20 * FRACUNIT;
pub unsafe fn A_SkullAttack(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut dest: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut an: angle_t = 0;
    let mut dist: i32 = 0;
    dest = match (*actor).target.and_then(|id| state.p_mobj.mobj_get(id)) {
        Some(dest) => dest,
        None => return,
    };
    (*actor).flags |= MF_SKULLFLY as i32;
    let attacksound = (*state.info.mobjinfo_mut((*actor).type_0)).attacksound;
    S_StartSound(state, actor as *mut ::core::ffi::c_void, attacksound);
    A_FaceTarget(state, (*actor).id);
    an = (*actor).angle >> ANGLETOFINESHIFT;
    (*actor).momx = FixedMul(SKULLSPEED, finecosine[an as isize]);
    (*actor).momy = FixedMul(SKULLSPEED, finesine[an as usize]);
    dist = P_AproxDistance((*dest).x - (*actor).x, (*dest).y - (*actor).y) as i32;
    dist = dist / SKULLSPEED;
    if dist < 1 as i32 {
        dist = 1 as i32;
    }
    (*actor).momz = (((*dest).z as i32 + ((*dest).height as i32 >> 1 as i32) - (*actor).z as i32)
        / dist) as fixed_t;
}
pub unsafe fn A_PainShootSkull(state: &mut GameState, mut actor: *mut mobj_t, mut angle: angle_t) {
    let mut x: fixed_t = 0;
    let mut y: fixed_t = 0;
    let mut z: fixed_t = 0;
    let mut newmobj: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut an: angle_t = 0;
    let mut prestep: i32 = 0;
    let mut count: i32 = 0;
    let mut currentthinker: *mut thinker_t = ::core::ptr::null_mut::<thinker_t>();
    count = 0 as i32;
    let mut cursor = state.p_tick.head();
    while let Some(id) = cursor {
        currentthinker = state.p_tick.raw(id);
        if matches!((*currentthinker).function, ThinkerFn::Mobj(_))
            && (*(currentthinker as *mut mobj_t)).type_0 as u32 == MT_SKULL as i32 as u32
        {
            count += 1;
        }
        cursor = state.p_tick.next(id);
    }
    if count > 20 as i32 {
        return;
    }
    an = angle >> ANGLETOFINESHIFT;
    prestep = 4 as i32 * FRACUNIT
        + 3 as i32
            * ((*state.info.mobjinfo_mut((*actor).type_0)).radius + state.info.mobjinfo[MT_SKULL as i32 as usize].radius)
            / 2 as i32;
    x = (*actor).x + FixedMul(prestep as fixed_t, finecosine[an as isize]);
    y = (*actor).y + FixedMul(prestep as fixed_t, finesine[an as usize]);
    z = ((*actor).z as i32 + 8 as i32 * FRACUNIT) as fixed_t;
    newmobj = P_SpawnMobj(state, x, y, z, MT_SKULL);
    if !P_TryMove(state, newmobj, (*newmobj).x, (*newmobj).y) {
        P_DamageMobj(state, newmobj, actor, actor, 10000 as i32);
        return;
    }
    (*newmobj).target = (*actor).target;
    A_SkullAttack(state, (*newmobj).id);
}
pub unsafe fn A_PainAttack(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    if (*actor).target.is_none() {
        return;
    }
    A_FaceTarget(state, (*actor).id);
    A_PainShootSkull(state, actor, (*actor).angle);
}
pub unsafe fn A_PainDie(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    A_Fall(state, (*actor).id);
    A_PainShootSkull(state, actor, (*actor).angle.wrapping_add(ANG90 as angle_t));
    A_PainShootSkull(state, actor, (*actor).angle.wrapping_add(ANG180));
    A_PainShootSkull(state, actor, (*actor).angle.wrapping_add(ANG270));
}
pub unsafe fn A_Scream(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let mut sound: i32 = 0;
    match (*state.info.mobjinfo_mut((*actor).type_0)).deathsound {
        0 => return,
        59 | 60 | 61 => {
            sound = sfx_podth1 as i32 + P_Random(&mut state.m_random) % 3 as i32;
        }
        62 | 63 => {
            sound = sfx_bgdth1 as i32 + P_Random(&mut state.m_random) % 2 as i32;
        }
        _ => {
            sound = (*state.info.mobjinfo_mut((*actor).type_0)).deathsound;
        }
    }
    if (*actor).type_0 as u32 == MT_SPIDER as i32 as u32
        || (*actor).type_0 as u32 == MT_CYBORG as i32 as u32
    {
        S_StartSound(state, NULL, sound);
    } else {
        S_StartSound(state, actor as *mut ::core::ffi::c_void, sound);
    };
}
pub unsafe fn A_XScream(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    S_StartSound(
        state,
        actor as *mut ::core::ffi::c_void,
        sfx_slop as i32,
    );
}
pub unsafe fn A_Pain(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    let painsound = (*state.info.mobjinfo_mut((*actor).type_0)).painsound;
    if painsound != 0 {
        S_StartSound(state, actor as *mut ::core::ffi::c_void, painsound);
    }
}
pub unsafe fn A_Fall(state: &mut GameState, id: MobjId) {
    let actor = state.p_mobj.mobj_get(id).unwrap();
    (*actor).flags &= !(MF_SOLID as i32);
}
pub unsafe fn A_Explode(state: &mut GameState, id: MobjId) {
    let thingy = state.p_mobj.mobj_get(id).unwrap();
    let target = (*thingy)
        .target
        .and_then(|id| state.p_mobj.mobj_get(id))
        .unwrap_or(::core::ptr::null_mut());
    P_RadiusAttack(state, thingy, target, 128 as i32);
}
unsafe fn CheckBossEnd(state: &mut GameState, mut motype: mobjtype_t) -> bool {
    if !state.doomstat.gameversion.is_ultimate_or_higher() {
        if state.g_game.gamemap != 8 as i32 {
            return false;
        }
        if motype as u32 == MT_BRUISER as i32 as u32 && state.g_game.gameepisode != 1 as i32 {
            return false;
        }
        return true;
    } else {
        match state.g_game.gameepisode {
            1 => {
                return state.g_game.gamemap == 8 as i32
                    && motype as u32 == MT_BRUISER as i32 as u32;
            }
            2 => {
                return state.g_game.gamemap == 8 as i32
                    && motype as u32 == MT_CYBORG as i32 as u32;
            }
            3 => {
                return state.g_game.gamemap == 8 as i32
                    && motype as u32 == MT_SPIDER as i32 as u32;
            }
            4 => {
                return state.g_game.gamemap == 6 as i32
                    && motype as u32 == MT_CYBORG as i32 as u32
                    || state.g_game.gamemap == 8 as i32
                        && motype as u32 == MT_SPIDER as i32 as u32;
            }
            _ => {
                return state.g_game.gamemap == 8 as i32;
            }
        }
    };
}
pub unsafe fn A_BossDeath(state: &mut GameState, id: MobjId) {
    let mo = state.p_mobj.mobj_get(id).unwrap();
    let mut th: *mut thinker_t = ::core::ptr::null_mut::<thinker_t>();
    let mut mo2: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut junk: line_t = line_s {
        v1: VertexId(0),
        v2: VertexId(0),
        dx: 0,
        dy: 0,
        flags: 0,
        special: 0,
        tag: 0,
        sidenum: [0; 2],
        bbox: [0; 4],
        slopetype: ST_HORIZONTAL,
        frontsector: None,
        backsector: None,
        validcount: 0,
    };
    let mut i: i32 = 0;
    if state.doomstat.gamemode as u32 == commercial as i32 as u32 {
        if state.g_game.gamemap != 7 as i32 {
            return;
        }
        if (*mo).type_0 as u32 != MT_FATSO as i32 as u32
            && (*mo).type_0 as u32 != MT_BABY as i32 as u32
        {
            return;
        }
    } else if !CheckBossEnd(state, (*mo).type_0) {
        return;
    }
    i = 0 as i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] != 0
            && state.g_game.players[i as usize].health > 0 as i32
        {
            break;
        }
        i += 1;
    }
    if i == MAXPLAYERS {
        return;
    }
    let mut cursor = state.p_tick.head();
    while let Some(id) = cursor {
        th = state.p_tick.raw(id);
        if matches!((*th).function, ThinkerFn::Mobj(_)) {
            mo2 = th as *mut mobj_t;
            if mo2 != mo && (*mo2).type_0 as u32 == (*mo).type_0 as u32 && (*mo2).health > 0 as i32
            {
                return;
            }
        }
        cursor = state.p_tick.next(id);
    }
    if state.doomstat.gamemode as u32 == commercial as i32 as u32 {
        if state.g_game.gamemap == 7 as i32 {
            if (*mo).type_0 as u32 == MT_FATSO as i32 as u32 {
                junk.tag = 666 as i16;
                EV_DoFloor(state, &raw mut junk, lowerFloorToLowest);
                return;
            }
            if (*mo).type_0 as u32 == MT_BABY as i32 as u32 {
                junk.tag = 667 as i16;
                EV_DoFloor(state, &raw mut junk, raiseToTexture);
                return;
            }
        }
    } else {
        match state.g_game.gameepisode {
            1 => {
                junk.tag = 666 as i16;
                EV_DoFloor(state, &raw mut junk, lowerFloorToLowest);
                return;
            }
            4 => match state.g_game.gamemap {
                6 => {
                    junk.tag = 666 as i16;
                    EV_DoDoor(state, &raw mut junk, vld_blazeOpen);
                    return;
                }
                8 => {
                    junk.tag = 666 as i16;
                    EV_DoFloor(state, &raw mut junk, lowerFloorToLowest);
                    return;
                }
                _ => {}
            },
            _ => {}
        }
    }
    G_ExitLevel(state);
}
pub unsafe fn A_Hoof(state: &mut GameState, id: MobjId) {
    let mo = state.p_mobj.mobj_get(id).unwrap();
    S_StartSound(
        state,
        mo as *mut ::core::ffi::c_void,
        sfx_hoof as i32,
    );
    A_Chase(state, (*mo).id);
}
pub unsafe fn A_Metal(state: &mut GameState, id: MobjId) {
    let mo = state.p_mobj.mobj_get(id).unwrap();
    S_StartSound(
        state,
        mo as *mut ::core::ffi::c_void,
        sfx_metal as i32,
    );
    A_Chase(state, (*mo).id);
}
pub unsafe fn A_BabyMetal(state: &mut GameState, id: MobjId) {
    let mo = state.p_mobj.mobj_get(id).unwrap();
    S_StartSound(
        state,
        mo as *mut ::core::ffi::c_void,
        sfx_bspwlk as i32,
    );
    A_Chase(state, (*mo).id);
}
pub unsafe fn A_OpenShotgun2(
    state: &mut GameState,
    mut player: *mut player_t,
    _psp: *mut pspdef_t,
) {
    S_StartSound(
        state,
        (*player).mo as *mut ::core::ffi::c_void,
        sfx_dbopn as i32,
    );
}
pub unsafe fn A_LoadShotgun2(
    state: &mut GameState,
    mut player: *mut player_t,
    _psp: *mut pspdef_t,
) {
    S_StartSound(
        state,
        (*player).mo as *mut ::core::ffi::c_void,
        sfx_dbload as i32,
    );
}
pub unsafe fn A_CloseShotgun2(
    state: &mut GameState,
    mut player: *mut player_t,
    mut psp: *mut pspdef_t,
) {
    S_StartSound(
        state,
        (*player).mo as *mut ::core::ffi::c_void,
        sfx_dbcls as i32,
    );
    A_ReFire(state, player, psp);
}
pub unsafe fn A_BrainAwake(state: &mut GameState, _id: MobjId) {
    let mut thinker: *mut thinker_t = ::core::ptr::null_mut::<thinker_t>();
    let mut m: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    state.p_enemy.numbraintargets = 0 as i32;
    state.p_enemy.braintargeton = 0 as i32;
    let mut cursor = state.p_tick.head();
    while let Some(id) = cursor {
        thinker = state.p_tick.raw(id);
        if matches!((*thinker).function, ThinkerFn::Mobj(_)) {
            m = thinker as *mut mobj_t;
            if (*m).type_0 as u32 == MT_BOSSTARGET as i32 as u32 {
                state.p_enemy.braintargets[state.p_enemy.numbraintargets as usize] = m;
                state.p_enemy.numbraintargets += 1;
            }
        }
        cursor = state.p_tick.next(id);
    }
    S_StartSound(state, NULL, sfx_bossit as i32);
}
pub unsafe fn A_BrainPain(state: &mut GameState, _id: MobjId) {
    S_StartSound(state, NULL, sfx_bospn as i32);
}
pub unsafe fn A_BrainScream(state: &mut GameState, id: MobjId) {
    let mo = state.p_mobj.mobj_get(id).unwrap();
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut z: i32 = 0;
    let mut th: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    x = (*mo).x as i32 - 196 as i32 * FRACUNIT;
    while x < (*mo).x as i32 + 320 as i32 * FRACUNIT {
        y = (*mo).y as i32 - 320 as i32 * FRACUNIT;
        z = 128 as i32 + P_Random(&mut state.m_random) * 2 as i32 * FRACUNIT;
        th = P_SpawnMobj(state, x as fixed_t, y as fixed_t, z as fixed_t, MT_ROCKET);
        (*th).momz = (P_Random(&mut state.m_random) * 512 as i32) as fixed_t;
        P_SetMobjState(state, th, S_BRAINEXPLODE1);
        (*th).tics -= P_Random(&mut state.m_random) & 7 as i32;
        if (*th).tics < 1 as i32 {
            (*th).tics = 1 as i32;
        }
        x += FRACUNIT * 8 as i32;
    }
    S_StartSound(state, NULL, sfx_bosdth as i32);
}
pub unsafe fn A_BrainExplode(state: &mut GameState, id: MobjId) {
    let mo = state.p_mobj.mobj_get(id).unwrap();
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut z: i32 = 0;
    let mut th: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    x = (*mo).x as i32
        + (P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) * 2048 as i32;
    y = (*mo).y as i32;
    z = 128 as i32 + P_Random(&mut state.m_random) * 2 as i32 * FRACUNIT;
    th = P_SpawnMobj(state, x as fixed_t, y as fixed_t, z as fixed_t, MT_ROCKET);
    (*th).momz = (P_Random(&mut state.m_random) * 512 as i32) as fixed_t;
    P_SetMobjState(state, th, S_BRAINEXPLODE1);
    (*th).tics -= P_Random(&mut state.m_random) & 7 as i32;
    if (*th).tics < 1 as i32 {
        (*th).tics = 1 as i32;
    }
}
pub unsafe fn A_BrainDie(state: &mut GameState, _id: MobjId) {
    G_ExitLevel(state);
}
pub unsafe fn A_BrainSpit(state: &mut GameState, id: MobjId) {
    let mo = state.p_mobj.mobj_get(id).unwrap();
    let mut targ: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut newmobj: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let state = state;
    state.p_enemy.easy ^= 1 as i32;
    if state.g_game.gameskill as i32 <= sk_easy as i32 && state.p_enemy.easy == 0 {
        return;
    }
    targ = state.p_enemy.braintargets[state.p_enemy.braintargeton as usize];
    state.p_enemy.braintargeton =
        (state.p_enemy.braintargeton + 1 as i32) % state.p_enemy.numbraintargets;
    newmobj = P_SpawnMissile(state, mo, targ, MT_SPAWNSHOT);
    (*newmobj).target = Some((*targ).id);
    (*newmobj).reactiontime = ((*targ).y as i32 - (*mo).y as i32) / (*newmobj).momy as i32
        / (*state.info.state_mut((*newmobj).state.unwrap())).tics;
    S_StartSound(state, NULL, sfx_bospit as i32);
}
pub unsafe fn A_SpawnSound(state: &mut GameState, id: MobjId) {
    let mo = state.p_mobj.mobj_get(id).unwrap();
    S_StartSound(
        state,
        mo as *mut ::core::ffi::c_void,
        sfx_boscub as i32,
    );
    A_SpawnFly(state, (*mo).id);
}
pub unsafe fn A_SpawnFly(state: &mut GameState, id: MobjId) {
    let mo = state.p_mobj.mobj_get(id).unwrap();
    let mut newmobj: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut fog: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut targ: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut r: i32 = 0;
    let mut type_0: mobjtype_t = MT_PLAYER;
    (*mo).reactiontime -= 1;
    if (*mo).reactiontime != 0 {
        return;
    }
    let targ_subst = (*mo)
        .target
        .and_then(|id| state.p_mobj.mobj_get(id))
        .unwrap_or(::core::ptr::null_mut());
    targ = P_SubstNullMobj(&mut state.p_mobj, targ_subst);
    fog = P_SpawnMobj(state, (*targ).x, (*targ).y, (*targ).z, MT_SPAWNFIRE);
    S_StartSound(
        state,
        fog as *mut ::core::ffi::c_void,
        sfx_telept as i32,
    );
    r = P_Random(&mut state.m_random);
    if r < 50 as i32 {
        type_0 = MT_TROOP;
    } else if r < 90 as i32 {
        type_0 = MT_SERGEANT;
    } else if r < 120 as i32 {
        type_0 = MT_SHADOWS;
    } else if r < 130 as i32 {
        type_0 = MT_PAIN;
    } else if r < 160 as i32 {
        type_0 = MT_HEAD;
    } else if r < 162 as i32 {
        type_0 = MT_VILE;
    } else if r < 172 as i32 {
        type_0 = MT_UNDEAD;
    } else if r < 192 as i32 {
        type_0 = MT_BABY;
    } else if r < 222 as i32 {
        type_0 = MT_FATSO;
    } else if r < 246 as i32 {
        type_0 = MT_KNIGHT;
    } else {
        type_0 = MT_BRUISER;
    }
    newmobj = P_SpawnMobj(state, (*targ).x, (*targ).y, (*targ).z, type_0);
    if P_LookForPlayers(state, newmobj, true) {
        let seestate = (*state.info.mobjinfo_mut((*newmobj).type_0)).seestate as statenum_t;
        P_SetMobjState(state, newmobj, seestate);
    }
    P_TeleportMove(state, newmobj, (*newmobj).x, (*newmobj).y);
    P_RemoveMobj(state, mo);
}
pub unsafe fn A_PlayerScream(state: &mut GameState, id: MobjId) {
    let mo = state.p_mobj.mobj_get(id).unwrap();
    let mut sound: i32 = sfx_pldeth as i32;
    if state.doomstat.gamemode as u32 == commercial as i32 as u32 && (*mo).health < -(50 as i32) {
        sound = sfx_pdiehi as i32;
    }
    S_StartSound(state, mo as *mut ::core::ffi::c_void, sound);
}
