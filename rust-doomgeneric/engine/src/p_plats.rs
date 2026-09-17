use crate::doomdef::TICRATE;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACUNIT;
use crate::m_random::P_Random;
use crate::p_floor::ResultE;
use crate::p_floor::T_MovePlane;
use crate::p_mobj::sector_t;
use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::plat_t;
use crate::p_spec::P_FindHighestFloorSurrounding;
use crate::p_spec::P_FindLowestFloorSurrounding;
use crate::p_spec::P_FindNextHighestFloor;
use crate::p_spec::P_FindSectorFromLineTag;
use crate::p_tick::P_AddThinker;
use crate::p_tick::P_RemoveThinker;
use crate::p_tick::P_ThinkerRaw;
use crate::p_tick::ThinkerId;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::{sfx_pstart, sfx_pstop, sfx_stnmov};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlatE {
    up = 0,
    down = 1,
    waiting = 2,
    in_stasis = 3,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlattypeE {
    perpetualRaise = 0,
    downWaitUpStay = 1,
    raiseAndChange = 2,
    raiseToNearestAndChange = 3,
    blazeDWUS = 4,
}
pub const PLATWAIT: i32 = 3;
pub const PLATSPEED: i32 = FRACUNIT;
pub const MAXPLATS: i32 = 30;

// Generation-checked handle into PPlatsState's arena -- mirrors DoorId.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PlatId {
    index: u32,
    generation: u32,
}

struct PlatSlot {
    generation: u32,
    plat: Option<Box<plat_t>>,
}

pub struct PPlatsState {
    pub activeplats: [Option<ThinkerId>; 30],
    plats: Vec<PlatSlot>,
    free_list: Vec<u32>,
}

impl PPlatsState {
    pub const fn new() -> Self {
        PPlatsState {
            activeplats: [None; 30],
            plats: Vec::new(),
            free_list: Vec::new(),
        }
    }

    // Moves a fully-defaulted (then caller-filled) plat_t onto the heap and
    // hands back both a stable generation-checked handle (stored in
    // ThinkerNode's payload by p_tick.rs, replacing what used to be a bare
    // raw pointer there) and a raw pointer for the caller's immediate
    // post-spawn field writes -- mirrors PDoorsState::spawn exactly.
    pub fn spawn(&mut self, value: plat_t) -> (PlatId, *mut plat_t) {
        let (index, generation) = if let Some(index) = self.free_list.pop() {
            let slot = &mut self.plats[index as usize];
            slot.generation = slot.generation.wrapping_add(1);
            (index, slot.generation)
        } else {
            let index = self.plats.len() as u32;
            self.plats.push(PlatSlot {
                generation: 0,
                plat: None,
            });
            (index, 0)
        };
        let id = PlatId { index, generation };
        let mut boxed = Box::new(value);
        let ptr = boxed.as_mut() as *mut plat_t;
        self.plats[index as usize].plat = Some(boxed);
        (id, ptr)
    }

    // Fallible materialization: None if the id is stale. Used by
    // p_tick.rs's P_ThinkerRaw to resolve a Plat-kind ThinkerNode's payload
    // back into the raw pointer every T_* function still expects.
    pub fn get(&self, id: PlatId) -> Option<*mut plat_t> {
        self.plats
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.plat.as_deref())
            .map(|r| r as *const plat_t as *mut plat_t)
    }

    // Called once, from P_RunThinkers' reaper, when a Plat-kind thinker is
    // reaped.
    pub fn dealloc(&mut self, id: PlatId) {
        if let Some(slot) = self.plats.get_mut(id.index as usize) {
            if slot.generation == id.generation {
                slot.plat = None;
                self.free_list.push(id.index);
            }
        }
    }
}

