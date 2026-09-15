use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedDiv;
use crate::m_fixed::FixedMul;
use crate::r_data::R_GetColumn;
use crate::r_defs::visplane_t;
use crate::r_main::LIGHTLEVELS;
use crate::r_main::LIGHTSEGSHIFT;
use crate::r_main::LIGHTZSHIFT;
use crate::r_main::MAXLIGHTZ;
use crate::r_segs::MAXDRAWSEGS;
use crate::stdint_types::byte;
use crate::stdint_types::size_t;
use crate::tables::angle_t;
use crate::tables::finecosine;
use crate::tables::finesine;
use crate::tables::ANG90;
use crate::tables::ANGLETOFINESHIFT;
use crate::w_wad::W_CacheLumpNum;
use crate::w_wad::W_ReleaseLumpNum;
use crate::mem_compat::memset;

pub struct RPlaneState {
    pub floorfunc: planefunction_t,
    pub ceilingfunc: planefunction_t,
    pub visplanes: [visplane_t; 128],
    pub lastvisplane: usize,
    pub floorplane: Option<usize>,
    pub ceilingplane: Option<usize>,
    pub openings: [i16; 20480],
    pub lastopening: usize,
    pub floorclip: [i16; 320],
    pub ceilingclip: [i16; 320],
    pub spanstart: [i32; 200],
    pub spanstop: [i32; 200],
    pub planezlight: usize,
    pub planeheight: fixed_t,
    pub yslope: [fixed_t; 200],
    pub distscale: [fixed_t; 320],
    pub basexscale: fixed_t,
    pub baseyscale: fixed_t,
    pub cachedheight: [fixed_t; 200],
    pub cacheddistance: [fixed_t; 200],
    pub cachedxstep: [fixed_t; 200],
    pub cachedystep: [fixed_t; 200],
}

impl RPlaneState {
    pub const fn new() -> Self {
        RPlaneState {
            floorfunc: None,
            ceilingfunc: None,
            visplanes: [visplane_t {
                height: 0,
                picnum: 0,
                lightlevel: 0,
                minx: 0,
                maxx: 0,
                pad1: 0,
                top: [0; 320],
                pad2: 0,
                pad3: 0,
                bottom: [0; 320],
                pad4: 0,
            }; 128],
            lastvisplane: 0,
            floorplane: None,
            ceilingplane: None,
            openings: [0; 20480],
            lastopening: 0,
            floorclip: [0; 320],
            ceilingclip: [0; 320],
            spanstart: [0; 200],
            spanstop: [0; 200],
            planezlight: 0,
            planeheight: 0,
            yslope: [0; 200],
            distscale: [0; 320],
            basexscale: 0,
            baseyscale: 0,
            cachedheight: [0; 200],
            cacheddistance: [0; 200],
            cachedxstep: [0; 200],
            cachedystep: [0; 200],
        }
    }
}

