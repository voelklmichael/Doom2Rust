use crate::src::doomdef::TICRATE;
use crate::src::game_state::GameState;
use crate::src::i_system::I_Error;
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FRACUNIT;
use crate::src::m_random::P_Random;
use crate::src::p_floor::T_MovePlane;
use crate::src::p_floor::ResultE;
use crate::src::p_mobj::SectorSpecial;
use crate::src::p_mobj::ThinkerFn;
use crate::src::p_mobj::{line_t, sector_t};
use crate::src::p_setup::SectorId;
use crate::src::p_spec::plat_t;
use crate::src::p_spec::P_FindHighestFloorSurrounding;
use crate::src::p_spec::P_FindLowestFloorSurrounding;
use crate::src::p_spec::P_FindNextHighestFloor;
use crate::src::p_spec::P_FindSectorFromLineTag;
use crate::src::p_tick::P_AddThinker;
use crate::src::p_tick::P_RemoveThinker;
use crate::src::s_sound::S_StartSound;
use crate::src::sounds::{sfx_pstart, sfx_pstop, sfx_stnmov};
use crate::src::z_zone::Z_Malloc;
use crate::src::z_zone::PU_LEVSPEC;

pub type plat_e = u32;
pub const in_stasis: plat_e = 3;
pub const waiting: plat_e = 2;
pub const down: plat_e = 1;
pub const up: plat_e = 0;
pub type plattype_e = u32;
pub const blazeDWUS: plattype_e = 4;
pub const raiseToNearestAndChange: plattype_e = 3;
pub const raiseAndChange: plattype_e = 2;
pub const downWaitUpStay: plattype_e = 1;
pub const perpetualRaise: plattype_e = 0;
pub const PLATWAIT: i32 = 3;
pub const PLATSPEED: i32 = FRACUNIT;
pub const MAXPLATS: i32 = 30;
pub struct PPlatsState {
    pub activeplats: [*mut plat_t; 30],
}

impl PPlatsState {
    pub const fn new() -> Self {
        PPlatsState {
            activeplats: [::core::ptr::null::<plat_t>() as *mut plat_t; 30],
        }
    }
}

