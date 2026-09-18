use crate::d_mode::GameMode_t;
use crate::d_ticcmd::{BT_ATTACK, BT_USE};
use crate::doomdef::false_0;
use crate::doomdef::true_0;
use crate::doomdef::MAXPLAYERS;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::doomdef::TICRATE;
use crate::g_game::G_WorldDone;
use crate::game_state::GameState;
use crate::v_video::Screen;
use crate::m_random::M_Random;
use crate::s_sound::S_ChangeMusic;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::{mus_dm2int, mus_inter};
use crate::sounds::{sfx_barexp, sfx_pistol, sfx_pldeth, sfx_sgcock, sfx_slop};
use crate::st_stuff::load_callback_t;
use crate::v_video::V_CachePatchNum;
use crate::v_video::V_DrawPatch;
use crate::w_wad::{W_CacheLumpNum, W_CheckNumForName, W_GetNumForName, W_ReleaseLumpName};

pub struct WiStuffState {
    pub epsd0animinfo: [anim_t; 10],
    pub epsd1animinfo: [anim_t; 9],
    pub epsd2animinfo: [anim_t; 6],
    pub NUMANIMS: [i32; 4],
    pub acceleratestage: i32,
    pub me: i32,
    pub state: StateEnum,
    pub cnt: i32,
    pub bcnt: i32,
    pub firstrefresh: i32,
    pub cnt_kills: [i32; 4],
    pub cnt_items: [i32; 4],
    pub cnt_secret: [i32; 4],
    pub cnt_time: i32,
    pub cnt_par: i32,
    pub cnt_pause: i32,
    pub NUMCMAPS: i32,
    pub yah: [i32; 3],
    pub splat: [i32; 2],
    pub percent: i32,
    pub colon: i32,
    pub num: [i32; 10],
    pub wiminus: i32,
    pub finished: i32,
    pub entering: i32,
    pub sp_secret: i32,
    pub kills: i32,
    pub secret: i32,
    pub items: i32,
    pub frags: i32,
    pub timepatch: i32,
    pub par: i32,
    pub sucks: i32,
    pub killers: i32,
    pub victims: i32,
    pub total: i32,
    pub star: i32,
    pub bstar: i32,
    pub p: [i32; 4],
    pub bp: [i32; 4],
    pub lnames: Vec<i32>,
    pub background: i32,
    pub snl_pointeron: bool,
    pub dm_state: i32,
    pub dm_frags: [[i32; 4]; 4],
    pub dm_totals: [i32; 4],
    pub cnt_frags: [i32; 4],
    pub dofrags: i32,
    pub ng_state: i32,
    pub sp_state: i32,
}

