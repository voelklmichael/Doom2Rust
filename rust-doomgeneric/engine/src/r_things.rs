use crate::d_player::PowerType;
use crate::d_player::NUMPSPRITES;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::m_fixed::fixed_div;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::p_mobj::MobjFlags;
use crate::p_mobj::MobjId;
use crate::p_mobj::PspDef;
use crate::p_setup::SectorId;
use crate::patch::Patch;
use crate::r_data::RDataState;
use crate::r_defs::ClipArray;
use crate::r_defs::SpriteRotate;
use crate::r_defs::{SpriteDef, SpriteFrame};
use crate::r_draw::{advance_source, read_source, ColumnSource};
use crate::r_main::point_on_seg_side;
use crate::r_main::point_to_angle;
use crate::r_main::ColormapId;
use crate::r_main::LightRow48;
use crate::r_main::LIGHTLEVELS;
use crate::r_main::LIGHTSCALESHIFT;
use crate::r_main::LIGHTSEGSHIFT;
use crate::r_main::MAXLIGHTSCALE;
use crate::r_segs::render_masked_seg_range;
use crate::r_segs::SIL_BOTTOM;
use crate::r_segs::SIL_TOP;
use crate::v_video::cache_patch_num;
use alloc::vec::Vec;

use crate::tables::Angle;
use crate::tables::ANG45;

use crate::w_wad::get_num_for_name;

pub struct RThingsState {
    pub pspritescale: Fixed,
    pub pspriteiscale: Fixed,
    pub spritelights: LightRow48,
    pub negonearray: [i16; 320],
    pub screenheightarray: [i16; 320],
    pub sprites: Vec<SpriteDef>,
    pub numsprites: i32,
    pub sprtemp: [SpriteFrame; 29],
    pub maxframe: i32,
    pub spritename: &'static str,
    pub vissprites: [VisSprite; 128],
    pub vissprite_p: usize,
    pub overflowsprite: VisSprite,
    pub mfloorclip: Option<ClipArray>,
    pub mceilingclip: Option<ClipArray>,
    pub spryscale: Fixed,
    pub sprtopscreen: Fixed,
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
        Self {
            pspritescale: Fixed::ZERO,
            pspriteiscale: Fixed::ZERO,
            spritelights: LightRow48::Normal(0),
            negonearray: [0; 320],
            screenheightarray: [0; 320],
            sprites: Vec::new(),
            numsprites: 0,
            sprtemp: [SpriteFrame {
                rotate: SpriteRotate::Unset,
                lump: [0; 8],
                flip: [0; 8],
            }; 29],
            maxframe: 0,
            spritename: "",
            vissprites: [VisSprite {
                x1: 0,
                x2: 0,
                gx: Fixed::ZERO,
                gy: Fixed::ZERO,
                gz: Fixed::ZERO,
                gzt: Fixed::ZERO,
                startfrac: Fixed::ZERO,
                scale: Fixed::ZERO,
                xiscale: Fixed::ZERO,
                texturemid: Fixed::ZERO,
                patch: 0,
                colormap: None,
                mobjflags: MobjFlags::empty(),
            }; 128],
            vissprite_p: 0,
            overflowsprite: VisSprite {
                x1: 0,
                x2: 0,
                gx: Fixed::ZERO,
                gy: Fixed::ZERO,
                gz: Fixed::ZERO,
                gzt: Fixed::ZERO,
                startfrac: Fixed::ZERO,
                scale: Fixed::ZERO,
                xiscale: Fixed::ZERO,
                texturemid: Fixed::ZERO,
                patch: 0,
                colormap: None,
                mobjflags: MobjFlags::empty(),
            },
            mfloorclip: None,
            mceilingclip: None,
            spryscale: Fixed::ZERO,
            sprtopscreen: Fixed::ZERO,
            vissprite_order: Vec::new(),
            clipbot: [0; 320],
            cliptop: [0; 320],
        }
    }
}

