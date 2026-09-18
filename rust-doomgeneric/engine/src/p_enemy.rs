use crate::d_mode::GameMode_t;
use crate::d_mode::SkillType;

use crate::d_player::PlayerId;
use crate::g_game::G_ExitLevel;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedMul;
use crate::m_random::P_Random;
use crate::p_doors::EV_DoDoor;
use crate::p_doors::VldoorE;
use crate::p_floor::EV_DoFloor;
use crate::p_floor::FloorE;
use crate::p_inter::P_DamageMobj;
use crate::p_map::P_AimLineAttack;
use crate::p_map::P_CheckPosition;
use crate::p_map::P_LineAttack;
use crate::p_map::P_RadiusAttack;
use crate::p_map::P_TeleportMove;
use crate::p_map::P_TryMove;
use crate::p_maputl::P_AproxDistance;
use crate::p_maputl::P_BlockThingsIterator;
use crate::p_maputl::P_LineOpening;
use crate::p_maputl::P_SetThingPosition;
use crate::p_maputl::P_UnsetThingPosition;
use crate::p_mobj::MobjId;
use crate::p_mobj::MobjType;
use crate::p_mobj::P_RemoveMobj;
use crate::p_mobj::P_SetMobjState;
use crate::p_mobj::P_SpawnMissile;

use crate::p_mobj::P_SpawnMobj;

use crate::p_mobj::P_SpawnPuff;
use crate::p_mobj::P_SubstNullMobj;

use crate::p_mobj::{
    MF_AMBUSH, MF_CORPSE, MF_FLOAT, MF_INFLOAT, MF_JUSTATTACKED, MF_JUSTHIT, MF_SHADOW,
    MF_SHOOTABLE, MF_SKULLFLY, MF_SOLID,
};
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_sight::P_CheckSight;
use crate::p_switch::P_UseSpecialLine;
use crate::p_tick::P_MobjThinkerIds;

use crate::r_main::R_PointToAngle2;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::{
    sfx_barexp, sfx_bgdth1, sfx_bgsit1, sfx_boscub, sfx_bosdth, sfx_bospit, sfx_bospn, sfx_bossit,
    sfx_bspwlk, sfx_claw, sfx_dbcls, sfx_dbload, sfx_dbopn, sfx_flame, sfx_flamst, sfx_hoof,
    sfx_manatk, sfx_metal, sfx_pdiehi, sfx_pistol, sfx_pldeth, sfx_podth1, sfx_posit1, sfx_shotgn,
    sfx_skepch, sfx_skeswg, sfx_slop, sfx_telept, sfx_vilatk,
};
use crate::tables::angle_t;
use crate::tables::finecosine;
use crate::tables::finesine;

use crate::doomdef::MAXPLAYERS;
use crate::game_state::GameState;
use crate::m_fixed::FRACUNIT;
use crate::p_maputl::MAPBLOCKSHIFT;
use crate::p_mobj::StateNum;
use crate::p_mobj::FLOATSPEED;
use crate::p_pspr::A_ReFire;
use crate::p_spec::ML_TWOSIDED;
use crate::tables::ANG180;
use crate::tables::ANG270;
use crate::tables::ANG90;
use crate::tables::ANGLETOFINESHIFT;

pub struct PEnemyState {
    pub soundtarget: Option<MobjId>,
    pub corpsehit: Option<MobjId>,
    pub vileobj: Option<MobjId>,
    pub viletryx: fixed_t,
    pub viletryy: fixed_t,
    pub braintargets: [Option<MobjId>; 32],
    pub numbraintargets: i32,
    pub braintargeton: i32,
    pub easy: i32,
}

impl Default for PEnemyState {
    fn default() -> Self {
        Self::new()
    }
}

impl PEnemyState {
    pub const fn new() -> Self {
        PEnemyState {
            soundtarget: None,
            corpsehit: None,
            vileobj: None,
            viletryx: 0,
            viletryy: 0,
            braintargets: [None; 32],
            numbraintargets: 0,
            braintargeton: 0,
            easy: 0,
        }
    }
}

