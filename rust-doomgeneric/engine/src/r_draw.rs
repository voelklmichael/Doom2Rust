use crate::d_mode::GameMode;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::i_video::IVideoState;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::patch::Patch;
use crate::r_data::RDataState;
use crate::v_video::cache_patch_name;
use crate::v_video::Screen;
use crate::v_video::VVideoState;
use crate::w_wad::WWadState;
use alloc::vec::Vec;

use crate::r_main::ColormapId;

use crate::v_video::draw_patch;
use crate::v_video::mark_rect;
use crate::w_wad::lump_bytes_name;

#[derive(Clone, Copy)]
pub enum ColumnSource {
    Lump { lump: i32, offset: usize },
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
            &w_wad.lumpinfo[lump as usize].cache.as_ref().unwrap()[..],
            offset as isize,
        ),
        ColumnSource::Composite { tex, offset } => (
            r_data.texturecomposite[tex as usize].as_ref().unwrap(),
            offset as isize,
        ),
    }
}

/// One light level of the colormap table: what a source pixel becomes at that brightness.
fn colormap_row(colormaps: &[u8], index: ColormapId) -> &[u8; 256] {
    let start = (index * 256) as usize;
    (&colormaps[start..start + 256]).try_into().unwrap()
}

/// One of the player colour translation tables, `dc_translation` bytes into `translationtables`.
fn translation_row(tables: &[u8], offset: usize) -> &[u8; 256] {
    (&tables[offset..offset + 256]).try_into().unwrap()
}

