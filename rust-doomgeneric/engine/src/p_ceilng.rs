use crate::src::game_state::GameState;
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FRACUNIT;
use crate::src::p_floor::T_MovePlane;
use crate::src::p_floor::ResultE;
use crate::src::p_mobj::SectorSpecial;
use crate::src::p_mobj::ThinkerFn;
use crate::src::p_mobj::sector_t;
use crate::src::p_setup::LineId;
use crate::src::p_setup::SectorId;
use crate::src::p_spec::ceiling_t;
use crate::src::p_spec::P_FindHighestCeilingSurrounding;
use crate::src::p_spec::P_FindSectorFromLineTag;
use crate::src::p_tick::P_AddThinker;
use crate::src::p_tick::P_RemoveThinker;
use crate::src::s_sound::S_StartSound;
use crate::src::s_sound::SoundOrigin;
use crate::src::sounds::{sfx_pstop, sfx_stnmov};
use crate::src::z_zone::Z_Malloc;
use crate::src::z_zone::PU_LEVSPEC;

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
pub struct PCeilngState {
    pub activeceilings: [*mut ceiling_t; 30],
}

impl PCeilngState {
    pub const fn new() -> Self {
        PCeilngState {
            activeceilings: [::core::ptr::null::<ceiling_t>() as *mut ceiling_t; 30],
        }
    }
}

