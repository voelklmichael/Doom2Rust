use crate::doomdef::TICRATE;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACUNIT;
use crate::m_random::p_random;
use crate::p_floor::move_plane;
use crate::p_floor::ResultE;
use crate::p_setup::PSetupState;
use crate::p_spec::{Direction, Plane};
use crate::p_tick::PTickState;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_setup::LineId;
use crate::p_spec::find_highest_floor_surrounding;
use crate::p_spec::find_lowest_floor_surrounding;
use crate::p_spec::find_next_highest_floor;
use crate::p_spec::sectors_with_line_tag;
use crate::p_spec::Plat;
use crate::p_tick::add_thinker;
use crate::p_tick::remove_thinker;

use crate::p_tick::ThinkerId;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlatE {
    Up,
    Down,
    Waiting,
    InStasis,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlattypeE {
    PerpetualRaise,
    DownWaitUpStay,
    RaiseAndChange,
    RaiseToNearestAndChange,
    BlazeDWUS,
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
    plat: Option<Box<Plat>>,
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
        Self {
            activeplats: [None; 30],
            plats: Vec::new(),
            free_list: Vec::new(),
        }
    }

    // Moves a fully-defaulted (then caller-filled) Plat onto the heap and
    // hands back both a stable generation-checked handle (stored in
    // ThinkerNode's payload by p_tick.rs, replacing what used to be a bare
    // raw pointer there) and a raw pointer for the caller's immediate
    // post-spawn field writes -- mirrors PDoorsState::spawn exactly.
    pub fn spawn(&mut self, value: Plat) -> PlatId {
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
        let boxed = Box::new(value);
        self.plats[index as usize].plat = Some(boxed);
        id
    }

    pub fn get_ref(&self, id: PlatId) -> Option<&Plat> {
        self.plats
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.plat.as_deref())
    }

    pub fn get_mut(&mut self, id: PlatId) -> Option<&mut Plat> {
        self.plats
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.plat.as_deref_mut())
    }

    // Called once, from run_thinkers' reaper, when a Plat-kind thinker is
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

