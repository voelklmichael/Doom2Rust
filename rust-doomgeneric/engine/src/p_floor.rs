use crate::game_state::GameState;
use crate::index::ToIndex;
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
/// Sets where a floor mover goes: which way, how fast, and to what height.
fn aim_floor(
    floor: &mut FloorMove,
    sec: SectorId,
    direction: Direction,
    speed: Fixed,
    destination: Fixed,
) {
    floor.direction = direction;
    floor.sector = sec;
    floor.speed = speed;
    floor.floordestheight = destination;
}

/// Aims the floor at the lowest ceiling around the sector, but not above its own ceiling, less
/// `undershoot` (a crusher stops 8 units short).
fn raise_to_lowest_ceiling(
    state: &mut GameState,
    floor: &mut FloorMove,
    sec: SectorId,
    undershoot: Fixed,
) {
    let ceilingheight = state.world.p_setup.sector_mut(sec).ceilingheight;
    let lowest_ceiling = find_lowest_ceiling_surrounding(&mut state.world.p_setup, sec);
    aim_floor(
        floor,
        sec,
        Direction::Up,
        FLOORSPEED,
        lowest_ceiling.min(ceilingheight) - undershoot,
    );
}

/// The height of the shortest lower texture on the sides of the sector's two-sided lines.
fn shortest_lower_texture(state: &mut GameState, sector: SectorId, linecount: i32) -> Fixed {
    let mut minsize: Fixed = Fixed(INT_MAX);
    for i in 0..linecount {
        if two_sided(&mut state.world.p_setup, sector, i) {
            for side_index in 0..2_i32 {
                let side = get_side(&mut state.world.p_setup, sector, i, side_index);
                let bottomtexture = state.world.p_setup.side_mut(side).bottomtexture;
                if i32::from(bottomtexture) >= 0
                    && state.render.r_data.textureheight[bottomtexture.idx()] < minsize
                {
                    minsize = state.render.r_data.textureheight[bottomtexture.idx()];
                }
            }
        }
    }
    minsize
}

/// Gives the floor the texture and special of the first neighbouring sector whose floor is at
/// the height it is lowering to.
fn take_neighbour_floor_texture(
    state: &mut GameState,
    floor: &mut FloorMove,
    sector: SectorId,
    linecount: i32,
) {
    floor.texture = state.world.p_setup.sector_mut(sector).floorpic;
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

/// The floor mover for `floortype` on sector `sec` (the `line`'s tag selected it).
fn plan_floor(state: &mut GameState, line: LineId, sec: SectorId, floortype: FloorE) -> FloorMove {
    let mut floor = FloorMove::default();
    floor.thinker.function = ThinkerFn::Floor(move_floor);
    floor.kind = floortype;
    floor.crush = false;
    let (floorheight, linecount) = {
        let s = state.world.p_setup.sector_mut(sec);
        (s.floorheight, s.linecount)
    };
    match floortype {
        FloorE::LowerFloor => {
            let destination = find_highest_floor_surrounding(&mut state.world.p_setup, sec);
            aim_floor(&mut floor, sec, Direction::Down, FLOORSPEED, destination);
        }
        FloorE::LowerFloorToLowest => {
            let destination = find_lowest_floor_surrounding(&mut state.world.p_setup, sec);
            aim_floor(&mut floor, sec, Direction::Down, FLOORSPEED, destination);
        }
        FloorE::TurboLower => {
            let mut destination = find_highest_floor_surrounding(&mut state.world.p_setup, sec);
            if destination != floorheight {
                destination += 8 * FRACUNIT;
            }
            aim_floor(
                &mut floor,
                sec,
                Direction::Down,
                FLOORSPEED * 4,
                destination,
            );
        }
        FloorE::RaiseFloorCrush => {
            floor.crush = true;
            raise_to_lowest_ceiling(state, &mut floor, sec, 8 * FRACUNIT);
        }
        FloorE::RaiseFloor => raise_to_lowest_ceiling(state, &mut floor, sec, Fixed::ZERO),
        FloorE::RaiseFloorTurbo => {
            let destination = find_next_highest_floor(&mut state.world.p_setup, sec, floorheight);
            aim_floor(&mut floor, sec, Direction::Up, FLOORSPEED * 4, destination);
        }
        FloorE::RaiseFloorToNearest => {
            let destination = find_next_highest_floor(&mut state.world.p_setup, sec, floorheight);
            aim_floor(&mut floor, sec, Direction::Up, FLOORSPEED, destination);
        }
        FloorE::RaiseFloor24 => {
            let destination = floorheight + 24 * FRACUNIT;
            aim_floor(&mut floor, sec, Direction::Up, FLOORSPEED, destination);
        }
        FloorE::RaiseFloor512 => {
            let destination = floorheight + 512 * FRACUNIT;
            aim_floor(&mut floor, sec, Direction::Up, FLOORSPEED, destination);
        }
        FloorE::RaiseFloor24AndChange => {
            let destination = floorheight + 24 * FRACUNIT;
            aim_floor(&mut floor, sec, Direction::Up, FLOORSPEED, destination);
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
            let destination = floorheight + shortest_lower_texture(state, sec, linecount);
            aim_floor(&mut floor, sec, Direction::Up, FLOORSPEED, destination);
        }
        FloorE::LowerAndChange => {
            let destination = find_lowest_floor_surrounding(&mut state.world.p_setup, sec);
            aim_floor(&mut floor, sec, Direction::Down, FLOORSPEED, destination);
            take_neighbour_floor_texture(state, &mut floor, sec, linecount);
        }
        FloorE::DonutRaise => {}
    }
    floor
}

pub fn do_floor(state: &mut GameState, line: LineId, floortype: FloorE) -> bool {
    let mut rtn = false;
    for sec in sectors_with_line_tag(&state.world.p_setup, line) {
        if state.world.p_setup.sector_mut(sec).specialdata.is_some() {
            continue;
        }
        rtn = true;
        let floor = plan_floor(state, line, sec, floortype);
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
        let mut sec = SectorId(secnum.cast_unsigned());
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
            for i in 0..linecount.idx() {
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
