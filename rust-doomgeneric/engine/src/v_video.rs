use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::filesystem::DoomFileSystem;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::i_video::get_palette_index;
use crate::i_video::IVideoState;
use crate::m_bbox::add_to_box;
use crate::m_fixed::Fixed;
use crate::patch::Patch;
use crate::platform::DoomPlatform;
use crate::w_wad::lump_bytes;
use crate::w_wad::lump_bytes_name;
use alloc::string::String;
use alloc::vec::Vec;

/// Which framebuffer a drawing call targets.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Screen {
    /// The main video buffer (what gets blitted to the window).
    Video,
    /// The status bar's pre-rendered backdrop.
    StatusBar,
    /// The pre-rendered border/background behind a reduced view window.
    Background,
}

impl GameState {
    pub fn screen(&self, screen: Screen) -> &[u8] {
        match screen {
            Screen::Video => &self.i_video.i_video_buffer,
            Screen::StatusBar => &self.st_stuff.st_backing_screen,
            Screen::Background => self
                .r_draw
                .background_buffer
                .as_deref()
                .expect("background screen not allocated"),
        }
    }

    pub fn screen_mut(&mut self, screen: Screen) -> &mut [u8] {
        match screen {
            Screen::Video => &mut self.i_video.i_video_buffer,
            Screen::StatusBar => &mut self.st_stuff.st_backing_screen,
            Screen::Background => self
                .r_draw
                .background_buffer
                .as_deref_mut()
                .expect("background screen not allocated"),
        }
    }
}

pub struct VVideoState {
    pub dirtybox: [i32; 4],
}
impl Default for VVideoState {
    fn default() -> Self {
        Self::new()
    }
}

