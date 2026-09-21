use crate::game_state::GameState;
use crate::m_bbox::BBox;
use crate::m_fixed::Fixed;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_setup::SegId;
use crate::p_setup::SideId;
use crate::p_setup::VertexId;
use crate::tables::Angle;
use crate::w_wad::LumpNum;
use alloc::vec::Vec;
pub type LightTable = u8;

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
            Self::Openings(offset) => state.render.r_plane.openings[(offset + x) as usize],
            Self::ScreenHeightArray => state.render.r_things.screenheightarray[x as usize],
            Self::NegOneArray => state.render.r_things.negonearray[x as usize],
            Self::ClipBot => state.render.r_things.clipbot[x as usize],
            Self::ClipTop => state.render.r_things.cliptop[x as usize],
        }
    }

    pub fn set(self, state: &mut GameState, x: isize, value: i16) {
        match self {
            Self::Openings(offset) => state.render.r_plane.openings[(offset + x) as usize] = value,
            Self::ScreenHeightArray => state.render.r_things.screenheightarray[x as usize] = value,
            Self::NegOneArray => state.render.r_things.negonearray[x as usize] = value,
            Self::ClipBot => state.render.r_things.clipbot[x as usize] = value,
            Self::ClipTop => state.render.r_things.cliptop[x as usize] = value,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Side {
    pub textureoffset: Fixed,
    pub rowoffset: Fixed,
    pub toptexture: i16,
    pub bottomtexture: i16,
    pub midtexture: i16,
    pub sector: SectorId,
}

#[derive(Copy, Clone)]
pub struct Seg {
    pub v1: VertexId,
    pub v2: VertexId,
    pub offset: Fixed,
    pub angle: Angle,
    pub sidedef: SideId,
    pub linedef: LineId,
    pub frontsector: Option<SectorId>,
    pub backsector: Option<SectorId>,
}

#[derive(Copy, Clone)]
pub struct Node {
    pub x: Fixed,
    pub y: Fixed,
    pub dx: Fixed,
    pub dy: Fixed,
    pub bbox: [BBox; 2],
    pub children: [u16; 2],
}

#[derive(Copy, Clone)]
pub struct DrawSeg {
    pub curline: SegId,
    pub x1: i32,
    pub x2: i32,
    pub scale1: Fixed,
    pub scale2: Fixed,
    pub scalestep: Fixed,
    pub silhouette: i32,
    pub bsilheight: Fixed,
    pub tsilheight: Fixed,
    pub sprtopclip: Option<ClipArray>,
    pub sprbottomclip: Option<ClipArray>,
    pub maskedtexturecol: Option<ClipArray>,
}

impl DrawSeg {
    /// The clip row below the sprites, present whenever the seg has a bottom silhouette.
    pub fn sprbottomclip(&self) -> ClipArray {
        self.sprbottomclip
            .expect("a bottom silhouette has a clip row")
    }
    /// The clip row above the sprites, present whenever the seg has a top silhouette.
    pub fn sprtopclip(&self) -> ClipArray {
        self.sprtopclip.expect("a top silhouette has a clip row")
    }
}

/// A visplane's per-column top/bottom rows. The arrays carry one extra
/// element at each end (index `x + 1` holds column `x`), because the span
/// builder deliberately reads/writes columns -1 and 320 (the sentinels around
/// the plane's real columns).
#[derive(Copy, Clone)]
pub struct VisPlane {
    pub height: Fixed,
    pub picnum: i32,
    pub lightlevel: i32,
    pub minx: i32,
    pub maxx: i32,
    top: [u8; 322],
    bottom: [u8; 322],
}

impl VisPlane {
    pub const EMPTY: Self = Self {
        height: Fixed::ZERO,
        picnum: 0,
        lightlevel: 0,
        minx: 0,
        maxx: 0,
        top: [0; 322],
        bottom: [0; 322],
    };

    pub fn top(&self, x: i32) -> u8 {
        self.top[(x + 1) as usize]
    }

    pub fn set_top(&mut self, x: i32, value: u8) {
        self.top[(x + 1) as usize] = value;
    }

    pub fn bottom(&self, x: i32) -> u8 {
        self.bottom[(x + 1) as usize]
    }

    pub fn set_bottom(&mut self, x: i32, value: u8) {
        self.bottom[(x + 1) as usize] = value;
    }

    /// Marks all 320 real columns as untouched (top == 0xff).
    pub fn clear_top(&mut self) {
        self.top[1..=320].fill(0xff);
    }
}

/// A sprite lump's position among the sprite lumps: its lump number minus `firstspritelump`.
/// It indexes `spritewidth`, `spriteoffset` and `spritetopoffset`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SpriteLump(pub u16);

impl SpriteLump {
    pub fn index(self) -> usize {
        usize::from(self.0)
    }

    /// The lump number, given the number of the first sprite lump.
    pub fn lump_num(self, firstspritelump: LumpNum) -> LumpNum {
        firstspritelump + i32::from(self.0)
    }
}

/// One picture of a sprite frame: a sprite lump, drawn mirrored or not.
#[derive(Copy, Clone)]
pub struct SpriteImage {
    pub lump: SpriteLump,
    pub flip: bool,
}

/// The pictures of one animation frame of a sprite, as `R_InitSprites` collects them from the
/// lump names (`TROOA1`, `TROOA2A8`, ...).
#[derive(Copy, Clone)]
pub enum SpriteFrame {
    /// No lump names this frame yet.
    Unset,
    /// One picture, whatever the viewing angle (rotation 0).
    NonRotating(SpriteImage),
    /// One picture per viewing angle (rotations 1-8); `None` until its lump has been seen.
    Rotating([Option<SpriteImage>; 8]),
}

impl SpriteFrame {
    pub fn is_rotating(&self) -> bool {
        matches!(self, Self::Rotating(_))
    }

    /// The picture seen from `rotation` (0-7, counted from the front; ignored by a frame
    /// without rotations).
    pub fn image(&self, rotation: usize) -> SpriteImage {
        match self {
            Self::NonRotating(image) => *image,
            Self::Rotating(images) => {
                images[rotation].expect("a sprite frame with rotations has all eight")
            }
            Self::Unset => panic!("sprite frame without pictures"),
        }
    }
}

#[derive(Clone)]
pub struct SpriteDef {
    pub numframes: i32,
    pub spriteframes: Vec<SpriteFrame>,
}
