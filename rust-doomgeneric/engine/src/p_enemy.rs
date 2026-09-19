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
use crate::sounds::SfxName;
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
pub const MISSILERANGE: i32 = 32 * 64 * FRACUNIT;
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
        if s.validcount == validcount && s.soundtraversed <= soundblocks + 1 {
            return;
        }
        s.validcount = validcount;
        s.soundtraversed = soundblocks + 1;
        s.soundtarget = state.p_enemy.soundtarget;
    }
    let linecount = state.p_setup.sector_mut(sec).linecount;
    for i in 0..linecount {
        let check = state.p_setup.sector_mut(sec).lines[i as usize];
        let checkv = state.p_setup.line(check);
        if checkv.flags as i32 & ML_TWOSIDED != 0 {
            P_LineOpening(state, check);
            if state.p_maputl.openrange > 0 {
                let other = if state.p_setup.sides[checkv.sidenum[0] as usize].sector == sec {
                    state.p_setup.sides[checkv.sidenum[1] as usize].sector
                } else {
                    state.p_setup.sides[checkv.sidenum[0] as usize].sector
                };
                if checkv.flags as i32 & ML_SOUNDBLOCK != 0 {
                    if soundblocks == 0 {
                        P_RecursiveSound(state, other, 1);
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
    P_RecursiveSound(state, sec, 0);
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
        (p.x, p.y, p.kind)
    };
    let (actor_x, actor_y) = {
        let a = state.p_mobj.mo(actor);
        (a.x, a.y)
    };
    let dist = P_AproxDistance(pl_x - actor_x, pl_y - actor_y);
    if dist >= MELEERANGE - 20 * FRACUNIT + state.info.mobjinfo_mut(pl_type).radius {
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
    if state.p_mobj.mo(actor).flags & MF_JUSTHIT != 0 {
        state.p_mobj.mo_mut(actor).flags &= !MF_JUSTHIT;
        return true;
    }
    if state.p_mobj.mo(actor).reactiontime != 0 {
        return false;
    }
    let (actor_x, actor_y, actor_type) = {
        let a = state.p_mobj.mo(actor);
        (a.x, a.y, a.kind)
    };
    let (target_x, target_y) = {
        let t = state.p_mobj.mo(target);
        (t.x, t.y)
    };
    let mut dist: fixed_t =
        (P_AproxDistance(actor_x - target_x, actor_y - target_y) - 64 * FRACUNIT) as fixed_t;
    if state.info.mobjinfo_mut(actor_type).meleestate == StateNum::S_NULL {
        dist -= 128 * FRACUNIT;
    }
    dist >>= 16;
    if actor_type as u32 == MobjType::MT_VILE as i32 as u32 && dist > 14 * 64 {
        return false;
    }
    if actor_type as u32 == MobjType::MT_UNDEAD as i32 as u32 {
        if dist < 196 {
            return false;
        }
        dist >>= 1;
    }
    if actor_type as u32 == MobjType::MT_CYBORG as i32 as u32
        || actor_type as u32 == MobjType::MT_SPIDER as i32 as u32
        || actor_type as u32 == MobjType::MT_SKULL as i32 as u32
    {
        dist >>= 1;
    }
    if dist > 200 {
        dist = 200;
    }
    if actor_type as u32 == MobjType::MT_CYBORG as i32 as u32 && dist > 160 {
        dist = 160;
    }
    if P_Random(&mut state.m_random) < dist {
        return false;
    }
    true
}
pub static xspeed: [fixed_t; 8] = [FRACUNIT, 47000, 0, -47000, -FRACUNIT, -47000, 0, 47000];
pub static yspeed: [fixed_t; 8] = [0, 47000, FRACUNIT, 47000, 0, -47000, -FRACUNIT, -47000];
pub fn P_Move(state: &mut GameState, actor: MobjId) -> bool {
    let mut ld: LineId;

    let mut good: bool;
    if state.p_mobj.mo(actor).movedir == DirType::DI_NODIR as i32 {
        return false;
    }
    if state.p_mobj.mo(actor).movedir as u32 >= 8 {
        I_Error("Weird actor->movedir!");
    }
    let tryx: fixed_t = state.p_mobj.mo(actor).x
        + state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed as fixed_t
            * xspeed[state.p_mobj.mo(actor).movedir as usize];
    let tryy: fixed_t = state.p_mobj.mo(actor).y
        + state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed as fixed_t
            * yspeed[state.p_mobj.mo(actor).movedir as usize];
    let try_ok: bool = P_TryMove(state, actor, tryx, tryy);
    if !try_ok {
        if state.p_mobj.mo(actor).flags & MF_FLOAT != 0 && state.p_map.floatok {
            if state.p_mobj.mo(actor).z < state.p_map.tmfloorz {
                state.p_mobj.mo_mut(actor).z += FLOATSPEED;
            } else {
                state.p_mobj.mo_mut(actor).z -= FLOATSPEED;
            }
            state.p_mobj.mo_mut(actor).flags |= MF_INFLOAT;
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
            if P_UseSpecialLine(state, actor, ld, 0) {
                good = true;
            }
        }
        return good;
    } else {
        state.p_mobj.mo_mut(actor).flags &= !MF_INFLOAT;
    }
    if state.p_mobj.mo(actor).flags & MF_FLOAT == 0 {
        state.p_mobj.mo_mut(actor).z = state.p_mobj.mo(actor).floorz;
    }
    true
}
pub fn P_TryWalk(state: &mut GameState, actor: MobjId) -> bool {
    if !P_Move(state, actor) {
        return false;
    }
    state.p_mobj.mo_mut(actor).movecount = P_Random(&mut state.m_random) & 15;
    true
}
pub fn P_NewChaseDir(state: &mut GameState, actor: MobjId) {
    let mut d: [DirType; 3] = [DirType::DI_EAST; 3];
    let mut tdir: i32;

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
    let olddir: DirType = dirtype_from_movedir(state.p_mobj.mo(actor).movedir);
    let turnaround: DirType = opposite[olddir as usize];
    let deltax: fixed_t = state.p_mobj.mo(target).x - state.p_mobj.mo(actor).x;
    let deltay: fixed_t = state.p_mobj.mo(target).y - state.p_mobj.mo(actor).y;
    if deltax > 10 * FRACUNIT {
        d[1] = DirType::DI_EAST;
    } else if deltax < -10 * FRACUNIT {
        d[1] = DirType::DI_WEST;
    } else {
        d[1] = DirType::DI_NODIR;
    }
    if deltay < -10 * FRACUNIT {
        d[2] = DirType::DI_SOUTH;
    } else if deltay > 10 * FRACUNIT {
        d[2] = DirType::DI_NORTH;
    } else {
        d[2] = DirType::DI_NODIR;
    }
    if d[1] != DirType::DI_NODIR && d[2] != DirType::DI_NODIR {
        state.p_mobj.mo_mut(actor).movedir =
            diags[((((deltay < 0) as i32) << 1) + (deltax > 0) as i32) as usize] as i32;
        if state.p_mobj.mo(actor).movedir != turnaround as i32 && P_TryWalk(state, actor) {
            return;
        }
    }
    if P_Random(&mut state.m_random) > 200 || deltay.abs() > deltax.abs() {
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
    if P_Random(&mut state.m_random) & 1 != 0 {
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
        while tdir != DirType::DI_EAST as i32 - 1 {
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
    let stop = (state.p_mobj.mo(actor).lastlook - 1) & 3;
    loop {
        let lastlook = state.p_mobj.mo(actor).lastlook;
        if state.g_game.playeringame[lastlook as usize] {
            let fresh1 = c;
            c += 1;
            if fresh1 == 2 || lastlook == stop {
                return false;
            }
            let (health, player_mo) = {
                let player = &state.g_game.players[lastlook as usize];
                (player.health, player.mo)
            };
            if health > 0 {
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
        state.p_mobj.mo_mut(actor).lastlook = (lastlook + 1) & 3;
    }
}
pub fn A_KeenDie(state: &mut GameState, id: MobjId) {
    let mo = id;
    A_Fall(state, mo);
    let mo_type = state.p_mobj.mo(mo).kind;
    for mo2 in P_MobjThinkerIds(state) {
        if mo2 != mo
            && state.p_mobj.mo(mo2).kind as u32 == mo_type as u32
            && state.p_mobj.mo(mo2).health > 0
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
        let current_block: u64;

        state.p_mobj.mo_mut(actor).threshold = 0;
        let targ: Option<MobjId> = state
            .p_setup
            .sector_mut(
                state.p_setup.subsectors[state.p_mobj.mo(actor).subsector.0 as usize].sector,
            )
            .soundtarget
            .filter(|&id| state.p_mobj.is_live(id));
        if let Some(targ) = targ.filter(|&t| state.p_mobj.mo(t).flags & MF_SHOOTABLE != 0) {
            state.p_mobj.mo_mut(actor).target = Some(targ);
            if state.p_mobj.mo(actor).flags & MF_AMBUSH != 0 {
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
            .mobjinfo_mut(state.p_mobj.mo(actor).kind)
            .seesound
            != 0
        {
            let sound: i32 = match state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                .seesound
            {
                36..=38 => SfxName::sfx_posit1 as i32 + P_Random(&mut state.m_random) % 3,
                39 | 40 => SfxName::sfx_bgsit1 as i32 + P_Random(&mut state.m_random) % 2,
                _ => {
                    state
                        .info
                        .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                        .seesound
                }
            };
            if state.p_mobj.mo(actor).kind as u32 == MobjType::MT_SPIDER as i32 as u32
                || state.p_mobj.mo(actor).kind as u32 == MobjType::MT_CYBORG as i32 as u32
            {
                S_StartSound(state, SoundOrigin::None, sound);
            } else {
                S_StartSound(state, SoundOrigin::Mobj(actor), sound);
            }
        }
        let seestate = state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).kind)
            .seestate;
        P_SetMobjState(state, actor, seestate);
    }
}
pub fn A_Chase(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let delta: i32;
        if state.p_mobj.mo(actor).reactiontime != 0 {
            state.p_mobj.mo_mut(actor).reactiontime -= 1;
        }
        let target = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        if state.p_mobj.mo(actor).threshold != 0 {
            if target.is_none() || state.p_mobj.mo(target.unwrap()).health <= 0 {
                state.p_mobj.mo_mut(actor).threshold = 0;
            } else {
                state.p_mobj.mo_mut(actor).threshold -= 1;
            }
        }
        if state.p_mobj.mo(actor).movedir < 8 {
            state.p_mobj.mo_mut(actor).angle &= (7 << 29) as angle_t;
            delta = state
                .p_mobj
                .mo(actor)
                .angle
                .wrapping_sub((state.p_mobj.mo(actor).movedir << 29) as angle_t)
                as i32;
            if delta > 0 {
                state.p_mobj.mo_mut(actor).angle = state
                    .p_mobj
                    .mo(actor)
                    .angle
                    .wrapping_sub((ANG90 / 2) as angle_t);
            } else if delta < 0 {
                state.p_mobj.mo_mut(actor).angle = state
                    .p_mobj
                    .mo(actor)
                    .angle
                    .wrapping_add((ANG90 / 2) as angle_t);
            }
        }
        if target.is_none() || state.p_mobj.mo(target.unwrap()).flags & MF_SHOOTABLE == 0 {
            if P_LookForPlayers(state, actor, true) {
                return;
            }
            let spawnstate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                .spawnstate;
            P_SetMobjState(state, actor, spawnstate);
            return;
        }
        if state.p_mobj.mo(actor).flags & MF_JUSTATTACKED != 0 {
            state.p_mobj.mo_mut(actor).flags &= !MF_JUSTATTACKED;
            if state.g_game.gameskill != SkillType::sk_nightmare && !state.d_main.fastparm {
                P_NewChaseDir(state, actor);
            }
            return;
        }
        let actor_info = state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind);
        if actor_info.meleestate != StateNum::S_NULL && P_CheckMeleeRange(state, actor) {
            let attacksound = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                .attacksound;
            if attacksound != 0 {
                S_StartSound(state, SoundOrigin::Mobj(actor), attacksound);
            }
            let meleestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                .meleestate;
            P_SetMobjState(state, actor, meleestate);
            return;
        }
        if state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).kind)
            .missilestate
            != StateNum::S_NULL
            && !(state.g_game.gameskill < SkillType::sk_nightmare
                && !state.d_main.fastparm
                && state.p_mobj.mo(actor).movecount != 0)
            && P_CheckMissileRange(state, actor)
        {
            let missilestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                .missilestate;
            P_SetMobjState(state, actor, missilestate);
            state.p_mobj.mo_mut(actor).flags |= MF_JUSTATTACKED;
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
        if state.p_mobj.mo(actor).movecount < 0 || !P_Move(state, actor) {
            P_NewChaseDir(state, actor);
        }
        let activesound = state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).kind)
            .activesound;
        if activesound != 0 && P_Random(&mut state.m_random) < 3 {
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
        state.p_mobj.mo_mut(actor).flags &= !MF_AMBUSH;
        state.p_mobj.mo_mut(actor).angle = R_PointToAngle2(
            state,
            state.p_mobj.mo(actor).x,
            state.p_mobj.mo(actor).y,
            state.p_mobj.mo(target).x,
            state.p_mobj.mo(target).y,
        );
        if state.p_mobj.mo(target).flags & MF_SHADOW != 0 {
            state.p_mobj.mo_mut(actor).angle = state.p_mobj.mo(actor).angle.wrapping_add(
                ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 21) as angle_t,
            );
        }
    }
}
pub fn A_PosAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut angle: i32;

        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        A_FaceTarget(state, actor);
        angle = state.p_mobj.mo(actor).angle as i32;
        let slope: i32 = P_AimLineAttack(state, Some(actor), angle as angle_t, MISSILERANGE);
        S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_pistol as i32);
        angle += (P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 20;
        let damage: i32 = (P_Random(&mut state.m_random) % 5 + 1) * 3;
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
        let mut i: i32;
        let mut angle: i32;

        let mut damage: i32;

        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_shotgn as i32);
        A_FaceTarget(state, actor);
        let bangle: i32 = state.p_mobj.mo(actor).angle as i32;
        let slope: i32 = P_AimLineAttack(state, Some(actor), bangle as angle_t, MISSILERANGE);
        i = 0;
        while i < 3 {
            angle =
                bangle + ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 20);
            damage = (P_Random(&mut state.m_random) % 5 + 1) * 3;
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

        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_shotgn as i32);
        A_FaceTarget(state, actor);
        let bangle: i32 = state.p_mobj.mo(actor).angle as i32;
        let slope: i32 = P_AimLineAttack(state, Some(actor), bangle as angle_t, MISSILERANGE);
        let angle: i32 =
            bangle + ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 20);
        let damage: i32 = (P_Random(&mut state.m_random) % 5 + 1) * 3;
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
        if P_Random(&mut state.m_random) < 40 {
            return;
        }
        let target = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        if target.is_none()
            || state.p_mobj.mo(target.unwrap()).health <= 0
            || !P_CheckSight(state, actor, state.p_mobj.mo(target.unwrap()).id)
        {
            let seestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                .seestate;
            P_SetMobjState(state, actor, seestate);
        }
    }
}
pub fn A_SpidRefire(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        A_FaceTarget(state, actor);
        if P_Random(&mut state.m_random) < 10 {
            return;
        }
        let target = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        if target.is_none()
            || state.p_mobj.mo(target.unwrap()).health <= 0
            || !P_CheckSight(state, actor, state.p_mobj.mo(target.unwrap()).id)
        {
            let seestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
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
        let damage: i32;
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
            S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_claw as i32);
            damage = (P_Random(&mut state.m_random) % 8 + 1) * 3;
            P_DamageMobj(state, target, Some(actor), Some(actor), damage);
            return;
        }
        P_SpawnMissile(state, actor, target, MobjType::MT_TROOPSHOT);
    }
}
pub fn A_SargAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let damage: i32;
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
            damage = (P_Random(&mut state.m_random) % 10 + 1) * 4;
            P_DamageMobj(state, target, Some(actor), Some(actor), damage);
        }
    }
}
pub fn A_HeadAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let damage: i32;
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
            damage = (P_Random(&mut state.m_random) % 6 + 1) * 10;
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
        let damage: i32;
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
            S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_claw as i32);
            damage = (P_Random(&mut state.m_random) % 8 + 1) * 10;
            P_DamageMobj(state, target, Some(actor), Some(actor), damage);
            return;
        }
        P_SpawnMissile(state, actor, target, MobjType::MT_BRUISERSHOT);
    }
}
pub fn A_SkelMissile(state: &mut GameState, id: MobjId) {
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
        state.p_mobj.mo_mut(actor).z += 16 * FRACUNIT;
        let mo: MobjId = P_SpawnMissile(state, actor, target, MobjType::MT_TRACER);
        state.p_mobj.mo_mut(actor).z -= 16 * FRACUNIT;
        state.p_mobj.mo_mut(mo).x += state.p_mobj.mo(mo).momx;
        state.p_mobj.mo_mut(mo).y += state.p_mobj.mo(mo).momy;
        state.p_mobj.mo_mut(mo).tracer = state.p_mobj.mo(actor).target;
    }
}
pub static TRACEANGLE: i32 = 0xc000000;
pub fn A_Tracer(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut exact: angle_t;
        let mut dist: fixed_t;

        if state.d_loop.gametic & 3 != 0 {
            return;
        }
        P_SpawnPuff(
            state,
            state.p_mobj.mo(actor).x,
            state.p_mobj.mo(actor).y,
            state.p_mobj.mo(actor).z,
        );
        let th: MobjId = P_SpawnMobj(
            state,
            state.p_mobj.mo(actor).x - state.p_mobj.mo(actor).momx,
            state.p_mobj.mo(actor).y - state.p_mobj.mo(actor).momy,
            state.p_mobj.mo(actor).z,
            MobjType::MT_SMOKE,
        );
        state.p_mobj.mo_mut(th).momz = FRACUNIT as fixed_t;
        state.p_mobj.mo_mut(th).tics -= P_Random(&mut state.m_random) & 3;
        if state.p_mobj.mo(th).tics < 1 {
            state.p_mobj.mo_mut(th).tics = 1;
        }
        let dest: Option<MobjId> = state
            .p_mobj
            .mo(actor)
            .tracer
            .filter(|&id| state.p_mobj.is_live(id));
        if dest.is_none() || state.p_mobj.mo(dest.unwrap()).health <= 0 {
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
            if exact.wrapping_sub(state.p_mobj.mo(actor).angle) > 0x80000000 {
                state.p_mobj.mo_mut(actor).angle = state
                    .p_mobj
                    .mo(actor)
                    .angle
                    .wrapping_sub(TRACEANGLE as angle_t);
                if exact.wrapping_sub(state.p_mobj.mo(actor).angle) < 0x80000000 {
                    state.p_mobj.mo_mut(actor).angle = exact;
                }
            } else {
                state.p_mobj.mo_mut(actor).angle = state
                    .p_mobj
                    .mo(actor)
                    .angle
                    .wrapping_add(TRACEANGLE as angle_t);
                if exact.wrapping_sub(state.p_mobj.mo(actor).angle) > 0x80000000 {
                    state.p_mobj.mo_mut(actor).angle = exact;
                }
            }
        }
        exact = state.p_mobj.mo(actor).angle >> ANGLETOFINESHIFT;
        state.p_mobj.mo_mut(actor).momx = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed as fixed_t,
            finecosine[exact as usize],
        );
        state.p_mobj.mo_mut(actor).momy = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed as fixed_t,
            finesine[exact as usize],
        );
        dist = P_AproxDistance(
            state.p_mobj.mo(dest.unwrap()).x - state.p_mobj.mo(actor).x,
            state.p_mobj.mo(dest.unwrap()).y - state.p_mobj.mo(actor).y,
        );
        dist = (dist / state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed) as fixed_t;
        if dist < 1 {
            dist = 1;
        }
        let slope: fixed_t =
            (state.p_mobj.mo(dest.unwrap()).z + 40 * FRACUNIT - state.p_mobj.mo(actor).z) / dist;
        if slope < state.p_mobj.mo(actor).momz {
            state.p_mobj.mo_mut(actor).momz -= FRACUNIT / 8;
        } else {
            state.p_mobj.mo_mut(actor).momz += FRACUNIT / 8;
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
        S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_skeswg as i32);
    }
}
pub fn A_SkelFist(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let damage: i32;
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
            damage = (P_Random(&mut state.m_random) % 10 + 1) * 6;
            S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_skepch as i32);
            P_DamageMobj(state, target, Some(actor), Some(actor), damage);
        }
    }
}
pub fn PIT_VileCheck(state: &mut GameState, thing_id: MobjId) -> bool {
    let thing = thing_id;

    if state.p_mobj.mo(thing).flags & MF_CORPSE == 0 {
        return true;
    }
    if state.p_mobj.mo(thing).tics != -1 {
        return true;
    }
    if state
        .info
        .mobjinfo_mut(state.p_mobj.mo(thing).kind)
        .raisestate
        == StateNum::S_NULL
    {
        return true;
    }
    let maxdist: i32 = state.info.mobjinfo_mut(state.p_mobj.mo(thing).kind).radius
        + state.info.mobjinfo[MobjType::MT_VILE as usize].radius;
    if (state.p_mobj.mo(thing).x - state.p_enemy.viletryx).abs() > maxdist
        || (state.p_mobj.mo(thing).y - state.p_enemy.viletryy).abs() > maxdist
    {
        return true;
    }
    state.p_enemy.corpsehit = Some(thing);
    state.p_mobj.mo_mut(thing).momy = 0;
    state.p_mobj.mo_mut(thing).momx = state.p_mobj.mo(thing).momy;
    state.p_mobj.mo_mut(thing).height <<= 2;
    let check: bool = P_CheckPosition(
        state,
        thing,
        state.p_mobj.mo(thing).x,
        state.p_mobj.mo(thing).y,
    );
    state.p_mobj.mo_mut(thing).height >>= 2;
    if !check {
        return true;
    }
    false
}
pub fn A_VileChase(state: &mut GameState, id: MobjId) {
    let actor = id;
    let xl: i32;
    let xh: i32;
    let yl: i32;
    let yh: i32;
    let mut bx: i32;
    let mut by: i32;
    let temp: Option<MobjId>;
    if state.p_mobj.mo(actor).movedir != DirType::DI_NODIR as i32 {
        state.p_enemy.viletryx = state.p_mobj.mo(actor).x
            + state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed as fixed_t
                * xspeed[state.p_mobj.mo(actor).movedir as usize];
        state.p_enemy.viletryy = state.p_mobj.mo(actor).y
            + state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed as fixed_t
                * yspeed[state.p_mobj.mo(actor).movedir as usize];
        xl = (state.p_enemy.viletryx - state.p_setup.bmaporgx - 32 * FRACUNIT * 2) >> MAPBLOCKSHIFT;
        xh = (state.p_enemy.viletryx - state.p_setup.bmaporgx + 32 * FRACUNIT * 2) >> MAPBLOCKSHIFT;
        yl = (state.p_enemy.viletryy - state.p_setup.bmaporgy - 32 * FRACUNIT * 2) >> MAPBLOCKSHIFT;
        yh = (state.p_enemy.viletryy - state.p_setup.bmaporgy + 32 * FRACUNIT * 2) >> MAPBLOCKSHIFT;
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
                    S_StartSound(
                        state,
                        SoundOrigin::Mobj(corpsehit_id),
                        SfxName::sfx_slop as i32,
                    );
                    let info = state.info.mobjinfo_mut(state.p_mobj.mo(corpsehit).kind);
                    let (raisestate, info_flags, spawnhealth) =
                        (info.raisestate, info.flags, info.spawnhealth);
                    P_SetMobjState(state, corpsehit, raisestate);
                    state.p_mobj.mo_mut(corpsehit).height <<= 2;
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
        S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_vilatk as i32);
    }
}
pub fn A_StartFire(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_flamst as i32);
        A_Fire(state, actor);
    }
}
pub fn A_FireCrackle(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_flame as i32);
        A_Fire(state, actor);
    }
}
pub fn A_Fire(state: &mut GameState, id: MobjId) {
    {
        let actor = id;

        let dest: Option<MobjId> = state
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
        let target: MobjId = target_id;
        if !P_CheckSight(state, target, dest.unwrap()) {
            return;
        }
        let an: u32 = state.p_mobj.mo(dest.unwrap()).angle >> ANGLETOFINESHIFT;
        P_UnsetThingPosition(state, actor);
        state.p_mobj.mo_mut(actor).x =
            state.p_mobj.mo(dest.unwrap()).x + FixedMul(24 * FRACUNIT, finecosine[an as usize]);
        state.p_mobj.mo_mut(actor).y =
            state.p_mobj.mo(dest.unwrap()).y + FixedMul(24 * FRACUNIT, finesine[an as usize]);
        state.p_mobj.mo_mut(actor).z = state.p_mobj.mo(dest.unwrap()).z;
        P_SetThingPosition(state, actor);
    }
}
pub fn A_VileTarget(state: &mut GameState, id: MobjId) {
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
        let fog: MobjId = P_SpawnMobj(
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
        S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_barexp as i32);
        P_DamageMobj(state, target, Some(actor), Some(actor), 20);
        state.p_mobj.mo_mut(target).momz = (1000 * FRACUNIT
            / state.info.mobjinfo_mut(state.p_mobj.mo(target).kind).mass)
            as fixed_t;
        let an: i32 = (state.p_mobj.mo(actor).angle >> ANGLETOFINESHIFT) as i32;
        let fire: Option<MobjId> = state
            .p_mobj
            .mo(actor)
            .tracer
            .filter(|&id| state.p_mobj.is_live(id));
        if fire.is_none() {
            return;
        }
        state.p_mobj.mo_mut(fire.unwrap()).x =
            state.p_mobj.mo(target).x - FixedMul(24 * FRACUNIT, finecosine[an as usize]);
        state.p_mobj.mo_mut(fire.unwrap()).y =
            state.p_mobj.mo(target).y - FixedMul(24 * FRACUNIT, finesine[an as usize]);
        P_RadiusAttack(state, fire.unwrap(), Some(actor), 70);
    }
}
pub const FATSPREAD: i32 = ANG90 / 8;
pub fn A_FatRaise(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        A_FaceTarget(state, actor);
        S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_manatk as i32);
    }
}
pub fn A_FatAttack1(state: &mut GameState, id: MobjId) {
    {
        let actor = id;

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
        let target: MobjId = target_id;
        P_SpawnMissile(state, actor, target, MobjType::MT_FATSHOT);
        let mo: MobjId = P_SpawnMissile(state, actor, target, MobjType::MT_FATSHOT);
        state.p_mobj.mo_mut(mo).angle =
            state.p_mobj.mo(mo).angle.wrapping_add(FATSPREAD as angle_t);
        let an: i32 = (state.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
        state.p_mobj.mo_mut(mo).momx = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as fixed_t,
            finecosine[an as usize],
        );
        state.p_mobj.mo_mut(mo).momy = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as fixed_t,
            finesine[an as usize],
        );
    }
}
pub fn A_FatAttack2(state: &mut GameState, id: MobjId) {
    {
        let actor = id;

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
        let target: MobjId = target_id;
        P_SpawnMissile(state, actor, target, MobjType::MT_FATSHOT);
        let mo: MobjId = P_SpawnMissile(state, actor, target, MobjType::MT_FATSHOT);
        state.p_mobj.mo_mut(mo).angle = state
            .p_mobj
            .mo(mo)
            .angle
            .wrapping_sub((FATSPREAD * 2) as angle_t);
        let an: i32 = (state.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
        state.p_mobj.mo_mut(mo).momx = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as fixed_t,
            finecosine[an as usize],
        );
        state.p_mobj.mo_mut(mo).momy = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as fixed_t,
            finesine[an as usize],
        );
    }
}
pub fn A_FatAttack3(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut mo: MobjId;

        let mut an: i32;
        A_FaceTarget(state, actor);
        let target_subst = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        let target_id = P_SubstNullMobj(&mut state.p_mobj, target_subst);
        let target: MobjId = target_id;
        mo = P_SpawnMissile(state, actor, target, MobjType::MT_FATSHOT);
        state.p_mobj.mo_mut(mo).angle = state
            .p_mobj
            .mo(mo)
            .angle
            .wrapping_sub((FATSPREAD / 2) as angle_t);
        an = (state.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
        state.p_mobj.mo_mut(mo).momx = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as fixed_t,
            finecosine[an as usize],
        );
        state.p_mobj.mo_mut(mo).momy = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as fixed_t,
            finesine[an as usize],
        );
        mo = P_SpawnMissile(state, actor, target, MobjType::MT_FATSHOT);
        state.p_mobj.mo_mut(mo).angle = state
            .p_mobj
            .mo(mo)
            .angle
            .wrapping_add((FATSPREAD / 2) as angle_t);
        an = (state.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
        state.p_mobj.mo_mut(mo).momx = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as fixed_t,
            finecosine[an as usize],
        );
        state.p_mobj.mo_mut(mo).momy = FixedMul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as fixed_t,
            finesine[an as usize],
        );
    }
}
pub const SKULLSPEED: i32 = 20 * FRACUNIT;
pub fn A_SkullAttack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;

        let mut dist: i32;
        let dest: MobjId = match state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id))
        {
            Some(dest) => dest,
            None => return,
        };
        state.p_mobj.mo_mut(actor).flags |= MF_SKULLFLY;
        let attacksound = state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).kind)
            .attacksound;
        S_StartSound(state, SoundOrigin::Mobj(actor), attacksound);
        A_FaceTarget(state, actor);
        let an: angle_t = state.p_mobj.mo(actor).angle >> ANGLETOFINESHIFT;
        state.p_mobj.mo_mut(actor).momx = FixedMul(SKULLSPEED, finecosine[an as usize]);
        state.p_mobj.mo_mut(actor).momy = FixedMul(SKULLSPEED, finesine[an as usize]);
        dist = P_AproxDistance(
            state.p_mobj.mo(dest).x - state.p_mobj.mo(actor).x,
            state.p_mobj.mo(dest).y - state.p_mobj.mo(actor).y,
        );
        dist /= SKULLSPEED;
        if dist < 1 {
            dist = 1;
        }
        state.p_mobj.mo_mut(actor).momz = ((state.p_mobj.mo(dest).z
            + (state.p_mobj.mo(dest).height >> 1)
            - state.p_mobj.mo(actor).z)
            / dist) as fixed_t;
    }
}
pub fn A_PainShootSkull(state: &mut GameState, actor: MobjId, angle: angle_t) {
    let mut count: i32 = 0;
    count += P_MobjThinkerIds(state)
        .into_iter()
        .filter(|&m| state.p_mobj.mo(m).kind as u32 == MobjType::MT_SKULL as i32 as u32)
        .count() as i32;
    if count > 20 {
        return;
    }
    let an: angle_t = angle >> ANGLETOFINESHIFT;
    let prestep: i32 = 4 * FRACUNIT
        + 3 * (state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).radius
            + state.info.mobjinfo[MobjType::MT_SKULL as usize].radius)
            / 2;
    let x: fixed_t =
        state.p_mobj.mo(actor).x + FixedMul(prestep as fixed_t, finecosine[an as usize]);
    let y: fixed_t = state.p_mobj.mo(actor).y + FixedMul(prestep as fixed_t, finesine[an as usize]);
    let z: fixed_t = (state.p_mobj.mo(actor).z + 8 * FRACUNIT) as fixed_t;
    let newmobj: MobjId = P_SpawnMobj(state, x, y, z, MobjType::MT_SKULL);
    let (new_x, new_y) = {
        let n = state.p_mobj.mo(newmobj);
        (n.x, n.y)
    };
    if !P_TryMove(state, newmobj, new_x, new_y) {
        P_DamageMobj(state, newmobj, Some(actor), Some(actor), 10000);
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

        let sound: i32 = match state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).kind)
            .deathsound
        {
            0 => return,
            59..=61 => SfxName::sfx_podth1 as i32 + P_Random(&mut state.m_random) % 3,
            62 | 63 => SfxName::sfx_bgdth1 as i32 + P_Random(&mut state.m_random) % 2,
            _ => {
                state
                    .info
                    .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                    .deathsound
            }
        };
        if state.p_mobj.mo(actor).kind as u32 == MobjType::MT_SPIDER as i32 as u32
            || state.p_mobj.mo(actor).kind as u32 == MobjType::MT_CYBORG as i32 as u32
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
        S_StartSound(state, SoundOrigin::Mobj(actor), SfxName::sfx_slop as i32);
    }
}
pub fn A_Pain(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let painsound = state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).kind)
            .painsound;
        if painsound != 0 {
            S_StartSound(state, SoundOrigin::Mobj(actor), painsound);
        }
    }
}
pub fn A_Fall(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        state.p_mobj.mo_mut(actor).flags &= !MF_SOLID;
    }
}
pub fn A_Explode(state: &mut GameState, id: MobjId) {
    let thingy = id;
    let target = state
        .p_mobj
        .mo(thingy)
        .target
        .filter(|&id| state.p_mobj.is_live(id));
    P_RadiusAttack(state, thingy, target, 128);
}
fn CheckBossEnd(state: &mut GameState, motype: MobjType) -> bool {
    if !state.doomstat.gameversion.is_ultimate_or_higher() {
        if state.g_game.gamemap != 8 {
            return false;
        }
        if motype as u32 == MobjType::MT_BRUISER as i32 as u32 && state.g_game.gameepisode != 1 {
            return false;
        }
        true
    } else {
        match state.g_game.gameepisode {
            1 => state.g_game.gamemap == 8 && motype as u32 == MobjType::MT_BRUISER as i32 as u32,
            2 => state.g_game.gamemap == 8 && motype as u32 == MobjType::MT_CYBORG as i32 as u32,
            3 => state.g_game.gamemap == 8 && motype as u32 == MobjType::MT_SPIDER as i32 as u32,
            4 => {
                state.g_game.gamemap == 6 && motype as u32 == MobjType::MT_CYBORG as i32 as u32
                    || state.g_game.gamemap == 8
                        && motype as u32 == MobjType::MT_SPIDER as i32 as u32
            }
            _ => state.g_game.gamemap == 8,
        }
    }
}
pub fn A_BossDeath(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        let mut i: i32;
        if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
            if state.g_game.gamemap != 7 {
                return;
            }
            if state.p_mobj.mo(mo).kind as u32 != MobjType::MT_FATSO as i32 as u32
                && state.p_mobj.mo(mo).kind as u32 != MobjType::MT_BABY as i32 as u32
            {
                return;
            }
        } else if !CheckBossEnd(state, state.p_mobj.mo(mo).kind) {
            return;
        }
        i = 0;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] && state.g_game.players[i as usize].health > 0
            {
                break;
            }
            i += 1;
        }
        if i == MAXPLAYERS {
            return;
        }
        let mo_type = state.p_mobj.mo(mo).kind;
        for mo2 in P_MobjThinkerIds(state) {
            if mo2 != mo
                && state.p_mobj.mo(mo2).kind as u32 == mo_type as u32
                && state.p_mobj.mo(mo2).health > 0
            {
                return;
            }
        }
        if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
            if state.g_game.gamemap == 7 {
                if state.p_mobj.mo(mo).kind as u32 == MobjType::MT_FATSO as i32 as u32 {
                    let junk = state.p_setup.junk_line(666_i16);
                    EV_DoFloor(state, junk, FloorE::lowerFloorToLowest);
                    return;
                }
                if state.p_mobj.mo(mo).kind as u32 == MobjType::MT_BABY as i32 as u32 {
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
        S_StartSound(state, SoundOrigin::Mobj(mo), SfxName::sfx_hoof as i32);
        A_Chase(state, mo);
    }
}
pub fn A_Metal(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        S_StartSound(state, SoundOrigin::Mobj(mo), SfxName::sfx_metal as i32);
        A_Chase(state, mo);
    }
}
pub fn A_BabyMetal(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        S_StartSound(state, SoundOrigin::Mobj(mo), SfxName::sfx_bspwlk as i32);
        A_Chase(state, mo);
    }
}
pub fn A_OpenShotgun2(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        S_StartSound(
            state,
            SoundOrigin::Mobj(state.g_game.players[player.0 as usize].mo.unwrap()),
            SfxName::sfx_dbopn as i32,
        );
    }
}
pub fn A_LoadShotgun2(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        S_StartSound(
            state,
            SoundOrigin::Mobj(state.g_game.players[player.0 as usize].mo.unwrap()),
            SfxName::sfx_dbload as i32,
        );
    }
}
pub fn A_CloseShotgun2(state: &mut GameState, player_id: PlayerId, position: i32) {
    {
        let player = player_id;
        S_StartSound(
            state,
            SoundOrigin::Mobj(state.g_game.players[player.0 as usize].mo.unwrap()),
            SfxName::sfx_dbcls as i32,
        );
        A_ReFire(state, player_id, position);
    }
}
pub fn A_BrainAwake(state: &mut GameState, _id: MobjId) {
    state.p_enemy.numbraintargets = 0;
    state.p_enemy.braintargeton = 0;
    for m in P_MobjThinkerIds(state) {
        if state.p_mobj.mo(m).kind as u32 == MobjType::MT_BOSSTARGET as i32 as u32 {
            let n = state.p_enemy.numbraintargets as usize;
            state.p_enemy.braintargets[n] = Some(m);
            state.p_enemy.numbraintargets += 1;
        }
    }
    S_StartSound(state, SoundOrigin::None, SfxName::sfx_bossit as i32);
}
pub fn A_BrainPain(state: &mut GameState, _id: MobjId) {
    S_StartSound(state, SoundOrigin::None, SfxName::sfx_bospn as i32);
}
pub fn A_BrainScream(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        let mut x: i32;
        let mut y: i32;
        let mut z: i32;
        let mut th: MobjId;
        x = state.p_mobj.mo(mo).x - 196 * FRACUNIT;
        while x < state.p_mobj.mo(mo).x + 320 * FRACUNIT {
            y = state.p_mobj.mo(mo).y - 320 * FRACUNIT;
            z = 128 + P_Random(&mut state.m_random) * 2 * FRACUNIT;
            th = P_SpawnMobj(
                state,
                x as fixed_t,
                y as fixed_t,
                z as fixed_t,
                MobjType::MT_ROCKET,
            );
            state.p_mobj.mo_mut(th).momz = (P_Random(&mut state.m_random) * 512) as fixed_t;
            P_SetMobjState(state, th, StateNum::S_BRAINEXPLODE1);
            state.p_mobj.mo_mut(th).tics -= P_Random(&mut state.m_random) & 7;
            if state.p_mobj.mo(th).tics < 1 {
                state.p_mobj.mo_mut(th).tics = 1;
            }
            x += FRACUNIT * 8;
        }
        S_StartSound(state, SoundOrigin::None, SfxName::sfx_bosdth as i32);
    }
}
pub fn A_BrainExplode(state: &mut GameState, id: MobjId) {
    {
        let mo = id;

        let x: i32 = state.p_mobj.mo(mo).x
            + (P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) * 2048;
        let y: i32 = state.p_mobj.mo(mo).y;
        let z: i32 = 128 + P_Random(&mut state.m_random) * 2 * FRACUNIT;
        let th: MobjId = P_SpawnMobj(
            state,
            x as fixed_t,
            y as fixed_t,
            z as fixed_t,
            MobjType::MT_ROCKET,
        );
        state.p_mobj.mo_mut(th).momz = (P_Random(&mut state.m_random) * 512) as fixed_t;
        P_SetMobjState(state, th, StateNum::S_BRAINEXPLODE1);
        state.p_mobj.mo_mut(th).tics -= P_Random(&mut state.m_random) & 7;
        if state.p_mobj.mo(th).tics < 1 {
            state.p_mobj.mo_mut(th).tics = 1;
        }
    }
}
pub fn A_BrainDie(state: &mut GameState, _id: MobjId) {
    G_ExitLevel(state);
}
pub fn A_BrainSpit(state: &mut GameState, id: MobjId) {
    {
        let mo = id;

        state.p_enemy.easy ^= 1;
        if state.g_game.gameskill <= SkillType::sk_easy && state.p_enemy.easy == 0 {
            return;
        }
        let targ_id = state.p_enemy.braintargets[state.p_enemy.braintargeton as usize].unwrap();
        let targ: MobjId = targ_id;
        state.p_enemy.braintargeton =
            (state.p_enemy.braintargeton + 1) % state.p_enemy.numbraintargets;
        let newmobj: MobjId = P_SpawnMissile(state, mo, targ, MobjType::MT_SPAWNSHOT);
        state.p_mobj.mo_mut(newmobj).target = Some(targ);
        state.p_mobj.mo_mut(newmobj).reactiontime = (state.p_mobj.mo(targ).y
            - state.p_mobj.mo(mo).y)
            / state.p_mobj.mo(newmobj).momy
            / state
                .info
                .state_mut(state.p_mobj.mo(newmobj).state.unwrap())
                .tics;
        S_StartSound(state, SoundOrigin::None, SfxName::sfx_bospit as i32);
    }
}
pub fn A_SpawnSound(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        S_StartSound(state, SoundOrigin::Mobj(mo), SfxName::sfx_boscub as i32);
        A_SpawnFly(state, mo);
    }
}
pub fn A_SpawnFly(state: &mut GameState, id: MobjId) {
    {
        let mo = id;

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
        let targ: MobjId = targ_id;
        let fog: MobjId = P_SpawnMobj(
            state,
            state.p_mobj.mo(targ).x,
            state.p_mobj.mo(targ).y,
            state.p_mobj.mo(targ).z,
            MobjType::MT_SPAWNFIRE,
        );
        S_StartSound(state, SoundOrigin::Mobj(fog), SfxName::sfx_telept as i32);
        let r: i32 = P_Random(&mut state.m_random);
        let kind: MobjType = if r < 50 {
            MobjType::MT_TROOP
        } else if r < 90 {
            MobjType::MT_SERGEANT
        } else if r < 120 {
            MobjType::MT_SHADOWS
        } else if r < 130 {
            MobjType::MT_PAIN
        } else if r < 160 {
            MobjType::MT_HEAD
        } else if r < 162 {
            MobjType::MT_VILE
        } else if r < 172 {
            MobjType::MT_UNDEAD
        } else if r < 192 {
            MobjType::MT_BABY
        } else if r < 222 {
            MobjType::MT_FATSO
        } else if r < 246 {
            MobjType::MT_KNIGHT
        } else {
            MobjType::MT_BRUISER
        };
        let newmobj: MobjId = P_SpawnMobj(
            state,
            state.p_mobj.mo(targ).x,
            state.p_mobj.mo(targ).y,
            state.p_mobj.mo(targ).z,
            kind,
        );
        if P_LookForPlayers(state, newmobj, true) {
            let seestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(newmobj).kind)
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
        let mut sound: i32 = SfxName::sfx_pldeth as i32;
        if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32
            && state.p_mobj.mo(mo).health < -50
        {
            sound = SfxName::sfx_pdiehi as i32;
        }
        S_StartSound(state, SoundOrigin::Mobj(mo), sound);
    }
}
