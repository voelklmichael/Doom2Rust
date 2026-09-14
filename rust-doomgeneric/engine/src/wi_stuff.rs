use crate::d_mode::GameMode_t;
use crate::d_player::player_t;
use crate::d_ticcmd::{BT_ATTACK, BT_USE};
use crate::doomdef::false_0;
use crate::doomdef::true_0;
use crate::doomdef::MAXPLAYERS;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::doomdef::TICRATE;
use crate::g_game::G_WorldDone;
use crate::game_state::GameState;
use crate::hu_lib::patch_t;
use crate::m_random::M_Random;
use crate::s_sound::S_ChangeMusic;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::{mus_dm2int, mus_inter};
use crate::sounds::{sfx_barexp, sfx_pistol, sfx_pldeth, sfx_sgcock, sfx_slop};
use crate::st_stuff::load_callback_t;
use crate::v_video::V_DrawPatch;
use crate::w_wad::{W_CacheLumpName, W_CheckNumForName, W_ReleaseLumpName};

pub struct WiStuffState {
    pub anims: [*mut anim_t; 4],
    pub epsd0animinfo: [anim_t; 10],
    pub epsd1animinfo: [anim_t; 9],
    pub epsd2animinfo: [anim_t; 6],
    pub NUMANIMS: [i32; 4],
    pub acceleratestage: i32,
    pub me: i32,
    pub state: StateEnum,
    pub wbs: *mut wbstartstruct_t,
    pub plrs: *mut wbplayerstruct_t,
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
    pub yah: [*mut patch_t; 3],
    pub splat: [*mut patch_t; 2],
    pub percent: *mut patch_t,
    pub colon: *mut patch_t,
    pub num: [*mut patch_t; 10],
    pub wiminus: *mut patch_t,
    pub finished: *mut patch_t,
    pub entering: *mut patch_t,
    pub sp_secret: *mut patch_t,
    pub kills: *mut patch_t,
    pub secret: *mut patch_t,
    pub items: *mut patch_t,
    pub frags: *mut patch_t,
    pub timepatch: *mut patch_t,
    pub par: *mut patch_t,
    pub sucks: *mut patch_t,
    pub killers: *mut patch_t,
    pub victims: *mut patch_t,
    pub total: *mut patch_t,
    pub star: *mut patch_t,
    pub bstar: *mut patch_t,
    pub p: [*mut patch_t; 4],
    pub bp: [*mut patch_t; 4],
    pub lnames: Vec<*mut patch_t>,
    pub background: *mut patch_t,
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
            anims: [
                ::core::ptr::null_mut::<anim_t>(),
                ::core::ptr::null_mut::<anim_t>(),
                ::core::ptr::null_mut::<anim_t>(),
                ::core::ptr::null_mut::<anim_t>(),
            ],
            epsd0animinfo: [
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 224 as i32,
                        y: 104 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 184 as i32,
                        y: 160 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 112 as i32,
                        y: 136 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 72 as i32,
                        y: 112 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 88 as i32,
                        y: 96 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 64 as i32,
                        y: 48 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 192 as i32,
                        y: 40 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 136 as i32,
                        y: 16 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 80 as i32,
                        y: 16 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 64 as i32,
                        y: 24 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
            ],
            epsd1animinfo: [
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35 as i32 / 3 as i32,
                    nanims: 1 as i32,
                    loc: point_t {
                        x: 128 as i32,
                        y: 136 as i32,
                    },
                    data1: 1 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35 as i32 / 3 as i32,
                    nanims: 1 as i32,
                    loc: point_t {
                        x: 128 as i32,
                        y: 136 as i32,
                    },
                    data1: 2 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35 as i32 / 3 as i32,
                    nanims: 1 as i32,
                    loc: point_t {
                        x: 128 as i32,
                        y: 136 as i32,
                    },
                    data1: 3 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35 as i32 / 3 as i32,
                    nanims: 1 as i32,
                    loc: point_t {
                        x: 128 as i32,
                        y: 136 as i32,
                    },
                    data1: 4 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35 as i32 / 3 as i32,
                    nanims: 1 as i32,
                    loc: point_t {
                        x: 128 as i32,
                        y: 136 as i32,
                    },
                    data1: 5 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35 as i32 / 3 as i32,
                    nanims: 1 as i32,
                    loc: point_t {
                        x: 128 as i32,
                        y: 136 as i32,
                    },
                    data1: 6 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35 as i32 / 3 as i32,
                    nanims: 1 as i32,
                    loc: point_t {
                        x: 128 as i32,
                        y: 136 as i32,
                    },
                    data1: 7 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 192 as i32,
                        y: 144 as i32,
                    },
                    data1: 8 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_LEVEL,
                    period: 35 as i32 / 3 as i32,
                    nanims: 1 as i32,
                    loc: point_t {
                        x: 128 as i32,
                        y: 136 as i32,
                    },
                    data1: 8 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
            ],
            epsd2animinfo: [
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 104 as i32,
                        y: 168 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 40 as i32,
                        y: 136 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 160 as i32,
                        y: 96 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 104 as i32,
                        y: 80 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 3 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 120 as i32,
                        y: 32 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
                anim_t {
                    type_0: AnimEnum::ANIM_ALWAYS,
                    period: 35 as i32 / 4 as i32,
                    nanims: 3 as i32,
                    loc: point_t {
                        x: 40 as i32,
                        y: 0 as i32,
                    },
                    data1: 0 as i32,
                    data2: 0 as i32,
                    p: [
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                        ::core::ptr::null::<patch_t>() as *mut patch_t,
                    ],
                    nexttic: 0 as i32,
                    lastdrawn: 0 as i32,
                    ctr: 0 as i32,
                    state: 0 as i32,
                },
            ],
            NUMANIMS: [0; 4],
            acceleratestage: 0,
            me: 0,
            state: StateEnum::StatCount,
            wbs: ::core::ptr::null::<wbstartstruct_t>() as *mut wbstartstruct_t,
            plrs: ::core::ptr::null::<wbplayerstruct_t>() as *mut wbplayerstruct_t,
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
            yah: [
                ::core::ptr::null::<patch_t>() as *mut patch_t,
                ::core::ptr::null::<patch_t>() as *mut patch_t,
                ::core::ptr::null::<patch_t>() as *mut patch_t,
            ],
            splat: [
                ::core::ptr::null::<patch_t>() as *mut patch_t,
                ::core::ptr::null::<patch_t>() as *mut patch_t,
            ],
            percent: ::core::ptr::null::<patch_t>() as *mut patch_t,
            colon: ::core::ptr::null::<patch_t>() as *mut patch_t,
            num: [::core::ptr::null::<patch_t>() as *mut patch_t; 10],
            wiminus: ::core::ptr::null::<patch_t>() as *mut patch_t,
            finished: ::core::ptr::null::<patch_t>() as *mut patch_t,
            entering: ::core::ptr::null::<patch_t>() as *mut patch_t,
            sp_secret: ::core::ptr::null::<patch_t>() as *mut patch_t,
            kills: ::core::ptr::null::<patch_t>() as *mut patch_t,
            secret: ::core::ptr::null::<patch_t>() as *mut patch_t,
            items: ::core::ptr::null::<patch_t>() as *mut patch_t,
            frags: ::core::ptr::null::<patch_t>() as *mut patch_t,
            timepatch: ::core::ptr::null::<patch_t>() as *mut patch_t,
            par: ::core::ptr::null::<patch_t>() as *mut patch_t,
            sucks: ::core::ptr::null::<patch_t>() as *mut patch_t,
            killers: ::core::ptr::null::<patch_t>() as *mut patch_t,
            victims: ::core::ptr::null::<patch_t>() as *mut patch_t,
            total: ::core::ptr::null::<patch_t>() as *mut patch_t,
            star: ::core::ptr::null::<patch_t>() as *mut patch_t,
            bstar: ::core::ptr::null::<patch_t>() as *mut patch_t,
            p: [::core::ptr::null::<patch_t>() as *mut patch_t; 4],
            bp: [::core::ptr::null::<patch_t>() as *mut patch_t; 4],
            lnames: Vec::new(),
            background: ::core::ptr::null::<patch_t>() as *mut patch_t,
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

    pub fn fixup_anims(&mut self) {
        self.anims = [
            &raw mut self.epsd0animinfo as *mut anim_t,
            &raw mut self.epsd1animinfo as *mut anim_t,
            &raw mut self.epsd2animinfo as *mut anim_t,
            ::core::ptr::null_mut::<anim_t>(),
        ];
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
    pub p: [*mut patch_t; 3],
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
pub const SP_TIMEY: i32 = SCREENHEIGHT - 32 as i32;
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
            x: 185 as i32,
            y: 164 as i32,
        },
        point_t {
            x: 148 as i32,
            y: 143 as i32,
        },
        point_t {
            x: 69 as i32,
            y: 122 as i32,
        },
        point_t {
            x: 209 as i32,
            y: 102 as i32,
        },
        point_t {
            x: 116 as i32,
            y: 89 as i32,
        },
        point_t {
            x: 166 as i32,
            y: 55 as i32,
        },
        point_t {
            x: 71 as i32,
            y: 56 as i32,
        },
        point_t {
            x: 135 as i32,
            y: 29 as i32,
        },
        point_t {
            x: 71 as i32,
            y: 24 as i32,
        },
    ],
    [
        point_t {
            x: 254 as i32,
            y: 25 as i32,
        },
        point_t {
            x: 97 as i32,
            y: 50 as i32,
        },
        point_t {
            x: 188 as i32,
            y: 64 as i32,
        },
        point_t {
            x: 128 as i32,
            y: 78 as i32,
        },
        point_t {
            x: 214 as i32,
            y: 92 as i32,
        },
        point_t {
            x: 133 as i32,
            y: 130 as i32,
        },
        point_t {
            x: 208 as i32,
            y: 136 as i32,
        },
        point_t {
            x: 148 as i32,
            y: 140 as i32,
        },
        point_t {
            x: 235 as i32,
            y: 158 as i32,
        },
    ],
    [
        point_t {
            x: 156 as i32,
            y: 168 as i32,
        },
        point_t {
            x: 48 as i32,
            y: 154 as i32,
        },
        point_t {
            x: 174 as i32,
            y: 95 as i32,
        },
        point_t {
            x: 265 as i32,
            y: 75 as i32,
        },
        point_t {
            x: 130 as i32,
            y: 48 as i32,
        },
        point_t {
            x: 279 as i32,
            y: 23 as i32,
        },
        point_t {
            x: 198 as i32,
            y: 48 as i32,
        },
        point_t {
            x: 140 as i32,
            y: 25 as i32,
        },
        point_t {
            x: 281 as i32,
            y: 136 as i32,
        },
    ],
    [point_t { x: 0, y: 0 }; 9],
];
pub const SHOWNEXTLOCDELAY: i32 = 4;
pub fn WI_slamBackground(state: &mut GameState) {
    unsafe { V_DrawPatch(state, 0 as i32, 0 as i32, state.wi_stuff.background) };
}
pub fn WI_Responder() -> bool {
    return false;
}
pub unsafe fn WI_drawLF(state: &mut GameState) {
    let mut y: i32 = WI_TITLEY;
    if state.doomstat.gamemode as u32 != GameMode_t::commercial as i32 as u32
        || (*state.wi_stuff.wbs).last < state.wi_stuff.NUMCMAPS
    {
        let last_patch = state.wi_stuff.lnames[(*state.wi_stuff.wbs).last as usize];
        V_DrawPatch(
            state,
            (SCREENWIDTH - (*last_patch).width as i32) / 2 as i32,
            y,
            last_patch,
        );
        y += 5 as i32 * (*last_patch).height as i32 / 4 as i32;
        V_DrawPatch(
            state,
            (SCREENWIDTH - (*state.wi_stuff.finished).width as i32) / 2 as i32,
            y,
            state.wi_stuff.finished,
        );
    } else if !((*state.wi_stuff.wbs).last == state.wi_stuff.NUMCMAPS) {
        if (*state.wi_stuff.wbs).last > state.wi_stuff.NUMCMAPS {
            let mut tmp: patch_t = patch_t {
                width: SCREENWIDTH as i16,
                height: SCREENHEIGHT as i16,
                leftoffset: 1 as i16,
                topoffset: 1 as i16,
                columnofs: [
                    0 as i32, 0 as i32, 0 as i32, 0 as i32, 0 as i32, 0 as i32, 0 as i32, 0 as i32,
                ],
            };
            V_DrawPatch(state, 0 as i32, y, &raw mut tmp);
        }
    }
}
pub unsafe fn WI_drawEL(state: &mut GameState) {
    let mut y: i32 = WI_TITLEY;
    V_DrawPatch(
        state,
        (SCREENWIDTH - (*state.wi_stuff.entering).width as i32) / 2 as i32,
        y,
        state.wi_stuff.entering,
    );
    let next_patch = state.wi_stuff.lnames[(*state.wi_stuff.wbs).next as usize];
    y += 5 as i32 * (*next_patch).height as i32 / 4 as i32;
    V_DrawPatch(
        state,
        (SCREENWIDTH - (*next_patch).width as i32) / 2 as i32,
        y,
        next_patch,
    );
}
pub unsafe fn WI_drawOnLnode(state: &mut GameState, mut n: i32, mut c: *mut *mut patch_t) {
    let mut i: i32 = 0;
    let mut left: i32 = 0;
    let mut top: i32 = 0;
    let mut right: i32 = 0;
    let mut bottom: i32 = 0;
    let mut fits: bool = false;
    i = 0 as i32;
    loop {
        left = lnodes[(*state.wi_stuff.wbs).epsd as usize][n as usize].x
            - (**c.offset(i as isize)).leftoffset as i32;
        top = lnodes[(*state.wi_stuff.wbs).epsd as usize][n as usize].y
            - (**c.offset(i as isize)).topoffset as i32;
        right = left + (**c.offset(i as isize)).width as i32;
        bottom = top + (**c.offset(i as isize)).height as i32;
        if left >= 0 as i32 && right < SCREENWIDTH && top >= 0 as i32 && bottom < SCREENHEIGHT {
            fits = true;
        } else {
            i += 1;
        }
        if !(!fits && i != 2 as i32 && !(*c.offset(i as isize)).is_null()) {
            break;
        }
    }
    if fits && i < 2 as i32 {
        V_DrawPatch(
            state,
            lnodes[(*state.wi_stuff.wbs).epsd as usize][n as usize].x,
            lnodes[(*state.wi_stuff.wbs).epsd as usize][n as usize].y,
            *c.offset(i as isize),
        );
    } else {
        print!("Could not place patch on level {}", n + 1 as i32);
    };
}
pub unsafe fn WI_initAnimatedBack(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut a: *mut anim_t = ::core::ptr::null_mut::<anim_t>();
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        return;
    }
    if (*state.wi_stuff.wbs).epsd > 2 as i32 {
        return;
    }
    i = 0 as i32;
    while i < state.wi_stuff.NUMANIMS[(*state.wi_stuff.wbs).epsd as usize] {
        a = (*(&raw mut state.wi_stuff.anims as *mut *mut anim_t)
            .offset((*state.wi_stuff.wbs).epsd as isize))
        .offset(i as isize) as *mut anim_t;
        (*a).ctr = -(1 as i32);
        if (*a).type_0 == AnimEnum::ANIM_ALWAYS {
            (*a).nexttic =
                state.wi_stuff.bcnt + 1 as i32 + M_Random(&mut state.m_random) % (*a).period;
        } else if (*a).type_0 == AnimEnum::ANIM_RANDOM {
            (*a).nexttic = state.wi_stuff.bcnt
                + 1 as i32
                + (*a).data2
                + M_Random(&mut state.m_random) % (*a).data1;
        } else if (*a).type_0 == AnimEnum::ANIM_LEVEL {
            (*a).nexttic = state.wi_stuff.bcnt + 1 as i32;
        }
        i += 1;
    }
}
pub unsafe fn WI_updateAnimatedBack(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut a: *mut anim_t = ::core::ptr::null_mut::<anim_t>();
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        return;
    }
    if (*state.wi_stuff.wbs).epsd > 2 as i32 {
        return;
    }
    i = 0 as i32;
    while i < state.wi_stuff.NUMANIMS[(*state.wi_stuff.wbs).epsd as usize] {
        a = (*(&raw mut state.wi_stuff.anims as *mut *mut anim_t)
            .offset((*state.wi_stuff.wbs).epsd as isize))
        .offset(i as isize) as *mut anim_t;
        if state.wi_stuff.bcnt == (*a).nexttic {
            match (*a).type_0 as u32 {
                0 => {
                    (*a).ctr += 1;
                    if (*a).ctr >= (*a).nanims {
                        (*a).ctr = 0 as i32;
                    }
                    (*a).nexttic = state.wi_stuff.bcnt + (*a).period;
                }
                1 => {
                    (*a).ctr += 1;
                    if (*a).ctr == (*a).nanims {
                        (*a).ctr = -(1 as i32);
                        (*a).nexttic = state.wi_stuff.bcnt
                            + (*a).data2
                            + M_Random(&mut state.m_random) % (*a).data1;
                    } else {
                        (*a).nexttic = state.wi_stuff.bcnt + (*a).period;
                    }
                }
                2 => {
                    if !(state.wi_stuff.state == StateEnum::StatCount && i == 7 as i32)
                        && (*state.wi_stuff.wbs).next == (*a).data1
                    {
                        (*a).ctr += 1;
                        if (*a).ctr == (*a).nanims {
                            (*a).ctr -= 1;
                        }
                        (*a).nexttic = state.wi_stuff.bcnt + (*a).period;
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
}
pub unsafe fn WI_drawAnimatedBack(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut a: *mut anim_t = ::core::ptr::null_mut::<anim_t>();
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        return;
    }
    if (*state.wi_stuff.wbs).epsd > 2 as i32 {
        return;
    }
    i = 0 as i32;
    while i < state.wi_stuff.NUMANIMS[(*state.wi_stuff.wbs).epsd as usize] {
        a = (*(&raw mut state.wi_stuff.anims as *mut *mut anim_t)
            .offset((*state.wi_stuff.wbs).epsd as isize))
        .offset(i as isize) as *mut anim_t;
        if (*a).ctr >= 0 as i32 {
            V_DrawPatch(state, (*a).loc.x, (*a).loc.y, (*a).p[(*a).ctr as usize]);
        }
        i += 1;
    }
}
pub unsafe fn WI_drawNum(
    state: &mut GameState,
    mut x: i32,
    mut y: i32,
    mut n: i32,
    mut digits: i32,
) -> i32 {
    let mut fontwidth: i32 = (*state.wi_stuff.num[0]).width as i32;
    let mut neg: i32 = 0;
    let mut temp: i32 = 0;
    if digits < 0 as i32 {
        if n == 0 {
            digits = 1 as i32;
        } else {
            digits = 0 as i32;
            temp = n;
            while temp != 0 {
                temp /= 10 as i32;
                digits += 1;
            }
        }
    }
    neg = (n < 0 as i32) as i32;
    if neg != 0 {
        n = -n;
    }
    if n == 1994 as i32 {
        return 0 as i32;
    }
    loop {
        let fresh0 = digits;
        digits = digits - 1;
        if !(fresh0 != 0) {
            break;
        }
        x -= fontwidth;
        V_DrawPatch(state, x, y, state.wi_stuff.num[(n % 10 as i32) as usize]);
        n /= 10 as i32;
    }
    if neg != 0 {
        x -= 8 as i32;
        V_DrawPatch(state, x, y, state.wi_stuff.wiminus);
    }
    return x;
}
pub fn WI_drawPercent(state: &mut GameState, mut x: i32, mut y: i32, mut p_0: i32) {
    if p_0 < 0 as i32 {
        return;
    }
    unsafe { V_DrawPatch(state, x, y, state.wi_stuff.percent) };
    unsafe { WI_drawNum(state, x, y, p_0, -(1 as i32)) };
}
pub unsafe fn WI_drawTime(state: &mut GameState, mut x: i32, mut y: i32, mut t: i32) {
    let mut div: i32 = 0;
    let mut n: i32 = 0;
    if t < 0 as i32 {
        return;
    }
    if t <= 61 as i32 * 59 as i32 {
        div = 1 as i32;
        loop {
            n = t / div % 60 as i32;
            x = WI_drawNum(state, x, y, n, 2 as i32) - (*state.wi_stuff.colon).width as i32;
            div *= 60 as i32;
            if div == 60 as i32 || t / div != 0 {
                V_DrawPatch(state, x, y, state.wi_stuff.colon);
            }
            if !(t / div != 0) {
                break;
            }
        }
    } else {
        V_DrawPatch(
            state,
            x - (*state.wi_stuff.sucks).width as i32,
            y,
            state.wi_stuff.sucks,
        );
    };
}
pub unsafe fn WI_End(state: &mut GameState) {
    pub unsafe fn WI_unloadData_0(state: &mut GameState) {
        WI_loadUnloadData(
            state,
            Some(WI_unloadCallback as unsafe fn(&mut GameState, &str, *mut *mut patch_t) -> ()),
        );
    }
    WI_unloadData_0(state);
}
pub fn WI_initNoState(state: &mut GameState) {
    state.wi_stuff.state = StateEnum::NoState;
    state.wi_stuff.acceleratestage = 0 as i32;
    state.wi_stuff.cnt = 10 as i32;
}
pub fn WI_updateNoState(state: &mut GameState) {
    unsafe { WI_updateAnimatedBack(state) };
    state.wi_stuff.cnt -= 1;
    if state.wi_stuff.cnt == 0 {
        G_WorldDone(state);
    }
}
pub fn WI_initShowNextLoc(state: &mut GameState) {
    state.wi_stuff.state = StateEnum::ShowNextLoc;
    state.wi_stuff.acceleratestage = 0 as i32;
    state.wi_stuff.cnt = SHOWNEXTLOCDELAY * TICRATE;
    unsafe { WI_initAnimatedBack(state) };
}
pub fn WI_updateShowNextLoc(state: &mut GameState) {
    unsafe { WI_updateAnimatedBack(state) };
    state.wi_stuff.cnt -= 1;
    if state.wi_stuff.cnt == 0 || state.wi_stuff.acceleratestage != 0 {
        WI_initNoState(state);
    } else {
        state.wi_stuff.snl_pointeron = (state.wi_stuff.cnt & 31 as i32) < 20 as i32;
    };
}
pub unsafe fn WI_drawShowNextLoc(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut last: i32 = 0;
    WI_slamBackground(state);
    WI_drawAnimatedBack(state);
    if state.doomstat.gamemode as u32 != GameMode_t::commercial as i32 as u32 {
        if (*state.wi_stuff.wbs).epsd > 2 as i32 {
            WI_drawEL(state);
            return;
        }
        last = if (*state.wi_stuff.wbs).last == 8 as i32 {
            (*state.wi_stuff.wbs).next - 1 as i32
        } else {
            (*state.wi_stuff.wbs).last
        };
        i = 0 as i32;
        while i <= last {
            let splat = &raw mut state.wi_stuff.splat as *mut *mut patch_t;
            WI_drawOnLnode(state, i, splat);
            i += 1;
        }
        if (*state.wi_stuff.wbs).didsecret {
            let splat = &raw mut state.wi_stuff.splat as *mut *mut patch_t;
            WI_drawOnLnode(state, 8 as i32, splat);
        }
        if state.wi_stuff.snl_pointeron {
            let next = (*state.wi_stuff.wbs).next;
            let yah = &raw mut state.wi_stuff.yah as *mut *mut patch_t;
            WI_drawOnLnode(state, next, yah);
        }
    }
    if state.doomstat.gamemode as u32 != GameMode_t::commercial as i32 as u32
        || (*state.wi_stuff.wbs).next != 30 as i32
    {
        WI_drawEL(state);
    }
}
pub fn WI_drawNoState(state: &mut GameState) {
    state.wi_stuff.snl_pointeron = true;
    unsafe { WI_drawShowNextLoc(state) };
}
pub unsafe fn WI_fragSum(state: &mut GameState, mut playernum: i32) -> i32 {
    let mut i: i32 = 0;
    let mut frags_0: i32 = 0 as i32;
    i = 0 as i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] && i != playernum {
            frags_0 += (*state.wi_stuff.plrs.offset(playernum as isize)).frags[i as usize];
        }
        i += 1;
    }
    frags_0 -= (*state.wi_stuff.plrs.offset(playernum as isize)).frags[playernum as usize];
    return frags_0;
}
pub fn WI_initDeathmatchStats(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    state.wi_stuff.state = StateEnum::StatCount;
    state.wi_stuff.acceleratestage = 0 as i32;
    state.wi_stuff.dm_state = 1 as i32;
    state.wi_stuff.cnt_pause = TICRATE;
    i = 0 as i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            j = 0 as i32;
            while j < MAXPLAYERS {
                if state.g_game.playeringame[j as usize] {
                    state.wi_stuff.dm_frags[i as usize][j as usize] = 0 as i32;
                }
                j += 1;
            }
            state.wi_stuff.dm_totals[i as usize] = 0 as i32;
        }
        i += 1;
    }
    unsafe { WI_initAnimatedBack(state) };
}
pub unsafe fn WI_updateDeathmatchStats(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut stillticking: bool = false;
    WI_updateAnimatedBack(state);
    if state.wi_stuff.acceleratestage != 0 && state.wi_stuff.dm_state != 4 as i32 {
        state.wi_stuff.acceleratestage = 0 as i32;
        i = 0 as i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                j = 0 as i32;
                while j < MAXPLAYERS {
                    if state.g_game.playeringame[j as usize] {
                        state.wi_stuff.dm_frags[i as usize][j as usize] =
                            (*state.wi_stuff.plrs.offset(i as isize)).frags[j as usize];
                    }
                    j += 1;
                }
                state.wi_stuff.dm_totals[i as usize] = WI_fragSum(state, i);
            }
            i += 1;
        }
        S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
        state.wi_stuff.dm_state = 4 as i32;
    }
    if state.wi_stuff.dm_state == 2 as i32 {
        if state.wi_stuff.bcnt & 3 as i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        stillticking = false;
        i = 0 as i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                j = 0 as i32;
                while j < MAXPLAYERS {
                    if state.g_game.playeringame[j as usize]
                        && state.wi_stuff.dm_frags[i as usize][j as usize]
                            != (*state.wi_stuff.plrs.offset(i as isize)).frags[j as usize]
                    {
                        if (*state.wi_stuff.plrs.offset(i as isize)).frags[j as usize] < 0 as i32 {
                            state.wi_stuff.dm_frags[i as usize][j as usize] -= 1;
                        } else {
                            state.wi_stuff.dm_frags[i as usize][j as usize] += 1;
                        }
                        if state.wi_stuff.dm_frags[i as usize][j as usize] > 99 as i32 {
                            state.wi_stuff.dm_frags[i as usize][j as usize] = 99 as i32;
                        }
                        if state.wi_stuff.dm_frags[i as usize][j as usize] < -(99 as i32) {
                            state.wi_stuff.dm_frags[i as usize][j as usize] = -(99 as i32);
                        }
                        stillticking = true;
                    }
                    j += 1;
                }
                state.wi_stuff.dm_totals[i as usize] = WI_fragSum(state, i);
                if state.wi_stuff.dm_totals[i as usize] > 99 as i32 {
                    state.wi_stuff.dm_totals[i as usize] = 99 as i32;
                }
                if state.wi_stuff.dm_totals[i as usize] < -(99 as i32) {
                    state.wi_stuff.dm_totals[i as usize] = -(99 as i32);
                }
            }
            i += 1;
        }
        if !stillticking {
            S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
            state.wi_stuff.dm_state += 1;
        }
    } else if state.wi_stuff.dm_state == 4 as i32 {
        if state.wi_stuff.acceleratestage != 0 {
            S_StartSound(state, SoundOrigin::None, sfx_slop as i32);
            if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
                WI_initNoState(state);
            } else {
                WI_initShowNextLoc(state);
            }
        }
    } else if state.wi_stuff.dm_state & 1 as i32 != 0 {
        state.wi_stuff.cnt_pause -= 1;
        if state.wi_stuff.cnt_pause == 0 {
            state.wi_stuff.dm_state += 1;
            state.wi_stuff.cnt_pause = TICRATE;
        }
    }
}
pub unsafe fn WI_drawDeathmatchStats(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut w: i32 = 0;
    WI_slamBackground(state);
    WI_drawAnimatedBack(state);
    WI_drawLF(state);
    V_DrawPatch(
        state,
        DM_TOTALSX - (*state.wi_stuff.total).width as i32 / 2 as i32,
        DM_MATRIXY - WI_SPACINGY + 10 as i32,
        state.wi_stuff.total,
    );
    V_DrawPatch(state, DM_KILLERSX, DM_KILLERSY, state.wi_stuff.killers);
    V_DrawPatch(state, DM_VICTIMSX, DM_VICTIMSY, state.wi_stuff.victims);
    x = DM_MATRIXX + DM_SPACINGX;
    y = DM_MATRIXY;
    i = 0 as i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            V_DrawPatch(
                state,
                x - (*state.wi_stuff.p[i as usize]).width as i32 / 2 as i32,
                DM_MATRIXY - WI_SPACINGY,
                state.wi_stuff.p[i as usize],
            );
            V_DrawPatch(
                state,
                DM_MATRIXX - (*state.wi_stuff.p[i as usize]).width as i32 / 2 as i32,
                y,
                state.wi_stuff.p[i as usize],
            );
            if i == state.wi_stuff.me {
                V_DrawPatch(
                    state,
                    x - (*state.wi_stuff.p[i as usize]).width as i32 / 2 as i32,
                    DM_MATRIXY - WI_SPACINGY,
                    state.wi_stuff.bstar,
                );
                V_DrawPatch(
                    state,
                    DM_MATRIXX - (*state.wi_stuff.p[i as usize]).width as i32 / 2 as i32,
                    y,
                    state.wi_stuff.star,
                );
            }
        }
        x += DM_SPACINGX;
        y += WI_SPACINGY;
        i += 1;
    }
    y = DM_MATRIXY + 10 as i32;
    w = (*state.wi_stuff.num[0]).width as i32;
    i = 0 as i32;
    while i < MAXPLAYERS {
        x = DM_MATRIXX + DM_SPACINGX;
        if state.g_game.playeringame[i as usize] {
            j = 0 as i32;
            while j < MAXPLAYERS {
                if state.g_game.playeringame[j as usize] {
                    let dm_frags = state.wi_stuff.dm_frags[i as usize][j as usize];
                    WI_drawNum(state, x + w, y, dm_frags, 2 as i32);
                }
                x += DM_SPACINGX;
                j += 1;
            }
            let dm_totals = state.wi_stuff.dm_totals[i as usize];
            WI_drawNum(state, DM_TOTALSX + w, y, dm_totals, 2 as i32);
        }
        y += WI_SPACINGY;
        i += 1;
    }
}
pub fn WI_initNetgameStats(state: &mut GameState) {
    let mut i: i32 = 0;
    state.wi_stuff.state = StateEnum::StatCount;
    state.wi_stuff.acceleratestage = 0 as i32;
    state.wi_stuff.ng_state = 1 as i32;
    state.wi_stuff.cnt_pause = TICRATE;
    i = 0 as i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            state.wi_stuff.cnt_frags[i as usize] = 0 as i32;
            state.wi_stuff.cnt_secret[i as usize] = state.wi_stuff.cnt_frags[i as usize];
            state.wi_stuff.cnt_items[i as usize] = state.wi_stuff.cnt_secret[i as usize];
            state.wi_stuff.cnt_kills[i as usize] = state.wi_stuff.cnt_items[i as usize];
            let fragsum = unsafe { WI_fragSum(state, i) };
            state.wi_stuff.dofrags += fragsum;
        }
        i += 1;
    }
    state.wi_stuff.dofrags = (state.wi_stuff.dofrags != 0) as i32;
    unsafe { WI_initAnimatedBack(state) };
}
pub unsafe fn WI_updateNetgameStats(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut fsum: i32 = 0;
    let mut stillticking: bool = false;
    WI_updateAnimatedBack(state);
    if state.wi_stuff.acceleratestage != 0 && state.wi_stuff.ng_state != 10 as i32 {
        state.wi_stuff.acceleratestage = 0 as i32;
        i = 0 as i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                state.wi_stuff.cnt_kills[i as usize] =
                    (*state.wi_stuff.plrs.offset(i as isize)).skills * 100 as i32
                        / (*state.wi_stuff.wbs).maxkills;
                state.wi_stuff.cnt_items[i as usize] =
                    (*state.wi_stuff.plrs.offset(i as isize)).sitems * 100 as i32
                        / (*state.wi_stuff.wbs).maxitems;
                state.wi_stuff.cnt_secret[i as usize] =
                    (*state.wi_stuff.plrs.offset(i as isize)).ssecret * 100 as i32
                        / (*state.wi_stuff.wbs).maxsecret;
                if state.wi_stuff.dofrags != 0 {
                    state.wi_stuff.cnt_frags[i as usize] = WI_fragSum(state, i);
                }
            }
            i += 1;
        }
        S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
        state.wi_stuff.ng_state = 10 as i32;
    }
    if state.wi_stuff.ng_state == 2 as i32 {
        if state.wi_stuff.bcnt & 3 as i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        stillticking = false;
        i = 0 as i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                state.wi_stuff.cnt_kills[i as usize] += 2 as i32;
                if state.wi_stuff.cnt_kills[i as usize]
                    >= (*state.wi_stuff.plrs.offset(i as isize)).skills * 100 as i32
                        / (*state.wi_stuff.wbs).maxkills
                {
                    state.wi_stuff.cnt_kills[i as usize] =
                        (*state.wi_stuff.plrs.offset(i as isize)).skills * 100 as i32
                            / (*state.wi_stuff.wbs).maxkills;
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
    } else if state.wi_stuff.ng_state == 4 as i32 {
        if state.wi_stuff.bcnt & 3 as i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        stillticking = false;
        i = 0 as i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                state.wi_stuff.cnt_items[i as usize] += 2 as i32;
                if state.wi_stuff.cnt_items[i as usize]
                    >= (*state.wi_stuff.plrs.offset(i as isize)).sitems * 100 as i32
                        / (*state.wi_stuff.wbs).maxitems
                {
                    state.wi_stuff.cnt_items[i as usize] =
                        (*state.wi_stuff.plrs.offset(i as isize)).sitems * 100 as i32
                            / (*state.wi_stuff.wbs).maxitems;
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
    } else if state.wi_stuff.ng_state == 6 as i32 {
        if state.wi_stuff.bcnt & 3 as i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        stillticking = false;
        i = 0 as i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                state.wi_stuff.cnt_secret[i as usize] += 2 as i32;
                if state.wi_stuff.cnt_secret[i as usize]
                    >= (*state.wi_stuff.plrs.offset(i as isize)).ssecret * 100 as i32
                        / (*state.wi_stuff.wbs).maxsecret
                {
                    state.wi_stuff.cnt_secret[i as usize] =
                        (*state.wi_stuff.plrs.offset(i as isize)).ssecret * 100 as i32
                            / (*state.wi_stuff.wbs).maxsecret;
                } else {
                    stillticking = true;
                }
            }
            i += 1;
        }
        if !stillticking {
            S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
            state.wi_stuff.ng_state += 1 as i32 + 2 as i32 * (state.wi_stuff.dofrags == 0) as i32;
        }
    } else if state.wi_stuff.ng_state == 8 as i32 {
        if state.wi_stuff.bcnt & 3 as i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        stillticking = false;
        i = 0 as i32;
        while i < MAXPLAYERS {
            if state.g_game.playeringame[i as usize] {
                state.wi_stuff.cnt_frags[i as usize] += 1 as i32;
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
    } else if state.wi_stuff.ng_state == 10 as i32 {
        if state.wi_stuff.acceleratestage != 0 {
            S_StartSound(state, SoundOrigin::None, sfx_sgcock as i32);
            if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
                WI_initNoState(state);
            } else {
                WI_initShowNextLoc(state);
            }
        }
    } else if state.wi_stuff.ng_state & 1 as i32 != 0 {
        state.wi_stuff.cnt_pause -= 1;
        if state.wi_stuff.cnt_pause == 0 {
            state.wi_stuff.ng_state += 1;
            state.wi_stuff.cnt_pause = TICRATE;
        }
    }
}
pub unsafe fn WI_drawNetgameStats(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut pwidth: i32 = (*state.wi_stuff.percent).width as i32;
    WI_slamBackground(state);
    WI_drawAnimatedBack(state);
    WI_drawLF(state);
    V_DrawPatch(
        state,
        32 as i32
            + (*state.wi_stuff.star).width as i32 / 2 as i32
            + 32 as i32 * (state.wi_stuff.dofrags == 0) as i32
            + NG_SPACINGX
            - (*state.wi_stuff.kills).width as i32,
        NG_STATSY,
        state.wi_stuff.kills,
    );
    V_DrawPatch(
        state,
        32 as i32
            + (*state.wi_stuff.star).width as i32 / 2 as i32
            + 32 as i32 * (state.wi_stuff.dofrags == 0) as i32
            + 2 as i32 * NG_SPACINGX
            - (*state.wi_stuff.items).width as i32,
        NG_STATSY,
        state.wi_stuff.items,
    );
    V_DrawPatch(
        state,
        32 as i32
            + (*state.wi_stuff.star).width as i32 / 2 as i32
            + 32 as i32 * (state.wi_stuff.dofrags == 0) as i32
            + 3 as i32 * NG_SPACINGX
            - (*state.wi_stuff.secret).width as i32,
        NG_STATSY,
        state.wi_stuff.secret,
    );
    if state.wi_stuff.dofrags != 0 {
        V_DrawPatch(
            state,
            32 as i32
                + (*state.wi_stuff.star).width as i32 / 2 as i32
                + 32 as i32 * (state.wi_stuff.dofrags == 0) as i32
                + 4 as i32 * NG_SPACINGX
                - (*state.wi_stuff.frags).width as i32,
            NG_STATSY,
            state.wi_stuff.frags,
        );
    }
    y = NG_STATSY + (*state.wi_stuff.kills).height as i32;
    i = 0 as i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            x = 32 as i32
                + (*state.wi_stuff.star).width as i32 / 2 as i32
                + 32 as i32 * (state.wi_stuff.dofrags == 0) as i32;
            V_DrawPatch(
                state,
                x - (*state.wi_stuff.p[i as usize]).width as i32,
                y,
                state.wi_stuff.p[i as usize],
            );
            if i == state.wi_stuff.me {
                V_DrawPatch(
                    state,
                    x - (*state.wi_stuff.p[i as usize]).width as i32,
                    y,
                    state.wi_stuff.star,
                );
            }
            x += NG_SPACINGX;
            let cnt_kills = state.wi_stuff.cnt_kills[i as usize];
            WI_drawPercent(state, x - pwidth, y + 10 as i32, cnt_kills);
            x += NG_SPACINGX;
            let cnt_items = state.wi_stuff.cnt_items[i as usize];
            WI_drawPercent(state, x - pwidth, y + 10 as i32, cnt_items);
            x += NG_SPACINGX;
            let cnt_secret = state.wi_stuff.cnt_secret[i as usize];
            WI_drawPercent(state, x - pwidth, y + 10 as i32, cnt_secret);
            x += NG_SPACINGX;
            if state.wi_stuff.dofrags != 0 {
                let cnt_frags = state.wi_stuff.cnt_frags[i as usize];
                WI_drawNum(state, x, y + 10 as i32, cnt_frags, -(1 as i32));
            }
            y += WI_SPACINGY;
        }
        i += 1;
    }
}
pub fn WI_initStats(state: &mut GameState) {
    state.wi_stuff.state = StateEnum::StatCount;
    state.wi_stuff.acceleratestage = 0 as i32;
    state.wi_stuff.sp_state = 1 as i32;
    state.wi_stuff.cnt_secret[0] = -(1 as i32);
    state.wi_stuff.cnt_items[0] = state.wi_stuff.cnt_secret[0];
    state.wi_stuff.cnt_kills[0] = state.wi_stuff.cnt_items[0];
    state.wi_stuff.cnt_par = -(1 as i32);
    state.wi_stuff.cnt_time = state.wi_stuff.cnt_par;
    state.wi_stuff.cnt_pause = TICRATE;
    unsafe { WI_initAnimatedBack(state) };
}
pub unsafe fn WI_updateStats(state: &mut GameState) {
    WI_updateAnimatedBack(state);
    if state.wi_stuff.acceleratestage != 0 && state.wi_stuff.sp_state != 10 as i32 {
        state.wi_stuff.acceleratestage = 0 as i32;
        state.wi_stuff.cnt_kills[0] =
            (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).skills * 100 as i32
                / (*state.wi_stuff.wbs).maxkills;
        state.wi_stuff.cnt_items[0] =
            (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).sitems * 100 as i32
                / (*state.wi_stuff.wbs).maxitems;
        state.wi_stuff.cnt_secret[0] =
            (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).ssecret * 100 as i32
                / (*state.wi_stuff.wbs).maxsecret;
        state.wi_stuff.cnt_time =
            (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).stime / TICRATE;
        state.wi_stuff.cnt_par = (*state.wi_stuff.wbs).partime / TICRATE;
        S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
        state.wi_stuff.sp_state = 10 as i32;
    }
    if state.wi_stuff.sp_state == 2 as i32 {
        state.wi_stuff.cnt_kills[0] += 2 as i32;
        if state.wi_stuff.bcnt & 3 as i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        if state.wi_stuff.cnt_kills[0]
            >= (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).skills * 100 as i32
                / (*state.wi_stuff.wbs).maxkills
        {
            state.wi_stuff.cnt_kills[0] =
                (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).skills * 100 as i32
                    / (*state.wi_stuff.wbs).maxkills;
            S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
            state.wi_stuff.sp_state += 1;
        }
    } else if state.wi_stuff.sp_state == 4 as i32 {
        state.wi_stuff.cnt_items[0] += 2 as i32;
        if state.wi_stuff.bcnt & 3 as i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        if state.wi_stuff.cnt_items[0]
            >= (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).sitems * 100 as i32
                / (*state.wi_stuff.wbs).maxitems
        {
            state.wi_stuff.cnt_items[0] =
                (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).sitems * 100 as i32
                    / (*state.wi_stuff.wbs).maxitems;
            S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
            state.wi_stuff.sp_state += 1;
        }
    } else if state.wi_stuff.sp_state == 6 as i32 {
        state.wi_stuff.cnt_secret[0] += 2 as i32;
        if state.wi_stuff.bcnt & 3 as i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        if state.wi_stuff.cnt_secret[0]
            >= (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).ssecret * 100 as i32
                / (*state.wi_stuff.wbs).maxsecret
        {
            state.wi_stuff.cnt_secret[0] =
                (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).ssecret * 100 as i32
                    / (*state.wi_stuff.wbs).maxsecret;
            S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
            state.wi_stuff.sp_state += 1;
        }
    } else if state.wi_stuff.sp_state == 8 as i32 {
        if state.wi_stuff.bcnt & 3 as i32 == 0 {
            S_StartSound(state, SoundOrigin::None, sfx_pistol as i32);
        }
        state.wi_stuff.cnt_time += 3 as i32;
        if state.wi_stuff.cnt_time
            >= (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).stime / TICRATE
        {
            state.wi_stuff.cnt_time =
                (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).stime / TICRATE;
        }
        state.wi_stuff.cnt_par += 3 as i32;
        if state.wi_stuff.cnt_par >= (*state.wi_stuff.wbs).partime / TICRATE {
            state.wi_stuff.cnt_par = (*state.wi_stuff.wbs).partime / TICRATE;
            if state.wi_stuff.cnt_time
                >= (*state.wi_stuff.plrs.offset(state.wi_stuff.me as isize)).stime / TICRATE
            {
                S_StartSound(state, SoundOrigin::None, sfx_barexp as i32);
                state.wi_stuff.sp_state += 1;
            }
        }
    } else if state.wi_stuff.sp_state == 10 as i32 {
        if state.wi_stuff.acceleratestage != 0 {
            S_StartSound(state, SoundOrigin::None, sfx_sgcock as i32);
            if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
                WI_initNoState(state);
            } else {
                WI_initShowNextLoc(state);
            }
        }
    } else if state.wi_stuff.sp_state & 1 as i32 != 0 {
        state.wi_stuff.cnt_pause -= 1;
        if state.wi_stuff.cnt_pause == 0 {
            state.wi_stuff.sp_state += 1;
            state.wi_stuff.cnt_pause = TICRATE;
        }
    }
}
pub unsafe fn WI_drawStats(state: &mut GameState) {
    let mut lh: i32 = 0;
    lh = 3 as i32 * (*state.wi_stuff.num[0]).height as i32 / 2 as i32;
    WI_slamBackground(state);
    WI_drawAnimatedBack(state);
    WI_drawLF(state);
    V_DrawPatch(state, SP_STATSX, SP_STATSY, state.wi_stuff.kills);
    let cnt_kills = state.wi_stuff.cnt_kills[0];
    WI_drawPercent(state, SCREENWIDTH - SP_STATSX, SP_STATSY, cnt_kills);
    V_DrawPatch(state, SP_STATSX, SP_STATSY + lh, state.wi_stuff.items);
    let cnt_items = state.wi_stuff.cnt_items[0];
    WI_drawPercent(state, SCREENWIDTH - SP_STATSX, SP_STATSY + lh, cnt_items);
    V_DrawPatch(
        state,
        SP_STATSX,
        SP_STATSY + 2 as i32 * lh,
        state.wi_stuff.sp_secret,
    );
    let cnt_secret = state.wi_stuff.cnt_secret[0];
    WI_drawPercent(
        state,
        SCREENWIDTH - SP_STATSX,
        SP_STATSY + 2 as i32 * lh,
        cnt_secret,
    );
    V_DrawPatch(state, SP_TIMEX, SP_TIMEY, state.wi_stuff.timepatch);
    let cnt_time = state.wi_stuff.cnt_time;
    WI_drawTime(state, SCREENWIDTH / 2 as i32 - SP_TIMEX, SP_TIMEY, cnt_time);
    if (*state.wi_stuff.wbs).epsd < 3 as i32 {
        V_DrawPatch(
            state,
            SCREENWIDTH / 2 as i32 + SP_TIMEX,
            SP_TIMEY,
            state.wi_stuff.par,
        );
        let cnt_par = state.wi_stuff.cnt_par;
        WI_drawTime(state, SCREENWIDTH - SP_TIMEX, SP_TIMEY, cnt_par);
    }
}
pub unsafe fn WI_checkForAccelerate(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut player: *mut player_t = ::core::ptr::null_mut::<player_t>();
    i = 0 as i32;
    player = &raw mut state.g_game.players as *mut player_t;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            if (*player).cmd.buttons as i32 & BT_ATTACK as i32 != 0 {
                if (*player).attackdown == 0 {
                    state.wi_stuff.acceleratestage = 1 as i32;
                }
                (*player).attackdown = true_0;
            } else {
                (*player).attackdown = false_0;
            }
            if (*player).cmd.buttons as i32 & BT_USE as i32 != 0 {
                if (*player).usedown == 0 {
                    state.wi_stuff.acceleratestage = 1 as i32;
                }
                (*player).usedown = true_0;
            } else {
                (*player).usedown = false_0;
            }
        }
        i += 1;
        player = player.offset(1);
    }
}
pub fn WI_Ticker(state: &mut GameState) {
    state.wi_stuff.bcnt += 1;
    if state.wi_stuff.bcnt == 1 as i32 {
        if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
            unsafe { S_ChangeMusic(state, mus_dm2int as i32, true_0) };
        } else {
            unsafe { S_ChangeMusic(state, mus_inter as i32, true_0) };
        }
    }
    unsafe { WI_checkForAccelerate(state) };
    match state.wi_stuff.state {
        StateEnum::StatCount => {
            if state.g_game.deathmatch != 0 {
                unsafe { WI_updateDeathmatchStats(state) };
            } else if state.g_game.netgame {
                unsafe { WI_updateNetgameStats(state) };
            } else {
                unsafe { WI_updateStats(state) };
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
unsafe fn WI_loadUnloadData(state: &mut GameState, mut callback: load_callback_t) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut a: *mut anim_t = ::core::ptr::null_mut::<anim_t>();
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        i = 0 as i32;
        while i < state.wi_stuff.NUMCMAPS {
            let cb_ptr = state.wi_stuff.lnames.as_mut_ptr().offset(i as isize) as *mut *mut patch_t;
            callback.expect("non-null function pointer")(state, &format!("CWILV{:02}", i,), cb_ptr);
            i += 1;
        }
    } else {
        i = 0 as i32;
        while i < NUMMAPS {
            let cb_ptr = state.wi_stuff.lnames.as_mut_ptr().offset(i as isize) as *mut *mut patch_t;
            callback.expect("non-null function pointer")(
                state,
                &format!("WILV{}{}", (*state.wi_stuff.wbs).epsd, i,),
                cb_ptr,
            );
            i += 1;
        }
        let cb_ptr = (&raw mut state.wi_stuff.yah as *mut *mut patch_t).offset(0 as i32 as isize)
            as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, "WIURH0", cb_ptr);
        let cb_ptr = (&raw mut state.wi_stuff.yah as *mut *mut patch_t).offset(1 as i32 as isize)
            as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, "WIURH1", cb_ptr);
        let cb_ptr = (&raw mut state.wi_stuff.splat as *mut *mut patch_t).offset(0 as i32 as isize)
            as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, "WISPLAT", cb_ptr);
        if (*state.wi_stuff.wbs).epsd < 3 as i32 {
            j = 0 as i32;
            while j < state.wi_stuff.NUMANIMS[(*state.wi_stuff.wbs).epsd as usize] {
                a = (*(&raw mut state.wi_stuff.anims as *mut *mut anim_t)
                    .offset((*state.wi_stuff.wbs).epsd as isize))
                .offset(j as isize) as *mut anim_t;
                i = 0 as i32;
                while i < (*a).nanims {
                    if (*state.wi_stuff.wbs).epsd != 1 as i32 || j != 8 as i32 {
                        callback.expect("non-null function pointer")(
                            state,
                            &format!("WIA{}{:02}{:02}", (*state.wi_stuff.wbs).epsd, j, i,),
                            (&raw mut (*a).p as *mut *mut patch_t).offset(i as isize)
                                as *mut *mut patch_t,
                        );
                    } else {
                        (*a).p[i as usize] =
                            (*state.wi_stuff.anims[1].offset(4 as i32 as isize)).p[i as usize];
                    }
                    i += 1;
                }
                j += 1;
            }
        }
    }
    let cb_ptr = &raw mut state.wi_stuff.wiminus;
    callback.expect("non-null function pointer")(state, "WIMINUS", cb_ptr);
    i = 0 as i32;
    while i < 10 as i32 {
        let cb_ptr = (&raw mut state.wi_stuff.num as *mut *mut patch_t).offset(i as isize)
            as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, &format!("WINUM{}", i,), cb_ptr);
        i += 1;
    }
    let cb_ptr = &raw mut state.wi_stuff.percent;
    callback.expect("non-null function pointer")(state, "WIPCNT", cb_ptr);
    let cb_ptr = &raw mut state.wi_stuff.finished;
    callback.expect("non-null function pointer")(state, "WIF", cb_ptr);
    let cb_ptr = &raw mut state.wi_stuff.entering;
    callback.expect("non-null function pointer")(state, "WIENTER", cb_ptr);
    let cb_ptr = &raw mut state.wi_stuff.kills;
    callback.expect("non-null function pointer")(state, "WIOSTK", cb_ptr);
    let cb_ptr = &raw mut state.wi_stuff.secret;
    callback.expect("non-null function pointer")(state, "WIOSTS", cb_ptr);
    let cb_ptr = &raw mut state.wi_stuff.sp_secret;
    callback.expect("non-null function pointer")(state, "WISCRT2", cb_ptr);
    if W_CheckNumForName(&mut state.w_wad, "WIOBJ") >= 0 as i32 {
        if state.g_game.netgame && state.g_game.deathmatch == 0 {
            let cb_ptr = &raw mut state.wi_stuff.items;
            callback.expect("non-null function pointer")(state, "WIOBJ", cb_ptr);
        } else {
            let cb_ptr = &raw mut state.wi_stuff.items;
            callback.expect("non-null function pointer")(state, "WIOSTI", cb_ptr);
        }
    } else {
        let cb_ptr = &raw mut state.wi_stuff.items;
        callback.expect("non-null function pointer")(state, "WIOSTI", cb_ptr);
    }
    let cb_ptr = &raw mut state.wi_stuff.frags;
    callback.expect("non-null function pointer")(state, "WIFRGS", cb_ptr);
    let cb_ptr = &raw mut state.wi_stuff.colon;
    callback.expect("non-null function pointer")(state, "WICOLON", cb_ptr);
    let cb_ptr = &raw mut state.wi_stuff.timepatch;
    callback.expect("non-null function pointer")(state, "WITIME", cb_ptr);
    let cb_ptr = &raw mut state.wi_stuff.sucks;
    callback.expect("non-null function pointer")(state, "WISUCKS", cb_ptr);
    let cb_ptr = &raw mut state.wi_stuff.par;
    callback.expect("non-null function pointer")(state, "WIPAR", cb_ptr);
    let cb_ptr = &raw mut state.wi_stuff.killers;
    callback.expect("non-null function pointer")(state, "WIKILRS", cb_ptr);
    let cb_ptr = &raw mut state.wi_stuff.victims;
    callback.expect("non-null function pointer")(state, "WIVCTMS", cb_ptr);
    let cb_ptr = &raw mut state.wi_stuff.total;
    callback.expect("non-null function pointer")(state, "WIMSTT", cb_ptr);
    i = 0 as i32;
    while i < MAXPLAYERS {
        let cb_ptr = (&raw mut state.wi_stuff.p as *mut *mut patch_t).offset(i as isize)
            as *mut *mut patch_t;
        callback.expect("non-null function pointer")(state, &format!("STPB{}", i,), cb_ptr);
        let cb_ptr = (&raw mut state.wi_stuff.bp as *mut *mut patch_t).offset(i as isize)
            as *mut *mut patch_t;
        callback.expect("non-null function pointer")(
            state,
            &format!("WIBP{}", i + 1 as i32,),
            cb_ptr,
        );
        i += 1;
    }
    let name = if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        "INTERPIC".to_string()
    } else if state.doomstat.gamemode as u32 == GameMode_t::retail as i32 as u32
        && (*state.wi_stuff.wbs).epsd == 3 as i32
    {
        "INTERPIC".to_string()
    } else {
        format!("WIMAP{}", (*state.wi_stuff.wbs).epsd)
    };
    let cb_ptr = &raw mut state.wi_stuff.background;
    callback.expect("non-null function pointer")(state, &name, cb_ptr);
}
unsafe fn WI_loadCallback(state: &mut GameState, name: &str, variable: *mut *mut patch_t) {
    *variable = W_CacheLumpName(state, name) as *mut patch_t;
}
pub unsafe fn WI_loadData(state: &mut GameState) {
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        state.wi_stuff.NUMCMAPS = 32 as i32;
        state.wi_stuff.lnames =
            vec![::core::ptr::null_mut::<patch_t>(); state.wi_stuff.NUMCMAPS as usize];
    } else {
        state.wi_stuff.lnames = vec![::core::ptr::null_mut::<patch_t>(); NUMMAPS as usize];
    }
    WI_loadUnloadData(
        state,
        Some(WI_loadCallback as unsafe fn(&mut GameState, &str, *mut *mut patch_t) -> ()),
    );
    state.wi_stuff.star = W_CacheLumpName(state, "STFST01") as *mut patch_t;
    state.wi_stuff.bstar = W_CacheLumpName(state, "STFDEAD0") as *mut patch_t;
}
fn WI_unloadCallback(state: &mut GameState, name: &str, variable: *mut *mut patch_t) {
    unsafe {
        W_ReleaseLumpName(&mut state.w_wad, name);
        *variable = ::core::ptr::null_mut::<patch_t>();
    }
}
pub fn WI_Drawer(state: &mut GameState) {
    match state.wi_stuff.state {
        StateEnum::StatCount => {
            if state.g_game.deathmatch != 0 {
                unsafe { WI_drawDeathmatchStats(state) };
            } else if state.g_game.netgame {
                unsafe { WI_drawNetgameStats(state) };
            } else {
                unsafe { WI_drawStats(state) };
            }
        }
        StateEnum::ShowNextLoc => {
            unsafe { WI_drawShowNextLoc(state) };
        }
        StateEnum::NoState => {
            WI_drawNoState(state);
        }
    };
}
pub unsafe fn WI_initVariables(state: &mut GameState, mut wbstartstruct: *mut wbstartstruct_t) {
    state.wi_stuff.wbs = wbstartstruct;
    state.wi_stuff.acceleratestage = 0 as i32;
    state.wi_stuff.bcnt = 0 as i32;
    state.wi_stuff.cnt = state.wi_stuff.bcnt;
    state.wi_stuff.firstrefresh = 1 as i32;
    state.wi_stuff.me = (*state.wi_stuff.wbs).pnum;
    state.wi_stuff.plrs = &raw mut (*state.wi_stuff.wbs).plyr as *mut wbplayerstruct_t;
    if (*state.wi_stuff.wbs).maxkills == 0 {
        (*state.wi_stuff.wbs).maxkills = 1 as i32;
    }
    if (*state.wi_stuff.wbs).maxitems == 0 {
        (*state.wi_stuff.wbs).maxitems = 1 as i32;
    }
    if (*state.wi_stuff.wbs).maxsecret == 0 {
        (*state.wi_stuff.wbs).maxsecret = 1 as i32;
    }
    if state.doomstat.gamemode as u32 != GameMode_t::retail as i32 as u32 {
        if (*state.wi_stuff.wbs).epsd > 2 as i32 {
            (*state.wi_stuff.wbs).epsd -= 3 as i32;
        }
    }
}
pub fn WI_Start(state: &mut GameState, mut wbstartstruct: *mut wbstartstruct_t) {
    unsafe { WI_initVariables(state, wbstartstruct) };
    unsafe { WI_loadData(state) };
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
        (::core::mem::size_of::<[anim_t; 10]>() as usize)
            .wrapping_div(::core::mem::size_of::<anim_t>() as usize) as i32,
        (::core::mem::size_of::<[anim_t; 9]>() as usize)
            .wrapping_div(::core::mem::size_of::<anim_t>() as usize) as i32,
        (::core::mem::size_of::<[anim_t; 6]>() as usize)
            .wrapping_div(::core::mem::size_of::<anim_t>() as usize) as i32,
        0,
    ];
}
