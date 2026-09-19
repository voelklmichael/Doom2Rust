use crate::d_mode::GameMode;
use crate::d_mode::SkillType;

use crate::d_player::PlayerId;
use crate::g_game::exit_level;
use crate::i_system::error;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_random::p_random;
use crate::p_doors::do_door;
use crate::p_doors::VldoorE;
use crate::p_floor::do_floor;
use crate::p_floor::FloorE;
use crate::p_inter::damage_mobj;
use crate::p_map::aim_line_attack;
use crate::p_map::check_position;
use crate::p_map::line_attack;
use crate::p_map::p_radius_attack;
use crate::p_map::teleport_move;
use crate::p_map::try_move;
use crate::p_maputl::aprox_distance;
use crate::p_maputl::block_things_iterator;
use crate::p_maputl::line_opening;
use crate::p_maputl::set_thing_position;
use crate::p_maputl::unset_thing_position;
use crate::p_mobj::remove_mobj;
use crate::p_mobj::set_mobj_state;
use crate::p_mobj::spawn_missile;
use crate::p_mobj::MobjId;
use crate::p_mobj::MobjType;

use crate::p_mobj::spawn_mobj;

use crate::p_mobj::spawn_puff;
use crate::p_mobj::subst_null_mobj;

use crate::p_mobj::{
    MF_AMBUSH, MF_CORPSE, MF_FLOAT, MF_INFLOAT, MF_JUSTATTACKED, MF_JUSTHIT, MF_SHADOW,
    MF_SHOOTABLE, MF_SKULLFLY, MF_SOLID,
};
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_sight::check_sight;
use crate::p_switch::use_special_line;
use crate::p_tick::mobj_thinker_ids;

use crate::r_main::point_to_angle2;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::tables::Angle;
use crate::tables::FINECOSINE;
use crate::tables::FINESINE;

use crate::doomdef::MAXPLAYERS;
use crate::game_state::GameState;
use crate::m_fixed::FRACUNIT;
use crate::p_maputl::MAPBLOCKSHIFT;
use crate::p_mobj::StateNum;
use crate::p_mobj::FLOATSPEED;
use crate::p_pspr::re_fire;
use crate::p_spec::ML_TWOSIDED;
use crate::tables::ANG180;
use crate::tables::ANG270;
use crate::tables::ANG90;
use crate::tables::ANGLETOFINESHIFT;