impl WiStuffState {
    pub const fn new() -> Self {
        WiStuffState {
            epsd0animinfo: [
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 224_i32,
                        y: 104_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 184_i32,
                        y: 160_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 112_i32,
                        y: 136_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 72_i32,
                        y: 112_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 88_i32,
                        y: 96_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 64_i32,
                        y: 48_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 192_i32,
                        y: 40_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 136_i32,
                        y: 16_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 80_i32,
                        y: 16_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 64_i32,
                        y: 24_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
            ],
            epsd1animinfo: [
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35_i32 / 3_i32,
                    nanims: 1_i32,
                    loc: point_t {
                        x: 128_i32,
                        y: 136_i32,
                    },
                    data1: 1_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35_i32 / 3_i32,
                    nanims: 1_i32,
                    loc: point_t {
                        x: 128_i32,
                        y: 136_i32,
                    },
                    data1: 2_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35_i32 / 3_i32,
                    nanims: 1_i32,
                    loc: point_t {
                        x: 128_i32,
                        y: 136_i32,
                    },
                    data1: 3_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35_i32 / 3_i32,
                    nanims: 1_i32,
                    loc: point_t {
                        x: 128_i32,
                        y: 136_i32,
                    },
                    data1: 4_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35_i32 / 3_i32,
                    nanims: 1_i32,
                    loc: point_t {
                        x: 128_i32,
                        y: 136_i32,
                    },
                    data1: 5_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35_i32 / 3_i32,
                    nanims: 1_i32,
                    loc: point_t {
                        x: 128_i32,
                        y: 136_i32,
                    },
                    data1: 6_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35_i32 / 3_i32,
                    nanims: 1_i32,
                    loc: point_t {
                        x: 128_i32,
                        y: 136_i32,
                    },
                    data1: 7_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 192_i32,
                        y: 144_i32,
                    },
                    data1: 8_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35_i32 / 3_i32,
                    nanims: 1_i32,
                    loc: point_t {
                        x: 128_i32,
                        y: 136_i32,
                    },
                    data1: 8_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
            ],
            epsd2animinfo: [
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 104_i32,
                        y: 168_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 40_i32,
                        y: 136_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 160_i32,
                        y: 96_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 104_i32,
                        y: 80_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 3_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 120_i32,
                        y: 32_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35_i32 / 4_i32,
                    nanims: 3_i32,
                    loc: point_t {
                        x: 40_i32,
                        y: 0_i32,
                    },
                    data1: 0_i32,
                    data2: 0_i32,
                    p: [-1, -1, -1],
                    nexttic: 0_i32,
                    lastdrawn: 0_i32,
                    ctr: 0_i32,
                    state: 0_i32,
                },
            ],
            NUMANIMS: [0; 4],
            acceleratestage: 0,
            me: 0,
            state: StateEnum::StatCount,
            cnt: 0,
            bcnt: 0,
            firstrefresh: 0,
            cnt_kills: [0; 4],
            cnt_items: [0; 4],
            cnt_secret: [0; 4],
            cnt_time: 0,
            cnt_par: 0,
            cnt_pause: 0,
            NUMCMAPS: 0,
            yah: [-1, -1, -1],
            splat: [-1, -1],
            percent: -1,
            colon: -1,
            num: [-1; 10],
            wiminus: -1,
            finished: -1,
            entering: -1,
            sp_secret: -1,
            kills: -1,
            secret: -1,
            items: -1,
            frags: -1,
            timepatch: -1,
            par: -1,
            sucks: -1,
            killers: -1,
            victims: -1,
            total: -1,
            star: -1,
            bstar: -1,
            p: [-1; 4],
            bp: [-1; 4],
            lnames: Vec::new(),
            background: -1,
            snl_pointeron: false,
            dm_state: 0,
            dm_frags: [[0; 4]; 4],
            dm_totals: [0; 4],
            cnt_frags: [0; 4],
            dofrags: 0,
            ng_state: 0,
            sp_state: 0,
        }
    }

    pub fn anims(&mut self) -> [&mut [anim_t]; 4] {
        [
            &mut self.epsd0animinfo,
            &mut self.epsd1animinfo,
            &mut self.epsd2animinfo,
            &mut [],
        ]
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct wbplayerstruct_t {
    pub in_0: bool,
    pub skills: i32,
    pub sitems: i32,
    pub ssecret: i32,
    pub stime: i32,
    pub frags: [i32; 4],
    pub score: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wbstartstruct_t {
    pub epsd: i32,
    pub didsecret: bool,
    pub last: i32,
    pub next: i32,
    pub maxkills: i32,
    pub maxitems: i32,
    pub maxsecret: i32,
    pub maxfrags: i32,
    pub partime: i32,
    pub pnum: i32,
    pub plyr: [wbplayerstruct_t; 4],
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StateEnum {
    NoState = -1,
    StatCount = 0,
    ShowNextLoc = 1,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct anim_t {
    pub type_0: AnimEnum,
    pub period: i32,
    pub nanims: i32,
    pub loc: point_t,
    pub data1: i32,
    pub data2: i32,
    pub p: [i32; 3],
    pub nexttic: i32,
    pub lastdrawn: i32,
    pub ctr: i32,
    pub state: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct point_t {
    pub x: i32,
    pub y: i32,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AnimEnum {
    ANIM_ALWAYS = 0,
    ANIM_RANDOM = 1,
    ANIM_LEVEL = 2,
}
pub const NUMMAPS: i32 = 9;
pub const WI_TITLEY: i32 = 2;
pub const WI_SPACINGY: i32 = 33;
pub const SP_STATSX: i32 = 50;
pub const SP_STATSY: i32 = 50;
pub const SP_TIMEX: i32 = 16;
pub const SP_TIMEY: i32 = SCREENHEIGHT - 32_i32;
pub const NG_STATSY: i32 = 50;
pub const NG_SPACINGX: i32 = 64;
pub const DM_MATRIXX: i32 = 42;
pub const DM_MATRIXY: i32 = 68;
pub const DM_SPACINGX: i32 = 40;
pub const DM_TOTALSX: i32 = 269;
pub const DM_KILLERSX: i32 = 10;
pub const DM_KILLERSY: i32 = 100;
pub const DM_VICTIMSX: i32 = 5;
pub const DM_VICTIMSY: i32 = 50;
static lnodes: [[point_t; 9]; 4] = [
    [
        point_t {
            x: 185_i32,
            y: 164_i32,
        },
        point_t {
            x: 148_i32,
            y: 143_i32,
        },
        point_t {
            x: 69_i32,
            y: 122_i32,
        },
        point_t {
            x: 209_i32,
            y: 102_i32,
        },
        point_t {
            x: 116_i32,
            y: 89_i32,
        },
        point_t {
            x: 166_i32,
            y: 55_i32,
        },
        point_t {
            x: 71_i32,
            y: 56_i32,
        },
        point_t {
            x: 135_i32,
            y: 29_i32,
        },
        point_t {
            x: 71_i32,
            y: 24_i32,
        },
    ],
    [
        point_t {
            x: 254_i32,
            y: 25_i32,
        },
        point_t {
            x: 97_i32,
            y: 50_i32,
        },
        point_t {
            x: 188_i32,
            y: 64_i32,
        },
        point_t {
            x: 128_i32,
            y: 78_i32,
        },
        point_t {
            x: 214_i32,
            y: 92_i32,
        },
        point_t {
            x: 133_i32,
            y: 130_i32,
        },
        point_t {
            x: 208_i32,
            y: 136_i32,
        },
        point_t {
            x: 148_i32,
            y: 140_i32,
        },
        point_t {
            x: 235_i32,
            y: 158_i32,
        },
    ],
    [
        point_t {
            x: 156_i32,
            y: 168_i32,
        },
        point_t {
            x: 48_i32,
            y: 154_i32,
        },
        point_t {
            x: 174_i32,
            y: 95_i32,
        },
        point_t {
            x: 265_i32,
            y: 75_i32,
        },
        point_t {
            x: 130_i32,
            y: 48_i32,
        },
        point_t {
            x: 279_i32,
            y: 23_i32,
        },
        point_t {
            x: 198_i32,
            y: 48_i32,
        },
        point_t {
            x: 140_i32,
            y: 25_i32,
        },
        point_t {
            x: 281_i32,
            y: 136_i32,
        },
    ],
    [point_t { x: 0, y: 0 }; 9],
];
pub const SHOWNEXTLOCDELAY: i32 = 4;
pub fn WI_slamBackground(state: &mut GameState) {
    let patch = V_CachePatchNum(state, state.wi_stuff.background);
    let dest_screen = Screen::Video;
    V_DrawPatch(state, dest_screen, 0_i32, 0_i32, &patch);
}
pub fn WI_Responder() -> bool {
    false
}
pub fn WI_drawLF(state: &mut GameState) {
    let mut y: i32 = WI_TITLEY;
    if state.doomstat.gamemode as u32 != GameMode_t::commercial as i32 as u32
        || state.wbs().last < state.wi_stuff.NUMCMAPS
    {
        let index = state.wbs().last as usize;
        let last_lump = state.wi_stuff.lnames[index];
        let last_patch = V_CachePatchNum(state, last_lump);
        let dest_screen = Screen::Video;
        V_DrawPatch(
            state,
            dest_screen,
            (SCREENWIDTH - last_patch.width()) / 2_i32,
            y,
            &last_patch,
        );
        y += 5_i32 * last_patch.height() / 4_i32;
        let finished_patch = V_CachePatchNum(state, state.wi_stuff.finished);
        let dest_screen = Screen::Video;
        V_DrawPatch(
            state,
            dest_screen,
            (SCREENWIDTH - finished_patch.width()) / 2_i32,
            y,
            &finished_patch,
        );
    }
}
pub fn WI_drawEL(state: &mut GameState) {
    let mut y: i32 = WI_TITLEY;
    let entering_patch = V_CachePatchNum(state, state.wi_stuff.entering);
    let dest_screen = Screen::Video;
    V_DrawPatch(
        state,
        dest_screen,
        (SCREENWIDTH - entering_patch.width()) / 2_i32,
        y,
        &entering_patch,
    );
    let index = state.wbs().next as usize;
    let next_lump = state.wi_stuff.lnames[index];
    let next_patch = V_CachePatchNum(state, next_lump);
    y += 5_i32 * next_patch.height() / 4_i32;
    let dest_screen = Screen::Video;
    V_DrawPatch(
        state,
        dest_screen,
        (SCREENWIDTH - next_patch.width()) / 2_i32,
        y,
        &next_patch,
    );
}
pub fn WI_drawOnLnode(state: &mut GameState, n: i32, c: &[i32]) {
    let mut i: i32 = 0;
    let mut left: i32 = 0;
    let mut top: i32 = 0;
    let mut right: i32 = 0;
    let mut bottom: i32 = 0;
    let mut fits: bool = false;
    i = 0_i32;
    loop {
        let patch = V_CachePatchNum(state, c[i as usize]);
        left = lnodes[state.wbs().epsd as usize][n as usize].x - patch.leftoffset();
        top = lnodes[state.wbs().epsd as usize][n as usize].y - patch.topoffset();
        right = left + patch.width();
        bottom = top + patch.height();
        if left >= 0_i32 && right < SCREENWIDTH && top >= 0_i32 && bottom < SCREENHEIGHT {
            fits = true;
        } else {
            i += 1;
        }
        if !(!fits && i != 2_i32 && c[i as usize] != -1) {
            break;
        }
    }
    if fits && i < 2_i32 {
        let patch = V_CachePatchNum(state, c[i as usize]);
        let index = state.wbs().epsd as usize;
        let dest_screen = Screen::Video;
        V_DrawPatch(
            state,
            dest_screen,
            lnodes[index][n as usize].x,
            lnodes[index][n as usize].y,
            &patch,
        );
    } else {
        print!("Could not place patch on level {}", n + 1_i32);
    };
}
pub fn WI_initAnimatedBack(state: &mut GameState) {
    let mut i: i32 = 0;
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        return;
    }
    if state.wbs().epsd > 2_i32 {
        return;
    }
    i = 0_i32;
    while i < state.wi_stuff.NUMANIMS[state.wbs().epsd as usize] {
        let index = state.wbs().epsd as usize;
        let mut a = state.wi_stuff.anims()[index][i as usize];
        a.ctr = -1_i32;
        if a.type_0 == AnimEnum::ANIM_ALWAYS {
            a.nexttic =
                state.wi_stuff.bcnt + 1_i32 + M_Random(&mut state.m_random) % a.period;
        } else if a.type_0 == AnimEnum::ANIM_RANDOM {
            a.nexttic = state.wi_stuff.bcnt
                + 1_i32
                + a.data2
                + M_Random(&mut state.m_random) % a.data1;
        } else if a.type_0 == AnimEnum::ANIM_LEVEL {
            a.nexttic = state.wi_stuff.bcnt + 1_i32;
        }
        state.wi_stuff.anims()[index][i as usize] = a;
        i += 1;
    }
}
pub fn WI_updateAnimatedBack(state: &mut GameState) {
    let mut i: i32 = 0;
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        return;
    }
    if state.wbs().epsd > 2_i32 {
        return;
    }
    i = 0_i32;
    while i < state.wi_stuff.NUMANIMS[state.wbs().epsd as usize] {
        let index = state.wbs().epsd as usize;
        let mut a = state.wi_stuff.anims()[index][i as usize];
        if state.wi_stuff.bcnt == a.nexttic {
            match a.type_0 as u32 {
                0 => {
                    a.ctr += 1;
                    if a.ctr >= a.nanims {
                        a.ctr = 0_i32;
                    }
                    a.nexttic = state.wi_stuff.bcnt + a.period;
                }
                1 => {
                    a.ctr += 1;
                    if a.ctr == a.nanims {
                        a.ctr = -1_i32;
                        a.nexttic = state.wi_stuff.bcnt
                            + a.data2
                            + M_Random(&mut state.m_random) % a.data1;
                    } else {
                        a.nexttic = state.wi_stuff.bcnt + a.period;
                    }
                }
                2 => {
                    if !(state.wi_stuff.state == StateEnum::StatCount && i == 7_i32)
                        && state.wbs().next == a.data1
                    {
                        a.ctr += 1;
                        if a.ctr == a.nanims {
                            a.ctr -= 1;
                        }
                        a.nexttic = state.wi_stuff.bcnt + a.period;
                    }
                }
                _ => {}
            }
        }
        state.wi_stuff.anims()[index][i as usize] = a;
        i += 1;
    }
}
pub fn WI_drawAnimatedBack(state: &mut GameState) {
    let mut i: i32 = 0;
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        return;
    }
    if state.wbs().epsd > 2_i32 {
        return;
    }
    i = 0_i32;
    while i < state.wi_stuff.NUMANIMS[state.wbs().epsd as usize] {
        let index = state.wbs().epsd as usize;
        let mut a = state.wi_stuff.anims()[index][i as usize];
        if a.ctr >= 0_i32 {
            let patch = V_CachePatchNum(state, a.p[a.ctr as usize]);
            let dest_screen = Screen::Video;
            V_DrawPatch(state, dest_screen, a.loc.x, a.loc.y, &patch);
        }
        i += 1;
    }
}
pub fn WI_drawNum(
    state: &mut GameState,
    mut x: i32,
    mut y: i32,
    mut n: i32,
    mut digits: i32,
) -> i32 {
    let zero_patch = V_CachePatchNum(state, state.wi_stuff.num[0]);
    let mut fontwidth: i32 = zero_patch.width();
    let mut neg: i32 = 0;
    let mut temp: i32 = 0;
    if digits < 0_i32 {
        if n == 0 {
            digits = 1_i32;
        } else {
            digits = 0_i32;
            temp = n;
            while temp != 0 {
                temp /= 10_i32;
                digits += 1;
            }
        }
    }
    neg = (n < 0_i32) as i32;
    if neg != 0 {
        n = -n;
    }
    if n == 1994_i32 {
        return 0_i32;
    }
    loop {
        let fresh0 = digits;
        digits -= 1;
        if fresh0 == 0 {
            break;
        }
        x -= fontwidth;
        let digit_patch = V_CachePatchNum(state, state.wi_stuff.num[(n % 10_i32) as usize]);
        let dest_screen = Screen::Video;
        V_DrawPatch(state, dest_screen, x, y, &digit_patch);
        n /= 10_i32;
    }
    if neg != 0 {
        x -= 8_i32;
        let minus_patch = V_CachePatchNum(state, state.wi_stuff.wiminus);
        let dest_screen = Screen::Video;
        V_DrawPatch(state, dest_screen, x, y, &minus_patch);
    }
    x
}
pub fn WI_drawPercent(state: &mut GameState, mut x: i32, mut y: i32, mut p_0: i32) {
    if p_0 < 0_i32 {
        return;
    }
    let percent_patch = V_CachePatchNum(state, state.wi_stuff.percent);
    let dest_screen = Screen::Video;
    V_DrawPatch(state, dest_screen, x, y, &percent_patch);
    WI_drawNum(state, x, y, p_0, -1_i32);
}
pub fn WI_drawTime(state: &mut GameState, mut x: i32, mut y: i32, mut t: i32) {
    let mut div: i32 = 0;
    let mut n: i32 = 0;
    if t < 0_i32 {
        return;
    }
    if t <= 61_i32 * 59_i32 {
        div = 1_i32;
        loop {
            n = t / div % 60_i32;
            let colon_patch = V_CachePatchNum(state, state.wi_stuff.colon);
            x = WI_drawNum(state, x, y, n, 2_i32) - colon_patch.width();
            div *= 60_i32;
            if div == 60_i32 || t / div != 0 {
                let dest_screen = Screen::Video;
                V_DrawPatch(state, dest_screen, x, y, &colon_patch);
            }
            if t / div == 0 {
                break;
            }
        }
    } else {
        let sucks_patch = V_CachePatchNum(state, state.wi_stuff.sucks);
        let dest_screen = Screen::Video;
        V_DrawPatch(
            state,
            dest_screen,
            x - sucks_patch.width(),
            y,
            &sucks_patch,
        );
    };
}
pub fn WI_End(state: &mut GameState) {
    WI_loadUnloadData(state, WI_unloadCallback);
}
pub fn WI_initNoState(state: &mut GameState) {
    state.wi_stuff.state = StateEnum::NoState;
    state.wi_stuff.acceleratestage = 0_i32;
    state.wi_stuff.cnt = 10_i32;
}
pub fn WI_updateNoState(state: &mut GameState) {
    WI_updateAnimatedBack(state);
    state.wi_stuff.cnt -= 1;
    if state.wi_stuff.cnt == 0 {
        G_WorldDone(state);
    }
}
pub fn WI_initShowNextLoc(state: &mut GameState) {
    state.wi_stuff.state = StateEnum::ShowNextLoc;
    state.wi_stuff.acceleratestage = 0_i32;
    state.wi_stuff.cnt = SHOWNEXTLOCDELAY * TICRATE;
    WI_initAnimatedBack(state);
}
pub fn WI_updateShowNextLoc(state: &mut GameState) {
    WI_updateAnimatedBack(state);
    state.wi_stuff.cnt -= 1;
    if state.wi_stuff.cnt == 0 || state.wi_stuff.acceleratestage != 0 {
        WI_initNoState(state);
    } else {
        state.wi_stuff.snl_pointeron = (state.wi_stuff.cnt & 31_i32) < 20_i32;
    };
}
pub fn WI_drawShowNextLoc(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut last: i32 = 0;
    WI_slamBackground(state);
    WI_drawAnimatedBack(state);
    if state.doomstat.gamemode as u32 != GameMode_t::commercial as i32 as u32 {
        if state.wbs().epsd > 2_i32 {
            WI_drawEL(state);
            return;
        }
        last = if state.wbs().last == 8_i32 {
            state.wbs().next - 1_i32
        } else {
            state.wbs().last
        };
        i = 0_i32;
        while i <= last {
            let splat = state.wi_stuff.splat;
            WI_drawOnLnode(state, i, &splat);
            i += 1;
        }
        if state.wbs().didsecret {
            let splat = state.wi_stuff.splat;
            WI_drawOnLnode(state, 8_i32, &splat);
        }
        if state.wi_stuff.snl_pointeron {
            let next = state.wbs().next;
            let yah = state.wi_stuff.yah;
            WI_drawOnLnode(state, next, &yah);
        }
    }
    if state.doomstat.gamemode as u32 != GameMode_t::commercial as i32 as u32
        || state.wbs().next != 30_i32
    {
        WI_drawEL(state);
    }
}
pub fn WI_drawNoState(state: &mut GameState) {
    state.wi_stuff.snl_pointeron = true;
    WI_drawShowNextLoc(state);
}
pub fn WI_fragSum(state: &mut GameState, mut playernum: i32) -> i32 {
    let mut i: i32 = 0;
    let mut frags_0: i32 = 0_i32;
    i = 0_i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] && i != playernum {
            frags_0 += state.plyr_index(playernum).frags[i as usize];
        }
        i += 1;
    }
    frags_0 -= state.plyr_index(playernum).frags[playernum as usize];
    frags_0
}
pub fn WI_initDeathmatchStats(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    state.wi_stuff.state = StateEnum::StatCount;
    state.wi_stuff.acceleratestage = 0_i32;
    state.wi_stuff.dm_state = 1_i32;
    state.wi_stuff.cnt_pause = TICRATE;
    i = 0_i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            j = 0_i32;
            while j < MAXPLAYERS {
                if state.g_game.playeringame[j as usize] {
                    state.wi_stuff.dm_frags[i as usize][j as usize] = 0_i32;
                }
                j += 1;
            }
            state.wi_stuff.dm_totals[i as usize] = 0_i32;
        }
        i += 1;
    }
    WI_initAnimatedBack(state);
}
pub fn WI_updateDeathmatchStats(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut stillticking: bool = false;
    WI_updateAnimatedBack(state);
    if state.wi_stuff.acceleratestage != 0 && state.wi_stuff.dm_state != 4_i32 {
        state.wi_stuff.acceleratestage = 0_i32;
        i = 0_i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                j = 0_i32;
                while j < MAXPLAYERS {
                    if state.g_game.playeringame[j as usize] {
                        state.wi_stuff.dm_frags[i as usize][j as usize] =
                            state.plyr_index(i).frags[j as usize];
                    }
                    j += 1;
                }
                state.wi_stuff.dm_totals[i as usize] = WI_fragSum(state, i);
            }
            i += 1;
        }
        S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
        state.wi_stuff.dm_state = 4_i32;
    }
    if state.wi_stuff.dm_state == 2_i32 {
        if state.wi_stuff.bcnt & 3_i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        stillticking = false;
        i = 0_i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                j = 0_i32;
                while j < MAXPLAYERS {
                    if state.g_game.playeringame[j as usize]
                        && state.wi_stuff.dm_frags[i as usize][j as usize]
                            != state.plyr_index(i).frags[j as usize]
                    {
                        if state.plyr_index(i).frags[j as usize] < 0_i32 {
                            state.wi_stuff.dm_frags[i as usize][j as usize] -= 1;
                        } else {
                            state.wi_stuff.dm_frags[i as usize][j as usize] += 1;
                        }
                        if state.wi_stuff.dm_frags[i as usize][j as usize] > 99_i32 {
                            state.wi_stuff.dm_frags[i as usize][j as usize] = 99_i32;
                        }
                        if state.wi_stuff.dm_frags[i as usize][j as usize] < -99_i32 {
                            state.wi_stuff.dm_frags[i as usize][j as usize] = -99_i32;
                        }
                        stillticking = true;
                    }
                    j += 1;
                }
                state.wi_stuff.dm_totals[i as usize] = WI_fragSum(state, i);
                if state.wi_stuff.dm_totals[i as usize] > 99_i32 {
                    state.wi_stuff.dm_totals[i as usize] = 99_i32;
                }
                if state.wi_stuff.dm_totals[i as usize] < -99_i32 {
                    state.wi_stuff.dm_totals[i as usize] = -99_i32;
                }
            }
            i += 1;
        }
        if !stillticking {
            S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
            state.wi_stuff.dm_state += 1;
        }
    } else if state.wi_stuff.dm_state == 4_i32 {
        if state.wi_stuff.acceleratestage != 0 {
            S_StartSound(state, SoundOrigin::None, sfx_slop as i32);
            if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
                WI_initNoState(state);
            } else {
                WI_initShowNextLoc(state);
            }
        }
    } else if state.wi_stuff.dm_state & 1_i32 != 0 {
        state.wi_stuff.cnt_pause -= 1;
        if state.wi_stuff.cnt_pause == 0 {
            state.wi_stuff.dm_state += 1;
            state.wi_stuff.cnt_pause = TICRATE;
        }
    }
}
pub fn WI_drawDeathmatchStats(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut w: i32 = 0;
    WI_slamBackground(state);
    WI_drawAnimatedBack(state);
    WI_drawLF(state);
    let total_patch = V_CachePatchNum(state, state.wi_stuff.total);
    let dest_screen = Screen::Video;
    V_DrawPatch(
        state,
        dest_screen,
        DM_TOTALSX - total_patch.width() / 2_i32,
        DM_MATRIXY - WI_SPACINGY + 10_i32,
        &total_patch,
    );
    let killers_patch = V_CachePatchNum(state, state.wi_stuff.killers);
    let dest_screen = Screen::Video;
    V_DrawPatch(state, dest_screen, DM_KILLERSX, DM_KILLERSY, &killers_patch);
    let victims_patch = V_CachePatchNum(state, state.wi_stuff.victims);
    let dest_screen = Screen::Video;
    V_DrawPatch(state, dest_screen, DM_VICTIMSX, DM_VICTIMSY, &victims_patch);
    x = DM_MATRIXX + DM_SPACINGX;
    y = DM_MATRIXY;
    i = 0_i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            let p_patch = V_CachePatchNum(state, state.wi_stuff.p[i as usize]);
            let dest_screen = Screen::Video;
            V_DrawPatch(
                state,
                dest_screen,
                x - p_patch.width() / 2_i32,
                DM_MATRIXY - WI_SPACINGY,
                &p_patch,
            );
            let dest_screen = Screen::Video;
            V_DrawPatch(
                state,
                dest_screen,
                DM_MATRIXX - p_patch.width() / 2_i32,
                y,
                &p_patch,
            );
            if i == state.wi_stuff.me {
                let bstar_patch = V_CachePatchNum(state, state.wi_stuff.bstar);
                let dest_screen = Screen::Video;
                V_DrawPatch(
                    state,
                    dest_screen,
                    x - p_patch.width() / 2_i32,
                    DM_MATRIXY - WI_SPACINGY,
                    &bstar_patch,
                );
                let star_patch = V_CachePatchNum(state, state.wi_stuff.star);
                let dest_screen = Screen::Video;
                V_DrawPatch(
                    state,
                    dest_screen,
                    DM_MATRIXX - p_patch.width() / 2_i32,
                    y,
                    &star_patch,
                );
            }
        }
        x += DM_SPACINGX;
        y += WI_SPACINGY;
        i += 1;
    }
    y = DM_MATRIXY + 10_i32;
    let zero_patch = V_CachePatchNum(state, state.wi_stuff.num[0]);
    w = zero_patch.width();
    i = 0_i32;
    while i < MAXPLAYERS {
        x = DM_MATRIXX + DM_SPACINGX;
        if state.g_game.playeringame[i as usize] {
            j = 0_i32;
            while j < MAXPLAYERS {
                if state.g_game.playeringame[j as usize] {
                    let dm_frags = state.wi_stuff.dm_frags[i as usize][j as usize];
                    WI_drawNum(state, x + w, y, dm_frags, 2_i32);
                }
                x += DM_SPACINGX;
                j += 1;
            }
            let dm_totals = state.wi_stuff.dm_totals[i as usize];
            WI_drawNum(state, DM_TOTALSX + w, y, dm_totals, 2_i32);
        }
        y += WI_SPACINGY;
        i += 1;
    }
}
pub fn WI_initNetgameStats(state: &mut GameState) {
    let mut i: i32 = 0;
    state.wi_stuff.state = StateEnum::StatCount;
    state.wi_stuff.acceleratestage = 0_i32;
    state.wi_stuff.ng_state = 1_i32;
    state.wi_stuff.cnt_pause = TICRATE;
    i = 0_i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            state.wi_stuff.cnt_frags[i as usize] = 0_i32;
            state.wi_stuff.cnt_secret[i as usize] = state.wi_stuff.cnt_frags[i as usize];
            state.wi_stuff.cnt_items[i as usize] = state.wi_stuff.cnt_secret[i as usize];
            state.wi_stuff.cnt_kills[i as usize] = state.wi_stuff.cnt_items[i as usize];
            let fragsum = WI_fragSum(state, i);
            state.wi_stuff.dofrags += fragsum;
        }
        i += 1;
    }
    state.wi_stuff.dofrags = (state.wi_stuff.dofrags != 0) as i32;
    WI_initAnimatedBack(state);
}
pub fn WI_updateNetgameStats(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut fsum: i32 = 0;
    let mut stillticking: bool = false;
    WI_updateAnimatedBack(state);
    if state.wi_stuff.acceleratestage != 0 && state.wi_stuff.ng_state != 10_i32 {
        state.wi_stuff.acceleratestage = 0_i32;
        i = 0_i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                state.wi_stuff.cnt_kills[i as usize] =
                    state.plyr_index(i).skills * 100_i32 / state.wbs().maxkills;
                state.wi_stuff.cnt_items[i as usize] =
                    state.plyr_index(i).sitems * 100_i32 / state.wbs().maxitems;
                state.wi_stuff.cnt_secret[i as usize] =
                    state.plyr_index(i).ssecret * 100_i32 / state.wbs().maxsecret;
                if state.wi_stuff.dofrags != 0 {
                    state.wi_stuff.cnt_frags[i as usize] = WI_fragSum(state, i);
                }
            }
            i += 1;
        }
        S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
        state.wi_stuff.ng_state = 10_i32;
    }
    if state.wi_stuff.ng_state == 2_i32 {
        if state.wi_stuff.bcnt & 3_i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        stillticking = false;
        i = 0_i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                state.wi_stuff.cnt_kills[i as usize] += 2_i32;
                if state.wi_stuff.cnt_kills[i as usize]
                    >= state.plyr_index(i).skills * 100_i32 / state.wbs().maxkills
                {
                    state.wi_stuff.cnt_kills[i as usize] =
                        state.plyr_index(i).skills * 100_i32 / state.wbs().maxkills;
                } else {
                    stillticking = true;
                }
            }
            i += 1;
        }
        if !stillticking {
            S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
            state.wi_stuff.ng_state += 1;
        }
    } else if state.wi_stuff.ng_state == 4_i32 {
        if state.wi_stuff.bcnt & 3_i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        stillticking = false;
        i = 0_i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                state.wi_stuff.cnt_items[i as usize] += 2_i32;
                if state.wi_stuff.cnt_items[i as usize]
                    >= state.plyr_index(i).sitems * 100_i32 / state.wbs().maxitems
                {
                    state.wi_stuff.cnt_items[i as usize] =
                        state.plyr_index(i).sitems * 100_i32 / state.wbs().maxitems;
                } else {
                    stillticking = true;
                }
            }
            i += 1;
        }
        if !stillticking {
            S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
            state.wi_stuff.ng_state += 1;
        }
    } else if state.wi_stuff.ng_state == 6_i32 {
        if state.wi_stuff.bcnt & 3_i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        stillticking = false;
        i = 0_i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                state.wi_stuff.cnt_secret[i as usize] += 2_i32;
                if state.wi_stuff.cnt_secret[i as usize]
                    >= state.plyr_index(i).ssecret * 100_i32 / state.wbs().maxsecret
                {
                    state.wi_stuff.cnt_secret[i as usize] =
                        state.plyr_index(i).ssecret * 100_i32 / state.wbs().maxsecret;
                } else {
                    stillticking = true;
                }
            }
            i += 1;
        }
        if !stillticking {
            S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
            state.wi_stuff.ng_state += 1_i32 + 2_i32 * (state.wi_stuff.dofrags == 0) as i32;
        }
    } else if state.wi_stuff.ng_state == 8_i32 {
        if state.wi_stuff.bcnt & 3_i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        stillticking = false;
        i = 0_i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                state.wi_stuff.cnt_frags[i as usize] += 1_i32;
                fsum = WI_fragSum(state, i);
                if state.wi_stuff.cnt_frags[i as usize] >= fsum {
                    state.wi_stuff.cnt_frags[i as usize] = fsum;
                } else {
                    stillticking = true;
                }
            }
            i += 1;
        }
        if !stillticking {
            S_StartSound(state, SoundOrigin::None, sfx_pldeth as i32);
            state.wi_stuff.ng_state += 1;
        }
    } else if state.wi_stuff.ng_state == 10_i32 {
        if state.wi_stuff.acceleratestage != 0 {
            S_StartSound(state, SoundOrigin::None, sfx_sgcock as i32);
            if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
                WI_initNoState(state);
            } else {
                WI_initShowNextLoc(state);
            }
        }
    } else if state.wi_stuff.ng_state & 1_i32 != 0 {
        state.wi_stuff.cnt_pause -= 1;
        if state.wi_stuff.cnt_pause == 0 {
            state.wi_stuff.ng_state += 1;
            state.wi_stuff.cnt_pause = TICRATE;
        }
    }
}
pub fn WI_drawNetgameStats(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let percent_patch = V_CachePatchNum(state, state.wi_stuff.percent);
    let mut pwidth: i32 = percent_patch.width();
    WI_slamBackground(state);
    WI_drawAnimatedBack(state);
    WI_drawLF(state);
    let star_patch = V_CachePatchNum(state, state.wi_stuff.star);
    let star_width = star_patch.width();
    let kills_patch = V_CachePatchNum(state, state.wi_stuff.kills);
    let dest_screen = Screen::Video;
    V_DrawPatch(
        state,
        dest_screen,
        32_i32 + star_width / 2_i32 + 32_i32 * (state.wi_stuff.dofrags == 0) as i32 + NG_SPACINGX
            - kills_patch.width(),
        NG_STATSY,
        &kills_patch,
    );
    let items_patch = V_CachePatchNum(state, state.wi_stuff.items);
    let dest_screen = Screen::Video;
    V_DrawPatch(
        state,
        dest_screen,
        32_i32
            + star_width / 2_i32
            + 32_i32 * (state.wi_stuff.dofrags == 0) as i32
            + 2_i32 * NG_SPACINGX
            - items_patch.width(),
        NG_STATSY,
        &items_patch,
    );
    let secret_patch = V_CachePatchNum(state, state.wi_stuff.secret);
    let dest_screen = Screen::Video;
    V_DrawPatch(
        state,
        dest_screen,
        32_i32
            + star_width / 2_i32
            + 32_i32 * (state.wi_stuff.dofrags == 0) as i32
            + 3_i32 * NG_SPACINGX
            - secret_patch.width(),
        NG_STATSY,
        &secret_patch,
    );
    if state.wi_stuff.dofrags != 0 {
        let frags_patch = V_CachePatchNum(state, state.wi_stuff.frags);
        let dest_screen = Screen::Video;
        V_DrawPatch(
            state,
            dest_screen,
            32_i32
                + star_width / 2_i32
                + 32_i32 * (state.wi_stuff.dofrags == 0) as i32
                + 4_i32 * NG_SPACINGX
                - frags_patch.width(),
            NG_STATSY,
            &frags_patch,
        );
    }
    y = NG_STATSY + kills_patch.height();
    i = 0_i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            x = 32_i32 + star_width / 2_i32 + 32_i32 * (state.wi_stuff.dofrags == 0) as i32;
            let p_patch = V_CachePatchNum(state, state.wi_stuff.p[i as usize]);
            let dest_screen = Screen::Video;
            V_DrawPatch(state, dest_screen, x - p_patch.width(), y, &p_patch);
            if i == state.wi_stuff.me {
                let dest_screen = Screen::Video;
                V_DrawPatch(
                    state,
                    dest_screen,
                    x - p_patch.width(),
                    y,
                    &star_patch,
                );
            }
            x += NG_SPACINGX;
            let cnt_kills = state.wi_stuff.cnt_kills[i as usize];
            WI_drawPercent(state, x - pwidth, y + 10_i32, cnt_kills);
            x += NG_SPACINGX;
            let cnt_items = state.wi_stuff.cnt_items[i as usize];
            WI_drawPercent(state, x - pwidth, y + 10_i32, cnt_items);
            x += NG_SPACINGX;
            let cnt_secret = state.wi_stuff.cnt_secret[i as usize];
            WI_drawPercent(state, x - pwidth, y + 10_i32, cnt_secret);
            x += NG_SPACINGX;
            if state.wi_stuff.dofrags != 0 {
                let cnt_frags = state.wi_stuff.cnt_frags[i as usize];
                WI_drawNum(state, x, y + 10_i32, cnt_frags, -1_i32);
            }
            y += WI_SPACINGY;
        }
        i += 1;
    }
}
pub fn WI_initStats(state: &mut GameState) {
    state.wi_stuff.state = StateEnum::StatCount;
    state.wi_stuff.acceleratestage = 0_i32;
    state.wi_stuff.sp_state = 1_i32;
    state.wi_stuff.cnt_secret[0] = -1_i32;
    state.wi_stuff.cnt_items[0] = state.wi_stuff.cnt_secret[0];
    state.wi_stuff.cnt_kills[0] = state.wi_stuff.cnt_items[0];
    state.wi_stuff.cnt_par = -1_i32;
    state.wi_stuff.cnt_time = state.wi_stuff.cnt_par;
    state.wi_stuff.cnt_pause = TICRATE;
    WI_initAnimatedBack(state);
}
pub fn WI_updateStats(state: &mut GameState) {
    WI_updateAnimatedBack(state);
    if state.wi_stuff.acceleratestage != 0 && state.wi_stuff.sp_state != 10_i32 {
        state.wi_stuff.acceleratestage = 0_i32;
        state.wi_stuff.cnt_kills[0] =
            state.plyr_index(state.wi_stuff.me).skills * 100_i32 / state.wbs().maxkills;
        state.wi_stuff.cnt_items[0] =
            state.plyr_index(state.wi_stuff.me).sitems * 100_i32 / state.wbs().maxitems;
        state.wi_stuff.cnt_secret[0] =
            state.plyr_index(state.wi_stuff.me).ssecret * 100_i32 / state.wbs().maxsecret;
        state.wi_stuff.cnt_time = state.plyr_index(state.wi_stuff.me).stime / TICRATE;
        state.wi_stuff.cnt_par = state.wbs().partime / TICRATE;
        S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
        state.wi_stuff.sp_state = 10_i32;
    }
    if state.wi_stuff.sp_state == 2_i32 {
        state.wi_stuff.cnt_kills[0] += 2_i32;
        if state.wi_stuff.bcnt & 3_i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        if state.wi_stuff.cnt_kills[0]
            >= state.plyr_index(state.wi_stuff.me).skills * 100_i32 / state.wbs().maxkills
        {
            state.wi_stuff.cnt_kills[0] =
                state.plyr_index(state.wi_stuff.me).skills * 100_i32 / state.wbs().maxkills;
            S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
            state.wi_stuff.sp_state += 1;
        }
    } else if state.wi_stuff.sp_state == 4_i32 {
        state.wi_stuff.cnt_items[0] += 2_i32;
        if state.wi_stuff.bcnt & 3_i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        if state.wi_stuff.cnt_items[0]
            >= state.plyr_index(state.wi_stuff.me).sitems * 100_i32 / state.wbs().maxitems
        {
            state.wi_stuff.cnt_items[0] =
                state.plyr_index(state.wi_stuff.me).sitems * 100_i32 / state.wbs().maxitems;
            S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
            state.wi_stuff.sp_state += 1;
        }
    } else if state.wi_stuff.sp_state == 6_i32 {
        state.wi_stuff.cnt_secret[0] += 2_i32;
        if state.wi_stuff.bcnt & 3_i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        if state.wi_stuff.cnt_secret[0]
            >= state.plyr_index(state.wi_stuff.me).ssecret * 100_i32 / state.wbs().maxsecret
        {
            state.wi_stuff.cnt_secret[0] =
                state.plyr_index(state.wi_stuff.me).ssecret * 100_i32 / state.wbs().maxsecret;
            S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
            state.wi_stuff.sp_state += 1;
        }
    } else if state.wi_stuff.sp_state == 8_i32 {
        if state.wi_stuff.bcnt & 3_i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        state.wi_stuff.cnt_time += 3_i32;
        if state.wi_stuff.cnt_time >= state.plyr_index(state.wi_stuff.me).stime / TICRATE {
            state.wi_stuff.cnt_time = state.plyr_index(state.wi_stuff.me).stime / TICRATE;
        }
        state.wi_stuff.cnt_par += 3_i32;
        if state.wi_stuff.cnt_par >= state.wbs().partime / TICRATE {
            state.wi_stuff.cnt_par = state.wbs().partime / TICRATE;
            if state.wi_stuff.cnt_time >= state.plyr_index(state.wi_stuff.me).stime / TICRATE {
                S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
                state.wi_stuff.sp_state += 1;
            }
        }
    } else if state.wi_stuff.sp_state == 10_i32 {
        if state.wi_stuff.acceleratestage != 0 {
            S_StartSound(state, SoundOrigin::None, sfx_sgcock as i32);
            if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
                WI_initNoState(state);
            } else {
                WI_initShowNextLoc(state);
            }
        }
    } else if state.wi_stuff.sp_state & 1_i32 != 0 {
        state.wi_stuff.cnt_pause -= 1;
        if state.wi_stuff.cnt_pause == 0 {
            state.wi_stuff.sp_state += 1;
            state.wi_stuff.cnt_pause = TICRATE;
        }
    }
}
pub fn WI_drawStats(state: &mut GameState) {
    let mut lh: i32 = 0;
    let zero_patch = V_CachePatchNum(state, state.wi_stuff.num[0]);
    lh = 3_i32 * zero_patch.height() / 2_i32;
    WI_slamBackground(state);
    WI_drawAnimatedBack(state);
    WI_drawLF(state);
    let kills_patch = V_CachePatchNum(state, state.wi_stuff.kills);
    let dest_screen = Screen::Video;
    V_DrawPatch(state, dest_screen, SP_STATSX, SP_STATSY, &kills_patch);
    let cnt_kills = state.wi_stuff.cnt_kills[0];
    WI_drawPercent(state, SCREENWIDTH - SP_STATSX, SP_STATSY, cnt_kills);
    let items_patch = V_CachePatchNum(state, state.wi_stuff.items);
    let dest_screen = Screen::Video;
    V_DrawPatch(state, dest_screen, SP_STATSX, SP_STATSY + lh, &items_patch);
    let cnt_items = state.wi_stuff.cnt_items[0];
    WI_drawPercent(state, SCREENWIDTH - SP_STATSX, SP_STATSY + lh, cnt_items);
    let sp_secret_patch = V_CachePatchNum(state, state.wi_stuff.sp_secret);
    let dest_screen = Screen::Video;
    V_DrawPatch(
        state,
        dest_screen,
        SP_STATSX,
        SP_STATSY + 2_i32 * lh,
        &sp_secret_patch,
    );
    let cnt_secret = state.wi_stuff.cnt_secret[0];
    WI_drawPercent(
        state,
        SCREENWIDTH - SP_STATSX,
        SP_STATSY + 2_i32 * lh,
        cnt_secret,
    );
    let timepatch_patch = V_CachePatchNum(state, state.wi_stuff.timepatch);
    let dest_screen = Screen::Video;
    V_DrawPatch(state, dest_screen, SP_TIMEX, SP_TIMEY, &timepatch_patch);
    let cnt_time = state.wi_stuff.cnt_time;
    WI_drawTime(state, SCREENWIDTH / 2_i32 - SP_TIMEX, SP_TIMEY, cnt_time);
    if state.wbs().epsd < 3_i32 {
        let par_patch = V_CachePatchNum(state, state.wi_stuff.par);
        let dest_screen = Screen::Video;
        V_DrawPatch(
            state,
            dest_screen,
            SCREENWIDTH / 2_i32 + SP_TIMEX,
            SP_TIMEY,
            &par_patch,
        );
        let cnt_par = state.wi_stuff.cnt_par;
        WI_drawTime(state, SCREENWIDTH - SP_TIMEX, SP_TIMEY, cnt_par);
    }
}
pub fn WI_checkForAccelerate(state: &mut GameState) {
    let mut i: i32 = 0;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            let player = &state.g_game.players[i as usize];
            if player.cmd.buttons as i32 & BT_ATTACK as i32 != 0 {
                if player.attackdown == 0 {
                    state.wi_stuff.acceleratestage = 1_i32;
                }
                state.g_game.players[i as usize].attackdown = true_0;
            } else {
                state.g_game.players[i as usize].attackdown = false_0;
            }
            let player = &state.g_game.players[i as usize];
            if player.cmd.buttons as i32 & BT_USE as i32 != 0 {
                if player.usedown == 0 {
                    state.wi_stuff.acceleratestage = 1_i32;
                }
                state.g_game.players[i as usize].usedown = true_0;
            } else {
                state.g_game.players[i as usize].usedown = false_0;
            }
        }
        i += 1;
    }
}
pub fn WI_Ticker(state: &mut GameState) {
    state.wi_stuff.bcnt += 1;
    if state.wi_stuff.bcnt == 1_i32 {
        if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
            S_ChangeMusic(state, mus_dm2int as i32, true_0);
        } else {
            S_ChangeMusic(state, mus_inter as i32, true_0);
        }
    }
    WI_checkForAccelerate(state);
    match state.wi_stuff.state {
        StateEnum::StatCount => {
            if state.g_game.deathmatch != 0 {
                WI_updateDeathmatchStats(state);
            } else if state.g_game.netgame {
                WI_updateNetgameStats(state);
            } else {
                WI_updateStats(state);
            }
        }
        StateEnum::ShowNextLoc => {
            WI_updateShowNextLoc(state);
        }
        StateEnum::NoState => {
            WI_updateNoState(state);
        }
    };
}
fn WI_loadUnloadData(state: &mut GameState, callback: load_callback_t) {
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        for i in 0..state.wi_stuff.NUMCMAPS as usize {
            state.wi_stuff.lnames[i] = callback(state, &format!("CWILV{:02}", i));
        }
    } else {
        for i in 0..NUMMAPS as usize {
            let name = format!("WILV{}{}", state.wbs().epsd, i);
            state.wi_stuff.lnames[i] = callback(state, &name);
        }
        state.wi_stuff.yah[0] = callback(state, "WIURH0");
        state.wi_stuff.yah[1] = callback(state, "WIURH1");
        state.wi_stuff.splat[0] = callback(state, "WISPLAT");
        if state.wbs().epsd < 3_i32 {
            let epsd = state.wbs().epsd as usize;
            for j in 0..state.wi_stuff.NUMANIMS[epsd] as usize {
                let nanims = state.wi_stuff.anims()[epsd][j].nanims as usize;
                for i in 0..nanims {
                    let lump = if epsd != 1 || j != 8 {
                        let name = format!("WIA{}{:02}{:02}", epsd, j, i);
                        callback(state, &name)
                    } else {
                        state.wi_stuff.anims()[1][4].p[i]
                    };
                    state.wi_stuff.anims()[epsd][j].p[i] = lump;
                }
            }
        }
    }
    state.wi_stuff.wiminus = callback(state, "WIMINUS");
    for i in 0..10_usize {
        state.wi_stuff.num[i] = callback(state, &format!("WINUM{}", i));
    }
    state.wi_stuff.percent = callback(state, "WIPCNT");
    state.wi_stuff.finished = callback(state, "WIF");
    state.wi_stuff.entering = callback(state, "WIENTER");
    state.wi_stuff.kills = callback(state, "WIOSTK");
    state.wi_stuff.secret = callback(state, "WIOSTS");
    state.wi_stuff.sp_secret = callback(state, "WISCRT2");
    let items_name = if W_CheckNumForName(&mut state.w_wad, "WIOBJ") >= 0_i32
        && state.g_game.netgame
        && state.g_game.deathmatch == 0
    {
        "WIOBJ"
    } else {
        "WIOSTI"
    };
    state.wi_stuff.items = callback(state, items_name);
    state.wi_stuff.frags = callback(state, "WIFRGS");
    state.wi_stuff.colon = callback(state, "WICOLON");
    state.wi_stuff.timepatch = callback(state, "WITIME");
    state.wi_stuff.sucks = callback(state, "WISUCKS");
    state.wi_stuff.par = callback(state, "WIPAR");
    state.wi_stuff.killers = callback(state, "WIKILRS");
    state.wi_stuff.victims = callback(state, "WIVCTMS");
    state.wi_stuff.total = callback(state, "WIMSTT");
    for i in 0..MAXPLAYERS as usize {
        state.wi_stuff.p[i] = callback(state, &format!("STPB{}", i));
        state.wi_stuff.bp[i] = callback(state, &format!("WIBP{}", i + 1));
    }
    let name = if state.doomstat.gamemode == GameMode_t::commercial {
        "INTERPIC".to_string()
    } else if state.doomstat.gamemode == GameMode_t::retail && state.wbs().epsd == 3_i32 {
        "INTERPIC".to_string()
    } else {
        format!("WIMAP{}", state.wbs().epsd)
    };
    state.wi_stuff.background = callback(state, &name);
}
fn WI_loadCallback(state: &mut GameState, name: &str) -> i32 {
    let lumpnum = W_GetNumForName(&mut state.w_wad, name);
    W_CacheLumpNum(state, lumpnum);
    lumpnum
}
pub fn WI_loadData(state: &mut GameState) {
    if state.doomstat.gamemode == GameMode_t::commercial {
        state.wi_stuff.NUMCMAPS = 32;
        state.wi_stuff.lnames = vec![-1; state.wi_stuff.NUMCMAPS as usize];
    } else {
        state.wi_stuff.lnames = vec![-1; NUMMAPS as usize];
    }
    WI_loadUnloadData(state, WI_loadCallback);
    let star_lump = W_GetNumForName(&mut state.w_wad, "STFST01");
    W_CacheLumpNum(state, star_lump);
    state.wi_stuff.star = star_lump;
    let bstar_lump = W_GetNumForName(&mut state.w_wad, "STFDEAD0");
    W_CacheLumpNum(state, bstar_lump);
    state.wi_stuff.bstar = bstar_lump;
}
fn WI_unloadCallback(state: &mut GameState, name: &str) -> i32 {
    W_ReleaseLumpName(&mut state.w_wad, name);
    -1
}
pub fn WI_Drawer(state: &mut GameState) {
    match state.wi_stuff.state {
        StateEnum::StatCount => {
            if state.g_game.deathmatch != 0 {
                WI_drawDeathmatchStats(state);
            } else if state.g_game.netgame {
                WI_drawNetgameStats(state);
            } else {
                WI_drawStats(state);
            }
        }
        StateEnum::ShowNextLoc => {
            WI_drawShowNextLoc(state);
        }
        StateEnum::NoState => {
            WI_drawNoState(state);
        }
    };
}
pub fn WI_initVariables(state: &mut GameState) {
    state.wi_stuff.acceleratestage = 0_i32;
    state.wi_stuff.bcnt = 0_i32;
    state.wi_stuff.cnt = state.wi_stuff.bcnt;
    state.wi_stuff.firstrefresh = 1_i32;
    state.wi_stuff.me = state.wbs().pnum;
    if state.wbs().maxkills == 0 {
        state.wbs().maxkills = 1;
    }
    if state.wbs().maxitems == 0 {
        state.wbs().maxitems = 1_i32;
    }
    if state.wbs().maxsecret == 0 {
        state.wbs().maxsecret = 1_i32;
    }
    if state.doomstat.gamemode != GameMode_t::retail && state.wbs().epsd > 2_i32 {
        state.wbs().epsd -= 3_i32;
    }
}
pub fn WI_Start(state: &mut GameState) {
    WI_initVariables(state);
    WI_loadData(state);
    if state.g_game.deathmatch != 0 {
        WI_initDeathmatchStats(state);
    } else if state.g_game.netgame {
        WI_initNetgameStats(state);
    } else {
        WI_initStats(state);
    };
}
pub fn fixup_numanims(state: &mut GameState) {
    state.wi_stuff.NUMANIMS = [
        ::core::mem::size_of::<[anim_t; 10]>().wrapping_div(::core::mem::size_of::<anim_t>())
            as i32,
        ::core::mem::size_of::<[anim_t; 9]>().wrapping_div(::core::mem::size_of::<anim_t>()) as i32,
        ::core::mem::size_of::<[anim_t; 6]>().wrapping_div(::core::mem::size_of::<anim_t>()) as i32,
        0,
    ];
}
