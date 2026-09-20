use crate::game_state::GameState;
use crate::i_system::error;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::INT_MAX;
use crate::m_fixed::INT_MIN;
use crate::p_mobj::LineFlags;

use crate::r_data::get_column;
use crate::r_defs::ClipArray;
use crate::r_defs::DrawSeg;
use crate::r_draw::advance_source;
use crate::r_main::point_to_dist;
use crate::r_main::scale_from_global_angle;
use crate::r_main::LightRow48;
use crate::r_main::LIGHTLEVELS;
use crate::r_main::LIGHTSCALESHIFT;
use crate::r_main::LIGHTSEGSHIFT;
use crate::r_main::MAXLIGHTSCALE;
use crate::r_plane::check_plane;
use crate::r_things::draw_masked_column;

use crate::tables::Angle;
use crate::tables::ANG180;
use crate::tables::ANG90;
use crate::tables::ANGLETOFINESHIFT;
use crate::tables::FINESINE;
use crate::tables::FINETANGENT;

pub struct RSegsState {
    pub segtextured: bool,
    pub markfloor: bool,
    pub markceiling: bool,
    pub maskedtexture: bool,
    pub toptexture: i32,
    pub bottomtexture: i32,
    pub midtexture: i32,
    pub rw_normalangle: Angle,
    pub rw_angle1: i32,
    pub rw_x: i32,
    pub rw_stopx: i32,
    pub rw_centerangle: Angle,
    pub rw_offset: Fixed,
    pub rw_distance: Fixed,
    pub rw_scale: Fixed,
    pub rw_scalestep: Fixed,
    pub rw_midtexturemid: Fixed,
    pub rw_toptexturemid: Fixed,
    pub rw_bottomtexturemid: Fixed,
    pub worldtop: i32,
    pub worldbottom: i32,
    pub worldhigh: i32,
    pub worldlow: i32,
    pub pixhigh: Fixed,
    pub pixlow: Fixed,
    pub pixhighstep: Fixed,
    pub pixlowstep: Fixed,
    pub topfrac: Fixed,
    pub topstep: Fixed,
    pub bottomfrac: Fixed,
    pub bottomstep: Fixed,
    pub walllights: LightRow48,
    pub maskedtexturecol: Option<ClipArray>,
}

impl Default for RSegsState {
    fn default() -> Self {
        Self::new()
    }
}

impl RSegsState {
    pub const fn new() -> Self {
        Self {
            segtextured: false,
            markfloor: false,
            markceiling: false,
            maskedtexture: false,
            toptexture: 0,
            bottomtexture: 0,
            midtexture: 0,
            rw_normalangle: 0,
            rw_angle1: 0,
            rw_x: 0,
            rw_stopx: 0,
            rw_centerangle: 0,
            rw_offset: 0,
            rw_distance: 0,
            rw_scale: 0,
            rw_scalestep: 0,
            rw_midtexturemid: 0,
            rw_toptexturemid: 0,
            rw_bottomtexturemid: 0,
            worldtop: 0,
            worldbottom: 0,
            worldhigh: 0,
            worldlow: 0,
            pixhigh: 0,
            pixlow: 0,
            pixhighstep: 0,
            pixlowstep: 0,
            topfrac: 0,
            topstep: 0,
            bottomfrac: 0,
            bottomstep: 0,
            walllights: LightRow48::Normal(0),
            maskedtexturecol: None,
        }
    }
}