pub fn plat_raise(state: &mut GameState, id: PlatId) {
    let plat = *state
        .world
        .p_plats
        .get_ref(id)
        .expect("ThinkerFn::Plat id must reference a live plat");
    match plat.status {
        PlatE::Up => {
            let res = move_plane(
                state,
                plat.sector,
                plat.speed,
                plat.high,
                plat.crush,
                Plane::Floor,
                Direction::Up,
            );
            if (plat.kind == PlattypeE::RaiseAndChange
                || plat.kind == PlattypeE::RaiseToNearestAndChange)
                && state.world.p_tick.leveltime & 7 == 0
            {
                s_start_sound(state, SoundOrigin::Sector(plat.sector), SfxName::Stnmov);
            }
            if res == ResultE::Crushed && !plat.crush {
                let p = state.world.p_plats.get_mut(id).expect("live plat");
                p.count = p.wait;
                p.status = PlatE::Down;
                s_start_sound(state, SoundOrigin::Sector(plat.sector), SfxName::Pstart);
            } else if res == ResultE::Pastdest {
                let p = state.world.p_plats.get_mut(id).expect("live plat");
                p.count = p.wait;
                p.status = PlatE::Waiting;
                s_start_sound(state, SoundOrigin::Sector(plat.sector), SfxName::Pstop);
                match plat.kind {
                    PlattypeE::BlazeDWUS | PlattypeE::DownWaitUpStay => {
                        remove_active_plat(
                            &mut state.world.p_plats,
                            &mut state.world.p_setup,
                            &state.world.p_tick,
                            id,
                        );
                    }
                    PlattypeE::RaiseAndChange | PlattypeE::RaiseToNearestAndChange => {
                        remove_active_plat(
                            &mut state.world.p_plats,
                            &mut state.world.p_setup,
                            &state.world.p_tick,
                            id,
                        );
                    }
                    PlattypeE::PerpetualRaise => {}
                }
            }
        }
        PlatE::Down => {
            let res = move_plane(
                state,
                plat.sector,
                plat.speed,
                plat.low,
                false,
                Plane::Floor,
                Direction::Down,
            );
            if res == ResultE::Pastdest {
                let p = state.world.p_plats.get_mut(id).expect("live plat");
                p.count = p.wait;
                p.status = PlatE::Waiting;
                s_start_sound(state, SoundOrigin::Sector(plat.sector), SfxName::Pstop);
            }
        }
        PlatE::Waiting => {
            let p = state.world.p_plats.get_mut(id).expect("live plat");
            p.count -= 1;
            if p.count == 0 {
                let low = p.low;
                if state.world.p_setup.sector_mut(plat.sector).floorheight == low {
                    state.world.p_plats.get_mut(id).expect("live plat").status = PlatE::Up;
                } else {
                    state.world.p_plats.get_mut(id).expect("live plat").status = PlatE::Down;
                }
                s_start_sound(state, SoundOrigin::Sector(plat.sector), SfxName::Pstart);
            }
        }
        PlatE::InStasis => {}
    }
}
pub fn do_plat(state: &mut GameState, line: LineId, kind: PlattypeE, amount: i32) -> bool {
    let linev = state.world.p_setup.line(line);
    let mut rtn = false;
    if kind == PlattypeE::PerpetualRaise {
        activate_in_stasis(
            &mut state.world.p_plats,
            &state.world.p_tick,
            i32::from(linev.tag),
        );
    }
    for sector in sectors_with_line_tag(&state.world.p_setup, line) {
        let sec = sector;
        if state.world.p_setup.sector_mut(sec).specialdata.is_some() {
            continue;
        }
        rtn = true;
        let mut plat = Plat {
            kind,
            sector: sec,
            ..Plat::default()
        };
        plat.thinker.function = ThinkerFn::Plat(plat_raise);
        plat.crush = false;
        plat.tag = i32::from(linev.tag);
        let floorheight = state.world.p_setup.sector_mut(sec).floorheight;
        match kind {
            PlattypeE::RaiseToNearestAndChange => {
                plat.speed = (PLATSPEED / 2) as Fixed;
                let neighbor_sector_id =
                    state.world.p_setup.sides[linev.sidenum[0] as usize].sector;
                let neighbor_pic = state.world.p_setup.sector_mut(neighbor_sector_id).floorpic;
                state.world.p_setup.sector_mut(sec).floorpic = neighbor_pic;
                plat.high = find_next_highest_floor(&mut state.world.p_setup, sec, floorheight);
                plat.wait = 0;
                plat.status = PlatE::Up;
                state.world.p_setup.sector_mut(sec).special = 0;
                s_start_sound(state, SoundOrigin::Sector(sec), SfxName::Stnmov);
            }
            PlattypeE::RaiseAndChange => {
                plat.speed = (PLATSPEED / 2) as Fixed;
                let neighbor_sector_id =
                    state.world.p_setup.sides[linev.sidenum[0] as usize].sector;
                let neighbor_pic = state.world.p_setup.sector_mut(neighbor_sector_id).floorpic;
                state.world.p_setup.sector_mut(sec).floorpic = neighbor_pic;
                plat.high = (floorheight + amount * FRACUNIT) as Fixed;
                plat.wait = 0;
                plat.status = PlatE::Up;
                s_start_sound(state, SoundOrigin::Sector(sec), SfxName::Stnmov);
            }
            PlattypeE::DownWaitUpStay => {
                plat.speed = (PLATSPEED * 4) as Fixed;
                plat.low = find_lowest_floor_surrounding(&mut state.world.p_setup, sec);
                if plat.low > floorheight {
                    plat.low = floorheight;
                }
                plat.high = floorheight;
                plat.wait = TICRATE * PLATWAIT;
                plat.status = PlatE::Down;
                s_start_sound(state, SoundOrigin::Sector(sec), SfxName::Pstart);
            }
            PlattypeE::BlazeDWUS => {
                plat.speed = (PLATSPEED * 8) as Fixed;
                plat.low = find_lowest_floor_surrounding(&mut state.world.p_setup, sec);
                if plat.low > floorheight {
                    plat.low = floorheight;
                }
                plat.high = floorheight;
                plat.wait = TICRATE * PLATWAIT;
                plat.status = PlatE::Down;
                s_start_sound(state, SoundOrigin::Sector(sec), SfxName::Pstart);
            }
            PlattypeE::PerpetualRaise => {
                plat.speed = PLATSPEED as Fixed;
                plat.low = find_lowest_floor_surrounding(&mut state.world.p_setup, sec);
                if plat.low > floorheight {
                    plat.low = floorheight;
                }
                plat.high = find_highest_floor_surrounding(&mut state.world.p_setup, sec);
                if plat.high < floorheight {
                    plat.high = floorheight;
                }
                plat.wait = TICRATE * PLATWAIT;
                plat.status = if p_random(&mut state.world.m_random) & 1 != 0 {
                    PlatE::Down
                } else {
                    PlatE::Up
                };
                s_start_sound(state, SoundOrigin::Sector(sec), SfxName::Pstart);
            }
        }
        let plat_arena_id = state.world.p_plats.spawn(plat);
        let plat_id = add_thinker(
            &mut state.world.p_tick,
            ThinkerPayload::Plat(plat_arena_id),
            ThinkerKind::Plat,
        );
        state.world.p_setup.sector_mut(sec).specialdata = Some(SectorSpecial::Plat(plat_id));
        add_active_plat(&mut state.world.p_plats, plat_id);
    }
    rtn
}
pub fn activate_in_stasis(p_plats: &mut PPlatsState, p_tick: &PTickState, tag: i32) {
    for i in 0..MAXPLATS as usize {
        if let Some(id) = p_plats.activeplats[i] {
            let plat_id = p_tick.plat_payload(id);
            let p = p_plats.get_mut(plat_id).expect("live plat");
            if p.tag == tag && p.status == PlatE::InStasis {
                p.status = p.oldstatus;
                p.thinker.function = ThinkerFn::Plat(plat_raise);
            }
        }
    }
}
pub fn stop_plat(p_plats: &mut PPlatsState, p_tick: &PTickState, tag: i32) {
    for j in 0..MAXPLATS as usize {
        if let Some(id) = p_plats.activeplats[j] {
            let plat_id = p_tick.plat_payload(id);
            let p = p_plats.get_mut(plat_id).expect("live plat");
            if p.status != PlatE::InStasis && p.tag == tag {
                p.oldstatus = p.status;
                p.status = PlatE::InStasis;
                p.thinker.function = ThinkerFn::Paused;
            }
        }
    }
}
pub fn add_active_plat(state: &mut PPlatsState, id: ThinkerId) {
    for i in 0..(MAXPLATS as usize) {
        if state.activeplats[i].is_none() {
            state.activeplats[i] = Some(id);
            return;
        }
    }
    error("P_AddActivePlat: no more plats!");
}
pub fn remove_active_plat(
    p_plats: &mut PPlatsState,
    p_setup: &mut PSetupState,
    p_tick: &PTickState,
    plat_id: PlatId,
) {
    for i in 0..MAXPLATS as usize {
        if let Some(id) = p_plats.activeplats[i] {
            if p_tick.plat_payload(id) == plat_id {
                let p = p_plats.get_mut(plat_id).expect("live plat");
                let sector = p.sector;
                remove_thinker(&mut p.thinker);
                p_setup.sector_mut(sector).specialdata = None;
                p_plats.activeplats[i] = None;
                return;
            }
        }
    }
    error("P_RemoveActivePlat: can't find plat!");
}
