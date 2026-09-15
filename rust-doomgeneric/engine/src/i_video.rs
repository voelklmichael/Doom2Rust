use crate::doomdef::pixel_t;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::doomgeneric::DOOMGENERIC_RESX;
use crate::doomgeneric::DOOMGENERIC_RESY;
use crate::game_state::GameState;
use crate::i_input::I_GetEvent;
use crate::i_system::I_Error;
use crate::m_argv::{M_ArgvAtoi, M_CheckParmWithArgs};
use crate::m_fixed::INT_MAX;
use crate::mem_compat::{memcpy, memset};
use crate::stdint_types::size_t;
use crate::stdint_types::uint32_t;
use crate::stdint_types::{byte, uint8_t};
use crate::tables::gammatable;

pub struct IVideoState {
    pub s_Fb: FB_ScreenInfo,
    pub fb_scaling: i32,
    pub usemouse: i32,
    pub colors: [color; 256],
    pub I_VideoBuffer: Vec<byte>,
    pub screensaver_mode: bool,
    pub screenvisible: bool,
    pub mouse_acceleration: f32,
    pub mouse_threshold: i32,
    pub usegamma: i32,
    // The engine's own writable framebuffer, handed to the platform layer's
    // init() once at startup -- the platform keeps its own independent
    // handle into the same allocation for display/blit purposes.
    pub dg_screen_buffer: Vec<pixel_t>,
}

impl IVideoState {
    pub const fn new() -> Self {
        IVideoState {
            s_Fb: FB_ScreenInfo {
                xres: 0,
                yres: 0,
                xres_virtual: 0,
                yres_virtual: 0,
                bits_per_pixel: 0,
                red: FB_BitField {
                    offset: 0,
                    length: 0,
                },
                green: FB_BitField {
                    offset: 0,
                    length: 0,
                },
                blue: FB_BitField {
                    offset: 0,
                    length: 0,
                },
                transp: FB_BitField {
                    offset: 0,
                    length: 0,
                },
            },
            fb_scaling: 1,
            usemouse: 0,
            colors: [color { b_g_r_a: [0; 4] }; 256],
            I_VideoBuffer: Vec::new(),
            screensaver_mode: false,
            screenvisible: false,
            mouse_acceleration: 2.0f32,
            mouse_threshold: 10,
            usegamma: 0,
            dg_screen_buffer: Vec::new(),
        }
    }
}

