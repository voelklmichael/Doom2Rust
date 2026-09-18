use crate::game_state::GameState;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACUNIT;
use crate::m_fixed::INT_MAX;
use crate::p_map::P_ChangeSector;
use crate::p_mobj::sector_t;
use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::floormove_t;
use crate::p_spec::getSector;
use crate::p_spec::getSide;
use crate::p_spec::twoSided;
use crate::p_spec::FloorId;
use crate::p_spec::P_FindHighestFloorSurrounding;
use crate::p_spec::P_FindLowestCeilingSurrounding;
use crate::p_spec::P_FindLowestFloorSurrounding;
use crate::p_spec::P_FindNextHighestFloor;
use crate::p_spec::P_FindSectorFromLineTag;
use crate::p_spec::ML_TWOSIDED;
use crate::p_tick::P_AddThinker;
use crate::p_tick::P_RemoveThinker;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::{sfx_pstop, sfx_stnmov};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FloorE {
    lowerFloor = 0,
    lowerFloorToLowest = 1,
    turboLower = 2,
    raiseFloor = 3,
    raiseFloorToNearest = 4,
    raiseToTexture = 5,
    lowerAndChange = 6,
    raiseFloor24 = 7,
    raiseFloor24AndChange = 8,
    raiseFloorCrush = 9,
    raiseFloorTurbo = 10,
    donutRaise = 11,
    raiseFloor512 = 12,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StairE {
    build8 = 0,
    turbo16 = 1,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ResultE {
    ok = 0,
    crushed = 1,
    pastdest = 2,
}
pub const FLOORSPEED: i32 = FRACUNIT;
fn change_sector(state: &mut GameState, sector: SectorId, crush: bool) -> bool {
    let sector_ptr: *mut sector_t = state.p_setup.sector_mut(sector);
    unsafe { P_ChangeSector(state, sector_ptr, crush) }
}
pub fn T_MovePlane(
    state: &mut GameState,
    sector: SectorId,
    mut speed: fixed_t,
    mut dest: fixed_t,
    mut crush: bool,
    mut floorOrCeiling: i32,
    mut direction: i32,
) -> ResultE {
    let mut flag: bool;
    let mut lastpos: fixed_t = 0;
    match floorOrCeiling {
        0 => match direction {
            -1 => {
                if state.p_setup.sector_mut(sector).floorheight - speed < dest {
                    lastpos = state.p_setup.sector_mut(sector).floorheight;
                    state.p_setup.sector_mut(sector).floorheight = dest;
                    flag = change_sector(state, sector, crush);
                    if flag {
                        state.p_setup.sector_mut(sector).floorheight = lastpos;
                        change_sector(state, sector, crush);
                    }
                    return ResultE::pastdest;
                } else {
                    lastpos = state.p_setup.sector_mut(sector).floorheight;
                    state.p_setup.sector_mut(sector).floorheight -= speed;
                    flag = change_sector(state, sector, crush);
                    if flag {
                        state.p_setup.sector_mut(sector).floorheight = lastpos;
                        change_sector(state, sector, crush);
                        return ResultE::crushed;
                    }
                }
            }
            1 => {
                if state.p_setup.sector_mut(sector).floorheight + speed > dest {
                    lastpos = state.p_setup.sector_mut(sector).floorheight;
                    state.p_setup.sector_mut(sector).floorheight = dest;
                    flag = change_sector(state, sector, crush);
                    if flag {
                        state.p_setup.sector_mut(sector).floorheight = lastpos;
                        change_sector(state, sector, crush);
                    }
                    return ResultE::pastdest;
                } else {
                    lastpos = state.p_setup.sector_mut(sector).floorheight;
                    state.p_setup.sector_mut(sector).floorheight += speed;
                    flag = change_sector(state, sector, crush);
                    if flag {
                        if crush {
                            return ResultE::crushed;
                        }
                        state.p_setup.sector_mut(sector).floorheight = lastpos;
                        change_sector(state, sector, crush);
                        return ResultE::crushed;
                    }
                }
            }
            _ => {}
        },
        1 => match direction {
            -1 => {
                if state.p_setup.sector_mut(sector).ceilingheight - speed < dest {
                    lastpos = state.p_setup.sector_mut(sector).ceilingheight;
                    state.p_setup.sector_mut(sector).ceilingheight = dest;
                    flag = change_sector(state, sector, crush);
                    if flag {
                        state.p_setup.sector_mut(sector).ceilingheight = lastpos;
                        change_sector(state, sector, crush);
                    }
                    return ResultE::pastdest;
                } else {
                    lastpos = state.p_setup.sector_mut(sector).ceilingheight;
                    state.p_setup.sector_mut(sector).ceilingheight -= speed;
                    flag = change_sector(state, sector, crush);
                    if flag {
                        if crush {
                            return ResultE::crushed;
                        }
                        state.p_setup.sector_mut(sector).ceilingheight = lastpos;
                        change_sector(state, sector, crush);
                        return ResultE::crushed;
                    }
                }
            }
            1 => {
                if state.p_setup.sector_mut(sector).ceilingheight + speed > dest {
                    lastpos = state.p_setup.sector_mut(sector).ceilingheight;
                    state.p_setup.sector_mut(sector).ceilingheight = dest;
                    flag = change_sector(state, sector, crush);
                    if flag {
                        state.p_setup.sector_mut(sector).ceilingheight = lastpos;
                        change_sector(state, sector, crush);
                    }
                    return ResultE::pastdest;
                } else {
                    lastpos = state.p_setup.sector_mut(sector).ceilingheight;
                    state.p_setup.sector_mut(sector).ceilingheight += speed;
                    flag = change_sector(state, sector, crush);
                }
            }
            _ => {}
        },
        _ => {}
    }
    ResultE::ok
}
pub fn T_MoveFloor(state: &mut GameState, id: FloorId) {
    let floor = *state
        .p_spec
        .get_floor_ref(id)
        .expect("ThinkerFn::Floor id must reference a live floor");
    let res = T_MovePlane(
        state,
        floor.sector,
        floor.speed,
        floor.floordestheight,
        floor.crush,
        0_i32,
        floor.direction,
    );
    if state.p_tick.leveltime & 7_i32 == 0 {
        S_StartSound(state, SoundOrigin::Sector(floor.sector), sfx_stnmov as i32);
    }
    if res == ResultE::pastdest {
        let sec = state.p_setup.sector_mut(floor.sector);
        sec.specialdata = None;
        if floor.direction == 1_i32 {
            if floor.type_0 == FloorE::donutRaise {
                sec.special = floor.newspecial as i16;
                sec.floorpic = floor.texture;
            }
        } else if floor.direction == -1_i32 && floor.type_0 == FloorE::lowerAndChange {
            sec.special = floor.newspecial as i16;
            sec.floorpic = floor.texture;
        }
        P_RemoveThinker(
            &mut state
                .p_spec
                .get_floor_mut(id)
                .expect("live floor")
                .thinker,
        );
        S_StartSound(state, SoundOrigin::Sector(floor.sector), sfx_pstop as i32);
    }
}
pub fn EV_DoFloor(state: &mut GameState, line: LineId, floortype: FloorE) -> i32 {
    let mut rtn: i32 = 0;
    let mut secnum: i32 = -1_i32;
    loop {
        secnum = P_FindSectorFromLineTag(state, line, secnum);
        if secnum < 0_i32 {
            break;
        }
        let sec = SectorId(secnum as u32);
        if state.p_setup.sector_mut(sec).specialdata.is_some() {
            continue;
        }
        rtn = 1_i32;
        let mut floor = floormove_t::default();
        floor.thinker.function = ThinkerFn::Floor(T_MoveFloor);
        floor.type_0 = floortype;
        floor.crush = false;
        let (floorheight, ceilingheight, linecount) = {
            let s = state.p_setup.sector_mut(sec);
            (s.floorheight, s.ceilingheight, s.linecount)
        };
        let mut raise_lowest_ceiling = false;
        match floortype {
            FloorE::lowerFloor => {
                floor.direction = -1_i32;
                floor.sector = sec;
                floor.speed = FLOORSPEED as fixed_t;
                floor.floordestheight = P_FindHighestFloorSurrounding(state, sec);
            }
            FloorE::lowerFloorToLowest => {
                floor.direction = -1_i32;
                floor.sector = sec;
                floor.speed = FLOORSPEED as fixed_t;
                floor.floordestheight = P_FindLowestFloorSurrounding(state, sec);
            }
            FloorE::turboLower => {
                floor.direction = -1_i32;
                floor.sector = sec;
                floor.speed = (FLOORSPEED * 4_i32) as fixed_t;
                floor.floordestheight = P_FindHighestFloorSurrounding(state, sec);
                if floor.floordestheight != floorheight {
                    floor.floordestheight += 8_i32 * FRACUNIT;
                }
            }
            FloorE::raiseFloorCrush => {
                floor.crush = true;
                raise_lowest_ceiling = true;
            }
            FloorE::raiseFloor => {
                raise_lowest_ceiling = true;
            }
            FloorE::raiseFloorTurbo => {
                floor.direction = 1_i32;
                floor.sector = sec;
                floor.speed = (FLOORSPEED * 4_i32) as fixed_t;
                floor.floordestheight = P_FindNextHighestFloor(state, sec, floorheight);
            }
            FloorE::raiseFloorToNearest => {
                floor.direction = 1_i32;
                floor.sector = sec;
                floor.speed = FLOORSPEED as fixed_t;
                floor.floordestheight = P_FindNextHighestFloor(state, sec, floorheight);
            }
            FloorE::raiseFloor24 => {
                floor.direction = 1_i32;
                floor.sector = sec;
                floor.speed = FLOORSPEED as fixed_t;
                floor.floordestheight = (floorheight + 24_i32 * FRACUNIT) as fixed_t;
            }
            FloorE::raiseFloor512 => {
                floor.direction = 1_i32;
                floor.sector = sec;
                floor.speed = FLOORSPEED as fixed_t;
                floor.floordestheight = (floorheight + 512_i32 * FRACUNIT) as fixed_t;
            }
            FloorE::raiseFloor24AndChange => {
                floor.direction = 1_i32;
                floor.sector = sec;
                floor.speed = FLOORSPEED as fixed_t;
                floor.floordestheight = (floorheight + 24_i32 * FRACUNIT) as fixed_t;
                let front = state.p_setup.line(line).frontsector.unwrap();
                let (front_pic, front_special) = {
                    let fsec = state.p_setup.sector_mut(front);
                    (fsec.floorpic, fsec.special)
                };
                let s = state.p_setup.sector_mut(sec);
                s.floorpic = front_pic;
                s.special = front_special;
            }
            FloorE::raiseToTexture => {
                let mut minsize: i32 = INT_MAX;
                floor.direction = 1_i32;
                floor.sector = sec;
                floor.speed = FLOORSPEED as fixed_t;
                for i in 0..linecount {
                    if twoSided(state, secnum, i) != 0 {
                        for side_index in 0..2_i32 {
                            let side = getSide(state, secnum, i, side_index);
                            let bottomtexture = state.p_setup.side_mut(side).bottomtexture;
                            if bottomtexture as i32 >= 0_i32
                                && state.r_data.textureheight[bottomtexture as usize] < minsize
                            {
                                minsize = state.r_data.textureheight[bottomtexture as usize];
                            }
                        }
                    }
                }
                floor.floordestheight = (floorheight + minsize) as fixed_t;
            }
            FloorE::lowerAndChange => {
                floor.direction = -1_i32;
                floor.sector = sec;
                floor.speed = FLOORSPEED as fixed_t;
                floor.floordestheight = P_FindLowestFloorSurrounding(state, sec);
                floor.texture = state.p_setup.sector_mut(sec).floorpic;
                for i in 0..linecount {
                    if twoSided(state, secnum, i) != 0 {
                        let side0 = getSide(state, secnum, i, 0_i32);
                        let side0_sector = state.p_setup.side_mut(side0).sector;
                        let other = if side0_sector.0 == secnum as u32 {
                            getSector(state, secnum, i, 1_i32)
                        } else {
                            getSector(state, secnum, i, 0_i32)
                        };
                        let (other_floor, other_pic, other_special) = {
                            let o = state.p_setup.sector_mut(other);
                            (o.floorheight, o.floorpic, o.special)
                        };
                        if other_floor == floor.floordestheight {
                            floor.texture = other_pic;
                            floor.newspecial = other_special as i32;
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
        if raise_lowest_ceiling {
            floor.direction = 1_i32;
            floor.sector = sec;
            floor.speed = FLOORSPEED as fixed_t;
            floor.floordestheight = P_FindLowestCeilingSurrounding(state, sec);
            if floor.floordestheight > ceilingheight {
                floor.floordestheight = ceilingheight;
            }
            floor.floordestheight -=
                8_i32 * FRACUNIT * (floortype == FloorE::raiseFloorCrush) as i32;
        }
        let (floor_arena_id, _) = state.p_spec.spawn_floor(floor);
        let floor_id = P_AddThinker(state, ThinkerPayload::Floor(floor_arena_id), ThinkerKind::Floor);
        state.p_setup.sector_mut(sec).specialdata = Some(SectorSpecial::Floor(floor_id));
    }
    rtn
}
fn spawn_stair(state: &mut GameState, sec: SectorId, speed: fixed_t, height: i32) {
    let mut floor = floormove_t::default();
    floor.thinker.function = ThinkerFn::Floor(T_MoveFloor);
    floor.direction = 1_i32;
    floor.sector = sec;
    floor.speed = speed;
    floor.floordestheight = height as fixed_t;
    let (floor_arena_id, _) = state.p_spec.spawn_floor(floor);
    let floor_id = P_AddThinker(state, ThinkerPayload::Floor(floor_arena_id), ThinkerKind::Floor);
    state.p_setup.sector_mut(sec).specialdata = Some(SectorSpecial::Floor(floor_id));
}
pub fn EV_BuildStairs(state: &mut GameState, line: LineId, type_0: StairE) -> i32 {
    let mut rtn: i32 = 0;
    let mut secnum: i32 = -1_i32;
    loop {
        secnum = P_FindSectorFromLineTag(state, line, secnum);
        if secnum < 0_i32 {
            break;
        }
        let mut sec = SectorId(secnum as u32);
        if state.p_setup.sector_mut(sec).specialdata.is_some() {
            continue;
        }
        rtn = 1_i32;
        let (speed, stairsize): (fixed_t, fixed_t) = match type_0 {
            StairE::build8 => (
                (FLOORSPEED / 4_i32) as fixed_t,
                (8_i32 * FRACUNIT) as fixed_t,
            ),
            StairE::turbo16 => (
                (FLOORSPEED * 4_i32) as fixed_t,
                (16_i32 * FRACUNIT) as fixed_t,
            ),
        };
        let mut height: i32 = state.p_setup.sector_mut(sec).floorheight + stairsize;
        let texture = state.p_setup.sector_mut(sec).floorpic as i32;
        spawn_stair(state, sec, speed, height);
        loop {
            let mut found = false;
            let linecount = state.p_setup.sector_mut(sec).linecount;
            for i in 0..linecount {
                let line_id = state.p_setup.sector_mut(sec).lines[i as usize];
                let iline = state.p_setup.line(line_id);
                if iline.flags as i32 & ML_TWOSIDED != 0 {
                    let front_id = iline.frontsector.unwrap();
                    if secnum == front_id.0 as i32 {
                        let back_id = iline.backsector.unwrap();
                        let (back_pic, back_free) = {
                            let tsec = state.p_setup.sector_mut(back_id);
                            (tsec.floorpic as i32, tsec.specialdata.is_none())
                        };
                        if back_pic == texture {
                            height += stairsize;
                            if back_free {
                                sec = back_id;
                                secnum = back_id.0 as i32;
                                spawn_stair(state, sec, speed, height);
                                found = true;
                                break;
                            }
                        }
                    }
                }
            }
            if !found {
                break;
            }
        }
    }
    rtn
}
