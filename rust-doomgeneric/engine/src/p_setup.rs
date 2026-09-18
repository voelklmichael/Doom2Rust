use crate::d_mode::GameMode_t;
use crate::doomdef::MAXPLAYERS;
use crate::fixed_cstr::FixedCStr;
use crate::g_game::G_DeathMatchSpawnPlayer;
use crate::game_state::GameState;
use crate::i_system::I_GetMemoryValue;
use crate::m_argv::M_CheckParm;
use crate::m_bbox::M_AddToBox;
use crate::m_bbox::M_ClearBox;
use crate::m_bbox::{BOXBOTTOM, BOXLEFT, BOXRIGHT, BOXTOP};
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedDiv;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::mem_compat::memset;
use crate::p_maputl::MAPBLOCKSHIFT;
use crate::p_mobj::P_SpawnMapThing;
use crate::p_mobj::{
    degenmobj_t, line_s, line_t, mapthing_t, sector_t, subsector_s, subsector_t, thinker_s,
    vertex_t, MobjId, SlopeType, ThinkerFn,
};
use crate::p_spec::P_InitPicAnims;
use crate::p_spec::P_SpawnSpecials;
use crate::p_spec::ML_TWOSIDED;
use crate::p_switch::P_InitSwitchList;
use crate::p_tick::P_InitThinkers;
use crate::r_data::R_FlatNumForName;
use crate::r_data::R_PrecacheLevel;
use crate::r_data::R_TextureNumForName;
use crate::r_defs::{node_t, seg_t, side_t};
use crate::r_things::R_InitSprites;
use crate::s_sound::S_Start;
use crate::stdint_types::byte;
use crate::stdint_types::size_t;
use crate::tables::angle_t;
use crate::w_wad::W_CacheLumpNum;
use crate::w_wad::W_GetNumForName;
use crate::w_wad::W_LumpLength;
use crate::w_wad::W_ReadLump;
use crate::w_wad::W_ReleaseLumpNum;

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
    slopetype: SlopeType::ST_HORIZONTAL,
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
    lines: Vec::new(),
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
    pub nodes: Vec<node_t>,
    pub numlines: i32,
    pub lines: Vec<line_t>,
    pub numsides: i32,
    pub sides: Vec<side_t>,
    pub totallines: i32,
    pub bmapwidth: i32,
    pub bmapheight: i32,
    pub blockmaplump: Vec<i16>,
    pub bmaporgx: fixed_t,
    pub bmaporgy: fixed_t,
    pub blocklinks: Vec<Option<MobjId>>,
    pub rejectmatrix: Vec<byte>,
    pub deathmatchstarts: [mapthing_t; 10],
    pub deathmatch_p: usize,
    pub playerstarts: [mapthing_t; 4],
    pub null_sector_id: Option<SectorId>,
    pub junk_line_id: Option<LineId>,
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
            nodes: Vec::new(),
            numlines: 0,
            lines: Vec::new(),
            numsides: 0,
            sides: Vec::new(),
            totallines: 0,
            bmapwidth: 0,
            bmapheight: 0,
            blockmaplump: Vec::new(),
            bmaporgx: 0,
            bmaporgy: 0,
            blocklinks: Vec::new(),
            rejectmatrix: Vec::new(),
            deathmatchstarts: [mapthing_t {
                x: 0,
                y: 0,
                angle: 0,
                type_0: 0,
                options: 0,
            }; 10],
            deathmatch_p: 0,
            playerstarts: [mapthing_t {
                x: 0,
                y: 0,
                angle: 0,
                type_0: 0,
                options: 0,
            }; 4],
            null_sector_id: None,
            junk_line_id: None,
        }
    }

    pub fn sector_mut(&mut self, id: SectorId) -> &mut sector_t {
        &mut self.sectors[id.0 as usize]
    }
    pub fn side_mut(&mut self, id: SideId) -> &mut side_t {
        &mut self.sides[id.0 as usize]
    }
    pub fn subsector(&self, id: SubsectorId) -> subsector_t {
        self.subsectors[id.0 as usize]
    }
    pub fn vertex(&self, id: VertexId) -> vertex_t {
        self.vertexes[id.0 as usize]
    }
    pub fn line_mut(&mut self, id: LineId) -> &mut line_t {
        &mut self.lines[id.0 as usize]
    }
    pub fn line(&self, id: LineId) -> line_t {
        self.lines[id.0 as usize]
    }
    /// Returns a `LineId` for a scratch line that isn't part of the map,
    /// with only `.tag` set -- for vanilla Doom's "tag 666/667 boss death
    /// trigger" idiom, which calls EV_DoDoor/EV_DoFloor with a fabricated
    /// line that exists only to carry a tag for P_FindSectorFromLineTag to
    /// match against. One slot is reused across all such calls; they never
    /// overlap (each EV_* call fully finishes before the next one reuses it).
    pub fn junk_line(&mut self, tag: i16) -> LineId {
        let id = match self.junk_line_id {
            Some(id) => id,
            None => {
                let id = LineId(self.lines.len() as u32);
                self.lines.push(ZERO_LINE);
                self.junk_line_id = Some(id);
                id
            }
        };
        self.lines[id.0 as usize].tag = tag;
        id
    }
    pub fn seg_mut(&mut self, id: SegId) -> &mut seg_t {
        &mut self.segs[id.0 as usize]
    }
    pub fn seg(&self, id: SegId) -> seg_t {
        self.segs[id.0 as usize]
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
        .wrapping_div(::core::mem::size_of::<mapvertex_t>()) as i32;
    state.p_setup.numvertexes = numvertexes;
    state.p_setup.vertexes = Vec::with_capacity(numvertexes as usize);
    data = W_CacheLumpNum(state, lump) as *mut byte;
    ml = data as *mut mapvertex_t;
    i = 0_i32;
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
pub fn GetSectorAtNullAddress(state: &mut GameState) -> SectorId {
    if state.p_setup.null_sector_id.is_none() {
        let mut sentinel = ZERO_SECTOR;
        if let Some(value) = I_GetMemoryValue(state, 0_u32, 4_i32) {
            sentinel.floorheight = value as i32;
        }
        if let Some(value) = I_GetMemoryValue(state, 4_u32, 4_i32) {
            sentinel.ceilingheight = value as i32;
        }
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
        .wrapping_div(::core::mem::size_of::<mapseg_t>()) as i32;
    state.p_setup.numsegs = numsegs;
    state.p_setup.segs = Vec::with_capacity(numsegs as usize);
    data = W_CacheLumpNum(state, lump) as *mut byte;
    ml = data as *mut mapseg_t;
    i = 0_i32;
    while i < numsegs {
        let v1 = VertexId((*ml).v1 as u32);
        let v2 = VertexId((*ml).v2 as u32);
        let seg_angle = (((*ml).angle as i32) << 16_i32) as angle_t;
        let seg_offset = (((*ml).offset as i32) << 16_i32) as fixed_t;
        linedef = (*ml).linedef as i32;
        let seg_linedef = LineId(linedef as u32);
        ldef = state.p_setup.line_mut(seg_linedef);
        side = (*ml).side as i32;
        let seg_sidenum = *(&raw mut (*ldef).sidenum as *mut i16).offset(side as isize) as u32;
        let frontsector = Some(state.p_setup.sides[seg_sidenum as usize].sector);
        let backsector = if (*ldef).flags as i32 & ML_TWOSIDED != 0 {
            sidenum = (*ldef).sidenum[(side ^ 1_i32) as usize] as i32;
            if sidenum < 0_i32 || sidenum >= state.p_setup.numsides {
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
        .wrapping_div(::core::mem::size_of::<mapsubsector_t>()) as i32;
    state.p_setup.numsubsectors = numsubsectors;
    state.p_setup.subsectors = Vec::with_capacity(numsubsectors as usize);
    data = W_CacheLumpNum(state, lump) as *mut byte;
    ms = data as *mut mapsubsector_t;
    i = 0_i32;
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
        .wrapping_div(::core::mem::size_of::<mapsector_t>()) as i32;
    state.p_setup.numsectors = numsectors;
    state.p_setup.sectors = vec![ZERO_SECTOR; numsectors as usize];
    data = W_CacheLumpNum(state, lump) as *mut byte;
    ms = data as *mut mapsector_t;
    i = 0_i32;
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
    state.p_setup.numnodes = (W_LumpLength(&mut state.w_wad, lump as u32) as usize)
        .wrapping_div(::core::mem::size_of::<mapnode_t>()) as i32;
    state.p_setup.nodes = vec![
        node_t {
            x: 0,
            y: 0,
            dx: 0,
            dy: 0,
            bbox: [[0; 4]; 2],
            children: [0; 2],
        };
        state.p_setup.numnodes as usize
    ];
    data = W_CacheLumpNum(state, lump) as *mut byte;
    mn = data as *mut mapnode_t;
    i = 0_i32;
    while i < state.p_setup.numnodes {
        let no = &mut state.p_setup.nodes[i as usize];
        no.x = (((*mn).x as i32) << FRACBITS) as fixed_t;
        no.y = (((*mn).y as i32) << FRACBITS) as fixed_t;
        no.dx = (((*mn).dx as i32) << FRACBITS) as fixed_t;
        no.dy = (((*mn).dy as i32) << FRACBITS) as fixed_t;
        j = 0_i32;
        while j < 2_i32 {
            no.children[j as usize] = (*mn).children[j as usize] as i16 as u16;
            k = 0_i32;
            while k < 4_i32 {
                no.bbox[j as usize][k as usize] =
                    (((*mn).bbox[j as usize][k as usize] as i32) << FRACBITS) as fixed_t;
                k += 1;
            }
            j += 1;
        }
        i += 1;
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
    data = W_CacheLumpNum(state, lump) as *mut byte;
    numthings = (W_LumpLength(&mut state.w_wad, lump as u32) as usize)
        .wrapping_div(::core::mem::size_of::<mapthing_t>()) as i32;
    mt = data as *mut mapthing_t;
    i = 0_i32;
    while i < numthings {
        spawn = true;
        if state.doomstat.gamemode as u32 != GameMode_t::commercial as i32 as u32 {
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
        P_SpawnMapThing(state, spawnthing);
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
    state.p_setup.numlines = (W_LumpLength(&mut state.w_wad, lump as u32) as usize)
        .wrapping_div(::core::mem::size_of::<maplinedef_t>()) as i32;
    state.p_setup.lines = vec![ZERO_LINE; state.p_setup.numlines as usize];
    data = W_CacheLumpNum(state, lump) as *mut byte;
    mld = data as *mut maplinedef_t;
    i = 0_i32;
    while i < state.p_setup.numlines {
        ld = &mut state.p_setup.lines[i as usize] as *mut line_t;
        (*ld).flags = (*mld).flags;
        (*ld).special = (*mld).special;
        (*ld).tag = (*mld).tag;
        (*ld).v1 = VertexId((*mld).v1 as u32);
        let v1 = state.p_setup.vertex((*ld).v1);
        (*ld).v2 = VertexId((*mld).v2 as u32);
        let v2 = state.p_setup.vertex((*ld).v2);
        (*ld).dx = v2.x - v1.x;
        (*ld).dy = v2.y - v1.y;
        if (*ld).dx == 0 {
            (*ld).slopetype = SlopeType::ST_VERTICAL;
        } else if (*ld).dy == 0 {
            (*ld).slopetype = SlopeType::ST_HORIZONTAL;
        } else if FixedDiv((*ld).dy, (*ld).dx) > 0_i32 {
            (*ld).slopetype = SlopeType::ST_POSITIVE;
        } else {
            (*ld).slopetype = SlopeType::ST_NEGATIVE;
        }
        if v1.x < v2.x {
            (*ld).bbox[BOXLEFT as usize] = v1.x;
            (*ld).bbox[BOXRIGHT as usize] = v2.x;
        } else {
            (*ld).bbox[BOXLEFT as usize] = v2.x;
            (*ld).bbox[BOXRIGHT as usize] = v1.x;
        }
        if v1.y < v2.y {
            (*ld).bbox[BOXBOTTOM as usize] = v1.y;
            (*ld).bbox[BOXTOP as usize] = v2.y;
        } else {
            (*ld).bbox[BOXBOTTOM as usize] = v2.y;
            (*ld).bbox[BOXTOP as usize] = v1.y;
        }
        (*ld).sidenum[0] = (*mld).sidenum[0];
        (*ld).sidenum[1] = (*mld).sidenum[1];
        if (*ld).sidenum[0] as i32 != -1_i32 {
            let side_sector = state.p_setup.sides[(*ld).sidenum[0] as usize].sector;
            (*ld).frontsector = Some(side_sector);
        } else {
            (*ld).frontsector = None;
        }
        if (*ld).sidenum[1] as i32 != -1_i32 {
            let side_sector = state.p_setup.sides[(*ld).sidenum[1] as usize].sector;
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
        .wrapping_div(::core::mem::size_of::<mapsidedef_t>()) as i32;
    state.p_setup.numsides = numsides;
    state.p_setup.sides = Vec::with_capacity(numsides as usize);
    data = W_CacheLumpNum(state, lump) as *mut byte;
    msd = data as *mut mapsidedef_t;
    i = 0_i32;
    while i < numsides {
        let sd = side_t {
            textureoffset: (((*msd).textureoffset as i32) << FRACBITS) as fixed_t,
            rowoffset: (((*msd).rowoffset as i32) << FRACBITS) as fixed_t,
            toptexture: R_TextureNumForName(&mut state.r_data, &(*msd).toptexture.as_str()) as i16,
            bottomtexture: R_TextureNumForName(&mut state.r_data, &(*msd).bottomtexture.as_str())
                as i16,
            midtexture: R_TextureNumForName(&mut state.r_data, &(*msd).midtexture.as_str()) as i16,
            sector: SectorId((*msd).sector as u32),
        };
        state.p_setup.sides.push(sd);
        i += 1;
        msd = msd.offset(1);
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub fn P_LoadBlockMap(state: &mut GameState, mut lump: i32) {
    let mut lumplen: i32 = 0;
    lumplen = W_LumpLength(&mut state.w_wad, lump as u32);
    let mut raw = vec![0u8; lumplen as usize];
    W_ReadLump(&mut state.w_wad, lump as u32, &mut raw);
    state.p_setup.blockmaplump = raw
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect();
    state.p_setup.bmaporgx = ((state.p_setup.blockmaplump[0] as i32) << FRACBITS) as fixed_t;
    state.p_setup.bmaporgy = ((state.p_setup.blockmaplump[1] as i32) << FRACBITS) as fixed_t;
    state.p_setup.bmapwidth = state.p_setup.blockmaplump[2] as i32;
    state.p_setup.bmapheight = state.p_setup.blockmaplump[3] as i32;
    state.p_setup.blocklinks =
        vec![None; (state.p_setup.bmapwidth as usize) * (state.p_setup.bmapheight as usize)];
}
pub unsafe fn P_GroupLines(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut sector: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut bbox: [fixed_t; 4] = [0; 4];
    let mut block: i32 = 0;
    i = 0_i32;
    while i < state.p_setup.numsubsectors {
        let firstline = state.p_setup.subsectors[i as usize].firstline;
        let seg_sidedef = state.p_setup.segs[firstline as usize].sidedef;
        state.p_setup.subsectors[i as usize].sector =
            state.p_setup.sides[seg_sidedef.0 as usize].sector;
        i += 1;
    }
    state.p_setup.totallines = 0_i32;
    i = 0_i32;
    while i < state.p_setup.numlines {
        state.p_setup.totallines += 1;
        let li = state.p_setup.lines[i as usize];
        let front_id = li.frontsector.unwrap();
        state.p_setup.sector_mut(front_id).linecount += 1;
        if li.backsector.is_some() && li.backsector != li.frontsector {
            let back_id = li.backsector.unwrap();
            state.p_setup.sector_mut(back_id).linecount += 1;
            state.p_setup.totallines += 1;
        }
        i += 1;
    }
    i = 0_i32;
    while i < state.p_setup.numsectors {
        let sec = &mut state.p_setup.sectors[i as usize];
        sec.lines = Vec::with_capacity(sec.linecount as usize);
        sec.linecount = 0_i32;
        i += 1;
    }
    i = 0_i32;
    while i < state.p_setup.numlines {
        let li_id = LineId(i as u32);
        let li = state.p_setup.lines[i as usize];
        if let Some(front_id) = li.frontsector {
            sector = state.p_setup.sector_mut(front_id);
            (*sector).lines.push(li_id);
            (*sector).linecount += 1;
        }
        if let Some(back_id) = li.backsector {
            if li.frontsector != li.backsector {
                sector = state.p_setup.sector_mut(back_id);
                (*sector).lines.push(li_id);
                (*sector).linecount += 1;
            }
        }
        i += 1;
    }
    i = 0_i32;
    while i < state.p_setup.numsectors {
        sector = &mut state.p_setup.sectors[i as usize] as *mut sector_t;
        M_ClearBox(&mut bbox);
        j = 0_i32;
        while j < (*sector).linecount {
            let li_id = (*sector).lines[j as usize];
            let li = state.p_setup.line(li_id);
            let li_v1 = state.p_setup.vertexes[li.v1.0 as usize];
            let li_v2 = state.p_setup.vertexes[li.v2.0 as usize];
            M_AddToBox(&mut bbox, li_v1.x, li_v1.y);
            M_AddToBox(&mut bbox, li_v2.x, li_v2.y);
            j += 1;
        }
        (*sector).soundorg.x =
            ((bbox[BOXRIGHT as usize] + bbox[BOXLEFT as usize]) / 2_i32) as fixed_t;
        (*sector).soundorg.y =
            ((bbox[BOXTOP as usize] + bbox[BOXBOTTOM as usize]) / 2_i32) as fixed_t;
        block = (bbox[BOXTOP as usize] - state.p_setup.bmaporgy + 32_i32 * FRACUNIT) >> MAPBLOCKSHIFT;
        block = if block >= state.p_setup.bmapheight {
            state.p_setup.bmapheight - 1_i32
        } else {
            block
        };
        (*sector).blockbox[BOXTOP as usize] = block;
        block =
            (bbox[BOXBOTTOM as usize] - state.p_setup.bmaporgy - 32_i32 * FRACUNIT) >> MAPBLOCKSHIFT;
        block = if block < 0_i32 { 0_i32 } else { block };
        (*sector).blockbox[BOXBOTTOM as usize] = block;
        block =
            (bbox[BOXRIGHT as usize] - state.p_setup.bmaporgx + 32_i32 * FRACUNIT) >> MAPBLOCKSHIFT;
        block = if block >= state.p_setup.bmapwidth {
            state.p_setup.bmapwidth - 1_i32
        } else {
            block
        };
        (*sector).blockbox[BOXRIGHT as usize] = block;
        block =
            (bbox[BOXLEFT as usize] - state.p_setup.bmaporgx - 32_i32 * FRACUNIT) >> MAPBLOCKSHIFT;
        block = if block < 0_i32 { 0_i32 } else { block };
        (*sector).blockbox[BOXLEFT as usize] = block;
        i += 1;
    }
}
unsafe fn PadRejectArray(state: &mut GameState, mut array: *mut byte, mut len: u32) {
    let mut i: u32 = 0;
    let mut byte_num: u32 = 0;
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut padvalue: u32 = 0;
    let mut rejectpad: [u32; 4] = [
        (((state.p_setup.totallines * 4_i32 + 3_i32) & !3_i32) + 24_i32) as u32,
        0_i32 as u32,
        50_i32 as u32,
        0x1d4a11_i32 as u32,
    ];
    dest = array;
    i = 0_u32;
    while i < len && (i as usize) < ::core::mem::size_of::<[u32; 4]>() {
        byte_num = i.wrapping_rem(4_u32);
        *dest = (rejectpad[i.wrapping_div(4_u32) as usize] >> byte_num.wrapping_mul(8_u32)
            & 0xff_u32) as byte;
        dest = dest.offset(1);
        i = i.wrapping_add(1);
    }
    if len as usize > ::core::mem::size_of::<[u32; 4]>() {
        eprintln!(
            "PadRejectArray: REJECT lump too short to pad! ({} > {})",
            len,
            ::core::mem::size_of::<[u32; 4]>() as i32,
        );
        if M_CheckParm(state, "-reject_pad_with_ff") != 0 {
            padvalue = 0xff_u32;
        } else {
            padvalue = 0xf00_u32;
        }
        memset(
            array.add(::core::mem::size_of::<[u32; 4]>()) as *mut ::core::ffi::c_void,
            padvalue as i32,
            (len as size_t).wrapping_sub(::core::mem::size_of::<[u32; 4]>() as size_t),
        );
    }
}
unsafe fn P_LoadReject(state: &mut GameState, mut lumpnum: i32) {
    let mut minlength: i32 = 0;
    let mut lumplen: i32 = 0;
    minlength = (state.p_setup.numsectors * state.p_setup.numsectors + 7_i32) / 8_i32;
    lumplen = W_LumpLength(&mut state.w_wad, lumpnum as u32);
    if lumplen >= minlength {
        let lump_ptr = W_CacheLumpNum(state, lumpnum) as *const byte;
        state.p_setup.rejectmatrix =
            ::core::slice::from_raw_parts(lump_ptr, minlength as usize).to_vec();
    } else {
        state.p_setup.rejectmatrix = vec![0u8; minlength as usize];
        W_ReadLump(
            &mut state.w_wad,
            lumpnum as u32,
            &mut state.p_setup.rejectmatrix,
        );
        let pad_ptr = state
            .p_setup
            .rejectmatrix
            .as_mut_ptr()
            .offset(lumplen as isize);
        PadRejectArray(state, pad_ptr, (minlength - lumplen) as u32);
    };
}
pub fn P_SetupLevel(state: &mut GameState, mut episode: i32, mut map: i32) {
    let mut i: i32 = 0;
    let mut lumpnum: i32 = 0;
    state.g_game.wminfo.maxfrags = 0_i32;
    state.g_game.totalsecret = state.g_game.wminfo.maxfrags;
    state.g_game.totalitems = state.g_game.totalsecret;
    state.g_game.totalkills = state.g_game.totalitems;
    state.g_game.wminfo.partime = 180_i32;
    i = 0_i32;
    while i < MAXPLAYERS {
        state.g_game.players[i as usize].itemcount = 0_i32;
        state.g_game.players[i as usize].secretcount = state.g_game.players[i as usize].itemcount;
        state.g_game.players[i as usize].killcount = state.g_game.players[i as usize].secretcount;
        i += 1;
    }
    state.g_game.players[state.g_game.consoleplayer as usize].viewz = 1_i32 as fixed_t;
    S_Start(state);
    P_InitThinkers(state);
    let lumpname = if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        if map < 10_i32 {
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
    state.p_tick.leveltime = 0_i32;
    P_LoadBlockMap(state, lumpnum + ML_BLOCKMAP as i32);
    unsafe { P_LoadVertexes(state, lumpnum + ML_VERTEXES as i32) };
    unsafe { P_LoadSectors(state, lumpnum + ML_SECTORS as i32) };
    unsafe { P_LoadSideDefs(state, lumpnum + ML_SIDEDEFS as i32) };
    unsafe { P_LoadLineDefs(state, lumpnum + ML_LINEDEFS as i32) };
    unsafe { P_LoadSubsectors(state, lumpnum + ML_SSECTORS as i32) };
    unsafe { P_LoadNodes(state, lumpnum + ML_NODES as i32) };
    unsafe { P_LoadSegs(state, lumpnum + ML_SEGS as i32) };
    unsafe { P_GroupLines(state) };
    unsafe { P_LoadReject(state, lumpnum + ML_REJECT as i32) };
    state.g_game.bodyqueslot = 0_i32;
    state.p_setup.deathmatch_p = 0;
    unsafe { P_LoadThings(state, lumpnum + ML_THINGS as i32) };
    if state.g_game.deathmatch != 0 {
        i = 0_i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                state.g_game.players[i as usize].mo = None;
                G_DeathMatchSpawnPlayer(state, i);
            }
            i += 1;
        }
    }
    let gs = state;
    gs.p_mobj.iquetail = 0_i32;
    gs.p_mobj.iquehead = gs.p_mobj.iquetail;
    P_SpawnSpecials(gs);
    if gs.g_game.precache {
        R_PrecacheLevel(gs);
    }
}
pub fn P_Init(state: &mut GameState) {
    P_InitSwitchList(state);
    P_InitPicAnims(state);
    let sprnames = state.info.sprnames;
    R_InitSprites(state, &sprnames);
}
