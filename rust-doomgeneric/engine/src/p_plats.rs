use crate::doomdef::TICRATE;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACUNIT;
use crate::m_random::P_Random;
use crate::p_floor::ResultE;
use crate::p_floor::T_MovePlane;

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

use crate::p_tick::ThinkerId;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;

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

impl Default for PPlatsState {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn spawn(&mut self, value: plat_t) -> PlatId {
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
        self.plats[index as usize].plat = Some(boxed);
        id
    }

    pub fn get_ref(&self, id: PlatId) -> Option<&plat_t> {
        self.plats
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.plat.as_deref())
    }

    pub fn get_mut(&mut self, id: PlatId) -> Option<&mut plat_t> {
        self.plats
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.plat.as_deref_mut())
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
    let plat = *state
        .p_plats
        .get_ref(id)
        .expect("ThinkerFn::Plat id must reference a live plat");
    match plat.status {
        PlatE::up => {
            let res = T_MovePlane(
                state,
                plat.sector,
                plat.speed,
                plat.high,
                plat.crush,
                0_i32,
                1_i32,
            );
            if (plat.type_0 == PlattypeE::raiseAndChange
                || plat.type_0 == PlattypeE::raiseToNearestAndChange)
                && state.p_tick.leveltime & 7_i32 == 0
            {
                S_StartSound(
                    state,
                    SoundOrigin::Sector(plat.sector),
                    SfxName::sfx_stnmov as i32,
                );
            }
            if res == ResultE::crushed && !plat.crush {
                let p = state.p_plats.get_mut(id).expect("live plat");
                p.count = p.wait;
                p.status = PlatE::down;
                S_StartSound(
                    state,
                    SoundOrigin::Sector(plat.sector),
                    SfxName::sfx_pstart as i32,
                );
            } else if res == ResultE::pastdest {
                let p = state.p_plats.get_mut(id).expect("live plat");
                p.count = p.wait;
                p.status = PlatE::waiting;
                S_StartSound(
                    state,
                    SoundOrigin::Sector(plat.sector),
                    SfxName::sfx_pstop as i32,
                );
                match plat.type_0 {
                    PlattypeE::blazeDWUS | PlattypeE::downWaitUpStay => {
                        P_RemoveActivePlat(state, id);
                    }
                    PlattypeE::raiseAndChange | PlattypeE::raiseToNearestAndChange => {
                        P_RemoveActivePlat(state, id);
                    }
                    PlattypeE::perpetualRaise => {}
                }
            }
        }
        PlatE::down => {
            let res = T_MovePlane(
                state,
                plat.sector,
                plat.speed,
                plat.low,
                false,
                0_i32,
                -1_i32,
            );
            if res == ResultE::pastdest {
                let p = state.p_plats.get_mut(id).expect("live plat");
                p.count = p.wait;
                p.status = PlatE::waiting;
                S_StartSound(
                    state,
                    SoundOrigin::Sector(plat.sector),
                    SfxName::sfx_pstop as i32,
                );
            }
        }
        PlatE::waiting => {
            let p = state.p_plats.get_mut(id).expect("live plat");
            p.count -= 1;
            if p.count == 0 {
                let low = p.low;
                if state.p_setup.sector_mut(plat.sector).floorheight == low {
                    state.p_plats.get_mut(id).expect("live plat").status = PlatE::up;
                } else {
                    state.p_plats.get_mut(id).expect("live plat").status = PlatE::down;
                }
                S_StartSound(
                    state,
                    SoundOrigin::Sector(plat.sector),
                    SfxName::sfx_pstart as i32,
                );
            }
        }
        PlatE::in_stasis => {}
    };
}
pub fn EV_DoPlat(state: &mut GameState, line: LineId, type_0: PlattypeE, amount: i32) -> i32 {
    let linev = state.p_setup.line(line);
    let mut secnum: i32 = -1_i32;
    let mut rtn: i32 = 0_i32;
    if type_0 == PlattypeE::perpetualRaise {
        P_ActivateInStasis(state, linev.tag as i32);
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
        let mut plat = plat_t {
            type_0,
            sector: sec,
            ..plat_t::default()
        };
        plat.thinker.function = ThinkerFn::Plat(T_PlatRaise);
        plat.crush = false;
        plat.tag = linev.tag as i32;
        let floorheight = state.p_setup.sector_mut(sec).floorheight;
        match type_0 {
            PlattypeE::raiseToNearestAndChange => {
                plat.speed = (PLATSPEED / 2_i32) as fixed_t;
                let neighbor_sector_id = state.p_setup.sides[linev.sidenum[0] as usize].sector;
                let neighbor_pic = state.p_setup.sector_mut(neighbor_sector_id).floorpic;
                state.p_setup.sector_mut(sec).floorpic = neighbor_pic;
                plat.high = P_FindNextHighestFloor(state, sec, floorheight);
                plat.wait = 0_i32;
                plat.status = PlatE::up;
                state.p_setup.sector_mut(sec).special = 0_i16;
                S_StartSound(state, SoundOrigin::Sector(sec), SfxName::sfx_stnmov as i32);
            }
            PlattypeE::raiseAndChange => {
                plat.speed = (PLATSPEED / 2_i32) as fixed_t;
                let neighbor_sector_id = state.p_setup.sides[linev.sidenum[0] as usize].sector;
                let neighbor_pic = state.p_setup.sector_mut(neighbor_sector_id).floorpic;
                state.p_setup.sector_mut(sec).floorpic = neighbor_pic;
                plat.high = (floorheight + amount * FRACUNIT) as fixed_t;
                plat.wait = 0_i32;
                plat.status = PlatE::up;
                S_StartSound(state, SoundOrigin::Sector(sec), SfxName::sfx_stnmov as i32);
            }
            PlattypeE::downWaitUpStay => {
                plat.speed = (PLATSPEED * 4_i32) as fixed_t;
                plat.low = P_FindLowestFloorSurrounding(state, sec);
                if plat.low > floorheight {
                    plat.low = floorheight;
                }
                plat.high = floorheight;
                plat.wait = TICRATE * PLATWAIT;
                plat.status = PlatE::down;
                S_StartSound(state, SoundOrigin::Sector(sec), SfxName::sfx_pstart as i32);
            }
            PlattypeE::blazeDWUS => {
                plat.speed = (PLATSPEED * 8_i32) as fixed_t;
                plat.low = P_FindLowestFloorSurrounding(state, sec);
                if plat.low > floorheight {
                    plat.low = floorheight;
                }
                plat.high = floorheight;
                plat.wait = TICRATE * PLATWAIT;
                plat.status = PlatE::down;
                S_StartSound(state, SoundOrigin::Sector(sec), SfxName::sfx_pstart as i32);
            }
            PlattypeE::perpetualRaise => {
                plat.speed = PLATSPEED as fixed_t;
                plat.low = P_FindLowestFloorSurrounding(state, sec);
                if plat.low > floorheight {
                    plat.low = floorheight;
                }
                plat.high = P_FindHighestFloorSurrounding(state, sec);
                if plat.high < floorheight {
                    plat.high = floorheight;
                }
                plat.wait = TICRATE * PLATWAIT;
                plat.status = if P_Random(&mut state.m_random) & 1_i32 != 0 {
                    PlatE::down
                } else {
                    PlatE::up
                };
                S_StartSound(state, SoundOrigin::Sector(sec), SfxName::sfx_pstart as i32);
            }
        }
        let plat_arena_id = state.p_plats.spawn(plat);
        let plat_id = P_AddThinker(
            state,
            ThinkerPayload::Plat(plat_arena_id),
            ThinkerKind::Plat,
        );
        state.p_setup.sector_mut(sec).specialdata = Some(SectorSpecial::Plat(plat_id));
        P_AddActivePlat(&mut state.p_plats, plat_id);
    }
    rtn
}
pub fn P_ActivateInStasis(state: &mut GameState, tag: i32) {
    for i in 0..MAXPLATS as usize {
        if let Some(id) = state.p_plats.activeplats[i] {
            let plat_id = state.p_tick.plat_payload(id);
            let p = state.p_plats.get_mut(plat_id).expect("live plat");
            if p.tag == tag && p.status == PlatE::in_stasis {
                p.status = p.oldstatus;
                p.thinker.function = ThinkerFn::Plat(T_PlatRaise);
            }
        }
    }
}
pub fn EV_StopPlat(state: &mut GameState, tag: i32) {
    for j in 0..MAXPLATS as usize {
        if let Some(id) = state.p_plats.activeplats[j] {
            let plat_id = state.p_tick.plat_payload(id);
            let p = state.p_plats.get_mut(plat_id).expect("live plat");
            if p.status != PlatE::in_stasis && p.tag == tag {
                p.oldstatus = p.status;
                p.status = PlatE::in_stasis;
                p.thinker.function = ThinkerFn::Paused;
            }
        }
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
pub fn P_RemoveActivePlat(state: &mut GameState, plat_id: PlatId) {
    for i in 0..MAXPLATS as usize {
        if let Some(id) = state.p_plats.activeplats[i] {
            if state.p_tick.plat_payload(id) == plat_id {
                let p = state.p_plats.get_mut(plat_id).expect("live plat");
                let sector = p.sector;
                P_RemoveThinker(&mut p.thinker);
                state.p_setup.sector_mut(sector).specialdata = None;
                state.p_plats.activeplats[i] = None;
                return;
            }
        }
    }
    I_Error("P_RemoveActivePlat: can't find plat!");
}
