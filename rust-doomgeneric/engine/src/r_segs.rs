use crate::src::game_state::GameState;
use crate::src::i_system::I_Error;
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FixedMul;
use crate::src::m_fixed::FRACBITS;
use crate::src::m_fixed::INT_MAX;
use crate::src::m_fixed::INT_MIN;
use crate::src::p_spec::ML_MAPPED;
use crate::src::r_data::column_t;
use crate::src::r_data::R_GetColumn;
use crate::src::r_defs::drawseg_t;
use crate::src::r_defs::lighttable_t;
use crate::src::r_main::R_PointToDist;
use crate::src::r_main::R_ScaleFromGlobalAngle;
use crate::src::r_main::LIGHTLEVELS;
use crate::src::r_main::LIGHTSCALESHIFT;
use crate::src::r_main::LIGHTSEGSHIFT;
use crate::src::r_main::MAXLIGHTSCALE;
use crate::src::r_plane::R_CheckPlane;
use crate::src::r_things::R_DrawMaskedColumn;
use crate::src::stdint_types::byte;
use crate::src::stdint_types::size_t;
use crate::src::tables::angle_t;
use crate::src::tables::finesine;
use crate::src::tables::finetangent;
use crate::src::tables::ANG180;
use crate::src::tables::ANG90;
use crate::src::tables::ANGLETOFINESHIFT;
use crate::src::mem_compat::memcpy;

pub struct RSegsState {
    pub segtextured: bool,
    pub markfloor: bool,
    pub markceiling: bool,
    pub maskedtexture: bool,
    pub toptexture: i32,
    pub bottomtexture: i32,
    pub midtexture: i32,
    pub rw_normalangle: angle_t,
    pub rw_angle1: i32,
    pub rw_x: i32,
    pub rw_stopx: i32,
    pub rw_centerangle: angle_t,
    pub rw_offset: fixed_t,
    pub rw_distance: fixed_t,
    pub rw_scale: fixed_t,
    pub rw_scalestep: fixed_t,
    pub rw_midtexturemid: fixed_t,
    pub rw_toptexturemid: fixed_t,
    pub rw_bottomtexturemid: fixed_t,
    pub worldtop: i32,
    pub worldbottom: i32,
    pub worldhigh: i32,
    pub worldlow: i32,
    pub pixhigh: fixed_t,
    pub pixlow: fixed_t,
    pub pixhighstep: fixed_t,
    pub pixlowstep: fixed_t,
    pub topfrac: fixed_t,
    pub topstep: fixed_t,
    pub bottomfrac: fixed_t,
    pub bottomstep: fixed_t,
    pub walllights: *mut *mut lighttable_t,
    pub maskedtexturecol: *mut i16,
}

impl RSegsState {
    pub const fn new() -> Self {
        RSegsState {
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
            walllights: ::core::ptr::null::<*mut lighttable_t>() as *mut *mut lighttable_t,
            maskedtexturecol: ::core::ptr::null::<i16>() as *mut i16,
        }
    }
}

