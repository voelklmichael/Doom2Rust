use crate::game_state::GameState;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACUNIT;
use crate::m_fixed::INT_MAX;
use crate::p_map::p_change_sector;
use crate::p_mobj::LineFlags;
use crate::p_setup::PSetupState;
use crate::p_spec::PSpecState;
use crate::p_spec::{Direction, Plane};
use crate::p_tick::PTickState;

use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::find_highest_floor_surrounding;
use crate::p_spec::find_lowest_ceiling_surrounding;
use crate::p_spec::find_lowest_floor_surrounding;
use crate::p_spec::find_next_highest_floor;
use crate::p_spec::get_sector;
use crate::p_spec::get_side;
use crate::p_spec::two_sided;
use crate::p_spec::FloorId;
use crate::p_spec::FloorMove;
use crate::p_spec::{find_sector_from_line_tag, sectors_with_line_tag};

use crate::p_tick::add_thinker;
use crate::p_tick::remove_thinker;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FloorE {
    LowerFloor,
    LowerFloorToLowest,
    TurboLower,
    RaiseFloor,
    RaiseFloorToNearest,
    RaiseToTexture,
    LowerAndChange,
    RaiseFloor24,
    RaiseFloor24AndChange,
    RaiseFloorCrush,
    RaiseFloorTurbo,
    DonutRaise,
    RaiseFloor512,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StairE {
    Build8,
    Turbo16,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ResultE {
    Ok,
    Crushed,
    Pastdest,
}
pub const FLOORSPEED: Fixed = FRACUNIT;
fn change_sector(state: &mut GameState, sector: SectorId, crush: bool) -> bool {
    p_change_sector(state, sector, crush)
}
pub fn move_plane(
    state: &mut GameState,
    sector: SectorId,
    speed: Fixed,
    dest: Fixed,
    crush: bool,
    floor_or_ceiling: Plane,
    direction: Direction,
) -> ResultE {
    match floor_or_ceiling {
        Plane::Floor => match direction {
            Direction::Down => {
                if state.world.p_setup.sector_mut(sector).floorheight - speed < dest {
                    let lastpos = state.world.p_setup.sector_mut(sector).floorheight;
                    state.world.p_setup.sector_mut(sector).floorheight = dest;
                    let flag: bool = change_sector(state, sector, crush);
                    if flag {
                        state.world.p_setup.sector_mut(sector).floorheight = lastpos;
                        change_sector(state, sector, crush);
                    }
                    return ResultE::Pastdest;
                }
                let lastpos = state.world.p_setup.sector_mut(sector).floorheight;
                state.world.p_setup.sector_mut(sector).floorheight -= speed;
                let flag: bool = change_sector(state, sector, crush);
                if flag {
                    state.world.p_setup.sector_mut(sector).floorheight = lastpos;
                    change_sector(state, sector, crush);
                    return ResultE::Crushed;
                }
            }
            Direction::Up => {
                if state.world.p_setup.sector_mut(sector).floorheight + speed > dest {
                    let lastpos = state.world.p_setup.sector_mut(sector).floorheight;
                    state.world.p_setup.sector_mut(sector).floorheight = dest;
                    let flag: bool = change_sector(state, sector, crush);
                    if flag {
                        state.world.p_setup.sector_mut(sector).floorheight = lastpos;
                        change_sector(state, sector, crush);
                    }
                    return ResultE::Pastdest;
                }
                let lastpos = state.world.p_setup.sector_mut(sector).floorheight;
                state.world.p_setup.sector_mut(sector).floorheight += speed;
                let flag: bool = change_sector(state, sector, crush);
                if flag {
                    if crush {
                        return ResultE::Crushed;
                    }
                    state.world.p_setup.sector_mut(sector).floorheight = lastpos;
                    change_sector(state, sector, crush);
                    return ResultE::Crushed;
                }
            }
            _ => {}
        },
        Plane::Ceiling => match direction {
            Direction::Down => {
                if state.world.p_setup.sector_mut(sector).ceilingheight - speed < dest {
                    let lastpos = state.world.p_setup.sector_mut(sector).ceilingheight;
                    state.world.p_setup.sector_mut(sector).ceilingheight = dest;
                    let flag: bool = change_sector(state, sector, crush);
                    if flag {
                        state.world.p_setup.sector_mut(sector).ceilingheight = lastpos;
                        change_sector(state, sector, crush);
                    }
                    return ResultE::Pastdest;
                }
                let lastpos = state.world.p_setup.sector_mut(sector).ceilingheight;
                state.world.p_setup.sector_mut(sector).ceilingheight -= speed;
                let flag: bool = change_sector(state, sector, crush);
                if flag {
                    if crush {
                        return ResultE::Crushed;
                    }
                    state.world.p_setup.sector_mut(sector).ceilingheight = lastpos;
                    change_sector(state, sector, crush);
                    return ResultE::Crushed;
                }
            }
            Direction::Up => {
                if state.world.p_setup.sector_mut(sector).ceilingheight + speed > dest {
                    let lastpos = state.world.p_setup.sector_mut(sector).ceilingheight;
                    state.world.p_setup.sector_mut(sector).ceilingheight = dest;
                    let flag: bool = change_sector(state, sector, crush);
                    if flag {
                        state.world.p_setup.sector_mut(sector).ceilingheight = lastpos;
                        change_sector(state, sector, crush);
                    }
                    return ResultE::Pastdest;
                }
                state.world.p_setup.sector_mut(sector).ceilingheight += speed;
                change_sector(state, sector, crush);
            }
            _ => {}
        },
    }
    ResultE::Ok
}
pub fn move_floor(state: &mut GameState, id: FloorId) {
    let floor = *state
        .world
        .p_spec
        .get_floor_ref(id)
        .expect("ThinkerFn::Floor id must reference a live floor");
    let res = move_plane(
        state,
        floor.sector,
        floor.speed,
        floor.floordestheight,
        floor.crush,
        Plane::Floor,
        floor.direction,
    );
    if state.world.p_tick.leveltime & 7 == 0 {
        s_start_sound(state, SoundOrigin::Sector(floor.sector), SfxName::Stnmov);
    }
    if res == ResultE::Pastdest {
        let sec = state.world.p_setup.sector_mut(floor.sector);
        sec.specialdata = None;
        if floor.direction == Direction::Up {
            if floor.kind == FloorE::DonutRaise {
                sec.special = floor.newspecial as i16;
                sec.floorpic = floor.texture;
            }
        } else if floor.direction == Direction::Down && floor.kind == FloorE::LowerAndChange {
            sec.special = floor.newspecial as i16;
            sec.floorpic = floor.texture;
        }
        remove_thinker(
            &mut state
                .world
                .p_spec
                .get_floor_mut(id)
                .expect("live floor")
                .thinker,
        );
        s_start_sound(state, SoundOrigin::Sector(floor.sector), SfxName::Pstop);
    }
}
pub fn do_floor(state: &mut GameState, line: LineId, floortype: FloorE) -> bool {
    let mut rtn = false;
    for sector in sectors_with_line_tag(&state.world.p_setup, line) {
        let sec = sector;
        if state.world.p_setup.sector_mut(sec).specialdata.is_some() {
            continue;
        }
        rtn = true;
        let mut floor = FloorMove::default();
        floor.thinker.function = ThinkerFn::Floor(move_floor);
        floor.kind = floortype;
        floor.crush = false;
        let (floorheight, ceilingheight, linecount) = {
            let s = state.world.p_setup.sector_mut(sec);
            (s.floorheight, s.ceilingheight, s.linecount)
        };
        let mut raise_lowest_ceiling = false;
        match floortype {
            FloorE::LowerFloor => {
                floor.direction = Direction::Down;
                floor.sector = sec;
                floor.speed = FLOORSPEED;
                floor.floordestheight =
                    find_highest_floor_surrounding(&mut state.world.p_setup, sec);
            }
            FloorE::LowerFloorToLowest => {
                floor.direction = Direction::Down;
                floor.sector = sec;
                floor.speed = FLOORSPEED;
                floor.floordestheight =
                    find_lowest_floor_surrounding(&mut state.world.p_setup, sec);
            }
            FloorE::TurboLower => {
                floor.direction = Direction::Down;
                floor.sector = sec;
                floor.speed = FLOORSPEED * 4;
                floor.floordestheight =
                    find_highest_floor_surrounding(&mut state.world.p_setup, sec);
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
                floor.direction = Direction::Up;
                floor.sector = sec;
                floor.speed = FLOORSPEED * 4;
                floor.floordestheight =
                    find_next_highest_floor(&mut state.world.p_setup, sec, floorheight);
            }
            FloorE::RaiseFloorToNearest => {
                floor.direction = Direction::Up;
                floor.sector = sec;
                floor.speed = FLOORSPEED;
                floor.floordestheight =
                    find_next_highest_floor(&mut state.world.p_setup, sec, floorheight);
            }
            FloorE::RaiseFloor24 => {
                floor.direction = Direction::Up;
                floor.sector = sec;
                floor.speed = FLOORSPEED;
                floor.floordestheight = floorheight + 24 * FRACUNIT;
            }
            FloorE::RaiseFloor512 => {
                floor.direction = Direction::Up;
                floor.sector = sec;
                floor.speed = FLOORSPEED;
                floor.floordestheight = floorheight + 512 * FRACUNIT;
            }
            FloorE::RaiseFloor24AndChange => {
                floor.direction = Direction::Up;
                floor.sector = sec;
                floor.speed = FLOORSPEED;
                floor.floordestheight = floorheight + 24 * FRACUNIT;
                let front = state.world.p_setup.line(line).front_sector();
                let (front_pic, front_special) = {
                    let fsec = state.world.p_setup.sector_mut(front);
                    (fsec.floorpic, fsec.special)
                };
                let s = state.world.p_setup.sector_mut(sec);
                s.floorpic = front_pic;
                s.special = front_special;
            }
            FloorE::RaiseToTexture => {
                let mut minsize: Fixed = Fixed(INT_MAX);
                floor.direction = Direction::Up;
                floor.sector = sec;
                floor.speed = FLOORSPEED;
                for i in 0..linecount {
                    if two_sided(&mut state.world.p_setup, sector, i) {
                        for side_index in 0..2_i32 {
                            let side = get_side(&mut state.world.p_setup, sector, i, side_index);
                            let bottomtexture = state.world.p_setup.side_mut(side).bottomtexture;
                            if i32::from(bottomtexture) >= 0
                                && state.render.r_data.textureheight[bottomtexture as usize]
                                    < minsize
                            {
                                minsize = state.render.r_data.textureheight[bottomtexture as usize];
                            }
                        }
                    }
                }
                floor.floordestheight = floorheight + minsize;
            }
            FloorE::LowerAndChange => {
                floor.direction = Direction::Down;
                floor.sector = sec;
                floor.speed = FLOORSPEED;
                floor.floordestheight =
                    find_lowest_floor_surrounding(&mut state.world.p_setup, sec);
                floor.texture = state.world.p_setup.sector_mut(sec).floorpic;
                for i in 0..linecount {
                    if two_sided(&mut state.world.p_setup, sector, i) {
                        let side0 = get_side(&mut state.world.p_setup, sector, i, 0);
                        let side0_sector = state.world.p_setup.side_mut(side0).sector;
                        let other = if side0_sector.0 == sector.0 {
                            get_sector(&mut state.world.p_setup, sector, i, 1)
                        } else {
                            get_sector(&mut state.world.p_setup, sector, i, 0)
                        };
                        let (other_floor, other_pic, other_special) = {
                            let o = state.world.p_setup.sector_mut(other);
                            (o.floorheight, o.floorpic, o.special)
                        };
                        if other_floor == floor.floordestheight {
                            floor.texture = other_pic;
                            floor.newspecial = i32::from(other_special);
                            break;
                        }
                    }
                }
            }
            FloorE::DonutRaise => {}
        }
        if raise_lowest_ceiling {
            floor.direction = Direction::Up;
            floor.sector = sec;
            floor.speed = FLOORSPEED;
            floor.floordestheight = find_lowest_ceiling_surrounding(&mut state.world.p_setup, sec);
            if floor.floordestheight > ceilingheight {
                floor.floordestheight = ceilingheight;
            }
            floor.floordestheight -= 8 * FRACUNIT * i32::from(floortype == FloorE::RaiseFloorCrush);
        }
        let floor_arena_id = state.world.p_spec.spawn_floor(floor);
        let floor_id = add_thinker(
            &mut state.world.p_tick,
            ThinkerPayload::Floor(floor_arena_id),
            ThinkerKind::Floor,
        );
        state.world.p_setup.sector_mut(sec).specialdata = Some(SectorSpecial::Floor(floor_id));
    }
    rtn
}
fn spawn_stair(
    p_setup: &mut PSetupState,
    p_spec: &mut PSpecState,
    p_tick: &mut PTickState,
    sec: SectorId,
    speed: Fixed,
    height: Fixed,
) {
    let mut floor = FloorMove::default();
    floor.thinker.function = ThinkerFn::Floor(move_floor);
    floor.direction = Direction::Up;
    floor.sector = sec;
    floor.speed = speed;
    floor.floordestheight = height;
    let floor_arena_id = p_spec.spawn_floor(floor);
    let floor_id = add_thinker(
        p_tick,
        ThinkerPayload::Floor(floor_arena_id),
        ThinkerKind::Floor,
    );
    p_setup.sector_mut(sec).specialdata = Some(SectorSpecial::Floor(floor_id));
}
pub fn build_stairs(
    p_setup: &mut PSetupState,
    p_spec: &mut PSpecState,
    p_tick: &mut PTickState,
    line: LineId,
    kind: StairE,
) -> bool {
    let mut rtn = false;
    let mut secnum: i32 = -1;
    // `secnum` is advanced inside the body, so the scan resumes after the
    // staircase just built (as vanilla does); a plain for loop would not.
    while let Some(next) = find_sector_from_line_tag(p_setup, line, secnum) {
        secnum = next;
        let mut sec = SectorId(secnum as u32);
        if p_setup.sector_mut(sec).specialdata.is_some() {
            continue;
        }
        rtn = true;
        let (speed, stairsize): (Fixed, Fixed) = match kind {
            StairE::Build8 => ((FLOORSPEED / 4), (8 * FRACUNIT)),
            StairE::Turbo16 => ((FLOORSPEED * 4), (16 * FRACUNIT)),
        };
        let mut height: Fixed = p_setup.sector_mut(sec).floorheight + stairsize;
        let texture = i32::from(p_setup.sector_mut(sec).floorpic);
        spawn_stair(p_setup, p_spec, p_tick, sec, speed, height);
        loop {
            let mut found = false;
            let linecount = p_setup.sector_mut(sec).linecount;
            for i in 0..linecount as usize {
                let line_id = p_setup.sector_mut(sec).lines[i];
                let iline = p_setup.line(line_id);
                if iline.flags.contains(LineFlags::TWOSIDED) {
                    let front_id = iline.front_sector();
                    if secnum == front_id.0 as i32 {
                        let back_id = iline
                            .backsector
                            .expect("a two-sided line has a back sector");
                        let (back_pic, back_free) = {
                            let tsec = p_setup.sector_mut(back_id);
                            (i32::from(tsec.floorpic), tsec.specialdata.is_none())
                        };
                        if back_pic == texture {
                            height += stairsize;
                            if back_free {
                                sec = back_id;
                                secnum = back_id.0 as i32;
                                spawn_stair(p_setup, p_spec, p_tick, sec, speed, height);
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
