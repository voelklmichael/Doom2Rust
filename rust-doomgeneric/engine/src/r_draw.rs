use crate::d_mode::GameMode_t;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::hu_lib::patch_t;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FRACBITS;
use crate::r_defs::lighttable_t;
use crate::stdint_types::byte;
use crate::stdint_types::size_t;
use crate::v_video::V_DrawPatch;
use crate::v_video::V_MarkRect;
use crate::v_video::V_RestoreBuffer;
use crate::v_video::V_UseBuffer;
use crate::w_wad::W_CacheLumpName;
use crate::mem_compat::memcpy;

pub struct RDrawState {
    pub viewimage: *mut byte,
    pub viewwidth: i32,
    pub scaledviewwidth: i32,
    pub viewheight: i32,
    pub viewwindowx: i32,
    pub viewwindowy: i32,
    pub ylookup: [*mut byte; 832],
    pub columnofs: [i32; 1120],
    pub background_buffer: Option<Vec<byte>>,
    pub dc_colormap: *mut lighttable_t,
    pub dc_x: i32,
    pub dc_yl: i32,
    pub dc_yh: i32,
    pub dc_iscale: fixed_t,
    pub dc_texturemid: fixed_t,
    pub dc_source: *mut byte,
    pub dccount: i32,
    pub fuzzpos: i32,
    pub dc_translation: *mut byte,
    pub translationtables: Vec<byte>,
    pub ds_y: i32,
    pub ds_x1: i32,
    pub ds_x2: i32,
    pub ds_colormap: *mut lighttable_t,
    pub ds_xfrac: fixed_t,
    pub ds_yfrac: fixed_t,
    pub ds_xstep: fixed_t,
    pub ds_ystep: fixed_t,
    pub ds_source: *mut byte,
    pub dscount: i32,
}

impl RDrawState {
    pub const fn new() -> Self {
        RDrawState {
            viewimage: ::core::ptr::null::<byte>() as *mut byte,
            viewwidth: 0,
            scaledviewwidth: 0,
            viewheight: 0,
            viewwindowx: 0,
            viewwindowy: 0,
            ylookup: [::core::ptr::null::<byte>() as *mut byte; 832],
            columnofs: [0; 1120],
            background_buffer: None,
            dc_colormap: ::core::ptr::null::<lighttable_t>() as *mut lighttable_t,
            dc_x: 0,
            dc_yl: 0,
            dc_yh: 0,
            dc_iscale: 0,
            dc_texturemid: 0,
            dc_source: ::core::ptr::null::<byte>() as *mut byte,
            dccount: 0,
            fuzzpos: 0,
            dc_translation: ::core::ptr::null::<byte>() as *mut byte,
            translationtables: Vec::new(),
            ds_y: 0,
            ds_x1: 0,
            ds_x2: 0,
            ds_colormap: ::core::ptr::null::<lighttable_t>() as *mut lighttable_t,
            ds_xfrac: 0,
            ds_yfrac: 0,
            ds_xstep: 0,
            ds_ystep: 0,
            ds_source: ::core::ptr::null::<byte>() as *mut byte,
            dscount: 0,
        }
    }
}