impl VVideoState {
    pub const fn new() -> Self {
        VVideoState { dirtybox: [0; 4] }
    }
}
pub fn mark_rect(state: &mut GameState, dest: Screen, x: i32, y: i32, width: i32, height: i32) {
    if dest == Screen::Video {
        add_to_box(&mut state.v_video.dirtybox, x as Fixed, y as Fixed);
        add_to_box(
            &mut state.v_video.dirtybox,
            x as Fixed + width as Fixed - 1,
            y as Fixed + height as Fixed - 1,
        );
    }
}
#[allow(clippy::too_many_arguments)]
pub fn copy_rect(
    state: &mut GameState,
    dest: Screen,
    srcx: i32,
    srcy: i32,
    source: Screen,
    width: i32,
    height: i32,
    destx: i32,
    desty: i32,
) {
    if srcx < 0
        || srcx + width > SCREENWIDTH
        || srcy < 0
        || srcy + height > SCREENHEIGHT
        || destx < 0
        || destx + width > SCREENWIDTH
        || desty < 0
        || desty + height > SCREENHEIGHT
    {
        error("Bad V_CopyRect");
    }
    mark_rect(state, dest, destx, desty, width, height);
    let width = width as usize;
    let mut rows: Vec<u8> = Vec::with_capacity(width * height as usize);
    {
        let src = state.screen(source);
        for row in 0..height {
            let start = ((srcy + row) * SCREENWIDTH + srcx) as usize;
            rows.extend_from_slice(&src[start..start + width]);
        }
    }
    let dst = state.screen_mut(dest);
    for row in 0..height {
        let start = ((desty + row) * SCREENWIDTH + destx) as usize;
        dst[start..start + width].copy_from_slice(&rows[row as usize * width..][..width]);
    }
}
/// Resolves a WAD lump number to its cached patch data. Cheap and
/// idempotent: the lump cache never evicts.
pub fn cache_patch_num(state: &mut GameState, lumpnum: i32) -> Patch {
    Patch::new(lump_bytes(state, lumpnum))
}
pub fn cache_patch_name(state: &mut GameState, name: &str) -> Patch {
    Patch::new(lump_bytes_name(state, name))
}
fn blit_patch(screen: &mut [u8], patch: &Patch, x: i32, y: i32, flipped: bool) {
    let w = patch.width();
    for col in 0..w {
        let source_column = if flipped { w - 1 - col } else { col };
        for post in patch.posts(source_column) {
            let mut index = ((y + post.topdelta as i32) * SCREENWIDTH + x + col) as usize;
            for &pixel in post.pixels {
                screen[index] = pixel;
                index += SCREENWIDTH as usize;
            }
        }
    }
}
pub fn draw_patch(state: &mut GameState, dest: Screen, x: i32, y: i32, patch: &Patch) {
    let y = y - patch.topoffset();
    let x = x - patch.leftoffset();
    if x < 0 || x + patch.width() > SCREENWIDTH || y < 0 || y + patch.height() > SCREENHEIGHT {
        error(&format!(
            "Bad V_DrawPatch x={} y={} patch.width={} patch.height={} topoffset={} leftoffset={}",
            x,
            y,
            patch.width(),
            patch.height(),
            patch.topoffset(),
            patch.leftoffset(),
        ));
    }
    mark_rect(state, dest, x, y, patch.width(), patch.height());
    blit_patch(state.screen_mut(dest), patch, x, y, false);
}
pub fn draw_patch_flipped(state: &mut GameState, dest: Screen, x: i32, y: i32, patch: &Patch) {
    let y = y - patch.topoffset();
    let x = x - patch.leftoffset();
    if x < 0 || x + patch.width() > SCREENWIDTH || y < 0 || y + patch.height() > SCREENHEIGHT {
        error("Bad V_DrawPatchFlipped");
    }
    mark_rect(state, dest, x, y, patch.width(), patch.height());
    blit_patch(state.screen_mut(dest), patch, x, y, true);
}
pub fn draw_patch_direct(state: &mut GameState, dest: Screen, x: i32, y: i32, patch: &Patch) {
    draw_patch(state, dest, x, y, patch);
}
pub fn draw_block(
    state: &mut GameState,
    dest: Screen,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    src: &[u8],
) {
    if x < 0 || x + width > SCREENWIDTH || y < 0 || y + height > SCREENHEIGHT {
        error("Bad V_DrawBlock");
    }
    mark_rect(state, dest, x, y, width, height);
    let width = width as usize;
    let dst = state.screen_mut(dest);
    for row in 0..height as usize {
        let start = (y as usize + row) * SCREENWIDTH as usize + x as usize;
        dst[start..start + width].copy_from_slice(&src[row * width..][..width]);
    }
}
pub fn draw_filled_box(state: &mut IVideoState, x: i32, y: i32, w: i32, h: i32, c: i32) {
    for row in 0..h {
        let start = (SCREENWIDTH * (y + row) + x) as usize;
        state.i_video_buffer[start..start + w as usize].fill(c as u8);
    }
}
pub fn draw_horiz_line(state: &mut IVideoState, x: i32, y: i32, w: i32, c: i32) {
    let start = (SCREENWIDTH * y + x) as usize;
    state.i_video_buffer[start..start + w as usize].fill(c as u8);
}
pub fn draw_vert_line(state: &mut IVideoState, x: i32, y: i32, h: i32, c: i32) {
    for row in 0..h {
        state.i_video_buffer[(SCREENWIDTH * (y + row) + x) as usize] = c as u8;
    }
}
pub fn draw_box(state: &mut IVideoState, x: i32, y: i32, w: i32, h: i32, c: i32) {
    draw_horiz_line(state, x, y, w, c);
    draw_horiz_line(state, x, y + h - 1, w, c);
    draw_vert_line(state, x, y, h, c);
    draw_vert_line(state, x + w - 1, y, h, c);
}
pub fn write_pcxfile(
    fs: &mut dyn DoomFileSystem,
    filename: &str,
    data: &[u8],
    width: i32,
    height: i32,
    palette: &[u8],
) {
    // 128-byte on-disk PCX header.
    let mut pack: Vec<u8> = Vec::with_capacity((128 + width * height * 2 + 768 + 1) as usize);
    pack.extend_from_slice(&[0xa, 5, 1, 8]);
    pack.extend_from_slice(&0_u16.to_le_bytes());
    pack.extend_from_slice(&0_u16.to_le_bytes());
    pack.extend_from_slice(&((width - 1) as i16 as u16).to_le_bytes());
    pack.extend_from_slice(&((height - 1) as i16 as u16).to_le_bytes());
    pack.extend_from_slice(&(width as i16 as u16).to_le_bytes());
    pack.extend_from_slice(&(height as i16 as u16).to_le_bytes());
    pack.extend_from_slice(&[0u8; 48]);
    pack.push(0);
    pack.push(1);
    pack.extend_from_slice(&(width as i16 as u16).to_le_bytes());
    pack.extend_from_slice(&(2_i16 as u16).to_le_bytes());
    pack.extend_from_slice(&[0u8; 58]);
    debug_assert_eq!(pack.len(), 128);
    for &pixel in &data[..(width * height) as usize] {
        if pixel as i32 & 0xc0 != 0xc0 {
            pack.push(pixel);
        } else {
            pack.push(0xc1_u8);
            pack.push(pixel);
        }
    }
    pack.push(0xc_u8);
    pack.extend_from_slice(&palette[..768]);
    fs.write_file(filename, &pack);
}
pub fn v_screen_shot(state: &mut GameState) {
    let mut i = 0i32;
    let mut lbmname = String::new();
    while i <= 99 {
        lbmname = format!("DOOM{i:02}.pcx");
        if !state.fs.exists(&lbmname) {
            break;
        }
        i += 1;
    }
    if i == 100 {
        error("V_ScreenShot: Couldn't create a PCX");
    }
    let palette = lump_bytes_name(state, "PLAYPAL");
    write_pcxfile(
        &mut *state.fs,
        &lbmname,
        &state.i_video.i_video_buffer,
        SCREENWIDTH,
        SCREENHEIGHT,
        &palette,
    );
}
pub const MOUSE_SPEED_BOX_WIDTH: i32 = 120;
pub const MOUSE_SPEED_BOX_HEIGHT: i32 = 9;
pub fn draw_mouse_speed_box(state: &mut IVideoState, platform: &mut dyn DoomPlatform, speed: i32) {
    let mut original_speed: i32;

    let mut linelen: i32;
    let bgcolor: i32 = get_palette_index(platform, 0x77, 0x77, 0x77);
    let bordercolor: i32 = get_palette_index(platform, 0x55, 0x55, 0x55);
    let red: i32 = get_palette_index(platform, 0xff, 0, 0);
    let black: i32 = get_palette_index(platform, 0, 0, 0);
    let yellow: i32 = get_palette_index(platform, 0xff, 0xff, 0);
    let white: i32 = get_palette_index(platform, 0xff, 0xff, 0xff);
    if state.usemouse == 0 || ((state.mouse_acceleration - 1_f32) as f64).abs() < 0.01f64 {
        return;
    }
    let box_x: i32 = SCREENWIDTH - MOUSE_SPEED_BOX_WIDTH - 10;
    let box_y: i32 = 15;
    draw_filled_box(
        state,
        box_x,
        box_y,
        MOUSE_SPEED_BOX_WIDTH,
        MOUSE_SPEED_BOX_HEIGHT,
        bgcolor,
    );
    draw_box(
        state,
        box_x,
        box_y,
        MOUSE_SPEED_BOX_WIDTH,
        MOUSE_SPEED_BOX_HEIGHT,
        bordercolor,
    );
    let redline_x: i32 = MOUSE_SPEED_BOX_WIDTH / 3;
    if speed < state.mouse_threshold {
        original_speed = speed;
    } else {
        original_speed = speed - state.mouse_threshold;
        original_speed = (original_speed as f32 / state.mouse_acceleration) as i32;
        original_speed += state.mouse_threshold;
    }
    linelen = original_speed * redline_x / state.mouse_threshold;
    if linelen > MOUSE_SPEED_BOX_WIDTH - 1 {
        linelen = MOUSE_SPEED_BOX_WIDTH - 1;
    }
    draw_horiz_line(
        state,
        box_x + 1,
        box_y + 4,
        MOUSE_SPEED_BOX_WIDTH - 2,
        black,
    );
    if linelen < redline_x {
        draw_horiz_line(
            state,
            box_x + 1,
            box_y + MOUSE_SPEED_BOX_HEIGHT / 2,
            linelen,
            white,
        );
    } else {
        draw_horiz_line(
            state,
            box_x + 1,
            box_y + MOUSE_SPEED_BOX_HEIGHT / 2,
            redline_x,
            white,
        );
        draw_horiz_line(
            state,
            box_x + redline_x,
            box_y + MOUSE_SPEED_BOX_HEIGHT / 2,
            linelen - redline_x,
            yellow,
        );
    }
    draw_vert_line(
        state,
        box_x + redline_x,
        box_y + 1,
        MOUSE_SPEED_BOX_HEIGHT - 2,
        red,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::MemFileSystem;

    #[test]
    fn pcx_is_written_through_the_filesystem() {
        let mut fs = MemFileSystem::default();
        let palette = [7u8; 768];
        write_pcxfile(&mut fs, "DOOM00.pcx", &[1, 2, 0xc5, 4], 2, 2, &palette);
        let file = &fs.files["DOOM00.pcx"];
        assert_eq!(file[0], 0x0a);
        // 128-byte header, 4 pixels (one needs an 0xc1 run-length escape), 0x0c + palette.
        assert_eq!(file.len(), 128 + 5 + 1 + 768);
        assert_eq!(file[128 + 2], 0xc1);
        assert_eq!(file[128 + 3], 0xc5);
    }
}
