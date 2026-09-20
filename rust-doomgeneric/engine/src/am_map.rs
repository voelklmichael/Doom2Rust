use crate::d_event::EvType;
use crate::d_event::Event;
use crate::filesystem::DoomFileSystem;
use crate::g_game::GGameState;
use crate::i_video::IVideoState;
use crate::p_mobj::LineFlags;
use crate::p_mobj::PMobjState;
use crate::p_setup::PSetupState;
use crate::platform::DoomPlatform;
use crate::w_wad::WWadState;
use alloc::string::ToString;

use crate::d_player::PlayerId;
use crate::d_player::PowerType;
use crate::doomdef::MAXPLAYERS;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::m_cheat::cht_check_cheat;
use crate::m_cheat::CheatSeq;
use crate::m_fixed::fixed_div;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::m_fixed::INT_MAX;
use crate::p_maputl::MAPBLOCKUNITS;
use crate::v_video::cache_patch_num;
use crate::v_video::Screen;

use crate::st_stuff::st_responder;

use crate::tables::Angle;
use crate::tables::ANGLETOFINESHIFT;
use crate::tables::FINECOSINE;
use crate::tables::FINESINE;
use crate::v_video::draw_patch;
use crate::v_video::mark_rect;
use crate::w_wad::{get_num_for_name, lump_bytes, release_lump_num};

pub struct AmMapState {
    pub cheating: i32,
    pub grid: bool,
    pub leveljuststarted: bool,
    pub automapactive: bool,
    pub f_x: i32,
    pub f_y: i32,
    pub f_w: i32,
    pub f_h: i32,
    pub lightlev: i32,
    pub amclock: i32,
    pub m_paninc: MPoint,
    pub mtof_zoommul: Fixed,
    pub ftom_zoommul: Fixed,
    pub m_y: Fixed,
    pub m_x: Fixed,
    pub m_x2: Fixed,
    pub m_y2: Fixed,
    pub m_w: Fixed,
    pub m_h: Fixed,
    pub min_x: Fixed,
    pub min_y: Fixed,
    pub max_x: Fixed,
    pub max_y: Fixed,
    pub max_w: Fixed,
    pub max_h: Fixed,
    pub min_w: Fixed,
    pub min_h: Fixed,
    pub min_scale_mtof: Fixed,
    pub max_scale_mtof: Fixed,
    pub old_m_h: Fixed,
    pub old_m_w: Fixed,
    pub old_m_y: Fixed,
    pub old_m_x: Fixed,
    pub f_oldloc: MPoint,
    pub scale_mtof: Fixed,
    pub scale_ftom: Fixed,
    pub plr: PlayerId,
    pub marknums: [i32; 10],
    pub markpoints: [MPoint; 10],
    pub markpointnum: i32,
    pub followplayer: bool,
    pub cheat_amap: CheatSeq,
    pub stopped: bool,
    pub am_start_lastlevel: i32,
    pub am_start_lastepisode: i32,
    pub am_responder_bigstate: bool,
    pub am_drawfline_fuck: i32,
    pub am_updatelightlev_nexttic: i32,
    pub am_updatelightlev_litelevelscnt: i32,
}

impl Default for AmMapState {
    fn default() -> Self {
        Self::new()
    }
}

