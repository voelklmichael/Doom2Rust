use crate::game_state::GameState;
use crate::m_random::P_Random;
use crate::p_mobj::ThinkerFn;
use crate::p_mobj::{sector_t, thinker_t};
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::getNextSector;
use crate::p_spec::P_FindMinSurroundingLight;
use crate::p_spec::P_FindSectorFromLineTag;
use crate::p_tick::P_AddThinker;
use crate::p_tick::ThinkerKind;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct fireflicker_t {
    pub thinker: thinker_t,
    pub sector: SectorId,
    pub count: i32,
    pub maxlight: i32,
    pub minlight: i32,
}
impl Default for fireflicker_t {
    fn default() -> Self {
        fireflicker_t {
            thinker: thinker_t {
                function: ThinkerFn::Unresolved,
            },
            sector: SectorId(0),
            count: 0,
            maxlight: 0,
            minlight: 0,
        }
    }
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
impl Default for lightflash_t {
    fn default() -> Self {
        lightflash_t {
            thinker: thinker_t {
                function: ThinkerFn::Unresolved,
            },
            sector: SectorId(0),
            count: 0,
            maxlight: 0,
            minlight: 0,
            maxtime: 0,
            mintime: 0,
        }
    }
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
impl Default for strobe_t {
    fn default() -> Self {
        strobe_t {
            thinker: thinker_t {
                function: ThinkerFn::Unresolved,
            },
            sector: SectorId(0),
            count: 0,
            minlight: 0,
            maxlight: 0,
            darktime: 0,
            brighttime: 0,
        }
    }
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
impl Default for glow_t {
    fn default() -> Self {
        glow_t {
            thinker: thinker_t {
                function: ThinkerFn::Unresolved,
            },
            sector: SectorId(0),
            minlight: 0,
            maxlight: 0,
            direction: 0,
        }
    }
}
pub const GLOWSPEED: i32 = 8;
pub const STROBEBRIGHT: i32 = 5;
pub const SLOWDARK: i32 = 35;

// None of these 4 types are looked up via a handle or an activeXXX-style
// array like ceiling_t/plat_t/vldoor_t are -- only ever referenced by the
// raw pointer handed back from spawn, exactly as today. Unlike those types
// there was no existing per-module state struct here at all before this;
// one small struct hosting all 4 arenas is simplest, matching how uniform
// and small these types are (they were already batched into one phase for
// the same reason).
pub struct PLightsState {
    fireflickers: Vec<Box<fireflicker_t>>,
    lightflashes: Vec<Box<lightflash_t>>,
    strobes: Vec<Box<strobe_t>>,
    glows: Vec<Box<glow_t>>,
}

impl PLightsState {
    pub const fn new() -> Self {
        PLightsState {
            fireflickers: Vec::new(),
            lightflashes: Vec::new(),
            strobes: Vec::new(),
            glows: Vec::new(),
        }
    }

    pub fn spawn_fireflicker(&mut self, value: fireflicker_t) -> *mut fireflicker_t {
        self.fireflickers.push(Box::new(value));
        self.fireflickers.last_mut().unwrap().as_mut()
    }
    pub fn dealloc_fireflicker(&mut self, ptr: *mut fireflicker_t) {
        self.fireflickers
            .retain(|b| !::core::ptr::eq(b.as_ref(), ptr));
    }

    pub fn spawn_lightflash(&mut self, value: lightflash_t) -> *mut lightflash_t {
        self.lightflashes.push(Box::new(value));
        self.lightflashes.last_mut().unwrap().as_mut()
    }
    pub fn dealloc_lightflash(&mut self, ptr: *mut lightflash_t) {
        self.lightflashes
            .retain(|b| !::core::ptr::eq(b.as_ref(), ptr));
    }

    pub fn spawn_strobe(&mut self, value: strobe_t) -> *mut strobe_t {
        self.strobes.push(Box::new(value));
        self.strobes.last_mut().unwrap().as_mut()
    }
    pub fn dealloc_strobe(&mut self, ptr: *mut strobe_t) {
        self.strobes.retain(|b| !::core::ptr::eq(b.as_ref(), ptr));
    }

