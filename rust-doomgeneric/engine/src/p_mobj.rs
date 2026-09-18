use crate::d_mode::SkillType;
use crate::d_player::CF_NOMOMENTUM;
use crate::doomdef::MAXPLAYERS;
use crate::doomdef::TICRATE;
use crate::g_game::G_PlayerReborn;
use crate::game_state::GameState;
use crate::hu_stuff::HU_Start;
use crate::i_system::I_Error;

use crate::info::StateId;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::m_fixed::INT_MAX;
use crate::m_fixed::INT_MIN;
use crate::m_random::P_Random;

use crate::p_ceilng::CeilingId;
use crate::p_doors::DoorId;
use crate::p_enemy::MELEERANGE;
use crate::p_inter::NUMCARDS;
use crate::p_lights::{FireFlickerId, GlowId, LightFlashId, StrobeId};
use crate::p_map::P_AimLineAttack;
use crate::p_map::P_CheckPosition;
use crate::p_map::P_SlideMove;
use crate::p_map::P_TryMove;
use crate::p_maputl::P_AproxDistance;
use crate::p_maputl::P_SetThingPosition;
use crate::p_maputl::P_UnsetThingPosition;
use crate::p_plats::PlatId;
use crate::p_pspr::P_SetupPsprites;
use crate::p_setup::{LineId, SectorId, SubsectorId, VertexId};
use crate::p_spec::FloorId;
use crate::p_tick::P_AddThinker;
use crate::p_tick::P_RemoveThinker;
use crate::p_tick::ThinkerId;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::p_user::VIEWHEIGHT;
use crate::r_main::R_PointInSubsector;
use crate::r_main::R_PointToAngle2;
use crate::s_sound::S_StartSound;
use crate::s_sound::S_StopSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::st_stuff::ST_Start;

use crate::tables::angle_t;
use crate::tables::finecosine;
use crate::tables::finesine;
use crate::tables::ANG45;
use crate::tables::ANGLETOFINESHIFT;

