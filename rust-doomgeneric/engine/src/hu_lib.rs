use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::m_controls::KEY_BACKSPACE;
use crate::m_controls::KEY_ENTER;
use crate::r_draw::video_erase;
use crate::v_video::cache_patch_num;
use crate::v_video::draw_patch_direct;
use crate::v_video::Screen;
use alloc::string::String;
#[derive(Clone)]
pub struct HuTextLine {
    pub x: i32,
    pub y: i32,
    pub sc: i32,
    pub l: String,
    pub needsupdate: i32,
}
#[derive(Clone)]
pub struct HuSText {
    pub l: [HuTextLine; 4],
    pub h: i32,
    pub cl: i32,
    pub laston: bool,
}
#[derive(Clone)]
pub struct HuIText {
    pub l: HuTextLine,
    pub lm: i32,
    pub laston: bool,
}
pub const HU_MAXLINELENGTH: i32 = 80;
pub fn hulib_clear_text_line(t: &mut HuTextLine) {
    t.l.clear();
    t.needsupdate = 1;
}
pub fn hulib_init_text_line(t: &mut HuTextLine, x: i32, y: i32, sc: i32) {
    t.x = x;
    t.y = y;
    t.sc = sc;
    hulib_clear_text_line(t);
}
pub fn hulib_add_char_to_text_line(t: &mut HuTextLine, ch: u8) -> bool {
    if t.l.len() as i32 == HU_MAXLINELENGTH {
        false
    } else {
        t.l.push(ch as char);
        t.needsupdate = 4;
        true
    }
}
pub fn hulib_del_char_from_text_line(t: &mut HuTextLine) -> bool {
    if t.l.is_empty() {
        false
    } else {
        t.l.pop();
        t.needsupdate = 4;
        true
    }
}
pub fn hulib_draw_text_line(state: &mut GameState, l: &HuTextLine, drawcursor: bool) {
    let mut x = l.x;
    for i in 0..l.l.len() {
        let c = l.l.as_bytes()[i].to_ascii_uppercase();
        if c as i32 != ' ' as i32 && c as i32 >= l.sc && c as i32 <= '_' as i32 {
            let glyph = state.hu_stuff.hu_font[(c as i32 - l.sc) as usize];
            let patch = cache_patch_num(state, glyph);
            let w = patch.width();
            if x + w > SCREENWIDTH {
                break;
            }
            draw_patch_direct(state, Screen::Video, x, l.y, &patch);
            x += w;
        } else {
            x += 4;
            if x >= SCREENWIDTH {
                break;
            }
        }
    }
    if drawcursor {
        let cursor_glyph = state.hu_stuff.hu_font[('_' as i32 - l.sc) as usize];
        let cursor_patch = cache_patch_num(state, cursor_glyph);
        if x + cursor_patch.width() <= SCREENWIDTH {
            draw_patch_direct(state, Screen::Video, x, l.y, &cursor_patch);
        }
    }
}
pub fn hulib_erase_text_line(state: &mut GameState, l: &mut HuTextLine) {
    if !state.am_map.automapactive && state.r_draw.viewwindowx != 0 && l.needsupdate != 0 {
        let glyph = state.hu_stuff.hu_font[0];
        let patch = cache_patch_num(state, glyph);
        let lh = patch.height() + 1;
        let mut y = l.y;
        let mut yoffset = y * SCREENWIDTH;
        while y < l.y + lh {
            if y < state.r_draw.viewwindowy
                || y >= state.r_draw.viewwindowy + state.r_draw.viewheight
            {
                video_erase(
                    &mut state.i_video,
                    &state.r_draw,
                    yoffset as u32,
                    SCREENWIDTH,
                );
            } else {
                let viewwindowx = state.r_draw.viewwindowx;
                let second_ofs =
                    (yoffset + state.r_draw.viewwindowx + state.r_draw.viewwidth) as u32;
                video_erase(
                    &mut state.i_video,
                    &state.r_draw,
                    yoffset as u32,
                    viewwindowx,
                );
                video_erase(&mut state.i_video, &state.r_draw, second_ofs, viewwindowx);
            }
            y += 1;
            yoffset += SCREENWIDTH;
        }
    }
    if l.needsupdate != 0 {
        l.needsupdate -= 1;
    }
}
pub fn hulib_init_stext(
    s: &mut HuSText,
    x: i32,
    y: i32,
    h: i32,
    startchar: i32,
    font0_height: i32,
) {
    s.h = h;
    s.laston = true;
    s.cl = 0;
    for i in 0..h {
        hulib_init_text_line(
            &mut s.l[i as usize],
            x,
            y - i * (font0_height + 1),
            startchar,
        );
    }
}
pub fn hulib_add_line_to_stext(s: &mut HuSText) {
    s.cl += 1;
    if s.cl == s.h {
        s.cl = 0;
    }
    hulib_clear_text_line(&mut s.l[s.cl as usize]);
    for i in 0..s.h {
        s.l[i as usize].needsupdate = 4;
    }
}
pub fn hulib_add_message_to_stext(s: &mut HuSText, prefix: Option<&str>, msg: &str) {
    hulib_add_line_to_stext(s);
    let cl = s.cl as usize;
    if let Some(prefix) = prefix {
        for b in prefix.bytes() {
            hulib_add_char_to_text_line(&mut s.l[cl], b);
        }
    }
    for b in msg.bytes() {
        hulib_add_char_to_text_line(&mut s.l[cl], b);
    }
}
pub fn hulib_draw_stext(state: &mut GameState, s: &HuSText, on: bool) {
    if !on {
        return;
    }
    for i in 0..s.h {
        let mut idx = s.cl - i;
        if idx < 0 {
            idx += s.h;
        }
        hulib_draw_text_line(state, &s.l[idx as usize], false);
    }
}
pub fn hulib_erase_stext(state: &mut GameState, s: &mut HuSText, on: bool) {
    for i in 0..s.h {
        if s.laston && !on {
            s.l[i as usize].needsupdate = 4;
        }
        hulib_erase_text_line(state, &mut s.l[i as usize]);
    }
    s.laston = on;
}
pub fn hulib_init_itext(it: &mut HuIText, x: i32, y: i32, startchar: i32) {
    it.lm = 0;
    it.laston = true;
    hulib_init_text_line(&mut it.l, x, y, startchar);
}
pub fn hulib_del_char_from_itext(it: &mut HuIText) {
    if it.l.l.len() as i32 != it.lm {
        hulib_del_char_from_text_line(&mut it.l);
    }
}
pub fn hulib_reset_itext(it: &mut HuIText) {
    it.lm = 0;
    hulib_clear_text_line(&mut it.l);
}
pub fn hulib_key_in_itext(it: &mut HuIText, ch: u8) -> bool {
    let ch = ch.to_ascii_uppercase();
    if ch as i32 >= ' ' as i32 && ch as i32 <= '_' as i32 {
        hulib_add_char_to_text_line(&mut it.l, ch);
    } else if ch as i32 == KEY_BACKSPACE {
        hulib_del_char_from_itext(it);
    } else if ch as i32 != KEY_ENTER {
        return false;
    }
    true
}
pub fn hulib_draw_itext(state: &mut GameState, it: &HuIText, on: bool) {
    if !on {
        return;
    }
    hulib_draw_text_line(state, &it.l, true);
}
pub fn hulib_erase_itext(state: &mut GameState, it: &mut HuIText, on: bool) {
    if it.laston && !on {
        it.l.needsupdate = 4;
    }
    hulib_erase_text_line(state, &mut it.l);
    it.laston = on;
}
