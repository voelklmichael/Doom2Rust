use crate::doomdef::true_0;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::m_controls::KEY_BACKSPACE;
use crate::m_controls::KEY_ENTER;
use crate::r_draw::R_VideoErase;
use crate::v_video::V_CachePatchNum;
use crate::v_video::Screen;
use crate::v_video::V_DrawPatchDirect;
#[derive(Clone)]
pub struct hu_textline_t {
    pub x: i32,
    pub y: i32,
    pub sc: i32,
    pub l: String,
    pub needsupdate: i32,
}
#[derive(Clone)]
pub struct hu_stext_t {
    pub l: [hu_textline_t; 4],
    pub h: i32,
    pub cl: i32,
    pub laston: bool,
}
#[derive(Clone)]
pub struct hu_itext_t {
    pub l: hu_textline_t,
    pub lm: i32,
    pub laston: bool,
}
pub const HU_MAXLINELENGTH: i32 = 80;
pub fn HUlib_clearTextLine(t: &mut hu_textline_t) {
    t.l.clear();
    t.needsupdate = true_0;
}
pub fn HUlib_initTextLine(t: &mut hu_textline_t, x: i32, y: i32, sc: i32) {
    t.x = x;
    t.y = y;
    t.sc = sc;
    HUlib_clearTextLine(t);
}
pub fn HUlib_addCharToTextLine(t: &mut hu_textline_t, ch: u8) -> bool {
    if t.l.len() as i32 == HU_MAXLINELENGTH {
        false
    } else {
        t.l.push(ch as char);
        t.needsupdate = 4_i32;
        true
    }
}
pub fn HUlib_delCharFromTextLine(t: &mut hu_textline_t) -> bool {
    if t.l.is_empty() {
        false
    } else {
        t.l.pop();
        t.needsupdate = 4_i32;
        true
    }
}
pub fn HUlib_drawTextLine(state: &mut GameState, l: &hu_textline_t, drawcursor: bool) {
    let mut x = l.x;
    for i in 0..l.l.len() {
        let c = l.l.as_bytes()[i].to_ascii_uppercase();
        if c as i32 != ' ' as i32 && c as i32 >= l.sc && c as i32 <= '_' as i32 {
            let glyph = state.hu_stuff.hu_font[(c as i32 - l.sc) as usize];
            let patch = V_CachePatchNum(state, glyph);
            let w = patch.width();
            if x + w > SCREENWIDTH {
                break;
            }
            V_DrawPatchDirect(state, Screen::Video, x, l.y, &patch);
            x += w;
        } else {
            x += 4_i32;
            if x >= SCREENWIDTH {
                break;
            }
        }
    }
    if drawcursor {
        let cursor_glyph = state.hu_stuff.hu_font[('_' as i32 - l.sc) as usize];
        let cursor_patch = V_CachePatchNum(state, cursor_glyph);
        if x + cursor_patch.width() <= SCREENWIDTH {
            V_DrawPatchDirect(state, Screen::Video, x, l.y, &cursor_patch);
        }
    }
}
pub fn HUlib_eraseTextLine(state: &mut GameState, l: &mut hu_textline_t) {
    if !state.am_map.automapactive && state.r_draw.viewwindowx != 0 && l.needsupdate != 0 {
        let glyph = state.hu_stuff.hu_font[0];
        let patch = V_CachePatchNum(state, glyph);
        let lh = patch.height() + 1_i32;
        let mut y = l.y;
        let mut yoffset = y * SCREENWIDTH;
        while y < l.y + lh {
            if y < state.r_draw.viewwindowy
                || y >= state.r_draw.viewwindowy + state.r_draw.viewheight
            {
                R_VideoErase(state, yoffset as u32, SCREENWIDTH);
            } else {
                let viewwindowx = state.r_draw.viewwindowx;
                let second_ofs =
                    (yoffset + state.r_draw.viewwindowx + state.r_draw.viewwidth) as u32;
                R_VideoErase(state, yoffset as u32, viewwindowx);
                R_VideoErase(state, second_ofs, viewwindowx);
            }
            y += 1;
            yoffset += SCREENWIDTH;
        }
    }
    if l.needsupdate != 0 {
        l.needsupdate -= 1;
    }
}
pub fn HUlib_initSText(
    s: &mut hu_stext_t,
    x: i32,
    y: i32,
    h: i32,
    startchar: i32,
    font0_height: i32,
) {
    s.h = h;
    s.laston = true;
    s.cl = 0_i32;
    for i in 0..h {
        HUlib_initTextLine(
            &mut s.l[i as usize],
            x,
            y - i * (font0_height + 1_i32),
            startchar,
        );
    }
}
pub fn HUlib_addLineToSText(s: &mut hu_stext_t) {
    s.cl += 1;
    if s.cl == s.h {
        s.cl = 0_i32;
    }
    HUlib_clearTextLine(&mut s.l[s.cl as usize]);
    for i in 0..s.h {
        s.l[i as usize].needsupdate = 4_i32;
    }
}
pub fn HUlib_addMessageToSText(s: &mut hu_stext_t, prefix: Option<&str>, msg: &str) {
    HUlib_addLineToSText(s);
    let cl = s.cl as usize;
    if let Some(prefix) = prefix {
        for b in prefix.bytes() {
            HUlib_addCharToTextLine(&mut s.l[cl], b);
        }
    }
    for b in msg.bytes() {
        HUlib_addCharToTextLine(&mut s.l[cl], b);
    }
}
pub fn HUlib_drawSText(state: &mut GameState, s: &hu_stext_t, on: bool) {
    if !on {
        return;
    }
    for i in 0..s.h {
        let mut idx = s.cl - i;
        if idx < 0_i32 {
            idx += s.h;
        }
        HUlib_drawTextLine(state, &s.l[idx as usize], false);
    }
}
pub fn HUlib_eraseSText(state: &mut GameState, s: &mut hu_stext_t, on: bool) {
    for i in 0..s.h {
        if s.laston && !on {
            s.l[i as usize].needsupdate = 4_i32;
        }
        HUlib_eraseTextLine(state, &mut s.l[i as usize]);
    }
    s.laston = on;
}
pub fn HUlib_initIText(it: &mut hu_itext_t, x: i32, y: i32, startchar: i32) {
    it.lm = 0_i32;
    it.laston = true;
    HUlib_initTextLine(&mut it.l, x, y, startchar);
}
pub fn HUlib_delCharFromIText(it: &mut hu_itext_t) {
    if it.l.l.len() as i32 != it.lm {
        HUlib_delCharFromTextLine(&mut it.l);
    }
}
pub fn HUlib_eraseLineFromIText(it: &mut hu_itext_t) {
    while it.lm != it.l.l.len() as i32 {
        HUlib_delCharFromTextLine(&mut it.l);
    }
}
pub fn HUlib_resetIText(it: &mut hu_itext_t) {
    it.lm = 0_i32;
    HUlib_clearTextLine(&mut it.l);
}
pub fn HUlib_addPrefixToIText(it: &mut hu_itext_t, s: &str) {
    for b in s.bytes() {
        HUlib_addCharToTextLine(&mut it.l, b);
    }
    it.lm = it.l.l.len() as i32;
}
pub fn HUlib_keyInIText(it: &mut hu_itext_t, ch: u8) -> bool {
    let ch = ch.to_ascii_uppercase();
    if ch as i32 >= ' ' as i32 && ch as i32 <= '_' as i32 {
        HUlib_addCharToTextLine(&mut it.l, ch);
    } else if ch as i32 == KEY_BACKSPACE {
        HUlib_delCharFromIText(it);
    } else if ch as i32 != KEY_ENTER {
        return false;
    }
    true
}
pub fn HUlib_drawIText(state: &mut GameState, it: &hu_itext_t, on: bool) {
    if !on {
        return;
    }
    HUlib_drawTextLine(state, &it.l, true);
}
pub fn HUlib_eraseIText(state: &mut GameState, it: &mut hu_itext_t, on: bool) {
    if it.laston && !on {
        it.l.needsupdate = 4_i32;
    }
    HUlib_eraseTextLine(state, &mut it.l);
    it.laston = on;
}
