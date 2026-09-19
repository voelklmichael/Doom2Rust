use crate::d_mode::GameMode;
use crate::doomdef::MAXPLAYERS;
use crate::fixed_cstr::FixedCStr;
use crate::g_game::death_match_spawn_player;
use crate::game_state::GameState;
use crate::i_system::get_memory_value;
use crate::i_system::ISystemState;
use crate::m_argv::parm_exists;
use crate::m_argv::MArgvState;
use crate::m_bbox::add_to_box;
use crate::m_bbox::clear_box;
use crate::m_bbox::BoxIndex;
use crate::m_fixed::fixed_div;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::p_maputl::MAPBLOCKSHIFT;
use crate::p_mobj::spawn_map_thing;
use crate::p_mobj::LineFlags;
use crate::p_mobj::{
    DegenMobj, Line, MapThing, MobjId, Sector, SlopeType, Subsector, Thinker, ThinkerFn, Vertex,
};
use crate::p_spec::init_pic_anims;
use crate::p_spec::spawn_specials;

use crate::p_switch::init_switch_list;
use crate::p_tick::init_thinkers;
use crate::r_data::flat_num_for_name;
use crate::r_data::precache_level;
use crate::r_data::texture_num_for_name;
use crate::r_defs::{Node, Seg, Side};
use crate::r_things::init_sprites;
use crate::s_sound::s_start;
use crate::tables::Angle;
use crate::w_wad::get_num_for_name;
use crate::w_wad::lump_bytes;
use crate::w_wad::lump_length;
use crate::w_wad::read_lump;
use crate::w_wad::release_lump_num;
use alloc::vec::Vec;

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

pub const ZERO_LINE: Line = Line {
    v1: VertexId(0),
    v2: VertexId(0),
    dx: 0,
    dy: 0,
    flags: LineFlags::empty(),
    special: 0,
    tag: 0,
    sidenum: [0; 2],
    bbox: [0; 4],
    slopetype: SlopeType::Horizontal,
    frontsector: None,
    backsector: None,
    validcount: 0,
};

