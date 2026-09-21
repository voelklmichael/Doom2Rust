use crate::d_mode::GameMode;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::i_video::IVideoState;
use crate::index::ToIndex;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::patch::Patch;
use crate::r_data::RDataState;
use crate::v_video::cache_patch_name;
use crate::v_video::Screen;
use crate::v_video::VVideoState;
use crate::w_wad::LumpNum;
use crate::w_wad::WWadState;
use alloc::vec::Vec;

use crate::r_main::ColormapId;

use crate::v_video::draw_patch;
use crate::v_video::mark_rect;
use crate::w_wad::lump_bytes_name;

#[derive(Clone, Copy)]
pub enum ColumnSource {
    Lump { lump: LumpNum, offset: usize },
    Composite { tex: i32, offset: usize },
}

pub fn advance_source(src: ColumnSource, delta: usize) -> ColumnSource {
    match src {
        ColumnSource::Lump { lump, offset } => ColumnSource::Lump {
            lump,
            offset: offset.wrapping_add(delta),
        },
        ColumnSource::Composite { tex, offset } => ColumnSource::Composite {
            tex,
            offset: offset.wrapping_add(delta),
        },
    }
}

/// The bytes a [`ColumnSource`] reads from, and the offset in them at which the source starts.
/// The pixel loops resolve this once per column or span instead of once per pixel.
fn source_bytes<'a>(
    r_data: &'a RDataState,
    w_wad: &'a WWadState,
    src: ColumnSource,
) -> (&'a [u8], isize) {
    match src {
        ColumnSource::Lump { lump, offset } => (
            &w_wad.lumpinfo[lump.index()]
                .cache
                .as_ref()
                .expect("a lump is cached while it is drawn")[..],
            offset as isize,
        ),
        ColumnSource::Composite { tex, offset } => (
            r_data.texturecomposite[tex.idx()]
                .as_ref()
                .expect("a texture is composited while it is drawn"),
            offset as isize,
        ),
    }
}

/// One light level of the colormap table: what a source pixel becomes at that brightness.
fn colormap_row(colormaps: &[u8], index: ColormapId) -> &[u8; 256] {
    let start = (index * 256).idx();
    (&colormaps[start..start + 256])
        .try_into()
        .expect("the slice is 256 bytes")
}

/// One of the player colour translation tables, `dc_translation` bytes into `translationtables`.
fn translation_row(tables: &[u8], offset: usize) -> &[u8; 256] {
    (&tables[offset..offset + 256])
        .try_into()
        .expect("the slice is 256 bytes")
}

pub fn read_source(r_data: &RDataState, w_wad: &WWadState, src: ColumnSource, idx: i32) -> u8 {
    let (bytes, base) = source_bytes(r_data, w_wad, src);
    bytes[(base + idx as isize).idx()]
}

pub struct RDrawState {
    pub viewwidth: i32,
    pub scaledviewwidth: i32,
    pub viewheight: i32,
    pub viewwindowx: i32,
    pub viewwindowy: i32,
    pub ylookup: [usize; 832],
    pub columnofs: [i32; 1120],
    pub background_buffer: Option<Vec<u8>>,
    pub dc_colormap: Option<ColormapId>,
    pub dc_x: i32,
    pub dc_yl: i32,
    pub dc_yh: i32,
    pub dc_iscale: Fixed,
    pub dc_texturemid: Fixed,
    pub dc_source: Option<ColumnSource>,
    pub dccount: i32,
    pub fuzzpos: i32,
    pub dc_translation: usize,
    pub translationtables: Vec<u8>,
    pub ds_y: i32,
    pub ds_x1: i32,
    pub ds_x2: i32,
    pub ds_colormap: ColormapId,
    pub ds_xfrac: Fixed,
    pub ds_yfrac: Fixed,
    pub ds_xstep: Fixed,
    pub ds_ystep: Fixed,
    pub ds_source: Option<ColumnSource>,
    pub dscount: i32,
}