pub const SHRT_MAX: i32 = __SHRT_MAX__;
pub const SIL_BOTTOM: i32 = 1;
pub const SIL_TOP: i32 = 2;
pub const SIL_BOTH: i32 = 3;
pub const MAXDRAWSEGS: i32 = 256;
pub fn render_masked_seg_range(state: &mut GameState, ds: &DrawSeg, x1: i32, x2: i32) {
    let mut index: u32;
    let mut lightnum: i32;

    state.render.r_bsp.curline = ds.curline;
    state.render.r_bsp.frontsector = state
        .world
        .p_setup
        .seg(state.render.r_bsp.curline)
        .frontsector;
    state.render.r_bsp.backsector = state
        .world
        .p_setup
        .seg(state.render.r_bsp.curline)
        .backsector;
    let texnum: i32 = state.render.r_data.texturetranslation[state
        .world
        .p_setup
        .side_mut(state.world.p_setup.seg(state.render.r_bsp.curline).sidedef)
        .midtexture as usize];
    lightnum = (state
        .world
        .p_setup
        .sector_mut(state.render.r_bsp.frontsector.unwrap())
        .lightlevel as i32
        >> LIGHTSEGSHIFT)
        + state.render.r_main.extralight;
    let curline_v1 = state.world.p_setup.vertexes
        [state.world.p_setup.seg(state.render.r_bsp.curline).v1.0 as usize];
    let curline_v2 = state.world.p_setup.vertexes
        [state.world.p_setup.seg(state.render.r_bsp.curline).v2.0 as usize];
    if curline_v1.y == curline_v2.y {
        lightnum -= 1;
    } else if curline_v1.x == curline_v2.x {
        lightnum += 1;
    }
    if lightnum < 0 {
        state.render.r_segs.walllights = LightRow48::Normal(0);
    } else if lightnum >= LIGHTLEVELS {
        state.render.r_segs.walllights = LightRow48::Normal((LIGHTLEVELS - 1) as usize);
    } else {
        state.render.r_segs.walllights = LightRow48::Normal(lightnum as usize);
    }
    state.render.r_segs.maskedtexturecol = ds.maskedtexturecol;
    state.render.r_segs.rw_scalestep = ds.scalestep;
    state.render.r_things.spryscale =
        ds.scale1 + (x1 as Fixed - ds.x1 as Fixed) * state.render.r_segs.rw_scalestep;
    state.render.r_things.mfloorclip = ds.sprbottomclip;
    state.render.r_things.mceilingclip = ds.sprtopclip;
    if state
        .world
        .p_setup
        .line_mut(state.world.p_setup.seg(state.render.r_bsp.curline).linedef)
        .flags
        .contains(LineFlags::DONTPEGBOTTOM)
    {
        state.render.r_draw.dc_texturemid = if state
            .world
            .p_setup
            .sector_mut(state.render.r_bsp.frontsector.unwrap())
            .floorheight
            > state
                .world
                .p_setup
                .sector_mut(state.render.r_bsp.backsector.unwrap())
                .floorheight
        {
            state
                .world
                .p_setup
                .sector_mut(state.render.r_bsp.frontsector.unwrap())
                .floorheight
        } else {
            state
                .world
                .p_setup
                .sector_mut(state.render.r_bsp.backsector.unwrap())
                .floorheight
        };
        state.render.r_draw.dc_texturemid = state.render.r_draw.dc_texturemid
            + state.render.r_data.textureheight[texnum as usize]
            - state.render.r_main.viewz;
    } else {
        state.render.r_draw.dc_texturemid = if state
            .world
            .p_setup
            .sector_mut(state.render.r_bsp.frontsector.unwrap())
            .ceilingheight
            < state
                .world
                .p_setup
                .sector_mut(state.render.r_bsp.backsector.unwrap())
                .ceilingheight
        {
            state
                .world
                .p_setup
                .sector_mut(state.render.r_bsp.frontsector.unwrap())
                .ceilingheight
        } else {
            state
                .world
                .p_setup
                .sector_mut(state.render.r_bsp.backsector.unwrap())
                .ceilingheight
        };
        state.render.r_draw.dc_texturemid -= state.render.r_main.viewz;
    }
    state.render.r_draw.dc_texturemid += state
        .world
        .p_setup
        .side_mut(state.world.p_setup.seg(state.render.r_bsp.curline).sidedef)
        .rowoffset;
    if state.render.r_main.fixedcolormap.is_some() {
        state.render.r_draw.dc_colormap = state.render.r_main.fixedcolormap;
    }
    let maskedtexturecol = state.render.r_segs.maskedtexturecol.unwrap();
    state.render.r_draw.dc_x = x1;
    while state.render.r_draw.dc_x <= x2 {
        if maskedtexturecol.get(state, state.render.r_draw.dc_x as isize) as i32 != SHRT_MAX {
            if state.render.r_main.fixedcolormap.is_none() {
                index = (state.render.r_things.spryscale >> LIGHTSCALESHIFT) as u32;
                if index >= MAXLIGHTSCALE as u32 {
                    index = (MAXLIGHTSCALE - 1) as u32;
                }
                state.render.r_draw.dc_colormap = Some(
                    state
                        .render
                        .r_main
                        .light_row48(state.render.r_segs.walllights)[index as usize],
                );
            }
            state.render.r_things.sprtopscreen = state.render.r_main.centeryfrac
                - fixed_mul(
                    state.render.r_draw.dc_texturemid,
                    state.render.r_things.spryscale,
                );
            state.render.r_draw.dc_iscale =
                0xffffffff_u32.wrapping_div(state.render.r_things.spryscale as u32) as Fixed;
            let column = maskedtexturecol.get(state, state.render.r_draw.dc_x as isize) as i32;
            let col = advance_source(
                get_column(
                    &*state.assets.fs,
                    &mut state.render.r_data,
                    &mut state.assets.w_wad,
                    texnum,
                    column,
                ),
                (-3_isize) as usize,
            );
            draw_masked_column(state, col);
            maskedtexturecol.set(state, state.render.r_draw.dc_x as isize, SHRT_MAX as i16);
        }
        state.render.r_things.spryscale += state.render.r_segs.rw_scalestep;
        state.render.r_draw.dc_x += 1;
    }
}
pub const HEIGHTBITS: i32 = 12;
pub const HEIGHTUNIT: i32 = 1 << HEIGHTBITS;
pub fn render_seg_loop(state: &mut GameState) {
    let mut angle: Angle;
    let mut index: u32;
    let mut yl: i32;
    let mut yh: i32;
    let mut mid: i32;
    let mut texturecolumn: Fixed;
    let mut top: i32;
    let mut bottom: i32;
    while state.render.r_segs.rw_x < state.render.r_segs.rw_stopx {
        yl = (state.render.r_segs.topfrac + HEIGHTUNIT - 1) >> HEIGHTBITS;
        if yl < state.render.r_plane.ceilingclip[state.render.r_segs.rw_x as usize] as i32 + 1 {
            yl = state.render.r_plane.ceilingclip[state.render.r_segs.rw_x as usize] as i32 + 1;
        }
        if state.render.r_segs.markceiling {
            top = state.render.r_plane.ceilingclip[state.render.r_segs.rw_x as usize] as i32 + 1;
            bottom = yl - 1;
            if bottom >= state.render.r_plane.floorclip[state.render.r_segs.rw_x as usize] as i32 {
                bottom =
                    state.render.r_plane.floorclip[state.render.r_segs.rw_x as usize] as i32 - 1;
            }
            if top <= bottom {
                let ceilingplane = state.render.r_plane.ceilingplane.unwrap();
                state.render.r_plane.visplanes[ceilingplane]
                    .set_top(state.render.r_segs.rw_x, top as u8);
                state.render.r_plane.visplanes[ceilingplane]
                    .set_bottom(state.render.r_segs.rw_x, bottom as u8);
            }
        }
        yh = state.render.r_segs.bottomfrac >> HEIGHTBITS;
        if yh >= state.render.r_plane.floorclip[state.render.r_segs.rw_x as usize] as i32 {
            yh = state.render.r_plane.floorclip[state.render.r_segs.rw_x as usize] as i32 - 1;
        }
        if state.render.r_segs.markfloor {
            top = yh + 1;
            bottom = state.render.r_plane.floorclip[state.render.r_segs.rw_x as usize] as i32 - 1;
            if top <= state.render.r_plane.ceilingclip[state.render.r_segs.rw_x as usize] as i32 {
                top =
                    state.render.r_plane.ceilingclip[state.render.r_segs.rw_x as usize] as i32 + 1;
            }
            if top <= bottom {
                let floorplane = state.render.r_plane.floorplane.unwrap();
                state.render.r_plane.visplanes[floorplane]
                    .set_top(state.render.r_segs.rw_x, top as u8);
                state.render.r_plane.visplanes[floorplane]
                    .set_bottom(state.render.r_segs.rw_x, bottom as u8);
            }
        }
        if state.render.r_segs.segtextured {
            angle =
                state.render.r_segs.rw_centerangle.wrapping_add(
                    state.render.r_main.xtoviewangle[state.render.r_segs.rw_x as usize],
                ) >> ANGLETOFINESHIFT;
            // A column at a seg's clipped edge can land just outside the
            // front half-plane; vanilla reads past finetangent[] there.
            angle = angle.min(FINETANGENT.len() as Angle - 1);
            texturecolumn = state.render.r_segs.rw_offset
                - fixed_mul(FINETANGENT[angle as usize], state.render.r_segs.rw_distance);
            texturecolumn >>= FRACBITS;
            index = (state.render.r_segs.rw_scale >> LIGHTSCALESHIFT) as u32;
            if index >= MAXLIGHTSCALE as u32 {
                index = (MAXLIGHTSCALE - 1) as u32;
            }
            state.render.r_draw.dc_colormap = Some(
                state
                    .render
                    .r_main
                    .light_row48(state.render.r_segs.walllights)[index as usize],
            );
            state.render.r_draw.dc_x = state.render.r_segs.rw_x;
            state.render.r_draw.dc_iscale =
                0xffffffff_u32.wrapping_div(state.render.r_segs.rw_scale as u32) as Fixed;
        } else {
            texturecolumn = 0;
        }
        if state.render.r_segs.midtexture != 0 {
            state.render.r_draw.dc_yl = yl;
            state.render.r_draw.dc_yh = yh;
            state.render.r_draw.dc_texturemid = state.render.r_segs.rw_midtexturemid;
            state.render.r_draw.dc_source = Some(get_column(
                &*state.assets.fs,
                &mut state.render.r_data,
                &mut state.assets.w_wad,
                state.render.r_segs.midtexture,
                texturecolumn,
            ));
            state
                .render
                .r_main
                .colfunc
                .expect("non-null function pointer")(state);
            state.render.r_plane.ceilingclip[state.render.r_segs.rw_x as usize] =
                state.render.r_draw.viewheight as i16;
            state.render.r_plane.floorclip[state.render.r_segs.rw_x as usize] = -1_i16;
        } else {
            if state.render.r_segs.toptexture != 0 {
                mid = state.render.r_segs.pixhigh >> HEIGHTBITS;
                state.render.r_segs.pixhigh += state.render.r_segs.pixhighstep;
                if mid >= state.render.r_plane.floorclip[state.render.r_segs.rw_x as usize] as i32 {
                    mid = state.render.r_plane.floorclip[state.render.r_segs.rw_x as usize] as i32
                        - 1;
                }
                if mid >= yl {
                    state.render.r_draw.dc_yl = yl;
                    state.render.r_draw.dc_yh = mid;
                    state.render.r_draw.dc_texturemid = state.render.r_segs.rw_toptexturemid;
                    state.render.r_draw.dc_source = Some(get_column(
                        &*state.assets.fs,
                        &mut state.render.r_data,
                        &mut state.assets.w_wad,
                        state.render.r_segs.toptexture,
                        texturecolumn,
                    ));
                    state
                        .render
                        .r_main
                        .colfunc
                        .expect("non-null function pointer")(state);
                    state.render.r_plane.ceilingclip[state.render.r_segs.rw_x as usize] =
                        mid as i16;
                } else {
                    state.render.r_plane.ceilingclip[state.render.r_segs.rw_x as usize] =
                        (yl - 1) as i16;
                }
            } else if state.render.r_segs.markceiling {
                state.render.r_plane.ceilingclip[state.render.r_segs.rw_x as usize] =
                    (yl - 1) as i16;
            }
            if state.render.r_segs.bottomtexture != 0 {
                mid = (state.render.r_segs.pixlow + HEIGHTUNIT - 1) >> HEIGHTBITS;
                state.render.r_segs.pixlow += state.render.r_segs.pixlowstep;
                if mid <= state.render.r_plane.ceilingclip[state.render.r_segs.rw_x as usize] as i32
                {
                    mid = state.render.r_plane.ceilingclip[state.render.r_segs.rw_x as usize]
                        as i32
                        + 1;
                }
                if mid <= yh {
                    state.render.r_draw.dc_yl = mid;
                    state.render.r_draw.dc_yh = yh;
                    state.render.r_draw.dc_texturemid = state.render.r_segs.rw_bottomtexturemid;
                    state.render.r_draw.dc_source = Some(get_column(
                        &*state.assets.fs,
                        &mut state.render.r_data,
                        &mut state.assets.w_wad,
                        state.render.r_segs.bottomtexture,
                        texturecolumn,
                    ));
                    state
                        .render
                        .r_main
                        .colfunc
                        .expect("non-null function pointer")(state);
                    state.render.r_plane.floorclip[state.render.r_segs.rw_x as usize] = mid as i16;
                } else {
                    state.render.r_plane.floorclip[state.render.r_segs.rw_x as usize] =
                        (yh + 1) as i16;
                }
            } else if state.render.r_segs.markfloor {
                state.render.r_plane.floorclip[state.render.r_segs.rw_x as usize] = (yh + 1) as i16;
            }
            if state.render.r_segs.maskedtexture {
                let maskedtexturecol = state.render.r_segs.maskedtexturecol.unwrap();
                maskedtexturecol.set(
                    state,
                    state.render.r_segs.rw_x as isize,
                    texturecolumn as i16,
                );
            }
        }
        state.render.r_segs.rw_scale += state.render.r_segs.rw_scalestep;
        state.render.r_segs.topfrac += state.render.r_segs.topstep;
        state.render.r_segs.bottomfrac += state.render.r_segs.bottomstep;
        state.render.r_segs.rw_x += 1;
    }
}
pub fn store_wall_range(state: &mut GameState, start: i32, stop: i32) {
    let mut sineval: Fixed;

    let mut offsetangle: Angle;
    let vtop: Fixed;
    let mut lightnum: i32;
    if state.render.r_bsp.ds_p == MAXDRAWSEGS as usize {
        return;
    }
    if start >= state.render.r_draw.viewwidth || start > stop {
        error(&format!("Bad R_RenderWallRange: {start} to {stop}"));
    }
    state.render.r_bsp.sidedef = state.world.p_setup.seg(state.render.r_bsp.curline).sidedef;
    state.render.r_bsp.linedef = state.world.p_setup.seg(state.render.r_bsp.curline).linedef;
    state
        .world
        .p_setup
        .line_mut(state.render.r_bsp.linedef)
        .flags
        .insert(LineFlags::MAPPED);
    state.render.r_segs.rw_normalangle = state
        .world
        .p_setup
        .seg(state.render.r_bsp.curline)
        .angle
        .wrapping_add(ANG90 as Angle);
    offsetangle = (state
        .render
        .r_segs
        .rw_normalangle
        .wrapping_sub(state.render.r_segs.rw_angle1 as Angle) as i32)
        .unsigned_abs();
    if offsetangle > ANG90 as Angle {
        offsetangle = ANG90 as Angle;
    }
    let distangle: Angle = (ANG90 as Angle).wrapping_sub(offsetangle);
    let curline_v1 = state.world.p_setup.vertexes
        [state.world.p_setup.seg(state.render.r_bsp.curline).v1.0 as usize];
    let (v1x, v1y) = (curline_v1.x, curline_v1.y);
    let hyp: Fixed = point_to_dist(&state.render.r_main, v1x, v1y);
    sineval = FINESINE[(distangle >> ANGLETOFINESHIFT) as usize];
    state.render.r_segs.rw_distance = fixed_mul(hyp, sineval);
    state.render.r_segs.rw_x = start;
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].x1 = state.render.r_segs.rw_x;
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].x2 = stop;
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].curline = state.render.r_bsp.curline;
    state.render.r_segs.rw_stopx = stop + 1;
    let angle1 = state
        .render
        .r_main
        .viewangle
        .wrapping_add(state.render.r_main.xtoviewangle[start as usize]);
    state.render.r_segs.rw_scale =
        scale_from_global_angle(&state.render.r_main, &state.render.r_segs, angle1);
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].scale1 = state.render.r_segs.rw_scale;
    if stop > start {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].scale2 = {
            let angle2 = state
                .render
                .r_main
                .viewangle
                .wrapping_add(state.render.r_main.xtoviewangle[stop as usize]);
            scale_from_global_angle(&state.render.r_main, &state.render.r_segs, angle2)
        };
        state.render.r_segs.rw_scalestep = ((state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p]
            .scale2
            - state.render.r_segs.rw_scale)
            / (stop - start)) as Fixed;
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].scalestep =
            state.render.r_segs.rw_scalestep;
    } else {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].scale2 =
            state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].scale1;
    }
    state.render.r_segs.worldtop = state
        .world
        .p_setup
        .sector_mut(state.render.r_bsp.frontsector.unwrap())
        .ceilingheight
        - state.render.r_main.viewz;
    state.render.r_segs.worldbottom = state
        .world
        .p_setup
        .sector_mut(state.render.r_bsp.frontsector.unwrap())
        .floorheight
        - state.render.r_main.viewz;
    state.render.r_segs.maskedtexture = false;
    state.render.r_segs.bottomtexture = state.render.r_segs.maskedtexture as i32;
    state.render.r_segs.toptexture = state.render.r_segs.bottomtexture;
    state.render.r_segs.midtexture = state.render.r_segs.toptexture;
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].maskedtexturecol = None;
    match state.render.r_bsp.backsector {
        None => {
            state.render.r_segs.midtexture = state.render.r_data.texturetranslation[state
                .world
                .p_setup
                .side_mut(state.render.r_bsp.sidedef)
                .midtexture
                as usize];
            state.render.r_segs.markceiling = true;
            state.render.r_segs.markfloor = state.render.r_segs.markceiling;
            if state
                .world
                .p_setup
                .line_mut(state.render.r_bsp.linedef)
                .flags
                .contains(LineFlags::DONTPEGBOTTOM)
            {
                vtop = state
                    .world
                    .p_setup
                    .sector_mut(state.render.r_bsp.frontsector.unwrap())
                    .floorheight
                    + state.render.r_data.textureheight[state
                        .world
                        .p_setup
                        .side_mut(state.render.r_bsp.sidedef)
                        .midtexture
                        as usize];
                state.render.r_segs.rw_midtexturemid = vtop - state.render.r_main.viewz;
            } else {
                state.render.r_segs.rw_midtexturemid = state.render.r_segs.worldtop as Fixed;
            }
            state.render.r_segs.rw_midtexturemid += state
                .world
                .p_setup
                .side_mut(state.render.r_bsp.sidedef)
                .rowoffset;
            state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette = SIL_BOTH;
            state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprtopclip =
                Some(ClipArray::ScreenHeightArray);
            state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprbottomclip =
                Some(ClipArray::NegOneArray);
            state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].bsilheight = INT_MAX as Fixed;
            state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].tsilheight = INT_MIN as Fixed;
        }
        Some(backsector) => {
            state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprbottomclip = None;
            state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprtopclip =
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprbottomclip;
            state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette = 0;
            if state
                .world
                .p_setup
                .sector_mut(state.render.r_bsp.frontsector.unwrap())
                .floorheight
                > state.world.p_setup.sector_mut(backsector).floorheight
            {
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette = SIL_BOTTOM;
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].bsilheight = state
                    .world
                    .p_setup
                    .sector_mut(state.render.r_bsp.frontsector.unwrap())
                    .floorheight;
            } else if state.world.p_setup.sector_mut(backsector).floorheight
                > state.render.r_main.viewz
            {
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette = SIL_BOTTOM;
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].bsilheight = INT_MAX as Fixed;
            }
            if state
                .world
                .p_setup
                .sector_mut(state.render.r_bsp.frontsector.unwrap())
                .ceilingheight
                < state.world.p_setup.sector_mut(backsector).ceilingheight
            {
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette |= SIL_TOP;
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].tsilheight = state
                    .world
                    .p_setup
                    .sector_mut(state.render.r_bsp.frontsector.unwrap())
                    .ceilingheight;
            } else if state.world.p_setup.sector_mut(backsector).ceilingheight
                < state.render.r_main.viewz
            {
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette |= SIL_TOP;
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].tsilheight = INT_MIN as Fixed;
            }
            if state.world.p_setup.sector_mut(backsector).ceilingheight
                <= state
                    .world
                    .p_setup
                    .sector_mut(state.render.r_bsp.frontsector.unwrap())
                    .floorheight
            {
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprbottomclip =
                    Some(ClipArray::NegOneArray);
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].bsilheight = INT_MAX as Fixed;
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette |= SIL_BOTTOM;
            }
            if state.world.p_setup.sector_mut(backsector).floorheight
                >= state
                    .world
                    .p_setup
                    .sector_mut(state.render.r_bsp.frontsector.unwrap())
                    .ceilingheight
            {
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprtopclip =
                    Some(ClipArray::ScreenHeightArray);
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].tsilheight = INT_MIN as Fixed;
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette |= SIL_TOP;
            }
            state.render.r_segs.worldhigh =
                state.world.p_setup.sector_mut(backsector).ceilingheight
                    - state.render.r_main.viewz;
            state.render.r_segs.worldlow =
                state.world.p_setup.sector_mut(backsector).floorheight - state.render.r_main.viewz;
            if state
                .world
                .p_setup
                .sector_mut(state.render.r_bsp.frontsector.unwrap())
                .ceilingpic as i32
                == state.render.r_sky.skyflatnum
                && state.world.p_setup.sector_mut(backsector).ceilingpic as i32
                    == state.render.r_sky.skyflatnum
            {
                state.render.r_segs.worldtop = state.render.r_segs.worldhigh;
            }
            state.render.r_segs.markfloor = state.render.r_segs.worldlow
                != state.render.r_segs.worldbottom
                || state.world.p_setup.sector_mut(backsector).floorpic as i32
                    != state
                        .world
                        .p_setup
                        .sector_mut(state.render.r_bsp.frontsector.unwrap())
                        .floorpic as i32
                || state.world.p_setup.sector_mut(backsector).lightlevel as i32
                    != state
                        .world
                        .p_setup
                        .sector_mut(state.render.r_bsp.frontsector.unwrap())
                        .lightlevel as i32;
            state.render.r_segs.markceiling = state.render.r_segs.worldhigh
                != state.render.r_segs.worldtop
                || state.world.p_setup.sector_mut(backsector).ceilingpic as i32
                    != state
                        .world
                        .p_setup
                        .sector_mut(state.render.r_bsp.frontsector.unwrap())
                        .ceilingpic as i32
                || state.world.p_setup.sector_mut(backsector).lightlevel as i32
                    != state
                        .world
                        .p_setup
                        .sector_mut(state.render.r_bsp.frontsector.unwrap())
                        .lightlevel as i32;
            if state.world.p_setup.sector_mut(backsector).ceilingheight
                <= state
                    .world
                    .p_setup
                    .sector_mut(state.render.r_bsp.frontsector.unwrap())
                    .floorheight
                || state.world.p_setup.sector_mut(backsector).floorheight
                    >= state
                        .world
                        .p_setup
                        .sector_mut(state.render.r_bsp.frontsector.unwrap())
                        .ceilingheight
            {
                state.render.r_segs.markfloor = true;
                state.render.r_segs.markceiling = state.render.r_segs.markfloor;
            }
            if state.render.r_segs.worldhigh < state.render.r_segs.worldtop {
                state.render.r_segs.toptexture = state.render.r_data.texturetranslation[state
                    .world
                    .p_setup
                    .side_mut(state.render.r_bsp.sidedef)
                    .toptexture
                    as usize];
                if state
                    .world
                    .p_setup
                    .line_mut(state.render.r_bsp.linedef)
                    .flags
                    .contains(LineFlags::DONTPEGTOP)
                {
                    state.render.r_segs.rw_toptexturemid = state.render.r_segs.worldtop as Fixed;
                } else {
                    vtop = state.world.p_setup.sector_mut(backsector).ceilingheight
                        + state.render.r_data.textureheight[state
                            .world
                            .p_setup
                            .side_mut(state.render.r_bsp.sidedef)
                            .toptexture
                            as usize];
                    state.render.r_segs.rw_toptexturemid = vtop - state.render.r_main.viewz;
                }
            }
            if state.render.r_segs.worldlow > state.render.r_segs.worldbottom {
                state.render.r_segs.bottomtexture = state.render.r_data.texturetranslation[state
                    .world
                    .p_setup
                    .side_mut(state.render.r_bsp.sidedef)
                    .bottomtexture
                    as usize];
                if state
                    .world
                    .p_setup
                    .line_mut(state.render.r_bsp.linedef)
                    .flags
                    .contains(LineFlags::DONTPEGBOTTOM)
                {
                    state.render.r_segs.rw_bottomtexturemid = state.render.r_segs.worldtop as Fixed;
                } else {
                    state.render.r_segs.rw_bottomtexturemid = state.render.r_segs.worldlow as Fixed;
                }
            }
            state.render.r_segs.rw_toptexturemid += state
                .world
                .p_setup
                .side_mut(state.render.r_bsp.sidedef)
                .rowoffset;
            state.render.r_segs.rw_bottomtexturemid += state
                .world
                .p_setup
                .side_mut(state.render.r_bsp.sidedef)
                .rowoffset;
            if state
                .world
                .p_setup
                .side_mut(state.render.r_bsp.sidedef)
                .midtexture
                != 0
            {
                state.render.r_segs.maskedtexture = true;
                state.render.r_segs.maskedtexturecol = Some(ClipArray::Openings(
                    state.render.r_plane.lastopening as isize - state.render.r_segs.rw_x as isize,
                ));
                state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].maskedtexturecol =
                    state.render.r_segs.maskedtexturecol;
                state.render.r_plane.lastopening +=
                    (state.render.r_segs.rw_stopx - state.render.r_segs.rw_x) as usize;
            }
        }
    }
    state.render.r_segs.segtextured = (state.render.r_segs.midtexture
        | state.render.r_segs.toptexture
        | state.render.r_segs.bottomtexture)
        != 0
        || state.render.r_segs.maskedtexture;
    if state.render.r_segs.segtextured {
        offsetangle = state
            .render
            .r_segs
            .rw_normalangle
            .wrapping_sub(state.render.r_segs.rw_angle1 as Angle);
        if offsetangle > ANG180 {
            offsetangle = offsetangle.wrapping_neg();
        }
        if offsetangle > ANG90 as Angle {
            offsetangle = ANG90 as Angle;
        }
        sineval = FINESINE[(offsetangle >> ANGLETOFINESHIFT) as usize];
        state.render.r_segs.rw_offset = fixed_mul(hyp, sineval);
        if state
            .render
            .r_segs
            .rw_normalangle
            .wrapping_sub(state.render.r_segs.rw_angle1 as Angle)
            < ANG180
        {
            state.render.r_segs.rw_offset = -state.render.r_segs.rw_offset;
        }
        state.render.r_segs.rw_offset += state
            .world
            .p_setup
            .side_mut(state.render.r_bsp.sidedef)
            .textureoffset
            + state.world.p_setup.seg(state.render.r_bsp.curline).offset;
        state.render.r_segs.rw_centerangle = (ANG90 as Angle)
            .wrapping_add(state.render.r_main.viewangle)
            .wrapping_sub(state.render.r_segs.rw_normalangle);
        if state.render.r_main.fixedcolormap.is_none() {
            lightnum = (state
                .world
                .p_setup
                .sector_mut(state.render.r_bsp.frontsector.unwrap())
                .lightlevel as i32
                >> LIGHTSEGSHIFT)
                + state.render.r_main.extralight;
            let curline_v1 = state.world.p_setup.vertexes
                [state.world.p_setup.seg(state.render.r_bsp.curline).v1.0 as usize];
            let curline_v2 = state.world.p_setup.vertexes
                [state.world.p_setup.seg(state.render.r_bsp.curline).v2.0 as usize];
            if curline_v1.y == curline_v2.y {
                lightnum -= 1;
            } else if curline_v1.x == curline_v2.x {
                lightnum += 1;
            }
            if lightnum < 0 {
                state.render.r_segs.walllights = LightRow48::Normal(0);
            } else if lightnum >= LIGHTLEVELS {
                state.render.r_segs.walllights = LightRow48::Normal((LIGHTLEVELS - 1) as usize);
            } else {
                state.render.r_segs.walllights = LightRow48::Normal(lightnum as usize);
            }
        }
    }
    if state
        .world
        .p_setup
        .sector_mut(state.render.r_bsp.frontsector.unwrap())
        .floorheight
        >= state.render.r_main.viewz
    {
        state.render.r_segs.markfloor = false;
    }
    if state
        .world
        .p_setup
        .sector_mut(state.render.r_bsp.frontsector.unwrap())
        .ceilingheight
        <= state.render.r_main.viewz
        && state
            .world
            .p_setup
            .sector_mut(state.render.r_bsp.frontsector.unwrap())
            .ceilingpic as i32
            != state.render.r_sky.skyflatnum
    {
        state.render.r_segs.markceiling = false;
    }
    state.render.r_segs.worldtop >>= 4;
    state.render.r_segs.worldbottom >>= 4;
    state.render.r_segs.topstep = -fixed_mul(
        state.render.r_segs.rw_scalestep,
        state.render.r_segs.worldtop as Fixed,
    );
    state.render.r_segs.topfrac = (state.render.r_main.centeryfrac >> 4)
        - fixed_mul(
            state.render.r_segs.worldtop as Fixed,
            state.render.r_segs.rw_scale,
        );
    state.render.r_segs.bottomstep = -fixed_mul(
        state.render.r_segs.rw_scalestep,
        state.render.r_segs.worldbottom as Fixed,
    );
    state.render.r_segs.bottomfrac = (state.render.r_main.centeryfrac >> 4)
        - fixed_mul(
            state.render.r_segs.worldbottom as Fixed,
            state.render.r_segs.rw_scale,
        );
    if state.render.r_bsp.backsector.is_some() {
        state.render.r_segs.worldhigh >>= 4;
        state.render.r_segs.worldlow >>= 4;
        if state.render.r_segs.worldhigh < state.render.r_segs.worldtop {
            state.render.r_segs.pixhigh = (state.render.r_main.centeryfrac >> 4)
                - fixed_mul(
                    state.render.r_segs.worldhigh as Fixed,
                    state.render.r_segs.rw_scale,
                );
            state.render.r_segs.pixhighstep = -fixed_mul(
                state.render.r_segs.rw_scalestep,
                state.render.r_segs.worldhigh as Fixed,
            );
        }
        if state.render.r_segs.worldlow > state.render.r_segs.worldbottom {
            state.render.r_segs.pixlow = (state.render.r_main.centeryfrac >> 4)
                - fixed_mul(
                    state.render.r_segs.worldlow as Fixed,
                    state.render.r_segs.rw_scale,
                );
            state.render.r_segs.pixlowstep = -fixed_mul(
                state.render.r_segs.rw_scalestep,
                state.render.r_segs.worldlow as Fixed,
            );
        }
    }
    if state.render.r_segs.markceiling {
        let (ceilingplane, rw_x, rw_stopx_1) = (
            state.render.r_plane.ceilingplane.unwrap(),
            state.render.r_segs.rw_x,
            state.render.r_segs.rw_stopx - 1,
        );
        state.render.r_plane.ceilingplane = Some(check_plane(
            &mut state.render.r_plane,
            ceilingplane,
            rw_x,
            rw_stopx_1,
        ));
    }
    if state.render.r_segs.markfloor {
        let (floorplane, rw_x2, rw_stopx_2) = (
            state.render.r_plane.floorplane.unwrap(),
            state.render.r_segs.rw_x,
            state.render.r_segs.rw_stopx - 1,
        );
        state.render.r_plane.floorplane = Some(check_plane(
            &mut state.render.r_plane,
            floorplane,
            rw_x2,
            rw_stopx_2,
        ));
    }
    render_seg_loop(state);
    if (state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette & SIL_TOP != 0
        || state.render.r_segs.maskedtexture)
        && state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p]
            .sprtopclip
            .is_none()
    {
        let count = (state.render.r_segs.rw_stopx - start) as usize;
        let lastopening = state.render.r_plane.lastopening;
        state.render.r_plane.openings[lastopening..lastopening + count].copy_from_slice(
            &state.render.r_plane.ceilingclip[start as usize..start as usize + count],
        );
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprtopclip = Some(
            ClipArray::Openings(state.render.r_plane.lastopening as isize - start as isize),
        );
        state.render.r_plane.lastopening += (state.render.r_segs.rw_stopx - start) as usize;
    }
    if (state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette & SIL_BOTTOM != 0
        || state.render.r_segs.maskedtexture)
        && state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p]
            .sprbottomclip
            .is_none()
    {
        let count = (state.render.r_segs.rw_stopx - start) as usize;
        let lastopening = state.render.r_plane.lastopening;
        state.render.r_plane.openings[lastopening..lastopening + count].copy_from_slice(
            &state.render.r_plane.floorclip[start as usize..start as usize + count],
        );
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprbottomclip = Some(
            ClipArray::Openings(state.render.r_plane.lastopening as isize - start as isize),
        );
        state.render.r_plane.lastopening += (state.render.r_segs.rw_stopx - start) as usize;
    }
    if state.render.r_segs.maskedtexture
        && state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette & SIL_TOP == 0
    {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette |= SIL_TOP;
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].tsilheight = INT_MIN as Fixed;
    }
    if state.render.r_segs.maskedtexture
        && state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette & SIL_BOTTOM == 0
    {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette |= SIL_BOTTOM;
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].bsilheight = INT_MAX as Fixed;
    }
    state.render.r_bsp.ds_p += 1;
}
pub const __SHRT_MAX__: i32 = 32767;
