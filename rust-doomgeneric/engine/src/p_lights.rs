use crate::game_state::GameState;
use crate::game_state::World;
use crate::m_random::p_random;
use crate::p_mobj::Thinker;
use crate::p_mobj::ThinkerFn;
use crate::p_setup::LineId;
use crate::p_setup::PSetupState;
use crate::p_setup::SectorId;
use crate::p_spec::find_min_surrounding_light;
use crate::p_spec::get_next_sector;
use crate::p_spec::sectors_with_line_tag;
use crate::p_spec::Direction;
use crate::p_tick::add_thinker;
use crate::p_tick::PTickState;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use alloc::boxed::Box;
use alloc::vec::Vec;

#[derive(Copy, Clone)]
pub struct FireFlicker {
    pub thinker: Thinker,
    pub sector: SectorId,
    pub count: i32,
    pub maxlight: i32,
    pub minlight: i32,
}
impl Default for FireFlicker {
    fn default() -> Self {
        Self {
            thinker: Thinker {
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
pub struct LightFlash {
    pub thinker: Thinker,
    pub sector: SectorId,
    pub count: i32,
    pub maxlight: i32,
    pub minlight: i32,
    pub maxtime: i32,
    pub mintime: i32,
}
impl Default for LightFlash {
    fn default() -> Self {
        Self {
            thinker: Thinker {
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
pub struct Strobe {
    pub thinker: Thinker,
    pub sector: SectorId,
    pub count: i32,
    pub minlight: i32,
    pub maxlight: i32,
    pub darktime: i32,
    pub brighttime: i32,
}
impl Default for Strobe {
    fn default() -> Self {
        Self {
            thinker: Thinker {
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
pub struct Glow {
    pub thinker: Thinker,
    pub sector: SectorId,
    pub minlight: i32,
    pub maxlight: i32,
    pub direction: Direction,
}
impl Default for Glow {
    fn default() -> Self {
        Self {
            thinker: Thinker {
                function: ThinkerFn::Unresolved,
            },
            sector: SectorId(0),
            minlight: 0,
            maxlight: 0,
            direction: Direction::Still,
        }
    }
}
pub const GLOWSPEED: i32 = 8;
pub const STROBEBRIGHT: i32 = 5;
pub const SLOWDARK: i32 = 35;

// Generation-checked handles into PLightsState's 4 independent arenas --
// mirror DoorId. None of these 4 types are looked up via a handle or an
// activeXXX-style array like Ceiling/Plat/VlDoor are -- only ever
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
    value: Option<Box<FireFlicker>>,
}
struct LightFlashSlot {
    generation: u32,
    value: Option<Box<LightFlash>>,
}
struct StrobeSlot {
    generation: u32,
    value: Option<Box<Strobe>>,
}
struct GlowSlot {
    generation: u32,
    value: Option<Box<Glow>>,
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
        Self {
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

    pub fn spawn_fireflicker(&mut self, value: FireFlicker) -> FireFlickerId {
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
        let boxed = Box::new(value);
        self.fireflickers[index as usize].value = Some(boxed);
        id
    }

    pub fn get_fireflicker_ref(&self, id: FireFlickerId) -> Option<&FireFlicker> {
        self.fireflickers
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value.as_deref())
    }

    pub fn get_fireflicker_mut(&mut self, id: FireFlickerId) -> Option<&mut FireFlicker> {
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

    pub fn spawn_lightflash(&mut self, value: LightFlash) -> LightFlashId {
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
        let boxed = Box::new(value);
        self.lightflashes[index as usize].value = Some(boxed);
        id
    }

    pub fn get_lightflash_ref(&self, id: LightFlashId) -> Option<&LightFlash> {
        self.lightflashes
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value.as_deref())
    }

    pub fn get_lightflash_mut(&mut self, id: LightFlashId) -> Option<&mut LightFlash> {
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

    pub fn spawn_strobe(&mut self, value: Strobe) -> StrobeId {
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
        let boxed = Box::new(value);
        self.strobes[index as usize].value = Some(boxed);
        id
    }

    pub fn get_strobe_ref(&self, id: StrobeId) -> Option<&Strobe> {
        self.strobes
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value.as_deref())
    }

    pub fn get_strobe_mut(&mut self, id: StrobeId) -> Option<&mut Strobe> {
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

    pub fn spawn_glow(&mut self, value: Glow) -> GlowId {
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
        let boxed = Box::new(value);
        self.glows[index as usize].value = Some(boxed);
        id
    }

    pub fn get_glow_ref(&self, id: GlowId) -> Option<&Glow> {
        self.glows
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value.as_deref())
    }

    pub fn get_glow_mut(&mut self, id: GlowId) -> Option<&mut Glow> {
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
pub fn fire_flicker(state: &mut GameState, id: FireFlickerId) {
    let flick = state
        .world
        .p_lights
        .get_fireflicker_mut(id)
        .expect("ThinkerFn::FireFlicker id must reference a live fireflicker");
    flick.count -= 1;
    if flick.count != 0 {
        return;
    }
    let amount = (p_random(&mut state.world.m_random) & 3) * 16;
    let sec = state.world.p_setup.sector_mut(flick.sector);
    if i32::from(sec.lightlevel) - amount < flick.minlight {
        sec.lightlevel = flick.minlight as i16;
    } else {
        sec.lightlevel = (flick.maxlight - amount) as i16;
    }
    flick.count = 4;
}
pub fn spawn_fire_flicker(
    p_lights: &mut PLightsState,
    p_setup: &mut PSetupState,
    p_tick: &mut PTickState,
    sector: SectorId,
) {
    p_setup.sector_mut(sector).special = 0;
    let lightlevel = i32::from(p_setup.sector_mut(sector).lightlevel);
    let mut flick = FireFlicker::default();
    flick.thinker.function = ThinkerFn::FireFlicker(fire_flicker);
    flick.sector = sector;
    flick.maxlight = lightlevel;
    flick.minlight = find_min_surrounding_light(p_setup, sector, lightlevel) + 16;
    flick.count = 4;
    let flick_arena_id = p_lights.spawn_fireflicker(flick);
    add_thinker(
        p_tick,
        ThinkerPayload::FireFlicker(flick_arena_id),
        ThinkerKind::FireFlicker,
    );
}
pub fn light_flash(state: &mut GameState, id: LightFlashId) {
    let flash = state
        .world
        .p_lights
        .get_lightflash_mut(id)
        .expect("ThinkerFn::LightFlash id must reference a live lightflash");
    flash.count -= 1;
    if flash.count != 0 {
        return;
    }
    let sec = state.world.p_setup.sector_mut(flash.sector);
    if i32::from(sec.lightlevel) == flash.maxlight {
        sec.lightlevel = flash.minlight as i16;
        flash.count = (p_random(&mut state.world.m_random) & flash.mintime) + 1;
    } else {
        sec.lightlevel = flash.maxlight as i16;
        flash.count = (p_random(&mut state.world.m_random) & flash.maxtime) + 1;
    }
}
pub fn spawn_light_flash(world: &mut World, sector: SectorId) {
    world.p_setup.sector_mut(sector).special = 0;
    let lightlevel = i32::from(world.p_setup.sector_mut(sector).lightlevel);
    let mut flash = LightFlash::default();
    flash.thinker.function = ThinkerFn::LightFlash(light_flash);
    flash.sector = sector;
    flash.maxlight = lightlevel;
    flash.minlight = find_min_surrounding_light(&mut world.p_setup, sector, lightlevel);
    flash.maxtime = 64;
    flash.mintime = 7;
    flash.count = (p_random(&mut world.m_random) & flash.maxtime) + 1;
    let flash_arena_id = world.p_lights.spawn_lightflash(flash);
    add_thinker(
        &mut world.p_tick,
        ThinkerPayload::LightFlash(flash_arena_id),
        ThinkerKind::LightFlash,
    );
}
pub fn strobe_flash(state: &mut GameState, id: StrobeId) {
    let flash = state
        .world
        .p_lights
        .get_strobe_mut(id)
        .expect("ThinkerFn::Strobe id must reference a live strobe");
    flash.count -= 1;
    if flash.count != 0 {
        return;
    }
    let sec = state.world.p_setup.sector_mut(flash.sector);
    if i32::from(sec.lightlevel) == flash.minlight {
        sec.lightlevel = flash.maxlight as i16;
        flash.count = flash.brighttime;
    } else {
        sec.lightlevel = flash.minlight as i16;
        flash.count = flash.darktime;
    }
}
pub fn spawn_strobe_flash(world: &mut World, sector: SectorId, fast_or_slow: i32, in_sync: i32) {
    let lightlevel = i32::from(world.p_setup.sector_mut(sector).lightlevel);
    let mut flash = Strobe {
        sector,
        darktime: fast_or_slow,
        brighttime: STROBEBRIGHT,
        ..Strobe::default()
    };
    flash.thinker.function = ThinkerFn::Strobe(strobe_flash);
    flash.maxlight = lightlevel;
    flash.minlight = find_min_surrounding_light(&mut world.p_setup, sector, lightlevel);
    if flash.minlight == flash.maxlight {
        flash.minlight = 0;
    }
    world.p_setup.sector_mut(sector).special = 0;
    if in_sync == 0 {
        flash.count = (p_random(&mut world.m_random) & 7) + 1;
    } else {
        flash.count = 1;
    }
    let flash_arena_id = world.p_lights.spawn_strobe(flash);
    add_thinker(
        &mut world.p_tick,
        ThinkerPayload::Strobe(flash_arena_id),
        ThinkerKind::Strobe,
    );
}
pub fn start_light_strobing(world: &mut World, line: LineId) {
    for sector in sectors_with_line_tag(&world.p_setup, line) {
        let sec = world.p_setup.sector_mut(sector);
        if sec.specialdata.is_some() {
            continue;
        }
        spawn_strobe_flash(world, sector, SLOWDARK, 0);
    }
}
pub fn turn_tag_lights_off(p_setup: &mut PSetupState, line: LineId) {
    let line_tag = p_setup.line(line).tag;
    for j in 0..p_setup.numsectors {
        let sector = SectorId(j as u32);
        if i32::from(p_setup.sector_mut(sector).tag) == i32::from(line_tag) {
            let mut min = i32::from(p_setup.sector_mut(sector).lightlevel);
            let linecount = p_setup.sector_mut(sector).linecount;
            for i in 0..linecount as usize {
                let templine = p_setup.sector_mut(sector).lines[i];
                if let Some(tsec) = get_next_sector(p_setup, templine, sector) {
                    let light = i32::from(p_setup.sector_mut(tsec).lightlevel);
                    if light < min {
                        min = light;
                    }
                }
            }
            p_setup.sector_mut(sector).lightlevel = min as i16;
        }
    }
}
pub fn light_turn_on(p_setup: &mut PSetupState, line: LineId, mut bright: i32) {
    let line_tag = p_setup.line(line).tag;
    for i in 0..p_setup.numsectors {
        let sector = SectorId(i as u32);
        if i32::from(p_setup.sector_mut(sector).tag) == i32::from(line_tag) {
            if bright == 0 {
                let linecount = p_setup.sector_mut(sector).linecount;
                for j in 0..linecount as usize {
                    let templine = p_setup.sector_mut(sector).lines[j];
                    if let Some(temp) = get_next_sector(p_setup, templine, sector) {
                        let light = i32::from(p_setup.sector_mut(temp).lightlevel);
                        if light > bright {
                            bright = light;
                        }
                    }
                }
            }
            p_setup.sector_mut(sector).lightlevel = bright as i16;
        }
    }
}
pub fn glow(state: &mut GameState, id: GlowId) {
    let g = state
        .world
        .p_lights
        .get_glow_mut(id)
        .expect("ThinkerFn::Glow id must reference a live glow");
    let sec = state.world.p_setup.sector_mut(g.sector);
    match g.direction {
        Direction::Down => {
            sec.lightlevel = (i32::from(sec.lightlevel) - GLOWSPEED) as i16;
            if i32::from(sec.lightlevel) <= g.minlight {
                sec.lightlevel = (i32::from(sec.lightlevel) + GLOWSPEED) as i16;
                g.direction = Direction::Up;
            }
        }
        Direction::Up => {
            sec.lightlevel = (i32::from(sec.lightlevel) + GLOWSPEED) as i16;
            if i32::from(sec.lightlevel) >= g.maxlight {
                sec.lightlevel = (i32::from(sec.lightlevel) - GLOWSPEED) as i16;
                g.direction = Direction::Down;
            }
        }
        _ => {}
    }
}
pub fn spawn_glowing_light(
    p_lights: &mut PLightsState,
    p_setup: &mut PSetupState,
    p_tick: &mut PTickState,
    sector: SectorId,
) {
    let lightlevel = i32::from(p_setup.sector_mut(sector).lightlevel);
    let mut g = Glow {
        sector,
        minlight: find_min_surrounding_light(p_setup, sector, lightlevel),
        maxlight: lightlevel,
        ..Glow::default()
    };
    g.thinker.function = ThinkerFn::Glow(glow);
    g.direction = Direction::Down;
    let g_arena_id = p_lights.spawn_glow(g);
    add_thinker(p_tick, ThinkerPayload::Glow(g_arena_id), ThinkerKind::Glow);
    p_setup.sector_mut(sector).special = 0;
}
