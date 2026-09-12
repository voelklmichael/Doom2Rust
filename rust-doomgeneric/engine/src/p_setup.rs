use crate::src::d_mode::commercial;
use crate::src::doomdef::MAXPLAYERS;
use crate::src::fixed_cstr::FixedCStr;
use crate::src::g_game::G_DeathMatchSpawnPlayer;
use crate::src::game_state::GameState;
use crate::src::i_system::I_GetMemoryValue;
use crate::src::m_argv::M_CheckParm;
use crate::src::m_bbox::M_AddToBox;
use crate::src::m_bbox::M_ClearBox;
use crate::src::m_bbox::{BOXBOTTOM, BOXLEFT, BOXRIGHT, BOXTOP};
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FixedDiv;
use crate::src::m_fixed::FRACBITS;
use crate::src::m_fixed::FRACUNIT;
use crate::src::p_maputl::MAPBLOCKSHIFT;
use crate::src::p_mobj::mobj_t;
use crate::src::p_mobj::P_SpawnMapThing;
use crate::src::p_mobj::{
    degenmobj_t, line_s, line_t, mapthing_t, sector_t, subsector_s, subsector_t, thinker_s,
    vertex_t, MobjId, ThinkerFn, ST_HORIZONTAL, ST_NEGATIVE, ST_POSITIVE, ST_VERTICAL,
};
use crate::src::p_spec::P_InitPicAnims;
use crate::src::p_spec::P_SpawnSpecials;
use crate::src::p_spec::ML_TWOSIDED;
use crate::src::p_switch::P_InitSwitchList;
use crate::src::p_tick::P_InitThinkers;
use crate::src::r_data::R_FlatNumForName;
use crate::src::r_data::R_PrecacheLevel;
use crate::src::r_data::R_TextureNumForName;
use crate::src::r_defs::{node_t, seg_t, side_t};
use crate::src::r_things::R_InitSprites;
use crate::src::s_sound::S_Start;
use crate::src::stdint_types::byte;
use crate::src::stdint_types::size_t;
use crate::src::tables::angle_t;
use crate::src::w_wad::W_CacheLumpNum;
use crate::src::w_wad::W_LumpLength;
use crate::src::w_wad::W_ReadLump;
use crate::src::w_wad::W_ReleaseLumpNum;
use crate::src::w_wad::W_GetNumForName;
use crate::src::z_zone::Z_FreeTags;
use crate::src::z_zone::Z_Malloc;
use crate::src::z_zone::{PU_LEVEL, PU_PURGELEVEL, PU_STATIC};
use crate::src::mem_compat::memset;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SectorId(pub u32);
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SideId(pub u32);
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SubsectorId(pub u32);
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct VertexId(pub u32);
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LineId(pub u32);
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SegId(pub u32);

pub const ZERO_LINE: line_s = line_s {
    v1: VertexId(0),
    v2: VertexId(0),
    dx: 0,
    dy: 0,
    flags: 0,
    special: 0,
    tag: 0,
    sidenum: [0; 2],
    bbox: [0; 4],
    slopetype: ST_HORIZONTAL,
    frontsector: None,
    backsector: None,
    validcount: 0,
};

pub const ZERO_SECTOR: sector_t = sector_t {
    floorheight: 0,
    ceilingheight: 0,
    floorpic: 0,
    ceilingpic: 0,
    lightlevel: 0,
    special: 0,
    tag: 0,
    soundtraversed: 0,
    soundtarget: None,
    blockbox: [0; 4],
    soundorg: degenmobj_t {
        thinker: thinker_s {
            function: ThinkerFn::Paused,
        },
        x: 0,
        y: 0,
        z: 0,
    },
    validcount: 0,
    thinglist: None,
    specialdata: None,
    linecount: 0,
    lines: ::core::ptr::null::<*mut line_s>() as *mut *mut line_s,
};

pub struct PSetupState {
    pub numvertexes: i32,
    pub vertexes: Vec<vertex_t>,
    pub numsegs: i32,
    pub segs: Vec<seg_t>,
    pub numsectors: i32,
    pub sectors: Vec<sector_t>,
    pub numsubsectors: i32,
    pub subsectors: Vec<subsector_t>,
    pub numnodes: i32,
    pub nodes: *mut node_t,
    pub numlines: i32,
    pub lines: Vec<line_t>,
    pub numsides: i32,
    pub sides: Vec<side_t>,
    pub totallines: i32,
    pub bmapwidth: i32,
    pub bmapheight: i32,
    pub blockmap: *mut i16,
    pub blockmaplump: Vec<i16>,
    pub bmaporgx: fixed_t,
    pub bmaporgy: fixed_t,
    pub blocklinks: Vec<Option<MobjId>>,
    pub rejectmatrix: *mut byte,
    pub deathmatchstarts: [mapthing_t; 10],
    pub deathmatch_p: *mut mapthing_t,
    pub playerstarts: [mapthing_t; 4],
    pub null_sector_id: Option<SectorId>,
}

