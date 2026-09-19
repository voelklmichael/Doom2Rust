use crate::d_event::event_t;
use crate::d_event::EvType;
use alloc::string::ToString;

use crate::d_player::PlayerId;
use crate::d_player::PowerType;
use crate::doomdef::MAXPLAYERS;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::game_state::GameState;
use crate::m_cheat::cheatseq_t;
use crate::m_cheat::cht_CheckCheat;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedDiv;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::m_fixed::INT_MAX;
use crate::p_maputl::MAPBLOCKUNITS;
use crate::v_video::Screen;
use crate::v_video::V_CachePatchNum;

use crate::p_spec::ML_MAPPED;
use crate::p_spec::ML_SECRET;
use crate::st_stuff::ST_Responder;

use crate::tables::angle_t;
use crate::tables::finecosine;
use crate::tables::finesine;
use crate::tables::ANGLETOFINESHIFT;
use crate::v_video::V_DrawPatch;
use crate::v_video::V_MarkRect;
use crate::w_wad::{W_GetNumForName, W_LumpBytes, W_ReleaseLumpNum};

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
    pub m_paninc: mpoint_t,
    pub mtof_zoommul: fixed_t,
    pub ftom_zoommul: fixed_t,
    pub m_y: fixed_t,
    pub m_x: fixed_t,
    pub m_x2: fixed_t,
    pub m_y2: fixed_t,
    pub m_w: fixed_t,
    pub m_h: fixed_t,
    pub min_x: fixed_t,
    pub min_y: fixed_t,
    pub max_x: fixed_t,
    pub max_y: fixed_t,
    pub max_w: fixed_t,
    pub max_h: fixed_t,
    pub min_w: fixed_t,
    pub min_h: fixed_t,
    pub min_scale_mtof: fixed_t,
    pub max_scale_mtof: fixed_t,
    pub old_m_h: fixed_t,
    pub old_m_w: fixed_t,
    pub old_m_y: fixed_t,
    pub old_m_x: fixed_t,
    pub f_oldloc: mpoint_t,
    pub scale_mtof: fixed_t,
    pub scale_ftom: fixed_t,
    pub plr: PlayerId,
    pub marknums: [i32; 10],
    pub markpoints: [mpoint_t; 10],
    pub markpointnum: i32,
    pub followplayer: bool,
    pub cheat_amap: cheatseq_t,
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
            m_paninc: mpoint_t { x: 0, y: 0 },
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
            f_oldloc: mpoint_t { x: 0, y: 0 },
            scale_mtof: INITSCALEMTOF as fixed_t,
            scale_ftom: 0,
            plr: PlayerId(0),
            marknums: [-1; 10],
            markpoints: [mpoint_t { x: 0, y: 0 }; 10],
            markpointnum: 0,
            followplayer: true,
            cheat_amap: cheatseq_t::new("iddt", 0),
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
pub struct mpoint_t {
    pub x: fixed_t,
    pub y: fixed_t,
}
#[derive(Copy, Clone)]
pub struct mline_t {
    pub a: mpoint_t,
    pub b: mpoint_t,
}
#[derive(Copy, Clone)]
pub struct fline_t {
    pub a: fpoint_t,
    pub b: fpoint_t,
}
#[derive(Copy, Clone)]
pub struct fpoint_t {
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
pub static player_arrow: [mline_t; 7] = [
    mline_t {
        a: mpoint_t {
            x: -R_0 + R_0 / 8,
            y: 0,
        },
        b: mpoint_t { x: R_0, y: 0 },
    },
    mline_t {
        a: mpoint_t { x: R_0, y: 0 },
        b: mpoint_t {
            x: R_0 - R_0 / 2,
            y: R_0 / 4,
        },
    },
    mline_t {
        a: mpoint_t { x: R_0, y: 0 },
        b: mpoint_t {
            x: R_0 - R_0 / 2,
            y: -R_0 / 4,
        },
    },
    mline_t {
        a: mpoint_t {
            x: -R_0 + R_0 / 8,
            y: 0,
        },
        b: mpoint_t {
            x: -R_0 - R_0 / 8,
            y: R_0 / 4,
        },
    },
    mline_t {
        a: mpoint_t {
            x: -R_0 + R_0 / 8,
            y: 0,
        },
        b: mpoint_t {
            x: -R_0 - R_0 / 8,
            y: -R_0 / 4,
        },
    },
    mline_t {
        a: mpoint_t {
            x: -R_0 + 3 * R_0 / 8,
            y: 0,
        },
        b: mpoint_t {
            x: -R_0 + R_0 / 8,
            y: R_0 / 4,
        },
    },
    mline_t {
        a: mpoint_t {
            x: -R_0 + 3 * R_0 / 8,
            y: 0,
        },
        b: mpoint_t {
            x: -R_0 + R_0 / 8,
            y: -R_0 / 4,
        },
    },
];
pub const R_1: i32 = 8 * 16 * FRACUNIT / 7;
pub static cheat_player_arrow: [mline_t; 16] = [
    mline_t {
        a: mpoint_t {
            x: -R_1 + R_1 / 8,
            y: 0,
        },
        b: mpoint_t { x: R_1, y: 0 },
    },
    mline_t {
        a: mpoint_t { x: R_1, y: 0 },
        b: mpoint_t {
            x: R_1 - R_1 / 2,
            y: R_1 / 6,
        },
    },
    mline_t {
        a: mpoint_t { x: R_1, y: 0 },
        b: mpoint_t {
            x: R_1 - R_1 / 2,
            y: -R_1 / 6,
        },
    },
    mline_t {
        a: mpoint_t {
            x: -R_1 + R_1 / 8,
            y: 0,
        },
        b: mpoint_t {
            x: -R_1 - R_1 / 8,
            y: R_1 / 6,
        },
    },
    mline_t {
        a: mpoint_t {
            x: -R_1 + R_1 / 8,
            y: 0,
        },
        b: mpoint_t {
            x: -R_1 - R_1 / 8,
            y: -R_1 / 6,
        },
    },
    mline_t {
        a: mpoint_t {
            x: -R_1 + 3 * R_1 / 8,
            y: 0,
        },
        b: mpoint_t {
            x: -R_1 + R_1 / 8,
            y: R_1 / 6,
        },
    },
    mline_t {
        a: mpoint_t {
            x: -R_1 + 3 * R_1 / 8,
            y: 0,
        },
        b: mpoint_t {
            x: -R_1 + R_1 / 8,
            y: -R_1 / 6,
        },
    },
    mline_t {
        a: mpoint_t { x: -R_1 / 2, y: 0 },
        b: mpoint_t {
            x: -R_1 / 2,
            y: -R_1 / 6,
        },
    },
    mline_t {
        a: mpoint_t {
            x: -R_1 / 2,
            y: -R_1 / 6,
        },
        b: mpoint_t {
            x: -R_1 / 2 + R_1 / 6,
            y: -R_1 / 6,
        },
    },
    mline_t {
        a: mpoint_t {
            x: -R_1 / 2 + R_1 / 6,
            y: -R_1 / 6,
        },
        b: mpoint_t {
            x: -R_1 / 2 + R_1 / 6,
            y: R_1 / 4,
        },
    },
    mline_t {
        a: mpoint_t { x: -R_1 / 6, y: 0 },
        b: mpoint_t {
            x: -R_1 / 6,
            y: -R_1 / 6,
        },
    },
    mline_t {
        a: mpoint_t {
            x: -R_1 / 6,
            y: -R_1 / 6,
        },
        b: mpoint_t { x: 0, y: -R_1 / 6 },
    },
    mline_t {
        a: mpoint_t { x: 0, y: -R_1 / 6 },
        b: mpoint_t { x: 0, y: R_1 / 4 },
    },
    mline_t {
        a: mpoint_t {
            x: R_1 / 6,
            y: R_1 / 4,
        },
        b: mpoint_t {
            x: R_1 / 6,
            y: -R_1 / 7,
        },
    },
    mline_t {
        a: mpoint_t {
            x: R_1 / 6,
            y: -R_1 / 7,
        },
        b: mpoint_t {
            x: R_1 / 6 + R_1 / 32,
            y: -R_1 / 7 - R_1 / 32,
        },
    },
    mline_t {
        a: mpoint_t {
            x: R_1 / 6 + R_1 / 32,
            y: -R_1 / 7 - R_1 / 32,
        },
        b: mpoint_t {
            x: R_1 / 6 + R_1 / 10,
            y: -R_1 / 7,
        },
    },
];
pub const R: i32 = 1 << FRACBITS;
pub static thintriangle_guy: [mline_t; 3] = [
    mline_t {
        a: mpoint_t {
            x: (-0.5f64 * R as f64) as fixed_t,
            y: (-0.7f64 * R as f64) as fixed_t,
        },
        b: mpoint_t {
            x: 1 << FRACBITS,
            y: 0,
        },
    },
    mline_t {
        a: mpoint_t {
            x: 1 << FRACBITS,
            y: 0,
        },
        b: mpoint_t {
            x: (-0.5f64 * R as f64) as fixed_t,
            y: (0.7f64 * R as f64) as fixed_t,
        },
    },
    mline_t {
        a: mpoint_t {
            x: (-0.5f64 * R as f64) as fixed_t,
            y: (0.7f64 * R as f64) as fixed_t,
        },
        b: mpoint_t {
            x: (-0.5f64 * R as f64) as fixed_t,
            y: (-0.7f64 * R as f64) as fixed_t,
        },
    },
];
static FINIT_WIDTH: i32 = SCREENWIDTH;
static FINIT_HEIGHT: i32 = SCREENHEIGHT - 32;
pub fn AM_activateNewScale(state: &mut GameState) {
    state.am_map.m_x += state.am_map.m_w / 2;
    state.am_map.m_y += state.am_map.m_h / 2;
    state.am_map.m_w = FixedMul((state.am_map.f_w as fixed_t) << 16, state.am_map.scale_ftom);
    state.am_map.m_h = FixedMul((state.am_map.f_h as fixed_t) << 16, state.am_map.scale_ftom);
    state.am_map.m_x -= state.am_map.m_w / 2;
    state.am_map.m_y -= state.am_map.m_h / 2;
    state.am_map.m_x2 = state.am_map.m_x + state.am_map.m_w;
    state.am_map.m_y2 = state.am_map.m_y + state.am_map.m_h;
}
pub fn AM_saveScaleAndLoc(state: &mut GameState) {
    state.am_map.old_m_x = state.am_map.m_x;
    state.am_map.old_m_y = state.am_map.m_y;
    state.am_map.old_m_w = state.am_map.m_w;
    state.am_map.old_m_h = state.am_map.m_h;
}
pub fn AM_restoreScaleAndLoc(state: &mut GameState) {
    state.am_map.m_w = state.am_map.old_m_w;
    state.am_map.m_h = state.am_map.old_m_h;
    if !state.am_map.followplayer {
        state.am_map.m_x = state.am_map.old_m_x;
        state.am_map.m_y = state.am_map.old_m_y;
    } else {
        let plr_mo_id = state.g_game.player_mut(state.am_map.plr).mo.unwrap();
        let plr_mo = state.p_mobj.mo(plr_mo_id);
        state.am_map.m_x = (plr_mo.x - state.am_map.m_w / 2) as fixed_t;
        state.am_map.m_y = (plr_mo.y - state.am_map.m_h / 2) as fixed_t;
    }
    state.am_map.m_x2 = state.am_map.m_x + state.am_map.m_w;
    state.am_map.m_y2 = state.am_map.m_y + state.am_map.m_h;
    state.am_map.scale_mtof = FixedDiv((state.am_map.f_w as fixed_t) << FRACBITS, state.am_map.m_w);
    state.am_map.scale_ftom = FixedDiv(FRACUNIT, state.am_map.scale_mtof);
}
pub fn AM_addMark(state: &mut GameState) {
    state.am_map.markpoints[state.am_map.markpointnum as usize].x =
        (state.am_map.m_x + state.am_map.m_w / 2) as fixed_t;
    state.am_map.markpoints[state.am_map.markpointnum as usize].y =
        (state.am_map.m_y + state.am_map.m_h / 2) as fixed_t;
    state.am_map.markpointnum = (state.am_map.markpointnum + 1) % AM_NUMMARKPOINTS;
}
pub fn AM_findMinMaxBoundaries(state: &mut GameState) {
    state.am_map.min_y = INT_MAX as fixed_t;
    state.am_map.min_x = state.am_map.min_y;
    state.am_map.max_y = -INT_MAX as fixed_t;
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
    state.am_map.min_w = (2 * 16 * FRACUNIT) as fixed_t;
    state.am_map.min_h = (2 * 16 * FRACUNIT) as fixed_t;
    let a: fixed_t = FixedDiv(
        (state.am_map.f_w as fixed_t) << FRACBITS,
        state.am_map.max_w,
    );
    let b: fixed_t = FixedDiv(
        (state.am_map.f_h as fixed_t) << FRACBITS,
        state.am_map.max_h,
    );
    state.am_map.min_scale_mtof = if a < b { a } else { b };
    state.am_map.max_scale_mtof =
        FixedDiv((state.am_map.f_h as fixed_t) << FRACBITS, 2 * 16 * FRACUNIT);
}
pub fn AM_changeWindowLoc(state: &mut GameState) {
    if state.am_map.m_paninc.x != 0 || state.am_map.m_paninc.y != 0 {
        state.am_map.followplayer = false;
        state.am_map.f_oldloc.x = INT_MAX as fixed_t;
    }
    state.am_map.m_x += state.am_map.m_paninc.x;
    state.am_map.m_y += state.am_map.m_paninc.y;
    if state.am_map.m_x + state.am_map.m_w / 2 > state.am_map.max_x {
        state.am_map.m_x = (state.am_map.max_x - state.am_map.m_w / 2) as fixed_t;
    } else if (state.am_map.m_x + state.am_map.m_w / 2) < state.am_map.min_x {
        state.am_map.m_x = (state.am_map.min_x - state.am_map.m_w / 2) as fixed_t;
    }
    if state.am_map.m_y + state.am_map.m_h / 2 > state.am_map.max_y {
        state.am_map.m_y = (state.am_map.max_y - state.am_map.m_h / 2) as fixed_t;
    } else if (state.am_map.m_y + state.am_map.m_h / 2) < state.am_map.min_y {
        state.am_map.m_y = (state.am_map.min_y - state.am_map.m_h / 2) as fixed_t;
    }
    state.am_map.m_x2 = state.am_map.m_x + state.am_map.m_w;
    state.am_map.m_y2 = state.am_map.m_y + state.am_map.m_h;
}
pub fn AM_initVariables(state: &mut GameState) {
    let mut pnum: i32;
    const ST_NOTIFY: event_t = event_t {
        kind: EvType::ev_keyup,
        data1: AM_MSGENTERED,
        data2: 0,
        data3: 0,
        data4: 0,
    };
    state.am_map.automapactive = true;
    state.am_map.f_oldloc.x = INT_MAX as fixed_t;
    state.am_map.amclock = 0;
    state.am_map.lightlev = 0;
    state.am_map.m_paninc.y = 0;
    state.am_map.m_paninc.x = state.am_map.m_paninc.y;
    state.am_map.ftom_zoommul = FRACUNIT as fixed_t;
    state.am_map.mtof_zoommul = FRACUNIT as fixed_t;
    state.am_map.m_w = FixedMul((state.am_map.f_w as fixed_t) << 16, state.am_map.scale_ftom);
    state.am_map.m_h = FixedMul((state.am_map.f_h as fixed_t) << 16, state.am_map.scale_ftom);
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
    state.am_map.m_x = (plr_mo.x - state.am_map.m_w / 2) as fixed_t;
    state.am_map.m_y = (plr_mo.y - state.am_map.m_h / 2) as fixed_t;
    AM_changeWindowLoc(state);
    state.am_map.old_m_x = state.am_map.m_x;
    state.am_map.old_m_y = state.am_map.m_y;
    state.am_map.old_m_w = state.am_map.m_w;
    state.am_map.old_m_h = state.am_map.m_h;
    ST_Responder(state, &ST_NOTIFY);
}
pub fn AM_loadPics(state: &mut GameState) {
    for i in 0..10 {
        let namebuf = format!("AMMNUM{}", i);
        let lumpnum = W_GetNumForName(&mut state.w_wad, &namebuf);
        W_LumpBytes(state, lumpnum);
        state.am_map.marknums[i as usize] = lumpnum;
    }
}
pub fn AM_unloadPics(state: &mut GameState) {
    for i in 0..10 {
        W_ReleaseLumpNum(&mut state.w_wad, state.am_map.marknums[i]);
    }
}
pub fn AM_clearMarks(state: &mut GameState) {
    for i in 0..(AM_NUMMARKPOINTS as usize) {
        state.am_map.markpoints[i].x = -1;
    }
    state.am_map.markpointnum = 0;
}
pub fn AM_LevelInit(state: &mut GameState) {
    state.am_map.leveljuststarted = false;
    state.am_map.f_y = 0;
    state.am_map.f_x = state.am_map.f_y;
    state.am_map.f_w = FINIT_WIDTH;
    state.am_map.f_h = FINIT_HEIGHT;
    AM_clearMarks(state);
    AM_findMinMaxBoundaries(state);
    state.am_map.scale_mtof = FixedDiv(
        state.am_map.min_scale_mtof,
        (0.7f64 * FRACUNIT as f64) as fixed_t,
    );
    if state.am_map.scale_mtof > state.am_map.max_scale_mtof {
        state.am_map.scale_mtof = state.am_map.min_scale_mtof;
    }
    state.am_map.scale_ftom = FixedDiv(FRACUNIT, state.am_map.scale_mtof);
}
pub fn AM_Stop(state: &mut GameState) {
    const ST_NOTIFY: event_t = event_t {
        kind: EvType::ev_keydown,
        data1: EvType::ev_keyup as i32,
        data2: AM_MSGEXITED,
        data3: 0,
        data4: 0,
    };
    AM_unloadPics(state);
    state.am_map.automapactive = false;
    ST_Responder(state, &ST_NOTIFY);
    state.am_map.stopped = true;
}
pub fn AM_Start(state: &mut GameState) {
    if !state.am_map.stopped {
        AM_Stop(state);
    }
    state.am_map.stopped = false;
    if state.am_map.am_start_lastlevel != state.g_game.gamemap
        || state.am_map.am_start_lastepisode != state.g_game.gameepisode
    {
        AM_LevelInit(state);
        state.am_map.am_start_lastlevel = state.g_game.gamemap;
        state.am_map.am_start_lastepisode = state.g_game.gameepisode;
    }
    AM_initVariables(state);
    AM_loadPics(state);
}
pub fn AM_minOutWindowScale(state: &mut GameState) {
    state.am_map.scale_mtof = state.am_map.min_scale_mtof;
    state.am_map.scale_ftom = FixedDiv(FRACUNIT, state.am_map.scale_mtof);
    AM_activateNewScale(state);
}
pub fn AM_maxOutWindowScale(state: &mut GameState) {
    state.am_map.scale_mtof = state.am_map.max_scale_mtof;
    state.am_map.scale_ftom = FixedDiv(FRACUNIT, state.am_map.scale_mtof);
    AM_activateNewScale(state);
}
pub fn AM_Responder(state: &mut GameState, ev: &event_t) -> bool {
    let mut rc: bool;
    let key: i32;
    rc = false;
    if !state.am_map.automapactive {
        if ev.kind == EvType::ev_keydown && ev.data1 == state.m_controls.key_map_toggle {
            AM_Start(state);
            state.g_game.viewactive = false;
            rc = true;
        }
    } else if ev.kind == EvType::ev_keydown {
        rc = true;
        key = ev.data1;
        if key == state.m_controls.key_map_east {
            if !state.am_map.followplayer {
                state.am_map.m_paninc.x = FixedMul((4) << 16, state.am_map.scale_ftom);
            } else {
                rc = false;
            }
        } else if key == state.m_controls.key_map_west {
            if !state.am_map.followplayer {
                state.am_map.m_paninc.x = -FixedMul((4) << 16, state.am_map.scale_ftom);
            } else {
                rc = false;
            }
        } else if key == state.m_controls.key_map_north {
            if !state.am_map.followplayer {
                state.am_map.m_paninc.y = FixedMul((4) << 16, state.am_map.scale_ftom);
            } else {
                rc = false;
            }
        } else if key == state.m_controls.key_map_south {
            if !state.am_map.followplayer {
                state.am_map.m_paninc.y = -FixedMul((4) << 16, state.am_map.scale_ftom);
            } else {
                rc = false;
            }
        } else if key == state.m_controls.key_map_zoomout {
            state.am_map.mtof_zoommul = M_ZOOMOUT as fixed_t;
            state.am_map.ftom_zoommul = M_ZOOMIN as fixed_t;
        } else if key == state.m_controls.key_map_zoomin {
            state.am_map.mtof_zoommul = M_ZOOMIN as fixed_t;
            state.am_map.ftom_zoommul = M_ZOOMOUT as fixed_t;
        } else if key == state.m_controls.key_map_toggle {
            state.am_map.am_responder_bigstate = false;
            state.g_game.viewactive = true;
            AM_Stop(state);
        } else if key == state.m_controls.key_map_maxzoom {
            state.am_map.am_responder_bigstate = !state.am_map.am_responder_bigstate;
            if state.am_map.am_responder_bigstate {
                AM_saveScaleAndLoc(state);
                AM_minOutWindowScale(state);
            } else {
                AM_restoreScaleAndLoc(state);
            }
        } else if key == state.m_controls.key_map_follow {
            state.am_map.followplayer = !state.am_map.followplayer;
            state.am_map.f_oldloc.x = INT_MAX as fixed_t;
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
            AM_addMark(state);
        } else if key == state.m_controls.key_map_clearmark {
            AM_clearMarks(state);
            state.g_game.player_mut(state.am_map.plr).message =
                Some("All Marks Cleared".to_string());
        } else {
            rc = false;
        }
        if state.g_game.deathmatch == 0
            && cht_CheckCheat(&mut state.am_map.cheat_amap, ev.data2 as u8)
        {
            rc = false;
            state.am_map.cheating = (state.am_map.cheating + 1) % 3;
        }
    } else if ev.kind == EvType::ev_keyup {
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
            state.am_map.mtof_zoommul = FRACUNIT as fixed_t;
            state.am_map.ftom_zoommul = FRACUNIT as fixed_t;
        }
    }
    rc
}
pub fn AM_changeWindowScale(state: &mut GameState) {
    state.am_map.scale_mtof = FixedMul(state.am_map.scale_mtof, state.am_map.mtof_zoommul);
    state.am_map.scale_ftom = FixedDiv(FRACUNIT, state.am_map.scale_mtof);
    if state.am_map.scale_mtof < state.am_map.min_scale_mtof {
        AM_minOutWindowScale(state);
    } else if state.am_map.scale_mtof > state.am_map.max_scale_mtof {
        AM_maxOutWindowScale(state);
    } else {
        AM_activateNewScale(state);
    };
}
pub fn AM_doFollowPlayer(state: &mut GameState) {
    let plr_mo_id = state.g_game.player_mut(state.am_map.plr).mo.unwrap();
    let plr_mo = state.p_mobj.mo(plr_mo_id);
    let (plr_x, plr_y) = (plr_mo.x, plr_mo.y);
    if state.am_map.f_oldloc.x != plr_x || state.am_map.f_oldloc.y != plr_y {
        state.am_map.m_x = (FixedMul(
            (FixedMul(plr_x, state.am_map.scale_mtof) >> 16) << 16,
            state.am_map.scale_ftom,
        ) - state.am_map.m_w / 2) as fixed_t;
        state.am_map.m_y = (FixedMul(
            (FixedMul(plr_y, state.am_map.scale_mtof) >> 16) << 16,
            state.am_map.scale_ftom,
        ) - state.am_map.m_h / 2) as fixed_t;
        state.am_map.m_x2 = state.am_map.m_x + state.am_map.m_w;
        state.am_map.m_y2 = state.am_map.m_y + state.am_map.m_h;
        state.am_map.f_oldloc.x = plr_x;
        state.am_map.f_oldloc.y = plr_y;
    }
}
pub fn AM_Ticker(state: &mut GameState) {
    if !state.am_map.automapactive {
        return;
    }
    state.am_map.amclock += 1;
    if state.am_map.followplayer {
        AM_doFollowPlayer(state);
    }
    if state.am_map.ftom_zoommul != FRACUNIT {
        AM_changeWindowScale(state);
    }
    if state.am_map.m_paninc.x != 0 || state.am_map.m_paninc.y != 0 {
        AM_changeWindowLoc(state);
    }
}
pub fn AM_clearFB(state: &mut GameState, color: i32) {
    let len = (state.am_map.f_w * state.am_map.f_h) as usize;
    state.i_video.I_VideoBuffer[..len].fill(color as u8);
}
pub fn AM_clipMline(state: &mut GameState, ml: &mline_t, fl: &mut fline_t) -> bool {
    let mut outcode1: i32 = 0;
    let mut outcode2: i32 = 0;
    let mut outside: i32;
    let mut tmp: fpoint_t = fpoint_t { x: 0, y: 0 };
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
    fl.a.x = state.am_map.f_x as fixed_t
        + (FixedMul(ml.a.x - state.am_map.m_x, state.am_map.scale_mtof) >> 16);
    fl.a.y = state.am_map.f_y as fixed_t
        + (state.am_map.f_h as fixed_t
            - (FixedMul(ml.a.y - state.am_map.m_y, state.am_map.scale_mtof) >> 16));
    fl.b.x = state.am_map.f_x as fixed_t
        + (FixedMul(ml.b.x - state.am_map.m_x, state.am_map.scale_mtof) >> 16);
    fl.b.y = state.am_map.f_y as fixed_t
        + (state.am_map.f_h as fixed_t
            - (FixedMul(ml.b.y - state.am_map.m_y, state.am_map.scale_mtof) >> 16));
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
pub fn AM_drawFline(state: &mut GameState, fl: &fline_t, color: i32) {
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
            state.i_video.I_VideoBuffer[(y * state.am_map.f_w + x) as usize] = color as u8;
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
            state.i_video.I_VideoBuffer[(y * state.am_map.f_w + x) as usize] = color as u8;
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
pub fn AM_drawMline(state: &mut GameState, ml: &mline_t, color: i32) {
    let mut fl: fline_t = fline_t {
        a: fpoint_t { x: 0, y: 0 },
        b: fpoint_t { x: 0, y: 0 },
    };
    if AM_clipMline(state, ml, &mut fl) {
        AM_drawFline(state, &fl, color);
    }
}
pub fn AM_drawGrid(state: &mut GameState, color: i32) {
    let mut x: fixed_t;
    let mut y: fixed_t;
    let mut start: fixed_t;
    let mut end: fixed_t;
    let mut ml: mline_t = mline_t {
        a: mpoint_t { x: 0, y: 0 },
        b: mpoint_t { x: 0, y: 0 },
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
        AM_drawMline(state, &ml, color);
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
        AM_drawMline(state, &ml, color);
        y += MAPBLOCKUNITS << FRACBITS;
    }
}
pub fn AM_drawWalls(state: &mut GameState) {
    let mut l: mline_t = mline_t {
        a: mpoint_t { x: 0, y: 0 },
        b: mpoint_t { x: 0, y: 0 },
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
                        AM_drawMline(state, &l, WALLCOLORS + lightlev);
                    }
                    Some(li_backsector) => {
                        if li_special == 39 {
                            AM_drawMline(state, &l, WALLCOLORS + WALLRANGE / 2);
                        } else if li_flags & ML_SECRET != 0 {
                            if state.am_map.cheating != 0 {
                                AM_drawMline(state, &l, SECRETWALLCOLORS + lightlev);
                            } else {
                                AM_drawMline(state, &l, WALLCOLORS + lightlev);
                            }
                        } else if state.p_setup.sector_mut(li_backsector).floorheight
                            != state
                                .p_setup
                                .sector_mut(li_frontsector.unwrap())
                                .floorheight
                        {
                            AM_drawMline(state, &l, FDWALLCOLORS + lightlev);
                        } else if state.p_setup.sector_mut(li_backsector).ceilingheight
                            != state
                                .p_setup
                                .sector_mut(li_frontsector.unwrap())
                                .ceilingheight
                        {
                            AM_drawMline(state, &l, CDWALLCOLORS + lightlev);
                        } else if state.am_map.cheating != 0 {
                            AM_drawMline(state, &l, TSWALLCOLORS + lightlev);
                        }
                    }
                }
            }
        } else if state.g_game.player_mut(state.am_map.plr).powers[PowerType::pw_allmap as usize]
            != 0
            && li_flags & LINE_NEVERSEE == 0
        {
            AM_drawMline(state, &l, GRAYS + 3);
        }
    }
}
pub fn AM_rotate(x: &mut fixed_t, y: &mut fixed_t, a: angle_t) {
    let tmpx: fixed_t = FixedMul(*x, finecosine[(a >> ANGLETOFINESHIFT) as usize])
        - FixedMul(*y, finesine[(a >> ANGLETOFINESHIFT) as usize]);
    *y = FixedMul(*x, finesine[(a >> ANGLETOFINESHIFT) as usize])
        + FixedMul(*y, finecosine[(a >> ANGLETOFINESHIFT) as usize]);
    *x = tmpx;
}
pub fn AM_drawLineCharacter(
    state: &mut GameState,
    lineguy: &[mline_t],
    scale: fixed_t,
    angle: angle_t,
    color: i32,
    x: fixed_t,
    y: fixed_t,
) {
    let mut l: mline_t = mline_t {
        a: mpoint_t { x: 0, y: 0 },
        b: mpoint_t { x: 0, y: 0 },
    };
    for line in lineguy {
        l.a.x = line.a.x;
        l.a.y = line.a.y;
        if scale != 0 {
            l.a.x = FixedMul(scale, l.a.x);
            l.a.y = FixedMul(scale, l.a.y);
        }
        if angle != 0 {
            AM_rotate(&mut l.a.x, &mut l.a.y, angle);
        }
        l.a.x += x;
        l.a.y += y;
        l.b.x = line.b.x;
        l.b.y = line.b.y;
        if scale != 0 {
            l.b.x = FixedMul(scale, l.b.x);
            l.b.y = FixedMul(scale, l.b.y);
        }
        if angle != 0 {
            AM_rotate(&mut l.b.x, &mut l.b.y, angle);
        }
        l.b.x += x;
        l.b.y += y;
        AM_drawMline(state, &l, color);
    }
}
pub fn AM_drawPlayers(state: &mut GameState) {
    const THEIR_COLORS: [i32; 4] = [GREENS, GRAYS, BROWNS, REDS];
    let mut their_color: i32 = -1;
    let mut color: i32;
    if !state.g_game.netgame {
        let plr_mo_id = state.g_game.player_mut(state.am_map.plr).mo.unwrap();
        let plr_mo = state.p_mobj.mo(plr_mo_id);
        let (plr_angle, plr_x, plr_y) = (plr_mo.angle, plr_mo.x, plr_mo.y);
        if state.am_map.cheating != 0 {
            AM_drawLineCharacter(
                state,
                &cheat_player_arrow,
                0,
                plr_angle,
                WHITE,
                plr_x,
                plr_y,
            );
        } else {
            AM_drawLineCharacter(state, &player_arrow, 0, plr_angle, WHITE, plr_x, plr_y);
        }
        return;
    }
    for i in 0..MAXPLAYERS {
        their_color += 1;
        let p = &state.g_game.players[i as usize];
        let (p_invisibility, p_mo_id) = (p.powers[PowerType::pw_invisibility as usize], p.mo);
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
            AM_drawLineCharacter(state, &player_arrow, 0, p_angle, color, p_x, p_y);
        }
    }
}
pub fn AM_drawThings(state: &mut GameState, colors: i32) {
    for i in 0..(state.p_setup.numsectors as usize) {
        let mut cursor = state.p_setup.sectors[i].thinglist;
        while let Some(id) = cursor {
            let t = state.p_mobj.mo(id);
            let (t_angle, t_x, t_y, t_snext) = (t.angle, t.x, t.y, t.snext);
            let lightlev = state.am_map.lightlev;
            AM_drawLineCharacter(
                state,
                &thintriangle_guy,
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
pub fn AM_drawMarks(state: &mut GameState) {
    let mut fx: i32;
    let mut fy: i32;
    let mut w: i32;
    let mut h: i32;
    for i in 0..(AM_NUMMARKPOINTS as usize) {
        if state.am_map.markpoints[i].x != -1 {
            w = 5;
            h = 6;
            fx = state.am_map.f_x as fixed_t
                + (FixedMul(
                    state.am_map.markpoints[i].x - state.am_map.m_x,
                    state.am_map.scale_mtof,
                ) >> 16);
            fy = state.am_map.f_y as fixed_t
                + (state.am_map.f_h as fixed_t
                    - (FixedMul(
                        state.am_map.markpoints[i].y - state.am_map.m_y,
                        state.am_map.scale_mtof,
                    ) >> 16));
            if fx >= state.am_map.f_x
                && fx <= state.am_map.f_w - w
                && fy >= state.am_map.f_y
                && fy <= state.am_map.f_h - h
            {
                let lumpnum = state.am_map.marknums[i];
                let patch = V_CachePatchNum(state, lumpnum);
                let dest_screen = Screen::Video;
                V_DrawPatch(state, dest_screen, fx, fy, &patch);
            }
        }
    }
}
pub fn AM_drawCrosshair(state: &mut GameState, color: i32) {
    let idx = (state.am_map.f_w * (state.am_map.f_h + 1) / 2) as usize;
    state.i_video.I_VideoBuffer[idx] = color as u8;
}
pub fn AM_Drawer(state: &mut GameState) {
    if !state.am_map.automapactive {
        return;
    }
    AM_clearFB(state, BACKGROUND);
    if state.am_map.grid {
        AM_drawGrid(state, GRIDCOLORS);
    }
    AM_drawWalls(state);
    AM_drawPlayers(state);
    if state.am_map.cheating == 2 {
        AM_drawThings(state, THINGCOLORS);
    }
    AM_drawCrosshair(state, XHAIRCOLORS);
    AM_drawMarks(state);
    let (f_x, f_y, f_w, f_h) = (
        state.am_map.f_x,
        state.am_map.f_y,
        state.am_map.f_w,
        state.am_map.f_h,
    );
    let dest_screen = Screen::Video;
    V_MarkRect(state, dest_screen, f_x, f_y, f_w, f_h);
}
