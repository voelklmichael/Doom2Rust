use crate::game_state::GameState;
use crate::i_system::error;
use crate::index::ToIndex;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::INT_MAX;
use crate::m_fixed::INT_MIN;
use crate::p_mobj::LineFlags;
use crate::p_setup::SectorId;
use crate::r_data::get_column;
use crate::r_defs::ClipArray;
use crate::r_defs::DrawSeg;
use crate::r_defs::VisPlane;
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

use crate::tables::fine_sine;
use crate::tables::fine_tangent;
use crate::tables::Angle;
use crate::tables::ANG180;
use crate::tables::ANG90;
use crate::tables::FINETANGENT_LEN;
use core::ops::{Add, AddAssign, Neg, Sub};

pub struct RSegsState {
    pub segtextured: bool,
    pub markfloor: bool,
    pub markceiling: bool,
    pub maskedtexture: bool,
    pub toptexture: i32,
    pub bottomtexture: i32,
    pub midtexture: i32,
    pub rw_normalangle: Angle,
    pub rw_angle1: Angle,
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
    pub worldtop: Fixed,
    pub worldbottom: Fixed,
    pub worldhigh: Fixed,
    pub worldlow: Fixed,
    pub pixhigh: HeightFrac,
    pub pixlow: HeightFrac,
    pub pixhighstep: HeightFrac,
    pub pixlowstep: HeightFrac,
    pub topfrac: HeightFrac,
    pub topstep: HeightFrac,
    pub bottomfrac: HeightFrac,
    pub bottomstep: HeightFrac,
    pub walllights: LightRow48,
    pub maskedtexturecol: Option<ClipArray>,
}

impl RSegsState {
    /// The column table of the masked texture being drawn (set by `store_wall_range`).
    pub fn maskedtexturecol(&self) -> ClipArray {
        self.maskedtexturecol
            .expect("no masked texture column table")
    }
}

impl Default for RSegsState {
    fn default() -> Self {
        Self {
            segtextured: false,
            markfloor: false,
            markceiling: false,
            maskedtexture: false,
            toptexture: 0,
            bottomtexture: 0,
            midtexture: 0,
            rw_normalangle: Angle::ZERO,
            rw_angle1: Angle::ZERO,
            rw_x: 0,
            rw_stopx: 0,
            rw_centerangle: Angle::ZERO,
            rw_offset: Fixed::ZERO,
            rw_distance: Fixed::ZERO,
            rw_scale: Fixed::ZERO,
            rw_scalestep: Fixed::ZERO,
            rw_midtexturemid: Fixed::ZERO,
            rw_toptexturemid: Fixed::ZERO,
            rw_bottomtexturemid: Fixed::ZERO,
            worldtop: Fixed::ZERO,
            worldbottom: Fixed::ZERO,
            worldhigh: Fixed::ZERO,
            worldlow: Fixed::ZERO,
            pixhigh: HeightFrac::ZERO,
            pixlow: HeightFrac::ZERO,
            pixhighstep: HeightFrac::ZERO,
            pixlowstep: HeightFrac::ZERO,
            topfrac: HeightFrac::ZERO,
            topstep: HeightFrac::ZERO,
            bottomfrac: HeightFrac::ZERO,
            bottomstep: HeightFrac::ZERO,
            walllights: LightRow48::Normal(0),
            maskedtexturecol: None,
        }
    }
}

pub const SHRT_MAX: i32 = __SHRT_MAX__;
pub const SIL_BOTTOM: i32 = 1;
pub const SIL_TOP: i32 = 2;
pub const SIL_BOTH: i32 = 3;
pub const MAXDRAWSEGS: usize = 256;
/// The light table of the current seg: its front sector's light level, a step darker for a
/// horizontal wall and lighter for a vertical one.
fn wall_lights(state: &GameState) -> LightRow48 {
    let mut lightnum: i32 = (i32::from(
        state
            .world
            .p_setup
            .sector(state.render.r_bsp.front())
            .lightlevel,
    ) >> LIGHTSEGSHIFT)
        + state.render.r_main.extralight;
    let seg = state.world.p_setup.seg(state.render.r_bsp.curline);
    let v1 = state.world.p_setup.vertexes[seg.v1.0 as usize];
    let v2 = state.world.p_setup.vertexes[seg.v2.0 as usize];
    if v1.y == v2.y {
        lightnum -= 1;
    } else if v1.x == v2.x {
        lightnum += 1;
    }
    LightRow48::Normal(lightnum.clamp(0, LIGHTLEVELS - 1).idx())
}

