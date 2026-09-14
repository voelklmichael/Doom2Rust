use crate::doomdef::TICRATE;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACUNIT;
use crate::m_random::P_Random;
use crate::p_floor::T_MovePlane;
use crate::p_floor::ResultE;
use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_mobj::sector_t;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::plat_t;
use crate::p_spec::P_FindHighestFloorSurrounding;
use crate::p_spec::P_FindLowestFloorSurrounding;
use crate::p_spec::P_FindNextHighestFloor;
use crate::p_spec::P_FindSectorFromLineTag;
use crate::p_tick::P_AddThinker;
use crate::p_tick::ThinkerKind;
use crate::p_tick::P_RemoveThinker;
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
pub struct PPlatsState {
    pub activeplats: [*mut plat_t; 30],
    plats: Vec<Box<plat_t>>,
}

impl PPlatsState {
    pub const fn new() -> Self {
        PPlatsState {
            activeplats: [::core::ptr::null::<plat_t>() as *mut plat_t; 30],
            plats: Vec::new(),
        }
    }

    // Direct replacement for Z_Malloc(size_of::<plat_t>(), ...) -- see
    // PDoorsState::spawn/dealloc (p_doors.rs) for why no generation-checked
    // id or two-phase retire/deallocate split is needed here either.
    pub fn spawn(&mut self, value: plat_t) -> *mut plat_t {
        self.plats.push(Box::new(value));
        self.plats.last_mut().unwrap().as_mut()
    }

    pub fn dealloc(&mut self, ptr: *mut plat_t) {
        self.plats.retain(|b| !::core::ptr::eq(b.as_ref(), ptr));
    }
}

