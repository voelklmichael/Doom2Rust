use crate::doomdef::true_0;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::m_controls::KEY_BACKSPACE;
use crate::m_controls::KEY_ENTER;
use crate::r_draw::R_VideoErase;
use crate::v_video::V_CachePatchNum;
use crate::v_video::V_DrawPatchDirect;
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct patch_t {
    pub width: i16,
    pub height: i16,
    pub leftoffset: i16,
    pub topoffset: i16,
    pub columnofs: [i32; 8],
}
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
pub unsafe fn HUlib_clearTextLine(mut t: *mut hu_textline_t) {
    (*t).l.clear();
    (*t).needsupdate = true_0;
}
pub unsafe fn HUlib_initTextLine(mut t: *mut hu_textline_t, mut x: i32, mut y: i32, mut sc: i32) {
    (*t).x = x;
    (*t).y = y;
    (*t).sc = sc;
    HUlib_clearTextLine(t);
}
pub unsafe fn HUlib_addCharToTextLine(t: *mut hu_textline_t, ch: u8) -> bool {
    if (*t).l.len() as i32 == HU_MAXLINELENGTH {
        false
    } else {
        (*t).l.push(ch as char);
        (*t).needsupdate = 4_i32;
        true
    }
}
pub unsafe fn HUlib_delCharFromTextLine(mut t: *mut hu_textline_t) -> bool {
    if (*t).l.is_empty() {
        false
    } else {
        (*t).l.pop();
        (*t).needsupdate = 4_i32;
        true
    }
}
pub unsafe fn HUlib_drawTextLine(
    state: &mut GameState,
    mut l: *mut hu_textline_t,
    mut drawcursor: bool,
) {
    let mut i: i32 = 0;
    let mut w: i32 = 0;
    let mut x: i32 = 0;
    let mut c: u8 = 0;
    x = (*l).x;
    i = 0_i32;
    while i < (*l).l.len() as i32 {
        c = (*l).l.as_bytes()[i as usize].to_ascii_uppercase();
        if c as i32 != ' ' as i32 && c as i32 >= (*l).sc && c as i32 <= '_' as i32 {
            let glyph = state.hu_stuff.hu_font[(c as i32 - (*l).sc) as usize];
            let patch = V_CachePatchNum(state, glyph);
            w = (*patch).width as i32;
            if x + w > SCREENWIDTH {
                break;
            }
            V_DrawPatchDirect(state, x, (*l).y, patch);
            x += w;
        } else {
            x += 4_i32;
            if x >= SCREENWIDTH {
                break;
            }
        }
        i += 1;
    }
    if drawcursor {
        let cursor_glyph = state.hu_stuff.hu_font[('_' as i32 - (*l).sc) as usize];
        let cursor_patch = V_CachePatchNum(state, cursor_glyph);
        if x + (*cursor_patch).width as i32 <= SCREENWIDTH {
            V_DrawPatchDirect(state, x, (*l).y, cursor_patch);
        }
    }
}
pub unsafe fn HUlib_eraseTextLine(state: &mut GameState, mut l: *mut hu_textline_t) {
    let mut lh: i32 = 0;
    let mut y: i32 = 0;
    let mut yoffset: i32 = 0;
    if !state.am_map.automapactive && state.r_draw.viewwindowx != 0 && (*l).needsupdate != 0 {
        let glyph = state.hu_stuff.hu_font[0];
        let patch = V_CachePatchNum(state, glyph);
        lh = (*patch).height as i32 + 1_i32;
        y = (*l).y;
        yoffset = y * SCREENWIDTH;
        while y < (*l).y + lh {
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
    if (*l).needsupdate != 0 {
        (*l).needsupdate -= 1;
    }
}
pub unsafe fn HUlib_initSText(
    state: &mut GameState,
    mut s: *mut hu_stext_t,
    mut x: i32,
    mut y: i32,
    mut h: i32,
    mut startchar: i32,
) {
    let mut i: i32 = 0;
    (*s).h = h;
    (*s).laston = true;
    (*s).cl = 0_i32;
    let glyph = state.hu_stuff.hu_font[0];
    let font0_patch = V_CachePatchNum(state, glyph);
    let font0_height = (*font0_patch).height as i32;
    i = 0_i32;
    while i < h {
        HUlib_initTextLine(
            (&raw mut (*s).l as *mut hu_textline_t).offset(i as isize) as *mut hu_textline_t,
            x,
            y - i * (font0_height + 1_i32),
            startchar,
        );
        i += 1;
    }
}
pub unsafe fn HUlib_addLineToSText(mut s: *mut hu_stext_t) {
    let mut i: i32 = 0;
    (*s).cl += 1;
    if (*s).cl == (*s).h {
        (*s).cl = 0_i32;
    }
    HUlib_clearTextLine(
        (&raw mut (*s).l as *mut hu_textline_t).offset((*s).cl as isize) as *mut hu_textline_t,
    );
    i = 0_i32;
    while i < (*s).h {
        (*s).l[i as usize].needsupdate = 4_i32;
        i += 1;
    }
}
pub unsafe fn HUlib_addMessageToSText(mut s: *mut hu_stext_t, prefix: Option<&str>, msg: &str) {
    HUlib_addLineToSText(s);
    if let Some(prefix) = prefix {
        for b in prefix.bytes() {
            HUlib_addCharToTextLine(
                (&raw mut (*s).l as *mut hu_textline_t).offset((*s).cl as isize)
                    as *mut hu_textline_t,
                b,
            );
        }
    }
    for b in msg.bytes() {
        HUlib_addCharToTextLine(
            (&raw mut (*s).l as *mut hu_textline_t).offset((*s).cl as isize) as *mut hu_textline_t,
            b,
        );
    }
}
pub unsafe fn HUlib_drawSText(state: &mut GameState, mut s: *mut hu_stext_t, on: bool) {
    let mut i: i32 = 0;
    let mut idx: i32 = 0;
    let mut l: *mut hu_textline_t = ::core::ptr::null_mut::<hu_textline_t>();
    if !on {
        return;
    }
    i = 0_i32;
    while i < (*s).h {
        idx = (*s).cl - i;
        if idx < 0_i32 {
            idx += (*s).h;
        }
        l = (&raw mut (*s).l as *mut hu_textline_t).offset(idx as isize) as *mut hu_textline_t;
        HUlib_drawTextLine(state, l, false);
        i += 1;
    }
}
pub unsafe fn HUlib_eraseSText(state: &mut GameState, mut s: *mut hu_stext_t, on: bool) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < (*s).h {
        if (*s).laston && !on {
            (*s).l[i as usize].needsupdate = 4_i32;
        }
        HUlib_eraseTextLine(
            state,
            (&raw mut (*s).l as *mut hu_textline_t).offset(i as isize) as *mut hu_textline_t,
        );
        i += 1;
    }
    (*s).laston = on;
}
pub unsafe fn HUlib_initIText(mut it: *mut hu_itext_t, mut x: i32, mut y: i32, mut startchar: i32) {
    (*it).lm = 0_i32;
    (*it).laston = true;
    HUlib_initTextLine(&raw mut (*it).l, x, y, startchar);
}
pub unsafe fn HUlib_delCharFromIText(mut it: *mut hu_itext_t) {
    if (*it).l.l.len() as i32 != (*it).lm {
        HUlib_delCharFromTextLine(&raw mut (*it).l);
    }
}
pub unsafe fn HUlib_eraseLineFromIText(mut it: *mut hu_itext_t) {
    while (*it).lm != (*it).l.l.len() as i32 {
        HUlib_delCharFromTextLine(&raw mut (*it).l);
    }
}
pub unsafe fn HUlib_resetIText(mut it: *mut hu_itext_t) {
    (*it).lm = 0_i32;
    HUlib_clearTextLine(&raw mut (*it).l);
}
pub unsafe fn HUlib_addPrefixToIText(it: *mut hu_itext_t, s: &str) {
    for b in s.bytes() {
        HUlib_addCharToTextLine(&raw mut (*it).l, b);
    }
    (*it).lm = (*it).l.l.len() as i32;
}
pub unsafe fn HUlib_keyInIText(mut it: *mut hu_itext_t, mut ch: u8) -> bool {
    ch = ch.to_ascii_uppercase();
    if ch as i32 >= ' ' as i32 && ch as i32 <= '_' as i32 {
        HUlib_addCharToTextLine(&raw mut (*it).l, ch);
    } else if ch as i32 == KEY_BACKSPACE {
        HUlib_delCharFromIText(it);
    } else if ch as i32 != KEY_ENTER {
        return false;
    }
    true
}
pub unsafe fn HUlib_drawIText(state: &mut GameState, mut it: *mut hu_itext_t, on: bool) {
    let mut l: *mut hu_textline_t = &raw mut (*it).l;
    if !on {
        return;
    }
    HUlib_drawTextLine(state, l, true);
}
pub unsafe fn HUlib_eraseIText(state: &mut GameState, mut it: *mut hu_itext_t, on: bool) {
    if (*it).laston && !on {
        (*it).l.needsupdate = 4_i32;
    }
    HUlib_eraseTextLine(state, &raw mut (*it).l);
    (*it).laston = on;
}
