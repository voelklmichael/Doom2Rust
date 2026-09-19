use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::m_fixed::fixed_div;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;

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
        RPlaneState {
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

pub const ANGLETOSKYSHIFT: i32 = 22;
pub const MAXVISPLANES: i32 = 128;
pub fn map_plane(state: &mut GameState, y: i32, x1: i32, x2: i32) {
    let distance: Fixed;

    let mut index: u32;
    if x2 < x1 || x1 < 0 || x2 >= state.r_draw.viewwidth || y > state.r_draw.viewheight {
        error(&format!("R_MapPlane: {}, {} at {}", x1, x2, y));
    }
    if state.r_plane.planeheight != state.r_plane.cachedheight[y as usize] {
        state.r_plane.cachedheight[y as usize] = state.r_plane.planeheight;
        state.r_plane.cacheddistance[y as usize] =
            fixed_mul(state.r_plane.planeheight, state.r_plane.yslope[y as usize]);
        distance = state.r_plane.cacheddistance[y as usize];
        state.r_plane.cachedxstep[y as usize] = fixed_mul(distance, state.r_plane.basexscale);
        state.r_draw.ds_xstep = state.r_plane.cachedxstep[y as usize];
        state.r_plane.cachedystep[y as usize] = fixed_mul(distance, state.r_plane.baseyscale);
        state.r_draw.ds_ystep = state.r_plane.cachedystep[y as usize];
    } else {
        distance = state.r_plane.cacheddistance[y as usize];
        state.r_draw.ds_xstep = state.r_plane.cachedxstep[y as usize];
        state.r_draw.ds_ystep = state.r_plane.cachedystep[y as usize];
    }
    let length: Fixed = fixed_mul(distance, state.r_plane.distscale[x1 as usize]);
    let angle: Angle = state
        .r_main
        .viewangle
        .wrapping_add(state.r_main.xtoviewangle[x1 as usize])
        >> ANGLETOFINESHIFT;
    state.r_draw.ds_xfrac = state.r_main.viewx + fixed_mul(FINECOSINE[angle as usize], length);
    state.r_draw.ds_yfrac = -state.r_main.viewy - fixed_mul(FINESINE[angle as usize], length);
    if let Some(colormap) = state.r_main.fixedcolormap {
        state.r_draw.ds_colormap = colormap;
    } else {
        index = (distance >> LIGHTZSHIFT) as u32;
        if index >= MAXLIGHTZ as u32 {
            index = (MAXLIGHTZ - 1) as u32;
        }
        state.r_draw.ds_colormap = state.r_main.zlight[state.r_plane.planezlight][index as usize];
    }
    state.r_draw.ds_y = y;
    state.r_draw.ds_x1 = x1;
    state.r_draw.ds_x2 = x2;
    state.r_main.spanfunc.expect("non-null function pointer")(state);
}
pub fn clear_planes(state: &mut GameState) {
    for i in 0..state.r_draw.viewwidth {
        state.r_plane.floorclip[i as usize] = state.r_draw.viewheight as i16;
        state.r_plane.ceilingclip[i as usize] = -1_i16;
    }
    state.r_plane.lastvisplane = 0;
    state.r_plane.lastopening = 0;
    state.r_plane.cachedheight = [0; 200];
    let angle: Angle = state.r_main.viewangle.wrapping_sub(ANG90 as Angle) >> ANGLETOFINESHIFT;
    state.r_plane.basexscale = fixed_div(FINECOSINE[angle as usize], state.r_main.centerxfrac);
    state.r_plane.baseyscale = -fixed_div(FINESINE[angle as usize], state.r_main.centerxfrac);
}
pub fn find_plane(
    state: &mut GameState,
    mut height: Fixed,
    picnum: i32,
    mut lightlevel: i32,
) -> usize {
    let mut check: usize = 0;
    if picnum == state.r_sky.skyflatnum {
        height = 0;
        lightlevel = 0;
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
        error("R_FindPlane: no more visplanes");
    }
    state.r_plane.lastvisplane += 1;
    let pl = &mut state.r_plane.visplanes[check];
    pl.height = height;
    pl.picnum = picnum;
    pl.lightlevel = lightlevel;
    pl.minx = SCREENWIDTH;
    pl.maxx = -1;
    pl.clear_top();
    check
}
pub fn check_plane(state: &mut GameState, pl: usize, start: i32, stop: i32) -> usize {
    let plv = &mut state.r_plane.visplanes[pl];
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
        if plv.top(x) as i32 != 0xff {
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
    let fresh0 = state.r_plane.lastvisplane;
    state.r_plane.visplanes[fresh0].height = height;
    state.r_plane.visplanes[fresh0].picnum = picnum;
    state.r_plane.visplanes[fresh0].lightlevel = lightlevel;
    state.r_plane.lastvisplane += 1;
    let plv = &mut state.r_plane.visplanes[fresh0];
    plv.minx = start;
    plv.maxx = stop;
    plv.clear_top();
    fresh0
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
        let spanstart_t1 = state.r_plane.spanstart[t1 as usize];
        map_plane(state, t1, spanstart_t1, x - 1);
        t1 += 1;
    }
    while b1 > b2 && b1 >= t1 {
        let spanstart_b1 = state.r_plane.spanstart[b1 as usize];
        map_plane(state, b1, spanstart_b1, x - 1);
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
pub fn draw_planes(state: &mut GameState) {
    let mut light: i32;
    let mut stop: i32;
    let mut angle: i32;
    let mut lumpnum: i32;
    if state.r_bsp.ds_p as i64 > MAXDRAWSEGS as i64 {
        error(&format!(
            "R_DrawPlanes: drawsegs overflow ({})",
            state.r_bsp.ds_p as i64,
        ));
    }
    if state.r_plane.lastvisplane as i64 > MAXVISPLANES as i64 {
        error(&format!(
            "R_DrawPlanes: visplane overflow ({})",
            state.r_plane.lastvisplane as i64,
        ));
    }
    if state.r_plane.lastopening as i64 > (SCREENWIDTH * 64) as i64 {
        error(&format!(
            "R_DrawPlanes: opening overflow ({})",
            state.r_plane.lastopening as i64,
        ));
    }
    for pl in 0..state.r_plane.lastvisplane {
        let mut plv = state.r_plane.visplanes[pl];
        if plv.minx <= plv.maxx {
            if plv.picnum == state.r_sky.skyflatnum {
                state.r_draw.dc_iscale = state.r_things.pspriteiscale >> state.r_main.detailshift;
                state.r_draw.dc_colormap = Some(0);
                state.r_draw.dc_texturemid = state.r_sky.skytexturemid as Fixed;
                for x in plv.minx..=plv.maxx {
                    state.r_draw.dc_yl = plv.top(x) as i32;
                    state.r_draw.dc_yh = plv.bottom(x) as i32;
                    if state.r_draw.dc_yl <= state.r_draw.dc_yh {
                        angle = (state
                            .r_main
                            .viewangle
                            .wrapping_add(state.r_main.xtoviewangle[x as usize])
                            >> ANGLETOSKYSHIFT) as i32;
                        state.r_draw.dc_x = x;
                        state.r_draw.dc_source =
                            Some(get_column(state, state.r_sky.skytexture, angle));
                        state.r_main.colfunc.expect("non-null function pointer")(state);
                    }
                }
            } else {
                lumpnum =
                    state.r_data.firstflat + state.r_data.flattranslation[plv.picnum as usize];
                lump_bytes(state, lumpnum);
                state.r_draw.ds_source = Some(ColumnSource::Lump {
                    lump: lumpnum,
                    offset: 0,
                });
                state.r_plane.planeheight = (plv.height - state.r_main.viewz).abs() as Fixed;
                light = (plv.lightlevel >> LIGHTSEGSHIFT) + state.r_main.extralight;
                if light >= LIGHTLEVELS {
                    light = LIGHTLEVELS - 1;
                }
                if light < 0 {
                    light = 0;
                }
                state.r_plane.planezlight = light as usize;
                plv.set_top(plv.maxx + 1, 0xff_u8);
                plv.set_top(plv.minx - 1, 0xff_u8);
                state.r_plane.visplanes[pl] = plv;
                stop = plv.maxx + 1;
                for x in plv.minx..=stop {
                    make_spans(
                        state,
                        x,
                        plv.top(x - 1) as i32,
                        plv.bottom(x - 1) as i32,
                        plv.top(x) as i32,
                        plv.bottom(x) as i32,
                    );
                }
                release_lump_num(&mut state.w_wad, lumpnum);
            }
        }
    }
}
