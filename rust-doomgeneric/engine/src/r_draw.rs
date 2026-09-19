use crate::d_mode::GameMode;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::patch::Patch;
use crate::v_video::cache_patch_name;
use crate::v_video::Screen;
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

pub(crate) fn advance_source(src: ColumnSource, delta: usize) -> ColumnSource {
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

pub(crate) fn read_source(state: &GameState, src: ColumnSource, idx: i32) -> u8 {
    match src {
        ColumnSource::Lump { lump, offset } => {
            state.w_wad.lumpinfo[lump as usize].cache.as_ref().unwrap()
                [(offset as isize + idx as isize) as usize]
        }
        ColumnSource::Composite { tex, offset } => state.r_data.texturecomposite[tex as usize]
            .as_ref()
            .unwrap()[(offset as isize + idx as isize) as usize],
    }
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
    let mut count: i32;
    let mut idx: usize;
    let mut frac: Fixed;

    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0 {
        return;
    }
    if state.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    idx = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[state.r_draw.dc_x as usize] as usize;
    let fracstep: Fixed = state.r_draw.dc_iscale;
    frac = state.r_draw.dc_texturemid
        + (state.r_draw.dc_yl as Fixed - state.r_main.centery as Fixed) * fracstep;
    loop {
        let src_pixel = read_source(
            state,
            state.r_draw.dc_source.unwrap(),
            frac >> FRACBITS & 127,
        );
        state.i_video.i_video_buffer[idx] = state.r_data.colormaps
            [(state.r_draw.dc_colormap.unwrap() * 256 + src_pixel as i32) as usize];
        idx += SCREENWIDTH as usize;
        frac += fracstep;
        let fresh0 = count;
        count -= 1;
        if fresh0 == 0 {
            break;
        }
    }
}
pub fn draw_column_low(state: &mut GameState) {
    let mut count: i32;
    let mut idx: usize;
    let mut idx2: usize;
    let mut frac: Fixed;

    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0 {
        return;
    }
    if state.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    let x: i32 = state.r_draw.dc_x << 1;
    idx = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[x as usize] as usize;
    idx2 = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[(x + 1) as usize] as usize;
    let fracstep: Fixed = state.r_draw.dc_iscale;
    frac = state.r_draw.dc_texturemid
        + (state.r_draw.dc_yl as Fixed - state.r_main.centery as Fixed) * fracstep;
    loop {
        let src_pixel = read_source(
            state,
            state.r_draw.dc_source.unwrap(),
            frac >> FRACBITS & 127,
        );
        let pixel = state.r_data.colormaps
            [(state.r_draw.dc_colormap.unwrap() * 256 + src_pixel as i32) as usize];
        state.i_video.i_video_buffer[idx] = pixel;
        state.i_video.i_video_buffer[idx2] = pixel;
        idx += SCREENWIDTH as usize;
        idx2 += SCREENWIDTH as usize;
        frac += fracstep;
        let fresh1 = count;
        count -= 1;
        if fresh1 == 0 {
            break;
        }
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
    let mut count: i32;
    let mut idx: usize;
    if state.r_draw.dc_yl == 0 {
        state.r_draw.dc_yl = 1;
    }
    if state.r_draw.dc_yh == state.r_draw.viewheight - 1 {
        state.r_draw.dc_yh = state.r_draw.viewheight - 2;
    }
    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0 {
        return;
    }
    if state.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawFuzzColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    idx = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[state.r_draw.dc_x as usize] as usize;
    loop {
        let neighbor_idx =
            (idx as isize + FUZZOFFSET[state.r_draw.fuzzpos as usize] as isize) as usize;
        let neighbor = state.i_video.i_video_buffer[neighbor_idx];
        state.i_video.i_video_buffer[idx] =
            state.r_data.colormaps[(6 * 256 + neighbor as i32) as usize];
        state.r_draw.fuzzpos += 1;
        if state.r_draw.fuzzpos == FUZZTABLE {
            state.r_draw.fuzzpos = 0;
        }
        idx += SCREENWIDTH as usize;
        let fresh2 = count;
        count -= 1;
        if fresh2 == 0 {
            break;
        }
    }
}
pub fn draw_fuzz_column_low(state: &mut GameState) {
    let mut count: i32;
    let mut idx: usize;
    let mut idx2: usize;

    if state.r_draw.dc_yl == 0 {
        state.r_draw.dc_yl = 1;
    }
    if state.r_draw.dc_yh == state.r_draw.viewheight - 1 {
        state.r_draw.dc_yh = state.r_draw.viewheight - 2;
    }
    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0 {
        return;
    }
    let x: i32 = state.r_draw.dc_x << 1;
    if x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawFuzzColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    idx = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[x as usize] as usize;
    idx2 = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[(x + 1) as usize] as usize;
    loop {
        let off = FUZZOFFSET[state.r_draw.fuzzpos as usize] as isize;
        let neighbor = state.i_video.i_video_buffer[(idx as isize + off) as usize];
        let neighbor2 = state.i_video.i_video_buffer[(idx2 as isize + off) as usize];
        state.i_video.i_video_buffer[idx] =
            state.r_data.colormaps[(6 * 256 + neighbor as i32) as usize];
        state.i_video.i_video_buffer[idx2] =
            state.r_data.colormaps[(6 * 256 + neighbor2 as i32) as usize];
        state.r_draw.fuzzpos += 1;
        if state.r_draw.fuzzpos == FUZZTABLE {
            state.r_draw.fuzzpos = 0;
        }
        idx += SCREENWIDTH as usize;
        idx2 += SCREENWIDTH as usize;
        let fresh3 = count;
        count -= 1;
        if fresh3 == 0 {
            break;
        }
    }
}
pub fn draw_translated_column(state: &mut GameState) {
    let mut count: i32;
    let mut idx: usize;
    let mut frac: Fixed;

    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0 {
        return;
    }
    if state.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    idx = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[state.r_draw.dc_x as usize] as usize;
    let fracstep: Fixed = state.r_draw.dc_iscale;
    frac = state.r_draw.dc_texturemid
        + (state.r_draw.dc_yl as Fixed - state.r_main.centery as Fixed) * fracstep;
    loop {
        let raw_pixel = read_source(state, state.r_draw.dc_source.unwrap(), frac >> FRACBITS);
        let src_pixel =
            state.r_draw.translationtables[state.r_draw.dc_translation + raw_pixel as usize];
        state.i_video.i_video_buffer[idx] = state.r_data.colormaps
            [(state.r_draw.dc_colormap.unwrap() * 256 + src_pixel as i32) as usize];
        idx += SCREENWIDTH as usize;
        frac += fracstep;
        let fresh4 = count;
        count -= 1;
        if fresh4 == 0 {
            break;
        }
    }
}
pub fn draw_translated_column_low(state: &mut GameState) {
    let mut count: i32;
    let mut idx: usize;
    let mut idx2: usize;
    let mut frac: Fixed;

    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0 {
        return;
    }
    let x: i32 = state.r_draw.dc_x << 1;
    if x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, x
        ));
    }
    idx = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[x as usize] as usize;
    idx2 = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[(x + 1) as usize] as usize;
    let fracstep: Fixed = state.r_draw.dc_iscale;
    frac = state.r_draw.dc_texturemid
        + (state.r_draw.dc_yl as Fixed - state.r_main.centery as Fixed) * fracstep;
    loop {
        let raw_pixel = read_source(state, state.r_draw.dc_source.unwrap(), frac >> FRACBITS);
        let src_pixel =
            state.r_draw.translationtables[state.r_draw.dc_translation + raw_pixel as usize];
        let colormap = state.r_draw.dc_colormap.unwrap();
        let pixel = state.r_data.colormaps[(colormap * 256 + src_pixel as i32) as usize];
        state.i_video.i_video_buffer[idx] = pixel;
        state.i_video.i_video_buffer[idx2] = pixel;
        idx += SCREENWIDTH as usize;
        idx2 += SCREENWIDTH as usize;
        frac += fracstep;
        let fresh5 = count;
        count -= 1;
        if fresh5 == 0 {
            break;
        }
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
            let fresh11 = i as u8;
            r_draw.translationtables[(i + 512) as usize] = fresh11;
            let fresh12 = fresh11;
            r_draw.translationtables[(i + 256) as usize] = fresh12;
            r_draw.translationtables[i as usize] = fresh12;
        }
    }
}
pub fn draw_span(state: &mut GameState) {
    let mut position: u32;

    let mut idx: usize;
    let mut count: i32;
    let mut spot: i32;
    let mut xtemp: u32;
    let mut ytemp: u32;
    if state.r_draw.ds_x2 < state.r_draw.ds_x1
        || state.r_draw.ds_x1 < 0
        || state.r_draw.ds_x2 >= SCREENWIDTH
        || state.r_draw.ds_y as u32 > SCREENHEIGHT as u32
    {
        error(&format!(
            "R_DrawSpan: {} to {} at {}",
            state.r_draw.ds_x1, state.r_draw.ds_x2, state.r_draw.ds_y
        ));
    }
    position = (state.r_draw.ds_xfrac << 10) as u32 & 0xffff0000
        | (state.r_draw.ds_yfrac >> 6 & 0xffff) as u32;
    let step: u32 = (state.r_draw.ds_xstep << 10) as u32 & 0xffff0000
        | (state.r_draw.ds_ystep >> 6 & 0xffff) as u32;
    idx = state.r_draw.ylookup[state.r_draw.ds_y as usize]
        + state.r_draw.columnofs[state.r_draw.ds_x1 as usize] as usize;
    count = state.r_draw.ds_x2 - state.r_draw.ds_x1;
    loop {
        ytemp = position >> 4 & 0xfc0;
        xtemp = position >> 26;
        spot = (xtemp | ytemp) as i32;
        let fresh6 = idx;
        idx += 1;
        let src_pixel = read_source(state, state.r_draw.ds_source.unwrap(), spot);
        state.i_video.i_video_buffer[fresh6] =
            state.r_data.colormaps[(state.r_draw.ds_colormap * 256 + src_pixel as i32) as usize];
        position = position.wrapping_add(step);
        let fresh7 = count;
        count -= 1;
        if fresh7 == 0 {
            break;
        }
    }
}
pub fn draw_span_low(state: &mut GameState) {
    let mut position: u32;

    let mut xtemp: u32;
    let mut ytemp: u32;
    let mut idx: usize;
    let mut count: i32;
    let mut spot: i32;
    if state.r_draw.ds_x2 < state.r_draw.ds_x1
        || state.r_draw.ds_x1 < 0
        || state.r_draw.ds_x2 >= SCREENWIDTH
        || state.r_draw.ds_y as u32 > SCREENHEIGHT as u32
    {
        error(&format!(
            "R_DrawSpan: {} to {} at {}",
            state.r_draw.ds_x1, state.r_draw.ds_x2, state.r_draw.ds_y
        ));
    }
    position = (state.r_draw.ds_xfrac << 10) as u32 & 0xffff0000
        | (state.r_draw.ds_yfrac >> 6 & 0xffff) as u32;
    let step: u32 = (state.r_draw.ds_xstep << 10) as u32 & 0xffff0000
        | (state.r_draw.ds_ystep >> 6 & 0xffff) as u32;
    count = state.r_draw.ds_x2 - state.r_draw.ds_x1;
    state.r_draw.ds_x1 <<= 1;
    state.r_draw.ds_x2 <<= 1;
    idx = state.r_draw.ylookup[state.r_draw.ds_y as usize]
        + state.r_draw.columnofs[state.r_draw.ds_x1 as usize] as usize;
    loop {
        ytemp = position >> 4 & 0xfc0;
        xtemp = position >> 26;
        spot = (xtemp | ytemp) as i32;
        let fresh8 = idx;
        idx += 1;
        let src_pixel = read_source(state, state.r_draw.ds_source.unwrap(), spot);
        let pixel =
            state.r_data.colormaps[(state.r_draw.ds_colormap * 256 + src_pixel as i32) as usize];
        state.i_video.i_video_buffer[fresh8] = pixel;
        let fresh9 = idx;
        idx += 1;
        state.i_video.i_video_buffer[fresh9] = pixel;
        position = position.wrapping_add(step);
        let fresh10 = count;
        count -= 1;
        if fresh10 == 0 {
            break;
        }
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
    let mut x: i32;
    let mut y: i32;
    let mut patch: Patch;
    let name1: &str = "FLOOR7_2";
    let name2: &str = "GRNROCK";

    if state.r_draw.scaledviewwidth == SCREENWIDTH {
        state.r_draw.background_buffer = None;
        return;
    }
    if state.r_draw.background_buffer.is_none() {
        state.r_draw.background_buffer = Some(vec![
            0u8;
            (SCREENWIDTH * (SCREENHEIGHT - SBARHEIGHT))
                as usize
        ]);
    }
    let name: &str = if state.doomstat.gamemode as u32 == GameMode::Commercial as i32 as u32 {
        name2
    } else {
        name1
    };
    let flat = lump_bytes_name(state, name);
    let background = state.r_draw.background_buffer.as_mut().unwrap();
    for y in 0..(SCREENHEIGHT - SBARHEIGHT) as usize {
        let row = &flat[(y & 63) << 6..][..64];
        let line = &mut background[y * SCREENWIDTH as usize..][..SCREENWIDTH as usize];
        for chunk in line.chunks_mut(64) {
            chunk.copy_from_slice(&row[..chunk.len()]);
        }
    }
    let backdrop = Screen::Background;
    patch = cache_patch_name(state, "brdr_t");
    x = 0;
    while x < state.r_draw.scaledviewwidth {
        draw_patch(
            state,
            backdrop,
            state.r_draw.viewwindowx + x,
            state.r_draw.viewwindowy - 8,
            &patch,
        );
        x += 8;
    }
    patch = cache_patch_name(state, "brdr_b");
    x = 0;
    while x < state.r_draw.scaledviewwidth {
        draw_patch(
            state,
            backdrop,
            state.r_draw.viewwindowx + x,
            state.r_draw.viewwindowy + state.r_draw.viewheight,
            &patch,
        );
        x += 8;
    }
    patch = cache_patch_name(state, "brdr_l");
    y = 0;
    while y < state.r_draw.viewheight {
        draw_patch(
            state,
            backdrop,
            state.r_draw.viewwindowx - 8,
            state.r_draw.viewwindowy + y,
            &patch,
        );
        y += 8;
    }
    patch = cache_patch_name(state, "brdr_r");
    y = 0;
    while y < state.r_draw.viewheight {
        draw_patch(
            state,
            backdrop,
            state.r_draw.viewwindowx + state.r_draw.scaledviewwidth,
            state.r_draw.viewwindowy + y,
            &patch,
        );
        y += 8;
    }
    let __wcache654_4 = cache_patch_name(state, "brdr_tl");
    draw_patch(
        state,
        backdrop,
        state.r_draw.viewwindowx - 8,
        state.r_draw.viewwindowy - 8,
        &__wcache654_4,
    );
    let __wcache660_3 = cache_patch_name(state, "brdr_tr");
    draw_patch(
        state,
        backdrop,
        state.r_draw.viewwindowx + state.r_draw.scaledviewwidth,
        state.r_draw.viewwindowy - 8,
        &__wcache660_3,
    );
    let __wcache666_2 = cache_patch_name(state, "brdr_bl");
    draw_patch(
        state,
        backdrop,
        state.r_draw.viewwindowx - 8,
        state.r_draw.viewwindowy + state.r_draw.viewheight,
        &__wcache666_2,
    );
    let __wcache672_1 = cache_patch_name(state, "brdr_br");
    draw_patch(
        state,
        backdrop,
        state.r_draw.viewwindowx + state.r_draw.scaledviewwidth,
        state.r_draw.viewwindowy + state.r_draw.viewheight,
        &__wcache672_1,
    );
}
pub fn video_erase(state: &mut GameState, ofs: u32, count: i32) {
    if let Some(background_buffer) = &state.r_draw.background_buffer {
        let range = ofs as usize..ofs as usize + count as usize;
        state.i_video.i_video_buffer[range.clone()].copy_from_slice(&background_buffer[range]);
    }
}
pub fn draw_view_border(state: &mut GameState) {
    let mut side: i32;
    let mut ofs: i32;
    if state.r_draw.scaledviewwidth == SCREENWIDTH {
        return;
    }
    let top: i32 = (SCREENHEIGHT - SBARHEIGHT - state.r_draw.viewheight) / 2;
    side = (SCREENWIDTH - state.r_draw.scaledviewwidth) / 2;
    video_erase(state, 0, top * SCREENWIDTH + side);
    ofs = (state.r_draw.viewheight + top) * SCREENWIDTH - side;
    video_erase(state, ofs as u32, top * SCREENWIDTH + side);
    ofs = top * SCREENWIDTH + SCREENWIDTH - side;
    side <<= 1;
    for _ in 1..state.r_draw.viewheight {
        video_erase(state, ofs as u32, side);
        ofs += SCREENWIDTH;
    }
    let dest_screen = Screen::Video;
    mark_rect(
        &mut state.v_video,
        dest_screen,
        0,
        0,
        SCREENWIDTH,
        SCREENHEIGHT - SBARHEIGHT,
    );
}