#[derive(Copy, Clone)]
pub struct VisSprite {
    pub x1: i32,
    pub x2: i32,
    pub gx: Fixed,
    pub gy: Fixed,
    pub gz: Fixed,
    pub gzt: Fixed,
    pub startfrac: Fixed,
    pub scale: Fixed,
    pub xiscale: Fixed,
    pub texturemid: Fixed,
    pub patch: i32,
    pub colormap: Option<ColormapId>,
    pub mobjflags: MobjFlags,
}
pub const FF_FULLBRIGHT: i32 = 0x8000;
pub const FF_FRAMEMASK: i32 = 0x7fff;
pub const MAXVISSPRITES: usize = 128;
pub const MINZ: Fixed = Fixed::from_int(4);
pub const BASEYCENTER: i32 = 100;
pub fn install_sprite_lump(
    r_data: &RDataState,
    r_things: &mut RThingsState,
    lump: i32,
    frame: u32,
    mut rotation: u32,
    flipped: bool,
) {
    if frame >= 29 || rotation > 8 {
        error(&format!(
            "R_InstallSpriteLump: Bad frame characters in lump {lump}"
        ));
    }
    if frame as i32 > r_things.maxframe {
        r_things.maxframe = frame as i32;
    }
    if rotation == 0 {
        if r_things.sprtemp[frame as usize].rotate == SpriteRotate::NonRotating {
            error(&format!(
                "R_InitSprites: Sprite {} frame {} has multip rot=0 lump",
                r_things.spritename,
                ('A' as i32 as u32).wrapping_add(frame) as u8 as char,
            ));
        }
        if r_things.sprtemp[frame as usize].rotate == SpriteRotate::Rotating {
            error(&format!(
                "R_InitSprites: Sprite {} frame {} has rotations and a rot=0 lump",
                r_things.spritename,
                ('A' as i32 as u32).wrapping_add(frame) as u8 as char,
            ));
        }
        r_things.sprtemp[frame as usize].rotate = SpriteRotate::NonRotating;
        for r in 0..8 {
            r_things.sprtemp[frame as usize].lump[r] = (lump - r_data.firstspritelump) as i16;
            r_things.sprtemp[frame as usize].flip[r] = u8::from(flipped);
        }
        return;
    }
    if r_things.sprtemp[frame as usize].rotate == SpriteRotate::NonRotating {
        error(&format!(
            "R_InitSprites: Sprite {} frame {} has rotations and a rot=0 lump",
            r_things.spritename,
            ('A' as i32 as u32).wrapping_add(frame) as u8 as char,
        ));
    }
    r_things.sprtemp[frame as usize].rotate = SpriteRotate::Rotating;
    rotation = rotation.wrapping_sub(1);
    if i32::from(r_things.sprtemp[frame as usize].lump[rotation as usize]) != -1 {
        error(&format!(
            "R_InitSprites: Sprite {} : {} : {} has two lumps mapped to it",
            r_things.spritename,
            ('A' as i32 as u32).wrapping_add(frame) as u8 as char,
            ('1' as i32 as u32).wrapping_add(rotation) as u8 as char,
        ));
    }
    r_things.sprtemp[frame as usize].lump[rotation as usize] =
        (lump - r_data.firstspritelump) as i16;
    r_things.sprtemp[frame as usize].flip[rotation as usize] = u8::from(flipped);
}
pub fn init_sprite_defs(state: &mut GameState, namelist: &[&'static str]) {
    state.render.r_things.numsprites = namelist.len() as i32;
    if state.render.r_things.numsprites == 0 {
        return;
    }
    state.render.r_things.sprites = Vec::with_capacity(state.render.r_things.numsprites as usize);
    let start: i32 = state.render.r_data.firstspritelump - 1;
    let end: i32 = state.render.r_data.lastspritelump + 1;
    for &name in namelist {
        state.render.r_things.spritename = name;
        state.render.r_things.sprtemp = [SpriteFrame {
            rotate: SpriteRotate::Unset,
            lump: [-1; 8],
            flip: [0xff; 8],
        }; 29];
        state.render.r_things.maxframe = -1;
        for l in start + 1..end {
            if state.assets.w_wad.lumpinfo[l as usize]
                .name
                .eq_bytes_ignore_ascii_case_n(state.render.r_things.spritename.as_bytes(), 4)
            {
                let frame = i32::from(state.assets.w_wad.lumpinfo[l as usize].name[4]) - 'A' as i32;
                let rotation =
                    i32::from(state.assets.w_wad.lumpinfo[l as usize].name[5]) - '0' as i32;
                let patched: i32 = if state.game.doomstat.modifiedgame {
                    let sprite_name = state.assets.w_wad.lumpinfo[l as usize].name;
                    get_num_for_name(&state.assets.w_wad, &sprite_name.as_str())
                } else {
                    l
                };
                install_sprite_lump(
                    &state.render.r_data,
                    &mut state.render.r_things,
                    patched,
                    frame as u32,
                    rotation as u32,
                    false,
                );
                if state.assets.w_wad.lumpinfo[l as usize].name[6] != 0 {
                    let frame =
                        i32::from(state.assets.w_wad.lumpinfo[l as usize].name[6]) - 'A' as i32;
                    let rotation =
                        i32::from(state.assets.w_wad.lumpinfo[l as usize].name[7]) - '0' as i32;
                    install_sprite_lump(
                        &state.render.r_data,
                        &mut state.render.r_things,
                        l,
                        frame as u32,
                        rotation as u32,
                        true,
                    );
                }
            }
        }
        if state.render.r_things.maxframe == -1 {
            state.render.r_things.sprites.push(SpriteDef {
                numframes: 0,
                spriteframes: Vec::new(),
            });
        } else {
            state.render.r_things.maxframe += 1;
            for frame in 0..state.render.r_things.maxframe {
                match state.render.r_things.sprtemp[frame as usize].rotate {
                    SpriteRotate::Unset => {
                        error(&format!(
                            "R_InitSprites: No patches found for {} frame {}",
                            state.render.r_things.spritename,
                            (frame + 'A' as i32) as u8 as char,
                        ));
                    }
                    SpriteRotate::Rotating => {
                        for rotation in 0..8 {
                            if i32::from(
                                state.render.r_things.sprtemp[frame as usize].lump[rotation],
                            ) == -1
                            {
                                error(&format!(
                                    "R_InitSprites: Sprite {} frame {} is missing rotations",
                                    state.render.r_things.spritename,
                                    (frame + 'A' as i32) as u8 as char,
                                ));
                            }
                        }
                    }
                    SpriteRotate::NonRotating => {}
                }
            }
            state.render.r_things.sprites.push(SpriteDef {
                numframes: state.render.r_things.maxframe,
                spriteframes: state.render.r_things.sprtemp
                    [..state.render.r_things.maxframe as usize]
                    .to_vec(),
            });
        }
    }
}
pub fn init_sprites(state: &mut GameState, namelist: &[&'static str]) {
    for i in 0..(SCREENWIDTH as usize) {
        state.render.r_things.negonearray[i] = -1_i16;
    }
    init_sprite_defs(state, namelist);
}
pub fn clear_sprites(r_things: &mut RThingsState) {
    r_things.vissprite_p = 0;
}
pub fn store_vis_sprite(r_things: &mut RThingsState, vis: VisSprite) {
    if r_things.vissprite_p == MAXVISSPRITES {
        r_things.overflowsprite = vis;
        return;
    }
    r_things.vissprites[r_things.vissprite_p] = vis;
    r_things.vissprite_p += 1;
}
pub fn draw_masked_column(state: &mut GameState, mut post: ColumnSource) {
    let basetexturemid: Fixed = state.render.r_draw.dc_texturemid;
    let mfloorclip = state.render.r_things.mfloorclip.unwrap();
    let mceilingclip = state.render.r_things.mceilingclip.unwrap();
    loop {
        let topdelta = read_source(&state.render.r_data, &state.assets.w_wad, post, 0);
        if i32::from(topdelta) == 0xff {
            break;
        }
        let length = read_source(&state.render.r_data, &state.assets.w_wad, post, 1);
        let topscreen: Fixed = state.render.r_things.sprtopscreen
            + state.render.r_things.spryscale * i32::from(topdelta);
        let bottomscreen: Fixed = topscreen + state.render.r_things.spryscale * i32::from(length);
        state.render.r_draw.dc_yl = (topscreen + FRACUNIT - Fixed(1)).to_int();
        state.render.r_draw.dc_yh = (bottomscreen - Fixed(1)).to_int();
        let floorclip = i32::from(mfloorclip.get(state, state.render.r_draw.dc_x as isize));
        let ceilingclip = i32::from(mceilingclip.get(state, state.render.r_draw.dc_x as isize));
        if state.render.r_draw.dc_yh >= floorclip {
            state.render.r_draw.dc_yh = floorclip - 1;
        }
        if state.render.r_draw.dc_yl <= ceilingclip {
            state.render.r_draw.dc_yl = ceilingclip + 1;
        }
        if state.render.r_draw.dc_yl <= state.render.r_draw.dc_yh {
            state.render.r_draw.dc_source = Some(advance_source(post, 3));
            state.render.r_draw.dc_texturemid =
                basetexturemid - Fixed::from_int(i32::from(topdelta));
            state
                .render
                .r_main
                .colfunc
                .expect("non-null function pointer")(state);
        }
        post = advance_source(post, length as usize + 4);
    }
    state.render.r_draw.dc_texturemid = basetexturemid;
}
pub fn draw_vis_sprite(state: &mut GameState, vis: &VisSprite) {
    let sprite_lump = vis.patch + state.render.r_data.firstspritelump;
    let patch: Patch = cache_patch_num(&*state.assets.fs, &mut state.assets.w_wad, sprite_lump);
    state.render.r_draw.dc_colormap = vis.colormap;
    if state.render.r_draw.dc_colormap.is_none() {
        state.render.r_main.colfunc = state.render.r_main.fuzzcolfunc;
    } else if vis.mobjflags.contains(MobjFlags::TRANSLATION) {
        state.render.r_main.colfunc = state.render.r_main.transcolfunc;
        state.render.r_draw.dc_translation = ((vis.mobjflags & MobjFlags::TRANSLATION).bits()
            >> (MobjFlags::TRANSLATION_SHIFT - 8))
            as usize
            - 256;
    }
    state.render.r_draw.dc_iscale = vis.xiscale.abs() >> state.render.r_main.detailshift;
    state.render.r_draw.dc_texturemid = vis.texturemid;
    let mut frac: Fixed = vis.startfrac;
    state.render.r_things.spryscale = vis.scale;
    state.render.r_things.sprtopscreen = state.render.r_main.centeryfrac
        - fixed_mul(
            state.render.r_draw.dc_texturemid,
            state.render.r_things.spryscale,
        );
    state.render.r_draw.dc_x = vis.x1;
    while state.render.r_draw.dc_x <= vis.x2 {
        let texturecolumn: i32 = frac.to_int();
        if texturecolumn < 0 || texturecolumn >= patch.width() {
            error("R_DrawSpriteRange: bad texturecolumn");
        }
        let column_offset = patch.columnofs(texturecolumn);
        draw_masked_column(
            state,
            ColumnSource::Lump {
                lump: sprite_lump,
                offset: column_offset,
            },
        );
        state.render.r_draw.dc_x += 1;
        frac += vis.xiscale;
    }
    state.render.r_main.colfunc = state.render.r_main.basecolfunc;
}
pub fn project_sprite(state: &mut GameState, thing_id: MobjId) {
    let thing = state.world.p_mobj.mo(thing_id);
    let (thing_x, thing_y, thing_z, thing_sprite, thing_frame, thing_angle, thing_flags) = (
        thing.x,
        thing.y,
        thing.z,
        thing.sprite,
        thing.frame,
        thing.angle,
        thing.flags,
    );

    let tr_x: Fixed = thing_x - state.render.r_main.viewx;
    let tr_y: Fixed = thing_y - state.render.r_main.viewy;
    let mut gxt: Fixed = fixed_mul(tr_x, state.render.r_main.viewcos);
    let mut gyt: Fixed = -fixed_mul(tr_y, state.render.r_main.viewsin);
    let tz: Fixed = gxt - gyt;
    if tz < MINZ {
        return;
    }
    let xscale: Fixed = fixed_div(state.render.r_main.projection, tz);
    gxt = -fixed_mul(tr_x, state.render.r_main.viewsin);
    gyt = fixed_mul(tr_y, state.render.r_main.viewcos);
    let mut tx: Fixed = -(gyt + gxt);
    if tx.abs() > tz << 2 {
        return;
    }
    if thing_sprite as u32 >= state.render.r_things.numsprites as u32 {
        error(&format!(
            "R_ProjectSprite: invalid sprite number {} ",
            thing_sprite as u32,
        ));
    }
    let sprdef = &state.render.r_things.sprites[thing_sprite as usize];
    if thing_frame & FF_FRAMEMASK >= sprdef.numframes {
        error(&format!(
            "R_ProjectSprite: invalid sprite frame {} : {} ",
            thing_sprite as u32, thing_frame,
        ));
    }
    let sprframe = sprdef.spriteframes[(thing_frame & FF_FRAMEMASK) as usize];
    let (lump, flip): (i32, bool) = if sprframe.rotate == SpriteRotate::NonRotating {
        (i32::from(sprframe.lump[0]), sprframe.flip[0] != 0)
    } else {
        let ang: Angle = point_to_angle(&state.render.r_main, thing_x, thing_y);
        let rot: usize = ((ang - thing_angle + ANG45 / 2 * 9).to_bits() >> 29) as usize;
        (i32::from(sprframe.lump[rot]), sprframe.flip[rot] != 0)
    };
    tx -= state.render.r_data.spriteoffset[lump as usize];
    let x1: i32 = (state.render.r_main.centerxfrac + fixed_mul(tx, xscale)).to_int();
    if x1 > state.render.r_draw.viewwidth {
        return;
    }
    tx += state.render.r_data.spritewidth[lump as usize];
    let x2: i32 = (((state.render.r_main.centerxfrac + fixed_mul(tx, xscale)) >> FRACBITS)
        - Fixed(1))
    .to_bits();
    if x2 < 0 {
        return;
    }
    let mut vis = VisSprite {
        x1: 0,
        x2: 0,
        gx: Fixed::ZERO,
        gy: Fixed::ZERO,
        gz: Fixed::ZERO,
        gzt: Fixed::ZERO,
        startfrac: Fixed::ZERO,
        scale: Fixed::ZERO,
        xiscale: Fixed::ZERO,
        texturemid: Fixed::ZERO,
        patch: 0,
        colormap: None,
        mobjflags: MobjFlags::empty(),
    };
    vis.mobjflags = thing_flags;
    vis.scale = xscale << state.render.r_main.detailshift;
    vis.gx = thing_x;
    vis.gy = thing_y;
    vis.gz = thing_z;
    vis.gzt = thing_z + state.render.r_data.spritetopoffset[lump as usize];
    vis.texturemid = vis.gzt - state.render.r_main.viewz;
    vis.x1 = if x1 < 0 { 0 } else { x1 };
    vis.x2 = if x2 >= state.render.r_draw.viewwidth {
        state.render.r_draw.viewwidth - 1
    } else {
        x2
    };
    let iscale: Fixed = fixed_div(FRACUNIT, xscale);
    if flip {
        vis.startfrac = state.render.r_data.spritewidth[lump as usize] - Fixed(1);
        vis.xiscale = -iscale;
    } else {
        vis.startfrac = Fixed::ZERO;
        vis.xiscale = iscale;
    }
    if vis.x1 > x1 {
        vis.startfrac += vis.xiscale * (vis.x1 - x1);
    }
    vis.patch = lump;
    if thing_flags.contains(MobjFlags::SHADOW) {
        vis.colormap = None;
    } else if let Some(colormap) = state.render.r_main.fixedcolormap {
        vis.colormap = Some(colormap);
    } else if thing_frame & FF_FULLBRIGHT != 0 {
        vis.colormap = Some(0);
    } else {
        let mut index =
            (xscale >> (LIGHTSCALESHIFT - state.render.r_main.detailshift)).to_bits() as usize;
        if index >= MAXLIGHTSCALE {
            index = MAXLIGHTSCALE - 1;
        }
        vis.colormap = Some(
            state
                .render
                .r_main
                .light_row48(state.render.r_things.spritelights)[index],
        );
    }
    store_vis_sprite(&mut state.render.r_things, vis);
}
pub fn add_sprites(state: &mut GameState, sec: SectorId) {
    let validcount = state.world.p_setup.validcount;
    let sector = state.world.p_setup.sector_mut(sec);
    if sector.validcount == validcount {
        return;
    }
    sector.validcount = validcount;
    let (sector_lightlevel, thinglist) = (i32::from(sector.lightlevel), sector.thinglist);
    let lightnum = (sector_lightlevel >> LIGHTSEGSHIFT) + state.render.r_main.extralight;
    if lightnum < 0 {
        state.render.r_things.spritelights = LightRow48::Normal(0);
    } else if lightnum >= LIGHTLEVELS {
        state.render.r_things.spritelights = LightRow48::Normal((LIGHTLEVELS - 1) as usize);
    } else {
        state.render.r_things.spritelights = LightRow48::Normal(lightnum as usize);
    }
    let mut cursor = thinglist;
    while let Some(id) = cursor {
        let snext = state.world.p_mobj.mo(id).snext;
        project_sprite(state, id);
        cursor = snext;
    }
}
pub fn draw_psprite(state: &mut GameState, psp: &PspDef) {
    let mut avis: VisSprite = VisSprite {
        x1: 0,
        x2: 0,
        gx: Fixed::ZERO,
        gy: Fixed::ZERO,
        gz: Fixed::ZERO,
        gzt: Fixed::ZERO,
        startfrac: Fixed::ZERO,
        scale: Fixed::ZERO,
        xiscale: Fixed::ZERO,
        texturemid: Fixed::ZERO,
        patch: 0,
        colormap: None,
        mobjflags: MobjFlags::empty(),
    };
    let psp_state = state.assets.info.state_mut(psp.state.unwrap());
    let (psp_state_sprite, psp_state_frame) = (psp_state.sprite, psp_state.frame);
    if psp_state_sprite as u32 >= state.render.r_things.numsprites as u32 {
        error(&format!(
            "R_ProjectSprite: invalid sprite number {} ",
            psp_state_sprite as u32,
        ));
    }
    let sprdef = &state.render.r_things.sprites[psp_state_sprite as usize];
    if psp_state_frame & FF_FRAMEMASK >= sprdef.numframes {
        error(&format!(
            "R_ProjectSprite: invalid sprite frame {} : {} ",
            psp_state_sprite as u32, psp_state_frame,
        ));
    }
    let sprframe = &sprdef.spriteframes[(psp_state_frame & FF_FRAMEMASK) as usize];
    let lump: i32 = i32::from(sprframe.lump[0]);
    let flip: bool = sprframe.flip[0] != 0;
    let mut tx: Fixed = psp.sx - 160 * FRACUNIT;
    tx -= state.render.r_data.spriteoffset[lump as usize];
    let x1: i32 = (state.render.r_main.centerxfrac
        + fixed_mul(tx, state.render.r_things.pspritescale))
    .to_int();
    if x1 > state.render.r_draw.viewwidth {
        return;
    }
    tx += state.render.r_data.spritewidth[lump as usize];
    let x2: i32 = (((state.render.r_main.centerxfrac
        + fixed_mul(tx, state.render.r_things.pspritescale))
        >> FRACBITS)
        - Fixed(1))
    .to_bits();
    if x2 < 0 {
        return;
    }
    avis.mobjflags = MobjFlags::empty();
    avis.texturemid = Fixed::from_int(BASEYCENTER) + FRACUNIT / 2
        - (psp.sy - state.render.r_data.spritetopoffset[lump as usize]);
    avis.x1 = if x1 < 0 { 0 } else { x1 };
    avis.x2 = if x2 >= state.render.r_draw.viewwidth {
        state.render.r_draw.viewwidth - 1
    } else {
        x2
    };
    avis.scale = state.render.r_things.pspritescale << state.render.r_main.detailshift;
    if flip {
        avis.xiscale = -state.render.r_things.pspriteiscale;
        avis.startfrac = state.render.r_data.spritewidth[lump as usize] - Fixed(1);
    } else {
        avis.xiscale = state.render.r_things.pspriteiscale;
        avis.startfrac = Fixed::ZERO;
    }
    if avis.x1 > x1 {
        avis.startfrac += avis.xiscale * (avis.x1 - x1);
    }
    avis.patch = lump;
    let invisibility = state
        .game
        .g_game
        .player_mut(state.render.r_main.viewplayer)
        .powers[PowerType::Invisibility];
    if invisibility > 4 * 32 || invisibility & 8 != 0 {
        avis.colormap = None;
    } else if let Some(colormap) = state.render.r_main.fixedcolormap {
        avis.colormap = Some(colormap);
    } else if psp_state_frame & FF_FULLBRIGHT != 0 {
        avis.colormap = Some(0);
    } else {
        avis.colormap = Some(
            state
                .render
                .r_main
                .light_row48(state.render.r_things.spritelights)[MAXLIGHTSCALE - 1],
        );
    }
    draw_vis_sprite(state, &avis);
}
pub fn draw_player_sprites(state: &mut GameState) {
    let viewplayer = state.game.g_game.player_mut(state.render.r_main.viewplayer);
    let psprites = viewplayer.psprites;
    let viewplayer_mo_id = viewplayer.mo.unwrap();
    let viewplayer_subsector = state.world.p_mobj.mo(viewplayer_mo_id).subsector;
    let lightnum = (i32::from(
        state
            .world
            .p_setup
            .sector_mut(state.world.p_setup.subsectors[viewplayer_subsector.0 as usize].sector)
            .lightlevel,
    ) >> LIGHTSEGSHIFT)
        + state.render.r_main.extralight;
    if lightnum < 0 {
        state.render.r_things.spritelights = LightRow48::Normal(0);
    } else if lightnum >= LIGHTLEVELS {
        state.render.r_things.spritelights = LightRow48::Normal((LIGHTLEVELS - 1) as usize);
    } else {
        state.render.r_things.spritelights = LightRow48::Normal(lightnum as usize);
    }
    state.render.r_things.mfloorclip = Some(ClipArray::ScreenHeightArray);
    state.render.r_things.mceilingclip = Some(ClipArray::NegOneArray);
    for psp in psprites.iter().take(NUMPSPRITES) {
        if psp.state.is_some() {
            draw_psprite(state, psp);
        }
    }
}
pub fn sort_vis_sprites(r_things: &mut RThingsState) {
    let count = r_things.vissprite_p as i32;
    let mut order = core::mem::take(&mut r_things.vissprite_order);
    order.clear();
    if count > 0 {
        order.extend(0..count as usize);
        // Stable sort: preserves the original selection-sort's leftmost-first
        // tie-break among vissprites sharing the same scale.
        order.sort_by_key(|&i| r_things.vissprites[i].scale);
    }
    r_things.vissprite_order = order;
}
pub fn draw_sprite(state: &mut GameState, spr: &VisSprite) {
    for x in spr.x1..=spr.x2 {
        state.render.r_things.cliptop[x as usize] = -2_i16;
        state.render.r_things.clipbot[x as usize] = state.render.r_things.cliptop[x as usize];
    }
    for ds_idx in (0..state.render.r_bsp.ds_p).rev() {
        let ds = state.render.r_bsp.drawsegs[ds_idx];
        if !(ds.x1 > spr.x2
            || ds.x2 < spr.x1
            || ds.silhouette == 0 && ds.maskedtexturecol.is_none())
        {
            let r1: i32 = if ds.x1 < spr.x1 { spr.x1 } else { ds.x1 };
            let r2: i32 = if ds.x2 > spr.x2 { spr.x2 } else { ds.x2 };
            let (scale, lowscale): (Fixed, Fixed) = if ds.scale1 > ds.scale2 {
                (ds.scale1, ds.scale2)
            } else {
                (ds.scale2, ds.scale1)
            };
            if scale < spr.scale
                || lowscale < spr.scale
                    && point_on_seg_side(&state.world.p_setup, spr.gx, spr.gy, ds.curline) == 0
            {
                if ds.maskedtexturecol.is_some() {
                    render_masked_seg_range(state, &ds, r1, r2);
                }
            } else {
                let mut silhouette: i32 = ds.silhouette;
                if spr.gz >= ds.bsilheight {
                    silhouette &= !SIL_BOTTOM;
                }
                if spr.gzt <= ds.tsilheight {
                    silhouette &= !SIL_TOP;
                }
                if silhouette == 1 {
                    let sprbottomclip = ds.sprbottomclip.unwrap();
                    for x in r1..=r2 {
                        if i32::from(state.render.r_things.clipbot[x as usize]) == -2 {
                            state.render.r_things.clipbot[x as usize] =
                                sprbottomclip.get(state, x as isize);
                        }
                    }
                } else if silhouette == 2 {
                    let sprtopclip = ds.sprtopclip.unwrap();
                    for x in r1..=r2 {
                        if i32::from(state.render.r_things.cliptop[x as usize]) == -2 {
                            state.render.r_things.cliptop[x as usize] =
                                sprtopclip.get(state, x as isize);
                        }
                    }
                } else if silhouette == 3 {
                    let sprbottomclip = ds.sprbottomclip.unwrap();
                    let sprtopclip = ds.sprtopclip.unwrap();
                    for x in r1..=r2 {
                        if i32::from(state.render.r_things.clipbot[x as usize]) == -2 {
                            state.render.r_things.clipbot[x as usize] =
                                sprbottomclip.get(state, x as isize);
                        }
                        if i32::from(state.render.r_things.cliptop[x as usize]) == -2 {
                            state.render.r_things.cliptop[x as usize] =
                                sprtopclip.get(state, x as isize);
                        }
                    }
                }
            }
        }
    }
    for x in spr.x1..=spr.x2 {
        if i32::from(state.render.r_things.clipbot[x as usize]) == -2 {
            state.render.r_things.clipbot[x as usize] = state.render.r_draw.viewheight as i16;
        }
        if i32::from(state.render.r_things.cliptop[x as usize]) == -2 {
            state.render.r_things.cliptop[x as usize] = -1_i16;
        }
    }
    state.render.r_things.mfloorclip = Some(ClipArray::ClipBot);
    state.render.r_things.mceilingclip = Some(ClipArray::ClipTop);
    draw_vis_sprite(state, spr);
}
pub fn draw_masked(state: &mut GameState) {
    sort_vis_sprites(&mut state.render.r_things);
    for i in 0..state.render.r_things.vissprite_order.len() {
        let idx = state.render.r_things.vissprite_order[i];
        let spr = state.render.r_things.vissprites[idx];
        draw_sprite(state, &spr);
    }
    for ds_idx in (0..state.render.r_bsp.ds_p).rev() {
        let ds = state.render.r_bsp.drawsegs[ds_idx];
        if ds.maskedtexturecol.is_some() {
            render_masked_seg_range(state, &ds, ds.x1, ds.x2);
        }
    }
    if state.render.r_main.viewangleoffset == Angle::ZERO {
        draw_player_sprites(state);
    }
}
