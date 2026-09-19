use crate::game_state::GameState;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACUNIT;
use crate::p_floor::move_plane;
use crate::p_floor::ResultE;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::find_highest_ceiling_surrounding;
use crate::p_spec::find_sector_from_line_tag;
use crate::p_spec::Ceiling;
use crate::p_tick::add_thinker;
use crate::p_tick::remove_thinker;

use crate::p_tick::ThinkerId;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CeilingE {
    LowerToFloor = 0,
    RaiseToHighest = 1,
    LowerAndCrush = 2,
    CrushAndRaise = 3,
    FastCrushAndRaise = 4,
    SilentCrushAndRaise = 5,
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
    ceiling: Option<Box<Ceiling>>,
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

    // Moves a fully-defaulted (then caller-filled) Ceiling onto the heap
    // and hands back both a stable generation-checked handle (stored in
    // ThinkerNode's payload by p_tick.rs, replacing what used to be a bare
    // raw pointer there) and a raw pointer for the caller's immediate
    // post-spawn field writes -- mirrors PDoorsState::spawn exactly.
    pub fn spawn(&mut self, value: Ceiling) -> CeilingId {
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
        let boxed = Box::new(value);
        self.ceilings[index as usize].ceiling = Some(boxed);
        id
    }

    pub fn get_ref(&self, id: CeilingId) -> Option<&Ceiling> {
        self.ceilings
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.ceiling.as_deref())
    }

    pub fn get_mut(&mut self, id: CeilingId) -> Option<&mut Ceiling> {
        self.ceilings
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.ceiling.as_deref_mut())
    }

    // Called once, from run_thinkers' reaper, when a Ceiling-kind thinker
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

