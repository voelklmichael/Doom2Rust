use crate::doomdef::TICRATE;
use crate::game_state::GameState;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACUNIT;
use crate::p_floor::ResultE;
use crate::p_floor::T_MovePlane;
use crate::p_inter::CardType;
use crate::p_mobj::thinker_t;
use crate::p_mobj::MobjId;
use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::P_FindLowestCeilingSurrounding;
use crate::p_spec::P_FindSectorFromLineTag;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::p_tick::P_AddThinker;
use crate::p_tick::P_RemoveThinker;

use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum VldoorE {
    vld_normal = 0,
    vld_close30ThenOpen = 1,
    vld_close = 2,
    vld_open = 3,
    vld_raiseIn5Mins = 4,
    vld_blazeRaise = 5,
    vld_blazeOpen = 6,
    vld_blazeClose = 7,
}
#[derive(Copy, Clone)]
pub struct vldoor_t {
    pub thinker: thinker_t,
    pub kind: VldoorE,
    pub sector: SectorId,
    pub topheight: fixed_t,
    pub speed: fixed_t,
    pub direction: i32,
    pub topwait: i32,
    pub topcountdown: i32,
}
// Every real field gets explicitly set by the caller within a few lines of
// spawn() returning (confirmed by reading every spawn site below) -- this
// placeholder's values are never read, only its shape matters.
impl Default for vldoor_t {
    fn default() -> Self {
        vldoor_t {
            thinker: thinker_t {
                function: ThinkerFn::Unresolved,
            },
            kind: VldoorE::vld_normal,
            sector: SectorId(0),
            topheight: 0,
            speed: 0,
            direction: 0,
            topwait: 0,
            topcountdown: 0,
        }
    }
}

// Generation-checked handle into PDoorsState's arena -- mirrors MobjId.
// Unlike mobj_t, a door slot is freed in one step (dealloc), not a
// retire()-then-deallocate() split: nothing keeps dereferencing a door's
// raw pointer after it's removed the way P_RemoveMobj's body does, so
// there's no use-after-free window to guard against by deferring the free.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct DoorId {
    index: u32,
    generation: u32,
}

struct DoorSlot {
    generation: u32,
    door: Option<Box<vldoor_t>>,
}

pub struct PDoorsState {
    doors: Vec<DoorSlot>,
    free_list: Vec<u32>,
}

impl Default for PDoorsState {
    fn default() -> Self {
        Self::new()
    }
}

impl PDoorsState {
    pub const fn new() -> Self {
        PDoorsState {
            doors: Vec::new(),
            free_list: Vec::new(),
        }
    }

    // Moves a fully-defaulted (then caller-filled) vldoor_t onto the heap
    // and hands back both a stable generation-checked handle (stored in
    // ThinkerNode's payload by p_tick.rs, replacing what used to be a bare
    // raw pointer there) and a raw pointer for the caller's immediate
    // post-spawn field writes -- mirrors PMobjState::spawn exactly.
    pub fn spawn(&mut self, value: vldoor_t) -> DoorId {
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

    pub fn get_ref(&self, id: DoorId) -> Option<&vldoor_t> {
        self.doors
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.door.as_deref())
    }

