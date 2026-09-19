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
use crate::platform::DoomPlatform;
use crate::stdint_types::byte;
use crate::stdint_types::uint32_t;
use crate::tables::gammatable;
use alloc::vec::Vec;

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

impl Default for IVideoState {
    fn default() -> Self {
        Self::new()
    }
}

impl IVideoState {
    pub const fn new() -> Self {
        IVideoState {
            s_Fb: FB_ScreenInfo::ZERO,
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
#[derive(Copy, Clone)]
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
impl FB_ScreenInfo {
    pub const ZERO: FB_ScreenInfo = FB_ScreenInfo {
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
    };
}
#[derive(Copy, Clone)]
pub struct FB_BitField {
    pub offset: uint32_t,
    pub length: uint32_t,
}
#[derive(Copy, Clone)]
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
pub struct col_t {
    pub r: byte,
    pub g: byte,
    pub b: byte,
}
static RGB565_PALETTE: [uint16_t; 256] = [0; 256];
pub fn I_InitGraphics(state: &mut GameState) {
    let mut i: i32;

    state.i_video.s_Fb = FB_ScreenInfo::ZERO;
    state.i_video.s_Fb.xres = DOOMGENERIC_RESX as uint32_t;
    state.i_video.s_Fb.yres = DOOMGENERIC_RESY as uint32_t;
    state.i_video.s_Fb.xres_virtual = state.i_video.s_Fb.xres;
    state.i_video.s_Fb.yres_virtual = state.i_video.s_Fb.yres;
    let gfxmodeparm: i32 = M_CheckParmWithArgs(state, "-gfxmode", 1_i32);
    let mode: &str = if gfxmodeparm != 0 {
        state.m_argv.myargv[(gfxmodeparm + 1_i32) as usize].as_str()
    } else {
        "rgba8888"
    };
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
    doom_println!(
        state.platform,
        "I_InitGraphics: framebuffer: x_res: {}, y_res: {}, x_virtual: {}, y_virtual: {}, bpp: {}",
        state.i_video.s_Fb.xres,
        state.i_video.s_Fb.yres,
        state.i_video.s_Fb.xres_virtual,
        state.i_video.s_Fb.yres_virtual,
        state.i_video.s_Fb.bits_per_pixel,
    );
    doom_println!(state.platform,
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
    doom_println!(
        state.platform,
        "I_InitGraphics: DOOM screen size: w x h: {} x {}",
        SCREENWIDTH,
        SCREENHEIGHT,
    );
    i = M_CheckParmWithArgs(state, "-scaling", 1_i32);
    if i > 0_i32 {
        i = M_ArgvAtoi(&state.m_argv.myargv[(i + 1_i32) as usize]);
        state.i_video.fb_scaling = i;
        doom_println!(
            state.platform,
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
        doom_println!(
            state.platform,
            "I_InitGraphics: Auto-scaling factor: {}",
            state.i_video.fb_scaling
        );
    }
    state.i_video.I_VideoBuffer = vec![0u8; (SCREENWIDTH * SCREENHEIGHT) as usize];
    state.i_video.screenvisible = true;
}
pub fn I_StartTic(state: &mut GameState) {
    I_GetEvent(state);
}
pub fn I_FinishUpdate(state: &mut GameState) {
    let fb = state.i_video.s_Fb;
    let scaling = state.i_video.fb_scaling as usize;
    let bytes_per_pixel = (fb.bits_per_pixel / 8) as usize;
    let line_bytes = fb.xres as usize * bytes_per_pixel;
    let x_offset = fb
        .xres
        .wrapping_sub((SCREENWIDTH * state.i_video.fb_scaling) as uint32_t)
        .wrapping_mul(fb.bits_per_pixel)
        .wrapping_div(8 as uint32_t)
        .wrapping_div(2 as uint32_t) as usize;
    if fb.bits_per_pixel != 16 && fb.bits_per_pixel != 32 {
        I_Error(&format!(
            "No idea how to convert {} bpp pixels",
            fb.bits_per_pixel
        ));
    }
    let mut frame = vec![0u8; line_bytes * SCREENHEIGHT as usize * scaling];
    let mut line_out = 0usize;
    for row in 0..SCREENHEIGHT as usize {
        let source =
            &state.i_video.I_VideoBuffer[row * SCREENWIDTH as usize..][..SCREENWIDTH as usize];
        let first_line = &mut frame[line_out * line_bytes..][..line_bytes];
        let mut out = x_offset;
        for &index in source {
            let c = state.i_video.colors[index as usize];
            if fb.bits_per_pixel == 16 {
                let p: uint16_t = ((c.r() as i32 & 0xf8_i32) << 8_i32
                    | (c.g() as i32 & 0xfc_i32) << 3_i32
                    | c.b() as i32 >> 3_i32) as uint16_t;
                for _ in 0..scaling {
                    first_line[out..out + 2].copy_from_slice(&p.to_ne_bytes());
                    out += 2;
                }
            } else {
                let pix = ((c.r() as i32) << fb.red.offset
                    | (c.g() as i32) << fb.green.offset
                    | (c.b() as i32) << fb.blue.offset) as uint32_t;
                for _ in 0..scaling {
                    first_line[out..out + 4].copy_from_slice(&pix.to_ne_bytes());
                    out += 4;
                }
            }
        }
        for copy in 1..scaling {
            frame.copy_within(
                line_out * line_bytes..(line_out + 1) * line_bytes,
                (line_out + copy) * line_bytes,
            );
        }
        line_out += scaling;
    }
    for (pixel, bytes) in state
        .i_video
        .dg_screen_buffer
        .iter_mut()
        .zip(frame.as_chunks::<4>().0.iter())
    {
        *pixel = u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    }
    state.platform.draw_frame();
}
pub fn I_ReadScreen(state: &GameState) -> Vec<byte> {
    state.i_video.I_VideoBuffer[..(SCREENWIDTH * SCREENHEIGHT) as usize].to_vec()
}
pub fn I_SetPalette(state: &mut GameState, palette: &[byte]) {
    let gamma = &gammatable[state.i_video.usegamma as usize];
    for (color, rgb) in state
        .i_video
        .colors
        .iter_mut()
        .zip(palette.as_chunks::<3>().0.iter())
    {
        color.set_a(0 as uint32_t);
        color.set_r(gamma[rgb[0] as usize] as uint32_t);
        color.set_g(gamma[rgb[1] as usize] as uint32_t);
        color.set_b(gamma[rgb[2] as usize] as uint32_t);
    }
}
pub fn I_GetPaletteIndex(platform: &mut dyn DoomPlatform, r: i32, g: i32, b: i32) -> i32 {
    let mut best: i32;
    let mut best_diff: i32;
    let mut diff: i32;
    let mut i: i32;
    let mut color: col_t = col_t { r: 0, g: 0, b: 0 };
    doom_println!(platform, "I_GetPaletteIndex");
    best = 0_i32;
    best_diff = INT_MAX;
    i = 0_i32;
    while i < 256_i32 {
        color.r = ((0xf800_i32 & RGB565_PALETTE[i as usize] as i32) >> 11_i32) as byte;
        color.g = ((0x7e0_i32 & RGB565_PALETTE[i as usize] as i32) >> 5_i32) as byte;
        color.b = (0x1f_i32 & RGB565_PALETTE[i as usize] as i32) as byte;
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