impl RDrawState {
    /// Where the column being drawn reads its texels (set by every caller before drawing).
    fn column_source(&self) -> ColumnSource {
        self.dc_source.expect("no column source set")
    }
    /// The light level of the column being drawn (set by every caller before drawing).
    fn column_colormap(&self) -> ColormapId {
        self.dc_colormap.expect("no column colormap set")
    }
    /// Where the span being drawn reads its texels (set by every caller before drawing).
    fn span_source(&self) -> ColumnSource {
        self.ds_source.expect("no span source set")
    }
}

impl Default for RDrawState {
    fn default() -> Self {
        Self {
            viewwidth: 0,
            scaledviewwidth: 0,
            viewheight: 0,
            viewwindowx: 0,
            viewwindowy: 0,
            ylookup: [0; 832],
            columnofs: [0; 1120],
            background_buffer: None,
            dc_colormap: None,
            dc_x: 0,
            dc_yl: 0,
            dc_yh: 0,
            dc_iscale: Fixed::ZERO,
            dc_texturemid: Fixed::ZERO,
            dc_source: None,
            dccount: 0,
            fuzzpos: 0,
            dc_translation: 0,
            translationtables: Vec::new(),
            ds_y: 0,
            ds_x1: 0,
            ds_x2: 0,
            ds_colormap: 0,
            ds_xfrac: Fixed::ZERO,
            ds_yfrac: Fixed::ZERO,
            ds_xstep: Fixed::ZERO,
            ds_ystep: Fixed::ZERO,
            ds_source: None,
            dscount: 0,
        }
    }
}

