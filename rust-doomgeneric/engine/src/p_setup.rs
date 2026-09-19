use crate::d_mode::GameMode_t;
use crate::doomdef::MAXPLAYERS;
use crate::fixed_cstr::FixedCStr;
use crate::g_game::G_DeathMatchSpawnPlayer;
use crate::game_state::GameState;
use crate::i_system::I_GetMemoryValue;
use crate::m_argv::M_CheckParm;
use crate::m_bbox::BoxIndex;
use crate::m_bbox::M_AddToBox;
use crate::m_bbox::M_ClearBox;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedDiv;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
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
use crate::tables::angle_t;
use crate::w_wad::W_GetNumForName;
use crate::w_wad::W_LumpBytes;
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

impl Default for PSetupState {
    fn default() -> Self {
        Self::new()
    }
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

#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; discriminants must stay
pub enum MapLump {
    ML_LABEL = 0,
    ML_THINGS = 1,
    ML_LINEDEFS = 2,
    ML_SIDEDEFS = 3,
    ML_VERTEXES = 4,
    ML_SEGS = 5,
    ML_SSECTORS = 6,
    ML_NODES = 7,
    ML_SECTORS = 8,
    ML_REJECT = 9,
    ML_BLOCKMAP = 10,
}
/// Bounds-checked little-endian reader over a map lump's raw bytes.
struct LumpReader {
    data: std::rc::Rc<[u8]>,
    pos: usize,
}
impl LumpReader {
    fn new(state: &mut GameState, lump: i32) -> Self {
        LumpReader {
            data: W_LumpBytes(state, lump),
            pos: 0,
        }
    }

    fn i16(&mut self) -> i16 {
        let value = i16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        value
    }

    fn u16(&mut self) -> u16 {
        self.i16() as u16
    }

