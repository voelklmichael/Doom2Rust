use crate::game_state::GameState;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACUNIT;
use crate::p_floor::ResultE;
use crate::p_floor::T_MovePlane;
use crate::p_mobj::sector_t;
use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::ceiling_t;
use crate::p_spec::P_FindHighestCeilingSurrounding;
use crate::p_spec::P_FindSectorFromLineTag;
use crate::p_tick::P_AddThinker;
use crate::p_tick::P_RemoveThinker;
use crate::p_tick::P_ThinkerRaw;
use crate::p_tick::ThinkerId;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::{sfx_pstop, sfx_stnmov};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CeilingE {
    lowerToFloor = 0,
    raiseToHighest = 1,
    lowerAndCrush = 2,
    crushAndRaise = 3,
    fastCrushAndRaise = 4,
    silentCrushAndRaise = 5,
}
pub const CEILSPEED: i32 = FRACUNIT;
pub const MAXCEILINGS: i32 = 30;

// Generation-checked handle into PCeilngState's arena -- mirrors DoorId.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct CeilingId {
    index: u32,
    generation: u32,
}

struct CeilingSlot {
    generation: u32,
    ceiling: Option<Box<ceiling_t>>,
}

pub struct PCeilngState {
    pub activeceilings: [Option<ThinkerId>; 30],
    ceilings: Vec<CeilingSlot>,
    free_list: Vec<u32>,
}

impl PCeilngState {
    pub const fn new() -> Self {
        PCeilngState {
            activeceilings: [None; 30],
            ceilings: Vec::new(),
            free_list: Vec::new(),
        }
    }

    // Moves a fully-defaulted (then caller-filled) ceiling_t onto the heap
    // and hands back both a stable generation-checked handle (stored in
    // ThinkerNode's payload by p_tick.rs, replacing what used to be a bare
    // raw pointer there) and a raw pointer for the caller's immediate
    // post-spawn field writes -- mirrors PDoorsState::spawn exactly.
    pub fn spawn(&mut self, value: ceiling_t) -> (CeilingId, *mut ceiling_t) {
        let (index, generation) = if let Some(index) = self.free_list.pop() {
            let slot = &mut self.ceilings[index as usize];
            slot.generation = slot.generation.wrapping_add(1);
            (index, slot.generation)
        } else {
            let index = self.ceilings.len() as u32;
            self.ceilings.push(CeilingSlot {
                generation: 0,
                ceiling: None,
            });
            (index, 0)
        };
        let id = CeilingId { index, generation };
        let mut boxed = Box::new(value);
        let ptr = boxed.as_mut() as *mut ceiling_t;
        self.ceilings[index as usize].ceiling = Some(boxed);
        (id, ptr)
    }

    // Fallible materialization: None if the id is stale. Used by
    // p_tick.rs's P_ThinkerRaw to resolve a Ceiling-kind ThinkerNode's
    // payload back into the raw pointer every T_* function still expects.
    pub fn get(&self, id: CeilingId) -> Option<*mut ceiling_t> {
        self.ceilings
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.ceiling.as_deref())
            .map(|r| r as *const ceiling_t as *mut ceiling_t)
    }

    // Called once, from P_RunThinkers' reaper, when a Ceiling-kind thinker
    // is reaped.
    pub fn dealloc(&mut self, id: CeilingId) {
        if let Some(slot) = self.ceilings.get_mut(id.index as usize) {
            if slot.generation == id.generation {
                slot.ceiling = None;
                self.free_list.push(id.index);
            }
        }
    }
}

