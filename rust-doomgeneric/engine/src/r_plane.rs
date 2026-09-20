use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::m_fixed::fixed_div;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::r_draw::RDrawState;
use crate::r_main::RMainState;
use crate::r_sky::RSkyState;

use crate::r_data::get_column;
use crate::r_defs::VisPlane;
use crate::r_draw::ColumnSource;
use crate::r_main::LIGHTLEVELS;
use crate::r_main::LIGHTSEGSHIFT;
use crate::r_main::LIGHTZSHIFT;
use crate::r_main::MAXLIGHTZ;
use crate::r_segs::MAXDRAWSEGS;

use crate::tables::Angle;
use crate::tables::ANG90;
use crate::tables::ANGLETOFINESHIFT;
use crate::tables::FINECOSINE;
use crate::tables::FINESINE;
use crate::w_wad::lump_bytes;
use crate::w_wad::release_lump_num;

pub struct RPlaneState {
    pub visplanes: [VisPlane; 128],
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
    pub planeheight: Fixed,
    pub yslope: [Fixed; 200],
    pub distscale: [Fixed; 320],
    pub basexscale: Fixed,
    pub baseyscale: Fixed,
    pub cachedheight: [Fixed; 200],
    pub cacheddistance: [Fixed; 200],
    pub cachedxstep: [Fixed; 200],
    pub cachedystep: [Fixed; 200],
}

impl Default for RPlaneState {
    fn default() -> Self {
        Self::new()
    }
}