pub unsafe fn T_PlatRaise(state: &mut GameState, mut plat: *mut plat_t) {
    let mut res: ResultE = ResultE::ok;
    let sec = state.p_setup.sector_mut((*plat).sector);
    match (*plat).status as u32 {
        0 => {
            res = T_MovePlane(
                state,
                sec,
                (*plat).speed,
                (*plat).high,
                (*plat).crush,
                0 as i32,
                1 as i32,
            );
            if (*plat).type_0 as u32 == raiseAndChange as i32 as u32
                || (*plat).type_0 as u32 == raiseToNearestAndChange as i32 as u32
            {
                if state.p_tick.leveltime & 7 as i32 == 0 {
                    S_StartSound(
                        state,
                        &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                        sfx_stnmov as i32,
                    );
                }
            }
            if res == ResultE::crushed && !(*plat).crush {
                (*plat).count = (*plat).wait;
                (*plat).status = down;
                S_StartSound(
                    state,
                    &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                    sfx_pstart as i32,
                );
            } else if res == ResultE::pastdest {
                (*plat).count = (*plat).wait;
                (*plat).status = waiting;
                S_StartSound(
                    state,
                    &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                    sfx_pstop as i32,
                );
                match (*plat).type_0 as u32 {
                    4 | 1 => {
                        P_RemoveActivePlat(state, plat);
                    }
                    2 | 3 => {
                        P_RemoveActivePlat(state, plat);
                    }
                    _ => {}
                }
            }
        }
        1 => {
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
                (*plat).status = waiting;
                S_StartSound(
                    state,
                    &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                    sfx_pstop as i32,
                );
            }
        }
        2 => {
            (*plat).count -= 1;
            if (*plat).count == 0 {
                if (*sec).floorheight == (*plat).low {
                    (*plat).status = up;
                } else {
                    (*plat).status = down;
                }
                S_StartSound(
                    state,
                    &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                    sfx_pstart as i32,
                );
            }
        }
        3 | _ => {}
    };
}
pub unsafe fn EV_DoPlat(
    state: &mut GameState,
    mut line: *mut line_t,
    mut type_0: plattype_e,
    mut amount: i32,
) -> i32 {
    let mut plat: *mut plat_t = ::core::ptr::null_mut::<plat_t>();
    let mut secnum: i32 = 0;
    let mut rtn: i32 = 0;
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    secnum = -(1 as i32);
    rtn = 0 as i32;
    match type_0 as u32 {
        0 => {
            P_ActivateInStasis(&mut state.p_plats, (*line).tag as i32);
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
        plat = Z_Malloc(
            &mut state.z_zone,
            ::core::mem::size_of::<plat_t>() as i32,
            PU_LEVSPEC as i32,
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ) as *mut plat_t;
        P_AddThinker(state, &raw mut (*plat).thinker);
        (*plat).type_0 = type_0;
        (*plat).sector = SectorId(secnum as u32);
        (*sec).specialdata = Some(SectorSpecial::Plat(plat));
        (*plat).thinker.function = ThinkerFn::Plat(T_PlatRaise);
        (*plat).crush = false;
        (*plat).tag = (*line).tag as i32;
        match type_0 as u32 {
            3 => {
                (*plat).speed = (PLATSPEED / 2 as i32) as fixed_t;
                let neighbor_sector_id =
                    state.p_setup.sides[(*line).sidenum[0 as i32 as usize] as usize].sector;
                (*sec).floorpic = (*state.p_setup.sector_mut(neighbor_sector_id)).floorpic;
                (*plat).high = P_FindNextHighestFloor(state, sec, (*sec).floorheight as i32);
                (*plat).wait = 0 as i32;
                (*plat).status = up;
                (*sec).special = 0 as i16;
                S_StartSound(
                    state,
                    &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                    sfx_stnmov as i32,
                );
            }
            2 => {
                (*plat).speed = (PLATSPEED / 2 as i32) as fixed_t;
                let neighbor_sector_id =
                    state.p_setup.sides[(*line).sidenum[0 as i32 as usize] as usize].sector;
                (*sec).floorpic = (*state.p_setup.sector_mut(neighbor_sector_id)).floorpic;
                (*plat).high = ((*sec).floorheight as i32 + amount * FRACUNIT) as fixed_t;
                (*plat).wait = 0 as i32;
                (*plat).status = up;
                S_StartSound(
                    state,
                    &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                    sfx_stnmov as i32,
                );
            }
            1 => {
                (*plat).speed = (PLATSPEED * 4 as i32) as fixed_t;
                (*plat).low = P_FindLowestFloorSurrounding(state, sec);
                if (*plat).low > (*sec).floorheight {
                    (*plat).low = (*sec).floorheight;
                }
                (*plat).high = (*sec).floorheight;
                (*plat).wait = TICRATE * PLATWAIT;
                (*plat).status = down;
                S_StartSound(
                    state,
                    &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                    sfx_pstart as i32,
                );
            }
            4 => {
                (*plat).speed = (PLATSPEED * 8 as i32) as fixed_t;
                (*plat).low = P_FindLowestFloorSurrounding(state, sec);
                if (*plat).low > (*sec).floorheight {
                    (*plat).low = (*sec).floorheight;
                }
                (*plat).high = (*sec).floorheight;
                (*plat).wait = TICRATE * PLATWAIT;
                (*plat).status = down;
                S_StartSound(
                    state,
                    &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                    sfx_pstart as i32,
                );
            }
            0 => {
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
                (*plat).status = (P_Random(&mut state.m_random) & 1 as i32) as plat_e;
                S_StartSound(
                    state,
                    &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                    sfx_pstart as i32,
                );
            }
            _ => {}
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
            && (*state.activeplats[i as usize]).status as u32 == in_stasis as i32 as u32
        {
            (*state.activeplats[i as usize]).status = (*state.activeplats[i as usize]).oldstatus;
            (*state.activeplats[i as usize]).thinker.function = ThinkerFn::Plat(T_PlatRaise);
        }
        i += 1;
    }
}
pub unsafe fn EV_StopPlat(state: &mut PPlatsState, mut line: *mut line_t) {
    let mut j: i32 = 0;
    j = 0 as i32;
    while j < MAXPLATS {
        if !state.activeplats[j as usize].is_null()
            && (*state.activeplats[j as usize]).status as u32 != in_stasis as i32 as u32
            && (*state.activeplats[j as usize]).tag == (*line).tag as i32
        {
            (*state.activeplats[j as usize]).oldstatus = (*state.activeplats[j as usize]).status;
            (*state.activeplats[j as usize]).status = in_stasis;
            (*state.activeplats[j as usize]).thinker.function = ThinkerFn::Paused;
        }
        j += 1;
    }
}
pub unsafe fn P_AddActivePlat(state: &mut PPlatsState, mut plat: *mut plat_t) {
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