pub fn read_source(r_data: &RDataState, w_wad: &WWadState, src: ColumnSource, idx: i32) -> u8 {
    let (bytes, base) = source_bytes(r_data, w_wad, src);
    bytes[(base + idx as isize) as usize]
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

impl Default for RDrawState {
    fn default() -> Self {
        Self::new()
    }
}

impl RDrawState {
    pub const fn new() -> Self {
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
            dc_iscale: 0,
            dc_texturemid: 0,
            dc_source: None,
            dccount: 0,
            fuzzpos: 0,
            dc_translation: 0,
            translationtables: Vec::new(),
            ds_y: 0,
            ds_x1: 0,
            ds_x2: 0,
            ds_colormap: 0,
            ds_xfrac: 0,
            ds_yfrac: 0,
            ds_xstep: 0,
            ds_ystep: 0,
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
    if state.render.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.render.r_draw.dc_yl < 0
        || state.render.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.render.r_draw.dc_yl, state.render.r_draw.dc_yh, state.render.r_draw.dc_x
        ));
    }
    let mut idx: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl as usize]
        + state.render.r_draw.columnofs[state.render.r_draw.dc_x as usize] as usize;
    let fracstep: Fixed = state.render.r_draw.dc_iscale;
    let mut frac: Fixed = state.render.r_draw.dc_texturemid
        + (state.render.r_draw.dc_yl as Fixed - state.render.r_main.centery as Fixed) * fracstep;
    let (source, base) = source_bytes(
        &state.render.r_data,
        &state.assets.w_wad,
        state.render.r_draw.dc_source.unwrap(),
    );
    let colormap = colormap_row(
        &state.render.r_data.colormaps,
        state.render.r_draw.dc_colormap.unwrap(),
    );
    let screen = &mut state.io.i_video.i_video_buffer[..];
    for _ in 0..=count {
        let src_pixel = source[(base + (frac >> FRACBITS & 127) as isize) as usize];
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
    if state.render.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.render.r_draw.dc_yl < 0
        || state.render.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.render.r_draw.dc_yl, state.render.r_draw.dc_yh, state.render.r_draw.dc_x
        ));
    }
    let x: i32 = state.render.r_draw.dc_x << 1;
    let mut idx: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl as usize]
        + state.render.r_draw.columnofs[x as usize] as usize;
    let mut idx2: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl as usize]
        + state.render.r_draw.columnofs[(x + 1) as usize] as usize;
    let fracstep: Fixed = state.render.r_draw.dc_iscale;
    let mut frac: Fixed = state.render.r_draw.dc_texturemid
        + (state.render.r_draw.dc_yl as Fixed - state.render.r_main.centery as Fixed) * fracstep;
    let (source, base) = source_bytes(
        &state.render.r_data,
        &state.assets.w_wad,
        state.render.r_draw.dc_source.unwrap(),
    );
    let colormap = colormap_row(
        &state.render.r_data.colormaps,
        state.render.r_draw.dc_colormap.unwrap(),
    );
    let screen = &mut state.io.i_video.i_video_buffer[..];
    for _ in 0..=count {
        let src_pixel = source[(base + (frac >> FRACBITS & 127) as isize) as usize];
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
    if state.render.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.render.r_draw.dc_yl < 0
        || state.render.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawFuzzColumn: {} to {} at {}",
            state.render.r_draw.dc_yl, state.render.r_draw.dc_yh, state.render.r_draw.dc_x
        ));
    }
    let mut idx: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl as usize]
        + state.render.r_draw.columnofs[state.render.r_draw.dc_x as usize] as usize;
    let colormap = colormap_row(&state.render.r_data.colormaps, 6);
    let screen = &mut state.io.i_video.i_video_buffer[..];
    let mut fuzzpos = state.render.r_draw.fuzzpos;
    for _ in 0..=count {
        let neighbor_idx = (idx as isize + FUZZOFFSET[fuzzpos as usize] as isize) as usize;
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
    if x as u32 >= SCREENWIDTH as u32
        || state.render.r_draw.dc_yl < 0
        || state.render.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawFuzzColumn: {} to {} at {}",
            state.render.r_draw.dc_yl, state.render.r_draw.dc_yh, state.render.r_draw.dc_x
        ));
    }
    let mut idx: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl as usize]
        + state.render.r_draw.columnofs[x as usize] as usize;
    let mut idx2: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl as usize]
        + state.render.r_draw.columnofs[(x + 1) as usize] as usize;
    let colormap = colormap_row(&state.render.r_data.colormaps, 6);
    let screen = &mut state.io.i_video.i_video_buffer[..];
    let mut fuzzpos = state.render.r_draw.fuzzpos;
    for _ in 0..=count {
        let off = FUZZOFFSET[fuzzpos as usize] as isize;
        let neighbor = screen[(idx as isize + off) as usize];
        let neighbor2 = screen[(idx2 as isize + off) as usize];
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
    if state.render.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.render.r_draw.dc_yl < 0
        || state.render.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.render.r_draw.dc_yl, state.render.r_draw.dc_yh, state.render.r_draw.dc_x
        ));
    }
    let mut idx: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl as usize]
        + state.render.r_draw.columnofs[state.render.r_draw.dc_x as usize] as usize;
    let fracstep: Fixed = state.render.r_draw.dc_iscale;
    let mut frac: Fixed = state.render.r_draw.dc_texturemid
        + (state.render.r_draw.dc_yl as Fixed - state.render.r_main.centery as Fixed) * fracstep;
    let (source, base) = source_bytes(
        &state.render.r_data,
        &state.assets.w_wad,
        state.render.r_draw.dc_source.unwrap(),
    );
    let translation = translation_row(
        &state.render.r_draw.translationtables,
        state.render.r_draw.dc_translation,
    );
    let colormap = colormap_row(
        &state.render.r_data.colormaps,
        state.render.r_draw.dc_colormap.unwrap(),
    );
    let screen = &mut state.io.i_video.i_video_buffer[..];
    for _ in 0..=count {
        let raw_pixel = source[(base + (frac >> FRACBITS) as isize) as usize];
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
    if x as u32 >= SCREENWIDTH as u32
        || state.render.r_draw.dc_yl < 0
        || state.render.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.render.r_draw.dc_yl, state.render.r_draw.dc_yh, x
        ));
    }
    let mut idx: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl as usize]
        + state.render.r_draw.columnofs[x as usize] as usize;
    let mut idx2: usize = state.render.r_draw.ylookup[state.render.r_draw.dc_yl as usize]
        + state.render.r_draw.columnofs[(x + 1) as usize] as usize;
    let fracstep: Fixed = state.render.r_draw.dc_iscale;
    let mut frac: Fixed = state.render.r_draw.dc_texturemid
        + (state.render.r_draw.dc_yl as Fixed - state.render.r_main.centery as Fixed) * fracstep;
    let (source, base) = source_bytes(
        &state.render.r_data,
        &state.assets.w_wad,
        state.render.r_draw.dc_source.unwrap(),
    );
    let translation = translation_row(
        &state.render.r_draw.translationtables,
        state.render.r_draw.dc_translation,
    );
    let colormap = colormap_row(
        &state.render.r_data.colormaps,
        state.render.r_draw.dc_colormap.unwrap(),
    );
    let screen = &mut state.io.i_video.i_video_buffer[..];
    for _ in 0..=count {
        let raw_pixel = source[(base + (frac >> FRACBITS) as isize) as usize];
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
    for i in 0..256 {
        if (0x70..=0x7f).contains(&i) {
            r_draw.translationtables[i as usize] = (0x60 + (i & 0xf)) as u8;
            r_draw.translationtables[(i + 256) as usize] = (0x40 + (i & 0xf)) as u8;
            r_draw.translationtables[(i + 512) as usize] = (0x20 + (i & 0xf)) as u8;
        } else {
            let identity = i as u8;
            r_draw.translationtables[(i + 512) as usize] = identity;
            r_draw.translationtables[(i + 256) as usize] = identity;
            r_draw.translationtables[i as usize] = identity;
        }
    }
}
pub fn draw_span(state: &mut GameState) {
    if state.render.r_draw.ds_x2 < state.render.r_draw.ds_x1
        || state.render.r_draw.ds_x1 < 0
        || state.render.r_draw.ds_x2 >= SCREENWIDTH
        || state.render.r_draw.ds_y as u32 > SCREENHEIGHT as u32
    {
        error(&format!(
            "R_DrawSpan: {} to {} at {}",
            state.render.r_draw.ds_x1, state.render.r_draw.ds_x2, state.render.r_draw.ds_y
        ));
    }
    let mut position: u32 = (state.render.r_draw.ds_xfrac << 10) as u32 & 0xffff0000
        | (state.render.r_draw.ds_yfrac >> 6 & 0xffff) as u32;
    let step: u32 = (state.render.r_draw.ds_xstep << 10) as u32 & 0xffff0000
        | (state.render.r_draw.ds_ystep >> 6 & 0xffff) as u32;
    let idx: usize = state.render.r_draw.ylookup[state.render.r_draw.ds_y as usize]
        + state.render.r_draw.columnofs[state.render.r_draw.ds_x1 as usize] as usize;
    let count: i32 = state.render.r_draw.ds_x2 - state.render.r_draw.ds_x1;
    let (source, base) = source_bytes(
        &state.render.r_data,
        &state.assets.w_wad,
        state.render.r_draw.ds_source.unwrap(),
    );
    let colormap = colormap_row(
        &state.render.r_data.colormaps,
        state.render.r_draw.ds_colormap,
    );
    let screen = &mut state.io.i_video.i_video_buffer[..];
    for dst in &mut screen[idx..idx + count as usize + 1] {
        let ytemp = position >> 4 & 0xfc0;
        let xtemp = position >> 26;
        let spot = (xtemp | ytemp) as i32;
        let src_pixel = source[(base + spot as isize) as usize];
        *dst = colormap[usize::from(src_pixel)];
        position = position.wrapping_add(step);
    }
}
#[allow(clippy::chunks_exact_to_as_chunks)] // as_chunks needs a newer toolchain than the ESP one
pub fn draw_span_low(state: &mut GameState) {
    if state.render.r_draw.ds_x2 < state.render.r_draw.ds_x1
        || state.render.r_draw.ds_x1 < 0
        || state.render.r_draw.ds_x2 >= SCREENWIDTH
        || state.render.r_draw.ds_y as u32 > SCREENHEIGHT as u32
    {
        error(&format!(
            "R_DrawSpan: {} to {} at {}",
            state.render.r_draw.ds_x1, state.render.r_draw.ds_x2, state.render.r_draw.ds_y
        ));
    }
    let mut position: u32 = (state.render.r_draw.ds_xfrac << 10) as u32 & 0xffff0000
        | (state.render.r_draw.ds_yfrac >> 6 & 0xffff) as u32;
    let step: u32 = (state.render.r_draw.ds_xstep << 10) as u32 & 0xffff0000
        | (state.render.r_draw.ds_ystep >> 6 & 0xffff) as u32;
    let count: i32 = state.render.r_draw.ds_x2 - state.render.r_draw.ds_x1;
    state.render.r_draw.ds_x1 <<= 1;
    state.render.r_draw.ds_x2 <<= 1;
    let idx: usize = state.render.r_draw.ylookup[state.render.r_draw.ds_y as usize]
        + state.render.r_draw.columnofs[state.render.r_draw.ds_x1 as usize] as usize;
    let (source, base) = source_bytes(
        &state.render.r_data,
        &state.assets.w_wad,
        state.render.r_draw.ds_source.unwrap(),
    );
    let colormap = colormap_row(
        &state.render.r_data.colormaps,
        state.render.r_draw.ds_colormap,
    );
    let screen = &mut state.io.i_video.i_video_buffer[..];
    for dst in screen[idx..idx + 2 * (count as usize + 1)].chunks_exact_mut(2) {
        let ytemp = position >> 4 & 0xfc0;
        let xtemp = position >> 26;
        let spot = (xtemp | ytemp) as i32;
        let src_pixel = source[(base + spot as isize) as usize];
        dst.fill(colormap[usize::from(src_pixel)]);
        position = position.wrapping_add(step);
    }
}
pub fn init_buffer(r_draw: &mut RDrawState, width: i32, height: i32) {
    r_draw.viewwindowx = (SCREENWIDTH - width) >> 1;
    for i in 0..width {
        r_draw.columnofs[i as usize] = r_draw.viewwindowx + i;
    }
    if width == SCREENWIDTH {
        r_draw.viewwindowy = 0;
    } else {
        r_draw.viewwindowy = (SCREENHEIGHT - SBARHEIGHT - height) >> 1;
    }
    for i in 0..height {
        r_draw.ylookup[i as usize] = ((i + r_draw.viewwindowy) * SCREENWIDTH) as usize;
    }
}
pub fn fill_back_screen(state: &mut GameState) {
    let mut y: i32;
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
    let background = state.render.r_draw.background_buffer.as_mut().unwrap();
    for y in 0..(SCREENHEIGHT - SBARHEIGHT) as usize {
        let row = &flat[(y & 63) << 6..][..64];
        let line = &mut background[y * SCREENWIDTH as usize..][..SCREENWIDTH as usize];
        for chunk in line.chunks_mut(64) {
            chunk.copy_from_slice(&row[..chunk.len()]);
        }
    }
    let backdrop = Screen::Background;
    let mut patch: Patch = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "brdr_t");
    let mut x: i32 = 0;
    while x < state.render.r_draw.scaledviewwidth {
        draw_patch(
            state,
            backdrop,
            state.render.r_draw.viewwindowx + x,
            state.render.r_draw.viewwindowy - 8,
            &patch,
        );
        x += 8;
    }
    patch = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "brdr_b");
    x = 0;
    while x < state.render.r_draw.scaledviewwidth {
        draw_patch(
            state,
            backdrop,
            state.render.r_draw.viewwindowx + x,
            state.render.r_draw.viewwindowy + state.render.r_draw.viewheight,
            &patch,
        );
        x += 8;
    }
    patch = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "brdr_l");
    y = 0;
    while y < state.render.r_draw.viewheight {
        draw_patch(
            state,
            backdrop,
            state.render.r_draw.viewwindowx - 8,
            state.render.r_draw.viewwindowy + y,
            &patch,
        );
        y += 8;
    }
    patch = cache_patch_name(&*state.assets.fs, &mut state.assets.w_wad, "brdr_r");
    y = 0;
    while y < state.render.r_draw.viewheight {
        draw_patch(
            state,
            backdrop,
            state.render.r_draw.viewwindowx + state.render.r_draw.scaledviewwidth,
            state.render.r_draw.viewwindowy + y,
            &patch,
        );
        y += 8;
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
        let range = ofs as usize..ofs as usize + count as usize;
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
    video_erase(i_video, r_draw, ofs as u32, top * SCREENWIDTH + side);
    ofs = top * SCREENWIDTH + SCREENWIDTH - side;
    side <<= 1;
    for _ in 1..r_draw.viewheight {
        video_erase(i_video, r_draw, ofs as u32, side);
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
