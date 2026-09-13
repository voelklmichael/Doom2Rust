use crate::src::game_state::GameState;
use crate::src::m_random::P_Random;
use crate::src::p_mobj::ThinkerFn;
use crate::src::p_mobj::{sector_t, thinker_t};
use crate::src::p_setup::LineId;
use crate::src::p_setup::SectorId;
use crate::src::p_spec::getNextSector;
use crate::src::p_spec::P_FindMinSurroundingLight;
use crate::src::p_spec::P_FindSectorFromLineTag;
use crate::src::p_tick::P_AddThinker;
use crate::src::z_zone::Z_Malloc;
use crate::src::z_zone::PU_LEVSPEC;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct fireflicker_t {
    pub thinker: thinker_t,
    pub sector: SectorId,
    pub count: i32,
    pub maxlight: i32,
    pub minlight: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lightflash_t {
    pub thinker: thinker_t,
    pub sector: SectorId,
    pub count: i32,
    pub maxlight: i32,
    pub minlight: i32,
    pub maxtime: i32,
    pub mintime: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct strobe_t {
    pub thinker: thinker_t,
    pub sector: SectorId,
    pub count: i32,
    pub minlight: i32,
    pub maxlight: i32,
    pub darktime: i32,
    pub brighttime: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct glow_t {
    pub thinker: thinker_t,
    pub sector: SectorId,
    pub minlight: i32,
    pub maxlight: i32,
    pub direction: i32,
}
pub const GLOWSPEED: i32 = 8;
pub const STROBEBRIGHT: i32 = 5;
pub const SLOWDARK: i32 = 35;
pub unsafe fn T_FireFlicker(state: &mut GameState, mut flick: *mut fireflicker_t) {
    let mut amount: i32 = 0;
    (*flick).count -= 1;
    if (*flick).count != 0 {
        return;
    }
    amount = (P_Random(&mut state.m_random) & 3 as i32) * 16 as i32;
    let sec = state.p_setup.sector_mut((*flick).sector);
    if (*sec).lightlevel as i32 - amount < (*flick).minlight {
        (*sec).lightlevel = (*flick).minlight as i16;
    } else {
        (*sec).lightlevel = ((*flick).maxlight - amount) as i16;
    }
    (*flick).count = 4 as i32;
}
pub unsafe fn P_SpawnFireFlicker(state: &mut GameState, mut sector: SectorId) {
    let mut flick: *mut fireflicker_t = ::core::ptr::null_mut::<fireflicker_t>();
    let sec = state.p_setup.sector_mut(sector);
    (*sec).special = 0 as i16;
    flick = Z_Malloc(
        &mut state.z_zone,
        ::core::mem::size_of::<fireflicker_t>() as i32,
        PU_LEVSPEC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut fireflicker_t;
    P_AddThinker(state, &raw mut (*flick).thinker);
    (*flick).thinker.function = ThinkerFn::FireFlicker(T_FireFlicker);
    (*flick).sector = sector;
    (*flick).maxlight = (*sec).lightlevel as i32;
    (*flick).minlight = P_FindMinSurroundingLight(state, sec, (*sec).lightlevel as i32) + 16 as i32;
    (*flick).count = 4 as i32;
}
pub unsafe fn T_LightFlash(state: &mut GameState, mut flash: *mut lightflash_t) {
    (*flash).count -= 1;
    if (*flash).count != 0 {
        return;
    }
    let sec = state.p_setup.sector_mut((*flash).sector);
    if (*sec).lightlevel as i32 == (*flash).maxlight {
        (*sec).lightlevel = (*flash).minlight as i16;
        (*flash).count = (P_Random(&mut state.m_random) & (*flash).mintime) + 1 as i32;
    } else {
        (*sec).lightlevel = (*flash).maxlight as i16;
        (*flash).count = (P_Random(&mut state.m_random) & (*flash).maxtime) + 1 as i32;
    };
}
pub unsafe fn P_SpawnLightFlash(state: &mut GameState, mut sector: SectorId) {
    let mut flash: *mut lightflash_t = ::core::ptr::null_mut::<lightflash_t>();
    let sec = state.p_setup.sector_mut(sector);
    (*sec).special = 0 as i16;
    flash = Z_Malloc(
        &mut state.z_zone,
        ::core::mem::size_of::<lightflash_t>() as i32,
        PU_LEVSPEC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut lightflash_t;
    P_AddThinker(state, &raw mut (*flash).thinker);
    (*flash).thinker.function = ThinkerFn::LightFlash(T_LightFlash);
    (*flash).sector = sector;
    (*flash).maxlight = (*sec).lightlevel as i32;
    (*flash).minlight = P_FindMinSurroundingLight(state, sec, (*sec).lightlevel as i32);
    (*flash).maxtime = 64 as i32;
    (*flash).mintime = 7 as i32;
    (*flash).count = (P_Random(&mut state.m_random) & (*flash).maxtime) + 1 as i32;
}
pub unsafe fn T_StrobeFlash(state: &mut GameState, mut flash: *mut strobe_t) {
    (*flash).count -= 1;
    if (*flash).count != 0 {
        return;
    }
    let sec = state.p_setup.sector_mut((*flash).sector);
    if (*sec).lightlevel as i32 == (*flash).minlight {
        (*sec).lightlevel = (*flash).maxlight as i16;
        (*flash).count = (*flash).brighttime;
    } else {
        (*sec).lightlevel = (*flash).minlight as i16;
        (*flash).count = (*flash).darktime;
    };
}
pub unsafe fn P_SpawnStrobeFlash(
    state: &mut GameState,
    mut sector: SectorId,
    mut fastOrSlow: i32,
    mut inSync: i32,
) {
    let mut flash: *mut strobe_t = ::core::ptr::null_mut::<strobe_t>();
    let sec = state.p_setup.sector_mut(sector);
    flash = Z_Malloc(
        &mut state.z_zone,
        ::core::mem::size_of::<strobe_t>() as i32,
        PU_LEVSPEC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut strobe_t;
    P_AddThinker(state, &raw mut (*flash).thinker);
    (*flash).sector = sector;
    (*flash).darktime = fastOrSlow;
    (*flash).brighttime = STROBEBRIGHT;
    (*flash).thinker.function = ThinkerFn::Strobe(T_StrobeFlash);
    (*flash).maxlight = (*sec).lightlevel as i32;
    (*flash).minlight = P_FindMinSurroundingLight(state, sec, (*sec).lightlevel as i32);
    if (*flash).minlight == (*flash).maxlight {
        (*flash).minlight = 0 as i32;
    }
    (*sec).special = 0 as i16;
    if inSync == 0 {
        (*flash).count = (P_Random(&mut state.m_random) & 7 as i32) + 1 as i32;
    } else {
        (*flash).count = 1 as i32;
    };
}
pub unsafe fn EV_StartLightStrobing(state: &mut GameState, mut line: LineId) {
    let mut secnum: i32 = 0;
    secnum = -(1 as i32);
    loop {
        secnum = P_FindSectorFromLineTag(state, line, secnum);
        if !(secnum >= 0 as i32) {
            break;
        }
        let sec = state.p_setup.sector_mut(SectorId(secnum as u32));
        if (*sec).specialdata.is_some() {
            continue;
        }
        P_SpawnStrobeFlash(state, SectorId(secnum as u32), SLOWDARK, 0 as i32);
    }
}
pub unsafe fn EV_TurnTagLightsOff(state: &mut GameState, mut line: LineId) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut min: i32 = 0;
    let mut sector: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut tsec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut templine: LineId;
    let line_tag = state.p_setup.line(line).tag;
    j = 0 as i32;
    while j < state.p_setup.numsectors {
        sector = state.p_setup.sector_mut(SectorId(j as u32));
        if (*sector).tag as i32 == line_tag as i32 {
            min = (*sector).lightlevel as i32;
            i = 0 as i32;
            while i < (*sector).linecount {
                templine = (*sector).lines[i as usize];
                tsec = getNextSector(state, templine, sector);
                if !tsec.is_null() {
                    if ((*tsec).lightlevel as i32) < min {
                        min = (*tsec).lightlevel as i32;
                    }
                }
                i += 1;
            }
            (*sector).lightlevel = min as i16;
        }
        j += 1;
    }
}
pub unsafe fn EV_LightTurnOn(state: &mut GameState, mut line: LineId, mut bright: i32) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut sector: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut temp: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut templine: LineId;
    let line_tag = state.p_setup.line(line).tag;
    i = 0 as i32;
    while i < state.p_setup.numsectors {
        sector = state.p_setup.sector_mut(SectorId(i as u32));
        if (*sector).tag as i32 == line_tag as i32 {
            if bright == 0 {
                j = 0 as i32;
                while j < (*sector).linecount {
                    templine = (*sector).lines[j as usize];
                    temp = getNextSector(state, templine, sector);
                    if !temp.is_null() {
                        if (*temp).lightlevel as i32 > bright {
                            bright = (*temp).lightlevel as i32;
                        }
                    }
                    j += 1;
                }
            }
            (*sector).lightlevel = bright as i16;
        }
        i += 1;
    }
}
pub unsafe fn T_Glow(state: &mut GameState, mut g: *mut glow_t) {
    let sec = state.p_setup.sector_mut((*g).sector);
    match (*g).direction {
        -1 => {
            (*sec).lightlevel = ((*sec).lightlevel as i32 - GLOWSPEED) as i16;
            if (*sec).lightlevel as i32 <= (*g).minlight {
                (*sec).lightlevel = ((*sec).lightlevel as i32 + GLOWSPEED) as i16;
                (*g).direction = 1 as i32;
            }
        }
        1 => {
            (*sec).lightlevel = ((*sec).lightlevel as i32 + GLOWSPEED) as i16;
            if (*sec).lightlevel as i32 >= (*g).maxlight {
                (*sec).lightlevel = ((*sec).lightlevel as i32 - GLOWSPEED) as i16;
                (*g).direction = -(1 as i32);
            }
        }
        _ => {}
    };
}
pub unsafe fn P_SpawnGlowingLight(state: &mut GameState, mut sector: SectorId) {
    let mut g: *mut glow_t = ::core::ptr::null_mut::<glow_t>();
    let sec = state.p_setup.sector_mut(sector);
    g = Z_Malloc(
        &mut state.z_zone,
        ::core::mem::size_of::<glow_t>() as i32,
        PU_LEVSPEC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut glow_t;
    P_AddThinker(state, &raw mut (*g).thinker);
    (*g).sector = sector;
    (*g).minlight = P_FindMinSurroundingLight(state, sec, (*sec).lightlevel as i32);
    (*g).maxlight = (*sec).lightlevel as i32;
    (*g).thinker.function = ThinkerFn::Glow(T_Glow);
    (*g).direction = -(1 as i32);
    (*sec).special = 0 as i16;
}