pub const ZERO_SECTOR: Sector = Sector {
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
    soundorg: DegenMobj {
        thinker: Thinker {
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
    pub vertexes: Vec<Vertex>,
    pub numsegs: i32,
    pub segs: Vec<Seg>,
    pub numsectors: i32,
    pub sectors: Vec<Sector>,
    pub numsubsectors: i32,
    pub subsectors: Vec<Subsector>,
    pub numnodes: i32,
    pub nodes: Vec<Node>,
    pub numlines: i32,
    pub lines: Vec<Line>,
    pub numsides: i32,
    pub sides: Vec<Side>,
    pub totallines: i32,
    pub bmapwidth: i32,
    pub bmapheight: i32,
    pub blockmaplump: Vec<i16>,
    pub bmaporgx: Fixed,
    pub bmaporgy: Fixed,
    pub blocklinks: Vec<Option<MobjId>>,
    pub rejectmatrix: Vec<u8>,
    pub deathmatchstarts: [MapThing; 10],
    pub deathmatch_p: usize,
    pub playerstarts: [MapThing; 4],
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
        Self {
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
            deathmatchstarts: [MapThing {
                x: 0,
                y: 0,
                angle: 0,
                kind: 0,
                options: 0,
            }; 10],
            deathmatch_p: 0,
            playerstarts: [MapThing {
                x: 0,
                y: 0,
                angle: 0,
                kind: 0,
                options: 0,
            }; 4],
            null_sector_id: None,
            junk_line_id: None,
        }
    }

    pub fn sector_mut(&mut self, id: SectorId) -> &mut Sector {
        &mut self.sectors[id.0 as usize]
    }
    pub fn side_mut(&mut self, id: SideId) -> &mut Side {
        &mut self.sides[id.0 as usize]
    }
    pub fn subsector(&self, id: SubsectorId) -> Subsector {
        self.subsectors[id.0 as usize]
    }
    pub fn vertex(&self, id: VertexId) -> Vertex {
        self.vertexes[id.0 as usize]
    }
    pub fn line_mut(&mut self, id: LineId) -> &mut Line {
        &mut self.lines[id.0 as usize]
    }
    pub fn line(&self, id: LineId) -> Line {
        self.lines[id.0 as usize]
    }
    /// Returns a `LineId` for a scratch line that isn't part of the map,
    /// with only `.tag` set -- for vanilla Doom's "tag 666/667 boss death
    /// trigger" idiom, which calls do_door/do_floor with a fabricated
    /// line that exists only to carry a tag for find_sector_from_line_tag to
    /// match against. One slot is reused across all such calls; they never
    /// overlap (each EV_* call fully finishes before the next one reuses it).
    pub fn junk_line(&mut self, tag: i16) -> LineId {
        let id = if let Some(id) = self.junk_line_id {
            id
        } else {
            let id = LineId(self.lines.len() as u32);
            self.lines.push(ZERO_LINE);
            self.junk_line_id = Some(id);
            id
        };
        self.lines[id.0 as usize].tag = tag;
        id
    }
    pub fn seg_mut(&mut self, id: SegId) -> &mut Seg {
        &mut self.segs[id.0 as usize]
    }
    pub fn seg(&self, id: SegId) -> Seg {
        self.segs[id.0 as usize]
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)] // mirrors a C index table; discriminants must stay
pub enum MapLump {
    Label = 0,
    Things = 1,
    Linedefs = 2,
    Sidedefs = 3,
    Vertexes = 4,
    Segs = 5,
    Ssectors = 6,
    Nodes = 7,
    Sectors = 8,
    Reject = 9,
    Blockmap = 10,
}
/// Bounds-checked little-endian reader over a map lump's raw bytes.
struct LumpReader {
    data: alloc::rc::Rc<[u8]>,
    pos: usize,
}
impl LumpReader {
    fn new(state: &mut GameState, lump: i32) -> Self {
        Self {
            data: lump_bytes(state, lump),
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
pub fn load_vertexes(state: &mut GameState, lump: i32) {
    let numvertexes = (lump_length(&state.w_wad, lump as u32) as usize / MAPVERTEX_SIZE) as i32;
    state.p_setup.numvertexes = numvertexes;
    state.p_setup.vertexes = Vec::with_capacity(numvertexes as usize);
    let mut reader = LumpReader::new(state, lump);
    for _ in 0..numvertexes {
        let x = reader.i16() as i32;
        let y = reader.i16() as i32;
        state.p_setup.vertexes.push(Vertex {
            x: (x << FRACBITS) as Fixed,
            y: (y << FRACBITS) as Fixed,
        });
    }
    release_lump_num(&state.w_wad, lump);
}
pub fn get_sector_at_null_address(
    i_system: &mut ISystemState,
    m_argv: &MArgvState,
    p_setup: &mut PSetupState,
) -> SectorId {
    if p_setup.null_sector_id.is_none() {
        let mut sentinel = ZERO_SECTOR;
        if let Some(value) = get_memory_value(i_system, m_argv, 0, 4) {
            sentinel.floorheight = value as i32;
        }
        if let Some(value) = get_memory_value(i_system, m_argv, 4, 4) {
            sentinel.ceilingheight = value as i32;
        }
        let id = SectorId(p_setup.sectors.len() as u32);
        p_setup.sectors.push(sentinel);
        p_setup.null_sector_id = Some(id);
    }
    p_setup.null_sector_id.unwrap()
}
pub fn load_segs(state: &mut GameState, lump: i32) {
    let numsegs = (lump_length(&state.w_wad, lump as u32) as usize / MAPSEG_SIZE) as i32;
    state.p_setup.numsegs = numsegs;
    state.p_setup.segs = Vec::with_capacity(numsegs as usize);
    let mut reader = LumpReader::new(state, lump);
    for _ in 0..numsegs {
        let v1 = VertexId(reader.i16() as u32);
        let v2 = VertexId(reader.i16() as u32);
        let seg_angle = ((reader.i16() as i32) << 16) as Angle;
        let linedef = reader.i16() as i32;
        let side = reader.i16() as i32;
        let seg_offset = ((reader.i16() as i32) << 16) as Fixed;
        let seg_linedef = LineId(linedef as u32);
        let ldef = state.p_setup.line(seg_linedef);
        let seg_sidenum = ldef.sidenum[side as usize] as u32;
        let frontsector = Some(state.p_setup.sides[seg_sidenum as usize].sector);
        let backsector = if ldef.flags.contains(LineFlags::TWOSIDED) {
            let sidenum = ldef.sidenum[(side ^ 1) as usize] as i32;
            if sidenum < 0 || sidenum >= state.p_setup.numsides {
                Some(get_sector_at_null_address(
                    &mut state.i_system,
                    &state.m_argv,
                    &mut state.p_setup,
                ))
            } else {
                Some(state.p_setup.sides[sidenum as usize].sector)
            }
        } else {
            None
        };
        state.p_setup.segs.push(Seg {
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
    release_lump_num(&state.w_wad, lump);
}
pub fn load_subsectors(state: &mut GameState, lump: i32) {
    let numsubsectors =
        (lump_length(&state.w_wad, lump as u32) as usize / MAPSUBSECTOR_SIZE) as i32;
    state.p_setup.numsubsectors = numsubsectors;
    state.p_setup.subsectors = Vec::with_capacity(numsubsectors as usize);
    let mut reader = LumpReader::new(state, lump);
    for _ in 0..numsubsectors {
        let numsegs = reader.i16();
        let firstseg = reader.i16();
        state.p_setup.subsectors.push(Subsector {
            sector: SectorId(0),
            numlines: numsegs,
            firstline: firstseg,
        });
    }
    release_lump_num(&state.w_wad, lump);
}
pub fn load_sectors(state: &mut GameState, lump: i32) {
    let numsectors = (lump_length(&state.w_wad, lump as u32) as usize / MAPSECTOR_SIZE) as i32;
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
        let floorpic =
            flat_num_for_name(&state.r_data, &state.w_wad, &floorpic_name.as_str()) as i16;
        let ceilingpic =
            flat_num_for_name(&state.r_data, &state.w_wad, &ceilingpic_name.as_str()) as i16;
        let ss = &mut state.p_setup.sectors[i];
        ss.floorheight = ((floorheight as i32) << FRACBITS) as Fixed;
        ss.ceilingheight = ((ceilingheight as i32) << FRACBITS) as Fixed;
        ss.floorpic = floorpic;
        ss.ceilingpic = ceilingpic;
        ss.lightlevel = lightlevel;
        ss.special = special;
        ss.tag = tag;
        ss.thinglist = None;
    }
    release_lump_num(&state.w_wad, lump);
}
pub fn load_nodes(state: &mut GameState, lump: i32) {
    state.p_setup.numnodes =
        (lump_length(&state.w_wad, lump as u32) as usize / MAPNODE_SIZE) as i32;
    state.p_setup.nodes = Vec::with_capacity(state.p_setup.numnodes as usize);
    let mut reader = LumpReader::new(state, lump);
    for _ in 0..state.p_setup.numnodes {
        let mut no = Node {
            x: ((reader.i16() as i32) << FRACBITS) as Fixed,
            y: ((reader.i16() as i32) << FRACBITS) as Fixed,
            dx: ((reader.i16() as i32) << FRACBITS) as Fixed,
            dy: ((reader.i16() as i32) << FRACBITS) as Fixed,
            bbox: [[0; 4]; 2],
            children: [0; 2],
        };
        for j in 0..2 {
            for k in 0..4 {
                no.bbox[j][k] = ((reader.i16() as i32) << FRACBITS) as Fixed;
            }
        }
        for j in 0..2 {
            no.children[j] = reader.u16();
        }
        state.p_setup.nodes.push(no);
    }
    release_lump_num(&state.w_wad, lump);
}
pub fn load_things(state: &mut GameState, lump: i32) {
    let numthings = (lump_length(&state.w_wad, lump as u32) as usize / MAPTHING_SIZE) as i32;
    let mut reader = LumpReader::new(state, lump);
    for _ in 0..numthings {
        let spawnthing = MapThing {
            x: reader.i16(),
            y: reader.i16(),
            angle: reader.i16(),
            kind: reader.i16(),
            options: reader.i16(),
        };
        let spawn = !(state.doomstat.gamemode as u32 != GameMode::Commercial as i32 as u32
            && matches!(
                spawnthing.kind,
                64 | 88 | 89 | 69 | 67 | 71 | 65 | 66 | 68 | 84
            ));
        if !spawn {
            break;
        }
        spawn_map_thing(state, spawnthing);
    }
    release_lump_num(&state.w_wad, lump);
}
pub fn load_line_defs(state: &mut GameState, lump: i32) {
    state.p_setup.numlines =
        (lump_length(&state.w_wad, lump as u32) as usize / MAPLINEDEF_SIZE) as i32;
    state.p_setup.lines = vec![ZERO_LINE; state.p_setup.numlines as usize];
    let mut reader = LumpReader::new(state, lump);
    for i in 0..state.p_setup.numlines as usize {
        let mut ld = state.p_setup.lines[i];
        ld.v1 = VertexId(reader.i16() as u32);
        ld.v2 = VertexId(reader.i16() as u32);
        ld.flags = LineFlags::from_bits_retain(reader.i16());
        ld.special = reader.i16();
        ld.tag = reader.i16();
        let v1 = state.p_setup.vertex(ld.v1);
        let v2 = state.p_setup.vertex(ld.v2);
        ld.dx = v2.x - v1.x;
        ld.dy = v2.y - v1.y;
        if ld.dx == 0 {
            ld.slopetype = SlopeType::Vertical;
        } else if ld.dy == 0 {
            ld.slopetype = SlopeType::Horizontal;
        } else if fixed_div(ld.dy, ld.dx) > 0 {
            ld.slopetype = SlopeType::Positive;
        } else {
            ld.slopetype = SlopeType::Negative;
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
        if ld.sidenum[0] as i32 == -1 {
            ld.frontsector = None;
        } else {
            ld.frontsector = Some(state.p_setup.sides[ld.sidenum[0] as usize].sector);
        }
        if ld.sidenum[1] as i32 == -1 {
            ld.backsector = None;
        } else {
            ld.backsector = Some(state.p_setup.sides[ld.sidenum[1] as usize].sector);
        }
        state.p_setup.lines[i] = ld;
    }
    release_lump_num(&state.w_wad, lump);
}
pub fn load_side_defs(state: &mut GameState, lump: i32) {
    let numsides = (lump_length(&state.w_wad, lump as u32) as usize / MAPSIDEDEF_SIZE) as i32;
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
        let sd = Side {
            textureoffset: ((textureoffset as i32) << FRACBITS) as Fixed,
            rowoffset: ((rowoffset as i32) << FRACBITS) as Fixed,
            toptexture: texture_num_for_name(&state.r_data, &toptexture.as_str()) as i16,
            bottomtexture: texture_num_for_name(&state.r_data, &bottomtexture.as_str()) as i16,
            midtexture: texture_num_for_name(&state.r_data, &midtexture.as_str()) as i16,
            sector: SectorId(sector as u32),
        };
        state.p_setup.sides.push(sd);
    }
    release_lump_num(&state.w_wad, lump);
}
pub fn load_block_map(state: &mut GameState, lump: i32) {
    let lumplen: i32 = lump_length(&state.w_wad, lump as u32);
    let mut raw = vec![0u8; lumplen as usize];
    read_lump(&state.w_wad, &*state.fs, lump as u32, &mut raw);
    state.p_setup.blockmaplump = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect();
    state.p_setup.bmaporgx = ((state.p_setup.blockmaplump[0] as i32) << FRACBITS) as Fixed;
    state.p_setup.bmaporgy = ((state.p_setup.blockmaplump[1] as i32) << FRACBITS) as Fixed;
    state.p_setup.bmapwidth = state.p_setup.blockmaplump[2] as i32;
    state.p_setup.bmapheight = state.p_setup.blockmaplump[3] as i32;
    state.p_setup.blocklinks =
        vec![None; (state.p_setup.bmapwidth as usize) * (state.p_setup.bmapheight as usize)];
}
pub fn group_lines(p_setup: &mut PSetupState) {
    let mut bbox: [Fixed; 4] = [0; 4];
    let mut block: i32;
    for i in 0..(p_setup.numsubsectors as usize) {
        let firstline = p_setup.subsectors[i].firstline;
        let seg_sidedef = p_setup.segs[firstline as usize].sidedef;
        p_setup.subsectors[i].sector = p_setup.sides[seg_sidedef.0 as usize].sector;
    }
    p_setup.totallines = 0;
    for i in 0..(p_setup.numlines as usize) {
        p_setup.totallines += 1;
        let li = p_setup.lines[i];
        let front_id = li.frontsector.unwrap();
        p_setup.sector_mut(front_id).linecount += 1;
        if let Some(back_id) = li.backsector.filter(|&b| Some(b) != li.frontsector) {
            p_setup.sector_mut(back_id).linecount += 1;
            p_setup.totallines += 1;
        }
    }
    for i in 0..(p_setup.numsectors as usize) {
        let sec = &mut p_setup.sectors[i];
        sec.lines = Vec::with_capacity(sec.linecount as usize);
        sec.linecount = 0;
    }
    for i in 0..p_setup.numlines {
        let li_id = LineId(i as u32);
        let li = p_setup.lines[i as usize];
        if let Some(front_id) = li.frontsector {
            let sector = p_setup.sector_mut(front_id);
            sector.lines.push(li_id);
            sector.linecount += 1;
        }
        if let Some(back_id) = li.backsector {
            if li.frontsector != li.backsector {
                let sector = p_setup.sector_mut(back_id);
                sector.lines.push(li_id);
                sector.linecount += 1;
            }
        }
    }
    for i in 0..(p_setup.numsectors as usize) {
        clear_box(&mut bbox);
        for j in 0..p_setup.sectors[i].linecount {
            let li_id = p_setup.sectors[i].lines[j as usize];
            let li = p_setup.line(li_id);
            let li_v1 = p_setup.vertexes[li.v1.0 as usize];
            let li_v2 = p_setup.vertexes[li.v2.0 as usize];
            add_to_box(&mut bbox, li_v1.x, li_v1.y);
            add_to_box(&mut bbox, li_v2.x, li_v2.y);
        }
        let sector = &mut p_setup.sectors[i];
        sector.soundorg.x =
            ((bbox[BoxIndex::Right as usize] + bbox[BoxIndex::Left as usize]) / 2) as Fixed;
        sector.soundorg.y =
            ((bbox[BoxIndex::Top as usize] + bbox[BoxIndex::Bottom as usize]) / 2) as Fixed;
        block = (bbox[BoxIndex::Top as usize] - p_setup.bmaporgy + 32 * FRACUNIT) >> MAPBLOCKSHIFT;
        block = if block >= p_setup.bmapheight {
            p_setup.bmapheight - 1
        } else {
            block
        };
        sector.blockbox[BoxIndex::Top as usize] = block;
        block =
            (bbox[BoxIndex::Bottom as usize] - p_setup.bmaporgy - 32 * FRACUNIT) >> MAPBLOCKSHIFT;
        block = if block < 0 { 0 } else { block };
        sector.blockbox[BoxIndex::Bottom as usize] = block;
        block =
            (bbox[BoxIndex::Right as usize] - p_setup.bmaporgx + 32 * FRACUNIT) >> MAPBLOCKSHIFT;
        block = if block >= p_setup.bmapwidth {
            p_setup.bmapwidth - 1
        } else {
            block
        };
        sector.blockbox[BoxIndex::Right as usize] = block;
        block = (bbox[BoxIndex::Left as usize] - p_setup.bmaporgx - 32 * FRACUNIT) >> MAPBLOCKSHIFT;
        block = if block < 0 { 0 } else { block };
        sector.blockbox[BoxIndex::Left as usize] = block;
    }
}
fn pad_reject_array(state: &mut GameState, offset: usize, len: u32) {
    let rejectpad: [u32; 4] = [
        (((state.p_setup.totallines * 4 + 3) & !3) + 24) as u32,
        0,
        50,
        0x1d4a11,
    ];
    let pad_bytes = ::core::mem::size_of::<[u32; 4]>();
    let mut padvalue: u8 = 0;
    if len as usize > pad_bytes {
        doom_eprintln!(
            state.platform,
            "PadRejectArray: REJECT lump too short to pad! ({} > {})",
            len,
            pad_bytes as i32,
        );
        padvalue = if parm_exists(&state.m_argv, "-reject_pad_with_ff") {
            0xff
        } else {
            // Upstream writes 0xf00 into a byte, which truncates to zero.
            0
        };
    }
    let array = &mut state.p_setup.rejectmatrix[offset..offset + len as usize];
    for (i, dest) in array.iter_mut().enumerate().take(pad_bytes) {
        *dest = (rejectpad[i / 4] >> ((i % 4) as u32 * 8) & 0xff) as u8;
    }
    if len as usize > pad_bytes {
        array[pad_bytes..].fill(padvalue);
    }
}
fn load_reject(state: &mut GameState, lumpnum: i32) {
    let minlength = (state.p_setup.numsectors * state.p_setup.numsectors + 7) / 8;
    let lumplen = lump_length(&state.w_wad, lumpnum as u32);
    if lumplen >= minlength {
        state.p_setup.rejectmatrix = lump_bytes(state, lumpnum)[..minlength as usize].to_vec();
    } else {
        state.p_setup.rejectmatrix = vec![0u8; minlength as usize];
        read_lump(
            &state.w_wad,
            &*state.fs,
            lumpnum as u32,
            &mut state.p_setup.rejectmatrix,
        );
        pad_reject_array(state, lumplen as usize, (minlength - lumplen) as u32);
    }
}
pub fn setup_level(state: &mut GameState, episode: i32, map: i32) {
    state.g_game.wminfo.maxfrags = 0;
    state.g_game.totalsecret = state.g_game.wminfo.maxfrags;
    state.g_game.totalitems = state.g_game.totalsecret;
    state.g_game.totalkills = state.g_game.totalitems;
    state.g_game.wminfo.partime = 180;
    for i in 0..(MAXPLAYERS as usize) {
        state.g_game.players[i].itemcount = 0;
        state.g_game.players[i].secretcount = state.g_game.players[i].itemcount;
        state.g_game.players[i].killcount = state.g_game.players[i].secretcount;
    }
    state.g_game.players[state.g_game.consoleplayer as usize].viewz = 1;
    s_start(state);
    init_thinkers(&mut state.p_tick);
    let lumpname = if state.doomstat.gamemode as u32 == GameMode::Commercial as i32 as u32 {
        if map < 10 {
            format!("map0{map}")
        } else {
            format!("map{map}")
        }
    } else {
        format!(
            "E{}M{}",
            char::from((('0' as i32) + episode) as u8),
            char::from((('0' as i32) + map) as u8)
        )
    };
    let lumpnum: i32 = get_num_for_name(&state.w_wad, &lumpname);
    state.p_tick.leveltime = 0;
    load_block_map(state, lumpnum + MapLump::Blockmap as i32);
    load_vertexes(state, lumpnum + MapLump::Vertexes as i32);
    load_sectors(state, lumpnum + MapLump::Sectors as i32);
    load_side_defs(state, lumpnum + MapLump::Sidedefs as i32);
    load_line_defs(state, lumpnum + MapLump::Linedefs as i32);
    load_subsectors(state, lumpnum + MapLump::Ssectors as i32);
    load_nodes(state, lumpnum + MapLump::Nodes as i32);
    load_segs(state, lumpnum + MapLump::Segs as i32);
    group_lines(&mut state.p_setup);
    load_reject(state, lumpnum + MapLump::Reject as i32);
    state.g_game.bodyqueslot = 0;
    state.p_setup.deathmatch_p = 0;
    load_things(state, lumpnum + MapLump::Things as i32);
    if state.g_game.deathmatch != 0 {
        for i in 0..MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                state.g_game.players[i as usize].mo = None;
                death_match_spawn_player(state, i);
            }
        }
    }
    let gs = state;
    gs.p_mobj.iquetail = 0;
    gs.p_mobj.iquehead = gs.p_mobj.iquetail;
    spawn_specials(gs);
    if gs.g_game.precache {
        precache_level(gs);
    }
}
pub fn p_init(state: &mut GameState) {
    init_switch_list(&state.doomstat, &mut state.p_switch, &state.r_data);
    init_pic_anims(&mut state.p_spec, &state.r_data, &state.w_wad);
    let sprnames = state.info.sprnames;
    init_sprites(state, &sprnames);
}