pub type __uint16_t = u16;
pub type uint16_t = __uint16_t;
pub type grabmouse_callback_t = Option<unsafe fn() -> bool>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FB_ScreenInfo {
    pub xres: uint32_t,
    pub yres: uint32_t,
    pub xres_virtual: uint32_t,
    pub yres_virtual: uint32_t,
    pub bits_per_pixel: uint32_t,
    pub red: FB_BitField,
    pub green: FB_BitField,
    pub blue: FB_BitField,
    pub transp: FB_BitField,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FB_BitField {
    pub offset: uint32_t,
    pub length: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct color {
    pub b_g_r_a: [u8; 4],
}
impl color {
    pub fn r(&self) -> u8 {
        self.b_g_r_a[2]
    }
    pub fn g(&self) -> u8 {
        self.b_g_r_a[1]
    }
    pub fn b(&self) -> u8 {
        self.b_g_r_a[0]
    }
    pub fn a(&self) -> u8 {
        self.b_g_r_a[3]
    }
    pub fn set_r(&mut self, value: u32) {
        self.b_g_r_a[2] = value as u8;
    }
    pub fn set_g(&mut self, value: u32) {
        self.b_g_r_a[1] = value as u8;
    }
    pub fn set_b(&mut self, value: u32) {
        self.b_g_r_a[0] = value as u8;
    }
    pub fn set_a(&mut self, value: u32) {
        self.b_g_r_a[3] = value as u8;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct col_t {
    pub r: byte,
    pub g: byte,
    pub b: byte,
}
static rgb565_palette: [uint16_t; 256] = [0; 256];
pub unsafe fn cmap_to_rgb565(
    state: &mut GameState,
    mut out: *mut uint16_t,
    mut in_0: *mut uint8_t,
    mut in_pixels: i32,
) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut c: color = color { b_g_r_a: [0; 4] };
    let mut r: uint16_t = 0;
    let mut g: uint16_t = 0;
    let mut b: uint16_t = 0;
    i = 0_i32;
    while i < in_pixels {
        c = state.i_video.colors[*in_0 as usize];
        r = (((c.r() as i32 >> 3_i32) as uint16_t as i32) << 11_i32) as uint16_t;
        g = (((c.g() as i32 >> 2_i32) as uint16_t as i32) << 5_i32) as uint16_t;
        b = (((c.b() as i32 >> 3_i32) as uint16_t as i32) << 0_i32) as uint16_t;
        *out = (r as i32 | g as i32 | b as i32) as uint16_t;
        in_0 = in_0.offset(1);
        j = 0_i32;
        while j < state.i_video.fb_scaling {
            out = out.offset(1);
            j += 1;
        }
        i += 1;
    }
}
pub unsafe fn cmap_to_fb(
    state: &mut GameState,
    mut out: *mut uint8_t,
    mut in_0: *mut uint8_t,
    mut in_pixels: i32,
) {
    let mut i: i32 = 0;
    let mut k: i32 = 0;
    let mut c: color = color { b_g_r_a: [0; 4] };
    let mut pix: uint32_t = 0;
    i = 0_i32;
    while i < in_pixels {
        c = state.i_video.colors[*in_0 as usize];
        if state.i_video.s_Fb.bits_per_pixel == 16 as uint32_t {
            let mut p: uint16_t = ((c.r() as i32 & 0xf8_i32) << 8_i32
                | (c.g() as i32 & 0xfc_i32) << 3_i32
                | c.b() as i32 >> 3_i32) as uint16_t;
            k = 0_i32;
            while k < state.i_video.fb_scaling {
                *(out as *mut uint16_t) = p;
                out = out.offset(2_i32 as isize);
                k += 1;
            }
        } else if state.i_video.s_Fb.bits_per_pixel == 32 as uint32_t {
            pix = ((c.r() as i32) << state.i_video.s_Fb.red.offset
                | (c.g() as i32) << state.i_video.s_Fb.green.offset
                | (c.b() as i32) << state.i_video.s_Fb.blue.offset) as uint32_t;
            k = 0_i32;
            while k < state.i_video.fb_scaling {
                *(out as *mut uint32_t) = pix;
                out = out.offset(4_i32 as isize);
                k += 1;
            }
        } else {
            I_Error(&format!(
                "No idea how to convert {} bpp pixels",
                state.i_video.s_Fb.bits_per_pixel
            ));
        }
        in_0 = in_0.offset(1);
        i += 1;
    }
}
pub unsafe fn I_InitGraphics(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut gfxmodeparm: i32 = 0;
    let mut mode: &str = "";
    memset(
        &raw mut state.i_video.s_Fb as *mut ::core::ffi::c_void,
        0_i32,
        ::core::mem::size_of::<FB_ScreenInfo>() as size_t,
    );
    state.i_video.s_Fb.xres = DOOMGENERIC_RESX as uint32_t;
    state.i_video.s_Fb.yres = DOOMGENERIC_RESY as uint32_t;
    state.i_video.s_Fb.xres_virtual = state.i_video.s_Fb.xres;
    state.i_video.s_Fb.yres_virtual = state.i_video.s_Fb.yres;
    gfxmodeparm = M_CheckParmWithArgs(state, "-gfxmode", 1_i32);
    if gfxmodeparm != 0 {
        mode = state.m_argv.myargv[(gfxmodeparm + 1_i32) as usize]
            .to_str()
            .unwrap();
    } else {
        mode = "rgba8888";
    }
    if mode == "rgba8888" {
        state.i_video.s_Fb.bits_per_pixel = 32 as uint32_t;
        state.i_video.s_Fb.blue.length = 8 as uint32_t;
        state.i_video.s_Fb.green.length = 8 as uint32_t;
        state.i_video.s_Fb.red.length = 8 as uint32_t;
        state.i_video.s_Fb.transp.length = 8 as uint32_t;
        state.i_video.s_Fb.blue.offset = 0 as uint32_t;
        state.i_video.s_Fb.green.offset = 8 as uint32_t;
        state.i_video.s_Fb.red.offset = 16 as uint32_t;
        state.i_video.s_Fb.transp.offset = 24 as uint32_t;
    } else if mode == "rgb565" {
        state.i_video.s_Fb.bits_per_pixel = 16 as uint32_t;
        state.i_video.s_Fb.blue.length = 5 as uint32_t;
        state.i_video.s_Fb.green.length = 6 as uint32_t;
        state.i_video.s_Fb.red.length = 5 as uint32_t;
        state.i_video.s_Fb.transp.length = 0 as uint32_t;
        state.i_video.s_Fb.blue.offset = 11 as uint32_t;
        state.i_video.s_Fb.green.offset = 5 as uint32_t;
        state.i_video.s_Fb.red.offset = 0 as uint32_t;
        state.i_video.s_Fb.transp.offset = 16 as uint32_t;
    } else {
        I_Error(&format!("Unknown gfxmode value: {}\n", mode));
    }
    println!(
        "I_InitGraphics: framebuffer: x_res: {}, y_res: {}, x_virtual: {}, y_virtual: {}, bpp: {}",
        state.i_video.s_Fb.xres,
        state.i_video.s_Fb.yres,
        state.i_video.s_Fb.xres_virtual,
        state.i_video.s_Fb.yres_virtual,
        state.i_video.s_Fb.bits_per_pixel,
    );
    println!(
        "I_InitGraphics: framebuffer: RGBA: {}{}{}{}, red_off: {}, green_off: {}, blue_off: {}, transp_off: {}",
        state.i_video.s_Fb.red.length,
        state.i_video.s_Fb.green.length,
        state.i_video.s_Fb.blue.length,
        state.i_video.s_Fb.transp.length,
        state.i_video.s_Fb.red.offset,
        state.i_video.s_Fb.green.offset,
        state.i_video.s_Fb.blue.offset,
        state.i_video.s_Fb.transp.offset,
    );
    println!(
        "I_InitGraphics: DOOM screen size: w x h: {} x {}",
        SCREENWIDTH, SCREENHEIGHT,
    );
    i = M_CheckParmWithArgs(state, "-scaling", 1_i32);
    if i > 0_i32 {
        i = M_ArgvAtoi(&state.m_argv.myargv[(i + 1_i32) as usize]);
        state.i_video.fb_scaling = i;
        println!(
            "I_InitGraphics: Scaling factor: {}",
            state.i_video.fb_scaling
        );
    } else {
        state.i_video.fb_scaling = state
            .i_video
            .s_Fb
            .xres
            .wrapping_div(SCREENWIDTH as uint32_t) as i32;
        if state
            .i_video
            .s_Fb
            .yres
            .wrapping_div(SCREENHEIGHT as uint32_t)
            < state.i_video.fb_scaling as uint32_t
        {
            state.i_video.fb_scaling = state
                .i_video
                .s_Fb
                .yres
                .wrapping_div(SCREENHEIGHT as uint32_t)
                as i32;
        }
        println!(
            "I_InitGraphics: Auto-scaling factor: {}",
            state.i_video.fb_scaling
        );
    }
    state.i_video.I_VideoBuffer = vec![0u8; (SCREENWIDTH * SCREENHEIGHT) as usize];
    state.i_video.screenvisible = true;
}
pub fn I_ShutdownGraphics(state: &mut GameState) {
    state.i_video.I_VideoBuffer = Vec::new();
}
pub fn I_StartTic(state: &mut GameState) {
    I_GetEvent(state);
}
pub unsafe fn I_FinishUpdate(state: &mut GameState) {
    let mut y: i32 = 0;
    let mut x_offset: i32 = 0;
    let mut x_offset_end: i32 = 0;
    let mut line_in: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut line_out: *mut u8 = ::core::ptr::null_mut::<u8>();
    x_offset = state
        .i_video
        .s_Fb
        .xres
        .wrapping_sub((SCREENWIDTH * state.i_video.fb_scaling) as uint32_t)
        .wrapping_mul(state.i_video.s_Fb.bits_per_pixel)
        .wrapping_div(8 as uint32_t)
        .wrapping_div(2 as uint32_t) as i32;
    x_offset_end = state
        .i_video
        .s_Fb
        .xres
        .wrapping_sub((SCREENWIDTH * state.i_video.fb_scaling) as uint32_t)
        .wrapping_mul(state.i_video.s_Fb.bits_per_pixel)
        .wrapping_div(8 as uint32_t)
        .wrapping_sub(x_offset as uint32_t) as i32;
    line_in = state.i_video.I_VideoBuffer.as_mut_ptr() as *mut u8;
    line_out = state.i_video.dg_screen_buffer.as_mut_ptr() as *mut u8;
    y = SCREENHEIGHT;
    loop {
        let fresh3 = y;
        y -= 1;
        if fresh3 == 0 {
            break;
        }
        let mut i: i32 = 0;
        i = 0_i32;
        while i < state.i_video.fb_scaling {
            line_out = line_out.offset(x_offset as isize);
            cmap_to_fb(
                state,
                line_out as *mut ::core::ffi::c_void as *mut uint8_t,
                line_in as *mut ::core::ffi::c_void as *mut uint8_t,
                SCREENWIDTH,
            );
            line_out = line_out.offset(
                ((SCREENWIDTH * state.i_video.fb_scaling) as uint32_t)
                    .wrapping_mul(
                        state
                            .i_video
                            .s_Fb
                            .bits_per_pixel
                            .wrapping_div(8 as uint32_t),
                    )
                    .wrapping_add(x_offset_end as uint32_t) as isize,
            );
            i += 1;
        }
        line_in = line_in.offset(SCREENWIDTH as isize);
    }
    state.platform.draw_frame();
}
pub unsafe fn I_ReadScreen(state: &mut GameState, mut scr: *mut byte) {
    memcpy(
        scr as *mut ::core::ffi::c_void,
        state.i_video.I_VideoBuffer.as_ptr() as *const ::core::ffi::c_void,
        (SCREENWIDTH * SCREENHEIGHT) as size_t,
    );
}
pub unsafe fn I_SetPalette(state: &mut GameState, mut palette: *mut byte) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < 256_i32 {
        state.i_video.colors[i as usize].set_a(0 as uint32_t as uint32_t);
        let mut rhs = {
            let fresh0 = palette;
            palette = palette.offset(1);
            gammatable[state.i_video.usegamma as usize][*fresh0 as usize] as uint32_t
        } as uint32_t;
        state.i_video.colors[i as usize].set_r(rhs);
        let mut rhs_0 = {
            let fresh1 = palette;
            palette = palette.offset(1);
            gammatable[state.i_video.usegamma as usize][*fresh1 as usize] as uint32_t
        } as uint32_t;
        state.i_video.colors[i as usize].set_g(rhs_0);
        let mut rhs_1 = {
            let fresh2 = palette;
            palette = palette.offset(1);
            gammatable[state.i_video.usegamma as usize][*fresh2 as usize] as uint32_t
        } as uint32_t;
        state.i_video.colors[i as usize].set_b(rhs_1);
        i += 1;
    }
}
pub fn I_GetPaletteIndex(mut r: i32, mut g: i32, mut b: i32) -> i32 {
    let mut best: i32 = 0;
    let mut best_diff: i32 = 0;
    let mut diff: i32 = 0;
    let mut i: i32 = 0;
    let mut color: col_t = col_t { r: 0, g: 0, b: 0 };
    println!("I_GetPaletteIndex");
    best = 0_i32;
    best_diff = INT_MAX;
    i = 0_i32;
    while i < 256_i32 {
        color.r = ((0xf800_i32 & rgb565_palette[i as usize] as i32) >> 11_i32) as byte;
        color.g = ((0x7e0_i32 & rgb565_palette[i as usize] as i32) >> 5_i32) as byte;
        color.b = (0x1f_i32 & rgb565_palette[i as usize] as i32) as byte;
        diff = (r - color.r as i32) * (r - color.r as i32)
            + (g - color.g as i32) * (g - color.g as i32)
            + (b - color.b as i32) * (b - color.b as i32);
        if diff < best_diff {
            best = i;
            best_diff = diff;
        }
        if diff == 0_i32 {
            break;
        }
        i += 1;
    }
    best
}
pub fn I_SetWindowTitle(state: &mut GameState, title: &str) {
    state.platform.set_window_title(title);
}
pub fn I_SetGrabMouseCallback() {}