pub const NUMDIRS: i32 = 9;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DirType {
    DI_EAST = 0,
    DI_NORTHEAST = 1,
    DI_NORTH = 2,
    DI_NORTHWEST = 3,
    DI_WEST = 4,
    DI_SOUTHWEST = 5,
    DI_SOUTH = 6,
    DI_SOUTHEAST = 7,
    DI_NODIR = 8,
}
fn dirtype_from_movedir(movedir: i32) -> DirType {
    match movedir {
        0 => DirType::DI_EAST,
        1 => DirType::DI_NORTHEAST,
        2 => DirType::DI_NORTH,
        3 => DirType::DI_NORTHWEST,
        4 => DirType::DI_WEST,
        5 => DirType::DI_SOUTHWEST,
        6 => DirType::DI_SOUTH,
        7 => DirType::DI_SOUTHEAST,
        8 => DirType::DI_NODIR,
        n => panic!("P_NewChaseDir: invalid movedir {n}"),
    }
}
pub const ML_SOUNDBLOCK: i32 = 64;
pub const MELEERANGE: i32 = 64 * FRACUNIT;
pub const MISSILERANGE: i32 = 32 * 64_i32 * FRACUNIT;
pub static opposite: [DirType; 9] = [
    DirType::DI_WEST,
    DirType::DI_SOUTHWEST,
    DirType::DI_SOUTH,
    DirType::DI_SOUTHEAST,
    DirType::DI_EAST,
    DirType::DI_NORTHEAST,
    DirType::DI_NORTH,
    DirType::DI_NORTHWEST,
    DirType::DI_NODIR,
];
pub static diags: [DirType; 4] = [
    DirType::DI_NORTHWEST,
    DirType::DI_NORTHEAST,
    DirType::DI_SOUTHWEST,
    DirType::DI_SOUTHEAST,
];
pub fn P_RecursiveSound(state: &mut GameState, sec: SectorId, soundblocks: i32) {
    let validcount = state.r_main.validcount;
    {
        let s = state.p_setup.sector_mut(sec);
        if s.validcount == validcount && s.soundtraversed <= soundblocks + 1_i32 {
            return;
        }
        s.validcount = validcount;
        s.soundtraversed = soundblocks + 1_i32;
        s.soundtarget = state.p_enemy.soundtarget;
    }
    let linecount = state.p_setup.sector_mut(sec).linecount;
    for i in 0..linecount {
        let check = state.p_setup.sector_mut(sec).lines[i as usize];
        let checkv = state.p_setup.line(check);
        if checkv.flags as i32 & ML_TWOSIDED != 0 {
            P_LineOpening(state, check);
            if state.p_maputl.openrange > 0_i32 {
                let other = if state.p_setup.sides[checkv.sidenum[0] as usize].sector == sec {
                    state.p_setup.sides[checkv.sidenum[1] as usize].sector
                } else {
                    state.p_setup.sides[checkv.sidenum[0] as usize].sector
                };
                if checkv.flags as i32 & ML_SOUNDBLOCK != 0 {
                    if soundblocks == 0 {
                        P_RecursiveSound(state, other, 1_i32);
                    }
                } else {
                    P_RecursiveSound(state, other, soundblocks);
                }
            }
        }
    }
}
pub fn P_NoiseAlert(state: &mut GameState, target: MobjId, emmiter: MobjId) {
    state.p_enemy.soundtarget = Some(target);
    state.r_main.validcount += 1;
    let emmiter_subsector = state.p_mobj.mo(emmiter).subsector;
    let sec = state.p_setup.subsectors[emmiter_subsector.0 as usize].sector;
    P_RecursiveSound(state, sec, 0_i32);
}
pub fn P_CheckMeleeRange(state: &mut GameState, actor: MobjId) -> bool {
    let Some(pl) = state
        .p_mobj
        .mo(actor)
        .target
        .filter(|&id| state.p_mobj.is_live(id))
    else {
        return false;
    };
    let (pl_x, pl_y, pl_type) = {
        let p = state.p_mobj.mo(pl);
        (p.x, p.y, p.type_0)
    };
    let (actor_x, actor_y) = {
        let a = state.p_mobj.mo(actor);
        (a.x, a.y)
    };
    let dist = P_AproxDistance(pl_x - actor_x, pl_y - actor_y);
    if dist >= MELEERANGE - 20_i32 * FRACUNIT + state.info.mobjinfo_mut(pl_type).radius {
        return false;
    }
    if !P_CheckSight(state, actor, pl) {
        return false;
    }
    true
}
pub fn P_CheckMissileRange(state: &mut GameState, actor: MobjId) -> bool {
    let Some(target) = state
        .p_mobj
        .mo(actor)
        .target
        .filter(|&id| state.p_mobj.is_live(id))
    else {
        return false;
    };
    if !P_CheckSight(state, actor, target) {
        return false;
    }
    if state.p_mobj.mo(actor).flags & MF_JUSTHIT as i32 != 0 {
        state.p_mobj.mo_mut(actor).flags &= !(MF_JUSTHIT as i32);
        return true;
    }
    if state.p_mobj.mo(actor).reactiontime != 0 {
        return false;
    }
    let (actor_x, actor_y, actor_type) = {
        let a = state.p_mobj.mo(actor);
        (a.x, a.y, a.type_0)
    };
    let (target_x, target_y) = {
        let t = state.p_mobj.mo(target);
        (t.x, t.y)
    };
    let mut dist: fixed_t =
        (P_AproxDistance(actor_x - target_x, actor_y - target_y) - 64_i32 * FRACUNIT) as fixed_t;
    if state.info.mobjinfo_mut(actor_type).meleestate == StateNum::S_NULL {
        dist -= 128_i32 * FRACUNIT;
    }
    dist >>= 16_i32;
    if actor_type as u32 == MobjType::MT_VILE as i32 as u32 && dist > 14_i32 * 64_i32 {
        return false;
    }
    if actor_type as u32 == MobjType::MT_UNDEAD as i32 as u32 {
        if dist < 196_i32 {
            return false;
        }
        dist >>= 1_i32;
    }
    if actor_type as u32 == MobjType::MT_CYBORG as i32 as u32
        || actor_type as u32 == MobjType::MT_SPIDER as i32 as u32
        || actor_type as u32 == MobjType::MT_SKULL as i32 as u32
    {
        dist >>= 1_i32;
    }
    if dist > 200_i32 {
        dist = 200_i32 as fixed_t;
    }
    if actor_type as u32 == MobjType::MT_CYBORG as i32 as u32 && dist > 160_i32 {
        dist = 160_i32 as fixed_t;
    }
    if P_Random(&mut state.m_random) < dist {
        return false;
    }
    true
}
pub static xspeed: [fixed_t; 8] = [
    FRACUNIT, 47000_i32, 0_i32, -47000_i32, -FRACUNIT, -47000_i32, 0_i32, 47000_i32,
];
pub static yspeed: [fixed_t; 8] = [
    0_i32, 47000_i32, FRACUNIT, 47000_i32, 0_i32, -47000_i32, -FRACUNIT, -47000_i32,
];
pub fn P_Move(state: &mut GameState, actor: MobjId) -> bool {
    let mut tryx: fixed_t = 0;
    let mut tryy: fixed_t = 0;
    let mut ld: LineId;
    let mut try_ok: bool;
    let mut good: bool;
    if state.p_mobj.mo(actor).movedir == DirType::DI_NODIR as i32 {
        return false;
    }
    if state.p_mobj.mo(actor).movedir as u32 >= 8_u32 {
        I_Error("Weird actor->movedir!");
    }
    tryx = state.p_mobj.mo(actor).x
        + state.info.mobjinfo_mut(state.p_mobj.mo(actor).type_0).speed as fixed_t
            * xspeed[state.p_mobj.mo(actor).movedir as usize];
    tryy = state.p_mobj.mo(actor).y
        + state.info.mobjinfo_mut(state.p_mobj.mo(actor).type_0).speed as fixed_t
            * yspeed[state.p_mobj.mo(actor).movedir as usize];
    try_ok = P_TryMove(state, actor, tryx, tryy);
    if !try_ok {
        if state.p_mobj.mo(actor).flags & MF_FLOAT as i32 != 0 && state.p_map.floatok {
            if state.p_mobj.mo(actor).z < state.p_map.tmfloorz {
                state.p_mobj.mo_mut(actor).z += FLOATSPEED;
            } else {
                state.p_mobj.mo_mut(actor).z -= FLOATSPEED;
            }
            state.p_mobj.mo_mut(actor).flags |= MF_INFLOAT as i32;
            return true;
        }
        if state.p_map.numspechit == 0 {
            return false;
        }
        state.p_mobj.mo_mut(actor).movedir = DirType::DI_NODIR as i32;
        good = false;
        loop {
            let fresh0 = state.p_map.numspechit;
            state.p_map.numspechit -= 1;
            if fresh0 == 0 {
                break;
            }
            ld = state.p_map.spechit[state.p_map.numspechit as usize];
            if P_UseSpecialLine(state, actor, ld, 0_i32) {
                good = true;
            }
        }
        return good;
    } else {
        state.p_mobj.mo_mut(actor).flags &= !(MF_INFLOAT as i32);
    }
    if state.p_mobj.mo(actor).flags & MF_FLOAT as i32 == 0 {
        state.p_mobj.mo_mut(actor).z = state.p_mobj.mo(actor).floorz;
    }
    true
}
pub fn P_TryWalk(state: &mut GameState, actor: MobjId) -> bool {
    if !P_Move(state, actor) {
        return false;
    }
    state.p_mobj.mo_mut(actor).movecount = P_Random(&mut state.m_random) & 15_i32;
    true
}
pub fn P_NewChaseDir(state: &mut GameState, actor: MobjId) {
    let mut deltax: fixed_t = 0;
    let mut deltay: fixed_t = 0;
    let mut d: [DirType; 3] = [DirType::DI_EAST; 3];
    let mut tdir: i32 = 0;
    let mut olddir: DirType;
    let mut turnaround: DirType;
    let target = match state
        .p_mobj
        .mo(actor)
        .target
        .filter(|&id| state.p_mobj.is_live(id))
    {
        Some(target) => target,
        None => {
            I_Error("P_NewChaseDir: called with no target");
        }
    };
    olddir = dirtype_from_movedir(state.p_mobj.mo(actor).movedir);
    turnaround = opposite[olddir as usize];
    deltax = state.p_mobj.mo(target).x - state.p_mobj.mo(actor).x;
    deltay = state.p_mobj.mo(target).y - state.p_mobj.mo(actor).y;
    if deltax > 10_i32 * FRACUNIT {
        d[1] = DirType::DI_EAST;
    } else if deltax < -10_i32 * FRACUNIT {
        d[1] = DirType::DI_WEST;
    } else {
        d[1] = DirType::DI_NODIR;
    }
    if deltay < -10_i32 * FRACUNIT {
        d[2] = DirType::DI_SOUTH;
    } else if deltay > 10_i32 * FRACUNIT {
        d[2] = DirType::DI_NORTH;
    } else {
        d[2] = DirType::DI_NODIR;
    }
    if d[1] != DirType::DI_NODIR && d[2] != DirType::DI_NODIR {
        state.p_mobj.mo_mut(actor).movedir =
            diags[((((deltay < 0_i32) as i32) << 1_i32) + (deltax > 0_i32) as i32) as usize] as i32;
        if state.p_mobj.mo(actor).movedir != turnaround as i32 && P_TryWalk(state, actor) {
            return;
        }
    }
    if P_Random(&mut state.m_random) > 200_i32 || deltay.abs() > deltax.abs() {
        d.swap(1, 2);
    }
    if d[1] == turnaround {
        d[1] = DirType::DI_NODIR;
    }
    if d[2] == turnaround {
        d[2] = DirType::DI_NODIR;
    }
    if d[1] != DirType::DI_NODIR {
        state.p_mobj.mo_mut(actor).movedir = d[1] as i32;
        if P_TryWalk(state, actor) {
            return;
        }
    }
    if d[2] != DirType::DI_NODIR {
        state.p_mobj.mo_mut(actor).movedir = d[2] as i32;
        if P_TryWalk(state, actor) {
            return;
        }
    }
    if olddir != DirType::DI_NODIR {
        state.p_mobj.mo_mut(actor).movedir = olddir as i32;
        if P_TryWalk(state, actor) {
            return;
        }
    }
    if P_Random(&mut state.m_random) & 1_i32 != 0 {
        tdir = DirType::DI_EAST as i32;
        while tdir <= DirType::DI_SOUTHEAST as i32 {
            if tdir != turnaround as i32 {
                state.p_mobj.mo_mut(actor).movedir = tdir;
                if P_TryWalk(state, actor) {
                    return;
                }
            }
            tdir += 1;
        }
    } else {
        tdir = DirType::DI_SOUTHEAST as i32;
        while tdir != DirType::DI_EAST as i32 - 1_i32 {
            if tdir != turnaround as i32 {
                state.p_mobj.mo_mut(actor).movedir = tdir;
                if P_TryWalk(state, actor) {
                    return;
                }
            }
            tdir -= 1;
        }
    }
    if turnaround != DirType::DI_NODIR {
        state.p_mobj.mo_mut(actor).movedir = turnaround as i32;
        if P_TryWalk(state, actor) {
            return;
        }
    }
    state.p_mobj.mo_mut(actor).movedir = DirType::DI_NODIR as i32;
}
pub fn P_LookForPlayers(state: &mut GameState, actor: MobjId, allaround: bool) -> bool {
    let mut c: i32 = 0;
    let stop = (state.p_mobj.mo(actor).lastlook - 1_i32) & 3_i32;
    loop {
        let lastlook = state.p_mobj.mo(actor).lastlook;
        if state.g_game.playeringame[lastlook as usize] {
            let fresh1 = c;
            c += 1;
            if fresh1 == 2_i32 || lastlook == stop {
                return false;
            }
            let (health, player_mo) = {
                let player = &state.g_game.players[lastlook as usize];
                (player.health, player.mo)
            };
            if health > 0_i32 {
                let player_mo = player_mo.unwrap();
                if P_CheckSight(state, actor, player_mo) {
                    let mut skip = false;
                    if !allaround {
                        let (actor_x, actor_y, actor_angle) = {
                            let a = state.p_mobj.mo(actor);
                            (a.x, a.y, a.angle)
                        };
                        let (pmo_x, pmo_y) = {
                            let p = state.p_mobj.mo(player_mo);
                            (p.x, p.y)
                        };
                        let an = R_PointToAngle2(state, actor_x, actor_y, pmo_x, pmo_y)
                            .wrapping_sub(actor_angle);
                        if an > ANG90 as angle_t && an < ANG270 {
                            let dist = P_AproxDistance(pmo_x - actor_x, pmo_y - actor_y);
                            if dist > MELEERANGE {
                                skip = true;
                            }
                        }
                    }
                    if !skip {
                        state.p_mobj.mo_mut(actor).target = Some(player_mo);
                        return true;
                    }
                }
            }
        }
        state.p_mobj.mo_mut(actor).lastlook = (lastlook + 1_i32) & 3_i32;
    }
}
pub fn A_KeenDie(state: &mut GameState, id: MobjId) {
    let mo = id;
    A_Fall(state, mo);
    let mo_type = state.p_mobj.mo(mo).type_0;
    for mo2 in P_MobjThinkerIds(state) {
        if mo2 != mo
            && state.p_mobj.mo(mo2).type_0 as u32 == mo_type as u32
            && state.p_mobj.mo(mo2).health > 0_i32
        {
            return;
        }
    }
    let junk = state.p_setup.junk_line(666_i16);
    EV_DoDoor(state, junk, VldoorE::vld_open);
}
pub fn A_Look(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut current_block: u64;
        let mut targ: Option<MobjId> = None;
        state.p_mobj.mo_mut(actor).threshold = 0_i32;
        targ = state
            .p_setup
            .sector_mut(
                state.p_setup.subsectors[state.p_mobj.mo(actor).subsector.0 as usize].sector,
            )
            .soundtarget
            .filter(|&id| state.p_mobj.is_live(id));
        if let Some(targ) = targ.filter(|&t| state.p_mobj.mo(t).flags & MF_SHOOTABLE as i32 != 0) {
            state.p_mobj.mo_mut(actor).target = Some(targ);
            if state.p_mobj.mo(actor).flags & MF_AMBUSH as i32 != 0 {
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
        if current_block == 15619007995458559411 && !P_LookForPlayers(state, actor, false) {
            return;
        }
        if state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
            .seesound
            != 0
        {
            let mut sound: i32 = 0;
            match state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
                .seesound
            {
                36..=38 => {
                    sound = sfx_posit1 as i32 + P_Random(&mut state.m_random) % 3_i32;
                }
                39 | 40 => {
                    sound = sfx_bgsit1 as i32 + P_Random(&mut state.m_random) % 2_i32;
                }
                _ => {
                    sound = state
                        .info
                        .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
                        .seesound;
                }
            }
            if state.p_mobj.mo(actor).type_0 as u32 == MobjType::MT_SPIDER as i32 as u32
                || state.p_mobj.mo(actor).type_0 as u32 == MobjType::MT_CYBORG as i32 as u32
            {
                S_StartSound(state, SoundOrigin::None, sound);
            } else {
                S_StartSound(state, SoundOrigin::Mobj(actor), sound);
            }
        }
        let seestate = state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
            .seestate;
        P_SetMobjState(state, actor, seestate);
    }
}
pub fn A_Chase(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut delta: i32 = 0;
        if state.p_mobj.mo(actor).reactiontime != 0 {
            state.p_mobj.mo_mut(actor).reactiontime -= 1;
        }
        let target = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        if state.p_mobj.mo(actor).threshold != 0 {
            if target.is_none() || state.p_mobj.mo(target.unwrap()).health <= 0_i32 {
                state.p_mobj.mo_mut(actor).threshold = 0_i32;
            } else {
                state.p_mobj.mo_mut(actor).threshold -= 1;
            }
        }
        if state.p_mobj.mo(actor).movedir < 8_i32 {
            state.p_mobj.mo_mut(actor).angle &= (7_i32 << 29_i32) as angle_t;
            delta = state
                .p_mobj
                .mo(actor)
                .angle
                .wrapping_sub((state.p_mobj.mo(actor).movedir << 29_i32) as angle_t)
                as i32;
            if delta > 0_i32 {
                state.p_mobj.mo_mut(actor).angle = state
                    .p_mobj
                    .mo(actor)
                    .angle
                    .wrapping_sub((ANG90 / 2_i32) as angle_t);
            } else if delta < 0_i32 {
                state.p_mobj.mo_mut(actor).angle = state
                    .p_mobj
                    .mo(actor)
                    .angle
                    .wrapping_add((ANG90 / 2_i32) as angle_t);
            }
        }
        if target.is_none() || state.p_mobj.mo(target.unwrap()).flags & MF_SHOOTABLE as i32 == 0 {
            if P_LookForPlayers(state, actor, true) {
                return;
            }
            let spawnstate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
                .spawnstate;
            P_SetMobjState(state, actor, spawnstate);
            return;
        }
        if state.p_mobj.mo(actor).flags & MF_JUSTATTACKED as i32 != 0 {
            state.p_mobj.mo_mut(actor).flags &= !(MF_JUSTATTACKED as i32);
            if state.g_game.gameskill != SkillType::sk_nightmare && !state.d_main.fastparm {
                P_NewChaseDir(state, actor);
            }
            return;
        }
        let actor_info = state.info.mobjinfo_mut(state.p_mobj.mo(actor).type_0);
        if actor_info.meleestate != StateNum::S_NULL && P_CheckMeleeRange(state, actor) {
            let attacksound = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
                .attacksound;
            if attacksound != 0 {
                S_StartSound(state, SoundOrigin::Mobj(actor), attacksound);
            }
            let meleestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
                .meleestate;
            P_SetMobjState(state, actor, meleestate);
            return;
        }
        if state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
            .missilestate
            != StateNum::S_NULL
            && !(state.g_game.gameskill < SkillType::sk_nightmare
                && !state.d_main.fastparm
                && state.p_mobj.mo(actor).movecount != 0)
            && P_CheckMissileRange(state, actor)
        {
            let missilestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
                .missilestate;
            P_SetMobjState(state, actor, missilestate);
            state.p_mobj.mo_mut(actor).flags |= MF_JUSTATTACKED as i32;
            return;
        }
        if state.g_game.netgame
            && state.p_mobj.mo(actor).threshold == 0
            && !P_CheckSight(state, actor, state.p_mobj.mo(target.unwrap()).id)
            && P_LookForPlayers(state, actor, true)
        {
            return;
        }
        state.p_mobj.mo_mut(actor).movecount -= 1;
        if state.p_mobj.mo(actor).movecount < 0_i32 || !P_Move(state, actor) {
            P_NewChaseDir(state, actor);
        }
        let activesound = state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
            .activesound;
        if activesound != 0 && P_Random(&mut state.m_random) < 3_i32 {
            S_StartSound(state, SoundOrigin::Mobj(actor), activesound);
        }
    }
}
pub fn A_FaceTarget(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let target = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(target) => target,
            None => return,
        };
        state.p_mobj.mo_mut(actor).flags &= !(MF_AMBUSH as i32);
        state.p_mobj.mo_mut(actor).angle = R_PointToAngle2(
            state,
            state.p_mobj.mo(actor).x,
            state.p_mobj.mo(actor).y,
            state.p_mobj.mo(target).x,
            state.p_mobj.mo(target).y,
        );
        if state.p_mobj.mo(target).flags & MF_SHADOW as i32 != 0 {
            state.p_mobj.mo_mut(actor).angle = state.p_mobj.mo(actor).angle.wrapping_add(
                ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 21_i32)
                    as angle_t,
            );
        }
    }
}
pub fn A_PosAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut angle: i32 = 0;
        let mut damage: i32 = 0;
        let mut slope: i32 = 0;
        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        A_FaceTarget(state, actor);
        angle = state.p_mobj.mo(actor).angle as i32;
        slope = P_AimLineAttack(state, Some(actor), angle as angle_t, MISSILERANGE);
        S_StartSound(state, SoundOrigin::Mobj(actor), sfx_pistol as i32);
        angle += (P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 20_i32;
        damage = (P_Random(&mut state.m_random) % 5_i32 + 1_i32) * 3_i32;
        P_LineAttack(
            state,
            actor,
            angle as angle_t,
            MISSILERANGE,
            slope as fixed_t,
            damage,
        );
    }
}
pub fn A_SPosAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut i: i32 = 0;
        let mut angle: i32 = 0;
        let mut bangle: i32 = 0;
        let mut damage: i32 = 0;
        let mut slope: i32 = 0;
        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        S_StartSound(state, SoundOrigin::Mobj(actor), sfx_shotgn as i32);
        A_FaceTarget(state, actor);
        bangle = state.p_mobj.mo(actor).angle as i32;
        slope = P_AimLineAttack(state, Some(actor), bangle as angle_t, MISSILERANGE);
        i = 0_i32;
        while i < 3_i32 {
            angle = bangle
                + ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 20_i32);
            damage = (P_Random(&mut state.m_random) % 5_i32 + 1_i32) * 3_i32;
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
}
pub fn A_CPosAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut angle: i32 = 0;
        let mut bangle: i32 = 0;
        let mut damage: i32 = 0;
        let mut slope: i32 = 0;
        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        S_StartSound(state, SoundOrigin::Mobj(actor), sfx_shotgn as i32);
        A_FaceTarget(state, actor);
        bangle = state.p_mobj.mo(actor).angle as i32;
        slope = P_AimLineAttack(state, Some(actor), bangle as angle_t, MISSILERANGE);
        angle =
            bangle + ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 20_i32);
        damage = (P_Random(&mut state.m_random) % 5_i32 + 1_i32) * 3_i32;
        P_LineAttack(
            state,
            actor,
            angle as angle_t,
            MISSILERANGE,
            slope as fixed_t,
            damage,
        );
    }
}
pub fn A_CPosRefire(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        A_FaceTarget(state, actor);
        if P_Random(&mut state.m_random) < 40_i32 {
            return;
        }
        let target = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        if target.is_none()
            || state.p_mobj.mo(target.unwrap()).health <= 0_i32
            || !P_CheckSight(state, actor, state.p_mobj.mo(target.unwrap()).id)
        {
            let seestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
                .seestate;
            P_SetMobjState(state, actor, seestate);
        }
    }
}
pub fn A_SpidRefire(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        A_FaceTarget(state, actor);
        if P_Random(&mut state.m_random) < 10_i32 {
            return;
        }
        let target = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        if target.is_none()
            || state.p_mobj.mo(target.unwrap()).health <= 0_i32
            || !P_CheckSight(state, actor, state.p_mobj.mo(target.unwrap()).id)
        {
            let seestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
                .seestate;
            P_SetMobjState(state, actor, seestate);
        }
    }
}
pub fn A_BspiAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let target = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(target) => target,
            None => return,
        };
        A_FaceTarget(state, actor);
        P_SpawnMissile(state, actor, target, MobjType::MT_ARACHPLAZ);
    }
}
pub fn A_TroopAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut damage: i32 = 0;
        let target = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(target) => target,
            None => return,
        };
        A_FaceTarget(state, actor);
        if P_CheckMeleeRange(state, actor) {
            S_StartSound(state, SoundOrigin::Mobj(actor), sfx_claw as i32);
            damage = (P_Random(&mut state.m_random) % 8_i32 + 1_i32) * 3_i32;
            P_DamageMobj(state, target, Some(actor), Some(actor), damage);
            return;
        }
        P_SpawnMissile(state, actor, target, MobjType::MT_TROOPSHOT);
    }
}
pub fn A_SargAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut damage: i32 = 0;
        let target = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(target) => target,
            None => return,
        };
        A_FaceTarget(state, actor);
        if P_CheckMeleeRange(state, actor) {
            damage = (P_Random(&mut state.m_random) % 10_i32 + 1_i32) * 4_i32;
            P_DamageMobj(state, target, Some(actor), Some(actor), damage);
        }
    }
}
pub fn A_HeadAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut damage: i32 = 0;
        let target = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(target) => target,
            None => return,
        };
        A_FaceTarget(state, actor);
        if P_CheckMeleeRange(state, actor) {
            damage = (P_Random(&mut state.m_random) % 6_i32 + 1_i32) * 10_i32;
            P_DamageMobj(state, target, Some(actor), Some(actor), damage);
            return;
        }
        P_SpawnMissile(state, actor, target, MobjType::MT_HEADSHOT);
    }
}
pub fn A_CyberAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let target = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(target) => target,
            None => return,
        };
        A_FaceTarget(state, actor);
        P_SpawnMissile(state, actor, target, MobjType::MT_ROCKET);
    }
}
pub fn A_BruisAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut damage: i32 = 0;
        let target = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(target) => target,
            None => return,
        };
        if P_CheckMeleeRange(state, actor) {
            S_StartSound(state, SoundOrigin::Mobj(actor), sfx_claw as i32);
            damage = (P_Random(&mut state.m_random) % 8_i32 + 1_i32) * 10_i32;
            P_DamageMobj(state, target, Some(actor), Some(actor), damage);
            return;
        }
        P_SpawnMissile(state, actor, target, MobjType::MT_BRUISERSHOT);
    }
}
pub fn A_SkelMissile(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut mo: MobjId;
        let target = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(target) => target,
            None => return,
        };
        A_FaceTarget(state, actor);
        state.p_mobj.mo_mut(actor).z += 16_i32 * FRACUNIT;
        mo = P_SpawnMissile(state, actor, target, MobjType::MT_TRACER);
        state.p_mobj.mo_mut(actor).z -= 16_i32 * FRACUNIT;
        state.p_mobj.mo_mut(mo).x += state.p_mobj.mo(mo).momx;
        state.p_mobj.mo_mut(mo).y += state.p_mobj.mo(mo).momy;
        state.p_mobj.mo_mut(mo).tracer = state.p_mobj.mo(actor).target;
    }
}
pub static TRACEANGLE: i32 = 0xc000000;
pub fn A_Tracer(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut exact: angle_t = 0;
        let mut dist: fixed_t = 0;
        let mut slope: fixed_t = 0;
        let mut dest: Option<MobjId> = None;
        let mut th: MobjId;
        if state.d_loop.gametic & 3_i32 != 0 {
            return;
        }
        P_SpawnPuff(
            state,
            state.p_mobj.mo(actor).x,
            state.p_mobj.mo(actor).y,
            state.p_mobj.mo(actor).z,
        );
        th = P_SpawnMobj(
            state,
            state.p_mobj.mo(actor).x - state.p_mobj.mo(actor).momx,
            state.p_mobj.mo(actor).y - state.p_mobj.mo(actor).momy,
            state.p_mobj.mo(actor).z,
            MobjType::MT_SMOKE,
        );
        state.p_mobj.mo_mut(th).momz = FRACUNIT as fixed_t;
        state.p_mobj.mo_mut(th).tics -= P_Random(&mut state.m_random) & 3_i32;
        if state.p_mobj.mo(th).tics < 1_i32 {
            state.p_mobj.mo_mut(th).tics = 1_i32;
        }
        dest = state
            .p_mobj
            .mo(actor)
            .tracer
            .filter(|&id| state.p_mobj.is_live(id));
        if dest.is_none() || state.p_mobj.mo(dest.unwrap()).health <= 0_i32 {
            return;
        }
        exact = R_PointToAngle2(
            state,
            state.p_mobj.mo(actor).x,
            state.p_mobj.mo(actor).y,
            state.p_mobj.mo(dest.unwrap()).x,
            state.p_mobj.mo(dest.unwrap()).y,
        );
        if exact != state.p_mobj.mo(actor).angle {
            if exact.wrapping_sub(state.p_mobj.mo(actor).angle) > 0x80000000_u32 {
                state.p_mobj.mo_mut(actor).angle = state
                    .p_mobj
                    .mo(actor)
                    .angle
                    .wrapping_sub(TRACEANGLE as angle_t);
                if exact.wrapping_sub(state.p_mobj.mo(actor).angle) < 0x80000000_u32 {
                    state.p_mobj.mo_mut(actor).angle = exact;
                }
            } else {
                state.p_mobj.mo_mut(actor).angle = state
                    .p_mobj
                    .mo(actor)
                    .angle
                    .wrapping_add(TRACEANGLE as angle_t);
                if exact.wrapping_sub(state.p_mobj.mo(actor).angle) > 0x80000000_u32 {
                    state.p_mobj.mo_mut(actor).angle = exact;
                }
            }
        }
        exact = state.p_mobj.mo(actor).angle >> ANGLETOFINESHIFT;
        state.p_mobj.mo_mut(actor).momx = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(actor).type_0).speed as fixed_t,
            finecosine[exact as isize],
        );
        state.p_mobj.mo_mut(actor).momy = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(actor).type_0).speed as fixed_t,
            finesine[exact as usize],
        );
        dist = P_AproxDistance(
            state.p_mobj.mo(dest.unwrap()).x - state.p_mobj.mo(actor).x,
            state.p_mobj.mo(dest.unwrap()).y - state.p_mobj.mo(actor).y,
        );
        dist = (dist / state.info.mobjinfo_mut(state.p_mobj.mo(actor).type_0).speed) as fixed_t;
        if dist < 1_i32 {
            dist = 1_i32 as fixed_t;
        }
        slope = (state.p_mobj.mo(dest.unwrap()).z + 40 as fixed_t * FRACUNIT
            - state.p_mobj.mo(actor).z)
            / dist;
        if slope < state.p_mobj.mo(actor).momz {
            state.p_mobj.mo_mut(actor).momz -= FRACUNIT / 8_i32;
        } else {
            state.p_mobj.mo_mut(actor).momz += FRACUNIT / 8_i32;
        };
    }
}
pub fn A_SkelWhoosh(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        A_FaceTarget(state, actor);
        S_StartSound(state, SoundOrigin::Mobj(actor), sfx_skeswg as i32);
    }
}
pub fn A_SkelFist(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut damage: i32 = 0;
        let target = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(target) => target,
            None => return,
        };
        A_FaceTarget(state, actor);
        if P_CheckMeleeRange(state, actor) {
            damage = (P_Random(&mut state.m_random) % 10_i32 + 1_i32) * 6_i32;
            S_StartSound(state, SoundOrigin::Mobj(actor), sfx_skepch as i32);
            P_DamageMobj(state, target, Some(actor), Some(actor), damage);
        }
    }
}
pub fn PIT_VileCheck(state: &mut GameState, mut thing_id: MobjId) -> bool {
    let thing = thing_id;
    let mut maxdist: i32 = 0;
    let mut check: bool = false;
    if state.p_mobj.mo(thing).flags & MF_CORPSE as i32 == 0 {
        return true;
    }
    if state.p_mobj.mo(thing).tics != -1_i32 {
        return true;
    }
    if state
        .info
        .mobjinfo_mut(state.p_mobj.mo(thing).type_0)
        .raisestate
        == StateNum::S_NULL
    {
        return true;
    }
    maxdist = state
        .info
        .mobjinfo_mut(state.p_mobj.mo(thing).type_0)
        .radius
        + state.info.mobjinfo[MobjType::MT_VILE as usize].radius;
    if (state.p_mobj.mo(thing).x - state.p_enemy.viletryx).abs() > maxdist
        || (state.p_mobj.mo(thing).y - state.p_enemy.viletryy).abs() > maxdist
    {
        return true;
    }
    state.p_enemy.corpsehit = Some(thing);
    state.p_mobj.mo_mut(thing).momy = 0_i32 as fixed_t;
    state.p_mobj.mo_mut(thing).momx = state.p_mobj.mo(thing).momy;
    state.p_mobj.mo_mut(thing).height <<= 2_i32;
    check = P_CheckPosition(
        state,
        thing,
        state.p_mobj.mo(thing).x,
        state.p_mobj.mo(thing).y,
    );
    state.p_mobj.mo_mut(thing).height >>= 2_i32;
    if !check {
        return true;
    }
    false
}
pub fn A_VileChase(state: &mut GameState, id: MobjId) {
    let actor = id;
    let mut xl: i32 = 0;
    let mut xh: i32 = 0;
    let mut yl: i32 = 0;
    let mut yh: i32 = 0;
    let mut bx: i32 = 0;
    let mut by: i32 = 0;
    let mut temp: Option<MobjId> = None;
    if state.p_mobj.mo(actor).movedir != DirType::DI_NODIR as i32 {
        state.p_enemy.viletryx = state.p_mobj.mo(actor).x
            + state.info.mobjinfo_mut(state.p_mobj.mo(actor).type_0).speed as fixed_t
                * xspeed[state.p_mobj.mo(actor).movedir as usize];
        state.p_enemy.viletryy = state.p_mobj.mo(actor).y
            + state.info.mobjinfo_mut(state.p_mobj.mo(actor).type_0).speed as fixed_t
                * yspeed[state.p_mobj.mo(actor).movedir as usize];
        xl = (state.p_enemy.viletryx - state.p_setup.bmaporgx - 32_i32 * FRACUNIT * 2_i32)
            >> MAPBLOCKSHIFT;
        xh = (state.p_enemy.viletryx - state.p_setup.bmaporgx + 32_i32 * FRACUNIT * 2_i32)
            >> MAPBLOCKSHIFT;
        yl = (state.p_enemy.viletryy - state.p_setup.bmaporgy - 32_i32 * FRACUNIT * 2_i32)
            >> MAPBLOCKSHIFT;
        yh = (state.p_enemy.viletryy - state.p_setup.bmaporgy + 32_i32 * FRACUNIT * 2_i32)
            >> MAPBLOCKSHIFT;
        state.p_enemy.vileobj = Some(actor);
        bx = xl;
        while bx <= xh {
            by = yl;
            while by <= yh {
                if !P_BlockThingsIterator(state, bx, by, PIT_VileCheck) {
                    let corpsehit_id = state.p_enemy.corpsehit.unwrap();
                    let corpsehit = corpsehit_id;
                    temp = state.p_mobj.mo(actor).target;
                    state.p_mobj.mo_mut(actor).target = Some(corpsehit_id);
                    A_FaceTarget(state, actor);
                    state.p_mobj.mo_mut(actor).target = temp;
                    P_SetMobjState(state, actor, StateNum::S_VILE_HEAL1);
                    S_StartSound(state, SoundOrigin::Mobj(corpsehit_id), sfx_slop as i32);
                    let info = state.info.mobjinfo_mut(state.p_mobj.mo(corpsehit).type_0);
                    let (raisestate, info_flags, spawnhealth) =
                        (info.raisestate, info.flags, info.spawnhealth);
                    P_SetMobjState(state, corpsehit, raisestate);
                    state.p_mobj.mo_mut(corpsehit).height <<= 2_i32;
                    state.p_mobj.mo_mut(corpsehit).flags = info_flags;
                    state.p_mobj.mo_mut(corpsehit).health = spawnhealth;
                    state.p_mobj.mo_mut(corpsehit).target = None;
                    return;
                }
                by += 1;
            }
            bx += 1;
        }
    }
    A_Chase(state, actor);
}
pub fn A_VileStart(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        S_StartSound(state, SoundOrigin::Mobj(actor), sfx_vilatk as i32);
    }
}
pub fn A_StartFire(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        S_StartSound(state, SoundOrigin::Mobj(actor), sfx_flamst as i32);
        A_Fire(state, actor);
    }
}
pub fn A_FireCrackle(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        S_StartSound(state, SoundOrigin::Mobj(actor), sfx_flame as i32);
        A_Fire(state, actor);
    }
}
pub fn A_Fire(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut dest: Option<MobjId> = None;
        let mut target: MobjId;
        let mut an: u32 = 0;
        dest = state
            .p_mobj
            .mo(actor)
            .tracer
            .filter(|&id| state.p_mobj.is_live(id));
        if dest.is_none() {
            return;
        }
        let target_subst = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        let target_id = P_SubstNullMobj(&mut state.p_mobj, target_subst);
        target = target_id;
        if !P_CheckSight(state, target, dest.unwrap()) {
            return;
        }
        an = state.p_mobj.mo(dest.unwrap()).angle >> ANGLETOFINESHIFT;
        P_UnsetThingPosition(state, actor);
        state.p_mobj.mo_mut(actor).x = state.p_mobj.mo(dest.unwrap()).x
            + FixedMul(24 as fixed_t * FRACUNIT, finecosine[an as isize]);
        state.p_mobj.mo_mut(actor).y = state.p_mobj.mo(dest.unwrap()).y
            + FixedMul(24 as fixed_t * FRACUNIT, finesine[an as usize]);
        state.p_mobj.mo_mut(actor).z = state.p_mobj.mo(dest.unwrap()).z;
        P_SetThingPosition(state, actor);
    }
}
pub fn A_VileTarget(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut fog: MobjId;
        let target = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(target) => target,
            None => return,
        };
        A_FaceTarget(state, actor);
        fog = P_SpawnMobj(
            state,
            state.p_mobj.mo(target).x,
            state.p_mobj.mo(target).x,
            state.p_mobj.mo(target).z,
            MobjType::MT_FIRE,
        );
        state.p_mobj.mo_mut(actor).tracer = Some(fog);
        state.p_mobj.mo_mut(fog).target = Some(actor);
        state.p_mobj.mo_mut(fog).tracer = state.p_mobj.mo(actor).target;
        A_Fire(state, fog);
    }
}
pub fn A_VileAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut fire: Option<MobjId> = None;
        let mut an: i32 = 0;
        let target = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(target) => target,
            None => return,
        };
        A_FaceTarget(state, actor);
        if !P_CheckSight(state, actor, target) {
            return;
        }
        S_StartSound(state, SoundOrigin::Mobj(actor), sfx_barexp as i32);
        P_DamageMobj(state, target, Some(actor), Some(actor), 20_i32);
        state.p_mobj.mo_mut(target).momz = (1000_i32 * FRACUNIT
            / state.info.mobjinfo_mut(state.p_mobj.mo(target).type_0).mass)
            as fixed_t;
        an = (state.p_mobj.mo(actor).angle >> ANGLETOFINESHIFT) as i32;
        fire = state
            .p_mobj
            .mo(actor)
            .tracer
            .filter(|&id| state.p_mobj.is_live(id));
        if fire.is_none() {
            return;
        }
        state.p_mobj.mo_mut(fire.unwrap()).x =
            state.p_mobj.mo(target).x - FixedMul(24 as fixed_t * FRACUNIT, finecosine[an as isize]);
        state.p_mobj.mo_mut(fire.unwrap()).y =
            state.p_mobj.mo(target).y - FixedMul(24 as fixed_t * FRACUNIT, finesine[an as usize]);
        P_RadiusAttack(state, fire.unwrap(), Some(actor), 70_i32);
    }
}
pub const FATSPREAD: i32 = ANG90 / 8_i32;
pub fn A_FatRaise(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        A_FaceTarget(state, actor);
        S_StartSound(state, SoundOrigin::Mobj(actor), sfx_manatk as i32);
    }
}
pub fn A_FatAttack1(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut mo: MobjId;
        let mut target: MobjId;
        let mut an: i32 = 0;
        A_FaceTarget(state, actor);
        state.p_mobj.mo_mut(actor).angle = state
            .p_mobj
            .mo(actor)
            .angle
            .wrapping_add(FATSPREAD as angle_t);
        let target_subst = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        let target_id = P_SubstNullMobj(&mut state.p_mobj, target_subst);
        target = target_id;
        P_SpawnMissile(state, actor, target, MobjType::MT_FATSHOT);
        mo = P_SpawnMissile(state, actor, target, MobjType::MT_FATSHOT);
        state.p_mobj.mo_mut(mo).angle =
            state.p_mobj.mo(mo).angle.wrapping_add(FATSPREAD as angle_t);
        an = (state.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
        state.p_mobj.mo_mut(mo).momx = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).type_0).speed as fixed_t,
            finecosine[an as isize],
        );
        state.p_mobj.mo_mut(mo).momy = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).type_0).speed as fixed_t,
            finesine[an as usize],
        );
    }
}
pub fn A_FatAttack2(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut mo: MobjId;
        let mut target: MobjId;
        let mut an: i32 = 0;
        A_FaceTarget(state, actor);
        state.p_mobj.mo_mut(actor).angle = state
            .p_mobj
            .mo(actor)
            .angle
            .wrapping_sub(FATSPREAD as angle_t);
        let target_subst = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        let target_id = P_SubstNullMobj(&mut state.p_mobj, target_subst);
        target = target_id;
        P_SpawnMissile(state, actor, target, MobjType::MT_FATSHOT);
        mo = P_SpawnMissile(state, actor, target, MobjType::MT_FATSHOT);
        state.p_mobj.mo_mut(mo).angle = state
            .p_mobj
            .mo(mo)
            .angle
            .wrapping_sub((FATSPREAD * 2_i32) as angle_t);
        an = (state.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
        state.p_mobj.mo_mut(mo).momx = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).type_0).speed as fixed_t,
            finecosine[an as isize],
        );
        state.p_mobj.mo_mut(mo).momy = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).type_0).speed as fixed_t,
            finesine[an as usize],
        );
    }
}
pub fn A_FatAttack3(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut mo: MobjId;
        let mut target: MobjId;
        let mut an: i32 = 0;
        A_FaceTarget(state, actor);
        let target_subst = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        let target_id = P_SubstNullMobj(&mut state.p_mobj, target_subst);
        target = target_id;
        mo = P_SpawnMissile(state, actor, target, MobjType::MT_FATSHOT);
        state.p_mobj.mo_mut(mo).angle = state
            .p_mobj
            .mo(mo)
            .angle
            .wrapping_sub((FATSPREAD / 2_i32) as angle_t);
        an = (state.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
        state.p_mobj.mo_mut(mo).momx = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).type_0).speed as fixed_t,
            finecosine[an as isize],
        );
        state.p_mobj.mo_mut(mo).momy = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).type_0).speed as fixed_t,
            finesine[an as usize],
        );
        mo = P_SpawnMissile(state, actor, target, MobjType::MT_FATSHOT);
        state.p_mobj.mo_mut(mo).angle = state
            .p_mobj
            .mo(mo)
            .angle
            .wrapping_add((FATSPREAD / 2_i32) as angle_t);
        an = (state.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
        state.p_mobj.mo_mut(mo).momx = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).type_0).speed as fixed_t,
            finecosine[an as isize],
        );
        state.p_mobj.mo_mut(mo).momy = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).type_0).speed as fixed_t,
            finesine[an as usize],
        );
    }
}
pub const SKULLSPEED: i32 = 20 * FRACUNIT;
pub fn A_SkullAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut dest: MobjId;
        let mut an: angle_t = 0;
        let mut dist: i32 = 0;
        dest = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(dest) => dest,
            None => return,
        };
        state.p_mobj.mo_mut(actor).flags |= MF_SKULLFLY as i32;
        let attacksound = state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
            .attacksound;
        S_StartSound(state, SoundOrigin::Mobj(actor), attacksound);
        A_FaceTarget(state, actor);
        an = state.p_mobj.mo(actor).angle >> ANGLETOFINESHIFT;
        state.p_mobj.mo_mut(actor).momx = FixedMul(SKULLSPEED, finecosine[an as isize]);
        state.p_mobj.mo_mut(actor).momy = FixedMul(SKULLSPEED, finesine[an as usize]);
        dist = P_AproxDistance(
            state.p_mobj.mo(dest).x - state.p_mobj.mo(actor).x,
            state.p_mobj.mo(dest).y - state.p_mobj.mo(actor).y,
        );
        dist /= SKULLSPEED;
        if dist < 1_i32 {
            dist = 1_i32;
        }
        state.p_mobj.mo_mut(actor).momz = ((state.p_mobj.mo(dest).z
            + (state.p_mobj.mo(dest).height >> 1_i32)
            - state.p_mobj.mo(actor).z)
            / dist) as fixed_t;
    }
}
pub fn A_PainShootSkull(state: &mut GameState, actor: MobjId, mut angle: angle_t) {
    let mut x: fixed_t = 0;
    let mut y: fixed_t = 0;
    let mut z: fixed_t = 0;
    let mut newmobj: MobjId;
    let mut an: angle_t = 0;
    let mut prestep: i32 = 0;
    let mut count: i32 = 0;
    count += P_MobjThinkerIds(state)
        .into_iter()
        .filter(|&m| state.p_mobj.mo(m).type_0 as u32 == MobjType::MT_SKULL as i32 as u32)
        .count() as i32;
    if count > 20_i32 {
        return;
    }
    an = angle >> ANGLETOFINESHIFT;
    prestep = 4_i32 * FRACUNIT
        + 3_i32
            * (state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
                .radius
                + state.info.mobjinfo[MobjType::MT_SKULL as usize].radius)
            / 2_i32;
    x = state.p_mobj.mo(actor).x + FixedMul(prestep as fixed_t, finecosine[an as isize]);
    y = state.p_mobj.mo(actor).y + FixedMul(prestep as fixed_t, finesine[an as usize]);
    z = (state.p_mobj.mo(actor).z + 8_i32 * FRACUNIT) as fixed_t;
    newmobj = P_SpawnMobj(state, x, y, z, MobjType::MT_SKULL);
    let (new_x, new_y) = {
        let n = state.p_mobj.mo(newmobj);
        (n.x, n.y)
    };
    if !P_TryMove(state, newmobj, new_x, new_y) {
        P_DamageMobj(state, newmobj, Some(actor), Some(actor), 10000_i32);
        return;
    }
    state.p_mobj.mo_mut(newmobj).target = state.p_mobj.mo(actor).target;
    A_SkullAttack(state, newmobj);
}
pub fn A_PainAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        A_FaceTarget(state, actor);
        A_PainShootSkull(state, actor, state.p_mobj.mo(actor).angle);
    }
}
pub fn A_PainDie(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        A_Fall(state, actor);
        A_PainShootSkull(
            state,
            actor,
            state.p_mobj.mo(actor).angle.wrapping_add(ANG90 as angle_t),
        );
        A_PainShootSkull(
            state,
            actor,
            state.p_mobj.mo(actor).angle.wrapping_add(ANG180),
        );
        A_PainShootSkull(
            state,
            actor,
            state.p_mobj.mo(actor).angle.wrapping_add(ANG270),
        );
    }
}
pub fn A_Scream(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut sound: i32 = 0;
        match state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
            .deathsound
        {
            0 => return,
            59..=61 => {
                sound = sfx_podth1 as i32 + P_Random(&mut state.m_random) % 3_i32;
            }
            62 | 63 => {
                sound = sfx_bgdth1 as i32 + P_Random(&mut state.m_random) % 2_i32;
            }
            _ => {
                sound = state
                    .info
                    .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
                    .deathsound;
            }
        }
        if state.p_mobj.mo(actor).type_0 as u32 == MobjType::MT_SPIDER as i32 as u32
            || state.p_mobj.mo(actor).type_0 as u32 == MobjType::MT_CYBORG as i32 as u32
        {
            S_StartSound(state, SoundOrigin::None, sound);
        } else {
            S_StartSound(state, SoundOrigin::Mobj(actor), sound);
        };
    }
}
pub fn A_XScream(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        S_StartSound(state, SoundOrigin::Mobj(actor), sfx_slop as i32);
    }
}
pub fn A_Pain(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let painsound = state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).type_0)
            .painsound;
        if painsound != 0 {
            S_StartSound(state, SoundOrigin::Mobj(actor), painsound);
        }
    }
}
pub fn A_Fall(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        state.p_mobj.mo_mut(actor).flags &= !(MF_SOLID as i32);
    }
}
pub fn A_Explode(state: &mut GameState, id: MobjId) {
    let thingy = id;
    let target = state
        .p_mobj
        .mo(thingy)
        .target
        .filter(|&id| state.p_mobj.is_live(id));
    P_RadiusAttack(state, thingy, target, 128_i32);
}
fn CheckBossEnd(state: &mut GameState, mut motype: MobjType) -> bool {
    if !state.doomstat.gameversion.is_ultimate_or_higher() {
        if state.g_game.gamemap != 8_i32 {
            return false;
        }
        if motype as u32 == MobjType::MT_BRUISER as i32 as u32 && state.g_game.gameepisode != 1_i32
        {
            return false;
        }
        true
    } else {
        match state.g_game.gameepisode {
            1 => {
                state.g_game.gamemap == 8_i32 && motype as u32 == MobjType::MT_BRUISER as i32 as u32
            }
            2 => {
                state.g_game.gamemap == 8_i32 && motype as u32 == MobjType::MT_CYBORG as i32 as u32
            }
            3 => {
                state.g_game.gamemap == 8_i32 && motype as u32 == MobjType::MT_SPIDER as i32 as u32
            }
            4 => {
                state.g_game.gamemap == 6_i32 && motype as u32 == MobjType::MT_CYBORG as i32 as u32
                    || state.g_game.gamemap == 8_i32
                        && motype as u32 == MobjType::MT_SPIDER as i32 as u32
            }
            _ => state.g_game.gamemap == 8_i32,
        }
    }
}
pub fn A_BossDeath(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        let mut i: i32 = 0;
        if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
            if state.g_game.gamemap != 7_i32 {
                return;
            }
            if state.p_mobj.mo(mo).type_0 as u32 != MobjType::MT_FATSO as i32 as u32
                && state.p_mobj.mo(mo).type_0 as u32 != MobjType::MT_BABY as i32 as u32
            {
                return;
            }
        } else if !CheckBossEnd(state, state.p_mobj.mo(mo).type_0) {
            return;
        }
        i = 0_i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize]
                && state.g_game.players[i as usize].health > 0_i32
            {
                break;
            }
            i += 1;
        }
        if i == MAXPLAYERS {
            return;
        }
        let mo_type = state.p_mobj.mo(mo).type_0;
        for mo2 in P_MobjThinkerIds(state) {
            if mo2 != mo
                && state.p_mobj.mo(mo2).type_0 as u32 == mo_type as u32
                && state.p_mobj.mo(mo2).health > 0_i32
            {
                return;
            }
        }
        if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
            if state.g_game.gamemap == 7_i32 {
                if state.p_mobj.mo(mo).type_0 as u32 == MobjType::MT_FATSO as i32 as u32 {
                    let junk = state.p_setup.junk_line(666_i16);
                    EV_DoFloor(state, junk, FloorE::lowerFloorToLowest);
                    return;
                }
                if state.p_mobj.mo(mo).type_0 as u32 == MobjType::MT_BABY as i32 as u32 {
                    let junk = state.p_setup.junk_line(667_i16);
                    EV_DoFloor(state, junk, FloorE::raiseToTexture);
                    return;
                }
            }
        } else {
            match state.g_game.gameepisode {
                1 => {
                    let junk = state.p_setup.junk_line(666_i16);
                    EV_DoFloor(state, junk, FloorE::lowerFloorToLowest);
                    return;
                }
                4 => match state.g_game.gamemap {
                    6 => {
                        let junk = state.p_setup.junk_line(666_i16);
                        EV_DoDoor(state, junk, VldoorE::vld_blazeOpen);
                        return;
                    }
                    8 => {
                        let junk = state.p_setup.junk_line(666_i16);
                        EV_DoFloor(state, junk, FloorE::lowerFloorToLowest);
                        return;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        G_ExitLevel(state);
    }
}
pub fn A_Hoof(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        S_StartSound(state, SoundOrigin::Mobj(mo), sfx_hoof as i32);
        A_Chase(state, mo);
    }
}
pub fn A_Metal(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        S_StartSound(state, SoundOrigin::Mobj(mo), sfx_metal as i32);
        A_Chase(state, mo);
    }
}
pub fn A_BabyMetal(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        S_StartSound(state, SoundOrigin::Mobj(mo), sfx_bspwlk as i32);
        A_Chase(state, mo);
    }
}
pub fn A_OpenShotgun2(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        S_StartSound(
            state,
            SoundOrigin::Mobj(state.g_game.players[player.0 as usize].mo.unwrap()),
            sfx_dbopn as i32,
        );
    }
}
pub fn A_LoadShotgun2(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        S_StartSound(
            state,
            SoundOrigin::Mobj(state.g_game.players[player.0 as usize].mo.unwrap()),
            sfx_dbload as i32,
        );
    }
}
pub fn A_CloseShotgun2(state: &mut GameState, player_id: PlayerId, position: i32) {
    {
        let player = player_id;
        S_StartSound(
            state,
            SoundOrigin::Mobj(state.g_game.players[player.0 as usize].mo.unwrap()),
            sfx_dbcls as i32,
        );
        A_ReFire(state, player_id, position);
    }
}
pub fn A_BrainAwake(state: &mut GameState, _id: MobjId) {
    state.p_enemy.numbraintargets = 0_i32;
    state.p_enemy.braintargeton = 0_i32;
    for m in P_MobjThinkerIds(state) {
        if state.p_mobj.mo(m).type_0 as u32 == MobjType::MT_BOSSTARGET as i32 as u32 {
            let n = state.p_enemy.numbraintargets as usize;
            state.p_enemy.braintargets[n] = Some(m);
            state.p_enemy.numbraintargets += 1;
        }
    }
    S_StartSound(state, SoundOrigin::None, sfx_bossit as i32);
}
pub fn A_BrainPain(state: &mut GameState, _id: MobjId) {
    S_StartSound(state, SoundOrigin::None, sfx_bospn as i32);
}
pub fn A_BrainScream(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut z: i32 = 0;
        let mut th: MobjId;
        x = state.p_mobj.mo(mo).x - 196_i32 * FRACUNIT;
        while x < state.p_mobj.mo(mo).x + 320_i32 * FRACUNIT {
            y = state.p_mobj.mo(mo).y - 320_i32 * FRACUNIT;
            z = 128_i32 + P_Random(&mut state.m_random) * 2_i32 * FRACUNIT;
            th = P_SpawnMobj(
                state,
                x as fixed_t,
                y as fixed_t,
                z as fixed_t,
                MobjType::MT_ROCKET,
            );
            state.p_mobj.mo_mut(th).momz = (P_Random(&mut state.m_random) * 512_i32) as fixed_t;
            P_SetMobjState(state, th, StateNum::S_BRAINEXPLODE1);
            state.p_mobj.mo_mut(th).tics -= P_Random(&mut state.m_random) & 7_i32;
            if state.p_mobj.mo(th).tics < 1_i32 {
                state.p_mobj.mo_mut(th).tics = 1_i32;
            }
            x += FRACUNIT * 8_i32;
        }
        S_StartSound(state, SoundOrigin::None, sfx_bosdth as i32);
    }
}
pub fn A_BrainExplode(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut z: i32 = 0;
        let mut th: MobjId;
        x = state.p_mobj.mo(mo).x
            + (P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) * 2048_i32;
        y = state.p_mobj.mo(mo).y;
        z = 128_i32 + P_Random(&mut state.m_random) * 2_i32 * FRACUNIT;
        th = P_SpawnMobj(
            state,
            x as fixed_t,
            y as fixed_t,
            z as fixed_t,
            MobjType::MT_ROCKET,
        );
        state.p_mobj.mo_mut(th).momz = (P_Random(&mut state.m_random) * 512_i32) as fixed_t;
        P_SetMobjState(state, th, StateNum::S_BRAINEXPLODE1);
        state.p_mobj.mo_mut(th).tics -= P_Random(&mut state.m_random) & 7_i32;
        if state.p_mobj.mo(th).tics < 1_i32 {
            state.p_mobj.mo_mut(th).tics = 1_i32;
        }
    }
}
pub fn A_BrainDie(state: &mut GameState, _id: MobjId) {
    G_ExitLevel(state);
}
pub fn A_BrainSpit(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        let mut targ: MobjId;
        let mut newmobj: MobjId;
        state.p_enemy.easy ^= 1_i32;
        if state.g_game.gameskill <= SkillType::sk_easy && state.p_enemy.easy == 0 {
            return;
        }
        let targ_id = state.p_enemy.braintargets[state.p_enemy.braintargeton as usize].unwrap();
        targ = targ_id;
        state.p_enemy.braintargeton =
            (state.p_enemy.braintargeton + 1_i32) % state.p_enemy.numbraintargets;
        newmobj = P_SpawnMissile(state, mo, targ, MobjType::MT_SPAWNSHOT);
        state.p_mobj.mo_mut(newmobj).target = Some(targ);
        state.p_mobj.mo_mut(newmobj).reactiontime = (state.p_mobj.mo(targ).y
            - state.p_mobj.mo(mo).y)
            / state.p_mobj.mo(newmobj).momy
            / state
                .info
                .state_mut(state.p_mobj.mo(newmobj).state.unwrap())
                .tics;
        S_StartSound(state, SoundOrigin::None, sfx_bospit as i32);
    }
}
pub fn A_SpawnSound(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        S_StartSound(state, SoundOrigin::Mobj(mo), sfx_boscub as i32);
        A_SpawnFly(state, mo);
    }
}
pub fn A_SpawnFly(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        let mut newmobj: MobjId;
        let mut fog: MobjId;
        let mut targ: MobjId;
        let mut r: i32 = 0;
        let mut type_0: MobjType = MobjType::MT_PLAYER;
        state.p_mobj.mo_mut(mo).reactiontime -= 1;
        if state.p_mobj.mo(mo).reactiontime != 0 {
            return;
        }
        let targ_subst = state
            .p_mobj
            .mo(mo)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        let targ_id = P_SubstNullMobj(&mut state.p_mobj, targ_subst);
        targ = targ_id;
        fog = P_SpawnMobj(
            state,
            state.p_mobj.mo(targ).x,
            state.p_mobj.mo(targ).y,
            state.p_mobj.mo(targ).z,
            MobjType::MT_SPAWNFIRE,
        );
        S_StartSound(state, SoundOrigin::Mobj(fog), sfx_telept as i32);
        r = P_Random(&mut state.m_random);
        if r < 50_i32 {
            type_0 = MobjType::MT_TROOP;
        } else if r < 90_i32 {
            type_0 = MobjType::MT_SERGEANT;
        } else if r < 120_i32 {
            type_0 = MobjType::MT_SHADOWS;
        } else if r < 130_i32 {
            type_0 = MobjType::MT_PAIN;
        } else if r < 160_i32 {
            type_0 = MobjType::MT_HEAD;
        } else if r < 162_i32 {
            type_0 = MobjType::MT_VILE;
        } else if r < 172_i32 {
            type_0 = MobjType::MT_UNDEAD;
        } else if r < 192_i32 {
            type_0 = MobjType::MT_BABY;
        } else if r < 222_i32 {
            type_0 = MobjType::MT_FATSO;
        } else if r < 246_i32 {
            type_0 = MobjType::MT_KNIGHT;
        } else {
            type_0 = MobjType::MT_BRUISER;
        }
        newmobj = P_SpawnMobj(
            state,
            state.p_mobj.mo(targ).x,
            state.p_mobj.mo(targ).y,
            state.p_mobj.mo(targ).z,
            type_0,
        );
        if P_LookForPlayers(state, newmobj, true) {
            let seestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(newmobj).type_0)
                .seestate;
            P_SetMobjState(state, newmobj, seestate);
        }
        P_TeleportMove(
            state,
            newmobj,
            state.p_mobj.mo(newmobj).x,
            state.p_mobj.mo(newmobj).y,
        );
        P_RemoveMobj(state, mo);
    }
}
pub fn A_PlayerScream(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        let mut sound: i32 = sfx_pldeth as i32;
        if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32
            && state.p_mobj.mo(mo).health < -50_i32
        {
            sound = sfx_pdiehi as i32;
        }
        S_StartSound(state, SoundOrigin::Mobj(mo), sound);
    }
}
