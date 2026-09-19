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

    state.r_bsp.curline = ds.curline;
    state.r_bsp.frontsector = state.p_setup.seg(state.r_bsp.curline).frontsector;
    state.r_bsp.backsector = state.p_setup.seg(state.r_bsp.curline).backsector;
    let texnum: i32 = state.r_data.texturetranslation[state
        .p_setup
        .side_mut(state.p_setup.seg(state.r_bsp.curline).sidedef)
        .midtexture as usize];
    lightnum = (state
        .p_setup
        .sector_mut(state.r_bsp.frontsector.unwrap())
        .lightlevel as i32
        >> LIGHTSEGSHIFT)
        + state.r_main.extralight;
    let curline_v1 = state.p_setup.vertexes[state.p_setup.seg(state.r_bsp.curline).v1.0 as usize];
    let curline_v2 = state.p_setup.vertexes[state.p_setup.seg(state.r_bsp.curline).v2.0 as usize];
    if curline_v1.y == curline_v2.y {
        lightnum -= 1;
    } else if curline_v1.x == curline_v2.x {
        lightnum += 1;
    }
    if lightnum < 0 {
        state.r_segs.walllights = LightRow48::Normal(0);
    } else if lightnum >= LIGHTLEVELS {
        state.r_segs.walllights = LightRow48::Normal((LIGHTLEVELS - 1) as usize);
    } else {
        state.r_segs.walllights = LightRow48::Normal(lightnum as usize);
    }
    state.r_segs.maskedtexturecol = ds.maskedtexturecol;
    state.r_segs.rw_scalestep = ds.scalestep;
    state.r_things.spryscale =
        ds.scale1 + (x1 as Fixed - ds.x1 as Fixed) * state.r_segs.rw_scalestep;
    state.r_things.mfloorclip = ds.sprbottomclip;
    state.r_things.mceilingclip = ds.sprtopclip;
    if state
        .p_setup
        .line_mut(state.p_setup.seg(state.r_bsp.curline).linedef)
        .flags
        .contains(LineFlags::DONTPEGBOTTOM)
    {
        state.r_draw.dc_texturemid = if state
            .p_setup
            .sector_mut(state.r_bsp.frontsector.unwrap())
            .floorheight
            > state
                .p_setup
                .sector_mut(state.r_bsp.backsector.unwrap())
                .floorheight
        {
            state
                .p_setup
                .sector_mut(state.r_bsp.frontsector.unwrap())
                .floorheight
        } else {
            state
                .p_setup
                .sector_mut(state.r_bsp.backsector.unwrap())
                .floorheight
        };
        state.r_draw.dc_texturemid = state.r_draw.dc_texturemid
            + state.r_data.textureheight[texnum as usize]
            - state.r_main.viewz;
    } else {
        state.r_draw.dc_texturemid = if state
            .p_setup
            .sector_mut(state.r_bsp.frontsector.unwrap())
            .ceilingheight
            < state
                .p_setup
                .sector_mut(state.r_bsp.backsector.unwrap())
                .ceilingheight
        {
            state
                .p_setup
                .sector_mut(state.r_bsp.frontsector.unwrap())
                .ceilingheight
        } else {
            state
                .p_setup
                .sector_mut(state.r_bsp.backsector.unwrap())
                .ceilingheight
        };
        state.r_draw.dc_texturemid -= state.r_main.viewz;
    }
    state.r_draw.dc_texturemid += state
        .p_setup
        .side_mut(state.p_setup.seg(state.r_bsp.curline).sidedef)
        .rowoffset;
    if state.r_main.fixedcolormap.is_some() {
        state.r_draw.dc_colormap = state.r_main.fixedcolormap;
    }
    let maskedtexturecol = state.r_segs.maskedtexturecol.unwrap();
    state.r_draw.dc_x = x1;
    while state.r_draw.dc_x <= x2 {
        if maskedtexturecol.get(state, state.r_draw.dc_x as isize) as i32 != SHRT_MAX {
            if state.r_main.fixedcolormap.is_none() {
                index = (state.r_things.spryscale >> LIGHTSCALESHIFT) as u32;
                if index >= MAXLIGHTSCALE as u32 {
                    index = (MAXLIGHTSCALE - 1) as u32;
                }
                state.r_draw.dc_colormap =
                    Some(state.r_main.light_row48(state.r_segs.walllights)[index as usize]);
            }
            state.r_things.sprtopscreen = state.r_main.centeryfrac
                - fixed_mul(state.r_draw.dc_texturemid, state.r_things.spryscale);
            state.r_draw.dc_iscale =
                0xffffffff_u32.wrapping_div(state.r_things.spryscale as u32) as Fixed;
            let col = advance_source(
                get_column(
                    state,
                    texnum,
                    maskedtexturecol.get(state, state.r_draw.dc_x as isize) as i32,
                ),
                (-3_isize) as usize,
            );
            draw_masked_column(state, col);
            maskedtexturecol.set(state, state.r_draw.dc_x as isize, SHRT_MAX as i16);
        }
        state.r_things.spryscale += state.r_segs.rw_scalestep;
        state.r_draw.dc_x += 1;
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
    while state.r_segs.rw_x < state.r_segs.rw_stopx {
        yl = (state.r_segs.topfrac + HEIGHTUNIT - 1) >> HEIGHTBITS;
        if yl < state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 + 1 {
            yl = state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 + 1;
        }
        if state.r_segs.markceiling {
            top = state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 + 1;
            bottom = yl - 1;
            if bottom >= state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 {
                bottom = state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 - 1;
            }
            if top <= bottom {
                let ceilingplane = state.r_plane.ceilingplane.unwrap();
                state.r_plane.visplanes[ceilingplane].set_top(state.r_segs.rw_x, top as u8);
                state.r_plane.visplanes[ceilingplane].set_bottom(state.r_segs.rw_x, bottom as u8);
            }
        }
        yh = state.r_segs.bottomfrac >> HEIGHTBITS;
        if yh >= state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 {
            yh = state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 - 1;
        }
        if state.r_segs.markfloor {
            top = yh + 1;
            bottom = state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 - 1;
            if top <= state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 {
                top = state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 + 1;
            }
            if top <= bottom {
                let floorplane = state.r_plane.floorplane.unwrap();
                state.r_plane.visplanes[floorplane].set_top(state.r_segs.rw_x, top as u8);
                state.r_plane.visplanes[floorplane].set_bottom(state.r_segs.rw_x, bottom as u8);
            }
        }
        if state.r_segs.segtextured {
            angle = state
                .r_segs
                .rw_centerangle
                .wrapping_add(state.r_main.xtoviewangle[state.r_segs.rw_x as usize])
                >> ANGLETOFINESHIFT;
            // A column at a seg's clipped edge can land just outside the
            // front half-plane; vanilla reads past finetangent[] there.
            angle = angle.min(FINETANGENT.len() as Angle - 1);
            texturecolumn = state.r_segs.rw_offset
                - fixed_mul(FINETANGENT[angle as usize], state.r_segs.rw_distance);
            texturecolumn >>= FRACBITS;
            index = (state.r_segs.rw_scale >> LIGHTSCALESHIFT) as u32;
            if index >= MAXLIGHTSCALE as u32 {
                index = (MAXLIGHTSCALE - 1) as u32;
            }
            state.r_draw.dc_colormap =
                Some(state.r_main.light_row48(state.r_segs.walllights)[index as usize]);
            state.r_draw.dc_x = state.r_segs.rw_x;
            state.r_draw.dc_iscale =
                0xffffffff_u32.wrapping_div(state.r_segs.rw_scale as u32) as Fixed;
        } else {
            texturecolumn = 0;
        }
        if state.r_segs.midtexture != 0 {
            state.r_draw.dc_yl = yl;
            state.r_draw.dc_yh = yh;
            state.r_draw.dc_texturemid = state.r_segs.rw_midtexturemid;
            state.r_draw.dc_source =
                Some(get_column(state, state.r_segs.midtexture, texturecolumn));
            state.r_main.colfunc.expect("non-null function pointer")(state);
            state.r_plane.ceilingclip[state.r_segs.rw_x as usize] = state.r_draw.viewheight as i16;
            state.r_plane.floorclip[state.r_segs.rw_x as usize] = -1_i16;
        } else {
            if state.r_segs.toptexture != 0 {
                mid = state.r_segs.pixhigh >> HEIGHTBITS;
                state.r_segs.pixhigh += state.r_segs.pixhighstep;
                if mid >= state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 {
                    mid = state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 - 1;
                }
                if mid >= yl {
                    state.r_draw.dc_yl = yl;
                    state.r_draw.dc_yh = mid;
                    state.r_draw.dc_texturemid = state.r_segs.rw_toptexturemid;
                    state.r_draw.dc_source =
                        Some(get_column(state, state.r_segs.toptexture, texturecolumn));
                    state.r_main.colfunc.expect("non-null function pointer")(state);
                    state.r_plane.ceilingclip[state.r_segs.rw_x as usize] = mid as i16;
                } else {
                    state.r_plane.ceilingclip[state.r_segs.rw_x as usize] = (yl - 1) as i16;
                }
            } else if state.r_segs.markceiling {
                state.r_plane.ceilingclip[state.r_segs.rw_x as usize] = (yl - 1) as i16;
            }
            if state.r_segs.bottomtexture != 0 {
                mid = (state.r_segs.pixlow + HEIGHTUNIT - 1) >> HEIGHTBITS;
                state.r_segs.pixlow += state.r_segs.pixlowstep;
                if mid <= state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 {
                    mid = state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 + 1;
                }
                if mid <= yh {
                    state.r_draw.dc_yl = mid;
                    state.r_draw.dc_yh = yh;
                    state.r_draw.dc_texturemid = state.r_segs.rw_bottomtexturemid;
                    state.r_draw.dc_source =
                        Some(get_column(state, state.r_segs.bottomtexture, texturecolumn));
                    state.r_main.colfunc.expect("non-null function pointer")(state);
                    state.r_plane.floorclip[state.r_segs.rw_x as usize] = mid as i16;
                } else {
                    state.r_plane.floorclip[state.r_segs.rw_x as usize] = (yh + 1) as i16;
                }
            } else if state.r_segs.markfloor {
                state.r_plane.floorclip[state.r_segs.rw_x as usize] = (yh + 1) as i16;
            }
            if state.r_segs.maskedtexture {
                let maskedtexturecol = state.r_segs.maskedtexturecol.unwrap();
                maskedtexturecol.set(state, state.r_segs.rw_x as isize, texturecolumn as i16);
            }
        }
        state.r_segs.rw_scale += state.r_segs.rw_scalestep;
        state.r_segs.topfrac += state.r_segs.topstep;
        state.r_segs.bottomfrac += state.r_segs.bottomstep;
        state.r_segs.rw_x += 1;
    }
}
pub fn store_wall_range(state: &mut GameState, start: i32, stop: i32) {
    let mut sineval: Fixed;

    let mut offsetangle: Angle;
    let vtop: Fixed;
    let mut lightnum: i32;
    if state.r_bsp.ds_p == MAXDRAWSEGS as usize {
        return;
    }
    if start >= state.r_draw.viewwidth || start > stop {
        error(&format!("Bad R_RenderWallRange: {start} to {stop}"));
    }
    state.r_bsp.sidedef = state.p_setup.seg(state.r_bsp.curline).sidedef;
    state.r_bsp.linedef = state.p_setup.seg(state.r_bsp.curline).linedef;
    state
        .p_setup
        .line_mut(state.r_bsp.linedef)
        .flags
        .insert(LineFlags::MAPPED);
    state.r_segs.rw_normalangle = state
        .p_setup
        .seg(state.r_bsp.curline)
        .angle
        .wrapping_add(ANG90 as Angle);
    offsetangle = (state
        .r_segs
        .rw_normalangle
        .wrapping_sub(state.r_segs.rw_angle1 as Angle) as i32)
        .unsigned_abs();
    if offsetangle > ANG90 as Angle {
        offsetangle = ANG90 as Angle;
    }
    let distangle: Angle = (ANG90 as Angle).wrapping_sub(offsetangle);
    let curline_v1 = state.p_setup.vertexes[state.p_setup.seg(state.r_bsp.curline).v1.0 as usize];
    let (v1x, v1y) = (curline_v1.x, curline_v1.y);
    let hyp: Fixed = point_to_dist(state, v1x, v1y);
    sineval = FINESINE[(distangle >> ANGLETOFINESHIFT) as usize];
    state.r_segs.rw_distance = fixed_mul(hyp, sineval);
    state.r_segs.rw_x = start;
    state.r_bsp.drawsegs[state.r_bsp.ds_p].x1 = state.r_segs.rw_x;
    state.r_bsp.drawsegs[state.r_bsp.ds_p].x2 = stop;
    state.r_bsp.drawsegs[state.r_bsp.ds_p].curline = state.r_bsp.curline;
    state.r_segs.rw_stopx = stop + 1;
    let angle1 = state
        .r_main
        .viewangle
        .wrapping_add(state.r_main.xtoviewangle[start as usize]);
    state.r_segs.rw_scale = scale_from_global_angle(state, angle1);
    state.r_bsp.drawsegs[state.r_bsp.ds_p].scale1 = state.r_segs.rw_scale;
    if stop > start {
        state.r_bsp.drawsegs[state.r_bsp.ds_p].scale2 = {
            let angle2 = state
                .r_main
                .viewangle
                .wrapping_add(state.r_main.xtoviewangle[stop as usize]);
            scale_from_global_angle(state, angle2)
        };
        state.r_segs.rw_scalestep = ((state.r_bsp.drawsegs[state.r_bsp.ds_p].scale2
            - state.r_segs.rw_scale)
            / (stop - start)) as Fixed;
        state.r_bsp.drawsegs[state.r_bsp.ds_p].scalestep = state.r_segs.rw_scalestep;
    } else {
        state.r_bsp.drawsegs[state.r_bsp.ds_p].scale2 =
            state.r_bsp.drawsegs[state.r_bsp.ds_p].scale1;
    }
    state.r_segs.worldtop = state
        .p_setup
        .sector_mut(state.r_bsp.frontsector.unwrap())
        .ceilingheight
        - state.r_main.viewz;
    state.r_segs.worldbottom = state
        .p_setup
        .sector_mut(state.r_bsp.frontsector.unwrap())
        .floorheight
        - state.r_main.viewz;
    state.r_segs.maskedtexture = false;
    state.r_segs.bottomtexture = state.r_segs.maskedtexture as i32;
    state.r_segs.toptexture = state.r_segs.bottomtexture;
    state.r_segs.midtexture = state.r_segs.toptexture;
    state.r_bsp.drawsegs[state.r_bsp.ds_p].maskedtexturecol = None;
    match state.r_bsp.backsector {
        None => {
            state.r_segs.midtexture = state.r_data.texturetranslation
                [state.p_setup.side_mut(state.r_bsp.sidedef).midtexture as usize];
            state.r_segs.markceiling = true;
            state.r_segs.markfloor = state.r_segs.markceiling;
            if state
                .p_setup
                .line_mut(state.r_bsp.linedef)
                .flags
                .contains(LineFlags::DONTPEGBOTTOM)
            {
                vtop = state
                    .p_setup
                    .sector_mut(state.r_bsp.frontsector.unwrap())
                    .floorheight
                    + state.r_data.textureheight
                        [state.p_setup.side_mut(state.r_bsp.sidedef).midtexture as usize];
                state.r_segs.rw_midtexturemid = vtop - state.r_main.viewz;
            } else {
                state.r_segs.rw_midtexturemid = state.r_segs.worldtop as Fixed;
            }
            state.r_segs.rw_midtexturemid += state.p_setup.side_mut(state.r_bsp.sidedef).rowoffset;
            state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette = SIL_BOTH;
            state.r_bsp.drawsegs[state.r_bsp.ds_p].sprtopclip = Some(ClipArray::ScreenHeightArray);
            state.r_bsp.drawsegs[state.r_bsp.ds_p].sprbottomclip = Some(ClipArray::NegOneArray);
            state.r_bsp.drawsegs[state.r_bsp.ds_p].bsilheight = INT_MAX as Fixed;
            state.r_bsp.drawsegs[state.r_bsp.ds_p].tsilheight = INT_MIN as Fixed;
        }
        Some(backsector) => {
            state.r_bsp.drawsegs[state.r_bsp.ds_p].sprbottomclip = None;
            state.r_bsp.drawsegs[state.r_bsp.ds_p].sprtopclip =
                state.r_bsp.drawsegs[state.r_bsp.ds_p].sprbottomclip;
            state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette = 0;
            if state
                .p_setup
                .sector_mut(state.r_bsp.frontsector.unwrap())
                .floorheight
                > state.p_setup.sector_mut(backsector).floorheight
            {
                state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette = SIL_BOTTOM;
                state.r_bsp.drawsegs[state.r_bsp.ds_p].bsilheight = state
                    .p_setup
                    .sector_mut(state.r_bsp.frontsector.unwrap())
                    .floorheight;
            } else if state.p_setup.sector_mut(backsector).floorheight > state.r_main.viewz {
                state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette = SIL_BOTTOM;
                state.r_bsp.drawsegs[state.r_bsp.ds_p].bsilheight = INT_MAX as Fixed;
            }
            if state
                .p_setup
                .sector_mut(state.r_bsp.frontsector.unwrap())
                .ceilingheight
                < state.p_setup.sector_mut(backsector).ceilingheight
            {
                state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette |= SIL_TOP;
                state.r_bsp.drawsegs[state.r_bsp.ds_p].tsilheight = state
                    .p_setup
                    .sector_mut(state.r_bsp.frontsector.unwrap())
                    .ceilingheight;
            } else if state.p_setup.sector_mut(backsector).ceilingheight < state.r_main.viewz {
                state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette |= SIL_TOP;
                state.r_bsp.drawsegs[state.r_bsp.ds_p].tsilheight = INT_MIN as Fixed;
            }
            if state.p_setup.sector_mut(backsector).ceilingheight
                <= state
                    .p_setup
                    .sector_mut(state.r_bsp.frontsector.unwrap())
                    .floorheight
            {
                state.r_bsp.drawsegs[state.r_bsp.ds_p].sprbottomclip = Some(ClipArray::NegOneArray);
                state.r_bsp.drawsegs[state.r_bsp.ds_p].bsilheight = INT_MAX as Fixed;
                state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette |= SIL_BOTTOM;
            }
            if state.p_setup.sector_mut(backsector).floorheight
                >= state
                    .p_setup
                    .sector_mut(state.r_bsp.frontsector.unwrap())
                    .ceilingheight
            {
                state.r_bsp.drawsegs[state.r_bsp.ds_p].sprtopclip =
                    Some(ClipArray::ScreenHeightArray);
                state.r_bsp.drawsegs[state.r_bsp.ds_p].tsilheight = INT_MIN as Fixed;
                state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette |= SIL_TOP;
            }
            state.r_segs.worldhigh =
                state.p_setup.sector_mut(backsector).ceilingheight - state.r_main.viewz;
            state.r_segs.worldlow =
                state.p_setup.sector_mut(backsector).floorheight - state.r_main.viewz;
            if state
                .p_setup
                .sector_mut(state.r_bsp.frontsector.unwrap())
                .ceilingpic as i32
                == state.r_sky.skyflatnum
                && state.p_setup.sector_mut(backsector).ceilingpic as i32 == state.r_sky.skyflatnum
            {
                state.r_segs.worldtop = state.r_segs.worldhigh;
            }
            state.r_segs.markfloor = state.r_segs.worldlow != state.r_segs.worldbottom
                || state.p_setup.sector_mut(backsector).floorpic as i32
                    != state
                        .p_setup
                        .sector_mut(state.r_bsp.frontsector.unwrap())
                        .floorpic as i32
                || state.p_setup.sector_mut(backsector).lightlevel as i32
                    != state
                        .p_setup
                        .sector_mut(state.r_bsp.frontsector.unwrap())
                        .lightlevel as i32;
            state.r_segs.markceiling = state.r_segs.worldhigh != state.r_segs.worldtop
                || state.p_setup.sector_mut(backsector).ceilingpic as i32
                    != state
                        .p_setup
                        .sector_mut(state.r_bsp.frontsector.unwrap())
                        .ceilingpic as i32
                || state.p_setup.sector_mut(backsector).lightlevel as i32
                    != state
                        .p_setup
                        .sector_mut(state.r_bsp.frontsector.unwrap())
                        .lightlevel as i32;
            if state.p_setup.sector_mut(backsector).ceilingheight
                <= state
                    .p_setup
                    .sector_mut(state.r_bsp.frontsector.unwrap())
                    .floorheight
                || state.p_setup.sector_mut(backsector).floorheight
                    >= state
                        .p_setup
                        .sector_mut(state.r_bsp.frontsector.unwrap())
                        .ceilingheight
            {
                state.r_segs.markfloor = true;
                state.r_segs.markceiling = state.r_segs.markfloor;
            }
            if state.r_segs.worldhigh < state.r_segs.worldtop {
                state.r_segs.toptexture = state.r_data.texturetranslation
                    [state.p_setup.side_mut(state.r_bsp.sidedef).toptexture as usize];
                if state
                    .p_setup
                    .line_mut(state.r_bsp.linedef)
                    .flags
                    .contains(LineFlags::DONTPEGTOP)
                {
                    state.r_segs.rw_toptexturemid = state.r_segs.worldtop as Fixed;
                } else {
                    vtop = state.p_setup.sector_mut(backsector).ceilingheight
                        + state.r_data.textureheight
                            [state.p_setup.side_mut(state.r_bsp.sidedef).toptexture as usize];
                    state.r_segs.rw_toptexturemid = vtop - state.r_main.viewz;
                }
            }
            if state.r_segs.worldlow > state.r_segs.worldbottom {
                state.r_segs.bottomtexture = state.r_data.texturetranslation
                    [state.p_setup.side_mut(state.r_bsp.sidedef).bottomtexture as usize];
                if state
                    .p_setup
                    .line_mut(state.r_bsp.linedef)
                    .flags
                    .contains(LineFlags::DONTPEGBOTTOM)
                {
                    state.r_segs.rw_bottomtexturemid = state.r_segs.worldtop as Fixed;
                } else {
                    state.r_segs.rw_bottomtexturemid = state.r_segs.worldlow as Fixed;
                }
            }
            state.r_segs.rw_toptexturemid += state.p_setup.side_mut(state.r_bsp.sidedef).rowoffset;
            state.r_segs.rw_bottomtexturemid +=
                state.p_setup.side_mut(state.r_bsp.sidedef).rowoffset;
            if state.p_setup.side_mut(state.r_bsp.sidedef).midtexture != 0 {
                state.r_segs.maskedtexture = true;
                state.r_segs.maskedtexturecol = Some(ClipArray::Openings(
                    state.r_plane.lastopening as isize - state.r_segs.rw_x as isize,
                ));
                state.r_bsp.drawsegs[state.r_bsp.ds_p].maskedtexturecol =
                    state.r_segs.maskedtexturecol;
                state.r_plane.lastopening += (state.r_segs.rw_stopx - state.r_segs.rw_x) as usize;
            }
        }
    }
    state.r_segs.segtextured =
        (state.r_segs.midtexture | state.r_segs.toptexture | state.r_segs.bottomtexture) != 0
            || state.r_segs.maskedtexture;
    if state.r_segs.segtextured {
        offsetangle = state
            .r_segs
            .rw_normalangle
            .wrapping_sub(state.r_segs.rw_angle1 as Angle);
        if offsetangle > ANG180 {
            offsetangle = offsetangle.wrapping_neg();
        }
        if offsetangle > ANG90 as Angle {
            offsetangle = ANG90 as Angle;
        }
        sineval = FINESINE[(offsetangle >> ANGLETOFINESHIFT) as usize];
        state.r_segs.rw_offset = fixed_mul(hyp, sineval);
        if state
            .r_segs
            .rw_normalangle
            .wrapping_sub(state.r_segs.rw_angle1 as Angle)
            < ANG180
        {
            state.r_segs.rw_offset = -state.r_segs.rw_offset;
        }
        state.r_segs.rw_offset += state.p_setup.side_mut(state.r_bsp.sidedef).textureoffset
            + state.p_setup.seg(state.r_bsp.curline).offset;
        state.r_segs.rw_centerangle = (ANG90 as Angle)
            .wrapping_add(state.r_main.viewangle)
            .wrapping_sub(state.r_segs.rw_normalangle);
        if state.r_main.fixedcolormap.is_none() {
            lightnum = (state
                .p_setup
                .sector_mut(state.r_bsp.frontsector.unwrap())
                .lightlevel as i32
                >> LIGHTSEGSHIFT)
                + state.r_main.extralight;
            let curline_v1 =
                state.p_setup.vertexes[state.p_setup.seg(state.r_bsp.curline).v1.0 as usize];
            let curline_v2 =
                state.p_setup.vertexes[state.p_setup.seg(state.r_bsp.curline).v2.0 as usize];
            if curline_v1.y == curline_v2.y {
                lightnum -= 1;
            } else if curline_v1.x == curline_v2.x {
                lightnum += 1;
            }
            if lightnum < 0 {
                state.r_segs.walllights = LightRow48::Normal(0);
            } else if lightnum >= LIGHTLEVELS {
                state.r_segs.walllights = LightRow48::Normal((LIGHTLEVELS - 1) as usize);
            } else {
                state.r_segs.walllights = LightRow48::Normal(lightnum as usize);
            }
        }
    }
    if state
        .p_setup
        .sector_mut(state.r_bsp.frontsector.unwrap())
        .floorheight
        >= state.r_main.viewz
    {
        state.r_segs.markfloor = false;
    }
    if state
        .p_setup
        .sector_mut(state.r_bsp.frontsector.unwrap())
        .ceilingheight
        <= state.r_main.viewz
        && state
            .p_setup
            .sector_mut(state.r_bsp.frontsector.unwrap())
            .ceilingpic as i32
            != state.r_sky.skyflatnum
    {
        state.r_segs.markceiling = false;
    }
    state.r_segs.worldtop >>= 4;
    state.r_segs.worldbottom >>= 4;
    state.r_segs.topstep = -fixed_mul(state.r_segs.rw_scalestep, state.r_segs.worldtop as Fixed);
    state.r_segs.topfrac = (state.r_main.centeryfrac >> 4)
        - fixed_mul(state.r_segs.worldtop as Fixed, state.r_segs.rw_scale);
    state.r_segs.bottomstep =
        -fixed_mul(state.r_segs.rw_scalestep, state.r_segs.worldbottom as Fixed);
    state.r_segs.bottomfrac = (state.r_main.centeryfrac >> 4)
        - fixed_mul(state.r_segs.worldbottom as Fixed, state.r_segs.rw_scale);
    if state.r_bsp.backsector.is_some() {
        state.r_segs.worldhigh >>= 4;
        state.r_segs.worldlow >>= 4;
        if state.r_segs.worldhigh < state.r_segs.worldtop {
            state.r_segs.pixhigh = (state.r_main.centeryfrac >> 4)
                - fixed_mul(state.r_segs.worldhigh as Fixed, state.r_segs.rw_scale);
            state.r_segs.pixhighstep =
                -fixed_mul(state.r_segs.rw_scalestep, state.r_segs.worldhigh as Fixed);
        }
        if state.r_segs.worldlow > state.r_segs.worldbottom {
            state.r_segs.pixlow = (state.r_main.centeryfrac >> 4)
                - fixed_mul(state.r_segs.worldlow as Fixed, state.r_segs.rw_scale);
            state.r_segs.pixlowstep =
                -fixed_mul(state.r_segs.rw_scalestep, state.r_segs.worldlow as Fixed);
        }
    }
    if state.r_segs.markceiling {
        let (ceilingplane, rw_x, rw_stopx_1) = (
            state.r_plane.ceilingplane.unwrap(),
            state.r_segs.rw_x,
            state.r_segs.rw_stopx - 1,
        );
        state.r_plane.ceilingplane = Some(check_plane(state, ceilingplane, rw_x, rw_stopx_1));
    }
    if state.r_segs.markfloor {
        let (floorplane, rw_x2, rw_stopx_2) = (
            state.r_plane.floorplane.unwrap(),
            state.r_segs.rw_x,
            state.r_segs.rw_stopx - 1,
        );
        state.r_plane.floorplane = Some(check_plane(state, floorplane, rw_x2, rw_stopx_2));
    }
    render_seg_loop(state);
    if (state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette & SIL_TOP != 0
        || state.r_segs.maskedtexture)
        && state.r_bsp.drawsegs[state.r_bsp.ds_p].sprtopclip.is_none()
    {
        let count = (state.r_segs.rw_stopx - start) as usize;
        let lastopening = state.r_plane.lastopening;
        state.r_plane.openings[lastopening..lastopening + count]
            .copy_from_slice(&state.r_plane.ceilingclip[start as usize..start as usize + count]);
        state.r_bsp.drawsegs[state.r_bsp.ds_p].sprtopclip = Some(ClipArray::Openings(
            state.r_plane.lastopening as isize - start as isize,
        ));
        state.r_plane.lastopening += (state.r_segs.rw_stopx - start) as usize;
    }
    if (state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette & SIL_BOTTOM != 0
        || state.r_segs.maskedtexture)
        && state.r_bsp.drawsegs[state.r_bsp.ds_p]
            .sprbottomclip
            .is_none()
    {
        let count = (state.r_segs.rw_stopx - start) as usize;
        let lastopening = state.r_plane.lastopening;
        state.r_plane.openings[lastopening..lastopening + count]
            .copy_from_slice(&state.r_plane.floorclip[start as usize..start as usize + count]);
        state.r_bsp.drawsegs[state.r_bsp.ds_p].sprbottomclip = Some(ClipArray::Openings(
            state.r_plane.lastopening as isize - start as isize,
        ));
        state.r_plane.lastopening += (state.r_segs.rw_stopx - start) as usize;
    }
    if state.r_segs.maskedtexture
        && state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette & SIL_TOP == 0
    {
        state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette |= SIL_TOP;
        state.r_bsp.drawsegs[state.r_bsp.ds_p].tsilheight = INT_MIN as Fixed;
    }
    if state.r_segs.maskedtexture
        && state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette & SIL_BOTTOM == 0
    {
        state.r_bsp.drawsegs[state.r_bsp.ds_p].silhouette |= SIL_BOTTOM;
        state.r_bsp.drawsegs[state.r_bsp.ds_p].bsilheight = INT_MAX as Fixed;
    }
    state.r_bsp.ds_p += 1;
}
pub const __SHRT_MAX__: i32 = 32767;