impl PSetupState {
    pub const fn new() -> Self {
        PSetupState {
            numvertexes: 0,
            vertexes: Vec::new(),
            numsegs: 0,
            segs: Vec::new(),
            numsectors: 0,
            sectors: Vec::new(),
            numsubsectors: 0,
            subsectors: Vec::new(),
            numnodes: 0,
            nodes: ::core::ptr::null::<node_t>() as *mut node_t,
            numlines: 0,
            lines: Vec::new(),
            numsides: 0,
            sides: Vec::new(),
            totallines: 0,
            bmapwidth: 0,
            bmapheight: 0,
            blockmap: ::core::ptr::null::<i16>() as *mut i16,
            blockmaplump: Vec::new(),
            bmaporgx: 0,
            bmaporgy: 0,
            blocklinks: Vec::new(),
            rejectmatrix: ::core::ptr::null::<byte>() as *mut byte,
            deathmatchstarts: [mapthing_t {
                x: 0,
                y: 0,
                angle: 0,
                type_0: 0,
                options: 0,
            }; 10],
            deathmatch_p: ::core::ptr::null::<mapthing_t>() as *mut mapthing_t,
            playerstarts: [mapthing_t {
                x: 0,
                y: 0,
                angle: 0,
                type_0: 0,
                options: 0,
            }; 4],
            null_sector_id: None,
        }
    }

    pub fn sector_mut(&mut self, id: SectorId) -> *mut sector_t {
        &mut self.sectors[id.0 as usize] as *mut sector_t
    }
    pub fn side_mut(&mut self, id: SideId) -> *mut side_t {
        &mut self.sides[id.0 as usize] as *mut side_t
    }
    pub fn subsector_mut(&mut self, id: SubsectorId) -> *mut subsector_t {
        &mut self.subsectors[id.0 as usize] as *mut subsector_t
    }
    pub fn vertex_mut(&mut self, id: VertexId) -> *mut vertex_t {
        &mut self.vertexes[id.0 as usize] as *mut vertex_t
    }
    pub fn line_mut(&mut self, id: LineId) -> *mut line_t {
        &mut self.lines[id.0 as usize] as *mut line_t
    }
    pub fn seg_mut(&mut self, id: SegId) -> *mut seg_t {
        &mut self.segs[id.0 as usize] as *mut seg_t
    }
}

