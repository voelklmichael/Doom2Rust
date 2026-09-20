use crate::d_event::DEventState;
use crate::doomdef::Pixel;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::doomgeneric::DOOMGENERIC_RESX;
use crate::doomgeneric::DOOMGENERIC_RESY;
use crate::i_input::get_event;
use crate::i_input::IInputState;
use crate::i_system::error;
use crate::m_fixed::INT_MAX;
use crate::options::Options;
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
        Self {
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
    pub const ZERO: Self = Self {
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
    pub fn r(self) -> u8 {
        self.b_g_r_a[2]
    }
    pub fn g(self) -> u8 {
        self.b_g_r_a[1]
    }
    pub fn b(self) -> u8 {
        self.b_g_r_a[0]
    }
    pub fn a(self) -> u8 {
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
pub fn init_graphics(
    i_video: &mut IVideoState,
    options: &Options,
    platform: &mut dyn DoomPlatform,
) {
    i_video.s_fb = FBScreenInfo::ZERO;
    i_video.s_fb.xres = DOOMGENERIC_RESX as u32;
    i_video.s_fb.yres = DOOMGENERIC_RESY as u32;
    i_video.s_fb.xres_virtual = i_video.s_fb.xres;
    i_video.s_fb.yres_virtual = i_video.s_fb.yres;
    let mode: &str = options.gfxmode.as_deref().unwrap_or("rgba8888");
    if mode == "rgba8888" {
        i_video.s_fb.bits_per_pixel = 32_u32;
        i_video.s_fb.blue.length = 8_u32;
        i_video.s_fb.green.length = 8_u32;
        i_video.s_fb.red.length = 8_u32;
        i_video.s_fb.transp.length = 8_u32;
        i_video.s_fb.blue.offset = 0_u32;
        i_video.s_fb.green.offset = 8_u32;
        i_video.s_fb.red.offset = 16_u32;
        i_video.s_fb.transp.offset = 24_u32;
    } else if mode == "rgb565" {
        i_video.s_fb.bits_per_pixel = 16_u32;
        i_video.s_fb.blue.length = 5_u32;
        i_video.s_fb.green.length = 6_u32;
        i_video.s_fb.red.length = 5_u32;
        i_video.s_fb.transp.length = 0_u32;
        i_video.s_fb.blue.offset = 11_u32;
        i_video.s_fb.green.offset = 5_u32;
        i_video.s_fb.red.offset = 0_u32;
        i_video.s_fb.transp.offset = 16_u32;
    } else {
        error(&format!("Unknown gfxmode value: {mode}\n"));
    }
    doom_println!(
        platform,
        "I_InitGraphics: framebuffer: x_res: {}, y_res: {}, x_virtual: {}, y_virtual: {}, bpp: {}",
        i_video.s_fb.xres,
        i_video.s_fb.yres,
        i_video.s_fb.xres_virtual,
        i_video.s_fb.yres_virtual,
        i_video.s_fb.bits_per_pixel,
    );
    doom_println!(platform,
        "I_InitGraphics: framebuffer: RGBA: {}{}{}{}, red_off: {}, green_off: {}, blue_off: {}, transp_off: {}",
        i_video.s_fb.red.length,
        i_video.s_fb.green.length,
        i_video.s_fb.blue.length,
        i_video.s_fb.transp.length,
        i_video.s_fb.red.offset,
        i_video.s_fb.green.offset,
        i_video.s_fb.blue.offset,
        i_video.s_fb.transp.offset,
    );
    doom_println!(
        platform,
        "I_InitGraphics: DOOM screen size: w x h: {} x {}",
        SCREENWIDTH,
        SCREENHEIGHT,
    );
    if let Some(scaling) = options.scaling {
        i_video.fb_scaling = scaling;
        doom_println!(
            platform,
            "I_InitGraphics: Scaling factor: {}",
            i_video.fb_scaling
        );
    } else {
        i_video.fb_scaling = i_video.s_fb.xres.wrapping_div(SCREENWIDTH as u32) as i32;
        if i_video.s_fb.yres.wrapping_div(SCREENHEIGHT as u32) < i_video.fb_scaling as u32 {
            i_video.fb_scaling = i_video.s_fb.yres.wrapping_div(SCREENHEIGHT as u32) as i32;
        }
        doom_println!(
            platform,
            "I_InitGraphics: Auto-scaling factor: {}",
            i_video.fb_scaling
        );
    }
    i_video.i_video_buffer = vec![0u8; (SCREENWIDTH * SCREENHEIGHT) as usize];
    i_video.screenvisible = true;
}
pub fn start_tic(
    d_event: &mut DEventState,
    i_input: &mut IInputState,
    platform: &mut dyn DoomPlatform,
) {
    get_event(d_event, i_input, &mut *platform);
}
pub fn finish_update(i_video: &mut IVideoState, platform: &mut dyn DoomPlatform) {
    let palette = i_video
        .colors
        .map(|c| (u32::from(c.r()) << 16) | (u32::from(c.g()) << 8) | u32::from(c.b()));
    if platform.draw_indexed_frame(&i_video.i_video_buffer, &palette) {
        return;
    }
    match i_video.s_fb.bits_per_pixel {
        32 => convert_frame_rgb32(i_video),
        16 => convert_frame_rgb565(i_video),
        bits_per_pixel => error(&format!(
            "No idea how to convert {bits_per_pixel} bpp pixels"
        )),
    }
    platform.draw_frame(&i_video.dg_screen_buffer);
}
/// Byte offset that centres a scaled Doom screen in a framebuffer line.
fn line_offset_bytes(i_video: &IVideoState) -> usize {
    i_video
        .s_fb
        .xres
        .wrapping_sub((SCREENWIDTH * i_video.fb_scaling) as u32)
        .wrapping_mul(i_video.s_fb.bits_per_pixel)
        .wrapping_div(8_u32)
        .wrapping_div(2_u32) as usize
}
/// Converts the palette-indexed screen straight into `dg_screen_buffer`, one
/// `u32` per pixel. Each source row is converted once and then copied to the
/// other `fb_scaling - 1` output lines.
fn convert_frame_rgb32(i_video: &mut IVideoState) {
    let fb = i_video.s_fb;
    let scaling = i_video.fb_scaling as usize;
    let width = fb.xres as usize;
    if scaling == 0 || width == 0 {
        return;
    }
    let x_offset = line_offset_bytes(i_video) / 4;
    let lines = i_video.dg_screen_buffer.len() / width;
    // Pack the palette once per frame, so the per-pixel work is one table load.
    let palette: [u32; 256] = i_video.colors.map(|c| {
        (i32::from(c.r()) << fb.red.offset
            | i32::from(c.g()) << fb.green.offset
            | i32::from(c.b()) << fb.blue.offset) as u32
    });
    for row in 0..SCREENHEIGHT as usize {
        let source = &i_video.i_video_buffer[row * SCREENWIDTH as usize..][..SCREENWIDTH as usize];
        let first_line = row * scaling;
        if first_line >= lines {
            break;
        }
        let first = first_line * width;
        let out = &mut i_video.dg_screen_buffer[first..first + width][x_offset..];
        for (&index, pixels) in source.iter().zip(out.chunks_exact_mut(scaling)) {
            pixels.fill(palette[usize::from(index)]);
        }
        for copy in 1..scaling.min(lines - first_line) {
            i_video
                .dg_screen_buffer
                .copy_within(first..first + width, first + copy * width);
        }
    }
}
/// Converts the palette-indexed screen to RGB565, two pixels packed per `u32`
/// of `dg_screen_buffer`.
fn convert_frame_rgb565(i_video: &mut IVideoState) {
    let scaling = i_video.fb_scaling as usize;
    let line_bytes = i_video.s_fb.xres as usize * 2;
    let x_offset = line_offset_bytes(i_video);
    let mut frame = vec![0u8; line_bytes * SCREENHEIGHT as usize * scaling];
    let mut line_out = 0usize;
    for row in 0..SCREENHEIGHT as usize {
        let source = &i_video.i_video_buffer[row * SCREENWIDTH as usize..][..SCREENWIDTH as usize];
        let first_line = &mut frame[line_out * line_bytes..][..line_bytes];
        let mut out = x_offset;
        for &index in source {
            let c = i_video.colors[index as usize];
            let p: u16 = ((i32::from(c.r()) & 0xf8) << 8
                | (i32::from(c.g()) & 0xfc) << 3
                | i32::from(c.b()) >> 3) as u16;
            for _ in 0..scaling {
                first_line[out..out + 2].copy_from_slice(&p.to_ne_bytes());
                out += 2;
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
    for (pixel, bytes) in i_video
        .dg_screen_buffer
        .iter_mut()
        .zip(frame.as_chunks::<4>().0.iter())
    {
        *pixel = u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    }
}
pub fn read_screen(i_video: &IVideoState) -> Vec<u8> {
    i_video.i_video_buffer[..(SCREENWIDTH * SCREENHEIGHT) as usize].to_vec()
}
pub fn set_palette(i_video: &mut IVideoState, palette: &[u8]) {
    let gamma = &GAMMATABLE[i_video.usegamma as usize];
    for (color, rgb) in i_video
        .colors
        .iter_mut()
        .zip(palette.as_chunks::<3>().0.iter())
    {
        color.set_a(0_u32);
        color.set_r(u32::from(gamma[rgb[0] as usize]));
        color.set_g(u32::from(gamma[rgb[1] as usize]));
        color.set_b(u32::from(gamma[rgb[2] as usize]));
    }
}
pub fn get_palette_index(platform: &mut dyn DoomPlatform, r: i32, g: i32, b: i32) -> i32 {
    let mut color: Column = Column { r: 0, g: 0, b: 0 };
    doom_println!(platform, "I_GetPaletteIndex");
    let mut best: i32 = 0;
    let mut best_diff: i32 = INT_MAX;
    for i in 0..256 {
        color.r = ((0xf800 & i32::from(RGB565_PALETTE[i as usize])) >> 11) as u8;
        color.g = ((0x7e0 & i32::from(RGB565_PALETTE[i as usize])) >> 5) as u8;
        color.b = (0x1f & i32::from(RGB565_PALETTE[i as usize])) as u8;
        let diff: i32 = (r - i32::from(color.r)) * (r - i32::from(color.r))
            + (g - i32::from(color.g)) * (g - i32::from(color.g))
            + (b - i32::from(color.b)) * (b - i32::from(color.b));
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
pub fn i_set_window_title(platform: &mut dyn DoomPlatform, title: &str) {
    platform.set_window_title(title);
}
pub fn set_grab_mouse_callback() {}
