use crate::src::doomdef::SCREENHEIGHT;
use crate::src::doomdef::SCREENWIDTH;
use crate::src::game_state::GameState;
use crate::src::hu_lib::patch_t;
use crate::src::i_system::I_Error;
use crate::src::i_video::IVideoState;
use crate::src::i_video::I_GetPaletteIndex;
use crate::src::m_bbox::M_AddToBox;
use crate::src::m_fixed::fixed_t;
use crate::src::m_misc::M_FileExists;
use crate::src::m_misc::M_WriteFile;
use crate::src::r_data::column_t;
use crate::src::stdint_types::size_t;
use crate::src::stdint_types::{byte, uint8_t};
use crate::src::w_wad::W_CacheLumpName;
use crate::src::w_wad::W_CacheLumpNum;
use crate::src::w_wad::W_GetNumForName;
use crate::src::w_wad::W_LumpLength;
use crate::src::mem_compat::memcpy;

pub type vpatchclipfunc_t = Option<unsafe fn(*mut patch_t, i32, i32) -> bool>;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct pcx_t {
    pub manufacturer: u8,
    pub version: u8,
    pub encoding: u8,
    pub bits_per_pixel: u8,
    pub xmin: u16,
    pub ymin: u16,
    pub xmax: u16,
    pub ymax: u16,
    pub hres: u16,
    pub vres: u16,
    pub palette: [u8; 48],
    pub reserved: u8,
    pub color_planes: u8,
    pub bytes_per_line: u16,
    pub palette_type: u16,
    pub filler: [u8; 58],
    pub data: u8,
}
pub struct VVideoState {
    pub tinttable: Vec<byte>,
    pub xlatab: Vec<byte>,
    pub dest_screen: *mut byte,
    pub dirtybox: [i32; 4],
    // No caller anywhere in the repo ever invokes V_SetPatchClipCallback, so
    // this is permanently None -- kept as-is (dead stub) rather than deleted
    // as a drive-by, same rationale as other vestigial fields in this crate.
    patchclip_callback: vpatchclipfunc_t,
}
impl VVideoState {
    pub const fn new() -> Self {
        VVideoState {
            tinttable: Vec::new(),
            xlatab: Vec::new(),
            dest_screen: ::core::ptr::null_mut::<byte>(),
            dirtybox: [0; 4],
            patchclip_callback: None,
        }
    }
}
pub unsafe fn V_MarkRect(state: &mut GameState, mut x: i32, mut y: i32, mut width: i32, mut height: i32) {
    if state.v_video.dest_screen == state.i_video.I_VideoBuffer.as_mut_ptr() {
        M_AddToBox(
            &raw mut state.v_video.dirtybox as *mut fixed_t,
            x as fixed_t,
            y as fixed_t,
        );
        M_AddToBox(
            &raw mut state.v_video.dirtybox as *mut fixed_t,
            x as fixed_t + width as fixed_t - 1 as fixed_t,
            y as fixed_t + height as fixed_t - 1 as fixed_t,
        );
    }
}
pub unsafe fn V_CopyRect(
    state: &mut GameState,
    mut srcx: i32,
    mut srcy: i32,
    mut source: *mut byte,
    mut width: i32,
    mut height: i32,
    mut destx: i32,
    mut desty: i32,
) {
    let mut src: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    if srcx < 0 as i32
        || srcx + width > SCREENWIDTH
        || srcy < 0 as i32
        || srcy + height > SCREENHEIGHT
        || destx < 0 as i32
        || destx + width > SCREENWIDTH
        || desty < 0 as i32
        || desty + height > SCREENHEIGHT
    {
        I_Error("Bad V_CopyRect");
    }
    V_MarkRect(state, destx, desty, width, height);
    src = source
        .offset((SCREENWIDTH * srcy) as isize)
        .offset(srcx as isize);
    dest = state
        .v_video
        .dest_screen
        .offset((SCREENWIDTH * desty) as isize)
        .offset(destx as isize);
    while height > 0 as i32 {
        memcpy(
            dest as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            width as size_t,
        );
        src = src.offset(SCREENWIDTH as isize);
        dest = dest.offset(SCREENWIDTH as isize);
        height -= 1;
    }
}
pub fn V_SetPatchClipCallback(state: &mut GameState, mut func: vpatchclipfunc_t) {
    state.v_video.patchclip_callback = func;
}
pub unsafe fn V_DrawPatch(state: &mut GameState, mut x: i32, mut y: i32, mut patch: *mut patch_t) {
    let mut count: i32 = 0;
    let mut col: i32 = 0;
    let mut column: *mut column_t = ::core::ptr::null_mut::<column_t>();
    let mut desttop: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut source: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut w: i32 = 0;
    y -= (*patch).topoffset as i32;
    x -= (*patch).leftoffset as i32;
    if state.v_video.patchclip_callback.is_some() {
        if !state.v_video.patchclip_callback.expect("non-null function pointer")(patch, x, y) {
            return;
        }
    }
    if x < 0 as i32
        || x + (*patch).width as i32 > SCREENWIDTH
        || y < 0 as i32
        || y + (*patch).height as i32 > SCREENHEIGHT
    {
        I_Error(&format!(
            "Bad V_DrawPatch x={} y={} patch.width={} patch.height={} topoffset={} leftoffset={}",
            x,
            y,
            (*patch).width as i32,
            (*patch).height as i32,
            (*patch).topoffset as i32,
            (*patch).leftoffset as i32,
        ));
    }
    V_MarkRect(state, x, y, (*patch).width as i32, (*patch).height as i32);
    col = 0 as i32;
    desttop = state
        .v_video
        .dest_screen
        .offset((y * SCREENWIDTH) as isize)
        .offset(x as isize);
    w = (*patch).width as i32;
    while col < w {
        column = (patch as *mut byte)
            .offset(*(&raw const (*patch).columnofs as *const i32).offset(col as isize) as isize)
            as *mut column_t;
        while (*column).topdelta as i32 != 0xff as i32 {
            source = (column as *mut byte).offset(3 as i32 as isize);
            dest = desttop.offset(((*column).topdelta as i32 * SCREENWIDTH) as isize);
            count = (*column).length as i32;
            loop {
                let fresh0 = count;
                count = count - 1;
                if !(fresh0 != 0) {
                    break;
                }
                let fresh1 = source;
                source = source.offset(1);
                *dest = *fresh1;
                dest = dest.offset(SCREENWIDTH as isize);
            }
            column = (column as *mut byte)
                .offset((*column).length as i32 as isize)
                .offset(4 as i32 as isize) as *mut column_t;
        }
        x += 1;
        col += 1;
        desttop = desttop.offset(1);
    }
}
pub unsafe fn V_DrawPatchFlipped(
    state: &mut GameState,
    mut x: i32,
    mut y: i32,
    mut patch: *mut patch_t,
) {
    let mut count: i32 = 0;
    let mut col: i32 = 0;
    let mut column: *mut column_t = ::core::ptr::null_mut::<column_t>();
    let mut desttop: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut source: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut w: i32 = 0;
    y -= (*patch).topoffset as i32;
    x -= (*patch).leftoffset as i32;
    if state.v_video.patchclip_callback.is_some() {
        if !state.v_video.patchclip_callback.expect("non-null function pointer")(patch, x, y) {
            return;
        }
    }
    if x < 0 as i32
        || x + (*patch).width as i32 > SCREENWIDTH
        || y < 0 as i32
        || y + (*patch).height as i32 > SCREENHEIGHT
    {
        I_Error("Bad V_DrawPatchFlipped");
    }
    V_MarkRect(state, x, y, (*patch).width as i32, (*patch).height as i32);
    col = 0 as i32;
    desttop = state
        .v_video
        .dest_screen
        .offset((y * SCREENWIDTH) as isize)
        .offset(x as isize);
    w = (*patch).width as i32;
    while col < w {
        column = (patch as *mut byte).offset(
            *(&raw const (*patch).columnofs as *const i32).offset((w - 1 as i32 - col) as isize)
                as isize,
        ) as *mut column_t;
        while (*column).topdelta as i32 != 0xff as i32 {
            source = (column as *mut byte).offset(3 as i32 as isize);
            dest = desttop.offset(((*column).topdelta as i32 * SCREENWIDTH) as isize);
            count = (*column).length as i32;
            loop {
                let fresh2 = count;
                count = count - 1;
                if !(fresh2 != 0) {
                    break;
                }
                let fresh3 = source;
                source = source.offset(1);
                *dest = *fresh3;
                dest = dest.offset(SCREENWIDTH as isize);
            }
            column = (column as *mut byte)
                .offset((*column).length as i32 as isize)
                .offset(4 as i32 as isize) as *mut column_t;
        }
        x += 1;
        col += 1;
        desttop = desttop.offset(1);
    }
}
pub unsafe fn V_DrawPatchDirect(
    state: &mut GameState,
    mut x: i32,
    mut y: i32,
    mut patch: *mut patch_t,
) {
    V_DrawPatch(state, x, y, patch);
}
pub unsafe fn V_DrawTLPatch(
    state: &mut VVideoState,
    mut x: i32,
    mut y: i32,
    mut patch: *mut patch_t,
) {
    let mut count: i32 = 0;
    let mut col: i32 = 0;
    let mut column: *mut column_t = ::core::ptr::null_mut::<column_t>();
    let mut desttop: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut source: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut w: i32 = 0;
    y -= (*patch).topoffset as i32;
    x -= (*patch).leftoffset as i32;
    if x < 0 as i32
        || x + (*patch).width as i32 > SCREENWIDTH
        || y < 0 as i32
        || y + (*patch).height as i32 > SCREENHEIGHT
    {
        I_Error("Bad V_DrawTLPatch");
    }
    col = 0 as i32;
    desttop = state
        .dest_screen
        .offset((y * SCREENWIDTH) as isize)
        .offset(x as isize);
    w = (*patch).width as i32;
    while col < w {
        column = (patch as *mut byte)
            .offset(*(&raw const (*patch).columnofs as *const i32).offset(col as isize) as isize)
            as *mut column_t;
        while (*column).topdelta as i32 != 0xff as i32 {
            source = (column as *mut byte).offset(3 as i32 as isize);
            dest = desttop.offset(((*column).topdelta as i32 * SCREENWIDTH) as isize);
            count = (*column).length as i32;
            loop {
                let fresh4 = count;
                count = count - 1;
                if !(fresh4 != 0) {
                    break;
                }
                let fresh5 = source;
                source = source.offset(1);
                *dest = state.tinttable[(((*dest as i32) << 8 as i32) + *fresh5 as i32) as usize];
                dest = dest.offset(SCREENWIDTH as isize);
            }
            column = (column as *mut byte)
                .offset((*column).length as i32 as isize)
                .offset(4 as i32 as isize) as *mut column_t;
        }
        x += 1;
        col += 1;
        desttop = desttop.offset(1);
    }
}
pub unsafe fn V_DrawXlaPatch(
    state: &mut VVideoState,
    mut x: i32,
    mut y: i32,
    mut patch: *mut patch_t,
) {
    let mut count: i32 = 0;
    let mut col: i32 = 0;
    let mut column: *mut column_t = ::core::ptr::null_mut::<column_t>();
    let mut desttop: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut source: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut w: i32 = 0;
    y -= (*patch).topoffset as i32;
    x -= (*patch).leftoffset as i32;
    if state.patchclip_callback.is_some() {
        if !state.patchclip_callback.expect("non-null function pointer")(patch, x, y) {
            return;
        }
    }
    col = 0 as i32;
    desttop = state
        .dest_screen
        .offset((y * SCREENWIDTH) as isize)
        .offset(x as isize);
    w = (*patch).width as i32;
    while col < w {
        column = (patch as *mut byte)
            .offset(*(&raw const (*patch).columnofs as *const i32).offset(col as isize) as isize)
            as *mut column_t;
        while (*column).topdelta as i32 != 0xff as i32 {
            source = (column as *mut byte).offset(3 as i32 as isize);
            dest = desttop.offset(((*column).topdelta as i32 * SCREENWIDTH) as isize);
            count = (*column).length as i32;
            loop {
                let fresh10 = count;
                count = count - 1;
                if !(fresh10 != 0) {
                    break;
                }
                *dest = state.xlatab[(*dest as i32 + ((*source as i32) << 8 as i32)) as usize];
                source = source.offset(1);
                dest = dest.offset(SCREENWIDTH as isize);
            }
            column = (column as *mut byte)
                .offset((*column).length as i32 as isize)
                .offset(4 as i32 as isize) as *mut column_t;
        }
        x += 1;
        col += 1;
        desttop = desttop.offset(1);
    }
}
pub unsafe fn V_DrawAltTLPatch(
    state: &mut VVideoState,
    mut x: i32,
    mut y: i32,
    mut patch: *mut patch_t,
) {
    let mut count: i32 = 0;
    let mut col: i32 = 0;
    let mut column: *mut column_t = ::core::ptr::null_mut::<column_t>();
    let mut desttop: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut source: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut w: i32 = 0;
    y -= (*patch).topoffset as i32;
    x -= (*patch).leftoffset as i32;
    if x < 0 as i32
        || x + (*patch).width as i32 > SCREENWIDTH
        || y < 0 as i32
        || y + (*patch).height as i32 > SCREENHEIGHT
    {
        I_Error("Bad V_DrawAltTLPatch");
    }
    col = 0 as i32;
    desttop = state
        .dest_screen
        .offset((y * SCREENWIDTH) as isize)
        .offset(x as isize);
    w = (*patch).width as i32;
    while col < w {
        column = (patch as *mut byte)
            .offset(*(&raw const (*patch).columnofs as *const i32).offset(col as isize) as isize)
            as *mut column_t;
        while (*column).topdelta as i32 != 0xff as i32 {
            source = (column as *mut byte).offset(3 as i32 as isize);
            dest = desttop.offset(((*column).topdelta as i32 * SCREENWIDTH) as isize);
            count = (*column).length as i32;
            loop {
                let fresh6 = count;
                count = count - 1;
                if !(fresh6 != 0) {
                    break;
                }
                let fresh7 = source;
                source = source.offset(1);
                *dest = state.tinttable[(((*dest as i32) << 8 as i32) + *fresh7 as i32) as usize];
                dest = dest.offset(SCREENWIDTH as isize);
            }
            column = (column as *mut byte)
                .offset((*column).length as i32 as isize)
                .offset(4 as i32 as isize) as *mut column_t;
        }
        x += 1;
        col += 1;
        desttop = desttop.offset(1);
    }
}
pub unsafe fn V_DrawShadowedPatch(
    state: &mut VVideoState,
    mut x: i32,
    mut y: i32,
    mut patch: *mut patch_t,
) {
    let mut count: i32 = 0;
    let mut col: i32 = 0;
    let mut column: *mut column_t = ::core::ptr::null_mut::<column_t>();
    let mut desttop: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut source: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut desttop2: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut dest2: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut w: i32 = 0;
    y -= (*patch).topoffset as i32;
    x -= (*patch).leftoffset as i32;
    if x < 0 as i32
        || x + (*patch).width as i32 > SCREENWIDTH
        || y < 0 as i32
        || y + (*patch).height as i32 > SCREENHEIGHT
    {
        I_Error("Bad V_DrawShadowedPatch");
    }
    col = 0 as i32;
    desttop = state
        .dest_screen
        .offset((y * SCREENWIDTH) as isize)
        .offset(x as isize);
    desttop2 = state
        .dest_screen
        .offset(((y + 2 as i32) * SCREENWIDTH) as isize)
        .offset(x as isize)
        .offset(2 as i32 as isize);
    w = (*patch).width as i32;
    while col < w {
        column = (patch as *mut byte)
            .offset(*(&raw const (*patch).columnofs as *const i32).offset(col as isize) as isize)
            as *mut column_t;
        while (*column).topdelta as i32 != 0xff as i32 {
            source = (column as *mut byte).offset(3 as i32 as isize);
            dest = desttop.offset(((*column).topdelta as i32 * SCREENWIDTH) as isize);
            dest2 = desttop2.offset(((*column).topdelta as i32 * SCREENWIDTH) as isize);
            count = (*column).length as i32;
            loop {
                let fresh8 = count;
                count = count - 1;
                if !(fresh8 != 0) {
                    break;
                }
                *dest2 = state.tinttable[((*dest2 as i32) << 8 as i32) as usize];
                dest2 = dest2.offset(SCREENWIDTH as isize);
                let fresh9 = source;
                source = source.offset(1);
                *dest = *fresh9;
                dest = dest.offset(SCREENWIDTH as isize);
            }
            column = (column as *mut byte)
                .offset((*column).length as i32 as isize)
                .offset(4 as i32 as isize) as *mut column_t;
        }
        x += 1;
        col += 1;
        desttop = desttop.offset(1);
        desttop2 = desttop2.offset(1);
    }
}
pub unsafe fn V_LoadTintTable(state: &mut GameState) {
    let lumpnum = W_GetNumForName(&mut state.w_wad, "TINTTAB");
    let lump_ptr = W_CacheLumpNum(state, lumpnum) as *const byte;
    let lumplen = W_LumpLength(&mut state.w_wad, lumpnum as u32) as usize;
    state.v_video.tinttable = ::core::slice::from_raw_parts(lump_ptr, lumplen).to_vec();
}
pub unsafe fn V_LoadXlaTable(state: &mut GameState) {
    let lumpnum = W_GetNumForName(&mut state.w_wad, "XLATAB");
    let lump_ptr = W_CacheLumpNum(state, lumpnum) as *const byte;
    let lumplen = W_LumpLength(&mut state.w_wad, lumpnum as u32) as usize;
    state.v_video.xlatab = ::core::slice::from_raw_parts(lump_ptr, lumplen).to_vec();
}
pub unsafe fn V_DrawBlock(
    state: &mut GameState,
    mut x: i32,
    mut y: i32,
    mut width: i32,
    mut height: i32,
    mut src: *mut byte,
) {
    let mut dest: *mut byte = ::core::ptr::null_mut::<byte>();
    if x < 0 as i32 || x + width > SCREENWIDTH || y < 0 as i32 || y + height > SCREENHEIGHT {
        I_Error("Bad V_DrawBlock");
    }
    V_MarkRect(state, x, y, width, height);
    dest = state
        .v_video
        .dest_screen
        .offset((y * SCREENWIDTH) as isize)
        .offset(x as isize);
    loop {
        let fresh11 = height;
        height = height - 1;
        if !(fresh11 != 0) {
            break;
        }
        memcpy(
            dest as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            width as size_t,
        );
        src = src.offset(width as isize);
        dest = dest.offset(SCREENWIDTH as isize);
    }
}
pub unsafe fn V_DrawFilledBox(
    state: &mut IVideoState,
    mut x: i32,
    mut y: i32,
    mut w: i32,
    mut h: i32,
    mut c: i32,
) {
    let mut buf: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut buf1: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut x1: i32 = 0;
    let mut y1: i32 = 0;
    buf = state
        .I_VideoBuffer
        .as_mut_ptr()
        .offset((SCREENWIDTH * y) as isize)
        .offset(x as isize) as *mut uint8_t;
    y1 = 0 as i32;
    while y1 < h {
        buf1 = buf;
        x1 = 0 as i32;
        while x1 < w {
            let fresh12 = buf1;
            buf1 = buf1.offset(1);
            *fresh12 = c as uint8_t;
            x1 += 1;
        }
        buf = buf.offset(SCREENWIDTH as isize);
        y1 += 1;
    }
}
pub unsafe fn V_DrawHorizLine(
    state: &mut IVideoState,
    mut x: i32,
    mut y: i32,
    mut w: i32,
    mut c: i32,
) {
    let mut buf: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut x1: i32 = 0;
    buf = state
        .I_VideoBuffer
        .as_mut_ptr()
        .offset((SCREENWIDTH * y) as isize)
        .offset(x as isize) as *mut uint8_t;
    x1 = 0 as i32;
    while x1 < w {
        let fresh13 = buf;
        buf = buf.offset(1);
        *fresh13 = c as uint8_t;
        x1 += 1;
    }
}
pub unsafe fn V_DrawVertLine(
    state: &mut IVideoState,
    mut x: i32,
    mut y: i32,
    mut h: i32,
    mut c: i32,
) {
    let mut buf: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut y1: i32 = 0;
    buf = state
        .I_VideoBuffer
        .as_mut_ptr()
        .offset((SCREENWIDTH * y) as isize)
        .offset(x as isize) as *mut uint8_t;
    y1 = 0 as i32;
    while y1 < h {
        *buf = c as uint8_t;
        buf = buf.offset(SCREENWIDTH as isize);
        y1 += 1;
    }
}
pub unsafe fn V_DrawBox(
    state: &mut IVideoState,
    mut x: i32,
    mut y: i32,
    mut w: i32,
    mut h: i32,
    mut c: i32,
) {
    V_DrawHorizLine(state, x, y, w, c);
    V_DrawHorizLine(state, x, y + h - 1 as i32, w, c);
    V_DrawVertLine(state, x, y, h, c);
    V_DrawVertLine(state, x + w - 1 as i32, y, h, c);
}
pub unsafe fn V_DrawRawScreen(state: &mut VVideoState, mut raw: *mut byte) {
    memcpy(
        state.dest_screen as *mut ::core::ffi::c_void,
        raw as *const ::core::ffi::c_void,
        (SCREENWIDTH * SCREENHEIGHT) as size_t,
    );
}
pub unsafe fn V_UseBuffer(state: &mut VVideoState, mut buffer: *mut byte) {
    state.dest_screen = buffer;
}
pub fn V_RestoreBuffer(state: &mut GameState) {
    state.v_video.dest_screen = state.i_video.I_VideoBuffer.as_mut_ptr();
}
pub unsafe fn WritePCXfile(
    filename: &str,
    mut data: *mut byte,
    mut width: i32,
    mut height: i32,
    mut palette: *mut byte,
) {
    let mut i: i32 = 0;
    // 128-byte on-disk PCX header, built as a plain local value instead of a
    // Z_Malloc'd struct whose trailing `data: u8` field was a C flexible-
    // array-member placeholder for the payload written past it.
    let header = pcx_t {
        manufacturer: 0xa as u8,
        version: 5 as u8,
        encoding: 1 as u8,
        bits_per_pixel: 8 as u8,
        xmin: 0 as u16,
        ymin: 0 as u16,
        xmax: (width - 1 as i32) as i16 as u16,
        ymax: (height - 1 as i32) as i16 as u16,
        hres: width as i16 as u16,
        vres: height as i16 as u16,
        palette: [0u8; 48],
        reserved: 0,
        color_planes: 1 as u8,
        bytes_per_line: width as i16 as u16,
        palette_type: 2 as i32 as i16 as u16,
        filler: [0u8; 58],
        data: 0,
    };
    let mut pack: Vec<u8> =
        Vec::with_capacity((128 + width * height * 2 as i32 + 768 as i32 + 1 as i32) as usize);
    pack.extend_from_slice(::core::slice::from_raw_parts(
        &header as *const pcx_t as *const u8,
        128,
    ));
    i = 0 as i32;
    while i < width * height {
        if *data as i32 & 0xc0 as i32 != 0xc0 as i32 {
            let fresh14 = data;
            data = data.offset(1);
            pack.push(*fresh14);
        } else {
            pack.push(0xc1 as byte);
            let fresh17 = data;
            data = data.offset(1);
            pack.push(*fresh17);
        }
        i += 1;
    }
    pack.push(0xc as byte);
    i = 0 as i32;
    while i < 768 as i32 {
        let fresh20 = palette;
        palette = palette.offset(1);
        pack.push(*fresh20);
        i += 1;
    }
    M_WriteFile(filename, &pack);
}
pub unsafe fn V_ScreenShot(state: &mut GameState) {
    let mut i = 0i32;
    let mut lbmname = String::new();
    while i <= 99 {
        lbmname = format!("DOOM{i:02}.pcx");
        if !M_FileExists(&lbmname) {
            break;
        }
        i += 1;
    }
    if i == 100 as i32 {
        I_Error("V_ScreenShot: Couldn't create a PCX");
    }
    let __wcache747_1 = W_CacheLumpName(state, "PLAYPAL") as *mut byte;
    WritePCXfile(
        &lbmname,
        state.i_video.I_VideoBuffer.as_mut_ptr(),
        SCREENWIDTH,
        SCREENHEIGHT,
        __wcache747_1,
    );
}
pub const MOUSE_SPEED_BOX_WIDTH: i32 = 120;
pub const MOUSE_SPEED_BOX_HEIGHT: i32 = 9;
pub unsafe fn V_DrawMouseSpeedBox(state: &mut IVideoState, mut speed: i32) {
    let mut bgcolor: i32 = 0;
    let mut bordercolor: i32 = 0;
    let mut red: i32 = 0;
    let mut black: i32 = 0;
    let mut white: i32 = 0;
    let mut yellow: i32 = 0;
    let mut box_x: i32 = 0;
    let mut box_y: i32 = 0;
    let mut original_speed: i32 = 0;
    let mut redline_x: i32 = 0;
    let mut linelen: i32 = 0;
    bgcolor = I_GetPaletteIndex(0x77 as i32, 0x77 as i32, 0x77 as i32);
    bordercolor = I_GetPaletteIndex(0x55 as i32, 0x55 as i32, 0x55 as i32);
    red = I_GetPaletteIndex(0xff as i32, 0 as i32, 0 as i32);
    black = I_GetPaletteIndex(0 as i32, 0 as i32, 0 as i32);
    yellow = I_GetPaletteIndex(0xff as i32, 0xff as i32, 0 as i32);
    white = I_GetPaletteIndex(0xff as i32, 0xff as i32, 0xff as i32);
    if state.usemouse == 0 || ((state.mouse_acceleration - 1 as i32 as f32) as f64).abs() < 0.01f64 {
        return;
    }
    box_x = SCREENWIDTH - MOUSE_SPEED_BOX_WIDTH - 10 as i32;
    box_y = 15 as i32;
    V_DrawFilledBox(
        state,
        box_x,
        box_y,
        MOUSE_SPEED_BOX_WIDTH,
        MOUSE_SPEED_BOX_HEIGHT,
        bgcolor,
    );
    V_DrawBox(
        state,
        box_x,
        box_y,
        MOUSE_SPEED_BOX_WIDTH,
        MOUSE_SPEED_BOX_HEIGHT,
        bordercolor,
    );
    redline_x = MOUSE_SPEED_BOX_WIDTH / 3 as i32;
    if speed < state.mouse_threshold {
        original_speed = speed;
    } else {
        original_speed = speed - state.mouse_threshold;
        original_speed = (original_speed as f32 / state.mouse_acceleration) as i32;
        original_speed += state.mouse_threshold;
    }
    linelen = original_speed * redline_x / state.mouse_threshold;
    if linelen > MOUSE_SPEED_BOX_WIDTH - 1 as i32 {
        linelen = MOUSE_SPEED_BOX_WIDTH - 1 as i32;
    }
    V_DrawHorizLine(
        state,
        box_x + 1 as i32,
        box_y + 4 as i32,
        MOUSE_SPEED_BOX_WIDTH - 2 as i32,
        black,
    );
    if linelen < redline_x {
        V_DrawHorizLine(
            state,
            box_x + 1 as i32,
            box_y + MOUSE_SPEED_BOX_HEIGHT / 2 as i32,
            linelen,
            white,
        );
    } else {
        V_DrawHorizLine(
            state,
            box_x + 1 as i32,
            box_y + MOUSE_SPEED_BOX_HEIGHT / 2 as i32,
            redline_x,
            white,
        );
        V_DrawHorizLine(
            state,
            box_x + redline_x,
            box_y + MOUSE_SPEED_BOX_HEIGHT / 2 as i32,
            linelen - redline_x,
            yellow,
        );
    }
    V_DrawVertLine(
        state,
        box_x + redline_x,
        box_y + 1 as i32,
        MOUSE_SPEED_BOX_HEIGHT - 2 as i32,
        red,
    );
}
