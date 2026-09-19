use crate::doomdef::Pixel;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::doomgeneric::DOOMGENERIC_RESX;
use crate::doomgeneric::DOOMGENERIC_RESY;
use crate::game_state::GameState;
use crate::i_input::get_event;
use crate::i_system::error;
use crate::m_argv::{argv_atoi, check_parm_with_args};
use crate::m_fixed::INT_MAX;
use crate::platform::DoomPlatform;
use crate::tables::GAMMATABLE;
use alloc::vec::Vec;

pub struct IVideoState {
    pub s_fb: FBScreenInfo,
    pub fb_scaling: i32,
    pub usemouse: i32,
    pub colors: [Color; 256],
    pub i_video_buffer: Vec<u8>,
    pub screensaver_mode: bool,
    pub screenvisible: bool,
    pub mouse_acceleration: f32,
    pub mouse_threshold: i32,
    pub usegamma: i32,
    // The engine's own writable framebuffer, handed to the platform layer's
    // init() once at startup -- the platform keeps its own independent
    // handle into the same allocation for display/blit purposes.
    pub dg_screen_buffer: Vec<Pixel>,
}

impl Default for IVideoState {
    fn default() -> Self {
        Self::new()
    }
}

impl IVideoState {
    pub const fn new() -> Self {
        IVideoState {
            s_fb: FBScreenInfo::ZERO,
            fb_scaling: 1,
            usemouse: 0,
            colors: [Color { b_g_r_a: [0; 4] }; 256],
            i_video_buffer: Vec::new(),
            screensaver_mode: false,
            screenvisible: false,
            mouse_acceleration: 2.0f32,
            mouse_threshold: 10,
            usegamma: 0,
            dg_screen_buffer: Vec::new(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct FBScreenInfo {
    pub xres: u32,
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub bits_per_pixel: u32,
    pub red: FBBitField,
    pub green: FBBitField,
    pub blue: FBBitField,
    pub transp: FBBitField,
}
impl FBScreenInfo {
    pub const ZERO: FBScreenInfo = FBScreenInfo {
        xres: 0,
        yres: 0,
        xres_virtual: 0,
        yres_virtual: 0,
        bits_per_pixel: 0,
        red: FBBitField {
            offset: 0,
            length: 0,
        },
        green: FBBitField {
            offset: 0,
            length: 0,
        },
        blue: FBBitField {
            offset: 0,
            length: 0,
        },
        transp: FBBitField {
            offset: 0,
            length: 0,
        },
    };
}
#[derive(Copy, Clone)]
pub struct FBBitField {
    pub offset: u32,
    pub length: u32,
}
#[derive(Copy, Clone)]
pub struct Color {
    pub b_g_r_a: [u8; 4],
}
impl Color {
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
pub struct Column {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
static RGB565_PALETTE: [u16; 256] = [0; 256];
pub fn init_graphics(state: &mut GameState) {
    let mut i: i32;

    state.i_video.s_fb = FBScreenInfo::ZERO;
    state.i_video.s_fb.xres = DOOMGENERIC_RESX as u32;
    state.i_video.s_fb.yres = DOOMGENERIC_RESY as u32;
    state.i_video.s_fb.xres_virtual = state.i_video.s_fb.xres;
    state.i_video.s_fb.yres_virtual = state.i_video.s_fb.yres;
    let gfxmodeparm: i32 = check_parm_with_args(state, "-gfxmode", 1);
    let mode: &str = if gfxmodeparm != 0 {
        state.m_argv.myargv[(gfxmodeparm + 1) as usize].as_str()
    } else {
        "rgba8888"
    };
    if mode == "rgba8888" {
        state.i_video.s_fb.bits_per_pixel = 32_u32;
        state.i_video.s_fb.blue.length = 8_u32;
        state.i_video.s_fb.green.length = 8_u32;
        state.i_video.s_fb.red.length = 8_u32;
        state.i_video.s_fb.transp.length = 8_u32;
        state.i_video.s_fb.blue.offset = 0_u32;
        state.i_video.s_fb.green.offset = 8_u32;
        state.i_video.s_fb.red.offset = 16_u32;
        state.i_video.s_fb.transp.offset = 24_u32;
    } else if mode == "rgb565" {
        state.i_video.s_fb.bits_per_pixel = 16_u32;
        state.i_video.s_fb.blue.length = 5_u32;
        state.i_video.s_fb.green.length = 6_u32;
        state.i_video.s_fb.red.length = 5_u32;
        state.i_video.s_fb.transp.length = 0_u32;
        state.i_video.s_fb.blue.offset = 11_u32;
        state.i_video.s_fb.green.offset = 5_u32;
        state.i_video.s_fb.red.offset = 0_u32;
        state.i_video.s_fb.transp.offset = 16_u32;
    } else {
        error(&format!("Unknown gfxmode value: {}\n", mode));
    }
    doom_println!(
        state.platform,
        "I_InitGraphics: framebuffer: x_res: {}, y_res: {}, x_virtual: {}, y_virtual: {}, bpp: {}",
        state.i_video.s_fb.xres,
        state.i_video.s_fb.yres,
        state.i_video.s_fb.xres_virtual,
        state.i_video.s_fb.yres_virtual,
        state.i_video.s_fb.bits_per_pixel,
    );
    doom_println!(state.platform,
        "I_InitGraphics: framebuffer: RGBA: {}{}{}{}, red_off: {}, green_off: {}, blue_off: {}, transp_off: {}",
        state.i_video.s_fb.red.length,
        state.i_video.s_fb.green.length,
        state.i_video.s_fb.blue.length,
        state.i_video.s_fb.transp.length,
        state.i_video.s_fb.red.offset,
        state.i_video.s_fb.green.offset,
        state.i_video.s_fb.blue.offset,
        state.i_video.s_fb.transp.offset,
    );
    doom_println!(
        state.platform,
        "I_InitGraphics: DOOM screen size: w x h: {} x {}",
        SCREENWIDTH,
        SCREENHEIGHT,
    );
    i = check_parm_with_args(state, "-scaling", 1);
    if i > 0 {
        i = argv_atoi(&state.m_argv.myargv[(i + 1) as usize]);
        state.i_video.fb_scaling = i;
        doom_println!(
            state.platform,
            "I_InitGraphics: Scaling factor: {}",
            state.i_video.fb_scaling
        );
    } else {
        state.i_video.fb_scaling = state.i_video.s_fb.xres.wrapping_div(SCREENWIDTH as u32) as i32;
        if state.i_video.s_fb.yres.wrapping_div(SCREENHEIGHT as u32)
            < state.i_video.fb_scaling as u32
        {
            state.i_video.fb_scaling =
                state.i_video.s_fb.yres.wrapping_div(SCREENHEIGHT as u32) as i32;
        }
        doom_println!(
            state.platform,
            "I_InitGraphics: Auto-scaling factor: {}",
            state.i_video.fb_scaling
        );
    }
    state.i_video.i_video_buffer = vec![0u8; (SCREENWIDTH * SCREENHEIGHT) as usize];
    state.i_video.screenvisible = true;
}
pub fn start_tic(state: &mut GameState) {
    get_event(state);
}
pub fn finish_update(state: &mut GameState) {
    let fb = state.i_video.s_fb;
    let scaling = state.i_video.fb_scaling as usize;
    let bytes_per_pixel = (fb.bits_per_pixel / 8) as usize;
    let line_bytes = fb.xres as usize * bytes_per_pixel;
    let x_offset = fb
        .xres
        .wrapping_sub((SCREENWIDTH * state.i_video.fb_scaling) as u32)
        .wrapping_mul(fb.bits_per_pixel)
        .wrapping_div(8_u32)
        .wrapping_div(2_u32) as usize;
    if fb.bits_per_pixel != 16 && fb.bits_per_pixel != 32 {
        error(&format!(
            "No idea how to convert {} bpp pixels",
            fb.bits_per_pixel
        ));
    }
    let mut frame = vec![0u8; line_bytes * SCREENHEIGHT as usize * scaling];
    let mut line_out = 0usize;
    for row in 0..SCREENHEIGHT as usize {
        let source =
            &state.i_video.i_video_buffer[row * SCREENWIDTH as usize..][..SCREENWIDTH as usize];
        let first_line = &mut frame[line_out * line_bytes..][..line_bytes];
        let mut out = x_offset;
        for &index in source {
            let c = state.i_video.colors[index as usize];
            if fb.bits_per_pixel == 16 {
                let p: u16 = ((c.r() as i32 & 0xf8) << 8
                    | (c.g() as i32 & 0xfc) << 3
                    | c.b() as i32 >> 3) as u16;
                for _ in 0..scaling {
                    first_line[out..out + 2].copy_from_slice(&p.to_ne_bytes());
                    out += 2;
                }
            } else {
                let pix = ((c.r() as i32) << fb.red.offset
                    | (c.g() as i32) << fb.green.offset
                    | (c.b() as i32) << fb.blue.offset) as u32;
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
    state.platform.draw_frame(&state.i_video.dg_screen_buffer);
}
pub fn read_screen(state: &GameState) -> Vec<u8> {
    state.i_video.i_video_buffer[..(SCREENWIDTH * SCREENHEIGHT) as usize].to_vec()
}
pub fn set_palette(state: &mut GameState, palette: &[u8]) {
    let gamma = &GAMMATABLE[state.i_video.usegamma as usize];
    for (color, rgb) in state
        .i_video
        .colors
        .iter_mut()
        .zip(palette.as_chunks::<3>().0.iter())
    {
        color.set_a(0_u32);
        color.set_r(gamma[rgb[0] as usize] as u32);
        color.set_g(gamma[rgb[1] as usize] as u32);
        color.set_b(gamma[rgb[2] as usize] as u32);
    }
}
pub fn get_palette_index(platform: &mut dyn DoomPlatform, r: i32, g: i32, b: i32) -> i32 {
    let mut best: i32;
    let mut best_diff: i32;
    let mut diff: i32;
    let mut color: Column = Column { r: 0, g: 0, b: 0 };
    doom_println!(platform, "I_GetPaletteIndex");
    best = 0;
    best_diff = INT_MAX;
    for i in 0..256 {
        color.r = ((0xf800 & RGB565_PALETTE[i as usize] as i32) >> 11) as u8;
        color.g = ((0x7e0 & RGB565_PALETTE[i as usize] as i32) >> 5) as u8;
        color.b = (0x1f & RGB565_PALETTE[i as usize] as i32) as u8;
        diff = (r - color.r as i32) * (r - color.r as i32)
            + (g - color.g as i32) * (g - color.g as i32)
            + (b - color.b as i32) * (b - color.b as i32);
        if diff < best_diff {
            best = i;
            best_diff = diff;
        }
        if diff == 0 {
            break;
        }
    }
    best
}
pub fn i_set_window_title(state: &mut GameState, title: &str) {
    state.platform.set_window_title(title);
}
pub fn set_grab_mouse_callback() {}
