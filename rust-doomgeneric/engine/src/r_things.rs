use crate::d_player::PowerType;
use crate::d_player::NUMPSPRITES;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedDiv;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::p_mobj::pspdef_t;
use crate::p_mobj::MobjId;
use crate::p_mobj::{MF_SHADOW, MF_TRANSLATION, MF_TRANSSHIFT};
use crate::p_setup::SectorId;
use crate::patch::Patch;
use crate::r_defs::ClipArray;
use crate::r_defs::SpriteRotate;
use crate::r_defs::{spritedef_t, spriteframe_t};
use crate::r_draw::{advance_source, read_source, ColumnSource};
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
use crate::v_video::V_CachePatchNum;
use alloc::vec::Vec;

use crate::tables::angle_t;
use crate::tables::ANG45;

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
    pub vissprite_p: usize,
    pub overflowsprite: vissprite_t,
    pub mfloorclip: Option<ClipArray>,
    pub mceilingclip: Option<ClipArray>,
    pub spryscale: fixed_t,
    pub sprtopscreen: fixed_t,
    pub vissprite_order: Vec<usize>,
    pub clipbot: [i16; 320],
    pub cliptop: [i16; 320],
}

impl Default for RThingsState {
    fn default() -> Self {
        Self::new()
    }
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
            vissprite_p: 0,
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
            mfloorclip: None,
            mceilingclip: None,
            spryscale: 0,
            sprtopscreen: 0,
            vissprite_order: Vec::new(),
            clipbot: [0; 320],
            cliptop: [0; 320],
        }
    }
}