/// `dc_texturemid` for the masked middle texture `texnum` of the current two-sided seg: the
/// bottom of the texture at the higher floor, or its top at the lower ceiling, relative to the eye
/// and shifted by the side's row offset.
fn masked_texturemid(state: &GameState, texnum: i32) -> Fixed {
    let p_setup = &state.world.p_setup;
    let seg = p_setup.seg(state.render.r_bsp.curline);
    let front = p_setup.sector(state.render.r_bsp.front());
    let back = p_setup.sector(state.render.r_bsp.back());
    let texturemid = if p_setup
        .line(seg.linedef)
        .flags
        .contains(LineFlags::DONTPEGBOTTOM)
    {
        front.floorheight.max(back.floorheight) + state.render.r_data.textureheight[texnum.idx()]
            - state.render.r_main.viewz
    } else {
        front.ceilingheight.min(back.ceilingheight) - state.render.r_main.viewz
    };
    texturemid + p_setup.side(seg.sidedef).rowoffset
}

/// Draws the masked texture column `dc_x` of a drawseg, if it has not been drawn yet.
fn render_masked_seg_column(state: &mut GameState, maskedtexturecol: ClipArray, texnum: i32) {
    let dc_x = state.render.r_draw.dc_x as isize;
    let column = i32::from(maskedtexturecol.get(state, dc_x));
    if column == SHRT_MAX {
        return;
    }
    if state.render.r_main.fixedcolormap.is_none() {
        let mut index: u32 = (state.render.r_things.spryscale >> LIGHTSCALESHIFT)
            .to_bits()
            .cast_unsigned();
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
    state.render.r_draw.dc_iscale = Fixed(
        (0xffffffff_u32.wrapping_div(state.render.r_things.spryscale.to_bits().cast_unsigned()))
            as i32,
    );
    let col = advance_source(
        get_column(
            &*state.assets.fs,
            &mut state.render.r_data,
            &mut state.assets.w_wad,
            texnum,
            column,
        ),
        (-3_isize).cast_unsigned(),
    );
    draw_masked_column(state, col);
    maskedtexturecol.set(state, dc_x, SHRT_MAX as i16);
}

pub fn render_masked_seg_range(state: &mut GameState, ds: &DrawSeg, x1: i32, x2: i32) {
    state.render.r_bsp.curline = ds.curline;
    let seg = state.world.p_setup.seg(state.render.r_bsp.curline);
    state.render.r_bsp.frontsector = seg.frontsector;
    state.render.r_bsp.backsector = seg.backsector;
    let texnum: i32 = state.render.r_data.texturetranslation
        [state.world.p_setup.side(seg.sidedef).midtexture.idx()];
    state.render.r_segs.walllights = wall_lights(state);
    state.render.r_segs.maskedtexturecol = ds.maskedtexturecol;
    state.render.r_segs.rw_scalestep = ds.scalestep;
    state.render.r_things.spryscale = ds.scale1 + (x1 - ds.x1) * state.render.r_segs.rw_scalestep;
    state.render.r_things.mfloorclip = ds.sprbottomclip;
    state.render.r_things.mceilingclip = ds.sprtopclip;
    state.render.r_draw.dc_texturemid = masked_texturemid(state, texnum);
    if state.render.r_main.fixedcolormap.is_some() {
        state.render.r_draw.dc_colormap = state.render.r_main.fixedcolormap;
    }
    let maskedtexturecol = state.render.r_segs.maskedtexturecol();
    state.render.r_draw.dc_x = x1;
    while state.render.r_draw.dc_x <= x2 {
        render_masked_seg_column(state, maskedtexturecol, texnum);
        state.render.r_things.spryscale += state.render.r_segs.rw_scalestep;
        state.render.r_draw.dc_x += 1;
    }
}
pub const HEIGHTBITS: u32 = 12;
pub const HEIGHTUNIT: i32 = 1 << HEIGHTBITS;

/// A screen row with 12 fractional bits (20.12), the format `R_RenderSegLoop` steps walls in
/// (`topfrac`, `pixhigh`, ...). It is a different scale from [`Fixed`] (16 fractional bits), so
/// it is its own type: a height converts in with [`HeightFrac::from_height`] and a row comes out
/// with [`HeightFrac::floor`] or [`HeightFrac::ceil`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct HeightFrac(i32);

impl HeightFrac {
    pub const ZERO: Self = Self(0);

    /// A 16.16 height or row, at 12 fractional bits (the original's `>> 4`).
    pub fn from_height(height: Fixed) -> Self {
        Self(height.to_bits() >> (FRACBITS - HEIGHTBITS))
    }

    /// This height projected at a 16.16 `scale`: the result keeps the 12 fractional bits.
    pub fn scaled(self, scale: Fixed) -> Self {
        Self(fixed_mul(Fixed(self.0), scale).to_bits())
    }

    /// The row this value falls in, rounded down.
    pub fn floor(self) -> i32 {
        self.0 >> HEIGHTBITS
    }

    /// The row this value falls in, rounded up.
    pub fn ceil(self) -> i32 {
        (self.0 + HEIGHTUNIT - 1) >> HEIGHTBITS
    }
}

impl Add for HeightFrac {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for HeightFrac {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for HeightFrac {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Neg for HeightFrac {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}
/// Draws the wall column `dc_yl..=dc_yh` of `texture` at `texturecolumn` with the current column
/// function.
fn draw_wall_column(
    state: &mut GameState,
    yl: i32,
    yh: i32,
    texturemid: Fixed,
    texture: i32,
    texturecolumn: Fixed,
) {
    state.render.r_draw.dc_yl = yl;
    state.render.r_draw.dc_yh = yh;
    state.render.r_draw.dc_texturemid = texturemid;
    state.render.r_draw.dc_source = Some(get_column(
        &*state.assets.fs,
        &mut state.render.r_data,
        &mut state.assets.w_wad,
        texture,
        texturecolumn.to_bits(),
    ));
    state
        .render
        .r_main
        .colfunc
        .expect("non-null function pointer")(state);
}

/// Records rows `top..=bottom` of column `x` as belonging to a visplane (when there are any).
fn mark_visplane_column(plane: &mut VisPlane, x: i32, top: i32, bottom: i32) {
    if top <= bottom {
        plane.set_top(x, top.cast_unsigned() as u8);
        plane.set_bottom(x, bottom.cast_unsigned() as u8);
    }
}

/// The texture column at `rw_x`, and the light and scale the column function will draw it with.
/// Zero for a wall without textures.
fn wall_texture_column(state: &mut GameState) -> Fixed {
    if !state.render.r_segs.segtextured {
        return Fixed::ZERO;
    }
    let mut angle: usize = (state.render.r_segs.rw_centerangle
        + state.render.r_main.xtoviewangle[state.render.r_segs.rw_x.idx()])
    .fine();
    // A column at a seg's clipped edge can land just outside the
    // front half-plane; vanilla reads past finetangent[] there.
    angle = angle.min(FINETANGENT_LEN - 1);
    let column = (state.render.r_segs.rw_offset
        - fixed_mul(fine_tangent(angle), state.render.r_segs.rw_distance))
        >> FRACBITS;
    let mut index: u32 = (state.render.r_segs.rw_scale >> LIGHTSCALESHIFT)
        .to_bits()
        .cast_unsigned();
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
    state.render.r_draw.dc_iscale = Fixed(
        (0xffffffff_u32.wrapping_div(state.render.r_segs.rw_scale.to_bits().cast_unsigned()))
            as i32,
    );
    column
}

/// The upper texture of a two-sided wall in column `rw_x`, from row `yl` down, and the ceiling
/// clip that follows from it.
fn draw_upper_wall_column(state: &mut GameState, yl: i32, texturecolumn: Fixed) {
    let x = state.render.r_segs.rw_x.idx();
    if state.render.r_segs.toptexture != 0 {
        let mut mid = state.render.r_segs.pixhigh.floor();
        state.render.r_segs.pixhigh += state.render.r_segs.pixhighstep;
        if mid >= i32::from(state.render.r_plane.floorclip[x]) {
            mid = i32::from(state.render.r_plane.floorclip[x]) - 1;
        }
        if mid >= yl {
            draw_wall_column(
                state,
                yl,
                mid,
                state.render.r_segs.rw_toptexturemid,
                state.render.r_segs.toptexture,
                texturecolumn,
            );
            state.render.r_plane.ceilingclip[x] = mid as i16;
        } else {
            state.render.r_plane.ceilingclip[x] = (yl - 1) as i16;
        }
    } else if state.render.r_segs.markceiling {
        state.render.r_plane.ceilingclip[x] = (yl - 1) as i16;
    }
}

/// The lower texture of a two-sided wall in column `rw_x`, down to row `yh`, and the floor clip
/// that follows from it.
fn draw_lower_wall_column(state: &mut GameState, yh: i32, texturecolumn: Fixed) {
    let x = state.render.r_segs.rw_x.idx();
    if state.render.r_segs.bottomtexture != 0 {
        let mut mid = state.render.r_segs.pixlow.ceil();
        state.render.r_segs.pixlow += state.render.r_segs.pixlowstep;
        if mid <= i32::from(state.render.r_plane.ceilingclip[x]) {
            mid = i32::from(state.render.r_plane.ceilingclip[x]) + 1;
        }
        if mid <= yh {
            draw_wall_column(
                state,
                mid,
                yh,
                state.render.r_segs.rw_bottomtexturemid,
                state.render.r_segs.bottomtexture,
                texturecolumn,
            );
            state.render.r_plane.floorclip[x] = mid as i16;
        } else {
            state.render.r_plane.floorclip[x] = (yh + 1) as i16;
        }
    } else if state.render.r_segs.markfloor {
        state.render.r_plane.floorclip[x] = (yh + 1) as i16;
    }
}

pub fn render_seg_loop(state: &mut GameState) {
    while state.render.r_segs.rw_x < state.render.r_segs.rw_stopx {
        let x = state.render.r_segs.rw_x;
        let ceilingclip = i32::from(state.render.r_plane.ceilingclip[x.idx()]);
        let floorclip = i32::from(state.render.r_plane.floorclip[x.idx()]);
        // The rows of the wall that are visible: below the ceiling clip, above the floor clip.
        let yl = state.render.r_segs.topfrac.ceil().max(ceilingclip + 1);
        let yh = state.render.r_segs.bottomfrac.floor().min(floorclip - 1);
        if state.render.r_segs.markceiling {
            let plane = state.render.r_plane.ceilingplane();
            mark_visplane_column(
                &mut state.render.r_plane.visplanes[plane],
                x,
                ceilingclip + 1,
                (yl - 1).min(floorclip - 1),
            );
        }
        if state.render.r_segs.markfloor {
            let plane = state.render.r_plane.floorplane();
            mark_visplane_column(
                &mut state.render.r_plane.visplanes[plane],
                x,
                (yh + 1).max(ceilingclip + 1),
                floorclip - 1,
            );
        }
        let texturecolumn = wall_texture_column(state);
        if state.render.r_segs.midtexture != 0 {
            draw_wall_column(
                state,
                yl,
                yh,
                state.render.r_segs.rw_midtexturemid,
                state.render.r_segs.midtexture,
                texturecolumn,
            );
            state.render.r_plane.ceilingclip[x.idx()] = state.render.r_draw.viewheight as i16;
            state.render.r_plane.floorclip[x.idx()] = -1_i16;
        } else {
            draw_upper_wall_column(state, yl, texturecolumn);
            draw_lower_wall_column(state, yh, texturecolumn);
            if state.render.r_segs.maskedtexture {
                let maskedtexturecol = state.render.r_segs.maskedtexturecol();
                maskedtexturecol.set(state, x as isize, texturecolumn.to_bits() as i16);
            }
        }
        state.render.r_segs.rw_scale += state.render.r_segs.rw_scalestep;
        state.render.r_segs.topfrac += state.render.r_segs.topstep;
        state.render.r_segs.bottomfrac += state.render.r_segs.bottomstep;
        state.render.r_segs.rw_x += 1;
    }
}
pub fn store_wall_range(state: &mut GameState, start: i32, stop: i32) {
    if state.render.r_bsp.ds_p == MAXDRAWSEGS {
        return;
    }
    if start >= state.render.r_draw.viewwidth || start > stop {
        error(&format!("Bad R_RenderWallRange: {start} to {stop}"));
    }
    let hyp = set_up_wall_geometry(state, start, stop);
    set_up_wall_heights(state);
    match state.render.r_bsp.backsector {
        None => set_up_solid_wall(state),
        Some(backsector) => set_up_two_sided_wall(state, backsector),
    }
    set_up_wall_texturing(state, hyp);
    mark_wall_planes(state);
    render_seg_loop(state);
    save_sprite_clips(state, start);
    state.render.r_bsp.ds_p += 1;
}

/// The drawseg's extent and scale, from the seg's angle to the viewer. Returns the distance to
/// the seg's first vertex, which the texture offset needs later.
fn set_up_wall_geometry(state: &mut GameState, start: i32, stop: i32) -> Fixed {
    state.render.r_bsp.sidedef = state.world.p_setup.seg(state.render.r_bsp.curline).sidedef;
    state.render.r_bsp.linedef = state.world.p_setup.seg(state.render.r_bsp.curline).linedef;
    state
        .world
        .p_setup
        .line_mut(state.render.r_bsp.linedef)
        .flags
        .insert(LineFlags::MAPPED);
    state.render.r_segs.rw_normalangle =
        state.world.p_setup.seg(state.render.r_bsp.curline).angle + ANG90;
    let mut offsetangle = Angle(
        (state.render.r_segs.rw_normalangle - state.render.r_segs.rw_angle1)
            .to_signed()
            .unsigned_abs(),
    );
    if offsetangle > ANG90 {
        offsetangle = ANG90;
    }
    let distangle: Angle = ANG90 - offsetangle;
    let curline_v1 = state.world.p_setup.vertexes
        [state.world.p_setup.seg(state.render.r_bsp.curline).v1.0 as usize];
    let (v1x, v1y) = (curline_v1.x, curline_v1.y);
    let hyp: Fixed = point_to_dist(&state.render.r_main, v1x, v1y);
    let sineval: Fixed = fine_sine(distangle.fine());
    state.render.r_segs.rw_distance = fixed_mul(hyp, sineval);
    state.render.r_segs.rw_x = start;
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].x1 = state.render.r_segs.rw_x;
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].x2 = stop;
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].curline = state.render.r_bsp.curline;
    state.render.r_segs.rw_stopx = stop + 1;
    let angle1 = state.render.r_main.viewangle + state.render.r_main.xtoviewangle[start.idx()];
    state.render.r_segs.rw_scale =
        scale_from_global_angle(&state.render.r_main, &state.render.r_segs, angle1);
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].scale1 = state.render.r_segs.rw_scale;
    if stop > start {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].scale2 = {
            let angle2 =
                state.render.r_main.viewangle + state.render.r_main.xtoviewangle[stop.idx()];
            scale_from_global_angle(&state.render.r_main, &state.render.r_segs, angle2)
        };
        state.render.r_segs.rw_scalestep = (state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p]
            .scale2
            - state.render.r_segs.rw_scale)
            / (stop - start);
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].scalestep =
            state.render.r_segs.rw_scalestep;
    } else {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].scale2 =
            state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].scale1;
    }
    hyp
}

