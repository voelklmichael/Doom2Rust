use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::filesystem::DoomFileSystem;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::i_video::IVideoState;
use crate::i_video::I_GetPaletteIndex;
use crate::m_bbox::M_AddToBox;
use crate::m_fixed::fixed_t;
use crate::patch::Patch;
use crate::platform::DoomPlatform;
use crate::stdint_types::byte;
use crate::w_wad::W_LumpBytes;
use crate::w_wad::W_LumpBytesName;
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
    pub fn screen(&self, screen: Screen) -> &[byte] {
        match screen {
            Screen::Video => &self.i_video.I_VideoBuffer,
            Screen::StatusBar => &self.st_stuff.st_backing_screen,
            Screen::Background => self
                .r_draw
                .background_buffer
                .as_deref()
                .expect("background screen not allocated"),
        }
    }

    pub fn screen_mut(&mut self, screen: Screen) -> &mut [byte] {
        match screen {
            Screen::Video => &mut self.i_video.I_VideoBuffer,
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
pub fn V_MarkRect(state: &mut GameState, dest: Screen, x: i32, y: i32, width: i32, height: i32) {
    if dest == Screen::Video {
        M_AddToBox(&mut state.v_video.dirtybox, x as fixed_t, y as fixed_t);
        M_AddToBox(
            &mut state.v_video.dirtybox,
            x as fixed_t + width as fixed_t - 1 as fixed_t,
            y as fixed_t + height as fixed_t - 1 as fixed_t,
        );
    }
}
#[allow(clippy::too_many_arguments)]
pub fn V_CopyRect(
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
    if srcx < 0_i32
        || srcx + width > SCREENWIDTH
        || srcy < 0_i32
        || srcy + height > SCREENHEIGHT
        || destx < 0_i32
        || destx + width > SCREENWIDTH
        || desty < 0_i32
        || desty + height > SCREENHEIGHT
    {
        I_Error("Bad V_CopyRect");
    }
    V_MarkRect(state, dest, destx, desty, width, height);
    let width = width as usize;
    let mut rows: Vec<byte> = Vec::with_capacity(width * height as usize);
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
pub fn V_CachePatchNum(state: &mut GameState, lumpnum: i32) -> Patch {
    Patch::new(W_LumpBytes(state, lumpnum))
}
pub fn V_CachePatchName(state: &mut GameState, name: &str) -> Patch {
    Patch::new(W_LumpBytesName(state, name))
}
fn blit_patch(screen: &mut [byte], patch: &Patch, x: i32, y: i32, flipped: bool) {
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
pub fn V_DrawPatch(state: &mut GameState, dest: Screen, x: i32, y: i32, patch: &Patch) {
    let y = y - patch.topoffset();
    let x = x - patch.leftoffset();
    if x < 0_i32
        || x + patch.width() > SCREENWIDTH
        || y < 0_i32
        || y + patch.height() > SCREENHEIGHT
    {
        I_Error(&format!(
            "Bad V_DrawPatch x={} y={} patch.width={} patch.height={} topoffset={} leftoffset={}",
            x,
            y,
            patch.width(),
            patch.height(),
            patch.topoffset(),
            patch.leftoffset(),
        ));
    }
    V_MarkRect(state, dest, x, y, patch.width(), patch.height());
    blit_patch(state.screen_mut(dest), patch, x, y, false);
}
pub fn V_DrawPatchFlipped(state: &mut GameState, dest: Screen, x: i32, y: i32, patch: &Patch) {
    let y = y - patch.topoffset();
    let x = x - patch.leftoffset();
    if x < 0_i32
        || x + patch.width() > SCREENWIDTH
        || y < 0_i32
        || y + patch.height() > SCREENHEIGHT
    {
        I_Error("Bad V_DrawPatchFlipped");
    }
    V_MarkRect(state, dest, x, y, patch.width(), patch.height());
    blit_patch(state.screen_mut(dest), patch, x, y, true);
}
pub fn V_DrawPatchDirect(state: &mut GameState, dest: Screen, x: i32, y: i32, patch: &Patch) {
    V_DrawPatch(state, dest, x, y, patch);
}
pub fn V_DrawBlock(
    state: &mut GameState,
    dest: Screen,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    src: &[byte],
) {
    if x < 0_i32 || x + width > SCREENWIDTH || y < 0_i32 || y + height > SCREENHEIGHT {
        I_Error("Bad V_DrawBlock");
    }
    V_MarkRect(state, dest, x, y, width, height);
    let width = width as usize;
    let dst = state.screen_mut(dest);
    for row in 0..height as usize {
        let start = (y as usize + row) * SCREENWIDTH as usize + x as usize;
        dst[start..start + width].copy_from_slice(&src[row * width..][..width]);
    }
}
pub fn V_DrawFilledBox(state: &mut IVideoState, x: i32, y: i32, w: i32, h: i32, c: i32) {
    for row in 0..h {
        let start = (SCREENWIDTH * (y + row) + x) as usize;
        state.I_VideoBuffer[start..start + w as usize].fill(c as byte);
    }
}
pub fn V_DrawHorizLine(state: &mut IVideoState, x: i32, y: i32, w: i32, c: i32) {
    let start = (SCREENWIDTH * y + x) as usize;
    state.I_VideoBuffer[start..start + w as usize].fill(c as byte);
}
pub fn V_DrawVertLine(state: &mut IVideoState, x: i32, y: i32, h: i32, c: i32) {
    for row in 0..h {
        state.I_VideoBuffer[(SCREENWIDTH * (y + row) + x) as usize] = c as byte;
    }
}
pub fn V_DrawBox(state: &mut IVideoState, x: i32, y: i32, w: i32, h: i32, c: i32) {
    V_DrawHorizLine(state, x, y, w, c);
    V_DrawHorizLine(state, x, y + h - 1_i32, w, c);
    V_DrawVertLine(state, x, y, h, c);
    V_DrawVertLine(state, x + w - 1_i32, y, h, c);
}
pub fn WritePCXfile(
    fs: &mut dyn DoomFileSystem,
    filename: &str,
    data: &[byte],
    width: i32,
    height: i32,
    palette: &[byte],
) {
    // 128-byte on-disk PCX header.
    let mut pack: Vec<u8> =
        Vec::with_capacity((128 + width * height * 2_i32 + 768_i32 + 1_i32) as usize);
    pack.extend_from_slice(&[0xa_u8, 5_u8, 1_u8, 8_u8]);
    pack.extend_from_slice(&0_u16.to_le_bytes());
    pack.extend_from_slice(&0_u16.to_le_bytes());
    pack.extend_from_slice(&((width - 1_i32) as i16 as u16).to_le_bytes());
    pack.extend_from_slice(&((height - 1_i32) as i16 as u16).to_le_bytes());
    pack.extend_from_slice(&(width as i16 as u16).to_le_bytes());
    pack.extend_from_slice(&(height as i16 as u16).to_le_bytes());
    pack.extend_from_slice(&[0u8; 48]);
    pack.push(0);
    pack.push(1_u8);
    pack.extend_from_slice(&(width as i16 as u16).to_le_bytes());
    pack.extend_from_slice(&(2_i32 as i16 as u16).to_le_bytes());
    pack.extend_from_slice(&[0u8; 58]);
    debug_assert_eq!(pack.len(), 128);
    for &pixel in &data[..(width * height) as usize] {
        if pixel as i32 & 0xc0_i32 != 0xc0_i32 {
            pack.push(pixel);
        } else {
            pack.push(0xc1 as byte);
            pack.push(pixel);
        }
    }
    pack.push(0xc as byte);
    pack.extend_from_slice(&palette[..768]);
    fs.write_file(filename, &pack);
}
pub fn V_ScreenShot(state: &mut GameState) {
    let mut i = 0i32;
    let mut lbmname = String::new();
    while i <= 99 {
        lbmname = format!("DOOM{i:02}.pcx");
        if !state.fs.exists(&lbmname) {
            break;
        }
        i += 1;
    }
    if i == 100_i32 {
        I_Error("V_ScreenShot: Couldn't create a PCX");
    }
    let palette = W_LumpBytesName(state, "PLAYPAL");
    WritePCXfile(
        &mut *state.fs,
        &lbmname,
        &state.i_video.I_VideoBuffer,
        SCREENWIDTH,
        SCREENHEIGHT,
        &palette,
    );
}
pub const MOUSE_SPEED_BOX_WIDTH: i32 = 120;
pub const MOUSE_SPEED_BOX_HEIGHT: i32 = 9;
pub fn V_DrawMouseSpeedBox(state: &mut IVideoState, platform: &mut dyn DoomPlatform, speed: i32) {
    let mut original_speed: i32;

    let mut linelen: i32;
    let bgcolor: i32 = I_GetPaletteIndex(platform, 0x77_i32, 0x77_i32, 0x77_i32);
    let bordercolor: i32 = I_GetPaletteIndex(platform, 0x55_i32, 0x55_i32, 0x55_i32);
    let red: i32 = I_GetPaletteIndex(platform, 0xff_i32, 0_i32, 0_i32);
    let black: i32 = I_GetPaletteIndex(platform, 0_i32, 0_i32, 0_i32);
    let yellow: i32 = I_GetPaletteIndex(platform, 0xff_i32, 0xff_i32, 0_i32);
    let white: i32 = I_GetPaletteIndex(platform, 0xff_i32, 0xff_i32, 0xff_i32);
    if state.usemouse == 0 || ((state.mouse_acceleration - 1_f32) as f64).abs() < 0.01f64 {
        return;
    }
    let box_x: i32 = SCREENWIDTH - MOUSE_SPEED_BOX_WIDTH - 10_i32;
    let box_y: i32 = 15_i32;
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
    let redline_x: i32 = MOUSE_SPEED_BOX_WIDTH / 3_i32;
    if speed < state.mouse_threshold {
        original_speed = speed;
    } else {
        original_speed = speed - state.mouse_threshold;
        original_speed = (original_speed as f32 / state.mouse_acceleration) as i32;
        original_speed += state.mouse_threshold;
    }
    linelen = original_speed * redline_x / state.mouse_threshold;
    if linelen > MOUSE_SPEED_BOX_WIDTH - 1_i32 {
        linelen = MOUSE_SPEED_BOX_WIDTH - 1_i32;
    }
    V_DrawHorizLine(
        state,
        box_x + 1_i32,
        box_y + 4_i32,
        MOUSE_SPEED_BOX_WIDTH - 2_i32,
        black,
    );
    if linelen < redline_x {
        V_DrawHorizLine(
            state,
            box_x + 1_i32,
            box_y + MOUSE_SPEED_BOX_HEIGHT / 2_i32,
            linelen,
            white,
        );
    } else {
        V_DrawHorizLine(
            state,
            box_x + 1_i32,
            box_y + MOUSE_SPEED_BOX_HEIGHT / 2_i32,
            redline_x,
            white,
        );
        V_DrawHorizLine(
            state,
            box_x + redline_x,
            box_y + MOUSE_SPEED_BOX_HEIGHT / 2_i32,
            linelen - redline_x,
            yellow,
        );
    }
    V_DrawVertLine(
        state,
        box_x + redline_x,
        box_y + 1_i32,
        MOUSE_SPEED_BOX_HEIGHT - 2_i32,
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
        WritePCXfile(&mut fs, "DOOM00.pcx", &[1, 2, 0xc5, 4], 2, 2, &palette);
        let file = &fs.files["DOOM00.pcx"];
        assert_eq!(file[0], 0x0a);
        // 128-byte header, 4 pixels (one needs an 0xc1 run-length escape), 0x0c + palette.
        assert_eq!(file.len(), 128 + 5 + 1 + 768);
        assert_eq!(file[128 + 2], 0xc1);
        assert_eq!(file[128 + 3], 0xc5);
    }
}