#[derive(Copy, Clone)]
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
pub const MINZ: i32 = FRACUNIT * 4_i32;
pub const BASEYCENTER: i32 = 100;
pub fn R_InstallSpriteLump(
    state: &mut GameState,
    lump: i32,
    frame: u32,
    mut rotation: u32,
    flipped: bool,
) {
    let mut r: i32;
    if frame >= 29_u32 || rotation > 8_u32 {
        I_Error(&format!(
            "R_InstallSpriteLump: Bad frame characters in lump {}",
            lump
        ));
    }
    if frame as i32 > state.r_things.maxframe {
        state.r_things.maxframe = frame as i32;
    }
    if rotation == 0_u32 {
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
        r = 0_i32;
        while r < 8_i32 {
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
    if state.r_things.sprtemp[frame as usize].lump[rotation as usize] as i32 != -1_i32 {
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
pub fn R_InitSpriteDefs(state: &mut GameState, namelist: &[&'static str]) {
    let mut i: i32;
    let mut l: i32;
    let mut frame: i32;
    let mut rotation: i32;

    let mut patched: i32;
    state.r_things.numsprites = namelist.len() as i32;
    if state.r_things.numsprites == 0 {
        return;
    }
    state.r_things.sprites = Vec::with_capacity(state.r_things.numsprites as usize);
    let start: i32 = state.r_data.firstspritelump - 1_i32;
    let end: i32 = state.r_data.lastspritelump + 1_i32;
    i = 0_i32;
    while i < state.r_things.numsprites {
        state.r_things.spritename = namelist[i as usize];
        state.r_things.sprtemp = [spriteframe_t {
            rotate: SpriteRotate::Unset,
            lump: [-1; 8],
            flip: [0xff; 8],
        }; 29];
        state.r_things.maxframe = -1_i32;
        l = start + 1_i32;
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
        if state.r_things.maxframe == -1_i32 {
            state.r_things.sprites.push(spritedef_t {
                numframes: 0,
                spriteframes: Vec::new(),
            });
        } else {
            state.r_things.maxframe += 1;
            frame = 0_i32;
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
                        rotation = 0_i32;
                        while rotation < 8_i32 {
                            if state.r_things.sprtemp[frame as usize].lump[rotation as usize] as i32
                                == -1_i32
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
                    _ => {}
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
pub fn R_InitSprites(state: &mut GameState, namelist: &[&'static str]) {
    let mut i: i32;
    i = 0_i32;
    while i < SCREENWIDTH {
        state.r_things.negonearray[i as usize] = -1_i32 as i16;
        i += 1;
    }
    R_InitSpriteDefs(state, namelist);
}
pub fn R_ClearSprites(state: &mut GameState) {
    state.r_things.vissprite_p = 0;
}
pub fn R_StoreVisSprite(state: &mut GameState, vis: vissprite_t) {
    if state.r_things.vissprite_p == MAXVISSPRITES as usize {
        state.r_things.overflowsprite = vis;
        return;
    }
    state.r_things.vissprites[state.r_things.vissprite_p] = vis;
    state.r_things.vissprite_p += 1;
}
pub fn R_DrawMaskedColumn(state: &mut GameState, mut post: ColumnSource) {
    let mut topscreen: i32;
    let mut bottomscreen: i32;

    let basetexturemid: fixed_t = state.r_draw.dc_texturemid;
    let mfloorclip = state.r_things.mfloorclip.unwrap();
    let mceilingclip = state.r_things.mceilingclip.unwrap();
    loop {
        let topdelta = read_source(state, post, 0);
        if topdelta as i32 == 0xff_i32 {
            break;
        }
        let length = read_source(state, post, 1);
        topscreen = state.r_things.sprtopscreen + state.r_things.spryscale * topdelta as i32;
        bottomscreen = topscreen + state.r_things.spryscale * length as i32;
        state.r_draw.dc_yl = (topscreen + FRACUNIT - 1_i32) >> FRACBITS;
        state.r_draw.dc_yh = (bottomscreen - 1_i32) >> FRACBITS;
        let floorclip = mfloorclip.get(state, state.r_draw.dc_x as isize) as i32;
        let ceilingclip = mceilingclip.get(state, state.r_draw.dc_x as isize) as i32;
        if state.r_draw.dc_yh >= floorclip {
            state.r_draw.dc_yh = floorclip - 1_i32;
        }
        if state.r_draw.dc_yl <= ceilingclip {
            state.r_draw.dc_yl = ceilingclip + 1_i32;
        }
        if state.r_draw.dc_yl <= state.r_draw.dc_yh {
            state.r_draw.dc_source = Some(advance_source(post, 3));
            state.r_draw.dc_texturemid =
                (basetexturemid - ((topdelta as i32) << FRACBITS)) as fixed_t;
            state.r_main.colfunc.expect("non-null function pointer")(state);
        }
        post = advance_source(post, length as usize + 4);
    }
    state.r_draw.dc_texturemid = basetexturemid;
}
pub fn R_DrawVisSprite(state: &mut GameState, vis: &vissprite_t) {
    let mut texturecolumn: i32;
    let mut frac: fixed_t;

    let sprite_lump = vis.patch + state.r_data.firstspritelump;
    let patch: Patch = V_CachePatchNum(state, sprite_lump);
    state.r_draw.dc_colormap = vis.colormap;
    if state.r_draw.dc_colormap.is_none() {
        state.r_main.colfunc = state.r_main.fuzzcolfunc;
    } else if vis.mobjflags & MF_TRANSLATION != 0 {
        state.r_main.colfunc = state.r_main.transcolfunc;
        state.r_draw.dc_translation =
            ((vis.mobjflags & MF_TRANSLATION) >> (MF_TRANSSHIFT - 8_i32)) as usize - 256;
    }
    state.r_draw.dc_iscale = (vis.xiscale.abs() >> state.r_main.detailshift) as fixed_t;
    state.r_draw.dc_texturemid = vis.texturemid;
    frac = vis.startfrac;
    state.r_things.spryscale = vis.scale;
    state.r_things.sprtopscreen =
        state.r_main.centeryfrac - FixedMul(state.r_draw.dc_texturemid, state.r_things.spryscale);
    state.r_draw.dc_x = vis.x1;
    while state.r_draw.dc_x <= vis.x2 {
        texturecolumn = frac >> FRACBITS;
        if texturecolumn < 0_i32 || texturecolumn >= patch.width() {
            I_Error("R_DrawSpriteRange: bad texturecolumn");
        }
        let column_offset = patch.columnofs(texturecolumn);
        R_DrawMaskedColumn(
            state,
            ColumnSource::Lump {
                lump: sprite_lump,
                offset: column_offset,
            },
        );
        state.r_draw.dc_x += 1;
        frac += vis.xiscale;
    }
    state.r_main.colfunc = state.r_main.basecolfunc;
}
pub fn R_ProjectSprite(state: &mut GameState, thing_id: MobjId) {
    let thing = state.p_mobj.mo(thing_id);
    let (thing_x, thing_y, thing_z, thing_sprite, thing_frame, thing_angle, thing_flags) = (
        thing.x,
        thing.y,
        thing.z,
        thing.sprite,
        thing.frame,
        thing.angle,
        thing.flags,
    );

    let mut gxt: fixed_t;
    let mut gyt: fixed_t;
    let mut tx: fixed_t;

    let lump: i32;
    let rot: u32;
    let flip: bool;
    let mut index: i32;
    let ang: angle_t;

    let tr_x: fixed_t = thing_x - state.r_main.viewx;
    let tr_y: fixed_t = thing_y - state.r_main.viewy;
    gxt = FixedMul(tr_x, state.r_main.viewcos);
    gyt = -FixedMul(tr_y, state.r_main.viewsin);
    let tz: fixed_t = gxt - gyt;
    if tz < MINZ {
        return;
    }
    let xscale: fixed_t = FixedDiv(state.r_main.projection, tz);
    gxt = -FixedMul(tr_x, state.r_main.viewsin);
    gyt = FixedMul(tr_y, state.r_main.viewcos);
    tx = -(gyt + gxt);
    if tx.abs() > tz << 2_i32 {
        return;
    }
    if thing_sprite as u32 >= state.r_things.numsprites as u32 {
        I_Error(&format!(
            "R_ProjectSprite: invalid sprite number {} ",
            thing_sprite as u32,
        ));
    }
    let sprdef = &state.r_things.sprites[thing_sprite as usize];
    if thing_frame & FF_FRAMEMASK >= sprdef.numframes {
        I_Error(&format!(
            "R_ProjectSprite: invalid sprite frame {} : {} ",
            thing_sprite as u32, thing_frame,
        ));
    }
    let sprframe = sprdef.spriteframes[(thing_frame & FF_FRAMEMASK) as usize];
    if sprframe.rotate != SpriteRotate::NonRotating {
        ang = R_PointToAngle(state, thing_x, thing_y);
        rot = ang
            .wrapping_sub(thing_angle)
            .wrapping_add(((ANG45 / 2_i32) as u32).wrapping_mul(9_u32))
            >> 29_i32;
        lump = sprframe.lump[rot as usize] as i32;
        flip = sprframe.flip[rot as usize] != 0;
    } else {
        lump = sprframe.lump[0] as i32;
        flip = sprframe.flip[0] != 0;
    }
    tx -= state.r_data.spriteoffset[lump as usize];
    let x1: i32 = (state.r_main.centerxfrac + FixedMul(tx, xscale)) >> FRACBITS;
    if x1 > state.r_draw.viewwidth {
        return;
    }
    tx += state.r_data.spritewidth[lump as usize];
    let x2: i32 = ((state.r_main.centerxfrac + FixedMul(tx, xscale)) >> FRACBITS) - 1_i32;
    if x2 < 0_i32 {
        return;
    }
    let mut vis = vissprite_s {
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
    vis.mobjflags = thing_flags;
    vis.scale = xscale << state.r_main.detailshift;
    vis.gx = thing_x;
    vis.gy = thing_y;
    vis.gz = thing_z;
    vis.gzt = thing_z + state.r_data.spritetopoffset[lump as usize];
    vis.texturemid = vis.gzt - state.r_main.viewz;
    vis.x1 = if x1 < 0_i32 { 0_i32 } else { x1 };
    vis.x2 = if x2 >= state.r_draw.viewwidth {
        state.r_draw.viewwidth - 1_i32
    } else {
        x2
    };
    let iscale: fixed_t = FixedDiv(FRACUNIT, xscale);
    if flip {
        vis.startfrac = (state.r_data.spritewidth[lump as usize] - 1_i32) as fixed_t;
        vis.xiscale = -iscale;
    } else {
        vis.startfrac = 0_i32 as fixed_t;
        vis.xiscale = iscale;
    }
    if vis.x1 > x1 {
        vis.startfrac += vis.xiscale * (vis.x1 - x1);
    }
    vis.patch = lump;
    if thing_flags & MF_SHADOW != 0 {
        vis.colormap = None;
    } else if let Some(colormap) = state.r_main.fixedcolormap {
        vis.colormap = Some(colormap);
    } else if thing_frame & FF_FULLBRIGHT != 0 {
        vis.colormap = Some(0);
    } else {
        index = xscale >> (LIGHTSCALESHIFT - state.r_main.detailshift);
        if index >= MAXLIGHTSCALE {
            index = MAXLIGHTSCALE - 1_i32;
        }
        vis.colormap = Some(state.r_main.light_row48(state.r_things.spritelights)[index as usize]);
    };
    R_StoreVisSprite(state, vis);
}
pub fn R_AddSprites(state: &mut GameState, sec: SectorId) {
    let sector = state.p_setup.sector_mut(sec);
    if sector.validcount == state.r_main.validcount {
        return;
    }
    sector.validcount = state.r_main.validcount;
    let (sector_lightlevel, thinglist) = (sector.lightlevel as i32, sector.thinglist);
    let lightnum = (sector_lightlevel >> LIGHTSEGSHIFT) + state.r_main.extralight;
    if lightnum < 0_i32 {
        state.r_things.spritelights = LightRow48::Normal(0);
    } else if lightnum >= LIGHTLEVELS {
        state.r_things.spritelights = LightRow48::Normal((LIGHTLEVELS - 1_i32) as usize);
    } else {
        state.r_things.spritelights = LightRow48::Normal(lightnum as usize);
    }
    let mut cursor = thinglist;
    while let Some(id) = cursor {
        let snext = state.p_mobj.mo(id).snext;
        R_ProjectSprite(state, id);
        cursor = snext;
    }
}
pub fn R_DrawPSprite(state: &mut GameState, psp: &pspdef_t) {
    let mut tx: fixed_t;

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
    let psp_state = state.info.state_mut(psp.state.unwrap());
    let (psp_state_sprite, psp_state_frame) = (psp_state.sprite, psp_state.frame);
    if psp_state_sprite as u32 >= state.r_things.numsprites as u32 {
        I_Error(&format!(
            "R_ProjectSprite: invalid sprite number {} ",
            psp_state_sprite as u32,
        ));
    }
    let sprdef = &state.r_things.sprites[psp_state_sprite as usize];
    if psp_state_frame & FF_FRAMEMASK >= sprdef.numframes {
        I_Error(&format!(
            "R_ProjectSprite: invalid sprite frame {} : {} ",
            psp_state_sprite as u32, psp_state_frame,
        ));
    }
    let sprframe = &sprdef.spriteframes[(psp_state_frame & FF_FRAMEMASK) as usize];
    let lump: i32 = sprframe.lump[0] as i32;
    let flip: bool = sprframe.flip[0] != 0;
    tx = (psp.sx - 160_i32 * FRACUNIT) as fixed_t;
    tx -= state.r_data.spriteoffset[lump as usize];
    let x1: i32 =
        (state.r_main.centerxfrac + FixedMul(tx, state.r_things.pspritescale)) >> FRACBITS;
    if x1 > state.r_draw.viewwidth {
        return;
    }
    tx += state.r_data.spritewidth[lump as usize];
    let x2: i32 = ((state.r_main.centerxfrac + FixedMul(tx, state.r_things.pspritescale))
        >> FRACBITS)
        - 1_i32;
    if x2 < 0_i32 {
        return;
    }
    avis.mobjflags = 0_i32;
    avis.texturemid = (BASEYCENTER << FRACBITS) + FRACUNIT / 2 as fixed_t
        - (psp.sy - state.r_data.spritetopoffset[lump as usize]);
    avis.x1 = if x1 < 0_i32 { 0_i32 } else { x1 };
    avis.x2 = if x2 >= state.r_draw.viewwidth {
        state.r_draw.viewwidth - 1_i32
    } else {
        x2
    };
    avis.scale = state.r_things.pspritescale << state.r_main.detailshift;
    if flip {
        avis.xiscale = -state.r_things.pspriteiscale;
        avis.startfrac = (state.r_data.spritewidth[lump as usize] - 1_i32) as fixed_t;
    } else {
        avis.xiscale = state.r_things.pspriteiscale;
        avis.startfrac = 0_i32 as fixed_t;
    }
    if avis.x1 > x1 {
        avis.startfrac += avis.xiscale * (avis.x1 - x1);
    }
    avis.patch = lump;
    let invisibility = state.g_game.player_mut(state.r_main.viewplayer).powers
        [PowerType::pw_invisibility as usize];
    if invisibility > 4_i32 * 32_i32 || invisibility & 8_i32 != 0 {
        avis.colormap = None;
    } else if let Some(colormap) = state.r_main.fixedcolormap {
        avis.colormap = Some(colormap);
    } else if psp_state_frame & FF_FULLBRIGHT != 0 {
        avis.colormap = Some(0);
    } else {
        avis.colormap = Some(
            state.r_main.light_row48(state.r_things.spritelights)[(MAXLIGHTSCALE - 1_i32) as usize],
        );
    }
    R_DrawVisSprite(state, &avis);
}
pub fn R_DrawPlayerSprites(state: &mut GameState) {
    let viewplayer = state.g_game.player_mut(state.r_main.viewplayer);
    let psprites = viewplayer.psprites;
    let viewplayer_mo_id = viewplayer.mo.unwrap();
    let viewplayer_subsector = state.p_mobj.mo(viewplayer_mo_id).subsector;
    let lightnum = (state
        .p_setup
        .sector_mut(state.p_setup.subsectors[viewplayer_subsector.0 as usize].sector)
        .lightlevel as i32
        >> LIGHTSEGSHIFT)
        + state.r_main.extralight;
    if lightnum < 0_i32 {
        state.r_things.spritelights = LightRow48::Normal(0);
    } else if lightnum >= LIGHTLEVELS {
        state.r_things.spritelights = LightRow48::Normal((LIGHTLEVELS - 1_i32) as usize);
    } else {
        state.r_things.spritelights = LightRow48::Normal(lightnum as usize);
    }
    state.r_things.mfloorclip = Some(ClipArray::ScreenHeightArray);
    state.r_things.mceilingclip = Some(ClipArray::NegOneArray);
    for psp in psprites.iter().take(NUMPSPRITES as usize) {
        if psp.state.is_some() {
            R_DrawPSprite(state, psp);
        }
    }
}
pub fn R_SortVisSprites(state: &mut GameState) {
    let count = state.r_things.vissprite_p as i32;
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
pub fn R_DrawSprite(state: &mut GameState, spr: &vissprite_t) {
    let mut x: i32;
    let mut r1: i32;
    let mut r2: i32;
    let mut scale: fixed_t;
    let mut lowscale: fixed_t;
    let mut silhouette: i32;
    x = spr.x1;
    while x <= spr.x2 {
        state.r_things.cliptop[x as usize] = -2_i32 as i16;
        state.r_things.clipbot[x as usize] = state.r_things.cliptop[x as usize];
        x += 1;
    }
    for ds_idx in (0..state.r_bsp.ds_p).rev() {
        let ds = state.r_bsp.drawsegs[ds_idx];
        if !(ds.x1 > spr.x2
            || ds.x2 < spr.x1
            || ds.silhouette == 0 && ds.maskedtexturecol.is_none())
        {
            r1 = if ds.x1 < spr.x1 { spr.x1 } else { ds.x1 };
            r2 = if ds.x2 > spr.x2 { spr.x2 } else { ds.x2 };
            if ds.scale1 > ds.scale2 {
                lowscale = ds.scale2;
                scale = ds.scale1;
            } else {
                lowscale = ds.scale1;
                scale = ds.scale2;
            }
            if scale < spr.scale
                || lowscale < spr.scale && R_PointOnSegSide(state, spr.gx, spr.gy, ds.curline) == 0
            {
                if ds.maskedtexturecol.is_some() {
                    R_RenderMaskedSegRange(state, &ds, r1, r2);
                }
            } else {
                silhouette = ds.silhouette;
                if spr.gz >= ds.bsilheight {
                    silhouette &= !SIL_BOTTOM;
                }
                if spr.gzt <= ds.tsilheight {
                    silhouette &= !SIL_TOP;
                }
                if silhouette == 1_i32 {
                    let sprbottomclip = ds.sprbottomclip.unwrap();
                    x = r1;
                    while x <= r2 {
                        if state.r_things.clipbot[x as usize] as i32 == -2_i32 {
                            state.r_things.clipbot[x as usize] =
                                sprbottomclip.get(state, x as isize);
                        }
                        x += 1;
                    }
                } else if silhouette == 2_i32 {
                    let sprtopclip = ds.sprtopclip.unwrap();
                    x = r1;
                    while x <= r2 {
                        if state.r_things.cliptop[x as usize] as i32 == -2_i32 {
                            state.r_things.cliptop[x as usize] = sprtopclip.get(state, x as isize);
                        }
                        x += 1;
                    }
                } else if silhouette == 3_i32 {
                    let sprbottomclip = ds.sprbottomclip.unwrap();
                    let sprtopclip = ds.sprtopclip.unwrap();
                    x = r1;
                    while x <= r2 {
                        if state.r_things.clipbot[x as usize] as i32 == -2_i32 {
                            state.r_things.clipbot[x as usize] =
                                sprbottomclip.get(state, x as isize);
                        }
                        if state.r_things.cliptop[x as usize] as i32 == -2_i32 {
                            state.r_things.cliptop[x as usize] = sprtopclip.get(state, x as isize);
                        }
                        x += 1;
                    }
                }
            }
        }
    }
    x = spr.x1;
    while x <= spr.x2 {
        if state.r_things.clipbot[x as usize] as i32 == -2_i32 {
            state.r_things.clipbot[x as usize] = state.r_draw.viewheight as i16;
        }
        if state.r_things.cliptop[x as usize] as i32 == -2_i32 {
            state.r_things.cliptop[x as usize] = -1_i32 as i16;
        }
        x += 1;
    }
    state.r_things.mfloorclip = Some(ClipArray::ClipBot);
    state.r_things.mceilingclip = Some(ClipArray::ClipTop);
    R_DrawVisSprite(state, spr);
}
pub fn R_DrawMasked(state: &mut GameState) {
    R_SortVisSprites(state);
    let mut i = 0;
    while i < state.r_things.vissprite_order.len() {
        let idx = state.r_things.vissprite_order[i];
        let spr = state.r_things.vissprites[idx];
        R_DrawSprite(state, &spr);
        i += 1;
    }
    for ds_idx in (0..state.r_bsp.ds_p).rev() {
        let ds = state.r_bsp.drawsegs[ds_idx];
        if ds.maskedtexturecol.is_some() {
            R_RenderMaskedSegRange(state, &ds, ds.x1, ds.x2);
        }
    }
    if state.r_main.viewangleoffset == 0 {
        R_DrawPlayerSprites(state);
    }
}