impl RPlaneState {
    pub const fn new() -> Self {
        Self {
            visplanes: [VisPlane::EMPTY; 128],
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

pub const ANGLETOSKYSHIFT: u32 = 22;
pub const MAXVISPLANES: usize = 128;
pub fn map_plane(state: &mut GameState, y: i32, x1: i32, x2: i32) {
    if x2 < x1
        || x1 < 0
        || x2 >= state.render.r_draw.viewwidth
        || y > state.render.r_draw.viewheight
    {
        error(&format!("R_MapPlane: {x1}, {x2} at {y}"));
    }
    let distance: Fixed =
        if state.render.r_plane.planeheight == state.render.r_plane.cachedheight[y as usize] {
            state.render.r_draw.ds_xstep = state.render.r_plane.cachedxstep[y as usize];
            state.render.r_draw.ds_ystep = state.render.r_plane.cachedystep[y as usize];
            state.render.r_plane.cacheddistance[y as usize]
        } else {
            state.render.r_plane.cachedheight[y as usize] = state.render.r_plane.planeheight;
            state.render.r_plane.cacheddistance[y as usize] = fixed_mul(
                state.render.r_plane.planeheight,
                state.render.r_plane.yslope[y as usize],
            );
            let distance = state.render.r_plane.cacheddistance[y as usize];
            state.render.r_plane.cachedxstep[y as usize] =
                fixed_mul(distance, state.render.r_plane.basexscale);
            state.render.r_draw.ds_xstep = state.render.r_plane.cachedxstep[y as usize];
            state.render.r_plane.cachedystep[y as usize] =
                fixed_mul(distance, state.render.r_plane.baseyscale);
            state.render.r_draw.ds_ystep = state.render.r_plane.cachedystep[y as usize];
            distance
        };
    let length: Fixed = fixed_mul(distance, state.render.r_plane.distscale[x1 as usize]);
    let angle: usize = (state
        .render
        .r_main
        .viewangle
        .wrapping_add(state.render.r_main.xtoviewangle[x1 as usize])
        >> ANGLETOFINESHIFT) as usize;
    state.render.r_draw.ds_xfrac = state.render.r_main.viewx + fixed_mul(FINECOSINE[angle], length);
    state.render.r_draw.ds_yfrac = -state.render.r_main.viewy - fixed_mul(FINESINE[angle], length);
    if let Some(colormap) = state.render.r_main.fixedcolormap {
        state.render.r_draw.ds_colormap = colormap;
    } else {
        let mut index = (distance >> LIGHTZSHIFT) as usize;
        if index >= MAXLIGHTZ {
            index = MAXLIGHTZ - 1;
        }
        state.render.r_draw.ds_colormap =
            state.render.r_main.zlight[state.render.r_plane.planezlight][index];
    }
    state.render.r_draw.ds_y = y;
    state.render.r_draw.ds_x1 = x1;
    state.render.r_draw.ds_x2 = x2;
    state
        .render
        .r_main
        .spanfunc
        .expect("non-null function pointer")(state);
}
pub fn clear_planes(r_draw: &RDrawState, r_main: &RMainState, r_plane: &mut RPlaneState) {
    for i in 0..r_draw.viewwidth as usize {
        r_plane.floorclip[i] = r_draw.viewheight as i16;
        r_plane.ceilingclip[i] = -1_i16;
    }
    r_plane.lastvisplane = 0;
    r_plane.lastopening = 0;
    r_plane.cachedheight = [0; 200];
    let angle: usize = (r_main.viewangle.wrapping_sub(ANG90 as Angle) >> ANGLETOFINESHIFT) as usize;
    r_plane.basexscale = fixed_div(FINECOSINE[angle], r_main.centerxfrac);
    r_plane.baseyscale = -fixed_div(FINESINE[angle], r_main.centerxfrac);
}
pub fn find_plane(
    r_plane: &mut RPlaneState,
    r_sky: &RSkyState,
    mut height: Fixed,
    picnum: i32,
    mut lightlevel: i32,
) -> usize {
    let mut check: usize = 0;
    if picnum == r_sky.skyflatnum {
        height = 0;
        lightlevel = 0;
    }
    while check < r_plane.lastvisplane {
        let pl = r_plane.visplanes[check];
        if height == pl.height && picnum == pl.picnum && lightlevel == pl.lightlevel {
            break;
        }
        check += 1;
    }
    if check < r_plane.lastvisplane {
        return check;
    }
    if r_plane.lastvisplane == MAXVISPLANES {
        error("R_FindPlane: no more visplanes");
    }
    r_plane.lastvisplane += 1;
    let pl = &mut r_plane.visplanes[check];
    pl.height = height;
    pl.picnum = picnum;
    pl.lightlevel = lightlevel;
    pl.minx = SCREENWIDTH;
    pl.maxx = -1;
    pl.clear_top();
    check
}
pub fn check_plane(r_plane: &mut RPlaneState, pl: usize, start: i32, stop: i32) -> usize {
    let plv = &mut r_plane.visplanes[pl];
    let (intrl, unionl) = if start < plv.minx {
        (plv.minx, start)
    } else {
        (start, plv.minx)
    };
    let (intrh, unionh) = if stop > plv.maxx {
        (plv.maxx, stop)
    } else {
        (stop, plv.maxx)
    };
    let mut x = intrl;
    while x <= intrh {
        if i32::from(plv.top(x)) != 0xff {
            break;
        }
        x += 1;
    }
    if x > intrh {
        plv.minx = unionl;
        plv.maxx = unionh;
        return pl;
    }
    let (height, picnum, lightlevel) = (plv.height, plv.picnum, plv.lightlevel);
    let new_plane = r_plane.lastvisplane;
    r_plane.visplanes[new_plane].height = height;
    r_plane.visplanes[new_plane].picnum = picnum;
    r_plane.visplanes[new_plane].lightlevel = lightlevel;
    r_plane.lastvisplane += 1;
    let plv = &mut r_plane.visplanes[new_plane];
    plv.minx = start;
    plv.maxx = stop;
    plv.clear_top();
    new_plane
}
pub fn make_spans(
    state: &mut GameState,
    x: i32,
    mut t1: i32,
    mut b1: i32,
    mut t2: i32,
    mut b2: i32,
) {
    while t1 < t2 && t1 <= b1 {
        let spanstart_t1 = state.render.r_plane.spanstart[t1 as usize];
        map_plane(state, t1, spanstart_t1, x - 1);
        t1 += 1;
    }
    while b1 > b2 && b1 >= t1 {
        let spanstart_b1 = state.render.r_plane.spanstart[b1 as usize];
        map_plane(state, b1, spanstart_b1, x - 1);
        b1 -= 1;
    }
    while t2 < t1 && t2 <= b2 {
        state.render.r_plane.spanstart[t2 as usize] = x;
        t2 += 1;
    }
    while b2 > b1 && b2 >= t2 {
        state.render.r_plane.spanstart[b2 as usize] = x;
        b2 -= 1;
    }
}
pub fn draw_planes(state: &mut GameState) {
    if state.render.r_bsp.ds_p > MAXDRAWSEGS {
        error(&format!(
            "R_DrawPlanes: drawsegs overflow ({})",
            state.render.r_bsp.ds_p as i64,
        ));
    }
    if state.render.r_plane.lastvisplane > MAXVISPLANES {
        error(&format!(
            "R_DrawPlanes: visplane overflow ({})",
            state.render.r_plane.lastvisplane as i64,
        ));
    }
    if state.render.r_plane.lastopening as i64 > i64::from(SCREENWIDTH * 64) {
        error(&format!(
            "R_DrawPlanes: opening overflow ({})",
            state.render.r_plane.lastopening as i64,
        ));
    }
    for pl in 0..state.render.r_plane.lastvisplane {
        let mut plv = state.render.r_plane.visplanes[pl];
        if plv.minx <= plv.maxx {
            if plv.picnum == state.render.r_sky.skyflatnum {
                state.render.r_draw.dc_iscale =
                    state.render.r_things.pspriteiscale >> state.render.r_main.detailshift;
                state.render.r_draw.dc_colormap = Some(0);
                state.render.r_draw.dc_texturemid = state.render.r_sky.skytexturemid as Fixed;
                for x in plv.minx..=plv.maxx {
                    state.render.r_draw.dc_yl = i32::from(plv.top(x));
                    state.render.r_draw.dc_yh = i32::from(plv.bottom(x));
                    if state.render.r_draw.dc_yl <= state.render.r_draw.dc_yh {
                        let angle: i32 = (state
                            .render
                            .r_main
                            .viewangle
                            .wrapping_add(state.render.r_main.xtoviewangle[x as usize])
                            >> ANGLETOSKYSHIFT) as i32;
                        state.render.r_draw.dc_x = x;
                        state.render.r_draw.dc_source = Some(get_column(
                            &*state.assets.fs,
                            &mut state.render.r_data,
                            &mut state.assets.w_wad,
                            state.render.r_sky.skytexture,
                            angle,
                        ));
                        state
                            .render
                            .r_main
                            .colfunc
                            .expect("non-null function pointer")(state);
                    }
                }
            } else {
                let lumpnum: i32 = state.render.r_data.firstflat
                    + state.render.r_data.flattranslation[plv.picnum as usize];
                lump_bytes(&*state.assets.fs, &mut state.assets.w_wad, lumpnum);
                state.render.r_draw.ds_source = Some(ColumnSource::Lump {
                    lump: lumpnum,
                    offset: 0,
                });
                state.render.r_plane.planeheight =
                    (plv.height - state.render.r_main.viewz).abs() as Fixed;
                let mut light: i32 =
                    (plv.lightlevel >> LIGHTSEGSHIFT) + state.render.r_main.extralight;
                if light >= LIGHTLEVELS {
                    light = LIGHTLEVELS - 1;
                }
                if light < 0 {
                    light = 0;
                }
                state.render.r_plane.planezlight = light as usize;
                plv.set_top(plv.maxx + 1, 0xff_u8);
                plv.set_top(plv.minx - 1, 0xff_u8);
                state.render.r_plane.visplanes[pl] = plv;
                let stop: i32 = plv.maxx + 1;
                for x in plv.minx..=stop {
                    make_spans(
                        state,
                        x,
                        i32::from(plv.top(x - 1)),
                        i32::from(plv.bottom(x - 1)),
                        i32::from(plv.top(x)),
                        i32::from(plv.bottom(x)),
                    );
                }
                release_lump_num(&state.assets.w_wad, lumpnum);
            }
        }
    }
}