pub use crate::d_ticcmd::ticcmd_t;
#[derive(Copy, Clone)]
pub enum StateAction {
    None,
    Mobj(fn(&mut GameState, MobjId)),
    Weapon(fn(&mut GameState, PlayerId, i32)),
}
#[derive(Copy, Clone)]
pub enum ThinkerFn {
    Paused,
    Removed,
    Unresolved,
    Mobj(fn(&mut GameState, MobjId)),
    Ceiling(fn(&mut GameState, CeilingId)),
    Door(fn(&mut GameState, DoorId)),
    Floor(fn(&mut GameState, FloorId)),
    Plat(fn(&mut GameState, PlatId)),
    FireFlicker(fn(&mut GameState, FireFlickerId)),
    LightFlash(fn(&mut GameState, LightFlashId)),
    Strobe(fn(&mut GameState, StrobeId)),
    Glow(fn(&mut GameState, GlowId)),
}
#[derive(Copy, Clone)]
pub enum SectorSpecial {
    Door(ThinkerId),
    Ceiling(ThinkerId),
    Floor(ThinkerId),
    Plat(ThinkerId),
}
#[derive(Copy, Clone)]
pub struct thinker_s {
    pub function: ThinkerFn,
}
pub type thinker_t = thinker_s;
#[derive(Copy, Clone)]
pub struct mapthing_t {
    pub x: i16,
    pub y: i16,
    pub angle: i16,
    pub type_0: i16,
    pub options: i16,
}
pub const NUMSPRITES: i32 = 138;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SpriteNum {
    SPR_TROO = 0,
    SPR_SHTG = 1,
    SPR_PUNG = 2,
    SPR_PISG = 3,
    SPR_PISF = 4,
    SPR_SHTF = 5,
    SPR_SHT2 = 6,
    SPR_CHGG = 7,
    SPR_CHGF = 8,
    SPR_MISG = 9,
    SPR_MISF = 10,
    SPR_SAWG = 11,
    SPR_PLSG = 12,
    SPR_PLSF = 13,
    SPR_BFGG = 14,
    SPR_BFGF = 15,
    SPR_BLUD = 16,
    SPR_PUFF = 17,
    SPR_BAL1 = 18,
    SPR_BAL2 = 19,
    SPR_PLSS = 20,
    SPR_PLSE = 21,
    SPR_MISL = 22,
    SPR_BFS1 = 23,
    SPR_BFE1 = 24,
    SPR_BFE2 = 25,
    SPR_TFOG = 26,
    SPR_IFOG = 27,
    SPR_PLAY = 28,
    SPR_POSS = 29,
    SPR_SPOS = 30,
    SPR_VILE = 31,
    SPR_FIRE = 32,
    SPR_FATB = 33,
    SPR_FBXP = 34,
    SPR_SKEL = 35,
    SPR_MANF = 36,
    SPR_FATT = 37,
    SPR_CPOS = 38,
    SPR_SARG = 39,
    SPR_HEAD = 40,
    SPR_BAL7 = 41,
    SPR_BOSS = 42,
    SPR_BOS2 = 43,
    SPR_SKUL = 44,
    SPR_SPID = 45,
    SPR_BSPI = 46,
    SPR_APLS = 47,
    SPR_APBX = 48,
    SPR_CYBR = 49,
    SPR_PAIN = 50,
    SPR_SSWV = 51,
    SPR_KEEN = 52,
    SPR_BBRN = 53,
    SPR_BOSF = 54,
    SPR_ARM1 = 55,
    SPR_ARM2 = 56,
    SPR_BAR1 = 57,
    SPR_BEXP = 58,
    SPR_FCAN = 59,
    SPR_BON1 = 60,
    SPR_BON2 = 61,
    SPR_BKEY = 62,
    SPR_RKEY = 63,
    SPR_YKEY = 64,
    SPR_BSKU = 65,
    SPR_RSKU = 66,
    SPR_YSKU = 67,
    SPR_STIM = 68,
    SPR_MEDI = 69,
    SPR_SOUL = 70,
    SPR_PINV = 71,
    SPR_PSTR = 72,
    SPR_PINS = 73,
    SPR_MEGA = 74,
    SPR_SUIT = 75,
    SPR_PMAP = 76,
    SPR_PVIS = 77,
    SPR_CLIP = 78,
    SPR_AMMO = 79,
    SPR_ROCK = 80,
    SPR_BROK = 81,
    SPR_CELL = 82,
    SPR_CELP = 83,
    SPR_SHEL = 84,
    SPR_SBOX = 85,
    SPR_BPAK = 86,
    SPR_BFUG = 87,
    SPR_MGUN = 88,
    SPR_CSAW = 89,
    SPR_LAUN = 90,
    SPR_PLAS = 91,
    SPR_SHOT = 92,
    SPR_SGN2 = 93,
    SPR_COLU = 94,
    SPR_SMT2 = 95,
    SPR_GOR1 = 96,
    SPR_POL2 = 97,
    SPR_POL5 = 98,
    SPR_POL4 = 99,
    SPR_POL3 = 100,
    SPR_POL1 = 101,
    SPR_POL6 = 102,
    SPR_GOR2 = 103,
    SPR_GOR3 = 104,
    SPR_GOR4 = 105,
    SPR_GOR5 = 106,
    SPR_SMIT = 107,
    SPR_COL1 = 108,
    SPR_COL2 = 109,
    SPR_COL3 = 110,
    SPR_COL4 = 111,
    SPR_CAND = 112,
    SPR_CBRA = 113,
    SPR_COL6 = 114,
    SPR_TRE1 = 115,
    SPR_TRE2 = 116,
    SPR_ELEC = 117,
    SPR_CEYE = 118,
    SPR_FSKU = 119,
    SPR_COL5 = 120,
    SPR_TBLU = 121,
    SPR_TGRN = 122,
    SPR_TRED = 123,
    SPR_SMBT = 124,
    SPR_SMGT = 125,
    SPR_SMRT = 126,
    SPR_HDB1 = 127,
    SPR_HDB2 = 128,
    SPR_HDB3 = 129,
    SPR_HDB4 = 130,
    SPR_HDB5 = 131,
    SPR_HDB6 = 132,
    SPR_POB1 = 133,
    SPR_POB2 = 134,
    SPR_BRS1 = 135,
    SPR_TLMP = 136,
    SPR_TLP2 = 137,
}
pub fn spritenum_from_raw(v: i32) -> SpriteNum {
    match v {
        0 => SpriteNum::SPR_TROO,
        1 => SpriteNum::SPR_SHTG,
        2 => SpriteNum::SPR_PUNG,
        3 => SpriteNum::SPR_PISG,
        4 => SpriteNum::SPR_PISF,
        5 => SpriteNum::SPR_SHTF,
        6 => SpriteNum::SPR_SHT2,
        7 => SpriteNum::SPR_CHGG,
        8 => SpriteNum::SPR_CHGF,
        9 => SpriteNum::SPR_MISG,
        10 => SpriteNum::SPR_MISF,
        11 => SpriteNum::SPR_SAWG,
        12 => SpriteNum::SPR_PLSG,
        13 => SpriteNum::SPR_PLSF,
        14 => SpriteNum::SPR_BFGG,
        15 => SpriteNum::SPR_BFGF,
        16 => SpriteNum::SPR_BLUD,
        17 => SpriteNum::SPR_PUFF,
        18 => SpriteNum::SPR_BAL1,
        19 => SpriteNum::SPR_BAL2,
        20 => SpriteNum::SPR_PLSS,
        21 => SpriteNum::SPR_PLSE,
        22 => SpriteNum::SPR_MISL,
        23 => SpriteNum::SPR_BFS1,
        24 => SpriteNum::SPR_BFE1,
        25 => SpriteNum::SPR_BFE2,
        26 => SpriteNum::SPR_TFOG,
        27 => SpriteNum::SPR_IFOG,
        28 => SpriteNum::SPR_PLAY,
        29 => SpriteNum::SPR_POSS,
        30 => SpriteNum::SPR_SPOS,
        31 => SpriteNum::SPR_VILE,
        32 => SpriteNum::SPR_FIRE,
        33 => SpriteNum::SPR_FATB,
        34 => SpriteNum::SPR_FBXP,
        35 => SpriteNum::SPR_SKEL,
        36 => SpriteNum::SPR_MANF,
        37 => SpriteNum::SPR_FATT,
        38 => SpriteNum::SPR_CPOS,
        39 => SpriteNum::SPR_SARG,
        40 => SpriteNum::SPR_HEAD,
        41 => SpriteNum::SPR_BAL7,
        42 => SpriteNum::SPR_BOSS,
        43 => SpriteNum::SPR_BOS2,
        44 => SpriteNum::SPR_SKUL,
        45 => SpriteNum::SPR_SPID,
        46 => SpriteNum::SPR_BSPI,
        47 => SpriteNum::SPR_APLS,
        48 => SpriteNum::SPR_APBX,
        49 => SpriteNum::SPR_CYBR,
        50 => SpriteNum::SPR_PAIN,
        51 => SpriteNum::SPR_SSWV,
        52 => SpriteNum::SPR_KEEN,
        53 => SpriteNum::SPR_BBRN,
        54 => SpriteNum::SPR_BOSF,
        55 => SpriteNum::SPR_ARM1,
        56 => SpriteNum::SPR_ARM2,
        57 => SpriteNum::SPR_BAR1,
        58 => SpriteNum::SPR_BEXP,
        59 => SpriteNum::SPR_FCAN,
        60 => SpriteNum::SPR_BON1,
        61 => SpriteNum::SPR_BON2,
        62 => SpriteNum::SPR_BKEY,
        63 => SpriteNum::SPR_RKEY,
        64 => SpriteNum::SPR_YKEY,
        65 => SpriteNum::SPR_BSKU,
        66 => SpriteNum::SPR_RSKU,
        67 => SpriteNum::SPR_YSKU,
        68 => SpriteNum::SPR_STIM,
        69 => SpriteNum::SPR_MEDI,
        70 => SpriteNum::SPR_SOUL,
        71 => SpriteNum::SPR_PINV,
        72 => SpriteNum::SPR_PSTR,
        73 => SpriteNum::SPR_PINS,
        74 => SpriteNum::SPR_MEGA,
        75 => SpriteNum::SPR_SUIT,
        76 => SpriteNum::SPR_PMAP,
        77 => SpriteNum::SPR_PVIS,
        78 => SpriteNum::SPR_CLIP,
        79 => SpriteNum::SPR_AMMO,
        80 => SpriteNum::SPR_ROCK,
        81 => SpriteNum::SPR_BROK,
        82 => SpriteNum::SPR_CELL,
        83 => SpriteNum::SPR_CELP,
        84 => SpriteNum::SPR_SHEL,
        85 => SpriteNum::SPR_SBOX,
        86 => SpriteNum::SPR_BPAK,
        87 => SpriteNum::SPR_BFUG,
        88 => SpriteNum::SPR_MGUN,
        89 => SpriteNum::SPR_CSAW,
        90 => SpriteNum::SPR_LAUN,
        91 => SpriteNum::SPR_PLAS,
        92 => SpriteNum::SPR_SHOT,
        93 => SpriteNum::SPR_SGN2,
        94 => SpriteNum::SPR_COLU,
        95 => SpriteNum::SPR_SMT2,
        96 => SpriteNum::SPR_GOR1,
        97 => SpriteNum::SPR_POL2,
        98 => SpriteNum::SPR_POL5,
        99 => SpriteNum::SPR_POL4,
        100 => SpriteNum::SPR_POL3,
        101 => SpriteNum::SPR_POL1,
        102 => SpriteNum::SPR_POL6,
        103 => SpriteNum::SPR_GOR2,
        104 => SpriteNum::SPR_GOR3,
        105 => SpriteNum::SPR_GOR4,
        106 => SpriteNum::SPR_GOR5,
        107 => SpriteNum::SPR_SMIT,
        108 => SpriteNum::SPR_COL1,
        109 => SpriteNum::SPR_COL2,
        110 => SpriteNum::SPR_COL3,
        111 => SpriteNum::SPR_COL4,
        112 => SpriteNum::SPR_CAND,
        113 => SpriteNum::SPR_CBRA,
        114 => SpriteNum::SPR_COL6,
        115 => SpriteNum::SPR_TRE1,
        116 => SpriteNum::SPR_TRE2,
        117 => SpriteNum::SPR_ELEC,
        118 => SpriteNum::SPR_CEYE,
        119 => SpriteNum::SPR_FSKU,
        120 => SpriteNum::SPR_COL5,
        121 => SpriteNum::SPR_TBLU,
        122 => SpriteNum::SPR_TGRN,
        123 => SpriteNum::SPR_TRED,
        124 => SpriteNum::SPR_SMBT,
        125 => SpriteNum::SPR_SMGT,
        126 => SpriteNum::SPR_SMRT,
        127 => SpriteNum::SPR_HDB1,
        128 => SpriteNum::SPR_HDB2,
        129 => SpriteNum::SPR_HDB3,
        130 => SpriteNum::SPR_HDB4,
        131 => SpriteNum::SPR_HDB5,
        132 => SpriteNum::SPR_HDB6,
        133 => SpriteNum::SPR_POB1,
        134 => SpriteNum::SPR_POB2,
        135 => SpriteNum::SPR_BRS1,
        136 => SpriteNum::SPR_TLMP,
        137 => SpriteNum::SPR_TLP2,
        n => panic!("invalid spritenum {n}"),
    }
}
pub const NUMSTATES: i32 = 967;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StateNum {
    S_NULL = 0,
    S_LIGHTDONE = 1,
    S_PUNCH = 2,
    S_PUNCHDOWN = 3,
    S_PUNCHUP = 4,
    S_PUNCH1 = 5,
    S_PUNCH2 = 6,
    S_PUNCH3 = 7,
    S_PUNCH4 = 8,
    S_PUNCH5 = 9,
    S_PISTOL = 10,
    S_PISTOLDOWN = 11,
    S_PISTOLUP = 12,
    S_PISTOL1 = 13,
    S_PISTOL2 = 14,
    S_PISTOL3 = 15,
    S_PISTOL4 = 16,
    S_PISTOLFLASH = 17,
    S_SGUN = 18,
    S_SGUNDOWN = 19,
    S_SGUNUP = 20,
    S_SGUN1 = 21,
    S_SGUN2 = 22,
    S_SGUN3 = 23,
    S_SGUN4 = 24,
    S_SGUN5 = 25,
    S_SGUN6 = 26,
    S_SGUN7 = 27,
    S_SGUN8 = 28,
    S_SGUN9 = 29,
    S_SGUNFLASH1 = 30,
    S_SGUNFLASH2 = 31,
    S_DSGUN = 32,
    S_DSGUNDOWN = 33,
    S_DSGUNUP = 34,
    S_DSGUN1 = 35,
    S_DSGUN2 = 36,
    S_DSGUN3 = 37,
    S_DSGUN4 = 38,
    S_DSGUN5 = 39,
    S_DSGUN6 = 40,
    S_DSGUN7 = 41,
    S_DSGUN8 = 42,
    S_DSGUN9 = 43,
    S_DSGUN10 = 44,
    S_DSNR1 = 45,
    S_DSNR2 = 46,
    S_DSGUNFLASH1 = 47,
    S_DSGUNFLASH2 = 48,
    S_CHAIN = 49,
    S_CHAINDOWN = 50,
    S_CHAINUP = 51,
    S_CHAIN1 = 52,
    S_CHAIN2 = 53,
    S_CHAIN3 = 54,
    S_CHAINFLASH1 = 55,
    S_CHAINFLASH2 = 56,
    S_MISSILE = 57,
    S_MISSILEDOWN = 58,
    S_MISSILEUP = 59,
    S_MISSILE1 = 60,
    S_MISSILE2 = 61,
    S_MISSILE3 = 62,
    S_MISSILEFLASH1 = 63,
    S_MISSILEFLASH2 = 64,
    S_MISSILEFLASH3 = 65,
    S_MISSILEFLASH4 = 66,
    S_SAW = 67,
    S_SAWB = 68,
    S_SAWDOWN = 69,
    S_SAWUP = 70,
    S_SAW1 = 71,
    S_SAW2 = 72,
    S_SAW3 = 73,
    S_PLASMA = 74,
    S_PLASMADOWN = 75,
    S_PLASMAUP = 76,
    S_PLASMA1 = 77,
    S_PLASMA2 = 78,
    S_PLASMAFLASH1 = 79,
    S_PLASMAFLASH2 = 80,
    S_BFG = 81,
    S_BFGDOWN = 82,
    S_BFGUP = 83,
    S_BFG1 = 84,
    S_BFG2 = 85,
    S_BFG3 = 86,
    S_BFG4 = 87,
    S_BFGFLASH1 = 88,
    S_BFGFLASH2 = 89,
    S_BLOOD1 = 90,
    S_BLOOD2 = 91,
    S_BLOOD3 = 92,
    S_PUFF1 = 93,
    S_PUFF2 = 94,
    S_PUFF3 = 95,
    S_PUFF4 = 96,
    S_TBALL1 = 97,
    S_TBALL2 = 98,
    S_TBALLX1 = 99,
    S_TBALLX2 = 100,
    S_TBALLX3 = 101,
    S_RBALL1 = 102,
    S_RBALL2 = 103,
    S_RBALLX1 = 104,
    S_RBALLX2 = 105,
    S_RBALLX3 = 106,
    S_PLASBALL = 107,
    S_PLASBALL2 = 108,
    S_PLASEXP = 109,
    S_PLASEXP2 = 110,
    S_PLASEXP3 = 111,
    S_PLASEXP4 = 112,
    S_PLASEXP5 = 113,
    S_ROCKET = 114,
    S_BFGSHOT = 115,
    S_BFGSHOT2 = 116,
    S_BFGLAND = 117,
    S_BFGLAND2 = 118,
    S_BFGLAND3 = 119,
    S_BFGLAND4 = 120,
    S_BFGLAND5 = 121,
    S_BFGLAND6 = 122,
    S_BFGEXP = 123,
    S_BFGEXP2 = 124,
    S_BFGEXP3 = 125,
    S_BFGEXP4 = 126,
    S_EXPLODE1 = 127,
    S_EXPLODE2 = 128,
    S_EXPLODE3 = 129,
    S_TFOG = 130,
    S_TFOG01 = 131,
    S_TFOG02 = 132,
    S_TFOG2 = 133,
    S_TFOG3 = 134,
    S_TFOG4 = 135,
    S_TFOG5 = 136,
    S_TFOG6 = 137,
    S_TFOG7 = 138,
    S_TFOG8 = 139,
    S_TFOG9 = 140,
    S_TFOG10 = 141,
    S_IFOG = 142,
    S_IFOG01 = 143,
    S_IFOG02 = 144,
    S_IFOG2 = 145,
    S_IFOG3 = 146,
    S_IFOG4 = 147,
    S_IFOG5 = 148,
    S_PLAY = 149,
    S_PLAY_RUN1 = 150,
    S_PLAY_RUN2 = 151,
    S_PLAY_RUN3 = 152,
    S_PLAY_RUN4 = 153,
    S_PLAY_ATK1 = 154,
    S_PLAY_ATK2 = 155,
    S_PLAY_PAIN = 156,
    S_PLAY_PAIN2 = 157,
    S_PLAY_DIE1 = 158,
    S_PLAY_DIE2 = 159,
    S_PLAY_DIE3 = 160,
    S_PLAY_DIE4 = 161,
    S_PLAY_DIE5 = 162,
    S_PLAY_DIE6 = 163,
    S_PLAY_DIE7 = 164,
    S_PLAY_XDIE1 = 165,
    S_PLAY_XDIE2 = 166,
    S_PLAY_XDIE3 = 167,
    S_PLAY_XDIE4 = 168,
    S_PLAY_XDIE5 = 169,
    S_PLAY_XDIE6 = 170,
    S_PLAY_XDIE7 = 171,
    S_PLAY_XDIE8 = 172,
    S_PLAY_XDIE9 = 173,
    S_POSS_STND = 174,
    S_POSS_STND2 = 175,
    S_POSS_RUN1 = 176,
    S_POSS_RUN2 = 177,
    S_POSS_RUN3 = 178,
    S_POSS_RUN4 = 179,
    S_POSS_RUN5 = 180,
    S_POSS_RUN6 = 181,
    S_POSS_RUN7 = 182,
    S_POSS_RUN8 = 183,
    S_POSS_ATK1 = 184,
    S_POSS_ATK2 = 185,
    S_POSS_ATK3 = 186,
    S_POSS_PAIN = 187,
    S_POSS_PAIN2 = 188,
    S_POSS_DIE1 = 189,
    S_POSS_DIE2 = 190,
    S_POSS_DIE3 = 191,
    S_POSS_DIE4 = 192,
    S_POSS_DIE5 = 193,
    S_POSS_XDIE1 = 194,
    S_POSS_XDIE2 = 195,
    S_POSS_XDIE3 = 196,
    S_POSS_XDIE4 = 197,
    S_POSS_XDIE5 = 198,
    S_POSS_XDIE6 = 199,
    S_POSS_XDIE7 = 200,
    S_POSS_XDIE8 = 201,
    S_POSS_XDIE9 = 202,
    S_POSS_RAISE1 = 203,
    S_POSS_RAISE2 = 204,
    S_POSS_RAISE3 = 205,
    S_POSS_RAISE4 = 206,
    S_SPOS_STND = 207,
    S_SPOS_STND2 = 208,
    S_SPOS_RUN1 = 209,
    S_SPOS_RUN2 = 210,
    S_SPOS_RUN3 = 211,
    S_SPOS_RUN4 = 212,
    S_SPOS_RUN5 = 213,
    S_SPOS_RUN6 = 214,
    S_SPOS_RUN7 = 215,
    S_SPOS_RUN8 = 216,
    S_SPOS_ATK1 = 217,
    S_SPOS_ATK2 = 218,
    S_SPOS_ATK3 = 219,
    S_SPOS_PAIN = 220,
    S_SPOS_PAIN2 = 221,
    S_SPOS_DIE1 = 222,
    S_SPOS_DIE2 = 223,
    S_SPOS_DIE3 = 224,
    S_SPOS_DIE4 = 225,
    S_SPOS_DIE5 = 226,
    S_SPOS_XDIE1 = 227,
    S_SPOS_XDIE2 = 228,
    S_SPOS_XDIE3 = 229,
    S_SPOS_XDIE4 = 230,
    S_SPOS_XDIE5 = 231,
    S_SPOS_XDIE6 = 232,
    S_SPOS_XDIE7 = 233,
    S_SPOS_XDIE8 = 234,
    S_SPOS_XDIE9 = 235,
    S_SPOS_RAISE1 = 236,
    S_SPOS_RAISE2 = 237,
    S_SPOS_RAISE3 = 238,
    S_SPOS_RAISE4 = 239,
    S_SPOS_RAISE5 = 240,
    S_VILE_STND = 241,
    S_VILE_STND2 = 242,
    S_VILE_RUN1 = 243,
    S_VILE_RUN2 = 244,
    S_VILE_RUN3 = 245,
    S_VILE_RUN4 = 246,
    S_VILE_RUN5 = 247,
    S_VILE_RUN6 = 248,
    S_VILE_RUN7 = 249,
    S_VILE_RUN8 = 250,
    S_VILE_RUN9 = 251,
    S_VILE_RUN10 = 252,
    S_VILE_RUN11 = 253,
    S_VILE_RUN12 = 254,
    S_VILE_ATK1 = 255,
    S_VILE_ATK2 = 256,
    S_VILE_ATK3 = 257,
    S_VILE_ATK4 = 258,
    S_VILE_ATK5 = 259,
    S_VILE_ATK6 = 260,
    S_VILE_ATK7 = 261,
    S_VILE_ATK8 = 262,
    S_VILE_ATK9 = 263,
    S_VILE_ATK10 = 264,
    S_VILE_ATK11 = 265,
    S_VILE_HEAL1 = 266,
    S_VILE_HEAL2 = 267,
    S_VILE_HEAL3 = 268,
    S_VILE_PAIN = 269,
    S_VILE_PAIN2 = 270,
    S_VILE_DIE1 = 271,
    S_VILE_DIE2 = 272,
    S_VILE_DIE3 = 273,
    S_VILE_DIE4 = 274,
    S_VILE_DIE5 = 275,
    S_VILE_DIE6 = 276,
    S_VILE_DIE7 = 277,
    S_VILE_DIE8 = 278,
    S_VILE_DIE9 = 279,
    S_VILE_DIE10 = 280,
    S_FIRE1 = 281,
    S_FIRE2 = 282,
    S_FIRE3 = 283,
    S_FIRE4 = 284,
    S_FIRE5 = 285,
    S_FIRE6 = 286,
    S_FIRE7 = 287,
    S_FIRE8 = 288,
    S_FIRE9 = 289,
    S_FIRE10 = 290,
    S_FIRE11 = 291,
    S_FIRE12 = 292,
    S_FIRE13 = 293,
    S_FIRE14 = 294,
    S_FIRE15 = 295,
    S_FIRE16 = 296,
    S_FIRE17 = 297,
    S_FIRE18 = 298,
    S_FIRE19 = 299,
    S_FIRE20 = 300,
    S_FIRE21 = 301,
    S_FIRE22 = 302,
    S_FIRE23 = 303,
    S_FIRE24 = 304,
    S_FIRE25 = 305,
    S_FIRE26 = 306,
    S_FIRE27 = 307,
    S_FIRE28 = 308,
    S_FIRE29 = 309,
    S_FIRE30 = 310,
    S_SMOKE1 = 311,
    S_SMOKE2 = 312,
    S_SMOKE3 = 313,
    S_SMOKE4 = 314,
    S_SMOKE5 = 315,
    S_TRACER = 316,
    S_TRACER2 = 317,
    S_TRACEEXP1 = 318,
    S_TRACEEXP2 = 319,
    S_TRACEEXP3 = 320,
    S_SKEL_STND = 321,
    S_SKEL_STND2 = 322,
    S_SKEL_RUN1 = 323,
    S_SKEL_RUN2 = 324,
    S_SKEL_RUN3 = 325,
    S_SKEL_RUN4 = 326,
    S_SKEL_RUN5 = 327,
    S_SKEL_RUN6 = 328,
    S_SKEL_RUN7 = 329,
    S_SKEL_RUN8 = 330,
    S_SKEL_RUN9 = 331,
    S_SKEL_RUN10 = 332,
    S_SKEL_RUN11 = 333,
    S_SKEL_RUN12 = 334,
    S_SKEL_FIST1 = 335,
    S_SKEL_FIST2 = 336,
    S_SKEL_FIST3 = 337,
    S_SKEL_FIST4 = 338,
    S_SKEL_MISS1 = 339,
    S_SKEL_MISS2 = 340,
    S_SKEL_MISS3 = 341,
    S_SKEL_MISS4 = 342,
    S_SKEL_PAIN = 343,
    S_SKEL_PAIN2 = 344,
    S_SKEL_DIE1 = 345,
    S_SKEL_DIE2 = 346,
    S_SKEL_DIE3 = 347,
    S_SKEL_DIE4 = 348,
    S_SKEL_DIE5 = 349,
    S_SKEL_DIE6 = 350,
    S_SKEL_RAISE1 = 351,
    S_SKEL_RAISE2 = 352,
    S_SKEL_RAISE3 = 353,
    S_SKEL_RAISE4 = 354,
    S_SKEL_RAISE5 = 355,
    S_SKEL_RAISE6 = 356,
    S_FATSHOT1 = 357,
    S_FATSHOT2 = 358,
    S_FATSHOTX1 = 359,
    S_FATSHOTX2 = 360,
    S_FATSHOTX3 = 361,
    S_FATT_STND = 362,
    S_FATT_STND2 = 363,
    S_FATT_RUN1 = 364,
    S_FATT_RUN2 = 365,
    S_FATT_RUN3 = 366,
    S_FATT_RUN4 = 367,
    S_FATT_RUN5 = 368,
    S_FATT_RUN6 = 369,
    S_FATT_RUN7 = 370,
    S_FATT_RUN8 = 371,
    S_FATT_RUN9 = 372,
    S_FATT_RUN10 = 373,
    S_FATT_RUN11 = 374,
    S_FATT_RUN12 = 375,
    S_FATT_ATK1 = 376,
    S_FATT_ATK2 = 377,
    S_FATT_ATK3 = 378,
    S_FATT_ATK4 = 379,
    S_FATT_ATK5 = 380,
    S_FATT_ATK6 = 381,
    S_FATT_ATK7 = 382,
    S_FATT_ATK8 = 383,
    S_FATT_ATK9 = 384,
    S_FATT_ATK10 = 385,
    S_FATT_PAIN = 386,
    S_FATT_PAIN2 = 387,
    S_FATT_DIE1 = 388,
    S_FATT_DIE2 = 389,
    S_FATT_DIE3 = 390,
    S_FATT_DIE4 = 391,
    S_FATT_DIE5 = 392,
    S_FATT_DIE6 = 393,
    S_FATT_DIE7 = 394,
    S_FATT_DIE8 = 395,
    S_FATT_DIE9 = 396,
    S_FATT_DIE10 = 397,
    S_FATT_RAISE1 = 398,
    S_FATT_RAISE2 = 399,
    S_FATT_RAISE3 = 400,
    S_FATT_RAISE4 = 401,
    S_FATT_RAISE5 = 402,
    S_FATT_RAISE6 = 403,
    S_FATT_RAISE7 = 404,
    S_FATT_RAISE8 = 405,
    S_CPOS_STND = 406,
    S_CPOS_STND2 = 407,
    S_CPOS_RUN1 = 408,
    S_CPOS_RUN2 = 409,
    S_CPOS_RUN3 = 410,
    S_CPOS_RUN4 = 411,
    S_CPOS_RUN5 = 412,
    S_CPOS_RUN6 = 413,
    S_CPOS_RUN7 = 414,
    S_CPOS_RUN8 = 415,
    S_CPOS_ATK1 = 416,
    S_CPOS_ATK2 = 417,
    S_CPOS_ATK3 = 418,
    S_CPOS_ATK4 = 419,
    S_CPOS_PAIN = 420,
    S_CPOS_PAIN2 = 421,
    S_CPOS_DIE1 = 422,
    S_CPOS_DIE2 = 423,
    S_CPOS_DIE3 = 424,
    S_CPOS_DIE4 = 425,
    S_CPOS_DIE5 = 426,
    S_CPOS_DIE6 = 427,
    S_CPOS_DIE7 = 428,
    S_CPOS_XDIE1 = 429,
    S_CPOS_XDIE2 = 430,
    S_CPOS_XDIE3 = 431,
    S_CPOS_XDIE4 = 432,
    S_CPOS_XDIE5 = 433,
    S_CPOS_XDIE6 = 434,
    S_CPOS_RAISE1 = 435,
    S_CPOS_RAISE2 = 436,
    S_CPOS_RAISE3 = 437,
    S_CPOS_RAISE4 = 438,
    S_CPOS_RAISE5 = 439,
    S_CPOS_RAISE6 = 440,
    S_CPOS_RAISE7 = 441,
    S_TROO_STND = 442,
    S_TROO_STND2 = 443,
    S_TROO_RUN1 = 444,
    S_TROO_RUN2 = 445,
    S_TROO_RUN3 = 446,
    S_TROO_RUN4 = 447,
    S_TROO_RUN5 = 448,
    S_TROO_RUN6 = 449,
    S_TROO_RUN7 = 450,
    S_TROO_RUN8 = 451,
    S_TROO_ATK1 = 452,
    S_TROO_ATK2 = 453,
    S_TROO_ATK3 = 454,
    S_TROO_PAIN = 455,
    S_TROO_PAIN2 = 456,
    S_TROO_DIE1 = 457,
    S_TROO_DIE2 = 458,
    S_TROO_DIE3 = 459,
    S_TROO_DIE4 = 460,
    S_TROO_DIE5 = 461,
    S_TROO_XDIE1 = 462,
    S_TROO_XDIE2 = 463,
    S_TROO_XDIE3 = 464,
    S_TROO_XDIE4 = 465,
    S_TROO_XDIE5 = 466,
    S_TROO_XDIE6 = 467,
    S_TROO_XDIE7 = 468,
    S_TROO_XDIE8 = 469,
    S_TROO_RAISE1 = 470,
    S_TROO_RAISE2 = 471,
    S_TROO_RAISE3 = 472,
    S_TROO_RAISE4 = 473,
    S_TROO_RAISE5 = 474,
    S_SARG_STND = 475,
    S_SARG_STND2 = 476,
    S_SARG_RUN1 = 477,
    S_SARG_RUN2 = 478,
    S_SARG_RUN3 = 479,
    S_SARG_RUN4 = 480,
    S_SARG_RUN5 = 481,
    S_SARG_RUN6 = 482,
    S_SARG_RUN7 = 483,
    S_SARG_RUN8 = 484,
    S_SARG_ATK1 = 485,
    S_SARG_ATK2 = 486,
    S_SARG_ATK3 = 487,
    S_SARG_PAIN = 488,
    S_SARG_PAIN2 = 489,
    S_SARG_DIE1 = 490,
    S_SARG_DIE2 = 491,
    S_SARG_DIE3 = 492,
    S_SARG_DIE4 = 493,
    S_SARG_DIE5 = 494,
    S_SARG_DIE6 = 495,
    S_SARG_RAISE1 = 496,
    S_SARG_RAISE2 = 497,
    S_SARG_RAISE3 = 498,
    S_SARG_RAISE4 = 499,
    S_SARG_RAISE5 = 500,
    S_SARG_RAISE6 = 501,
    S_HEAD_STND = 502,
    S_HEAD_RUN1 = 503,
    S_HEAD_ATK1 = 504,
    S_HEAD_ATK2 = 505,
    S_HEAD_ATK3 = 506,
    S_HEAD_PAIN = 507,
    S_HEAD_PAIN2 = 508,
    S_HEAD_PAIN3 = 509,
    S_HEAD_DIE1 = 510,
    S_HEAD_DIE2 = 511,
    S_HEAD_DIE3 = 512,
    S_HEAD_DIE4 = 513,
    S_HEAD_DIE5 = 514,
    S_HEAD_DIE6 = 515,
    S_HEAD_RAISE1 = 516,
    S_HEAD_RAISE2 = 517,
    S_HEAD_RAISE3 = 518,
    S_HEAD_RAISE4 = 519,
    S_HEAD_RAISE5 = 520,
    S_HEAD_RAISE6 = 521,
    S_BRBALL1 = 522,
    S_BRBALL2 = 523,
    S_BRBALLX1 = 524,
    S_BRBALLX2 = 525,
    S_BRBALLX3 = 526,
    S_BOSS_STND = 527,
    S_BOSS_STND2 = 528,
    S_BOSS_RUN1 = 529,
    S_BOSS_RUN2 = 530,
    S_BOSS_RUN3 = 531,
    S_BOSS_RUN4 = 532,
    S_BOSS_RUN5 = 533,
    S_BOSS_RUN6 = 534,
    S_BOSS_RUN7 = 535,
    S_BOSS_RUN8 = 536,
    S_BOSS_ATK1 = 537,
    S_BOSS_ATK2 = 538,
    S_BOSS_ATK3 = 539,
    S_BOSS_PAIN = 540,
    S_BOSS_PAIN2 = 541,
    S_BOSS_DIE1 = 542,
    S_BOSS_DIE2 = 543,
    S_BOSS_DIE3 = 544,
    S_BOSS_DIE4 = 545,
    S_BOSS_DIE5 = 546,
    S_BOSS_DIE6 = 547,
    S_BOSS_DIE7 = 548,
    S_BOSS_RAISE1 = 549,
    S_BOSS_RAISE2 = 550,
    S_BOSS_RAISE3 = 551,
    S_BOSS_RAISE4 = 552,
    S_BOSS_RAISE5 = 553,
    S_BOSS_RAISE6 = 554,
    S_BOSS_RAISE7 = 555,
    S_BOS2_STND = 556,
    S_BOS2_STND2 = 557,
    S_BOS2_RUN1 = 558,
    S_BOS2_RUN2 = 559,
    S_BOS2_RUN3 = 560,
    S_BOS2_RUN4 = 561,
    S_BOS2_RUN5 = 562,
    S_BOS2_RUN6 = 563,
    S_BOS2_RUN7 = 564,
    S_BOS2_RUN8 = 565,
    S_BOS2_ATK1 = 566,
    S_BOS2_ATK2 = 567,
    S_BOS2_ATK3 = 568,
    S_BOS2_PAIN = 569,
    S_BOS2_PAIN2 = 570,
    S_BOS2_DIE1 = 571,
    S_BOS2_DIE2 = 572,
    S_BOS2_DIE3 = 573,
    S_BOS2_DIE4 = 574,
    S_BOS2_DIE5 = 575,
    S_BOS2_DIE6 = 576,
    S_BOS2_DIE7 = 577,
    S_BOS2_RAISE1 = 578,
    S_BOS2_RAISE2 = 579,
    S_BOS2_RAISE3 = 580,
    S_BOS2_RAISE4 = 581,
    S_BOS2_RAISE5 = 582,
    S_BOS2_RAISE6 = 583,
    S_BOS2_RAISE7 = 584,
    S_SKULL_STND = 585,
    S_SKULL_STND2 = 586,
    S_SKULL_RUN1 = 587,
    S_SKULL_RUN2 = 588,
    S_SKULL_ATK1 = 589,
    S_SKULL_ATK2 = 590,
    S_SKULL_ATK3 = 591,
    S_SKULL_ATK4 = 592,
    S_SKULL_PAIN = 593,
    S_SKULL_PAIN2 = 594,
    S_SKULL_DIE1 = 595,
    S_SKULL_DIE2 = 596,
    S_SKULL_DIE3 = 597,
    S_SKULL_DIE4 = 598,
    S_SKULL_DIE5 = 599,
    S_SKULL_DIE6 = 600,
    S_SPID_STND = 601,
    S_SPID_STND2 = 602,
    S_SPID_RUN1 = 603,
    S_SPID_RUN2 = 604,
    S_SPID_RUN3 = 605,
    S_SPID_RUN4 = 606,
    S_SPID_RUN5 = 607,
    S_SPID_RUN6 = 608,
    S_SPID_RUN7 = 609,
    S_SPID_RUN8 = 610,
    S_SPID_RUN9 = 611,
    S_SPID_RUN10 = 612,
    S_SPID_RUN11 = 613,
    S_SPID_RUN12 = 614,
    S_SPID_ATK1 = 615,
    S_SPID_ATK2 = 616,
    S_SPID_ATK3 = 617,
    S_SPID_ATK4 = 618,
    S_SPID_PAIN = 619,
    S_SPID_PAIN2 = 620,
    S_SPID_DIE1 = 621,
    S_SPID_DIE2 = 622,
    S_SPID_DIE3 = 623,
    S_SPID_DIE4 = 624,
    S_SPID_DIE5 = 625,
    S_SPID_DIE6 = 626,
    S_SPID_DIE7 = 627,
    S_SPID_DIE8 = 628,
    S_SPID_DIE9 = 629,
    S_SPID_DIE10 = 630,
    S_SPID_DIE11 = 631,
    S_BSPI_STND = 632,
    S_BSPI_STND2 = 633,
    S_BSPI_SIGHT = 634,
    S_BSPI_RUN1 = 635,
    S_BSPI_RUN2 = 636,
    S_BSPI_RUN3 = 637,
    S_BSPI_RUN4 = 638,
    S_BSPI_RUN5 = 639,
    S_BSPI_RUN6 = 640,
    S_BSPI_RUN7 = 641,
    S_BSPI_RUN8 = 642,
    S_BSPI_RUN9 = 643,
    S_BSPI_RUN10 = 644,
    S_BSPI_RUN11 = 645,
    S_BSPI_RUN12 = 646,
    S_BSPI_ATK1 = 647,
    S_BSPI_ATK2 = 648,
    S_BSPI_ATK3 = 649,
    S_BSPI_ATK4 = 650,
    S_BSPI_PAIN = 651,
    S_BSPI_PAIN2 = 652,
    S_BSPI_DIE1 = 653,
    S_BSPI_DIE2 = 654,
    S_BSPI_DIE3 = 655,
    S_BSPI_DIE4 = 656,
    S_BSPI_DIE5 = 657,
    S_BSPI_DIE6 = 658,
    S_BSPI_DIE7 = 659,
    S_BSPI_RAISE1 = 660,
    S_BSPI_RAISE2 = 661,
    S_BSPI_RAISE3 = 662,
    S_BSPI_RAISE4 = 663,
    S_BSPI_RAISE5 = 664,
    S_BSPI_RAISE6 = 665,
    S_BSPI_RAISE7 = 666,
    S_ARACH_PLAZ = 667,
    S_ARACH_PLAZ2 = 668,
    S_ARACH_PLEX = 669,
    S_ARACH_PLEX2 = 670,
    S_ARACH_PLEX3 = 671,
    S_ARACH_PLEX4 = 672,
    S_ARACH_PLEX5 = 673,
    S_CYBER_STND = 674,
    S_CYBER_STND2 = 675,
    S_CYBER_RUN1 = 676,
    S_CYBER_RUN2 = 677,
    S_CYBER_RUN3 = 678,
    S_CYBER_RUN4 = 679,
    S_CYBER_RUN5 = 680,
    S_CYBER_RUN6 = 681,
    S_CYBER_RUN7 = 682,
    S_CYBER_RUN8 = 683,
    S_CYBER_ATK1 = 684,
    S_CYBER_ATK2 = 685,
    S_CYBER_ATK3 = 686,
    S_CYBER_ATK4 = 687,
    S_CYBER_ATK5 = 688,
    S_CYBER_ATK6 = 689,
    S_CYBER_PAIN = 690,
    S_CYBER_DIE1 = 691,
    S_CYBER_DIE2 = 692,
    S_CYBER_DIE3 = 693,
    S_CYBER_DIE4 = 694,
    S_CYBER_DIE5 = 695,
    S_CYBER_DIE6 = 696,
    S_CYBER_DIE7 = 697,
    S_CYBER_DIE8 = 698,
    S_CYBER_DIE9 = 699,
    S_CYBER_DIE10 = 700,
    S_PAIN_STND = 701,
    S_PAIN_RUN1 = 702,
    S_PAIN_RUN2 = 703,
    S_PAIN_RUN3 = 704,
    S_PAIN_RUN4 = 705,
    S_PAIN_RUN5 = 706,
    S_PAIN_RUN6 = 707,
    S_PAIN_ATK1 = 708,
    S_PAIN_ATK2 = 709,
    S_PAIN_ATK3 = 710,
    S_PAIN_ATK4 = 711,
    S_PAIN_PAIN = 712,
    S_PAIN_PAIN2 = 713,
    S_PAIN_DIE1 = 714,
    S_PAIN_DIE2 = 715,
    S_PAIN_DIE3 = 716,
    S_PAIN_DIE4 = 717,
    S_PAIN_DIE5 = 718,
    S_PAIN_DIE6 = 719,
    S_PAIN_RAISE1 = 720,
    S_PAIN_RAISE2 = 721,
    S_PAIN_RAISE3 = 722,
    S_PAIN_RAISE4 = 723,
    S_PAIN_RAISE5 = 724,
    S_PAIN_RAISE6 = 725,
    S_SSWV_STND = 726,
    S_SSWV_STND2 = 727,
    S_SSWV_RUN1 = 728,
    S_SSWV_RUN2 = 729,
    S_SSWV_RUN3 = 730,
    S_SSWV_RUN4 = 731,
    S_SSWV_RUN5 = 732,
    S_SSWV_RUN6 = 733,
    S_SSWV_RUN7 = 734,
    S_SSWV_RUN8 = 735,
    S_SSWV_ATK1 = 736,
    S_SSWV_ATK2 = 737,
    S_SSWV_ATK3 = 738,
    S_SSWV_ATK4 = 739,
    S_SSWV_ATK5 = 740,
    S_SSWV_ATK6 = 741,
    S_SSWV_PAIN = 742,
    S_SSWV_PAIN2 = 743,
    S_SSWV_DIE1 = 744,
    S_SSWV_DIE2 = 745,
    S_SSWV_DIE3 = 746,
    S_SSWV_DIE4 = 747,
    S_SSWV_DIE5 = 748,
    S_SSWV_XDIE1 = 749,
    S_SSWV_XDIE2 = 750,
    S_SSWV_XDIE3 = 751,
    S_SSWV_XDIE4 = 752,
    S_SSWV_XDIE5 = 753,
    S_SSWV_XDIE6 = 754,
    S_SSWV_XDIE7 = 755,
    S_SSWV_XDIE8 = 756,
    S_SSWV_XDIE9 = 757,
    S_SSWV_RAISE1 = 758,
    S_SSWV_RAISE2 = 759,
    S_SSWV_RAISE3 = 760,
    S_SSWV_RAISE4 = 761,
    S_SSWV_RAISE5 = 762,
    S_KEENSTND = 763,
    S_COMMKEEN = 764,
    S_COMMKEEN2 = 765,
    S_COMMKEEN3 = 766,
    S_COMMKEEN4 = 767,
    S_COMMKEEN5 = 768,
    S_COMMKEEN6 = 769,
    S_COMMKEEN7 = 770,
    S_COMMKEEN8 = 771,
    S_COMMKEEN9 = 772,
    S_COMMKEEN10 = 773,
    S_COMMKEEN11 = 774,
    S_COMMKEEN12 = 775,
    S_KEENPAIN = 776,
    S_KEENPAIN2 = 777,
    S_BRAIN = 778,
    S_BRAIN_PAIN = 779,
    S_BRAIN_DIE1 = 780,
    S_BRAIN_DIE2 = 781,
    S_BRAIN_DIE3 = 782,
    S_BRAIN_DIE4 = 783,
    S_BRAINEYE = 784,
    S_BRAINEYESEE = 785,
    S_BRAINEYE1 = 786,
    S_SPAWN1 = 787,
    S_SPAWN2 = 788,
    S_SPAWN3 = 789,
    S_SPAWN4 = 790,
    S_SPAWNFIRE1 = 791,
    S_SPAWNFIRE2 = 792,
    S_SPAWNFIRE3 = 793,
    S_SPAWNFIRE4 = 794,
    S_SPAWNFIRE5 = 795,
    S_SPAWNFIRE6 = 796,
    S_SPAWNFIRE7 = 797,
    S_SPAWNFIRE8 = 798,
    S_BRAINEXPLODE1 = 799,
    S_BRAINEXPLODE2 = 800,
    S_BRAINEXPLODE3 = 801,
    S_ARM1 = 802,
    S_ARM1A = 803,
    S_ARM2 = 804,
    S_ARM2A = 805,
    S_BAR1 = 806,
    S_BAR2 = 807,
    S_BEXP = 808,
    S_BEXP2 = 809,
    S_BEXP3 = 810,
    S_BEXP4 = 811,
    S_BEXP5 = 812,
    S_BBAR1 = 813,
    S_BBAR2 = 814,
    S_BBAR3 = 815,
    S_BON1 = 816,
    S_BON1A = 817,
    S_BON1B = 818,
    S_BON1C = 819,
    S_BON1D = 820,
    S_BON1E = 821,
    S_BON2 = 822,
    S_BON2A = 823,
    S_BON2B = 824,
    S_BON2C = 825,
    S_BON2D = 826,
    S_BON2E = 827,
    S_BKEY = 828,
    S_BKEY2 = 829,
    S_RKEY = 830,
    S_RKEY2 = 831,
    S_YKEY = 832,
    S_YKEY2 = 833,
    S_BSKULL = 834,
    S_BSKULL2 = 835,
    S_RSKULL = 836,
    S_RSKULL2 = 837,
    S_YSKULL = 838,
    S_YSKULL2 = 839,
    S_STIM = 840,
    S_MEDI = 841,
    S_SOUL = 842,
    S_SOUL2 = 843,
    S_SOUL3 = 844,
    S_SOUL4 = 845,
    S_SOUL5 = 846,
    S_SOUL6 = 847,
    S_PINV = 848,
    S_PINV2 = 849,
    S_PINV3 = 850,
    S_PINV4 = 851,
    S_PSTR = 852,
    S_PINS = 853,
    S_PINS2 = 854,
    S_PINS3 = 855,
    S_PINS4 = 856,
    S_MEGA = 857,
    S_MEGA2 = 858,
    S_MEGA3 = 859,
    S_MEGA4 = 860,
    S_SUIT = 861,
    S_PMAP = 862,
    S_PMAP2 = 863,
    S_PMAP3 = 864,
    S_PMAP4 = 865,
    S_PMAP5 = 866,
    S_PMAP6 = 867,
    S_PVIS = 868,
    S_PVIS2 = 869,
    S_CLIP = 870,
    S_AMMO = 871,
    S_ROCK = 872,
    S_BROK = 873,
    S_CELL = 874,
    S_CELP = 875,
    S_SHEL = 876,
    S_SBOX = 877,
    S_BPAK = 878,
    S_BFUG = 879,
    S_MGUN = 880,
    S_CSAW = 881,
    S_LAUN = 882,
    S_PLAS = 883,
    S_SHOT = 884,
    S_SHOT2 = 885,
    S_COLU = 886,
    S_STALAG = 887,
    S_BLOODYTWITCH = 888,
    S_BLOODYTWITCH2 = 889,
    S_BLOODYTWITCH3 = 890,
    S_BLOODYTWITCH4 = 891,
    S_DEADTORSO = 892,
    S_DEADBOTTOM = 893,
    S_HEADSONSTICK = 894,
    S_GIBS = 895,
    S_HEADONASTICK = 896,
    S_HEADCANDLES = 897,
    S_HEADCANDLES2 = 898,
    S_DEADSTICK = 899,
    S_LIVESTICK = 900,
    S_LIVESTICK2 = 901,
    S_MEAT2 = 902,
    S_MEAT3 = 903,
    S_MEAT4 = 904,
    S_MEAT5 = 905,
    S_STALAGTITE = 906,
    S_TALLGRNCOL = 907,
    S_SHRTGRNCOL = 908,
    S_TALLREDCOL = 909,
    S_SHRTREDCOL = 910,
    S_CANDLESTIK = 911,
    S_CANDELABRA = 912,
    S_SKULLCOL = 913,
    S_TORCHTREE = 914,
    S_BIGTREE = 915,
    S_TECHPILLAR = 916,
    S_EVILEYE = 917,
    S_EVILEYE2 = 918,
    S_EVILEYE3 = 919,
    S_EVILEYE4 = 920,
    S_FLOATSKULL = 921,
    S_FLOATSKULL2 = 922,
    S_FLOATSKULL3 = 923,
    S_HEARTCOL = 924,
    S_HEARTCOL2 = 925,
    S_BLUETORCH = 926,
    S_BLUETORCH2 = 927,
    S_BLUETORCH3 = 928,
    S_BLUETORCH4 = 929,
    S_GREENTORCH = 930,
    S_GREENTORCH2 = 931,
    S_GREENTORCH3 = 932,
    S_GREENTORCH4 = 933,
    S_REDTORCH = 934,
    S_REDTORCH2 = 935,
    S_REDTORCH3 = 936,
    S_REDTORCH4 = 937,
    S_BTORCHSHRT = 938,
    S_BTORCHSHRT2 = 939,
    S_BTORCHSHRT3 = 940,
    S_BTORCHSHRT4 = 941,
    S_GTORCHSHRT = 942,
    S_GTORCHSHRT2 = 943,
    S_GTORCHSHRT3 = 944,
    S_GTORCHSHRT4 = 945,
    S_RTORCHSHRT = 946,
    S_RTORCHSHRT2 = 947,
    S_RTORCHSHRT3 = 948,
    S_RTORCHSHRT4 = 949,
    S_HANGNOGUTS = 950,
    S_HANGBNOBRAIN = 951,
    S_HANGTLOOKDN = 952,
    S_HANGTSKULL = 953,
    S_HANGTLOOKUP = 954,
    S_HANGTNOBRAIN = 955,
    S_COLONGIBS = 956,
    S_SMALLPOOL = 957,
    S_BRAINSTEM = 958,
    S_TECHLAMP = 959,
    S_TECHLAMP2 = 960,
    S_TECHLAMP3 = 961,
    S_TECHLAMP4 = 962,
    S_TECH2LAMP = 963,
    S_TECH2LAMP2 = 964,
    S_TECH2LAMP3 = 965,
    S_TECH2LAMP4 = 966,
}
pub fn statenum_from_raw(v: i32) -> StateNum {
    match v {
        0 => StateNum::S_NULL,
        1 => StateNum::S_LIGHTDONE,
        2 => StateNum::S_PUNCH,
        3 => StateNum::S_PUNCHDOWN,
        4 => StateNum::S_PUNCHUP,
        5 => StateNum::S_PUNCH1,
        6 => StateNum::S_PUNCH2,
        7 => StateNum::S_PUNCH3,
        8 => StateNum::S_PUNCH4,
        9 => StateNum::S_PUNCH5,
        10 => StateNum::S_PISTOL,
        11 => StateNum::S_PISTOLDOWN,
        12 => StateNum::S_PISTOLUP,
        13 => StateNum::S_PISTOL1,
        14 => StateNum::S_PISTOL2,
        15 => StateNum::S_PISTOL3,
        16 => StateNum::S_PISTOL4,
        17 => StateNum::S_PISTOLFLASH,
        18 => StateNum::S_SGUN,
        19 => StateNum::S_SGUNDOWN,
        20 => StateNum::S_SGUNUP,
        21 => StateNum::S_SGUN1,
        22 => StateNum::S_SGUN2,
        23 => StateNum::S_SGUN3,
        24 => StateNum::S_SGUN4,
        25 => StateNum::S_SGUN5,
        26 => StateNum::S_SGUN6,
        27 => StateNum::S_SGUN7,
        28 => StateNum::S_SGUN8,
        29 => StateNum::S_SGUN9,
        30 => StateNum::S_SGUNFLASH1,
        31 => StateNum::S_SGUNFLASH2,
        32 => StateNum::S_DSGUN,
        33 => StateNum::S_DSGUNDOWN,
        34 => StateNum::S_DSGUNUP,
        35 => StateNum::S_DSGUN1,
        36 => StateNum::S_DSGUN2,
        37 => StateNum::S_DSGUN3,
        38 => StateNum::S_DSGUN4,
        39 => StateNum::S_DSGUN5,
        40 => StateNum::S_DSGUN6,
        41 => StateNum::S_DSGUN7,
        42 => StateNum::S_DSGUN8,
        43 => StateNum::S_DSGUN9,
        44 => StateNum::S_DSGUN10,
        45 => StateNum::S_DSNR1,
        46 => StateNum::S_DSNR2,
        47 => StateNum::S_DSGUNFLASH1,
        48 => StateNum::S_DSGUNFLASH2,
        49 => StateNum::S_CHAIN,
        50 => StateNum::S_CHAINDOWN,
        51 => StateNum::S_CHAINUP,
        52 => StateNum::S_CHAIN1,
        53 => StateNum::S_CHAIN2,
        54 => StateNum::S_CHAIN3,
        55 => StateNum::S_CHAINFLASH1,
        56 => StateNum::S_CHAINFLASH2,
        57 => StateNum::S_MISSILE,
        58 => StateNum::S_MISSILEDOWN,
        59 => StateNum::S_MISSILEUP,
        60 => StateNum::S_MISSILE1,
        61 => StateNum::S_MISSILE2,
        62 => StateNum::S_MISSILE3,
        63 => StateNum::S_MISSILEFLASH1,
        64 => StateNum::S_MISSILEFLASH2,
        65 => StateNum::S_MISSILEFLASH3,
        66 => StateNum::S_MISSILEFLASH4,
        67 => StateNum::S_SAW,
        68 => StateNum::S_SAWB,
        69 => StateNum::S_SAWDOWN,
        70 => StateNum::S_SAWUP,
        71 => StateNum::S_SAW1,
        72 => StateNum::S_SAW2,
        73 => StateNum::S_SAW3,
        74 => StateNum::S_PLASMA,
        75 => StateNum::S_PLASMADOWN,
        76 => StateNum::S_PLASMAUP,
        77 => StateNum::S_PLASMA1,
        78 => StateNum::S_PLASMA2,
        79 => StateNum::S_PLASMAFLASH1,
        80 => StateNum::S_PLASMAFLASH2,
        81 => StateNum::S_BFG,
        82 => StateNum::S_BFGDOWN,
        83 => StateNum::S_BFGUP,
        84 => StateNum::S_BFG1,
        85 => StateNum::S_BFG2,
        86 => StateNum::S_BFG3,
        87 => StateNum::S_BFG4,
        88 => StateNum::S_BFGFLASH1,
        89 => StateNum::S_BFGFLASH2,
        90 => StateNum::S_BLOOD1,
        91 => StateNum::S_BLOOD2,
        92 => StateNum::S_BLOOD3,
        93 => StateNum::S_PUFF1,
        94 => StateNum::S_PUFF2,
        95 => StateNum::S_PUFF3,
        96 => StateNum::S_PUFF4,
        97 => StateNum::S_TBALL1,
        98 => StateNum::S_TBALL2,
        99 => StateNum::S_TBALLX1,
        100 => StateNum::S_TBALLX2,
        101 => StateNum::S_TBALLX3,
        102 => StateNum::S_RBALL1,
        103 => StateNum::S_RBALL2,
        104 => StateNum::S_RBALLX1,
        105 => StateNum::S_RBALLX2,
        106 => StateNum::S_RBALLX3,
        107 => StateNum::S_PLASBALL,
        108 => StateNum::S_PLASBALL2,
        109 => StateNum::S_PLASEXP,
        110 => StateNum::S_PLASEXP2,
        111 => StateNum::S_PLASEXP3,
        112 => StateNum::S_PLASEXP4,
        113 => StateNum::S_PLASEXP5,
        114 => StateNum::S_ROCKET,
        115 => StateNum::S_BFGSHOT,
        116 => StateNum::S_BFGSHOT2,
        117 => StateNum::S_BFGLAND,
        118 => StateNum::S_BFGLAND2,
        119 => StateNum::S_BFGLAND3,
        120 => StateNum::S_BFGLAND4,
        121 => StateNum::S_BFGLAND5,
        122 => StateNum::S_BFGLAND6,
        123 => StateNum::S_BFGEXP,
        124 => StateNum::S_BFGEXP2,
        125 => StateNum::S_BFGEXP3,
        126 => StateNum::S_BFGEXP4,
        127 => StateNum::S_EXPLODE1,
        128 => StateNum::S_EXPLODE2,
        129 => StateNum::S_EXPLODE3,
        130 => StateNum::S_TFOG,
        131 => StateNum::S_TFOG01,
        132 => StateNum::S_TFOG02,
        133 => StateNum::S_TFOG2,
        134 => StateNum::S_TFOG3,
        135 => StateNum::S_TFOG4,
        136 => StateNum::S_TFOG5,
        137 => StateNum::S_TFOG6,
        138 => StateNum::S_TFOG7,
        139 => StateNum::S_TFOG8,
        140 => StateNum::S_TFOG9,
        141 => StateNum::S_TFOG10,
        142 => StateNum::S_IFOG,
        143 => StateNum::S_IFOG01,
        144 => StateNum::S_IFOG02,
        145 => StateNum::S_IFOG2,
        146 => StateNum::S_IFOG3,
        147 => StateNum::S_IFOG4,
        148 => StateNum::S_IFOG5,
        149 => StateNum::S_PLAY,
        150 => StateNum::S_PLAY_RUN1,
        151 => StateNum::S_PLAY_RUN2,
        152 => StateNum::S_PLAY_RUN3,
        153 => StateNum::S_PLAY_RUN4,
        154 => StateNum::S_PLAY_ATK1,
        155 => StateNum::S_PLAY_ATK2,
        156 => StateNum::S_PLAY_PAIN,
        157 => StateNum::S_PLAY_PAIN2,
        158 => StateNum::S_PLAY_DIE1,
        159 => StateNum::S_PLAY_DIE2,
        160 => StateNum::S_PLAY_DIE3,
        161 => StateNum::S_PLAY_DIE4,
        162 => StateNum::S_PLAY_DIE5,
        163 => StateNum::S_PLAY_DIE6,
        164 => StateNum::S_PLAY_DIE7,
        165 => StateNum::S_PLAY_XDIE1,
        166 => StateNum::S_PLAY_XDIE2,
        167 => StateNum::S_PLAY_XDIE3,
        168 => StateNum::S_PLAY_XDIE4,
        169 => StateNum::S_PLAY_XDIE5,
        170 => StateNum::S_PLAY_XDIE6,
        171 => StateNum::S_PLAY_XDIE7,
        172 => StateNum::S_PLAY_XDIE8,
        173 => StateNum::S_PLAY_XDIE9,
        174 => StateNum::S_POSS_STND,
        175 => StateNum::S_POSS_STND2,
        176 => StateNum::S_POSS_RUN1,
        177 => StateNum::S_POSS_RUN2,
        178 => StateNum::S_POSS_RUN3,
        179 => StateNum::S_POSS_RUN4,
        180 => StateNum::S_POSS_RUN5,
        181 => StateNum::S_POSS_RUN6,
        182 => StateNum::S_POSS_RUN7,
        183 => StateNum::S_POSS_RUN8,
        184 => StateNum::S_POSS_ATK1,
        185 => StateNum::S_POSS_ATK2,
        186 => StateNum::S_POSS_ATK3,
        187 => StateNum::S_POSS_PAIN,
        188 => StateNum::S_POSS_PAIN2,
        189 => StateNum::S_POSS_DIE1,
        190 => StateNum::S_POSS_DIE2,
        191 => StateNum::S_POSS_DIE3,
        192 => StateNum::S_POSS_DIE4,
        193 => StateNum::S_POSS_DIE5,
        194 => StateNum::S_POSS_XDIE1,
        195 => StateNum::S_POSS_XDIE2,
        196 => StateNum::S_POSS_XDIE3,
        197 => StateNum::S_POSS_XDIE4,
        198 => StateNum::S_POSS_XDIE5,
        199 => StateNum::S_POSS_XDIE6,
        200 => StateNum::S_POSS_XDIE7,
        201 => StateNum::S_POSS_XDIE8,
        202 => StateNum::S_POSS_XDIE9,
        203 => StateNum::S_POSS_RAISE1,
        204 => StateNum::S_POSS_RAISE2,
        205 => StateNum::S_POSS_RAISE3,
        206 => StateNum::S_POSS_RAISE4,
        207 => StateNum::S_SPOS_STND,
        208 => StateNum::S_SPOS_STND2,
        209 => StateNum::S_SPOS_RUN1,
        210 => StateNum::S_SPOS_RUN2,
        211 => StateNum::S_SPOS_RUN3,
        212 => StateNum::S_SPOS_RUN4,
        213 => StateNum::S_SPOS_RUN5,
        214 => StateNum::S_SPOS_RUN6,
        215 => StateNum::S_SPOS_RUN7,
        216 => StateNum::S_SPOS_RUN8,
        217 => StateNum::S_SPOS_ATK1,
        218 => StateNum::S_SPOS_ATK2,
        219 => StateNum::S_SPOS_ATK3,
        220 => StateNum::S_SPOS_PAIN,
        221 => StateNum::S_SPOS_PAIN2,
        222 => StateNum::S_SPOS_DIE1,
        223 => StateNum::S_SPOS_DIE2,
        224 => StateNum::S_SPOS_DIE3,
        225 => StateNum::S_SPOS_DIE4,
        226 => StateNum::S_SPOS_DIE5,
        227 => StateNum::S_SPOS_XDIE1,
        228 => StateNum::S_SPOS_XDIE2,
        229 => StateNum::S_SPOS_XDIE3,
        230 => StateNum::S_SPOS_XDIE4,
        231 => StateNum::S_SPOS_XDIE5,
        232 => StateNum::S_SPOS_XDIE6,
        233 => StateNum::S_SPOS_XDIE7,
        234 => StateNum::S_SPOS_XDIE8,
        235 => StateNum::S_SPOS_XDIE9,
        236 => StateNum::S_SPOS_RAISE1,
        237 => StateNum::S_SPOS_RAISE2,
        238 => StateNum::S_SPOS_RAISE3,
        239 => StateNum::S_SPOS_RAISE4,
        240 => StateNum::S_SPOS_RAISE5,
        241 => StateNum::S_VILE_STND,
        242 => StateNum::S_VILE_STND2,
        243 => StateNum::S_VILE_RUN1,
        244 => StateNum::S_VILE_RUN2,
        245 => StateNum::S_VILE_RUN3,
        246 => StateNum::S_VILE_RUN4,
        247 => StateNum::S_VILE_RUN5,
        248 => StateNum::S_VILE_RUN6,
        249 => StateNum::S_VILE_RUN7,
        250 => StateNum::S_VILE_RUN8,
        251 => StateNum::S_VILE_RUN9,
        252 => StateNum::S_VILE_RUN10,
        253 => StateNum::S_VILE_RUN11,
        254 => StateNum::S_VILE_RUN12,
        255 => StateNum::S_VILE_ATK1,
        256 => StateNum::S_VILE_ATK2,
        257 => StateNum::S_VILE_ATK3,
        258 => StateNum::S_VILE_ATK4,
        259 => StateNum::S_VILE_ATK5,
        260 => StateNum::S_VILE_ATK6,
        261 => StateNum::S_VILE_ATK7,
        262 => StateNum::S_VILE_ATK8,
        263 => StateNum::S_VILE_ATK9,
        264 => StateNum::S_VILE_ATK10,
        265 => StateNum::S_VILE_ATK11,
        266 => StateNum::S_VILE_HEAL1,
        267 => StateNum::S_VILE_HEAL2,
        268 => StateNum::S_VILE_HEAL3,
        269 => StateNum::S_VILE_PAIN,
        270 => StateNum::S_VILE_PAIN2,
        271 => StateNum::S_VILE_DIE1,
        272 => StateNum::S_VILE_DIE2,
        273 => StateNum::S_VILE_DIE3,
        274 => StateNum::S_VILE_DIE4,
        275 => StateNum::S_VILE_DIE5,
        276 => StateNum::S_VILE_DIE6,
        277 => StateNum::S_VILE_DIE7,
        278 => StateNum::S_VILE_DIE8,
        279 => StateNum::S_VILE_DIE9,
        280 => StateNum::S_VILE_DIE10,
        281 => StateNum::S_FIRE1,
        282 => StateNum::S_FIRE2,
        283 => StateNum::S_FIRE3,
        284 => StateNum::S_FIRE4,
        285 => StateNum::S_FIRE5,
        286 => StateNum::S_FIRE6,
        287 => StateNum::S_FIRE7,
        288 => StateNum::S_FIRE8,
        289 => StateNum::S_FIRE9,
        290 => StateNum::S_FIRE10,
        291 => StateNum::S_FIRE11,
        292 => StateNum::S_FIRE12,
        293 => StateNum::S_FIRE13,
        294 => StateNum::S_FIRE14,
        295 => StateNum::S_FIRE15,
        296 => StateNum::S_FIRE16,
        297 => StateNum::S_FIRE17,
        298 => StateNum::S_FIRE18,
        299 => StateNum::S_FIRE19,
        300 => StateNum::S_FIRE20,
        301 => StateNum::S_FIRE21,
        302 => StateNum::S_FIRE22,
        303 => StateNum::S_FIRE23,
        304 => StateNum::S_FIRE24,
        305 => StateNum::S_FIRE25,
        306 => StateNum::S_FIRE26,
        307 => StateNum::S_FIRE27,
        308 => StateNum::S_FIRE28,
        309 => StateNum::S_FIRE29,
        310 => StateNum::S_FIRE30,
        311 => StateNum::S_SMOKE1,
        312 => StateNum::S_SMOKE2,
        313 => StateNum::S_SMOKE3,
        314 => StateNum::S_SMOKE4,
        315 => StateNum::S_SMOKE5,
        316 => StateNum::S_TRACER,
        317 => StateNum::S_TRACER2,
        318 => StateNum::S_TRACEEXP1,
        319 => StateNum::S_TRACEEXP2,
        320 => StateNum::S_TRACEEXP3,
        321 => StateNum::S_SKEL_STND,
        322 => StateNum::S_SKEL_STND2,
        323 => StateNum::S_SKEL_RUN1,
        324 => StateNum::S_SKEL_RUN2,
        325 => StateNum::S_SKEL_RUN3,
        326 => StateNum::S_SKEL_RUN4,
        327 => StateNum::S_SKEL_RUN5,
        328 => StateNum::S_SKEL_RUN6,
        329 => StateNum::S_SKEL_RUN7,
        330 => StateNum::S_SKEL_RUN8,
        331 => StateNum::S_SKEL_RUN9,
        332 => StateNum::S_SKEL_RUN10,
        333 => StateNum::S_SKEL_RUN11,
        334 => StateNum::S_SKEL_RUN12,
        335 => StateNum::S_SKEL_FIST1,
        336 => StateNum::S_SKEL_FIST2,
        337 => StateNum::S_SKEL_FIST3,
        338 => StateNum::S_SKEL_FIST4,
        339 => StateNum::S_SKEL_MISS1,
        340 => StateNum::S_SKEL_MISS2,
        341 => StateNum::S_SKEL_MISS3,
        342 => StateNum::S_SKEL_MISS4,
        343 => StateNum::S_SKEL_PAIN,
        344 => StateNum::S_SKEL_PAIN2,
        345 => StateNum::S_SKEL_DIE1,
        346 => StateNum::S_SKEL_DIE2,
        347 => StateNum::S_SKEL_DIE3,
        348 => StateNum::S_SKEL_DIE4,
        349 => StateNum::S_SKEL_DIE5,
        350 => StateNum::S_SKEL_DIE6,
        351 => StateNum::S_SKEL_RAISE1,
        352 => StateNum::S_SKEL_RAISE2,
        353 => StateNum::S_SKEL_RAISE3,
        354 => StateNum::S_SKEL_RAISE4,
        355 => StateNum::S_SKEL_RAISE5,
        356 => StateNum::S_SKEL_RAISE6,
        357 => StateNum::S_FATSHOT1,
        358 => StateNum::S_FATSHOT2,
        359 => StateNum::S_FATSHOTX1,
        360 => StateNum::S_FATSHOTX2,
        361 => StateNum::S_FATSHOTX3,
        362 => StateNum::S_FATT_STND,
        363 => StateNum::S_FATT_STND2,
        364 => StateNum::S_FATT_RUN1,
        365 => StateNum::S_FATT_RUN2,
        366 => StateNum::S_FATT_RUN3,
        367 => StateNum::S_FATT_RUN4,
        368 => StateNum::S_FATT_RUN5,
        369 => StateNum::S_FATT_RUN6,
        370 => StateNum::S_FATT_RUN7,
        371 => StateNum::S_FATT_RUN8,
        372 => StateNum::S_FATT_RUN9,
        373 => StateNum::S_FATT_RUN10,
        374 => StateNum::S_FATT_RUN11,
        375 => StateNum::S_FATT_RUN12,
        376 => StateNum::S_FATT_ATK1,
        377 => StateNum::S_FATT_ATK2,
        378 => StateNum::S_FATT_ATK3,
        379 => StateNum::S_FATT_ATK4,
        380 => StateNum::S_FATT_ATK5,
        381 => StateNum::S_FATT_ATK6,
        382 => StateNum::S_FATT_ATK7,
        383 => StateNum::S_FATT_ATK8,
        384 => StateNum::S_FATT_ATK9,
        385 => StateNum::S_FATT_ATK10,
        386 => StateNum::S_FATT_PAIN,
        387 => StateNum::S_FATT_PAIN2,
        388 => StateNum::S_FATT_DIE1,
        389 => StateNum::S_FATT_DIE2,
        390 => StateNum::S_FATT_DIE3,
        391 => StateNum::S_FATT_DIE4,
        392 => StateNum::S_FATT_DIE5,
        393 => StateNum::S_FATT_DIE6,
        394 => StateNum::S_FATT_DIE7,
        395 => StateNum::S_FATT_DIE8,
        396 => StateNum::S_FATT_DIE9,
        397 => StateNum::S_FATT_DIE10,
        398 => StateNum::S_FATT_RAISE1,
        399 => StateNum::S_FATT_RAISE2,
        400 => StateNum::S_FATT_RAISE3,
        401 => StateNum::S_FATT_RAISE4,
        402 => StateNum::S_FATT_RAISE5,
        403 => StateNum::S_FATT_RAISE6,
        404 => StateNum::S_FATT_RAISE7,
        405 => StateNum::S_FATT_RAISE8,
        406 => StateNum::S_CPOS_STND,
        407 => StateNum::S_CPOS_STND2,
        408 => StateNum::S_CPOS_RUN1,
        409 => StateNum::S_CPOS_RUN2,
        410 => StateNum::S_CPOS_RUN3,
        411 => StateNum::S_CPOS_RUN4,
        412 => StateNum::S_CPOS_RUN5,
        413 => StateNum::S_CPOS_RUN6,
        414 => StateNum::S_CPOS_RUN7,
        415 => StateNum::S_CPOS_RUN8,
        416 => StateNum::S_CPOS_ATK1,
        417 => StateNum::S_CPOS_ATK2,
        418 => StateNum::S_CPOS_ATK3,
        419 => StateNum::S_CPOS_ATK4,
        420 => StateNum::S_CPOS_PAIN,
        421 => StateNum::S_CPOS_PAIN2,
        422 => StateNum::S_CPOS_DIE1,
        423 => StateNum::S_CPOS_DIE2,
        424 => StateNum::S_CPOS_DIE3,
        425 => StateNum::S_CPOS_DIE4,
        426 => StateNum::S_CPOS_DIE5,
        427 => StateNum::S_CPOS_DIE6,
        428 => StateNum::S_CPOS_DIE7,
        429 => StateNum::S_CPOS_XDIE1,
        430 => StateNum::S_CPOS_XDIE2,
        431 => StateNum::S_CPOS_XDIE3,
        432 => StateNum::S_CPOS_XDIE4,
        433 => StateNum::S_CPOS_XDIE5,
        434 => StateNum::S_CPOS_XDIE6,
        435 => StateNum::S_CPOS_RAISE1,
        436 => StateNum::S_CPOS_RAISE2,
        437 => StateNum::S_CPOS_RAISE3,
        438 => StateNum::S_CPOS_RAISE4,
        439 => StateNum::S_CPOS_RAISE5,
        440 => StateNum::S_CPOS_RAISE6,
        441 => StateNum::S_CPOS_RAISE7,
        442 => StateNum::S_TROO_STND,
        443 => StateNum::S_TROO_STND2,
        444 => StateNum::S_TROO_RUN1,
        445 => StateNum::S_TROO_RUN2,
        446 => StateNum::S_TROO_RUN3,
        447 => StateNum::S_TROO_RUN4,
        448 => StateNum::S_TROO_RUN5,
        449 => StateNum::S_TROO_RUN6,
        450 => StateNum::S_TROO_RUN7,
        451 => StateNum::S_TROO_RUN8,
        452 => StateNum::S_TROO_ATK1,
        453 => StateNum::S_TROO_ATK2,
        454 => StateNum::S_TROO_ATK3,
        455 => StateNum::S_TROO_PAIN,
        456 => StateNum::S_TROO_PAIN2,
        457 => StateNum::S_TROO_DIE1,
        458 => StateNum::S_TROO_DIE2,
        459 => StateNum::S_TROO_DIE3,
        460 => StateNum::S_TROO_DIE4,
        461 => StateNum::S_TROO_DIE5,
        462 => StateNum::S_TROO_XDIE1,
        463 => StateNum::S_TROO_XDIE2,
        464 => StateNum::S_TROO_XDIE3,
        465 => StateNum::S_TROO_XDIE4,
        466 => StateNum::S_TROO_XDIE5,
        467 => StateNum::S_TROO_XDIE6,
        468 => StateNum::S_TROO_XDIE7,
        469 => StateNum::S_TROO_XDIE8,
        470 => StateNum::S_TROO_RAISE1,
        471 => StateNum::S_TROO_RAISE2,
        472 => StateNum::S_TROO_RAISE3,
        473 => StateNum::S_TROO_RAISE4,
        474 => StateNum::S_TROO_RAISE5,
        475 => StateNum::S_SARG_STND,
        476 => StateNum::S_SARG_STND2,
        477 => StateNum::S_SARG_RUN1,
        478 => StateNum::S_SARG_RUN2,
        479 => StateNum::S_SARG_RUN3,
        480 => StateNum::S_SARG_RUN4,
        481 => StateNum::S_SARG_RUN5,
        482 => StateNum::S_SARG_RUN6,
        483 => StateNum::S_SARG_RUN7,
        484 => StateNum::S_SARG_RUN8,
        485 => StateNum::S_SARG_ATK1,
        486 => StateNum::S_SARG_ATK2,
        487 => StateNum::S_SARG_ATK3,
        488 => StateNum::S_SARG_PAIN,
        489 => StateNum::S_SARG_PAIN2,
        490 => StateNum::S_SARG_DIE1,
        491 => StateNum::S_SARG_DIE2,
        492 => StateNum::S_SARG_DIE3,
        493 => StateNum::S_SARG_DIE4,
        494 => StateNum::S_SARG_DIE5,
        495 => StateNum::S_SARG_DIE6,
        496 => StateNum::S_SARG_RAISE1,
        497 => StateNum::S_SARG_RAISE2,
        498 => StateNum::S_SARG_RAISE3,
        499 => StateNum::S_SARG_RAISE4,
        500 => StateNum::S_SARG_RAISE5,
        501 => StateNum::S_SARG_RAISE6,
        502 => StateNum::S_HEAD_STND,
        503 => StateNum::S_HEAD_RUN1,
        504 => StateNum::S_HEAD_ATK1,
        505 => StateNum::S_HEAD_ATK2,
        506 => StateNum::S_HEAD_ATK3,
        507 => StateNum::S_HEAD_PAIN,
        508 => StateNum::S_HEAD_PAIN2,
        509 => StateNum::S_HEAD_PAIN3,
        510 => StateNum::S_HEAD_DIE1,
        511 => StateNum::S_HEAD_DIE2,
        512 => StateNum::S_HEAD_DIE3,
        513 => StateNum::S_HEAD_DIE4,
        514 => StateNum::S_HEAD_DIE5,
        515 => StateNum::S_HEAD_DIE6,
        516 => StateNum::S_HEAD_RAISE1,
        517 => StateNum::S_HEAD_RAISE2,
        518 => StateNum::S_HEAD_RAISE3,
        519 => StateNum::S_HEAD_RAISE4,
        520 => StateNum::S_HEAD_RAISE5,
        521 => StateNum::S_HEAD_RAISE6,
        522 => StateNum::S_BRBALL1,
        523 => StateNum::S_BRBALL2,
        524 => StateNum::S_BRBALLX1,
        525 => StateNum::S_BRBALLX2,
        526 => StateNum::S_BRBALLX3,
        527 => StateNum::S_BOSS_STND,
        528 => StateNum::S_BOSS_STND2,
        529 => StateNum::S_BOSS_RUN1,
        530 => StateNum::S_BOSS_RUN2,
        531 => StateNum::S_BOSS_RUN3,
        532 => StateNum::S_BOSS_RUN4,
        533 => StateNum::S_BOSS_RUN5,
        534 => StateNum::S_BOSS_RUN6,
        535 => StateNum::S_BOSS_RUN7,
        536 => StateNum::S_BOSS_RUN8,
        537 => StateNum::S_BOSS_ATK1,
        538 => StateNum::S_BOSS_ATK2,
        539 => StateNum::S_BOSS_ATK3,
        540 => StateNum::S_BOSS_PAIN,
        541 => StateNum::S_BOSS_PAIN2,
        542 => StateNum::S_BOSS_DIE1,
        543 => StateNum::S_BOSS_DIE2,
        544 => StateNum::S_BOSS_DIE3,
        545 => StateNum::S_BOSS_DIE4,
        546 => StateNum::S_BOSS_DIE5,
        547 => StateNum::S_BOSS_DIE6,
        548 => StateNum::S_BOSS_DIE7,
        549 => StateNum::S_BOSS_RAISE1,
        550 => StateNum::S_BOSS_RAISE2,
        551 => StateNum::S_BOSS_RAISE3,
        552 => StateNum::S_BOSS_RAISE4,
        553 => StateNum::S_BOSS_RAISE5,
        554 => StateNum::S_BOSS_RAISE6,
        555 => StateNum::S_BOSS_RAISE7,
        556 => StateNum::S_BOS2_STND,
        557 => StateNum::S_BOS2_STND2,
        558 => StateNum::S_BOS2_RUN1,
        559 => StateNum::S_BOS2_RUN2,
        560 => StateNum::S_BOS2_RUN3,
        561 => StateNum::S_BOS2_RUN4,
        562 => StateNum::S_BOS2_RUN5,
        563 => StateNum::S_BOS2_RUN6,
        564 => StateNum::S_BOS2_RUN7,
        565 => StateNum::S_BOS2_RUN8,
        566 => StateNum::S_BOS2_ATK1,
        567 => StateNum::S_BOS2_ATK2,
        568 => StateNum::S_BOS2_ATK3,
        569 => StateNum::S_BOS2_PAIN,
        570 => StateNum::S_BOS2_PAIN2,
        571 => StateNum::S_BOS2_DIE1,
        572 => StateNum::S_BOS2_DIE2,
        573 => StateNum::S_BOS2_DIE3,
        574 => StateNum::S_BOS2_DIE4,
        575 => StateNum::S_BOS2_DIE5,
        576 => StateNum::S_BOS2_DIE6,
        577 => StateNum::S_BOS2_DIE7,
        578 => StateNum::S_BOS2_RAISE1,
        579 => StateNum::S_BOS2_RAISE2,
        580 => StateNum::S_BOS2_RAISE3,
        581 => StateNum::S_BOS2_RAISE4,
        582 => StateNum::S_BOS2_RAISE5,
        583 => StateNum::S_BOS2_RAISE6,
        584 => StateNum::S_BOS2_RAISE7,
        585 => StateNum::S_SKULL_STND,
        586 => StateNum::S_SKULL_STND2,
        587 => StateNum::S_SKULL_RUN1,
        588 => StateNum::S_SKULL_RUN2,
        589 => StateNum::S_SKULL_ATK1,
        590 => StateNum::S_SKULL_ATK2,
        591 => StateNum::S_SKULL_ATK3,
        592 => StateNum::S_SKULL_ATK4,
        593 => StateNum::S_SKULL_PAIN,
        594 => StateNum::S_SKULL_PAIN2,
        595 => StateNum::S_SKULL_DIE1,
        596 => StateNum::S_SKULL_DIE2,
        597 => StateNum::S_SKULL_DIE3,
        598 => StateNum::S_SKULL_DIE4,
        599 => StateNum::S_SKULL_DIE5,
        600 => StateNum::S_SKULL_DIE6,
        601 => StateNum::S_SPID_STND,
        602 => StateNum::S_SPID_STND2,
        603 => StateNum::S_SPID_RUN1,
        604 => StateNum::S_SPID_RUN2,
        605 => StateNum::S_SPID_RUN3,
        606 => StateNum::S_SPID_RUN4,
        607 => StateNum::S_SPID_RUN5,
        608 => StateNum::S_SPID_RUN6,
        609 => StateNum::S_SPID_RUN7,
        610 => StateNum::S_SPID_RUN8,
        611 => StateNum::S_SPID_RUN9,
        612 => StateNum::S_SPID_RUN10,
        613 => StateNum::S_SPID_RUN11,
        614 => StateNum::S_SPID_RUN12,
        615 => StateNum::S_SPID_ATK1,
        616 => StateNum::S_SPID_ATK2,
        617 => StateNum::S_SPID_ATK3,
        618 => StateNum::S_SPID_ATK4,
        619 => StateNum::S_SPID_PAIN,
        620 => StateNum::S_SPID_PAIN2,
        621 => StateNum::S_SPID_DIE1,
        622 => StateNum::S_SPID_DIE2,
        623 => StateNum::S_SPID_DIE3,
        624 => StateNum::S_SPID_DIE4,
        625 => StateNum::S_SPID_DIE5,
        626 => StateNum::S_SPID_DIE6,
        627 => StateNum::S_SPID_DIE7,
        628 => StateNum::S_SPID_DIE8,
        629 => StateNum::S_SPID_DIE9,
        630 => StateNum::S_SPID_DIE10,
        631 => StateNum::S_SPID_DIE11,
        632 => StateNum::S_BSPI_STND,
        633 => StateNum::S_BSPI_STND2,
        634 => StateNum::S_BSPI_SIGHT,
        635 => StateNum::S_BSPI_RUN1,
        636 => StateNum::S_BSPI_RUN2,
        637 => StateNum::S_BSPI_RUN3,
        638 => StateNum::S_BSPI_RUN4,
        639 => StateNum::S_BSPI_RUN5,
        640 => StateNum::S_BSPI_RUN6,
        641 => StateNum::S_BSPI_RUN7,
        642 => StateNum::S_BSPI_RUN8,
        643 => StateNum::S_BSPI_RUN9,
        644 => StateNum::S_BSPI_RUN10,
        645 => StateNum::S_BSPI_RUN11,
        646 => StateNum::S_BSPI_RUN12,
        647 => StateNum::S_BSPI_ATK1,
        648 => StateNum::S_BSPI_ATK2,
        649 => StateNum::S_BSPI_ATK3,
        650 => StateNum::S_BSPI_ATK4,
        651 => StateNum::S_BSPI_PAIN,
        652 => StateNum::S_BSPI_PAIN2,
        653 => StateNum::S_BSPI_DIE1,
        654 => StateNum::S_BSPI_DIE2,
        655 => StateNum::S_BSPI_DIE3,
        656 => StateNum::S_BSPI_DIE4,
        657 => StateNum::S_BSPI_DIE5,
        658 => StateNum::S_BSPI_DIE6,
        659 => StateNum::S_BSPI_DIE7,
        660 => StateNum::S_BSPI_RAISE1,
        661 => StateNum::S_BSPI_RAISE2,
        662 => StateNum::S_BSPI_RAISE3,
        663 => StateNum::S_BSPI_RAISE4,
        664 => StateNum::S_BSPI_RAISE5,
        665 => StateNum::S_BSPI_RAISE6,
        666 => StateNum::S_BSPI_RAISE7,
        667 => StateNum::S_ARACH_PLAZ,
        668 => StateNum::S_ARACH_PLAZ2,
        669 => StateNum::S_ARACH_PLEX,
        670 => StateNum::S_ARACH_PLEX2,
        671 => StateNum::S_ARACH_PLEX3,
        672 => StateNum::S_ARACH_PLEX4,
        673 => StateNum::S_ARACH_PLEX5,
        674 => StateNum::S_CYBER_STND,
        675 => StateNum::S_CYBER_STND2,
        676 => StateNum::S_CYBER_RUN1,
        677 => StateNum::S_CYBER_RUN2,
        678 => StateNum::S_CYBER_RUN3,
        679 => StateNum::S_CYBER_RUN4,
        680 => StateNum::S_CYBER_RUN5,
        681 => StateNum::S_CYBER_RUN6,
        682 => StateNum::S_CYBER_RUN7,
        683 => StateNum::S_CYBER_RUN8,
        684 => StateNum::S_CYBER_ATK1,
        685 => StateNum::S_CYBER_ATK2,
        686 => StateNum::S_CYBER_ATK3,
        687 => StateNum::S_CYBER_ATK4,
        688 => StateNum::S_CYBER_ATK5,
        689 => StateNum::S_CYBER_ATK6,
        690 => StateNum::S_CYBER_PAIN,
        691 => StateNum::S_CYBER_DIE1,
        692 => StateNum::S_CYBER_DIE2,
        693 => StateNum::S_CYBER_DIE3,
        694 => StateNum::S_CYBER_DIE4,
        695 => StateNum::S_CYBER_DIE5,
        696 => StateNum::S_CYBER_DIE6,
        697 => StateNum::S_CYBER_DIE7,
        698 => StateNum::S_CYBER_DIE8,
        699 => StateNum::S_CYBER_DIE9,
        700 => StateNum::S_CYBER_DIE10,
        701 => StateNum::S_PAIN_STND,
        702 => StateNum::S_PAIN_RUN1,
        703 => StateNum::S_PAIN_RUN2,
        704 => StateNum::S_PAIN_RUN3,
        705 => StateNum::S_PAIN_RUN4,
        706 => StateNum::S_PAIN_RUN5,
        707 => StateNum::S_PAIN_RUN6,
        708 => StateNum::S_PAIN_ATK1,
        709 => StateNum::S_PAIN_ATK2,
        710 => StateNum::S_PAIN_ATK3,
        711 => StateNum::S_PAIN_ATK4,
        712 => StateNum::S_PAIN_PAIN,
        713 => StateNum::S_PAIN_PAIN2,
        714 => StateNum::S_PAIN_DIE1,
        715 => StateNum::S_PAIN_DIE2,
        716 => StateNum::S_PAIN_DIE3,
        717 => StateNum::S_PAIN_DIE4,
        718 => StateNum::S_PAIN_DIE5,
        719 => StateNum::S_PAIN_DIE6,
        720 => StateNum::S_PAIN_RAISE1,
        721 => StateNum::S_PAIN_RAISE2,
        722 => StateNum::S_PAIN_RAISE3,
        723 => StateNum::S_PAIN_RAISE4,
        724 => StateNum::S_PAIN_RAISE5,
        725 => StateNum::S_PAIN_RAISE6,
        726 => StateNum::S_SSWV_STND,
        727 => StateNum::S_SSWV_STND2,
        728 => StateNum::S_SSWV_RUN1,
        729 => StateNum::S_SSWV_RUN2,
        730 => StateNum::S_SSWV_RUN3,
        731 => StateNum::S_SSWV_RUN4,
        732 => StateNum::S_SSWV_RUN5,
        733 => StateNum::S_SSWV_RUN6,
        734 => StateNum::S_SSWV_RUN7,
        735 => StateNum::S_SSWV_RUN8,
        736 => StateNum::S_SSWV_ATK1,
        737 => StateNum::S_SSWV_ATK2,
        738 => StateNum::S_SSWV_ATK3,
        739 => StateNum::S_SSWV_ATK4,
        740 => StateNum::S_SSWV_ATK5,
        741 => StateNum::S_SSWV_ATK6,
        742 => StateNum::S_SSWV_PAIN,
        743 => StateNum::S_SSWV_PAIN2,
        744 => StateNum::S_SSWV_DIE1,
        745 => StateNum::S_SSWV_DIE2,
        746 => StateNum::S_SSWV_DIE3,
        747 => StateNum::S_SSWV_DIE4,
        748 => StateNum::S_SSWV_DIE5,
        749 => StateNum::S_SSWV_XDIE1,
        750 => StateNum::S_SSWV_XDIE2,
        751 => StateNum::S_SSWV_XDIE3,
        752 => StateNum::S_SSWV_XDIE4,
        753 => StateNum::S_SSWV_XDIE5,
        754 => StateNum::S_SSWV_XDIE6,
        755 => StateNum::S_SSWV_XDIE7,
        756 => StateNum::S_SSWV_XDIE8,
        757 => StateNum::S_SSWV_XDIE9,
        758 => StateNum::S_SSWV_RAISE1,
        759 => StateNum::S_SSWV_RAISE2,
        760 => StateNum::S_SSWV_RAISE3,
        761 => StateNum::S_SSWV_RAISE4,
        762 => StateNum::S_SSWV_RAISE5,
        763 => StateNum::S_KEENSTND,
        764 => StateNum::S_COMMKEEN,
        765 => StateNum::S_COMMKEEN2,
        766 => StateNum::S_COMMKEEN3,
        767 => StateNum::S_COMMKEEN4,
        768 => StateNum::S_COMMKEEN5,
        769 => StateNum::S_COMMKEEN6,
        770 => StateNum::S_COMMKEEN7,
        771 => StateNum::S_COMMKEEN8,
        772 => StateNum::S_COMMKEEN9,
        773 => StateNum::S_COMMKEEN10,
        774 => StateNum::S_COMMKEEN11,
        775 => StateNum::S_COMMKEEN12,
        776 => StateNum::S_KEENPAIN,
        777 => StateNum::S_KEENPAIN2,
        778 => StateNum::S_BRAIN,
        779 => StateNum::S_BRAIN_PAIN,
        780 => StateNum::S_BRAIN_DIE1,
        781 => StateNum::S_BRAIN_DIE2,
        782 => StateNum::S_BRAIN_DIE3,
        783 => StateNum::S_BRAIN_DIE4,
        784 => StateNum::S_BRAINEYE,
        785 => StateNum::S_BRAINEYESEE,
        786 => StateNum::S_BRAINEYE1,
        787 => StateNum::S_SPAWN1,
        788 => StateNum::S_SPAWN2,
        789 => StateNum::S_SPAWN3,
        790 => StateNum::S_SPAWN4,
        791 => StateNum::S_SPAWNFIRE1,
        792 => StateNum::S_SPAWNFIRE2,
        793 => StateNum::S_SPAWNFIRE3,
        794 => StateNum::S_SPAWNFIRE4,
        795 => StateNum::S_SPAWNFIRE5,
        796 => StateNum::S_SPAWNFIRE6,
        797 => StateNum::S_SPAWNFIRE7,
        798 => StateNum::S_SPAWNFIRE8,
        799 => StateNum::S_BRAINEXPLODE1,
        800 => StateNum::S_BRAINEXPLODE2,
        801 => StateNum::S_BRAINEXPLODE3,
        802 => StateNum::S_ARM1,
        803 => StateNum::S_ARM1A,
        804 => StateNum::S_ARM2,
        805 => StateNum::S_ARM2A,
        806 => StateNum::S_BAR1,
        807 => StateNum::S_BAR2,
        808 => StateNum::S_BEXP,
        809 => StateNum::S_BEXP2,
        810 => StateNum::S_BEXP3,
        811 => StateNum::S_BEXP4,
        812 => StateNum::S_BEXP5,
        813 => StateNum::S_BBAR1,
        814 => StateNum::S_BBAR2,
        815 => StateNum::S_BBAR3,
        816 => StateNum::S_BON1,
        817 => StateNum::S_BON1A,
        818 => StateNum::S_BON1B,
        819 => StateNum::S_BON1C,
        820 => StateNum::S_BON1D,
        821 => StateNum::S_BON1E,
        822 => StateNum::S_BON2,
        823 => StateNum::S_BON2A,
        824 => StateNum::S_BON2B,
        825 => StateNum::S_BON2C,
        826 => StateNum::S_BON2D,
        827 => StateNum::S_BON2E,
        828 => StateNum::S_BKEY,
        829 => StateNum::S_BKEY2,
        830 => StateNum::S_RKEY,
        831 => StateNum::S_RKEY2,
        832 => StateNum::S_YKEY,
        833 => StateNum::S_YKEY2,
        834 => StateNum::S_BSKULL,
        835 => StateNum::S_BSKULL2,
        836 => StateNum::S_RSKULL,
        837 => StateNum::S_RSKULL2,
        838 => StateNum::S_YSKULL,
        839 => StateNum::S_YSKULL2,
        840 => StateNum::S_STIM,
        841 => StateNum::S_MEDI,
        842 => StateNum::S_SOUL,
        843 => StateNum::S_SOUL2,
        844 => StateNum::S_SOUL3,
        845 => StateNum::S_SOUL4,
        846 => StateNum::S_SOUL5,
        847 => StateNum::S_SOUL6,
        848 => StateNum::S_PINV,
        849 => StateNum::S_PINV2,
        850 => StateNum::S_PINV3,
        851 => StateNum::S_PINV4,
        852 => StateNum::S_PSTR,
        853 => StateNum::S_PINS,
        854 => StateNum::S_PINS2,
        855 => StateNum::S_PINS3,
        856 => StateNum::S_PINS4,
        857 => StateNum::S_MEGA,
        858 => StateNum::S_MEGA2,
        859 => StateNum::S_MEGA3,
        860 => StateNum::S_MEGA4,
        861 => StateNum::S_SUIT,
        862 => StateNum::S_PMAP,
        863 => StateNum::S_PMAP2,
        864 => StateNum::S_PMAP3,
        865 => StateNum::S_PMAP4,
        866 => StateNum::S_PMAP5,
        867 => StateNum::S_PMAP6,
        868 => StateNum::S_PVIS,
        869 => StateNum::S_PVIS2,
        870 => StateNum::S_CLIP,
        871 => StateNum::S_AMMO,
        872 => StateNum::S_ROCK,
        873 => StateNum::S_BROK,
        874 => StateNum::S_CELL,
        875 => StateNum::S_CELP,
        876 => StateNum::S_SHEL,
        877 => StateNum::S_SBOX,
        878 => StateNum::S_BPAK,
        879 => StateNum::S_BFUG,
        880 => StateNum::S_MGUN,
        881 => StateNum::S_CSAW,
        882 => StateNum::S_LAUN,
        883 => StateNum::S_PLAS,
        884 => StateNum::S_SHOT,
        885 => StateNum::S_SHOT2,
        886 => StateNum::S_COLU,
        887 => StateNum::S_STALAG,
        888 => StateNum::S_BLOODYTWITCH,
        889 => StateNum::S_BLOODYTWITCH2,
        890 => StateNum::S_BLOODYTWITCH3,
        891 => StateNum::S_BLOODYTWITCH4,
        892 => StateNum::S_DEADTORSO,
        893 => StateNum::S_DEADBOTTOM,
        894 => StateNum::S_HEADSONSTICK,
        895 => StateNum::S_GIBS,
        896 => StateNum::S_HEADONASTICK,
        897 => StateNum::S_HEADCANDLES,
        898 => StateNum::S_HEADCANDLES2,
        899 => StateNum::S_DEADSTICK,
        900 => StateNum::S_LIVESTICK,
        901 => StateNum::S_LIVESTICK2,
        902 => StateNum::S_MEAT2,
        903 => StateNum::S_MEAT3,
        904 => StateNum::S_MEAT4,
        905 => StateNum::S_MEAT5,
        906 => StateNum::S_STALAGTITE,
        907 => StateNum::S_TALLGRNCOL,
        908 => StateNum::S_SHRTGRNCOL,
        909 => StateNum::S_TALLREDCOL,
        910 => StateNum::S_SHRTREDCOL,
        911 => StateNum::S_CANDLESTIK,
        912 => StateNum::S_CANDELABRA,
        913 => StateNum::S_SKULLCOL,
        914 => StateNum::S_TORCHTREE,
        915 => StateNum::S_BIGTREE,
        916 => StateNum::S_TECHPILLAR,
        917 => StateNum::S_EVILEYE,
        918 => StateNum::S_EVILEYE2,
        919 => StateNum::S_EVILEYE3,
        920 => StateNum::S_EVILEYE4,
        921 => StateNum::S_FLOATSKULL,
        922 => StateNum::S_FLOATSKULL2,
        923 => StateNum::S_FLOATSKULL3,
        924 => StateNum::S_HEARTCOL,
        925 => StateNum::S_HEARTCOL2,
        926 => StateNum::S_BLUETORCH,
        927 => StateNum::S_BLUETORCH2,
        928 => StateNum::S_BLUETORCH3,
        929 => StateNum::S_BLUETORCH4,
        930 => StateNum::S_GREENTORCH,
        931 => StateNum::S_GREENTORCH2,
        932 => StateNum::S_GREENTORCH3,
        933 => StateNum::S_GREENTORCH4,
        934 => StateNum::S_REDTORCH,
        935 => StateNum::S_REDTORCH2,
        936 => StateNum::S_REDTORCH3,
        937 => StateNum::S_REDTORCH4,
        938 => StateNum::S_BTORCHSHRT,
        939 => StateNum::S_BTORCHSHRT2,
        940 => StateNum::S_BTORCHSHRT3,
        941 => StateNum::S_BTORCHSHRT4,
        942 => StateNum::S_GTORCHSHRT,
        943 => StateNum::S_GTORCHSHRT2,
        944 => StateNum::S_GTORCHSHRT3,
        945 => StateNum::S_GTORCHSHRT4,
        946 => StateNum::S_RTORCHSHRT,
        947 => StateNum::S_RTORCHSHRT2,
        948 => StateNum::S_RTORCHSHRT3,
        949 => StateNum::S_RTORCHSHRT4,
        950 => StateNum::S_HANGNOGUTS,
        951 => StateNum::S_HANGBNOBRAIN,
        952 => StateNum::S_HANGTLOOKDN,
        953 => StateNum::S_HANGTSKULL,
        954 => StateNum::S_HANGTLOOKUP,
        955 => StateNum::S_HANGTNOBRAIN,
        956 => StateNum::S_COLONGIBS,
        957 => StateNum::S_SMALLPOOL,
        958 => StateNum::S_BRAINSTEM,
        959 => StateNum::S_TECHLAMP,
        960 => StateNum::S_TECHLAMP2,
        961 => StateNum::S_TECHLAMP3,
        962 => StateNum::S_TECHLAMP4,
        963 => StateNum::S_TECH2LAMP,
        964 => StateNum::S_TECH2LAMP2,
        965 => StateNum::S_TECH2LAMP3,
        966 => StateNum::S_TECH2LAMP4,
        n => panic!("invalid statenum {n}"),
    }
}
#[derive(Copy, Clone)]
pub struct state_t {
    pub sprite: SpriteNum,
    pub frame: i32,
    pub tics: i32,
    pub action: StateAction,
    pub nextstate: StateNum,
    pub misc1: i32,
    pub misc2: i32,
}
pub const NUMMOBJTYPES: i32 = 137;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MobjType {
    MT_PLAYER = 0,
    MT_POSSESSED = 1,
    MT_SHOTGUY = 2,
    MT_VILE = 3,
    MT_FIRE = 4,
    MT_UNDEAD = 5,
    MT_TRACER = 6,
    MT_SMOKE = 7,
    MT_FATSO = 8,
    MT_FATSHOT = 9,
    MT_CHAINGUY = 10,
    MT_TROOP = 11,
    MT_SERGEANT = 12,
    MT_SHADOWS = 13,
    MT_HEAD = 14,
    MT_BRUISER = 15,
    MT_BRUISERSHOT = 16,
    MT_KNIGHT = 17,
    MT_SKULL = 18,
    MT_SPIDER = 19,
    MT_BABY = 20,
    MT_CYBORG = 21,
    MT_PAIN = 22,
    MT_WOLFSS = 23,
    MT_KEEN = 24,
    MT_BOSSBRAIN = 25,
    MT_BOSSSPIT = 26,
    MT_BOSSTARGET = 27,
    MT_SPAWNSHOT = 28,
    MT_SPAWNFIRE = 29,
    MT_BARREL = 30,
    MT_TROOPSHOT = 31,
    MT_HEADSHOT = 32,
    MT_ROCKET = 33,
    MT_PLASMA = 34,
    MT_BFG = 35,
    MT_ARACHPLAZ = 36,
    MT_PUFF = 37,
    MT_BLOOD = 38,
    MT_TFOG = 39,
    MT_IFOG = 40,
    MT_TELEPORTMAN = 41,
    MT_EXTRABFG = 42,
    MT_MISC0 = 43,
    MT_MISC1 = 44,
    MT_MISC2 = 45,
    MT_MISC3 = 46,
    MT_MISC4 = 47,
    MT_MISC5 = 48,
    MT_MISC6 = 49,
    MT_MISC7 = 50,
    MT_MISC8 = 51,
    MT_MISC9 = 52,
    MT_MISC10 = 53,
    MT_MISC11 = 54,
    MT_MISC12 = 55,
    MT_INV = 56,
    MT_MISC13 = 57,
    MT_INS = 58,
    MT_MISC14 = 59,
    MT_MISC15 = 60,
    MT_MISC16 = 61,
    MT_MEGA = 62,
    MT_CLIP = 63,
    MT_MISC17 = 64,
    MT_MISC18 = 65,
    MT_MISC19 = 66,
    MT_MISC20 = 67,
    MT_MISC21 = 68,
    MT_MISC22 = 69,
    MT_MISC23 = 70,
    MT_MISC24 = 71,
    MT_MISC25 = 72,
    MT_CHAINGUN = 73,
    MT_MISC26 = 74,
    MT_MISC27 = 75,
    MT_MISC28 = 76,
    MT_SHOTGUN = 77,
    MT_SUPERSHOTGUN = 78,
    MT_MISC29 = 79,
    MT_MISC30 = 80,
    MT_MISC31 = 81,
    MT_MISC32 = 82,
    MT_MISC33 = 83,
    MT_MISC34 = 84,
    MT_MISC35 = 85,
    MT_MISC36 = 86,
    MT_MISC37 = 87,
    MT_MISC38 = 88,
    MT_MISC39 = 89,
    MT_MISC40 = 90,
    MT_MISC41 = 91,
    MT_MISC42 = 92,
    MT_MISC43 = 93,
    MT_MISC44 = 94,
    MT_MISC45 = 95,
    MT_MISC46 = 96,
    MT_MISC47 = 97,
    MT_MISC48 = 98,
    MT_MISC49 = 99,
    MT_MISC50 = 100,
    MT_MISC51 = 101,
    MT_MISC52 = 102,
    MT_MISC53 = 103,
    MT_MISC54 = 104,
    MT_MISC55 = 105,
    MT_MISC56 = 106,
    MT_MISC57 = 107,
    MT_MISC58 = 108,
    MT_MISC59 = 109,
    MT_MISC60 = 110,
    MT_MISC61 = 111,
    MT_MISC62 = 112,
    MT_MISC63 = 113,
    MT_MISC64 = 114,
    MT_MISC65 = 115,
    MT_MISC66 = 116,
    MT_MISC67 = 117,
    MT_MISC68 = 118,
    MT_MISC69 = 119,
    MT_MISC70 = 120,
    MT_MISC71 = 121,
    MT_MISC72 = 122,
    MT_MISC73 = 123,
    MT_MISC74 = 124,
    MT_MISC75 = 125,
    MT_MISC76 = 126,
    MT_MISC77 = 127,
    MT_MISC78 = 128,
    MT_MISC79 = 129,
    MT_MISC80 = 130,
    MT_MISC81 = 131,
    MT_MISC82 = 132,
    MT_MISC83 = 133,
    MT_MISC84 = 134,
    MT_MISC85 = 135,
    MT_MISC86 = 136,
}
pub fn mobjtype_from_raw(v: i32) -> MobjType {
    match v {
        0 => MobjType::MT_PLAYER,
        1 => MobjType::MT_POSSESSED,
        2 => MobjType::MT_SHOTGUY,
        3 => MobjType::MT_VILE,
        4 => MobjType::MT_FIRE,
        5 => MobjType::MT_UNDEAD,
        6 => MobjType::MT_TRACER,
        7 => MobjType::MT_SMOKE,
        8 => MobjType::MT_FATSO,
        9 => MobjType::MT_FATSHOT,
        10 => MobjType::MT_CHAINGUY,
        11 => MobjType::MT_TROOP,
        12 => MobjType::MT_SERGEANT,
        13 => MobjType::MT_SHADOWS,
        14 => MobjType::MT_HEAD,
        15 => MobjType::MT_BRUISER,
        16 => MobjType::MT_BRUISERSHOT,
        17 => MobjType::MT_KNIGHT,
        18 => MobjType::MT_SKULL,
        19 => MobjType::MT_SPIDER,
        20 => MobjType::MT_BABY,
        21 => MobjType::MT_CYBORG,
        22 => MobjType::MT_PAIN,
        23 => MobjType::MT_WOLFSS,
        24 => MobjType::MT_KEEN,
        25 => MobjType::MT_BOSSBRAIN,
        26 => MobjType::MT_BOSSSPIT,
        27 => MobjType::MT_BOSSTARGET,
        28 => MobjType::MT_SPAWNSHOT,
        29 => MobjType::MT_SPAWNFIRE,
        30 => MobjType::MT_BARREL,
        31 => MobjType::MT_TROOPSHOT,
        32 => MobjType::MT_HEADSHOT,
        33 => MobjType::MT_ROCKET,
        34 => MobjType::MT_PLASMA,
        35 => MobjType::MT_BFG,
        36 => MobjType::MT_ARACHPLAZ,
        37 => MobjType::MT_PUFF,
        38 => MobjType::MT_BLOOD,
        39 => MobjType::MT_TFOG,
        40 => MobjType::MT_IFOG,
        41 => MobjType::MT_TELEPORTMAN,
        42 => MobjType::MT_EXTRABFG,
        43 => MobjType::MT_MISC0,
        44 => MobjType::MT_MISC1,
        45 => MobjType::MT_MISC2,
        46 => MobjType::MT_MISC3,
        47 => MobjType::MT_MISC4,
        48 => MobjType::MT_MISC5,
        49 => MobjType::MT_MISC6,
        50 => MobjType::MT_MISC7,
        51 => MobjType::MT_MISC8,
        52 => MobjType::MT_MISC9,
        53 => MobjType::MT_MISC10,
        54 => MobjType::MT_MISC11,
        55 => MobjType::MT_MISC12,
        56 => MobjType::MT_INV,
        57 => MobjType::MT_MISC13,
        58 => MobjType::MT_INS,
        59 => MobjType::MT_MISC14,
        60 => MobjType::MT_MISC15,
        61 => MobjType::MT_MISC16,
        62 => MobjType::MT_MEGA,
        63 => MobjType::MT_CLIP,
        64 => MobjType::MT_MISC17,
        65 => MobjType::MT_MISC18,
        66 => MobjType::MT_MISC19,
        67 => MobjType::MT_MISC20,
        68 => MobjType::MT_MISC21,
        69 => MobjType::MT_MISC22,
        70 => MobjType::MT_MISC23,
        71 => MobjType::MT_MISC24,
        72 => MobjType::MT_MISC25,
        73 => MobjType::MT_CHAINGUN,
        74 => MobjType::MT_MISC26,
        75 => MobjType::MT_MISC27,
        76 => MobjType::MT_MISC28,
        77 => MobjType::MT_SHOTGUN,
        78 => MobjType::MT_SUPERSHOTGUN,
        79 => MobjType::MT_MISC29,
        80 => MobjType::MT_MISC30,
        81 => MobjType::MT_MISC31,
        82 => MobjType::MT_MISC32,
        83 => MobjType::MT_MISC33,
        84 => MobjType::MT_MISC34,
        85 => MobjType::MT_MISC35,
        86 => MobjType::MT_MISC36,
        87 => MobjType::MT_MISC37,
        88 => MobjType::MT_MISC38,
        89 => MobjType::MT_MISC39,
        90 => MobjType::MT_MISC40,
        91 => MobjType::MT_MISC41,
        92 => MobjType::MT_MISC42,
        93 => MobjType::MT_MISC43,
        94 => MobjType::MT_MISC44,
        95 => MobjType::MT_MISC45,
        96 => MobjType::MT_MISC46,
        97 => MobjType::MT_MISC47,
        98 => MobjType::MT_MISC48,
        99 => MobjType::MT_MISC49,
        100 => MobjType::MT_MISC50,
        101 => MobjType::MT_MISC51,
        102 => MobjType::MT_MISC52,
        103 => MobjType::MT_MISC53,
        104 => MobjType::MT_MISC54,
        105 => MobjType::MT_MISC55,
        106 => MobjType::MT_MISC56,
        107 => MobjType::MT_MISC57,
        108 => MobjType::MT_MISC58,
        109 => MobjType::MT_MISC59,
        110 => MobjType::MT_MISC60,
        111 => MobjType::MT_MISC61,
        112 => MobjType::MT_MISC62,
        113 => MobjType::MT_MISC63,
        114 => MobjType::MT_MISC64,
        115 => MobjType::MT_MISC65,
        116 => MobjType::MT_MISC66,
        117 => MobjType::MT_MISC67,
        118 => MobjType::MT_MISC68,
        119 => MobjType::MT_MISC69,
        120 => MobjType::MT_MISC70,
        121 => MobjType::MT_MISC71,
        122 => MobjType::MT_MISC72,
        123 => MobjType::MT_MISC73,
        124 => MobjType::MT_MISC74,
        125 => MobjType::MT_MISC75,
        126 => MobjType::MT_MISC76,
        127 => MobjType::MT_MISC77,
        128 => MobjType::MT_MISC78,
        129 => MobjType::MT_MISC79,
        130 => MobjType::MT_MISC80,
        131 => MobjType::MT_MISC81,
        132 => MobjType::MT_MISC82,
        133 => MobjType::MT_MISC83,
        134 => MobjType::MT_MISC84,
        135 => MobjType::MT_MISC85,
        136 => MobjType::MT_MISC86,
        n => panic!("invalid mobjtype {n}"),
    }
}
#[derive(Copy, Clone)]
pub struct mobjinfo_t {
    pub doomednum: i32,
    pub spawnstate: StateNum,
    pub spawnhealth: i32,
    pub seestate: StateNum,
    pub seesound: i32,
    pub reactiontime: i32,
    pub attacksound: i32,
    pub painstate: StateNum,
    pub painchance: i32,
    pub painsound: i32,
    pub meleestate: StateNum,
    pub missilestate: StateNum,
    pub deathstate: StateNum,
    pub xdeathstate: StateNum,
    pub deathsound: i32,
    pub speed: i32,
    pub radius: i32,
    pub height: i32,
    pub mass: i32,
    pub damage: i32,
    pub activesound: i32,
    pub flags: i32,
    pub raisestate: StateNum,
}
pub const MF_TRANSSHIFT: i32 = 26;
pub const MF_TRANSLATION: i32 = 201326592;
pub const MF_NOTDMATCH: i32 = 33554432;
pub const MF_SKULLFLY: i32 = 16777216;
pub const MF_COUNTITEM: i32 = 8388608;
pub const MF_COUNTKILL: i32 = 4194304;
pub const MF_INFLOAT: i32 = 2097152;
pub const MF_CORPSE: i32 = 1048576;
pub const MF_NOBLOOD: i32 = 524288;
pub const MF_SHADOW: i32 = 262144;
pub const MF_DROPPED: i32 = 131072;
pub const MF_MISSILE: i32 = 65536;
pub const MF_TELEPORT: i32 = 32768;
pub const MF_FLOAT: i32 = 16384;
pub const MF_SLIDE: i32 = 8192;
pub const MF_NOCLIP: i32 = 4096;
pub const MF_PICKUP: i32 = 2048;
pub const MF_DROPOFF: i32 = 1024;
pub const MF_NOGRAVITY: i32 = 512;
pub const MF_SPAWNCEILING: i32 = 256;
pub const MF_JUSTATTACKED: i32 = 128;
pub const MF_JUSTHIT: i32 = 64;
pub const MF_AMBUSH: i32 = 32;
pub const MF_NOBLOCKMAP: i32 = 16;
pub const MF_NOSECTOR: i32 = 8;
pub const MF_SHOOTABLE: i32 = 4;
pub const MF_SOLID: i32 = 2;
pub const MF_SPECIAL: i32 = 1;
#[derive(Copy, Clone)]
pub struct mobj_s {
    pub thinker: thinker_t,
    pub x: fixed_t,
    pub y: fixed_t,
    pub z: fixed_t,
    pub snext: Option<MobjId>,
    pub sprev: Option<MobjId>,
    pub angle: angle_t,
    pub sprite: SpriteNum,
    pub frame: i32,
    pub bnext: Option<MobjId>,
    pub bprev: Option<MobjId>,
    pub subsector: SubsectorId,
    pub floorz: fixed_t,
    pub ceilingz: fixed_t,
    pub radius: fixed_t,
    pub height: fixed_t,
    pub momx: fixed_t,
    pub momy: fixed_t,
    pub momz: fixed_t,
    pub validcount: i32,
    pub type_0: MobjType,
    pub tics: i32,
    pub state: Option<StateId>,
    pub flags: i32,
    pub health: i32,
    pub movedir: i32,
    pub movecount: i32,
    pub target: Option<MobjId>,
    pub reactiontime: i32,
    pub threshold: i32,
    pub player: Option<PlayerId>,
    pub lastlook: i32,
    pub spawnpoint: mapthing_t,
    pub tracer: Option<MobjId>,
    pub id: MobjId,
}
#[derive(Copy, Clone)]
pub struct pspdef_t {
    pub state: Option<StateId>,
    pub tics: i32,
    pub sx: fixed_t,
    pub sy: fixed_t,
}
pub type mobj_t = mobj_s;
pub use crate::d_player::{player_s, player_t, PlayerId, PlayerState};
#[derive(Copy, Clone)]
pub struct subsector_s {
    pub sector: SectorId,
    pub numlines: i16,
    pub firstline: i16,
}
#[derive(Clone)]
pub struct sector_t {
    pub floorheight: fixed_t,
    pub ceilingheight: fixed_t,
    pub floorpic: i16,
    pub ceilingpic: i16,
    pub lightlevel: i16,
    pub special: i16,
    pub tag: i16,
    pub soundtraversed: i32,
    pub soundtarget: Option<MobjId>,
    pub blockbox: [i32; 4],
    pub soundorg: degenmobj_t,
    pub validcount: i32,
    pub thinglist: Option<MobjId>,
    pub specialdata: Option<SectorSpecial>,
    pub linecount: i32,
    pub lines: Vec<LineId>,
}
#[derive(Copy, Clone)]
pub struct line_s {
    pub v1: VertexId,
    pub v2: VertexId,
    pub dx: fixed_t,
    pub dy: fixed_t,
    pub flags: i16,
    pub special: i16,
    pub tag: i16,
    pub sidenum: [i16; 2],
    pub bbox: [fixed_t; 4],
    pub slopetype: SlopeType,
    pub frontsector: Option<SectorId>,
    pub backsector: Option<SectorId>,
    pub validcount: i32,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SlopeType {
    ST_HORIZONTAL = 0,
    ST_VERTICAL = 1,
    ST_POSITIVE = 2,
    ST_NEGATIVE = 3,
}
#[derive(Copy, Clone)]
pub struct vertex_t {
    pub x: fixed_t,
    pub y: fixed_t,
}
#[derive(Copy, Clone)]
pub struct degenmobj_t {
    pub thinker: thinker_t,
    pub x: fixed_t,
    pub y: fixed_t,
    pub z: fixed_t,
}
pub type line_t = line_s;
pub type subsector_t = subsector_s;
pub const MTF_AMBUSH: i32 = 8;
pub const FLOATSPEED: i32 = FRACUNIT * 4_i32;
pub const GRAVITY: i32 = FRACUNIT;
pub const MAXMOVE: i32 = 30 * FRACUNIT;
pub const ONFLOORZ: i32 = INT_MIN;
pub const ONCEILINGZ: i32 = INT_MAX;
pub const ITEMQUESIZE: i32 = 128;
pub fn P_SetMobjState(state: &mut GameState, mobj: MobjId, mut statenum: StateNum) -> bool {
    loop {
        if statenum == StateNum::S_NULL {
            state.p_mobj.mo_mut(mobj).state = None;
            P_RemoveMobj(state, mobj);
            return false;
        }
        let state_id = StateId(statenum as u32);
        let (tics, sprite, frame, action, nextstate) = {
            let st = state.info.state_mut(state_id);
            (st.tics, st.sprite, st.frame, st.action, st.nextstate)
        };
        {
            let m = state.p_mobj.mo_mut(mobj);
            m.state = Some(state_id);
            m.tics = tics;
            m.sprite = sprite;
            m.frame = frame;
        }
        if let StateAction::Mobj(f) = action {
            f(state, mobj);
        }
        statenum = nextstate;
        if state.p_mobj.mo(mobj).tics != 0 {
            break;
        }
    }
    true
}
pub fn P_ExplodeMissile(state: &mut GameState, mo: MobjId) {
    state.p_mobj.mo_mut(mo).momz = 0_i32 as fixed_t;
    state.p_mobj.mo_mut(mo).momy = state.p_mobj.mo(mo).momz;
    state.p_mobj.mo_mut(mo).momx = state.p_mobj.mo(mo).momy;
    P_SetMobjState(
        state,
        mo,
        state.info.mobjinfo[state.p_mobj.mo(mo).type_0 as usize].deathstate,
    );
    state.p_mobj.mo_mut(mo).tics -= P_Random(&mut state.m_random) & 3_i32;
    if state.p_mobj.mo(mo).tics < 1_i32 {
        state.p_mobj.mo_mut(mo).tics = 1_i32;
    }
    state.p_mobj.mo_mut(mo).flags &= !MF_MISSILE;
    let deathsound = state
        .info
        .mobjinfo_mut(state.p_mobj.mo(mo).type_0)
        .deathsound;
    if deathsound != 0 {
        S_StartSound(state, SoundOrigin::Mobj(mo), deathsound);
    }
}
pub const STOPSPEED: i32 = 0x1000;
pub const FRICTION: i32 = 0xe800;
pub fn P_XYMovement(state: &mut GameState, mo: MobjId) {
    let (momx, momy, flags) = {
        let m = state.p_mobj.mo(mo);
        (m.momx, m.momy, m.flags)
    };
    if momx == 0 && momy == 0 {
        if flags & MF_SKULLFLY != 0 {
            let mo_type = {
                let m = state.p_mobj.mo_mut(mo);
                m.flags &= !MF_SKULLFLY;
                m.momz = 0_i32 as fixed_t;
                m.momy = m.momz;
                m.momx = m.momy;
                m.type_0
            };
            let spawnstate = state.info.mobjinfo_mut(mo_type).spawnstate;
            P_SetMobjState(state, mo, spawnstate);
        }
        return;
    }
    let player = state.p_mobj.mo(mo).player;
    let (mut xmove, mut ymove) = {
        let m = state.p_mobj.mo_mut(mo);
        if m.momx > MAXMOVE {
            m.momx = MAXMOVE as fixed_t;
        } else if m.momx < -MAXMOVE {
            m.momx = -MAXMOVE as fixed_t;
        }
        if m.momy > MAXMOVE {
            m.momy = MAXMOVE as fixed_t;
        } else if m.momy < -MAXMOVE {
            m.momy = -MAXMOVE as fixed_t;
        }
        (m.momx, m.momy)
    };
    loop {
        let (mx, my) = {
            let m = state.p_mobj.mo(mo);
            (m.x, m.y)
        };
        let ptryx: fixed_t;
        let ptryy: fixed_t;
        if xmove > MAXMOVE / 2_i32 || ymove > MAXMOVE / 2_i32 {
            ptryx = (mx + xmove / 2_i32) as fixed_t;
            ptryy = (my + ymove / 2_i32) as fixed_t;
            xmove >>= 1_i32;
            ymove >>= 1_i32;
        } else {
            ptryx = mx + xmove;
            ptryy = my + ymove;
            ymove = 0_i32 as fixed_t;
            xmove = ymove;
        }
        if !P_TryMove(state, mo, ptryx, ptryy) {
            if player.is_some() {
                P_SlideMove(state, mo);
            } else if state.p_mobj.mo(mo).flags & MF_MISSILE != 0 {
                if state.p_map.ceilingline.is_some_and(|ceilingline| {
                    state
                        .p_setup
                        .line(ceilingline)
                        .backsector
                        .is_some_and(|backsector| {
                            state.p_setup.sector_mut(backsector).ceilingpic as i32
                                == state.r_sky.skyflatnum
                        })
                }) {
                    P_RemoveMobj(state, mo);
                    return;
                }
                P_ExplodeMissile(state, mo);
            } else {
                let m = state.p_mobj.mo_mut(mo);
                m.momy = 0_i32 as fixed_t;
                m.momx = m.momy;
            }
        }
        if !(xmove != 0 || ymove != 0) {
            break;
        }
    }
    if let Some(player_id) = player {
        if state.g_game.players[player_id.0 as usize].cheats & CF_NOMOMENTUM != 0 {
            let m = state.p_mobj.mo_mut(mo);
            m.momy = 0_i32 as fixed_t;
            m.momx = m.momy;
            return;
        }
    }
    let (flags, z, floorz, momx, momy, subsector) = {
        let m = state.p_mobj.mo(mo);
        (m.flags, m.z, m.floorz, m.momx, m.momy, m.subsector)
    };
    if flags & (MF_MISSILE | MF_SKULLFLY) != 0 {
        return;
    }
    if z > floorz {
        return;
    }
    if flags & MF_CORPSE != 0
        && (!(-FRACUNIT / 4_i32..=FRACUNIT / 4_i32).contains(&momx)
            || !(-FRACUNIT / 4_i32..=FRACUNIT / 4_i32).contains(&momy))
        && floorz
            != state
                .p_setup
                .sector_mut(state.p_setup.subsectors[subsector.0 as usize].sector)
                .floorheight
    {
        return;
    }
    let player_idle = match player {
        None => true,
        Some(player_id) => {
            let cmd = &state.g_game.players[player_id.0 as usize].cmd;
            cmd.forwardmove as i32 == 0_i32 && cmd.sidemove as i32 == 0_i32
        }
    };
    if momx > -STOPSPEED && momx < STOPSPEED && momy > -STOPSPEED && momy < STOPSPEED && player_idle
    {
        if player.is_some()
            && (state
                .p_mobj
                .mo(mo)
                .state
                .unwrap()
                .0
                .wrapping_sub(StateNum::S_PLAY_RUN1 as u32))
                < 4_u32
        {
            P_SetMobjState(state, mo, StateNum::S_PLAY);
        }
        let m = state.p_mobj.mo_mut(mo);
        m.momx = 0_i32 as fixed_t;
        m.momy = 0_i32 as fixed_t;
    } else {
        let m = state.p_mobj.mo_mut(mo);
        m.momx = FixedMul(m.momx, FRICTION);
        m.momy = FixedMul(m.momy, FRICTION);
    };
}
pub fn P_ZMovement(state: &mut GameState, mo: MobjId) {
    let mut dist: fixed_t = 0;
    let mut delta: fixed_t = 0;
    if state.p_mobj.mo(mo).player.is_some() && state.p_mobj.mo(mo).z < state.p_mobj.mo(mo).floorz {
        let mo_player = state.g_game.player_mut(state.p_mobj.mo(mo).player.unwrap());
        mo_player.viewheight -= state.p_mobj.mo(mo).floorz - state.p_mobj.mo(mo).z;
        mo_player.deltaviewheight = (VIEWHEIGHT - mo_player.viewheight) >> 3_i32;
    }
    state.p_mobj.mo_mut(mo).z += state.p_mobj.mo(mo).momz;
    let mo_target = state
        .p_mobj
        .mo(mo)
        .target
        .filter(|&id| state.p_mobj.is_live(id));
    let mo_flags = state.p_mobj.mo(mo).flags;
    if let Some(target) = mo_target.filter(|_| {
        mo_flags & MF_FLOAT != 0 && mo_flags & MF_SKULLFLY == 0 && mo_flags & MF_INFLOAT == 0
    }) {
        dist = P_AproxDistance(
            state.p_mobj.mo(mo).x - state.p_mobj.mo(target).x,
            state.p_mobj.mo(mo).y - state.p_mobj.mo(target).y,
        );
        delta = state.p_mobj.mo(target).z + (state.p_mobj.mo(mo).height >> 1_i32)
            - state.p_mobj.mo(mo).z;
        if delta < 0_i32 && dist < -(delta * 3_i32) {
            state.p_mobj.mo_mut(mo).z -= FLOATSPEED;
        } else if delta > 0_i32 && dist < delta * 3_i32 {
            state.p_mobj.mo_mut(mo).z += FLOATSPEED;
        }
    }
    if state.p_mobj.mo(mo).z <= state.p_mobj.mo(mo).floorz {
        let mut correct_lost_soul_bounce: i32 =
            (state.doomstat.gameversion.is_ultimate_or_higher()) as i32;
        if correct_lost_soul_bounce != 0 && state.p_mobj.mo(mo).flags & MF_SKULLFLY != 0 {
            state.p_mobj.mo_mut(mo).momz = -state.p_mobj.mo(mo).momz;
        }
        if state.p_mobj.mo(mo).momz < 0_i32 {
            if state.p_mobj.mo(mo).player.is_some() && state.p_mobj.mo(mo).momz < -GRAVITY * 8_i32 {
                state
                    .g_game
                    .player_mut(state.p_mobj.mo(mo).player.unwrap())
                    .deltaviewheight = state.p_mobj.mo(mo).momz >> 3_i32;
                S_StartSound(state, SoundOrigin::Mobj(mo), SfxName::sfx_oof as i32);
            }
            state.p_mobj.mo_mut(mo).momz = 0_i32 as fixed_t;
        }
        state.p_mobj.mo_mut(mo).z = state.p_mobj.mo(mo).floorz;
        if correct_lost_soul_bounce == 0 && state.p_mobj.mo(mo).flags & MF_SKULLFLY != 0 {
            state.p_mobj.mo_mut(mo).momz = -state.p_mobj.mo(mo).momz;
        }
        if state.p_mobj.mo(mo).flags & MF_MISSILE != 0 && state.p_mobj.mo(mo).flags & MF_NOCLIP == 0
        {
            P_ExplodeMissile(state, mo);
            return;
        }
    } else if state.p_mobj.mo(mo).flags & MF_NOGRAVITY == 0 {
        if state.p_mobj.mo(mo).momz == 0_i32 {
            state.p_mobj.mo_mut(mo).momz = (-GRAVITY * 2_i32) as fixed_t;
        } else {
            state.p_mobj.mo_mut(mo).momz -= GRAVITY;
        }
    }
    if state.p_mobj.mo(mo).z + state.p_mobj.mo(mo).height > state.p_mobj.mo(mo).ceilingz {
        if state.p_mobj.mo(mo).momz > 0_i32 {
            state.p_mobj.mo_mut(mo).momz = 0_i32 as fixed_t;
        }
        state.p_mobj.mo_mut(mo).z = state.p_mobj.mo(mo).ceilingz - state.p_mobj.mo(mo).height;
        if state.p_mobj.mo(mo).flags & MF_SKULLFLY != 0 {
            state.p_mobj.mo_mut(mo).momz = -state.p_mobj.mo(mo).momz;
        }
        if state.p_mobj.mo(mo).flags & MF_MISSILE != 0 && state.p_mobj.mo(mo).flags & MF_NOCLIP == 0
        {
            P_ExplodeMissile(state, mo);
        }
    }
}
pub fn P_NightmareRespawn(state: &mut GameState, mobj: MobjId) {
    let spawnpoint = state.p_mobj.mo(mobj).spawnpoint;
    let x = ((spawnpoint.x as i32) << FRACBITS) as fixed_t;
    let y = ((spawnpoint.y as i32) << FRACBITS) as fixed_t;
    if !P_CheckPosition(state, mobj, x, y) {
        return;
    }
    let (mobj_x, mobj_y, mobj_subsector, mobj_type) = {
        let m = state.p_mobj.mo(mobj);
        (m.x, m.y, m.subsector, m.type_0)
    };
    let floorheight1 = state
        .p_setup
        .sector_mut(state.p_setup.subsectors[mobj_subsector.0 as usize].sector)
        .floorheight;
    let fog = P_SpawnMobj(state, mobj_x, mobj_y, floorheight1, MobjType::MT_TFOG);
    S_StartSound(state, SoundOrigin::Mobj(fog), SfxName::sfx_telept as i32);
    let ss = R_PointInSubsector(state, x, y);
    let floorheight2 = state
        .p_setup
        .sector_mut(state.p_setup.subsectors[ss.0 as usize].sector)
        .floorheight;
    let fog = P_SpawnMobj(state, x, y, floorheight2, MobjType::MT_TFOG);
    S_StartSound(state, SoundOrigin::Mobj(fog), SfxName::sfx_telept as i32);
    let z = if state.info.mobjinfo_mut(mobj_type).flags & MF_SPAWNCEILING != 0 {
        ONCEILINGZ as fixed_t
    } else {
        ONFLOORZ as fixed_t
    };
    let mo = P_SpawnMobj(state, x, y, z, mobj_type);
    {
        let m = state.p_mobj.mo_mut(mo);
        m.spawnpoint = spawnpoint;
        m.angle = (ANG45 * (spawnpoint.angle as i32 / 45_i32)) as angle_t;
        if spawnpoint.options as i32 & MTF_AMBUSH != 0 {
            m.flags |= MF_AMBUSH;
        }
        m.reactiontime = 18_i32;
    }
    P_RemoveMobj(state, mobj);
}
pub fn P_MobjThinker(state: &mut GameState, id: MobjId) {
    let removed =
        |state: &GameState| matches!(state.p_mobj.mo(id).thinker.function, ThinkerFn::Removed);
    {
        let m = state.p_mobj.mo(id);
        if m.momx != 0 || m.momy != 0 || m.flags & MF_SKULLFLY != 0 {
            P_XYMovement(state, id);
            if removed(state) {
                return;
            }
        }
    }
    {
        let m = state.p_mobj.mo(id);
        if m.z != m.floorz || m.momz != 0 {
            P_ZMovement(state, id);
            if removed(state) {
                return;
            }
        }
    }
    if state.p_mobj.mo(id).tics != -1_i32 {
        state.p_mobj.mo_mut(id).tics -= 1;
        if state.p_mobj.mo(id).tics == 0 {
            let current = state.p_mobj.mo(id).state.unwrap();
            let nextstate = state.info.state_mut(current).nextstate;
            P_SetMobjState(state, id, nextstate);
        }
    } else {
        if state.p_mobj.mo(id).flags & MF_COUNTKILL == 0 {
            return;
        }
        if !state.g_game.respawnmonsters {
            return;
        }
        state.p_mobj.mo_mut(id).movecount += 1;
        if state.p_mobj.mo(id).movecount < 12_i32 * TICRATE {
            return;
        }
        if state.p_tick.leveltime & 31_i32 != 0 {
            return;
        }
        if P_Random(&mut state.m_random) > 4_i32 {
            return;
        }
        P_NightmareRespawn(state, id);
    }
}
pub fn P_SpawnMobj(
    state: &mut GameState,
    x: fixed_t,
    y: fixed_t,
    z: fixed_t,
    type_0: MobjType,
) -> MobjId {
    // Built as a local value (starting from the same all-defaults template
    // used for PMobjState::dummy_mobj); spawn() moves it into the arena once
    // fully populated.
    let mut value = state.p_mobj.dummy_mobj;
    let (radius, height, flags, spawnhealth, reactiontime, spawnstate) = {
        let info = state.info.mobjinfo_mut(type_0);
        (
            info.radius,
            info.height,
            info.flags,
            info.spawnhealth,
            info.reactiontime,
            info.spawnstate,
        )
    };
    value.type_0 = type_0;
    value.x = x;
    value.y = y;
    value.radius = radius as fixed_t;
    value.height = height as fixed_t;
    value.flags = flags;
    value.health = spawnhealth;
    if state.g_game.gameskill != SkillType::sk_nightmare {
        value.reactiontime = reactiontime;
    }
    value.lastlook = P_Random(&mut state.m_random) % MAXPLAYERS;
    let spawnstate_id = StateId(spawnstate as u32);
    let (tics, sprite, frame) = {
        let st = state.info.state_mut(spawnstate_id);
        (st.tics, st.sprite, st.frame)
    };
    value.state = Some(spawnstate_id);
    value.tics = tics;
    value.sprite = sprite;
    value.frame = frame;
    let id = state.p_mobj.spawn(value);
    P_SetThingPosition(state, id);
    let subsector = state.p_mobj.mo(id).subsector;
    let sector = state.p_setup.subsectors[subsector.0 as usize].sector;
    let (floorz, ceilingz) = {
        let s = state.p_setup.sector_mut(sector);
        (s.floorheight, s.ceilingheight)
    };
    let final_z = if z == ONFLOORZ {
        floorz
    } else if z == ONCEILINGZ {
        (ceilingz - state.info.mobjinfo_mut(type_0).height) as fixed_t
    } else {
        z
    };
    {
        let m = state.p_mobj.mo_mut(id);
        m.floorz = floorz;
        m.ceilingz = ceilingz;
        m.z = final_z;
        m.thinker.function = ThinkerFn::Mobj(P_MobjThinker);
    }
    P_AddThinker(state, ThinkerPayload::Mobj(id), ThinkerKind::Mobj);
    id
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct MobjId {
    index: u32,
    generation: u32,
}

impl MobjId {
    /// Exposes the raw arena slot index. Not meant for constructing or
    /// comparing ids (generation is deliberately hidden for that) -- just
    /// for callers that need a plain distinguishing number, e.g. the
    /// vanilla-demo-compatibility overrun emulation in p_maputl.rs.
    pub fn raw_index(&self) -> u32 {
        self.index
    }
}

struct MobjSlot {
    generation: u32,
    // Owns the mobj's memory (unlike the raw pointer this replaces) -- see
    // PMobjState::spawn/retire/deallocate for why retirement and
    // deallocation are two separate steps despite that.
    mobj: Option<Box<mobj_t>>,
    // Set by retire() (mirrors the old ptr=None, but must NOT drop `mobj`
    // yet -- see deallocate()); is_live() reports "gone" once this is
    // true, matching the old ptr=None-based check exactly.
    retired: bool,
}

pub struct PMobjState {
    // Genuinely unused anywhere in the codebase (confirmed by full-codebase
    // grep) -- a vestigial c2rust-transpiled global. Kept, not deleted:
    // dead-code removal is a different track's mandate, not this one's.
    pub test: i32,
    pub itemrespawnque: [mapthing_t; 128],
    pub itemrespawntime: [i32; 128],
    pub iquehead: i32,
    pub iquetail: i32,
    pub dummy_mobj: mobj_t,
    dummy_id: Option<MobjId>,
    mobjs: Vec<MobjSlot>,
    free_list: Vec<u32>,
}

impl Default for PMobjState {
    fn default() -> Self {
        Self::new()
    }
}

impl PMobjState {
    // Takes a fully-constructed mobj_t *value*, moves it onto the heap (a
    // fresh Box, not a Z_Malloc'd block), and hands back both a stable
    // generation-checked handle and a raw pointer for the caller's
    // remaining post-spawn field writes (P_SetThingPosition, floorz/
    // ceilingz/z, thinker linkage -- exactly as before). The only two call
    // sites are P_SpawnMobj and p_saveg.rs's P_UnArchiveThinkers
    // mobj-reconstruction branch -- the only two places that construct a
    // mobj_t from scratch.
    pub fn spawn(&mut self, mut value: mobj_t) -> MobjId {
        let (index, generation) = if let Some(index) = self.free_list.pop() {
            let slot = &mut self.mobjs[index as usize];
            slot.generation = slot.generation.wrapping_add(1);
            (index, slot.generation)
        } else {
            let index = self.mobjs.len() as u32;
            self.mobjs.push(MobjSlot {
                generation: 0,
                mobj: None,
                retired: false,
            });
            (index, 0)
        };
        let id = MobjId { index, generation };
        value.id = id;
        let slot = &mut self.mobjs[index as usize];
        slot.mobj = Some(Box::new(value));
        slot.retired = false;
        id
    }

    // Logical removal: marks the slot retired so is_live() immediately
    // reports "gone", without touching the backing memory yet. A slot's
    // index is NOT reused (see deallocate()) until the memory is actually
    // freed -- reusing it any earlier, now that the slot *owns* a Box
    // instead of just remembering a Z_Malloc'd address, would drop (and so
    // free) the old mobj's memory out from under whoever still holds its
    // raw pointer (the reaper, mid-deferred-free-schedule; P_RemoveMobj's
    // own remaining body, which keeps dereferencing its `mobj` parameter
    // after calling retire()).
    pub fn retire(&mut self, id: MobjId) {
        if let Some(slot) = self.mobjs.get_mut(id.index as usize) {
            if slot.generation == id.generation {
                slot.retired = true;
            }
        }
    }

    // Actually deallocates: drops the owning Box (freeing the memory) and
    // only now returns the slot's index to the free list for reuse. Called
    // from exactly the two places that used to Z_Free a mobj: P_RunThinkers'
    // reaper, and P_UnArchiveThinkers' pre-load teardown loop.
    pub fn deallocate(&mut self, id: MobjId) {
        if let Some(slot) = self.mobjs.get_mut(id.index as usize) {
            if slot.generation == id.generation {
                slot.mobj = None;
                self.free_list.push(id.index);
            }
        }
    }

    // Safe borrows. These still resolve a retired-but-not-yet-deallocated
    // mobj (the memory stays valid until the reaper runs) -- e.g. a blockmap
    // iteration must read `bnext` of a mobj its own callback just removed.
    // None only for a stale id (freed / slot reused); is_live() is the
    // stricter check that also treats a retired mobj as gone.
    pub fn mobj_ref(&self, id: MobjId) -> Option<&mobj_t> {
        self.mobjs
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.mobj.as_deref())
    }

    pub fn mobj_mut(&mut self, id: MobjId) -> Option<&mut mobj_t> {
        self.mobjs
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.mobj.as_deref_mut())
    }