pub fn T_PlatRaise(state: &mut GameState, id: PlatId) {
    let plat = state
        .p_plats
        .get(id)
        .expect("ThinkerFn::Plat id must reference a live plat");
    unsafe {
        let mut res: ResultE = ResultE::ok;
        let sec: *mut sector_t = state.p_setup.sector_mut((*plat).sector);
        match (*plat).status {
            PlatE::up => {
                res = T_MovePlane(
                    state,
                    sec,
                    (*plat).speed,
                    (*plat).high,
                    (*plat).crush,
                    0_i32,
                    1_i32,
                );
                if ((*plat).type_0 == PlattypeE::raiseAndChange || (*plat).type_0 == PlattypeE::raiseToNearestAndChange) && state.p_tick.leveltime & 7_i32 == 0 {
                    S_StartSound(
                        state,
                        SoundOrigin::Sector((*plat).sector),
                        sfx_stnmov as i32,
                    );
                }
                if res == ResultE::crushed && !(*plat).crush {
                    (*plat).count = (*plat).wait;
                    (*plat).status = PlatE::down;
                    S_StartSound(
                        state,
                        SoundOrigin::Sector((*plat).sector),
                        sfx_pstart as i32,
                    );
                } else if res == ResultE::pastdest {
                    (*plat).count = (*plat).wait;
                    (*plat).status = PlatE::waiting;
                    S_StartSound(state, SoundOrigin::Sector((*plat).sector), sfx_pstop as i32);
                    match (*plat).type_0 {
                        PlattypeE::blazeDWUS | PlattypeE::downWaitUpStay => {
                            P_RemoveActivePlat(state, plat);
                        }
                        PlattypeE::raiseAndChange | PlattypeE::raiseToNearestAndChange => {
                            P_RemoveActivePlat(state, plat);
                        }
                        PlattypeE::perpetualRaise => {}
                    }
                }
            }
            PlatE::down => {
                res = T_MovePlane(state, sec, (*plat).speed, (*plat).low, false, 0_i32, -1_i32);
                if res == ResultE::pastdest {
                    (*plat).count = (*plat).wait;
                    (*plat).status = PlatE::waiting;
                    S_StartSound(state, SoundOrigin::Sector((*plat).sector), sfx_pstop as i32);
                }
            }
            PlatE::waiting => {
                (*plat).count -= 1;
                if (*plat).count == 0 {
                    if (*sec).floorheight == (*plat).low {
                        (*plat).status = PlatE::up;
                    } else {
                        (*plat).status = PlatE::down;
                    }
                    S_StartSound(
                        state,
                        SoundOrigin::Sector((*plat).sector),
                        sfx_pstart as i32,
                    );
                }
            }
            PlatE::in_stasis => {}
        };
    }
}
pub unsafe fn EV_DoPlat(
    state: &mut GameState,
    mut line: LineId,
    mut type_0: PlattypeE,
    mut amount: i32,
) -> i32 {
    let mut plat: *mut plat_t = ::core::ptr::null_mut::<plat_t>();
    let mut secnum: i32 = 0;
    let mut rtn: i32 = 0;
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let linev = state.p_setup.line(line);
    secnum = -1_i32;
    rtn = 0_i32;
    match type_0 {
        PlattypeE::perpetualRaise => {
            P_ActivateInStasis(state, linev.tag as i32);
        }
        _ => {}
    }
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
        let (plat_arena_id, plat_ptr) = state.p_plats.spawn(plat_t::default());
        plat = plat_ptr;
        let plat_id = P_AddThinker(state, ThinkerPayload::Plat(plat_arena_id), ThinkerKind::Plat);
        (*plat).type_0 = type_0;
        (*plat).sector = SectorId(secnum as u32);
        (*sec).specialdata = Some(SectorSpecial::Plat(plat_id));
        (*plat).thinker.function = ThinkerFn::Plat(T_PlatRaise);
        (*plat).crush = false;
        (*plat).tag = linev.tag as i32;
        match type_0 {
            PlattypeE::raiseToNearestAndChange => {
                (*plat).speed = (PLATSPEED / 2_i32) as fixed_t;
                let neighbor_sector_id = state.p_setup.sides[linev.sidenum[0] as usize].sector;
                (*sec).floorpic = state.p_setup.sector_mut(neighbor_sector_id).floorpic;
                (*plat).high = P_FindNextHighestFloor(state, sec, (*sec).floorheight);
                (*plat).wait = 0_i32;
                (*plat).status = PlatE::up;
                (*sec).special = 0_i16;
                S_StartSound(
                    state,
                    SoundOrigin::Sector(SectorId(secnum as u32)),
                    sfx_stnmov as i32,
                );
            }
            PlattypeE::raiseAndChange => {
                (*plat).speed = (PLATSPEED / 2_i32) as fixed_t;
                let neighbor_sector_id = state.p_setup.sides[linev.sidenum[0] as usize].sector;
                (*sec).floorpic = state.p_setup.sector_mut(neighbor_sector_id).floorpic;
                (*plat).high = ((*sec).floorheight + amount * FRACUNIT) as fixed_t;
                (*plat).wait = 0_i32;
                (*plat).status = PlatE::up;
                S_StartSound(
                    state,
                    SoundOrigin::Sector(SectorId(secnum as u32)),
                    sfx_stnmov as i32,
                );
            }
            PlattypeE::downWaitUpStay => {
                (*plat).speed = (PLATSPEED * 4_i32) as fixed_t;
                (*plat).low = P_FindLowestFloorSurrounding(state, sec);
                if (*plat).low > (*sec).floorheight {
                    (*plat).low = (*sec).floorheight;
                }
                (*plat).high = (*sec).floorheight;
                (*plat).wait = TICRATE * PLATWAIT;
                (*plat).status = PlatE::down;
                S_StartSound(
                    state,
                    SoundOrigin::Sector(SectorId(secnum as u32)),
                    sfx_pstart as i32,
                );
            }
            PlattypeE::blazeDWUS => {
                (*plat).speed = (PLATSPEED * 8_i32) as fixed_t;
                (*plat).low = P_FindLowestFloorSurrounding(state, sec);
                if (*plat).low > (*sec).floorheight {
                    (*plat).low = (*sec).floorheight;
                }
                (*plat).high = (*sec).floorheight;
                (*plat).wait = TICRATE * PLATWAIT;
                (*plat).status = PlatE::down;
                S_StartSound(
                    state,
                    SoundOrigin::Sector(SectorId(secnum as u32)),
                    sfx_pstart as i32,
                );
            }
            PlattypeE::perpetualRaise => {
                (*plat).speed = PLATSPEED as fixed_t;
                (*plat).low = P_FindLowestFloorSurrounding(state, sec);
                if (*plat).low > (*sec).floorheight {
                    (*plat).low = (*sec).floorheight;
                }
                (*plat).high = P_FindHighestFloorSurrounding(state, sec);
                if (*plat).high < (*sec).floorheight {
                    (*plat).high = (*sec).floorheight;
                }
                (*plat).wait = TICRATE * PLATWAIT;
                (*plat).status = if P_Random(&mut state.m_random) & 1_i32 != 0 {
                    PlatE::down
                } else {
                    PlatE::up
                };
                S_StartSound(
                    state,
                    SoundOrigin::Sector(SectorId(secnum as u32)),
                    sfx_pstart as i32,
                );
            }
        }
        P_AddActivePlat(&mut state.p_plats, plat_id);
    }
    rtn
}
pub unsafe fn P_ActivateInStasis(state: &mut GameState, mut tag: i32) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < MAXPLATS {
        if let Some(id) = state.p_plats.activeplats[i as usize] {
            let plat = P_ThinkerRaw(state, id) as *mut plat_t;
            if (*plat).tag == tag && (*plat).status == PlatE::in_stasis {
                (*plat).status = (*plat).oldstatus;
                (*plat).thinker.function = ThinkerFn::Plat(T_PlatRaise);
            }
        }
        i += 1;
    }
}
pub unsafe fn EV_StopPlat(state: &mut GameState, mut tag: i32) {
    let mut j: i32 = 0;
    j = 0_i32;
    while j < MAXPLATS {
        if let Some(id) = state.p_plats.activeplats[j as usize] {
            let plat = P_ThinkerRaw(state, id) as *mut plat_t;
            if (*plat).status != PlatE::in_stasis && (*plat).tag == tag {
                (*plat).oldstatus = (*plat).status;
                (*plat).status = PlatE::in_stasis;
                (*plat).thinker.function = ThinkerFn::Paused;
            }
        }
        j += 1;
    }
}
pub fn P_AddActivePlat(state: &mut PPlatsState, id: ThinkerId) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < MAXPLATS {
        if state.activeplats[i as usize].is_none() {
            state.activeplats[i as usize] = Some(id);
            return;
        }
        i += 1;
    }
    I_Error("P_AddActivePlat: no more plats!");
}
pub unsafe fn P_RemoveActivePlat(state: &mut GameState, mut plat: *mut plat_t) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < MAXPLATS {
        if let Some(id) = state.p_plats.activeplats[i as usize] {
            if P_ThinkerRaw(state, id) as *mut plat_t == plat {
                state.p_setup.sector_mut((*plat).sector).specialdata = None;
                P_RemoveThinker(&raw mut (*plat).thinker);
                state.p_plats.activeplats[i as usize] = None;
                return;
            }
        }
        i += 1;
    }
    I_Error("P_RemoveActivePlat: can't find plat!");
}
