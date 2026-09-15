use crate::d_player::PowerType;
use crate::d_player::NUMPSPRITES;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::hu_lib::patch_t;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedDiv;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::mem_compat::memset;
use crate::p_mobj::sector_t;
use crate::p_mobj::{mobj_t, pspdef_t};
use crate::p_mobj::{MF_SHADOW, MF_TRANSLATION, MF_TRANSSHIFT};
use crate::r_data::column_t;
use crate::r_defs::SpriteRotate;
use crate::r_defs::{drawseg_t, spritedef_t, spriteframe_t};
use crate::r_main::ColormapId;
use crate::r_main::LightRow48;
use crate::r_main::R_PointOnSegSide;
use crate::r_main::R_PointToAngle;
use crate::r_main::LIGHTLEVELS;
use crate::r_main::LIGHTSCALESHIFT;
use crate::r_main::LIGHTSEGSHIFT;
use crate::r_main::MAXLIGHTSCALE;
use crate::r_segs::R_RenderMaskedSegRange;
use crate::r_segs::SIL_BOTTOM;
use crate::r_segs::SIL_TOP;
use crate::stdint_types::byte;
use crate::stdint_types::size_t;
use crate::tables::angle_t;
use crate::tables::ANG45;
use crate::w_wad::W_CacheLumpNum;
use crate::w_wad::W_GetNumForName;

pub struct RThingsState {
    pub pspritescale: fixed_t,
    pub pspriteiscale: fixed_t,
    pub spritelights: LightRow48,
    pub negonearray: [i16; 320],
    pub screenheightarray: [i16; 320],
    pub sprites: Vec<spritedef_t>,
    pub numsprites: i32,
    pub sprtemp: [spriteframe_t; 29],
    pub maxframe: i32,
    pub spritename: &'static str,
    pub vissprites: [vissprite_t; 128],
    pub vissprite_p: *mut vissprite_t,
    pub overflowsprite: vissprite_t,
    pub mfloorclip: *mut i16,
    pub mceilingclip: *mut i16,
    pub spryscale: fixed_t,
    pub sprtopscreen: fixed_t,
    pub vissprite_order: Vec<usize>,
    pub clipbot: [i16; 320],
    pub cliptop: [i16; 320],
}

