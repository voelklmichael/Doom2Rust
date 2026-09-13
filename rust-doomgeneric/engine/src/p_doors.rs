use crate::src::d_player::player_t;
use crate::src::doomdef::NULL;
use crate::src::doomdef::TICRATE;
use crate::src::game_state::GameState;
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FRACUNIT;
use crate::src::p_floor::T_MovePlane;
use crate::src::p_floor::ResultE;
use crate::src::p_inter::{
    it_bluecard, it_blueskull, it_redcard, it_redskull, it_yellowcard, it_yellowskull,
};
use crate::src::p_mobj::mobj_t;
use crate::src::p_mobj::SectorSpecial;
use crate::src::p_mobj::ThinkerFn;
use crate::src::p_mobj::{line_t, sector_t, thinker_t};
use crate::src::p_setup::SectorId;
use crate::src::p_spec::P_FindLowestCeilingSurrounding;
use crate::src::p_spec::P_FindSectorFromLineTag;
use crate::src::p_tick::P_AddThinker;
use crate::src::p_tick::P_RemoveThinker;
use crate::src::s_sound::S_StartSound;
use crate::src::sounds::{sfx_bdcls, sfx_bdopn, sfx_dorcls, sfx_doropn, sfx_oof};
use crate::src::z_zone::Z_Malloc;
use crate::src::z_zone::PU_LEVSPEC;
pub type vldoor_e = u32;
pub const vld_blazeClose: vldoor_e = 7;
pub const vld_blazeOpen: vldoor_e = 6;
pub const vld_blazeRaise: vldoor_e = 5;
pub const vld_raiseIn5Mins: vldoor_e = 4;
pub const vld_open: vldoor_e = 3;
pub const vld_close: vldoor_e = 2;
pub const vld_close30ThenOpen: vldoor_e = 1;
pub const vld_normal: vldoor_e = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vldoor_t {
    pub thinker: thinker_t,
    pub type_0: vldoor_e,
    pub sector: SectorId,
    pub topheight: fixed_t,
    pub speed: fixed_t,
    pub direction: i32,
    pub topwait: i32,
    pub topcountdown: i32,
}
pub const VDOORWAIT: i32 = 150;
pub unsafe fn T_VerticalDoor(state: &mut GameState, mut door: *mut vldoor_t) {
    let mut res: ResultE = ResultE::ok;
    let sec = state.p_setup.sector_mut((*door).sector);
    match (*door).direction {
        0 => {
            (*door).topcountdown -= 1;
            if (*door).topcountdown == 0 {
                match (*door).type_0 as u32 {
                    5 => {
                        (*door).direction = -(1 as i32);
                        S_StartSound(
                            state,
                            &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                            sfx_bdcls as i32,
                        );
                    }
                    0 => {
                        (*door).direction = -(1 as i32);
                        S_StartSound(
                            state,
                            &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                            sfx_dorcls as i32,
                        );
                    }
                    1 => {
                        (*door).direction = 1 as i32;
                        S_StartSound(
                            state,
                            &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
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
                match (*door).type_0 as u32 {
                    4 => {
                        (*door).direction = 1 as i32;
                        (*door).type_0 = vld_normal;
                        S_StartSound(
                            state,
                            &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
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
                1 as i32,
                (*door).direction,
            );
            if res == ResultE::pastdest {
                match (*door).type_0 as u32 {
                    5 | 7 => {
                        (*sec).specialdata = None;
                        P_RemoveThinker(&raw mut (*door).thinker);
                        S_StartSound(
                            state,
                            &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                            sfx_bdcls as i32,
                        );
                    }
                    0 | 2 => {
                        (*sec).specialdata = None;
                        P_RemoveThinker(&raw mut (*door).thinker);
                    }
                    1 => {
                        (*door).direction = 0 as i32;
                        (*door).topcountdown = TICRATE * 30 as i32;
                    }
                    _ => {}
                }
            } else if res == ResultE::crushed {
                match (*door).type_0 as u32 {
                    7 | 2 => {}
                    _ => {
                        (*door).direction = 1 as i32;
                        S_StartSound(
                            state,
                            &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
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
                1 as i32,
                (*door).direction,
            );
            if res == ResultE::pastdest {
                match (*door).type_0 as u32 {
                    5 | 0 => {
                        (*door).direction = 0 as i32;
                        (*door).topcountdown = (*door).topwait;
                    }
                    1 | 6 | 3 => {
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
    mut line: *mut line_t,
    mut type_0: vldoor_e,
    mut thing: *mut mobj_t,
) -> i32 {
    let mut p: *mut player_t = ::core::ptr::null_mut::<player_t>();
    let thing_player = (*thing).player;
    if thing_player.is_none() {
        return 0 as i32;
    }
    p = state.g_game.player_mut(thing_player.unwrap());
    match (*line).special as i32 {
        99 | 133 => {
            if p.is_null() {
                return 0 as i32;
            }
            if !(*p).cards[it_bluecard as i32 as usize] && !(*p).cards[it_blueskull as i32 as usize]
            {
                (*p).message = Some("You need a blue key to activate this object".to_string());
                S_StartSound(state, NULL, sfx_oof as i32);
                return 0 as i32;
            }
        }
        134 | 135 => {
            if p.is_null() {
                return 0 as i32;
            }
            if !(*p).cards[it_redcard as i32 as usize] && !(*p).cards[it_redskull as i32 as usize] {
                (*p).message = Some("You need a red key to activate this object".to_string());
                S_StartSound(state, NULL, sfx_oof as i32);
                return 0 as i32;
            }
        }
        136 | 137 => {
            if p.is_null() {
                return 0 as i32;
            }
            if !(*p).cards[it_yellowcard as i32 as usize]
                && !(*p).cards[it_yellowskull as i32 as usize]
            {
                (*p).message = Some("You need a yellow key to activate this object".to_string());
                S_StartSound(state, NULL, sfx_oof as i32);
                return 0 as i32;
            }
        }
        _ => {}
    }
    return EV_DoDoor(state, line, type_0);
}
pub unsafe fn EV_DoDoor(state: &mut GameState, mut line: *mut line_t, mut type_0: vldoor_e) -> i32 {
    let mut secnum: i32 = 0;
    let mut rtn: i32 = 0;
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut door: *mut vldoor_t = ::core::ptr::null_mut::<vldoor_t>();
    secnum = -(1 as i32);
    rtn = 0 as i32;
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
        door = Z_Malloc(
            &mut state.z_zone,
            ::core::mem::size_of::<vldoor_t>() as i32,
            PU_LEVSPEC as i32,
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ) as *mut vldoor_t;
        P_AddThinker(state, &raw mut (*door).thinker);
        (*sec).specialdata = Some(SectorSpecial::Door(door));
        (*door).thinker.function = ThinkerFn::Door(T_VerticalDoor);
        (*door).sector = SectorId(secnum as u32);
        (*door).type_0 = type_0;
        (*door).topwait = VDOORWAIT;
        (*door).speed = (FRACUNIT * 2 as i32) as fixed_t;
        match type_0 as u32 {
            7 => {
                (*door).topheight = P_FindLowestCeilingSurrounding(state, sec);
                (*door).topheight -= 4 as i32 * FRACUNIT;
                (*door).direction = -(1 as i32);
                (*door).speed = (FRACUNIT * 2 as i32 * 4 as i32) as fixed_t;
                S_StartSound(
                    state,
                    &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                    sfx_bdcls as i32,
                );
            }
            2 => {
                (*door).topheight = P_FindLowestCeilingSurrounding(state, sec);
                (*door).topheight -= 4 as i32 * FRACUNIT;
                (*door).direction = -(1 as i32);
                S_StartSound(
                    state,
                    &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                    sfx_dorcls as i32,
                );
            }
            1 => {
                (*door).topheight = (*sec).ceilingheight;
                (*door).direction = -(1 as i32);
                S_StartSound(
                    state,
                    &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                    sfx_dorcls as i32,
                );
            }
            5 | 6 => {
                (*door).direction = 1 as i32;
                (*door).topheight = P_FindLowestCeilingSurrounding(state, sec);
                (*door).topheight -= 4 as i32 * FRACUNIT;
                (*door).speed = (FRACUNIT * 2 as i32 * 4 as i32) as fixed_t;
                if (*door).topheight != (*sec).ceilingheight {
                    S_StartSound(
                        state,
                        &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                        sfx_bdopn as i32,
                    );
                }
            }
            0 | 3 => {
                (*door).direction = 1 as i32;
                (*door).topheight = P_FindLowestCeilingSurrounding(state, sec);
                (*door).topheight -= 4 as i32 * FRACUNIT;
                if (*door).topheight != (*sec).ceilingheight {
                    S_StartSound(
                        state,
                        &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                        sfx_doropn as i32,
                    );
                }
            }
            _ => {}
        }
    }
    return rtn;
}
pub unsafe fn EV_VerticalDoor(
    state: &mut GameState,
    mut line: *mut line_t,
    mut thing: *mut mobj_t,
) {
    let mut player: *mut player_t = ::core::ptr::null_mut::<player_t>();
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut door: *mut vldoor_t = ::core::ptr::null_mut::<vldoor_t>();
    let mut side: i32 = 0;
    side = 0 as i32;
    player = match (*thing).player {
        Some(id) => state.g_game.player_mut(id),
        None => ::core::ptr::null_mut::<player_t>(),
    };
    match (*line).special as i32 {
        26 | 32 => {
            if player.is_null() {
                return;
            }
            if !(*player).cards[it_bluecard as i32 as usize]
                && !(*player).cards[it_blueskull as i32 as usize]
            {
                (*player).message = Some("You need a blue key to open this door".to_string());
                S_StartSound(state, NULL, sfx_oof as i32);
                return;
            }
        }
        27 | 34 => {
            if player.is_null() {
                return;
            }
            if !(*player).cards[it_yellowcard as i32 as usize]
                && !(*player).cards[it_yellowskull as i32 as usize]
            {
                (*player).message = Some("You need a yellow key to open this door".to_string());
                S_StartSound(state, NULL, sfx_oof as i32);
                return;
            }
        }
        28 | 33 => {
            if player.is_null() {
                return;
            }
            if !(*player).cards[it_redcard as i32 as usize]
                && !(*player).cards[it_redskull as i32 as usize]
            {
                (*player).message = Some("You need a red key to open this door".to_string());
                S_StartSound(state, NULL, sfx_oof as i32);
                return;
            }
        }
        _ => {}
    }
    let door_sector_id =
        state.p_setup.sides[(*line).sidenum[(side ^ 1 as i32) as usize] as usize].sector;
    sec = state.p_setup.sector_mut(door_sector_id);
    if let Some(special) = (*sec).specialdata {
        match (*line).special as i32 {
            1 | 26 | 27 | 28 | 117 => {
                match special {
                    SectorSpecial::Door(d) => {
                        door = d;
                        if (*door).direction == -(1 as i32) {
                            (*door).direction = 1 as i32;
                        } else {
                            if (*thing).player.is_none() {
                                return;
                            }
                            (*door).direction = -(1 as i32);
                        }
                    }
                    SectorSpecial::Plat(plat) => {
                        if (*thing).player.is_none() {
                            return;
                        }
                        (*plat).wait = -(1 as i32);
                    }
                    SectorSpecial::Ceiling(ceiling) => {
                        if (*thing).player.is_none() {
                            return;
                        }
                        eprintln!("EV_VerticalDoor: Tried to close something that wasn't a door.");
                        (*ceiling).direction = -(1 as i32);
                    }
                    SectorSpecial::Floor(floor) => {
                        if (*thing).player.is_none() {
                            return;
                        }
                        eprintln!("EV_VerticalDoor: Tried to close something that wasn't a door.");
                        (*floor).direction = -(1 as i32);
                    }
                }
                return;
            }
            _ => {}
        }
    }
    match (*line).special as i32 {
        117 | 118 => {
            S_StartSound(
                state,
                &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                sfx_bdopn as i32,
            );
        }
        1 | 31 => {
            S_StartSound(
                state,
                &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                sfx_doropn as i32,
            );
        }
        _ => {
            S_StartSound(
                state,
                &raw mut (*sec).soundorg as *mut ::core::ffi::c_void,
                sfx_doropn as i32,
            );
        }
    }
    door = Z_Malloc(
        &mut state.z_zone,
        ::core::mem::size_of::<vldoor_t>() as i32,
        PU_LEVSPEC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut vldoor_t;
    P_AddThinker(state, &raw mut (*door).thinker);
    (*sec).specialdata = Some(SectorSpecial::Door(door));
    (*door).thinker.function = ThinkerFn::Door(T_VerticalDoor);
    (*door).sector = door_sector_id;
    (*door).direction = 1 as i32;
    (*door).speed = (FRACUNIT * 2 as i32) as fixed_t;
    (*door).topwait = VDOORWAIT;
    match (*line).special as i32 {
        1 | 26 | 27 | 28 => {
            (*door).type_0 = vld_normal;
        }
        31 | 32 | 33 | 34 => {
            (*door).type_0 = vld_open;
            (*line).special = 0 as i16;
        }
        117 => {
            (*door).type_0 = vld_blazeRaise;
            (*door).speed = (FRACUNIT * 2 as i32 * 4 as i32) as fixed_t;
        }
        118 => {
            (*door).type_0 = vld_blazeOpen;
            (*line).special = 0 as i16;
            (*door).speed = (FRACUNIT * 2 as i32 * 4 as i32) as fixed_t;
        }
        _ => {}
    }
    (*door).topheight = P_FindLowestCeilingSurrounding(state, sec);
    (*door).topheight -= 4 as i32 * FRACUNIT;
}
pub unsafe fn P_SpawnDoorCloseIn30(state: &mut GameState, mut sector: SectorId) {
    let mut door: *mut vldoor_t = ::core::ptr::null_mut::<vldoor_t>();
    let sec = state.p_setup.sector_mut(sector);
    door = Z_Malloc(
        &mut state.z_zone,
        ::core::mem::size_of::<vldoor_t>() as i32,
        PU_LEVSPEC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut vldoor_t;
    P_AddThinker(state, &raw mut (*door).thinker);
    (*sec).specialdata = Some(SectorSpecial::Door(door));
    (*sec).special = 0 as i16;
    (*door).thinker.function = ThinkerFn::Door(T_VerticalDoor);
    (*door).sector = sector;
    (*door).direction = 0 as i32;
    (*door).type_0 = vld_normal;
    (*door).speed = (FRACUNIT * 2 as i32) as fixed_t;
    (*door).topcountdown = 30 as i32 * TICRATE;
}
pub unsafe fn P_SpawnDoorRaiseIn5Mins(state: &mut GameState, mut sector: SectorId) {
    let mut door: *mut vldoor_t = ::core::ptr::null_mut::<vldoor_t>();
    let sec = state.p_setup.sector_mut(sector);
    door = Z_Malloc(
        &mut state.z_zone,
        ::core::mem::size_of::<vldoor_t>() as i32,
        PU_LEVSPEC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut vldoor_t;
    P_AddThinker(state, &raw mut (*door).thinker);
    (*sec).specialdata = Some(SectorSpecial::Door(door));
    (*sec).special = 0 as i16;
    (*door).thinker.function = ThinkerFn::Door(T_VerticalDoor);
    (*door).sector = sector;
    (*door).direction = 2 as i32;
    (*door).type_0 = vld_raiseIn5Mins;
    (*door).speed = (FRACUNIT * 2 as i32) as fixed_t;
    (*door).topheight = P_FindLowestCeilingSurrounding(state, sec);
    (*door).topheight -= 4 as i32 * FRACUNIT;
    (*door).topwait = VDOORWAIT;
    (*door).topcountdown = 5 as i32 * 60 as i32 * TICRATE;
}