pub unsafe fn T_MoveCeiling(state: &mut GameState, mut ceiling: *mut ceiling_t) {
    let mut res: ResultE = ResultE::ok;
    let sec = state.p_setup.sector_mut((*ceiling).sector);
    match (*ceiling).direction {
        1 => {
            res = T_MovePlane(
                state,
                sec,
                (*ceiling).speed,
                (*ceiling).topheight,
                false,
                1 as i32,
                (*ceiling).direction,
            );
            if state.p_tick.leveltime & 7 as i32 == 0 {
                match (*ceiling).type_0 {
                    CeilingE::silentCrushAndRaise => {}
                    _ => {
                        S_StartSound(state, SoundOrigin::Sector((*ceiling).sector), sfx_stnmov as i32);
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
                        S_StartSound(state, SoundOrigin::Sector((*ceiling).sector), sfx_pstop as i32);
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
                        (*ceiling).direction = -(1 as i32);
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
                1 as i32,
                (*ceiling).direction,
            );
            if state.p_tick.leveltime & 7 as i32 == 0 {
                match (*ceiling).type_0 {
                    CeilingE::silentCrushAndRaise => {}
                    _ => {
                        S_StartSound(state, SoundOrigin::Sector((*ceiling).sector), sfx_stnmov as i32);
                    }
                }
            }
            if res == ResultE::pastdest {
                let mut current_block_19: u64;
                match (*ceiling).type_0 {
                    CeilingE::silentCrushAndRaise => {
                        S_StartSound(state, SoundOrigin::Sector((*ceiling).sector), sfx_pstop as i32);
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
                        (*ceiling).direction = 1 as i32;
                    }
                    _ => {}
                }
            } else if res == ResultE::crushed {
                match (*ceiling).type_0 {
                    CeilingE::silentCrushAndRaise | CeilingE::crushAndRaise | CeilingE::lowerAndCrush => {
                        (*ceiling).speed = (CEILSPEED / 8 as i32) as fixed_t;
                    }
                    _ => {}
                }
            }
        }
        0 | _ => {}
    };
}
pub unsafe fn EV_DoCeiling(
    state: &mut GameState,
    mut line: LineId,
    mut type_0: CeilingE,
) -> i32 {
    let mut secnum: i32 = 0;
    let mut rtn: i32 = 0;
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut ceiling: *mut ceiling_t = ::core::ptr::null_mut::<ceiling_t>();
    secnum = -(1 as i32);
    rtn = 0 as i32;
    match type_0 {
        CeilingE::fastCrushAndRaise | CeilingE::silentCrushAndRaise | CeilingE::crushAndRaise => {
            P_ActivateInStasisCeiling(&mut state.p_ceilng, state.p_setup.line(line).tag as i32);
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
        ceiling = Z_Malloc(
            &mut state.z_zone,
            ::core::mem::size_of::<ceiling_t>() as i32,
            PU_LEVSPEC as i32,
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ) as *mut ceiling_t;
        let ceiling_id = P_AddThinker(state, &raw mut (*ceiling).thinker);
        (*sec).specialdata = Some(SectorSpecial::Ceiling(ceiling_id));
        (*ceiling).thinker.function = ThinkerFn::Ceiling(T_MoveCeiling);
        (*ceiling).sector = SectorId(secnum as u32);
        (*ceiling).crush = false;
        let mut current_block_26: u64;
        match type_0 {
            CeilingE::fastCrushAndRaise => {
                (*ceiling).crush = true;
                (*ceiling).topheight = (*sec).ceilingheight;
                (*ceiling).bottomheight =
                    ((*sec).floorheight as i32 + 8 as i32 * FRACUNIT) as fixed_t;
                (*ceiling).direction = -(1 as i32);
                (*ceiling).speed = (CEILSPEED * 2 as i32) as fixed_t;
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
                (*ceiling).direction = 1 as i32;
                (*ceiling).speed = CEILSPEED as fixed_t;
                current_block_26 = 7056779235015430508;
            }
        }
        match current_block_26 {
            6994972524166957283 => {
                (*ceiling).bottomheight = (*sec).floorheight;
                if type_0 != CeilingE::lowerToFloor {
                    (*ceiling).bottomheight += 8 as i32 * FRACUNIT;
                }
                (*ceiling).direction = -(1 as i32);
                (*ceiling).speed = CEILSPEED as fixed_t;
            }
            _ => {}
        }
        (*ceiling).tag = (*sec).tag as i32;
        (*ceiling).type_0 = type_0;
        P_AddActiveCeiling(&mut state.p_ceilng, ceiling);
    }
    return rtn;
}
pub unsafe fn P_AddActiveCeiling(state: &mut PCeilngState, mut c: *mut ceiling_t) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < MAXCEILINGS {
        if state.activeceilings[i as usize].is_null() {
            state.activeceilings[i as usize] = c;
            return;
        }
        i += 1;
    }
}
pub unsafe fn P_RemoveActiveCeiling(state: &mut GameState, mut c: *mut ceiling_t) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < MAXCEILINGS {
        if state.p_ceilng.activeceilings[i as usize] == c {
            (*state
                .p_setup
                .sector_mut((*state.p_ceilng.activeceilings[i as usize]).sector))
            .specialdata = None;
            P_RemoveThinker(
                &raw mut (**(&raw mut state.p_ceilng.activeceilings as *mut *mut ceiling_t)
                    .offset(i as isize))
                .thinker,
            );
            state.p_ceilng.activeceilings[i as usize] = ::core::ptr::null_mut::<ceiling_t>();
            break;
        } else {
            i += 1;
        }
    }
}
pub unsafe fn P_ActivateInStasisCeiling(state: &mut PCeilngState, mut tag: i32) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < MAXCEILINGS {
        if !state.activeceilings[i as usize].is_null()
            && (*state.activeceilings[i as usize]).tag == tag
            && (*state.activeceilings[i as usize]).direction == 0 as i32
        {
            (*state.activeceilings[i as usize]).direction =
                (*state.activeceilings[i as usize]).olddirection;
            (*state.activeceilings[i as usize]).thinker.function =
                ThinkerFn::Ceiling(T_MoveCeiling);
        }
        i += 1;
    }
}
pub unsafe fn EV_CeilingCrushStop(state: &mut PCeilngState, mut tag: i32) -> i32 {
    let mut i: i32 = 0;
    let mut rtn: i32 = 0;
    rtn = 0 as i32;
    i = 0 as i32;
    while i < MAXCEILINGS {
        if !state.activeceilings[i as usize].is_null()
            && (*state.activeceilings[i as usize]).tag == tag
            && (*state.activeceilings[i as usize]).direction != 0 as i32
        {
            (*state.activeceilings[i as usize]).olddirection =
                (*state.activeceilings[i as usize]).direction;
            (*state.activeceilings[i as usize]).thinker.function = ThinkerFn::Paused;
            (*state.activeceilings[i as usize]).direction = 0 as i32;
            rtn = 1 as i32;
        }
        i += 1;
    }
    return rtn;
}