impl RThingsState {
    pub const fn new() -> Self {
        RThingsState {
            pspritescale: 0,
            pspriteiscale: 0,
            spritelights: LightRow48::Normal(0),
            negonearray: [0; 320],
            screenheightarray: [0; 320],
            sprites: Vec::new(),
            numsprites: 0,
            sprtemp: [spriteframe_t {
                rotate: SpriteRotate::Unset,
                lump: [0; 8],
                flip: [0; 8],
            }; 29],
            maxframe: 0,
            spritename: "",
            vissprites: [vissprite_s {
                x1: 0,
                x2: 0,
                gx: 0,
                gy: 0,
                gz: 0,
                gzt: 0,
                startfrac: 0,
                scale: 0,
                xiscale: 0,
                texturemid: 0,
                patch: 0,
                colormap: None,
                mobjflags: 0,
            }; 128],
            vissprite_p: ::core::ptr::null::<vissprite_t>() as *mut vissprite_t,
            overflowsprite: vissprite_s {
                x1: 0,
                x2: 0,
                gx: 0,
                gy: 0,
                gz: 0,
                gzt: 0,
                startfrac: 0,
                scale: 0,
                xiscale: 0,
                texturemid: 0,
                patch: 0,
                colormap: None,
                mobjflags: 0,
            },
            mfloorclip: ::core::ptr::null::<i16>() as *mut i16,
            mceilingclip: ::core::ptr::null::<i16>() as *mut i16,
            spryscale: 0,
            sprtopscreen: 0,
            vissprite_order: Vec::new(),
            clipbot: [0; 320],
            cliptop: [0; 320],
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct vissprite_s {
    pub x1: i32,
    pub x2: i32,
    pub gx: fixed_t,
    pub gy: fixed_t,
    pub gz: fixed_t,
    pub gzt: fixed_t,
    pub startfrac: fixed_t,
    pub scale: fixed_t,
    pub xiscale: fixed_t,
    pub texturemid: fixed_t,
    pub patch: i32,
    pub colormap: Option<ColormapId>,
    pub mobjflags: i32,
}
pub type vissprite_t = vissprite_s;
pub const FF_FULLBRIGHT: i32 = 0x8000;
pub const FF_FRAMEMASK: i32 = 0x7fff;
pub const MAXVISSPRITES: i32 = 128;
pub const MINZ: i32 = FRACUNIT * 4 as i32;
pub const BASEYCENTER: i32 = 100;
pub fn R_InstallSpriteLump(
    state: &mut GameState,
    mut lump: i32,
    mut frame: u32,
    mut rotation: u32,
    mut flipped: bool,
) {
    let mut r: i32 = 0;
    if frame >= 29 as u32 || rotation > 8 as u32 {
        I_Error(&format!(
            "R_InstallSpriteLump: Bad frame characters in lump {}",
            lump
        ));
    }
    if frame as i32 > state.r_things.maxframe {
        state.r_things.maxframe = frame as i32;
    }
    if rotation == 0 as u32 {
        if state.r_things.sprtemp[frame as usize].rotate == SpriteRotate::NonRotating {
            I_Error(&format!(
                "R_InitSprites: Sprite {} frame {} has multip rot=0 lump",
                state.r_things.spritename,
                ('A' as i32 as u32).wrapping_add(frame) as u8 as char,
            ));
        }
        if state.r_things.sprtemp[frame as usize].rotate == SpriteRotate::Rotating {
            I_Error(&format!(
                "R_InitSprites: Sprite {} frame {} has rotations and a rot=0 lump",
                state.r_things.spritename,
                ('A' as i32 as u32).wrapping_add(frame) as u8 as char,
            ));
        }
        state.r_things.sprtemp[frame as usize].rotate = SpriteRotate::NonRotating;
        r = 0 as i32;
        while r < 8 as i32 {
            state.r_things.sprtemp[frame as usize].lump[r as usize] =
                (lump - state.r_data.firstspritelump) as i16;
            state.r_things.sprtemp[frame as usize].flip[r as usize] = flipped as byte;
            r += 1;
        }
        return;
    }
    if state.r_things.sprtemp[frame as usize].rotate == SpriteRotate::NonRotating {
        I_Error(&format!(
            "R_InitSprites: Sprite {} frame {} has rotations and a rot=0 lump",
            state.r_things.spritename,
            ('A' as i32 as u32).wrapping_add(frame) as u8 as char,
        ));
    }
    state.r_things.sprtemp[frame as usize].rotate = SpriteRotate::Rotating;
    rotation = rotation.wrapping_sub(1);
    if state.r_things.sprtemp[frame as usize].lump[rotation as usize] as i32 != -(1 as i32) {
        I_Error(&format!(
            "R_InitSprites: Sprite {} : {} : {} has two lumps mapped to it",
            state.r_things.spritename,
            ('A' as i32 as u32).wrapping_add(frame) as u8 as char,
            ('1' as i32 as u32).wrapping_add(rotation) as u8 as char,
        ));
    }
    state.r_things.sprtemp[frame as usize].lump[rotation as usize] =
        (lump - state.r_data.firstspritelump) as i16;
    state.r_things.sprtemp[frame as usize].flip[rotation as usize] = flipped as byte;
}
pub unsafe fn R_InitSpriteDefs(state: &mut GameState, namelist: &[&'static str]) {
    let mut i: i32 = 0;
    let mut l: i32 = 0;
    let mut frame: i32 = 0;
    let mut rotation: i32 = 0;
    let mut start: i32 = 0;
    let mut end: i32 = 0;
    let mut patched: i32 = 0;
    state.r_things.numsprites = namelist.len() as i32;
    if state.r_things.numsprites == 0 {
        return;
    }
    state.r_things.sprites = Vec::with_capacity(state.r_things.numsprites as usize);
    start = state.r_data.firstspritelump - 1 as i32;
    end = state.r_data.lastspritelump + 1 as i32;
    i = 0 as i32;
    while i < state.r_things.numsprites {
        state.r_things.spritename = namelist[i as usize];
        memset(
            &raw mut state.r_things.sprtemp as *mut spriteframe_t as *mut ::core::ffi::c_void,
            -(1 as i32),
            ::core::mem::size_of::<[spriteframe_t; 29]>() as size_t,
        );
        state.r_things.maxframe = -(1 as i32);
        l = start + 1 as i32;
        while l < end {
            if state.w_wad.lumpinfo[l as usize]
                .name
                .eq_bytes_ignore_ascii_case_n(state.r_things.spritename.as_bytes(), 4)
            {
                frame = state.w_wad.lumpinfo[l as usize].name[4] as i32 - 'A' as i32;
                rotation = state.w_wad.lumpinfo[l as usize].name[5] as i32 - '0' as i32;
                if state.doomstat.modifiedgame {
                    let sprite_name = state.w_wad.lumpinfo[l as usize].name;
                    patched = W_GetNumForName(&mut state.w_wad, &sprite_name.as_str());
                } else {
                    patched = l;
                }
                R_InstallSpriteLump(state, patched, frame as u32, rotation as u32, false);
                if state.w_wad.lumpinfo[l as usize].name[6] != 0 {
                    frame = state.w_wad.lumpinfo[l as usize].name[6] as i32 - 'A' as i32;
                    rotation = state.w_wad.lumpinfo[l as usize].name[7] as i32 - '0' as i32;
                    R_InstallSpriteLump(state, l, frame as u32, rotation as u32, true);
                }
            }
            l += 1;
        }
        if state.r_things.maxframe == -(1 as i32) {
            state.r_things.sprites.push(spritedef_t {
                numframes: 0,
                spriteframes: Vec::new(),
            });
        } else {
            state.r_things.maxframe += 1;
            frame = 0 as i32;
            while frame < state.r_things.maxframe {
                match state.r_things.sprtemp[frame as usize].rotate as i32 {
                    -1 => {
                        I_Error(&format!(
                            "R_InitSprites: No patches found for {} frame {}",
                            state.r_things.spritename,
                            (frame + 'A' as i32) as u8 as char,
                        ));
                    }
                    1 => {
                        rotation = 0 as i32;
                        while rotation < 8 as i32 {
                            if state.r_things.sprtemp[frame as usize].lump[rotation as usize] as i32
                                == -(1 as i32)
                            {
                                I_Error(&format!(
                                    "R_InitSprites: Sprite {} frame {} is missing rotations",
                                    state.r_things.spritename,
                                    (frame + 'A' as i32) as u8 as char,
                                ));
                            }
                            rotation += 1;
                        }
                    }
                    0 | _ => {}
                }
                frame += 1;
            }
            state.r_things.sprites.push(spritedef_t {
                numframes: state.r_things.maxframe,
                spriteframes: state.r_things.sprtemp[..state.r_things.maxframe as usize].to_vec(),
            });
        }
        i += 1;
    }
}
pub static newvissprite: i32 = 0;
pub fn R_InitSprites(state: &mut GameState, namelist: &[&'static str]) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < SCREENWIDTH {
        state.r_things.negonearray[i as usize] = -(1 as i32) as i16;
        i += 1;
    }
    unsafe { R_InitSpriteDefs(state, namelist) };
}
pub unsafe fn R_ClearSprites(state: &mut GameState) {
    state.r_things.vissprite_p = &raw mut state.r_things.vissprites as *mut vissprite_t;
}
pub unsafe fn R_NewVisSprite(state: &mut GameState) -> *mut vissprite_t {
    if state.r_things.vissprite_p
        == (&raw mut state.r_things.vissprites as *mut vissprite_t).offset(MAXVISSPRITES as isize)
            as *mut vissprite_t
    {
        return &raw mut state.r_things.overflowsprite;
    }
    state.r_things.vissprite_p = state.r_things.vissprite_p.offset(1);
    return state.r_things.vissprite_p.offset(-(1 as i32 as isize));
}
pub unsafe fn R_DrawMaskedColumn(state: &mut GameState, mut column: *mut column_t) {
    let mut topscreen: i32 = 0;
    let mut bottomscreen: i32 = 0;
    let mut basetexturemid: fixed_t = 0;
    basetexturemid = state.r_draw.dc_texturemid;
    while (*column).topdelta as i32 != 0xff as i32 {
        topscreen = state.r_things.sprtopscreen as i32
            + state.r_things.spryscale as i32 * (*column).topdelta as i32;
        bottomscreen = topscreen + state.r_things.spryscale as i32 * (*column).length as i32;
        state.r_draw.dc_yl = topscreen + FRACUNIT - 1 as i32 >> FRACBITS;
        state.r_draw.dc_yh = bottomscreen - 1 as i32 >> FRACBITS;
        if state.r_draw.dc_yh
            >= *state.r_things.mfloorclip.offset(state.r_draw.dc_x as isize) as i32
        {
            state.r_draw.dc_yh =
                *state.r_things.mfloorclip.offset(state.r_draw.dc_x as isize) as i32 - 1 as i32;
        }
        if state.r_draw.dc_yl
            <= *state
                .r_things
                .mceilingclip
                .offset(state.r_draw.dc_x as isize) as i32
        {
            state.r_draw.dc_yl = *state
                .r_things
                .mceilingclip
                .offset(state.r_draw.dc_x as isize) as i32
                + 1 as i32;
        }
        if state.r_draw.dc_yl <= state.r_draw.dc_yh {
            state.r_draw.dc_source = (column as *mut byte).offset(3 as i32 as isize);
            state.r_draw.dc_texturemid =
                (basetexturemid as i32 - (((*column).topdelta as i32) << FRACBITS)) as fixed_t;
            state.r_main.colfunc.expect("non-null function pointer")(state);
        }
        column = (column as *mut byte)
            .offset((*column).length as i32 as isize)
            .offset(4 as i32 as isize) as *mut column_t;
    }
    state.r_draw.dc_texturemid = basetexturemid;
}
pub unsafe fn R_DrawVisSprite(state: &mut GameState, mut vis: *mut vissprite_t) {
    let mut column: *mut column_t = ::core::ptr::null_mut::<column_t>();
    let mut texturecolumn: i32 = 0;
    let mut frac: fixed_t = 0;
    let mut patch: *mut patch_t = ::core::ptr::null_mut::<patch_t>();
    patch = W_CacheLumpNum(
        state,
        (*vis).patch + state.r_data.firstspritelump) as *mut patch_t;
    state.r_draw.dc_colormap = (*vis).colormap;
    if state.r_draw.dc_colormap.is_none() {
        state.r_main.colfunc = state.r_main.fuzzcolfunc;
    } else if (*vis).mobjflags & MF_TRANSLATION as i32 != 0 {
        state.r_main.colfunc = state.r_main.transcolfunc;
        state.r_draw.dc_translation = state
            .r_draw
            .translationtables
            .as_mut_ptr()
            .offset(-(256 as i32 as isize))
            .offset(
                (((*vis).mobjflags & MF_TRANSLATION as i32) >> MF_TRANSSHIFT as i32 - 8 as i32)
                    as isize,
            );
    }
    state.r_draw.dc_iscale = (((*vis).xiscale as i32).abs() >> state.r_main.detailshift) as fixed_t;
    state.r_draw.dc_texturemid = (*vis).texturemid;
    frac = (*vis).startfrac;
    state.r_things.spryscale = (*vis).scale;
    state.r_things.sprtopscreen =
        state.r_main.centeryfrac - FixedMul(state.r_draw.dc_texturemid, state.r_things.spryscale);
    state.r_draw.dc_x = (*vis).x1;
    while state.r_draw.dc_x <= (*vis).x2 {
        texturecolumn = (frac >> FRACBITS) as i32;
        if texturecolumn < 0 as i32 || texturecolumn >= (*patch).width as i32 {
            I_Error("R_DrawSpriteRange: bad texturecolumn");
        }
        column = (patch as *mut byte).offset(
            *(&raw const (*patch).columnofs as *const i32).offset(texturecolumn as isize) as isize,
        ) as *mut column_t;
        R_DrawMaskedColumn(state, column);
        state.r_draw.dc_x += 1;
        frac += (*vis).xiscale;
    }
    state.r_main.colfunc = state.r_main.basecolfunc;
}
pub unsafe fn R_ProjectSprite(state: &mut GameState, mut thing: *mut mobj_t) {
    let mut tr_x: fixed_t = 0;
    let mut tr_y: fixed_t = 0;
    let mut gxt: fixed_t = 0;
    let mut gyt: fixed_t = 0;
    let mut tx: fixed_t = 0;
    let mut tz: fixed_t = 0;
    let mut xscale: fixed_t = 0;
    let mut x1: i32 = 0;
    let mut x2: i32 = 0;
    let mut sprdef: *mut spritedef_t = ::core::ptr::null_mut::<spritedef_t>();
    let mut sprframe: *mut spriteframe_t = ::core::ptr::null_mut::<spriteframe_t>();
    let mut lump: i32 = 0;
    let mut rot: u32 = 0;
    let mut flip: bool = false;
    let mut index: i32 = 0;
    let mut vis: *mut vissprite_t = ::core::ptr::null_mut::<vissprite_t>();
    let mut ang: angle_t = 0;
    let mut iscale: fixed_t = 0;
    tr_x = (*thing).x - state.r_main.viewx;
    tr_y = (*thing).y - state.r_main.viewy;
    gxt = FixedMul(tr_x, state.r_main.viewcos);
    gyt = -FixedMul(tr_y, state.r_main.viewsin);
    tz = gxt - gyt;
    if tz < MINZ {
        return;
    }
    xscale = FixedDiv(state.r_main.projection, tz);
    gxt = -FixedMul(tr_x, state.r_main.viewsin);
    gyt = FixedMul(tr_y, state.r_main.viewcos);
    tx = -(gyt + gxt);
    if (tx as i32).abs() > tz << 2 as i32 {
        return;
    }
    if (*thing).sprite as u32 >= state.r_things.numsprites as u32 {
        I_Error(&format!(
            "R_ProjectSprite: invalid sprite number {} ",
            (*thing).sprite as u32,
        ));
    }
    sprdef = &raw mut state.r_things.sprites[(*thing).sprite as usize] as *mut spritedef_t;
    if (*thing).frame & FF_FRAMEMASK >= (*sprdef).numframes {
        I_Error(&format!(
            "R_ProjectSprite: invalid sprite frame {} : {} ",
            (*thing).sprite as u32,
            (*thing).frame,
        ));
    }
    sprframe = &raw mut (*sprdef).spriteframes[((*thing).frame & FF_FRAMEMASK) as usize]
        as *mut spriteframe_t;
    if (*sprframe).rotate != SpriteRotate::NonRotating {
        ang = R_PointToAngle(state, (*thing).x, (*thing).y);
        rot = (ang as u32)
            .wrapping_sub((*thing).angle as u32)
            .wrapping_add(((ANG45 / 2 as i32) as u32).wrapping_mul(9 as u32))
            >> 29 as i32;
        lump = (*sprframe).lump[rot as usize] as i32;
        flip = (*sprframe).flip[rot as usize] != 0;
    } else {
        lump = (*sprframe).lump[0] as i32;
        flip = (*sprframe).flip[0] != 0;
    }
    tx -= state.r_data.spriteoffset[lump as usize];
    x1 = (state.r_main.centerxfrac + FixedMul(tx, xscale) >> FRACBITS) as i32;
    if x1 > state.r_draw.viewwidth {
        return;
    }
    tx += state.r_data.spritewidth[lump as usize];
    x2 = (state.r_main.centerxfrac as i32 + FixedMul(tx, xscale) as i32 >> FRACBITS) - 1 as i32;
    if x2 < 0 as i32 {
        return;
    }
    vis = R_NewVisSprite(state);
    (*vis).mobjflags = (*thing).flags;
    (*vis).scale = xscale << state.r_main.detailshift;
    (*vis).gx = (*thing).x;
    (*vis).gy = (*thing).y;
    (*vis).gz = (*thing).z;
    (*vis).gzt = (*thing).z + state.r_data.spritetopoffset[lump as usize];
    (*vis).texturemid = (*vis).gzt - state.r_main.viewz;
    (*vis).x1 = if x1 < 0 as i32 { 0 as i32 } else { x1 };
    (*vis).x2 = if x2 >= state.r_draw.viewwidth {
        state.r_draw.viewwidth - 1 as i32
    } else {
        x2
    };
    iscale = FixedDiv(FRACUNIT, xscale);
    if flip {
        (*vis).startfrac = (state.r_data.spritewidth[lump as usize] as i32 - 1 as i32) as fixed_t;
        (*vis).xiscale = -iscale;
    } else {
        (*vis).startfrac = 0 as i32 as fixed_t;
        (*vis).xiscale = iscale;
    }
    if (*vis).x1 > x1 {
        (*vis).startfrac += (*vis).xiscale as i32 * ((*vis).x1 - x1);
    }
    (*vis).patch = lump;
    if (*thing).flags & MF_SHADOW as i32 != 0 {
        (*vis).colormap = None;
    } else if let Some(colormap) = state.r_main.fixedcolormap {
        (*vis).colormap = Some(colormap);
    } else if (*thing).frame & FF_FULLBRIGHT != 0 {
        (*vis).colormap = Some(0);
    } else {
        index = (xscale >> LIGHTSCALESHIFT - state.r_main.detailshift) as i32;
        if index >= MAXLIGHTSCALE {
            index = MAXLIGHTSCALE - 1 as i32;
        }
        (*vis).colormap =
            Some(state.r_main.light_row48(state.r_things.spritelights)[index as usize]);
    };
}
pub unsafe fn R_AddSprites(state: &mut GameState, mut sec: *mut sector_t) {
    let mut thing: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut lightnum: i32 = 0;
    if (*sec).validcount == state.r_main.validcount {
        return;
    }
    (*sec).validcount = state.r_main.validcount;
    lightnum = ((*sec).lightlevel as i32 >> LIGHTSEGSHIFT) + state.r_main.extralight;
    if lightnum < 0 as i32 {
        state.r_things.spritelights = LightRow48::Normal(0);
    } else if lightnum >= LIGHTLEVELS {
        state.r_things.spritelights = LightRow48::Normal((LIGHTLEVELS - 1 as i32) as usize);
    } else {
        state.r_things.spritelights = LightRow48::Normal(lightnum as usize);
    }
    let mut cursor = (*sec).thinglist;
    while let Some(id) = cursor {
        thing = state
            .p_mobj
            .mobj_get(id)
            .expect("sector thinglist entry is always live");
        R_ProjectSprite(state, thing);
        cursor = (*thing).snext;
    }
}
pub unsafe fn R_DrawPSprite(state: &mut GameState, mut psp: *mut pspdef_t) {
    let mut tx: fixed_t = 0;
    let mut x1: i32 = 0;
    let mut x2: i32 = 0;
    let mut sprdef: *mut spritedef_t = ::core::ptr::null_mut::<spritedef_t>();
    let mut sprframe: *mut spriteframe_t = ::core::ptr::null_mut::<spriteframe_t>();
    let mut lump: i32 = 0;
    let mut flip: bool = false;
    let mut vis: *mut vissprite_t = ::core::ptr::null_mut::<vissprite_t>();
    let mut avis: vissprite_t = vissprite_s {
        x1: 0,
        x2: 0,
        gx: 0,
        gy: 0,
        gz: 0,
        gzt: 0,
        startfrac: 0,
        scale: 0,
        xiscale: 0,
        texturemid: 0,
        patch: 0,
        colormap: None,
        mobjflags: 0,
    };
    let psp_state = state.info.state_mut((*psp).state.unwrap());
    if (*psp_state).sprite as u32 >= state.r_things.numsprites as u32 {
        I_Error(&format!(
            "R_ProjectSprite: invalid sprite number {} ",
            (*psp_state).sprite as u32,
        ));
    }
    sprdef = &raw mut state.r_things.sprites[(*psp_state).sprite as usize] as *mut spritedef_t;
    if (*psp_state).frame & FF_FRAMEMASK >= (*sprdef).numframes {
        I_Error(&format!(
            "R_ProjectSprite: invalid sprite frame {} : {} ",
            (*psp_state).sprite as u32,
            (*psp_state).frame,
        ));
    }
    sprframe = &raw mut (*sprdef).spriteframes[((*psp_state).frame & FF_FRAMEMASK) as usize]
        as *mut spriteframe_t;
    lump = (*sprframe).lump[0] as i32;
    flip = (*sprframe).flip[0] != 0;
    tx = ((*psp).sx as i32 - 160 as i32 * FRACUNIT) as fixed_t;
    tx -= state.r_data.spriteoffset[lump as usize];
    x1 = (state.r_main.centerxfrac + FixedMul(tx, state.r_things.pspritescale) >> FRACBITS) as i32;
    if x1 > state.r_draw.viewwidth {
        return;
    }
    tx += state.r_data.spritewidth[lump as usize];
    x2 = (state.r_main.centerxfrac as i32 + FixedMul(tx, state.r_things.pspritescale) as i32
        >> FRACBITS)
        - 1 as i32;
    if x2 < 0 as i32 {
        return;
    }
    vis = &raw mut avis;
    (*vis).mobjflags = 0 as i32;
    (*vis).texturemid = (BASEYCENTER << FRACBITS) + FRACUNIT / 2 as fixed_t
        - ((*psp).sy - state.r_data.spritetopoffset[lump as usize]);
    (*vis).x1 = if x1 < 0 as i32 { 0 as i32 } else { x1 };
    (*vis).x2 = if x2 >= state.r_draw.viewwidth {
        state.r_draw.viewwidth - 1 as i32
    } else {
        x2
    };
    (*vis).scale = state.r_things.pspritescale << state.r_main.detailshift;
    if flip {
        (*vis).xiscale = -state.r_things.pspriteiscale;
        (*vis).startfrac = (state.r_data.spritewidth[lump as usize] as i32 - 1 as i32) as fixed_t;
    } else {
        (*vis).xiscale = state.r_things.pspriteiscale;
        (*vis).startfrac = 0 as i32 as fixed_t;
    }
    if (*vis).x1 > x1 {
        (*vis).startfrac += (*vis).xiscale as i32 * ((*vis).x1 - x1);
    }
    (*vis).patch = lump;
    let viewplayer = state.g_game.player_mut(state.r_main.viewplayer);
    if (*viewplayer).powers[PowerType::pw_invisibility as usize] > 4 as i32 * 32 as i32
        || (*viewplayer).powers[PowerType::pw_invisibility as usize] & 8 as i32 != 0
    {
        (*vis).colormap = None;
    } else if let Some(colormap) = state.r_main.fixedcolormap {
        (*vis).colormap = Some(colormap);
    } else if (*psp_state).frame & FF_FULLBRIGHT != 0 {
        (*vis).colormap = Some(0);
    } else {
        (*vis).colormap = Some(
            state.r_main.light_row48(state.r_things.spritelights)
                [(MAXLIGHTSCALE - 1 as i32) as usize],
        );
    }
    R_DrawVisSprite(state, vis);
}
pub unsafe fn R_DrawPlayerSprites(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut lightnum: i32 = 0;
    let mut psp: *mut pspdef_t = ::core::ptr::null_mut::<pspdef_t>();
    let viewplayer = state.g_game.player_mut(state.r_main.viewplayer);
    let viewplayer_mo = state.p_mobj.mobj_get((*viewplayer).mo.unwrap()).unwrap();
    lightnum = ((*state
        .p_setup
        .sector_mut(state.p_setup.subsectors[(*viewplayer_mo).subsector.0 as usize].sector))
    .lightlevel as i32
        >> LIGHTSEGSHIFT)
        + state.r_main.extralight;
    if lightnum < 0 as i32 {
        state.r_things.spritelights = LightRow48::Normal(0);
    } else if lightnum >= LIGHTLEVELS {
        state.r_things.spritelights = LightRow48::Normal((LIGHTLEVELS - 1 as i32) as usize);
    } else {
        state.r_things.spritelights = LightRow48::Normal(lightnum as usize);
    }
    state.r_things.mfloorclip = &raw mut state.r_things.screenheightarray as *mut i16;
    state.r_things.mceilingclip = &raw mut state.r_things.negonearray as *mut i16;
    i = 0 as i32;
    psp = &raw mut (*viewplayer).psprites as *mut pspdef_t;
    while i < NUMPSPRITES as i32 {
        if (*psp).state.is_some() {
            R_DrawPSprite(state, psp);
        }
        i += 1;
        psp = psp.offset(1);
    }
}
pub unsafe fn R_SortVisSprites(state: &mut GameState) {
    let count = state
        .r_things
        .vissprite_p
        .offset_from(&raw mut state.r_things.vissprites as *mut vissprite_t) as i64
        as i32;
    let mut order = core::mem::take(&mut state.r_things.vissprite_order);
    order.clear();
    if count > 0 {
        order.extend(0..count as usize);
        // Stable sort: preserves the original selection-sort's leftmost-first
        // tie-break among vissprites sharing the same scale.
        order.sort_by_key(|&i| state.r_things.vissprites[i].scale);
    }
    state.r_things.vissprite_order = order;
}
pub unsafe fn R_DrawSprite(state: &mut GameState, mut spr: *mut vissprite_t) {
    let mut ds: *mut drawseg_t = ::core::ptr::null_mut::<drawseg_t>();
    let mut x: i32 = 0;
    let mut r1: i32 = 0;
    let mut r2: i32 = 0;
    let mut scale: fixed_t = 0;
    let mut lowscale: fixed_t = 0;
    let mut silhouette: i32 = 0;
    x = (*spr).x1;
    while x <= (*spr).x2 {
        state.r_things.cliptop[x as usize] = -(2 as i32) as i16;
        state.r_things.clipbot[x as usize] = state.r_things.cliptop[x as usize];
        x += 1;
    }
    ds = state.r_bsp.ds_p.offset(-(1 as i32 as isize));
    while ds >= &raw mut state.r_bsp.drawsegs as *mut drawseg_t {
        if !((*ds).x1 > (*spr).x2
            || (*ds).x2 < (*spr).x1
            || (*ds).silhouette == 0 && (*ds).maskedtexturecol.is_null())
        {
            r1 = if (*ds).x1 < (*spr).x1 {
                (*spr).x1
            } else {
                (*ds).x1
            };
            r2 = if (*ds).x2 > (*spr).x2 {
                (*spr).x2
            } else {
                (*ds).x2
            };
            if (*ds).scale1 > (*ds).scale2 {
                lowscale = (*ds).scale2;
                scale = (*ds).scale1;
            } else {
                lowscale = (*ds).scale1;
                scale = (*ds).scale2;
            }
            if scale < (*spr).scale
                || lowscale < (*spr).scale
                    && R_PointOnSegSide(state, (*spr).gx, (*spr).gy, (*ds).curline) == 0
            {
                if !(*ds).maskedtexturecol.is_null() {
                    R_RenderMaskedSegRange(state, ds, r1, r2);
                }
            } else {
                silhouette = (*ds).silhouette;
                if (*spr).gz >= (*ds).bsilheight {
                    silhouette &= !SIL_BOTTOM;
                }
                if (*spr).gzt <= (*ds).tsilheight {
                    silhouette &= !SIL_TOP;
                }
                if silhouette == 1 as i32 {
                    x = r1;
                    while x <= r2 {
                        if state.r_things.clipbot[x as usize] as i32 == -(2 as i32) {
                            state.r_things.clipbot[x as usize] =
                                *(*ds).sprbottomclip.offset(x as isize);
                        }
                        x += 1;
                    }
                } else if silhouette == 2 as i32 {
                    x = r1;
                    while x <= r2 {
                        if state.r_things.cliptop[x as usize] as i32 == -(2 as i32) {
                            state.r_things.cliptop[x as usize] =
                                *(*ds).sprtopclip.offset(x as isize);
                        }
                        x += 1;
                    }
                } else if silhouette == 3 as i32 {
                    x = r1;
                    while x <= r2 {
                        if state.r_things.clipbot[x as usize] as i32 == -(2 as i32) {
                            state.r_things.clipbot[x as usize] =
                                *(*ds).sprbottomclip.offset(x as isize);
                        }
                        if state.r_things.cliptop[x as usize] as i32 == -(2 as i32) {
                            state.r_things.cliptop[x as usize] =
                                *(*ds).sprtopclip.offset(x as isize);
                        }
                        x += 1;
                    }
                }
            }
        }
        ds = ds.offset(-1);
    }
    x = (*spr).x1;
    while x <= (*spr).x2 {
        if state.r_things.clipbot[x as usize] as i32 == -(2 as i32) {
            state.r_things.clipbot[x as usize] = state.r_draw.viewheight as i16;
        }
        if state.r_things.cliptop[x as usize] as i32 == -(2 as i32) {
            state.r_things.cliptop[x as usize] = -(1 as i32) as i16;
        }
        x += 1;
    }
    state.r_things.mfloorclip = &raw mut state.r_things.clipbot as *mut i16;
    state.r_things.mceilingclip = &raw mut state.r_things.cliptop as *mut i16;
    R_DrawVisSprite(state, spr);
}
pub unsafe fn R_DrawMasked(state: &mut GameState) {
    let mut ds: *mut drawseg_t = ::core::ptr::null_mut::<drawseg_t>();
    R_SortVisSprites(state);
    let mut i = 0;
    while i < state.r_things.vissprite_order.len() {
        let idx = state.r_things.vissprite_order[i];
        let spr = &raw mut state.r_things.vissprites[idx] as *mut vissprite_t;
        R_DrawSprite(state, spr);
        i += 1;
    }
    ds = state.r_bsp.ds_p.offset(-(1 as i32 as isize));
    while ds >= &raw mut state.r_bsp.drawsegs as *mut drawseg_t {
        if !(*ds).maskedtexturecol.is_null() {
            R_RenderMaskedSegRange(state, ds, (*ds).x1, (*ds).x2);
        }
        ds = ds.offset(-1);
    }
    if state.r_main.viewangleoffset == 0 {
        R_DrawPlayerSprites(state);
    }
}
