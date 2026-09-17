use crate::d_player::player_t;
use crate::doomdef::TICRATE;
use crate::game_state::GameState;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACUNIT;
use crate::p_floor::ResultE;
use crate::p_floor::T_MovePlane;
use crate::p_inter::CardType;
use crate::p_mobj::mobj_t;
use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_mobj::{sector_t, thinker_t};
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::P_FindLowestCeilingSurrounding;
use crate::p_spec::P_FindSectorFromLineTag;
use crate::p_spec::{ceiling_t, floormove_t, plat_t};
use crate::p_tick::P_AddThinker;
use crate::p_tick::P_RemoveThinker;
use crate::p_tick::P_ThinkerRaw;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::{sfx_bdcls, sfx_bdopn, sfx_dorcls, sfx_doropn, sfx_oof};
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
#[repr(C)]
pub struct vldoor_t {
    pub thinker: thinker_t,
    pub type_0: VldoorE,
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
            type_0: VldoorE::vld_normal,
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
    pub fn spawn(&mut self, value: vldoor_t) -> (DoorId, *mut vldoor_t) {
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
        let mut boxed = Box::new(value);
        let ptr = boxed.as_mut() as *mut vldoor_t;
        self.doors[index as usize].door = Some(boxed);
        (id, ptr)
    }