pub const SBARHEIGHT: i32 = 32;
pub static translations: [[byte; 256]; 3] = [[0; 256]; 3];
pub unsafe fn R_DrawColumn(state: &mut GameState) {
    let mut count: i32 = 0;
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut frac: fixed_t = 0;
    let mut fracstep: fixed_t = 0;
    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0 as i32 {
        return;
    }
    if state.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0 as i32
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        I_Error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    dest = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        .offset(state.r_draw.columnofs[state.r_draw.dc_x as usize] as isize);
    fracstep = state.r_draw.dc_iscale;
    frac = state.r_draw.dc_texturemid
        + (state.r_draw.dc_yl as fixed_t - state.r_main.centery as fixed_t) * fracstep;
    loop {
        *dest = *state.r_draw.dc_colormap.offset(
            *state
                .r_draw
                .dc_source
                .offset((frac as i32 >> FRACBITS & 127 as i32) as isize) as isize,
        ) as byte;
        dest = dest.offset(SCREENWIDTH as isize);
        frac += fracstep;
        let fresh0 = count;
        count = count - 1;
        if !(fresh0 != 0) {
            break;
        }
    }
}
pub unsafe fn R_DrawColumnLow(state: &mut GameState) {
    let mut count: i32 = 0;
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut dest2: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut frac: fixed_t = 0;
    let mut fracstep: fixed_t = 0;
    let mut x: i32 = 0;
    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0 as i32 {
        return;
    }
    if state.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0 as i32
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        I_Error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    x = state.r_draw.dc_x << 1 as i32;
    dest = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        .offset(state.r_draw.columnofs[x as usize] as isize);
    dest2 = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        .offset(state.r_draw.columnofs[(x + 1 as i32) as usize] as isize);
    fracstep = state.r_draw.dc_iscale;
    frac = state.r_draw.dc_texturemid
        + (state.r_draw.dc_yl as fixed_t - state.r_main.centery as fixed_t) * fracstep;
    loop {
        *dest = *state.r_draw.dc_colormap.offset(
            *state
                .r_draw
                .dc_source
                .offset((frac as i32 >> FRACBITS & 127 as i32) as isize) as isize,
        ) as byte;
        *dest2 = *dest;
        dest = dest.offset(SCREENWIDTH as isize);
        dest2 = dest2.offset(SCREENWIDTH as isize);
        frac += fracstep;
        let fresh1 = count;
        count = count - 1;
        if !(fresh1 != 0) {
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
pub unsafe fn R_DrawFuzzColumn(state: &mut GameState) {
    let mut count: i32 = 0;
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    if state.r_draw.dc_yl == 0 {
        state.r_draw.dc_yl = 1 as i32;
    }
    if state.r_draw.dc_yh == state.r_draw.viewheight - 1 as i32 {
        state.r_draw.dc_yh = state.r_draw.viewheight - 2 as i32;
    }
    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0 as i32 {
        return;
    }
    if state.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0 as i32
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        I_Error(&format!(
            "R_DrawFuzzColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    dest = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        .offset(state.r_draw.columnofs[state.r_draw.dc_x as usize] as isize);
    loop {
        *dest = state.r_data.colormaps[(6 as i32 * 256 as i32
            + *dest.offset(fuzzoffset[state.r_draw.fuzzpos as usize] as isize) as i32)
            as usize];
        state.r_draw.fuzzpos += 1;
        if state.r_draw.fuzzpos == FUZZTABLE {
            state.r_draw.fuzzpos = 0 as i32;
        }
        dest = dest.offset(SCREENWIDTH as isize);
        let fresh2 = count;
        count = count - 1;
        if !(fresh2 != 0) {
            break;
        }
    }
}
pub unsafe fn R_DrawFuzzColumnLow(state: &mut GameState) {
    let mut count: i32 = 0;
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut dest2: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut x: i32 = 0;
    if state.r_draw.dc_yl == 0 {
        state.r_draw.dc_yl = 1 as i32;
    }
    if state.r_draw.dc_yh == state.r_draw.viewheight - 1 as i32 {
        state.r_draw.dc_yh = state.r_draw.viewheight - 2 as i32;
    }
    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0 as i32 {
        return;
    }
    x = state.r_draw.dc_x << 1 as i32;
    if x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0 as i32
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        I_Error(&format!(
            "R_DrawFuzzColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    dest = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        .offset(state.r_draw.columnofs[x as usize] as isize);
    dest2 = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        .offset(state.r_draw.columnofs[(x + 1 as i32) as usize] as isize);
    loop {
        *dest = state.r_data.colormaps[(6 as i32 * 256 as i32
            + *dest.offset(fuzzoffset[state.r_draw.fuzzpos as usize] as isize) as i32)
            as usize];
        *dest2 = state.r_data.colormaps[(6 as i32 * 256 as i32
            + *dest2.offset(fuzzoffset[state.r_draw.fuzzpos as usize] as isize) as i32)
            as usize];
        state.r_draw.fuzzpos += 1;
        if state.r_draw.fuzzpos == FUZZTABLE {
            state.r_draw.fuzzpos = 0 as i32;
        }
        dest = dest.offset(SCREENWIDTH as isize);
        dest2 = dest2.offset(SCREENWIDTH as isize);
        let fresh3 = count;
        count = count - 1;
        if !(fresh3 != 0) {
            break;
        }
    }
}
pub unsafe fn R_DrawTranslatedColumn(state: &mut GameState) {
    let mut count: i32 = 0;
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut frac: fixed_t = 0;
    let mut fracstep: fixed_t = 0;
    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0 as i32 {
        return;
    }
    if state.r_draw.dc_x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0 as i32
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        I_Error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, state.r_draw.dc_x
        ));
    }
    dest = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        .offset(state.r_draw.columnofs[state.r_draw.dc_x as usize] as isize);
    fracstep = state.r_draw.dc_iscale;
    frac = state.r_draw.dc_texturemid
        + (state.r_draw.dc_yl as fixed_t - state.r_main.centery as fixed_t) * fracstep;
    loop {
        *dest = *state.r_draw.dc_colormap.offset(
            *state
                .r_draw
                .dc_translation
                .offset(*state.r_draw.dc_source.offset((frac >> FRACBITS) as isize) as isize)
                as isize,
        ) as byte;
        dest = dest.offset(SCREENWIDTH as isize);
        frac += fracstep;
        let fresh4 = count;
        count = count - 1;
        if !(fresh4 != 0) {
            break;
        }
    }
}
pub unsafe fn R_DrawTranslatedColumnLow(state: &mut GameState) {
    let mut count: i32 = 0;
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut dest2: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut frac: fixed_t = 0;
    let mut fracstep: fixed_t = 0;
    let mut x: i32 = 0;
    count = state.r_draw.dc_yh - state.r_draw.dc_yl;
    if count < 0 as i32 {
        return;
    }
    x = state.r_draw.dc_x << 1 as i32;
    if x as u32 >= SCREENWIDTH as u32
        || state.r_draw.dc_yl < 0 as i32
        || state.r_draw.dc_yh >= SCREENHEIGHT
    {
        I_Error(&format!(
            "R_DrawColumn: {} to {} at {}",
            state.r_draw.dc_yl, state.r_draw.dc_yh, x
        ));
    }
    dest = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        .offset(state.r_draw.columnofs[x as usize] as isize);
    dest2 = state.r_draw.ylookup[state.r_draw.dc_yl as usize]
        .offset(state.r_draw.columnofs[(x + 1 as i32) as usize] as isize);
    fracstep = state.r_draw.dc_iscale;
    frac = state.r_draw.dc_texturemid
        + (state.r_draw.dc_yl as fixed_t - state.r_main.centery as fixed_t) * fracstep;
    loop {
        *dest = *state.r_draw.dc_colormap.offset(
            *state
                .r_draw
                .dc_translation
                .offset(*state.r_draw.dc_source.offset((frac >> FRACBITS) as isize) as isize)
                as isize,
        ) as byte;
        *dest2 = *state.r_draw.dc_colormap.offset(
            *state
                .r_draw
                .dc_translation
                .offset(*state.r_draw.dc_source.offset((frac >> FRACBITS) as isize) as isize)
                as isize,
        ) as byte;
        dest = dest.offset(SCREENWIDTH as isize);
        dest2 = dest2.offset(SCREENWIDTH as isize);
        frac += fracstep;
        let fresh5 = count;
        count = count - 1;
        if !(fresh5 != 0) {
            break;
        }
    }
}
pub fn R_InitTranslationTables(state: &mut GameState) {
    let mut i: i32 = 0;
    state.r_draw.translationtables = vec![0u8; 256 * 3];
    i = 0 as i32;
    while i < 256 as i32 {
        if i >= 0x70 as i32 && i <= 0x7f as i32 {
            state.r_draw.translationtables[i as usize] = (0x60 as i32 + (i & 0xf as i32)) as byte;
            state.r_draw.translationtables[(i + 256 as i32) as usize] =
                (0x40 as i32 + (i & 0xf as i32)) as byte;
            state.r_draw.translationtables[(i + 512 as i32) as usize] =
                (0x20 as i32 + (i & 0xf as i32)) as byte;
        } else {
            let fresh11 = i as byte;
            state.r_draw.translationtables[(i + 512 as i32) as usize] = fresh11;
            let fresh12 = fresh11;
            state.r_draw.translationtables[(i + 256 as i32) as usize] = fresh12;
            state.r_draw.translationtables[i as usize] = fresh12;
        }
        i += 1;
    }
}
pub unsafe fn R_DrawSpan(state: &mut GameState) {
    let mut position: u32 = 0;
    let mut step: u32 = 0;
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut count: i32 = 0;
    let mut spot: i32 = 0;
    let mut xtemp: u32 = 0;
    let mut ytemp: u32 = 0;
    if state.r_draw.ds_x2 < state.r_draw.ds_x1
        || state.r_draw.ds_x1 < 0 as i32
        || state.r_draw.ds_x2 >= SCREENWIDTH
        || state.r_draw.ds_y as u32 > SCREENHEIGHT as u32
    {
        I_Error(&format!(
            "R_DrawSpan: {} to {} at {}",
            state.r_draw.ds_x1, state.r_draw.ds_x2, state.r_draw.ds_y
        ));
    }
    position = (state.r_draw.ds_xfrac << 10 as i32) as u32 & 0xffff0000 as u32
        | (state.r_draw.ds_yfrac as i32 >> 6 as i32 & 0xffff as i32) as u32;
    step = (state.r_draw.ds_xstep << 10 as i32) as u32 & 0xffff0000 as u32
        | (state.r_draw.ds_ystep as i32 >> 6 as i32 & 0xffff as i32) as u32;
    dest = state.r_draw.ylookup[state.r_draw.ds_y as usize]
        .offset(state.r_draw.columnofs[state.r_draw.ds_x1 as usize] as isize);
    count = state.r_draw.ds_x2 - state.r_draw.ds_x1;
    loop {
        ytemp = position >> 4 as i32 & 0xfc0 as u32;
        xtemp = position >> 26 as i32;
        spot = (xtemp | ytemp) as i32;
        let fresh6 = dest;
        dest = dest.offset(1);
        *fresh6 = *state
            .r_draw
            .ds_colormap
            .offset(*state.r_draw.ds_source.offset(spot as isize) as isize)
            as byte;
        position = position.wrapping_add(step);
        let fresh7 = count;
        count = count - 1;
        if !(fresh7 != 0) {
            break;
        }
    }
}
pub unsafe fn R_DrawSpanLow(state: &mut GameState) {
    let mut position: u32 = 0;
    let mut step: u32 = 0;
    let mut xtemp: u32 = 0;
    let mut ytemp: u32 = 0;
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut count: i32 = 0;
    let mut spot: i32 = 0;
    if state.r_draw.ds_x2 < state.r_draw.ds_x1
        || state.r_draw.ds_x1 < 0 as i32
        || state.r_draw.ds_x2 >= SCREENWIDTH
        || state.r_draw.ds_y as u32 > SCREENHEIGHT as u32
    {
        I_Error(&format!(
            "R_DrawSpan: {} to {} at {}",
            state.r_draw.ds_x1, state.r_draw.ds_x2, state.r_draw.ds_y
        ));
    }
    position = (state.r_draw.ds_xfrac << 10 as i32) as u32 & 0xffff0000 as u32
        | (state.r_draw.ds_yfrac as i32 >> 6 as i32 & 0xffff as i32) as u32;
    step = (state.r_draw.ds_xstep << 10 as i32) as u32 & 0xffff0000 as u32
        | (state.r_draw.ds_ystep as i32 >> 6 as i32 & 0xffff as i32) as u32;
    count = state.r_draw.ds_x2 - state.r_draw.ds_x1;
    state.r_draw.ds_x1 <<= 1 as i32;
    state.r_draw.ds_x2 <<= 1 as i32;
    dest = state.r_draw.ylookup[state.r_draw.ds_y as usize]
        .offset(state.r_draw.columnofs[state.r_draw.ds_x1 as usize] as isize);
    loop {
        ytemp = position >> 4 as i32 & 0xfc0 as u32;
        xtemp = position >> 26 as i32;
        spot = (xtemp | ytemp) as i32;
        let fresh8 = dest;
        dest = dest.offset(1);
        *fresh8 = *state
            .r_draw
            .ds_colormap
            .offset(*state.r_draw.ds_source.offset(spot as isize) as isize)
            as byte;
        let fresh9 = dest;
        dest = dest.offset(1);
        *fresh9 = *state
            .r_draw
            .ds_colormap
            .offset(*state.r_draw.ds_source.offset(spot as isize) as isize)
            as byte;
        position = position.wrapping_add(step);
        let fresh10 = count;
        count = count - 1;
        if !(fresh10 != 0) {
            break;
        }
    }
}
pub unsafe fn R_InitBuffer(state: &mut GameState, mut width: i32, mut height: i32) {
    let mut i: i32 = 0;
    state.r_draw.viewwindowx = SCREENWIDTH - width >> 1 as i32;
    i = 0 as i32;
    while i < width {
        state.r_draw.columnofs[i as usize] = state.r_draw.viewwindowx + i;
        i += 1;
    }
    if width == SCREENWIDTH {
        state.r_draw.viewwindowy = 0 as i32;
    } else {
        state.r_draw.viewwindowy = SCREENHEIGHT - SBARHEIGHT - height >> 1 as i32;
    }
    i = 0 as i32;
    while i < height {
        state.r_draw.ylookup[i as usize] = state
            .i_video
            .I_VideoBuffer
            .as_mut_ptr()
            .offset(((i + state.r_draw.viewwindowy) * SCREENWIDTH) as isize);
        i += 1;
    }
}
pub unsafe fn R_FillBackScreen(state: &mut GameState) {
    let mut src: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut patch: *mut patch_t = ::core::ptr::null_mut::<patch_t>();
    let name1: &str = "FLOOR7_2";
    let name2: &str = "GRNROCK";
    let name: &str;
    if state.r_draw.scaledviewwidth == SCREENWIDTH {
        state.r_draw.background_buffer = None;
        return;
    }
    if state.r_draw.background_buffer.is_none() {
        state.r_draw.background_buffer =
            Some(vec![0u8; (SCREENWIDTH * (SCREENHEIGHT - SBARHEIGHT)) as usize]);
    }
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        name = name2;
    } else {
        name = name1;
    }
    src = W_CacheLumpName(state, name) as *mut byte;
    dest = state.r_draw.background_buffer.as_mut().unwrap().as_mut_ptr();
    y = 0 as i32;
    while y < SCREENHEIGHT - SBARHEIGHT {
        x = 0 as i32;
        while x < SCREENWIDTH / 64 as i32 {
            memcpy(
                dest as *mut ::core::ffi::c_void,
                src.offset(((y & 63 as i32) << 6 as i32) as isize) as *const ::core::ffi::c_void,
                64 as size_t,
            );
            dest = dest.offset(64 as i32 as isize);
            x += 1;
        }
        if SCREENWIDTH & 63 as i32 != 0 {
            memcpy(
                dest as *mut ::core::ffi::c_void,
                src.offset(((y & 63 as i32) << 6 as i32) as isize) as *const ::core::ffi::c_void,
                (SCREENWIDTH & 63 as i32) as size_t,
            );
            dest = dest.offset((SCREENWIDTH & 63 as i32) as isize);
        }
        y += 1;
    }
    V_UseBuffer(
        &mut state.v_video,
        state.r_draw.background_buffer.as_mut().unwrap().as_mut_ptr(),
    );
    patch = W_CacheLumpName(state, "brdr_t") as *mut patch_t;
    x = 0 as i32;
    while x < state.r_draw.scaledviewwidth {
        V_DrawPatch(state,
            state.r_draw.viewwindowx + x,
            state.r_draw.viewwindowy - 8 as i32,
            patch,
        );
        x += 8 as i32;
    }
    patch = W_CacheLumpName(state, "brdr_b") as *mut patch_t;
    x = 0 as i32;
    while x < state.r_draw.scaledviewwidth {
        V_DrawPatch(state,
            state.r_draw.viewwindowx + x,
            state.r_draw.viewwindowy + state.r_draw.viewheight,
            patch,
        );
        x += 8 as i32;
    }
    patch = W_CacheLumpName(state, "brdr_l") as *mut patch_t;
    y = 0 as i32;
    while y < state.r_draw.viewheight {
        V_DrawPatch(state,
            state.r_draw.viewwindowx - 8 as i32,
            state.r_draw.viewwindowy + y,
            patch,
        );
        y += 8 as i32;
    }
    patch = W_CacheLumpName(state, "brdr_r") as *mut patch_t;
    y = 0 as i32;
    while y < state.r_draw.viewheight {
        V_DrawPatch(state,
            state.r_draw.viewwindowx + state.r_draw.scaledviewwidth,
            state.r_draw.viewwindowy + y,
            patch,
        );
        y += 8 as i32;
    }
    let __wcache654_4 = W_CacheLumpName(state, "brdr_tl") as *mut patch_t;
    V_DrawPatch(state,
        state.r_draw.viewwindowx - 8 as i32,
        state.r_draw.viewwindowy - 8 as i32,
        __wcache654_4,
    );
    let __wcache660_3 = W_CacheLumpName(state, "brdr_tr") as *mut patch_t;
    V_DrawPatch(state,
        state.r_draw.viewwindowx + state.r_draw.scaledviewwidth,
        state.r_draw.viewwindowy - 8 as i32,
        __wcache660_3,
    );
    let __wcache666_2 = W_CacheLumpName(state, "brdr_bl") as *mut patch_t;
    V_DrawPatch(state,
        state.r_draw.viewwindowx - 8 as i32,
        state.r_draw.viewwindowy + state.r_draw.viewheight,
        __wcache666_2,
    );
    let __wcache672_1 = W_CacheLumpName(state, "brdr_br") as *mut patch_t;
    V_DrawPatch(state,
        state.r_draw.viewwindowx + state.r_draw.scaledviewwidth,
        state.r_draw.viewwindowy + state.r_draw.viewheight,
        __wcache672_1,
    );
    V_RestoreBuffer(state);
}
pub unsafe fn R_VideoErase(state: &mut GameState, mut ofs: u32, mut count: i32) {
    if let Some(background_buffer) = &state.r_draw.background_buffer {
        memcpy(
            state.i_video.I_VideoBuffer.as_mut_ptr().offset(ofs as isize) as *mut ::core::ffi::c_void,
            background_buffer.as_ptr().offset(ofs as isize) as *const ::core::ffi::c_void,
            count as size_t,
        );
    }
}
pub unsafe fn R_DrawViewBorder(state: &mut GameState) {
    let mut top: i32 = 0;
    let mut side: i32 = 0;
    let mut ofs: i32 = 0;
    let mut i: i32 = 0;
    if state.r_draw.scaledviewwidth == SCREENWIDTH {
        return;
    }
    top = (SCREENHEIGHT - SBARHEIGHT - state.r_draw.viewheight) / 2 as i32;
    side = (SCREENWIDTH - state.r_draw.scaledviewwidth) / 2 as i32;
    R_VideoErase(state, 0 as u32, top * SCREENWIDTH + side);
    ofs = (state.r_draw.viewheight + top) * SCREENWIDTH - side;
    R_VideoErase(state, ofs as u32, top * SCREENWIDTH + side);
    ofs = top * SCREENWIDTH + SCREENWIDTH - side;
    side <<= 1 as i32;
    i = 1 as i32;
    while i < state.r_draw.viewheight {
        R_VideoErase(state, ofs as u32, side);
        ofs += SCREENWIDTH;
        i += 1;
    }
    V_MarkRect(state,
        0 as i32,
        0 as i32,
        SCREENWIDTH,
        SCREENHEIGHT - SBARHEIGHT,
    );
}
