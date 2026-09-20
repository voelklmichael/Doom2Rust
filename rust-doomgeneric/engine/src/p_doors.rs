use crate::doomdef::TICRATE;
use crate::game_state::GameState;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACUNIT;
use crate::p_floor::move_plane;
use crate::p_floor::ResultE;
use crate::p_inter::CardType;
use crate::p_mobj::MobjId;
use crate::p_mobj::SectorSpecial;
use crate::p_mobj::Thinker;
use crate::p_mobj::ThinkerFn;
use crate::p_setup::LineId;
use crate::p_setup::PSetupState;
use crate::p_setup::SectorId;
use crate::p_spec::find_lowest_ceiling_surrounding;
use crate::p_spec::sectors_with_line_tag;
use crate::p_spec::{Direction, Plane};
use crate::p_tick::PTickState;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::p_tick::add_thinker;
use crate::p_tick::remove_thinker;

use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum VldoorE {
    Normal,
    Close30ThenOpen,
    Close,
    Open,
    RaiseIn5Mins,
    BlazeRaise,
    BlazeOpen,
    BlazeClose,
}
#[derive(Copy, Clone)]
pub struct VlDoor {
    pub thinker: Thinker,
    pub kind: VldoorE,
    pub sector: SectorId,
    pub topheight: Fixed,
    pub speed: Fixed,
    pub direction: Direction,
    pub topwait: i32,
    pub topcountdown: i32,
}
// Every real field gets explicitly set by the caller within a few lines of
// spawn() returning (confirmed by reading every spawn site below) -- this
// placeholder's values are never read, only its shape matters.
impl Default for VlDoor {
    fn default() -> Self {
        Self {
            thinker: Thinker {
                function: ThinkerFn::Unresolved,
            },
            kind: VldoorE::Normal,
            sector: SectorId(0),
            topheight: Fixed::ZERO,
            speed: Fixed::ZERO,
            direction: Direction::Still,
            topwait: 0,
            topcountdown: 0,
        }
    }
}

// Generation-checked handle into PDoorsState's arena -- mirrors MobjId.
// Unlike Mobj, a door slot is freed in one step (dealloc), not a
// retire()-then-deallocate() split: nothing keeps dereferencing a door's
// raw pointer after it's removed the way remove_mobj's body does, so
// there's no use-after-free window to guard against by deferring the free.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct DoorId {
    index: u32,
    generation: u32,
}

struct DoorSlot {
    generation: u32,
    door: Option<Box<VlDoor>>,
}

#[derive(Default)]
pub struct PDoorsState {
    doors: Vec<DoorSlot>,
    free_list: Vec<u32>,
}

impl PDoorsState {
    // Moves a fully-defaulted (then caller-filled) VlDoor onto the heap
    // and hands back both a stable generation-checked handle (stored in
    // ThinkerNode's payload by p_tick.rs, replacing what used to be a bare
    // raw pointer there) and a raw pointer for the caller's immediate
    // post-spawn field writes -- mirrors PMobjState::spawn exactly.
    pub fn spawn(&mut self, value: VlDoor) -> DoorId {
        let (index, generation) = if let Some(index) = self.free_list.pop() {
            let slot = &mut self.doors[index as usize];
            slot.generation = slot.generation.wrapping_add(1);
            (index, slot.generation)
        } else {
            let index = self.doors.len() as u32;
            self.doors.push(DoorSlot {
                generation: 0,
                door: None,
            });
            (index, 0)
        };
        let id = DoorId { index, generation };
        let boxed = Box::new(value);
        self.doors[index as usize].door = Some(boxed);
        id
    }

    pub fn get_ref(&self, id: DoorId) -> Option<&VlDoor> {
        self.doors
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.door.as_deref())
    }

    pub fn get_mut(&mut self, id: DoorId) -> Option<&mut VlDoor> {
        self.doors
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.door.as_deref_mut())
    }

    // Called once, from run_thinkers' reaper, when a Door-kind thinker is
    // reaped.
    pub fn dealloc(&mut self, id: DoorId) {
        if let Some(slot) = self.doors.get_mut(id.index as usize) {
            if slot.generation == id.generation {
                slot.door = None;
                self.free_list.push(id.index);
            }
        }
    }
}