    pub fn get_mut(&mut self, id: DoorId) -> Option<&mut vldoor_t> {
        self.doors
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.door.as_deref_mut())
    }

    // Called once, from P_RunThinkers' reaper, when a Door-kind thinker is
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
pub fn T_VerticalDoor(state: &mut GameState, id: DoorId) {
    let door = *state
        .p_doors
        .get_ref(id)
        .expect("ThinkerFn::Door id must reference a live door");
    macro_rules! door_mut {
        () => {
            state.p_doors.get_mut(id).expect("live door")
        };
    }
    match door.direction {
        0 => {
            door_mut!().topcountdown -= 1;
            if door.topcountdown - 1 == 0 {
                match door.kind {
                    VldoorE::vld_blazeRaise => {
                        door_mut!().direction = -1;
                        S_StartSound(
                            state,
                            SoundOrigin::Sector(door.sector),
                            SfxName::sfx_bdcls as i32,
                        );
                    }
                    VldoorE::vld_normal => {
                        door_mut!().direction = -1;
                        S_StartSound(
                            state,
                            SoundOrigin::Sector(door.sector),
                            SfxName::sfx_dorcls as i32,
                        );
                    }
                    VldoorE::vld_close30ThenOpen => {
                        door_mut!().direction = 1;
                        S_StartSound(
                            state,
                            SoundOrigin::Sector(door.sector),
                            SfxName::sfx_doropn as i32,
                        );
                    }
                    _ => {}
                }
            }
        }
        2 => {
            door_mut!().topcountdown -= 1;
            if door.topcountdown - 1 == 0 && door.kind == VldoorE::vld_raiseIn5Mins {
                let d = door_mut!();
                d.direction = 1;
                d.kind = VldoorE::vld_normal;
                S_StartSound(
                    state,
                    SoundOrigin::Sector(door.sector),
                    SfxName::sfx_doropn as i32,
                );
            }
        }
        -1 => {
            let floorheight = state.p_setup.sector_mut(door.sector).floorheight;
            let res = T_MovePlane(
                state,
                door.sector,
                door.speed,
                floorheight,
                false,
                1,
                door.direction,
            );
            if res == ResultE::pastdest {
                match door.kind {
                    VldoorE::vld_blazeRaise | VldoorE::vld_blazeClose => {
                        state.p_setup.sector_mut(door.sector).specialdata = None;
                        P_RemoveThinker(&mut door_mut!().thinker);
                        S_StartSound(
                            state,
                            SoundOrigin::Sector(door.sector),
                            SfxName::sfx_bdcls as i32,
                        );
                    }
                    VldoorE::vld_normal | VldoorE::vld_close => {
                        state.p_setup.sector_mut(door.sector).specialdata = None;
                        P_RemoveThinker(&mut door_mut!().thinker);
                    }
                    VldoorE::vld_close30ThenOpen => {
                        let d = door_mut!();
                        d.direction = 0;
                        d.topcountdown = TICRATE * 30;
                    }
                    _ => {}
                }
            } else if res == ResultE::crushed {
                match door.kind {
                    VldoorE::vld_blazeClose | VldoorE::vld_close => {}
                    _ => {
                        door_mut!().direction = 1;
                        S_StartSound(
                            state,
                            SoundOrigin::Sector(door.sector),
                            SfxName::sfx_doropn as i32,
                        );
                    }
                }
            }
        }
        1 => {
            let res = T_MovePlane(
                state,
                door.sector,
                door.speed,
                door.topheight,
                false,
                1,
                door.direction,
            );
            if res == ResultE::pastdest {
                match door.kind {
                    VldoorE::vld_blazeRaise | VldoorE::vld_normal => {
                        let d = door_mut!();
                        d.direction = 0;
                        d.topcountdown = d.topwait;
                    }
                    VldoorE::vld_close30ThenOpen | VldoorE::vld_blazeOpen | VldoorE::vld_open => {
                        state.p_setup.sector_mut(door.sector).specialdata = None;
                        P_RemoveThinker(&mut door_mut!().thinker);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    };
}
pub fn EV_DoLockedDoor(state: &mut GameState, line: LineId, kind: VldoorE, thing: MobjId) -> bool {
    let Some(player_id) = state.p_mobj.mo(thing).player else {
        return false;
    };
    let (blue, red, yellow) = {
        let p = state.g_game.player_mut(player_id);
        (
            p.cards[CardType::it_bluecard as usize] || p.cards[CardType::it_blueskull as usize],
            p.cards[CardType::it_redcard as usize] || p.cards[CardType::it_redskull as usize],
            p.cards[CardType::it_yellowcard as usize] || p.cards[CardType::it_yellowskull as usize],
        )
    };
    let missing = match state.p_setup.line(line).special as i32 {
        99 | 133 if !blue => Some("You need a blue key to activate this object"),
        134 | 135 if !red => Some("You need a red key to activate this object"),
        136 | 137 if !yellow => Some("You need a yellow key to activate this object"),
        _ => None,
    };
    if let Some(message) = missing {
        state.g_game.player_mut(player_id).message = Some(message.to_string());
        S_StartSound(state, SoundOrigin::None, SfxName::sfx_oof as i32);
        return false;
    }
    EV_DoDoor(state, line, kind)
}
pub fn EV_DoDoor(state: &mut GameState, line: LineId, kind: VldoorE) -> bool {
    let mut rtn = false;
    let mut secnum: i32 = -1;
    loop {
        secnum = P_FindSectorFromLineTag(state, line, secnum);
        if secnum < 0 {
            break;
        }
        let sec = SectorId(secnum as u32);
        if state.p_setup.sector_mut(sec).specialdata.is_some() {
            continue;
        }
        rtn = true;
        let ceilingheight = state.p_setup.sector_mut(sec).ceilingheight;
        let mut door = vldoor_t::default();
        door.thinker.function = ThinkerFn::Door(T_VerticalDoor);
        door.sector = sec;
        door.kind = kind;
        door.topwait = VDOORWAIT;
        door.speed = (FRACUNIT * 2) as fixed_t;
        match kind {
            VldoorE::vld_blazeClose => {
                door.topheight = P_FindLowestCeilingSurrounding(state, sec);
                door.topheight -= 4 * FRACUNIT;
                door.direction = -1;
                door.speed = (FRACUNIT * 2 * 4) as fixed_t;
                S_StartSound(state, SoundOrigin::Sector(sec), SfxName::sfx_bdcls as i32);
            }
            VldoorE::vld_close => {
                door.topheight = P_FindLowestCeilingSurrounding(state, sec);
                door.topheight -= 4 * FRACUNIT;
                door.direction = -1;
                S_StartSound(state, SoundOrigin::Sector(sec), SfxName::sfx_dorcls as i32);
            }
            VldoorE::vld_close30ThenOpen => {
                door.topheight = ceilingheight;
                door.direction = -1;
                S_StartSound(state, SoundOrigin::Sector(sec), SfxName::sfx_dorcls as i32);
            }
            VldoorE::vld_blazeRaise | VldoorE::vld_blazeOpen => {
                door.direction = 1;
                door.topheight = P_FindLowestCeilingSurrounding(state, sec);
                door.topheight -= 4 * FRACUNIT;
                door.speed = (FRACUNIT * 2 * 4) as fixed_t;
                if door.topheight != ceilingheight {
                    S_StartSound(state, SoundOrigin::Sector(sec), SfxName::sfx_bdopn as i32);
                }
            }
            VldoorE::vld_normal | VldoorE::vld_open => {
                door.direction = 1;
                door.topheight = P_FindLowestCeilingSurrounding(state, sec);
                door.topheight -= 4 * FRACUNIT;
                if door.topheight != ceilingheight {
                    S_StartSound(state, SoundOrigin::Sector(sec), SfxName::sfx_doropn as i32);
                }
            }
            _ => {}
        }
        let door_arena_id = state.p_doors.spawn(door);
        let door_id = P_AddThinker(
            state,
            ThinkerPayload::Door(door_arena_id),
            ThinkerKind::Door,
        );
        state.p_setup.sector_mut(sec).specialdata = Some(SectorSpecial::Door(door_id));
    }
    rtn
}
pub fn EV_VerticalDoor(state: &mut GameState, line: LineId, thing: MobjId) {
    let side: i32 = 0;
    let thing_player = state.p_mobj.mo(thing).player;
    let linev = state.p_setup.line(line);
    let key_message = |has: bool, message: &'static str| if has { None } else { Some(message) };
    if let Some(player_id) = thing_player {
        let (blue, red, yellow) = {
            let p = state.g_game.player_mut(player_id);
            (
                p.cards[CardType::it_bluecard as usize] || p.cards[CardType::it_blueskull as usize],
                p.cards[CardType::it_redcard as usize] || p.cards[CardType::it_redskull as usize],
                p.cards[CardType::it_yellowcard as usize]
                    || p.cards[CardType::it_yellowskull as usize],
            )
        };
        let missing = match linev.special as i32 {
            26 | 32 => key_message(blue, "You need a blue key to open this door"),
            27 | 34 => key_message(yellow, "You need a yellow key to open this door"),
            28 | 33 => key_message(red, "You need a red key to open this door"),
            _ => None,
        };
        if let Some(message) = missing {
            state.g_game.player_mut(player_id).message = Some(message.to_string());
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_oof as i32);
            return;
        }
    } else if matches!(linev.special as i32, 26 | 32 | 27 | 34 | 28 | 33) {
        return;
    }
    let door_sector_id = state.p_setup.sides[linev.sidenum[(side ^ 1) as usize] as usize].sector;
    if let Some(special) = state.p_setup.sector_mut(door_sector_id).specialdata {
        match linev.special as i32 {
            1 | 26 | 27 | 28 | 117 => {
                match special {
                    SectorSpecial::Door(id) => {
                        let door_id = state.p_tick.door_payload(id);
                        let door = state.p_doors.get_mut(door_id).expect("live door");
                        if door.direction == -1 {
                            door.direction = 1;
                        } else {
                            if thing_player.is_none() {
                                return;
                            }
                            door.direction = -1;
                        }
                    }
                    SectorSpecial::Plat(id) => {
                        if thing_player.is_none() {
                            return;
                        }
                        let plat_id = state.p_tick.plat_payload(id);
                        state.p_plats.get_mut(plat_id).expect("live plat").wait = -1;
                    }
                    SectorSpecial::Ceiling(id) => {
                        if thing_player.is_none() {
                            return;
                        }
                        doom_eprintln!(
                            state.platform,
                            "EV_VerticalDoor: Tried to close something that wasn't a door."
                        );
                        let ceiling_id = state.p_tick.ceiling_payload(id);
                        state
                            .p_ceilng
                            .get_mut(ceiling_id)
                            .expect("live ceiling")
                            .direction = -1;
                    }
                    SectorSpecial::Floor(id) => {
                        if thing_player.is_none() {
                            return;
                        }
                        doom_eprintln!(
                            state.platform,
                            "EV_VerticalDoor: Tried to close something that wasn't a door."
                        );
                        let floor_id = state.p_tick.floor_payload(id);
                        state
                            .p_spec
                            .get_floor_mut(floor_id)
                            .expect("live floor")
                            .direction = -1;
                    }
                }
                return;
            }
            _ => {}
        }
    }
    match linev.special as i32 {
        117 | 118 => {
            S_StartSound(
                state,
                SoundOrigin::Sector(door_sector_id),
                SfxName::sfx_bdopn as i32,
            );
        }
        _ => {
            S_StartSound(
                state,
                SoundOrigin::Sector(door_sector_id),
                SfxName::sfx_doropn as i32,
            );
        }
    }
    let mut door = vldoor_t::default();
    door.thinker.function = ThinkerFn::Door(T_VerticalDoor);
    door.sector = door_sector_id;
    door.direction = 1;
    door.speed = (FRACUNIT * 2) as fixed_t;
    door.topwait = VDOORWAIT;
    match linev.special as i32 {
        1 | 26 | 27 | 28 => {
            door.kind = VldoorE::vld_normal;
        }
        31..=34 => {
            door.kind = VldoorE::vld_open;
            state.p_setup.line_mut(line).special = 0;
        }
        117 => {
            door.kind = VldoorE::vld_blazeRaise;
            door.speed = (FRACUNIT * 2 * 4) as fixed_t;
        }
        118 => {
            door.kind = VldoorE::vld_blazeOpen;
            state.p_setup.line_mut(line).special = 0;
            door.speed = (FRACUNIT * 2 * 4) as fixed_t;
        }
        _ => {}
    }
    door.topheight = P_FindLowestCeilingSurrounding(state, door_sector_id);
    door.topheight -= 4 * FRACUNIT;
    let door_arena_id = state.p_doors.spawn(door);
    let door_id = P_AddThinker(
        state,
        ThinkerPayload::Door(door_arena_id),
        ThinkerKind::Door,
    );
    state.p_setup.sector_mut(door_sector_id).specialdata = Some(SectorSpecial::Door(door_id));
}
pub fn P_SpawnDoorCloseIn30(state: &mut GameState, sector: SectorId) {
    let mut door = vldoor_t::default();
    door.thinker.function = ThinkerFn::Door(T_VerticalDoor);
    door.sector = sector;
    door.direction = 0;
    door.kind = VldoorE::vld_normal;
    door.speed = (FRACUNIT * 2) as fixed_t;
    door.topcountdown = 30 * TICRATE;
    let door_arena_id = state.p_doors.spawn(door);
    let door_id = P_AddThinker(
        state,
        ThinkerPayload::Door(door_arena_id),
        ThinkerKind::Door,
    );
    let sec = state.p_setup.sector_mut(sector);
    sec.specialdata = Some(SectorSpecial::Door(door_id));
    sec.special = 0;
}
pub fn P_SpawnDoorRaiseIn5Mins(state: &mut GameState, sector: SectorId) {
    let mut door = vldoor_t::default();
    door.thinker.function = ThinkerFn::Door(T_VerticalDoor);
    door.sector = sector;
    door.direction = 2;
    door.kind = VldoorE::vld_raiseIn5Mins;
    door.speed = (FRACUNIT * 2) as fixed_t;
    door.topheight = P_FindLowestCeilingSurrounding(state, sector);
    door.topheight -= 4 * FRACUNIT;
    door.topwait = VDOORWAIT;
    door.topcountdown = 5 * 60 * TICRATE;
    let door_arena_id = state.p_doors.spawn(door);
    let door_id = P_AddThinker(
        state,
        ThinkerPayload::Door(door_arena_id),
        ThinkerKind::Door,
    );
    let sec = state.p_setup.sector_mut(sector);
    sec.specialdata = Some(SectorSpecial::Door(door_id));
    sec.special = 0;
}
