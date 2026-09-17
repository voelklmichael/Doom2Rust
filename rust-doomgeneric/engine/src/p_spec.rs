use crate::d_player::player_t;
use crate::d_player::PowerType;
use crate::d_player::CF_GODMODE;
use crate::fixed_cstr::FixedCStr;
use crate::g_game::G_ExitLevel;
use crate::g_game::G_SecretExitLevel;
use crate::i_system::I_Error;
use crate::m_argv::M_CheckParmWithArgs;
use crate::m_fixed::fixed_t;
use crate::m_misc::M_StrToInt;
use crate::m_random::P_Random;
use crate::mem_compat::memset;
use crate::p_ceilng::CeilingE;
use crate::p_ceilng::EV_CeilingCrushStop;
use crate::p_ceilng::EV_DoCeiling;
use crate::p_doors::EV_DoDoor;
use crate::p_doors::P_SpawnDoorCloseIn30;
use crate::p_doors::P_SpawnDoorRaiseIn5Mins;
use crate::p_doors::VldoorE;
use crate::p_floor::EV_BuildStairs;
use crate::p_floor::EV_DoFloor;
use crate::p_floor::FloorE;
use crate::p_floor::StairE;
use crate::p_inter::P_DamageMobj;
use crate::p_lights::EV_LightTurnOn;
use crate::p_lights::EV_StartLightStrobing;
use crate::p_lights::EV_TurnTagLightsOff;
use crate::p_lights::P_SpawnFireFlicker;
use crate::p_lights::P_SpawnGlowingLight;
use crate::p_lights::P_SpawnLightFlash;
use crate::p_lights::P_SpawnStrobeFlash;
use crate::p_mobj::mobj_t;
use crate::p_mobj::SectorSpecial;
use crate::p_mobj::ThinkerFn;
use crate::p_mobj::{sector_t, thinker_t};
use crate::p_plats::EV_DoPlat;
use crate::p_plats::EV_StopPlat;
use crate::p_plats::PlatE;
use crate::p_plats::PlattypeE;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_setup::SideId;
use crate::p_switch::BWhere;
use crate::p_switch::P_ChangeSwitchTexture;
use crate::p_telept::EV_Teleport;
use crate::p_tick::P_AddThinker;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::r_data::R_CheckTextureNumForName;
use crate::r_data::R_FlatNumForName;
use crate::r_data::R_TextureNumForName;
use crate::r_defs::side_t;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::sfx_swtchn;
use crate::stdint_types::size_t;
use crate::w_wad::W_CheckNumForName;

use crate::doomdef::false_0;
use crate::doomdef::true_0;
use crate::doomdef::TICRATE;
use crate::game_state::GameState;
use crate::m_fixed::FRACUNIT;
use crate::m_fixed::INT_MAX;
use crate::p_ceilng::MAXCEILINGS;
use crate::p_floor::T_MoveFloor;
use crate::p_floor::FLOORSPEED;
use crate::p_lights::SLOWDARK;
use crate::p_plats::MAXPLATS;
use crate::p_switch::MAXBUTTONS;

// Generation-checked handle into PSpecState's floor arena -- mirrors DoorId.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FloorId {
    index: u32,
    generation: u32,
}

struct FloorSlot {
    generation: u32,
    floor: Option<Box<floormove_t>>,
}

pub struct PSpecState {
    pub anims: [anim_t; 32],
    pub lastanim: usize,
    pub levelTimer: bool,
    pub levelTimeCount: i32,
    pub numlinespecials: i16,
    pub linespeciallist: [LineId; 64],
    pub donut_overrun_first: i32,
    pub donut_overrun_tmp_s3_floorheight: i32,
    pub donut_overrun_tmp_s3_floorpic: i32,
    floors: Vec<FloorSlot>,
    floor_free_list: Vec<u32>,
}

impl PSpecState {
    pub const fn new() -> Self {
        PSpecState {
            anims: [anim_t {
                istexture: false,
                picnum: 0,
                basepic: 0,
                numpics: 0,
                speed: 0,
            }; 32],
            lastanim: 0,
            levelTimer: false,
            levelTimeCount: 0,
            numlinespecials: 0,
            linespeciallist: [LineId(0); 64],
            donut_overrun_first: 1,
            donut_overrun_tmp_s3_floorheight: 0,
            donut_overrun_tmp_s3_floorpic: 0,
            floors: Vec::new(),
            floor_free_list: Vec::new(),
        }
    }

    // Moves a fully-defaulted (then caller-filled) floormove_t onto the
    // heap and hands back both a stable generation-checked handle (stored
    // in ThinkerNode's payload by p_tick.rs, replacing what used to be a
    // bare raw pointer there) and a raw pointer for the caller's immediate
    // post-spawn field writes -- mirrors PDoorsState::spawn exactly. Lives
    // on PSpecState (rather than a new PFloorState) because floormove_t
    // itself is defined here, and both p_floor.rs and this file's own
    // donut-overrun special case construct one.
    pub fn spawn_floor(&mut self, value: floormove_t) -> (FloorId, *mut floormove_t) {
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
        let mut boxed = Box::new(value);
        let ptr = boxed.as_mut() as *mut floormove_t;
        self.floors[index as usize].floor = Some(boxed);
        (id, ptr)
    }

