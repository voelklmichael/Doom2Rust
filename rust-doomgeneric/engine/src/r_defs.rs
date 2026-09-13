use crate::src::m_fixed::fixed_t;
use crate::src::p_setup::LineId;
use crate::src::p_setup::SectorId;
use crate::src::p_setup::SegId;
use crate::src::p_setup::SideId;
use crate::src::p_setup::VertexId;
use crate::src::stdint_types::byte;
use crate::src::tables::angle_t;
pub type lighttable_t = byte;

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
    pub sprtopclip: *mut i16,
    pub sprbottomclip: *mut i16,
    pub maskedtexturecol: *mut i16,
}
pub type drawseg_t = drawseg_s;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct visplane_t {
    pub height: fixed_t,
    pub picnum: i32,
    pub lightlevel: i32,
    pub minx: i32,
    pub maxx: i32,
    pub pad1: byte,
    pub top: [byte; 320],
    pub pad2: byte,
    pub pad3: byte,
    pub bottom: [byte; 320],
    pub pad4: byte,
}

// #[repr(i32)] with these exact discriminant values is load-bearing, not
// just documentation: R_InitSpriteDefs bulk-initializes a whole array of
// spriteframe_t via a raw memset(..., -1, ...) rather than setting `rotate`
// field-by-field, relying on the resulting all-0xff bytes being a valid
// `SpriteRotate` bit pattern. That only holds because -1 is this enum's
// actual `Unset` discriminant under repr(i32) -- changing these values (or
// dropping the repr) would make that memset produce an invalid/UB enum
// value instead of `Unset`.
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