pub type planefunction_t = Option<unsafe fn(i32, i32) -> ()>;
pub const ANGLETOSKYSHIFT: i32 = 22;
pub const MAXVISPLANES: i32 = 128;
pub unsafe fn R_MapPlane(state: &mut GameState, mut y: i32, mut x1: i32, mut x2: i32) {
    let mut angle: angle_t = 0;
    let mut distance: fixed_t = 0;
    let mut length: fixed_t = 0;
    let mut index: u32 = 0;
    if x2 < x1 || x1 < 0 as i32 || x2 >= state.r_draw.viewwidth || y > state.r_draw.viewheight {
        I_Error(&format!("R_MapPlane: {}, {} at {}", x1, x2, y));
    }
    if state.r_plane.planeheight != state.r_plane.cachedheight[y as usize] {
        state.r_plane.cachedheight[y as usize] = state.r_plane.planeheight;
        state.r_plane.cacheddistance[y as usize] =
            FixedMul(state.r_plane.planeheight, state.r_plane.yslope[y as usize]);
        distance = state.r_plane.cacheddistance[y as usize];
        state.r_plane.cachedxstep[y as usize] = FixedMul(distance, state.r_plane.basexscale);
        state.r_draw.ds_xstep = state.r_plane.cachedxstep[y as usize];
        state.r_plane.cachedystep[y as usize] = FixedMul(distance, state.r_plane.baseyscale);
        state.r_draw.ds_ystep = state.r_plane.cachedystep[y as usize];
    } else {
        distance = state.r_plane.cacheddistance[y as usize];
        state.r_draw.ds_xstep = state.r_plane.cachedxstep[y as usize];
        state.r_draw.ds_ystep = state.r_plane.cachedystep[y as usize];
    }
    length = FixedMul(distance, state.r_plane.distscale[x1 as usize]);
    angle = state
        .r_main
        .viewangle
        .wrapping_add(state.r_main.xtoviewangle[x1 as usize])
        >> ANGLETOFINESHIFT;
    state.r_draw.ds_xfrac = state.r_main.viewx + FixedMul(finecosine[angle as isize], length);
    state.r_draw.ds_yfrac = -state.r_main.viewy - FixedMul(finesine[angle as usize], length);
    if let Some(colormap) = state.r_main.fixedcolormap {
        state.r_draw.ds_colormap = colormap;
    } else {
        index = (distance >> LIGHTZSHIFT) as u32;
        if index >= MAXLIGHTZ as u32 {
            index = (MAXLIGHTZ - 1 as i32) as u32;
        }
        state.r_draw.ds_colormap = state.r_main.zlight[state.r_plane.planezlight][index as usize];
    }
    state.r_draw.ds_y = y;
    state.r_draw.ds_x1 = x1;
    state.r_draw.ds_x2 = x2;
    state.r_main.spanfunc.expect("non-null function pointer")(state);
}
pub unsafe fn R_ClearPlanes(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut angle: angle_t = 0;
    i = 0 as i32;
    while i < state.r_draw.viewwidth {
        state.r_plane.floorclip[i as usize] = state.r_draw.viewheight as i16;
        state.r_plane.ceilingclip[i as usize] = -(1 as i32) as i16;
        i += 1;
    }
    state.r_plane.lastvisplane = 0;
    state.r_plane.lastopening = 0;
    memset(
        &raw mut state.r_plane.cachedheight as *mut fixed_t as *mut ::core::ffi::c_void,
        0 as i32,
        ::core::mem::size_of::<[fixed_t; 200]>() as size_t,
    );
    angle = state.r_main.viewangle.wrapping_sub(ANG90 as angle_t) >> ANGLETOFINESHIFT;
    state.r_plane.basexscale = FixedDiv(finecosine[angle as isize], state.r_main.centerxfrac);
    state.r_plane.baseyscale = -FixedDiv(finesine[angle as usize], state.r_main.centerxfrac);
}
pub unsafe fn R_FindPlane(
    state: &mut GameState,
    mut height: fixed_t,
    mut picnum: i32,
    mut lightlevel: i32,
) -> usize {
    let mut check: usize = 0;
    if picnum == state.r_sky.skyflatnum {
        height = 0 as i32 as fixed_t;
        lightlevel = 0 as i32;
    }
    while check < state.r_plane.lastvisplane {
        let pl = state.r_plane.visplanes[check];
        if height == pl.height && picnum == pl.picnum && lightlevel == pl.lightlevel {
            break;
        }
        check += 1;
    }
    if check < state.r_plane.lastvisplane {
        return check;
    }
    if state.r_plane.lastvisplane == MAXVISPLANES as usize {
        I_Error("R_FindPlane: no more visplanes");
    }
    state.r_plane.lastvisplane += 1;
    let pl = &raw mut state.r_plane.visplanes[check];
    (*pl).height = height;
    (*pl).picnum = picnum;
    (*pl).lightlevel = lightlevel;
    (*pl).minx = SCREENWIDTH;
    (*pl).maxx = -(1 as i32);
    memset(
        &raw mut (*pl).top as *mut byte as *mut ::core::ffi::c_void,
        0xff as i32,
        ::core::mem::size_of::<[byte; 320]>() as size_t,
    );
    return check;
}
pub unsafe fn R_CheckPlane(
    state: &mut GameState,
    mut pl: usize,
    mut start: i32,
    mut stop: i32,
) -> usize {
    let mut intrl: i32 = 0;
    let mut intrh: i32 = 0;
    let mut unionl: i32 = 0;
    let mut unionh: i32 = 0;
    let mut x: i32 = 0;
    let plv = &raw mut state.r_plane.visplanes[pl];
    if start < (*plv).minx {
        intrl = (*plv).minx;
        unionl = start;
    } else {
        unionl = (*plv).minx;
        intrl = start;
    }
    if stop > (*plv).maxx {
        intrh = (*plv).maxx;
        unionh = stop;
    } else {
        unionh = (*plv).maxx;
        intrh = stop;
    }
    x = intrl;
    while x <= intrh {
        if (*plv).top[x as usize] as i32 != 0xff as i32 {
            break;
        }
        x += 1;
    }
    if x > intrh {
        (*plv).minx = unionl;
        (*plv).maxx = unionh;
        return pl;
    }
    let (height, picnum, lightlevel) = ((*plv).height, (*plv).picnum, (*plv).lightlevel);
    let fresh0 = state.r_plane.lastvisplane;
    state.r_plane.visplanes[fresh0].height = height;
    state.r_plane.visplanes[fresh0].picnum = picnum;
    state.r_plane.visplanes[fresh0].lightlevel = lightlevel;
    state.r_plane.lastvisplane += 1;
    pl = fresh0;
    let plv = &raw mut state.r_plane.visplanes[pl];
    (*plv).minx = start;
    (*plv).maxx = stop;
    memset(
        &raw mut (*plv).top as *mut byte as *mut ::core::ffi::c_void,
        0xff as i32,
        ::core::mem::size_of::<[byte; 320]>() as size_t,
    );
    return pl;
}
pub fn R_MakeSpans(
    state: &mut GameState,
    mut x: i32,
    mut t1: i32,
    mut b1: i32,
    mut t2: i32,
    mut b2: i32,
) {
    while t1 < t2 && t1 <= b1 {
        let spanstart_t1 = state.r_plane.spanstart[t1 as usize];
        unsafe { R_MapPlane(state, t1, spanstart_t1, x - 1 as i32) };
        t1 += 1;
    }
    while b1 > b2 && b1 >= t1 {
        let spanstart_b1 = state.r_plane.spanstart[b1 as usize];
        unsafe { R_MapPlane(state, b1, spanstart_b1, x - 1 as i32) };
        b1 -= 1;
    }
    while t2 < t1 && t2 <= b2 {
        state.r_plane.spanstart[t2 as usize] = x;
        t2 += 1;
    }
    while b2 > b1 && b2 >= t2 {
        state.r_plane.spanstart[b2 as usize] = x;
        b2 -= 1;
    }
}
pub unsafe fn R_DrawPlanes(state: &mut GameState) {
    let mut pl: usize = 0;
    let mut light: i32 = 0;
    let mut x: i32 = 0;
    let mut stop: i32 = 0;
    let mut angle: i32 = 0;
    let mut lumpnum: i32 = 0;
    if state.r_bsp.ds_p as i64 > MAXDRAWSEGS as i64 {
        I_Error(&format!(
            "R_DrawPlanes: drawsegs overflow ({})",
            state.r_bsp.ds_p as i64,
        ));
    }
    if state.r_plane.lastvisplane as i64 > MAXVISPLANES as i64 {
        I_Error(&format!(
            "R_DrawPlanes: visplane overflow ({})",
            state.r_plane.lastvisplane as i64,
        ));
    }
    if state.r_plane.lastopening as i64 > (SCREENWIDTH * 64 as i32) as i64 {
        I_Error(&format!(
            "R_DrawPlanes: opening overflow ({})",
            state.r_plane.lastopening as i64,
        ));
    }
    pl = 0;
    while pl < state.r_plane.lastvisplane {
        let plv = &raw mut state.r_plane.visplanes[pl];
        if !((*plv).minx > (*plv).maxx) {
            if (*plv).picnum == state.r_sky.skyflatnum {
                state.r_draw.dc_iscale = state.r_things.pspriteiscale >> state.r_main.detailshift;
                state.r_draw.dc_colormap = Some(0);
                state.r_draw.dc_texturemid = state.r_sky.skytexturemid as fixed_t;
                x = (*plv).minx;
                while x <= (*plv).maxx {
                    state.r_draw.dc_yl = (*plv).top[x as usize] as i32;
                    state.r_draw.dc_yh = (*plv).bottom[x as usize] as i32;
                    if state.r_draw.dc_yl <= state.r_draw.dc_yh {
                        angle = (state
                            .r_main
                            .viewangle
                            .wrapping_add(state.r_main.xtoviewangle[x as usize])
                            >> ANGLETOSKYSHIFT) as i32;
                        state.r_draw.dc_x = x;
                        state.r_draw.dc_source = R_GetColumn(state, state.r_sky.skytexture, angle);
                        state.r_main.colfunc.expect("non-null function pointer")(state);
                    }
                    x += 1;
                }
            } else {
                lumpnum = state.r_data.firstflat
                    + state.r_data.flattranslation[(*plv).picnum as usize];
                state.r_draw.ds_source = W_CacheLumpNum(state, lumpnum) as *mut byte;
                state.r_plane.planeheight =
                    ((*plv).height as i32 - state.r_main.viewz as i32).abs() as fixed_t;
                light = ((*plv).lightlevel >> LIGHTSEGSHIFT) + state.r_main.extralight;
                if light >= LIGHTLEVELS {
                    light = LIGHTLEVELS - 1 as i32;
                }
                if light < 0 as i32 {
                    light = 0 as i32;
                }
                state.r_plane.planezlight = light as usize;
                *(&raw mut (*plv).top as *mut byte).offset(((*plv).maxx + 1 as i32) as isize) =
                    0xff as byte;
                *(&raw mut (*plv).top as *mut byte).offset(((*plv).minx - 1 as i32) as isize) =
                    0xff as byte;
                stop = (*plv).maxx + 1 as i32;
                x = (*plv).minx;
                while x <= stop {
                    R_MakeSpans(
                        state,
                        x,
                        *(&raw const (*plv).top as *const byte).offset((x - 1 as i32) as isize)
                            as i32,
                        *(&raw const (*plv).bottom as *const byte).offset((x - 1 as i32) as isize)
                            as i32,
                        *(&raw const (*plv).top as *const byte).offset(x as isize) as i32,
                        *(&raw const (*plv).bottom as *const byte).offset(x as isize) as i32,
                    );
                    x += 1;
                }
                W_ReleaseLumpNum(&mut state.w_wad, lumpnum);
            }
        }
        pl += 1;
    }
}
