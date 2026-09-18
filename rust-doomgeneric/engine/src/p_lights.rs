use crate::game_state::GameState;
use crate::m_random::P_Random;
use crate::p_mobj::thinker_t;
use crate::p_mobj::ThinkerFn;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::getNextSector;
use crate::p_spec::P_FindMinSurroundingLight;
use crate::p_spec::P_FindSectorFromLineTag;
use crate::p_tick::P_AddThinker;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;

#[derive(Copy, Clone)]
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

// Generation-checked handles into PLightsState's 4 independent arenas --
// mirror DoorId. None of these 4 types are looked up via a handle or an
// activeXXX-style array like ceiling_t/plat_t/vldoor_t are -- only ever
// referenced by the id handed back from spawn (resolved through
// P_ThinkerRaw), exactly like the other converted kinds.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FireFlickerId {
    index: u32,
    generation: u32,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LightFlashId {
    index: u32,
    generation: u32,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StrobeId {
    index: u32,
    generation: u32,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct GlowId {
    index: u32,
    generation: u32,
}

struct FireFlickerSlot {
    generation: u32,
    value: Option<Box<fireflicker_t>>,
}
struct LightFlashSlot {
    generation: u32,
    value: Option<Box<lightflash_t>>,
}
struct StrobeSlot {
    generation: u32,
    value: Option<Box<strobe_t>>,
}
struct GlowSlot {
    generation: u32,
    value: Option<Box<glow_t>>,
}

// Unlike those types there was no existing per-module state struct here at
// all before this; one small struct hosting all 4 arenas is simplest,
// matching how uniform and small these types are (they were already
// batched into one phase for the same reason). Each arena keeps its own
// id/slot/free_list -- not unified into one generic table, matching this
// codebase's existing style of separate per-kind tables (e.g. PSpecState
// keeps its floor arena separate from its other state).
pub struct PLightsState {
    fireflickers: Vec<FireFlickerSlot>,
    fireflicker_free_list: Vec<u32>,
    lightflashes: Vec<LightFlashSlot>,
    lightflash_free_list: Vec<u32>,
    strobes: Vec<StrobeSlot>,
    strobe_free_list: Vec<u32>,
    glows: Vec<GlowSlot>,
    glow_free_list: Vec<u32>,
}

impl Default for PLightsState {
    fn default() -> Self {
        Self::new()
    }
}

impl PLightsState {
    pub const fn new() -> Self {
        PLightsState {
            fireflickers: Vec::new(),
            fireflicker_free_list: Vec::new(),
            lightflashes: Vec::new(),
            lightflash_free_list: Vec::new(),
            strobes: Vec::new(),
            strobe_free_list: Vec::new(),
            glows: Vec::new(),
            glow_free_list: Vec::new(),
        }
    }

    pub fn spawn_fireflicker(&mut self, value: fireflicker_t) -> FireFlickerId {
        let (index, generation) = if let Some(index) = self.fireflicker_free_list.pop() {
            let slot = &mut self.fireflickers[index as usize];
            slot.generation = slot.generation.wrapping_add(1);
            (index, slot.generation)
        } else {
            let index = self.fireflickers.len() as u32;
            self.fireflickers.push(FireFlickerSlot {
                generation: 0,
                value: None,
            });
            (index, 0)
        };
        let id = FireFlickerId { index, generation };
        let mut boxed = Box::new(value);
        self.fireflickers[index as usize].value = Some(boxed);
        id
    }

    pub fn get_fireflicker_ref(&self, id: FireFlickerId) -> Option<&fireflicker_t> {
        self.fireflickers
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value.as_deref())
    }

    pub fn get_fireflicker_mut(&mut self, id: FireFlickerId) -> Option<&mut fireflicker_t> {
        self.fireflickers
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value.as_deref_mut())
    }
    pub fn dealloc_fireflicker(&mut self, id: FireFlickerId) {
        if let Some(slot) = self.fireflickers.get_mut(id.index as usize) {
            if slot.generation == id.generation {
                slot.value = None;
                self.fireflicker_free_list.push(id.index);
            }
        }
    }

    pub fn spawn_lightflash(&mut self, value: lightflash_t) -> LightFlashId {
        let (index, generation) = if let Some(index) = self.lightflash_free_list.pop() {
            let slot = &mut self.lightflashes[index as usize];
            slot.generation = slot.generation.wrapping_add(1);
            (index, slot.generation)
        } else {
            let index = self.lightflashes.len() as u32;
            self.lightflashes.push(LightFlashSlot {
                generation: 0,
                value: None,
            });
            (index, 0)
        };
        let id = LightFlashId { index, generation };
        let mut boxed = Box::new(value);
        self.lightflashes[index as usize].value = Some(boxed);
        id
    }

    pub fn get_lightflash_ref(&self, id: LightFlashId) -> Option<&lightflash_t> {
        self.lightflashes
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value.as_deref())
    }

    pub fn get_lightflash_mut(&mut self, id: LightFlashId) -> Option<&mut lightflash_t> {
        self.lightflashes
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value.as_deref_mut())
    }
    pub fn dealloc_lightflash(&mut self, id: LightFlashId) {
        if let Some(slot) = self.lightflashes.get_mut(id.index as usize) {
            if slot.generation == id.generation {
                slot.value = None;
                self.lightflash_free_list.push(id.index);
            }
        }
    }

    pub fn spawn_strobe(&mut self, value: strobe_t) -> StrobeId {
        let (index, generation) = if let Some(index) = self.strobe_free_list.pop() {
            let slot = &mut self.strobes[index as usize];
            slot.generation = slot.generation.wrapping_add(1);
            (index, slot.generation)
        } else {
            let index = self.strobes.len() as u32;
            self.strobes.push(StrobeSlot {
                generation: 0,
                value: None,
            });
            (index, 0)
        };
        let id = StrobeId { index, generation };
        let mut boxed = Box::new(value);
        self.strobes[index as usize].value = Some(boxed);
        id
    }

    pub fn get_strobe_ref(&self, id: StrobeId) -> Option<&strobe_t> {
        self.strobes
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value.as_deref())
    }

    pub fn get_strobe_mut(&mut self, id: StrobeId) -> Option<&mut strobe_t> {
        self.strobes
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value.as_deref_mut())
    }
    pub fn dealloc_strobe(&mut self, id: StrobeId) {
        if let Some(slot) = self.strobes.get_mut(id.index as usize) {
            if slot.generation == id.generation {
                slot.value = None;
                self.strobe_free_list.push(id.index);
            }
        }
    }

    pub fn spawn_glow(&mut self, value: glow_t) -> GlowId {
        let (index, generation) = if let Some(index) = self.glow_free_list.pop() {
            let slot = &mut self.glows[index as usize];
            slot.generation = slot.generation.wrapping_add(1);
            (index, slot.generation)
        } else {
            let index = self.glows.len() as u32;
            self.glows.push(GlowSlot {
                generation: 0,
                value: None,
            });
            (index, 0)
        };
        let id = GlowId { index, generation };
        let mut boxed = Box::new(value);
        self.glows[index as usize].value = Some(boxed);
        id
    }

    pub fn get_glow_ref(&self, id: GlowId) -> Option<&glow_t> {
        self.glows
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value.as_deref())
    }

    pub fn get_glow_mut(&mut self, id: GlowId) -> Option<&mut glow_t> {
        self.glows
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value.as_deref_mut())
    }
    pub fn dealloc_glow(&mut self, id: GlowId) {
        if let Some(slot) = self.glows.get_mut(id.index as usize) {
            if slot.generation == id.generation {
                slot.value = None;
                self.glow_free_list.push(id.index);
            }
        }
    }
}
pub fn T_FireFlicker(state: &mut GameState, id: FireFlickerId) {
    let flick = state
        .p_lights
        .get_fireflicker_mut(id)
        .expect("ThinkerFn::FireFlicker id must reference a live fireflicker");
    flick.count -= 1;
    if flick.count != 0 {
        return;
    }
    let amount = (P_Random(&mut state.m_random) & 3_i32) * 16_i32;
    let sec = state.p_setup.sector_mut(flick.sector);
    if sec.lightlevel as i32 - amount < flick.minlight {
        sec.lightlevel = flick.minlight as i16;
    } else {
        sec.lightlevel = (flick.maxlight - amount) as i16;
    }
    flick.count = 4_i32;
}
pub fn P_SpawnFireFlicker(state: &mut GameState, sector: SectorId) {
    state.p_setup.sector_mut(sector).special = 0_i16;
    let lightlevel = state.p_setup.sector_mut(sector).lightlevel as i32;
    let mut flick = fireflicker_t::default();
    flick.thinker.function = ThinkerFn::FireFlicker(T_FireFlicker);
    flick.sector = sector;
    flick.maxlight = lightlevel;
    flick.minlight = P_FindMinSurroundingLight(state, sector, lightlevel) + 16_i32;
    flick.count = 4_i32;
    let flick_arena_id = state.p_lights.spawn_fireflicker(flick);
    P_AddThinker(
        state,
        ThinkerPayload::FireFlicker(flick_arena_id),
        ThinkerKind::FireFlicker,
    );
}
pub fn T_LightFlash(state: &mut GameState, id: LightFlashId) {
    let flash = state
        .p_lights
        .get_lightflash_mut(id)
        .expect("ThinkerFn::LightFlash id must reference a live lightflash");
    flash.count -= 1;
    if flash.count != 0 {
        return;
    }
    let sec = state.p_setup.sector_mut(flash.sector);
    if sec.lightlevel as i32 == flash.maxlight {
        sec.lightlevel = flash.minlight as i16;
        flash.count = (P_Random(&mut state.m_random) & flash.mintime) + 1_i32;
    } else {
        sec.lightlevel = flash.maxlight as i16;
        flash.count = (P_Random(&mut state.m_random) & flash.maxtime) + 1_i32;
    };
}
pub fn P_SpawnLightFlash(state: &mut GameState, sector: SectorId) {
    state.p_setup.sector_mut(sector).special = 0_i16;
    let lightlevel = state.p_setup.sector_mut(sector).lightlevel as i32;
    let mut flash = lightflash_t::default();
    flash.thinker.function = ThinkerFn::LightFlash(T_LightFlash);
    flash.sector = sector;
    flash.maxlight = lightlevel;
    flash.minlight = P_FindMinSurroundingLight(state, sector, lightlevel);
    flash.maxtime = 64_i32;
    flash.mintime = 7_i32;
    flash.count = (P_Random(&mut state.m_random) & flash.maxtime) + 1_i32;
    let flash_arena_id = state.p_lights.spawn_lightflash(flash);
    P_AddThinker(
        state,
        ThinkerPayload::LightFlash(flash_arena_id),
        ThinkerKind::LightFlash,
    );
}
pub fn T_StrobeFlash(state: &mut GameState, id: StrobeId) {
    let flash = state
        .p_lights
        .get_strobe_mut(id)
        .expect("ThinkerFn::Strobe id must reference a live strobe");
    flash.count -= 1;
    if flash.count != 0 {
        return;
    }
    let sec = state.p_setup.sector_mut(flash.sector);
    if sec.lightlevel as i32 == flash.minlight {
        sec.lightlevel = flash.maxlight as i16;
        flash.count = flash.brighttime;
    } else {
        sec.lightlevel = flash.minlight as i16;
        flash.count = flash.darktime;
    };
}
pub fn P_SpawnStrobeFlash(state: &mut GameState, sector: SectorId, fastOrSlow: i32, inSync: i32) {
    let lightlevel = state.p_setup.sector_mut(sector).lightlevel as i32;
    let mut flash = strobe_t {
        sector,
        darktime: fastOrSlow,
        brighttime: STROBEBRIGHT,
        ..strobe_t::default()
    };
    flash.thinker.function = ThinkerFn::Strobe(T_StrobeFlash);
    flash.maxlight = lightlevel;
    flash.minlight = P_FindMinSurroundingLight(state, sector, lightlevel);
    if flash.minlight == flash.maxlight {
        flash.minlight = 0_i32;
    }
    state.p_setup.sector_mut(sector).special = 0_i16;
    if inSync == 0 {
        flash.count = (P_Random(&mut state.m_random) & 7_i32) + 1_i32;
    } else {
        flash.count = 1_i32;
    };
    let flash_arena_id = state.p_lights.spawn_strobe(flash);
    P_AddThinker(
        state,
        ThinkerPayload::Strobe(flash_arena_id),
        ThinkerKind::Strobe,
    );
}
pub fn EV_StartLightStrobing(state: &mut GameState, mut line: LineId) {
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
pub fn EV_TurnTagLightsOff(state: &mut GameState, line: LineId) {
    let line_tag = state.p_setup.line(line).tag;
    for j in 0..state.p_setup.numsectors {
        let sector = SectorId(j as u32);
        if state.p_setup.sector_mut(sector).tag as i32 == line_tag as i32 {
            let mut min = state.p_setup.sector_mut(sector).lightlevel as i32;
            let linecount = state.p_setup.sector_mut(sector).linecount;
            for i in 0..linecount {
                let templine = state.p_setup.sector_mut(sector).lines[i as usize];
                if let Some(tsec) = getNextSector(state, templine, sector) {
                    let light = state.p_setup.sector_mut(tsec).lightlevel as i32;
                    if light < min {
                        min = light;
                    }
                }
            }
            state.p_setup.sector_mut(sector).lightlevel = min as i16;
        }
    }
}
pub fn EV_LightTurnOn(state: &mut GameState, line: LineId, mut bright: i32) {
    let line_tag = state.p_setup.line(line).tag;
    for i in 0..state.p_setup.numsectors {
        let sector = SectorId(i as u32);
        if state.p_setup.sector_mut(sector).tag as i32 == line_tag as i32 {
            if bright == 0 {
                let linecount = state.p_setup.sector_mut(sector).linecount;
                for j in 0..linecount {
                    let templine = state.p_setup.sector_mut(sector).lines[j as usize];
                    if let Some(temp) = getNextSector(state, templine, sector) {
                        let light = state.p_setup.sector_mut(temp).lightlevel as i32;
                        if light > bright {
                            bright = light;
                        }
                    }
                }
            }
            state.p_setup.sector_mut(sector).lightlevel = bright as i16;
        }
    }
}
pub fn T_Glow(state: &mut GameState, id: GlowId) {
    let g = state
        .p_lights
        .get_glow_mut(id)
        .expect("ThinkerFn::Glow id must reference a live glow");
    let sec = state.p_setup.sector_mut(g.sector);
    match g.direction {
        -1 => {
            sec.lightlevel = (sec.lightlevel as i32 - GLOWSPEED) as i16;
            if sec.lightlevel as i32 <= g.minlight {
                sec.lightlevel = (sec.lightlevel as i32 + GLOWSPEED) as i16;
                g.direction = 1_i32;
            }
        }
        1 => {
            sec.lightlevel = (sec.lightlevel as i32 + GLOWSPEED) as i16;
            if sec.lightlevel as i32 >= g.maxlight {
                sec.lightlevel = (sec.lightlevel as i32 - GLOWSPEED) as i16;
                g.direction = -1_i32;
            }
        }
        _ => {}
    };
}
pub fn P_SpawnGlowingLight(state: &mut GameState, sector: SectorId) {
    let lightlevel = state.p_setup.sector_mut(sector).lightlevel as i32;
    let mut g = glow_t {
        sector,
        minlight: P_FindMinSurroundingLight(state, sector, lightlevel),
        maxlight: lightlevel,
        ..glow_t::default()
    };
    g.thinker.function = ThinkerFn::Glow(T_Glow);
    g.direction = -1_i32;
    let g_arena_id = state.p_lights.spawn_glow(g);
    P_AddThinker(state, ThinkerPayload::Glow(g_arena_id), ThinkerKind::Glow);
    state.p_setup.sector_mut(sector).special = 0_i16;
}