    fn name8(&mut self) -> FixedCStr<8> {
        let name = FixedCStr::<8>::from_bytes(&self.data[self.pos..self.pos + 8]);
        self.pos += 8;
        name
    }
}
const MAPVERTEX_SIZE: usize = 4;
const MAPSIDEDEF_SIZE: usize = 30;
const MAPLINEDEF_SIZE: usize = 14;
const MAPSECTOR_SIZE: usize = 26;
const MAPSUBSECTOR_SIZE: usize = 4;
const MAPSEG_SIZE: usize = 12;
const MAPNODE_SIZE: usize = 28;
const MAPTHING_SIZE: usize = 10;
pub fn P_LoadVertexes(state: &mut GameState, lump: i32) {
    let numvertexes =
        (W_LumpLength(&mut state.w_wad, lump as u32) as usize / MAPVERTEX_SIZE) as i32;
    state.p_setup.numvertexes = numvertexes;
    state.p_setup.vertexes = Vec::with_capacity(numvertexes as usize);
    let mut reader = LumpReader::new(state, lump);
    for _ in 0..numvertexes {
        let x = reader.i16() as i32;
        let y = reader.i16() as i32;
        state.p_setup.vertexes.push(vertex_t {
            x: (x << FRACBITS) as fixed_t,
            y: (y << FRACBITS) as fixed_t,
        });
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
pub fn P_LoadSegs(state: &mut GameState, lump: i32) {
    let numsegs = (W_LumpLength(&mut state.w_wad, lump as u32) as usize / MAPSEG_SIZE) as i32;
    state.p_setup.numsegs = numsegs;
    state.p_setup.segs = Vec::with_capacity(numsegs as usize);
    let mut reader = LumpReader::new(state, lump);
    for _ in 0..numsegs {
        let v1 = VertexId(reader.i16() as u32);
        let v2 = VertexId(reader.i16() as u32);
        let seg_angle = ((reader.i16() as i32) << 16_i32) as angle_t;
        let linedef = reader.i16() as i32;
        let side = reader.i16() as i32;
        let seg_offset = ((reader.i16() as i32) << 16_i32) as fixed_t;
        let seg_linedef = LineId(linedef as u32);
        let ldef = state.p_setup.line(seg_linedef);
        let seg_sidenum = ldef.sidenum[side as usize] as u32;
        let frontsector = Some(state.p_setup.sides[seg_sidenum as usize].sector);
        let backsector = if ldef.flags as i32 & ML_TWOSIDED != 0 {
            let sidenum = ldef.sidenum[(side ^ 1_i32) as usize] as i32;
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
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub fn P_LoadSubsectors(state: &mut GameState, lump: i32) {
    let numsubsectors =
        (W_LumpLength(&mut state.w_wad, lump as u32) as usize / MAPSUBSECTOR_SIZE) as i32;
    state.p_setup.numsubsectors = numsubsectors;
    state.p_setup.subsectors = Vec::with_capacity(numsubsectors as usize);
    let mut reader = LumpReader::new(state, lump);
    for _ in 0..numsubsectors {
        let numsegs = reader.i16();
        let firstseg = reader.i16();
        state.p_setup.subsectors.push(subsector_s {
            sector: SectorId(0),
            numlines: numsegs,
            firstline: firstseg,
        });
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub fn P_LoadSectors(state: &mut GameState, lump: i32) {
    let numsectors = (W_LumpLength(&mut state.w_wad, lump as u32) as usize / MAPSECTOR_SIZE) as i32;
    state.p_setup.numsectors = numsectors;
    state.p_setup.sectors = vec![ZERO_SECTOR; numsectors as usize];
    let mut reader = LumpReader::new(state, lump);
    for i in 0..numsectors as usize {
        let floorheight = reader.i16();
        let ceilingheight = reader.i16();
        let floorpic_name = reader.name8();
        let ceilingpic_name = reader.name8();
        let lightlevel = reader.i16();
        let special = reader.i16();
        let tag = reader.i16();
        let floorpic = R_FlatNumForName(state, &floorpic_name.as_str()) as i16;
        let ceilingpic = R_FlatNumForName(state, &ceilingpic_name.as_str()) as i16;
        let ss = &mut state.p_setup.sectors[i];
        ss.floorheight = ((floorheight as i32) << FRACBITS) as fixed_t;
        ss.ceilingheight = ((ceilingheight as i32) << FRACBITS) as fixed_t;
        ss.floorpic = floorpic;
        ss.ceilingpic = ceilingpic;
        ss.lightlevel = lightlevel;
        ss.special = special;
        ss.tag = tag;
        ss.thinglist = None;
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub fn P_LoadNodes(state: &mut GameState, lump: i32) {
    state.p_setup.numnodes =
        (W_LumpLength(&mut state.w_wad, lump as u32) as usize / MAPNODE_SIZE) as i32;
    state.p_setup.nodes = Vec::with_capacity(state.p_setup.numnodes as usize);
    let mut reader = LumpReader::new(state, lump);
    for _ in 0..state.p_setup.numnodes {
        let mut no = node_t {
            x: ((reader.i16() as i32) << FRACBITS) as fixed_t,
            y: ((reader.i16() as i32) << FRACBITS) as fixed_t,
            dx: ((reader.i16() as i32) << FRACBITS) as fixed_t,
            dy: ((reader.i16() as i32) << FRACBITS) as fixed_t,
            bbox: [[0; 4]; 2],
            children: [0; 2],
        };
        for j in 0..2 {
            for k in 0..4 {
                no.bbox[j][k] = ((reader.i16() as i32) << FRACBITS) as fixed_t;
            }
        }
        for j in 0..2 {
            no.children[j] = reader.u16();
        }
        state.p_setup.nodes.push(no);
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub fn P_LoadThings(state: &mut GameState, lump: i32) {
    let numthings = (W_LumpLength(&mut state.w_wad, lump as u32) as usize / MAPTHING_SIZE) as i32;
    let mut reader = LumpReader::new(state, lump);
    for _ in 0..numthings {
        let spawnthing = mapthing_t {
            x: reader.i16(),
            y: reader.i16(),
            angle: reader.i16(),
            type_0: reader.i16(),
            options: reader.i16(),
        };
        let spawn = !(state.doomstat.gamemode as u32 != GameMode_t::commercial as i32 as u32
            && matches!(
                spawnthing.type_0,
                64 | 88 | 89 | 69 | 67 | 71 | 65 | 66 | 68 | 84
            ));
        if !spawn {
            break;
        }
        P_SpawnMapThing(state, spawnthing);
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub fn P_LoadLineDefs(state: &mut GameState, lump: i32) {
    state.p_setup.numlines =
        (W_LumpLength(&mut state.w_wad, lump as u32) as usize / MAPLINEDEF_SIZE) as i32;
    state.p_setup.lines = vec![ZERO_LINE; state.p_setup.numlines as usize];
    let mut reader = LumpReader::new(state, lump);
    for i in 0..state.p_setup.numlines as usize {
        let mut ld = state.p_setup.lines[i];
        ld.v1 = VertexId(reader.i16() as u32);
        ld.v2 = VertexId(reader.i16() as u32);
        ld.flags = reader.i16();
        ld.special = reader.i16();
        ld.tag = reader.i16();
        let v1 = state.p_setup.vertex(ld.v1);
        let v2 = state.p_setup.vertex(ld.v2);
        ld.dx = v2.x - v1.x;
        ld.dy = v2.y - v1.y;
        if ld.dx == 0 {
            ld.slopetype = SlopeType::ST_VERTICAL;
        } else if ld.dy == 0 {
            ld.slopetype = SlopeType::ST_HORIZONTAL;
        } else if FixedDiv(ld.dy, ld.dx) > 0_i32 {
            ld.slopetype = SlopeType::ST_POSITIVE;
        } else {
            ld.slopetype = SlopeType::ST_NEGATIVE;
        }
        if v1.x < v2.x {
            ld.bbox[BoxIndex::Left as usize] = v1.x;
            ld.bbox[BoxIndex::Right as usize] = v2.x;
        } else {
            ld.bbox[BoxIndex::Left as usize] = v2.x;
            ld.bbox[BoxIndex::Right as usize] = v1.x;
        }
        if v1.y < v2.y {
            ld.bbox[BoxIndex::Bottom as usize] = v1.y;
            ld.bbox[BoxIndex::Top as usize] = v2.y;
        } else {
            ld.bbox[BoxIndex::Bottom as usize] = v2.y;
            ld.bbox[BoxIndex::Top as usize] = v1.y;
        }
        ld.sidenum[0] = reader.i16();
        ld.sidenum[1] = reader.i16();
        if ld.sidenum[0] as i32 != -1_i32 {
            ld.frontsector = Some(state.p_setup.sides[ld.sidenum[0] as usize].sector);
        } else {
            ld.frontsector = None;
        }
        if ld.sidenum[1] as i32 != -1_i32 {
            ld.backsector = Some(state.p_setup.sides[ld.sidenum[1] as usize].sector);
        } else {
            ld.backsector = None;
        }
        state.p_setup.lines[i] = ld;
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub fn P_LoadSideDefs(state: &mut GameState, lump: i32) {
    let numsides = (W_LumpLength(&mut state.w_wad, lump as u32) as usize / MAPSIDEDEF_SIZE) as i32;
    state.p_setup.numsides = numsides;
    state.p_setup.sides = Vec::with_capacity(numsides as usize);
    let mut reader = LumpReader::new(state, lump);
    for _ in 0..numsides {
        let textureoffset = reader.i16();
        let rowoffset = reader.i16();
        let toptexture = reader.name8();
        let bottomtexture = reader.name8();
        let midtexture = reader.name8();
        let sector = reader.i16();
        let sd = side_t {
            textureoffset: ((textureoffset as i32) << FRACBITS) as fixed_t,
            rowoffset: ((rowoffset as i32) << FRACBITS) as fixed_t,
            toptexture: R_TextureNumForName(&mut state.r_data, &toptexture.as_str()) as i16,
            bottomtexture: R_TextureNumForName(&mut state.r_data, &bottomtexture.as_str()) as i16,
            midtexture: R_TextureNumForName(&mut state.r_data, &midtexture.as_str()) as i16,
            sector: SectorId(sector as u32),
        };
        state.p_setup.sides.push(sd);
    }
    W_ReleaseLumpNum(&mut state.w_wad, lump);
}
pub fn P_LoadBlockMap(state: &mut GameState, lump: i32) {
    let lumplen: i32 = W_LumpLength(&mut state.w_wad, lump as u32);
    let mut raw = vec![0u8; lumplen as usize];
    W_ReadLump(&mut state.w_wad, &*state.fs, lump as u32, &mut raw);
    state.p_setup.blockmaplump = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect();
    state.p_setup.bmaporgx = ((state.p_setup.blockmaplump[0] as i32) << FRACBITS) as fixed_t;
    state.p_setup.bmaporgy = ((state.p_setup.blockmaplump[1] as i32) << FRACBITS) as fixed_t;
    state.p_setup.bmapwidth = state.p_setup.blockmaplump[2] as i32;
    state.p_setup.bmapheight = state.p_setup.blockmaplump[3] as i32;
    state.p_setup.blocklinks =
        vec![None; (state.p_setup.bmapwidth as usize) * (state.p_setup.bmapheight as usize)];
}
pub fn P_GroupLines(state: &mut GameState) {
    let mut i: i32;
    let mut j: i32;
    let mut bbox: [fixed_t; 4] = [0; 4];
    let mut block: i32;
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
        if let Some(back_id) = li.backsector.filter(|&b| Some(b) != li.frontsector) {
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
            let sector = state.p_setup.sector_mut(front_id);
            sector.lines.push(li_id);
            sector.linecount += 1;
        }
        if let Some(back_id) = li.backsector {
            if li.frontsector != li.backsector {
                let sector = state.p_setup.sector_mut(back_id);
                sector.lines.push(li_id);
                sector.linecount += 1;
            }
        }
        i += 1;
    }
    i = 0_i32;
    while i < state.p_setup.numsectors {
        M_ClearBox(&mut bbox);
        j = 0_i32;
        while j < state.p_setup.sectors[i as usize].linecount {
            let li_id = state.p_setup.sectors[i as usize].lines[j as usize];
            let li = state.p_setup.line(li_id);
            let li_v1 = state.p_setup.vertexes[li.v1.0 as usize];
            let li_v2 = state.p_setup.vertexes[li.v2.0 as usize];
            M_AddToBox(&mut bbox, li_v1.x, li_v1.y);
            M_AddToBox(&mut bbox, li_v2.x, li_v2.y);
            j += 1;
        }
        let sector = &mut state.p_setup.sectors[i as usize];
        sector.soundorg.x =
            ((bbox[BoxIndex::Right as usize] + bbox[BoxIndex::Left as usize]) / 2_i32) as fixed_t;
        sector.soundorg.y =
            ((bbox[BoxIndex::Top as usize] + bbox[BoxIndex::Bottom as usize]) / 2_i32) as fixed_t;
        block = (bbox[BoxIndex::Top as usize] - state.p_setup.bmaporgy + 32_i32 * FRACUNIT)
            >> MAPBLOCKSHIFT;
        block = if block >= state.p_setup.bmapheight {
            state.p_setup.bmapheight - 1_i32
        } else {
            block
        };
        sector.blockbox[BoxIndex::Top as usize] = block;
        block = (bbox[BoxIndex::Bottom as usize] - state.p_setup.bmaporgy - 32_i32 * FRACUNIT)
            >> MAPBLOCKSHIFT;
        block = if block < 0_i32 { 0_i32 } else { block };
        sector.blockbox[BoxIndex::Bottom as usize] = block;
        block = (bbox[BoxIndex::Right as usize] - state.p_setup.bmaporgx + 32_i32 * FRACUNIT)
            >> MAPBLOCKSHIFT;
        block = if block >= state.p_setup.bmapwidth {
            state.p_setup.bmapwidth - 1_i32
        } else {
            block
        };
        sector.blockbox[BoxIndex::Right as usize] = block;
        block = (bbox[BoxIndex::Left as usize] - state.p_setup.bmaporgx - 32_i32 * FRACUNIT)
            >> MAPBLOCKSHIFT;
        block = if block < 0_i32 { 0_i32 } else { block };
        sector.blockbox[BoxIndex::Left as usize] = block;
        i += 1;
    }
}
fn PadRejectArray(state: &mut GameState, offset: usize, len: u32) {
    let rejectpad: [u32; 4] = [
        (((state.p_setup.totallines * 4_i32 + 3_i32) & !3_i32) + 24_i32) as u32,
        0_i32 as u32,
        50_i32 as u32,
        0x1d4a11_i32 as u32,
    ];
    let pad_bytes = ::core::mem::size_of::<[u32; 4]>();
    let mut padvalue: u8 = 0;
    if len as usize > pad_bytes {
        eprintln!(
            "PadRejectArray: REJECT lump too short to pad! ({} > {})",
            len, pad_bytes as i32,
        );
        padvalue = if M_CheckParm(state, "-reject_pad_with_ff") != 0 {
            0xff
        } else {
            0xf00_u32 as u8
        };
    }
    let array = &mut state.p_setup.rejectmatrix[offset..offset + len as usize];
    for (i, dest) in array.iter_mut().enumerate().take(pad_bytes) {
        *dest = (rejectpad[i / 4] >> ((i % 4) as u32 * 8) & 0xff_u32) as byte;
    }
    if len as usize > pad_bytes {
        array[pad_bytes..].fill(padvalue);
    }
}
fn P_LoadReject(state: &mut GameState, lumpnum: i32) {
    let minlength = (state.p_setup.numsectors * state.p_setup.numsectors + 7_i32) / 8_i32;
    let lumplen = W_LumpLength(&mut state.w_wad, lumpnum as u32);
    if lumplen >= minlength {
        state.p_setup.rejectmatrix = W_LumpBytes(state, lumpnum)[..minlength as usize].to_vec();
    } else {
        state.p_setup.rejectmatrix = vec![0u8; minlength as usize];
        W_ReadLump(
            &mut state.w_wad,
            &*state.fs,
            lumpnum as u32,
            &mut state.p_setup.rejectmatrix,
        );
        PadRejectArray(state, lumplen as usize, (minlength - lumplen) as u32);
    };
}
pub fn P_SetupLevel(state: &mut GameState, episode: i32, map: i32) {
    let mut i: i32;

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
    let lumpnum: i32 = W_GetNumForName(&mut state.w_wad, &lumpname);
    state.p_tick.leveltime = 0_i32;
    P_LoadBlockMap(state, lumpnum + MapLump::ML_BLOCKMAP as i32);
    P_LoadVertexes(state, lumpnum + MapLump::ML_VERTEXES as i32);
    P_LoadSectors(state, lumpnum + MapLump::ML_SECTORS as i32);
    P_LoadSideDefs(state, lumpnum + MapLump::ML_SIDEDEFS as i32);
    P_LoadLineDefs(state, lumpnum + MapLump::ML_LINEDEFS as i32);
    P_LoadSubsectors(state, lumpnum + MapLump::ML_SSECTORS as i32);
    P_LoadNodes(state, lumpnum + MapLump::ML_NODES as i32);
    P_LoadSegs(state, lumpnum + MapLump::ML_SEGS as i32);
    P_GroupLines(state);
    P_LoadReject(state, lumpnum + MapLump::ML_REJECT as i32);
    state.g_game.bodyqueslot = 0_i32;
    state.p_setup.deathmatch_p = 0;
    P_LoadThings(state, lumpnum + MapLump::ML_THINGS as i32);
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
