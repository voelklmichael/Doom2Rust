use crate::game_state::GameState;
use crate::m_fixed::fixed_t;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_setup::SegId;
use crate::p_setup::SideId;
use crate::p_setup::VertexId;
use crate::stdint_types::byte;
use crate::tables::angle_t;
pub type lighttable_t = byte;

// A sprite/wall vertical-clip array, always one of these fixed i16 arrays --
// never an independently-allocated buffer. `Openings(i)` is an index into
// r_plane's openings scratch array (i can be used as a base that, added to
// an in-range screen x, is always non-negative even though the base itself
// may briefly go negative, mirroring the original `lastopening.offset(-start)`
// pointer arithmetic).
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ClipArray {
    Openings(isize),
    ScreenHeightArray,
    NegOneArray,
    ClipBot,
    ClipTop,
}

impl ClipArray {
    pub fn get(self, state: &GameState, x: isize) -> i16 {
        match self {
            ClipArray::Openings(offset) => state.r_plane.openings[(offset + x) as usize],
            ClipArray::ScreenHeightArray => state.r_things.screenheightarray[x as usize],
            ClipArray::NegOneArray => state.r_things.negonearray[x as usize],
            ClipArray::ClipBot => state.r_things.clipbot[x as usize],
            ClipArray::ClipTop => state.r_things.cliptop[x as usize],
        }
    }

    pub fn set(self, state: &mut GameState, x: isize, value: i16) {
        match self {
            ClipArray::Openings(offset) => state.r_plane.openings[(offset + x) as usize] = value,
            ClipArray::ScreenHeightArray => state.r_things.screenheightarray[x as usize] = value,
            ClipArray::NegOneArray => state.r_things.negonearray[x as usize] = value,
            ClipArray::ClipBot => state.r_things.clipbot[x as usize] = value,
            ClipArray::ClipTop => state.r_things.cliptop[x as usize] = value,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct side_t {
    pub textureoffset: fixed_t,
    pub rowoffset: fixed_t,
    pub toptexture: i16,
    pub bottomtexture: i16,
    pub midtexture: i16,
    pub sector: SectorId,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct seg_t {
    pub v1: VertexId,
    pub v2: VertexId,
    pub offset: fixed_t,
    pub angle: angle_t,
    pub sidedef: SideId,
    pub linedef: LineId,
    pub frontsector: Option<SectorId>,
    pub backsector: Option<SectorId>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct node_t {
    pub x: fixed_t,
    pub y: fixed_t,
    pub dx: fixed_t,
    pub dy: fixed_t,
    pub bbox: [[fixed_t; 4]; 2],
    pub children: [u16; 2],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct drawseg_s {
    pub curline: SegId,
    pub x1: i32,
    pub x2: i32,
    pub scale1: fixed_t,
    pub scale2: fixed_t,
    pub scalestep: fixed_t,
    pub silhouette: i32,
    pub bsilheight: fixed_t,
    pub tsilheight: fixed_t,
    pub sprtopclip: Option<ClipArray>,
    pub sprbottomclip: Option<ClipArray>,
    pub maskedtexturecol: Option<ClipArray>,
}
pub type drawseg_t = drawseg_s;

/// A visplane's per-column top/bottom rows. The arrays carry one extra
/// element at each end (index `x + 1` holds column `x`), because the span
/// builder deliberately reads/writes columns -1 and 320 (the sentinels around
/// the plane's real columns).
#[derive(Copy, Clone)]
pub struct visplane_t {
    pub height: fixed_t,
    pub picnum: i32,
    pub lightlevel: i32,
    pub minx: i32,
    pub maxx: i32,
    top: [byte; 322],
    bottom: [byte; 322],
}

impl visplane_t {
    pub const EMPTY: visplane_t = visplane_t {
        height: 0,
        picnum: 0,
        lightlevel: 0,
        minx: 0,
        maxx: 0,
        top: [0; 322],
        bottom: [0; 322],
    };

    pub fn top(&self, x: i32) -> byte {
        self.top[(x + 1) as usize]
    }

    pub fn set_top(&mut self, x: i32, value: byte) {
        self.top[(x + 1) as usize] = value;
    }

    pub fn bottom(&self, x: i32) -> byte {
        self.bottom[(x + 1) as usize]
    }

    pub fn set_bottom(&mut self, x: i32, value: byte) {
        self.bottom[(x + 1) as usize] = value;
    }

    /// Marks all 320 real columns as untouched (top == 0xff).
    pub fn clear_top(&mut self) {
        self.top[1..=320].fill(0xff);
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
pub enum SpriteRotate {
    Unset = -1,
    NonRotating = 0,
    Rotating = 1,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct spriteframe_t {
    pub rotate: SpriteRotate,
    pub lump: [i16; 8],
    pub flip: [byte; 8],
}

#[derive(Clone)]
pub struct spritedef_t {
    pub numframes: i32,
    pub spriteframes: Vec<spriteframe_t>,
}