pub const SBARHEIGHT: i32 = 32;
pub fn draw_column(state: &mut GameState) {
    let count: i32 = state.render.r_draw.dc_yh - state.render.r_draw.dc_yl;
    if count < 0 {
        return;
    }
    if state.render.r_draw.dc_x.cast_unsigned() >= SCREENWIDTH as u32
        || state.render.r_draw.dc_yl < 0
        || state.render.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.render.r_draw.dc_yl, state.render.r_draw.dc_yh, state.render.r_draw.dc_x
        ));
    }
    let mut idx: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl.idx()]
        + state.render.r_draw.columnofs[state.render.r_draw.dc_x.idx()].idx();
    let fracstep: Fixed = state.render.r_draw.dc_iscale;
    let mut frac: Fixed = state.render.r_draw.dc_texturemid
        + (state.render.r_draw.dc_yl - state.render.r_main.centery) * fracstep;
    let (source, base) = source_bytes(
        &state.render.r_data,
        &state.assets.w_wad,
        state.render.r_draw.column_source(),
    );
    let colormap = colormap_row(
        &state.render.r_data.colormaps,
        state.render.r_draw.column_colormap(),
    );
    let screen = &mut state.io.i_video.i_video_buffer[..];
    for _ in 0..=count {
        let src_pixel = source[(base + (frac >> FRACBITS & 127).to_bits() as isize).idx()];
        screen[idx] = colormap[usize::from(src_pixel)];
        idx += SCREENWIDTH as usize;
        frac += fracstep;
    }
}
pub fn draw_column_low(state: &mut GameState) {
    let count: i32 = state.render.r_draw.dc_yh - state.render.r_draw.dc_yl;
    if count < 0 {
        return;
    }
    if state.render.r_draw.dc_x.cast_unsigned() >= SCREENWIDTH as u32
        || state.render.r_draw.dc_yl < 0
        || state.render.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.render.r_draw.dc_yl, state.render.r_draw.dc_yh, state.render.r_draw.dc_x
        ));
    }
    let x: i32 = state.render.r_draw.dc_x << 1;
    let mut idx: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl.idx()]
        + state.render.r_draw.columnofs[x.idx()].idx();
    let mut idx2: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl.idx()]
        + state.render.r_draw.columnofs[(x + 1).idx()].idx();
    let fracstep: Fixed = state.render.r_draw.dc_iscale;
    let mut frac: Fixed = state.render.r_draw.dc_texturemid
        + (state.render.r_draw.dc_yl - state.render.r_main.centery) * fracstep;
    let (source, base) = source_bytes(
        &state.render.r_data,
        &state.assets.w_wad,
        state.render.r_draw.column_source(),
    );
    let colormap = colormap_row(
        &state.render.r_data.colormaps,
        state.render.r_draw.column_colormap(),
    );
    let screen = &mut state.io.i_video.i_video_buffer[..];
    for _ in 0..=count {
        let src_pixel = source[(base + (frac >> FRACBITS & 127).to_bits() as isize).idx()];
        let pixel = colormap[usize::from(src_pixel)];
        screen[idx] = pixel;
        screen[idx2] = pixel;
        idx += SCREENWIDTH as usize;
        idx2 += SCREENWIDTH as usize;
        frac += fracstep;
    }
}
pub const FUZZTABLE: i32 = 50;
pub const FUZZOFF: i32 = 320;
pub static FUZZOFFSET: [i32; 50] = [
    FUZZOFF, -FUZZOFF, FUZZOFF, -FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF,
    FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, -FUZZOFF, -FUZZOFF,
    -FUZZOFF, FUZZOFF, -FUZZOFF, -FUZZOFF, FUZZOFF, FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, FUZZOFF,
    -FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, -FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, -FUZZOFF, -FUZZOFF,
    -FUZZOFF, FUZZOFF, FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, FUZZOFF,
];
pub fn draw_fuzz_column(state: &mut GameState) {
    if state.render.r_draw.dc_yl == 0 {
        state.render.r_draw.dc_yl = 1;
    }
    if state.render.r_draw.dc_yh == state.render.r_draw.viewheight - 1 {
        state.render.r_draw.dc_yh = state.render.r_draw.viewheight - 2;
    }
    let count: i32 = state.render.r_draw.dc_yh - state.render.r_draw.dc_yl;
    if count < 0 {
        return;
    }
    if state.render.r_draw.dc_x.cast_unsigned() >= SCREENWIDTH as u32
        || state.render.r_draw.dc_yl < 0
        || state.render.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawFuzzColumn: {} to {} at {}",
            state.render.r_draw.dc_yl, state.render.r_draw.dc_yh, state.render.r_draw.dc_x
        ));
    }
    let mut idx: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl.idx()]
        + state.render.r_draw.columnofs[state.render.r_draw.dc_x.idx()].idx();
    let colormap = colormap_row(&state.render.r_data.colormaps, 6);
    let screen = &mut state.io.i_video.i_video_buffer[..];
    let mut fuzzpos = state.render.r_draw.fuzzpos;
    for _ in 0..=count {
        let neighbor_idx = (idx as isize + FUZZOFFSET[fuzzpos.idx()] as isize).idx();
        let neighbor = screen[neighbor_idx];
        screen[idx] = colormap[usize::from(neighbor)];
        fuzzpos += 1;
        if fuzzpos == FUZZTABLE {
            fuzzpos = 0;
        }
        idx += SCREENWIDTH as usize;
    }
    state.render.r_draw.fuzzpos = fuzzpos;
}
pub fn draw_fuzz_column_low(state: &mut GameState) {
    if state.render.r_draw.dc_yl == 0 {
        state.render.r_draw.dc_yl = 1;
    }
    if state.render.r_draw.dc_yh == state.render.r_draw.viewheight - 1 {
        state.render.r_draw.dc_yh = state.render.r_draw.viewheight - 2;
    }
    let count: i32 = state.render.r_draw.dc_yh - state.render.r_draw.dc_yl;
    if count < 0 {
        return;
    }
    let x: i32 = state.render.r_draw.dc_x << 1;
    if x.cast_unsigned() >= SCREENWIDTH as u32
        || state.render.r_draw.dc_yl < 0
        || state.render.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawFuzzColumn: {} to {} at {}",
            state.render.r_draw.dc_yl, state.render.r_draw.dc_yh, state.render.r_draw.dc_x
        ));
    }
    let mut idx: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl.idx()]
        + state.render.r_draw.columnofs[x.idx()].idx();
    let mut idx2: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl.idx()]
        + state.render.r_draw.columnofs[(x + 1).idx()].idx();
    let colormap = colormap_row(&state.render.r_data.colormaps, 6);
    let screen = &mut state.io.i_video.i_video_buffer[..];
    let mut fuzzpos = state.render.r_draw.fuzzpos;
    for _ in 0..=count {
        let off = FUZZOFFSET[fuzzpos.idx()] as isize;
        let neighbor = screen[(idx as isize + off).idx()];
        let neighbor2 = screen[(idx2 as isize + off).idx()];
        screen[idx] = colormap[usize::from(neighbor)];
        screen[idx2] = colormap[usize::from(neighbor2)];
        fuzzpos += 1;
        if fuzzpos == FUZZTABLE {
            fuzzpos = 0;
        }
        idx += SCREENWIDTH as usize;
        idx2 += SCREENWIDTH as usize;
    }
    state.render.r_draw.fuzzpos = fuzzpos;
}
pub fn draw_translated_column(state: &mut GameState) {
    let count: i32 = state.render.r_draw.dc_yh - state.render.r_draw.dc_yl;
    if count < 0 {
        return;
    }
    if state.render.r_draw.dc_x.cast_unsigned() >= SCREENWIDTH as u32
        || state.render.r_draw.dc_yl < 0
        || state.render.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.render.r_draw.dc_yl, state.render.r_draw.dc_yh, state.render.r_draw.dc_x
        ));
    }
    let mut idx: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl.idx()]
        + state.render.r_draw.columnofs[state.render.r_draw.dc_x.idx()].idx();
    let fracstep: Fixed = state.render.r_draw.dc_iscale;
    let mut frac: Fixed = state.render.r_draw.dc_texturemid
        + (state.render.r_draw.dc_yl - state.render.r_main.centery) * fracstep;
    let (source, base) = source_bytes(
        &state.render.r_data,
        &state.assets.w_wad,
        state.render.r_draw.column_source(),
    );
    let translation = translation_row(
        &state.render.r_draw.translationtables,
        state.render.r_draw.dc_translation,
    );
    let colormap = colormap_row(
        &state.render.r_data.colormaps,
        state.render.r_draw.column_colormap(),
    );
    let screen = &mut state.io.i_video.i_video_buffer[..];
    for _ in 0..=count {
        let raw_pixel = source[(base + (frac >> FRACBITS).to_bits() as isize).idx()];
        let src_pixel = translation[usize::from(raw_pixel)];
        screen[idx] = colormap[usize::from(src_pixel)];
        idx += SCREENWIDTH as usize;
        frac += fracstep;
    }
}
pub fn draw_translated_column_low(state: &mut GameState) {
    let count: i32 = state.render.r_draw.dc_yh - state.render.r_draw.dc_yl;
    if count < 0 {
        return;
    }
    let x: i32 = state.render.r_draw.dc_x << 1;
    if x.cast_unsigned() >= SCREENWIDTH as u32
        || state.render.r_draw.dc_yl < 0
        || state.render.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.render.r_draw.dc_yl, state.render.r_draw.dc_yh, x
        ));
    }
    let mut idx: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl.idx()]
        + state.render.r_draw.columnofs[x.idx()].idx();
    let mut idx2: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl.idx()]
        + state.render.r_draw.columnofs[(x + 1).idx()].idx();
    let fracstep: Fixed = state.render.r_draw.dc_iscale;
    let mut frac: Fixed = state.render.r_draw.dc_texturemid
        + (state.render.r_draw.dc_yl - state.render.r_main.centery) * fracstep;
    let (source, base) = source_bytes(
        &state.render.r_data,
        &state.assets.w_wad,
        state.render.r_draw.column_source(),
    );
    let translation = translation_row(
        &state.render.r_draw.translationtables,
        state.render.r_draw.dc_translation,
    );
    let colormap = colormap_row(
        &state.render.r_data.colormaps,
        state.render.r_draw.column_colormap(),
    );
    let screen = &mut state.io.i_video.i_video_buffer[..];
    for _ in 0..=count {
        let raw_pixel = source[(base + (frac >> FRACBITS).to_bits() as isize).idx()];
        let src_pixel = translation[usize::from(raw_pixel)];
        let pixel = colormap[usize::from(src_pixel)];
        screen[idx] = pixel;
        screen[idx2] = pixel;
        idx += SCREENWIDTH as usize;
        idx2 += SCREENWIDTH as usize;
        frac += fracstep;
    }
}
pub fn init_translation_tables(r_draw: &mut RDrawState) {
    r_draw.translationtables = vec![0u8; 256 * 3];
    for i in 0..256_i32 {
        if (0x70..=0x7f).contains(&i) {
            r_draw.translationtables[i.idx()] = (0x60 + (i & 0xf)).cast_unsigned() as u8;
            r_draw.translationtables[(i + 256).idx()] = (0x40 + (i & 0xf)).cast_unsigned() as u8;
            r_draw.translationtables[(i + 512).idx()] = (0x20 + (i & 0xf)).cast_unsigned() as u8;
        } else {
            let identity = i.cast_unsigned() as u8;
            r_draw.translationtables[(i + 512).idx()] = identity;
            r_draw.translationtables[(i + 256).idx()] = identity;
            r_draw.translationtables[i.idx()] = identity;
        }
    }
}
pub fn draw_span(state: &mut GameState) {
    if state.render.r_draw.ds_x2 < state.render.r_draw.ds_x1
        || state.render.r_draw.ds_x1 < 0
        || state.render.r_draw.ds_x2 >= SCREENWIDTH
        || state.render.r_draw.ds_y.cast_unsigned() > SCREENHEIGHT as u32
    {
        error(&format!(
            "R_DrawSpan: {} to {} at {}",
            state.render.r_draw.ds_x1, state.render.r_draw.ds_x2, state.render.r_draw.ds_y
        ));
    }
    let mut position: u32 = (state.render.r_draw.ds_xfrac << 10)
        .to_bits()
        .cast_unsigned()
        & 0xffff0000
        | (state.render.r_draw.ds_yfrac >> 6 & 0xffff)
            .to_bits()
            .cast_unsigned();
    let step: u32 = (state.render.r_draw.ds_xstep << 10)
        .to_bits()
        .cast_unsigned()
        & 0xffff0000
        | (state.render.r_draw.ds_ystep >> 6 & 0xffff)
            .to_bits()
            .cast_unsigned();
    let idx: usize = state.render.r_draw.ylookup[state.render.r_draw.ds_y.idx()]
        + state.render.r_draw.columnofs[state.render.r_draw.ds_x1.idx()].idx();
    let count: i32 = state.render.r_draw.ds_x2 - state.render.r_draw.ds_x1;
    let (source, base) = source_bytes(
        &state.render.r_data,
        &state.assets.w_wad,
        state.render.r_draw.span_source(),
    );
    let colormap = colormap_row(
        &state.render.r_data.colormaps,
        state.render.r_draw.ds_colormap,
    );
    let screen = &mut state.io.i_video.i_video_buffer[..];
    for dst in &mut screen[idx..=(idx + count.idx())] {
        let ytemp = position >> 4 & 0xfc0;
        let xtemp = position >> 26;
        let spot = (xtemp | ytemp) as i32;
        let src_pixel = source[(base + spot as isize).idx()];
        *dst = colormap[usize::from(src_pixel)];
        position = position.wrapping_add(step);
    }
}
#[allow(clippy::chunks_exact_to_as_chunks)] // as_chunks needs a newer toolchain than the ESP one
pub fn draw_span_low(state: &mut GameState) {
    if state.render.r_draw.ds_x2 < state.render.r_draw.ds_x1
        || state.render.r_draw.ds_x1 < 0
        || state.render.r_draw.ds_x2 >= SCREENWIDTH
        || state.render.r_draw.ds_y.cast_unsigned() > SCREENHEIGHT as u32
    {
        error(&format!(
            "R_DrawSpan: {} to {} at {}",
            state.render.r_draw.ds_x1, state.render.r_draw.ds_x2, state.render.r_draw.ds_y
        ));
    }
    let mut position: u32 = (state.render.r_draw.ds_xfrac << 10)
        .to_bits()
        .cast_unsigned()
        & 0xffff0000
        | (state.render.r_draw.ds_yfrac >> 6 & 0xffff)
            .to_bits()
            .cast_unsigned();
    let step: u32 = (state.render.r_draw.ds_xstep << 10)
        .to_bits()
        .cast_unsigned()
        & 0xffff0000
        | (state.render.r_draw.ds_ystep >> 6 & 0xffff)
            .to_bits()
            .cast_unsigned();
    let count: i32 = state.render.r_draw.ds_x2 - state.render.r_draw.ds_x1;
    state.render.r_draw.ds_x1 <<= 1;
    state.render.r_draw.ds_x2 <<= 1;
    let idx: usize = state.render.r_draw.ylookup[state.render.r_draw.ds_y.idx()]
        + state.render.r_draw.columnofs[state.render.r_draw.ds_x1.idx()].idx();
    let (source, base) = source_bytes(
        &state.render.r_data,
        &state.assets.w_wad,
        state.render.r_draw.span_source(),
    );
    let colormap = colormap_row(
        &state.render.r_data.colormaps,
        state.render.r_draw.ds_colormap,
    );
    let screen = &mut state.io.i_video.i_video_buffer[..];
    for dst in screen[idx..idx + 2 * (count.idx() + 1)].chunks_exact_mut(2) {
        let ytemp = position >> 4 & 0xfc0;
        let xtemp = position >> 26;
        let spot = (xtemp | ytemp) as i32;
        let src_pixel = source[(base + spot as isize).idx()];
        dst.fill(colormap[usize::from(src_pixel)]);
        position = position.wrapping_add(step);
    }
}
pub fn init_buffer(r_draw: &mut RDrawState, width: i32, height: i32) {
    r_draw.viewwindowx = (SCREENWIDTH - width) >> 1;
    for i in 0..width {
        r_draw.columnofs[i.idx()] = r_draw.viewwindowx + i;
    }
    if width == SCREENWIDTH {
        r_draw.viewwindowy = 0;
    } else {
        r_draw.viewwindowy = (SCREENHEIGHT - SBARHEIGHT - height) >> 1;
    }
    for i in 0..height {
        r_draw.ylookup[i.idx()] = ((i + r_draw.viewwindowy) * SCREENWIDTH).idx();
    }
}
pub fn fill_back_screen(state: &mut GameState) {
    let name1: &str = "FLOOR7_2";
    let name2: &str = "GRNROCK";

    if state.render.r_draw.scaledviewwidth == SCREENWIDTH {
        state.render.r_draw.background_buffer = None;
        return;
    }
    if state.render.r_draw.background_buffer.is_none() {
        state.render.r_draw.background_buffer = Some(vec![
            0u8;
            (SCREENWIDTH * (SCREENHEIGHT - SBARHEIGHT))
                as usize
        ]);
    }
    let name: &str = if state.game.doomstat.gamemode == GameMode::Commercial {
        name2
    } else {
        name1
    };
    let flat = lump_bytes_name(&*state.assets.fs, &mut state.assets.w_wad, name);
    let background = state
        .render
        .r_draw
        .background_buffer
        .as_mut()
        .expect("the background buffer is allocated at start-up");
    for y in 0..(SCREENHEIGHT - SBARHEIGHT) as usize {
        let row = &flat[(y & 63) << 6..][..64];
        let line = &mut background[y * SCREENWIDTH as usize..][..SCREENWIDTH as usize];
        for chunk in line.chunks_mut(64) {
            chunk.copy_from_slice(&row[..chunk.len()]);
        }
    }
    let backdrop = Screen::Background;
    let mut patch: Patch = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "brdr_t");
    for x in (0..state.render.r_draw.scaledviewwidth).step_by(8) {
        draw_patch(
            state,
            backdrop,
            state.render.r_draw.viewwindowx + x,
            state.render.r_draw.viewwindowy - 8,
            &patch,
        );
    }
    patch = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "brdr_b");
    for x in (0..state.render.r_draw.scaledviewwidth).step_by(8) {
        draw_patch(
            state,
            backdrop,
            state.render.r_draw.viewwindowx + x,
            state.render.r_draw.viewwindowy + state.render.r_draw.viewheight,
            &patch,
        );
    }
    patch = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "brdr_l");
    for y in (0..state.render.r_draw.viewheight).step_by(8) {
        draw_patch(
            state,
            backdrop,
            state.render.r_draw.viewwindowx - 8,
            state.render.r_draw.viewwindowy + y,
            &patch,
        );
    }
    patch = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "brdr_r");
    for y in (0..state.render.r_draw.viewheight).step_by(8) {
        draw_patch(
            state,
            backdrop,
            state.render.r_draw.viewwindowx + state.render.r_draw.scaledviewwidth,
            state.render.r_draw.viewwindowy + y,
            &patch,
        );
    }
    let __wcache654_4 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "brdr_tl");
    draw_patch(
        state,
        backdrop,
        state.render.r_draw.viewwindowx - 8,
        state.render.r_draw.viewwindowy - 8,
        &__wcache654_4,
    );
    let __wcache660_3 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "brdr_tr");
    draw_patch(
        state,
        backdrop,
        state.render.r_draw.viewwindowx + state.render.r_draw.scaledviewwidth,
        state.render.r_draw.viewwindowy - 8,
        &__wcache660_3,
    );
    let __wcache666_2 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "brdr_bl");
    draw_patch(
        state,
        backdrop,
        state.render.r_draw.viewwindowx - 8,
        state.render.r_draw.viewwindowy + state.render.r_draw.viewheight,
        &__wcache666_2,
    );
    let __wcache672_1 = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "brdr_br");
    draw_patch(
        state,
        backdrop,
        state.render.r_draw.viewwindowx + state.render.r_draw.scaledviewwidth,
        state.render.r_draw.viewwindowy + state.render.r_draw.viewheight,
        &__wcache672_1,
    );
}
pub fn video_erase(i_video: &mut IVideoState, r_draw: &RDrawState, ofs: u32, count: i32) {
    if let Some(background_buffer) = &r_draw.background_buffer {
        let range = ofs as usize..ofs as usize + count.idx();
        i_video.i_video_buffer[range.clone()].copy_from_slice(&background_buffer[range]);
    }
}
pub fn draw_view_border(i_video: &mut IVideoState, r_draw: &RDrawState, v_video: &mut VVideoState) {
    if r_draw.scaledviewwidth == SCREENWIDTH {
        return;
    }
    let top: i32 = (SCREENHEIGHT - SBARHEIGHT - r_draw.viewheight) / 2;
    let mut side: i32 = (SCREENWIDTH - r_draw.scaledviewwidth) / 2;
    video_erase(i_video, r_draw, 0, top * SCREENWIDTH + side);
    let mut ofs: i32 = (r_draw.viewheight + top) * SCREENWIDTH - side;
    video_erase(
        i_video,
        r_draw,
        ofs.cast_unsigned(),
        top * SCREENWIDTH + side,
    );
    ofs = top * SCREENWIDTH + SCREENWIDTH - side;
    side <<= 1;
    for _ in 1..r_draw.viewheight {
        video_erase(i_video, r_draw, ofs.cast_unsigned(), side);
        ofs += SCREENWIDTH;
    }
    let dest_screen = Screen::Video;
    mark_rect(
        v_video,
        dest_screen,
        0,
        0,
        SCREENWIDTH,
        SCREENHEIGHT - SBARHEIGHT,
    );
}