pub struct PEnemyState {
    pub soundtarget: Option<MobjId>,
    pub corpsehit: Option<MobjId>,
    pub vileobj: Option<MobjId>,
    pub viletryx: Fixed,
    pub viletryy: Fixed,
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
    East = 0,
    Northeast = 1,
    North = 2,
    Northwest = 3,
    West = 4,
    Southwest = 5,
    South = 6,
    Southeast = 7,
    Nodir = 8,
}
fn dirtype_from_movedir(movedir: i32) -> DirType {
    match movedir {
        0 => DirType::East,
        1 => DirType::Northeast,
        2 => DirType::North,
        3 => DirType::Northwest,
        4 => DirType::West,
        5 => DirType::Southwest,
        6 => DirType::South,
        7 => DirType::Southeast,
        8 => DirType::Nodir,
        n => panic!("P_NewChaseDir: invalid movedir {n}"),
    }
}
pub const ML_SOUNDBLOCK: i32 = 64;
pub const MELEERANGE: i32 = 64 * FRACUNIT;
pub const MISSILERANGE: i32 = 32 * 64 * FRACUNIT;
pub static OPPOSITE: [DirType; 9] = [
    DirType::West,
    DirType::Southwest,
    DirType::South,
    DirType::Southeast,
    DirType::East,
    DirType::Northeast,
    DirType::North,
    DirType::Northwest,
    DirType::Nodir,
];
pub static DIAGS: [DirType; 4] = [
    DirType::Northwest,
    DirType::Northeast,
    DirType::Southwest,
    DirType::Southeast,
];
pub fn recursive_sound(state: &mut GameState, sec: SectorId, soundblocks: i32) {
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
            line_opening(state, check);
            if state.p_maputl.openrange > 0 {
                let other = if state.p_setup.sides[checkv.sidenum[0] as usize].sector == sec {
                    state.p_setup.sides[checkv.sidenum[1] as usize].sector
                } else {
                    state.p_setup.sides[checkv.sidenum[0] as usize].sector
                };
                if checkv.flags as i32 & ML_SOUNDBLOCK != 0 {
                    if soundblocks == 0 {
                        recursive_sound(state, other, 1);
                    }
                } else {
                    recursive_sound(state, other, soundblocks);
                }
            }
        }
    }
}
pub fn noise_alert(state: &mut GameState, target: MobjId, emmiter: MobjId) {
    state.p_enemy.soundtarget = Some(target);
    state.r_main.validcount += 1;
    let emmiter_subsector = state.p_mobj.mo(emmiter).subsector;
    let sec = state.p_setup.subsectors[emmiter_subsector.0 as usize].sector;
    recursive_sound(state, sec, 0);
}
pub fn check_melee_range(state: &mut GameState, actor: MobjId) -> bool {
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
    let dist = aprox_distance(pl_x - actor_x, pl_y - actor_y);
    if dist >= MELEERANGE - 20 * FRACUNIT + state.info.mobjinfo_mut(pl_type).radius {
        return false;
    }
    if !check_sight(state, actor, pl) {
        return false;
    }
    true
}
pub fn check_missile_range(state: &mut GameState, actor: MobjId) -> bool {
    let Some(target) = state
        .p_mobj
        .mo(actor)
        .target
        .filter(|&id| state.p_mobj.is_live(id))
    else {
        return false;
    };
    if !check_sight(state, actor, target) {
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
    let mut dist: Fixed =
        (aprox_distance(actor_x - target_x, actor_y - target_y) - 64 * FRACUNIT) as Fixed;
    if state.info.mobjinfo_mut(actor_type).meleestate == StateNum::Null {
        dist -= 128 * FRACUNIT;
    }
    dist >>= 16;
    if actor_type as u32 == MobjType::Vile as i32 as u32 && dist > 14 * 64 {
        return false;
    }
    if actor_type as u32 == MobjType::Undead as i32 as u32 {
        if dist < 196 {
            return false;
        }
        dist >>= 1;
    }
    if actor_type as u32 == MobjType::Cyborg as i32 as u32
        || actor_type as u32 == MobjType::Spider as i32 as u32
        || actor_type as u32 == MobjType::Skull as i32 as u32
    {
        dist >>= 1;
    }
    if dist > 200 {
        dist = 200;
    }
    if actor_type as u32 == MobjType::Cyborg as i32 as u32 && dist > 160 {
        dist = 160;
    }
    if p_random(&mut state.m_random) < dist {
        return false;
    }
    true
}
pub static XSPEED: [Fixed; 8] = [FRACUNIT, 47000, 0, -47000, -FRACUNIT, -47000, 0, 47000];
pub static YSPEED: [Fixed; 8] = [0, 47000, FRACUNIT, 47000, 0, -47000, -FRACUNIT, -47000];
pub fn p_move(state: &mut GameState, actor: MobjId) -> bool {
    let mut ld: LineId;

    let mut good: bool;
    if state.p_mobj.mo(actor).movedir == DirType::Nodir as i32 {
        return false;
    }
    if state.p_mobj.mo(actor).movedir as u32 >= 8 {
        error("Weird actor->movedir!");
    }
    let tryx: Fixed = state.p_mobj.mo(actor).x
        + state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed as Fixed
            * XSPEED[state.p_mobj.mo(actor).movedir as usize];
    let tryy: Fixed = state.p_mobj.mo(actor).y
        + state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed as Fixed
            * YSPEED[state.p_mobj.mo(actor).movedir as usize];
    let try_ok: bool = try_move(state, actor, tryx, tryy);
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
        state.p_mobj.mo_mut(actor).movedir = DirType::Nodir as i32;
        good = false;
        loop {
            let fresh0 = state.p_map.numspechit;
            state.p_map.numspechit -= 1;
            if fresh0 == 0 {
                break;
            }
            ld = state.p_map.spechit[state.p_map.numspechit as usize];
            if use_special_line(state, actor, ld, 0) {
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
pub fn try_walk(state: &mut GameState, actor: MobjId) -> bool {
    if !p_move(state, actor) {
        return false;
    }
    state.p_mobj.mo_mut(actor).movecount = p_random(&mut state.m_random) & 15;
    true
}
pub fn new_chase_dir(state: &mut GameState, actor: MobjId) {
    let mut d: [DirType; 3] = [DirType::East; 3];
    let mut tdir: i32;

    let target = match state
        .p_mobj
        .mo(actor)
        .target
        .filter(|&id| state.p_mobj.is_live(id))
    {
        Some(target) => target,
        None => {
            error("P_NewChaseDir: called with no target");
        }
    };
    let olddir: DirType = dirtype_from_movedir(state.p_mobj.mo(actor).movedir);
    let turnaround: DirType = OPPOSITE[olddir as usize];
    let deltax: Fixed = state.p_mobj.mo(target).x - state.p_mobj.mo(actor).x;
    let deltay: Fixed = state.p_mobj.mo(target).y - state.p_mobj.mo(actor).y;
    if deltax > 10 * FRACUNIT {
        d[1] = DirType::East;
    } else if deltax < -10 * FRACUNIT {
        d[1] = DirType::West;
    } else {
        d[1] = DirType::Nodir;
    }
    if deltay < -10 * FRACUNIT {
        d[2] = DirType::South;
    } else if deltay > 10 * FRACUNIT {
        d[2] = DirType::North;
    } else {
        d[2] = DirType::Nodir;
    }
    if d[1] != DirType::Nodir && d[2] != DirType::Nodir {
        state.p_mobj.mo_mut(actor).movedir =
            DIAGS[((((deltay < 0) as i32) << 1) + (deltax > 0) as i32) as usize] as i32;
        if state.p_mobj.mo(actor).movedir != turnaround as i32 && try_walk(state, actor) {
            return;
        }
    }
    if p_random(&mut state.m_random) > 200 || deltay.abs() > deltax.abs() {
        d.swap(1, 2);
    }
    if d[1] == turnaround {
        d[1] = DirType::Nodir;
    }
    if d[2] == turnaround {
        d[2] = DirType::Nodir;
    }
    if d[1] != DirType::Nodir {
        state.p_mobj.mo_mut(actor).movedir = d[1] as i32;
        if try_walk(state, actor) {
            return;
        }
    }
    if d[2] != DirType::Nodir {
        state.p_mobj.mo_mut(actor).movedir = d[2] as i32;
        if try_walk(state, actor) {
            return;
        }
    }
    if olddir != DirType::Nodir {
        state.p_mobj.mo_mut(actor).movedir = olddir as i32;
        if try_walk(state, actor) {
            return;
        }
    }
    if p_random(&mut state.m_random) & 1 != 0 {
        for tdir in DirType::East as i32..=DirType::Southeast as i32 {
            if tdir != turnaround as i32 {
                state.p_mobj.mo_mut(actor).movedir = tdir;
                if try_walk(state, actor) {
                    return;
                }
            }
        }
    } else {
        tdir = DirType::Southeast as i32;
        while tdir != DirType::East as i32 - 1 {
            if tdir != turnaround as i32 {
                state.p_mobj.mo_mut(actor).movedir = tdir;
                if try_walk(state, actor) {
                    return;
                }
            }
            tdir -= 1;
        }
    }
    if turnaround != DirType::Nodir {
        state.p_mobj.mo_mut(actor).movedir = turnaround as i32;
        if try_walk(state, actor) {
            return;
        }
    }
    state.p_mobj.mo_mut(actor).movedir = DirType::Nodir as i32;
}
pub fn look_for_players(state: &mut GameState, actor: MobjId, allaround: bool) -> bool {
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
                if check_sight(state, actor, player_mo) {
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
                        let an = point_to_angle2(state, actor_x, actor_y, pmo_x, pmo_y)
                            .wrapping_sub(actor_angle);
                        if an > ANG90 as Angle && an < ANG270 {
                            let dist = aprox_distance(pmo_x - actor_x, pmo_y - actor_y);
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
pub fn keen_die(state: &mut GameState, id: MobjId) {
    let mo = id;
    fall(state, mo);
    let mo_type = state.p_mobj.mo(mo).kind;
    for mo2 in mobj_thinker_ids(state) {
        if mo2 != mo
            && state.p_mobj.mo(mo2).kind as u32 == mo_type as u32
            && state.p_mobj.mo(mo2).health > 0
        {
            return;
        }
    }
    let junk = state.p_setup.junk_line(666_i16);
    do_door(state, junk, VldoorE::Open);
}
pub fn look(state: &mut GameState, id: MobjId) {
    {
        let actor = id;

        state.p_mobj.mo_mut(actor).threshold = 0;
        let targ: Option<MobjId> = state
            .p_setup
            .sector_mut(
                state.p_setup.subsectors[state.p_mobj.mo(actor).subsector.0 as usize].sector,
            )
            .soundtarget
            .filter(|&id| state.p_mobj.is_live(id));
        // Whether the actor already sees its target (and so skips looking around).
        let sees_target =
            if let Some(targ) = targ.filter(|&t| state.p_mobj.mo(t).flags & MF_SHOOTABLE != 0) {
                state.p_mobj.mo_mut(actor).target = Some(targ);
                state.p_mobj.mo(actor).flags & MF_AMBUSH == 0 || check_sight(state, actor, targ)
            } else {
                false
            };
        if !sees_target && !look_for_players(state, actor, false) {
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
                36..=38 => SfxName::Posit1 as i32 + p_random(&mut state.m_random) % 3,
                39 | 40 => SfxName::Bgsit1 as i32 + p_random(&mut state.m_random) % 2,
                _ => {
                    state
                        .info
                        .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                        .seesound
                }
            };
            if state.p_mobj.mo(actor).kind as u32 == MobjType::Spider as i32 as u32
                || state.p_mobj.mo(actor).kind as u32 == MobjType::Cyborg as i32 as u32
            {
                s_start_sound(state, SoundOrigin::None, sound);
            } else {
                s_start_sound(state, SoundOrigin::Mobj(actor), sound);
            }
        }
        let seestate = state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).kind)
            .seestate;
        set_mobj_state(state, actor, seestate);
    }
}
pub fn chase(state: &mut GameState, id: MobjId) {
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
            state.p_mobj.mo_mut(actor).angle &= (7 << 29) as Angle;
            delta = state
                .p_mobj
                .mo(actor)
                .angle
                .wrapping_sub((state.p_mobj.mo(actor).movedir << 29) as Angle)
                as i32;
            if delta > 0 {
                state.p_mobj.mo_mut(actor).angle = state
                    .p_mobj
                    .mo(actor)
                    .angle
                    .wrapping_sub((ANG90 / 2) as Angle);
            } else if delta < 0 {
                state.p_mobj.mo_mut(actor).angle = state
                    .p_mobj
                    .mo(actor)
                    .angle
                    .wrapping_add((ANG90 / 2) as Angle);
            }
        }
        if target.is_none() || state.p_mobj.mo(target.unwrap()).flags & MF_SHOOTABLE == 0 {
            if look_for_players(state, actor, true) {
                return;
            }
            let spawnstate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                .spawnstate;
            set_mobj_state(state, actor, spawnstate);
            return;
        }
        if state.p_mobj.mo(actor).flags & MF_JUSTATTACKED != 0 {
            state.p_mobj.mo_mut(actor).flags &= !MF_JUSTATTACKED;
            if state.g_game.gameskill != SkillType::Nightmare && !state.d_main.fastparm {
                new_chase_dir(state, actor);
            }
            return;
        }
        let actor_info = state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind);
        if actor_info.meleestate != StateNum::Null && check_melee_range(state, actor) {
            let attacksound = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                .attacksound;
            if attacksound != 0 {
                s_start_sound(state, SoundOrigin::Mobj(actor), attacksound);
            }
            let meleestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                .meleestate;
            set_mobj_state(state, actor, meleestate);
            return;
        }
        if state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).kind)
            .missilestate
            != StateNum::Null
            && !(state.g_game.gameskill < SkillType::Nightmare
                && !state.d_main.fastparm
                && state.p_mobj.mo(actor).movecount != 0)
            && check_missile_range(state, actor)
        {
            let missilestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                .missilestate;
            set_mobj_state(state, actor, missilestate);
            state.p_mobj.mo_mut(actor).flags |= MF_JUSTATTACKED;
            return;
        }
        if state.g_game.netgame
            && state.p_mobj.mo(actor).threshold == 0
            && !check_sight(state, actor, state.p_mobj.mo(target.unwrap()).id)
            && look_for_players(state, actor, true)
        {
            return;
        }
        state.p_mobj.mo_mut(actor).movecount -= 1;
        if state.p_mobj.mo(actor).movecount < 0 || !p_move(state, actor) {
            new_chase_dir(state, actor);
        }
        let activesound = state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).kind)
            .activesound;
        if activesound != 0 && p_random(&mut state.m_random) < 3 {
            s_start_sound(state, SoundOrigin::Mobj(actor), activesound);
        }
    }
}
pub fn face_target(state: &mut GameState, id: MobjId) {
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
        state.p_mobj.mo_mut(actor).angle = point_to_angle2(
            state,
            state.p_mobj.mo(actor).x,
            state.p_mobj.mo(actor).y,
            state.p_mobj.mo(target).x,
            state.p_mobj.mo(target).y,
        );
        if state.p_mobj.mo(target).flags & MF_SHADOW != 0 {
            state.p_mobj.mo_mut(actor).angle = state.p_mobj.mo(actor).angle.wrapping_add(
                ((p_random(&mut state.m_random) - p_random(&mut state.m_random)) << 21) as Angle,
            );
        }
    }
}
pub fn pos_attack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut angle: i32;

        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        face_target(state, actor);
        angle = state.p_mobj.mo(actor).angle as i32;
        let slope: i32 = aim_line_attack(state, Some(actor), angle as Angle, MISSILERANGE);
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Pistol as i32);
        angle += (p_random(&mut state.m_random) - p_random(&mut state.m_random)) << 20;
        let damage: i32 = (p_random(&mut state.m_random) % 5 + 1) * 3;
        line_attack(
            state,
            actor,
            angle as Angle,
            MISSILERANGE,
            slope as Fixed,
            damage,
        );
    }
}
pub fn spos_attack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut angle: i32;

        let mut damage: i32;

        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Shotgn as i32);
        face_target(state, actor);
        let bangle: i32 = state.p_mobj.mo(actor).angle as i32;
        let slope: i32 = aim_line_attack(state, Some(actor), bangle as Angle, MISSILERANGE);
        for _ in 0..3 {
            angle =
                bangle + ((p_random(&mut state.m_random) - p_random(&mut state.m_random)) << 20);
            damage = (p_random(&mut state.m_random) % 5 + 1) * 3;
            line_attack(
                state,
                actor,
                angle as Angle,
                MISSILERANGE,
                slope as Fixed,
                damage,
            );
        }
    }
}
pub fn cpos_attack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;

        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Shotgn as i32);
        face_target(state, actor);
        let bangle: i32 = state.p_mobj.mo(actor).angle as i32;
        let slope: i32 = aim_line_attack(state, Some(actor), bangle as Angle, MISSILERANGE);
        let angle: i32 =
            bangle + ((p_random(&mut state.m_random) - p_random(&mut state.m_random)) << 20);
        let damage: i32 = (p_random(&mut state.m_random) % 5 + 1) * 3;
        line_attack(
            state,
            actor,
            angle as Angle,
            MISSILERANGE,
            slope as Fixed,
            damage,
        );
    }
}
pub fn cpos_refire(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        face_target(state, actor);
        if p_random(&mut state.m_random) < 40 {
            return;
        }
        let target = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        if target.is_none()
            || state.p_mobj.mo(target.unwrap()).health <= 0
            || !check_sight(state, actor, state.p_mobj.mo(target.unwrap()).id)
        {
            let seestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                .seestate;
            set_mobj_state(state, actor, seestate);
        }
    }
}
pub fn spid_refire(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        face_target(state, actor);
        if p_random(&mut state.m_random) < 10 {
            return;
        }
        let target = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        if target.is_none()
            || state.p_mobj.mo(target.unwrap()).health <= 0
            || !check_sight(state, actor, state.p_mobj.mo(target.unwrap()).id)
        {
            let seestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                .seestate;
            set_mobj_state(state, actor, seestate);
        }
    }
}
pub fn bspi_attack(state: &mut GameState, id: MobjId) {
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
        face_target(state, actor);
        spawn_missile(state, actor, target, MobjType::Arachplaz);
    }
}
pub fn troop_attack(state: &mut GameState, id: MobjId) {
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
        face_target(state, actor);
        if check_melee_range(state, actor) {
            s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Claw as i32);
            damage = (p_random(&mut state.m_random) % 8 + 1) * 3;
            damage_mobj(state, target, Some(actor), Some(actor), damage);
            return;
        }
        spawn_missile(state, actor, target, MobjType::Troopshot);
    }
}
pub fn sarg_attack(state: &mut GameState, id: MobjId) {
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
        face_target(state, actor);
        if check_melee_range(state, actor) {
            damage = (p_random(&mut state.m_random) % 10 + 1) * 4;
            damage_mobj(state, target, Some(actor), Some(actor), damage);
        }
    }
}
pub fn head_attack(state: &mut GameState, id: MobjId) {
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
        face_target(state, actor);
        if check_melee_range(state, actor) {
            damage = (p_random(&mut state.m_random) % 6 + 1) * 10;
            damage_mobj(state, target, Some(actor), Some(actor), damage);
            return;
        }
        spawn_missile(state, actor, target, MobjType::Headshot);
    }
}
pub fn cyber_attack(state: &mut GameState, id: MobjId) {
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
        face_target(state, actor);
        spawn_missile(state, actor, target, MobjType::Rocket);
    }
}
pub fn bruis_attack(state: &mut GameState, id: MobjId) {
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
        if check_melee_range(state, actor) {
            s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Claw as i32);
            damage = (p_random(&mut state.m_random) % 8 + 1) * 10;
            damage_mobj(state, target, Some(actor), Some(actor), damage);
            return;
        }
        spawn_missile(state, actor, target, MobjType::Bruisershot);
    }
}
pub fn skel_missile(state: &mut GameState, id: MobjId) {
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
        face_target(state, actor);
        state.p_mobj.mo_mut(actor).z += 16 * FRACUNIT;
        let mo: MobjId = spawn_missile(state, actor, target, MobjType::Tracer);
        state.p_mobj.mo_mut(actor).z -= 16 * FRACUNIT;
        state.p_mobj.mo_mut(mo).x += state.p_mobj.mo(mo).momx;
        state.p_mobj.mo_mut(mo).y += state.p_mobj.mo(mo).momy;
        state.p_mobj.mo_mut(mo).tracer = state.p_mobj.mo(actor).target;
    }
}
pub static TRACEANGLE: i32 = 0xc000000;
pub fn a_tracer(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut exact: Angle;
        let mut dist: Fixed;

        if state.d_loop.gametic & 3 != 0 {
            return;
        }
        spawn_puff(
            state,
            state.p_mobj.mo(actor).x,
            state.p_mobj.mo(actor).y,
            state.p_mobj.mo(actor).z,
        );
        let th: MobjId = spawn_mobj(
            state,
            state.p_mobj.mo(actor).x - state.p_mobj.mo(actor).momx,
            state.p_mobj.mo(actor).y - state.p_mobj.mo(actor).momy,
            state.p_mobj.mo(actor).z,
            MobjType::Smoke,
        );
        state.p_mobj.mo_mut(th).momz = FRACUNIT as Fixed;
        state.p_mobj.mo_mut(th).tics -= p_random(&mut state.m_random) & 3;
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
        exact = point_to_angle2(
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
                    .wrapping_sub(TRACEANGLE as Angle);
                if exact.wrapping_sub(state.p_mobj.mo(actor).angle) < 0x80000000 {
                    state.p_mobj.mo_mut(actor).angle = exact;
                }
            } else {
                state.p_mobj.mo_mut(actor).angle = state
                    .p_mobj
                    .mo(actor)
                    .angle
                    .wrapping_add(TRACEANGLE as Angle);
                if exact.wrapping_sub(state.p_mobj.mo(actor).angle) > 0x80000000 {
                    state.p_mobj.mo_mut(actor).angle = exact;
                }
            }
        }
        exact = state.p_mobj.mo(actor).angle >> ANGLETOFINESHIFT;
        state.p_mobj.mo_mut(actor).momx = fixed_mul(
            state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed as Fixed,
            FINECOSINE[exact as usize],
        );
        state.p_mobj.mo_mut(actor).momy = fixed_mul(
            state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed as Fixed,
            FINESINE[exact as usize],
        );
        dist = aprox_distance(
            state.p_mobj.mo(dest.unwrap()).x - state.p_mobj.mo(actor).x,
            state.p_mobj.mo(dest.unwrap()).y - state.p_mobj.mo(actor).y,
        );
        dist = (dist / state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed) as Fixed;
        if dist < 1 {
            dist = 1;
        }
        let slope: Fixed =
            (state.p_mobj.mo(dest.unwrap()).z + 40 * FRACUNIT - state.p_mobj.mo(actor).z) / dist;
        if slope < state.p_mobj.mo(actor).momz {
            state.p_mobj.mo_mut(actor).momz -= FRACUNIT / 8;
        } else {
            state.p_mobj.mo_mut(actor).momz += FRACUNIT / 8;
        };
    }
}
pub fn skel_whoosh(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        face_target(state, actor);
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Skeswg as i32);
    }
}
pub fn skel_fist(state: &mut GameState, id: MobjId) {
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
        face_target(state, actor);
        if check_melee_range(state, actor) {
            damage = (p_random(&mut state.m_random) % 10 + 1) * 6;
            s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Skepch as i32);
            damage_mobj(state, target, Some(actor), Some(actor), damage);
        }
    }
}
pub fn vile_check(state: &mut GameState, thing_id: MobjId) -> bool {
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
        == StateNum::Null
    {
        return true;
    }
    let maxdist: i32 = state.info.mobjinfo_mut(state.p_mobj.mo(thing).kind).radius
        + state.info.mobjinfo[MobjType::Vile as usize].radius;
    if (state.p_mobj.mo(thing).x - state.p_enemy.viletryx).abs() > maxdist
        || (state.p_mobj.mo(thing).y - state.p_enemy.viletryy).abs() > maxdist
    {
        return true;
    }
    state.p_enemy.corpsehit = Some(thing);
    state.p_mobj.mo_mut(thing).momy = 0;
    state.p_mobj.mo_mut(thing).momx = state.p_mobj.mo(thing).momy;
    state.p_mobj.mo_mut(thing).height <<= 2;
    let check: bool = check_position(
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
pub fn vile_chase(state: &mut GameState, id: MobjId) {
    let actor = id;
    let xl: i32;
    let xh: i32;
    let yl: i32;
    let yh: i32;
    let temp: Option<MobjId>;
    if state.p_mobj.mo(actor).movedir != DirType::Nodir as i32 {
        state.p_enemy.viletryx = state.p_mobj.mo(actor).x
            + state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed as Fixed
                * XSPEED[state.p_mobj.mo(actor).movedir as usize];
        state.p_enemy.viletryy = state.p_mobj.mo(actor).y
            + state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).speed as Fixed
                * YSPEED[state.p_mobj.mo(actor).movedir as usize];
        xl = (state.p_enemy.viletryx - state.p_setup.bmaporgx - 32 * FRACUNIT * 2) >> MAPBLOCKSHIFT;
        xh = (state.p_enemy.viletryx - state.p_setup.bmaporgx + 32 * FRACUNIT * 2) >> MAPBLOCKSHIFT;
        yl = (state.p_enemy.viletryy - state.p_setup.bmaporgy - 32 * FRACUNIT * 2) >> MAPBLOCKSHIFT;
        yh = (state.p_enemy.viletryy - state.p_setup.bmaporgy + 32 * FRACUNIT * 2) >> MAPBLOCKSHIFT;
        state.p_enemy.vileobj = Some(actor);
        for bx in xl..=xh {
            for by in yl..=yh {
                if !block_things_iterator(state, bx, by, vile_check) {
                    let corpsehit_id = state.p_enemy.corpsehit.unwrap();
                    let corpsehit = corpsehit_id;
                    temp = state.p_mobj.mo(actor).target;
                    state.p_mobj.mo_mut(actor).target = Some(corpsehit_id);
                    face_target(state, actor);
                    state.p_mobj.mo_mut(actor).target = temp;
                    set_mobj_state(state, actor, StateNum::VileHeal1);
                    s_start_sound(state, SoundOrigin::Mobj(corpsehit_id), SfxName::Slop as i32);
                    let info = state.info.mobjinfo_mut(state.p_mobj.mo(corpsehit).kind);
                    let (raisestate, info_flags, spawnhealth) =
                        (info.raisestate, info.flags, info.spawnhealth);
                    set_mobj_state(state, corpsehit, raisestate);
                    state.p_mobj.mo_mut(corpsehit).height <<= 2;
                    state.p_mobj.mo_mut(corpsehit).flags = info_flags;
                    state.p_mobj.mo_mut(corpsehit).health = spawnhealth;
                    state.p_mobj.mo_mut(corpsehit).target = None;
                    return;
                }
            }
        }
    }
    chase(state, actor);
}
pub fn vile_start(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Vilatk as i32);
    }
}
pub fn start_fire(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Flamst as i32);
        a_fire(state, actor);
    }
}
pub fn fire_crackle(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Flame as i32);
        a_fire(state, actor);
    }
}
pub fn a_fire(state: &mut GameState, id: MobjId) {
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
        let target_id = subst_null_mobj(&mut state.p_mobj, target_subst);
        let target: MobjId = target_id;
        if !check_sight(state, target, dest.unwrap()) {
            return;
        }
        let an: u32 = state.p_mobj.mo(dest.unwrap()).angle >> ANGLETOFINESHIFT;
        unset_thing_position(state, actor);
        state.p_mobj.mo_mut(actor).x =
            state.p_mobj.mo(dest.unwrap()).x + fixed_mul(24 * FRACUNIT, FINECOSINE[an as usize]);
        state.p_mobj.mo_mut(actor).y =
            state.p_mobj.mo(dest.unwrap()).y + fixed_mul(24 * FRACUNIT, FINESINE[an as usize]);
        state.p_mobj.mo_mut(actor).z = state.p_mobj.mo(dest.unwrap()).z;
        set_thing_position(state, actor);
    }
}
pub fn vile_target(state: &mut GameState, id: MobjId) {
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
        face_target(state, actor);
        let fog: MobjId = spawn_mobj(
            state,
            state.p_mobj.mo(target).x,
            state.p_mobj.mo(target).x,
            state.p_mobj.mo(target).z,
            MobjType::Fire,
        );
        state.p_mobj.mo_mut(actor).tracer = Some(fog);
        state.p_mobj.mo_mut(fog).target = Some(actor);
        state.p_mobj.mo_mut(fog).tracer = state.p_mobj.mo(actor).target;
        a_fire(state, fog);
    }
}
pub fn vile_attack(state: &mut GameState, id: MobjId) {
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
        face_target(state, actor);
        if !check_sight(state, actor, target) {
            return;
        }
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Barexp as i32);
        damage_mobj(state, target, Some(actor), Some(actor), 20);
        state.p_mobj.mo_mut(target).momz =
            (1000 * FRACUNIT / state.info.mobjinfo_mut(state.p_mobj.mo(target).kind).mass) as Fixed;
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
            state.p_mobj.mo(target).x - fixed_mul(24 * FRACUNIT, FINECOSINE[an as usize]);
        state.p_mobj.mo_mut(fire.unwrap()).y =
            state.p_mobj.mo(target).y - fixed_mul(24 * FRACUNIT, FINESINE[an as usize]);
        p_radius_attack(state, fire.unwrap(), Some(actor), 70);
    }
}
pub const FATSPREAD: i32 = ANG90 / 8;
pub fn fat_raise(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        face_target(state, actor);
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Manatk as i32);
    }
}
pub fn fat_attack1(state: &mut GameState, id: MobjId) {
    {
        let actor = id;

        face_target(state, actor);
        state.p_mobj.mo_mut(actor).angle = state
            .p_mobj
            .mo(actor)
            .angle
            .wrapping_add(FATSPREAD as Angle);
        let target_subst = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        let target_id = subst_null_mobj(&mut state.p_mobj, target_subst);
        let target: MobjId = target_id;
        spawn_missile(state, actor, target, MobjType::Fatshot);
        let mo: MobjId = spawn_missile(state, actor, target, MobjType::Fatshot);
        state.p_mobj.mo_mut(mo).angle = state.p_mobj.mo(mo).angle.wrapping_add(FATSPREAD as Angle);
        let an: i32 = (state.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
        state.p_mobj.mo_mut(mo).momx = fixed_mul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as Fixed,
            FINECOSINE[an as usize],
        );
        state.p_mobj.mo_mut(mo).momy = fixed_mul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as Fixed,
            FINESINE[an as usize],
        );
    }
}
pub fn fat_attack2(state: &mut GameState, id: MobjId) {
    {
        let actor = id;

        face_target(state, actor);
        state.p_mobj.mo_mut(actor).angle = state
            .p_mobj
            .mo(actor)
            .angle
            .wrapping_sub(FATSPREAD as Angle);
        let target_subst = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        let target_id = subst_null_mobj(&mut state.p_mobj, target_subst);
        let target: MobjId = target_id;
        spawn_missile(state, actor, target, MobjType::Fatshot);
        let mo: MobjId = spawn_missile(state, actor, target, MobjType::Fatshot);
        state.p_mobj.mo_mut(mo).angle = state
            .p_mobj
            .mo(mo)
            .angle
            .wrapping_sub((FATSPREAD * 2) as Angle);
        let an: i32 = (state.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
        state.p_mobj.mo_mut(mo).momx = fixed_mul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as Fixed,
            FINECOSINE[an as usize],
        );
        state.p_mobj.mo_mut(mo).momy = fixed_mul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as Fixed,
            FINESINE[an as usize],
        );
    }
}
pub fn fat_attack3(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let mut mo: MobjId;

        let mut an: i32;
        face_target(state, actor);
        let target_subst = state
            .p_mobj
            .mo(actor)
            .target
            .filter(|&id| state.p_mobj.is_live(id));
        let target_id = subst_null_mobj(&mut state.p_mobj, target_subst);
        let target: MobjId = target_id;
        mo = spawn_missile(state, actor, target, MobjType::Fatshot);
        state.p_mobj.mo_mut(mo).angle = state
            .p_mobj
            .mo(mo)
            .angle
            .wrapping_sub((FATSPREAD / 2) as Angle);
        an = (state.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
        state.p_mobj.mo_mut(mo).momx = fixed_mul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as Fixed,
            FINECOSINE[an as usize],
        );
        state.p_mobj.mo_mut(mo).momy = fixed_mul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as Fixed,
            FINESINE[an as usize],
        );
        mo = spawn_missile(state, actor, target, MobjType::Fatshot);
        state.p_mobj.mo_mut(mo).angle = state
            .p_mobj
            .mo(mo)
            .angle
            .wrapping_add((FATSPREAD / 2) as Angle);
        an = (state.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
        state.p_mobj.mo_mut(mo).momx = fixed_mul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as Fixed,
            FINECOSINE[an as usize],
        );
        state.p_mobj.mo_mut(mo).momy = fixed_mul(
            state.info.mobjinfo_mut(state.p_mobj.mo(mo).kind).speed as Fixed,
            FINESINE[an as usize],
        );
    }
}
pub const SKULLSPEED: i32 = 20 * FRACUNIT;
pub fn skull_attack(state: &mut GameState, id: MobjId) {
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
        s_start_sound(state, SoundOrigin::Mobj(actor), attacksound);
        face_target(state, actor);
        let an: Angle = state.p_mobj.mo(actor).angle >> ANGLETOFINESHIFT;
        state.p_mobj.mo_mut(actor).momx = fixed_mul(SKULLSPEED, FINECOSINE[an as usize]);
        state.p_mobj.mo_mut(actor).momy = fixed_mul(SKULLSPEED, FINESINE[an as usize]);
        dist = aprox_distance(
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
            / dist) as Fixed;
    }
}
pub fn pain_shoot_skull(state: &mut GameState, actor: MobjId, angle: Angle) {
    let mut count: i32 = 0;
    count += mobj_thinker_ids(state)
        .into_iter()
        .filter(|&m| state.p_mobj.mo(m).kind as u32 == MobjType::Skull as i32 as u32)
        .count() as i32;
    if count > 20 {
        return;
    }
    let an: Angle = angle >> ANGLETOFINESHIFT;
    let prestep: i32 = 4 * FRACUNIT
        + 3 * (state.info.mobjinfo_mut(state.p_mobj.mo(actor).kind).radius
            + state.info.mobjinfo[MobjType::Skull as usize].radius)
            / 2;
    let x: Fixed = state.p_mobj.mo(actor).x + fixed_mul(prestep as Fixed, FINECOSINE[an as usize]);
    let y: Fixed = state.p_mobj.mo(actor).y + fixed_mul(prestep as Fixed, FINESINE[an as usize]);
    let z: Fixed = (state.p_mobj.mo(actor).z + 8 * FRACUNIT) as Fixed;
    let newmobj: MobjId = spawn_mobj(state, x, y, z, MobjType::Skull);
    let (new_x, new_y) = {
        let n = state.p_mobj.mo(newmobj);
        (n.x, n.y)
    };
    if !try_move(state, newmobj, new_x, new_y) {
        damage_mobj(state, newmobj, Some(actor), Some(actor), 10000);
        return;
    }
    state.p_mobj.mo_mut(newmobj).target = state.p_mobj.mo(actor).target;
    skull_attack(state, newmobj);
}
pub fn pain_attack(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        if state.p_mobj.mo(actor).target.is_none() {
            return;
        }
        face_target(state, actor);
        pain_shoot_skull(state, actor, state.p_mobj.mo(actor).angle);
    }
}
pub fn pain_die(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        fall(state, actor);
        pain_shoot_skull(
            state,
            actor,
            state.p_mobj.mo(actor).angle.wrapping_add(ANG90 as Angle),
        );
        pain_shoot_skull(
            state,
            actor,
            state.p_mobj.mo(actor).angle.wrapping_add(ANG180),
        );
        pain_shoot_skull(
            state,
            actor,
            state.p_mobj.mo(actor).angle.wrapping_add(ANG270),
        );
    }
}
pub fn scream(state: &mut GameState, id: MobjId) {
    {
        let actor = id;

        let sound: i32 = match state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).kind)
            .deathsound
        {
            0 => return,
            59..=61 => SfxName::Podth1 as i32 + p_random(&mut state.m_random) % 3,
            62 | 63 => SfxName::Bgdth1 as i32 + p_random(&mut state.m_random) % 2,
            _ => {
                state
                    .info
                    .mobjinfo_mut(state.p_mobj.mo(actor).kind)
                    .deathsound
            }
        };
        if state.p_mobj.mo(actor).kind as u32 == MobjType::Spider as i32 as u32
            || state.p_mobj.mo(actor).kind as u32 == MobjType::Cyborg as i32 as u32
        {
            s_start_sound(state, SoundOrigin::None, sound);
        } else {
            s_start_sound(state, SoundOrigin::Mobj(actor), sound);
        };
    }
}
pub fn xscream(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Slop as i32);
    }
}
pub fn pain(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        let painsound = state
            .info
            .mobjinfo_mut(state.p_mobj.mo(actor).kind)
            .painsound;
        if painsound != 0 {
            s_start_sound(state, SoundOrigin::Mobj(actor), painsound);
        }
    }
}
pub fn fall(state: &mut GameState, id: MobjId) {
    {
        let actor = id;
        state.p_mobj.mo_mut(actor).flags &= !MF_SOLID;
    }
}
pub fn explode(state: &mut GameState, id: MobjId) {
    let thingy = id;
    let target = state
        .p_mobj
        .mo(thingy)
        .target
        .filter(|&id| state.p_mobj.is_live(id));
    p_radius_attack(state, thingy, target, 128);
}
fn check_boss_end(state: &mut GameState, motype: MobjType) -> bool {
    if !state.doomstat.gameversion.is_ultimate_or_higher() {
        if state.g_game.gamemap != 8 {
            return false;
        }
        if motype as u32 == MobjType::Bruiser as i32 as u32 && state.g_game.gameepisode != 1 {
            return false;
        }
        true
    } else {
        match state.g_game.gameepisode {
            1 => state.g_game.gamemap == 8 && motype as u32 == MobjType::Bruiser as i32 as u32,
            2 => state.g_game.gamemap == 8 && motype as u32 == MobjType::Cyborg as i32 as u32,
            3 => state.g_game.gamemap == 8 && motype as u32 == MobjType::Spider as i32 as u32,
            4 => {
                state.g_game.gamemap == 6 && motype as u32 == MobjType::Cyborg as i32 as u32
                    || state.g_game.gamemap == 8 && motype as u32 == MobjType::Spider as i32 as u32
            }
            _ => state.g_game.gamemap == 8,
        }
    }
}
pub fn boss_death(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        let mut i: i32;
        if state.doomstat.gamemode as u32 == GameMode::Commercial as i32 as u32 {
            if state.g_game.gamemap != 7 {
                return;
            }
            if state.p_mobj.mo(mo).kind as u32 != MobjType::Fatso as i32 as u32
                && state.p_mobj.mo(mo).kind as u32 != MobjType::Baby as i32 as u32
            {
                return;
            }
        } else if !check_boss_end(state, state.p_mobj.mo(mo).kind) {
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
        for mo2 in mobj_thinker_ids(state) {
            if mo2 != mo
                && state.p_mobj.mo(mo2).kind as u32 == mo_type as u32
                && state.p_mobj.mo(mo2).health > 0
            {
                return;
            }
        }
        if state.doomstat.gamemode as u32 == GameMode::Commercial as i32 as u32 {
            if state.g_game.gamemap == 7 {
                if state.p_mobj.mo(mo).kind as u32 == MobjType::Fatso as i32 as u32 {
                    let junk = state.p_setup.junk_line(666_i16);
                    do_floor(state, junk, FloorE::LowerFloorToLowest);
                    return;
                }
                if state.p_mobj.mo(mo).kind as u32 == MobjType::Baby as i32 as u32 {
                    let junk = state.p_setup.junk_line(667_i16);
                    do_floor(state, junk, FloorE::RaiseToTexture);
                    return;
                }
            }
        } else {
            match state.g_game.gameepisode {
                1 => {
                    let junk = state.p_setup.junk_line(666_i16);
                    do_floor(state, junk, FloorE::LowerFloorToLowest);
                    return;
                }
                4 => match state.g_game.gamemap {
                    6 => {
                        let junk = state.p_setup.junk_line(666_i16);
                        do_door(state, junk, VldoorE::BlazeOpen);
                        return;
                    }
                    8 => {
                        let junk = state.p_setup.junk_line(666_i16);
                        do_floor(state, junk, FloorE::LowerFloorToLowest);
                        return;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        exit_level(state);
    }
}
pub fn hoof(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        s_start_sound(state, SoundOrigin::Mobj(mo), SfxName::Hoof as i32);
        chase(state, mo);
    }
}
pub fn metal(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        s_start_sound(state, SoundOrigin::Mobj(mo), SfxName::Metal as i32);
        chase(state, mo);
    }
}
pub fn baby_metal(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        s_start_sound(state, SoundOrigin::Mobj(mo), SfxName::Bspwlk as i32);
        chase(state, mo);
    }
}
pub fn open_shotgun2(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        s_start_sound(
            state,
            SoundOrigin::Mobj(state.g_game.players[player.0 as usize].mo.unwrap()),
            SfxName::Dbopn as i32,
        );
    }
}
pub fn load_shotgun2(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        s_start_sound(
            state,
            SoundOrigin::Mobj(state.g_game.players[player.0 as usize].mo.unwrap()),
            SfxName::Dbload as i32,
        );
    }
}
pub fn close_shotgun2(state: &mut GameState, player_id: PlayerId, position: i32) {
    {
        let player = player_id;
        s_start_sound(
            state,
            SoundOrigin::Mobj(state.g_game.players[player.0 as usize].mo.unwrap()),
            SfxName::Dbcls as i32,
        );
        re_fire(state, player_id, position);
    }
}
pub fn brain_awake(state: &mut GameState, _id: MobjId) {
    state.p_enemy.numbraintargets = 0;
    state.p_enemy.braintargeton = 0;
    for m in mobj_thinker_ids(state) {
        if state.p_mobj.mo(m).kind as u32 == MobjType::Bosstarget as i32 as u32 {
            let n = state.p_enemy.numbraintargets as usize;
            state.p_enemy.braintargets[n] = Some(m);
            state.p_enemy.numbraintargets += 1;
        }
    }
    s_start_sound(state, SoundOrigin::None, SfxName::Bossit as i32);
}
pub fn brain_pain(state: &mut GameState, _id: MobjId) {
    s_start_sound(state, SoundOrigin::None, SfxName::Bospn as i32);
}
pub fn brain_scream(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        let mut x: i32;
        let mut y: i32;
        let mut z: i32;
        let mut th: MobjId;
        x = state.p_mobj.mo(mo).x - 196 * FRACUNIT;
        while x < state.p_mobj.mo(mo).x + 320 * FRACUNIT {
            y = state.p_mobj.mo(mo).y - 320 * FRACUNIT;
            z = 128 + p_random(&mut state.m_random) * 2 * FRACUNIT;
            th = spawn_mobj(state, x as Fixed, y as Fixed, z as Fixed, MobjType::Rocket);
            state.p_mobj.mo_mut(th).momz = (p_random(&mut state.m_random) * 512) as Fixed;
            set_mobj_state(state, th, StateNum::Brainexplode1);
            state.p_mobj.mo_mut(th).tics -= p_random(&mut state.m_random) & 7;
            if state.p_mobj.mo(th).tics < 1 {
                state.p_mobj.mo_mut(th).tics = 1;
            }
            x += FRACUNIT * 8;
        }
        s_start_sound(state, SoundOrigin::None, SfxName::Bosdth as i32);
    }
}
pub fn brain_explode(state: &mut GameState, id: MobjId) {
    {
        let mo = id;

        let x: i32 = state.p_mobj.mo(mo).x
            + (p_random(&mut state.m_random) - p_random(&mut state.m_random)) * 2048;
        let y: i32 = state.p_mobj.mo(mo).y;
        let z: i32 = 128 + p_random(&mut state.m_random) * 2 * FRACUNIT;
        let th: MobjId = spawn_mobj(state, x as Fixed, y as Fixed, z as Fixed, MobjType::Rocket);
        state.p_mobj.mo_mut(th).momz = (p_random(&mut state.m_random) * 512) as Fixed;
        set_mobj_state(state, th, StateNum::Brainexplode1);
        state.p_mobj.mo_mut(th).tics -= p_random(&mut state.m_random) & 7;
        if state.p_mobj.mo(th).tics < 1 {
            state.p_mobj.mo_mut(th).tics = 1;
        }
    }
}
pub fn brain_die(state: &mut GameState, _id: MobjId) {
    exit_level(state);
}
pub fn brain_spit(state: &mut GameState, id: MobjId) {
    {
        let mo = id;

        state.p_enemy.easy ^= 1;
        if state.g_game.gameskill <= SkillType::Easy && state.p_enemy.easy == 0 {
            return;
        }
        let targ_id = state.p_enemy.braintargets[state.p_enemy.braintargeton as usize].unwrap();
        let targ: MobjId = targ_id;
        state.p_enemy.braintargeton =
            (state.p_enemy.braintargeton + 1) % state.p_enemy.numbraintargets;
        let newmobj: MobjId = spawn_missile(state, mo, targ, MobjType::Spawnshot);
        state.p_mobj.mo_mut(newmobj).target = Some(targ);
        state.p_mobj.mo_mut(newmobj).reactiontime = (state.p_mobj.mo(targ).y
            - state.p_mobj.mo(mo).y)
            / state.p_mobj.mo(newmobj).momy
            / state
                .info
                .state_mut(state.p_mobj.mo(newmobj).state.unwrap())
                .tics;
        s_start_sound(state, SoundOrigin::None, SfxName::Bospit as i32);
    }
}
pub fn spawn_sound(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        s_start_sound(state, SoundOrigin::Mobj(mo), SfxName::Boscub as i32);
        spawn_fly(state, mo);
    }
}
pub fn spawn_fly(state: &mut GameState, id: MobjId) {
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
        let targ_id = subst_null_mobj(&mut state.p_mobj, targ_subst);
        let targ: MobjId = targ_id;
        let fog: MobjId = spawn_mobj(
            state,
            state.p_mobj.mo(targ).x,
            state.p_mobj.mo(targ).y,
            state.p_mobj.mo(targ).z,
            MobjType::Spawnfire,
        );
        s_start_sound(state, SoundOrigin::Mobj(fog), SfxName::Telept as i32);
        let r: i32 = p_random(&mut state.m_random);
        let kind: MobjType = if r < 50 {
            MobjType::Troop
        } else if r < 90 {
            MobjType::Sergeant
        } else if r < 120 {
            MobjType::Shadows
        } else if r < 130 {
            MobjType::Pain
        } else if r < 160 {
            MobjType::Head
        } else if r < 162 {
            MobjType::Vile
        } else if r < 172 {
            MobjType::Undead
        } else if r < 192 {
            MobjType::Baby
        } else if r < 222 {
            MobjType::Fatso
        } else if r < 246 {
            MobjType::Knight
        } else {
            MobjType::Bruiser
        };
        let newmobj: MobjId = spawn_mobj(
            state,
            state.p_mobj.mo(targ).x,
            state.p_mobj.mo(targ).y,
            state.p_mobj.mo(targ).z,
            kind,
        );
        if look_for_players(state, newmobj, true) {
            let seestate = state
                .info
                .mobjinfo_mut(state.p_mobj.mo(newmobj).kind)
                .seestate;
            set_mobj_state(state, newmobj, seestate);
        }
        teleport_move(
            state,
            newmobj,
            state.p_mobj.mo(newmobj).x,
            state.p_mobj.mo(newmobj).y,
        );
        remove_mobj(state, mo);
    }
}
pub fn player_scream(state: &mut GameState, id: MobjId) {
    {
        let mo = id;
        let mut sound: i32 = SfxName::Pldeth as i32;
        if state.doomstat.gamemode as u32 == GameMode::Commercial as i32 as u32
            && state.p_mobj.mo(mo).health < -50
        {
            sound = SfxName::Pdiehi as i32;
        }
        s_start_sound(state, SoundOrigin::Mobj(mo), sound);
    }
}