/// The seg's heights relative to the eye, and no textures picked yet.
fn set_up_wall_heights(state: &mut GameState) {
    state.render.r_segs.worldtop = state
        .world
        .p_setup
        .sector(state.render.r_bsp.front())
        .ceilingheight
        - state.render.r_main.viewz;
    state.render.r_segs.worldbottom = state
        .world
        .p_setup
        .sector(state.render.r_bsp.front())
        .floorheight
        - state.render.r_main.viewz;
    state.render.r_segs.maskedtexture = false;
    state.render.r_segs.bottomtexture = i32::from(state.render.r_segs.maskedtexture);
    state.render.r_segs.toptexture = state.render.r_segs.bottomtexture;
    state.render.r_segs.midtexture = state.render.r_segs.toptexture;
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].maskedtexturecol = None;
}

/// A one-sided seg: one middle texture, and everything behind it is hidden.
fn set_up_solid_wall(state: &mut GameState) {
    state.render.r_segs.midtexture = state.render.r_data.texturetranslation[state
        .world
        .p_setup
        .side_mut(state.render.r_bsp.sidedef)
        .midtexture
        .idx()];
    state.render.r_segs.markceiling = true;
    state.render.r_segs.markfloor = state.render.r_segs.markceiling;
    if state
        .world
        .p_setup
        .line_mut(state.render.r_bsp.linedef)
        .flags
        .contains(LineFlags::DONTPEGBOTTOM)
    {
        let vtop = state
            .world
            .p_setup
            .sector(state.render.r_bsp.front())
            .floorheight
            + state.render.r_data.textureheight[state
                .world
                .p_setup
                .side_mut(state.render.r_bsp.sidedef)
                .midtexture
                .idx()];
        state.render.r_segs.rw_midtexturemid = vtop - state.render.r_main.viewz;
    } else {
        state.render.r_segs.rw_midtexturemid = state.render.r_segs.worldtop;
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
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].bsilheight = Fixed(INT_MAX);
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].tsilheight = Fixed(INT_MIN);
}