pub fn T_MoveCeiling(state: &mut GameState, id: CeilingId) {
    let ceiling = state
        .p_ceilng
        .get(id)
        .expect("ThinkerFn::Ceiling id must reference a live ceiling");
    unsafe {
        let mut res: ResultE = ResultE::ok;
        let sec: *mut sector_t = state.p_setup.sector_mut((*ceiling).sector);
        match (*ceiling).direction {
            1 => {
                res = T_MovePlane(
                    state,
                    sec,
                    (*ceiling).speed,
                    (*ceiling).topheight,
                    false,
                    1_i32,
                    (*ceiling).direction,
                );
                if state.p_tick.leveltime & 7_i32 == 0 {
                    match (*ceiling).type_0 {
                        CeilingE::silentCrushAndRaise => {}
                        _ => {
                            S_StartSound(
                                state,
                                SoundOrigin::Sector((*ceiling).sector),
                                sfx_stnmov as i32,
                            );
                        }
                    }
                }
                if res == ResultE::pastdest {
                    let mut current_block_7: u64;
                    match (*ceiling).type_0 {
                        CeilingE::raiseToHighest => {
                            P_RemoveActiveCeiling(state, ceiling);
                            current_block_7 = 10599921512955367680;
                        }
                        CeilingE::silentCrushAndRaise => {
                            S_StartSound(
                                state,
                                SoundOrigin::Sector((*ceiling).sector),
                                sfx_pstop as i32,
                            );
                            current_block_7 = 16040908003852494439;
                        }
                        CeilingE::fastCrushAndRaise | CeilingE::crushAndRaise => {
                            current_block_7 = 16040908003852494439;
                        }
                        _ => {
                            current_block_7 = 10599921512955367680;
                        }
                    }
                    match current_block_7 {
                        16040908003852494439 => {
                            (*ceiling).direction = -1_i32;
                        }
                        _ => {}
                    }
                }
            }
            -1 => {
                res = T_MovePlane(
                    state,
                    sec,
                    (*ceiling).speed,
                    (*ceiling).bottomheight,
                    (*ceiling).crush,
                    1_i32,
                    (*ceiling).direction,
                );
                if state.p_tick.leveltime & 7_i32 == 0 {
                    match (*ceiling).type_0 {
                        CeilingE::silentCrushAndRaise => {}
                        _ => {
                            S_StartSound(
                                state,
                                SoundOrigin::Sector((*ceiling).sector),
                                sfx_stnmov as i32,
                            );
                        }
                    }
                }
                if res == ResultE::pastdest {
                    let mut current_block_19: u64;
                    match (*ceiling).type_0 {
                        CeilingE::silentCrushAndRaise => {
                            S_StartSound(
                                state,
                                SoundOrigin::Sector((*ceiling).sector),
                                sfx_pstop as i32,
                            );
                            current_block_19 = 3850642056257311267;
                        }
                        CeilingE::crushAndRaise => {
                            current_block_19 = 3850642056257311267;
                        }
                        CeilingE::fastCrushAndRaise => {
                            current_block_19 = 14600216857840559743;
                        }
                        CeilingE::lowerAndCrush | CeilingE::lowerToFloor => {
                            P_RemoveActiveCeiling(state, ceiling);
                            current_block_19 = 16924917904204750491;
                        }
                        _ => {
                            current_block_19 = 16924917904204750491;
                        }
                    }
                    match current_block_19 {
                        3850642056257311267 => {
                            (*ceiling).speed = CEILSPEED as fixed_t;
                            current_block_19 = 14600216857840559743;
                        }
                        _ => {}
                    }
                    match current_block_19 {
                        14600216857840559743 => {
                            (*ceiling).direction = 1_i32;
                        }
                        _ => {}
                    }
                } else if res == ResultE::crushed {
                    match (*ceiling).type_0 {
                        CeilingE::silentCrushAndRaise
                        | CeilingE::crushAndRaise
                        | CeilingE::lowerAndCrush => {
                            (*ceiling).speed = (CEILSPEED / 8_i32) as fixed_t;
                        }
                        _ => {}
                    }
                }
            }
            0 | _ => {}
        };
    }
}
pub unsafe fn EV_DoCeiling(state: &mut GameState, mut line: LineId, mut type_0: CeilingE) -> i32 {
    let mut secnum: i32 = 0;
    let mut rtn: i32 = 0;
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut ceiling: *mut ceiling_t = ::core::ptr::null_mut::<ceiling_t>();
    secnum = -1_i32;
    rtn = 0_i32;
    match type_0 {
        CeilingE::fastCrushAndRaise | CeilingE::silentCrushAndRaise | CeilingE::crushAndRaise => {
            P_ActivateInStasisCeiling(state, state.p_setup.line(line).tag as i32);
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
        let (ceiling_arena_id, ceiling_ptr) = state.p_ceilng.spawn(ceiling_t::default());
        ceiling = ceiling_ptr;
        let ceiling_id = P_AddThinker(
            state,
            ThinkerPayload::Ceiling(ceiling_arena_id),
            ThinkerKind::Ceiling,
        );
        (*sec).specialdata = Some(SectorSpecial::Ceiling(ceiling_id));
        (*ceiling).thinker.function = ThinkerFn::Ceiling(T_MoveCeiling);
        (*ceiling).sector = SectorId(secnum as u32);
        (*ceiling).crush = false;
        let mut current_block_26: u64;
        match type_0 {
            CeilingE::fastCrushAndRaise => {
                (*ceiling).crush = true;
                (*ceiling).topheight = (*sec).ceilingheight;
                (*ceiling).bottomheight = ((*sec).floorheight + 8_i32 * FRACUNIT) as fixed_t;
                (*ceiling).direction = -1_i32;
                (*ceiling).speed = (CEILSPEED * 2_i32) as fixed_t;
                current_block_26 = 7056779235015430508;
            }
            CeilingE::silentCrushAndRaise | CeilingE::crushAndRaise => {
                (*ceiling).crush = true;
                (*ceiling).topheight = (*sec).ceilingheight;
                current_block_26 = 6994972524166957283;
            }
            CeilingE::lowerAndCrush | CeilingE::lowerToFloor => {
                current_block_26 = 6994972524166957283;
            }
            CeilingE::raiseToHighest => {
                (*ceiling).topheight = P_FindHighestCeilingSurrounding(state, sec);
                (*ceiling).direction = 1_i32;
                (*ceiling).speed = CEILSPEED as fixed_t;
                current_block_26 = 7056779235015430508;
            }
        }
        match current_block_26 {
            6994972524166957283 => {
                (*ceiling).bottomheight = (*sec).floorheight;
                if type_0 != CeilingE::lowerToFloor {
                    (*ceiling).bottomheight += 8_i32 * FRACUNIT;
                }
                (*ceiling).direction = -1_i32;
                (*ceiling).speed = CEILSPEED as fixed_t;
            }
            _ => {}
        }
        (*ceiling).tag = (*sec).tag as i32;
        (*ceiling).type_0 = type_0;
        P_AddActiveCeiling(&mut state.p_ceilng, ceiling_id);
    }
    rtn
}
pub fn P_AddActiveCeiling(state: &mut PCeilngState, id: ThinkerId) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < MAXCEILINGS {
        if state.activeceilings[i as usize].is_none() {
            state.activeceilings[i as usize] = Some(id);
            return;
        }
        i += 1;
    }
}
pub unsafe fn P_RemoveActiveCeiling(state: &mut GameState, mut c: *mut ceiling_t) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < MAXCEILINGS {
        if let Some(id) = state.p_ceilng.activeceilings[i as usize] {
            if P_ThinkerRaw(state, id) as *mut ceiling_t == c {
                state.p_setup.sector_mut((*c).sector).specialdata = None;
                P_RemoveThinker(&raw mut (*c).thinker);
                state.p_ceilng.activeceilings[i as usize] = None;
                break;
            }
        }
        i += 1;
    }
}
pub unsafe fn P_ActivateInStasisCeiling(state: &mut GameState, mut tag: i32) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < MAXCEILINGS {
        if let Some(id) = state.p_ceilng.activeceilings[i as usize] {
            let ceiling = P_ThinkerRaw(state, id) as *mut ceiling_t;
            if (*ceiling).tag == tag && (*ceiling).direction == 0_i32 {
                (*ceiling).direction = (*ceiling).olddirection;
                (*ceiling).thinker.function = ThinkerFn::Ceiling(T_MoveCeiling);
            }
        }
        i += 1;
    }
}
pub unsafe fn EV_CeilingCrushStop(state: &mut GameState, mut tag: i32) -> i32 {
    let mut i: i32 = 0;
    let mut rtn: i32 = 0;
    rtn = 0_i32;
    i = 0_i32;
    while i < MAXCEILINGS {
        if let Some(id) = state.p_ceilng.activeceilings[i as usize] {
            let ceiling = P_ThinkerRaw(state, id) as *mut ceiling_t;
            if (*ceiling).tag == tag && (*ceiling).direction != 0_i32 {
                (*ceiling).olddirection = (*ceiling).direction;
                (*ceiling).thinker.function = ThinkerFn::Paused;
                (*ceiling).direction = 0_i32;
                rtn = 1_i32;
            }
        }
        i += 1;
    }
    rtn
}
