use crate::game_state::GameState;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACUNIT;
use crate::m_fixed::INT_MAX;
use crate::p_map::P_ChangeSector;
use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_mobj::sector_t;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::floormove_t;
use crate::p_spec::getSector;
use crate::p_spec::getSide;
use crate::p_spec::twoSided;
use crate::p_spec::P_FindHighestFloorSurrounding;
use crate::p_spec::P_FindLowestCeilingSurrounding;
use crate::p_spec::P_FindLowestFloorSurrounding;
use crate::p_spec::P_FindNextHighestFloor;
use crate::p_spec::P_FindSectorFromLineTag;
use crate::p_spec::ML_TWOSIDED;
use crate::p_tick::P_AddThinker;
use crate::p_tick::ThinkerKind;
use crate::p_tick::P_RemoveThinker;
use crate::r_defs::side_t;
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
pub unsafe fn T_MovePlane(
    state: &mut GameState,
    mut sector: *mut sector_t,
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
                if (*sector).floorheight - speed < dest {
                    lastpos = (*sector).floorheight;
                    (*sector).floorheight = dest;
                    flag = P_ChangeSector(state, sector, crush);
                    if flag {
                        (*sector).floorheight = lastpos;
                        P_ChangeSector(state, sector, crush);
                    }
                    return ResultE::pastdest;
                } else {
                    lastpos = (*sector).floorheight;
                    (*sector).floorheight -= speed;
                    flag = P_ChangeSector(state, sector, crush);
                    if flag {
                        (*sector).floorheight = lastpos;
                        P_ChangeSector(state, sector, crush);
                        return ResultE::crushed;
                    }
                }
            }
            1 => {
                if (*sector).floorheight + speed > dest {
                    lastpos = (*sector).floorheight;
                    (*sector).floorheight = dest;
                    flag = P_ChangeSector(state, sector, crush);
                    if flag {
                        (*sector).floorheight = lastpos;
                        P_ChangeSector(state, sector, crush);
                    }
                    return ResultE::pastdest;
                } else {
                    lastpos = (*sector).floorheight;
                    (*sector).floorheight += speed;
                    flag = P_ChangeSector(state, sector, crush);
                    if flag {
                        if crush {
                            return ResultE::crushed;
                        }
                        (*sector).floorheight = lastpos;
                        P_ChangeSector(state, sector, crush);
                        return ResultE::crushed;
                    }
                }
            }
            _ => {}
        },
        1 => match direction {
            -1 => {
                if (*sector).ceilingheight - speed < dest {
                    lastpos = (*sector).ceilingheight;
                    (*sector).ceilingheight = dest;
                    flag = P_ChangeSector(state, sector, crush);
                    if flag {
                        (*sector).ceilingheight = lastpos;
                        P_ChangeSector(state, sector, crush);
                    }
                    return ResultE::pastdest;
                } else {
                    lastpos = (*sector).ceilingheight;
                    (*sector).ceilingheight -= speed;
                    flag = P_ChangeSector(state, sector, crush);
                    if flag {
                        if crush {
                            return ResultE::crushed;
                        }
                        (*sector).ceilingheight = lastpos;
                        P_ChangeSector(state, sector, crush);
                        return ResultE::crushed;
                    }
                }
            }
            1 => {
                if (*sector).ceilingheight + speed > dest {
                    lastpos = (*sector).ceilingheight;
                    (*sector).ceilingheight = dest;
                    flag = P_ChangeSector(state, sector, crush);
                    if flag {
                        (*sector).ceilingheight = lastpos;
                        P_ChangeSector(state, sector, crush);
                    }
                    return ResultE::pastdest;
                } else {
                    lastpos = (*sector).ceilingheight;
                    (*sector).ceilingheight += speed;
                    flag = P_ChangeSector(state, sector, crush);
                }
            }
            _ => {}
        },
        _ => {}
    }
    return ResultE::ok;
}
pub unsafe fn T_MoveFloor(state: &mut GameState, mut floor: *mut floormove_t) {
    let mut res: ResultE = ResultE::ok;
    let sec: *mut sector_t = state.p_setup.sector_mut((*floor).sector);
    res = T_MovePlane(
        state,
        sec,
        (*floor).speed,
        (*floor).floordestheight,
        (*floor).crush,
        0 as i32,
        (*floor).direction,
    );
    if state.p_tick.leveltime & 7 as i32 == 0 {
        S_StartSound(state, SoundOrigin::Sector((*floor).sector), sfx_stnmov as i32);
    }
    if res == ResultE::pastdest {
        (*sec).specialdata = None;
        if (*floor).direction == 1 as i32 {
            match (*floor).type_0 {
                FloorE::donutRaise => {
                    (*sec).special = (*floor).newspecial as i16;
                    (*sec).floorpic = (*floor).texture;
                }
                _ => {}
            }
        } else if (*floor).direction == -(1 as i32) {
            match (*floor).type_0 {
                FloorE::lowerAndChange => {
                    (*sec).special = (*floor).newspecial as i16;
                    (*sec).floorpic = (*floor).texture;
                }
                _ => {}
            }
        }
        P_RemoveThinker(&raw mut (*floor).thinker);
        S_StartSound(state, SoundOrigin::Sector((*floor).sector), sfx_pstop as i32);
    }
}
pub unsafe fn EV_DoFloor(
    state: &mut GameState,
    mut line: LineId,
    mut floortype: FloorE,
) -> i32 {
    let mut secnum: i32 = 0;
    let mut rtn: i32 = 0;
    let mut i: i32 = 0;
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut floor: *mut floormove_t = ::core::ptr::null_mut::<floormove_t>();
    secnum = -(1 as i32);
    rtn = 0 as i32;
    loop {
        secnum = P_FindSectorFromLineTag(state, line, secnum);
        if !(secnum >= 0 as i32) {
            break;
        }
        sec = state.p_setup.sector_mut(SectorId(secnum as u32));
        if (*sec).specialdata.is_some() {
            continue;
        }
        rtn = 1 as i32;
        floor = state.p_spec.spawn_floor(floormove_t::default());
        let floor_id = P_AddThinker(state, &raw mut (*floor).thinker, ThinkerKind::Floor);
        (*sec).specialdata = Some(SectorSpecial::Floor(floor_id));
        (*floor).thinker.function = ThinkerFn::Floor(T_MoveFloor);
        (*floor).type_0 = floortype;
        (*floor).crush = false;
        let mut current_block_84: u64;
        match floortype {
            FloorE::lowerFloor => {
                (*floor).direction = -(1 as i32);
                (*floor).sector = SectorId(secnum as u32);
                (*floor).speed = FLOORSPEED as fixed_t;
                (*floor).floordestheight = P_FindHighestFloorSurrounding(state, sec);
                current_block_84 = 15514718523126015390;
            }
            FloorE::lowerFloorToLowest => {
                (*floor).direction = -(1 as i32);
                (*floor).sector = SectorId(secnum as u32);
                (*floor).speed = FLOORSPEED as fixed_t;
                (*floor).floordestheight = P_FindLowestFloorSurrounding(state, sec);
                current_block_84 = 15514718523126015390;
            }
            FloorE::turboLower => {
                (*floor).direction = -(1 as i32);
                (*floor).sector = SectorId(secnum as u32);
                (*floor).speed = (FLOORSPEED * 4 as i32) as fixed_t;
                (*floor).floordestheight = P_FindHighestFloorSurrounding(state, sec);
                if (*floor).floordestheight != (*sec).floorheight {
                    (*floor).floordestheight += 8 as i32 * FRACUNIT;
                }
                current_block_84 = 15514718523126015390;
            }
            FloorE::raiseFloorCrush => {
                (*floor).crush = true;
                current_block_84 = 7690836263840410806;
            }
            FloorE::raiseFloor => {
                current_block_84 = 7690836263840410806;
            }
            FloorE::raiseFloorTurbo => {
                (*floor).direction = 1 as i32;
                (*floor).sector = SectorId(secnum as u32);
                (*floor).speed = (FLOORSPEED * 4 as i32) as fixed_t;
                (*floor).floordestheight = P_FindNextHighestFloor(state, sec, (*sec).floorheight as i32);
                current_block_84 = 15514718523126015390;
            }
            FloorE::raiseFloorToNearest => {
                (*floor).direction = 1 as i32;
                (*floor).sector = SectorId(secnum as u32);
                (*floor).speed = FLOORSPEED as fixed_t;
                (*floor).floordestheight = P_FindNextHighestFloor(state, sec, (*sec).floorheight as i32);
                current_block_84 = 15514718523126015390;
            }
            FloorE::raiseFloor24 => {
                (*floor).direction = 1 as i32;
                (*floor).sector = SectorId(secnum as u32);
                (*floor).speed = FLOORSPEED as fixed_t;
                (*floor).floordestheight =
                    ((*sec).floorheight as i32 + 24 as i32 * FRACUNIT) as fixed_t;
                current_block_84 = 15514718523126015390;
            }
            FloorE::raiseFloor512 => {
                (*floor).direction = 1 as i32;
                (*floor).sector = SectorId(secnum as u32);
                (*floor).speed = FLOORSPEED as fixed_t;
                (*floor).floordestheight =
                    ((*sec).floorheight as i32 + 512 as i32 * FRACUNIT) as fixed_t;
                current_block_84 = 15514718523126015390;
            }
            FloorE::raiseFloor24AndChange => {
                (*floor).direction = 1 as i32;
                (*floor).sector = SectorId(secnum as u32);
                (*floor).speed = FLOORSPEED as fixed_t;
                (*floor).floordestheight =
                    ((*sec).floorheight as i32 + 24 as i32 * FRACUNIT) as fixed_t;
                let fsec = state.p_setup.sector_mut(state.p_setup.line(line).frontsector.unwrap());
                (*sec).floorpic = (*fsec).floorpic;
                (*sec).special = (*fsec).special;
                current_block_84 = 15514718523126015390;
            }
            FloorE::raiseToTexture => {
                let mut minsize: i32 = INT_MAX;
                let mut side: *mut side_t = ::core::ptr::null_mut::<side_t>();
                (*floor).direction = 1 as i32;
                (*floor).sector = SectorId(secnum as u32);
                (*floor).speed = FLOORSPEED as fixed_t;
                i = 0 as i32;
                while i < (*sec).linecount {
                    if twoSided(state, secnum, i) != 0 {
                        side = getSide(state, secnum, i, 0 as i32);
                        if (*side).bottomtexture as i32 >= 0 as i32 {
                            if state.r_data.textureheight[(*side).bottomtexture as usize] < minsize
                            {
                                minsize =
                                    state.r_data.textureheight[(*side).bottomtexture as usize]
                                        as i32;
                            }
                        }
                        side = getSide(state, secnum, i, 1 as i32);
                        if (*side).bottomtexture as i32 >= 0 as i32 {
                            if state.r_data.textureheight[(*side).bottomtexture as usize] < minsize
                            {
                                minsize =
                                    state.r_data.textureheight[(*side).bottomtexture as usize]
                                        as i32;
                            }
                        }
                    }
                    i += 1;
                }
                (*floor).floordestheight = ((*sec).floorheight as i32 + minsize) as fixed_t;
                current_block_84 = 15514718523126015390;
            }
            FloorE::lowerAndChange => {
                (*floor).direction = -(1 as i32);
                (*floor).sector = SectorId(secnum as u32);
                (*floor).speed = FLOORSPEED as fixed_t;
                (*floor).floordestheight = P_FindLowestFloorSurrounding(state, sec);
                (*floor).texture = (*sec).floorpic;
                i = 0 as i32;
                while i < (*sec).linecount {
                    if twoSided(state, secnum, i) != 0 {
                        if (*getSide(state, secnum, i, 0 as i32)).sector.0 == secnum as u32 {
                            sec = getSector(state, secnum, i, 1 as i32);
                            if (*sec).floorheight == (*floor).floordestheight {
                                (*floor).texture = (*sec).floorpic;
                                (*floor).newspecial = (*sec).special as i32;
                                break;
                            }
                        } else {
                            sec = getSector(state, secnum, i, 0 as i32);
                            if (*sec).floorheight == (*floor).floordestheight {
                                (*floor).texture = (*sec).floorpic;
                                (*floor).newspecial = (*sec).special as i32;
                                break;
                            }
                        }
                    }
                    i += 1;
                }
                current_block_84 = 15514718523126015390;
            }
            _ => {
                current_block_84 = 15514718523126015390;
            }
        }
        match current_block_84 {
            7690836263840410806 => {
                (*floor).direction = 1 as i32;
                (*floor).sector = SectorId(secnum as u32);
                (*floor).speed = FLOORSPEED as fixed_t;
                (*floor).floordestheight = P_FindLowestCeilingSurrounding(state, sec);
                if (*floor).floordestheight > (*sec).ceilingheight {
                    (*floor).floordestheight = (*sec).ceilingheight;
                }
                (*floor).floordestheight -= 8 as i32
                    * FRACUNIT
                    * (floortype == FloorE::raiseFloorCrush) as i32;
            }
            _ => {}
        }
    }
    return rtn;
}
pub unsafe fn EV_BuildStairs(
    state: &mut GameState,
    mut line: LineId,
    mut type_0: StairE,
) -> i32 {
    let mut secnum: i32 = 0;
    let mut height: i32 = 0;
    let mut i: i32 = 0;
    let mut newsecnum: i32 = 0;
    let mut texture: i32 = 0;
    let mut ok_0: i32 = 0;
    let mut rtn: i32 = 0;
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut tsec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut floor: *mut floormove_t = ::core::ptr::null_mut::<floormove_t>();
    let mut stairsize: fixed_t = 0 as fixed_t;
    let mut speed: fixed_t = 0 as fixed_t;
    secnum = -(1 as i32);
    rtn = 0 as i32;
    loop {
        secnum = P_FindSectorFromLineTag(state, line, secnum);
        if !(secnum >= 0 as i32) {
            break;
        }
        sec = state.p_setup.sector_mut(SectorId(secnum as u32));
        if (*sec).specialdata.is_some() {
            continue;
        }
        rtn = 1 as i32;
        floor = state.p_spec.spawn_floor(floormove_t::default());
        let floor_id = P_AddThinker(state, &raw mut (*floor).thinker, ThinkerKind::Floor);
        (*sec).specialdata = Some(SectorSpecial::Floor(floor_id));
        (*floor).thinker.function = ThinkerFn::Floor(T_MoveFloor);
        (*floor).direction = 1 as i32;
        (*floor).sector = SectorId(secnum as u32);
        match type_0 {
            StairE::build8 => {
                speed = (FLOORSPEED / 4 as i32) as fixed_t;
                stairsize = (8 as i32 * FRACUNIT) as fixed_t;
            }
            StairE::turbo16 => {
                speed = (FLOORSPEED * 4 as i32) as fixed_t;
                stairsize = (16 as i32 * FRACUNIT) as fixed_t;
            }
        }
        (*floor).speed = speed;
        height = ((*sec).floorheight + stairsize) as i32;
        (*floor).floordestheight = height as fixed_t;
        texture = (*sec).floorpic as i32;
        loop {
            ok_0 = 0 as i32;
            i = 0 as i32;
            while i < (*sec).linecount {
                let iline = state.p_setup.line((*sec).lines[i as usize]);
                if !(iline.flags as i32 & ML_TWOSIDED == 0) {
                    let front_id = iline.frontsector.unwrap();
                    newsecnum = front_id.0 as i32;
                    if !(secnum != newsecnum) {
                        let back_id = iline.backsector.unwrap();
                        newsecnum = back_id.0 as i32;
                        tsec = state.p_setup.sector_mut(back_id);
                        if !((*tsec).floorpic as i32 != texture) {
                            height += stairsize as i32;
                            if (*tsec).specialdata.is_none() {
                                sec = tsec;
                                secnum = newsecnum;
                                floor = state.p_spec.spawn_floor(floormove_t::default());
                                let floor_id = P_AddThinker(state, &raw mut (*floor).thinker, ThinkerKind::Floor);
                                (*sec).specialdata = Some(SectorSpecial::Floor(floor_id));
                                (*floor).thinker.function = ThinkerFn::Floor(T_MoveFloor);
                                (*floor).direction = 1 as i32;
                                (*floor).sector = SectorId(secnum as u32);
                                (*floor).speed = speed;
                                (*floor).floordestheight = height as fixed_t;
                                ok_0 = 1 as i32;
                                break;
                            }
                        }
                    }
                }
                i += 1;
            }
            if !(ok_0 != 0) {
                break;
            }
        }
    }
    return rtn;
}
