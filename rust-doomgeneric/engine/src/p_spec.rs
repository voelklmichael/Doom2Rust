use crate::d_player::CheatFlags;
use crate::d_player::PowerType;
use crate::p_mobj::LineFlags;
use crate::p_setup::PSetupState;
use crate::r_data::RDataState;
use crate::w_wad::WWadState;

use crate::fixed_cstr::FixedCStr;
use crate::g_game::exit_level;
use crate::g_game::secret_exit_level;
use crate::i_system::error;
use crate::m_argv::check_parm_with_args;
use crate::m_fixed::Fixed;
use crate::m_misc::str_to_int;
use crate::m_random::p_random;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::p_ceilng::ceiling_crush_stop;
use crate::p_ceilng::do_ceiling;
use crate::p_ceilng::CeilingE;
use crate::p_doors::do_door;
use crate::p_doors::spawn_door_close_in30;
use crate::p_doors::spawn_door_raise_in5_mins;
use crate::p_doors::VldoorE;
use crate::p_floor::build_stairs;
use crate::p_floor::do_floor;
use crate::p_floor::FloorE;
use crate::p_floor::StairE;
use crate::p_inter::damage_mobj;
use crate::p_lights::light_turn_on;
use crate::p_lights::spawn_fire_flicker;
use crate::p_lights::spawn_glowing_light;
use crate::p_lights::spawn_light_flash;
use crate::p_lights::spawn_strobe_flash;
use crate::p_lights::start_light_strobing;
use crate::p_lights::turn_tag_lights_off;

use crate::p_mobj::SectorSpecial;
use crate::p_mobj::Thinker;
use crate::p_mobj::ThinkerFn;
use crate::p_plats::do_plat;
use crate::p_plats::stop_plat;
use crate::p_plats::PlatE;
use crate::p_plats::PlattypeE;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_setup::SideId;
use crate::p_switch::change_switch_texture;
use crate::p_switch::BWhere;
use crate::p_telept::teleport;
use crate::p_tick::add_thinker;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::r_data::check_texture_num_for_name;
use crate::r_data::flat_num_for_name;
use crate::r_data::texture_num_for_name;

use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;

use crate::w_wad::check_num_for_name;

use crate::d_player::PlayerId;
use crate::doomdef::TICRATE;
use crate::game_state::GameState;
use crate::m_fixed::FRACUNIT;
use crate::m_fixed::INT_MAX;
use crate::p_ceilng::MAXCEILINGS;
use crate::p_floor::move_floor;
use crate::p_floor::FLOORSPEED;
use crate::p_lights::SLOWDARK;
use crate::p_mobj::MobjId;
use crate::p_plats::MAXPLATS;
use crate::p_switch::EMPTY_BUTTON;
use crate::p_switch::MAXBUTTONS;

// Generation-checked handle into PSpecState's floor arena -- mirrors DoorId.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FloorId {
    index: u32,
    generation: u32,
}

struct FloorSlot {
    generation: u32,
    floor: Option<Box<FloorMove>>,
}

pub struct PSpecState {
    pub anims: [Anim; 32],
    pub lastanim: usize,
    pub level_timer: bool,
    pub level_time_count: i32,
    pub numlinespecials: i16,
    pub linespeciallist: [LineId; 64],
    pub donut_overrun_first: bool,
    pub donut_overrun_tmp_s3_floorheight: i32,
    pub donut_overrun_tmp_s3_floorpic: i32,
    floors: Vec<FloorSlot>,
    floor_free_list: Vec<u32>,
}

impl Default for PSpecState {
    fn default() -> Self {
        Self::new()
    }
}

impl PSpecState {
    pub const fn new() -> Self {
        Self {
            anims: [Anim {
                istexture: false,
                picnum: 0,
                basepic: 0,
                numpics: 0,
                speed: 0,
            }; 32],
            lastanim: 0,
            level_timer: false,
            level_time_count: 0,
            numlinespecials: 0,
            linespeciallist: [LineId(0); 64],
            donut_overrun_first: true,
            donut_overrun_tmp_s3_floorheight: 0,
            donut_overrun_tmp_s3_floorpic: 0,
            floors: Vec::new(),
            floor_free_list: Vec::new(),
        }
    }

    // Moves a fully-defaulted (then caller-filled) FloorMove onto the
    // heap and hands back both a stable generation-checked handle (stored
    // in ThinkerNode's payload by p_tick.rs, replacing what used to be a
    // bare raw pointer there) and a raw pointer for the caller's immediate
    // post-spawn field writes -- mirrors PDoorsState::spawn exactly. Lives
    // on PSpecState (rather than a new PFloorState) because FloorMove
    // itself is defined here, and both p_floor.rs and this file's own
    // donut-overrun special case construct one.
    pub fn spawn_floor(&mut self, value: FloorMove) -> FloorId {
        let (index, generation) = if let Some(index) = self.floor_free_list.pop() {
            let slot = &mut self.floors[index as usize];
            slot.generation = slot.generation.wrapping_add(1);
            (index, slot.generation)
        } else {
            let index = self.floors.len() as u32;
            self.floors.push(FloorSlot {
                generation: 0,
                floor: None,
            });
            (index, 0)
        };
        let id = FloorId { index, generation };
        let boxed = Box::new(value);
        self.floors[index as usize].floor = Some(boxed);
        id
    }