    // Panicking shorthands over mobj_ref/mobj_mut for ids that must be live.
    pub fn mo(&self, id: MobjId) -> &mobj_t {
        self.mobj_ref(id).expect("stale MobjId")
    }

    pub fn mo_mut(&mut self, id: MobjId) -> &mut mobj_t {
        self.mobj_mut(id).expect("stale MobjId")
    }

    // Same liveness rule as is_live() (a retired mobj is "gone"), for the
    // `target.filter(|id| is_live(*id))` validity checks.
    pub fn is_live(&self, id: MobjId) -> bool {
        self.mobjs.get(id.index as usize).is_some_and(|slot| {
            slot.generation == id.generation && !slot.retired && slot.mobj.is_some()
        })
    }

    pub const fn new() -> Self {
        PMobjState {
            test: 0,
            itemrespawnque: [mapthing_t {
                x: 0,
                y: 0,
                angle: 0,
                type_0: 0,
                options: 0,
            }; 128],
            itemrespawntime: [0; 128],
            iquehead: 0,
            iquetail: 0,
            mobjs: Vec::new(),
            free_list: Vec::new(),
            dummy_id: None,
            dummy_mobj: mobj_s {
                thinker: thinker_s {
                    function: ThinkerFn::Paused,
                },
                x: 0,
                y: 0,
                z: 0,
                snext: None,
                sprev: None,
                angle: 0,
                sprite: SpriteNum::SPR_TROO,
                frame: 0,
                bnext: None,
                bprev: None,
                subsector: SubsectorId(0),
                floorz: 0,
                ceilingz: 0,
                radius: 0,
                height: 0,
                momx: 0,
                momy: 0,
                momz: 0,
                validcount: 0,
                type_0: MobjType::MT_PLAYER,
                tics: 0,
                state: None,
                flags: 0,
                health: 0,
                movedir: 0,
                movecount: 0,
                target: None,
                reactiontime: 0,
                threshold: 0,
                player: None,
                lastlook: 0,
                spawnpoint: mapthing_t {
                    x: 0,
                    y: 0,
                    angle: 0,
                    type_0: 0,
                    options: 0,
                },
                tracer: None,
                id: MobjId {
                    index: 0,
                    generation: 0,
                },
            },
        }
    }
}

