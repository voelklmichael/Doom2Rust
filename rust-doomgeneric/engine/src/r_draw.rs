use crate::d_mode::GameMode_t;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACBITS;
use crate::patch::Patch;
use crate::v_video::Screen;
use crate::v_video::V_CachePatchName;
use alloc::vec::Vec;

use crate::r_main::ColormapId;
use crate::stdint_types::byte;

use crate::v_video::V_DrawPatch;
use crate::v_video::V_MarkRect;
use crate::w_wad::W_LumpBytesName;

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

pub(crate) fn read_source(state: &GameState, src: ColumnSource, idx: i32) -> byte {
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
    pub background_buffer: Option<Vec<byte>>,
    pub dc_colormap: Option<ColormapId>,
    pub dc_x: i32,
    pub dc_yl: i32,
    pub dc_yh: i32,
    pub dc_iscale: fixed_t,
    pub dc_texturemid: fixed_t,
    pub dc_source: Option<ColumnSource>,
    pub dccount: i32,
    pub fuzzpos: i32,
    pub dc_translation: usize,
    pub translationtables: Vec<byte>,
    pub ds_y: i32,
    pub ds_x1: i32,
    pub ds_x2: i32,
    pub ds_colormap: ColormapId,
    pub ds_xfrac: fixed_t,
    pub ds_yfrac: fixed_t,
    pub ds_xstep: fixed_t,
    pub ds_ystep: fixed_t,
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
        RDrawState {
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
pub fn R_DrawColumn(state: &mut GameState) {
    let mut count: i32;
    let mut idx: usize;
    let mut frac: fixed_t;

    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0_i32 {
        return;
    }
    if state.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0_i32
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        I_Error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    idx = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[state.r_draw.dc_x as usize] as usize;
    let fracstep: fixed_t = state.r_draw.dc_iscale;
    frac = state.r_draw.dc_texturemid
        + (state.r_draw.dc_yl as fixed_t - state.r_main.centery as fixed_t) * fracstep;
    loop {
        let src_pixel = read_source(
            state,
            state.r_draw.dc_source.unwrap(),
            frac >> FRACBITS & 127_i32,
        );
        state.i_video.I_VideoBuffer[idx] = state.r_data.colormaps
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
pub fn R_DrawColumnLow(state: &mut GameState) {
    let mut count: i32;
    let mut idx: usize;
    let mut idx2: usize;
    let mut frac: fixed_t;

    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0_i32 {
        return;
    }
    if state.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0_i32
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        I_Error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    let x: i32 = state.r_draw.dc_x << 1_i32;
    idx = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[x as usize] as usize;
    idx2 = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[(x + 1_i32) as usize] as usize;
    let fracstep: fixed_t = state.r_draw.dc_iscale;
    frac = state.r_draw.dc_texturemid
        + (state.r_draw.dc_yl as fixed_t - state.r_main.centery as fixed_t) * fracstep;
    loop {
        let src_pixel = read_source(
            state,
            state.r_draw.dc_source.unwrap(),
            frac >> FRACBITS & 127_i32,
        );
        let pixel = state.r_data.colormaps
            [(state.r_draw.dc_colormap.unwrap() * 256 + src_pixel as i32) as usize];
        state.i_video.I_VideoBuffer[idx] = pixel;
        state.i_video.I_VideoBuffer[idx2] = pixel;
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
pub static fuzzoffset: [i32; 50] = [
    FUZZOFF, -FUZZOFF, FUZZOFF, -FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF,
    FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, -FUZZOFF, -FUZZOFF,
    -FUZZOFF, FUZZOFF, -FUZZOFF, -FUZZOFF, FUZZOFF, FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, FUZZOFF,
    -FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, -FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, -FUZZOFF, -FUZZOFF,
    -FUZZOFF, FUZZOFF, FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, FUZZOFF, FUZZOFF, -FUZZOFF, FUZZOFF,
];
pub fn R_DrawFuzzColumn(state: &mut GameState) {
    let mut count: i32;
    let mut idx: usize;
    if state.r_draw.dc_yl == 0 {
        state.r_draw.dc_yl = 1_i32;
    }
    if state.r_draw.dc_yh == state.r_draw.viewheight - 1_i32 {
        state.r_draw.dc_yh = state.r_draw.viewheight - 2_i32;
    }
    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0_i32 {
        return;
    }
    if state.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0_i32
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        I_Error(&format!(
            "R_DrawFuzzColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    idx = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[state.r_draw.dc_x as usize] as usize;
    loop {
        let neighbor_idx =
            (idx as isize + fuzzoffset[state.r_draw.fuzzpos as usize] as isize) as usize;
        let neighbor = state.i_video.I_VideoBuffer[neighbor_idx];
        state.i_video.I_VideoBuffer[idx] =
            state.r_data.colormaps[(6_i32 * 256_i32 + neighbor as i32) as usize];
        state.r_draw.fuzzpos += 1;
        if state.r_draw.fuzzpos == FUZZTABLE {
            state.r_draw.fuzzpos = 0_i32;
        }
        idx += SCREENWIDTH as usize;
        let fresh2 = count;
        count -= 1;
        if fresh2 == 0 {
            break;
        }
    }
}
pub fn R_DrawFuzzColumnLow(state: &mut GameState) {
    let mut count: i32;
    let mut idx: usize;
    let mut idx2: usize;

    if state.r_draw.dc_yl == 0 {
        state.r_draw.dc_yl = 1_i32;
    }
    if state.r_draw.dc_yh == state.r_draw.viewheight - 1_i32 {
        state.r_draw.dc_yh = state.r_draw.viewheight - 2_i32;
    }
    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0_i32 {
        return;
    }
    let x: i32 = state.r_draw.dc_x << 1_i32;
    if x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0_i32
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        I_Error(&format!(
            "R_DrawFuzzColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    idx = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[x as usize] as usize;
    idx2 = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[(x + 1_i32) as usize] as usize;
    loop {
        let off = fuzzoffset[state.r_draw.fuzzpos as usize] as isize;
        let neighbor = state.i_video.I_VideoBuffer[(idx as isize + off) as usize];
        let neighbor2 = state.i_video.I_VideoBuffer[(idx2 as isize + off) as usize];
        state.i_video.I_VideoBuffer[idx] =
            state.r_data.colormaps[(6_i32 * 256_i32 + neighbor as i32) as usize];
        state.i_video.I_VideoBuffer[idx2] =
            state.r_data.colormaps[(6_i32 * 256_i32 + neighbor2 as i32) as usize];
        state.r_draw.fuzzpos += 1;
        if state.r_draw.fuzzpos == FUZZTABLE {
            state.r_draw.fuzzpos = 0_i32;
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
pub fn R_DrawTranslatedColumn(state: &mut GameState) {
    let mut count: i32;
    let mut idx: usize;
    let mut frac: fixed_t;

    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0_i32 {
        return;
    }
    if state.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0_i32
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        I_Error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    idx = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[state.r_draw.dc_x as usize] as usize;
    let fracstep: fixed_t = state.r_draw.dc_iscale;
    frac = state.r_draw.dc_texturemid
        + (state.r_draw.dc_yl as fixed_t - state.r_main.centery as fixed_t) * fracstep;
    loop {
        let raw_pixel = read_source(state, state.r_draw.dc_source.unwrap(), frac >> FRACBITS);
        let src_pixel =
            state.r_draw.translationtables[state.r_draw.dc_translation + raw_pixel as usize];
        state.i_video.I_VideoBuffer[idx] = state.r_data.colormaps
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
pub fn R_DrawTranslatedColumnLow(state: &mut GameState) {
    let mut count: i32;
    let mut idx: usize;
    let mut idx2: usize;
    let mut frac: fixed_t;

    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0_i32 {
        return;
    }
    let x: i32 = state.r_draw.dc_x << 1_i32;
    if x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0_i32
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        I_Error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, x
        ));
    }
    idx = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[x as usize] as usize;
    idx2 = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        + state.r_draw.columnofs[(x + 1_i32) as usize] as usize;
    let fracstep: fixed_t = state.r_draw.dc_iscale;
    frac = state.r_draw.dc_texturemid
        + (state.r_draw.dc_yl as fixed_t - state.r_main.centery as fixed_t) * fracstep;
    loop {
        let raw_pixel = read_source(state, state.r_draw.dc_source.unwrap(), frac >> FRACBITS);
        let src_pixel =
            state.r_draw.translationtables[state.r_draw.dc_translation + raw_pixel as usize];
        let colormap = state.r_draw.dc_colormap.unwrap();
        let pixel = state.r_data.colormaps[(colormap * 256 + src_pixel as i32) as usize];
        state.i_video.I_VideoBuffer[idx] = pixel;
        state.i_video.I_VideoBuffer[idx2] = pixel;
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
pub fn R_InitTranslationTables(state: &mut GameState) {
    let mut i: i32;
    state.r_draw.translationtables = vec![0u8; 256 * 3];
    i = 0_i32;
    while i < 256_i32 {
        if (0x70_i32..=0x7f_i32).contains(&i) {
            state.r_draw.translationtables[i as usize] = (0x60_i32 + (i & 0xf_i32)) as byte;
            state.r_draw.translationtables[(i + 256_i32) as usize] =
                (0x40_i32 + (i & 0xf_i32)) as byte;
            state.r_draw.translationtables[(i + 512_i32) as usize] =
                (0x20_i32 + (i & 0xf_i32)) as byte;
        } else {
            let fresh11 = i as byte;
            state.r_draw.translationtables[(i + 512_i32) as usize] = fresh11;
            let fresh12 = fresh11;
            state.r_draw.translationtables[(i + 256_i32) as usize] = fresh12;
            state.r_draw.translationtables[i as usize] = fresh12;
        }
        i += 1;
    }
}
pub fn R_DrawSpan(state: &mut GameState) {
    let mut position: u32;

    let mut idx: usize;
    let mut count: i32;
    let mut spot: i32;
    let mut xtemp: u32;
    let mut ytemp: u32;
    if state.r_draw.ds_x2 < state.r_draw.ds_x1
        || state.r_draw.ds_x1 < 0_i32
        || state.r_draw.ds_x2 >= SCREENWIDTH
        || state.r_draw.ds_y as u32 > SCREENHEIGHT as u32
    {
        I_Error(&format!(
            "R_DrawSpan: {} to {} at {}",
            state.r_draw.ds_x1, state.r_draw.ds_x2, state.r_draw.ds_y
        ));
    }
    position = (state.r_draw.ds_xfrac << 10_i32) as u32 & 0xffff0000_u32
        | (state.r_draw.ds_yfrac >> 6_i32 & 0xffff_i32) as u32;
    let step: u32 = (state.r_draw.ds_xstep << 10_i32) as u32 & 0xffff0000_u32
        | (state.r_draw.ds_ystep >> 6_i32 & 0xffff_i32) as u32;
    idx = state.r_draw.ylookup[state.r_draw.ds_y as usize]
        + state.r_draw.columnofs[state.r_draw.ds_x1 as usize] as usize;
    count = state.r_draw.ds_x2 - state.r_draw.ds_x1;
    loop {
        ytemp = position >> 4_i32 & 0xfc0_u32;
        xtemp = position >> 26_i32;
        spot = (xtemp | ytemp) as i32;
        let fresh6 = idx;
        idx += 1;
        let src_pixel = read_source(state, state.r_draw.ds_source.unwrap(), spot);
        state.i_video.I_VideoBuffer[fresh6] =
            state.r_data.colormaps[(state.r_draw.ds_colormap * 256 + src_pixel as i32) as usize];
        position = position.wrapping_add(step);
        let fresh7 = count;
        count -= 1;
        if fresh7 == 0 {
            break;
        }
    }
}
pub fn R_DrawSpanLow(state: &mut GameState) {
    let mut position: u32;

    let mut xtemp: u32;
    let mut ytemp: u32;
    let mut idx: usize;
    let mut count: i32;
    let mut spot: i32;
    if state.r_draw.ds_x2 < state.r_draw.ds_x1
        || state.r_draw.ds_x1 < 0_i32
        || state.r_draw.ds_x2 >= SCREENWIDTH
        || state.r_draw.ds_y as u32 > SCREENHEIGHT as u32
    {
        I_Error(&format!(
            "R_DrawSpan: {} to {} at {}",
            state.r_draw.ds_x1, state.r_draw.ds_x2, state.r_draw.ds_y
        ));
    }
    position = (state.r_draw.ds_xfrac << 10_i32) as u32 & 0xffff0000_u32
        | (state.r_draw.ds_yfrac >> 6_i32 & 0xffff_i32) as u32;
    let step: u32 = (state.r_draw.ds_xstep << 10_i32) as u32 & 0xffff0000_u32
        | (state.r_draw.ds_ystep >> 6_i32 & 0xffff_i32) as u32;
    count = state.r_draw.ds_x2 - state.r_draw.ds_x1;
    state.r_draw.ds_x1 <<= 1_i32;
    state.r_draw.ds_x2 <<= 1_i32;
    idx = state.r_draw.ylookup[state.r_draw.ds_y as usize]
        + state.r_draw.columnofs[state.r_draw.ds_x1 as usize] as usize;
    loop {
        ytemp = position >> 4_i32 & 0xfc0_u32;
        xtemp = position >> 26_i32;
        spot = (xtemp | ytemp) as i32;
        let fresh8 = idx;
        idx += 1;
        let src_pixel = read_source(state, state.r_draw.ds_source.unwrap(), spot);
        let pixel =
            state.r_data.colormaps[(state.r_draw.ds_colormap * 256 + src_pixel as i32) as usize];
        state.i_video.I_VideoBuffer[fresh8] = pixel;
        let fresh9 = idx;
        idx += 1;
        state.i_video.I_VideoBuffer[fresh9] = pixel;
        position = position.wrapping_add(step);
        let fresh10 = count;
        count -= 1;
        if fresh10 == 0 {
            break;
        }
    }
}
pub fn R_InitBuffer(state: &mut GameState, width: i32, height: i32) {
    let mut i: i32;
    state.r_draw.viewwindowx = (SCREENWIDTH - width) >> 1_i32;
    i = 0_i32;
    while i < width {
        state.r_draw.columnofs[i as usize] = state.r_draw.viewwindowx + i;
        i += 1;
    }
    if width == SCREENWIDTH {
        state.r_draw.viewwindowy = 0_i32;
    } else {
        state.r_draw.viewwindowy = (SCREENHEIGHT - SBARHEIGHT - height) >> 1_i32;
    }
    i = 0_i32;
    while i < height {
        state.r_draw.ylookup[i as usize] = ((i + state.r_draw.viewwindowy) * SCREENWIDTH) as usize;
        i += 1;
    }
}
pub fn R_FillBackScreen(state: &mut GameState) {
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
    let name: &str = if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        name2
    } else {
        name1
    };
    let flat = W_LumpBytesName(state, name);
    let background = state.r_draw.background_buffer.as_mut().unwrap();
    for y in 0..(SCREENHEIGHT - SBARHEIGHT) as usize {
        let row = &flat[(y & 63) << 6..][..64];
        let line = &mut background[y * SCREENWIDTH as usize..][..SCREENWIDTH as usize];
        for chunk in line.chunks_mut(64) {
            chunk.copy_from_slice(&row[..chunk.len()]);
        }
    }
    let backdrop = Screen::Background;
    patch = V_CachePatchName(state, "brdr_t");
    x = 0_i32;
    while x < state.r_draw.scaledviewwidth {
        V_DrawPatch(
            state,
            backdrop,
            state.r_draw.viewwindowx + x,
            state.r_draw.viewwindowy - 8_i32,
            &patch,
        );
        x += 8_i32;
    }
    patch = V_CachePatchName(state, "brdr_b");
    x = 0_i32;
    while x < state.r_draw.scaledviewwidth {
        V_DrawPatch(
            state,
            backdrop,
            state.r_draw.viewwindowx + x,
            state.r_draw.viewwindowy + state.r_draw.viewheight,
            &patch,
        );
        x += 8_i32;
    }
    patch = V_CachePatchName(state, "brdr_l");
    y = 0_i32;
    while y < state.r_draw.viewheight {
        V_DrawPatch(
            state,
            backdrop,
            state.r_draw.viewwindowx - 8_i32,
            state.r_draw.viewwindowy + y,
            &patch,
        );
        y += 8_i32;
    }
    patch = V_CachePatchName(state, "brdr_r");
    y = 0_i32;
    while y < state.r_draw.viewheight {
        V_DrawPatch(
            state,
            backdrop,
            state.r_draw.viewwindowx + state.r_draw.scaledviewwidth,
            state.r_draw.viewwindowy + y,
            &patch,
        );
        y += 8_i32;
    }
    let __wcache654_4 = V_CachePatchName(state, "brdr_tl");
    V_DrawPatch(
        state,
        backdrop,
        state.r_draw.viewwindowx - 8_i32,
        state.r_draw.viewwindowy - 8_i32,
        &__wcache654_4,
    );
    let __wcache660_3 = V_CachePatchName(state, "brdr_tr");
    V_DrawPatch(
        state,
        backdrop,
        state.r_draw.viewwindowx + state.r_draw.scaledviewwidth,
        state.r_draw.viewwindowy - 8_i32,
        &__wcache660_3,
    );
    let __wcache666_2 = V_CachePatchName(state, "brdr_bl");
    V_DrawPatch(
        state,
        backdrop,
        state.r_draw.viewwindowx - 8_i32,
        state.r_draw.viewwindowy + state.r_draw.viewheight,
        &__wcache666_2,
    );
    let __wcache672_1 = V_CachePatchName(state, "brdr_br");
    V_DrawPatch(
        state,
        backdrop,
        state.r_draw.viewwindowx + state.r_draw.scaledviewwidth,
        state.r_draw.viewwindowy + state.r_draw.viewheight,
        &__wcache672_1,
    );
}
pub fn R_VideoErase(state: &mut GameState, ofs: u32, count: i32) {
    if let Some(background_buffer) = &state.r_draw.background_buffer {
        let range = ofs as usize..ofs as usize + count as usize;
        state.i_video.I_VideoBuffer[range.clone()].copy_from_slice(&background_buffer[range]);
    }
}
pub fn R_DrawViewBorder(state: &mut GameState) {
    let mut side: i32;
    let mut ofs: i32;
    let mut i: i32;
    if state.r_draw.scaledviewwidth == SCREENWIDTH {
        return;
    }
    let top: i32 = (SCREENHEIGHT - SBARHEIGHT - state.r_draw.viewheight) / 2_i32;
    side = (SCREENWIDTH - state.r_draw.scaledviewwidth) / 2_i32;
    R_VideoErase(state, 0_u32, top * SCREENWIDTH + side);
    ofs = (state.r_draw.viewheight + top) * SCREENWIDTH - side;
    R_VideoErase(state, ofs as u32, top * SCREENWIDTH + side);
    ofs = top * SCREENWIDTH + SCREENWIDTH - side;
    side <<= 1_i32;
    i = 1_i32;
    while i < state.r_draw.viewheight {
        R_VideoErase(state, ofs as u32, side);
        ofs += SCREENWIDTH;
        i += 1;
    }
    let dest_screen = Screen::Video;
    V_MarkRect(
        state,
        dest_screen,
        0_i32,
        0_i32,
        SCREENWIDTH,
        SCREENHEIGHT - SBARHEIGHT,
    );
}
