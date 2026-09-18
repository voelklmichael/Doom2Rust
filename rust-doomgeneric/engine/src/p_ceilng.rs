use crate::game_state::GameState;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACUNIT;
use crate::p_floor::ResultE;
use crate::p_floor::T_MovePlane;

use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::ceiling_t;
use crate::p_spec::P_FindHighestCeilingSurrounding;
use crate::p_spec::P_FindSectorFromLineTag;
use crate::p_tick::P_AddThinker;
use crate::p_tick::P_RemoveThinker;

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

impl Default for PCeilngState {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn spawn(&mut self, value: ceiling_t) -> CeilingId {
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
        self.ceilings[index as usize].ceiling = Some(boxed);
        id
    }


    pub fn get_ref(&self, id: CeilingId) -> Option<&ceiling_t> {
        self.ceilings
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.ceiling.as_deref())
    }

    pub fn get_mut(&mut self, id: CeilingId) -> Option<&mut ceiling_t> {
        self.ceilings
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.ceiling.as_deref_mut())
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
    let ceiling = *state
        .p_ceilng
        .get_ref(id)
        .expect("ThinkerFn::Ceiling id must reference a live ceiling");
    match ceiling.direction {
        1 => {
            let res = T_MovePlane(
                state,
                ceiling.sector,
                ceiling.speed,
                ceiling.topheight,
                false,
                1_i32,
                ceiling.direction,
            );
            if state.p_tick.leveltime & 7_i32 == 0 && ceiling.type_0 != CeilingE::silentCrushAndRaise {
                S_StartSound(state, SoundOrigin::Sector(ceiling.sector), sfx_stnmov as i32);
            }
            if res == ResultE::pastdest {
                match ceiling.type_0 {
                    CeilingE::raiseToHighest => {
                        P_RemoveActiveCeiling(state, id);
                    }
                    CeilingE::silentCrushAndRaise => {
                        S_StartSound(state, SoundOrigin::Sector(ceiling.sector), sfx_pstop as i32);
                        state.p_ceilng.get_mut(id).expect("live ceiling").direction = -1_i32;
                    }
                    CeilingE::fastCrushAndRaise | CeilingE::crushAndRaise => {
                        state.p_ceilng.get_mut(id).expect("live ceiling").direction = -1_i32;
                    }
                    _ => {}
                }
            }
        }
        -1 => {
            let res = T_MovePlane(
                state,
                ceiling.sector,
                ceiling.speed,
                ceiling.bottomheight,
                ceiling.crush,
                1_i32,
                ceiling.direction,
            );
            if state.p_tick.leveltime & 7_i32 == 0 && ceiling.type_0 != CeilingE::silentCrushAndRaise {
                S_StartSound(state, SoundOrigin::Sector(ceiling.sector), sfx_stnmov as i32);
            }
            if res == ResultE::pastdest {
                match ceiling.type_0 {
                    CeilingE::silentCrushAndRaise => {
                        S_StartSound(state, SoundOrigin::Sector(ceiling.sector), sfx_pstop as i32);
                        let c = state.p_ceilng.get_mut(id).expect("live ceiling");
                        c.speed = CEILSPEED as fixed_t;
                        c.direction = 1_i32;
                    }
                    CeilingE::crushAndRaise => {
                        let c = state.p_ceilng.get_mut(id).expect("live ceiling");
                        c.speed = CEILSPEED as fixed_t;
                        c.direction = 1_i32;
                    }
                    CeilingE::fastCrushAndRaise => {
                        state.p_ceilng.get_mut(id).expect("live ceiling").direction = 1_i32;
                    }
                    CeilingE::lowerAndCrush | CeilingE::lowerToFloor => {
                        P_RemoveActiveCeiling(state, id);
                    }
                    _ => {}
                }
            } else if res == ResultE::crushed {
                match ceiling.type_0 {
                    CeilingE::silentCrushAndRaise
                    | CeilingE::crushAndRaise
                    | CeilingE::lowerAndCrush => {
                        state.p_ceilng.get_mut(id).expect("live ceiling").speed =
                            (CEILSPEED / 8_i32) as fixed_t;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    };
}
pub fn EV_DoCeiling(state: &mut GameState, line: LineId, type_0: CeilingE) -> i32 {
    let mut rtn: i32 = 0;
    let mut secnum: i32 = -1_i32;
    match type_0 {
        CeilingE::fastCrushAndRaise | CeilingE::silentCrushAndRaise | CeilingE::crushAndRaise => {
            let tag = state.p_setup.line(line).tag as i32;
            P_ActivateInStasisCeiling(state, tag);
        }
        _ => {}
    }
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
        let (ceilingheight, floorheight, tag) = {
            let s = state.p_setup.sector_mut(sec);
            (s.ceilingheight, s.floorheight, s.tag as i32)
        };
        let mut ceiling = ceiling_t::default();
        ceiling.thinker.function = ThinkerFn::Ceiling(T_MoveCeiling);
        ceiling.sector = sec;
        ceiling.crush = false;
        let mut lower_block = false;
        match type_0 {
            CeilingE::fastCrushAndRaise => {
                ceiling.crush = true;
                ceiling.topheight = ceilingheight;
                ceiling.bottomheight = (floorheight + 8_i32 * FRACUNIT) as fixed_t;
                ceiling.direction = -1_i32;
                ceiling.speed = (CEILSPEED * 2_i32) as fixed_t;
            }
            CeilingE::silentCrushAndRaise | CeilingE::crushAndRaise => {
                ceiling.crush = true;
                ceiling.topheight = ceilingheight;
                lower_block = true;
            }
            CeilingE::lowerAndCrush | CeilingE::lowerToFloor => {
                lower_block = true;
            }
            CeilingE::raiseToHighest => {
                ceiling.topheight = P_FindHighestCeilingSurrounding(state, sec);
                ceiling.direction = 1_i32;
                ceiling.speed = CEILSPEED as fixed_t;
            }
        }
        if lower_block {
            ceiling.bottomheight = floorheight;
            if type_0 != CeilingE::lowerToFloor {
                ceiling.bottomheight += 8_i32 * FRACUNIT;
            }
            ceiling.direction = -1_i32;
            ceiling.speed = CEILSPEED as fixed_t;
        }
        ceiling.tag = tag;
        ceiling.type_0 = type_0;
        let ceiling_arena_id = state.p_ceilng.spawn(ceiling);
        let ceiling_id = P_AddThinker(
            state,
            ThinkerPayload::Ceiling(ceiling_arena_id),
            ThinkerKind::Ceiling,
        );
        state.p_setup.sector_mut(sec).specialdata = Some(SectorSpecial::Ceiling(ceiling_id));
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
pub fn P_RemoveActiveCeiling(state: &mut GameState, ceiling_id: CeilingId) {
    for i in 0..MAXCEILINGS as usize {
        if let Some(id) = state.p_ceilng.activeceilings[i] {
            if state.p_tick.ceiling_payload(id) == ceiling_id {
                let c = state.p_ceilng.get_mut(ceiling_id).expect("live ceiling");
                let sector = c.sector;
                P_RemoveThinker(&mut c.thinker);
                state.p_setup.sector_mut(sector).specialdata = None;
                state.p_ceilng.activeceilings[i] = None;
                break;
            }
        }
    }
}
pub fn P_ActivateInStasisCeiling(state: &mut GameState, tag: i32) {
    for i in 0..MAXCEILINGS as usize {
        if let Some(id) = state.p_ceilng.activeceilings[i] {
            let ceiling_id = state.p_tick.ceiling_payload(id);
            let c = state.p_ceilng.get_mut(ceiling_id).expect("live ceiling");
            if c.tag == tag && c.direction == 0_i32 {
                c.direction = c.olddirection;
                c.thinker.function = ThinkerFn::Ceiling(T_MoveCeiling);
            }
        }
    }
}
pub fn EV_CeilingCrushStop(state: &mut GameState, tag: i32) -> i32 {
    let mut rtn: i32 = 0_i32;
    for i in 0..MAXCEILINGS as usize {
        if let Some(id) = state.p_ceilng.activeceilings[i] {
            let ceiling_id = state.p_tick.ceiling_payload(id);
            let c = state.p_ceilng.get_mut(ceiling_id).expect("live ceiling");
            if c.tag == tag && c.direction != 0_i32 {
                c.olddirection = c.direction;
                c.thinker.function = ThinkerFn::Paused;
                c.direction = 0_i32;
                rtn = 1_i32;
            }
        }
    }
    rtn
}
