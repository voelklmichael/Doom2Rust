use crate::game_state::GameState;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACUNIT;
use crate::m_fixed::INT_MAX;
use crate::p_map::p_change_sector;

use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::find_highest_floor_surrounding;
use crate::p_spec::find_lowest_ceiling_surrounding;
use crate::p_spec::find_lowest_floor_surrounding;
use crate::p_spec::find_next_highest_floor;
use crate::p_spec::find_sector_from_line_tag;
use crate::p_spec::get_sector;
use crate::p_spec::get_side;
use crate::p_spec::two_sided;
use crate::p_spec::FloorId;
use crate::p_spec::FloorMove;
use crate::p_spec::ML_TWOSIDED;
use crate::p_tick::add_thinker;
use crate::p_tick::remove_thinker;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FloorE {
    LowerFloor = 0,
    LowerFloorToLowest = 1,
    TurboLower = 2,
    RaiseFloor = 3,
    RaiseFloorToNearest = 4,
    RaiseToTexture = 5,
    LowerAndChange = 6,
    RaiseFloor24 = 7,
    RaiseFloor24AndChange = 8,
    RaiseFloorCrush = 9,
    RaiseFloorTurbo = 10,
    DonutRaise = 11,
    RaiseFloor512 = 12,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StairE {
    Build8 = 0,
    Turbo16 = 1,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ResultE {
    Ok = 0,
    Crushed = 1,
    Pastdest = 2,
}
pub const FLOORSPEED: i32 = FRACUNIT;
fn change_sector(state: &mut GameState, sector: SectorId, crush: bool) -> bool {
    p_change_sector(state, sector, crush)
}
pub fn move_plane(
    state: &mut GameState,
    sector: SectorId,
    speed: Fixed,
    dest: Fixed,
    crush: bool,
    floor_or_ceiling: i32,
    direction: i32,
) -> ResultE {
    let flag: bool;
    let lastpos: Fixed;
    match floor_or_ceiling {
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
                    return ResultE::Pastdest;
                } else {
                    lastpos = state.p_setup.sector_mut(sector).floorheight;
                    state.p_setup.sector_mut(sector).floorheight -= speed;
                    flag = change_sector(state, sector, crush);
                    if flag {
                        state.p_setup.sector_mut(sector).floorheight = lastpos;
                        change_sector(state, sector, crush);
                        return ResultE::Crushed;
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
                    return ResultE::Pastdest;
                } else {
                    lastpos = state.p_setup.sector_mut(sector).floorheight;
                    state.p_setup.sector_mut(sector).floorheight += speed;
                    flag = change_sector(state, sector, crush);
                    if flag {
                        if crush {
                            return ResultE::Crushed;
                        }
                        state.p_setup.sector_mut(sector).floorheight = lastpos;
                        change_sector(state, sector, crush);
                        return ResultE::Crushed;
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
                    return ResultE::Pastdest;
                } else {
                    lastpos = state.p_setup.sector_mut(sector).ceilingheight;
                    state.p_setup.sector_mut(sector).ceilingheight -= speed;
                    flag = change_sector(state, sector, crush);
                    if flag {
                        if crush {
                            return ResultE::Crushed;
                        }
                        state.p_setup.sector_mut(sector).ceilingheight = lastpos;
                        change_sector(state, sector, crush);
                        return ResultE::Crushed;
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
                    return ResultE::Pastdest;
                } else {
                    state.p_setup.sector_mut(sector).ceilingheight += speed;
                    change_sector(state, sector, crush);
                }
            }
            _ => {}
        },
        _ => {}
    }
    ResultE::Ok
}
pub fn move_floor(state: &mut GameState, id: FloorId) {
    let floor = *state
        .p_spec
        .get_floor_ref(id)
        .expect("ThinkerFn::Floor id must reference a live floor");
    let res = move_plane(
        state,
        floor.sector,
        floor.speed,
        floor.floordestheight,
        floor.crush,
        0,
        floor.direction,
    );
    if state.p_tick.leveltime & 7 == 0 {
        s_start_sound(
            state,
            SoundOrigin::Sector(floor.sector),
            SfxName::Stnmov as i32,
        );
    }
    if res == ResultE::Pastdest {
        let sec = state.p_setup.sector_mut(floor.sector);
        sec.specialdata = None;
        if floor.direction == 1 {
            if floor.kind == FloorE::DonutRaise {
                sec.special = floor.newspecial as i16;
                sec.floorpic = floor.texture;
            }
        } else if floor.direction == -1 && floor.kind == FloorE::LowerAndChange {
            sec.special = floor.newspecial as i16;
            sec.floorpic = floor.texture;
        }
        remove_thinker(&mut state.p_spec.get_floor_mut(id).expect("live floor").thinker);
        s_start_sound(
            state,
            SoundOrigin::Sector(floor.sector),
            SfxName::Pstop as i32,
        );
    }
}
pub fn do_floor(state: &mut GameState, line: LineId, floortype: FloorE) -> bool {
    let mut rtn = false;
    let mut secnum: i32 = -1;
    loop {
        secnum = find_sector_from_line_tag(state, line, secnum);
        if secnum < 0 {
            break;
        }
        let sec = SectorId(secnum as u32);
        if state.p_setup.sector_mut(sec).specialdata.is_some() {
            continue;
        }
        rtn = true;
        let mut floor = FloorMove::default();
        floor.thinker.function = ThinkerFn::Floor(move_floor);
        floor.kind = floortype;
        floor.crush = false;
        let (floorheight, ceilingheight, linecount) = {
            let s = state.p_setup.sector_mut(sec);
            (s.floorheight, s.ceilingheight, s.linecount)
        };
        let mut raise_lowest_ceiling = false;
        match floortype {
            FloorE::LowerFloor => {
                floor.direction = -1;
                floor.sector = sec;
                floor.speed = FLOORSPEED as Fixed;
                floor.floordestheight = find_highest_floor_surrounding(state, sec);
            }
            FloorE::LowerFloorToLowest => {
                floor.direction = -1;
                floor.sector = sec;
                floor.speed = FLOORSPEED as Fixed;
                floor.floordestheight = find_lowest_floor_surrounding(state, sec);
            }
            FloorE::TurboLower => {
                floor.direction = -1;
                floor.sector = sec;
                floor.speed = (FLOORSPEED * 4) as Fixed;
                floor.floordestheight = find_highest_floor_surrounding(state, sec);
                if floor.floordestheight != floorheight {
                    floor.floordestheight += 8 * FRACUNIT;
                }
            }
            FloorE::RaiseFloorCrush => {
                floor.crush = true;
                raise_lowest_ceiling = true;
            }
            FloorE::RaiseFloor => {
                raise_lowest_ceiling = true;
            }
            FloorE::RaiseFloorTurbo => {
                floor.direction = 1;
                floor.sector = sec;
                floor.speed = (FLOORSPEED * 4) as Fixed;
                floor.floordestheight = find_next_highest_floor(state, sec, floorheight);
            }
            FloorE::RaiseFloorToNearest => {
                floor.direction = 1;
                floor.sector = sec;
                floor.speed = FLOORSPEED as Fixed;
                floor.floordestheight = find_next_highest_floor(state, sec, floorheight);
            }
            FloorE::RaiseFloor24 => {
                floor.direction = 1;
                floor.sector = sec;
                floor.speed = FLOORSPEED as Fixed;
                floor.floordestheight = (floorheight + 24 * FRACUNIT) as Fixed;
            }
            FloorE::RaiseFloor512 => {
                floor.direction = 1;
                floor.sector = sec;
                floor.speed = FLOORSPEED as Fixed;
                floor.floordestheight = (floorheight + 512 * FRACUNIT) as Fixed;
            }
            FloorE::RaiseFloor24AndChange => {
                floor.direction = 1;
                floor.sector = sec;
                floor.speed = FLOORSPEED as Fixed;
                floor.floordestheight = (floorheight + 24 * FRACUNIT) as Fixed;
                let front = state.p_setup.line(line).frontsector.unwrap();
                let (front_pic, front_special) = {
                    let fsec = state.p_setup.sector_mut(front);
                    (fsec.floorpic, fsec.special)
                };
                let s = state.p_setup.sector_mut(sec);
                s.floorpic = front_pic;
                s.special = front_special;
            }
            FloorE::RaiseToTexture => {
                let mut minsize: i32 = INT_MAX;
                floor.direction = 1;
                floor.sector = sec;
                floor.speed = FLOORSPEED as Fixed;
                for i in 0..linecount {
                    if two_sided(state, secnum, i) != 0 {
                        for side_index in 0..2_i32 {
                            let side = get_side(state, secnum, i, side_index);
                            let bottomtexture = state.p_setup.side_mut(side).bottomtexture;
                            if bottomtexture as i32 >= 0
                                && state.r_data.textureheight[bottomtexture as usize] < minsize
                            {
                                minsize = state.r_data.textureheight[bottomtexture as usize];
                            }
                        }
                    }
                }
                floor.floordestheight = (floorheight + minsize) as Fixed;
            }
            FloorE::LowerAndChange => {
                floor.direction = -1;
                floor.sector = sec;
                floor.speed = FLOORSPEED as Fixed;
                floor.floordestheight = find_lowest_floor_surrounding(state, sec);
                floor.texture = state.p_setup.sector_mut(sec).floorpic;
                for i in 0..linecount {
                    if two_sided(state, secnum, i) != 0 {
                        let side0 = get_side(state, secnum, i, 0);
                        let side0_sector = state.p_setup.side_mut(side0).sector;
                        let other = if side0_sector.0 == secnum as u32 {
                            get_sector(state, secnum, i, 1)
                        } else {
                            get_sector(state, secnum, i, 0)
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
            floor.direction = 1;
            floor.sector = sec;
            floor.speed = FLOORSPEED as Fixed;
            floor.floordestheight = find_lowest_ceiling_surrounding(state, sec);
            if floor.floordestheight > ceilingheight {
                floor.floordestheight = ceilingheight;
            }
            floor.floordestheight -= 8 * FRACUNIT * (floortype == FloorE::RaiseFloorCrush) as i32;
        }
        let floor_arena_id = state.p_spec.spawn_floor(floor);
        let floor_id = add_thinker(
            state,
            ThinkerPayload::Floor(floor_arena_id),
            ThinkerKind::Floor,
        );
        state.p_setup.sector_mut(sec).specialdata = Some(SectorSpecial::Floor(floor_id));
    }
    rtn
}
fn spawn_stair(state: &mut GameState, sec: SectorId, speed: Fixed, height: i32) {
    let mut floor = FloorMove::default();
    floor.thinker.function = ThinkerFn::Floor(move_floor);
    floor.direction = 1;
    floor.sector = sec;
    floor.speed = speed;
    floor.floordestheight = height as Fixed;
    let floor_arena_id = state.p_spec.spawn_floor(floor);
    let floor_id = add_thinker(
        state,
        ThinkerPayload::Floor(floor_arena_id),
        ThinkerKind::Floor,
    );
    state.p_setup.sector_mut(sec).specialdata = Some(SectorSpecial::Floor(floor_id));
}
pub fn build_stairs(state: &mut GameState, line: LineId, kind: StairE) -> bool {
    let mut rtn = false;
    let mut secnum: i32 = -1;
    loop {
        secnum = find_sector_from_line_tag(state, line, secnum);
        if secnum < 0 {
            break;
        }
        let mut sec = SectorId(secnum as u32);
        if state.p_setup.sector_mut(sec).specialdata.is_some() {
            continue;
        }
        rtn = true;
        let (speed, stairsize): (Fixed, Fixed) = match kind {
            StairE::Build8 => ((FLOORSPEED / 4) as Fixed, (8 * FRACUNIT) as Fixed),
            StairE::Turbo16 => ((FLOORSPEED * 4) as Fixed, (16 * FRACUNIT) as Fixed),
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