    // Fallible materialization: None if the id is stale. Used by
    // p_tick.rs's P_ThinkerRaw to resolve a Floor-kind ThinkerNode's
    // payload back into the raw pointer every T_* function still expects.
    pub fn get_floor(&self, id: FloorId) -> Option<*mut floormove_t> {
        self.floors
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.floor.as_deref())
            .map(|r| r as *const floormove_t as *mut floormove_t)
    }

    // Called once, from P_RunThinkers' reaper, when a Floor-kind thinker is
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
#[repr(C)]
pub struct anim_t {
    pub istexture: bool,
    pub picnum: i32,
    pub basepic: i32,
    pub numpics: i32,
    pub speed: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct animdef_t {
    pub istexture: i32,
    pub endname: FixedCStr<9>,
    pub startname: FixedCStr<9>,
    pub speed: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct button_t {
    pub line: LineId,
    pub where_0: BWhere,
    pub btexture: i32,
    pub btimer: i32,
    pub soundorg: SectorId,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct plat_t {
    pub thinker: thinker_t,
    pub sector: SectorId,
    pub speed: fixed_t,
    pub low: fixed_t,
    pub high: fixed_t,
    pub wait: i32,
    pub count: i32,
    pub status: PlatE,
    pub oldstatus: PlatE,
    pub crush: bool,
    pub tag: i32,
    pub type_0: PlattypeE,
}
// Placeholder passed to PPlatsState::spawn() -- every real field is set by
// the caller within a few lines of spawn() returning (EV_DoPlat, and
// p_saveg.rs's restore branch), so these values are never actually read.
impl Default for plat_t {
    fn default() -> Self {
        plat_t {
            thinker: thinker_t {
                function: ThinkerFn::Unresolved,
            },
            sector: SectorId(0),
            speed: 0,
            low: 0,
            high: 0,
            wait: 0,
            count: 0,
            status: PlatE::up,
            oldstatus: PlatE::up,
            crush: false,
            tag: 0,
            type_0: PlattypeE::perpetualRaise,
        }
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ceiling_t {
    pub thinker: thinker_t,
    pub type_0: CeilingE,
    pub sector: SectorId,
    pub bottomheight: fixed_t,
    pub topheight: fixed_t,
    pub speed: fixed_t,
    pub crush: bool,
    pub direction: i32,
    pub tag: i32,
    pub olddirection: i32,
}
// Placeholder passed to PCeilngState::spawn() -- every real field is set by
// the caller within a few lines of spawn() returning (EV_DoCeiling, and
// p_saveg.rs's restore branch), so these values are never actually read.
impl Default for ceiling_t {
    fn default() -> Self {
        ceiling_t {
            thinker: thinker_t {
                function: ThinkerFn::Unresolved,
            },
            type_0: CeilingE::lowerToFloor,
            sector: SectorId(0),
            bottomheight: 0,
            topheight: 0,
            speed: 0,
            crush: false,
            direction: 0,
            tag: 0,
            olddirection: 0,
        }
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct floormove_t {
    pub thinker: thinker_t,
    pub type_0: FloorE,
    pub crush: bool,
    pub sector: SectorId,
    pub direction: i32,
    pub newspecial: i32,
    pub texture: i16,
    pub floordestheight: fixed_t,
    pub speed: fixed_t,
}
// Placeholder passed to PSpecState::spawn_floor() -- every real field is set
// by the caller within a few lines of spawn() returning (EV_DoFloor,
// EV_BuildStairs x2, the donut-overrun sites below, and p_saveg.rs's
// restore branch), so these values are never actually read.
impl Default for floormove_t {
    fn default() -> Self {
        floormove_t {
            thinker: thinker_t {
                function: ThinkerFn::Unresolved,
            },
            type_0: FloorE::lowerFloor,
            crush: false,
            sector: SectorId(0),
            direction: 0,
            newspecial: 0,
            texture: 0,
            floordestheight: 0,
            speed: 0,
        }
    }
}
pub const ML_TWOSIDED: i32 = 4;
pub const FASTDARK: i32 = 15;
pub static animdefs: [animdef_t; 23] = [
    animdef_t {
        istexture: false_0,
        endname: FixedCStr(*b"NUKAGE3\0\0"),
        startname: FixedCStr(*b"NUKAGE1\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: false_0,
        endname: FixedCStr(*b"FWATER4\0\0"),
        startname: FixedCStr(*b"FWATER1\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: false_0,
        endname: FixedCStr(*b"SWATER4\0\0"),
        startname: FixedCStr(*b"SWATER1\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: false_0,
        endname: FixedCStr(*b"LAVA4\0\0\0\0"),
        startname: FixedCStr(*b"LAVA1\0\0\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: false_0,
        endname: FixedCStr(*b"BLOOD3\0\0\0"),
        startname: FixedCStr(*b"BLOOD1\0\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: false_0,
        endname: FixedCStr(*b"RROCK08\0\0"),
        startname: FixedCStr(*b"RROCK05\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: false_0,
        endname: FixedCStr(*b"SLIME04\0\0"),
        startname: FixedCStr(*b"SLIME01\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: false_0,
        endname: FixedCStr(*b"SLIME08\0\0"),
        startname: FixedCStr(*b"SLIME05\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: false_0,
        endname: FixedCStr(*b"SLIME12\0\0"),
        startname: FixedCStr(*b"SLIME09\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"BLODGR4\0\0"),
        startname: FixedCStr(*b"BLODGR1\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"SLADRIP3\0"),
        startname: FixedCStr(*b"SLADRIP1\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"BLODRIP4\0"),
        startname: FixedCStr(*b"BLODRIP1\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"FIREWALL\0"),
        startname: FixedCStr(*b"FIREWALA\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"GSTFONT3\0"),
        startname: FixedCStr(*b"GSTFONT1\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"FIRELAVA\0"),
        startname: FixedCStr(*b"FIRELAV3\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"FIREMAG3\0"),
        startname: FixedCStr(*b"FIREMAG1\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"FIREBLU2\0"),
        startname: FixedCStr(*b"FIREBLU1\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"ROCKRED3\0"),
        startname: FixedCStr(*b"ROCKRED1\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"BFALL4\0\0\0"),
        startname: FixedCStr(*b"BFALL1\0\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"SFALL4\0\0\0"),
        startname: FixedCStr(*b"SFALL1\0\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"WFALL4\0\0\0"),
        startname: FixedCStr(*b"WFALL1\0\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: true_0,
        endname: FixedCStr(*b"DBRAIN4\0\0"),
        startname: FixedCStr(*b"DBRAIN1\0\0"),
        speed: 8_i32,
    },
    animdef_t {
        istexture: -1_i32,
        endname: FixedCStr(*b"\0\0\0\0\0\0\0\0\0"),
        startname: FixedCStr(*b"\0\0\0\0\0\0\0\0\0"),
        speed: 0_i32,
    },
];
pub const MAXLINEANIMS: i32 = 64;
pub fn P_InitPicAnims(state: &mut GameState) {
    let mut i: i32 = 0;
    state.p_spec.lastanim = 0;
    let mut current_block_13: u64;
    i = 0_i32;
    while animdefs[i as usize].istexture != -1_i32 {
        let startname = animdefs[i as usize].startname.as_str();
        let endname = animdefs[i as usize].endname.as_str();
        let anim = &mut state.p_spec.anims[state.p_spec.lastanim];
        if animdefs[i as usize].istexture != 0 {
            if R_CheckTextureNumForName(&mut state.r_data, &startname) == -1_i32 {
                current_block_13 = 12237857397564741460;
            } else {
                anim.picnum = R_TextureNumForName(&mut state.r_data, &endname);
                anim.basepic = R_TextureNumForName(&mut state.r_data, &startname);
                current_block_13 = 11650488183268122163;
            }
        } else if W_CheckNumForName(&mut state.w_wad, &startname) == -1_i32 {
            current_block_13 = 12237857397564741460;
        } else {
            let picnum = R_FlatNumForName(state, &endname);
            let basepic = R_FlatNumForName(state, &startname);
            let anim = &mut state.p_spec.anims[state.p_spec.lastanim];
            anim.picnum = picnum;
            anim.basepic = basepic;
            current_block_13 = 11650488183268122163;
        }
        match current_block_13 {
            11650488183268122163 => {
                let anim = &mut state.p_spec.anims[state.p_spec.lastanim];
                anim.istexture = animdefs[i as usize].istexture != 0;
                anim.numpics = anim.picnum - anim.basepic + 1_i32;
                if anim.numpics < 2_i32 {
                    I_Error(&format!(
                        "P_InitPicAnims: bad cycle from {} to {}",
                        startname, endname,
                    ));
                }
                anim.speed = animdefs[i as usize].speed;
                state.p_spec.lastanim += 1;
            }
            _ => {}
        }
        i += 1;
    }
}
pub unsafe fn getSide(
    state: &mut GameState,
    mut currentSector: i32,
    mut line: i32,
    mut side: i32,
) -> *mut side_t {
    let sec = state.p_setup.sector_mut(SectorId(currentSector as u32));
    let line_id = sec.lines[line as usize];
    let sidenum = state.p_setup.line(line_id).sidenum[side as usize];
    return state.p_setup.side_mut(SideId(sidenum as u32));
}
pub unsafe fn getSector(
    state: &mut GameState,
    mut currentSector: i32,
    mut line: i32,
    mut side: i32,
) -> *mut sector_t {
    let sec = state.p_setup.sector_mut(SectorId(currentSector as u32));
    let line_id = sec.lines[line as usize];
    let sidenum = state.p_setup.line(line_id).sidenum[side as usize];
    let sector_id = state.p_setup.sides[sidenum as usize].sector;
    return state.p_setup.sector_mut(sector_id);
}
pub unsafe fn twoSided(state: &mut GameState, mut sector: i32, mut line: i32) -> i32 {
    let sec = state.p_setup.sector_mut(SectorId(sector as u32));
    let line_id = sec.lines[line as usize];
    state.p_setup.line(line_id).flags as i32 & ML_TWOSIDED
}
pub fn getNextSector(
    state: &mut GameState,
    mut line: LineId,
    mut sec: *mut sector_t,
) -> *mut sector_t {
    let linev = state.p_setup.line(line);
    if linev.flags as i32 & ML_TWOSIDED == 0 {
        return ::core::ptr::null_mut::<sector_t>();
    }
    let front: *mut sector_t = state.p_setup.sector_mut(linev.frontsector.unwrap());
    if front == sec {
        return match linev.backsector {
            Some(id) => state.p_setup.sector_mut(id),
            None => ::core::ptr::null_mut::<sector_t>(),
        };
    }
    front
}
pub unsafe fn P_FindLowestFloorSurrounding(
    state: &mut GameState,
    mut sec: *mut sector_t,
) -> fixed_t {
    let mut i: i32 = 0;
    let mut check: LineId;
    let mut other: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut floor: fixed_t = (*sec).floorheight;
    i = 0_i32;
    while i < (*sec).linecount {
        check = (*sec).lines[i as usize];
        other = getNextSector(state, check, sec);
        if !other.is_null() && (*other).floorheight < floor {
            floor = (*other).floorheight;
        }
        i += 1;
    }
    floor
}
pub unsafe fn P_FindHighestFloorSurrounding(
    state: &mut GameState,
    mut sec: *mut sector_t,
) -> fixed_t {
    let mut i: i32 = 0;
    let mut check: LineId;
    let mut other: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut floor: fixed_t = -(500 as fixed_t) * FRACUNIT;
    i = 0_i32;
    while i < (*sec).linecount {
        check = (*sec).lines[i as usize];
        other = getNextSector(state, check, sec);
        if !other.is_null() && (*other).floorheight > floor {
            floor = (*other).floorheight;
        }
        i += 1;
    }
    floor
}
pub const MAX_ADJOINING_SECTORS: i32 = 20;
pub unsafe fn P_FindNextHighestFloor(
    state: &mut GameState,
    mut sec: *mut sector_t,
    mut currentheight: i32,
) -> fixed_t {
    let mut i: i32 = 0;
    let mut h: i32 = 0;
    let mut min: i32 = 0;
    let mut check: LineId;
    let mut other: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut height: fixed_t = currentheight as fixed_t;
    let mut heightlist: [fixed_t; 22] = [0; 22];
    i = 0_i32;
    h = 0_i32;
    while i < (*sec).linecount {
        check = (*sec).lines[i as usize];
        other = getNextSector(state, check, sec);
        if !other.is_null() && (*other).floorheight > height {
            if h == MAX_ADJOINING_SECTORS + 1_i32 {
                height = (*other).floorheight;
            } else if h == MAX_ADJOINING_SECTORS + 2_i32 {
                I_Error("Sector with more than 22 adjoining sectors. Vanilla will crash here");
            }
            let fresh1 = h;
            h += 1;
            heightlist[fresh1 as usize] = (*other).floorheight;
        }
        i += 1;
    }
    if h == 0 {
        return currentheight as fixed_t;
    }
    min = heightlist[0];
    i = 1_i32;
    while i < h {
        if heightlist[i as usize] < min {
            min = heightlist[i as usize];
        }
        i += 1;
    }
    min as fixed_t
}
pub unsafe fn P_FindLowestCeilingSurrounding(
    state: &mut GameState,
    mut sec: *mut sector_t,
) -> fixed_t {
    let mut i: i32 = 0;
    let mut check: LineId;
    let mut other: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut height: fixed_t = INT_MAX;
    i = 0_i32;
    while i < (*sec).linecount {
        check = (*sec).lines[i as usize];
        other = getNextSector(state, check, sec);
        if !other.is_null() && (*other).ceilingheight < height {
            height = (*other).ceilingheight;
        }
        i += 1;
    }
    height
}
pub unsafe fn P_FindHighestCeilingSurrounding(
    state: &mut GameState,
    mut sec: *mut sector_t,
) -> fixed_t {
    let mut i: i32 = 0;
    let mut check: LineId;
    let mut other: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut height: fixed_t = 0 as fixed_t;
    i = 0_i32;
    while i < (*sec).linecount {
        check = (*sec).lines[i as usize];
        other = getNextSector(state, check, sec);
        if !other.is_null() && (*other).ceilingheight > height {
            height = (*other).ceilingheight;
        }
        i += 1;
    }
    height
}
pub fn P_FindSectorFromLineTag(state: &mut GameState, mut line: LineId, mut start: i32) -> i32 {
    let mut i: i32 = 0;
    let line_tag = state.p_setup.line(line).tag;
    i = start + 1_i32;
    while i < state.p_setup.numsectors {
        if state.p_setup.sectors[i as usize].tag as i32 == line_tag as i32 {
            return i;
        }
        i += 1;
    }
    -1_i32
}
pub unsafe fn P_FindMinSurroundingLight(
    state: &mut GameState,
    mut sector: *mut sector_t,
    mut max: i32,
) -> i32 {
    let mut i: i32 = 0;
    let mut min: i32 = 0;
    let mut line: LineId;
    let mut check: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    min = max;
    i = 0_i32;
    while i < (*sector).linecount {
        line = (*sector).lines[i as usize];
        check = getNextSector(state, line, sector);
        if !check.is_null() && ((*check).lightlevel as i32) < min {
            min = (*check).lightlevel as i32;
        }
        i += 1;
    }
    min
}
pub unsafe fn P_CrossSpecialLine(
    state: &mut GameState,
    mut linenum: i32,
    mut side: i32,
    mut thing: *mut mobj_t,
) {
    let line: LineId = LineId(linenum as u32);
    let mut ok: i32 = 0;
    let special = state.p_setup.line(line).special;
    if (*thing).player.is_none() {
        match (*thing).type_0 as u32 {
            33 | 34 | 35 | 31 | 32 | 16 => return,
            _ => {}
        }
        ok = 0_i32;
        match special as i32 {
            39 | 97 | 125 | 126 | 4 | 10 | 88 => {
                ok = 1_i32;
            }
            _ => {}
        }
        if ok == 0 {
            return;
        }
    }
    match special as i32 {
        2 => {
            EV_DoDoor(state, line, VldoorE::vld_open);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        3 => {
            EV_DoDoor(state, line, VldoorE::vld_close);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        4 => {
            EV_DoDoor(state, line, VldoorE::vld_normal);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        5 => {
            EV_DoFloor(state, line, FloorE::raiseFloor);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        6 => {
            EV_DoCeiling(state, line, CeilingE::fastCrushAndRaise);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        8 => {
            EV_BuildStairs(state, line, StairE::build8);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        10 => {
            EV_DoPlat(state, line, PlattypeE::downWaitUpStay, 0_i32);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        12 => {
            EV_LightTurnOn(state, line, 0_i32);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        13 => {
            EV_LightTurnOn(state, line, 255_i32);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        16 => {
            EV_DoDoor(state, line, VldoorE::vld_close30ThenOpen);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        17 => {
            EV_StartLightStrobing(state, line);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        19 => {
            EV_DoFloor(state, line, FloorE::lowerFloor);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        22 => {
            EV_DoPlat(state, line, PlattypeE::raiseToNearestAndChange, 0_i32);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        25 => {
            EV_DoCeiling(state, line, CeilingE::crushAndRaise);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        30 => {
            EV_DoFloor(state, line, FloorE::raiseToTexture);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        35 => {
            EV_LightTurnOn(state, line, 35_i32);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        36 => {
            EV_DoFloor(state, line, FloorE::turboLower);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        37 => {
            EV_DoFloor(state, line, FloorE::lowerAndChange);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        38 => {
            EV_DoFloor(state, line, FloorE::lowerFloorToLowest);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        39 => {
            EV_Teleport(state, line, side, thing);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        40 => {
            EV_DoCeiling(state, line, CeilingE::raiseToHighest);
            EV_DoFloor(state, line, FloorE::lowerFloorToLowest);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        44 => {
            EV_DoCeiling(state, line, CeilingE::lowerAndCrush);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        52 => {
            G_ExitLevel(state);
        }
        53 => {
            EV_DoPlat(state, line, PlattypeE::perpetualRaise, 0_i32);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        54 => {
            EV_StopPlat(state, state.p_setup.line(line).tag as i32);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        56 => {
            EV_DoFloor(state, line, FloorE::raiseFloorCrush);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        57 => {
            EV_CeilingCrushStop(state, state.p_setup.line(line).tag as i32);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        58 => {
            EV_DoFloor(state, line, FloorE::raiseFloor24);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        59 => {
            EV_DoFloor(state, line, FloorE::raiseFloor24AndChange);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        104 => {
            EV_TurnTagLightsOff(state, line);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        108 => {
            EV_DoDoor(state, line, VldoorE::vld_blazeRaise);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        109 => {
            EV_DoDoor(state, line, VldoorE::vld_blazeOpen);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        100 => {
            EV_BuildStairs(state, line, StairE::turbo16);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        110 => {
            EV_DoDoor(state, line, VldoorE::vld_blazeClose);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        119 => {
            EV_DoFloor(state, line, FloorE::raiseFloorToNearest);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        121 => {
            EV_DoPlat(state, line, PlattypeE::blazeDWUS, 0_i32);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        124 => {
            G_SecretExitLevel(state);
        }
        125 => {
            if (*thing).player.is_none() {
                EV_Teleport(state, line, side, thing);
                state.p_setup.line_mut(line).special = 0_i16;
            }
        }
        130 => {
            EV_DoFloor(state, line, FloorE::raiseFloorTurbo);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        141 => {
            EV_DoCeiling(state, line, CeilingE::silentCrushAndRaise);
            state.p_setup.line_mut(line).special = 0_i16;
        }
        72 => {
            EV_DoCeiling(state, line, CeilingE::lowerAndCrush);
        }
        73 => {
            EV_DoCeiling(state, line, CeilingE::crushAndRaise);
        }
        74 => {
            EV_CeilingCrushStop(state, state.p_setup.line(line).tag as i32);
        }
        75 => {
            EV_DoDoor(state, line, VldoorE::vld_close);
        }
        76 => {
            EV_DoDoor(state, line, VldoorE::vld_close30ThenOpen);
        }
        77 => {
            EV_DoCeiling(state, line, CeilingE::fastCrushAndRaise);
        }
        79 => {
            EV_LightTurnOn(state, line, 35_i32);
        }
        80 => {
            EV_LightTurnOn(state, line, 0_i32);
        }
        81 => {
            EV_LightTurnOn(state, line, 255_i32);
        }
        82 => {
            EV_DoFloor(state, line, FloorE::lowerFloorToLowest);
        }
        83 => {
            EV_DoFloor(state, line, FloorE::lowerFloor);
        }
        84 => {
            EV_DoFloor(state, line, FloorE::lowerAndChange);
        }
        86 => {
            EV_DoDoor(state, line, VldoorE::vld_open);
        }
        87 => {
            EV_DoPlat(state, line, PlattypeE::perpetualRaise, 0_i32);
        }
        88 => {
            EV_DoPlat(state, line, PlattypeE::downWaitUpStay, 0_i32);
        }
        89 => {
            EV_StopPlat(state, state.p_setup.line(line).tag as i32);
        }
        90 => {
            EV_DoDoor(state, line, VldoorE::vld_normal);
        }
        91 => {
            EV_DoFloor(state, line, FloorE::raiseFloor);
        }
        92 => {
            EV_DoFloor(state, line, FloorE::raiseFloor24);
        }
        93 => {
            EV_DoFloor(state, line, FloorE::raiseFloor24AndChange);
        }
        94 => {
            EV_DoFloor(state, line, FloorE::raiseFloorCrush);
        }
        95 => {
            EV_DoPlat(state, line, PlattypeE::raiseToNearestAndChange, 0_i32);
        }
        96 => {
            EV_DoFloor(state, line, FloorE::raiseToTexture);
        }
        97 => {
            EV_Teleport(state, line, side, thing);
        }
        98 => {
            EV_DoFloor(state, line, FloorE::turboLower);
        }
        105 => {
            EV_DoDoor(state, line, VldoorE::vld_blazeRaise);
        }
        106 => {
            EV_DoDoor(state, line, VldoorE::vld_blazeOpen);
        }
        107 => {
            EV_DoDoor(state, line, VldoorE::vld_blazeClose);
        }
        120 => {
            EV_DoPlat(state, line, PlattypeE::blazeDWUS, 0_i32);
        }
        126 => {
            if (*thing).player.is_none() {
                EV_Teleport(state, line, side, thing);
            }
        }
        128 => {
            EV_DoFloor(state, line, FloorE::raiseFloorToNearest);
        }
        129 => {
            EV_DoFloor(state, line, FloorE::raiseFloorTurbo);
        }
        _ => {}
    };
}
pub unsafe fn P_ShootSpecialLine(state: &mut GameState, mut thing: *mut mobj_t, mut line: LineId) {
    let mut ok: i32 = 0;
    let special = state.p_setup.line(line).special;
    if (*thing).player.is_none() {
        ok = 0_i32;
        match special as i32 {
            46 => {
                ok = 1_i32;
            }
            _ => {}
        }
        if ok == 0 {
            return;
        }
    }
    match special as i32 {
        24 => {
            EV_DoFloor(state, line, FloorE::raiseFloor);
            P_ChangeSwitchTexture(state, line, 0_i32);
        }
        46 => {
            EV_DoDoor(state, line, VldoorE::vld_open);
            P_ChangeSwitchTexture(state, line, 1_i32);
        }
        47 => {
            EV_DoPlat(state, line, PlattypeE::raiseToNearestAndChange, 0_i32);
            P_ChangeSwitchTexture(state, line, 0_i32);
        }
        _ => {}
    };
}
pub unsafe fn P_PlayerInSpecialSector(state: &mut GameState, mut player: *mut player_t) {
    let mut sector: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    sector = state
        .p_setup
        .sector_mut(state.p_setup.subsectors[(*player_mo).subsector.0 as usize].sector);
    if (*player_mo).z != (*sector).floorheight {
        return;
    }
    match (*sector).special as i32 {
        5 => {
            if (*player).powers[PowerType::pw_ironfeet as usize] == 0
                && state.p_tick.leveltime & 0x1f_i32 == 0
            {
                P_DamageMobj(
                    state,
                    player_mo,
                    ::core::ptr::null_mut::<mobj_t>(),
                    ::core::ptr::null_mut::<mobj_t>(),
                    10_i32,
                );
            }
        }
        7 => {
            if (*player).powers[PowerType::pw_ironfeet as usize] == 0
                && state.p_tick.leveltime & 0x1f_i32 == 0
            {
                P_DamageMobj(
                    state,
                    player_mo,
                    ::core::ptr::null_mut::<mobj_t>(),
                    ::core::ptr::null_mut::<mobj_t>(),
                    5_i32,
                );
            }
        }
        16 | 4 => {
            if ((*player).powers[PowerType::pw_ironfeet as usize] == 0
                || P_Random(&mut state.m_random) < 5_i32)
                && state.p_tick.leveltime & 0x1f_i32 == 0
            {
                P_DamageMobj(
                    state,
                    player_mo,
                    ::core::ptr::null_mut::<mobj_t>(),
                    ::core::ptr::null_mut::<mobj_t>(),
                    20_i32,
                );
            }
        }
        9 => {
            (*player).secretcount += 1;
            (*sector).special = 0_i16;
        }
        11 => {
            (*player).cheats &= !CF_GODMODE;
            if state.p_tick.leveltime & 0x1f_i32 == 0 {
                P_DamageMobj(
                    state,
                    player_mo,
                    ::core::ptr::null_mut::<mobj_t>(),
                    ::core::ptr::null_mut::<mobj_t>(),
                    20_i32,
                );
            }
            if (*player).health <= 10_i32 {
                G_ExitLevel(state);
            }
        }
        _ => {
            I_Error(&format!(
                "P_PlayerInSpecialSector: unknown special {}",
                (*sector).special as i32,
            ));
        }
    };
}
pub unsafe fn P_UpdateSpecials(state: &mut GameState) {
    let mut pic: i32 = 0;
    let mut i: i32 = 0;
    let mut line: LineId;
    if state.p_spec.levelTimer {
        state.p_spec.levelTimeCount -= 1;
        if state.p_spec.levelTimeCount == 0 {
            G_ExitLevel(state);
        }
    }
    let mut anim_idx: usize = 0;
    while anim_idx < state.p_spec.lastanim {
        let anim = &state.p_spec.anims[anim_idx];
        i = anim.basepic;
        while i < anim.basepic + anim.numpics {
            pic = anim.basepic + (state.p_tick.leveltime / anim.speed + i) % anim.numpics;
            if anim.istexture {
                state.r_data.texturetranslation[i as usize] = pic;
            } else {
                state.r_data.flattranslation[i as usize] = pic;
            }
            i += 1;
        }
        anim_idx += 1;
    }
    i = 0_i32;
    while i < state.p_spec.numlinespecials as i32 {
        line = state.p_spec.linespeciallist[i as usize];
        let linev = state.p_setup.line(line);
        match linev.special as i32 {
            48 => {
                let fresh0 = &mut state.p_setup.sides[linev.sidenum[0] as usize].textureoffset;
                *fresh0 += FRACUNIT;
            }
            _ => {}
        }
        i += 1;
    }
    i = 0_i32;
    while i < MAXBUTTONS {
        if state.p_switch.buttonlist[i as usize].btimer != 0 {
            state.p_switch.buttonlist[i as usize].btimer -= 1;
            if state.p_switch.buttonlist[i as usize].btimer == 0 {
                let button_line_id = state.p_switch.buttonlist[i as usize].line;
                match state.p_switch.buttonlist[i as usize].where_0 {
                    BWhere::top => {
                        state.p_setup.sides
                            [state.p_setup.lines[button_line_id.0 as usize].sidenum[0] as usize]
                            .toptexture = state.p_switch.buttonlist[i as usize].btexture as i16;
                    }
                    BWhere::middle => {
                        state.p_setup.sides
                            [state.p_setup.lines[button_line_id.0 as usize].sidenum[0] as usize]
                            .midtexture = state.p_switch.buttonlist[i as usize].btexture as i16;
                    }
                    BWhere::bottom => {
                        state.p_setup.sides
                            [state.p_setup.lines[button_line_id.0 as usize].sidenum[0] as usize]
                            .bottomtexture = state.p_switch.buttonlist[i as usize].btexture as i16;
                    }
                }
                S_StartSound(
                    state,
                    SoundOrigin::Sector(state.p_switch.buttonlist[i as usize].soundorg),
                    sfx_swtchn as i32,
                );
                memset(
                    (&raw mut state.p_switch.buttonlist as *mut button_t).offset(i as isize)
                        as *mut button_t as *mut ::core::ffi::c_void,
                    0_i32,
                    ::core::mem::size_of::<button_t>() as size_t,
                );
            }
        }
        i += 1;
    }
}
pub const DONUT_FLOORHEIGHT_DEFAULT: i32 = 0;
pub const DONUT_FLOORPIC_DEFAULT: i32 = 0x16;
fn DonutOverrun(
    state: &mut GameState,
    mut s3_floorheight: *mut fixed_t,
    mut s3_floorpic: *mut i16,
) {
    let state = state;
    if state.p_spec.donut_overrun_first != 0 {
        let mut p: i32 = 0;
        state.p_spec.donut_overrun_first = 0_i32;
        state.p_spec.donut_overrun_tmp_s3_floorheight = DONUT_FLOORHEIGHT_DEFAULT;
        state.p_spec.donut_overrun_tmp_s3_floorpic = DONUT_FLOORPIC_DEFAULT;
        p = M_CheckParmWithArgs(state, "-donut", 2_i32);
        if p > 0_i32 {
            M_StrToInt(
                state.m_argv.myargv[(p + 1_i32) as usize].to_str().unwrap(),
                &mut state.p_spec.donut_overrun_tmp_s3_floorheight,
            );
            M_StrToInt(
                state.m_argv.myargv[(p + 2_i32) as usize].to_str().unwrap(),
                &mut state.p_spec.donut_overrun_tmp_s3_floorpic,
            );
            if state.p_spec.donut_overrun_tmp_s3_floorpic >= state.r_data.numflats {
                eprintln!(
                    "DonutOverrun: The second parameter for \"-donut\" switch should be greater than 0 and less than number of flats ({}). Using default value ({}) instead. ",
                    state.r_data.numflats,
                    DONUT_FLOORPIC_DEFAULT,
                );
                state.p_spec.donut_overrun_tmp_s3_floorpic = DONUT_FLOORPIC_DEFAULT;
            }
        }
    }
    unsafe {
        *s3_floorheight = state.p_spec.donut_overrun_tmp_s3_floorheight;
        *s3_floorpic = state.p_spec.donut_overrun_tmp_s3_floorpic as i16;
    }
}
pub unsafe fn EV_DoDonut(state: &mut GameState, mut line: LineId) -> i32 {
    let mut s1: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut s2: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut s3: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut secnum: i32 = 0;
    let mut rtn: i32 = 0;
    let mut i: i32 = 0;
    let mut floor: *mut floormove_t = ::core::ptr::null_mut::<floormove_t>();
    let mut s3_floorheight: fixed_t = 0;
    let mut s3_floorpic: i16 = 0;
    secnum = -1_i32;
    rtn = 0_i32;
    loop {
        secnum = P_FindSectorFromLineTag(state, line, secnum);
        if secnum < 0_i32 {
            break;
        }
        s1 = state.p_setup.sector_mut(SectorId(secnum as u32));
        if (*s1).specialdata.is_some() {
            continue;
        }
        rtn = 1_i32;
        s2 = getNextSector(state, (*s1).lines[0], s1);
        if s2.is_null() {
            eprintln!(
                "EV_DoDonut: linedef had no second sidedef! Unexpected behavior may occur in Vanilla Doom. "
            );
            break;
        } else {
            let s2_id = SectorId(s2.offset_from(state.p_setup.sectors.as_mut_ptr()) as i64 as u32);
            i = 0_i32;
            while i < (*s2).linecount {
                let s2_line_id = (*s2).lines[i as usize];
                s3 = match state.p_setup.line(s2_line_id).backsector {
                    Some(id) => state.p_setup.sector_mut(id),
                    None => ::core::ptr::null_mut::<sector_t>(),
                };
                if s3 == s1 {
                    i += 1;
                } else {
                    if s3.is_null() {
                        eprintln!(
                            "EV_DoDonut: WARNING: emulating buffer overrun due to NULL back sector. Unexpected behavior may occur in Vanilla Doom."
                        );
                        DonutOverrun(state, &raw mut s3_floorheight, &raw mut s3_floorpic);
                    } else {
                        s3_floorheight = (*s3).floorheight;
                        s3_floorpic = (*s3).floorpic;
                    }
                    let (floor_arena_id, floor_ptr) = state.p_spec.spawn_floor(floormove_t::default());
                    floor = floor_ptr;
                    let floor_id =
                        P_AddThinker(state, ThinkerPayload::Floor(floor_arena_id), ThinkerKind::Floor);
                    (*s2).specialdata = Some(SectorSpecial::Floor(floor_id));
                    (*floor).thinker.function = ThinkerFn::Floor(T_MoveFloor);
                    (*floor).type_0 = FloorE::donutRaise;
                    (*floor).crush = false;
                    (*floor).direction = 1_i32;
                    (*floor).sector = s2_id;
                    (*floor).speed = (FLOORSPEED / 2_i32) as fixed_t;
                    (*floor).texture = s3_floorpic;
                    (*floor).newspecial = 0_i32;
                    (*floor).floordestheight = s3_floorheight;
                    let (floor_arena_id, floor_ptr) = state.p_spec.spawn_floor(floormove_t::default());
                    floor = floor_ptr;
                    let floor_id =
                        P_AddThinker(state, ThinkerPayload::Floor(floor_arena_id), ThinkerKind::Floor);
                    (*s1).specialdata = Some(SectorSpecial::Floor(floor_id));
                    (*floor).thinker.function = ThinkerFn::Floor(T_MoveFloor);
                    (*floor).type_0 = FloorE::lowerFloor;
                    (*floor).crush = false;
                    (*floor).direction = -1_i32;
                    (*floor).sector = SectorId(secnum as u32);
                    (*floor).speed = (FLOORSPEED / 2_i32) as fixed_t;
                    (*floor).floordestheight = s3_floorheight;
                    break;
                }
            }
        }
    }
    rtn
}
pub unsafe fn P_SpawnSpecials(state: &mut GameState) {
    let mut sector: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut i: i32 = 0;
    if state.g_game.timelimit > 0_i32 && state.g_game.deathmatch != 0 {
        state.p_spec.levelTimer = true;
        state.p_spec.levelTimeCount = state.g_game.timelimit * 60_i32 * TICRATE;
    } else {
        state.p_spec.levelTimer = false;
    }
    i = 0_i32;
    while i < state.p_setup.numsectors {
        let secid = SectorId(i as u32);
        sector = state.p_setup.sector_mut(secid);
        if (*sector).special != 0 {
            match (*sector).special as i32 {
                1 => {
                    P_SpawnLightFlash(state, secid);
                }
                2 => {
                    P_SpawnStrobeFlash(state, secid, FASTDARK, 0_i32);
                }
                3 => {
                    P_SpawnStrobeFlash(state, secid, SLOWDARK, 0_i32);
                }
                4 => {
                    P_SpawnStrobeFlash(state, secid, FASTDARK, 0_i32);
                    (*sector).special = 4_i16;
                }
                8 => {
                    P_SpawnGlowingLight(state, secid);
                }
                9 => {
                    state.g_game.totalsecret += 1;
                }
                10 => {
                    P_SpawnDoorCloseIn30(state, secid);
                }
                12 => {
                    P_SpawnStrobeFlash(state, secid, SLOWDARK, 1_i32);
                }
                13 => {
                    P_SpawnStrobeFlash(state, secid, FASTDARK, 1_i32);
                }
                14 => {
                    P_SpawnDoorRaiseIn5Mins(state, secid);
                }
                17 => {
                    P_SpawnFireFlicker(state, secid);
                }
                _ => {}
            }
        }
        i += 1;
    }
    state.p_spec.numlinespecials = 0_i16;
    i = 0_i32;
    while i < state.p_setup.numlines {
        match state.p_setup.lines[i as usize].special as i32 {
            48 => {
                if state.p_spec.numlinespecials as i32 >= MAXLINEANIMS {
                    I_Error("Too many scrolling wall linedefs! (Vanilla limit is 64)");
                }
                state.p_spec.linespeciallist[state.p_spec.numlinespecials as usize] =
                    LineId(i as u32);
                state.p_spec.numlinespecials += 1;
            }
            _ => {}
        }
        i += 1;
    }
    i = 0_i32;
    while i < MAXCEILINGS {
        state.p_ceilng.activeceilings[i as usize] = None;
        i += 1;
    }
    i = 0_i32;
    while i < MAXPLATS {
        state.p_plats.activeplats[i as usize] = None;
        i += 1;
    }
    i = 0_i32;
    while i < MAXBUTTONS {
        memset(
            (&raw mut state.p_switch.buttonlist as *mut button_t).offset(i as isize)
                as *mut button_t as *mut ::core::ffi::c_void,
            0_i32,
            ::core::mem::size_of::<button_t>() as size_t,
        );
        i += 1;
    }
}
pub const ML_SECRET: i32 = 32;
pub const ML_MAPPED: i32 = 256;