    pub fn get_floor_ref(&self, id: FloorId) -> Option<&FloorMove> {
        self.floors
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.floor.as_deref())
    }

    pub fn get_floor_mut(&mut self, id: FloorId) -> Option<&mut FloorMove> {
        self.floors
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.floor.as_deref_mut())
    }

    // Called once, from run_thinkers' reaper, when a Floor-kind thinker is
    // reaped.
    pub fn dealloc_floor(&mut self, id: FloorId) {
        if let Some(slot) = self.floors.get_mut(id.index as usize) {
            if slot.generation == id.generation {
                slot.floor = None;
                self.floor_free_list.push(id.index);
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct Anim {
    pub istexture: bool,
    pub picnum: i32,
    pub basepic: i32,
    pub numpics: i32,
    pub speed: i32,
}
#[derive(Copy, Clone)]
pub struct AnimDef {
    pub istexture: bool,
    pub endname: FixedCStr<9>,
    pub startname: FixedCStr<9>,
    pub speed: i32,
}
#[derive(Copy, Clone)]
pub struct Button {
    pub line: LineId,
    pub position: BWhere,
    pub btexture: i32,
    pub btimer: i32,
    pub soundorg: SectorId,
}
#[derive(Copy, Clone)]
pub struct Plat {
    pub thinker: Thinker,
    pub sector: SectorId,
    pub speed: Fixed,
    pub low: Fixed,
    pub high: Fixed,
    pub wait: i32,
    pub count: i32,
    pub status: PlatE,
    pub oldstatus: PlatE,
    pub crush: bool,
    pub tag: i32,
    pub kind: PlattypeE,
}
// Placeholder passed to PPlatsState::spawn() -- every real field is set by
// the caller within a few lines of spawn() returning (do_plat, and
// p_saveg.rs's restore branch), so these values are never actually read.
impl Default for Plat {
    fn default() -> Self {
        Self {
            thinker: Thinker {
                function: ThinkerFn::Unresolved,
            },
            sector: SectorId(0),
            speed: 0,
            low: 0,
            high: 0,
            wait: 0,
            count: 0,
            status: PlatE::Up,
            oldstatus: PlatE::Up,
            crush: false,
            tag: 0,
            kind: PlattypeE::PerpetualRaise,
        }
    }
}
/// Which way a moving sector plane (door, ceiling, floor) is heading: `direction` in the C source.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Direction {
    /// Lowering / closing.
    Down = -1,
    /// Not moving: a ceiling in stasis, or a door waiting at the top for its countdown.
    #[default]
    Still,
    /// Raising / opening.
    Up,
    /// A door that has not started its countdown yet (`P_SpawnDoorRaiseIn5Mins`).
    InitialWait,
}

impl Direction {
    /// The value the savegame stores.
    pub const fn to_save(self) -> i32 {
        self as i32
    }

    /// The direction a savegame value stands for. A value this engine never writes is read as
    /// [`Direction::Still`] (vanilla kept the garbage, which no thinker `match` handled).
    pub const fn from_save(value: i32) -> Self {
        match value {
            -1 => Self::Down,
            1 => Self::Up,
            2 => Self::InitialWait,
            _ => Self::Still,
        }
    }
}

/// Which surface `move_plane` moves: `floorOrCeiling` in the C source.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Plane {
    Floor,
    Ceiling,
}

#[derive(Copy, Clone)]
pub struct Ceiling {
    pub thinker: Thinker,
    pub kind: CeilingE,
    pub sector: SectorId,
    pub bottomheight: Fixed,
    pub topheight: Fixed,
    pub speed: Fixed,
    pub crush: bool,
    pub direction: Direction,
    pub tag: i32,
    pub olddirection: Direction,
}
// Placeholder passed to PCeilngState::spawn() -- every real field is set by
// the caller within a few lines of spawn() returning (do_ceiling, and
// p_saveg.rs's restore branch), so these values are never actually read.
impl Default for Ceiling {
    fn default() -> Self {
        Self {
            thinker: Thinker {
                function: ThinkerFn::Unresolved,
            },
            kind: CeilingE::LowerToFloor,
            sector: SectorId(0),
            bottomheight: 0,
            topheight: 0,
            speed: 0,
            crush: false,
            direction: Direction::Still,
            tag: 0,
            olddirection: Direction::Still,
        }
    }
}
#[derive(Copy, Clone)]
pub struct FloorMove {
    pub thinker: Thinker,
    pub kind: FloorE,
    pub crush: bool,
    pub sector: SectorId,
    pub direction: Direction,
    pub newspecial: i32,
    pub texture: i16,
    pub floordestheight: Fixed,
    pub speed: Fixed,
}
// Placeholder passed to PSpecState::spawn_floor() -- every real field is set
// by the caller within a few lines of spawn() returning (do_floor,
// build_stairs x2, the donut-overrun sites below, and p_saveg.rs's
// restore branch), so these values are never actually read.
impl Default for FloorMove {
    fn default() -> Self {
        Self {
            thinker: Thinker {
                function: ThinkerFn::Unresolved,
            },
            kind: FloorE::LowerFloor,
            crush: false,
            sector: SectorId(0),
            direction: Direction::Still,
            newspecial: 0,
            texture: 0,
            floordestheight: 0,
            speed: 0,
        }
    }
}
pub const FASTDARK: i32 = 15;
pub static ANIMDEFS: [AnimDef; 22] = [
    AnimDef {
        istexture: false,
        endname: FixedCStr(*b"NUKAGE3\0\0"),
        startname: FixedCStr(*b"NUKAGE1\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: false,
        endname: FixedCStr(*b"FWATER4\0\0"),
        startname: FixedCStr(*b"FWATER1\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: false,
        endname: FixedCStr(*b"SWATER4\0\0"),
        startname: FixedCStr(*b"SWATER1\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: false,
        endname: FixedCStr(*b"LAVA4\0\0\0\0"),
        startname: FixedCStr(*b"LAVA1\0\0\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: false,
        endname: FixedCStr(*b"BLOOD3\0\0\0"),
        startname: FixedCStr(*b"BLOOD1\0\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: false,
        endname: FixedCStr(*b"RROCK08\0\0"),
        startname: FixedCStr(*b"RROCK05\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: false,
        endname: FixedCStr(*b"SLIME04\0\0"),
        startname: FixedCStr(*b"SLIME01\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: false,
        endname: FixedCStr(*b"SLIME08\0\0"),
        startname: FixedCStr(*b"SLIME05\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: false,
        endname: FixedCStr(*b"SLIME12\0\0"),
        startname: FixedCStr(*b"SLIME09\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"BLODGR4\0\0"),
        startname: FixedCStr(*b"BLODGR1\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"SLADRIP3\0"),
        startname: FixedCStr(*b"SLADRIP1\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"BLODRIP4\0"),
        startname: FixedCStr(*b"BLODRIP1\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"FIREWALL\0"),
        startname: FixedCStr(*b"FIREWALA\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"GSTFONT3\0"),
        startname: FixedCStr(*b"GSTFONT1\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"FIRELAVA\0"),
        startname: FixedCStr(*b"FIRELAV3\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"FIREMAG3\0"),
        startname: FixedCStr(*b"FIREMAG1\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"FIREBLU2\0"),
        startname: FixedCStr(*b"FIREBLU1\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"ROCKRED3\0"),
        startname: FixedCStr(*b"ROCKRED1\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"BFALL4\0\0\0"),
        startname: FixedCStr(*b"BFALL1\0\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"SFALL4\0\0\0"),
        startname: FixedCStr(*b"SFALL1\0\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"WFALL4\0\0\0"),
        startname: FixedCStr(*b"WFALL1\0\0\0"),
        speed: 8,
    },
    AnimDef {
        istexture: true,
        endname: FixedCStr(*b"DBRAIN4\0\0"),
        startname: FixedCStr(*b"DBRAIN1\0\0"),
        speed: 8,
    },
];
pub const MAXLINEANIMS: i32 = 64;
pub fn init_pic_anims(p_spec: &mut PSpecState, r_data: &RDataState, w_wad: &WWadState) {
    p_spec.lastanim = 0;
    for def in &ANIMDEFS {
        let startname = def.startname.as_str();
        let endname = def.endname.as_str();
        let (picnum, basepic) = if def.istexture {
            if check_texture_num_for_name(r_data, &startname).is_none() {
                continue;
            }
            (
                texture_num_for_name(r_data, &endname),
                texture_num_for_name(r_data, &startname),
            )
        } else {
            if check_num_for_name(w_wad, &startname).is_none() {
                continue;
            }
            (
                flat_num_for_name(r_data, w_wad, &endname),
                flat_num_for_name(r_data, w_wad, &startname),
            )
        };
        let anim = &mut p_spec.anims[p_spec.lastanim];
        anim.picnum = picnum;
        anim.basepic = basepic;
        anim.istexture = def.istexture;
        anim.numpics = picnum - basepic + 1;
        if anim.numpics < 2 {
            error(&format!(
                "P_InitPicAnims: bad cycle from {startname} to {endname}",
            ));
        }
        anim.speed = def.speed;
        p_spec.lastanim += 1;
    }
}
pub fn get_side(
    p_setup: &mut PSetupState,
    current_sector: SectorId,
    line: i32,
    side: i32,
) -> SideId {
    let line_id = p_setup.sector_mut(current_sector).lines[line as usize];
    let sidenum = p_setup.line(line_id).sidenum[side as usize];
    SideId(sidenum as u32)
}
pub fn get_sector(
    p_setup: &mut PSetupState,
    current_sector: SectorId,
    line: i32,
    side: i32,
) -> SectorId {
    let line_id = p_setup.sector_mut(current_sector).lines[line as usize];
    let sidenum = p_setup.line(line_id).sidenum[side as usize];
    p_setup.sides[sidenum as usize].sector
}
pub fn two_sided(p_setup: &mut PSetupState, sector: SectorId, line: i32) -> bool {
    let sec = p_setup.sector_mut(sector);
    let line_id = sec.lines[line as usize];
    p_setup.line(line_id).flags.contains(LineFlags::TWOSIDED)
}
pub fn get_next_sector(p_setup: &PSetupState, line: LineId, sec: SectorId) -> Option<SectorId> {
    let linev = p_setup.line(line);
    if !linev.flags.contains(LineFlags::TWOSIDED) {
        return None;
    }
    let front = linev.frontsector.unwrap();
    if front == sec {
        return linev.backsector;
    }
    Some(front)
}
pub fn find_lowest_floor_surrounding(p_setup: &mut PSetupState, sec: SectorId) -> Fixed {
    let mut floor: Fixed = p_setup.sector_mut(sec).floorheight;
    let linecount = p_setup.sector_mut(sec).linecount;
    for i in 0..linecount as usize {
        let check = p_setup.sector_mut(sec).lines[i];
        if let Some(other) = get_next_sector(p_setup, check, sec) {
            let other_floor = p_setup.sector_mut(other).floorheight;
            if other_floor < floor {
                floor = other_floor;
            }
        }
    }
    floor
}
pub fn find_highest_floor_surrounding(p_setup: &mut PSetupState, sec: SectorId) -> Fixed {
    let mut floor: Fixed = -(500) * FRACUNIT;
    let linecount = p_setup.sector_mut(sec).linecount;
    for i in 0..linecount as usize {
        let check = p_setup.sector_mut(sec).lines[i];
        if let Some(other) = get_next_sector(p_setup, check, sec) {
            let other_floor = p_setup.sector_mut(other).floorheight;
            if other_floor > floor {
                floor = other_floor;
            }
        }
    }
    floor
}
pub const MAX_ADJOINING_SECTORS: i32 = 20;
pub fn find_next_highest_floor(
    p_setup: &mut PSetupState,
    sec: SectorId,
    currentheight: i32,
) -> Fixed {
    let mut height: Fixed = currentheight as Fixed;
    let mut heightlist: [Fixed; 22] = [0; 22];
    let mut h: i32 = 0;
    let linecount = p_setup.sector_mut(sec).linecount;
    for i in 0..linecount as usize {
        let check = p_setup.sector_mut(sec).lines[i];
        if let Some(other) = get_next_sector(p_setup, check, sec) {
            let other_floor = p_setup.sector_mut(other).floorheight;
            if other_floor > height {
                if h == MAX_ADJOINING_SECTORS + 1 {
                    height = other_floor;
                } else if h == MAX_ADJOINING_SECTORS + 2 {
                    error("Sector with more than 22 adjoining sectors. Vanilla will crash here");
                }
                heightlist[h as usize] = other_floor;
                h += 1;
            }
        }
    }
    if h == 0 {
        return currentheight as Fixed;
    }
    let mut min = heightlist[0];
    for i in 1..h {
        if heightlist[i as usize] < min {
            min = heightlist[i as usize];
        }
    }
    min as Fixed
}
pub fn find_lowest_ceiling_surrounding(p_setup: &mut PSetupState, sec: SectorId) -> Fixed {
    let mut height: Fixed = INT_MAX;
    let linecount = p_setup.sector_mut(sec).linecount;
    for i in 0..linecount as usize {
        let check = p_setup.sector_mut(sec).lines[i];
        if let Some(other) = get_next_sector(p_setup, check, sec) {
            let other_ceiling = p_setup.sector_mut(other).ceilingheight;
            if other_ceiling < height {
                height = other_ceiling;
            }
        }
    }
    height
}
pub fn find_highest_ceiling_surrounding(p_setup: &mut PSetupState, sec: SectorId) -> Fixed {
    let mut height: Fixed = 0;
    let linecount = p_setup.sector_mut(sec).linecount;
    for i in 0..linecount as usize {
        let check = p_setup.sector_mut(sec).lines[i];
        if let Some(other) = get_next_sector(p_setup, check, sec) {
            let other_ceiling = p_setup.sector_mut(other).ceilingheight;
            if other_ceiling > height {
                height = other_ceiling;
            }
        }
    }
    height
}
/// The next sector after `start` (`-1` to begin) that carries the same tag as
/// `line`.
pub fn find_sector_from_line_tag(p_setup: &PSetupState, line: LineId, start: i32) -> Option<i32> {
    let line_tag = p_setup.line(line).tag;
    (start + 1..p_setup.numsectors).find(|&i| p_setup.sectors[i as usize].tag == line_tag)
}
/// The sectors that carry the same tag as `line`, in map order.
pub fn sectors_with_line_tag(p_setup: &PSetupState, line: LineId) -> Vec<SectorId> {
    let mut found = Vec::new();
    let mut next = -1;
    while let Some(i) = find_sector_from_line_tag(p_setup, line, next) {
        found.push(SectorId(i as u32));
        next = i;
    }
    found
}
pub fn find_min_surrounding_light(p_setup: &mut PSetupState, sector: SectorId, max: i32) -> i32 {
    let mut min = max;
    let linecount = p_setup.sector_mut(sector).linecount;
    for i in 0..linecount as usize {
        let line = p_setup.sector_mut(sector).lines[i];
        if let Some(check) = get_next_sector(p_setup, line, sector) {
            let light = p_setup.sector_mut(check).lightlevel as i32;
            if light < min {
                min = light;
            }
        }
    }
    min
}
pub fn cross_special_line(state: &mut GameState, linenum: i32, side: i32, thing: MobjId) {
    let line: LineId = LineId(linenum as u32);
    let special = state.world.p_setup.line(line).special;
    if state.world.p_mobj.mo(thing).player.is_none() {
        match state.world.p_mobj.mo(thing).kind as u32 {
            33 | 34 | 35 | 31 | 32 | 16 => return,
            _ => {}
        }
        let mut ok: i32 = 0;
        match special as i32 {
            39 | 97 | 125 | 126 | 4 | 10 | 88 => {
                ok = 1;
            }
            _ => {}
        }
        if ok == 0 {
            return;
        }
    }
    match special as i32 {
        2 => {
            do_door(state, line, VldoorE::Open);
            state.world.p_setup.line_mut(line).special = 0;
        }
        3 => {
            do_door(state, line, VldoorE::Close);
            state.world.p_setup.line_mut(line).special = 0;
        }
        4 => {
            do_door(state, line, VldoorE::Normal);
            state.world.p_setup.line_mut(line).special = 0;
        }
        5 => {
            do_floor(state, line, FloorE::RaiseFloor);
            state.world.p_setup.line_mut(line).special = 0;
        }
        6 => {
            do_ceiling(
                &mut state.world.p_ceilng,
                &mut state.world.p_setup,
                &mut state.world.p_tick,
                line,
                CeilingE::FastCrushAndRaise,
            );
            state.world.p_setup.line_mut(line).special = 0;
        }
        8 => {
            build_stairs(
                &mut state.world.p_setup,
                &mut state.world.p_spec,
                &mut state.world.p_tick,
                line,
                StairE::Build8,
            );
            state.world.p_setup.line_mut(line).special = 0;
        }
        10 => {
            do_plat(state, line, PlattypeE::DownWaitUpStay, 0);
            state.world.p_setup.line_mut(line).special = 0;
        }
        12 => {
            light_turn_on(&mut state.world.p_setup, line, 0);
            state.world.p_setup.line_mut(line).special = 0;
        }
        13 => {
            light_turn_on(&mut state.world.p_setup, line, 255);
            state.world.p_setup.line_mut(line).special = 0;
        }
        16 => {
            do_door(state, line, VldoorE::Close30ThenOpen);
            state.world.p_setup.line_mut(line).special = 0;
        }
        17 => {
            start_light_strobing(&mut state.world, line);
            state.world.p_setup.line_mut(line).special = 0;
        }
        19 => {
            do_floor(state, line, FloorE::LowerFloor);
            state.world.p_setup.line_mut(line).special = 0;
        }
        22 => {
            do_plat(state, line, PlattypeE::RaiseToNearestAndChange, 0);
            state.world.p_setup.line_mut(line).special = 0;
        }
        25 => {
            do_ceiling(
                &mut state.world.p_ceilng,
                &mut state.world.p_setup,
                &mut state.world.p_tick,
                line,
                CeilingE::CrushAndRaise,
            );
            state.world.p_setup.line_mut(line).special = 0;
        }
        30 => {
            do_floor(state, line, FloorE::RaiseToTexture);
            state.world.p_setup.line_mut(line).special = 0;
        }
        35 => {
            light_turn_on(&mut state.world.p_setup, line, 35);
            state.world.p_setup.line_mut(line).special = 0;
        }
        36 => {
            do_floor(state, line, FloorE::TurboLower);
            state.world.p_setup.line_mut(line).special = 0;
        }
        37 => {
            do_floor(state, line, FloorE::LowerAndChange);
            state.world.p_setup.line_mut(line).special = 0;
        }
        38 => {
            do_floor(state, line, FloorE::LowerFloorToLowest);
            state.world.p_setup.line_mut(line).special = 0;
        }
        39 => {
            teleport(state, line, side, thing);
            state.world.p_setup.line_mut(line).special = 0;
        }
        40 => {
            do_ceiling(
                &mut state.world.p_ceilng,
                &mut state.world.p_setup,
                &mut state.world.p_tick,
                line,
                CeilingE::RaiseToHighest,
            );
            do_floor(state, line, FloorE::LowerFloorToLowest);
            state.world.p_setup.line_mut(line).special = 0;
        }
        44 => {
            do_ceiling(
                &mut state.world.p_ceilng,
                &mut state.world.p_setup,
                &mut state.world.p_tick,
                line,
                CeilingE::LowerAndCrush,
            );
            state.world.p_setup.line_mut(line).special = 0;
        }
        52 => {
            exit_level(&mut state.game.g_game);
        }
        53 => {
            do_plat(state, line, PlattypeE::PerpetualRaise, 0);
            state.world.p_setup.line_mut(line).special = 0;
        }
        54 => {
            stop_plat(
                &mut state.world.p_plats,
                &state.world.p_tick,
                state.world.p_setup.line(line).tag as i32,
            );
            state.world.p_setup.line_mut(line).special = 0;
        }
        56 => {
            do_floor(state, line, FloorE::RaiseFloorCrush);
            state.world.p_setup.line_mut(line).special = 0;
        }
        57 => {
            ceiling_crush_stop(
                &mut state.world.p_ceilng,
                &state.world.p_tick,
                state.world.p_setup.line(line).tag as i32,
            );
            state.world.p_setup.line_mut(line).special = 0;
        }
        58 => {
            do_floor(state, line, FloorE::RaiseFloor24);
            state.world.p_setup.line_mut(line).special = 0;
        }
        59 => {
            do_floor(state, line, FloorE::RaiseFloor24AndChange);
            state.world.p_setup.line_mut(line).special = 0;
        }
        104 => {
            turn_tag_lights_off(&mut state.world.p_setup, line);
            state.world.p_setup.line_mut(line).special = 0;
        }
        108 => {
            do_door(state, line, VldoorE::BlazeRaise);
            state.world.p_setup.line_mut(line).special = 0;
        }
        109 => {
            do_door(state, line, VldoorE::BlazeOpen);
            state.world.p_setup.line_mut(line).special = 0;
        }
        100 => {
            build_stairs(
                &mut state.world.p_setup,
                &mut state.world.p_spec,
                &mut state.world.p_tick,
                line,
                StairE::Turbo16,
            );
            state.world.p_setup.line_mut(line).special = 0;
        }
        110 => {
            do_door(state, line, VldoorE::BlazeClose);
            state.world.p_setup.line_mut(line).special = 0;
        }
        119 => {
            do_floor(state, line, FloorE::RaiseFloorToNearest);
            state.world.p_setup.line_mut(line).special = 0;
        }
        121 => {
            do_plat(state, line, PlattypeE::BlazeDWUS, 0);
            state.world.p_setup.line_mut(line).special = 0;
        }
        124 => {
            secret_exit_level(
                &state.game.doomstat,
                &mut state.game.g_game,
                &state.assets.w_wad,
            );
        }
        125 => {
            if state.world.p_mobj.mo(thing).player.is_none() {
                teleport(state, line, side, thing);
                state.world.p_setup.line_mut(line).special = 0;
            }
        }
        130 => {
            do_floor(state, line, FloorE::RaiseFloorTurbo);
            state.world.p_setup.line_mut(line).special = 0;
        }
        141 => {
            do_ceiling(
                &mut state.world.p_ceilng,
                &mut state.world.p_setup,
                &mut state.world.p_tick,
                line,
                CeilingE::SilentCrushAndRaise,
            );
            state.world.p_setup.line_mut(line).special = 0;
        }
        72 => {
            do_ceiling(
                &mut state.world.p_ceilng,
                &mut state.world.p_setup,
                &mut state.world.p_tick,
                line,
                CeilingE::LowerAndCrush,
            );
        }
        73 => {
            do_ceiling(
                &mut state.world.p_ceilng,
                &mut state.world.p_setup,
                &mut state.world.p_tick,
                line,
                CeilingE::CrushAndRaise,
            );
        }
        74 => {
            ceiling_crush_stop(
                &mut state.world.p_ceilng,
                &state.world.p_tick,
                state.world.p_setup.line(line).tag as i32,
            );
        }
        75 => {
            do_door(state, line, VldoorE::Close);
        }
        76 => {
            do_door(state, line, VldoorE::Close30ThenOpen);
        }
        77 => {
            do_ceiling(
                &mut state.world.p_ceilng,
                &mut state.world.p_setup,
                &mut state.world.p_tick,
                line,
                CeilingE::FastCrushAndRaise,
            );
        }
        79 => {
            light_turn_on(&mut state.world.p_setup, line, 35);
        }
        80 => {
            light_turn_on(&mut state.world.p_setup, line, 0);
        }
        81 => {
            light_turn_on(&mut state.world.p_setup, line, 255);
        }
        82 => {
            do_floor(state, line, FloorE::LowerFloorToLowest);
        }
        83 => {
            do_floor(state, line, FloorE::LowerFloor);
        }
        84 => {
            do_floor(state, line, FloorE::LowerAndChange);
        }
        86 => {
            do_door(state, line, VldoorE::Open);
        }
        87 => {
            do_plat(state, line, PlattypeE::PerpetualRaise, 0);
        }
        88 => {
            do_plat(state, line, PlattypeE::DownWaitUpStay, 0);
        }
        89 => {
            stop_plat(
                &mut state.world.p_plats,
                &state.world.p_tick,
                state.world.p_setup.line(line).tag as i32,
            );
        }
        90 => {
            do_door(state, line, VldoorE::Normal);
        }
        91 => {
            do_floor(state, line, FloorE::RaiseFloor);
        }
        92 => {
            do_floor(state, line, FloorE::RaiseFloor24);
        }
        93 => {
            do_floor(state, line, FloorE::RaiseFloor24AndChange);
        }
        94 => {
            do_floor(state, line, FloorE::RaiseFloorCrush);
        }
        95 => {
            do_plat(state, line, PlattypeE::RaiseToNearestAndChange, 0);
        }
        96 => {
            do_floor(state, line, FloorE::RaiseToTexture);
        }
        97 => {
            teleport(state, line, side, thing);
        }
        98 => {
            do_floor(state, line, FloorE::TurboLower);
        }
        105 => {
            do_door(state, line, VldoorE::BlazeRaise);
        }
        106 => {
            do_door(state, line, VldoorE::BlazeOpen);
        }
        107 => {
            do_door(state, line, VldoorE::BlazeClose);
        }
        120 => {
            do_plat(state, line, PlattypeE::BlazeDWUS, 0);
        }
        126 => {
            if state.world.p_mobj.mo(thing).player.is_none() {
                teleport(state, line, side, thing);
            }
        }
        128 => {
            do_floor(state, line, FloorE::RaiseFloorToNearest);
        }
        129 => {
            do_floor(state, line, FloorE::RaiseFloorTurbo);
        }
        _ => {}
    }
}
pub fn shoot_special_line(state: &mut GameState, thing: MobjId, line: LineId) {
    let special = state.world.p_setup.line(line).special;
    if state.world.p_mobj.mo(thing).player.is_none() {
        let mut ok: i32 = 0;
        if special as i32 == 46 {
            ok = 1;
        }
        if ok == 0 {
            return;
        }
    }
    match special as i32 {
        24 => {
            do_floor(state, line, FloorE::RaiseFloor);
            change_switch_texture(state, line, false);
        }
        46 => {
            do_door(state, line, VldoorE::Open);
            change_switch_texture(state, line, true);
        }
        47 => {
            do_plat(state, line, PlattypeE::RaiseToNearestAndChange, 0);
            change_switch_texture(state, line, false);
        }
        _ => {}
    }
}
pub fn player_in_special_sector(state: &mut GameState, player: PlayerId) {
    let player_mo = state.game.g_game.player_mut(player).mo.unwrap();
    let (subsector, mo_z) = {
        let m = state.world.p_mobj.mo(player_mo);
        (m.subsector, m.z)
    };
    let sector_id = state.world.p_setup.subsectors[subsector.0 as usize].sector;
    let (floorheight, special) = {
        let s = state.world.p_setup.sector_mut(sector_id);
        (s.floorheight, s.special)
    };
    if mo_z != floorheight {
        return;
    }
    match special as i32 {
        5 => {
            if state.game.g_game.player_mut(player).powers[PowerType::Ironfeet] == 0
                && state.world.p_tick.leveltime & 0x1f == 0
            {
                damage_mobj(state, player_mo, None, None, 10);
            }
        }
        7 => {
            if state.game.g_game.player_mut(player).powers[PowerType::Ironfeet] == 0
                && state.world.p_tick.leveltime & 0x1f == 0
            {
                damage_mobj(state, player_mo, None, None, 5);
            }
        }
        16 | 4 => {
            if (state.game.g_game.player_mut(player).powers[PowerType::Ironfeet] == 0
                || p_random(&mut state.world.m_random) < 5)
                && state.world.p_tick.leveltime & 0x1f == 0
            {
                damage_mobj(state, player_mo, None, None, 20);
            }
        }
        9 => {
            state.game.g_game.player_mut(player).secretcount += 1;
            state.world.p_setup.sector_mut(sector_id).special = 0;
        }
        11 => {
            state.game.g_game.player_mut(player).cheats &= !CheatFlags::GODMODE;
            if state.world.p_tick.leveltime & 0x1f == 0 {
                damage_mobj(state, player_mo, None, None, 20);
            }
            if state.game.g_game.player_mut(player).health <= 10 {
                exit_level(&mut state.game.g_game);
            }
        }
        _ => {
            error(&format!(
                "P_PlayerInSpecialSector: unknown special {}",
                special as i32,
            ));
        }
    }
}
pub fn update_specials(state: &mut GameState) {
    if state.world.p_spec.level_timer {
        state.world.p_spec.level_time_count -= 1;
        if state.world.p_spec.level_time_count == 0 {
            exit_level(&mut state.game.g_game);
        }
    }
    for anim_idx in 0..state.world.p_spec.lastanim {
        let anim = &state.world.p_spec.anims[anim_idx];
        for i in anim.basepic..anim.basepic + anim.numpics {
            let pic: i32 =
                anim.basepic + (state.world.p_tick.leveltime / anim.speed + i) % anim.numpics;
            if anim.istexture {
                state.render.r_data.texturetranslation[i as usize] = pic;
            } else {
                state.render.r_data.flattranslation[i as usize] = pic;
            }
        }
    }
    for i in 0..(state.world.p_spec.numlinespecials as usize) {
        let line: LineId = state.world.p_spec.linespeciallist[i];
        let linev = state.world.p_setup.line(line);
        if linev.special as i32 == 48 {
            state.world.p_setup.sides[linev.sidenum[0] as usize].textureoffset += FRACUNIT;
        }
    }
    for i in 0..(MAXBUTTONS as usize) {
        if state.world.p_switch.buttonlist[i].btimer != 0 {
            state.world.p_switch.buttonlist[i].btimer -= 1;
            if state.world.p_switch.buttonlist[i].btimer == 0 {
                let button_line_id = state.world.p_switch.buttonlist[i].line;
                match state.world.p_switch.buttonlist[i].position {
                    BWhere::Top => {
                        state.world.p_setup.sides[state.world.p_setup.lines
                            [button_line_id.0 as usize]
                            .sidenum[0]
                            as usize]
                            .toptexture = state.world.p_switch.buttonlist[i].btexture as i16;
                    }
                    BWhere::Middle => {
                        state.world.p_setup.sides[state.world.p_setup.lines
                            [button_line_id.0 as usize]
                            .sidenum[0]
                            as usize]
                            .midtexture = state.world.p_switch.buttonlist[i].btexture as i16;
                    }
                    BWhere::Bottom => {
                        state.world.p_setup.sides[state.world.p_setup.lines
                            [button_line_id.0 as usize]
                            .sidenum[0]
                            as usize]
                            .bottomtexture = state.world.p_switch.buttonlist[i].btexture as i16;
                    }
                }
                s_start_sound(
                    state,
                    SoundOrigin::Sector(state.world.p_switch.buttonlist[i].soundorg),
                    SfxName::Swtchn,
                );
                state.world.p_switch.buttonlist[i] = EMPTY_BUTTON;
            }
        }
    }
}
pub const DONUT_FLOORHEIGHT_DEFAULT: i32 = 0;
pub const DONUT_FLOORPIC_DEFAULT: i32 = 0x16;
fn donut_overrun(state: &mut GameState) -> (Fixed, i16) {
    if state.world.p_spec.donut_overrun_first {
        state.world.p_spec.donut_overrun_first = false;
        state.world.p_spec.donut_overrun_tmp_s3_floorheight = DONUT_FLOORHEIGHT_DEFAULT;
        state.world.p_spec.donut_overrun_tmp_s3_floorpic = DONUT_FLOORPIC_DEFAULT;
        if let Some(p) = check_parm_with_args(&state.game.m_argv, "-donut", 2) {
            str_to_int(
                state.game.m_argv.myargv[p + 1].as_str(),
                &mut state.world.p_spec.donut_overrun_tmp_s3_floorheight,
            );
            str_to_int(
                state.game.m_argv.myargv[p + 2].as_str(),
                &mut state.world.p_spec.donut_overrun_tmp_s3_floorpic,
            );
            if state.world.p_spec.donut_overrun_tmp_s3_floorpic >= state.render.r_data.numflats {
                doom_eprintln!(state.io.platform,
                    "DonutOverrun: The second parameter for \"-donut\" switch should be greater than 0 and less than number of flats ({}). Using default value ({}) instead. ",
                    state.render.r_data.numflats,
                    DONUT_FLOORPIC_DEFAULT,
                );
                state.world.p_spec.donut_overrun_tmp_s3_floorpic = DONUT_FLOORPIC_DEFAULT;
            }
        }
    }
    (
        state.world.p_spec.donut_overrun_tmp_s3_floorheight,
        state.world.p_spec.donut_overrun_tmp_s3_floorpic as i16,
    )
}
pub fn do_donut(state: &mut GameState, line: LineId) -> bool {
    let mut rtn = false;
    for sector in sectors_with_line_tag(&state.world.p_setup, line) {
        let s1 = sector;
        if state.world.p_setup.sector_mut(s1).specialdata.is_some() {
            continue;
        }
        rtn = true;
        let first_line = state.world.p_setup.sector_mut(s1).lines[0];
        let Some(s2) = get_next_sector(&state.world.p_setup, first_line, s1) else {
            doom_eprintln!(state.io.platform,
                "EV_DoDonut: linedef had no second sidedef! Unexpected behavior may occur in Vanilla Doom. "
            );
            break;
        };
        let linecount = state.world.p_setup.sector_mut(s2).linecount;
        for i in 0..linecount as usize {
            let s2_line_id = state.world.p_setup.sector_mut(s2).lines[i];
            let s3 = state.world.p_setup.line(s2_line_id).backsector;
            if s3 == Some(s1) {
                continue;
            }
            let (s3_floorheight, s3_floorpic) = if let Some(id) = s3 {
                let s3 = state.world.p_setup.sector_mut(id);
                (s3.floorheight, s3.floorpic)
            } else {
                doom_eprintln!(state.io.platform,
                    "EV_DoDonut: WARNING: emulating buffer overrun due to NULL back sector. Unexpected behavior may occur in Vanilla Doom."
                );
                donut_overrun(state)
            };
            let mut floor = FloorMove::default();
            floor.thinker.function = ThinkerFn::Floor(move_floor);
            floor.kind = FloorE::DonutRaise;
            floor.crush = false;
            floor.direction = Direction::Up;
            floor.sector = s2;
            floor.speed = (FLOORSPEED / 2) as Fixed;
            floor.texture = s3_floorpic;
            floor.newspecial = 0;
            floor.floordestheight = s3_floorheight;
            let floor_arena_id = state.world.p_spec.spawn_floor(floor);
            let floor_id = add_thinker(
                &mut state.world.p_tick,
                ThinkerPayload::Floor(floor_arena_id),
                ThinkerKind::Floor,
            );
            state.world.p_setup.sector_mut(s2).specialdata = Some(SectorSpecial::Floor(floor_id));
            let mut floor = FloorMove::default();
            floor.thinker.function = ThinkerFn::Floor(move_floor);
            floor.kind = FloorE::LowerFloor;
            floor.crush = false;
            floor.direction = Direction::Down;
            floor.sector = s1;
            floor.speed = (FLOORSPEED / 2) as Fixed;
            floor.floordestheight = s3_floorheight;
            let floor_arena_id = state.world.p_spec.spawn_floor(floor);
            let floor_id = add_thinker(
                &mut state.world.p_tick,
                ThinkerPayload::Floor(floor_arena_id),
                ThinkerKind::Floor,
            );
            state.world.p_setup.sector_mut(s1).specialdata = Some(SectorSpecial::Floor(floor_id));
            break;
        }
    }
    rtn
}
pub fn spawn_specials(state: &mut GameState) {
    if state.game.g_game.timelimit > 0 && state.game.g_game.deathmatch != 0 {
        state.world.p_spec.level_timer = true;
        state.world.p_spec.level_time_count = state.game.g_game.timelimit * 60 * TICRATE;
    } else {
        state.world.p_spec.level_timer = false;
    }
    for i in 0..state.world.p_setup.numsectors {
        let secid = SectorId(i as u32);
        let special = state.world.p_setup.sector_mut(secid).special;
        if special != 0 {
            match special as i32 {
                1 => {
                    spawn_light_flash(&mut state.world, secid);
                }
                2 => {
                    spawn_strobe_flash(&mut state.world, secid, FASTDARK, 0);
                }
                3 => {
                    spawn_strobe_flash(&mut state.world, secid, SLOWDARK, 0);
                }
                4 => {
                    spawn_strobe_flash(&mut state.world, secid, FASTDARK, 0);
                    state.world.p_setup.sector_mut(secid).special = 4;
                }
                8 => {
                    spawn_glowing_light(
                        &mut state.world.p_lights,
                        &mut state.world.p_setup,
                        &mut state.world.p_tick,
                        secid,
                    );
                }
                9 => {
                    state.game.g_game.totalsecret += 1;
                }
                10 => {
                    spawn_door_close_in30(
                        &mut state.world.p_doors,
                        &mut state.world.p_setup,
                        &mut state.world.p_tick,
                        secid,
                    );
                }
                12 => {
                    spawn_strobe_flash(&mut state.world, secid, SLOWDARK, 1);
                }
                13 => {
                    spawn_strobe_flash(&mut state.world, secid, FASTDARK, 1);
                }
                14 => {
                    spawn_door_raise_in5_mins(
                        &mut state.world.p_doors,
                        &mut state.world.p_setup,
                        &mut state.world.p_tick,
                        secid,
                    );
                }
                17 => {
                    spawn_fire_flicker(
                        &mut state.world.p_lights,
                        &mut state.world.p_setup,
                        &mut state.world.p_tick,
                        secid,
                    );
                }
                _ => {}
            }
        }
    }
    state.world.p_spec.numlinespecials = 0;
    for i in 0..state.world.p_setup.numlines {
        if state.world.p_setup.lines[i as usize].special as i32 == 48 {
            if state.world.p_spec.numlinespecials as i32 >= MAXLINEANIMS {
                error("Too many scrolling wall linedefs! (Vanilla limit is 64)");
            }
            state.world.p_spec.linespeciallist[state.world.p_spec.numlinespecials as usize] =
                LineId(i as u32);
            state.world.p_spec.numlinespecials += 1;
        }
    }
    for i in 0..(MAXCEILINGS as usize) {
        state.world.p_ceilng.activeceilings[i] = None;
    }
    for i in 0..(MAXPLATS as usize) {
        state.world.p_plats.activeplats[i] = None;
    }
    for i in 0..(MAXBUTTONS as usize) {
        state.world.p_switch.buttonlist[i] = EMPTY_BUTTON;
    }
}

#[cfg(test)]
mod direction_tests {
    use super::{Direction, Plane};

    #[test]
    fn directions_keep_the_values_the_c_code_and_savegames_use() {
        assert_eq!(Direction::Down as i32, -1);
        assert_eq!(Direction::Still as i32, 0);
        assert_eq!(Direction::Up as i32, 1);
        assert_eq!(Direction::InitialWait as i32, 2);
        assert_eq!(Plane::Floor as i32, 0);
        assert_eq!(Plane::Ceiling as i32, 1);
    }

    #[test]
    fn a_direction_survives_the_savegame_round_trip() {
        for direction in [
            Direction::Down,
            Direction::Still,
            Direction::Up,
            Direction::InitialWait,
        ] {
            assert_eq!(Direction::from_save(direction.to_save()), direction);
        }
    }

    #[test]
    fn a_value_the_engine_never_writes_reads_as_still() {
        assert_eq!(Direction::from_save(7), Direction::Still);
        assert_eq!(Direction::from_save(-5), Direction::Still);
    }
}