/// A two-sided seg: what shows above and below the opening, and what hides sprites.
fn set_up_two_sided_wall(state: &mut GameState, backsector: SectorId) {
    set_up_two_sided_silhouette(state, backsector);
    set_up_two_sided_planes(state, backsector);
    set_up_two_sided_textures(state, backsector);
}

/// Which sprites the seg hides (its silhouette) and where their clip rows come from.
fn set_up_two_sided_silhouette(state: &mut GameState, backsector: SectorId) {
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprbottomclip = None;
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprtopclip =
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprbottomclip;
    state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette = 0;
    if state
        .world
        .p_setup
        .sector(state.render.r_bsp.front())
        .floorheight
        > state.world.p_setup.sector(backsector).floorheight
    {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette = SIL_BOTTOM;
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].bsilheight = state
            .world
            .p_setup
            .sector(state.render.r_bsp.front())
            .floorheight;
    } else if state.world.p_setup.sector(backsector).floorheight > state.render.r_main.viewz {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette = SIL_BOTTOM;
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].bsilheight = Fixed(INT_MAX);
    }
    if state
        .world
        .p_setup
        .sector(state.render.r_bsp.front())
        .ceilingheight
        < state.world.p_setup.sector(backsector).ceilingheight
    {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette |= SIL_TOP;
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].tsilheight = state
            .world
            .p_setup
            .sector(state.render.r_bsp.front())
            .ceilingheight;
    } else if state.world.p_setup.sector(backsector).ceilingheight < state.render.r_main.viewz {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette |= SIL_TOP;
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].tsilheight = Fixed(INT_MIN);
    }
    if state.world.p_setup.sector(backsector).ceilingheight
        <= state
            .world
            .p_setup
            .sector(state.render.r_bsp.front())
            .floorheight
    {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprbottomclip =
            Some(ClipArray::NegOneArray);
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].bsilheight = Fixed(INT_MAX);
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette |= SIL_BOTTOM;
    }
    if state.world.p_setup.sector(backsector).floorheight
        >= state
            .world
            .p_setup
            .sector(state.render.r_bsp.front())
            .ceilingheight
    {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprtopclip =
            Some(ClipArray::ScreenHeightArray);
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].tsilheight = Fixed(INT_MIN);
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette |= SIL_TOP;
    }
}

