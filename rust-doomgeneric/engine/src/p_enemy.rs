use crate::d_mode::GameMode;
use crate::d_mode::SkillType;
use crate::doomstat::DoomstatState;
use crate::g_game::GGameState;
use crate::p_mobj::LineFlags;
use crate::p_mobj::MobjFlags;
use crate::p_mobj::PMobjState;

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

use crate::game_state::GameState;
use crate::m_fixed::FRACUNIT;
use crate::p_maputl::MAPBLOCKSHIFT;
use crate::p_mobj::StateNum;
use crate::p_mobj::FLOATSPEED;
use crate::p_pspr::re_fire;

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
        Self {
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
    East,
    Northeast,
    North,
    Northwest,
    West,
    Southwest,
    South,
    Southeast,
    Nodir,
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
/// The mobj's target, unless that target has since been removed.
fn live_target(p_mobj: &PMobjState, mobj: MobjId) -> Option<MobjId> {
    p_mobj.mo(mobj).target.filter(|&id| p_mobj.is_live(id))
}
pub fn recursive_sound(state: &mut GameState, sec: SectorId, soundblocks: i32) {
    let validcount = state.world.p_setup.validcount;
    {
        let s = state.world.p_setup.sector_mut(sec);
        if s.validcount == validcount && s.soundtraversed <= soundblocks + 1 {
            return;
        }
        s.validcount = validcount;
        s.soundtraversed = soundblocks + 1;
        s.soundtarget = state.world.p_enemy.soundtarget;
    }
    let linecount = state.world.p_setup.sector_mut(sec).linecount;
    for i in 0..linecount as usize {
        let check = state.world.p_setup.sector_mut(sec).lines[i];
        let checkv = state.world.p_setup.line(check);
        if checkv.flags.contains(LineFlags::TWOSIDED) {
            line_opening(&mut state.world.p_maputl, &mut state.world.p_setup, check);
            if state.world.p_maputl.openrange > 0 {
                let other = if state.world.p_setup.sides[checkv.sidenum[0] as usize].sector == sec {
                    state.world.p_setup.sides[checkv.sidenum[1] as usize].sector
                } else {
                    state.world.p_setup.sides[checkv.sidenum[0] as usize].sector
                };
                if checkv.flags.contains(LineFlags::SOUNDBLOCK) {
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
    state.world.p_enemy.soundtarget = Some(target);
    state.world.p_setup.validcount += 1;
    let emmiter_subsector = state.world.p_mobj.mo(emmiter).subsector;
    let sec = state.world.p_setup.subsectors[emmiter_subsector.0 as usize].sector;
    recursive_sound(state, sec, 0);
}
pub fn check_melee_range(state: &mut GameState, actor: MobjId) -> bool {
    let Some(pl) = state
        .world
        .p_mobj
        .mo(actor)
        .target
        .filter(|&id| state.world.p_mobj.is_live(id))
    else {
        return false;
    };
    let (pl_x, pl_y, pl_type) = {
        let p = state.world.p_mobj.mo(pl);
        (p.x, p.y, p.kind)
    };
    let (actor_x, actor_y) = {
        let a = state.world.p_mobj.mo(actor);
        (a.x, a.y)
    };
    let dist = aprox_distance(pl_x - actor_x, pl_y - actor_y);
    if dist >= MELEERANGE - 20 * FRACUNIT + state.assets.info.mobjinfo_mut(pl_type).radius {
        return false;
    }
    if !check_sight(state, actor, pl) {
        return false;
    }
    true
}
pub fn check_missile_range(state: &mut GameState, actor: MobjId) -> bool {
    let Some(target) = state
        .world
        .p_mobj
        .mo(actor)
        .target
        .filter(|&id| state.world.p_mobj.is_live(id))
    else {
        return false;
    };
    if !check_sight(state, actor, target) {
        return false;
    }
    if state
        .world
        .p_mobj
        .mo(actor)
        .flags
        .contains(MobjFlags::JUSTHIT)
    {
        state.world.p_mobj.mo_mut(actor).flags &= !MobjFlags::JUSTHIT;
        return true;
    }
    if state.world.p_mobj.mo(actor).reactiontime != 0 {
        return false;
    }
    let (actor_x, actor_y, actor_type) = {
        let a = state.world.p_mobj.mo(actor);
        (a.x, a.y, a.kind)
    };
    let (target_x, target_y) = {
        let t = state.world.p_mobj.mo(target);
        (t.x, t.y)
    };
    let mut dist: Fixed =
        (aprox_distance(actor_x - target_x, actor_y - target_y) - 64 * FRACUNIT) as Fixed;
    if state.assets.info.mobjinfo_mut(actor_type).meleestate == StateNum::Null {
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
    if p_random(&mut state.world.m_random) < dist {
        return false;
    }
    true
}
pub static XSPEED: [Fixed; 8] = [FRACUNIT, 47000, 0, -47000, -FRACUNIT, -47000, 0, 47000];
pub static YSPEED: [Fixed; 8] = [0, 47000, FRACUNIT, 47000, 0, -47000, -FRACUNIT, -47000];
pub fn p_move(state: &mut GameState, actor: MobjId) -> bool {
    if state.world.p_mobj.mo(actor).movedir == DirType::Nodir as i32 {
        return false;
    }
    if state.world.p_mobj.mo(actor).movedir as u32 >= 8 {
        error("Weird actor->movedir!");
    }
    let tryx: Fixed = state.world.p_mobj.mo(actor).x
        + state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .speed as Fixed
            * XSPEED[state.world.p_mobj.mo(actor).movedir as usize];
    let tryy: Fixed = state.world.p_mobj.mo(actor).y
        + state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .speed as Fixed
            * YSPEED[state.world.p_mobj.mo(actor).movedir as usize];
    let try_ok: bool = try_move(state, actor, tryx, tryy);
    if try_ok {
        state.world.p_mobj.mo_mut(actor).flags &= !MobjFlags::INFLOAT;
    } else {
        if state
            .world
            .p_mobj
            .mo(actor)
            .flags
            .contains(MobjFlags::FLOAT)
            && state.world.p_map.floatok
        {
            if state.world.p_mobj.mo(actor).z < state.world.p_map.tmfloorz {
                state.world.p_mobj.mo_mut(actor).z += FLOATSPEED;
            } else {
                state.world.p_mobj.mo_mut(actor).z -= FLOATSPEED;
            }
            state.world.p_mobj.mo_mut(actor).flags |= MobjFlags::INFLOAT;
            return true;
        }
        if state.world.p_map.numspechit == 0 {
            return false;
        }
        state.world.p_mobj.mo_mut(actor).movedir = DirType::Nodir as i32;
        let mut good: bool = false;
        while state.world.p_map.numspechit > 0 {
            state.world.p_map.numspechit -= 1;
            let ld: LineId = state.world.p_map.spechit[state.world.p_map.numspechit as usize];
            if use_special_line(state, actor, ld, 0) {
                good = true;
            }
        }
        return good;
    }
    if !state
        .world
        .p_mobj
        .mo(actor)
        .flags
        .contains(MobjFlags::FLOAT)
    {
        state.world.p_mobj.mo_mut(actor).z = state.world.p_mobj.mo(actor).floorz;
    }
    true
}
pub fn try_walk(state: &mut GameState, actor: MobjId) -> bool {
    if !p_move(state, actor) {
        return false;
    }
    state.world.p_mobj.mo_mut(actor).movecount = p_random(&mut state.world.m_random) & 15;
    true
}
pub fn new_chase_dir(state: &mut GameState, actor: MobjId) {
    let mut d: [DirType; 3] = [DirType::East; 3];

    let Some(target) = live_target(&state.world.p_mobj, actor) else {
        error("P_NewChaseDir: called with no target");
    };
    let olddir: DirType = dirtype_from_movedir(state.world.p_mobj.mo(actor).movedir);
    let turnaround: DirType = OPPOSITE[olddir as usize];
    let deltax: Fixed = state.world.p_mobj.mo(target).x - state.world.p_mobj.mo(actor).x;
    let deltay: Fixed = state.world.p_mobj.mo(target).y - state.world.p_mobj.mo(actor).y;
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
        state.world.p_mobj.mo_mut(actor).movedir =
            DIAGS[((((deltay < 0) as i32) << 1) + (deltax > 0) as i32) as usize] as i32;
        if state.world.p_mobj.mo(actor).movedir != turnaround as i32 && try_walk(state, actor) {
            return;
        }
    }
    if p_random(&mut state.world.m_random) > 200 || deltay.abs() > deltax.abs() {
        d.swap(1, 2);
    }
    if d[1] == turnaround {
        d[1] = DirType::Nodir;
    }
    if d[2] == turnaround {
        d[2] = DirType::Nodir;
    }
    if d[1] != DirType::Nodir {
        state.world.p_mobj.mo_mut(actor).movedir = d[1] as i32;
        if try_walk(state, actor) {
            return;
        }
    }
    if d[2] != DirType::Nodir {
        state.world.p_mobj.mo_mut(actor).movedir = d[2] as i32;
        if try_walk(state, actor) {
            return;
        }
    }
    if olddir != DirType::Nodir {
        state.world.p_mobj.mo_mut(actor).movedir = olddir as i32;
        if try_walk(state, actor) {
            return;
        }
    }
    if p_random(&mut state.world.m_random) & 1 != 0 {
        for tdir in DirType::East as i32..=DirType::Southeast as i32 {
            if tdir != turnaround as i32 {
                state.world.p_mobj.mo_mut(actor).movedir = tdir;
                if try_walk(state, actor) {
                    return;
                }
            }
        }
    } else {
        let mut tdir: i32 = DirType::Southeast as i32;
        while tdir != DirType::East as i32 - 1 {
            if tdir != turnaround as i32 {
                state.world.p_mobj.mo_mut(actor).movedir = tdir;
                if try_walk(state, actor) {
                    return;
                }
            }
            tdir -= 1;
        }
    }
    if turnaround != DirType::Nodir {
        state.world.p_mobj.mo_mut(actor).movedir = turnaround as i32;
        if try_walk(state, actor) {
            return;
        }
    }
    state.world.p_mobj.mo_mut(actor).movedir = DirType::Nodir as i32;
}
pub fn look_for_players(state: &mut GameState, actor: MobjId, allaround: bool) -> bool {
    let mut c: i32 = 0;
    let stop = (state.world.p_mobj.mo(actor).lastlook - 1) & 3;
    loop {
        let lastlook = state.world.p_mobj.mo(actor).lastlook;
        if state.game.g_game.playeringame[lastlook as usize] {
            if c == 2 || lastlook == stop {
                return false;
            }
            c += 1;
            let (health, player_mo) = {
                let player = &state.game.g_game.players[lastlook as usize];
                (player.health, player.mo)
            };
            if health > 0 {
                let player_mo = player_mo.unwrap();
                if check_sight(state, actor, player_mo) {
                    let mut skip = false;
                    if !allaround {
                        let (actor_x, actor_y, actor_angle) = {
                            let a = state.world.p_mobj.mo(actor);
                            (a.x, a.y, a.angle)
                        };
                        let (pmo_x, pmo_y) = {
                            let p = state.world.p_mobj.mo(player_mo);
                            (p.x, p.y)
                        };
                        let an = point_to_angle2(actor_x, actor_y, pmo_x, pmo_y)
                            .wrapping_sub(actor_angle);
                        if an > ANG90 as Angle && an < ANG270 {
                            let dist = aprox_distance(pmo_x - actor_x, pmo_y - actor_y);
                            if dist > MELEERANGE {
                                skip = true;
                            }
                        }
                    }
                    if !skip {
                        state.world.p_mobj.mo_mut(actor).target = Some(player_mo);
                        return true;
                    }
                }
            }
        }
        state.world.p_mobj.mo_mut(actor).lastlook = (lastlook + 1) & 3;
    }
}
pub fn keen_die(state: &mut GameState, id: MobjId) {
    let mo = id;
    fall(state, mo);
    let mo_type = state.world.p_mobj.mo(mo).kind;
    for mo2 in mobj_thinker_ids(&state.world.p_mobj, &state.world.p_tick) {
        if mo2 != mo
            && state.world.p_mobj.mo(mo2).kind as u32 == mo_type as u32
            && state.world.p_mobj.mo(mo2).health > 0
        {
            return;
        }
    }
    let junk = state.world.p_setup.junk_line(666_i16);
    do_door(state, junk, VldoorE::Open);
}
pub fn look(state: &mut GameState, actor: MobjId) {
    state.world.p_mobj.mo_mut(actor).threshold = 0;
    let targ: Option<MobjId> = state
        .world
        .p_setup
        .sector_mut(
            state.world.p_setup.subsectors[state.world.p_mobj.mo(actor).subsector.0 as usize]
                .sector,
        )
        .soundtarget
        .filter(|&target| state.world.p_mobj.is_live(target));
    // Whether the actor already sees its target (and so skips looking around).
    let sees_target = if let Some(targ) = targ.filter(|&t| {
        state
            .world
            .p_mobj
            .mo(t)
            .flags
            .contains(MobjFlags::SHOOTABLE)
    }) {
        state.world.p_mobj.mo_mut(actor).target = Some(targ);
        !state
            .world
            .p_mobj
            .mo(actor)
            .flags
            .contains(MobjFlags::AMBUSH)
            || check_sight(state, actor, targ)
    } else {
        false
    };
    if !sees_target && !look_for_players(state, actor, false) {
        return;
    }
    if state
        .assets
        .info
        .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
        .seesound
        != SfxName::SfxNone
    {
        let seesound = state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .seesound;
        let sound = match seesound {
            SfxName::Posit1 | SfxName::Posit2 | SfxName::Posit3 => {
                [SfxName::Posit1, SfxName::Posit2, SfxName::Posit3]
                    [(p_random(&mut state.world.m_random) % 3) as usize]
            }
            SfxName::Bgsit1 | SfxName::Bgsit2 => [SfxName::Bgsit1, SfxName::Bgsit2]
                [(p_random(&mut state.world.m_random) % 2) as usize],
            other => other,
        };
        if state.world.p_mobj.mo(actor).kind as u32 == MobjType::Spider as i32 as u32
            || state.world.p_mobj.mo(actor).kind as u32 == MobjType::Cyborg as i32 as u32
        {
            s_start_sound(state, SoundOrigin::None, sound);
        } else {
            s_start_sound(state, SoundOrigin::Mobj(actor), sound);
        }
    }
    let seestate = state
        .assets
        .info
        .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
        .seestate;
    set_mobj_state(state, actor, seestate);
}
pub fn chase(state: &mut GameState, actor: MobjId) {
    if state.world.p_mobj.mo(actor).reactiontime != 0 {
        state.world.p_mobj.mo_mut(actor).reactiontime -= 1;
    }
    let target = state
        .world
        .p_mobj
        .mo(actor)
        .target
        .filter(|&target| state.world.p_mobj.is_live(target));
    if state.world.p_mobj.mo(actor).threshold != 0 {
        if target.is_none() || state.world.p_mobj.mo(target.unwrap()).health <= 0 {
            state.world.p_mobj.mo_mut(actor).threshold = 0;
        } else {
            state.world.p_mobj.mo_mut(actor).threshold -= 1;
        }
    }
    if state.world.p_mobj.mo(actor).movedir < 8 {
        state.world.p_mobj.mo_mut(actor).angle &= (7 << 29) as Angle;
        let delta: i32 = state
            .world
            .p_mobj
            .mo(actor)
            .angle
            .wrapping_sub((state.world.p_mobj.mo(actor).movedir << 29) as Angle)
            as i32;
        if delta > 0 {
            state.world.p_mobj.mo_mut(actor).angle = state
                .world
                .p_mobj
                .mo(actor)
                .angle
                .wrapping_sub((ANG90 / 2) as Angle);
        } else if delta < 0 {
            state.world.p_mobj.mo_mut(actor).angle = state
                .world
                .p_mobj
                .mo(actor)
                .angle
                .wrapping_add((ANG90 / 2) as Angle);
        }
    }
    if target.is_none()
        || !state
            .world
            .p_mobj
            .mo(target.unwrap())
            .flags
            .contains(MobjFlags::SHOOTABLE)
    {
        if look_for_players(state, actor, true) {
            return;
        }
        let spawnstate = state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .spawnstate;
        set_mobj_state(state, actor, spawnstate);
        return;
    }
    if state
        .world
        .p_mobj
        .mo(actor)
        .flags
        .contains(MobjFlags::JUSTATTACKED)
    {
        state.world.p_mobj.mo_mut(actor).flags &= !MobjFlags::JUSTATTACKED;
        if state.game.g_game.gameskill != SkillType::Nightmare && !state.game.d_main.fastparm {
            new_chase_dir(state, actor);
        }
        return;
    }
    let actor_info = state
        .assets
        .info
        .mobjinfo_mut(state.world.p_mobj.mo(actor).kind);
    if actor_info.meleestate != StateNum::Null && check_melee_range(state, actor) {
        let attacksound = state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .attacksound;
        if attacksound != SfxName::SfxNone {
            s_start_sound(state, SoundOrigin::Mobj(actor), attacksound);
        }
        let meleestate = state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .meleestate;
        set_mobj_state(state, actor, meleestate);
        return;
    }
    if state
        .assets
        .info
        .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
        .missilestate
        != StateNum::Null
        && !(state.game.g_game.gameskill < SkillType::Nightmare
            && !state.game.d_main.fastparm
            && state.world.p_mobj.mo(actor).movecount != 0)
        && check_missile_range(state, actor)
    {
        let missilestate = state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .missilestate;
        set_mobj_state(state, actor, missilestate);
        state.world.p_mobj.mo_mut(actor).flags |= MobjFlags::JUSTATTACKED;
        return;
    }
    if state.game.g_game.netgame
        && state.world.p_mobj.mo(actor).threshold == 0
        && !check_sight(state, actor, state.world.p_mobj.mo(target.unwrap()).id)
        && look_for_players(state, actor, true)
    {
        return;
    }
    state.world.p_mobj.mo_mut(actor).movecount -= 1;
    if state.world.p_mobj.mo(actor).movecount < 0 || !p_move(state, actor) {
        new_chase_dir(state, actor);
    }
    let activesound = state
        .assets
        .info
        .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
        .activesound;
    if activesound != SfxName::SfxNone && p_random(&mut state.world.m_random) < 3 {
        s_start_sound(state, SoundOrigin::Mobj(actor), activesound);
    }
}
pub fn face_target(state: &mut GameState, actor: MobjId) {
    let Some(target) = live_target(&state.world.p_mobj, actor) else {
        return;
    };
    state.world.p_mobj.mo_mut(actor).flags &= !MobjFlags::AMBUSH;
    state.world.p_mobj.mo_mut(actor).angle = point_to_angle2(
        state.world.p_mobj.mo(actor).x,
        state.world.p_mobj.mo(actor).y,
        state.world.p_mobj.mo(target).x,
        state.world.p_mobj.mo(target).y,
    );
    if state
        .world
        .p_mobj
        .mo(target)
        .flags
        .contains(MobjFlags::SHADOW)
    {
        state.world.p_mobj.mo_mut(actor).angle = state.world.p_mobj.mo(actor).angle.wrapping_add(
            ((p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 21)
                as Angle,
        );
    }
}
pub fn pos_attack(state: &mut GameState, actor: MobjId) {
    if state.world.p_mobj.mo(actor).target.is_none() {
        return;
    }
    face_target(state, actor);
    let mut angle: i32 = state.world.p_mobj.mo(actor).angle as i32;
    let slope: i32 = aim_line_attack(state, Some(actor), angle as Angle, MISSILERANGE);
    s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Pistol);
    angle += (p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 20;
    let damage: i32 = (p_random(&mut state.world.m_random) % 5 + 1) * 3;
    line_attack(
        state,
        actor,
        angle as Angle,
        MISSILERANGE,
        slope as Fixed,
        damage,
    );
}
pub fn spos_attack(state: &mut GameState, actor: MobjId) {
    if state.world.p_mobj.mo(actor).target.is_none() {
        return;
    }
    s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Shotgn);
    face_target(state, actor);
    let bangle: i32 = state.world.p_mobj.mo(actor).angle as i32;
    let slope: i32 = aim_line_attack(state, Some(actor), bangle as Angle, MISSILERANGE);
    for _ in 0..3 {
        let angle: i32 = bangle
            + ((p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 20);
        let damage: i32 = (p_random(&mut state.world.m_random) % 5 + 1) * 3;
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
pub fn cpos_attack(state: &mut GameState, actor: MobjId) {
    if state.world.p_mobj.mo(actor).target.is_none() {
        return;
    }
    s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Shotgn);
    face_target(state, actor);
    let bangle: i32 = state.world.p_mobj.mo(actor).angle as i32;
    let slope: i32 = aim_line_attack(state, Some(actor), bangle as Angle, MISSILERANGE);
    let angle: i32 = bangle
        + ((p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 20);
    let damage: i32 = (p_random(&mut state.world.m_random) % 5 + 1) * 3;
    line_attack(
        state,
        actor,
        angle as Angle,
        MISSILERANGE,
        slope as Fixed,
        damage,
    );
}
pub fn cpos_refire(state: &mut GameState, actor: MobjId) {
    face_target(state, actor);
    if p_random(&mut state.world.m_random) < 40 {
        return;
    }
    let target = state
        .world
        .p_mobj
        .mo(actor)
        .target
        .filter(|&target| state.world.p_mobj.is_live(target));
    if target.is_none()
        || state.world.p_mobj.mo(target.unwrap()).health <= 0
        || !check_sight(state, actor, state.world.p_mobj.mo(target.unwrap()).id)
    {
        let seestate = state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .seestate;
        set_mobj_state(state, actor, seestate);
    }
}
pub fn spid_refire(state: &mut GameState, actor: MobjId) {
    face_target(state, actor);
    if p_random(&mut state.world.m_random) < 10 {
        return;
    }
    let target = state
        .world
        .p_mobj
        .mo(actor)
        .target
        .filter(|&target| state.world.p_mobj.is_live(target));
    if target.is_none()
        || state.world.p_mobj.mo(target.unwrap()).health <= 0
        || !check_sight(state, actor, state.world.p_mobj.mo(target.unwrap()).id)
    {
        let seestate = state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .seestate;
        set_mobj_state(state, actor, seestate);
    }
}
pub fn bspi_attack(state: &mut GameState, actor: MobjId) {
    let Some(target) = live_target(&state.world.p_mobj, actor) else {
        return;
    };
    face_target(state, actor);
    spawn_missile(state, actor, target, MobjType::Arachplaz);
}
pub fn troop_attack(state: &mut GameState, actor: MobjId) {
    let Some(target) = live_target(&state.world.p_mobj, actor) else {
        return;
    };
    face_target(state, actor);
    if check_melee_range(state, actor) {
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Claw);
        let damage: i32 = (p_random(&mut state.world.m_random) % 8 + 1) * 3;
        damage_mobj(state, target, Some(actor), Some(actor), damage);
        return;
    }
    spawn_missile(state, actor, target, MobjType::Troopshot);
}
pub fn sarg_attack(state: &mut GameState, actor: MobjId) {
    let Some(target) = live_target(&state.world.p_mobj, actor) else {
        return;
    };
    face_target(state, actor);
    if check_melee_range(state, actor) {
        let damage: i32 = (p_random(&mut state.world.m_random) % 10 + 1) * 4;
        damage_mobj(state, target, Some(actor), Some(actor), damage);
    }
}
pub fn head_attack(state: &mut GameState, actor: MobjId) {
    let Some(target) = live_target(&state.world.p_mobj, actor) else {
        return;
    };
    face_target(state, actor);
    if check_melee_range(state, actor) {
        let damage: i32 = (p_random(&mut state.world.m_random) % 6 + 1) * 10;
        damage_mobj(state, target, Some(actor), Some(actor), damage);
        return;
    }
    spawn_missile(state, actor, target, MobjType::Headshot);
}
pub fn cyber_attack(state: &mut GameState, actor: MobjId) {
    let Some(target) = live_target(&state.world.p_mobj, actor) else {
        return;
    };
    face_target(state, actor);
    spawn_missile(state, actor, target, MobjType::Rocket);
}
pub fn bruis_attack(state: &mut GameState, actor: MobjId) {
    let Some(target) = live_target(&state.world.p_mobj, actor) else {
        return;
    };
    if check_melee_range(state, actor) {
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Claw);
        let damage: i32 = (p_random(&mut state.world.m_random) % 8 + 1) * 10;
        damage_mobj(state, target, Some(actor), Some(actor), damage);
        return;
    }
    spawn_missile(state, actor, target, MobjType::Bruisershot);
}
pub fn skel_missile(state: &mut GameState, actor: MobjId) {
    let Some(target) = live_target(&state.world.p_mobj, actor) else {
        return;
    };
    face_target(state, actor);
    state.world.p_mobj.mo_mut(actor).z += 16 * FRACUNIT;
    let mo: MobjId = spawn_missile(state, actor, target, MobjType::Tracer);
    state.world.p_mobj.mo_mut(actor).z -= 16 * FRACUNIT;
    state.world.p_mobj.mo_mut(mo).x += state.world.p_mobj.mo(mo).momx;
    state.world.p_mobj.mo_mut(mo).y += state.world.p_mobj.mo(mo).momy;
    state.world.p_mobj.mo_mut(mo).tracer = state.world.p_mobj.mo(actor).target;
}
pub static TRACEANGLE: i32 = 0xc000000;
pub fn a_tracer(state: &mut GameState, actor: MobjId) {
    if state.game.d_loop.gametic & 3 != 0 {
        return;
    }
    spawn_puff(
        state,
        state.world.p_mobj.mo(actor).x,
        state.world.p_mobj.mo(actor).y,
        state.world.p_mobj.mo(actor).z,
    );
    let th: MobjId = spawn_mobj(
        state,
        state.world.p_mobj.mo(actor).x - state.world.p_mobj.mo(actor).momx,
        state.world.p_mobj.mo(actor).y - state.world.p_mobj.mo(actor).momy,
        state.world.p_mobj.mo(actor).z,
        MobjType::Smoke,
    );
    state.world.p_mobj.mo_mut(th).momz = FRACUNIT as Fixed;
    state.world.p_mobj.mo_mut(th).tics -= p_random(&mut state.world.m_random) & 3;
    if state.world.p_mobj.mo(th).tics < 1 {
        state.world.p_mobj.mo_mut(th).tics = 1;
    }
    let dest: Option<MobjId> = state
        .world
        .p_mobj
        .mo(actor)
        .tracer
        .filter(|&target| state.world.p_mobj.is_live(target));
    if dest.is_none() || state.world.p_mobj.mo(dest.unwrap()).health <= 0 {
        return;
    }
    let mut exact: Angle = point_to_angle2(
        state.world.p_mobj.mo(actor).x,
        state.world.p_mobj.mo(actor).y,
        state.world.p_mobj.mo(dest.unwrap()).x,
        state.world.p_mobj.mo(dest.unwrap()).y,
    );
    if exact != state.world.p_mobj.mo(actor).angle {
        if exact.wrapping_sub(state.world.p_mobj.mo(actor).angle) > 0x80000000 {
            state.world.p_mobj.mo_mut(actor).angle = state
                .world
                .p_mobj
                .mo(actor)
                .angle
                .wrapping_sub(TRACEANGLE as Angle);
            if exact.wrapping_sub(state.world.p_mobj.mo(actor).angle) < 0x80000000 {
                state.world.p_mobj.mo_mut(actor).angle = exact;
            }
        } else {
            state.world.p_mobj.mo_mut(actor).angle = state
                .world
                .p_mobj
                .mo(actor)
                .angle
                .wrapping_add(TRACEANGLE as Angle);
            if exact.wrapping_sub(state.world.p_mobj.mo(actor).angle) > 0x80000000 {
                state.world.p_mobj.mo_mut(actor).angle = exact;
            }
        }
    }
    exact = state.world.p_mobj.mo(actor).angle >> ANGLETOFINESHIFT;
    state.world.p_mobj.mo_mut(actor).momx = fixed_mul(
        state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .speed as Fixed,
        FINECOSINE[exact as usize],
    );
    state.world.p_mobj.mo_mut(actor).momy = fixed_mul(
        state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .speed as Fixed,
        FINESINE[exact as usize],
    );
    let mut dist: Fixed = aprox_distance(
        state.world.p_mobj.mo(dest.unwrap()).x - state.world.p_mobj.mo(actor).x,
        state.world.p_mobj.mo(dest.unwrap()).y - state.world.p_mobj.mo(actor).y,
    );
    dist = (dist
        / state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .speed) as Fixed;
    if dist < 1 {
        dist = 1;
    }
    let slope: Fixed = (state.world.p_mobj.mo(dest.unwrap()).z + 40 * FRACUNIT
        - state.world.p_mobj.mo(actor).z)
        / dist;
    if slope < state.world.p_mobj.mo(actor).momz {
        state.world.p_mobj.mo_mut(actor).momz -= FRACUNIT / 8;
    } else {
        state.world.p_mobj.mo_mut(actor).momz += FRACUNIT / 8;
    }
}
pub fn skel_whoosh(state: &mut GameState, actor: MobjId) {
    if state.world.p_mobj.mo(actor).target.is_none() {
        return;
    }
    face_target(state, actor);
    s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Skeswg);
}
pub fn skel_fist(state: &mut GameState, actor: MobjId) {
    let Some(target) = live_target(&state.world.p_mobj, actor) else {
        return;
    };
    face_target(state, actor);
    if check_melee_range(state, actor) {
        let damage: i32 = (p_random(&mut state.world.m_random) % 10 + 1) * 6;
        s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Skepch);
        damage_mobj(state, target, Some(actor), Some(actor), damage);
    }
}
pub fn vile_check(state: &mut GameState, thing_id: MobjId) -> bool {
    let thing = thing_id;

    if !state
        .world
        .p_mobj
        .mo(thing)
        .flags
        .contains(MobjFlags::CORPSE)
    {
        return true;
    }
    if state.world.p_mobj.mo(thing).tics != -1 {
        return true;
    }
    if state
        .assets
        .info
        .mobjinfo_mut(state.world.p_mobj.mo(thing).kind)
        .raisestate
        == StateNum::Null
    {
        return true;
    }
    let maxdist: i32 = state
        .assets
        .info
        .mobjinfo_mut(state.world.p_mobj.mo(thing).kind)
        .radius
        + state.assets.info.mobjinfo[MobjType::Vile as usize].radius;
    if (state.world.p_mobj.mo(thing).x - state.world.p_enemy.viletryx).abs() > maxdist
        || (state.world.p_mobj.mo(thing).y - state.world.p_enemy.viletryy).abs() > maxdist
    {
        return true;
    }
    state.world.p_enemy.corpsehit = Some(thing);
    state.world.p_mobj.mo_mut(thing).momy = 0;
    state.world.p_mobj.mo_mut(thing).momx = state.world.p_mobj.mo(thing).momy;
    state.world.p_mobj.mo_mut(thing).height <<= 2;
    let check: bool = check_position(
        state,
        thing,
        state.world.p_mobj.mo(thing).x,
        state.world.p_mobj.mo(thing).y,
    );
    state.world.p_mobj.mo_mut(thing).height >>= 2;
    if !check {
        return true;
    }
    false
}
pub fn vile_chase(state: &mut GameState, id: MobjId) {
    let actor = id;
    if state.world.p_mobj.mo(actor).movedir != DirType::Nodir as i32 {
        state.world.p_enemy.viletryx = state.world.p_mobj.mo(actor).x
            + state
                .assets
                .info
                .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
                .speed as Fixed
                * XSPEED[state.world.p_mobj.mo(actor).movedir as usize];
        state.world.p_enemy.viletryy = state.world.p_mobj.mo(actor).y
            + state
                .assets
                .info
                .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
                .speed as Fixed
                * YSPEED[state.world.p_mobj.mo(actor).movedir as usize];
        let xl: i32 =
            (state.world.p_enemy.viletryx - state.world.p_setup.bmaporgx - 32 * FRACUNIT * 2)
                >> MAPBLOCKSHIFT;
        let xh: i32 = (state.world.p_enemy.viletryx - state.world.p_setup.bmaporgx
            + 32 * FRACUNIT * 2)
            >> MAPBLOCKSHIFT;
        let yl: i32 =
            (state.world.p_enemy.viletryy - state.world.p_setup.bmaporgy - 32 * FRACUNIT * 2)
                >> MAPBLOCKSHIFT;
        let yh: i32 = (state.world.p_enemy.viletryy - state.world.p_setup.bmaporgy
            + 32 * FRACUNIT * 2)
            >> MAPBLOCKSHIFT;
        state.world.p_enemy.vileobj = Some(actor);
        for bx in xl..=xh {
            for by in yl..=yh {
                if !block_things_iterator(state, bx, by, vile_check) {
                    let corpsehit_id = state.world.p_enemy.corpsehit.unwrap();
                    let corpsehit = corpsehit_id;
                    let temp: Option<MobjId> = state.world.p_mobj.mo(actor).target;
                    state.world.p_mobj.mo_mut(actor).target = Some(corpsehit_id);
                    face_target(state, actor);
                    state.world.p_mobj.mo_mut(actor).target = temp;
                    set_mobj_state(state, actor, StateNum::VileHeal1);
                    s_start_sound(state, SoundOrigin::Mobj(corpsehit_id), SfxName::Slop);
                    let info = state
                        .assets
                        .info
                        .mobjinfo_mut(state.world.p_mobj.mo(corpsehit).kind);
                    let (raisestate, info_flags, spawnhealth) =
                        (info.raisestate, info.flags, info.spawnhealth);
                    set_mobj_state(state, corpsehit, raisestate);
                    state.world.p_mobj.mo_mut(corpsehit).height <<= 2;
                    state.world.p_mobj.mo_mut(corpsehit).flags = info_flags;
                    state.world.p_mobj.mo_mut(corpsehit).health = spawnhealth;
                    state.world.p_mobj.mo_mut(corpsehit).target = None;
                    return;
                }
            }
        }
    }
    chase(state, actor);
}
pub fn vile_start(state: &mut GameState, actor: MobjId) {
    s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Vilatk);
}
pub fn start_fire(state: &mut GameState, actor: MobjId) {
    s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Flamst);
    a_fire(state, actor);
}
pub fn fire_crackle(state: &mut GameState, actor: MobjId) {
    s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Flame);
    a_fire(state, actor);
}
pub fn a_fire(state: &mut GameState, actor: MobjId) {
    let dest: Option<MobjId> = state
        .world
        .p_mobj
        .mo(actor)
        .tracer
        .filter(|&target| state.world.p_mobj.is_live(target));
    if dest.is_none() {
        return;
    }
    let target_subst = state
        .world
        .p_mobj
        .mo(actor)
        .target
        .filter(|&target| state.world.p_mobj.is_live(target));
    let target_id = subst_null_mobj(&mut state.world.p_mobj, target_subst);
    let target: MobjId = target_id;
    if !check_sight(state, target, dest.unwrap()) {
        return;
    }
    let an: u32 = state.world.p_mobj.mo(dest.unwrap()).angle >> ANGLETOFINESHIFT;
    unset_thing_position(&mut state.world.p_mobj, &mut state.world.p_setup, actor);
    state.world.p_mobj.mo_mut(actor).x =
        state.world.p_mobj.mo(dest.unwrap()).x + fixed_mul(24 * FRACUNIT, FINECOSINE[an as usize]);
    state.world.p_mobj.mo_mut(actor).y =
        state.world.p_mobj.mo(dest.unwrap()).y + fixed_mul(24 * FRACUNIT, FINESINE[an as usize]);
    state.world.p_mobj.mo_mut(actor).z = state.world.p_mobj.mo(dest.unwrap()).z;
    set_thing_position(&mut state.world.p_mobj, &mut state.world.p_setup, actor);
}
pub fn vile_target(state: &mut GameState, actor: MobjId) {
    let Some(target) = live_target(&state.world.p_mobj, actor) else {
        return;
    };
    face_target(state, actor);
    let fog: MobjId = spawn_mobj(
        state,
        state.world.p_mobj.mo(target).x,
        state.world.p_mobj.mo(target).x,
        state.world.p_mobj.mo(target).z,
        MobjType::Fire,
    );
    state.world.p_mobj.mo_mut(actor).tracer = Some(fog);
    state.world.p_mobj.mo_mut(fog).target = Some(actor);
    state.world.p_mobj.mo_mut(fog).tracer = state.world.p_mobj.mo(actor).target;
    a_fire(state, fog);
}
pub fn vile_attack(state: &mut GameState, actor: MobjId) {
    let Some(target) = live_target(&state.world.p_mobj, actor) else {
        return;
    };
    face_target(state, actor);
    if !check_sight(state, actor, target) {
        return;
    }
    s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Barexp);
    damage_mobj(state, target, Some(actor), Some(actor), 20);
    state.world.p_mobj.mo_mut(target).momz = (1000 * FRACUNIT
        / state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(target).kind)
            .mass) as Fixed;
    let an: i32 = (state.world.p_mobj.mo(actor).angle >> ANGLETOFINESHIFT) as i32;
    let fire: Option<MobjId> = state
        .world
        .p_mobj
        .mo(actor)
        .tracer
        .filter(|&target| state.world.p_mobj.is_live(target));
    if fire.is_none() {
        return;
    }
    state.world.p_mobj.mo_mut(fire.unwrap()).x =
        state.world.p_mobj.mo(target).x - fixed_mul(24 * FRACUNIT, FINECOSINE[an as usize]);
    state.world.p_mobj.mo_mut(fire.unwrap()).y =
        state.world.p_mobj.mo(target).y - fixed_mul(24 * FRACUNIT, FINESINE[an as usize]);
    p_radius_attack(state, fire.unwrap(), Some(actor), 70);
}
pub const FATSPREAD: i32 = ANG90 / 8;
pub fn fat_raise(state: &mut GameState, actor: MobjId) {
    face_target(state, actor);
    s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Manatk);
}
pub fn fat_attack1(state: &mut GameState, actor: MobjId) {
    face_target(state, actor);
    state.world.p_mobj.mo_mut(actor).angle = state
        .world
        .p_mobj
        .mo(actor)
        .angle
        .wrapping_add(FATSPREAD as Angle);
    let target_subst = state
        .world
        .p_mobj
        .mo(actor)
        .target
        .filter(|&target| state.world.p_mobj.is_live(target));
    let target_id = subst_null_mobj(&mut state.world.p_mobj, target_subst);
    let target: MobjId = target_id;
    spawn_missile(state, actor, target, MobjType::Fatshot);
    let mo: MobjId = spawn_missile(state, actor, target, MobjType::Fatshot);
    state.world.p_mobj.mo_mut(mo).angle = state
        .world
        .p_mobj
        .mo(mo)
        .angle
        .wrapping_add(FATSPREAD as Angle);
    let an: i32 = (state.world.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
    state.world.p_mobj.mo_mut(mo).momx = fixed_mul(
        state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(mo).kind)
            .speed as Fixed,
        FINECOSINE[an as usize],
    );
    state.world.p_mobj.mo_mut(mo).momy = fixed_mul(
        state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(mo).kind)
            .speed as Fixed,
        FINESINE[an as usize],
    );
}
pub fn fat_attack2(state: &mut GameState, actor: MobjId) {
    face_target(state, actor);
    state.world.p_mobj.mo_mut(actor).angle = state
        .world
        .p_mobj
        .mo(actor)
        .angle
        .wrapping_sub(FATSPREAD as Angle);
    let target_subst = state
        .world
        .p_mobj
        .mo(actor)
        .target
        .filter(|&target| state.world.p_mobj.is_live(target));
    let target_id = subst_null_mobj(&mut state.world.p_mobj, target_subst);
    let target: MobjId = target_id;
    spawn_missile(state, actor, target, MobjType::Fatshot);
    let mo: MobjId = spawn_missile(state, actor, target, MobjType::Fatshot);
    state.world.p_mobj.mo_mut(mo).angle = state
        .world
        .p_mobj
        .mo(mo)
        .angle
        .wrapping_sub((FATSPREAD * 2) as Angle);
    let an: i32 = (state.world.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
    state.world.p_mobj.mo_mut(mo).momx = fixed_mul(
        state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(mo).kind)
            .speed as Fixed,
        FINECOSINE[an as usize],
    );
    state.world.p_mobj.mo_mut(mo).momy = fixed_mul(
        state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(mo).kind)
            .speed as Fixed,
        FINESINE[an as usize],
    );
}
pub fn fat_attack3(state: &mut GameState, actor: MobjId) {
    face_target(state, actor);
    let target_subst = state
        .world
        .p_mobj
        .mo(actor)
        .target
        .filter(|&target| state.world.p_mobj.is_live(target));
    let target_id = subst_null_mobj(&mut state.world.p_mobj, target_subst);
    let target: MobjId = target_id;
    let mut mo: MobjId = spawn_missile(state, actor, target, MobjType::Fatshot);
    state.world.p_mobj.mo_mut(mo).angle = state
        .world
        .p_mobj
        .mo(mo)
        .angle
        .wrapping_sub((FATSPREAD / 2) as Angle);
    let mut an: i32 = (state.world.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
    state.world.p_mobj.mo_mut(mo).momx = fixed_mul(
        state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(mo).kind)
            .speed as Fixed,
        FINECOSINE[an as usize],
    );
    state.world.p_mobj.mo_mut(mo).momy = fixed_mul(
        state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(mo).kind)
            .speed as Fixed,
        FINESINE[an as usize],
    );
    mo = spawn_missile(state, actor, target, MobjType::Fatshot);
    state.world.p_mobj.mo_mut(mo).angle = state
        .world
        .p_mobj
        .mo(mo)
        .angle
        .wrapping_add((FATSPREAD / 2) as Angle);
    an = (state.world.p_mobj.mo(mo).angle >> ANGLETOFINESHIFT) as i32;
    state.world.p_mobj.mo_mut(mo).momx = fixed_mul(
        state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(mo).kind)
            .speed as Fixed,
        FINECOSINE[an as usize],
    );
    state.world.p_mobj.mo_mut(mo).momy = fixed_mul(
        state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(mo).kind)
            .speed as Fixed,
        FINESINE[an as usize],
    );
}
pub const SKULLSPEED: i32 = 20 * FRACUNIT;
pub fn skull_attack(state: &mut GameState, actor: MobjId) {
    let dest: MobjId = match state
        .world
        .p_mobj
        .mo(actor)
        .target
        .filter(|&target| state.world.p_mobj.is_live(target))
    {
        Some(dest) => dest,
        None => return,
    };
    state.world.p_mobj.mo_mut(actor).flags |= MobjFlags::SKULLFLY;
    let attacksound = state
        .assets
        .info
        .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
        .attacksound;
    s_start_sound(state, SoundOrigin::Mobj(actor), attacksound);
    face_target(state, actor);
    let an: Angle = state.world.p_mobj.mo(actor).angle >> ANGLETOFINESHIFT;
    state.world.p_mobj.mo_mut(actor).momx = fixed_mul(SKULLSPEED, FINECOSINE[an as usize]);
    state.world.p_mobj.mo_mut(actor).momy = fixed_mul(SKULLSPEED, FINESINE[an as usize]);
    let mut dist: i32 = aprox_distance(
        state.world.p_mobj.mo(dest).x - state.world.p_mobj.mo(actor).x,
        state.world.p_mobj.mo(dest).y - state.world.p_mobj.mo(actor).y,
    );
    dist /= SKULLSPEED;
    if dist < 1 {
        dist = 1;
    }
    state.world.p_mobj.mo_mut(actor).momz = ((state.world.p_mobj.mo(dest).z
        + (state.world.p_mobj.mo(dest).height >> 1)
        - state.world.p_mobj.mo(actor).z)
        / dist) as Fixed;
}
pub fn pain_shoot_skull(state: &mut GameState, actor: MobjId, angle: Angle) {
    let mut count: i32 = 0;
    count += mobj_thinker_ids(&state.world.p_mobj, &state.world.p_tick)
        .into_iter()
        .filter(|&m| state.world.p_mobj.mo(m).kind as u32 == MobjType::Skull as i32 as u32)
        .count() as i32;
    if count > 20 {
        return;
    }
    let an: Angle = angle >> ANGLETOFINESHIFT;
    let prestep: i32 = 4 * FRACUNIT
        + 3 * (state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
            .radius
            + state.assets.info.mobjinfo[MobjType::Skull as usize].radius)
            / 2;
    let x: Fixed =
        state.world.p_mobj.mo(actor).x + fixed_mul(prestep as Fixed, FINECOSINE[an as usize]);
    let y: Fixed =
        state.world.p_mobj.mo(actor).y + fixed_mul(prestep as Fixed, FINESINE[an as usize]);
    let z: Fixed = (state.world.p_mobj.mo(actor).z + 8 * FRACUNIT) as Fixed;
    let newmobj: MobjId = spawn_mobj(state, x, y, z, MobjType::Skull);
    let (new_x, new_y) = {
        let n = state.world.p_mobj.mo(newmobj);
        (n.x, n.y)
    };
    if !try_move(state, newmobj, new_x, new_y) {
        damage_mobj(state, newmobj, Some(actor), Some(actor), 10000);
        return;
    }
    state.world.p_mobj.mo_mut(newmobj).target = state.world.p_mobj.mo(actor).target;
    skull_attack(state, newmobj);
}
pub fn pain_attack(state: &mut GameState, actor: MobjId) {
    if state.world.p_mobj.mo(actor).target.is_none() {
        return;
    }
    face_target(state, actor);
    pain_shoot_skull(state, actor, state.world.p_mobj.mo(actor).angle);
}
pub fn pain_die(state: &mut GameState, actor: MobjId) {
    fall(state, actor);
    pain_shoot_skull(
        state,
        actor,
        state
            .world
            .p_mobj
            .mo(actor)
            .angle
            .wrapping_add(ANG90 as Angle),
    );
    pain_shoot_skull(
        state,
        actor,
        state.world.p_mobj.mo(actor).angle.wrapping_add(ANG180),
    );
    pain_shoot_skull(
        state,
        actor,
        state.world.p_mobj.mo(actor).angle.wrapping_add(ANG270),
    );
}
pub fn scream(state: &mut GameState, actor: MobjId) {
    let deathsound = state
        .assets
        .info
        .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
        .deathsound;
    let sound = match deathsound {
        SfxName::SfxNone => return,
        SfxName::Podth1 | SfxName::Podth2 | SfxName::Podth3 => {
            [SfxName::Podth1, SfxName::Podth2, SfxName::Podth3]
                [(p_random(&mut state.world.m_random) % 3) as usize]
        }
        SfxName::Bgdth1 | SfxName::Bgdth2 => {
            [SfxName::Bgdth1, SfxName::Bgdth2][(p_random(&mut state.world.m_random) % 2) as usize]
        }
        other => other,
    };
    if state.world.p_mobj.mo(actor).kind as u32 == MobjType::Spider as i32 as u32
        || state.world.p_mobj.mo(actor).kind as u32 == MobjType::Cyborg as i32 as u32
    {
        s_start_sound(state, SoundOrigin::None, sound);
    } else {
        s_start_sound(state, SoundOrigin::Mobj(actor), sound);
    }
}
pub fn xscream(state: &mut GameState, actor: MobjId) {
    s_start_sound(state, SoundOrigin::Mobj(actor), SfxName::Slop);
}
pub fn pain(state: &mut GameState, actor: MobjId) {
    let painsound = state
        .assets
        .info
        .mobjinfo_mut(state.world.p_mobj.mo(actor).kind)
        .painsound;
    if painsound != SfxName::SfxNone {
        s_start_sound(state, SoundOrigin::Mobj(actor), painsound);
    }
}
pub fn fall(state: &mut GameState, actor: MobjId) {
    state.world.p_mobj.mo_mut(actor).flags &= !MobjFlags::SOLID;
}
pub fn explode(state: &mut GameState, id: MobjId) {
    let thingy = id;
    let target = state
        .world
        .p_mobj
        .mo(thingy)
        .target
        .filter(|&id| state.world.p_mobj.is_live(id));
    p_radius_attack(state, thingy, target, 128);
}
fn check_boss_end(doomstat: &DoomstatState, g_game: &GGameState, motype: MobjType) -> bool {
    if doomstat.gameversion.is_ultimate_or_higher() {
        match g_game.gameepisode {
            1 => g_game.gamemap == 8 && motype as u32 == MobjType::Bruiser as i32 as u32,
            2 => g_game.gamemap == 8 && motype as u32 == MobjType::Cyborg as i32 as u32,
            3 => g_game.gamemap == 8 && motype as u32 == MobjType::Spider as i32 as u32,
            4 => {
                g_game.gamemap == 6 && motype as u32 == MobjType::Cyborg as i32 as u32
                    || g_game.gamemap == 8 && motype as u32 == MobjType::Spider as i32 as u32
            }
            _ => g_game.gamemap == 8,
        }
    } else {
        if g_game.gamemap != 8 {
            return false;
        }
        if motype as u32 == MobjType::Bruiser as i32 as u32 && g_game.gameepisode != 1 {
            return false;
        }
        true
    }
}
pub fn boss_death(state: &mut GameState, mo: MobjId) {
    if state.game.doomstat.gamemode == GameMode::Commercial {
        if state.game.g_game.gamemap != 7 {
            return;
        }
        if state.world.p_mobj.mo(mo).kind as u32 != MobjType::Fatso as i32 as u32
            && state.world.p_mobj.mo(mo).kind as u32 != MobjType::Baby as i32 as u32
        {
            return;
        }
    } else if !check_boss_end(
        &state.game.doomstat,
        &state.game.g_game,
        state.world.p_mobj.mo(mo).kind,
    ) {
        return;
    }
    let anyone_alive = PlayerId::all()
        .any(|p| state.game.g_game.playeringame[p] && state.game.g_game.players[p].health > 0);
    if !anyone_alive {
        return;
    }
    let mo_type = state.world.p_mobj.mo(mo).kind;
    for mo2 in mobj_thinker_ids(&state.world.p_mobj, &state.world.p_tick) {
        if mo2 != mo
            && state.world.p_mobj.mo(mo2).kind as u32 == mo_type as u32
            && state.world.p_mobj.mo(mo2).health > 0
        {
            return;
        }
    }
    if state.game.doomstat.gamemode == GameMode::Commercial {
        if state.game.g_game.gamemap == 7 {
            if state.world.p_mobj.mo(mo).kind as u32 == MobjType::Fatso as i32 as u32 {
                let junk = state.world.p_setup.junk_line(666_i16);
                do_floor(state, junk, FloorE::LowerFloorToLowest);
                return;
            }
            if state.world.p_mobj.mo(mo).kind as u32 == MobjType::Baby as i32 as u32 {
                let junk = state.world.p_setup.junk_line(667_i16);
                do_floor(state, junk, FloorE::RaiseToTexture);
                return;
            }
        }
    } else {
        match state.game.g_game.gameepisode {
            1 => {
                let junk = state.world.p_setup.junk_line(666_i16);
                do_floor(state, junk, FloorE::LowerFloorToLowest);
                return;
            }
            4 => match state.game.g_game.gamemap {
                6 => {
                    let junk = state.world.p_setup.junk_line(666_i16);
                    do_door(state, junk, VldoorE::BlazeOpen);
                    return;
                }
                8 => {
                    let junk = state.world.p_setup.junk_line(666_i16);
                    do_floor(state, junk, FloorE::LowerFloorToLowest);
                    return;
                }
                _ => {}
            },
            _ => {}
        }
    }
    exit_level(&mut state.game.g_game);
}
pub fn hoof(state: &mut GameState, mo: MobjId) {
    s_start_sound(state, SoundOrigin::Mobj(mo), SfxName::Hoof);
    chase(state, mo);
}
pub fn metal(state: &mut GameState, mo: MobjId) {
    s_start_sound(state, SoundOrigin::Mobj(mo), SfxName::Metal);
    chase(state, mo);
}
pub fn baby_metal(state: &mut GameState, mo: MobjId) {
    s_start_sound(state, SoundOrigin::Mobj(mo), SfxName::Bspwlk);
    chase(state, mo);
}
pub fn open_shotgun2(state: &mut GameState, player: PlayerId, _position: i32) {
    s_start_sound(
        state,
        SoundOrigin::Mobj(state.game.g_game.players[player].mo.unwrap()),
        SfxName::Dbopn,
    );
}
pub fn load_shotgun2(state: &mut GameState, player: PlayerId, _position: i32) {
    s_start_sound(
        state,
        SoundOrigin::Mobj(state.game.g_game.players[player].mo.unwrap()),
        SfxName::Dbload,
    );
}
pub fn close_shotgun2(state: &mut GameState, player: PlayerId, position: i32) {
    s_start_sound(
        state,
        SoundOrigin::Mobj(state.game.g_game.players[player].mo.unwrap()),
        SfxName::Dbcls,
    );
    re_fire(state, player, position);
}
pub fn brain_awake(state: &mut GameState, _id: MobjId) {
    state.world.p_enemy.numbraintargets = 0;
    state.world.p_enemy.braintargeton = 0;
    for m in mobj_thinker_ids(&state.world.p_mobj, &state.world.p_tick) {
        if state.world.p_mobj.mo(m).kind as u32 == MobjType::Bosstarget as i32 as u32 {
            let n = state.world.p_enemy.numbraintargets as usize;
            state.world.p_enemy.braintargets[n] = Some(m);
            state.world.p_enemy.numbraintargets += 1;
        }
    }
    s_start_sound(state, SoundOrigin::None, SfxName::Bossit);
}
pub fn brain_pain(state: &mut GameState, _id: MobjId) {
    s_start_sound(state, SoundOrigin::None, SfxName::Bospn);
}
pub fn brain_scream(state: &mut GameState, mo: MobjId) {
    let mut x: i32 = state.world.p_mobj.mo(mo).x - 196 * FRACUNIT;
    while x < state.world.p_mobj.mo(mo).x + 320 * FRACUNIT {
        let y: i32 = state.world.p_mobj.mo(mo).y - 320 * FRACUNIT;
        let z: i32 = 128 + p_random(&mut state.world.m_random) * 2 * FRACUNIT;
        let th: MobjId = spawn_mobj(state, x as Fixed, y as Fixed, z as Fixed, MobjType::Rocket);
        state.world.p_mobj.mo_mut(th).momz = (p_random(&mut state.world.m_random) * 512) as Fixed;
        set_mobj_state(state, th, StateNum::Brainexplode1);
        state.world.p_mobj.mo_mut(th).tics -= p_random(&mut state.world.m_random) & 7;
        if state.world.p_mobj.mo(th).tics < 1 {
            state.world.p_mobj.mo_mut(th).tics = 1;
        }
        x += FRACUNIT * 8;
    }
    s_start_sound(state, SoundOrigin::None, SfxName::Bosdth);
}
pub fn brain_explode(state: &mut GameState, mo: MobjId) {
    let x: i32 = state.world.p_mobj.mo(mo).x
        + (p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) * 2048;
    let y: i32 = state.world.p_mobj.mo(mo).y;
    let z: i32 = 128 + p_random(&mut state.world.m_random) * 2 * FRACUNIT;
    let th: MobjId = spawn_mobj(state, x as Fixed, y as Fixed, z as Fixed, MobjType::Rocket);
    state.world.p_mobj.mo_mut(th).momz = (p_random(&mut state.world.m_random) * 512) as Fixed;
    set_mobj_state(state, th, StateNum::Brainexplode1);
    state.world.p_mobj.mo_mut(th).tics -= p_random(&mut state.world.m_random) & 7;
    if state.world.p_mobj.mo(th).tics < 1 {
        state.world.p_mobj.mo_mut(th).tics = 1;
    }
}
pub fn brain_die(state: &mut GameState, _id: MobjId) {
    exit_level(&mut state.game.g_game);
}
pub fn brain_spit(state: &mut GameState, mo: MobjId) {
    state.world.p_enemy.easy ^= 1;
    if state.game.g_game.gameskill <= SkillType::Easy && state.world.p_enemy.easy == 0 {
        return;
    }
    let targ_id =
        state.world.p_enemy.braintargets[state.world.p_enemy.braintargeton as usize].unwrap();
    let targ: MobjId = targ_id;
    state.world.p_enemy.braintargeton =
        (state.world.p_enemy.braintargeton + 1) % state.world.p_enemy.numbraintargets;
    let newmobj: MobjId = spawn_missile(state, mo, targ, MobjType::Spawnshot);
    state.world.p_mobj.mo_mut(newmobj).target = Some(targ);
    state.world.p_mobj.mo_mut(newmobj).reactiontime = (state.world.p_mobj.mo(targ).y
        - state.world.p_mobj.mo(mo).y)
        / state.world.p_mobj.mo(newmobj).momy
        / state
            .assets
            .info
            .state_mut(state.world.p_mobj.mo(newmobj).state.unwrap())
            .tics;
    s_start_sound(state, SoundOrigin::None, SfxName::Bospit);
}
pub fn spawn_sound(state: &mut GameState, mo: MobjId) {
    s_start_sound(state, SoundOrigin::Mobj(mo), SfxName::Boscub);
    spawn_fly(state, mo);
}
pub fn spawn_fly(state: &mut GameState, mo: MobjId) {
    state.world.p_mobj.mo_mut(mo).reactiontime -= 1;
    if state.world.p_mobj.mo(mo).reactiontime != 0 {
        return;
    }
    let targ_subst = state
        .world
        .p_mobj
        .mo(mo)
        .target
        .filter(|&mo| state.world.p_mobj.is_live(mo));
    let targ_id = subst_null_mobj(&mut state.world.p_mobj, targ_subst);
    let targ: MobjId = targ_id;
    let fog: MobjId = spawn_mobj(
        state,
        state.world.p_mobj.mo(targ).x,
        state.world.p_mobj.mo(targ).y,
        state.world.p_mobj.mo(targ).z,
        MobjType::Spawnfire,
    );
    s_start_sound(state, SoundOrigin::Mobj(fog), SfxName::Telept);
    let r: i32 = p_random(&mut state.world.m_random);
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
        state.world.p_mobj.mo(targ).x,
        state.world.p_mobj.mo(targ).y,
        state.world.p_mobj.mo(targ).z,
        kind,
    );
    if look_for_players(state, newmobj, true) {
        let seestate = state
            .assets
            .info
            .mobjinfo_mut(state.world.p_mobj.mo(newmobj).kind)
            .seestate;
        set_mobj_state(state, newmobj, seestate);
    }
    teleport_move(
        state,
        newmobj,
        state.world.p_mobj.mo(newmobj).x,
        state.world.p_mobj.mo(newmobj).y,
    );
    remove_mobj(state, mo);
}
pub fn player_scream(state: &mut GameState, mo: MobjId) {
    let mut sound: SfxName = SfxName::Pldeth;
    if state.game.doomstat.gamemode == GameMode::Commercial
        && state.world.p_mobj.mo(mo).health < -50
    {
        sound = SfxName::Pdiehi;
    }
    s_start_sound(state, SoundOrigin::Mobj(mo), sound);
}