pub const VDOORWAIT: i32 = 150;
pub fn t_vertical_door(state: &mut GameState, id: DoorId) {
    let door = *state
        .world
        .p_doors
        .get_ref(id)
        .expect("ThinkerFn::Door id must reference a live door");
    macro_rules! door_mut {
        () => {
            state.world.p_doors.get_mut(id).expect("live door")
        };
    }
    match door.direction {
        Direction::Still => {
            door_mut!().topcountdown -= 1;
            if door.topcountdown - 1 == 0 {
                match door.kind {
                    VldoorE::BlazeRaise => {
                        door_mut!().direction = Direction::Down;
                        s_start_sound(state, SoundOrigin::Sector(door.sector), SfxName::Bdcls);
                    }
                    VldoorE::Normal => {
                        door_mut!().direction = Direction::Down;
                        s_start_sound(state, SoundOrigin::Sector(door.sector), SfxName::Dorcls);
                    }
                    VldoorE::Close30ThenOpen => {
                        door_mut!().direction = Direction::Up;
                        s_start_sound(state, SoundOrigin::Sector(door.sector), SfxName::Doropn);
                    }
                    _ => {}
                }
            }
        }
        Direction::InitialWait => {
            door_mut!().topcountdown -= 1;
            if door.topcountdown - 1 == 0 && door.kind == VldoorE::RaiseIn5Mins {
                let d = door_mut!();
                d.direction = Direction::Up;
                d.kind = VldoorE::Normal;
                s_start_sound(state, SoundOrigin::Sector(door.sector), SfxName::Doropn);
            }
        }
        Direction::Down => {
            let floorheight = state.world.p_setup.sector_mut(door.sector).floorheight;
            let res = move_plane(
                state,
                door.sector,
                door.speed,
                floorheight,
                false,
                Plane::Ceiling,
                door.direction,
            );
            if res == ResultE::Pastdest {
                match door.kind {
                    VldoorE::BlazeRaise | VldoorE::BlazeClose => {
                        state.world.p_setup.sector_mut(door.sector).specialdata = None;
                        remove_thinker(&mut door_mut!().thinker);
                        s_start_sound(state, SoundOrigin::Sector(door.sector), SfxName::Bdcls);
                    }
                    VldoorE::Normal | VldoorE::Close => {
                        state.world.p_setup.sector_mut(door.sector).specialdata = None;
                        remove_thinker(&mut door_mut!().thinker);
                    }
                    VldoorE::Close30ThenOpen => {
                        let d = door_mut!();
                        d.direction = Direction::Still;
                        d.topcountdown = TICRATE * 30;
                    }
                    _ => {}
                }
            } else if res == ResultE::Crushed {
                match door.kind {
                    VldoorE::BlazeClose | VldoorE::Close => {}
                    _ => {
                        door_mut!().direction = Direction::Up;
                        s_start_sound(state, SoundOrigin::Sector(door.sector), SfxName::Doropn);
                    }
                }
            }
        }
        Direction::Up => {
            let res = move_plane(
                state,
                door.sector,
                door.speed,
                door.topheight,
                false,
                Plane::Ceiling,
                door.direction,
            );
            if res == ResultE::Pastdest {
                match door.kind {
                    VldoorE::BlazeRaise | VldoorE::Normal => {
                        let d = door_mut!();
                        d.direction = Direction::Still;
                        d.topcountdown = d.topwait;
                    }
                    VldoorE::Close30ThenOpen | VldoorE::BlazeOpen | VldoorE::Open => {
                        state.world.p_setup.sector_mut(door.sector).specialdata = None;
                        remove_thinker(&mut door_mut!().thinker);
                    }
                    _ => {}
                }
            }
        }
    }
}
pub fn do_locked_door(state: &mut GameState, line: LineId, kind: VldoorE, thing: MobjId) -> bool {
    let Some(player_id) = state.world.p_mobj.mo(thing).player else {
        return false;
    };
    let (blue, red, yellow) = {
        let p = state.game.g_game.player_mut(player_id);
        (
            p.cards[CardType::Bluecard] || p.cards[CardType::Blueskull],
            p.cards[CardType::Redcard] || p.cards[CardType::Redskull],
            p.cards[CardType::Yellowcard] || p.cards[CardType::Yellowskull],
        )
    };
    let missing = match i32::from(state.world.p_setup.line(line).special) {
        99 | 133 if !blue => Some("You need a blue key to activate this object"),
        134 | 135 if !red => Some("You need a red key to activate this object"),
        136 | 137 if !yellow => Some("You need a yellow key to activate this object"),
        _ => None,
    };
    if let Some(message) = missing {
        state.game.g_game.player_mut(player_id).message = Some(message.to_string());
        s_start_sound(state, SoundOrigin::None, SfxName::Oof);
        return false;
    }
    do_door(state, line, kind)
}
pub fn do_door(state: &mut GameState, line: LineId, kind: VldoorE) -> bool {
    let mut rtn = false;
    for sector in sectors_with_line_tag(&state.world.p_setup, line) {
        let sec = sector;
        if state.world.p_setup.sector_mut(sec).specialdata.is_some() {
            continue;
        }
        rtn = true;
        let ceilingheight = state.world.p_setup.sector_mut(sec).ceilingheight;
        let mut door = VlDoor::default();
        door.thinker.function = ThinkerFn::Door(t_vertical_door);
        door.sector = sec;
        door.kind = kind;
        door.topwait = VDOORWAIT;
        door.speed = FRACUNIT * 2;
        match kind {
            VldoorE::BlazeClose => {
                door.topheight = find_lowest_ceiling_surrounding(&mut state.world.p_setup, sec);
                door.topheight -= 4 * FRACUNIT;
                door.direction = Direction::Down;
                door.speed = FRACUNIT * 2 * 4;
                s_start_sound(state, SoundOrigin::Sector(sec), SfxName::Bdcls);
            }
            VldoorE::Close => {
                door.topheight = find_lowest_ceiling_surrounding(&mut state.world.p_setup, sec);
                door.topheight -= 4 * FRACUNIT;
                door.direction = Direction::Down;
                s_start_sound(state, SoundOrigin::Sector(sec), SfxName::Dorcls);
            }
            VldoorE::Close30ThenOpen => {
                door.topheight = ceilingheight;
                door.direction = Direction::Down;
                s_start_sound(state, SoundOrigin::Sector(sec), SfxName::Dorcls);
            }
            VldoorE::BlazeRaise | VldoorE::BlazeOpen => {
                door.direction = Direction::Up;
                door.topheight = find_lowest_ceiling_surrounding(&mut state.world.p_setup, sec);
                door.topheight -= 4 * FRACUNIT;
                door.speed = FRACUNIT * 2 * 4;
                if door.topheight != ceilingheight {
                    s_start_sound(state, SoundOrigin::Sector(sec), SfxName::Bdopn);
                }
            }
            VldoorE::Normal | VldoorE::Open => {
                door.direction = Direction::Up;
                door.topheight = find_lowest_ceiling_surrounding(&mut state.world.p_setup, sec);
                door.topheight -= 4 * FRACUNIT;
                if door.topheight != ceilingheight {
                    s_start_sound(state, SoundOrigin::Sector(sec), SfxName::Doropn);
                }
            }
            VldoorE::RaiseIn5Mins => {}
        }
        let door_arena_id = state.world.p_doors.spawn(door);
        let door_id = add_thinker(
            &mut state.world.p_tick,
            ThinkerPayload::Door(door_arena_id),
            ThinkerKind::Door,
        );
        state.world.p_setup.sector_mut(sec).specialdata = Some(SectorSpecial::Door(door_id));
    }
    rtn
}
pub fn ev_vertical_door(state: &mut GameState, line: LineId, thing: MobjId) {
    let side: i32 = 0;
    let thing_player = state.world.p_mobj.mo(thing).player;
    let linev = state.world.p_setup.line(line);
    let key_message = |has: bool, message: &'static str| if has { None } else { Some(message) };
    if let Some(player_id) = thing_player {
        let (blue, red, yellow) = {
            let p = state.game.g_game.player_mut(player_id);
            (
                p.cards[CardType::Bluecard] || p.cards[CardType::Blueskull],
                p.cards[CardType::Redcard] || p.cards[CardType::Redskull],
                p.cards[CardType::Yellowcard] || p.cards[CardType::Yellowskull],
            )
        };
        let missing = match i32::from(linev.special) {
            26 | 32 => key_message(blue, "You need a blue key to open this door"),
            27 | 34 => key_message(yellow, "You need a yellow key to open this door"),
            28 | 33 => key_message(red, "You need a red key to open this door"),
            _ => None,
        };
        if let Some(message) = missing {
            state.game.g_game.player_mut(player_id).message = Some(message.to_string());
            s_start_sound(state, SoundOrigin::None, SfxName::Oof);
            return;
        }
    } else if matches!(i32::from(linev.special), 26 | 32 | 27 | 34 | 28 | 33) {
        return;
    }
    let door_sector_id = state.world.p_setup.sides[linev.sidenum[(side ^ 1) as usize]
        .expect("two-sided line without a back side")
        .0 as usize]
        .sector;
    if let Some(special) = state.world.p_setup.sector_mut(door_sector_id).specialdata {
        match i32::from(linev.special) {
            1 | 26 | 27 | 28 | 117 => {
                match special {
                    SectorSpecial::Door(id) => {
                        let door_id = state.world.p_tick.door_payload(id);
                        let door = state.world.p_doors.get_mut(door_id).expect("live door");
                        if door.direction == Direction::Down {
                            door.direction = Direction::Up;
                        } else {
                            if thing_player.is_none() {
                                return;
                            }
                            door.direction = Direction::Down;
                        }
                    }
                    SectorSpecial::Plat(id) => {
                        if thing_player.is_none() {
                            return;
                        }
                        let plat_id = state.world.p_tick.plat_payload(id);
                        state
                            .world
                            .p_plats
                            .get_mut(plat_id)
                            .expect("live plat")
                            .wait = -1;
                    }
                    SectorSpecial::Ceiling(id) => {
                        if thing_player.is_none() {
                            return;
                        }
                        doom_eprintln!(
                            state.io.platform,
                            "EV_VerticalDoor: Tried to close something that wasn't a door."
                        );
                        let ceiling_id = state.world.p_tick.ceiling_payload(id);
                        state
                            .world
                            .p_ceilng
                            .get_mut(ceiling_id)
                            .expect("live ceiling")
                            .direction = Direction::Down;
                    }
                    SectorSpecial::Floor(id) => {
                        if thing_player.is_none() {
                            return;
                        }
                        doom_eprintln!(
                            state.io.platform,
                            "EV_VerticalDoor: Tried to close something that wasn't a door."
                        );
                        let floor_id = state.world.p_tick.floor_payload(id);
                        state
                            .world
                            .p_spec
                            .get_floor_mut(floor_id)
                            .expect("live floor")
                            .direction = Direction::Down;
                    }
                }
                return;
            }
            _ => {}
        }
    }
    match i32::from(linev.special) {
        117 | 118 => {
            s_start_sound(state, SoundOrigin::Sector(door_sector_id), SfxName::Bdopn);
        }
        _ => {
            s_start_sound(state, SoundOrigin::Sector(door_sector_id), SfxName::Doropn);
        }
    }
    let mut door = VlDoor::default();
    door.thinker.function = ThinkerFn::Door(t_vertical_door);
    door.sector = door_sector_id;
    door.direction = Direction::Up;
    door.speed = FRACUNIT * 2;
    door.topwait = VDOORWAIT;
    match i32::from(linev.special) {
        1 | 26 | 27 | 28 => {
            door.kind = VldoorE::Normal;
        }
        31..=34 => {
            door.kind = VldoorE::Open;
            state.world.p_setup.line_mut(line).special = 0;
        }
        117 => {
            door.kind = VldoorE::BlazeRaise;
            door.speed = FRACUNIT * 2 * 4;
        }
        118 => {
            door.kind = VldoorE::BlazeOpen;
            state.world.p_setup.line_mut(line).special = 0;
            door.speed = FRACUNIT * 2 * 4;
        }
        _ => {}
    }
    door.topheight = find_lowest_ceiling_surrounding(&mut state.world.p_setup, door_sector_id);
    door.topheight -= 4 * FRACUNIT;
    let door_arena_id = state.world.p_doors.spawn(door);
    let door_id = add_thinker(
        &mut state.world.p_tick,
        ThinkerPayload::Door(door_arena_id),
        ThinkerKind::Door,
    );
    state.world.p_setup.sector_mut(door_sector_id).specialdata = Some(SectorSpecial::Door(door_id));
}
pub fn spawn_door_close_in30(
    p_doors: &mut PDoorsState,
    p_setup: &mut PSetupState,
    p_tick: &mut PTickState,
    sector: SectorId,
) {
    let mut door = VlDoor::default();
    door.thinker.function = ThinkerFn::Door(t_vertical_door);
    door.sector = sector;
    door.direction = Direction::Still;
    door.kind = VldoorE::Normal;
    door.speed = FRACUNIT * 2;
    door.topcountdown = 30 * TICRATE;
    let door_arena_id = p_doors.spawn(door);
    let door_id = add_thinker(
        p_tick,
        ThinkerPayload::Door(door_arena_id),
        ThinkerKind::Door,
    );
    let sec = p_setup.sector_mut(sector);
    sec.specialdata = Some(SectorSpecial::Door(door_id));
    sec.special = 0;
}
pub fn spawn_door_raise_in5_mins(
    p_doors: &mut PDoorsState,
    p_setup: &mut PSetupState,
    p_tick: &mut PTickState,
    sector: SectorId,
) {
    let mut door = VlDoor::default();
    door.thinker.function = ThinkerFn::Door(t_vertical_door);
    door.sector = sector;
    door.direction = Direction::InitialWait;
    door.kind = VldoorE::RaiseIn5Mins;
    door.speed = FRACUNIT * 2;
    door.topheight = find_lowest_ceiling_surrounding(p_setup, sector);
    door.topheight -= 4 * FRACUNIT;
    door.topwait = VDOORWAIT;
    door.topcountdown = 5 * 60 * TICRATE;
    let door_arena_id = p_doors.spawn(door);
    let door_id = add_thinker(
        p_tick,
        ThinkerPayload::Door(door_arena_id),
        ThinkerKind::Door,
    );
    let sec = p_setup.sector_mut(sector);
    sec.specialdata = Some(SectorSpecial::Door(door_id));
    sec.special = 0;
}