/// The heights of the opening, and whether the floor and ceiling planes need marking.
fn set_up_two_sided_planes(state: &mut GameState, backsector: SectorId) {
    state.render.r_segs.worldhigh =
        state.world.p_setup.sector(backsector).ceilingheight - state.render.r_main.viewz;
    state.render.r_segs.worldlow =
        state.world.p_setup.sector(backsector).floorheight - state.render.r_main.viewz;
    if i32::from(
        state
            .world
            .p_setup
            .sector(state.render.r_bsp.front())
            .ceilingpic,
    ) == state.render.r_sky.skyflatnum
        && i32::from(state.world.p_setup.sector(backsector).ceilingpic)
            == state.render.r_sky.skyflatnum
    {
        state.render.r_segs.worldtop = state.render.r_segs.worldhigh;
    }
    state.render.r_segs.markfloor = state.render.r_segs.worldlow != state.render.r_segs.worldbottom
        || i32::from(state.world.p_setup.sector(backsector).floorpic)
            != i32::from(
                state
                    .world
                    .p_setup
                    .sector(state.render.r_bsp.front())
                    .floorpic,
            )
        || i32::from(state.world.p_setup.sector(backsector).lightlevel)
            != i32::from(
                state
                    .world
                    .p_setup
                    .sector(state.render.r_bsp.front())
                    .lightlevel,
            );
    state.render.r_segs.markceiling = state.render.r_segs.worldhigh != state.render.r_segs.worldtop
        || i32::from(state.world.p_setup.sector(backsector).ceilingpic)
            != i32::from(
                state
                    .world
                    .p_setup
                    .sector(state.render.r_bsp.front())
                    .ceilingpic,
            )
        || i32::from(state.world.p_setup.sector(backsector).lightlevel)
            != i32::from(
                state
                    .world
                    .p_setup
                    .sector(state.render.r_bsp.front())
                    .lightlevel,
            );
    if state.world.p_setup.sector(backsector).ceilingheight
        <= state
            .world
            .p_setup
            .sector(state.render.r_bsp.front())
            .floorheight
        || state.world.p_setup.sector(backsector).floorheight
            >= state
                .world
                .p_setup
                .sector(state.render.r_bsp.front())
                .ceilingheight
    {
        state.render.r_segs.markfloor = true;
        state.render.r_segs.markceiling = state.render.r_segs.markfloor;
    }
}