pub type C2RustUnnamed_1 = u32;
pub const ML_BLOCKMAP: C2RustUnnamed_1 = 10;
pub const ML_REJECT: C2RustUnnamed_1 = 9;
pub const ML_SECTORS: C2RustUnnamed_1 = 8;
pub const ML_NODES: C2RustUnnamed_1 = 7;
pub const ML_SSECTORS: C2RustUnnamed_1 = 6;
pub const ML_SEGS: C2RustUnnamed_1 = 5;
pub const ML_VERTEXES: C2RustUnnamed_1 = 4;
pub const ML_SIDEDEFS: C2RustUnnamed_1 = 3;
pub const ML_LINEDEFS: C2RustUnnamed_1 = 2;
pub const ML_THINGS: C2RustUnnamed_1 = 1;
pub const ML_LABEL: C2RustUnnamed_1 = 0;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct mapvertex_t {
    pub x: i16,
    pub y: i16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct mapsidedef_t {
    pub textureoffset: i16,
    pub rowoffset: i16,
    pub toptexture: FixedCStr<8>,
    pub bottomtexture: FixedCStr<8>,
    pub midtexture: FixedCStr<8>,
    pub sector: i16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct maplinedef_t {
    pub v1: i16,
    pub v2: i16,
    pub flags: i16,
    pub special: i16,
    pub tag: i16,
    pub sidenum: [i16; 2],
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct mapsector_t {
    pub floorheight: i16,
    pub ceilingheight: i16,
    pub floorpic: FixedCStr<8>,
    pub ceilingpic: FixedCStr<8>,
    pub lightlevel: i16,
    pub special: i16,
    pub tag: i16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct mapsubsector_t {
    pub numsegs: i16,
    pub firstseg: i16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct mapseg_t {
    pub v1: i16,
    pub v2: i16,
    pub angle: i16,
    pub linedef: i16,
    pub side: i16,
    pub offset: i16,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct mapnode_t {
    pub x: i16,
    pub y: i16,
    pub dx: i16,
    pub dy: i16,
    pub bbox: [[i16; 4]; 2],
    pub children: [u16; 2],
}
pub unsafe fn P_LoadVertexes(state: &mut GameState, mut lump: i32) {
    let mut data: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut i: i32 = 0;
    let mut ml: *mut mapvertex_t = ::core::ptr::null_mut::<mapvertex_t>();
    let numvertexes = (W_LumpLength(&mut state.w_wad, lump as u32) as usize)
        .wrapping_div(::core::mem::size_of::<mapvertex_t>() as usize)
        as i32;
    state.p_setup.numvertexes = numvertexes;
    state.p_setup.vertexes = Vec::with_capacity(numvertexes as usize);
    data = W_CacheLumpNum(state, lump, PU_STATIC as i32) as *mut byte;
    ml = data as *mut mapvertex_t;
    i = 0 as i32;
    while i < numvertexes {
        state.p_setup.vertexes.push(vertex_t {
            x: (((*ml).x as i32) << FRACBITS) as fixed_t,
            y: (((*ml).y as i32) << FRACBITS) as fixed_t,
        });
        i += 1;
        ml = ml.offset(1);
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub unsafe fn GetSectorAtNullAddress(state: &mut GameState) -> SectorId {
    if state.p_setup.null_sector_id.is_none() {
        let mut sentinel = ZERO_SECTOR;
        I_GetMemoryValue(
            state,
            0 as u32,
            &raw mut sentinel.floorheight as *mut ::core::ffi::c_void,
            4 as i32,
        );
        I_GetMemoryValue(
            state,
            4 as u32,
            &raw mut sentinel.ceilingheight as *mut ::core::ffi::c_void,
            4 as i32,
        );
        let id = SectorId(state.p_setup.sectors.len() as u32);
        state.p_setup.sectors.push(sentinel);
        state.p_setup.null_sector_id = Some(id);
    }
    state.p_setup.null_sector_id.unwrap()
}
pub unsafe fn P_LoadSegs(state: &mut GameState, mut lump: i32) {
    let mut data: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut i: i32 = 0;
    let mut ml: *mut mapseg_t = ::core::ptr::null_mut::<mapseg_t>();
    let mut ldef: *mut line_t = ::core::ptr::null_mut::<line_t>();
    let mut linedef: i32 = 0;
    let mut side: i32 = 0;
    let mut sidenum: i32 = 0;
    let numsegs = (W_LumpLength(&mut state.w_wad, lump as u32) as usize)
        .wrapping_div(::core::mem::size_of::<mapseg_t>() as usize)
        as i32;
    state.p_setup.numsegs = numsegs;
    state.p_setup.segs = Vec::with_capacity(numsegs as usize);
    data = W_CacheLumpNum(state, lump, PU_STATIC as i32) as *mut byte;
    ml = data as *mut mapseg_t;
    i = 0 as i32;
    while i < numsegs {
        let v1 = VertexId((*ml).v1 as u32);
        let v2 = VertexId((*ml).v2 as u32);
        let seg_angle = (((*ml).angle as i32) << 16 as i32) as angle_t;
        let seg_offset = (((*ml).offset as i32) << 16 as i32) as fixed_t;
        linedef = (*ml).linedef as i32;
        let seg_linedef = LineId(linedef as u32);
        ldef = state.p_setup.line_mut(seg_linedef);
        side = (*ml).side as i32;
        let seg_sidenum = *(&raw mut (*ldef).sidenum as *mut i16).offset(side as isize) as u32;
        let frontsector = Some(state.p_setup.sides[seg_sidenum as usize].sector);
        let backsector = if (*ldef).flags as i32 & ML_TWOSIDED != 0 {
            sidenum = (*ldef).sidenum[(side ^ 1 as i32) as usize] as i32;
            if sidenum < 0 as i32 || sidenum >= state.p_setup.numsides {
                Some(GetSectorAtNullAddress(state))
            } else {
                Some(state.p_setup.sides[sidenum as usize].sector)
            }
        } else {
            None
        };
        state.p_setup.segs.push(seg_t {
            v1,
            v2,
            offset: seg_offset,
            angle: seg_angle,
            sidedef: SideId(seg_sidenum),
            linedef: seg_linedef,
            frontsector,
            backsector,
        });
        i += 1;
        ml = ml.offset(1);
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub unsafe fn P_LoadSubsectors(state: &mut GameState, mut lump: i32) {
    let mut data: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut i: i32 = 0;
    let mut ms: *mut mapsubsector_t = ::core::ptr::null_mut::<mapsubsector_t>();
    let numsubsectors = (W_LumpLength(&mut state.w_wad, lump as u32) as usize)
        .wrapping_div(::core::mem::size_of::<mapsubsector_t>() as usize)
        as i32;
    state.p_setup.numsubsectors = numsubsectors;
    state.p_setup.subsectors = Vec::with_capacity(numsubsectors as usize);
    data = W_CacheLumpNum(state, lump, PU_STATIC as i32) as *mut byte;
    ms = data as *mut mapsubsector_t;
    i = 0 as i32;
    while i < numsubsectors {
        state.p_setup.subsectors.push(subsector_s {
            sector: SectorId(0),
            numlines: (*ms).numsegs,
            firstline: (*ms).firstseg,
        });
        i += 1;
        ms = ms.offset(1);
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub unsafe fn P_LoadSectors(state: &mut GameState, mut lump: i32) {
    let mut data: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut i: i32 = 0;
    let mut ms: *mut mapsector_t = ::core::ptr::null_mut::<mapsector_t>();
    let numsectors = (W_LumpLength(&mut state.w_wad, lump as u32) as usize)
        .wrapping_div(::core::mem::size_of::<mapsector_t>() as usize) as i32;
    state.p_setup.numsectors = numsectors;
    state.p_setup.sectors = vec![ZERO_SECTOR; numsectors as usize];
    data = W_CacheLumpNum(state, lump, PU_STATIC as i32) as *mut byte;
    ms = data as *mut mapsector_t;
    i = 0 as i32;
    while i < numsectors {
        let floorpic = R_FlatNumForName(state, &(*ms).floorpic.as_str()) as i16;
        let ceilingpic = R_FlatNumForName(state, &(*ms).ceilingpic.as_str()) as i16;
        let ss = &mut state.p_setup.sectors[i as usize];
        ss.floorheight = (((*ms).floorheight as i32) << FRACBITS) as fixed_t;
        ss.ceilingheight = (((*ms).ceilingheight as i32) << FRACBITS) as fixed_t;
        ss.floorpic = floorpic;
        ss.ceilingpic = ceilingpic;
        ss.lightlevel = (*ms).lightlevel;
        ss.special = (*ms).special;
        ss.tag = (*ms).tag;
        ss.thinglist = None;
        i += 1;
        ms = ms.offset(1);
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub unsafe fn P_LoadNodes(state: &mut GameState, mut lump: i32) {
    let mut data: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut k: i32 = 0;
    let mut mn: *mut mapnode_t = ::core::ptr::null_mut::<mapnode_t>();
    let mut no: *mut node_t = ::core::ptr::null_mut::<node_t>();
    state.p_setup.numnodes = (W_LumpLength(&mut state.w_wad, lump as u32) as usize)
        .wrapping_div(::core::mem::size_of::<mapnode_t>() as usize)
        as i32;
    state.p_setup.nodes = Z_Malloc(
        &mut state.z_zone,
        (state.p_setup.numnodes as usize).wrapping_mul(::core::mem::size_of::<node_t>() as usize)
            as i32,
        PU_LEVEL as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut node_t;
    data = W_CacheLumpNum(state, lump, PU_STATIC as i32) as *mut byte;
    mn = data as *mut mapnode_t;
    no = state.p_setup.nodes;
    i = 0 as i32;
    while i < state.p_setup.numnodes {
        (*no).x = (((*mn).x as i32) << FRACBITS) as fixed_t;
        (*no).y = (((*mn).y as i32) << FRACBITS) as fixed_t;
        (*no).dx = (((*mn).dx as i32) << FRACBITS) as fixed_t;
        (*no).dy = (((*mn).dy as i32) << FRACBITS) as fixed_t;
        j = 0 as i32;
        while j < 2 as i32 {
            (*no).children[j as usize] = (*mn).children[j as usize] as i16 as u16;
            k = 0 as i32;
            while k < 4 as i32 {
                (*no).bbox[j as usize][k as usize] =
                    (((*mn).bbox[j as usize][k as usize] as i32) << FRACBITS) as fixed_t;
                k += 1;
            }
            j += 1;
        }
        i += 1;
        no = no.offset(1);
        mn = mn.offset(1);
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub unsafe fn P_LoadThings(state: &mut GameState, mut lump: i32) {
    let mut data: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut i: i32 = 0;
    let mut mt: *mut mapthing_t = ::core::ptr::null_mut::<mapthing_t>();
    let mut spawnthing: mapthing_t = mapthing_t {
        x: 0,
        y: 0,
        angle: 0,
        type_0: 0,
        options: 0,
    };
    let mut numthings: i32 = 0;
    let mut spawn: bool = false;
    data = W_CacheLumpNum(state, lump, PU_STATIC as i32) as *mut byte;
    numthings = (W_LumpLength(&mut state.w_wad, lump as u32) as usize)
        .wrapping_div(::core::mem::size_of::<mapthing_t>() as usize) as i32;
    mt = data as *mut mapthing_t;
    i = 0 as i32;
    while i < numthings {
        spawn = true;
        if state.doomstat.gamemode as u32 != commercial as i32 as u32 {
            let mut current_block_5: u64;
            match (*mt).type_0 as i32 {
                64 => {
                    current_block_5 = 10716006297776741838;
                }
                88 => {
                    current_block_5 = 10716006297776741838;
                }
                89 => {
                    current_block_5 = 4903439290872339201;
                }
                69 => {
                    current_block_5 = 4274804826831059371;
                }
                67 => {
                    current_block_5 = 12519425194970330903;
                }
                71 => {
                    current_block_5 = 14837270904287063365;
                }
                65 => {
                    current_block_5 = 10735630984003381802;
                }
                66 => {
                    current_block_5 = 14624214236247155710;
                }
                68 | 84 => {
                    current_block_5 = 16749256938512238719;
                }
                _ => {
                    current_block_5 = 2979737022853876585;
                }
            }
            match current_block_5 {
                10716006297776741838 => {
                    current_block_5 = 4903439290872339201;
                }
                _ => {}
            }
            match current_block_5 {
                4903439290872339201 => {
                    current_block_5 = 4274804826831059371;
                }
                _ => {}
            }
            match current_block_5 {
                4274804826831059371 => {
                    current_block_5 = 12519425194970330903;
                }
                _ => {}
            }
            match current_block_5 {
                12519425194970330903 => {
                    current_block_5 = 14837270904287063365;
                }
                _ => {}
            }
            match current_block_5 {
                14837270904287063365 => {
                    current_block_5 = 10735630984003381802;
                }
                _ => {}
            }
            match current_block_5 {
                10735630984003381802 => {
                    current_block_5 = 14624214236247155710;
                }
                _ => {}
            }
            match current_block_5 {
                14624214236247155710 => {
                    current_block_5 = 16749256938512238719;
                }
                _ => {}
            }
            match current_block_5 {
                16749256938512238719 => {
                    spawn = false;
                }
                _ => {}
            }
        }
        if !spawn {
            break;
        }
        spawnthing.x = (*mt).x;
        spawnthing.y = (*mt).y;
        spawnthing.angle = (*mt).angle;
        spawnthing.type_0 = (*mt).type_0;
        spawnthing.options = (*mt).options;
        P_SpawnMapThing(state, &raw mut spawnthing);
        i += 1;
        mt = mt.offset(1);
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub unsafe fn P_LoadLineDefs(state: &mut GameState, mut lump: i32) {
    let mut data: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut i: i32 = 0;
    let mut mld: *mut maplinedef_t = ::core::ptr::null_mut::<maplinedef_t>();
    let mut ld: *mut line_t = ::core::ptr::null_mut::<line_t>();
    let mut v1: *mut vertex_t = ::core::ptr::null_mut::<vertex_t>();
    let mut v2: *mut vertex_t = ::core::ptr::null_mut::<vertex_t>();
    state.p_setup.numlines = (W_LumpLength(&mut state.w_wad, lump as u32) as usize)
        .wrapping_div(::core::mem::size_of::<maplinedef_t>() as usize)
        as i32;
    state.p_setup.lines = vec![ZERO_LINE; state.p_setup.numlines as usize];
    data = W_CacheLumpNum(state, lump, PU_STATIC as i32) as *mut byte;
    mld = data as *mut maplinedef_t;
    i = 0 as i32;
    while i < state.p_setup.numlines {
        ld = &mut state.p_setup.lines[i as usize] as *mut line_t;
        (*ld).flags = (*mld).flags;
        (*ld).special = (*mld).special;
        (*ld).tag = (*mld).tag;
        (*ld).v1 = VertexId((*mld).v1 as u32);
        v1 = state.p_setup.vertex_mut((*ld).v1);
        (*ld).v2 = VertexId((*mld).v2 as u32);
        v2 = state.p_setup.vertex_mut((*ld).v2);
        (*ld).dx = (*v2).x - (*v1).x;
        (*ld).dy = (*v2).y - (*v1).y;
        if (*ld).dx == 0 {
            (*ld).slopetype = ST_VERTICAL;
        } else if (*ld).dy == 0 {
            (*ld).slopetype = ST_HORIZONTAL;
        } else if FixedDiv((*ld).dy, (*ld).dx) > 0 as i32 {
            (*ld).slopetype = ST_POSITIVE;
        } else {
            (*ld).slopetype = ST_NEGATIVE;
        }
        if (*v1).x < (*v2).x {
            (*ld).bbox[BOXLEFT as i32 as usize] = (*v1).x;
            (*ld).bbox[BOXRIGHT as i32 as usize] = (*v2).x;
        } else {
            (*ld).bbox[BOXLEFT as i32 as usize] = (*v2).x;
            (*ld).bbox[BOXRIGHT as i32 as usize] = (*v1).x;
        }
        if (*v1).y < (*v2).y {
            (*ld).bbox[BOXBOTTOM as i32 as usize] = (*v1).y;
            (*ld).bbox[BOXTOP as i32 as usize] = (*v2).y;
        } else {
            (*ld).bbox[BOXBOTTOM as i32 as usize] = (*v2).y;
            (*ld).bbox[BOXTOP as i32 as usize] = (*v1).y;
        }
        (*ld).sidenum[0 as i32 as usize] = (*mld).sidenum[0 as i32 as usize];
        (*ld).sidenum[1 as i32 as usize] = (*mld).sidenum[1 as i32 as usize];
        if (*ld).sidenum[0 as i32 as usize] as i32 != -(1 as i32) {
            let side_sector = state.p_setup.sides[(*ld).sidenum[0 as i32 as usize] as usize].sector;
            (*ld).frontsector = Some(side_sector);
        } else {
            (*ld).frontsector = None;
        }
        if (*ld).sidenum[1 as i32 as usize] as i32 != -(1 as i32) {
            let side_sector = state.p_setup.sides[(*ld).sidenum[1 as i32 as usize] as usize].sector;
            (*ld).backsector = Some(side_sector);
        } else {
            (*ld).backsector = None;
        }
        i += 1;
        mld = mld.offset(1);
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub unsafe fn P_LoadSideDefs(state: &mut GameState, mut lump: i32) {
    let mut data: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut i: i32 = 0;
    let mut msd: *mut mapsidedef_t = ::core::ptr::null_mut::<mapsidedef_t>();
    let numsides = (W_LumpLength(&mut state.w_wad, lump as u32) as usize)
        .wrapping_div(::core::mem::size_of::<mapsidedef_t>() as usize) as i32;
    state.p_setup.numsides = numsides;
    state.p_setup.sides = Vec::with_capacity(numsides as usize);
    data = W_CacheLumpNum(state, lump, PU_STATIC as i32) as *mut byte;
    msd = data as *mut mapsidedef_t;
    i = 0 as i32;
    while i < numsides {
        let sd = side_t {
            textureoffset: (((*msd).textureoffset as i32) << FRACBITS) as fixed_t,
            rowoffset: (((*msd).rowoffset as i32) << FRACBITS) as fixed_t,
            toptexture: R_TextureNumForName(&mut state.r_data, &(*msd).toptexture.as_str())
                as i16,
            bottomtexture: R_TextureNumForName(&mut state.r_data, &(*msd).bottomtexture.as_str())
                as i16,
            midtexture: R_TextureNumForName(&mut state.r_data, &(*msd).midtexture.as_str())
                as i16,
            sector: SectorId((*msd).sector as u32),
        };
        state.p_setup.sides.push(sd);
        i += 1;
        msd = msd.offset(1);
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub unsafe fn P_LoadBlockMap(state: &mut GameState, mut lump: i32) {
    let mut i: i32 = 0;
    let mut count: i32 = 0;
    let mut lumplen: i32 = 0;
    lumplen = W_LumpLength(&mut state.w_wad, lump as u32);
    count = lumplen / 2 as i32;
    state.p_setup.blockmaplump = vec![0 as i16; count as usize];
    W_ReadLump(
        &mut state.w_wad,
        lump as u32,
        state.p_setup.blockmaplump.as_mut_ptr() as *mut ::core::ffi::c_void,
    );
    state.p_setup.blockmap = state.p_setup.blockmaplump.as_mut_ptr().offset(4 as i32 as isize);
    i = 0 as i32;
    while i < count {
        state.p_setup.blockmaplump[i as usize] = state.p_setup.blockmaplump[i as usize];
        i += 1;
    }
    state.p_setup.bmaporgx = ((state.p_setup.blockmaplump[0] as i32) << FRACBITS) as fixed_t;
    state.p_setup.bmaporgy = ((state.p_setup.blockmaplump[1] as i32) << FRACBITS) as fixed_t;
    state.p_setup.bmapwidth = state.p_setup.blockmaplump[2] as i32;
    state.p_setup.bmapheight = state.p_setup.blockmaplump[3] as i32;
    state.p_setup.blocklinks =
        vec![None; (state.p_setup.bmapwidth as usize) * (state.p_setup.bmapheight as usize)];
}
pub unsafe fn P_GroupLines(state: &mut GameState) {
    let mut linebuffer: *mut *mut line_t = ::core::ptr::null_mut::<*mut line_t>();
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut li: *mut line_t = ::core::ptr::null_mut::<line_t>();
    let mut sector: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut seg: *mut seg_t = ::core::ptr::null_mut::<seg_t>();
    let mut bbox: [fixed_t; 4] = [0; 4];
    let mut block: i32 = 0;
    i = 0 as i32;
    while i < state.p_setup.numsubsectors {
        let firstline = state.p_setup.subsectors[i as usize].firstline;
        seg = state.p_setup.segs.as_mut_ptr().offset(firstline as isize);
        let seg_sidedef = (*seg).sidedef;
        state.p_setup.subsectors[i as usize].sector =
            state.p_setup.sides[seg_sidedef.0 as usize].sector;
        i += 1;
    }
    li = state.p_setup.lines.as_mut_ptr();
    state.p_setup.totallines = 0 as i32;
    i = 0 as i32;
    while i < state.p_setup.numlines {
        state.p_setup.totallines += 1;
        let front_id = (*li).frontsector.unwrap();
        (*state.p_setup.sector_mut(front_id)).linecount += 1;
        if (*li).backsector.is_some() && (*li).backsector != (*li).frontsector {
            let back_id = (*li).backsector.unwrap();
            (*state.p_setup.sector_mut(back_id)).linecount += 1;
            state.p_setup.totallines += 1;
        }
        i += 1;
        li = li.offset(1);
    }
    linebuffer = Z_Malloc(
        &mut state.z_zone,
        (state.p_setup.totallines as usize)
            .wrapping_mul(::core::mem::size_of::<*mut line_t>() as usize) as i32,
        PU_LEVEL as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut *mut line_t;
    i = 0 as i32;
    while i < state.p_setup.numsectors {
        let sec = &mut state.p_setup.sectors[i as usize];
        sec.lines = linebuffer as *mut *mut line_s;
        linebuffer = linebuffer.offset(sec.linecount as isize);
        sec.linecount = 0 as i32;
        i += 1;
    }
    i = 0 as i32;
    while i < state.p_setup.numlines {
        li = state.p_setup.lines.as_mut_ptr().offset(i as isize);
        if let Some(front_id) = (*li).frontsector {
            sector = state.p_setup.sector_mut(front_id);
            let ref mut fresh1 = *(*sector).lines.offset((*sector).linecount as isize);
            *fresh1 = li as *mut line_s;
            (*sector).linecount += 1;
        }
        if let Some(back_id) = (*li).backsector {
            if (*li).frontsector != (*li).backsector {
                sector = state.p_setup.sector_mut(back_id);
                let ref mut fresh2 = *(*sector).lines.offset((*sector).linecount as isize);
                *fresh2 = li as *mut line_s;
                (*sector).linecount += 1;
            }
        }
        i += 1;
    }
    i = 0 as i32;
    while i < state.p_setup.numsectors {
        sector = &mut state.p_setup.sectors[i as usize] as *mut sector_t;
        M_ClearBox(&raw mut bbox as *mut fixed_t);
        j = 0 as i32;
        while j < (*sector).linecount {
            li = *(*sector).lines.offset(j as isize) as *mut line_t;
            let li_v1 = state.p_setup.vertexes[(*li).v1.0 as usize];
            let li_v2 = state.p_setup.vertexes[(*li).v2.0 as usize];
            M_AddToBox(&raw mut bbox as *mut fixed_t, li_v1.x, li_v1.y);
            M_AddToBox(&raw mut bbox as *mut fixed_t, li_v2.x, li_v2.y);
            j += 1;
        }
        (*sector).soundorg.x = ((bbox[BOXRIGHT as i32 as usize] + bbox[BOXLEFT as i32 as usize])
            / 2 as i32) as fixed_t;
        (*sector).soundorg.y = ((bbox[BOXTOP as i32 as usize] + bbox[BOXBOTTOM as i32 as usize])
            / 2 as i32) as fixed_t;
        block = bbox[BOXTOP as i32 as usize] - state.p_setup.bmaporgy as i32 + 32 as i32 * FRACUNIT
            >> MAPBLOCKSHIFT;
        block = if block >= state.p_setup.bmapheight {
            state.p_setup.bmapheight - 1 as i32
        } else {
            block
        };
        (*sector).blockbox[BOXTOP as i32 as usize] = block;
        block =
            bbox[BOXBOTTOM as i32 as usize] - state.p_setup.bmaporgy as i32 - 32 as i32 * FRACUNIT
                >> MAPBLOCKSHIFT;
        block = if block < 0 as i32 { 0 as i32 } else { block };
        (*sector).blockbox[BOXBOTTOM as i32 as usize] = block;
        block = bbox[BOXRIGHT as i32 as usize] - state.p_setup.bmaporgx as i32
            + 32 as i32 * FRACUNIT
            >> MAPBLOCKSHIFT;
        block = if block >= state.p_setup.bmapwidth {
            state.p_setup.bmapwidth - 1 as i32
        } else {
            block
        };
        (*sector).blockbox[BOXRIGHT as i32 as usize] = block;
        block =
            bbox[BOXLEFT as i32 as usize] - state.p_setup.bmaporgx as i32 - 32 as i32 * FRACUNIT
                >> MAPBLOCKSHIFT;
        block = if block < 0 as i32 { 0 as i32 } else { block };
        (*sector).blockbox[BOXLEFT as i32 as usize] = block;
        i += 1;
    }
}
unsafe fn PadRejectArray(state: &mut GameState, mut array: *mut byte, mut len: u32) {
    let mut i: u32 = 0;
    let mut byte_num: u32 = 0;
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut padvalue: u32 = 0;
    let mut rejectpad: [u32; 4] = [
        ((state.p_setup.totallines * 4 as i32 + 3 as i32 & !(3 as i32)) + 24 as i32) as u32,
        0 as i32 as u32,
        50 as i32 as u32,
        0x1d4a11 as i32 as u32,
    ];
    dest = array;
    i = 0 as u32;
    while i < len && (i as usize) < ::core::mem::size_of::<[u32; 4]>() as usize {
        byte_num = i.wrapping_rem(4 as u32);
        *dest = (rejectpad[i.wrapping_div(4 as u32) as usize] >> byte_num.wrapping_mul(8 as u32)
            & 0xff as u32) as byte;
        dest = dest.offset(1);
        i = i.wrapping_add(1);
    }
    if len as usize > ::core::mem::size_of::<[u32; 4]>() as usize {
        eprintln!(
            "PadRejectArray: REJECT lump too short to pad! ({} > {})",
            len,
            ::core::mem::size_of::<[u32; 4]>() as i32,
        );
        if M_CheckParm(state, "-reject_pad_with_ff") != 0 {
            padvalue = 0xff as u32;
        } else {
            padvalue = 0xf00 as u32;
        }
        memset(
            array.offset(::core::mem::size_of::<[u32; 4]>() as usize as isize)
                as *mut ::core::ffi::c_void,
            padvalue as i32,
            (len as size_t).wrapping_sub(::core::mem::size_of::<[u32; 4]>() as size_t),
        );
    }
}
unsafe fn P_LoadReject(state: &mut GameState, mut lumpnum: i32) {
    let mut minlength: i32 = 0;
    let mut lumplen: i32 = 0;
    minlength = (state.p_setup.numsectors * state.p_setup.numsectors + 7 as i32) / 8 as i32;
    lumplen = W_LumpLength(&mut state.w_wad, lumpnum as u32);
    if lumplen >= minlength {
        state.p_setup.rejectmatrix = W_CacheLumpNum(state, lumpnum, PU_LEVEL as i32) as *mut byte;
    } else {
        state.p_setup.rejectmatrix = Z_Malloc(
            &mut state.z_zone,
            minlength,
            PU_LEVEL as i32,
            &raw mut state.p_setup.rejectmatrix as *mut ::core::ffi::c_void,
        ) as *mut byte;
        W_ReadLump(&mut state.w_wad, 
            lumpnum as u32,
            state.p_setup.rejectmatrix as *mut ::core::ffi::c_void,
        );
        PadRejectArray(
            state,
            state.p_setup.rejectmatrix.offset(lumplen as isize),
            (minlength - lumplen) as u32,
        );
    };
}
pub unsafe fn P_SetupLevel(state: &mut GameState, mut episode: i32, mut map: i32) {
    let mut i: i32 = 0;
    let mut lumpnum: i32 = 0;
    state.g_game.wminfo.maxfrags = 0 as i32;
    state.g_game.totalsecret = state.g_game.wminfo.maxfrags;
    state.g_game.totalitems = state.g_game.totalsecret;
    state.g_game.totalkills = state.g_game.totalitems;
    state.g_game.wminfo.partime = 180 as i32;
    i = 0 as i32;
    while i < MAXPLAYERS {
        state.g_game.players[i as usize].itemcount = 0 as i32;
        state.g_game.players[i as usize].secretcount = state.g_game.players[i as usize].itemcount;
        state.g_game.players[i as usize].killcount = state.g_game.players[i as usize].secretcount;
        i += 1;
    }
    state.g_game.players[state.g_game.consoleplayer as usize].viewz = 1 as i32 as fixed_t;
    S_Start(state);
    Z_FreeTags(
        &mut state.z_zone,
        PU_LEVEL as i32,
        PU_PURGELEVEL as i32 - 1 as i32,
    );
    P_InitThinkers(state);
    let lumpname = if state.doomstat.gamemode as u32 == commercial as i32 as u32 {
        if map < 10 as i32 {
            format!("map0{}", map)
        } else {
            format!("map{}", map)
        }
    } else {
        format!(
            "E{}M{}",
            char::from((('0' as i32) + episode) as u8),
            char::from((('0' as i32) + map) as u8)
        )
    };
    lumpnum = W_GetNumForName(&mut state.w_wad, &lumpname);
    state.p_tick.leveltime = 0 as i32;
    P_LoadBlockMap(state, lumpnum + ML_BLOCKMAP as i32);
    P_LoadVertexes(state, lumpnum + ML_VERTEXES as i32);
    P_LoadSectors(state, lumpnum + ML_SECTORS as i32);
    P_LoadSideDefs(state, lumpnum + ML_SIDEDEFS as i32);
    P_LoadLineDefs(state, lumpnum + ML_LINEDEFS as i32);
    P_LoadSubsectors(state, lumpnum + ML_SSECTORS as i32);
    P_LoadNodes(state, lumpnum + ML_NODES as i32);
    P_LoadSegs(state, lumpnum + ML_SEGS as i32);
    P_GroupLines(state);
    P_LoadReject(state, lumpnum + ML_REJECT as i32);
    state.g_game.bodyqueslot = 0 as i32;
    state.p_setup.deathmatch_p = &raw mut state.p_setup.deathmatchstarts as *mut mapthing_t;
    P_LoadThings(state, lumpnum + ML_THINGS as i32);
    if state.g_game.deathmatch != 0 {
        i = 0 as i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] != 0 {
                state.g_game.players[i as usize].mo = ::core::ptr::null_mut::<mobj_t>();
                G_DeathMatchSpawnPlayer(state, i);
            }
            i += 1;
        }
    }
    let gs = state;
    gs.p_mobj.iquetail = 0 as i32;
    gs.p_mobj.iquehead = gs.p_mobj.iquetail;
    P_SpawnSpecials(gs);
    if gs.g_game.precache {
        R_PrecacheLevel(gs);
    }
}
pub unsafe fn P_Init(state: &mut GameState) {
    P_InitSwitchList(state);
    P_InitPicAnims(state);
    let sprnames = state.info.sprnames;
    R_InitSprites(state, &sprnames);
}
