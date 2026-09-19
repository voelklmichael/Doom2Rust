use crate::d_event::EvType;
use crate::d_event::Event;
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

use crate::p_spec::ML_MAPPED;
use crate::p_spec::ML_SECRET;
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
        AmMapState {
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
pub const RIGHT: i32 = 2;
pub const LEFT: i32 = 1;
pub const BOTTOM: i32 = 4;
pub const TOP: i32 = 8;
pub const ML_DONTDRAW: i32 = 128;
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
pub const LINE_NEVERSEE: i32 = ML_DONTDRAW;
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
pub fn activate_new_scale(state: &mut GameState) {
    state.am_map.m_x += state.am_map.m_w / 2;
    state.am_map.m_y += state.am_map.m_h / 2;
    state.am_map.m_w = fixed_mul((state.am_map.f_w as Fixed) << 16, state.am_map.scale_ftom);
    state.am_map.m_h = fixed_mul((state.am_map.f_h as Fixed) << 16, state.am_map.scale_ftom);
    state.am_map.m_x -= state.am_map.m_w / 2;
    state.am_map.m_y -= state.am_map.m_h / 2;
    state.am_map.m_x2 = state.am_map.m_x + state.am_map.m_w;
    state.am_map.m_y2 = state.am_map.m_y + state.am_map.m_h;
}
pub fn save_scale_and_loc(state: &mut GameState) {
    state.am_map.old_m_x = state.am_map.m_x;
    state.am_map.old_m_y = state.am_map.m_y;
    state.am_map.old_m_w = state.am_map.m_w;
    state.am_map.old_m_h = state.am_map.m_h;
}
pub fn restore_scale_and_loc(state: &mut GameState) {
    state.am_map.m_w = state.am_map.old_m_w;
    state.am_map.m_h = state.am_map.old_m_h;
    if !state.am_map.followplayer {
        state.am_map.m_x = state.am_map.old_m_x;
        state.am_map.m_y = state.am_map.old_m_y;
    } else {
        let plr_mo_id = state.g_game.player_mut(state.am_map.plr).mo.unwrap();
        let plr_mo = state.p_mobj.mo(plr_mo_id);
        state.am_map.m_x = (plr_mo.x - state.am_map.m_w / 2) as Fixed;
        state.am_map.m_y = (plr_mo.y - state.am_map.m_h / 2) as Fixed;
    }
    state.am_map.m_x2 = state.am_map.m_x + state.am_map.m_w;
    state.am_map.m_y2 = state.am_map.m_y + state.am_map.m_h;
    state.am_map.scale_mtof = fixed_div((state.am_map.f_w as Fixed) << FRACBITS, state.am_map.m_w);
    state.am_map.scale_ftom = fixed_div(FRACUNIT, state.am_map.scale_mtof);
}
pub fn add_mark(state: &mut GameState) {
    state.am_map.markpoints[state.am_map.markpointnum as usize].x =
        (state.am_map.m_x + state.am_map.m_w / 2) as Fixed;
    state.am_map.markpoints[state.am_map.markpointnum as usize].y =
        (state.am_map.m_y + state.am_map.m_h / 2) as Fixed;
    state.am_map.markpointnum = (state.am_map.markpointnum + 1) % AM_NUMMARKPOINTS;
}
pub fn find_min_max_boundaries(state: &mut GameState) {
    state.am_map.min_y = INT_MAX as Fixed;
    state.am_map.min_x = state.am_map.min_y;
    state.am_map.max_y = -INT_MAX as Fixed;
    state.am_map.max_x = state.am_map.max_y;
    for i in 0..(state.p_setup.numvertexes as usize) {
        let v = state.p_setup.vertexes[i];
        if v.x < state.am_map.min_x {
            state.am_map.min_x = v.x;
        } else if v.x > state.am_map.max_x {
            state.am_map.max_x = v.x;
        }
        if v.y < state.am_map.min_y {
            state.am_map.min_y = v.y;
        } else if v.y > state.am_map.max_y {
            state.am_map.max_y = v.y;
        }
    }
    state.am_map.max_w = state.am_map.max_x - state.am_map.min_x;
    state.am_map.max_h = state.am_map.max_y - state.am_map.min_y;
    state.am_map.min_w = (2 * 16 * FRACUNIT) as Fixed;
    state.am_map.min_h = (2 * 16 * FRACUNIT) as Fixed;
    let a: Fixed = fixed_div((state.am_map.f_w as Fixed) << FRACBITS, state.am_map.max_w);
    let b: Fixed = fixed_div((state.am_map.f_h as Fixed) << FRACBITS, state.am_map.max_h);
    state.am_map.min_scale_mtof = if a < b { a } else { b };
    state.am_map.max_scale_mtof =
        fixed_div((state.am_map.f_h as Fixed) << FRACBITS, 2 * 16 * FRACUNIT);
}
pub fn change_window_loc(state: &mut GameState) {
    if state.am_map.m_paninc.x != 0 || state.am_map.m_paninc.y != 0 {
        state.am_map.followplayer = false;
        state.am_map.f_oldloc.x = INT_MAX as Fixed;
    }
    state.am_map.m_x += state.am_map.m_paninc.x;
    state.am_map.m_y += state.am_map.m_paninc.y;
    if state.am_map.m_x + state.am_map.m_w / 2 > state.am_map.max_x {
        state.am_map.m_x = (state.am_map.max_x - state.am_map.m_w / 2) as Fixed;
    } else if (state.am_map.m_x + state.am_map.m_w / 2) < state.am_map.min_x {
        state.am_map.m_x = (state.am_map.min_x - state.am_map.m_w / 2) as Fixed;
    }
    if state.am_map.m_y + state.am_map.m_h / 2 > state.am_map.max_y {
        state.am_map.m_y = (state.am_map.max_y - state.am_map.m_h / 2) as Fixed;
    } else if (state.am_map.m_y + state.am_map.m_h / 2) < state.am_map.min_y {
        state.am_map.m_y = (state.am_map.min_y - state.am_map.m_h / 2) as Fixed;
    }
    state.am_map.m_x2 = state.am_map.m_x + state.am_map.m_w;
    state.am_map.m_y2 = state.am_map.m_y + state.am_map.m_h;
}
pub fn am_init_variables(state: &mut GameState) {
    let mut pnum: i32;
    const ST_NOTIFY: Event = Event {
        kind: EvType::Keyup,
        data1: AM_MSGENTERED,
        data2: 0,
        data3: 0,
        data4: 0,
    };
    state.am_map.automapactive = true;
    state.am_map.f_oldloc.x = INT_MAX as Fixed;
    state.am_map.amclock = 0;
    state.am_map.lightlev = 0;
    state.am_map.m_paninc.y = 0;
    state.am_map.m_paninc.x = state.am_map.m_paninc.y;
    state.am_map.ftom_zoommul = FRACUNIT as Fixed;
    state.am_map.mtof_zoommul = FRACUNIT as Fixed;
    state.am_map.m_w = fixed_mul((state.am_map.f_w as Fixed) << 16, state.am_map.scale_ftom);
    state.am_map.m_h = fixed_mul((state.am_map.f_h as Fixed) << 16, state.am_map.scale_ftom);
    if state.g_game.playeringame[state.g_game.consoleplayer as usize] {
        state.am_map.plr = PlayerId(state.g_game.consoleplayer as u8);
    } else {
        state.am_map.plr = PlayerId(0);
        pnum = 0;
        while pnum < MAXPLAYERS {
            if state.g_game.playeringame[pnum as usize] {
                state.am_map.plr = PlayerId(pnum as u8);
                break;
            } else {
                pnum += 1;
            }
        }
    }
    let plr_mo_id = state.g_game.player_mut(state.am_map.plr).mo.unwrap();
    let plr_mo = state.p_mobj.mo(plr_mo_id);
    state.am_map.m_x = (plr_mo.x - state.am_map.m_w / 2) as Fixed;
    state.am_map.m_y = (plr_mo.y - state.am_map.m_h / 2) as Fixed;
    change_window_loc(state);
    state.am_map.old_m_x = state.am_map.m_x;
    state.am_map.old_m_y = state.am_map.m_y;
    state.am_map.old_m_w = state.am_map.m_w;
    state.am_map.old_m_h = state.am_map.m_h;
    st_responder(state, &ST_NOTIFY);
}
pub fn load_pics(state: &mut GameState) {
    for i in 0..10 {
        let namebuf = format!("AMMNUM{}", i);
        let lumpnum = get_num_for_name(&mut state.w_wad, &namebuf);
        lump_bytes(state, lumpnum);
        state.am_map.marknums[i as usize] = lumpnum;
    }
}
pub fn unload_pics(state: &mut GameState) {
    for i in 0..10 {
        release_lump_num(&mut state.w_wad, state.am_map.marknums[i]);
    }
}
pub fn clear_marks(state: &mut GameState) {
    for i in 0..(AM_NUMMARKPOINTS as usize) {
        state.am_map.markpoints[i].x = -1;
    }
    state.am_map.markpointnum = 0;
}
pub fn level_init(state: &mut GameState) {
    state.am_map.leveljuststarted = false;
    state.am_map.f_y = 0;
    state.am_map.f_x = state.am_map.f_y;
    state.am_map.f_w = FINIT_WIDTH;
    state.am_map.f_h = FINIT_HEIGHT;
    clear_marks(state);
    find_min_max_boundaries(state);
    state.am_map.scale_mtof = fixed_div(
        state.am_map.min_scale_mtof,
        (0.7f64 * FRACUNIT as f64) as Fixed,
    );
    if state.am_map.scale_mtof > state.am_map.max_scale_mtof {
        state.am_map.scale_mtof = state.am_map.min_scale_mtof;
    }
    state.am_map.scale_ftom = fixed_div(FRACUNIT, state.am_map.scale_mtof);
}
pub fn am_stop(state: &mut GameState) {
    const ST_NOTIFY: Event = Event {
        kind: EvType::Keydown,
        data1: EvType::Keyup as i32,
        data2: AM_MSGEXITED,
        data3: 0,
        data4: 0,
    };
    unload_pics(state);
    state.am_map.automapactive = false;
    st_responder(state, &ST_NOTIFY);
    state.am_map.stopped = true;
}
pub fn am_start(state: &mut GameState) {
    if !state.am_map.stopped {
        am_stop(state);
    }
    state.am_map.stopped = false;
    if state.am_map.am_start_lastlevel != state.g_game.gamemap
        || state.am_map.am_start_lastepisode != state.g_game.gameepisode
    {
        level_init(state);
        state.am_map.am_start_lastlevel = state.g_game.gamemap;
        state.am_map.am_start_lastepisode = state.g_game.gameepisode;
    }
    am_init_variables(state);
    load_pics(state);
}
pub fn min_out_window_scale(state: &mut GameState) {
    state.am_map.scale_mtof = state.am_map.min_scale_mtof;
    state.am_map.scale_ftom = fixed_div(FRACUNIT, state.am_map.scale_mtof);
    activate_new_scale(state);
}
pub fn max_out_window_scale(state: &mut GameState) {
    state.am_map.scale_mtof = state.am_map.max_scale_mtof;
    state.am_map.scale_ftom = fixed_div(FRACUNIT, state.am_map.scale_mtof);
    activate_new_scale(state);
}
pub fn am_responder(state: &mut GameState, ev: &Event) -> bool {
    let mut rc: bool;
    let key: i32;
    rc = false;
    if !state.am_map.automapactive {
        if ev.kind == EvType::Keydown && ev.data1 == state.m_controls.key_map_toggle {
            am_start(state);
            state.g_game.viewactive = false;
            rc = true;
        }
    } else if ev.kind == EvType::Keydown {
        rc = true;
        key = ev.data1;
        if key == state.m_controls.key_map_east {
            if !state.am_map.followplayer {
                state.am_map.m_paninc.x = fixed_mul((4) << 16, state.am_map.scale_ftom);
            } else {
                rc = false;
            }
        } else if key == state.m_controls.key_map_west {
            if !state.am_map.followplayer {
                state.am_map.m_paninc.x = -fixed_mul((4) << 16, state.am_map.scale_ftom);
            } else {
                rc = false;
            }
        } else if key == state.m_controls.key_map_north {
            if !state.am_map.followplayer {
                state.am_map.m_paninc.y = fixed_mul((4) << 16, state.am_map.scale_ftom);
            } else {
                rc = false;
            }
        } else if key == state.m_controls.key_map_south {
            if !state.am_map.followplayer {
                state.am_map.m_paninc.y = -fixed_mul((4) << 16, state.am_map.scale_ftom);
            } else {
                rc = false;
            }
        } else if key == state.m_controls.key_map_zoomout {
            state.am_map.mtof_zoommul = M_ZOOMOUT as Fixed;
            state.am_map.ftom_zoommul = M_ZOOMIN as Fixed;
        } else if key == state.m_controls.key_map_zoomin {
            state.am_map.mtof_zoommul = M_ZOOMIN as Fixed;
            state.am_map.ftom_zoommul = M_ZOOMOUT as Fixed;
        } else if key == state.m_controls.key_map_toggle {
            state.am_map.am_responder_bigstate = false;
            state.g_game.viewactive = true;
            am_stop(state);
        } else if key == state.m_controls.key_map_maxzoom {
            state.am_map.am_responder_bigstate = !state.am_map.am_responder_bigstate;
            if state.am_map.am_responder_bigstate {
                save_scale_and_loc(state);
                min_out_window_scale(state);
            } else {
                restore_scale_and_loc(state);
            }
        } else if key == state.m_controls.key_map_follow {
            state.am_map.followplayer = !state.am_map.followplayer;
            state.am_map.f_oldloc.x = INT_MAX as Fixed;
            if state.am_map.followplayer {
                state.g_game.player_mut(state.am_map.plr).message =
                    Some("Follow Mode ON".to_string());
            } else {
                state.g_game.player_mut(state.am_map.plr).message =
                    Some("Follow Mode OFF".to_string());
            }
        } else if key == state.m_controls.key_map_grid {
            state.am_map.grid = !state.am_map.grid;
            if state.am_map.grid {
                state.g_game.player_mut(state.am_map.plr).message = Some("Grid ON".to_string());
            } else {
                state.g_game.player_mut(state.am_map.plr).message = Some("Grid OFF".to_string());
            }
        } else if key == state.m_controls.key_map_mark {
            state.g_game.player_mut(state.am_map.plr).message =
                Some(format!("Marked Spot {}", state.am_map.markpointnum));
            add_mark(state);
        } else if key == state.m_controls.key_map_clearmark {
            clear_marks(state);
            state.g_game.player_mut(state.am_map.plr).message =
                Some("All Marks Cleared".to_string());
        } else {
            rc = false;
        }
        if state.g_game.deathmatch == 0
            && cht_check_cheat(&mut state.am_map.cheat_amap, ev.data2 as u8)
        {
            rc = false;
            state.am_map.cheating = (state.am_map.cheating + 1) % 3;
        }
    } else if ev.kind == EvType::Keyup {
        rc = false;
        key = ev.data1;
        if key == state.m_controls.key_map_east || key == state.m_controls.key_map_west {
            if !state.am_map.followplayer {
                state.am_map.m_paninc.x = 0;
            }
        } else if key == state.m_controls.key_map_north || key == state.m_controls.key_map_south {
            if !state.am_map.followplayer {
                state.am_map.m_paninc.y = 0;
            }
        } else if key == state.m_controls.key_map_zoomout || key == state.m_controls.key_map_zoomin
        {
            state.am_map.mtof_zoommul = FRACUNIT as Fixed;
            state.am_map.ftom_zoommul = FRACUNIT as Fixed;
        }
    }
    rc
}
pub fn change_window_scale(state: &mut GameState) {
    state.am_map.scale_mtof = fixed_mul(state.am_map.scale_mtof, state.am_map.mtof_zoommul);
    state.am_map.scale_ftom = fixed_div(FRACUNIT, state.am_map.scale_mtof);
    if state.am_map.scale_mtof < state.am_map.min_scale_mtof {
        min_out_window_scale(state);
    } else if state.am_map.scale_mtof > state.am_map.max_scale_mtof {
        max_out_window_scale(state);
    } else {
        activate_new_scale(state);
    };
}
pub fn do_follow_player(state: &mut GameState) {
    let plr_mo_id = state.g_game.player_mut(state.am_map.plr).mo.unwrap();
    let plr_mo = state.p_mobj.mo(plr_mo_id);
    let (plr_x, plr_y) = (plr_mo.x, plr_mo.y);
    if state.am_map.f_oldloc.x != plr_x || state.am_map.f_oldloc.y != plr_y {
        state.am_map.m_x = (fixed_mul(
            (fixed_mul(plr_x, state.am_map.scale_mtof) >> 16) << 16,
            state.am_map.scale_ftom,
        ) - state.am_map.m_w / 2) as Fixed;
        state.am_map.m_y = (fixed_mul(
            (fixed_mul(plr_y, state.am_map.scale_mtof) >> 16) << 16,
            state.am_map.scale_ftom,
        ) - state.am_map.m_h / 2) as Fixed;
        state.am_map.m_x2 = state.am_map.m_x + state.am_map.m_w;
        state.am_map.m_y2 = state.am_map.m_y + state.am_map.m_h;
        state.am_map.f_oldloc.x = plr_x;
        state.am_map.f_oldloc.y = plr_y;
    }
}
pub fn am_ticker(state: &mut GameState) {
    if !state.am_map.automapactive {
        return;
    }
    state.am_map.amclock += 1;
    if state.am_map.followplayer {
        do_follow_player(state);
    }
    if state.am_map.ftom_zoommul != FRACUNIT {
        change_window_scale(state);
    }
    if state.am_map.m_paninc.x != 0 || state.am_map.m_paninc.y != 0 {
        change_window_loc(state);
    }
}
pub fn clear_fb(state: &mut GameState, color: i32) {
    let len = (state.am_map.f_w * state.am_map.f_h) as usize;
    state.i_video.i_video_buffer[..len].fill(color as u8);
}
pub fn clip_mline(state: &mut GameState, ml: &MLine, fl: &mut FLine) -> bool {
    let mut outcode1: i32 = 0;
    let mut outcode2: i32 = 0;
    let mut outside: i32;
    let mut tmp: FPoint = FPoint { x: 0, y: 0 };
    let mut dx: i32;
    let mut dy: i32;
    if ml.a.y > state.am_map.m_y2 {
        outcode1 = TOP;
    } else if ml.a.y < state.am_map.m_y {
        outcode1 = BOTTOM;
    }
    if ml.b.y > state.am_map.m_y2 {
        outcode2 = TOP;
    } else if ml.b.y < state.am_map.m_y {
        outcode2 = BOTTOM;
    }
    if outcode1 & outcode2 != 0 {
        return false;
    }
    if ml.a.x < state.am_map.m_x {
        outcode1 |= LEFT;
    } else if ml.a.x > state.am_map.m_x2 {
        outcode1 |= RIGHT;
    }
    if ml.b.x < state.am_map.m_x {
        outcode2 |= LEFT;
    } else if ml.b.x > state.am_map.m_x2 {
        outcode2 |= RIGHT;
    }
    if outcode1 & outcode2 != 0 {
        return false;
    }
    fl.a.x = state.am_map.f_x as Fixed
        + (fixed_mul(ml.a.x - state.am_map.m_x, state.am_map.scale_mtof) >> 16);
    fl.a.y = state.am_map.f_y as Fixed
        + (state.am_map.f_h as Fixed
            - (fixed_mul(ml.a.y - state.am_map.m_y, state.am_map.scale_mtof) >> 16));
    fl.b.x = state.am_map.f_x as Fixed
        + (fixed_mul(ml.b.x - state.am_map.m_x, state.am_map.scale_mtof) >> 16);
    fl.b.y = state.am_map.f_y as Fixed
        + (state.am_map.f_h as Fixed
            - (fixed_mul(ml.b.y - state.am_map.m_y, state.am_map.scale_mtof) >> 16));
    outcode1 = 0;
    if fl.a.y < 0 {
        outcode1 |= TOP;
    } else if fl.a.y >= state.am_map.f_h {
        outcode1 |= BOTTOM;
    }
    if fl.a.x < 0 {
        outcode1 |= LEFT;
    } else if fl.a.x >= state.am_map.f_w {
        outcode1 |= RIGHT;
    }
    outcode2 = 0;
    if fl.b.y < 0 {
        outcode2 |= TOP;
    } else if fl.b.y >= state.am_map.f_h {
        outcode2 |= BOTTOM;
    }
    if fl.b.x < 0 {
        outcode2 |= LEFT;
    } else if fl.b.x >= state.am_map.f_w {
        outcode2 |= RIGHT;
    }
    if outcode1 & outcode2 != 0 {
        return false;
    }
    while outcode1 | outcode2 != 0 {
        if outcode1 != 0 {
            outside = outcode1;
        } else {
            outside = outcode2;
        }
        if outside & TOP != 0 {
            dy = fl.a.y - fl.b.y;
            dx = fl.b.x - fl.a.x;
            tmp.x = fl.a.x + dx * fl.a.y / dy;
            tmp.y = 0;
        } else if outside & BOTTOM != 0 {
            dy = fl.a.y - fl.b.y;
            dx = fl.b.x - fl.a.x;
            tmp.x = fl.a.x + dx * (fl.a.y - state.am_map.f_h) / dy;
            tmp.y = state.am_map.f_h - 1;
        } else if outside & RIGHT != 0 {
            dy = fl.b.y - fl.a.y;
            dx = fl.b.x - fl.a.x;
            tmp.y = fl.a.y + dy * (state.am_map.f_w - 1 - fl.a.x) / dx;
            tmp.x = state.am_map.f_w - 1;
        } else if outside & LEFT != 0 {
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
            outcode1 = 0;
            if fl.a.y < 0 {
                outcode1 |= TOP;
            } else if fl.a.y >= state.am_map.f_h {
                outcode1 |= BOTTOM;
            }
            if fl.a.x < 0 {
                outcode1 |= LEFT;
            } else if fl.a.x >= state.am_map.f_w {
                outcode1 |= RIGHT;
            }
        } else {
            fl.b = tmp;
            outcode2 = 0;
            if fl.b.y < 0 {
                outcode2 |= TOP;
            } else if fl.b.y >= state.am_map.f_h {
                outcode2 |= BOTTOM;
            }
            if fl.b.x < 0 {
                outcode2 |= LEFT;
            } else if fl.b.x >= state.am_map.f_w {
                outcode2 |= RIGHT;
            }
        }
        if outcode1 & outcode2 != 0 {
            return false;
        }
    }
    true
}
pub fn draw_fline(state: &mut GameState, fl: &FLine, color: i32) {
    let mut x: i32;
    let mut y: i32;

    let mut d: i32;
    if fl.a.x < 0
        || fl.a.x >= state.am_map.f_w
        || fl.a.y < 0
        || fl.a.y >= state.am_map.f_h
        || fl.b.x < 0
        || fl.b.x >= state.am_map.f_w
        || fl.b.y < 0
        || fl.b.y >= state.am_map.f_h
    {
        let fresh0 = state.am_map.am_drawfline_fuck;
        state.am_map.am_drawfline_fuck += 1;
        doom_eprint!(state.platform, "fuck {} \r", fresh0);
        return;
    }
    let dx: i32 = fl.b.x - fl.a.x;
    let ax: i32 = 2 * (if dx < 0 { -dx } else { dx });
    let sx: i32 = if dx < 0 { -1 } else { 1 };
    let dy: i32 = fl.b.y - fl.a.y;
    let ay: i32 = 2 * (if dy < 0 { -dy } else { dy });
    let sy: i32 = if dy < 0 { -1 } else { 1 };
    x = fl.a.x;
    y = fl.a.y;
    if ax > ay {
        d = ay - ax / 2;
        loop {
            state.i_video.i_video_buffer[(y * state.am_map.f_w + x) as usize] = color as u8;
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
            state.i_video.i_video_buffer[(y * state.am_map.f_w + x) as usize] = color as u8;
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
pub fn draw_mline(state: &mut GameState, ml: &MLine, color: i32) {
    let mut fl: FLine = FLine {
        a: FPoint { x: 0, y: 0 },
        b: FPoint { x: 0, y: 0 },
    };
    if clip_mline(state, ml, &mut fl) {
        draw_fline(state, &fl, color);
    }
}
pub fn draw_grid(state: &mut GameState, color: i32) {
    let mut x: Fixed;
    let mut y: Fixed;
    let mut start: Fixed;
    let mut end: Fixed;
    let mut ml: MLine = MLine {
        a: MPoint { x: 0, y: 0 },
        b: MPoint { x: 0, y: 0 },
    };
    start = state.am_map.m_x;
    if (start - state.p_setup.bmaporgx) % (MAPBLOCKUNITS << FRACBITS) != 0 {
        start += (MAPBLOCKUNITS << FRACBITS)
            - (start - state.p_setup.bmaporgx) % (MAPBLOCKUNITS << FRACBITS);
    }
    end = state.am_map.m_x + state.am_map.m_w;
    ml.a.y = state.am_map.m_y;
    ml.b.y = state.am_map.m_y + state.am_map.m_h;
    x = start;
    while x < end {
        ml.a.x = x;
        ml.b.x = x;
        draw_mline(state, &ml, color);
        x += MAPBLOCKUNITS << FRACBITS;
    }
    start = state.am_map.m_y;
    if (start - state.p_setup.bmaporgy) % (MAPBLOCKUNITS << FRACBITS) != 0 {
        start += (MAPBLOCKUNITS << FRACBITS)
            - (start - state.p_setup.bmaporgy) % (MAPBLOCKUNITS << FRACBITS);
    }
    end = state.am_map.m_y + state.am_map.m_h;
    ml.a.x = state.am_map.m_x;
    ml.b.x = state.am_map.m_x + state.am_map.m_w;
    y = start;
    while y < end {
        ml.a.y = y;
        ml.b.y = y;
        draw_mline(state, &ml, color);
        y += MAPBLOCKUNITS << FRACBITS;
    }
}
pub fn draw_walls(state: &mut GameState) {
    let mut l: MLine = MLine {
        a: MPoint { x: 0, y: 0 },
        b: MPoint { x: 0, y: 0 },
    };
    for i in 0..(state.p_setup.numlines as usize) {
        let li = &state.p_setup.lines[i];
        let (li_flags, li_special) = (li.flags as i32, li.special as i32);
        let (li_backsector, li_frontsector) = (li.backsector, li.frontsector);
        let li_v1 = state.p_setup.vertexes[li.v1.0 as usize];
        let li_v2 = state.p_setup.vertexes[li.v2.0 as usize];
        l.a.x = li_v1.x;
        l.a.y = li_v1.y;
        l.b.x = li_v2.x;
        l.b.y = li_v2.y;
        let lightlev = state.am_map.lightlev;
        if state.am_map.cheating != 0 || li_flags & ML_MAPPED != 0 {
            if !(li_flags & LINE_NEVERSEE != 0 && state.am_map.cheating == 0) {
                match li_backsector {
                    None => {
                        draw_mline(state, &l, WALLCOLORS + lightlev);
                    }
                    Some(li_backsector) => {
                        if li_special == 39 {
                            draw_mline(state, &l, WALLCOLORS + WALLRANGE / 2);
                        } else if li_flags & ML_SECRET != 0 {
                            if state.am_map.cheating != 0 {
                                draw_mline(state, &l, SECRETWALLCOLORS + lightlev);
                            } else {
                                draw_mline(state, &l, WALLCOLORS + lightlev);
                            }
                        } else if state.p_setup.sector_mut(li_backsector).floorheight
                            != state
                                .p_setup
                                .sector_mut(li_frontsector.unwrap())
                                .floorheight
                        {
                            draw_mline(state, &l, FDWALLCOLORS + lightlev);
                        } else if state.p_setup.sector_mut(li_backsector).ceilingheight
                            != state
                                .p_setup
                                .sector_mut(li_frontsector.unwrap())
                                .ceilingheight
                        {
                            draw_mline(state, &l, CDWALLCOLORS + lightlev);
                        } else if state.am_map.cheating != 0 {
                            draw_mline(state, &l, TSWALLCOLORS + lightlev);
                        }
                    }
                }
            }
        } else if state.g_game.player_mut(state.am_map.plr).powers[PowerType::Allmap as usize] != 0
            && li_flags & LINE_NEVERSEE == 0
        {
            draw_mline(state, &l, GRAYS + 3);
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
        draw_mline(state, &l, color);
    }
}
pub fn draw_players(state: &mut GameState) {
    const THEIR_COLORS: [i32; 4] = [GREENS, GRAYS, BROWNS, REDS];
    let mut their_color: i32 = -1;
    let mut color: i32;
    if !state.g_game.netgame {
        let plr_mo_id = state.g_game.player_mut(state.am_map.plr).mo.unwrap();
        let plr_mo = state.p_mobj.mo(plr_mo_id);
        let (plr_angle, plr_x, plr_y) = (plr_mo.angle, plr_mo.x, plr_mo.y);
        if state.am_map.cheating != 0 {
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
        let p = &state.g_game.players[i as usize];
        let (p_invisibility, p_mo_id) = (p.powers[PowerType::Invisibility as usize], p.mo);
        if !(state.g_game.deathmatch != 0
            && !state.g_game.singledemo
            && PlayerId(i as u8) != state.am_map.plr)
            && state.g_game.playeringame[i as usize]
        {
            if p_invisibility != 0 {
                color = 246;
            } else {
                color = THEIR_COLORS[their_color as usize];
            }
            let p_mo = state.p_mobj.mo(p_mo_id.unwrap());
            let (p_angle, p_x, p_y) = (p_mo.angle, p_mo.x, p_mo.y);
            draw_line_character(state, &PLAYER_ARROW, 0, p_angle, color, p_x, p_y);
        }
    }
}
pub fn draw_things(state: &mut GameState, colors: i32) {
    for i in 0..(state.p_setup.numsectors as usize) {
        let mut cursor = state.p_setup.sectors[i].thinglist;
        while let Some(id) = cursor {
            let t = state.p_mobj.mo(id);
            let (t_angle, t_x, t_y, t_snext) = (t.angle, t.x, t.y, t.snext);
            let lightlev = state.am_map.lightlev;
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
        if state.am_map.markpoints[i].x != -1 {
            w = 5;
            h = 6;
            fx = state.am_map.f_x as Fixed
                + (fixed_mul(
                    state.am_map.markpoints[i].x - state.am_map.m_x,
                    state.am_map.scale_mtof,
                ) >> 16);
            fy = state.am_map.f_y as Fixed
                + (state.am_map.f_h as Fixed
                    - (fixed_mul(
                        state.am_map.markpoints[i].y - state.am_map.m_y,
                        state.am_map.scale_mtof,
                    ) >> 16));
            if fx >= state.am_map.f_x
                && fx <= state.am_map.f_w - w
                && fy >= state.am_map.f_y
                && fy <= state.am_map.f_h - h
            {
                let lumpnum = state.am_map.marknums[i];
                let patch = cache_patch_num(state, lumpnum);
                let dest_screen = Screen::Video;
                draw_patch(state, dest_screen, fx, fy, &patch);
            }
        }
    }
}
pub fn draw_crosshair(state: &mut GameState, color: i32) {
    let idx = (state.am_map.f_w * (state.am_map.f_h + 1) / 2) as usize;
    state.i_video.i_video_buffer[idx] = color as u8;
}
pub fn am_drawer(state: &mut GameState) {
    if !state.am_map.automapactive {
        return;
    }
    clear_fb(state, BACKGROUND);
    if state.am_map.grid {
        draw_grid(state, GRIDCOLORS);
    }
    draw_walls(state);
    draw_players(state);
    if state.am_map.cheating == 2 {
        draw_things(state, THINGCOLORS);
    }
    draw_crosshair(state, XHAIRCOLORS);
    draw_marks(state);
    let (f_x, f_y, f_w, f_h) = (
        state.am_map.f_x,
        state.am_map.f_y,
        state.am_map.f_w,
        state.am_map.f_h,
    );
    let dest_screen = Screen::Video;
    mark_rect(state, dest_screen, f_x, f_y, f_w, f_h);
}