    // Fallible materialization: None if the id is stale. Used by
    // p_tick.rs's P_ThinkerRaw to resolve a Door-kind ThinkerNode's payload
    // back into the raw pointer every T_* function still expects.
    pub fn get(&self, id: DoorId) -> Option<*mut vldoor_t> {
        self.doors
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.door.as_deref())
            .map(|r| r as *const vldoor_t as *mut vldoor_t)
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
pub unsafe fn T_VerticalDoor(state: &mut GameState, mut door: *mut vldoor_t) {
    let mut res: ResultE = ResultE::ok;
    let sec: *mut sector_t = state.p_setup.sector_mut((*door).sector);
    match (*door).direction {
        0 => {
            (*door).topcountdown -= 1;
            if (*door).topcountdown == 0 {
                match (*door).type_0 {
                    VldoorE::vld_blazeRaise => {
                        (*door).direction = -1_i32;
                        S_StartSound(state, SoundOrigin::Sector((*door).sector), sfx_bdcls as i32);
                    }
                    VldoorE::vld_normal => {
                        (*door).direction = -1_i32;
                        S_StartSound(
                            state,
                            SoundOrigin::Sector((*door).sector),
                            sfx_dorcls as i32,
                        );
                    }
                    VldoorE::vld_close30ThenOpen => {
                        (*door).direction = 1_i32;
                        S_StartSound(
                            state,
                            SoundOrigin::Sector((*door).sector),
                            sfx_doropn as i32,
                        );
                    }
                    _ => {}
                }
            }
        }
        2 => {
            (*door).topcountdown -= 1;
            if (*door).topcountdown == 0 {
                match (*door).type_0 {
                    VldoorE::vld_raiseIn5Mins => {
                        (*door).direction = 1_i32;
                        (*door).type_0 = VldoorE::vld_normal;
                        S_StartSound(
                            state,
                            SoundOrigin::Sector((*door).sector),
                            sfx_doropn as i32,
                        );
                    }
                    _ => {}
                }
            }
        }
        -1 => {
            res = T_MovePlane(
                state,
                sec,
                (*door).speed,
                (*sec).floorheight,
                false,
                1_i32,
                (*door).direction,
            );
            if res == ResultE::pastdest {
                match (*door).type_0 {
                    VldoorE::vld_blazeRaise | VldoorE::vld_blazeClose => {
                        (*sec).specialdata = None;
                        P_RemoveThinker(&raw mut (*door).thinker);
                        S_StartSound(state, SoundOrigin::Sector((*door).sector), sfx_bdcls as i32);
                    }
                    VldoorE::vld_normal | VldoorE::vld_close => {
                        (*sec).specialdata = None;
                        P_RemoveThinker(&raw mut (*door).thinker);
                    }
                    VldoorE::vld_close30ThenOpen => {
                        (*door).direction = 0_i32;
                        (*door).topcountdown = TICRATE * 30_i32;
                    }
                    _ => {}
                }
            } else if res == ResultE::crushed {
                match (*door).type_0 {
                    VldoorE::vld_blazeClose | VldoorE::vld_close => {}
                    _ => {
                        (*door).direction = 1_i32;
                        S_StartSound(
                            state,
                            SoundOrigin::Sector((*door).sector),
                            sfx_doropn as i32,
                        );
                    }
                }
            }
        }
        1 => {
            res = T_MovePlane(
                state,
                sec,
                (*door).speed,
                (*door).topheight,
                false,
                1_i32,
                (*door).direction,
            );
            if res == ResultE::pastdest {
                match (*door).type_0 {
                    VldoorE::vld_blazeRaise | VldoorE::vld_normal => {
                        (*door).direction = 0_i32;
                        (*door).topcountdown = (*door).topwait;
                    }
                    VldoorE::vld_close30ThenOpen | VldoorE::vld_blazeOpen | VldoorE::vld_open => {
                        (*sec).specialdata = None;
                        P_RemoveThinker(&raw mut (*door).thinker);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    };
}
pub unsafe fn EV_DoLockedDoor(
    state: &mut GameState,
    mut line: LineId,
    mut type_0: VldoorE,
    mut thing: *mut mobj_t,
) -> i32 {
    let mut p: *mut player_t = ::core::ptr::null_mut::<player_t>();
    let thing_player = (*thing).player;
    if thing_player.is_none() {
        return 0_i32;
    }
    p = state.g_game.player_mut(thing_player.unwrap());
    match state.p_setup.line(line).special as i32 {
        99 | 133 => {
            if p.is_null() {
                return 0_i32;
            }
            if !(*p).cards[CardType::it_bluecard as usize]
                && !(*p).cards[CardType::it_blueskull as usize]
            {
                (*p).message = Some("You need a blue key to activate this object".to_string());
                S_StartSound(state, SoundOrigin::None, sfx_oof as i32);
                return 0_i32;
            }
        }
        134 | 135 => {
            if p.is_null() {
                return 0_i32;
            }
            if !(*p).cards[CardType::it_redcard as usize]
                && !(*p).cards[CardType::it_redskull as usize]
            {
                (*p).message = Some("You need a red key to activate this object".to_string());
                S_StartSound(state, SoundOrigin::None, sfx_oof as i32);
                return 0_i32;
            }
        }
        136 | 137 => {
            if p.is_null() {
                return 0_i32;
            }
            if !(*p).cards[CardType::it_yellowcard as usize]
                && !(*p).cards[CardType::it_yellowskull as usize]
            {
                (*p).message = Some("You need a yellow key to activate this object".to_string());
                S_StartSound(state, SoundOrigin::None, sfx_oof as i32);
                return 0_i32;
            }
        }
        _ => {}
    }
    EV_DoDoor(state, line, type_0)
}
pub unsafe fn EV_DoDoor(state: &mut GameState, mut line: LineId, mut type_0: VldoorE) -> i32 {
    let mut secnum: i32 = 0;
    let mut rtn: i32 = 0;
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut door: *mut vldoor_t = ::core::ptr::null_mut::<vldoor_t>();
    secnum = -1_i32;
    rtn = 0_i32;
    loop {
        secnum = P_FindSectorFromLineTag(state, line, secnum);
        if secnum < 0_i32 {
            break;
        }
        sec = state.p_setup.sector_mut(SectorId(secnum as u32));
        if (*sec).specialdata.is_some() {
            continue;
        }
        rtn = 1_i32;
        let (door_arena_id, door_ptr) = state.p_doors.spawn(vldoor_t::default());
        door = door_ptr;
        let door_id = P_AddThinker(state, ThinkerPayload::Door(door_arena_id), ThinkerKind::Door);
        (*sec).specialdata = Some(SectorSpecial::Door(door_id));
        (*door).thinker.function = ThinkerFn::Door(T_VerticalDoor);
        (*door).sector = SectorId(secnum as u32);
        (*door).type_0 = type_0;
        (*door).topwait = VDOORWAIT;
        (*door).speed = (FRACUNIT * 2_i32) as fixed_t;
        match type_0 {
            VldoorE::vld_blazeClose => {
                (*door).topheight = P_FindLowestCeilingSurrounding(state, sec);
                (*door).topheight -= 4_i32 * FRACUNIT;
                (*door).direction = -1_i32;
                (*door).speed = (FRACUNIT * 2_i32 * 4_i32) as fixed_t;
                S_StartSound(
                    state,
                    SoundOrigin::Sector(SectorId(secnum as u32)),
                    sfx_bdcls as i32,
                );
            }
            VldoorE::vld_close => {
                (*door).topheight = P_FindLowestCeilingSurrounding(state, sec);
                (*door).topheight -= 4_i32 * FRACUNIT;
                (*door).direction = -1_i32;
                S_StartSound(
                    state,
                    SoundOrigin::Sector(SectorId(secnum as u32)),
                    sfx_dorcls as i32,
                );
            }
            VldoorE::vld_close30ThenOpen => {
                (*door).topheight = (*sec).ceilingheight;
                (*door).direction = -1_i32;
                S_StartSound(
                    state,
                    SoundOrigin::Sector(SectorId(secnum as u32)),
                    sfx_dorcls as i32,
                );
            }
            VldoorE::vld_blazeRaise | VldoorE::vld_blazeOpen => {
                (*door).direction = 1_i32;
                (*door).topheight = P_FindLowestCeilingSurrounding(state, sec);
                (*door).topheight -= 4_i32 * FRACUNIT;
                (*door).speed = (FRACUNIT * 2_i32 * 4_i32) as fixed_t;
                if (*door).topheight != (*sec).ceilingheight {
                    S_StartSound(
                        state,
                        SoundOrigin::Sector(SectorId(secnum as u32)),
                        sfx_bdopn as i32,
                    );
                }
            }
            VldoorE::vld_normal | VldoorE::vld_open => {
                (*door).direction = 1_i32;
                (*door).topheight = P_FindLowestCeilingSurrounding(state, sec);
                (*door).topheight -= 4_i32 * FRACUNIT;
                if (*door).topheight != (*sec).ceilingheight {
                    S_StartSound(
                        state,
                        SoundOrigin::Sector(SectorId(secnum as u32)),
                        sfx_doropn as i32,
                    );
                }
            }
            _ => {}
        }
    }
    rtn
}
pub unsafe fn EV_VerticalDoor(state: &mut GameState, mut line: LineId, mut thing: *mut mobj_t) {
    let mut player: *mut player_t = ::core::ptr::null_mut::<player_t>();
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut door: *mut vldoor_t = ::core::ptr::null_mut::<vldoor_t>();
    let mut side: i32 = 0;
    side = 0_i32;
    player = match (*thing).player {
        Some(id) => state.g_game.player_mut(id),
        None => ::core::ptr::null_mut::<player_t>(),
    };
    let linev = state.p_setup.line(line);
    match linev.special as i32 {
        26 | 32 => {
            if player.is_null() {
                return;
            }
            if !(*player).cards[CardType::it_bluecard as usize]
                && !(*player).cards[CardType::it_blueskull as usize]
            {
                (*player).message = Some("You need a blue key to open this door".to_string());
                S_StartSound(state, SoundOrigin::None, sfx_oof as i32);
                return;
            }
        }
        27 | 34 => {
            if player.is_null() {
                return;
            }
            if !(*player).cards[CardType::it_yellowcard as usize]
                && !(*player).cards[CardType::it_yellowskull as usize]
            {
                (*player).message = Some("You need a yellow key to open this door".to_string());
                S_StartSound(state, SoundOrigin::None, sfx_oof as i32);
                return;
            }
        }
        28 | 33 => {
            if player.is_null() {
                return;
            }
            if !(*player).cards[CardType::it_redcard as usize]
                && !(*player).cards[CardType::it_redskull as usize]
            {
                (*player).message = Some("You need a red key to open this door".to_string());
                S_StartSound(state, SoundOrigin::None, sfx_oof as i32);
                return;
            }
        }
        _ => {}
    }
    let door_sector_id =
        state.p_setup.sides[linev.sidenum[(side ^ 1_i32) as usize] as usize].sector;
    sec = state.p_setup.sector_mut(door_sector_id);
    if let Some(special) = (*sec).specialdata {
        match linev.special as i32 {
            1 | 26 | 27 | 28 | 117 => {
                match special {
                    SectorSpecial::Door(id) => {
                        door = P_ThinkerRaw(state, id) as *mut vldoor_t;
                        if (*door).direction == -1_i32 {
                            (*door).direction = 1_i32;
                        } else {
                            if (*thing).player.is_none() {
                                return;
                            }
                            (*door).direction = -1_i32;
                        }
                    }
                    SectorSpecial::Plat(id) => {
                        if (*thing).player.is_none() {
                            return;
                        }
                        let plat = P_ThinkerRaw(state, id) as *mut plat_t;
                        (*plat).wait = -1_i32;
                    }
                    SectorSpecial::Ceiling(id) => {
                        if (*thing).player.is_none() {
                            return;
                        }
                        eprintln!("EV_VerticalDoor: Tried to close something that wasn't a door.");
                        let ceiling = P_ThinkerRaw(state, id) as *mut ceiling_t;
                        (*ceiling).direction = -1_i32;
                    }
                    SectorSpecial::Floor(id) => {
                        if (*thing).player.is_none() {
                            return;
                        }
                        eprintln!("EV_VerticalDoor: Tried to close something that wasn't a door.");
                        let floor = P_ThinkerRaw(state, id) as *mut floormove_t;
                        (*floor).direction = -1_i32;
                    }
                }
                return;
            }
            _ => {}
        }
    }
    match linev.special as i32 {
        117 | 118 => {
            S_StartSound(state, SoundOrigin::Sector(door_sector_id), sfx_bdopn as i32);
        }
        1 | 31 => {
            S_StartSound(
                state,
                SoundOrigin::Sector(door_sector_id),
                sfx_doropn as i32,
            );
        }
        _ => {
            S_StartSound(
                state,
                SoundOrigin::Sector(door_sector_id),
                sfx_doropn as i32,
            );
        }
    }
    let (door_arena_id, door_ptr) = state.p_doors.spawn(vldoor_t::default());
    door = door_ptr;
    let door_id = P_AddThinker(state, ThinkerPayload::Door(door_arena_id), ThinkerKind::Door);
    (*sec).specialdata = Some(SectorSpecial::Door(door_id));
    (*door).thinker.function = ThinkerFn::Door(T_VerticalDoor);
    (*door).sector = door_sector_id;
    (*door).direction = 1_i32;
    (*door).speed = (FRACUNIT * 2_i32) as fixed_t;
    (*door).topwait = VDOORWAIT;
    match linev.special as i32 {
        1 | 26 | 27 | 28 => {
            (*door).type_0 = VldoorE::vld_normal;
        }
        31 | 32 | 33 | 34 => {
            (*door).type_0 = VldoorE::vld_open;
            state.p_setup.line_mut(line).special = 0_i16;
        }
        117 => {
            (*door).type_0 = VldoorE::vld_blazeRaise;
            (*door).speed = (FRACUNIT * 2_i32 * 4_i32) as fixed_t;
        }
        118 => {
            (*door).type_0 = VldoorE::vld_blazeOpen;
            state.p_setup.line_mut(line).special = 0_i16;
            (*door).speed = (FRACUNIT * 2_i32 * 4_i32) as fixed_t;
        }
        _ => {}
    }
    (*door).topheight = P_FindLowestCeilingSurrounding(state, sec);
    (*door).topheight -= 4_i32 * FRACUNIT;
}
pub unsafe fn P_SpawnDoorCloseIn30(state: &mut GameState, mut sector: SectorId) {
    let mut door: *mut vldoor_t = ::core::ptr::null_mut::<vldoor_t>();
    let sec: *mut sector_t = state.p_setup.sector_mut(sector);
    let (door_arena_id, door_ptr) = state.p_doors.spawn(vldoor_t::default());
    door = door_ptr;
    let door_id = P_AddThinker(state, ThinkerPayload::Door(door_arena_id), ThinkerKind::Door);
    (*sec).specialdata = Some(SectorSpecial::Door(door_id));
    (*sec).special = 0_i16;
    (*door).thinker.function = ThinkerFn::Door(T_VerticalDoor);
    (*door).sector = sector;
    (*door).direction = 0_i32;
    (*door).type_0 = VldoorE::vld_normal;
    (*door).speed = (FRACUNIT * 2_i32) as fixed_t;
    (*door).topcountdown = 30_i32 * TICRATE;
}
pub unsafe fn P_SpawnDoorRaiseIn5Mins(state: &mut GameState, mut sector: SectorId) {
    let mut door: *mut vldoor_t = ::core::ptr::null_mut::<vldoor_t>();
    let sec: *mut sector_t = state.p_setup.sector_mut(sector);
    let (door_arena_id, door_ptr) = state.p_doors.spawn(vldoor_t::default());
    door = door_ptr;
    let door_id = P_AddThinker(state, ThinkerPayload::Door(door_arena_id), ThinkerKind::Door);
    (*sec).specialdata = Some(SectorSpecial::Door(door_id));
    (*sec).special = 0_i16;
    (*door).thinker.function = ThinkerFn::Door(T_VerticalDoor);
    (*door).sector = sector;
    (*door).direction = 2_i32;
    (*door).type_0 = VldoorE::vld_raiseIn5Mins;
    (*door).speed = (FRACUNIT * 2_i32) as fixed_t;
    (*door).topheight = P_FindLowestCeilingSurrounding(state, sec);
    (*door).topheight -= 4_i32 * FRACUNIT;
    (*door).topwait = VDOORWAIT;
    (*door).topcountdown = 5_i32 * 60_i32 * TICRATE;
}