pub const SHRT_MAX: i32 = __SHRT_MAX__;
pub const ML_DONTPEGTOP: i32 = 8;
pub const ML_DONTPEGBOTTOM: i32 = 16;
pub const SIL_BOTTOM: i32 = 1;
pub const SIL_TOP: i32 = 2;
pub const SIL_BOTH: i32 = 3;
pub const MAXDRAWSEGS: i32 = 256;
pub unsafe fn R_RenderMaskedSegRange(
    state: &mut GameState,
    mut ds: *mut drawseg_t,
    mut x1: i32,
    mut x2: i32,
) {
    let mut index: u32 = 0;
    let mut col: *mut column_t = ::core::ptr::null_mut::<column_t>();
    let mut lightnum: i32 = 0;
    let mut texnum: i32 = 0;
    state.r_bsp.curline = (*ds).curline;
    state.r_bsp.frontsector = state.p_setup.seg(state.r_bsp.curline).frontsector;
    state.r_bsp.backsector = state.p_setup.seg(state.r_bsp.curline).backsector;
    texnum = state.r_data.texturetranslation
        [(*state.p_setup.side_mut(state.p_setup.seg(state.r_bsp.curline).sidedef)).midtexture
            as usize];
    lightnum = ((*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).lightlevel as i32
        >> LIGHTSEGSHIFT)
        + state.r_main.extralight;
    let curline_v1 = state.p_setup.vertexes[state.p_setup.seg(state.r_bsp.curline).v1.0 as usize];
    let curline_v2 = state.p_setup.vertexes[state.p_setup.seg(state.r_bsp.curline).v2.0 as usize];
    if curline_v1.y == curline_v2.y {
        lightnum -= 1;
    } else if curline_v1.x == curline_v2.x {
        lightnum += 1;
    }
    if lightnum < 0 as i32 {
        state.r_segs.walllights = &raw mut *(&raw mut state.r_main.scalelight
            as *mut [*mut lighttable_t; 48])
            .offset(0 as i32 as isize) as *mut *mut lighttable_t;
    } else if lightnum >= LIGHTLEVELS {
        state.r_segs.walllights =
            &raw mut *(&raw mut state.r_main.scalelight as *mut [*mut lighttable_t; 48])
                .offset((LIGHTLEVELS - 1 as i32) as isize) as *mut *mut lighttable_t;
    } else {
        state.r_segs.walllights = &raw mut *(&raw mut state.r_main.scalelight
            as *mut [*mut lighttable_t; 48])
            .offset(lightnum as isize) as *mut *mut lighttable_t;
    }
    state.r_segs.maskedtexturecol = (*ds).maskedtexturecol;
    state.r_segs.rw_scalestep = (*ds).scalestep;
    state.r_things.spryscale =
        (*ds).scale1 + (x1 as fixed_t - (*ds).x1 as fixed_t) * state.r_segs.rw_scalestep;
    state.r_things.mfloorclip = (*ds).sprbottomclip;
    state.r_things.mceilingclip = (*ds).sprtopclip;
    if (*state.p_setup.line_mut(state.p_setup.seg(state.r_bsp.curline).linedef)).flags as i32
        & ML_DONTPEGBOTTOM
        != 0
    {
        state.r_draw.dc_texturemid =
            if (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).floorheight
                > (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).floorheight
            {
                (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).floorheight
            } else {
                (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).floorheight
            };
        state.r_draw.dc_texturemid = state.r_draw.dc_texturemid
            + state.r_data.textureheight[texnum as usize]
            - state.r_main.viewz;
    } else {
        state.r_draw.dc_texturemid =
            if (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingheight
                < (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).ceilingheight
            {
                (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingheight
            } else {
                (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).ceilingheight
            };
        state.r_draw.dc_texturemid = state.r_draw.dc_texturemid - state.r_main.viewz;
    }
    state.r_draw.dc_texturemid +=
        (*state.p_setup.side_mut(state.p_setup.seg(state.r_bsp.curline).sidedef)).rowoffset;
    if !state.r_main.fixedcolormap.is_null() {
        state.r_draw.dc_colormap = state.r_main.fixedcolormap;
    }
    state.r_draw.dc_x = x1;
    while state.r_draw.dc_x <= x2 {
        if *state
            .r_segs
            .maskedtexturecol
            .offset(state.r_draw.dc_x as isize) as i32
            != SHRT_MAX
        {
            if state.r_main.fixedcolormap.is_null() {
                index = (state.r_things.spryscale >> LIGHTSCALESHIFT) as u32;
                if index >= MAXLIGHTSCALE as u32 {
                    index = (MAXLIGHTSCALE - 1 as i32) as u32;
                }
                state.r_draw.dc_colormap = *state.r_segs.walllights.offset(index as isize);
            }
            state.r_things.sprtopscreen = state.r_main.centeryfrac
                - FixedMul(state.r_draw.dc_texturemid, state.r_things.spryscale);
            state.r_draw.dc_iscale =
                (0xffffffff as u32).wrapping_div(state.r_things.spryscale as u32) as fixed_t;
            col = R_GetColumn(
                state,
                texnum,
                *state
                    .r_segs
                    .maskedtexturecol
                    .offset(state.r_draw.dc_x as isize) as i32,
            )
            .offset(-(3 as i32 as isize)) as *mut column_t;
            R_DrawMaskedColumn(state, col);
            *state
                .r_segs
                .maskedtexturecol
                .offset(state.r_draw.dc_x as isize) = SHRT_MAX as i16;
        }
        state.r_things.spryscale += state.r_segs.rw_scalestep;
        state.r_draw.dc_x += 1;
    }
}
pub const HEIGHTBITS: i32 = 12;
pub const HEIGHTUNIT: i32 = (1 as i32) << HEIGHTBITS;
pub unsafe fn R_RenderSegLoop(state: &mut GameState) {
    let mut angle: angle_t = 0;
    let mut index: u32 = 0;
    let mut yl: i32 = 0;
    let mut yh: i32 = 0;
    let mut mid: i32 = 0;
    let mut texturecolumn: fixed_t = 0;
    let mut top: i32 = 0;
    let mut bottom: i32 = 0;
    while state.r_segs.rw_x < state.r_segs.rw_stopx {
        yl = state.r_segs.topfrac as i32 + HEIGHTUNIT - 1 as i32 >> HEIGHTBITS;
        if yl < state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 + 1 as i32 {
            yl = state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 + 1 as i32;
        }
        if state.r_segs.markceiling {
            top = state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 + 1 as i32;
            bottom = yl - 1 as i32;
            if bottom >= state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 {
                bottom = state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 - 1 as i32;
            }
            if top <= bottom {
                (*state.r_plane.ceilingplane).top[state.r_segs.rw_x as usize] = top as byte;
                (*state.r_plane.ceilingplane).bottom[state.r_segs.rw_x as usize] = bottom as byte;
            }
        }
        yh = (state.r_segs.bottomfrac >> HEIGHTBITS) as i32;
        if yh >= state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 {
            yh = state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 - 1 as i32;
        }
        if state.r_segs.markfloor {
            top = yh + 1 as i32;
            bottom = state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 - 1 as i32;
            if top <= state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 {
                top = state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 + 1 as i32;
            }
            if top <= bottom {
                (*state.r_plane.floorplane).top[state.r_segs.rw_x as usize] = top as byte;
                (*state.r_plane.floorplane).bottom[state.r_segs.rw_x as usize] = bottom as byte;
            }
        }
        if state.r_segs.segtextured {
            angle = state
                .r_segs
                .rw_centerangle
                .wrapping_add(state.r_main.xtoviewangle[state.r_segs.rw_x as usize])
                >> ANGLETOFINESHIFT;
            texturecolumn = state.r_segs.rw_offset
                - FixedMul(finetangent[angle as usize], state.r_segs.rw_distance);
            texturecolumn >>= FRACBITS;
            index = (state.r_segs.rw_scale >> LIGHTSCALESHIFT) as u32;
            if index >= MAXLIGHTSCALE as u32 {
                index = (MAXLIGHTSCALE - 1 as i32) as u32;
            }
            state.r_draw.dc_colormap = *state.r_segs.walllights.offset(index as isize);
            state.r_draw.dc_x = state.r_segs.rw_x;
            state.r_draw.dc_iscale =
                (0xffffffff as u32).wrapping_div(state.r_segs.rw_scale as u32) as fixed_t;
        } else {
            texturecolumn = 0 as i32 as fixed_t;
        }
        if state.r_segs.midtexture != 0 {
            state.r_draw.dc_yl = yl;
            state.r_draw.dc_yh = yh;
            state.r_draw.dc_texturemid = state.r_segs.rw_midtexturemid;
            state.r_draw.dc_source =
                R_GetColumn(state, state.r_segs.midtexture, texturecolumn as i32);
            state.r_main.colfunc.expect("non-null function pointer")(state);
            state.r_plane.ceilingclip[state.r_segs.rw_x as usize] = state.r_draw.viewheight as i16;
            state.r_plane.floorclip[state.r_segs.rw_x as usize] = -(1 as i32) as i16;
        } else {
            if state.r_segs.toptexture != 0 {
                mid = (state.r_segs.pixhigh >> HEIGHTBITS) as i32;
                state.r_segs.pixhigh += state.r_segs.pixhighstep;
                if mid >= state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 {
                    mid = state.r_plane.floorclip[state.r_segs.rw_x as usize] as i32 - 1 as i32;
                }
                if mid >= yl {
                    state.r_draw.dc_yl = yl;
                    state.r_draw.dc_yh = mid;
                    state.r_draw.dc_texturemid = state.r_segs.rw_toptexturemid;
                    state.r_draw.dc_source =
                        R_GetColumn(state, state.r_segs.toptexture, texturecolumn as i32);
                    state.r_main.colfunc.expect("non-null function pointer")(state);
                    state.r_plane.ceilingclip[state.r_segs.rw_x as usize] = mid as i16;
                } else {
                    state.r_plane.ceilingclip[state.r_segs.rw_x as usize] = (yl - 1 as i32) as i16;
                }
            } else if state.r_segs.markceiling {
                state.r_plane.ceilingclip[state.r_segs.rw_x as usize] = (yl - 1 as i32) as i16;
            }
            if state.r_segs.bottomtexture != 0 {
                mid = state.r_segs.pixlow as i32 + HEIGHTUNIT - 1 as i32 >> HEIGHTBITS;
                state.r_segs.pixlow += state.r_segs.pixlowstep;
                if mid <= state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 {
                    mid = state.r_plane.ceilingclip[state.r_segs.rw_x as usize] as i32 + 1 as i32;
                }
                if mid <= yh {
                    state.r_draw.dc_yl = mid;
                    state.r_draw.dc_yh = yh;
                    state.r_draw.dc_texturemid = state.r_segs.rw_bottomtexturemid;
                    state.r_draw.dc_source =
                        R_GetColumn(state, state.r_segs.bottomtexture, texturecolumn as i32);
                    state.r_main.colfunc.expect("non-null function pointer")(state);
                    state.r_plane.floorclip[state.r_segs.rw_x as usize] = mid as i16;
                } else {
                    state.r_plane.floorclip[state.r_segs.rw_x as usize] = (yh + 1 as i32) as i16;
                }
            } else if state.r_segs.markfloor {
                state.r_plane.floorclip[state.r_segs.rw_x as usize] = (yh + 1 as i32) as i16;
            }
            if state.r_segs.maskedtexture {
                *state
                    .r_segs
                    .maskedtexturecol
                    .offset(state.r_segs.rw_x as isize) = texturecolumn as i16;
            }
        }
        state.r_segs.rw_scale += state.r_segs.rw_scalestep;
        state.r_segs.topfrac += state.r_segs.topstep;
        state.r_segs.bottomfrac += state.r_segs.bottomstep;
        state.r_segs.rw_x += 1;
    }
}
pub unsafe fn R_StoreWallRange(state: &mut GameState, mut start: i32, mut stop: i32) {
    let mut hyp: fixed_t = 0;
    let mut sineval: fixed_t = 0;
    let mut distangle: angle_t = 0;
    let mut offsetangle: angle_t = 0;
    let mut vtop: fixed_t = 0;
    let mut lightnum: i32 = 0;
    if state.r_bsp.ds_p
        == (&raw mut state.r_bsp.drawsegs as *mut drawseg_t).offset(MAXDRAWSEGS as isize)
            as *mut drawseg_t
    {
        return;
    }
    if start >= state.r_draw.viewwidth || start > stop {
        I_Error(&format!("Bad R_RenderWallRange: {} to {}", start, stop));
    }
    state.r_bsp.sidedef = state.p_setup.seg(state.r_bsp.curline).sidedef;
    state.r_bsp.linedef = state.p_setup.seg(state.r_bsp.curline).linedef;
    (*state.p_setup.line_mut(state.r_bsp.linedef)).flags =
        ((*state.p_setup.line_mut(state.r_bsp.linedef)).flags as i32 | ML_MAPPED) as i16;
    state.r_segs.rw_normalangle = state
        .p_setup
        .seg(state.r_bsp.curline)
        .angle
        .wrapping_add(ANG90 as angle_t);
    offsetangle = (state
        .r_segs
        .rw_normalangle
        .wrapping_sub(state.r_segs.rw_angle1 as angle_t) as i32)
        .abs() as angle_t;
    if offsetangle > ANG90 as angle_t {
        offsetangle = ANG90 as angle_t;
    }
    distangle = (ANG90 as angle_t).wrapping_sub(offsetangle);
    let curline_v1 = state.p_setup.vertexes[state.p_setup.seg(state.r_bsp.curline).v1.0 as usize];
    let (v1x, v1y) = (curline_v1.x, curline_v1.y);
    hyp = R_PointToDist(state, v1x, v1y);
    sineval = finesine[(distangle >> ANGLETOFINESHIFT) as usize];
    state.r_segs.rw_distance = FixedMul(hyp, sineval);
    state.r_segs.rw_x = start;
    (*state.r_bsp.ds_p).x1 = state.r_segs.rw_x;
    (*state.r_bsp.ds_p).x2 = stop;
    (*state.r_bsp.ds_p).curline = state.r_bsp.curline;
    state.r_segs.rw_stopx = stop + 1 as i32;
    let angle1 = state
        .r_main
        .viewangle
        .wrapping_add(state.r_main.xtoviewangle[start as usize]);
    state.r_segs.rw_scale = R_ScaleFromGlobalAngle(state, angle1);
    (*state.r_bsp.ds_p).scale1 = state.r_segs.rw_scale;
    if stop > start {
        (*state.r_bsp.ds_p).scale2 = {
            let angle2 = state
                .r_main
                .viewangle
                .wrapping_add(state.r_main.xtoviewangle[stop as usize]);
            R_ScaleFromGlobalAngle(state, angle2)
        };
        state.r_segs.rw_scalestep = (((*state.r_bsp.ds_p).scale2 as i32
            - state.r_segs.rw_scale as i32)
            / (stop - start)) as fixed_t;
        (*state.r_bsp.ds_p).scalestep = state.r_segs.rw_scalestep;
    } else {
        (*state.r_bsp.ds_p).scale2 = (*state.r_bsp.ds_p).scale1;
    }
    state.r_segs.worldtop = ((*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap()))
        .ceilingheight
        - state.r_main.viewz) as i32;
    state.r_segs.worldbottom = ((*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap()))
        .floorheight
        - state.r_main.viewz) as i32;
    state.r_segs.maskedtexture = false;
    state.r_segs.bottomtexture = state.r_segs.maskedtexture as i32;
    state.r_segs.toptexture = state.r_segs.bottomtexture;
    state.r_segs.midtexture = state.r_segs.toptexture;
    (*state.r_bsp.ds_p).maskedtexturecol = ::core::ptr::null_mut::<i16>();
    if state.r_bsp.backsector.is_none() {
        state.r_segs.midtexture = state.r_data.texturetranslation
            [(*state.p_setup.side_mut(state.r_bsp.sidedef)).midtexture as usize];
        state.r_segs.markceiling = true;
        state.r_segs.markfloor = state.r_segs.markceiling;
        if (*state.p_setup.line_mut(state.r_bsp.linedef)).flags as i32 & ML_DONTPEGBOTTOM != 0 {
            vtop = (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).floorheight
                + state.r_data.textureheight
                    [(*state.p_setup.side_mut(state.r_bsp.sidedef)).midtexture as usize];
            state.r_segs.rw_midtexturemid = vtop - state.r_main.viewz;
        } else {
            state.r_segs.rw_midtexturemid = state.r_segs.worldtop as fixed_t;
        }
        state.r_segs.rw_midtexturemid += (*state.p_setup.side_mut(state.r_bsp.sidedef)).rowoffset;
        (*state.r_bsp.ds_p).silhouette = SIL_BOTH;
        (*state.r_bsp.ds_p).sprtopclip = &raw mut state.r_things.screenheightarray as *mut i16;
        (*state.r_bsp.ds_p).sprbottomclip = &raw mut state.r_things.negonearray as *mut i16;
        (*state.r_bsp.ds_p).bsilheight = INT_MAX as fixed_t;
        (*state.r_bsp.ds_p).tsilheight = INT_MIN as fixed_t;
    } else {
        (*state.r_bsp.ds_p).sprbottomclip = ::core::ptr::null_mut::<i16>();
        (*state.r_bsp.ds_p).sprtopclip = (*state.r_bsp.ds_p).sprbottomclip;
        (*state.r_bsp.ds_p).silhouette = 0 as i32;
        if (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).floorheight
            > (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).floorheight
        {
            (*state.r_bsp.ds_p).silhouette = SIL_BOTTOM;
            (*state.r_bsp.ds_p).bsilheight =
                (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).floorheight;
        } else if (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).floorheight
            > state.r_main.viewz
        {
            (*state.r_bsp.ds_p).silhouette = SIL_BOTTOM;
            (*state.r_bsp.ds_p).bsilheight = INT_MAX as fixed_t;
        }
        if (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingheight
            < (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).ceilingheight
        {
            (*state.r_bsp.ds_p).silhouette |= SIL_TOP;
            (*state.r_bsp.ds_p).tsilheight =
                (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingheight;
        } else if (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).ceilingheight
            < state.r_main.viewz
        {
            (*state.r_bsp.ds_p).silhouette |= SIL_TOP;
            (*state.r_bsp.ds_p).tsilheight = INT_MIN as fixed_t;
        }
        if (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).ceilingheight
            <= (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).floorheight
        {
            (*state.r_bsp.ds_p).sprbottomclip = &raw mut state.r_things.negonearray as *mut i16;
            (*state.r_bsp.ds_p).bsilheight = INT_MAX as fixed_t;
            (*state.r_bsp.ds_p).silhouette |= SIL_BOTTOM;
        }
        if (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).floorheight
            >= (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingheight
        {
            (*state.r_bsp.ds_p).sprtopclip = &raw mut state.r_things.screenheightarray as *mut i16;
            (*state.r_bsp.ds_p).tsilheight = INT_MIN as fixed_t;
            (*state.r_bsp.ds_p).silhouette |= SIL_TOP;
        }
        state.r_segs.worldhigh = ((*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap()))
            .ceilingheight
            - state.r_main.viewz) as i32;
        state.r_segs.worldlow = ((*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap()))
            .floorheight
            - state.r_main.viewz) as i32;
        if (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingpic as i32
            == state.r_sky.skyflatnum
            && (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).ceilingpic as i32
                == state.r_sky.skyflatnum
        {
            state.r_segs.worldtop = state.r_segs.worldhigh;
        }
        if state.r_segs.worldlow != state.r_segs.worldbottom
            || (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).floorpic as i32
                != (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).floorpic as i32
            || (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).lightlevel as i32
                != (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).lightlevel as i32
        {
            state.r_segs.markfloor = true;
        } else {
            state.r_segs.markfloor = false;
        }
        if state.r_segs.worldhigh != state.r_segs.worldtop
            || (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).ceilingpic as i32
                != (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingpic as i32
            || (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).lightlevel as i32
                != (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).lightlevel as i32
        {
            state.r_segs.markceiling = true;
        } else {
            state.r_segs.markceiling = false;
        }
        if (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).ceilingheight
            <= (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).floorheight
            || (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).floorheight
                >= (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingheight
        {
            state.r_segs.markfloor = true;
            state.r_segs.markceiling = state.r_segs.markfloor;
        }
        if state.r_segs.worldhigh < state.r_segs.worldtop {
            state.r_segs.toptexture = state.r_data.texturetranslation
                [(*state.p_setup.side_mut(state.r_bsp.sidedef)).toptexture as usize];
            if (*state.p_setup.line_mut(state.r_bsp.linedef)).flags as i32 & ML_DONTPEGTOP != 0 {
                state.r_segs.rw_toptexturemid = state.r_segs.worldtop as fixed_t;
            } else {
                vtop = (*state.p_setup.sector_mut(state.r_bsp.backsector.unwrap())).ceilingheight
                    + state.r_data.textureheight
                        [(*state.p_setup.side_mut(state.r_bsp.sidedef)).toptexture as usize];
                state.r_segs.rw_toptexturemid = vtop - state.r_main.viewz;
            }
        }
        if state.r_segs.worldlow > state.r_segs.worldbottom {
            state.r_segs.bottomtexture = state.r_data.texturetranslation
                [(*state.p_setup.side_mut(state.r_bsp.sidedef)).bottomtexture as usize];
            if (*state.p_setup.line_mut(state.r_bsp.linedef)).flags as i32 & ML_DONTPEGBOTTOM != 0 {
                state.r_segs.rw_bottomtexturemid = state.r_segs.worldtop as fixed_t;
            } else {
                state.r_segs.rw_bottomtexturemid = state.r_segs.worldlow as fixed_t;
            }
        }
        state.r_segs.rw_toptexturemid += (*state.p_setup.side_mut(state.r_bsp.sidedef)).rowoffset;
        state.r_segs.rw_bottomtexturemid +=
            (*state.p_setup.side_mut(state.r_bsp.sidedef)).rowoffset;
        if (*state.p_setup.side_mut(state.r_bsp.sidedef)).midtexture != 0 {
            state.r_segs.maskedtexture = true;
            state.r_segs.maskedtexturecol = state
                .r_plane
                .lastopening
                .offset(-(state.r_segs.rw_x as isize));
            (*state.r_bsp.ds_p).maskedtexturecol = state.r_segs.maskedtexturecol;
            state.r_plane.lastopening = state
                .r_plane
                .lastopening
                .offset((state.r_segs.rw_stopx - state.r_segs.rw_x) as isize);
        }
    }
    state.r_segs.segtextured =
        (state.r_segs.midtexture | state.r_segs.toptexture | state.r_segs.bottomtexture) != 0
            || state.r_segs.maskedtexture;
    if state.r_segs.segtextured {
        offsetangle = state
            .r_segs
            .rw_normalangle
            .wrapping_sub(state.r_segs.rw_angle1 as angle_t);
        if offsetangle > ANG180 {
            offsetangle = offsetangle.wrapping_neg();
        }
        if offsetangle > ANG90 as angle_t {
            offsetangle = ANG90 as angle_t;
        }
        sineval = finesine[(offsetangle >> ANGLETOFINESHIFT) as usize];
        state.r_segs.rw_offset = FixedMul(hyp, sineval);
        if state
            .r_segs
            .rw_normalangle
            .wrapping_sub(state.r_segs.rw_angle1 as angle_t)
            < ANG180
        {
            state.r_segs.rw_offset = -state.r_segs.rw_offset;
        }
        state.r_segs.rw_offset += (*state.p_setup.side_mut(state.r_bsp.sidedef)).textureoffset
            + state.p_setup.seg(state.r_bsp.curline).offset;
        state.r_segs.rw_centerangle = (ANG90 as angle_t)
            .wrapping_add(state.r_main.viewangle)
            .wrapping_sub(state.r_segs.rw_normalangle);
        if state.r_main.fixedcolormap.is_null() {
            lightnum = ((*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).lightlevel
                as i32
                >> LIGHTSEGSHIFT)
                + state.r_main.extralight;
            let curline_v1 = state.p_setup.vertexes[state.p_setup.seg(state.r_bsp.curline).v1.0 as usize];
            let curline_v2 = state.p_setup.vertexes[state.p_setup.seg(state.r_bsp.curline).v2.0 as usize];
            if curline_v1.y == curline_v2.y {
                lightnum -= 1;
            } else if curline_v1.x == curline_v2.x {
                lightnum += 1;
            }
            if lightnum < 0 as i32 {
                state.r_segs.walllights =
                    &raw mut *(&raw mut state.r_main.scalelight as *mut [*mut lighttable_t; 48])
                        .offset(0 as i32 as isize) as *mut *mut lighttable_t;
            } else if lightnum >= LIGHTLEVELS {
                state.r_segs.walllights = &raw mut *(&raw mut state.r_main.scalelight
                    as *mut [*mut lighttable_t; 48])
                    .offset((LIGHTLEVELS - 1 as i32) as isize)
                    as *mut *mut lighttable_t;
            } else {
                state.r_segs.walllights =
                    &raw mut *(&raw mut state.r_main.scalelight as *mut [*mut lighttable_t; 48])
                        .offset(lightnum as isize) as *mut *mut lighttable_t;
            }
        }
    }
    if (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).floorheight
        >= state.r_main.viewz
    {
        state.r_segs.markfloor = false;
    }
    if (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingheight
        <= state.r_main.viewz
        && (*state.p_setup.sector_mut(state.r_bsp.frontsector.unwrap())).ceilingpic as i32
            != state.r_sky.skyflatnum
    {
        state.r_segs.markceiling = false;
    }
    state.r_segs.worldtop >>= 4 as i32;
    state.r_segs.worldbottom >>= 4 as i32;
    state.r_segs.topstep = -FixedMul(state.r_segs.rw_scalestep, state.r_segs.worldtop as fixed_t);
    state.r_segs.topfrac = (state.r_main.centeryfrac >> 4 as i32)
        - FixedMul(state.r_segs.worldtop as fixed_t, state.r_segs.rw_scale);
    state.r_segs.bottomstep = -FixedMul(
        state.r_segs.rw_scalestep,
        state.r_segs.worldbottom as fixed_t,
    );
    state.r_segs.bottomfrac = (state.r_main.centeryfrac >> 4 as i32)
        - FixedMul(state.r_segs.worldbottom as fixed_t, state.r_segs.rw_scale);
    if !state.r_bsp.backsector.is_none() {
        state.r_segs.worldhigh >>= 4 as i32;
        state.r_segs.worldlow >>= 4 as i32;
        if state.r_segs.worldhigh < state.r_segs.worldtop {
            state.r_segs.pixhigh = (state.r_main.centeryfrac >> 4 as i32)
                - FixedMul(state.r_segs.worldhigh as fixed_t, state.r_segs.rw_scale);
            state.r_segs.pixhighstep =
                -FixedMul(state.r_segs.rw_scalestep, state.r_segs.worldhigh as fixed_t);
        }
        if state.r_segs.worldlow > state.r_segs.worldbottom {
            state.r_segs.pixlow = (state.r_main.centeryfrac >> 4 as i32)
                - FixedMul(state.r_segs.worldlow as fixed_t, state.r_segs.rw_scale);
            state.r_segs.pixlowstep =
                -FixedMul(state.r_segs.rw_scalestep, state.r_segs.worldlow as fixed_t);
        }
    }
    if state.r_segs.markceiling {
        let (ceilingplane, rw_x, rw_stopx_1) = (
            state.r_plane.ceilingplane,
            state.r_segs.rw_x,
            state.r_segs.rw_stopx - 1 as i32,
        );
        state.r_plane.ceilingplane = R_CheckPlane(state, ceilingplane, rw_x, rw_stopx_1);
    }
    if state.r_segs.markfloor {
        let (floorplane, rw_x2, rw_stopx_2) = (
            state.r_plane.floorplane,
            state.r_segs.rw_x,
            state.r_segs.rw_stopx - 1 as i32,
        );
        state.r_plane.floorplane = R_CheckPlane(state, floorplane, rw_x2, rw_stopx_2);
    }
    R_RenderSegLoop(state);
    if ((*state.r_bsp.ds_p).silhouette & SIL_TOP != 0 || state.r_segs.maskedtexture)
        && (*state.r_bsp.ds_p).sprtopclip.is_null()
    {
        memcpy(
            state.r_plane.lastopening as *mut ::core::ffi::c_void,
            (&raw mut state.r_plane.ceilingclip as *mut i16).offset(start as isize)
                as *const ::core::ffi::c_void,
            (2 as i32 * (state.r_segs.rw_stopx - start)) as size_t,
        );
        (*state.r_bsp.ds_p).sprtopclip = state.r_plane.lastopening.offset(-(start as isize));
        state.r_plane.lastopening = state
            .r_plane
            .lastopening
            .offset((state.r_segs.rw_stopx - start) as isize);
    }
    if ((*state.r_bsp.ds_p).silhouette & SIL_BOTTOM != 0 || state.r_segs.maskedtexture)
        && (*state.r_bsp.ds_p).sprbottomclip.is_null()
    {
        memcpy(
            state.r_plane.lastopening as *mut ::core::ffi::c_void,
            (&raw mut state.r_plane.floorclip as *mut i16).offset(start as isize)
                as *const ::core::ffi::c_void,
            (2 as i32 * (state.r_segs.rw_stopx - start)) as size_t,
        );
        (*state.r_bsp.ds_p).sprbottomclip = state.r_plane.lastopening.offset(-(start as isize));
        state.r_plane.lastopening = state
            .r_plane
            .lastopening
            .offset((state.r_segs.rw_stopx - start) as isize);
    }
    if state.r_segs.maskedtexture && (*state.r_bsp.ds_p).silhouette & SIL_TOP == 0 {
        (*state.r_bsp.ds_p).silhouette |= SIL_TOP;
        (*state.r_bsp.ds_p).tsilheight = INT_MIN as fixed_t;
    }
    if state.r_segs.maskedtexture && (*state.r_bsp.ds_p).silhouette & SIL_BOTTOM == 0 {
        (*state.r_bsp.ds_p).silhouette |= SIL_BOTTOM;
        (*state.r_bsp.ds_p).bsilheight = INT_MAX as fixed_t;
    }
    state.r_bsp.ds_p = state.r_bsp.ds_p.offset(1);
}
pub const __SHRT_MAX__: i32 = 32767;