pub fn P_RemoveMobj(state: &mut GameState, mobj: MobjId) {
    state.p_mobj.retire(mobj);
    let (flags, type_0, spawnpoint) = {
        let m = state.p_mobj.mo(mobj);
        (m.flags, m.type_0, m.spawnpoint)
    };
    if flags & MF_SPECIAL != 0
        && flags & MF_DROPPED == 0
        && type_0 as u32 != MobjType::MT_INV as i32 as u32
        && type_0 as u32 != MobjType::MT_INS as i32 as u32
    {
        state.p_mobj.itemrespawnque[state.p_mobj.iquehead as usize] = spawnpoint;
        state.p_mobj.itemrespawntime[state.p_mobj.iquehead as usize] = state.p_tick.leveltime;
        state.p_mobj.iquehead = (state.p_mobj.iquehead + 1_i32) & (ITEMQUESIZE - 1_i32);
        if state.p_mobj.iquehead == state.p_mobj.iquetail {
            state.p_mobj.iquetail = (state.p_mobj.iquetail + 1_i32) & (ITEMQUESIZE - 1_i32);
        }
    }
    P_UnsetThingPosition(state, mobj);
    S_StopSound(state, SoundOrigin::Mobj(mobj));
    P_RemoveThinker(&mut state.p_mobj.mo_mut(mobj).thinker);
}
pub fn P_RespawnSpecials(state: &mut GameState) {
    if state.g_game.deathmatch != 2_i32 {
        return;
    }
    if state.p_mobj.iquehead == state.p_mobj.iquetail {
        return;
    }
    if state.p_tick.leveltime - state.p_mobj.itemrespawntime[state.p_mobj.iquetail as usize]
        < 30_i32 * TICRATE
    {
        return;
    }
    let mthing = state.p_mobj.itemrespawnque[state.p_mobj.iquetail as usize];
    let x = ((mthing.x as i32) << FRACBITS) as fixed_t;
    let y = ((mthing.y as i32) << FRACBITS) as fixed_t;
    let ss = R_PointInSubsector(state, x, y);
    let floorheight = state
        .p_setup
        .sector_mut(state.p_setup.subsectors[ss.0 as usize].sector)
        .floorheight;
    let fog = P_SpawnMobj(state, x, y, floorheight, MobjType::MT_IFOG);
    S_StartSound(state, SoundOrigin::Mobj(fog), SfxName::sfx_itmbk as i32);
    let mut i: i32 = 0_i32;
    while i < NUMMOBJTYPES {
        if mthing.type_0 as i32 == state.info.mobjinfo[i as usize].doomednum {
            break;
        }
        i += 1;
    }
    let z = if state.info.mobjinfo[i as usize].flags & MF_SPAWNCEILING != 0 {
        ONCEILINGZ as fixed_t
    } else {
        ONFLOORZ as fixed_t
    };
    let mo = P_SpawnMobj(state, x, y, z, mobjtype_from_raw(i));
    {
        let m = state.p_mobj.mo_mut(mo);
        m.spawnpoint = mthing;
        m.angle = (ANG45 * (mthing.angle as i32 / 45_i32)) as angle_t;
    }
    state.p_mobj.iquetail = (state.p_mobj.iquetail + 1_i32) & (ITEMQUESIZE - 1_i32);
}
pub fn P_SpawnPlayer(state: &mut GameState, mthing: mapthing_t) {
    if mthing.type_0 as i32 == 0_i32 {
        return;
    }
    let player_index = (mthing.type_0 as i32 - 1_i32) as usize;
    if !state.g_game.playeringame[player_index] {
        return;
    }
    if state.g_game.players[player_index].playerstate == PlayerState::PST_REBORN {
        G_PlayerReborn(&mut state.g_game, mthing.type_0 as i32 - 1_i32);
    }
    let x = ((mthing.x as i32) << FRACBITS) as fixed_t;
    let y = ((mthing.y as i32) << FRACBITS) as fixed_t;
    let z = ONFLOORZ as fixed_t;
    let mobj = P_SpawnMobj(state, x, y, z, MobjType::MT_PLAYER);
    let player_health = state.g_game.players[player_index].health;
    {
        let m = state.p_mobj.mo_mut(mobj);
        if mthing.type_0 as i32 > 1_i32 {
            m.flags |= (mthing.type_0 as i32 - 1_i32) << MF_TRANSSHIFT;
        }
        m.angle = (ANG45 * (mthing.angle as i32 / 45_i32)) as angle_t;
        m.player = Some(PlayerId(player_index as u8));
        m.health = player_health;
    }
    {
        let p = &mut state.g_game.players[player_index];
        p.mo = Some(mobj);
        p.playerstate = PlayerState::PST_LIVE;
        p.refire = 0_i32;
        p.message = None;
        p.damagecount = 0_i32;
        p.bonuscount = 0_i32;
        p.extralight = 0_i32;
        p.fixedcolormap = 0_i32;
        p.viewheight = VIEWHEIGHT as fixed_t;
    }
    P_SetupPsprites(state, PlayerId(player_index as u8));
    if state.g_game.deathmatch != 0 {
        for i in 0..NUMCARDS {
            state.g_game.players[player_index].cards[i as usize] = true;
        }
    }
    if mthing.type_0 as i32 - 1_i32 == state.g_game.consoleplayer {
        {
            ST_Start(state);
            HU_Start(state);
        }
    }
}
pub fn P_SpawnMapThing(state: &mut GameState, mthing: mapthing_t) {
    if mthing.type_0 as i32 == 11_i32 {
        if state.p_setup.deathmatch_p < 10 {
            let idx = state.p_setup.deathmatch_p;
            state.p_setup.deathmatchstarts[idx] = mthing;
            state.p_setup.deathmatch_p += 1;
        }
        return;
    }
    if mthing.type_0 as i32 <= 0_i32 {
        return;
    }
    if mthing.type_0 as i32 <= 4_i32 {
        state.p_setup.playerstarts[(mthing.type_0 as i32 - 1_i32) as usize] = mthing;
        if state.g_game.deathmatch == 0 {
            P_SpawnPlayer(state, mthing);
        }
        return;
    }
    if !state.g_game.netgame && mthing.options as i32 & 16_i32 != 0 {
        return;
    }
    let bit: i32 = if state.g_game.gameskill == SkillType::sk_baby {
        1_i32
    } else if state.g_game.gameskill == SkillType::sk_nightmare {
        4_i32
    } else {
        1_i32 << (state.g_game.gameskill as i32 - 1_i32)
    };
    if mthing.options as i32 & bit == 0 {
        return;
    }
    let mut i: i32 = 0_i32;
    while i < NUMMOBJTYPES {
        if mthing.type_0 as i32 == state.info.mobjinfo[i as usize].doomednum {
            break;
        }
        i += 1;
    }
    if i == NUMMOBJTYPES {
        I_Error(&format!(
            "P_SpawnMapThing: Unknown type {} at ({}, {})",
            mthing.type_0 as i32, mthing.x as i32, mthing.y as i32,
        ));
    }
    if state.g_game.deathmatch != 0 && state.info.mobjinfo[i as usize].flags & MF_NOTDMATCH != 0 {
        return;
    }
    if state.d_main.nomonsters
        && (i == MobjType::MT_SKULL as i32
            || state.info.mobjinfo[i as usize].flags & MF_COUNTKILL != 0)
    {
        return;
    }
    let x = ((mthing.x as i32) << FRACBITS) as fixed_t;
    let y = ((mthing.y as i32) << FRACBITS) as fixed_t;
    let z = if state.info.mobjinfo[i as usize].flags & MF_SPAWNCEILING != 0 {
        ONCEILINGZ as fixed_t
    } else {
        ONFLOORZ as fixed_t
    };
    let mobj = P_SpawnMobj(state, x, y, z, mobjtype_from_raw(i));
    state.p_mobj.mo_mut(mobj).spawnpoint = mthing;
    if state.p_mobj.mo(mobj).tics > 0_i32 {
        let tics = state.p_mobj.mo(mobj).tics;
        state.p_mobj.mo_mut(mobj).tics = 1_i32 + P_Random(&mut state.m_random) % tics;
    }
    let flags = state.p_mobj.mo(mobj).flags;
    if flags & MF_COUNTKILL != 0 {
        state.g_game.totalkills += 1;
    }
    if flags & MF_COUNTITEM != 0 {
        state.g_game.totalitems += 1;
    }
    let m = state.p_mobj.mo_mut(mobj);
    m.angle = (ANG45 * (mthing.angle as i32 / 45_i32)) as angle_t;
    if mthing.options as i32 & MTF_AMBUSH != 0 {
        m.flags |= MF_AMBUSH;
    }
}
pub fn P_SpawnPuff(state: &mut GameState, x: fixed_t, y: fixed_t, mut z: fixed_t) {
    z += (P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 10_i32;
    let th = P_SpawnMobj(state, x, y, z, MobjType::MT_PUFF);
    state.p_mobj.mo_mut(th).momz = FRACUNIT as fixed_t;
    state.p_mobj.mo_mut(th).tics -= P_Random(&mut state.m_random) & 3_i32;
    if state.p_mobj.mo(th).tics < 1_i32 {
        state.p_mobj.mo_mut(th).tics = 1_i32;
    }
    if state.p_map.attackrange == MELEERANGE {
        P_SetMobjState(state, th, StateNum::S_PUFF3);
    }
}
pub fn P_SpawnBlood(state: &mut GameState, x: fixed_t, y: fixed_t, mut z: fixed_t, damage: i32) {
    z += (P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 10_i32;
    let th = P_SpawnMobj(state, x, y, z, MobjType::MT_BLOOD);
    state.p_mobj.mo_mut(th).momz = (FRACUNIT * 2_i32) as fixed_t;
    state.p_mobj.mo_mut(th).tics -= P_Random(&mut state.m_random) & 3_i32;
    if state.p_mobj.mo(th).tics < 1_i32 {
        state.p_mobj.mo_mut(th).tics = 1_i32;
    }
    if (9_i32..=12_i32).contains(&damage) {
        P_SetMobjState(state, th, StateNum::S_BLOOD2);
    } else if damage < 9_i32 {
        P_SetMobjState(state, th, StateNum::S_BLOOD3);
    }
}
pub fn P_CheckMissileSpawn(state: &mut GameState, th: MobjId) {
    state.p_mobj.mo_mut(th).tics -= P_Random(&mut state.m_random) & 3_i32;
    let (x, y) = {
        let t = state.p_mobj.mo_mut(th);
        if t.tics < 1_i32 {
            t.tics = 1_i32;
        }
        t.x += t.momx >> 1_i32;
        t.y += t.momy >> 1_i32;
        t.z += t.momz >> 1_i32;
        (t.x, t.y)
    };
    if !P_TryMove(state, th, x, y) {
        P_ExplodeMissile(state, th);
    }
}
// Stand-in for a missing target (vanilla's `dummy_mobj`): a permanent arena
// slot, so callers can treat it like any other mobj id. Its position/flags
// are reset on every substitution, as vanilla does.
pub fn P_SubstNullMobj(state: &mut PMobjState, mobj: Option<MobjId>) -> MobjId {
    if let Some(id) = mobj {
        return id;
    }
    let id = match state.dummy_id.filter(|&id| state.mobj_ref(id).is_some()) {
        Some(id) => id,
        None => {
            let template = state.dummy_mobj;
            let id = state.spawn(template);
            state.dummy_id = Some(id);
            id
        }
    };
    let dummy = state.mo_mut(id);
    dummy.x = 0_i32 as fixed_t;
    dummy.y = 0_i32 as fixed_t;
    dummy.z = 0_i32 as fixed_t;
    dummy.flags = 0_i32;
    id
}
pub fn P_SpawnMissile(
    state: &mut GameState,
    source: MobjId,
    dest: MobjId,
    type_0: MobjType,
) -> MobjId {
    let (sx, sy, sz) = {
        let s = state.p_mobj.mo(source);
        (s.x, s.y, s.z)
    };
    let (dx, dy, dz, dflags) = {
        let d = state.p_mobj.mo(dest);
        (d.x, d.y, d.z, d.flags)
    };
    let th = P_SpawnMobj(
        state,
        sx,
        sy,
        sz + 4 as fixed_t * 8 as fixed_t * FRACUNIT,
        type_0,
    );
    let th_type = state.p_mobj.mo(th).type_0;
    let seesound = state.info.mobjinfo_mut(th_type).seesound;
    if seesound != 0 {
        S_StartSound(state, SoundOrigin::Mobj(th), seesound);
    }
    state.p_mobj.mo_mut(th).target = Some(source);
    let mut an: angle_t = R_PointToAngle2(state, sx, sy, dx, dy);
    if dflags & MF_SHADOW != 0 {
        an = an.wrapping_add(
            ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 20_i32) as angle_t,
        );
    }
    state.p_mobj.mo_mut(th).angle = an;
    an >>= ANGLETOFINESHIFT;
    let speed = state.info.mobjinfo_mut(th_type).speed;
    {
        let t = state.p_mobj.mo_mut(th);
        t.momx = FixedMul(speed as fixed_t, finecosine[an as usize]);
        t.momy = FixedMul(speed as fixed_t, finesine[an as usize]);
    }
    let mut dist: i32 = P_AproxDistance(dx - sx, dy - sy);
    dist /= speed;
    if dist < 1_i32 {
        dist = 1_i32;
    }
    state.p_mobj.mo_mut(th).momz = ((dz - sz) / dist) as fixed_t;
    P_CheckMissileSpawn(state, th);
    th
}

pub fn P_SpawnPlayerMissile(state: &mut GameState, source: MobjId, type_0: MobjType) {
    let mut an: angle_t = state.p_mobj.mo(source).angle;
    let mut slope = P_AimLineAttack(
        state,
        Some(source),
        an,
        16 as fixed_t * 64 as fixed_t * FRACUNIT,
    );
    if state.p_map.linetarget.is_none() {
        an = an.wrapping_add((1_i32 << 26_i32) as angle_t);
        slope = P_AimLineAttack(
            state,
            Some(source),
            an,
            16 as fixed_t * 64 as fixed_t * FRACUNIT,
        );
        if state.p_map.linetarget.is_none() {
            an = an.wrapping_sub((2_i32 << 26_i32) as angle_t);
            slope = P_AimLineAttack(
                state,
                Some(source),
                an,
                16 as fixed_t * 64 as fixed_t * FRACUNIT,
            );
        }
        if state.p_map.linetarget.is_none() {
            an = state.p_mobj.mo(source).angle;
            slope = 0_i32 as fixed_t;
        }
    }
    let (x, y, z) = {
        let s = state.p_mobj.mo(source);
        (s.x, s.y, (s.z + 4_i32 * 8_i32 * FRACUNIT) as fixed_t)
    };
    let th = P_SpawnMobj(state, x, y, z, type_0);
    let th_type = state.p_mobj.mo(th).type_0;
    let seesound = state.info.mobjinfo_mut(th_type).seesound;
    if seesound != 0 {
        S_StartSound(state, SoundOrigin::Mobj(th), seesound);
    }
    let speed = state.info.mobjinfo_mut(th_type).speed;
    {
        let t = state.p_mobj.mo_mut(th);
        t.target = Some(source);
        t.angle = an;
        t.momx = FixedMul(
            speed as fixed_t,
            finecosine[(an >> ANGLETOFINESHIFT) as usize],
        );
        t.momy = FixedMul(
            speed as fixed_t,
            finesine[(an >> ANGLETOFINESHIFT) as usize],
        );
        t.momz = FixedMul(speed as fixed_t, slope);
    }
    P_CheckMissileSpawn(state, th);
}