/// The upper, lower and masked middle textures of a two-sided seg.
fn set_up_two_sided_textures(state: &mut GameState, backsector: SectorId) {
    if state.render.r_segs.worldhigh < state.render.r_segs.worldtop {
        state.render.r_segs.toptexture = state.render.r_data.texturetranslation[state
            .world
            .p_setup
            .side_mut(state.render.r_bsp.sidedef)
            .toptexture
            .idx()];
        if state
            .world
            .p_setup
            .line_mut(state.render.r_bsp.linedef)
            .flags
            .contains(LineFlags::DONTPEGTOP)
        {
            state.render.r_segs.rw_toptexturemid = state.render.r_segs.worldtop;
        } else {
            let vtop = state.world.p_setup.sector(backsector).ceilingheight
                + state.render.r_data.textureheight[state
                    .world
                    .p_setup
                    .side_mut(state.render.r_bsp.sidedef)
                    .toptexture
                    .idx()];
            state.render.r_segs.rw_toptexturemid = vtop - state.render.r_main.viewz;
        }
    }
    if state.render.r_segs.worldlow > state.render.r_segs.worldbottom {
        state.render.r_segs.bottomtexture = state.render.r_data.texturetranslation[state
            .world
            .p_setup
            .side_mut(state.render.r_bsp.sidedef)
            .bottomtexture
            .idx()];
        if state
            .world
            .p_setup
            .line_mut(state.render.r_bsp.linedef)
            .flags
            .contains(LineFlags::DONTPEGBOTTOM)
        {
            state.render.r_segs.rw_bottomtexturemid = state.render.r_segs.worldtop;
        } else {
            state.render.r_segs.rw_bottomtexturemid = state.render.r_segs.worldlow;
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
            (state.render.r_segs.rw_stopx - state.render.r_segs.rw_x).idx();
    }
}

/// The texture offset, and the light table of the wall.
fn set_up_wall_texturing(state: &mut GameState, hyp: Fixed) {
    state.render.r_segs.segtextured = (state.render.r_segs.midtexture
        | state.render.r_segs.toptexture
        | state.render.r_segs.bottomtexture)
        != 0
        || state.render.r_segs.maskedtexture;
    if state.render.r_segs.segtextured {
        let mut offsetangle = state.render.r_segs.rw_normalangle - state.render.r_segs.rw_angle1;
        if offsetangle > ANG180 {
            offsetangle = -offsetangle;
        }
        if offsetangle > ANG90 {
            offsetangle = ANG90;
        }
        let sineval = fine_sine(offsetangle.fine());
        state.render.r_segs.rw_offset = fixed_mul(hyp, sineval);
        if (state.render.r_segs.rw_normalangle - state.render.r_segs.rw_angle1) < ANG180 {
            state.render.r_segs.rw_offset = -state.render.r_segs.rw_offset;
        }
        state.render.r_segs.rw_offset += state
            .world
            .p_setup
            .side_mut(state.render.r_bsp.sidedef)
            .textureoffset
            + state.world.p_setup.seg(state.render.r_bsp.curline).offset;
        state.render.r_segs.rw_centerangle =
            (ANG90 + state.render.r_main.viewangle) - state.render.r_segs.rw_normalangle;
        if state.render.r_main.fixedcolormap.is_none() {
            state.render.r_segs.walllights = wall_lights(state);
        }
    }
}

/// Clears the plane marks that cannot show, steps the wall's top and bottom down the columns,
/// and finds the visplanes the wall's floor and ceiling belong to.
fn mark_wall_planes(state: &mut GameState) {
    if state
        .world
        .p_setup
        .sector(state.render.r_bsp.front())
        .floorheight
        >= state.render.r_main.viewz
    {
        state.render.r_segs.markfloor = false;
    }
    if state
        .world
        .p_setup
        .sector(state.render.r_bsp.front())
        .ceilingheight
        <= state.render.r_main.viewz
        && i32::from(
            state
                .world
                .p_setup
                .sector(state.render.r_bsp.front())
                .ceilingpic,
        ) != state.render.r_sky.skyflatnum
    {
        state.render.r_segs.markceiling = false;
    }
    let centery = HeightFrac::from_height(state.render.r_main.centeryfrac);
    let rw_scale = state.render.r_segs.rw_scale;
    let rw_scalestep = state.render.r_segs.rw_scalestep;
    let worldtop = HeightFrac::from_height(state.render.r_segs.worldtop);
    let worldbottom = HeightFrac::from_height(state.render.r_segs.worldbottom);
    state.render.r_segs.topstep = -worldtop.scaled(rw_scalestep);
    state.render.r_segs.topfrac = centery - worldtop.scaled(rw_scale);
    state.render.r_segs.bottomstep = -worldbottom.scaled(rw_scalestep);
    state.render.r_segs.bottomfrac = centery - worldbottom.scaled(rw_scale);
    if state.render.r_bsp.backsector.is_some() {
        let worldhigh = HeightFrac::from_height(state.render.r_segs.worldhigh);
        let worldlow = HeightFrac::from_height(state.render.r_segs.worldlow);
        if worldhigh < worldtop {
            state.render.r_segs.pixhigh = centery - worldhigh.scaled(rw_scale);
            state.render.r_segs.pixhighstep = -worldhigh.scaled(rw_scalestep);
        }
        if worldlow > worldbottom {
            state.render.r_segs.pixlow = centery - worldlow.scaled(rw_scale);
            state.render.r_segs.pixlowstep = -worldlow.scaled(rw_scalestep);
        }
    }
    if state.render.r_segs.markceiling {
        let (ceilingplane, rw_x, rw_stopx_1) = (
            state.render.r_plane.ceilingplane(),
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
            state.render.r_plane.floorplane(),
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
}

/// Saves the ceiling and floor clip rows for the sprites drawn behind the seg, and completes the
/// silhouette of a masked texture.
fn save_sprite_clips(state: &mut GameState, start: i32) {
    if (state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette & SIL_TOP != 0
        || state.render.r_segs.maskedtexture)
        && state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p]
            .sprtopclip
            .is_none()
    {
        let count = (state.render.r_segs.rw_stopx - start).idx();
        let lastopening = state.render.r_plane.lastopening;
        state.render.r_plane.openings[lastopening..lastopening + count]
            .copy_from_slice(&state.render.r_plane.ceilingclip[start.idx()..start.idx() + count]);
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprtopclip = Some(
            ClipArray::Openings(state.render.r_plane.lastopening as isize - start as isize),
        );
        state.render.r_plane.lastopening += (state.render.r_segs.rw_stopx - start).idx();
    }
    if (state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette & SIL_BOTTOM != 0
        || state.render.r_segs.maskedtexture)
        && state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p]
            .sprbottomclip
            .is_none()
    {
        let count = (state.render.r_segs.rw_stopx - start).idx();
        let lastopening = state.render.r_plane.lastopening;
        state.render.r_plane.openings[lastopening..lastopening + count]
            .copy_from_slice(&state.render.r_plane.floorclip[start.idx()..start.idx() + count]);
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].sprbottomclip = Some(
            ClipArray::Openings(state.render.r_plane.lastopening as isize - start as isize),
        );
        state.render.r_plane.lastopening += (state.render.r_segs.rw_stopx - start).idx();
    }
    if state.render.r_segs.maskedtexture
        && state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette & SIL_TOP == 0
    {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette |= SIL_TOP;
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].tsilheight = Fixed(INT_MIN);
    }
    if state.render.r_segs.maskedtexture
        && state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette & SIL_BOTTOM == 0
    {
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].silhouette |= SIL_BOTTOM;
        state.render.r_bsp.drawsegs[state.render.r_bsp.ds_p].bsilheight = Fixed(INT_MAX);
    }
}
pub const __SHRT_MAX__: i32 = 32767;