pub unsafe fn T_PlatRaise(state: &mut GameState, mut plat: *mut plat_t) {
    let mut res: ResultE = ResultE::ok;
    let sec = state.p_setup.sector_mut((*plat).sector);
    match (*plat).status {
        PlatE::up => {
            res = T_MovePlane(
                state,
                sec,
                (*plat).speed,
                (*plat).high,
                (*plat).crush,
                0 as i32,
                1 as i32,
            );
            if (*plat).type_0 == PlattypeE::raiseAndChange
                || (*plat).type_0 == PlattypeE::raiseToNearestAndChange
            {
                if state.p_tick.leveltime & 7 as i32 == 0 {
                    S_StartSound(state, SoundOrigin::Sector((*plat).sector), sfx_stnmov as i32);
                }
            }
            if res == ResultE::crushed && !(*plat).crush {
                (*plat).count = (*plat).wait;
                (*plat).status = PlatE::down;
                S_StartSound(state, SoundOrigin::Sector((*plat).sector), sfx_pstart as i32);
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
            res = T_MovePlane(
                state,
                sec,
                (*plat).speed,
                (*plat).low,
                false,
                0 as i32,
                -(1 as i32),
            );
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
                S_StartSound(state, SoundOrigin::Sector((*plat).sector), sfx_pstart as i32);
            }
        }
        PlatE::in_stasis => {}
    };
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
    secnum = -(1 as i32);
    rtn = 0 as i32;
    match type_0 {
        PlattypeE::perpetualRaise => {
            P_ActivateInStasis(&mut state.p_plats, linev.tag as i32);
        }
        _ => {}
    }
    loop {
        secnum = P_FindSectorFromLineTag(state, line, secnum);
        if !(secnum >= 0 as i32) {
            break;
        }
        sec = state.p_setup.sector_mut(SectorId(secnum as u32));
        if (*sec).specialdata.is_some() {
            continue;
        }
        rtn = 1 as i32;
        plat = state.p_plats.spawn(plat_t::default());
        let plat_id = P_AddThinker(state, &raw mut (*plat).thinker, ThinkerKind::Plat);
        (*plat).type_0 = type_0;
        (*plat).sector = SectorId(secnum as u32);
        (*sec).specialdata = Some(SectorSpecial::Plat(plat_id));
        (*plat).thinker.function = ThinkerFn::Plat(T_PlatRaise);
        (*plat).crush = false;
        (*plat).tag = linev.tag as i32;
        match type_0 {
            PlattypeE::raiseToNearestAndChange => {
                (*plat).speed = (PLATSPEED / 2 as i32) as fixed_t;
                let neighbor_sector_id =
                    state.p_setup.sides[linev.sidenum[0] as usize].sector;
                (*sec).floorpic = (*state.p_setup.sector_mut(neighbor_sector_id)).floorpic;
                (*plat).high = P_FindNextHighestFloor(state, sec, (*sec).floorheight as i32);
                (*plat).wait = 0 as i32;
                (*plat).status = PlatE::up;
                (*sec).special = 0 as i16;
                S_StartSound(state, SoundOrigin::Sector(SectorId(secnum as u32)), sfx_stnmov as i32);
            }
            PlattypeE::raiseAndChange => {
                (*plat).speed = (PLATSPEED / 2 as i32) as fixed_t;
                let neighbor_sector_id =
                    state.p_setup.sides[linev.sidenum[0] as usize].sector;
                (*sec).floorpic = (*state.p_setup.sector_mut(neighbor_sector_id)).floorpic;
                (*plat).high = ((*sec).floorheight as i32 + amount * FRACUNIT) as fixed_t;
                (*plat).wait = 0 as i32;
                (*plat).status = PlatE::up;
                S_StartSound(state, SoundOrigin::Sector(SectorId(secnum as u32)), sfx_stnmov as i32);
            }
            PlattypeE::downWaitUpStay => {
                (*plat).speed = (PLATSPEED * 4 as i32) as fixed_t;
                (*plat).low = P_FindLowestFloorSurrounding(state, sec);
                if (*plat).low > (*sec).floorheight {
                    (*plat).low = (*sec).floorheight;
                }
                (*plat).high = (*sec).floorheight;
                (*plat).wait = TICRATE * PLATWAIT;
                (*plat).status = PlatE::down;
                S_StartSound(state, SoundOrigin::Sector(SectorId(secnum as u32)), sfx_pstart as i32);
            }
            PlattypeE::blazeDWUS => {
                (*plat).speed = (PLATSPEED * 8 as i32) as fixed_t;
                (*plat).low = P_FindLowestFloorSurrounding(state, sec);
                if (*plat).low > (*sec).floorheight {
                    (*plat).low = (*sec).floorheight;
                }
                (*plat).high = (*sec).floorheight;
                (*plat).wait = TICRATE * PLATWAIT;
                (*plat).status = PlatE::down;
                S_StartSound(state, SoundOrigin::Sector(SectorId(secnum as u32)), sfx_pstart as i32);
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
                (*plat).status = if P_Random(&mut state.m_random) & 1 as i32 != 0 {
                    PlatE::down
                } else {
                    PlatE::up
                };
                S_StartSound(state, SoundOrigin::Sector(SectorId(secnum as u32)), sfx_pstart as i32);
            }
        }
        P_AddActivePlat(&mut state.p_plats, plat);
    }
    return rtn;
}
pub unsafe fn P_ActivateInStasis(state: &mut PPlatsState, mut tag: i32) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < MAXPLATS {
        if !state.activeplats[i as usize].is_null()
            && (*state.activeplats[i as usize]).tag == tag
            && (*state.activeplats[i as usize]).status == PlatE::in_stasis
        {
            (*state.activeplats[i as usize]).status = (*state.activeplats[i as usize]).oldstatus;
            (*state.activeplats[i as usize]).thinker.function = ThinkerFn::Plat(T_PlatRaise);
        }
        i += 1;
    }
}
pub unsafe fn EV_StopPlat(state: &mut PPlatsState, mut tag: i32) {
    let mut j: i32 = 0;
    j = 0 as i32;
    while j < MAXPLATS {
        if !state.activeplats[j as usize].is_null()
            && (*state.activeplats[j as usize]).status != PlatE::in_stasis
            && (*state.activeplats[j as usize]).tag == tag
        {
            (*state.activeplats[j as usize]).oldstatus = (*state.activeplats[j as usize]).status;
            (*state.activeplats[j as usize]).status = PlatE::in_stasis;
            (*state.activeplats[j as usize]).thinker.function = ThinkerFn::Paused;
        }
        j += 1;
    }
}
pub fn P_AddActivePlat(state: &mut PPlatsState, mut plat: *mut plat_t) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < MAXPLATS {
        if state.activeplats[i as usize].is_null() {
            state.activeplats[i as usize] = plat;
            return;
        }
        i += 1;
    }
    I_Error("P_AddActivePlat: no more plats!");
}
pub unsafe fn P_RemoveActivePlat(state: &mut GameState, mut plat: *mut plat_t) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < MAXPLATS {
        if plat == state.p_plats.activeplats[i as usize] {
            (*state
                .p_setup
                .sector_mut((*state.p_plats.activeplats[i as usize]).sector))
            .specialdata = None;
            P_RemoveThinker(
                &raw mut (**(&raw mut state.p_plats.activeplats as *mut *mut plat_t)
                    .offset(i as isize))
                .thinker,
            );
            state.p_plats.activeplats[i as usize] = ::core::ptr::null_mut::<plat_t>();
            return;
        }
        i += 1;
    }
    I_Error("P_RemoveActivePlat: can't find plat!");
}
