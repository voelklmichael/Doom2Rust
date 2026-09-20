use crate::d_loop::net_update;
use crate::d_player::PlayerId;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::game_state::Render;
use crate::m_fixed::fixed_div;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::p_setup::PSetupState;
use crate::p_setup::SegId;
use crate::p_setup::SubsectorId;
use crate::r_bsp::clear_clip_segs;
use crate::r_bsp::clear_draw_segs;
use crate::r_bsp::render_bspnode;
use crate::r_bsp::NF_SUBSECTOR;
use crate::r_data::r_init_data;
use crate::r_defs::Node;
use crate::r_draw::init_buffer;
use crate::r_draw::init_translation_tables;
use crate::r_draw::RDrawState;
use crate::r_draw::{
    draw_column, draw_column_low, draw_fuzz_column, draw_fuzz_column_low, draw_span, draw_span_low,
    draw_translated_column, draw_translated_column_low,
};
use crate::r_plane::clear_planes;
use crate::r_plane::draw_planes;
use crate::r_segs::RSegsState;
use crate::r_sky::init_sky_map;
use crate::r_things::clear_sprites;
use crate::r_things::draw_masked;
use crate::tables::slope_div;
use crate::tables::Angle;
use crate::tables::ANG180;
use crate::tables::ANG270;
use crate::tables::ANG90;
use crate::tables::ANGLETOFINESHIFT;
use crate::tables::FINEANGLES;
use crate::tables::FINECOSINE;
use crate::tables::FINESINE;
use crate::tables::FINETANGENT;
use crate::tables::TANTOANGLE;

// A colormap is identified by its row index into r_data.colormaps (each row
// is 256 bytes) rather than a raw pointer into that Vec.
pub type ColormapId = i32;

// scalelight/zlight rows are picked by light level (see execute_set_view_size/
// init_light_tables); walllights/spritelights instead reference *which* row
// to use, since setup_frame can redirect them at scalelightfixed when the
// player has a fixed colormap active (invulnerability/light-amp goggles).
#[derive(Copy, Clone)]
pub enum LightRow48 {
    Normal(usize),
    Fixed,
}

pub struct RMainState {
    pub viewangleoffset: i32,
    pub fixedcolormap: Option<ColormapId>,
    pub centerx: i32,
    pub centery: i32,
    pub centerxfrac: Fixed,
    pub centeryfrac: Fixed,
    pub projection: Fixed,
    pub framecount: i32,
    pub sscount: i32,
    pub linecount: i32,
    pub loopcount: i32,
    pub viewx: Fixed,
    pub viewy: Fixed,
    pub viewz: Fixed,
    pub viewangle: Angle,
    pub viewcos: Fixed,
    pub viewsin: Fixed,
    pub viewplayer: PlayerId,
    pub detailshift: i32,
    pub clipangle: Angle,
    pub viewangletox: [i32; 4096],
    pub xtoviewangle: [Angle; 321],
    pub scalelight: [[ColormapId; 48]; 16],
    pub scalelightfixed: [ColormapId; 48],
    pub zlight: [[ColormapId; 128]; 16],
    pub extralight: i32,
    pub colfunc: Option<fn(&mut GameState)>,
    pub basecolfunc: Option<fn(&mut GameState)>,
    pub fuzzcolfunc: Option<fn(&mut GameState)>,
    pub transcolfunc: Option<fn(&mut GameState)>,
    pub spanfunc: Option<fn(&mut GameState)>,
    pub setsizeneeded: bool,
    pub setblocks: i32,
    pub setdetail: i32,
}

impl Default for RMainState {
    fn default() -> Self {
        Self::new()
    }
}

impl RMainState {
    pub fn new() -> Self {
        Self {
            viewangleoffset: 0,
            fixedcolormap: None,
            centerx: 0,
            centery: 0,
            centerxfrac: 0,
            centeryfrac: 0,
            projection: 0,
            framecount: 0,
            sscount: 0,
            linecount: 0,
            loopcount: 0,
            viewx: 0,
            viewy: 0,
            viewz: 0,
            viewangle: 0,
            viewcos: 0,
            viewsin: 0,
            viewplayer: PlayerId(0),
            detailshift: 0,
            clipangle: 0,
            viewangletox: [0; 4096],
            xtoviewangle: [0; 321],
            scalelight: [[0; 48]; 16],
            scalelightfixed: [0; 48],
            zlight: [[0; 128]; 16],
            extralight: 0,
            colfunc: None,
            basecolfunc: None,
            fuzzcolfunc: None,
            transcolfunc: None,
            spanfunc: None,
            setsizeneeded: false,
            setblocks: 0,
            setdetail: 0,
        }
    }