    pub fn spawn_glow(&mut self, value: glow_t) -> *mut glow_t {
        self.glows.push(Box::new(value));
        self.glows.last_mut().unwrap().as_mut()
    }
    pub fn dealloc_glow(&mut self, ptr: *mut glow_t) {
        self.glows.retain(|b| !::core::ptr::eq(b.as_ref(), ptr));
    }
}
pub unsafe fn T_FireFlicker(state: &mut GameState, mut flick: *mut fireflicker_t) {
    let mut amount: i32 = 0;
    (*flick).count -= 1;
    if (*flick).count != 0 {
        return;
    }
    amount = (P_Random(&mut state.m_random) & 3_i32) * 16_i32;
    let sec = state.p_setup.sector_mut((*flick).sector);
    if sec.lightlevel as i32 - amount < (*flick).minlight {
        sec.lightlevel = (*flick).minlight as i16;
    } else {
        sec.lightlevel = ((*flick).maxlight - amount) as i16;
    }
    (*flick).count = 4_i32;
}
pub unsafe fn P_SpawnFireFlicker(state: &mut GameState, mut sector: SectorId) {
    let mut flick: *mut fireflicker_t = ::core::ptr::null_mut::<fireflicker_t>();
    let sec: *mut sector_t = state.p_setup.sector_mut(sector);
    (*sec).special = 0_i16;
    flick = state.p_lights.spawn_fireflicker(fireflicker_t::default());
    P_AddThinker(state, &raw mut (*flick).thinker, ThinkerKind::FireFlicker);
    (*flick).thinker.function = ThinkerFn::FireFlicker(T_FireFlicker);
    (*flick).sector = sector;
    (*flick).maxlight = (*sec).lightlevel as i32;
    (*flick).minlight = P_FindMinSurroundingLight(state, sec, (*sec).lightlevel as i32) + 16_i32;
    (*flick).count = 4_i32;
}
pub unsafe fn T_LightFlash(state: &mut GameState, mut flash: *mut lightflash_t) {
    (*flash).count -= 1;
    if (*flash).count != 0 {
        return;
    }
    let sec = state.p_setup.sector_mut((*flash).sector);
    if sec.lightlevel as i32 == (*flash).maxlight {
        sec.lightlevel = (*flash).minlight as i16;
        (*flash).count = (P_Random(&mut state.m_random) & (*flash).mintime) + 1_i32;
    } else {
        sec.lightlevel = (*flash).maxlight as i16;
        (*flash).count = (P_Random(&mut state.m_random) & (*flash).maxtime) + 1_i32;
    };
}
pub unsafe fn P_SpawnLightFlash(state: &mut GameState, mut sector: SectorId) {
    let mut flash: *mut lightflash_t = ::core::ptr::null_mut::<lightflash_t>();
    let sec: *mut sector_t = state.p_setup.sector_mut(sector);
    (*sec).special = 0_i16;
    flash = state.p_lights.spawn_lightflash(lightflash_t::default());
    P_AddThinker(state, &raw mut (*flash).thinker, ThinkerKind::LightFlash);
    (*flash).thinker.function = ThinkerFn::LightFlash(T_LightFlash);
    (*flash).sector = sector;
    (*flash).maxlight = (*sec).lightlevel as i32;
    (*flash).minlight = P_FindMinSurroundingLight(state, sec, (*sec).lightlevel as i32);
    (*flash).maxtime = 64_i32;
    (*flash).mintime = 7_i32;
    (*flash).count = (P_Random(&mut state.m_random) & (*flash).maxtime) + 1_i32;
}
pub unsafe fn T_StrobeFlash(state: &mut GameState, mut flash: *mut strobe_t) {
    (*flash).count -= 1;
    if (*flash).count != 0 {
        return;
    }
    let sec = state.p_setup.sector_mut((*flash).sector);
    if sec.lightlevel as i32 == (*flash).minlight {
        sec.lightlevel = (*flash).maxlight as i16;
        (*flash).count = (*flash).brighttime;
    } else {
        sec.lightlevel = (*flash).minlight as i16;
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
    let sec: *mut sector_t = state.p_setup.sector_mut(sector);
    flash = state.p_lights.spawn_strobe(strobe_t::default());
    P_AddThinker(state, &raw mut (*flash).thinker, ThinkerKind::Strobe);
    (*flash).sector = sector;
    (*flash).darktime = fastOrSlow;
    (*flash).brighttime = STROBEBRIGHT;
    (*flash).thinker.function = ThinkerFn::Strobe(T_StrobeFlash);
    (*flash).maxlight = (*sec).lightlevel as i32;
    (*flash).minlight = P_FindMinSurroundingLight(state, sec, (*sec).lightlevel as i32);
    if (*flash).minlight == (*flash).maxlight {
        (*flash).minlight = 0_i32;
    }
    (*sec).special = 0_i16;
    if inSync == 0 {
        (*flash).count = (P_Random(&mut state.m_random) & 7_i32) + 1_i32;
    } else {
        (*flash).count = 1_i32;
    };
}
pub unsafe fn EV_StartLightStrobing(state: &mut GameState, mut line: LineId) {
    let mut secnum: i32 = 0;
    secnum = -1_i32;
    loop {
        secnum = P_FindSectorFromLineTag(state, line, secnum);
        if secnum < 0_i32 {
            break;
        }
        let sec = state.p_setup.sector_mut(SectorId(secnum as u32));
        if sec.specialdata.is_some() {
            continue;
        }
        P_SpawnStrobeFlash(state, SectorId(secnum as u32), SLOWDARK, 0_i32);
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
    j = 0_i32;
    while j < state.p_setup.numsectors {
        sector = state.p_setup.sector_mut(SectorId(j as u32));
        if (*sector).tag as i32 == line_tag as i32 {
            min = (*sector).lightlevel as i32;
            i = 0_i32;
            while i < (*sector).linecount {
                templine = (*sector).lines[i as usize];
                tsec = getNextSector(state, templine, sector);
                if !tsec.is_null() && ((*tsec).lightlevel as i32) < min {
                    min = (*tsec).lightlevel as i32;
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
    i = 0_i32;
    while i < state.p_setup.numsectors {
        sector = state.p_setup.sector_mut(SectorId(i as u32));
        if (*sector).tag as i32 == line_tag as i32 {
            if bright == 0 {
                j = 0_i32;
                while j < (*sector).linecount {
                    templine = (*sector).lines[j as usize];
                    temp = getNextSector(state, templine, sector);
                    if !temp.is_null() && (*temp).lightlevel as i32 > bright {
                        bright = (*temp).lightlevel as i32;
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
            sec.lightlevel = (sec.lightlevel as i32 - GLOWSPEED) as i16;
            if sec.lightlevel as i32 <= (*g).minlight {
                sec.lightlevel = (sec.lightlevel as i32 + GLOWSPEED) as i16;
                (*g).direction = 1_i32;
            }
        }
        1 => {
            sec.lightlevel = (sec.lightlevel as i32 + GLOWSPEED) as i16;
            if sec.lightlevel as i32 >= (*g).maxlight {
                sec.lightlevel = (sec.lightlevel as i32 - GLOWSPEED) as i16;
                (*g).direction = -1_i32;
            }
        }
        _ => {}
    };
}
pub unsafe fn P_SpawnGlowingLight(state: &mut GameState, mut sector: SectorId) {
    let mut g: *mut glow_t = ::core::ptr::null_mut::<glow_t>();
    let sec: *mut sector_t = state.p_setup.sector_mut(sector);
    g = state.p_lights.spawn_glow(glow_t::default());
    P_AddThinker(state, &raw mut (*g).thinker, ThinkerKind::Glow);
    (*g).sector = sector;
    (*g).minlight = P_FindMinSurroundingLight(state, sec, (*sec).lightlevel as i32);
    (*g).maxlight = (*sec).lightlevel as i32;
    (*g).thinker.function = ThinkerFn::Glow(T_Glow);
    (*g).direction = -1_i32;
    (*sec).special = 0_i16;
}