impl AmMapState {
    pub const fn new() -> Self {
        Self {
            cheating: 0,
            grid: false,
            leveljuststarted: true,
            automapactive: false,
            f_x: 0,
            f_y: 0,
            f_w: 0,
            f_h: 0,
            lightlev: 0,
            amclock: 0,
            m_paninc: MPoint { x: 0, y: 0 },
            mtof_zoommul: 0,
            ftom_zoommul: 0,
            m_y: 0,
            m_x: 0,
            m_x2: 0,
            m_y2: 0,
            m_w: 0,
            m_h: 0,
            min_x: 0,
            min_y: 0,
            max_x: 0,
            max_y: 0,
            max_w: 0,
            max_h: 0,
            min_w: 0,
            min_h: 0,
            min_scale_mtof: 0,
            max_scale_mtof: 0,
            old_m_h: 0,
            old_m_w: 0,
            old_m_y: 0,
            old_m_x: 0,
            f_oldloc: MPoint { x: 0, y: 0 },
            scale_mtof: INITSCALEMTOF as Fixed,
            scale_ftom: 0,
            plr: PlayerId(0),
            marknums: [-1; 10],
            markpoints: [MPoint { x: 0, y: 0 }; 10],
            markpointnum: 0,
            followplayer: true,
            cheat_amap: CheatSeq::new("iddt", 0),
            stopped: true,
            am_start_lastlevel: -1,
            am_start_lastepisode: -1,
            am_responder_bigstate: false,
            am_drawfline_fuck: 0,
            am_updatelightlev_nexttic: 0,
            am_updatelightlev_litelevelscnt: 0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct MPoint {
    pub x: Fixed,
    pub y: Fixed,
}
#[derive(Copy, Clone)]
pub struct MLine {
    pub a: MPoint,
    pub b: MPoint,
}
#[derive(Copy, Clone)]
pub struct FLine {
    pub a: FPoint,
    pub b: FPoint,
}
#[derive(Copy, Clone)]
pub struct FPoint {
    pub x: i32,
    pub y: i32,
}
bitflags::bitflags! {
    /// The sides of a rectangle a point lies outside of (a Cohen-Sutherland
    /// "outcode"); [`clip_mline`] uses it to clip automap lines to the screen.
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    struct Outcode: u8 {
        const LEFT = 1;
        const RIGHT = 2;
        const BOTTOM = 4;
        const TOP = 8;
    }
}

impl Outcode {
    /// Which sides of the `width` x `height` screen (origin at the top left)
    /// `point` is outside of.
    fn of_screen_point(point: FPoint, width: i32, height: i32) -> Self {
        let mut code = Self::empty();
        if point.y < 0 {
            code |= Self::TOP;
        } else if point.y >= height {
            code |= Self::BOTTOM;
        }
        if point.x < 0 {
            code |= Self::LEFT;
        } else if point.x >= width {
            code |= Self::RIGHT;
        }
        code
    }
}
pub const AM_MSGHEADER: i32 = (('a' as i32) << 24) + (('m' as i32) << 16);
pub const AM_MSGENTERED: i32 = AM_MSGHEADER | ('e' as i32) << 8;
pub const AM_MSGEXITED: i32 = AM_MSGHEADER | ('x' as i32) << 8;
pub const REDS: i32 = 256 - 5 * 16;
pub const REDRANGE: i32 = 16;
pub const GREENS: i32 = 7 * 16;
pub const GRAYS: i32 = 6 * 16;
pub const GRAYSRANGE: i32 = 16;
pub const BROWNS: i32 = 4 * 16;
pub const YELLOWS: i32 = 256 - 32 + 7;
pub const BLACK: i32 = 0;
pub const WHITE: i32 = 256 - 47;
pub const BACKGROUND: i32 = BLACK;
pub const WALLCOLORS: i32 = REDS;
pub const WALLRANGE: i32 = REDRANGE;
pub const TSWALLCOLORS: i32 = GRAYS;
pub const FDWALLCOLORS: i32 = BROWNS;
pub const CDWALLCOLORS: i32 = YELLOWS;
pub const THINGCOLORS: i32 = GREENS;
pub const SECRETWALLCOLORS: i32 = WALLCOLORS;
pub const GRIDCOLORS: i32 = GRAYS + GRAYSRANGE / 2;
pub const XHAIRCOLORS: i32 = GRAYS;
pub const AM_NUMMARKPOINTS: i32 = 10;
pub const INITSCALEMTOF: f64 = 0.2f64 * FRACUNIT as f64;
pub const M_ZOOMIN: i32 = (1.02f64 * FRACUNIT as f64) as i32;
pub const M_ZOOMOUT: i32 = (FRACUNIT as f64 / 1.02f64) as i32;
pub const R_0: i32 = 8 * 16 * FRACUNIT / 7;
pub static PLAYER_ARROW: [MLine; 7] = [
    MLine {
        a: MPoint {
            x: -R_0 + R_0 / 8,
            y: 0,
        },
        b: MPoint { x: R_0, y: 0 },
    },
    MLine {
        a: MPoint { x: R_0, y: 0 },
        b: MPoint {
            x: R_0 - R_0 / 2,
            y: R_0 / 4,
        },
    },
    MLine {
        a: MPoint { x: R_0, y: 0 },
        b: MPoint {
            x: R_0 - R_0 / 2,
            y: -R_0 / 4,
        },
    },
    MLine {
        a: MPoint {
            x: -R_0 + R_0 / 8,
            y: 0,
        },
        b: MPoint {
            x: -R_0 - R_0 / 8,
            y: R_0 / 4,
        },
    },
    MLine {
        a: MPoint {
            x: -R_0 + R_0 / 8,
            y: 0,
        },
        b: MPoint {
            x: -R_0 - R_0 / 8,
            y: -R_0 / 4,
        },
    },
    MLine {
        a: MPoint {
            x: -R_0 + 3 * R_0 / 8,
            y: 0,
        },
        b: MPoint {
            x: -R_0 + R_0 / 8,
            y: R_0 / 4,
        },
    },
    MLine {
        a: MPoint {
            x: -R_0 + 3 * R_0 / 8,
            y: 0,
        },
        b: MPoint {
            x: -R_0 + R_0 / 8,
            y: -R_0 / 4,
        },
    },
];
pub const R_1: i32 = 8 * 16 * FRACUNIT / 7;
pub static CHEAT_PLAYER_ARROW: [MLine; 16] = [
    MLine {
        a: MPoint {
            x: -R_1 + R_1 / 8,
            y: 0,
        },
        b: MPoint { x: R_1, y: 0 },
    },
    MLine {
        a: MPoint { x: R_1, y: 0 },
        b: MPoint {
            x: R_1 - R_1 / 2,
            y: R_1 / 6,
        },
    },
    MLine {
        a: MPoint { x: R_1, y: 0 },
        b: MPoint {
            x: R_1 - R_1 / 2,
            y: -R_1 / 6,
        },
    },
    MLine {
        a: MPoint {
            x: -R_1 + R_1 / 8,
            y: 0,
        },
        b: MPoint {
            x: -R_1 - R_1 / 8,
            y: R_1 / 6,
        },
    },
    MLine {
        a: MPoint {
            x: -R_1 + R_1 / 8,
            y: 0,
        },
        b: MPoint {
            x: -R_1 - R_1 / 8,
            y: -R_1 / 6,
        },
    },
    MLine {
        a: MPoint {
            x: -R_1 + 3 * R_1 / 8,
            y: 0,
        },
        b: MPoint {
            x: -R_1 + R_1 / 8,
            y: R_1 / 6,
        },
    },
    MLine {
        a: MPoint {
            x: -R_1 + 3 * R_1 / 8,
            y: 0,
        },
        b: MPoint {
            x: -R_1 + R_1 / 8,
            y: -R_1 / 6,
        },
    },
    MLine {
        a: MPoint { x: -R_1 / 2, y: 0 },
        b: MPoint {
            x: -R_1 / 2,
            y: -R_1 / 6,
        },
    },
    MLine {
        a: MPoint {
            x: -R_1 / 2,
            y: -R_1 / 6,
        },
        b: MPoint {
            x: -R_1 / 2 + R_1 / 6,
            y: -R_1 / 6,
        },
    },
    MLine {
        a: MPoint {
            x: -R_1 / 2 + R_1 / 6,
            y: -R_1 / 6,
        },
        b: MPoint {
            x: -R_1 / 2 + R_1 / 6,
            y: R_1 / 4,
        },
    },
    MLine {
        a: MPoint { x: -R_1 / 6, y: 0 },
        b: MPoint {
            x: -R_1 / 6,
            y: -R_1 / 6,
        },
    },
    MLine {
        a: MPoint {
            x: -R_1 / 6,
            y: -R_1 / 6,
        },
        b: MPoint { x: 0, y: -R_1 / 6 },
    },
    MLine {
        a: MPoint { x: 0, y: -R_1 / 6 },
        b: MPoint { x: 0, y: R_1 / 4 },
    },
    MLine {
        a: MPoint {
            x: R_1 / 6,
            y: R_1 / 4,
        },
        b: MPoint {
            x: R_1 / 6,
            y: -R_1 / 7,
        },
    },
    MLine {
        a: MPoint {
            x: R_1 / 6,
            y: -R_1 / 7,
        },
        b: MPoint {
            x: R_1 / 6 + R_1 / 32,
            y: -R_1 / 7 - R_1 / 32,
        },
    },
    MLine {
        a: MPoint {
            x: R_1 / 6 + R_1 / 32,
            y: -R_1 / 7 - R_1 / 32,
        },
        b: MPoint {
            x: R_1 / 6 + R_1 / 10,
            y: -R_1 / 7,
        },
    },
];
pub const R: i32 = 1 << FRACBITS;
pub static THINTRIANGLE_GUY: [MLine; 3] = [
    MLine {
        a: MPoint {
            x: (-0.5f64 * R as f64) as Fixed,
            y: (-0.7f64 * R as f64) as Fixed,
        },
        b: MPoint {
            x: 1 << FRACBITS,
            y: 0,
        },
    },
    MLine {
        a: MPoint {
            x: 1 << FRACBITS,
            y: 0,
        },
        b: MPoint {
            x: (-0.5f64 * R as f64) as Fixed,
            y: (0.7f64 * R as f64) as Fixed,
        },
    },
    MLine {
        a: MPoint {
            x: (-0.5f64 * R as f64) as Fixed,
            y: (0.7f64 * R as f64) as Fixed,
        },
        b: MPoint {
            x: (-0.5f64 * R as f64) as Fixed,
            y: (-0.7f64 * R as f64) as Fixed,
        },
    },
];
static FINIT_WIDTH: i32 = SCREENWIDTH;
static FINIT_HEIGHT: i32 = SCREENHEIGHT - 32;
pub fn activate_new_scale(am_map: &mut AmMapState) {
    am_map.m_x += am_map.m_w / 2;
    am_map.m_y += am_map.m_h / 2;
    am_map.m_w = fixed_mul((am_map.f_w as Fixed) << 16, am_map.scale_ftom);
    am_map.m_h = fixed_mul((am_map.f_h as Fixed) << 16, am_map.scale_ftom);
    am_map.m_x -= am_map.m_w / 2;
    am_map.m_y -= am_map.m_h / 2;
    am_map.m_x2 = am_map.m_x + am_map.m_w;
    am_map.m_y2 = am_map.m_y + am_map.m_h;
}
pub fn save_scale_and_loc(am_map: &mut AmMapState) {
    am_map.old_m_x = am_map.m_x;
    am_map.old_m_y = am_map.m_y;
    am_map.old_m_w = am_map.m_w;
    am_map.old_m_h = am_map.m_h;
}
pub fn restore_scale_and_loc(
    am_map: &mut AmMapState,
    g_game: &mut GGameState,
    p_mobj: &PMobjState,
) {
    am_map.m_w = am_map.old_m_w;
    am_map.m_h = am_map.old_m_h;
    if am_map.followplayer {
        let plr_mo_id = g_game.player_mut(am_map.plr).mo.unwrap();
        let plr_mo = p_mobj.mo(plr_mo_id);
        am_map.m_x = (plr_mo.x - am_map.m_w / 2) as Fixed;
        am_map.m_y = (plr_mo.y - am_map.m_h / 2) as Fixed;
    } else {
        am_map.m_x = am_map.old_m_x;
        am_map.m_y = am_map.old_m_y;
    }
    am_map.m_x2 = am_map.m_x + am_map.m_w;
    am_map.m_y2 = am_map.m_y + am_map.m_h;
    am_map.scale_mtof = fixed_div((am_map.f_w as Fixed) << FRACBITS, am_map.m_w);
    am_map.scale_ftom = fixed_div(FRACUNIT, am_map.scale_mtof);
}
pub fn add_mark(am_map: &mut AmMapState) {
    am_map.markpoints[am_map.markpointnum as usize].x = (am_map.m_x + am_map.m_w / 2) as Fixed;
    am_map.markpoints[am_map.markpointnum as usize].y = (am_map.m_y + am_map.m_h / 2) as Fixed;
    am_map.markpointnum = (am_map.markpointnum + 1) % AM_NUMMARKPOINTS;
}
pub fn find_min_max_boundaries(am_map: &mut AmMapState, p_setup: &PSetupState) {
    am_map.min_y = INT_MAX as Fixed;
    am_map.min_x = am_map.min_y;
    am_map.max_y = -INT_MAX as Fixed;
    am_map.max_x = am_map.max_y;
    for i in 0..(p_setup.numvertexes as usize) {
        let v = p_setup.vertexes[i];
        if v.x < am_map.min_x {
            am_map.min_x = v.x;
        } else if v.x > am_map.max_x {
            am_map.max_x = v.x;
        }
        if v.y < am_map.min_y {
            am_map.min_y = v.y;
        } else if v.y > am_map.max_y {
            am_map.max_y = v.y;
        }
    }
    am_map.max_w = am_map.max_x - am_map.min_x;
    am_map.max_h = am_map.max_y - am_map.min_y;
    am_map.min_w = (2 * 16 * FRACUNIT) as Fixed;
    am_map.min_h = (2 * 16 * FRACUNIT) as Fixed;
    let a: Fixed = fixed_div((am_map.f_w as Fixed) << FRACBITS, am_map.max_w);
    let b: Fixed = fixed_div((am_map.f_h as Fixed) << FRACBITS, am_map.max_h);
    am_map.min_scale_mtof = if a < b { a } else { b };
    am_map.max_scale_mtof = fixed_div((am_map.f_h as Fixed) << FRACBITS, 2 * 16 * FRACUNIT);
}
pub fn change_window_loc(am_map: &mut AmMapState) {
    if am_map.m_paninc.x != 0 || am_map.m_paninc.y != 0 {
        am_map.followplayer = false;
        am_map.f_oldloc.x = INT_MAX as Fixed;
    }
    am_map.m_x += am_map.m_paninc.x;
    am_map.m_y += am_map.m_paninc.y;
    if am_map.m_x + am_map.m_w / 2 > am_map.max_x {
        am_map.m_x = (am_map.max_x - am_map.m_w / 2) as Fixed;
    } else if (am_map.m_x + am_map.m_w / 2) < am_map.min_x {
        am_map.m_x = (am_map.min_x - am_map.m_w / 2) as Fixed;
    }
    if am_map.m_y + am_map.m_h / 2 > am_map.max_y {
        am_map.m_y = (am_map.max_y - am_map.m_h / 2) as Fixed;
    } else if (am_map.m_y + am_map.m_h / 2) < am_map.min_y {
        am_map.m_y = (am_map.min_y - am_map.m_h / 2) as Fixed;
    }
    am_map.m_x2 = am_map.m_x + am_map.m_w;
    am_map.m_y2 = am_map.m_y + am_map.m_h;
}
pub fn am_init_variables(state: &mut GameState) {
    const ST_NOTIFY: Event = Event {
        kind: EvType::Keyup,
        data1: AM_MSGENTERED,
        data2: 0,
        data3: 0,
        data4: 0,
    };
    state.ui.am_map.automapactive = true;
    state.ui.am_map.f_oldloc.x = INT_MAX as Fixed;
    state.ui.am_map.amclock = 0;
    state.ui.am_map.lightlev = 0;
    state.ui.am_map.m_paninc.y = 0;
    state.ui.am_map.m_paninc.x = state.ui.am_map.m_paninc.y;
    state.ui.am_map.ftom_zoommul = FRACUNIT as Fixed;
    state.ui.am_map.mtof_zoommul = FRACUNIT as Fixed;
    state.ui.am_map.m_w = fixed_mul(
        (state.ui.am_map.f_w as Fixed) << 16,
        state.ui.am_map.scale_ftom,
    );
    state.ui.am_map.m_h = fixed_mul(
        (state.ui.am_map.f_h as Fixed) << 16,
        state.ui.am_map.scale_ftom,
    );
    if state.game.g_game.playeringame[state.game.g_game.consoleplayer] {
        state.ui.am_map.plr = state.game.g_game.consoleplayer;
    } else {
        state.ui.am_map.plr = PlayerId(0);
        for pnum in 0..MAXPLAYERS {
            if state.game.g_game.playeringame[pnum as usize] {
                state.ui.am_map.plr = PlayerId(pnum as u8);
                break;
            }
        }
    }
    let plr_mo_id = state
        .game
        .g_game
        .player_mut(state.ui.am_map.plr)
        .mo
        .unwrap();
    let plr_mo = state.world.p_mobj.mo(plr_mo_id);
    state.ui.am_map.m_x = (plr_mo.x - state.ui.am_map.m_w / 2) as Fixed;
    state.ui.am_map.m_y = (plr_mo.y - state.ui.am_map.m_h / 2) as Fixed;
    change_window_loc(&mut state.ui.am_map);
    state.ui.am_map.old_m_x = state.ui.am_map.m_x;
    state.ui.am_map.old_m_y = state.ui.am_map.m_y;
    state.ui.am_map.old_m_w = state.ui.am_map.m_w;
    state.ui.am_map.old_m_h = state.ui.am_map.m_h;
    st_responder(state, &ST_NOTIFY);
}
pub fn load_pics(am_map: &mut AmMapState, fs: &dyn DoomFileSystem, w_wad: &mut WWadState) {
    for i in 0..10 {
        let namebuf = format!("AMMNUM{i}");
        let lumpnum = get_num_for_name(w_wad, &namebuf);
        lump_bytes(fs, w_wad, lumpnum);
        am_map.marknums[i] = lumpnum;
    }
}
pub fn unload_pics(am_map: &AmMapState, w_wad: &WWadState) {
    for i in 0..10 {
        release_lump_num(w_wad, am_map.marknums[i]);
    }
}
pub fn clear_marks(am_map: &mut AmMapState) {
    for i in 0..(AM_NUMMARKPOINTS as usize) {
        am_map.markpoints[i].x = -1;
    }
    am_map.markpointnum = 0;
}
pub fn level_init(am_map: &mut AmMapState, p_setup: &PSetupState) {
    am_map.leveljuststarted = false;
    am_map.f_y = 0;
    am_map.f_x = am_map.f_y;
    am_map.f_w = FINIT_WIDTH;
    am_map.f_h = FINIT_HEIGHT;
    clear_marks(am_map);
    find_min_max_boundaries(am_map, p_setup);
    am_map.scale_mtof = fixed_div(am_map.min_scale_mtof, (0.7f64 * FRACUNIT as f64) as Fixed);
    if am_map.scale_mtof > am_map.max_scale_mtof {
        am_map.scale_mtof = am_map.min_scale_mtof;
    }
    am_map.scale_ftom = fixed_div(FRACUNIT, am_map.scale_mtof);
}
pub fn am_stop(state: &mut GameState) {
    const ST_NOTIFY: Event = Event {
        kind: EvType::Keydown,
        data1: EvType::Keyup as i32,
        data2: AM_MSGEXITED,
        data3: 0,
        data4: 0,
    };
    unload_pics(&state.ui.am_map, &state.assets.w_wad);
    state.ui.am_map.automapactive = false;
    st_responder(state, &ST_NOTIFY);
    state.ui.am_map.stopped = true;
}
pub fn am_start(state: &mut GameState) {
    if !state.ui.am_map.stopped {
        am_stop(state);
    }
    state.ui.am_map.stopped = false;
    if state.ui.am_map.am_start_lastlevel != state.game.g_game.gamemap
        || state.ui.am_map.am_start_lastepisode != state.game.g_game.gameepisode
    {
        level_init(&mut state.ui.am_map, &state.world.p_setup);
        state.ui.am_map.am_start_lastlevel = state.game.g_game.gamemap;
        state.ui.am_map.am_start_lastepisode = state.game.g_game.gameepisode;
    }
    am_init_variables(state);
    load_pics(
        &mut state.ui.am_map,
        &*state.assets.fs,
        &mut state.assets.w_wad,
    );
}
pub fn min_out_window_scale(am_map: &mut AmMapState) {
    am_map.scale_mtof = am_map.min_scale_mtof;
    am_map.scale_ftom = fixed_div(FRACUNIT, am_map.scale_mtof);
    activate_new_scale(am_map);
}
pub fn max_out_window_scale(am_map: &mut AmMapState) {
    am_map.scale_mtof = am_map.max_scale_mtof;
    am_map.scale_ftom = fixed_div(FRACUNIT, am_map.scale_mtof);
    activate_new_scale(am_map);
}
pub fn am_responder(state: &mut GameState, ev: &Event) -> bool {
    let key: i32;
    let mut rc: bool = false;
    if !state.ui.am_map.automapactive {
        if ev.kind == EvType::Keydown && ev.data1 == state.game.m_controls.key_map_toggle {
            am_start(state);
            state.game.g_game.viewactive = false;
            rc = true;
        }
    } else if ev.kind == EvType::Keydown {
        rc = true;
        key = ev.data1;
        if key == state.game.m_controls.key_map_east {
            if state.ui.am_map.followplayer {
                rc = false;
            } else {
                state.ui.am_map.m_paninc.x = fixed_mul((4) << 16, state.ui.am_map.scale_ftom);
            }
        } else if key == state.game.m_controls.key_map_west {
            if state.ui.am_map.followplayer {
                rc = false;
            } else {
                state.ui.am_map.m_paninc.x = -fixed_mul((4) << 16, state.ui.am_map.scale_ftom);
            }
        } else if key == state.game.m_controls.key_map_north {
            if state.ui.am_map.followplayer {
                rc = false;
            } else {
                state.ui.am_map.m_paninc.y = fixed_mul((4) << 16, state.ui.am_map.scale_ftom);
            }
        } else if key == state.game.m_controls.key_map_south {
            if state.ui.am_map.followplayer {
                rc = false;
            } else {
                state.ui.am_map.m_paninc.y = -fixed_mul((4) << 16, state.ui.am_map.scale_ftom);
            }
        } else if key == state.game.m_controls.key_map_zoomout {
            state.ui.am_map.mtof_zoommul = M_ZOOMOUT as Fixed;
            state.ui.am_map.ftom_zoommul = M_ZOOMIN as Fixed;
        } else if key == state.game.m_controls.key_map_zoomin {
            state.ui.am_map.mtof_zoommul = M_ZOOMIN as Fixed;
            state.ui.am_map.ftom_zoommul = M_ZOOMOUT as Fixed;
        } else if key == state.game.m_controls.key_map_toggle {
            state.ui.am_map.am_responder_bigstate = false;
            state.game.g_game.viewactive = true;
            am_stop(state);
        } else if key == state.game.m_controls.key_map_maxzoom {
            state.ui.am_map.am_responder_bigstate = !state.ui.am_map.am_responder_bigstate;
            if state.ui.am_map.am_responder_bigstate {
                save_scale_and_loc(&mut state.ui.am_map);
                min_out_window_scale(&mut state.ui.am_map);
            } else {
                restore_scale_and_loc(
                    &mut state.ui.am_map,
                    &mut state.game.g_game,
                    &state.world.p_mobj,
                );
            }
        } else if key == state.game.m_controls.key_map_follow {
            state.ui.am_map.followplayer = !state.ui.am_map.followplayer;
            state.ui.am_map.f_oldloc.x = INT_MAX as Fixed;
            if state.ui.am_map.followplayer {
                state.game.g_game.player_mut(state.ui.am_map.plr).message =
                    Some("Follow Mode ON".to_string());
            } else {
                state.game.g_game.player_mut(state.ui.am_map.plr).message =
                    Some("Follow Mode OFF".to_string());
            }
        } else if key == state.game.m_controls.key_map_grid {
            state.ui.am_map.grid = !state.ui.am_map.grid;
            if state.ui.am_map.grid {
                state.game.g_game.player_mut(state.ui.am_map.plr).message =
                    Some("Grid ON".to_string());
            } else {
                state.game.g_game.player_mut(state.ui.am_map.plr).message =
                    Some("Grid OFF".to_string());
            }
        } else if key == state.game.m_controls.key_map_mark {
            state.game.g_game.player_mut(state.ui.am_map.plr).message =
                Some(format!("Marked Spot {}", state.ui.am_map.markpointnum));
            add_mark(&mut state.ui.am_map);
        } else if key == state.game.m_controls.key_map_clearmark {
            clear_marks(&mut state.ui.am_map);
            state.game.g_game.player_mut(state.ui.am_map.plr).message =
                Some("All Marks Cleared".to_string());
        } else {
            rc = false;
        }
        if state.game.g_game.deathmatch == 0
            && cht_check_cheat(&mut state.ui.am_map.cheat_amap, ev.data2 as u8)
        {
            rc = false;
            state.ui.am_map.cheating = (state.ui.am_map.cheating + 1) % 3;
        }
    } else if ev.kind == EvType::Keyup {
        rc = false;
        key = ev.data1;
        if key == state.game.m_controls.key_map_east || key == state.game.m_controls.key_map_west {
            if !state.ui.am_map.followplayer {
                state.ui.am_map.m_paninc.x = 0;
            }
        } else if key == state.game.m_controls.key_map_north
            || key == state.game.m_controls.key_map_south
        {
            if !state.ui.am_map.followplayer {
                state.ui.am_map.m_paninc.y = 0;
            }
        } else if key == state.game.m_controls.key_map_zoomout
            || key == state.game.m_controls.key_map_zoomin
        {
            state.ui.am_map.mtof_zoommul = FRACUNIT as Fixed;
            state.ui.am_map.ftom_zoommul = FRACUNIT as Fixed;
        }
    }
    rc
}
pub fn change_window_scale(am_map: &mut AmMapState) {
    am_map.scale_mtof = fixed_mul(am_map.scale_mtof, am_map.mtof_zoommul);
    am_map.scale_ftom = fixed_div(FRACUNIT, am_map.scale_mtof);
    if am_map.scale_mtof < am_map.min_scale_mtof {
        min_out_window_scale(am_map);
    } else if am_map.scale_mtof > am_map.max_scale_mtof {
        max_out_window_scale(am_map);
    } else {
        activate_new_scale(am_map);
    }
}
pub fn do_follow_player(am_map: &mut AmMapState, g_game: &mut GGameState, p_mobj: &PMobjState) {
    let plr_mo_id = g_game.player_mut(am_map.plr).mo.unwrap();
    let plr_mo = p_mobj.mo(plr_mo_id);
    let (plr_x, plr_y) = (plr_mo.x, plr_mo.y);
    if am_map.f_oldloc.x != plr_x || am_map.f_oldloc.y != plr_y {
        am_map.m_x = (fixed_mul(
            (fixed_mul(plr_x, am_map.scale_mtof) >> 16) << 16,
            am_map.scale_ftom,
        ) - am_map.m_w / 2) as Fixed;
        am_map.m_y = (fixed_mul(
            (fixed_mul(plr_y, am_map.scale_mtof) >> 16) << 16,
            am_map.scale_ftom,
        ) - am_map.m_h / 2) as Fixed;
        am_map.m_x2 = am_map.m_x + am_map.m_w;
        am_map.m_y2 = am_map.m_y + am_map.m_h;
        am_map.f_oldloc.x = plr_x;
        am_map.f_oldloc.y = plr_y;
    }
}
pub fn am_ticker(am_map: &mut AmMapState, g_game: &mut GGameState, p_mobj: &PMobjState) {
    if !am_map.automapactive {
        return;
    }
    am_map.amclock += 1;
    if am_map.followplayer {
        do_follow_player(am_map, g_game, p_mobj);
    }
    if am_map.ftom_zoommul != FRACUNIT {
        change_window_scale(am_map);
    }
    if am_map.m_paninc.x != 0 || am_map.m_paninc.y != 0 {
        change_window_loc(am_map);
    }
}
pub fn clear_fb(am_map: &AmMapState, i_video: &mut IVideoState, color: i32) {
    let len = (am_map.f_w * am_map.f_h) as usize;
    i_video.i_video_buffer[..len].fill(color as u8);
}
pub fn clip_mline(am_map: &AmMapState, ml: &MLine, fl: &mut FLine) -> bool {
    let mut outcode1 = Outcode::empty();
    let mut outcode2 = Outcode::empty();
    let mut tmp: FPoint = FPoint { x: 0, y: 0 };
    let mut dx: i32;
    let mut dy: i32;
    if ml.a.y > am_map.m_y2 {
        outcode1 = Outcode::TOP;
    } else if ml.a.y < am_map.m_y {
        outcode1 = Outcode::BOTTOM;
    }
    if ml.b.y > am_map.m_y2 {
        outcode2 = Outcode::TOP;
    } else if ml.b.y < am_map.m_y {
        outcode2 = Outcode::BOTTOM;
    }
    if outcode1.intersects(outcode2) {
        return false;
    }
    if ml.a.x < am_map.m_x {
        outcode1 |= Outcode::LEFT;
    } else if ml.a.x > am_map.m_x2 {
        outcode1 |= Outcode::RIGHT;
    }
    if ml.b.x < am_map.m_x {
        outcode2 |= Outcode::LEFT;
    } else if ml.b.x > am_map.m_x2 {
        outcode2 |= Outcode::RIGHT;
    }
    if outcode1.intersects(outcode2) {
        return false;
    }
    fl.a.x = am_map.f_x as Fixed + (fixed_mul(ml.a.x - am_map.m_x, am_map.scale_mtof) >> 16);
    fl.a.y = am_map.f_y as Fixed
        + (am_map.f_h as Fixed - (fixed_mul(ml.a.y - am_map.m_y, am_map.scale_mtof) >> 16));
    fl.b.x = am_map.f_x as Fixed + (fixed_mul(ml.b.x - am_map.m_x, am_map.scale_mtof) >> 16);
    fl.b.y = am_map.f_y as Fixed
        + (am_map.f_h as Fixed - (fixed_mul(ml.b.y - am_map.m_y, am_map.scale_mtof) >> 16));
    outcode1 = Outcode::of_screen_point(fl.a, am_map.f_w, am_map.f_h);
    outcode2 = Outcode::of_screen_point(fl.b, am_map.f_w, am_map.f_h);
    if outcode1.intersects(outcode2) {
        return false;
    }
    while !(outcode1 | outcode2).is_empty() {
        let outside = if outcode1.is_empty() {
            outcode2
        } else {
            outcode1
        };
        if outside.contains(Outcode::TOP) {
            dy = fl.a.y - fl.b.y;
            dx = fl.b.x - fl.a.x;
            tmp.x = fl.a.x + dx * fl.a.y / dy;
            tmp.y = 0;
        } else if outside.contains(Outcode::BOTTOM) {
            dy = fl.a.y - fl.b.y;
            dx = fl.b.x - fl.a.x;
            tmp.x = fl.a.x + dx * (fl.a.y - am_map.f_h) / dy;
            tmp.y = am_map.f_h - 1;
        } else if outside.contains(Outcode::RIGHT) {
            dy = fl.b.y - fl.a.y;
            dx = fl.b.x - fl.a.x;
            tmp.y = fl.a.y + dy * (am_map.f_w - 1 - fl.a.x) / dx;
            tmp.x = am_map.f_w - 1;
        } else if outside.contains(Outcode::LEFT) {
            dy = fl.b.y - fl.a.y;
            dx = fl.b.x - fl.a.x;
            tmp.y = fl.a.y + dy * -fl.a.x / dx;
            tmp.x = 0;
        } else {
            tmp.x = 0;
            tmp.y = 0;
        }
        if outside == outcode1 {
            fl.a = tmp;
            outcode1 = Outcode::of_screen_point(fl.a, am_map.f_w, am_map.f_h);
        } else {
            fl.b = tmp;
            outcode2 = Outcode::of_screen_point(fl.b, am_map.f_w, am_map.f_h);
        }
        if outcode1.intersects(outcode2) {
            return false;
        }
    }
    true
}
pub fn draw_fline(
    am_map: &mut AmMapState,
    i_video: &mut IVideoState,
    platform: &mut dyn DoomPlatform,
    fl: &FLine,
    color: i32,
) {
    let mut d: i32;
    if fl.a.x < 0
        || fl.a.x >= am_map.f_w
        || fl.a.y < 0
        || fl.a.y >= am_map.f_h
        || fl.b.x < 0
        || fl.b.x >= am_map.f_w
        || fl.b.y < 0
        || fl.b.y >= am_map.f_h
    {
        doom_eprint!(platform, "fuck {} \r", am_map.am_drawfline_fuck);
        am_map.am_drawfline_fuck += 1;
        return;
    }
    let dx: i32 = fl.b.x - fl.a.x;
    let ax: i32 = 2 * (if dx < 0 { -dx } else { dx });
    let sx: i32 = if dx < 0 { -1 } else { 1 };
    let dy: i32 = fl.b.y - fl.a.y;
    let ay: i32 = 2 * (if dy < 0 { -dy } else { dy });
    let sy: i32 = if dy < 0 { -1 } else { 1 };
    let mut x: i32 = fl.a.x;
    let mut y: i32 = fl.a.y;
    if ax > ay {
        d = ay - ax / 2;
        loop {
            i_video.i_video_buffer[(y * am_map.f_w + x) as usize] = color as u8;
            if x == fl.b.x {
                return;
            }
            if d >= 0 {
                y += sy;
                d -= ax;
            }
            x += sx;
            d += ay;
        }
    } else {
        d = ax - ay / 2;
        loop {
            i_video.i_video_buffer[(y * am_map.f_w + x) as usize] = color as u8;
            if y == fl.b.y {
                return;
            }
            if d >= 0 {
                x += sx;
                d -= ay;
            }
            y += sy;
            d += ax;
        }
    };
}
pub fn draw_mline(
    am_map: &mut AmMapState,
    i_video: &mut IVideoState,
    platform: &mut dyn DoomPlatform,
    ml: &MLine,
    color: i32,
) {
    let mut fl: FLine = FLine {
        a: FPoint { x: 0, y: 0 },
        b: FPoint { x: 0, y: 0 },
    };
    if clip_mline(am_map, ml, &mut fl) {
        draw_fline(am_map, i_video, &mut *platform, &fl, color);
    }
}
pub fn draw_grid(state: &mut GameState, color: i32) {
    let mut ml: MLine = MLine {
        a: MPoint { x: 0, y: 0 },
        b: MPoint { x: 0, y: 0 },
    };
    let mut start: Fixed = state.ui.am_map.m_x;
    if (start - state.world.p_setup.bmaporgx) % (MAPBLOCKUNITS << FRACBITS) != 0 {
        start += (MAPBLOCKUNITS << FRACBITS)
            - (start - state.world.p_setup.bmaporgx) % (MAPBLOCKUNITS << FRACBITS);
    }
    let mut end: Fixed = state.ui.am_map.m_x + state.ui.am_map.m_w;
    ml.a.y = state.ui.am_map.m_y;
    ml.b.y = state.ui.am_map.m_y + state.ui.am_map.m_h;
    for x in (start..end).step_by((MAPBLOCKUNITS << FRACBITS) as usize) {
        ml.a.x = x;
        ml.b.x = x;
        draw_mline(
            &mut state.ui.am_map,
            &mut state.io.i_video,
            &mut *state.io.platform,
            &ml,
            color,
        );
    }
    start = state.ui.am_map.m_y;
    if (start - state.world.p_setup.bmaporgy) % (MAPBLOCKUNITS << FRACBITS) != 0 {
        start += (MAPBLOCKUNITS << FRACBITS)
            - (start - state.world.p_setup.bmaporgy) % (MAPBLOCKUNITS << FRACBITS);
    }
    end = state.ui.am_map.m_y + state.ui.am_map.m_h;
    ml.a.x = state.ui.am_map.m_x;
    ml.b.x = state.ui.am_map.m_x + state.ui.am_map.m_w;
    for y in (start..end).step_by((MAPBLOCKUNITS << FRACBITS) as usize) {
        ml.a.y = y;
        ml.b.y = y;
        draw_mline(
            &mut state.ui.am_map,
            &mut state.io.i_video,
            &mut *state.io.platform,
            &ml,
            color,
        );
    }
}
pub fn draw_walls(state: &mut GameState) {
    let mut l: MLine = MLine {
        a: MPoint { x: 0, y: 0 },
        b: MPoint { x: 0, y: 0 },
    };
    for i in 0..(state.world.p_setup.numlines as usize) {
        let li = &state.world.p_setup.lines[i];
        let (li_flags, li_special) = (li.flags, li.special as i32);
        let (li_backsector, li_frontsector) = (li.backsector, li.frontsector);
        let li_v1 = state.world.p_setup.vertexes[li.v1.0 as usize];
        let li_v2 = state.world.p_setup.vertexes[li.v2.0 as usize];
        l.a.x = li_v1.x;
        l.a.y = li_v1.y;
        l.b.x = li_v2.x;
        l.b.y = li_v2.y;
        let lightlev = state.ui.am_map.lightlev;
        if state.ui.am_map.cheating != 0 || li_flags.contains(LineFlags::MAPPED) {
            if !(li_flags.contains(LineFlags::DONTDRAW) && state.ui.am_map.cheating == 0) {
                match li_backsector {
                    None => {
                        draw_mline(
                            &mut state.ui.am_map,
                            &mut state.io.i_video,
                            &mut *state.io.platform,
                            &l,
                            WALLCOLORS + lightlev,
                        );
                    }
                    Some(li_backsector) => {
                        if li_special == 39 {
                            draw_mline(
                                &mut state.ui.am_map,
                                &mut state.io.i_video,
                                &mut *state.io.platform,
                                &l,
                                WALLCOLORS + WALLRANGE / 2,
                            );
                        } else if li_flags.contains(LineFlags::SECRET) {
                            if state.ui.am_map.cheating != 0 {
                                draw_mline(
                                    &mut state.ui.am_map,
                                    &mut state.io.i_video,
                                    &mut *state.io.platform,
                                    &l,
                                    SECRETWALLCOLORS + lightlev,
                                );
                            } else {
                                draw_mline(
                                    &mut state.ui.am_map,
                                    &mut state.io.i_video,
                                    &mut *state.io.platform,
                                    &l,
                                    WALLCOLORS + lightlev,
                                );
                            }
                        } else if state.world.p_setup.sector_mut(li_backsector).floorheight
                            != state
                                .world
                                .p_setup
                                .sector_mut(li_frontsector.unwrap())
                                .floorheight
                        {
                            draw_mline(
                                &mut state.ui.am_map,
                                &mut state.io.i_video,
                                &mut *state.io.platform,
                                &l,
                                FDWALLCOLORS + lightlev,
                            );
                        } else if state.world.p_setup.sector_mut(li_backsector).ceilingheight
                            != state
                                .world
                                .p_setup
                                .sector_mut(li_frontsector.unwrap())
                                .ceilingheight
                        {
                            draw_mline(
                                &mut state.ui.am_map,
                                &mut state.io.i_video,
                                &mut *state.io.platform,
                                &l,
                                CDWALLCOLORS + lightlev,
                            );
                        } else if state.ui.am_map.cheating != 0 {
                            draw_mline(
                                &mut state.ui.am_map,
                                &mut state.io.i_video,
                                &mut *state.io.platform,
                                &l,
                                TSWALLCOLORS + lightlev,
                            );
                        }
                    }
                }
            }
        } else if state.game.g_game.player_mut(state.ui.am_map.plr).powers
            [PowerType::Allmap as usize]
            != 0
            && !li_flags.contains(LineFlags::DONTDRAW)
        {
            draw_mline(
                &mut state.ui.am_map,
                &mut state.io.i_video,
                &mut *state.io.platform,
                &l,
                GRAYS + 3,
            );
        }
    }
}
pub fn am_rotate(x: &mut Fixed, y: &mut Fixed, a: Angle) {
    let tmpx: Fixed = fixed_mul(*x, FINECOSINE[(a >> ANGLETOFINESHIFT) as usize])
        - fixed_mul(*y, FINESINE[(a >> ANGLETOFINESHIFT) as usize]);
    *y = fixed_mul(*x, FINESINE[(a >> ANGLETOFINESHIFT) as usize])
        + fixed_mul(*y, FINECOSINE[(a >> ANGLETOFINESHIFT) as usize]);
    *x = tmpx;
}
pub fn draw_line_character(
    state: &mut GameState,
    lineguy: &[MLine],
    scale: Fixed,
    angle: Angle,
    color: i32,
    x: Fixed,
    y: Fixed,
) {
    let mut l: MLine = MLine {
        a: MPoint { x: 0, y: 0 },
        b: MPoint { x: 0, y: 0 },
    };
    for line in lineguy {
        l.a.x = line.a.x;
        l.a.y = line.a.y;
        if scale != 0 {
            l.a.x = fixed_mul(scale, l.a.x);
            l.a.y = fixed_mul(scale, l.a.y);
        }
        if angle != 0 {
            am_rotate(&mut l.a.x, &mut l.a.y, angle);
        }
        l.a.x += x;
        l.a.y += y;
        l.b.x = line.b.x;
        l.b.y = line.b.y;
        if scale != 0 {
            l.b.x = fixed_mul(scale, l.b.x);
            l.b.y = fixed_mul(scale, l.b.y);
        }
        if angle != 0 {
            am_rotate(&mut l.b.x, &mut l.b.y, angle);
        }
        l.b.x += x;
        l.b.y += y;
        draw_mline(
            &mut state.ui.am_map,
            &mut state.io.i_video,
            &mut *state.io.platform,
            &l,
            color,
        );
    }
}
pub fn draw_players(state: &mut GameState) {
    const THEIR_COLORS: [i32; 4] = [GREENS, GRAYS, BROWNS, REDS];
    let mut their_color: i32 = -1;
    let mut color: i32;
    if !state.game.g_game.netgame {
        let plr_mo_id = state
            .game
            .g_game
            .player_mut(state.ui.am_map.plr)
            .mo
            .unwrap();
        let plr_mo = state.world.p_mobj.mo(plr_mo_id);
        let (plr_angle, plr_x, plr_y) = (plr_mo.angle, plr_mo.x, plr_mo.y);
        if state.ui.am_map.cheating != 0 {
            draw_line_character(
                state,
                &CHEAT_PLAYER_ARROW,
                0,
                plr_angle,
                WHITE,
                plr_x,
                plr_y,
            );
        } else {
            draw_line_character(state, &PLAYER_ARROW, 0, plr_angle, WHITE, plr_x, plr_y);
        }
        return;
    }
    for i in 0..MAXPLAYERS {
        their_color += 1;
        let p = &state.game.g_game.players[i as usize];
        let (p_invisibility, p_mo_id) = (p.powers[PowerType::Invisibility], p.mo);
        if !(state.game.g_game.deathmatch != 0
            && !state.game.g_game.singledemo
            && PlayerId(i as u8) != state.ui.am_map.plr)
            && state.game.g_game.playeringame[i as usize]
        {
            if p_invisibility != 0 {
                color = 246;
            } else {
                color = THEIR_COLORS[their_color as usize];
            }
            let p_mo = state.world.p_mobj.mo(p_mo_id.unwrap());
            let (p_angle, p_x, p_y) = (p_mo.angle, p_mo.x, p_mo.y);
            draw_line_character(state, &PLAYER_ARROW, 0, p_angle, color, p_x, p_y);
        }
    }
}
pub fn draw_things(state: &mut GameState, colors: i32) {
    for i in 0..(state.world.p_setup.numsectors as usize) {
        let mut cursor = state.world.p_setup.sectors[i].thinglist;
        while let Some(id) = cursor {
            let t = state.world.p_mobj.mo(id);
            let (t_angle, t_x, t_y, t_snext) = (t.angle, t.x, t.y, t.snext);
            let lightlev = state.ui.am_map.lightlev;
            draw_line_character(
                state,
                &THINTRIANGLE_GUY,
                (16) << FRACBITS,
                t_angle,
                colors + lightlev,
                t_x,
                t_y,
            );
            cursor = t_snext;
        }
    }
}
pub fn draw_marks(state: &mut GameState) {
    let mut fx: i32;
    let mut fy: i32;
    let mut w: i32;
    let mut h: i32;
    for i in 0..(AM_NUMMARKPOINTS as usize) {
        if state.ui.am_map.markpoints[i].x != -1 {
            w = 5;
            h = 6;
            fx = state.ui.am_map.f_x as Fixed
                + (fixed_mul(
                    state.ui.am_map.markpoints[i].x - state.ui.am_map.m_x,
                    state.ui.am_map.scale_mtof,
                ) >> 16);
            fy = state.ui.am_map.f_y as Fixed
                + (state.ui.am_map.f_h as Fixed
                    - (fixed_mul(
                        state.ui.am_map.markpoints[i].y - state.ui.am_map.m_y,
                        state.ui.am_map.scale_mtof,
                    ) >> 16));
            if fx >= state.ui.am_map.f_x
                && fx <= state.ui.am_map.f_w - w
                && fy >= state.ui.am_map.f_y
                && fy <= state.ui.am_map.f_h - h
            {
                let lumpnum = state.ui.am_map.marknums[i];
                let patch = cache_patch_num(&*state.assets.fs, &mut state.assets.w_wad, lumpnum);
                let dest_screen = Screen::Video;
                draw_patch(state, dest_screen, fx, fy, &patch);
            }
        }
    }
}
pub fn draw_crosshair(am_map: &AmMapState, i_video: &mut IVideoState, color: i32) {
    let idx = (am_map.f_w * (am_map.f_h + 1) / 2) as usize;
    i_video.i_video_buffer[idx] = color as u8;
}
pub fn am_drawer(state: &mut GameState) {
    if !state.ui.am_map.automapactive {
        return;
    }
    clear_fb(&state.ui.am_map, &mut state.io.i_video, BACKGROUND);
    if state.ui.am_map.grid {
        draw_grid(state, GRIDCOLORS);
    }
    draw_walls(state);
    draw_players(state);
    if state.ui.am_map.cheating == 2 {
        draw_things(state, THINGCOLORS);
    }
    draw_crosshair(&state.ui.am_map, &mut state.io.i_video, XHAIRCOLORS);
    draw_marks(state);
    let (f_x, f_y, f_w, f_h) = (
        state.ui.am_map.f_x,
        state.ui.am_map.f_y,
        state.ui.am_map.f_w,
        state.ui.am_map.f_h,
    );
    let dest_screen = Screen::Video;
    mark_rect(&mut state.io.v_video, dest_screen, f_x, f_y, f_w, f_h);
}