    pub fn light_row48(&self, row: LightRow48) -> &[ColormapId; 48] {
        match row {
            LightRow48::Normal(i) => &self.scalelight[i],
            LightRow48::Fixed => &self.scalelightfixed,
        }
    }
}

pub const SLOPEBITS: i32 = 11;
pub const DBITS: i32 = FRACBITS - SLOPEBITS;
pub const FIELDOFVIEW: i32 = 2048;
pub fn point_on_side(x: Fixed, y: Fixed, node: &Node) -> i32 {
    if node.dx == 0 {
        if x <= node.x {
            return i32::from(node.dy > 0);
        }
        return i32::from(node.dy < 0);
    }
    if node.dy == 0 {
        if y <= node.y {
            return i32::from(node.dx < 0);
        }
        return i32::from(node.dx > 0);
    }
    let dx = x - node.x;
    let dy = y - node.y;
    if (node.dy ^ node.dx ^ dx ^ dy) as u32 & 0x80000000 != 0 {
        if (node.dy ^ dx) as u32 & 0x80000000 != 0 {
            return 1;
        }
        return 0;
    }
    let left = fixed_mul(node.dy >> FRACBITS, dx);
    let right = fixed_mul(dy, node.dx >> FRACBITS);
    if right < left {
        return 0;
    }
    1
}
pub fn point_on_seg_side(p_setup: &PSetupState, x: Fixed, y: Fixed, line: SegId) -> i32 {
    let line_v1 = p_setup.vertexes[p_setup.seg(line).v1.0 as usize];
    let line_v2 = p_setup.vertexes[p_setup.seg(line).v2.0 as usize];
    let lx: Fixed = line_v1.x;
    let ly: Fixed = line_v1.y;
    let ldx: Fixed = line_v2.x - lx;
    let ldy: Fixed = line_v2.y - ly;
    if ldx == 0 {
        if x <= lx {
            return i32::from(ldy > 0);
        }
        return i32::from(ldy < 0);
    }
    if ldy == 0 {
        if y <= ly {
            return i32::from(ldx < 0);
        }
        return i32::from(ldx > 0);
    }
    let dx: Fixed = x - lx;
    let dy: Fixed = y - ly;
    if (ldy ^ ldx ^ dx ^ dy) as u32 & 0x80000000 != 0 {
        if (ldy ^ dx) as u32 & 0x80000000 != 0 {
            return 1;
        }
        return 0;
    }
    let left: Fixed = fixed_mul(ldy >> FRACBITS, dx);
    let right: Fixed = fixed_mul(dy, ldx >> FRACBITS);
    if right < left {
        return 0;
    }
    1
}
/// The angle of the vector `(x, y)`; 0 for the zero vector.
fn vector_to_angle(mut x: Fixed, mut y: Fixed) -> Angle {
    if x == 0 && y == 0 {
        return 0 as Angle;
    }
    if x >= 0 {
        if y >= 0 {
            if x > y {
                TANTOANGLE[slope_div(y as u32, x as u32) as usize]
            } else {
                ((ANG90 - 1) as Angle)
                    .wrapping_sub(TANTOANGLE[slope_div(x as u32, y as u32) as usize])
            }
        } else {
            y = -y;
            if x > y {
                TANTOANGLE[slope_div(y as u32, x as u32) as usize].wrapping_neg()
            } else {
                ANG270.wrapping_add(TANTOANGLE[slope_div(x as u32, y as u32) as usize])
            }
        }
    } else {
        x = -x;
        if y >= 0 {
            if x > y {
                ANG180
                    .wrapping_sub(1 as Angle)
                    .wrapping_sub(TANTOANGLE[slope_div(y as u32, x as u32) as usize])
            } else {
                (ANG90 as Angle).wrapping_add(TANTOANGLE[slope_div(x as u32, y as u32) as usize])
            }
        } else {
            y = -y;
            if x > y {
                ANG180.wrapping_add(TANTOANGLE[slope_div(y as u32, x as u32) as usize])
            } else {
                ANG270
                    .wrapping_sub(1 as Angle)
                    .wrapping_sub(TANTOANGLE[slope_div(x as u32, y as u32) as usize])
            }
        }
    }
}