pub fn move_ceiling(state: &mut GameState, id: CeilingId) {
    let ceiling = *state
        .p_ceilng
        .get_ref(id)
        .expect("ThinkerFn::Ceiling id must reference a live ceiling");
    match ceiling.direction {
        1 => {
            let res = move_plane(
                state,
                ceiling.sector,
                ceiling.speed,
                ceiling.topheight,
                false,
                1,
                ceiling.direction,
            );
            if state.p_tick.leveltime & 7 == 0 && ceiling.kind != CeilingE::SilentCrushAndRaise {
                s_start_sound(
                    state,
                    SoundOrigin::Sector(ceiling.sector),
                    SfxName::Stnmov as i32,
                );
            }
            if res == ResultE::Pastdest {
                match ceiling.kind {
                    CeilingE::RaiseToHighest => {
                        remove_active_ceiling(state, id);
                    }
                    CeilingE::SilentCrushAndRaise => {
                        s_start_sound(
                            state,
                            SoundOrigin::Sector(ceiling.sector),
                            SfxName::Pstop as i32,
                        );
                        state.p_ceilng.get_mut(id).expect("live ceiling").direction = -1;
                    }
                    CeilingE::FastCrushAndRaise | CeilingE::CrushAndRaise => {
                        state.p_ceilng.get_mut(id).expect("live ceiling").direction = -1;
                    }
                    _ => {}
                }
            }
        }
        -1 => {
            let res = move_plane(
                state,
                ceiling.sector,
                ceiling.speed,
                ceiling.bottomheight,
                ceiling.crush,
                1,
                ceiling.direction,
            );
            if state.p_tick.leveltime & 7 == 0 && ceiling.kind != CeilingE::SilentCrushAndRaise {
                s_start_sound(
                    state,
                    SoundOrigin::Sector(ceiling.sector),
                    SfxName::Stnmov as i32,
                );
            }
            if res == ResultE::Pastdest {
                match ceiling.kind {
                    CeilingE::SilentCrushAndRaise => {
                        s_start_sound(
                            state,
                            SoundOrigin::Sector(ceiling.sector),
                            SfxName::Pstop as i32,
                        );
                        let c = state.p_ceilng.get_mut(id).expect("live ceiling");
                        c.speed = CEILSPEED as Fixed;
                        c.direction = 1;
                    }
                    CeilingE::CrushAndRaise => {
                        let c = state.p_ceilng.get_mut(id).expect("live ceiling");
                        c.speed = CEILSPEED as Fixed;
                        c.direction = 1;
                    }
                    CeilingE::FastCrushAndRaise => {
                        state.p_ceilng.get_mut(id).expect("live ceiling").direction = 1;
                    }
                    CeilingE::LowerAndCrush | CeilingE::LowerToFloor => {
                        remove_active_ceiling(state, id);
                    }
                    _ => {}
                }
            } else if res == ResultE::Crushed {
                match ceiling.kind {
                    CeilingE::SilentCrushAndRaise
                    | CeilingE::CrushAndRaise
                    | CeilingE::LowerAndCrush => {
                        state.p_ceilng.get_mut(id).expect("live ceiling").speed =
                            (CEILSPEED / 8) as Fixed;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    };
}
pub fn do_ceiling(state: &mut GameState, line: LineId, kind: CeilingE) -> bool {
    let mut rtn = false;
    let mut secnum: i32 = -1;
    match kind {
        CeilingE::FastCrushAndRaise | CeilingE::SilentCrushAndRaise | CeilingE::CrushAndRaise => {
            let tag = state.p_setup.line(line).tag as i32;
            activate_in_stasis_ceiling(state, tag);
        }
        _ => {}
    }
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
        let (ceilingheight, floorheight, tag) = {
            let s = state.p_setup.sector_mut(sec);
            (s.ceilingheight, s.floorheight, s.tag as i32)
        };
        let mut ceiling = Ceiling::default();
        ceiling.thinker.function = ThinkerFn::Ceiling(move_ceiling);
        ceiling.sector = sec;
        ceiling.crush = false;
        let mut lower_block = false;
        match kind {
            CeilingE::FastCrushAndRaise => {
                ceiling.crush = true;
                ceiling.topheight = ceilingheight;
                ceiling.bottomheight = (floorheight + 8 * FRACUNIT) as Fixed;
                ceiling.direction = -1;
                ceiling.speed = (CEILSPEED * 2) as Fixed;
            }
            CeilingE::SilentCrushAndRaise | CeilingE::CrushAndRaise => {
                ceiling.crush = true;
                ceiling.topheight = ceilingheight;
                lower_block = true;
            }
            CeilingE::LowerAndCrush | CeilingE::LowerToFloor => {
                lower_block = true;
            }
            CeilingE::RaiseToHighest => {
                ceiling.topheight = find_highest_ceiling_surrounding(state, sec);
                ceiling.direction = 1;
                ceiling.speed = CEILSPEED as Fixed;
            }
        }
        if lower_block {
            ceiling.bottomheight = floorheight;
            if kind != CeilingE::LowerToFloor {
                ceiling.bottomheight += 8 * FRACUNIT;
            }
            ceiling.direction = -1;
            ceiling.speed = CEILSPEED as Fixed;
        }
        ceiling.tag = tag;
        ceiling.kind = kind;
        let ceiling_arena_id = state.p_ceilng.spawn(ceiling);
        let ceiling_id = add_thinker(
            state,
            ThinkerPayload::Ceiling(ceiling_arena_id),
            ThinkerKind::Ceiling,
        );
        state.p_setup.sector_mut(sec).specialdata = Some(SectorSpecial::Ceiling(ceiling_id));
        add_active_ceiling(&mut state.p_ceilng, ceiling_id);
    }
    rtn
}
pub fn add_active_ceiling(state: &mut PCeilngState, id: ThinkerId) {
    for i in 0..(MAXCEILINGS as usize) {
        if state.activeceilings[i].is_none() {
            state.activeceilings[i] = Some(id);
            return;
        }
    }
}
pub fn remove_active_ceiling(state: &mut GameState, ceiling_id: CeilingId) {
    for i in 0..MAXCEILINGS as usize {
        if let Some(id) = state.p_ceilng.activeceilings[i] {
            if state.p_tick.ceiling_payload(id) == ceiling_id {
                let c = state.p_ceilng.get_mut(ceiling_id).expect("live ceiling");
                let sector = c.sector;
                remove_thinker(&mut c.thinker);
                state.p_setup.sector_mut(sector).specialdata = None;
                state.p_ceilng.activeceilings[i] = None;
                break;
            }
        }
    }
}
pub fn activate_in_stasis_ceiling(state: &mut GameState, tag: i32) {
    for i in 0..MAXCEILINGS as usize {
        if let Some(id) = state.p_ceilng.activeceilings[i] {
            let ceiling_id = state.p_tick.ceiling_payload(id);
            let c = state.p_ceilng.get_mut(ceiling_id).expect("live ceiling");
            if c.tag == tag && c.direction == 0 {
                c.direction = c.olddirection;
                c.thinker.function = ThinkerFn::Ceiling(move_ceiling);
            }
        }
    }
}
pub fn ceiling_crush_stop(state: &mut GameState, tag: i32) -> bool {
    let mut rtn = false;
    for i in 0..MAXCEILINGS as usize {
        if let Some(id) = state.p_ceilng.activeceilings[i] {
            let ceiling_id = state.p_tick.ceiling_payload(id);
            let c = state.p_ceilng.get_mut(ceiling_id).expect("live ceiling");
            if c.tag == tag && c.direction != 0 {
                c.olddirection = c.direction;
                c.thinker.function = ThinkerFn::Paused;
                c.direction = 0;
                rtn = true;
            }
        }
    }
    rtn
}