/// The angle from the view point to `(x, y)`.
pub fn point_to_angle(r_main: &RMainState, x: Fixed, y: Fixed) -> Angle {
    vector_to_angle(x - r_main.viewx, y - r_main.viewy)
}
/// The angle of the line from `(x1, y1)` to `(x2, y2)`.
///
/// Vanilla's `R_PointToAngle2` does this by moving the view point to `(x1, y1)`, clobbering
/// `viewx`/`viewy`; nothing reads them before `R_SetupFrame` sets them again, so this leaves them.
pub fn point_to_angle2(x1: Fixed, y1: Fixed, x2: Fixed, y2: Fixed) -> Angle {
    vector_to_angle(x2 - x1, y2 - y1)
}
pub fn point_to_dist(r_main: &RMainState, x: Fixed, y: Fixed) -> Fixed {
    let mut dx: Fixed = (x - r_main.viewx).abs() as Fixed;
    let mut dy: Fixed = (y - r_main.viewy).abs() as Fixed;
    if dy > dx {
        core::mem::swap(&mut dx, &mut dy);
    }
    let frac: Fixed = if dx != 0 { fixed_div(dy, dx) } else { 0 };
    let angle: i32 = (TANTOANGLE[(frac >> DBITS) as usize].wrapping_add(ANG90 as Angle)
        >> ANGLETOFINESHIFT) as i32;
    let dist: Fixed = fixed_div(dx, FINESINE[angle as usize]);
    dist
}
pub fn scale_from_global_angle(r_main: &RMainState, r_segs: &RSegsState, visangle: Angle) -> Fixed {
    let mut scale: Fixed;

    let anglea: Angle = (ANG90 as Angle).wrapping_add(visangle.wrapping_sub(r_main.viewangle));
    let angleb: Angle = (ANG90 as Angle).wrapping_add(visangle.wrapping_sub(r_segs.rw_normalangle));
    let sinea: i32 = FINESINE[(anglea >> ANGLETOFINESHIFT) as usize];
    let sineb: i32 = FINESINE[(angleb >> ANGLETOFINESHIFT) as usize];
    let num: Fixed = fixed_mul(r_main.projection, sineb as Fixed) << r_main.detailshift;
    let den: i32 = fixed_mul(r_segs.rw_distance, sinea as Fixed);
    if den > num >> 16 {
        scale = fixed_div(num, den as Fixed);
        if scale > 64 * FRACUNIT {
            scale = (64 * FRACUNIT) as Fixed;
        } else if scale < 256 {
            scale = 256;
        }
    } else {
        scale = (64 * FRACUNIT) as Fixed;
    }
    scale
}
pub fn init_texture_mapping(r_draw: &RDrawState, r_main: &mut RMainState) {
    let mut i: i32;

    let focallength: Fixed = fixed_div(
        r_main.centerxfrac,
        FINETANGENT[(FINEANGLES / 4 + FIELDOFVIEW / 2) as usize],
    );
    for (i, &tangent) in FINETANGENT
        .iter()
        .enumerate()
        .take((FINEANGLES / 2) as usize)
    {
        let mut t: i32;

        if tangent > FRACUNIT * 2 {
            t = -1;
        } else if tangent < -FRACUNIT * 2 {
            t = r_draw.viewwidth + 1;
        } else {
            t = fixed_mul(tangent, focallength);
            t = (r_main.centerxfrac - t + FRACUNIT - 1) >> FRACBITS;
            if t < -1 {
                t = -1;
            } else if t > r_draw.viewwidth + 1 {
                t = r_draw.viewwidth + 1;
            }
        }
        r_main.viewangletox[i] = t;
    }
    for x in 0..=r_draw.viewwidth {
        i = 0;
        while r_main.viewangletox[i as usize] > x {
            i += 1;
        }
        r_main.xtoviewangle[x as usize] = ((i << ANGLETOFINESHIFT) - ANG90) as Angle;
    }
    for i in 0..(FINEANGLES / 2) as usize {
        if r_main.viewangletox[i] == -1 {
            r_main.viewangletox[i] = 0;
        } else if r_main.viewangletox[i] == r_draw.viewwidth + 1 {
            r_main.viewangletox[i] = r_draw.viewwidth;
        }
    }
    r_main.clipangle = r_main.xtoviewangle[0];
}
pub const DISTMAP: i32 = 2;
pub fn init_light_tables(r_main: &mut RMainState) {
    for i in 0..LIGHTLEVELS {
        let startmap: i32 = (LIGHTLEVELS - 1 - i) * 2 * NUMCOLORMAPS / LIGHTLEVELS;
        for j in 0..MAXLIGHTZ {
            let mut scale: i32 =
                fixed_div(SCREENWIDTH / 2 * FRACUNIT, (j as Fixed + 1) << LIGHTZSHIFT);
            scale >>= LIGHTSCALESHIFT;
            let mut level: i32 = startmap - scale / DISTMAP;
            if level < 0 {
                level = 0;
            }
            if level >= NUMCOLORMAPS {
                level = NUMCOLORMAPS - 1;
            }
            r_main.zlight[i as usize][j as usize] = level;
        }
    }
}
pub fn set_view_size(r_main: &mut RMainState, blocks: i32, detail: i32) {
    r_main.setsizeneeded = true;
    r_main.setblocks = blocks;
    r_main.setdetail = detail;
}
pub fn execute_set_view_size(render: &mut Render) {
    render.r_main.setsizeneeded = false;
    if render.r_main.setblocks == 11 {
        render.r_draw.scaledviewwidth = SCREENWIDTH;
        render.r_draw.viewheight = SCREENHEIGHT;
    } else {
        render.r_draw.scaledviewwidth = render.r_main.setblocks * 32;
        render.r_draw.viewheight = (render.r_main.setblocks * 168 / 10) & !7;
    }
    render.r_main.detailshift = render.r_main.setdetail;
    render.r_draw.viewwidth = render.r_draw.scaledviewwidth >> render.r_main.detailshift;
    render.r_main.centery = render.r_draw.viewheight / 2;
    render.r_main.centerx = render.r_draw.viewwidth / 2;
    render.r_main.centerxfrac = (render.r_main.centerx << FRACBITS) as Fixed;
    render.r_main.centeryfrac = (render.r_main.centery << FRACBITS) as Fixed;
    render.r_main.projection = render.r_main.centerxfrac;
    if render.r_main.detailshift == 0 {
        render.r_main.basecolfunc = Some(draw_column);
        render.r_main.colfunc = render.r_main.basecolfunc;
        render.r_main.fuzzcolfunc = Some(draw_fuzz_column);
        render.r_main.transcolfunc = Some(draw_translated_column);
        render.r_main.spanfunc = Some(draw_span);
    } else {
        render.r_main.basecolfunc = Some(draw_column_low);
        render.r_main.colfunc = render.r_main.basecolfunc;
        render.r_main.fuzzcolfunc = Some(draw_fuzz_column_low);
        render.r_main.transcolfunc = Some(draw_translated_column_low);
        render.r_main.spanfunc = Some(draw_span_low);
    }
    let scaledviewwidth = render.r_draw.scaledviewwidth;
    let viewheight = render.r_draw.viewheight;
    init_buffer(&mut render.r_draw, scaledviewwidth, viewheight);
    init_texture_mapping(&render.r_draw, &mut render.r_main);
    render.r_things.pspritescale = (FRACUNIT * render.r_draw.viewwidth / SCREENWIDTH) as Fixed;
    render.r_things.pspriteiscale = (FRACUNIT * SCREENWIDTH / render.r_draw.viewwidth) as Fixed;
    for i in 0..render.r_draw.viewwidth as usize {
        render.r_things.screenheightarray[i] = render.r_draw.viewheight as i16;
    }
    for i in 0..render.r_draw.viewheight {
        let mut dy: Fixed =
            (((i - render.r_draw.viewheight / 2) << FRACBITS) + FRACUNIT / 2) as Fixed;
        dy = dy.abs() as Fixed;
        render.r_plane.yslope[i as usize] = fixed_div(
            ((render.r_draw.viewwidth as Fixed) << render.r_main.detailshift) / 2 * FRACUNIT,
            dy,
        );
    }
    for i in 0..render.r_draw.viewwidth as usize {
        let cosadj: Fixed =
            FINECOSINE[(render.r_main.xtoviewangle[i] >> ANGLETOFINESHIFT) as usize].abs() as Fixed;
        render.r_plane.distscale[i] = fixed_div(FRACUNIT, cosadj);
    }
    for i in 0..LIGHTLEVELS {
        let startmap: i32 = (LIGHTLEVELS - 1 - i) * 2 * NUMCOLORMAPS / LIGHTLEVELS;
        for j in 0..MAXLIGHTSCALE {
            let mut level: i32 = startmap
                - j * SCREENWIDTH
                    / (render.r_draw.viewwidth << render.r_main.detailshift)
                    / DISTMAP;
            if level < 0 {
                level = 0;
            }
            if level >= NUMCOLORMAPS {
                level = NUMCOLORMAPS - 1;
            }
            render.r_main.scalelight[i as usize][j as usize] = level;
        }
    }
}
pub fn r_init(state: &mut GameState) {
    r_init_data(state);
    doom_print!(state.io.platform, ".");
    doom_print!(state.io.platform, ".");
    let (screenblocks, detail_level) = (state.ui.m_menu.screenblocks, state.ui.m_menu.detail_level);
    set_view_size(&mut state.render.r_main, screenblocks, detail_level);
    doom_print!(state.io.platform, ".");
    init_light_tables(&mut state.render.r_main);
    doom_print!(state.io.platform, ".");
    init_sky_map(&mut state.render.r_sky);
    init_translation_tables(&mut state.render.r_draw);
    doom_print!(state.io.platform, ".");
    state.render.r_main.framecount = 0;
}
pub fn point_in_subsector(p_setup: &PSetupState, x: Fixed, y: Fixed) -> SubsectorId {
    if p_setup.numnodes == 0 {
        return SubsectorId(0);
    }
    let mut nodenum = p_setup.numnodes - 1;
    while nodenum & NF_SUBSECTOR == 0 {
        let node = &p_setup.nodes[nodenum as usize];
        let side = point_on_side(x, y, node);
        nodenum = i32::from(node.children[side as usize]);
    }
    SubsectorId((nodenum & !NF_SUBSECTOR) as u32)
}
pub fn setup_frame(state: &mut GameState, player_id: PlayerId) {
    let player = state.game.g_game.player_mut(player_id);
    state.render.r_main.viewplayer = player_id;
    let (player_mo_id, extralight, viewz, fixedcolormap) = (
        player.mo.unwrap(),
        player.extralight,
        player.viewz,
        player.fixedcolormap,
    );
    let player_mo = state.world.p_mobj.mo(player_mo_id);
    state.render.r_main.viewx = player_mo.x;
    state.render.r_main.viewy = player_mo.y;
    state.render.r_main.viewangle = player_mo
        .angle
        .wrapping_add(state.render.r_main.viewangleoffset as Angle);
    state.render.r_main.extralight = extralight;
    state.render.r_main.viewz = viewz;
    state.render.r_main.viewsin =
        FINESINE[(state.render.r_main.viewangle >> ANGLETOFINESHIFT) as usize];
    state.render.r_main.viewcos =
        FINECOSINE[(state.render.r_main.viewangle >> ANGLETOFINESHIFT) as usize];
    state.render.r_main.sscount = 0;
    if fixedcolormap != 0 {
        let colormap = fixedcolormap;
        state.render.r_main.fixedcolormap = Some(colormap);
        state.render.r_segs.walllights = LightRow48::Fixed;
        for i in 0..(MAXLIGHTSCALE as usize) {
            state.render.r_main.scalelightfixed[i] = colormap;
        }
    } else {
        state.render.r_main.fixedcolormap = None;
    }
    state.render.r_main.framecount += 1;
    state.world.p_setup.validcount += 1;
}
pub fn render_player_view(state: &mut GameState, player_id: PlayerId) {
    setup_frame(state, player_id);
    clear_clip_segs(&mut state.render.r_bsp, &state.render.r_draw);
    clear_draw_segs(&mut state.render.r_bsp);
    clear_planes(
        &state.render.r_draw,
        &state.render.r_main,
        &mut state.render.r_plane,
    );
    clear_sprites(&mut state.render.r_things);
    net_update(state);
    let root_bspnum = state.world.p_setup.numnodes - 1;
    render_bspnode(state, root_bspnum);
    net_update(state);
    draw_planes(state);
    net_update(state);
    draw_masked(state);
    net_update(state);
}
pub const LIGHTLEVELS: i32 = 16;
pub const MAXLIGHTSCALE: i32 = 48;
pub const LIGHTSCALESHIFT: i32 = 12;
pub const MAXLIGHTZ: i32 = 128;
pub const LIGHTZSHIFT: i32 = 20;
pub const NUMCOLORMAPS: i32 = 32;
pub const LIGHTSEGSHIFT: i32 = 4;
