use crate::d_mode::SkillType;
use crate::d_player::CheatFlags;
use crate::enum_array::EnumArray;
use crate::m_bbox::BBox;
use crate::m_bbox::BoxIndex;

use crate::doomdef::MAXPLAYERS;
use crate::doomdef::TICRATE;
use crate::g_game::player_reborn;
use crate::game_state::GameState;
use crate::hu_stuff::hu_start;
use crate::i_system::error;
use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::info::StateId;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::m_fixed::INT_MAX;
use crate::m_fixed::INT_MIN;
use crate::m_random::p_random;

use crate::p_ceilng::CeilingId;
use crate::p_doors::DoorId;
use crate::p_enemy::MELEERANGE;
use crate::p_inter::NUMCARDS;
use crate::p_lights::{FireFlickerId, GlowId, LightFlashId, StrobeId};
use crate::p_map::aim_line_attack;
use crate::p_map::check_position;
use crate::p_map::slide_move;
use crate::p_map::try_move;
use crate::p_maputl::aprox_distance;
use crate::p_maputl::set_thing_position;
use crate::p_maputl::unset_thing_position;
use crate::p_plats::PlatId;
use crate::p_pspr::setup_psprites;
use crate::p_setup::{LineId, SectorId, SubsectorId, VertexId};
use crate::p_spec::FloorId;
use crate::p_tick::add_thinker;
use crate::p_tick::remove_thinker;
use crate::p_tick::ThinkerId;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::p_user::VIEWHEIGHT;
use crate::r_main::point_in_subsector;
use crate::r_main::point_to_angle2;
use crate::s_sound::s_start_sound;
use crate::s_sound::s_stop_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::st_stuff::st_start;

use crate::tables::Angle;
use crate::tables::ANG45;
use crate::tables::FINECOSINE;
use crate::tables::FINESINE;

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
pub struct Thinker {
    pub function: ThinkerFn,
}
#[derive(Copy, Clone)]
pub struct MapThing {
    pub x: i16,
    pub y: i16,
    pub angle: i16,
    pub kind: i16,
    pub options: i16,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SpriteNum {
    Troo,
    Shtg,
    Pung,
    Pisg,
    Pisf,
    Shtf,
    Sht2,
    Chgg,
    Chgf,
    Misg,
    Misf,
    Sawg,
    Plsg,
    Plsf,
    Bfgg,
    Bfgf,
    Blud,
    Puff,
    Bal1,
    Bal2,
    Plss,
    Plse,
    Misl,
    Bfs1,
    Bfe1,
    Bfe2,
    Tfog,
    Ifog,
    Play,
    Poss,
    Spos,
    Vile,
    Fire,
    Fatb,
    Fbxp,
    Skel,
    Manf,
    Fatt,
    Cpos,
    Sarg,
    Head,
    Bal7,
    Boss,
    Bos2,
    Skul,
    Spid,
    Bspi,
    Apls,
    Apbx,
    Cybr,
    Pain,
    Sswv,
    Keen,
    Bbrn,
    Bosf,
    Arm1,
    Arm2,
    Bar1,
    Bexp,
    Fcan,
    Bon1,
    Bon2,
    Bkey,
    Rkey,
    Ykey,
    Bsku,
    Rsku,
    Ysku,
    Stim,
    Medi,
    Soul,
    Pinv,
    Pstr,
    Pins,
    Mega,
    Suit,
    Pmap,
    Pvis,
    Clip,
    Ammo,
    Rock,
    Brok,
    Cell,
    Celp,
    Shel,
    Sbox,
    Bpak,
    Bfug,
    Mgun,
    Csaw,
    Laun,
    Plas,
    Shot,
    Sgn2,
    Colu,
    Smt2,
    Gor1,
    Pol2,
    Pol5,
    Pol4,
    Pol3,
    Pol1,
    Pol6,
    Gor2,
    Gor3,
    Gor4,
    Gor5,
    Smit,
    Col1,
    Col2,
    Col3,
    Col4,
    Cand,
    Cbra,
    Col6,
    Tre1,
    Tre2,
    Elec,
    Ceye,
    Fsku,
    Col5,
    Tblu,
    Tgrn,
    Tred,
    Smbt,
    Smgt,
    Smrt,
    Hdb1,
    Hdb2,
    Hdb3,
    Hdb4,
    Hdb5,
    Hdb6,
    Pob1,
    Pob2,
    Brs1,
    Tlmp,
    Tlp2,
}
pub fn spritenum_from_raw(v: i32) -> SpriteNum {
    match v {
        0 => SpriteNum::Troo,
        1 => SpriteNum::Shtg,
        2 => SpriteNum::Pung,
        3 => SpriteNum::Pisg,
        4 => SpriteNum::Pisf,
        5 => SpriteNum::Shtf,
        6 => SpriteNum::Sht2,
        7 => SpriteNum::Chgg,
        8 => SpriteNum::Chgf,
        9 => SpriteNum::Misg,
        10 => SpriteNum::Misf,
        11 => SpriteNum::Sawg,
        12 => SpriteNum::Plsg,
        13 => SpriteNum::Plsf,
        14 => SpriteNum::Bfgg,
        15 => SpriteNum::Bfgf,
        16 => SpriteNum::Blud,
        17 => SpriteNum::Puff,
        18 => SpriteNum::Bal1,
        19 => SpriteNum::Bal2,
        20 => SpriteNum::Plss,
        21 => SpriteNum::Plse,
        22 => SpriteNum::Misl,
        23 => SpriteNum::Bfs1,
        24 => SpriteNum::Bfe1,
        25 => SpriteNum::Bfe2,
        26 => SpriteNum::Tfog,
        27 => SpriteNum::Ifog,
        28 => SpriteNum::Play,
        29 => SpriteNum::Poss,
        30 => SpriteNum::Spos,
        31 => SpriteNum::Vile,
        32 => SpriteNum::Fire,
        33 => SpriteNum::Fatb,
        34 => SpriteNum::Fbxp,
        35 => SpriteNum::Skel,
        36 => SpriteNum::Manf,
        37 => SpriteNum::Fatt,
        38 => SpriteNum::Cpos,
        39 => SpriteNum::Sarg,
        40 => SpriteNum::Head,
        41 => SpriteNum::Bal7,
        42 => SpriteNum::Boss,
        43 => SpriteNum::Bos2,
        44 => SpriteNum::Skul,
        45 => SpriteNum::Spid,
        46 => SpriteNum::Bspi,
        47 => SpriteNum::Apls,
        48 => SpriteNum::Apbx,
        49 => SpriteNum::Cybr,
        50 => SpriteNum::Pain,
        51 => SpriteNum::Sswv,
        52 => SpriteNum::Keen,
        53 => SpriteNum::Bbrn,
        54 => SpriteNum::Bosf,
        55 => SpriteNum::Arm1,
        56 => SpriteNum::Arm2,
        57 => SpriteNum::Bar1,
        58 => SpriteNum::Bexp,
        59 => SpriteNum::Fcan,
        60 => SpriteNum::Bon1,
        61 => SpriteNum::Bon2,
        62 => SpriteNum::Bkey,
        63 => SpriteNum::Rkey,
        64 => SpriteNum::Ykey,
        65 => SpriteNum::Bsku,
        66 => SpriteNum::Rsku,
        67 => SpriteNum::Ysku,
        68 => SpriteNum::Stim,
        69 => SpriteNum::Medi,
        70 => SpriteNum::Soul,
        71 => SpriteNum::Pinv,
        72 => SpriteNum::Pstr,
        73 => SpriteNum::Pins,
        74 => SpriteNum::Mega,
        75 => SpriteNum::Suit,
        76 => SpriteNum::Pmap,
        77 => SpriteNum::Pvis,
        78 => SpriteNum::Clip,
        79 => SpriteNum::Ammo,
        80 => SpriteNum::Rock,
        81 => SpriteNum::Brok,
        82 => SpriteNum::Cell,
        83 => SpriteNum::Celp,
        84 => SpriteNum::Shel,
        85 => SpriteNum::Sbox,
        86 => SpriteNum::Bpak,
        87 => SpriteNum::Bfug,
        88 => SpriteNum::Mgun,
        89 => SpriteNum::Csaw,
        90 => SpriteNum::Laun,
        91 => SpriteNum::Plas,
        92 => SpriteNum::Shot,
        93 => SpriteNum::Sgn2,
        94 => SpriteNum::Colu,
        95 => SpriteNum::Smt2,
        96 => SpriteNum::Gor1,
        97 => SpriteNum::Pol2,
        98 => SpriteNum::Pol5,
        99 => SpriteNum::Pol4,
        100 => SpriteNum::Pol3,
        101 => SpriteNum::Pol1,
        102 => SpriteNum::Pol6,
        103 => SpriteNum::Gor2,
        104 => SpriteNum::Gor3,
        105 => SpriteNum::Gor4,
        106 => SpriteNum::Gor5,
        107 => SpriteNum::Smit,
        108 => SpriteNum::Col1,
        109 => SpriteNum::Col2,
        110 => SpriteNum::Col3,
        111 => SpriteNum::Col4,
        112 => SpriteNum::Cand,
        113 => SpriteNum::Cbra,
        114 => SpriteNum::Col6,
        115 => SpriteNum::Tre1,
        116 => SpriteNum::Tre2,
        117 => SpriteNum::Elec,
        118 => SpriteNum::Ceye,
        119 => SpriteNum::Fsku,
        120 => SpriteNum::Col5,
        121 => SpriteNum::Tblu,
        122 => SpriteNum::Tgrn,
        123 => SpriteNum::Tred,
        124 => SpriteNum::Smbt,
        125 => SpriteNum::Smgt,
        126 => SpriteNum::Smrt,
        127 => SpriteNum::Hdb1,
        128 => SpriteNum::Hdb2,
        129 => SpriteNum::Hdb3,
        130 => SpriteNum::Hdb4,
        131 => SpriteNum::Hdb5,
        132 => SpriteNum::Hdb6,
        133 => SpriteNum::Pob1,
        134 => SpriteNum::Pob2,
        135 => SpriteNum::Brs1,
        136 => SpriteNum::Tlmp,
        137 => SpriteNum::Tlp2,
        n => panic!("invalid spritenum {n}"),
    }
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StateNum {
    Null,
    Lightdone,
    Punch,
    Punchdown,
    Punchup,
    Punch1,
    Punch2,
    Punch3,
    Punch4,
    Punch5,
    Pistol,
    Pistoldown,
    Pistolup,
    Pistol1,
    Pistol2,
    Pistol3,
    Pistol4,
    Pistolflash,
    Sgun,
    Sgundown,
    Sgunup,
    Sgun1,
    Sgun2,
    Sgun3,
    Sgun4,
    Sgun5,
    Sgun6,
    Sgun7,
    Sgun8,
    Sgun9,
    Sgunflash1,
    Sgunflash2,
    Dsgun,
    Dsgundown,
    Dsgunup,
    Dsgun1,
    Dsgun2,
    Dsgun3,
    Dsgun4,
    Dsgun5,
    Dsgun6,
    Dsgun7,
    Dsgun8,
    Dsgun9,
    Dsgun10,
    Dsnr1,
    Dsnr2,
    Dsgunflash1,
    Dsgunflash2,
    Chain,
    Chaindown,
    Chainup,
    Chain1,
    Chain2,
    Chain3,
    Chainflash1,
    Chainflash2,
    Missile,
    Missiledown,
    Missileup,
    Missile1,
    Missile2,
    Missile3,
    Missileflash1,
    Missileflash2,
    Missileflash3,
    Missileflash4,
    Saw,
    Sawb,
    Sawdown,
    Sawup,
    Saw1,
    Saw2,
    Saw3,
    Plasma,
    Plasmadown,
    Plasmaup,
    Plasma1,
    Plasma2,
    Plasmaflash1,
    Plasmaflash2,
    Bfg,
    Bfgdown,
    Bfgup,
    Bfg1,
    Bfg2,
    Bfg3,
    Bfg4,
    Bfgflash1,
    Bfgflash2,
    Blood1,
    Blood2,
    Blood3,
    Puff1,
    Puff2,
    Puff3,
    Puff4,
    Tball1,
    Tball2,
    Tballx1,
    Tballx2,
    Tballx3,
    Rball1,
    Rball2,
    Rballx1,
    Rballx2,
    Rballx3,
    Plasball,
    Plasball2,
    Plasexp,
    Plasexp2,
    Plasexp3,
    Plasexp4,
    Plasexp5,
    Rocket,
    Bfgshot,
    Bfgshot2,
    Bfgland,
    Bfgland2,
    Bfgland3,
    Bfgland4,
    Bfgland5,
    Bfgland6,
    Bfgexp,
    Bfgexp2,
    Bfgexp3,
    Bfgexp4,
    Explode1,
    Explode2,
    Explode3,
    Tfog,
    Tfog01,
    Tfog02,
    Tfog2,
    Tfog3,
    Tfog4,
    Tfog5,
    Tfog6,
    Tfog7,
    Tfog8,
    Tfog9,
    Tfog10,
    Ifog,
    Ifog01,
    Ifog02,
    Ifog2,
    Ifog3,
    Ifog4,
    Ifog5,
    Play,
    PlayRun1,
    PlayRun2,
    PlayRun3,
    PlayRun4,
    PlayAtk1,
    PlayAtk2,
    PlayPain,
    PlayPain2,
    PlayDie1,
    PlayDie2,
    PlayDie3,
    PlayDie4,
    PlayDie5,
    PlayDie6,
    PlayDie7,
    PlayXdie1,
    PlayXdie2,
    PlayXdie3,
    PlayXdie4,
    PlayXdie5,
    PlayXdie6,
    PlayXdie7,
    PlayXdie8,
    PlayXdie9,
    PossStnd,
    PossStnd2,
    PossRun1,
    PossRun2,
    PossRun3,
    PossRun4,
    PossRun5,
    PossRun6,
    PossRun7,
    PossRun8,
    PossAtk1,
    PossAtk2,
    PossAtk3,
    PossPain,
    PossPain2,
    PossDie1,
    PossDie2,
    PossDie3,
    PossDie4,
    PossDie5,
    PossXdie1,
    PossXdie2,
    PossXdie3,
    PossXdie4,
    PossXdie5,
    PossXdie6,
    PossXdie7,
    PossXdie8,
    PossXdie9,
    PossRaise1,
    PossRaise2,
    PossRaise3,
    PossRaise4,
    SposStnd,
    SposStnd2,
    SposRun1,
    SposRun2,
    SposRun3,
    SposRun4,
    SposRun5,
    SposRun6,
    SposRun7,
    SposRun8,
    SposAtk1,
    SposAtk2,
    SposAtk3,
    SposPain,
    SposPain2,
    SposDie1,
    SposDie2,
    SposDie3,
    SposDie4,
    SposDie5,
    SposXdie1,
    SposXdie2,
    SposXdie3,
    SposXdie4,
    SposXdie5,
    SposXdie6,
    SposXdie7,
    SposXdie8,
    SposXdie9,
    SposRaise1,
    SposRaise2,
    SposRaise3,
    SposRaise4,
    SposRaise5,
    VileStnd,
    VileStnd2,
    VileRun1,
    VileRun2,
    VileRun3,
    VileRun4,
    VileRun5,
    VileRun6,
    VileRun7,
    VileRun8,
    VileRun9,
    VileRun10,
    VileRun11,
    VileRun12,
    VileAtk1,
    VileAtk2,
    VileAtk3,
    VileAtk4,
    VileAtk5,
    VileAtk6,
    VileAtk7,
    VileAtk8,
    VileAtk9,
    VileAtk10,
    VileAtk11,
    VileHeal1,
    VileHeal2,
    VileHeal3,
    VilePain,
    VilePain2,
    VileDie1,
    VileDie2,
    VileDie3,
    VileDie4,
    VileDie5,
    VileDie6,
    VileDie7,
    VileDie8,
    VileDie9,
    VileDie10,
    Fire1,
    Fire2,
    Fire3,
    Fire4,
    Fire5,
    Fire6,
    Fire7,
    Fire8,
    Fire9,
    Fire10,
    Fire11,
    Fire12,
    Fire13,
    Fire14,
    Fire15,
    Fire16,
    Fire17,
    Fire18,
    Fire19,
    Fire20,
    Fire21,
    Fire22,
    Fire23,
    Fire24,
    Fire25,
    Fire26,
    Fire27,
    Fire28,
    Fire29,
    Fire30,
    Smoke1,
    Smoke2,
    Smoke3,
    Smoke4,
    Smoke5,
    Tracer,
    Tracer2,
    Traceexp1,
    Traceexp2,
    Traceexp3,
    SkelStnd,
    SkelStnd2,
    SkelRun1,
    SkelRun2,
    SkelRun3,
    SkelRun4,
    SkelRun5,
    SkelRun6,
    SkelRun7,
    SkelRun8,
    SkelRun9,
    SkelRun10,
    SkelRun11,
    SkelRun12,
    SkelFist1,
    SkelFist2,
    SkelFist3,
    SkelFist4,
    SkelMiss1,
    SkelMiss2,
    SkelMiss3,
    SkelMiss4,
    SkelPain,
    SkelPain2,
    SkelDie1,
    SkelDie2,
    SkelDie3,
    SkelDie4,
    SkelDie5,
    SkelDie6,
    SkelRaise1,
    SkelRaise2,
    SkelRaise3,
    SkelRaise4,
    SkelRaise5,
    SkelRaise6,
    Fatshot1,
    Fatshot2,
    Fatshotx1,
    Fatshotx2,
    Fatshotx3,
    FattStnd,
    FattStnd2,
    FattRun1,
    FattRun2,
    FattRun3,
    FattRun4,
    FattRun5,
    FattRun6,
    FattRun7,
    FattRun8,
    FattRun9,
    FattRun10,
    FattRun11,
    FattRun12,
    FattAtk1,
    FattAtk2,
    FattAtk3,
    FattAtk4,
    FattAtk5,
    FattAtk6,
    FattAtk7,
    FattAtk8,
    FattAtk9,
    FattAtk10,
    FattPain,
    FattPain2,
    FattDie1,
    FattDie2,
    FattDie3,
    FattDie4,
    FattDie5,
    FattDie6,
    FattDie7,
    FattDie8,
    FattDie9,
    FattDie10,
    FattRaise1,
    FattRaise2,
    FattRaise3,
    FattRaise4,
    FattRaise5,
    FattRaise6,
    FattRaise7,
    FattRaise8,
    CposStnd,
    CposStnd2,
    CposRun1,
    CposRun2,
    CposRun3,
    CposRun4,
    CposRun5,
    CposRun6,
    CposRun7,
    CposRun8,
    CposAtk1,
    CposAtk2,
    CposAtk3,
    CposAtk4,
    CposPain,
    CposPain2,
    CposDie1,
    CposDie2,
    CposDie3,
    CposDie4,
    CposDie5,
    CposDie6,
    CposDie7,
    CposXdie1,
    CposXdie2,
    CposXdie3,
    CposXdie4,
    CposXdie5,
    CposXdie6,
    CposRaise1,
    CposRaise2,
    CposRaise3,
    CposRaise4,
    CposRaise5,
    CposRaise6,
    CposRaise7,
    TrooStnd,
    TrooStnd2,
    TrooRun1,
    TrooRun2,
    TrooRun3,
    TrooRun4,
    TrooRun5,
    TrooRun6,
    TrooRun7,
    TrooRun8,
    TrooAtk1,
    TrooAtk2,
    TrooAtk3,
    TrooPain,
    TrooPain2,
    TrooDie1,
    TrooDie2,
    TrooDie3,
    TrooDie4,
    TrooDie5,
    TrooXdie1,
    TrooXdie2,
    TrooXdie3,
    TrooXdie4,
    TrooXdie5,
    TrooXdie6,
    TrooXdie7,
    TrooXdie8,
    TrooRaise1,
    TrooRaise2,
    TrooRaise3,
    TrooRaise4,
    TrooRaise5,
    SargStnd,
    SargStnd2,
    SargRun1,
    SargRun2,
    SargRun3,
    SargRun4,
    SargRun5,
    SargRun6,
    SargRun7,
    SargRun8,
    SargAtk1,
    SargAtk2,
    SargAtk3,
    SargPain,
    SargPain2,
    SargDie1,
    SargDie2,
    SargDie3,
    SargDie4,
    SargDie5,
    SargDie6,
    SargRaise1,
    SargRaise2,
    SargRaise3,
    SargRaise4,
    SargRaise5,
    SargRaise6,
    HeadStnd,
    HeadRun1,
    HeadAtk1,
    HeadAtk2,
    HeadAtk3,
    HeadPain,
    HeadPain2,
    HeadPain3,
    HeadDie1,
    HeadDie2,
    HeadDie3,
    HeadDie4,
    HeadDie5,
    HeadDie6,
    HeadRaise1,
    HeadRaise2,
    HeadRaise3,
    HeadRaise4,
    HeadRaise5,
    HeadRaise6,
    Brball1,
    Brball2,
    Brballx1,
    Brballx2,
    Brballx3,
    BossStnd,
    BossStnd2,
    BossRun1,
    BossRun2,
    BossRun3,
    BossRun4,
    BossRun5,
    BossRun6,
    BossRun7,
    BossRun8,
    BossAtk1,
    BossAtk2,
    BossAtk3,
    BossPain,
    BossPain2,
    BossDie1,
    BossDie2,
    BossDie3,
    BossDie4,
    BossDie5,
    BossDie6,
    BossDie7,
    BossRaise1,
    BossRaise2,
    BossRaise3,
    BossRaise4,
    BossRaise5,
    BossRaise6,
    BossRaise7,
    Bos2Stnd,
    Bos2Stnd2,
    Bos2Run1,
    Bos2Run2,
    Bos2Run3,
    Bos2Run4,
    Bos2Run5,
    Bos2Run6,
    Bos2Run7,
    Bos2Run8,
    Bos2Atk1,
    Bos2Atk2,
    Bos2Atk3,
    Bos2Pain,
    Bos2Pain2,
    Bos2Die1,
    Bos2Die2,
    Bos2Die3,
    Bos2Die4,
    Bos2Die5,
    Bos2Die6,
    Bos2Die7,
    Bos2Raise1,
    Bos2Raise2,
    Bos2Raise3,
    Bos2Raise4,
    Bos2Raise5,
    Bos2Raise6,
    Bos2Raise7,
    SkullStnd,
    SkullStnd2,
    SkullRun1,
    SkullRun2,
    SkullAtk1,
    SkullAtk2,
    SkullAtk3,
    SkullAtk4,
    SkullPain,
    SkullPain2,
    SkullDie1,
    SkullDie2,
    SkullDie3,
    SkullDie4,
    SkullDie5,
    SkullDie6,
    SpidStnd,
    SpidStnd2,
    SpidRun1,
    SpidRun2,
    SpidRun3,
    SpidRun4,
    SpidRun5,
    SpidRun6,
    SpidRun7,
    SpidRun8,
    SpidRun9,
    SpidRun10,
    SpidRun11,
    SpidRun12,
    SpidAtk1,
    SpidAtk2,
    SpidAtk3,
    SpidAtk4,
    SpidPain,
    SpidPain2,
    SpidDie1,
    SpidDie2,
    SpidDie3,
    SpidDie4,
    SpidDie5,
    SpidDie6,
    SpidDie7,
    SpidDie8,
    SpidDie9,
    SpidDie10,
    SpidDie11,
    BspiStnd,
    BspiStnd2,
    BspiSight,
    BspiRun1,
    BspiRun2,
    BspiRun3,
    BspiRun4,
    BspiRun5,
    BspiRun6,
    BspiRun7,
    BspiRun8,
    BspiRun9,
    BspiRun10,
    BspiRun11,
    BspiRun12,
    BspiAtk1,
    BspiAtk2,
    BspiAtk3,
    BspiAtk4,
    BspiPain,
    BspiPain2,
    BspiDie1,
    BspiDie2,
    BspiDie3,
    BspiDie4,
    BspiDie5,
    BspiDie6,
    BspiDie7,
    BspiRaise1,
    BspiRaise2,
    BspiRaise3,
    BspiRaise4,
    BspiRaise5,
    BspiRaise6,
    BspiRaise7,
    ArachPlaz,
    ArachPlaz2,
    ArachPlex,
    ArachPlex2,
    ArachPlex3,
    ArachPlex4,
    ArachPlex5,
    CyberStnd,
    CyberStnd2,
    CyberRun1,
    CyberRun2,
    CyberRun3,
    CyberRun4,
    CyberRun5,
    CyberRun6,
    CyberRun7,
    CyberRun8,
    CyberAtk1,
    CyberAtk2,
    CyberAtk3,
    CyberAtk4,
    CyberAtk5,
    CyberAtk6,
    CyberPain,
    CyberDie1,
    CyberDie2,
    CyberDie3,
    CyberDie4,
    CyberDie5,
    CyberDie6,
    CyberDie7,
    CyberDie8,
    CyberDie9,
    CyberDie10,
    PainStnd,
    PainRun1,
    PainRun2,
    PainRun3,
    PainRun4,
    PainRun5,
    PainRun6,
    PainAtk1,
    PainAtk2,
    PainAtk3,
    PainAtk4,
    PainPain,
    PainPain2,
    PainDie1,
    PainDie2,
    PainDie3,
    PainDie4,
    PainDie5,
    PainDie6,
    PainRaise1,
    PainRaise2,
    PainRaise3,
    PainRaise4,
    PainRaise5,
    PainRaise6,
    SswvStnd,
    SswvStnd2,
    SswvRun1,
    SswvRun2,
    SswvRun3,
    SswvRun4,
    SswvRun5,
    SswvRun6,
    SswvRun7,
    SswvRun8,
    SswvAtk1,
    SswvAtk2,
    SswvAtk3,
    SswvAtk4,
    SswvAtk5,
    SswvAtk6,
    SswvPain,
    SswvPain2,
    SswvDie1,
    SswvDie2,
    SswvDie3,
    SswvDie4,
    SswvDie5,
    SswvXdie1,
    SswvXdie2,
    SswvXdie3,
    SswvXdie4,
    SswvXdie5,
    SswvXdie6,
    SswvXdie7,
    SswvXdie8,
    SswvXdie9,
    SswvRaise1,
    SswvRaise2,
    SswvRaise3,
    SswvRaise4,
    SswvRaise5,
    Keenstnd,
    Commkeen,
    Commkeen2,
    Commkeen3,
    Commkeen4,
    Commkeen5,
    Commkeen6,
    Commkeen7,
    Commkeen8,
    Commkeen9,
    Commkeen10,
    Commkeen11,
    Commkeen12,
    Keenpain,
    Keenpain2,
    Brain,
    BrainPain,
    BrainDie1,
    BrainDie2,
    BrainDie3,
    BrainDie4,
    Braineye,
    Braineyesee,
    Braineye1,
    Spawn1,
    Spawn2,
    Spawn3,
    Spawn4,
    Spawnfire1,
    Spawnfire2,
    Spawnfire3,
    Spawnfire4,
    Spawnfire5,
    Spawnfire6,
    Spawnfire7,
    Spawnfire8,
    Brainexplode1,
    Brainexplode2,
    Brainexplode3,
    Arm1,
    Arm1a,
    Arm2,
    Arm2a,
    Bar1,
    Bar2,
    Bexp,
    Bexp2,
    Bexp3,
    Bexp4,
    Bexp5,
    Bbar1,
    Bbar2,
    Bbar3,
    Bon1,
    Bon1a,
    Bon1b,
    Bon1c,
    Bon1d,
    Bon1e,
    Bon2,
    Bon2a,
    Bon2b,
    Bon2c,
    Bon2d,
    Bon2e,
    Bkey,
    Bkey2,
    Rkey,
    Rkey2,
    Ykey,
    Ykey2,
    Bskull,
    Bskull2,
    Rskull,
    Rskull2,
    Yskull,
    Yskull2,
    Stim,
    Medi,
    Soul,
    Soul2,
    Soul3,
    Soul4,
    Soul5,
    Soul6,
    Pinv,
    Pinv2,
    Pinv3,
    Pinv4,
    Pstr,
    Pins,
    Pins2,
    Pins3,
    Pins4,
    Mega,
    Mega2,
    Mega3,
    Mega4,
    Suit,
    Pmap,
    Pmap2,
    Pmap3,
    Pmap4,
    Pmap5,
    Pmap6,
    Pvis,
    Pvis2,
    Clip,
    Ammo,
    Rock,
    Brok,
    Cell,
    Celp,
    Shel,
    Sbox,
    Bpak,
    Bfug,
    Mgun,
    Csaw,
    Laun,
    Plas,
    Shot,
    Shot2,
    Colu,
    Stalag,
    Bloodytwitch,
    Bloodytwitch2,
    Bloodytwitch3,
    Bloodytwitch4,
    Deadtorso,
    Deadbottom,
    Headsonstick,
    Gibs,
    Headonastick,
    Headcandles,
    Headcandles2,
    Deadstick,
    Livestick,
    Livestick2,
    Meat2,
    Meat3,
    Meat4,
    Meat5,
    Stalagtite,
    Tallgrncol,
    Shrtgrncol,
    Tallredcol,
    Shrtredcol,
    Candlestik,
    Candelabra,
    Skullcol,
    Torchtree,
    Bigtree,
    Techpillar,
    Evileye,
    Evileye2,
    Evileye3,
    Evileye4,
    Floatskull,
    Floatskull2,
    Floatskull3,
    Heartcol,
    Heartcol2,
    Bluetorch,
    Bluetorch2,
    Bluetorch3,
    Bluetorch4,
    Greentorch,
    Greentorch2,
    Greentorch3,
    Greentorch4,
    Redtorch,
    Redtorch2,
    Redtorch3,
    Redtorch4,
    Btorchshrt,
    Btorchshrt2,
    Btorchshrt3,
    Btorchshrt4,
    Gtorchshrt,
    Gtorchshrt2,
    Gtorchshrt3,
    Gtorchshrt4,
    Rtorchshrt,
    Rtorchshrt2,
    Rtorchshrt3,
    Rtorchshrt4,
    Hangnoguts,
    Hangbnobrain,
    Hangtlookdn,
    Hangtskull,
    Hangtlookup,
    Hangtnobrain,
    Colongibs,
    Smallpool,
    Brainstem,
    Techlamp,
    Techlamp2,
    Techlamp3,
    Techlamp4,
    Tech2lamp,
    Tech2lamp2,
    Tech2lamp3,
    Tech2lamp4,
}
pub fn statenum_from_raw(v: i32) -> StateNum {
    match v {
        0 => StateNum::Null,
        1 => StateNum::Lightdone,
        2 => StateNum::Punch,
        3 => StateNum::Punchdown,
        4 => StateNum::Punchup,
        5 => StateNum::Punch1,
        6 => StateNum::Punch2,
        7 => StateNum::Punch3,
        8 => StateNum::Punch4,
        9 => StateNum::Punch5,
        10 => StateNum::Pistol,
        11 => StateNum::Pistoldown,
        12 => StateNum::Pistolup,
        13 => StateNum::Pistol1,
        14 => StateNum::Pistol2,
        15 => StateNum::Pistol3,
        16 => StateNum::Pistol4,
        17 => StateNum::Pistolflash,
        18 => StateNum::Sgun,
        19 => StateNum::Sgundown,
        20 => StateNum::Sgunup,
        21 => StateNum::Sgun1,
        22 => StateNum::Sgun2,
        23 => StateNum::Sgun3,
        24 => StateNum::Sgun4,
        25 => StateNum::Sgun5,
        26 => StateNum::Sgun6,
        27 => StateNum::Sgun7,
        28 => StateNum::Sgun8,
        29 => StateNum::Sgun9,
        30 => StateNum::Sgunflash1,
        31 => StateNum::Sgunflash2,
        32 => StateNum::Dsgun,
        33 => StateNum::Dsgundown,
        34 => StateNum::Dsgunup,
        35 => StateNum::Dsgun1,
        36 => StateNum::Dsgun2,
        37 => StateNum::Dsgun3,
        38 => StateNum::Dsgun4,
        39 => StateNum::Dsgun5,
        40 => StateNum::Dsgun6,
        41 => StateNum::Dsgun7,
        42 => StateNum::Dsgun8,
        43 => StateNum::Dsgun9,
        44 => StateNum::Dsgun10,
        45 => StateNum::Dsnr1,
        46 => StateNum::Dsnr2,
        47 => StateNum::Dsgunflash1,
        48 => StateNum::Dsgunflash2,
        49 => StateNum::Chain,
        50 => StateNum::Chaindown,
        51 => StateNum::Chainup,
        52 => StateNum::Chain1,
        53 => StateNum::Chain2,
        54 => StateNum::Chain3,
        55 => StateNum::Chainflash1,
        56 => StateNum::Chainflash2,
        57 => StateNum::Missile,
        58 => StateNum::Missiledown,
        59 => StateNum::Missileup,
        60 => StateNum::Missile1,
        61 => StateNum::Missile2,
        62 => StateNum::Missile3,
        63 => StateNum::Missileflash1,
        64 => StateNum::Missileflash2,
        65 => StateNum::Missileflash3,
        66 => StateNum::Missileflash4,
        67 => StateNum::Saw,
        68 => StateNum::Sawb,
        69 => StateNum::Sawdown,
        70 => StateNum::Sawup,
        71 => StateNum::Saw1,
        72 => StateNum::Saw2,
        73 => StateNum::Saw3,
        74 => StateNum::Plasma,
        75 => StateNum::Plasmadown,
        76 => StateNum::Plasmaup,
        77 => StateNum::Plasma1,
        78 => StateNum::Plasma2,
        79 => StateNum::Plasmaflash1,
        80 => StateNum::Plasmaflash2,
        81 => StateNum::Bfg,
        82 => StateNum::Bfgdown,
        83 => StateNum::Bfgup,
        84 => StateNum::Bfg1,
        85 => StateNum::Bfg2,
        86 => StateNum::Bfg3,
        87 => StateNum::Bfg4,
        88 => StateNum::Bfgflash1,
        89 => StateNum::Bfgflash2,
        90 => StateNum::Blood1,
        91 => StateNum::Blood2,
        92 => StateNum::Blood3,
        93 => StateNum::Puff1,
        94 => StateNum::Puff2,
        95 => StateNum::Puff3,
        96 => StateNum::Puff4,
        97 => StateNum::Tball1,
        98 => StateNum::Tball2,
        99 => StateNum::Tballx1,
        100 => StateNum::Tballx2,
        101 => StateNum::Tballx3,
        102 => StateNum::Rball1,
        103 => StateNum::Rball2,
        104 => StateNum::Rballx1,
        105 => StateNum::Rballx2,
        106 => StateNum::Rballx3,
        107 => StateNum::Plasball,
        108 => StateNum::Plasball2,
        109 => StateNum::Plasexp,
        110 => StateNum::Plasexp2,
        111 => StateNum::Plasexp3,
        112 => StateNum::Plasexp4,
        113 => StateNum::Plasexp5,
        114 => StateNum::Rocket,
        115 => StateNum::Bfgshot,
        116 => StateNum::Bfgshot2,
        117 => StateNum::Bfgland,
        118 => StateNum::Bfgland2,
        119 => StateNum::Bfgland3,
        120 => StateNum::Bfgland4,
        121 => StateNum::Bfgland5,
        122 => StateNum::Bfgland6,
        123 => StateNum::Bfgexp,
        124 => StateNum::Bfgexp2,
        125 => StateNum::Bfgexp3,
        126 => StateNum::Bfgexp4,
        127 => StateNum::Explode1,
        128 => StateNum::Explode2,
        129 => StateNum::Explode3,
        130 => StateNum::Tfog,
        131 => StateNum::Tfog01,
        132 => StateNum::Tfog02,
        133 => StateNum::Tfog2,
        134 => StateNum::Tfog3,
        135 => StateNum::Tfog4,
        136 => StateNum::Tfog5,
        137 => StateNum::Tfog6,
        138 => StateNum::Tfog7,
        139 => StateNum::Tfog8,
        140 => StateNum::Tfog9,
        141 => StateNum::Tfog10,
        142 => StateNum::Ifog,
        143 => StateNum::Ifog01,
        144 => StateNum::Ifog02,
        145 => StateNum::Ifog2,
        146 => StateNum::Ifog3,
        147 => StateNum::Ifog4,
        148 => StateNum::Ifog5,
        149 => StateNum::Play,
        150 => StateNum::PlayRun1,
        151 => StateNum::PlayRun2,
        152 => StateNum::PlayRun3,
        153 => StateNum::PlayRun4,
        154 => StateNum::PlayAtk1,
        155 => StateNum::PlayAtk2,
        156 => StateNum::PlayPain,
        157 => StateNum::PlayPain2,
        158 => StateNum::PlayDie1,
        159 => StateNum::PlayDie2,
        160 => StateNum::PlayDie3,
        161 => StateNum::PlayDie4,
        162 => StateNum::PlayDie5,
        163 => StateNum::PlayDie6,
        164 => StateNum::PlayDie7,
        165 => StateNum::PlayXdie1,
        166 => StateNum::PlayXdie2,
        167 => StateNum::PlayXdie3,
        168 => StateNum::PlayXdie4,
        169 => StateNum::PlayXdie5,
        170 => StateNum::PlayXdie6,
        171 => StateNum::PlayXdie7,
        172 => StateNum::PlayXdie8,
        173 => StateNum::PlayXdie9,
        174 => StateNum::PossStnd,
        175 => StateNum::PossStnd2,
        176 => StateNum::PossRun1,
        177 => StateNum::PossRun2,
        178 => StateNum::PossRun3,
        179 => StateNum::PossRun4,
        180 => StateNum::PossRun5,
        181 => StateNum::PossRun6,
        182 => StateNum::PossRun7,
        183 => StateNum::PossRun8,
        184 => StateNum::PossAtk1,
        185 => StateNum::PossAtk2,
        186 => StateNum::PossAtk3,
        187 => StateNum::PossPain,
        188 => StateNum::PossPain2,
        189 => StateNum::PossDie1,
        190 => StateNum::PossDie2,
        191 => StateNum::PossDie3,
        192 => StateNum::PossDie4,
        193 => StateNum::PossDie5,
        194 => StateNum::PossXdie1,
        195 => StateNum::PossXdie2,
        196 => StateNum::PossXdie3,
        197 => StateNum::PossXdie4,
        198 => StateNum::PossXdie5,
        199 => StateNum::PossXdie6,
        200 => StateNum::PossXdie7,
        201 => StateNum::PossXdie8,
        202 => StateNum::PossXdie9,
        203 => StateNum::PossRaise1,
        204 => StateNum::PossRaise2,
        205 => StateNum::PossRaise3,
        206 => StateNum::PossRaise4,
        207 => StateNum::SposStnd,
        208 => StateNum::SposStnd2,
        209 => StateNum::SposRun1,
        210 => StateNum::SposRun2,
        211 => StateNum::SposRun3,
        212 => StateNum::SposRun4,
        213 => StateNum::SposRun5,
        214 => StateNum::SposRun6,
        215 => StateNum::SposRun7,
        216 => StateNum::SposRun8,
        217 => StateNum::SposAtk1,
        218 => StateNum::SposAtk2,
        219 => StateNum::SposAtk3,
        220 => StateNum::SposPain,
        221 => StateNum::SposPain2,
        222 => StateNum::SposDie1,
        223 => StateNum::SposDie2,
        224 => StateNum::SposDie3,
        225 => StateNum::SposDie4,
        226 => StateNum::SposDie5,
        227 => StateNum::SposXdie1,
        228 => StateNum::SposXdie2,
        229 => StateNum::SposXdie3,
        230 => StateNum::SposXdie4,
        231 => StateNum::SposXdie5,
        232 => StateNum::SposXdie6,
        233 => StateNum::SposXdie7,
        234 => StateNum::SposXdie8,
        235 => StateNum::SposXdie9,
        236 => StateNum::SposRaise1,
        237 => StateNum::SposRaise2,
        238 => StateNum::SposRaise3,
        239 => StateNum::SposRaise4,
        240 => StateNum::SposRaise5,
        241 => StateNum::VileStnd,
        242 => StateNum::VileStnd2,
        243 => StateNum::VileRun1,
        244 => StateNum::VileRun2,
        245 => StateNum::VileRun3,
        246 => StateNum::VileRun4,
        247 => StateNum::VileRun5,
        248 => StateNum::VileRun6,
        249 => StateNum::VileRun7,
        250 => StateNum::VileRun8,
        251 => StateNum::VileRun9,
        252 => StateNum::VileRun10,
        253 => StateNum::VileRun11,
        254 => StateNum::VileRun12,
        255 => StateNum::VileAtk1,
        256 => StateNum::VileAtk2,
        257 => StateNum::VileAtk3,
        258 => StateNum::VileAtk4,
        259 => StateNum::VileAtk5,
        260 => StateNum::VileAtk6,
        261 => StateNum::VileAtk7,
        262 => StateNum::VileAtk8,
        263 => StateNum::VileAtk9,
        264 => StateNum::VileAtk10,
        265 => StateNum::VileAtk11,
        266 => StateNum::VileHeal1,
        267 => StateNum::VileHeal2,
        268 => StateNum::VileHeal3,
        269 => StateNum::VilePain,
        270 => StateNum::VilePain2,
        271 => StateNum::VileDie1,
        272 => StateNum::VileDie2,
        273 => StateNum::VileDie3,
        274 => StateNum::VileDie4,
        275 => StateNum::VileDie5,
        276 => StateNum::VileDie6,
        277 => StateNum::VileDie7,
        278 => StateNum::VileDie8,
        279 => StateNum::VileDie9,
        280 => StateNum::VileDie10,
        281 => StateNum::Fire1,
        282 => StateNum::Fire2,
        283 => StateNum::Fire3,
        284 => StateNum::Fire4,
        285 => StateNum::Fire5,
        286 => StateNum::Fire6,
        287 => StateNum::Fire7,
        288 => StateNum::Fire8,
        289 => StateNum::Fire9,
        290 => StateNum::Fire10,
        291 => StateNum::Fire11,
        292 => StateNum::Fire12,
        293 => StateNum::Fire13,
        294 => StateNum::Fire14,
        295 => StateNum::Fire15,
        296 => StateNum::Fire16,
        297 => StateNum::Fire17,
        298 => StateNum::Fire18,
        299 => StateNum::Fire19,
        300 => StateNum::Fire20,
        301 => StateNum::Fire21,
        302 => StateNum::Fire22,
        303 => StateNum::Fire23,
        304 => StateNum::Fire24,
        305 => StateNum::Fire25,
        306 => StateNum::Fire26,
        307 => StateNum::Fire27,
        308 => StateNum::Fire28,
        309 => StateNum::Fire29,
        310 => StateNum::Fire30,
        311 => StateNum::Smoke1,
        312 => StateNum::Smoke2,
        313 => StateNum::Smoke3,
        314 => StateNum::Smoke4,
        315 => StateNum::Smoke5,
        316 => StateNum::Tracer,
        317 => StateNum::Tracer2,
        318 => StateNum::Traceexp1,
        319 => StateNum::Traceexp2,
        320 => StateNum::Traceexp3,
        321 => StateNum::SkelStnd,
        322 => StateNum::SkelStnd2,
        323 => StateNum::SkelRun1,
        324 => StateNum::SkelRun2,
        325 => StateNum::SkelRun3,
        326 => StateNum::SkelRun4,
        327 => StateNum::SkelRun5,
        328 => StateNum::SkelRun6,
        329 => StateNum::SkelRun7,
        330 => StateNum::SkelRun8,
        331 => StateNum::SkelRun9,
        332 => StateNum::SkelRun10,
        333 => StateNum::SkelRun11,
        334 => StateNum::SkelRun12,
        335 => StateNum::SkelFist1,
        336 => StateNum::SkelFist2,
        337 => StateNum::SkelFist3,
        338 => StateNum::SkelFist4,
        339 => StateNum::SkelMiss1,
        340 => StateNum::SkelMiss2,
        341 => StateNum::SkelMiss3,
        342 => StateNum::SkelMiss4,
        343 => StateNum::SkelPain,
        344 => StateNum::SkelPain2,
        345 => StateNum::SkelDie1,
        346 => StateNum::SkelDie2,
        347 => StateNum::SkelDie3,
        348 => StateNum::SkelDie4,
        349 => StateNum::SkelDie5,
        350 => StateNum::SkelDie6,
        351 => StateNum::SkelRaise1,
        352 => StateNum::SkelRaise2,
        353 => StateNum::SkelRaise3,
        354 => StateNum::SkelRaise4,
        355 => StateNum::SkelRaise5,
        356 => StateNum::SkelRaise6,
        357 => StateNum::Fatshot1,
        358 => StateNum::Fatshot2,
        359 => StateNum::Fatshotx1,
        360 => StateNum::Fatshotx2,
        361 => StateNum::Fatshotx3,
        362 => StateNum::FattStnd,
        363 => StateNum::FattStnd2,
        364 => StateNum::FattRun1,
        365 => StateNum::FattRun2,
        366 => StateNum::FattRun3,
        367 => StateNum::FattRun4,
        368 => StateNum::FattRun5,
        369 => StateNum::FattRun6,
        370 => StateNum::FattRun7,
        371 => StateNum::FattRun8,
        372 => StateNum::FattRun9,
        373 => StateNum::FattRun10,
        374 => StateNum::FattRun11,
        375 => StateNum::FattRun12,
        376 => StateNum::FattAtk1,
        377 => StateNum::FattAtk2,
        378 => StateNum::FattAtk3,
        379 => StateNum::FattAtk4,
        380 => StateNum::FattAtk5,
        381 => StateNum::FattAtk6,
        382 => StateNum::FattAtk7,
        383 => StateNum::FattAtk8,
        384 => StateNum::FattAtk9,
        385 => StateNum::FattAtk10,
        386 => StateNum::FattPain,
        387 => StateNum::FattPain2,
        388 => StateNum::FattDie1,
        389 => StateNum::FattDie2,
        390 => StateNum::FattDie3,
        391 => StateNum::FattDie4,
        392 => StateNum::FattDie5,
        393 => StateNum::FattDie6,
        394 => StateNum::FattDie7,
        395 => StateNum::FattDie8,
        396 => StateNum::FattDie9,
        397 => StateNum::FattDie10,
        398 => StateNum::FattRaise1,
        399 => StateNum::FattRaise2,
        400 => StateNum::FattRaise3,
        401 => StateNum::FattRaise4,
        402 => StateNum::FattRaise5,
        403 => StateNum::FattRaise6,
        404 => StateNum::FattRaise7,
        405 => StateNum::FattRaise8,
        406 => StateNum::CposStnd,
        407 => StateNum::CposStnd2,
        408 => StateNum::CposRun1,
        409 => StateNum::CposRun2,
        410 => StateNum::CposRun3,
        411 => StateNum::CposRun4,
        412 => StateNum::CposRun5,
        413 => StateNum::CposRun6,
        414 => StateNum::CposRun7,
        415 => StateNum::CposRun8,
        416 => StateNum::CposAtk1,
        417 => StateNum::CposAtk2,
        418 => StateNum::CposAtk3,
        419 => StateNum::CposAtk4,
        420 => StateNum::CposPain,
        421 => StateNum::CposPain2,
        422 => StateNum::CposDie1,
        423 => StateNum::CposDie2,
        424 => StateNum::CposDie3,
        425 => StateNum::CposDie4,
        426 => StateNum::CposDie5,
        427 => StateNum::CposDie6,
        428 => StateNum::CposDie7,
        429 => StateNum::CposXdie1,
        430 => StateNum::CposXdie2,
        431 => StateNum::CposXdie3,
        432 => StateNum::CposXdie4,
        433 => StateNum::CposXdie5,
        434 => StateNum::CposXdie6,
        435 => StateNum::CposRaise1,
        436 => StateNum::CposRaise2,
        437 => StateNum::CposRaise3,
        438 => StateNum::CposRaise4,
        439 => StateNum::CposRaise5,
        440 => StateNum::CposRaise6,
        441 => StateNum::CposRaise7,
        442 => StateNum::TrooStnd,
        443 => StateNum::TrooStnd2,
        444 => StateNum::TrooRun1,
        445 => StateNum::TrooRun2,
        446 => StateNum::TrooRun3,
        447 => StateNum::TrooRun4,
        448 => StateNum::TrooRun5,
        449 => StateNum::TrooRun6,
        450 => StateNum::TrooRun7,
        451 => StateNum::TrooRun8,
        452 => StateNum::TrooAtk1,
        453 => StateNum::TrooAtk2,
        454 => StateNum::TrooAtk3,
        455 => StateNum::TrooPain,
        456 => StateNum::TrooPain2,
        457 => StateNum::TrooDie1,
        458 => StateNum::TrooDie2,
        459 => StateNum::TrooDie3,
        460 => StateNum::TrooDie4,
        461 => StateNum::TrooDie5,
        462 => StateNum::TrooXdie1,
        463 => StateNum::TrooXdie2,
        464 => StateNum::TrooXdie3,
        465 => StateNum::TrooXdie4,
        466 => StateNum::TrooXdie5,
        467 => StateNum::TrooXdie6,
        468 => StateNum::TrooXdie7,
        469 => StateNum::TrooXdie8,
        470 => StateNum::TrooRaise1,
        471 => StateNum::TrooRaise2,
        472 => StateNum::TrooRaise3,
        473 => StateNum::TrooRaise4,
        474 => StateNum::TrooRaise5,
        475 => StateNum::SargStnd,
        476 => StateNum::SargStnd2,
        477 => StateNum::SargRun1,
        478 => StateNum::SargRun2,
        479 => StateNum::SargRun3,
        480 => StateNum::SargRun4,
        481 => StateNum::SargRun5,
        482 => StateNum::SargRun6,
        483 => StateNum::SargRun7,
        484 => StateNum::SargRun8,
        485 => StateNum::SargAtk1,
        486 => StateNum::SargAtk2,
        487 => StateNum::SargAtk3,
        488 => StateNum::SargPain,
        489 => StateNum::SargPain2,
        490 => StateNum::SargDie1,
        491 => StateNum::SargDie2,
        492 => StateNum::SargDie3,
        493 => StateNum::SargDie4,
        494 => StateNum::SargDie5,
        495 => StateNum::SargDie6,
        496 => StateNum::SargRaise1,
        497 => StateNum::SargRaise2,
        498 => StateNum::SargRaise3,
        499 => StateNum::SargRaise4,
        500 => StateNum::SargRaise5,
        501 => StateNum::SargRaise6,
        502 => StateNum::HeadStnd,
        503 => StateNum::HeadRun1,
        504 => StateNum::HeadAtk1,
        505 => StateNum::HeadAtk2,
        506 => StateNum::HeadAtk3,
        507 => StateNum::HeadPain,
        508 => StateNum::HeadPain2,
        509 => StateNum::HeadPain3,
        510 => StateNum::HeadDie1,
        511 => StateNum::HeadDie2,
        512 => StateNum::HeadDie3,
        513 => StateNum::HeadDie4,
        514 => StateNum::HeadDie5,
        515 => StateNum::HeadDie6,
        516 => StateNum::HeadRaise1,
        517 => StateNum::HeadRaise2,
        518 => StateNum::HeadRaise3,
        519 => StateNum::HeadRaise4,
        520 => StateNum::HeadRaise5,
        521 => StateNum::HeadRaise6,
        522 => StateNum::Brball1,
        523 => StateNum::Brball2,
        524 => StateNum::Brballx1,
        525 => StateNum::Brballx2,
        526 => StateNum::Brballx3,
        527 => StateNum::BossStnd,
        528 => StateNum::BossStnd2,
        529 => StateNum::BossRun1,
        530 => StateNum::BossRun2,
        531 => StateNum::BossRun3,
        532 => StateNum::BossRun4,
        533 => StateNum::BossRun5,
        534 => StateNum::BossRun6,
        535 => StateNum::BossRun7,
        536 => StateNum::BossRun8,
        537 => StateNum::BossAtk1,
        538 => StateNum::BossAtk2,
        539 => StateNum::BossAtk3,
        540 => StateNum::BossPain,
        541 => StateNum::BossPain2,
        542 => StateNum::BossDie1,
        543 => StateNum::BossDie2,
        544 => StateNum::BossDie3,
        545 => StateNum::BossDie4,
        546 => StateNum::BossDie5,
        547 => StateNum::BossDie6,
        548 => StateNum::BossDie7,
        549 => StateNum::BossRaise1,
        550 => StateNum::BossRaise2,
        551 => StateNum::BossRaise3,
        552 => StateNum::BossRaise4,
        553 => StateNum::BossRaise5,
        554 => StateNum::BossRaise6,
        555 => StateNum::BossRaise7,
        556 => StateNum::Bos2Stnd,
        557 => StateNum::Bos2Stnd2,
        558 => StateNum::Bos2Run1,
        559 => StateNum::Bos2Run2,
        560 => StateNum::Bos2Run3,
        561 => StateNum::Bos2Run4,
        562 => StateNum::Bos2Run5,
        563 => StateNum::Bos2Run6,
        564 => StateNum::Bos2Run7,
        565 => StateNum::Bos2Run8,
        566 => StateNum::Bos2Atk1,
        567 => StateNum::Bos2Atk2,
        568 => StateNum::Bos2Atk3,
        569 => StateNum::Bos2Pain,
        570 => StateNum::Bos2Pain2,
        571 => StateNum::Bos2Die1,
        572 => StateNum::Bos2Die2,
        573 => StateNum::Bos2Die3,
        574 => StateNum::Bos2Die4,
        575 => StateNum::Bos2Die5,
        576 => StateNum::Bos2Die6,
        577 => StateNum::Bos2Die7,
        578 => StateNum::Bos2Raise1,
        579 => StateNum::Bos2Raise2,
        580 => StateNum::Bos2Raise3,
        581 => StateNum::Bos2Raise4,
        582 => StateNum::Bos2Raise5,
        583 => StateNum::Bos2Raise6,
        584 => StateNum::Bos2Raise7,
        585 => StateNum::SkullStnd,
        586 => StateNum::SkullStnd2,
        587 => StateNum::SkullRun1,
        588 => StateNum::SkullRun2,
        589 => StateNum::SkullAtk1,
        590 => StateNum::SkullAtk2,
        591 => StateNum::SkullAtk3,
        592 => StateNum::SkullAtk4,
        593 => StateNum::SkullPain,
        594 => StateNum::SkullPain2,
        595 => StateNum::SkullDie1,
        596 => StateNum::SkullDie2,
        597 => StateNum::SkullDie3,
        598 => StateNum::SkullDie4,
        599 => StateNum::SkullDie5,
        600 => StateNum::SkullDie6,
        601 => StateNum::SpidStnd,
        602 => StateNum::SpidStnd2,
        603 => StateNum::SpidRun1,
        604 => StateNum::SpidRun2,
        605 => StateNum::SpidRun3,
        606 => StateNum::SpidRun4,
        607 => StateNum::SpidRun5,
        608 => StateNum::SpidRun6,
        609 => StateNum::SpidRun7,
        610 => StateNum::SpidRun8,
        611 => StateNum::SpidRun9,
        612 => StateNum::SpidRun10,
        613 => StateNum::SpidRun11,
        614 => StateNum::SpidRun12,
        615 => StateNum::SpidAtk1,
        616 => StateNum::SpidAtk2,
        617 => StateNum::SpidAtk3,
        618 => StateNum::SpidAtk4,
        619 => StateNum::SpidPain,
        620 => StateNum::SpidPain2,
        621 => StateNum::SpidDie1,
        622 => StateNum::SpidDie2,
        623 => StateNum::SpidDie3,
        624 => StateNum::SpidDie4,
        625 => StateNum::SpidDie5,
        626 => StateNum::SpidDie6,
        627 => StateNum::SpidDie7,
        628 => StateNum::SpidDie8,
        629 => StateNum::SpidDie9,
        630 => StateNum::SpidDie10,
        631 => StateNum::SpidDie11,
        632 => StateNum::BspiStnd,
        633 => StateNum::BspiStnd2,
        634 => StateNum::BspiSight,
        635 => StateNum::BspiRun1,
        636 => StateNum::BspiRun2,
        637 => StateNum::BspiRun3,
        638 => StateNum::BspiRun4,
        639 => StateNum::BspiRun5,
        640 => StateNum::BspiRun6,
        641 => StateNum::BspiRun7,
        642 => StateNum::BspiRun8,
        643 => StateNum::BspiRun9,
        644 => StateNum::BspiRun10,
        645 => StateNum::BspiRun11,
        646 => StateNum::BspiRun12,
        647 => StateNum::BspiAtk1,
        648 => StateNum::BspiAtk2,
        649 => StateNum::BspiAtk3,
        650 => StateNum::BspiAtk4,
        651 => StateNum::BspiPain,
        652 => StateNum::BspiPain2,
        653 => StateNum::BspiDie1,
        654 => StateNum::BspiDie2,
        655 => StateNum::BspiDie3,
        656 => StateNum::BspiDie4,
        657 => StateNum::BspiDie5,
        658 => StateNum::BspiDie6,
        659 => StateNum::BspiDie7,
        660 => StateNum::BspiRaise1,
        661 => StateNum::BspiRaise2,
        662 => StateNum::BspiRaise3,
        663 => StateNum::BspiRaise4,
        664 => StateNum::BspiRaise5,
        665 => StateNum::BspiRaise6,
        666 => StateNum::BspiRaise7,
        667 => StateNum::ArachPlaz,
        668 => StateNum::ArachPlaz2,
        669 => StateNum::ArachPlex,
        670 => StateNum::ArachPlex2,
        671 => StateNum::ArachPlex3,
        672 => StateNum::ArachPlex4,
        673 => StateNum::ArachPlex5,
        674 => StateNum::CyberStnd,
        675 => StateNum::CyberStnd2,
        676 => StateNum::CyberRun1,
        677 => StateNum::CyberRun2,
        678 => StateNum::CyberRun3,
        679 => StateNum::CyberRun4,
        680 => StateNum::CyberRun5,
        681 => StateNum::CyberRun6,
        682 => StateNum::CyberRun7,
        683 => StateNum::CyberRun8,
        684 => StateNum::CyberAtk1,
        685 => StateNum::CyberAtk2,
        686 => StateNum::CyberAtk3,
        687 => StateNum::CyberAtk4,
        688 => StateNum::CyberAtk5,
        689 => StateNum::CyberAtk6,
        690 => StateNum::CyberPain,
        691 => StateNum::CyberDie1,
        692 => StateNum::CyberDie2,
        693 => StateNum::CyberDie3,
        694 => StateNum::CyberDie4,
        695 => StateNum::CyberDie5,
        696 => StateNum::CyberDie6,
        697 => StateNum::CyberDie7,
        698 => StateNum::CyberDie8,
        699 => StateNum::CyberDie9,
        700 => StateNum::CyberDie10,
        701 => StateNum::PainStnd,
        702 => StateNum::PainRun1,
        703 => StateNum::PainRun2,
        704 => StateNum::PainRun3,
        705 => StateNum::PainRun4,
        706 => StateNum::PainRun5,
        707 => StateNum::PainRun6,
        708 => StateNum::PainAtk1,
        709 => StateNum::PainAtk2,
        710 => StateNum::PainAtk3,
        711 => StateNum::PainAtk4,
        712 => StateNum::PainPain,
        713 => StateNum::PainPain2,
        714 => StateNum::PainDie1,
        715 => StateNum::PainDie2,
        716 => StateNum::PainDie3,
        717 => StateNum::PainDie4,
        718 => StateNum::PainDie5,
        719 => StateNum::PainDie6,
        720 => StateNum::PainRaise1,
        721 => StateNum::PainRaise2,
        722 => StateNum::PainRaise3,
        723 => StateNum::PainRaise4,
        724 => StateNum::PainRaise5,
        725 => StateNum::PainRaise6,
        726 => StateNum::SswvStnd,
        727 => StateNum::SswvStnd2,
        728 => StateNum::SswvRun1,
        729 => StateNum::SswvRun2,
        730 => StateNum::SswvRun3,
        731 => StateNum::SswvRun4,
        732 => StateNum::SswvRun5,
        733 => StateNum::SswvRun6,
        734 => StateNum::SswvRun7,
        735 => StateNum::SswvRun8,
        736 => StateNum::SswvAtk1,
        737 => StateNum::SswvAtk2,
        738 => StateNum::SswvAtk3,
        739 => StateNum::SswvAtk4,
        740 => StateNum::SswvAtk5,
        741 => StateNum::SswvAtk6,
        742 => StateNum::SswvPain,
        743 => StateNum::SswvPain2,
        744 => StateNum::SswvDie1,
        745 => StateNum::SswvDie2,
        746 => StateNum::SswvDie3,
        747 => StateNum::SswvDie4,
        748 => StateNum::SswvDie5,
        749 => StateNum::SswvXdie1,
        750 => StateNum::SswvXdie2,
        751 => StateNum::SswvXdie3,
        752 => StateNum::SswvXdie4,
        753 => StateNum::SswvXdie5,
        754 => StateNum::SswvXdie6,
        755 => StateNum::SswvXdie7,
        756 => StateNum::SswvXdie8,
        757 => StateNum::SswvXdie9,
        758 => StateNum::SswvRaise1,
        759 => StateNum::SswvRaise2,
        760 => StateNum::SswvRaise3,
        761 => StateNum::SswvRaise4,
        762 => StateNum::SswvRaise5,
        763 => StateNum::Keenstnd,
        764 => StateNum::Commkeen,
        765 => StateNum::Commkeen2,
        766 => StateNum::Commkeen3,
        767 => StateNum::Commkeen4,
        768 => StateNum::Commkeen5,
        769 => StateNum::Commkeen6,
        770 => StateNum::Commkeen7,
        771 => StateNum::Commkeen8,
        772 => StateNum::Commkeen9,
        773 => StateNum::Commkeen10,
        774 => StateNum::Commkeen11,
        775 => StateNum::Commkeen12,
        776 => StateNum::Keenpain,
        777 => StateNum::Keenpain2,
        778 => StateNum::Brain,
        779 => StateNum::BrainPain,
        780 => StateNum::BrainDie1,
        781 => StateNum::BrainDie2,
        782 => StateNum::BrainDie3,
        783 => StateNum::BrainDie4,
        784 => StateNum::Braineye,
        785 => StateNum::Braineyesee,
        786 => StateNum::Braineye1,
        787 => StateNum::Spawn1,
        788 => StateNum::Spawn2,
        789 => StateNum::Spawn3,
        790 => StateNum::Spawn4,
        791 => StateNum::Spawnfire1,
        792 => StateNum::Spawnfire2,
        793 => StateNum::Spawnfire3,
        794 => StateNum::Spawnfire4,
        795 => StateNum::Spawnfire5,
        796 => StateNum::Spawnfire6,
        797 => StateNum::Spawnfire7,
        798 => StateNum::Spawnfire8,
        799 => StateNum::Brainexplode1,
        800 => StateNum::Brainexplode2,
        801 => StateNum::Brainexplode3,
        802 => StateNum::Arm1,
        803 => StateNum::Arm1a,
        804 => StateNum::Arm2,
        805 => StateNum::Arm2a,
        806 => StateNum::Bar1,
        807 => StateNum::Bar2,
        808 => StateNum::Bexp,
        809 => StateNum::Bexp2,
        810 => StateNum::Bexp3,
        811 => StateNum::Bexp4,
        812 => StateNum::Bexp5,
        813 => StateNum::Bbar1,
        814 => StateNum::Bbar2,
        815 => StateNum::Bbar3,
        816 => StateNum::Bon1,
        817 => StateNum::Bon1a,
        818 => StateNum::Bon1b,
        819 => StateNum::Bon1c,
        820 => StateNum::Bon1d,
        821 => StateNum::Bon1e,
        822 => StateNum::Bon2,
        823 => StateNum::Bon2a,
        824 => StateNum::Bon2b,
        825 => StateNum::Bon2c,
        826 => StateNum::Bon2d,
        827 => StateNum::Bon2e,
        828 => StateNum::Bkey,
        829 => StateNum::Bkey2,
        830 => StateNum::Rkey,
        831 => StateNum::Rkey2,
        832 => StateNum::Ykey,
        833 => StateNum::Ykey2,
        834 => StateNum::Bskull,
        835 => StateNum::Bskull2,
        836 => StateNum::Rskull,
        837 => StateNum::Rskull2,
        838 => StateNum::Yskull,
        839 => StateNum::Yskull2,
        840 => StateNum::Stim,
        841 => StateNum::Medi,
        842 => StateNum::Soul,
        843 => StateNum::Soul2,
        844 => StateNum::Soul3,
        845 => StateNum::Soul4,
        846 => StateNum::Soul5,
        847 => StateNum::Soul6,
        848 => StateNum::Pinv,
        849 => StateNum::Pinv2,
        850 => StateNum::Pinv3,
        851 => StateNum::Pinv4,
        852 => StateNum::Pstr,
        853 => StateNum::Pins,
        854 => StateNum::Pins2,
        855 => StateNum::Pins3,
        856 => StateNum::Pins4,
        857 => StateNum::Mega,
        858 => StateNum::Mega2,
        859 => StateNum::Mega3,
        860 => StateNum::Mega4,
        861 => StateNum::Suit,
        862 => StateNum::Pmap,
        863 => StateNum::Pmap2,
        864 => StateNum::Pmap3,
        865 => StateNum::Pmap4,
        866 => StateNum::Pmap5,
        867 => StateNum::Pmap6,
        868 => StateNum::Pvis,
        869 => StateNum::Pvis2,
        870 => StateNum::Clip,
        871 => StateNum::Ammo,
        872 => StateNum::Rock,
        873 => StateNum::Brok,
        874 => StateNum::Cell,
        875 => StateNum::Celp,
        876 => StateNum::Shel,
        877 => StateNum::Sbox,
        878 => StateNum::Bpak,
        879 => StateNum::Bfug,
        880 => StateNum::Mgun,
        881 => StateNum::Csaw,
        882 => StateNum::Laun,
        883 => StateNum::Plas,
        884 => StateNum::Shot,
        885 => StateNum::Shot2,
        886 => StateNum::Colu,
        887 => StateNum::Stalag,
        888 => StateNum::Bloodytwitch,
        889 => StateNum::Bloodytwitch2,
        890 => StateNum::Bloodytwitch3,
        891 => StateNum::Bloodytwitch4,
        892 => StateNum::Deadtorso,
        893 => StateNum::Deadbottom,
        894 => StateNum::Headsonstick,
        895 => StateNum::Gibs,
        896 => StateNum::Headonastick,
        897 => StateNum::Headcandles,
        898 => StateNum::Headcandles2,
        899 => StateNum::Deadstick,
        900 => StateNum::Livestick,
        901 => StateNum::Livestick2,
        902 => StateNum::Meat2,
        903 => StateNum::Meat3,
        904 => StateNum::Meat4,
        905 => StateNum::Meat5,
        906 => StateNum::Stalagtite,
        907 => StateNum::Tallgrncol,
        908 => StateNum::Shrtgrncol,
        909 => StateNum::Tallredcol,
        910 => StateNum::Shrtredcol,
        911 => StateNum::Candlestik,
        912 => StateNum::Candelabra,
        913 => StateNum::Skullcol,
        914 => StateNum::Torchtree,
        915 => StateNum::Bigtree,
        916 => StateNum::Techpillar,
        917 => StateNum::Evileye,
        918 => StateNum::Evileye2,
        919 => StateNum::Evileye3,
        920 => StateNum::Evileye4,
        921 => StateNum::Floatskull,
        922 => StateNum::Floatskull2,
        923 => StateNum::Floatskull3,
        924 => StateNum::Heartcol,
        925 => StateNum::Heartcol2,
        926 => StateNum::Bluetorch,
        927 => StateNum::Bluetorch2,
        928 => StateNum::Bluetorch3,
        929 => StateNum::Bluetorch4,
        930 => StateNum::Greentorch,
        931 => StateNum::Greentorch2,
        932 => StateNum::Greentorch3,
        933 => StateNum::Greentorch4,
        934 => StateNum::Redtorch,
        935 => StateNum::Redtorch2,
        936 => StateNum::Redtorch3,
        937 => StateNum::Redtorch4,
        938 => StateNum::Btorchshrt,
        939 => StateNum::Btorchshrt2,
        940 => StateNum::Btorchshrt3,
        941 => StateNum::Btorchshrt4,
        942 => StateNum::Gtorchshrt,
        943 => StateNum::Gtorchshrt2,
        944 => StateNum::Gtorchshrt3,
        945 => StateNum::Gtorchshrt4,
        946 => StateNum::Rtorchshrt,
        947 => StateNum::Rtorchshrt2,
        948 => StateNum::Rtorchshrt3,
        949 => StateNum::Rtorchshrt4,
        950 => StateNum::Hangnoguts,
        951 => StateNum::Hangbnobrain,
        952 => StateNum::Hangtlookdn,
        953 => StateNum::Hangtskull,
        954 => StateNum::Hangtlookup,
        955 => StateNum::Hangtnobrain,
        956 => StateNum::Colongibs,
        957 => StateNum::Smallpool,
        958 => StateNum::Brainstem,
        959 => StateNum::Techlamp,
        960 => StateNum::Techlamp2,
        961 => StateNum::Techlamp3,
        962 => StateNum::Techlamp4,
        963 => StateNum::Tech2lamp,
        964 => StateNum::Tech2lamp2,
        965 => StateNum::Tech2lamp3,
        966 => StateNum::Tech2lamp4,
        n => panic!("invalid statenum {n}"),
    }
}
#[derive(Copy, Clone)]
pub struct State {
    pub sprite: SpriteNum,
    pub frame: i32,
    pub tics: i32,
    pub action: StateAction,
    pub nextstate: StateNum,
    pub misc1: i32,
    pub misc2: i32,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MobjType {
    Player,
    Possessed,
    Shotguy,
    Vile,
    Fire,
    Undead,
    Tracer,
    Smoke,
    Fatso,
    Fatshot,
    Chainguy,
    Troop,
    Sergeant,
    Shadows,
    Head,
    Bruiser,
    Bruisershot,
    Knight,
    Skull,
    Spider,
    Baby,
    Cyborg,
    Pain,
    Wolfss,
    Keen,
    Bossbrain,
    Bossspit,
    Bosstarget,
    Spawnshot,
    Spawnfire,
    Barrel,
    Troopshot,
    Headshot,
    Rocket,
    Plasma,
    Bfg,
    Arachplaz,
    Puff,
    Blood,
    Tfog,
    Ifog,
    Teleportman,
    Extrabfg,
    Misc0,
    Misc1,
    Misc2,
    Misc3,
    Misc4,
    Misc5,
    Misc6,
    Misc7,
    Misc8,
    Misc9,
    Misc10,
    Misc11,
    Misc12,
    Inv,
    Misc13,
    Ins,
    Misc14,
    Misc15,
    Misc16,
    Mega,
    Clip,
    Misc17,
    Misc18,
    Misc19,
    Misc20,
    Misc21,
    Misc22,
    Misc23,
    Misc24,
    Misc25,
    Chaingun,
    Misc26,
    Misc27,
    Misc28,
    Shotgun,
    Supershotgun,
    Misc29,
    Misc30,
    Misc31,
    Misc32,
    Misc33,
    Misc34,
    Misc35,
    Misc36,
    Misc37,
    Misc38,
    Misc39,
    Misc40,
    Misc41,
    Misc42,
    Misc43,
    Misc44,
    Misc45,
    Misc46,
    Misc47,
    Misc48,
    Misc49,
    Misc50,
    Misc51,
    Misc52,
    Misc53,
    Misc54,
    Misc55,
    Misc56,
    Misc57,
    Misc58,
    Misc59,
    Misc60,
    Misc61,
    Misc62,
    Misc63,
    Misc64,
    Misc65,
    Misc66,
    Misc67,
    Misc68,
    Misc69,
    Misc70,
    Misc71,
    Misc72,
    Misc73,
    Misc74,
    Misc75,
    Misc76,
    Misc77,
    Misc78,
    Misc79,
    Misc80,
    Misc81,
    Misc82,
    Misc83,
    Misc84,
    Misc85,
    Misc86,
}
pub fn mobjtype_from_raw(v: i32) -> MobjType {
    match v {
        0 => MobjType::Player,
        1 => MobjType::Possessed,
        2 => MobjType::Shotguy,
        3 => MobjType::Vile,
        4 => MobjType::Fire,
        5 => MobjType::Undead,
        6 => MobjType::Tracer,
        7 => MobjType::Smoke,
        8 => MobjType::Fatso,
        9 => MobjType::Fatshot,
        10 => MobjType::Chainguy,
        11 => MobjType::Troop,
        12 => MobjType::Sergeant,
        13 => MobjType::Shadows,
        14 => MobjType::Head,
        15 => MobjType::Bruiser,
        16 => MobjType::Bruisershot,
        17 => MobjType::Knight,
        18 => MobjType::Skull,
        19 => MobjType::Spider,
        20 => MobjType::Baby,
        21 => MobjType::Cyborg,
        22 => MobjType::Pain,
        23 => MobjType::Wolfss,
        24 => MobjType::Keen,
        25 => MobjType::Bossbrain,
        26 => MobjType::Bossspit,
        27 => MobjType::Bosstarget,
        28 => MobjType::Spawnshot,
        29 => MobjType::Spawnfire,
        30 => MobjType::Barrel,
        31 => MobjType::Troopshot,
        32 => MobjType::Headshot,
        33 => MobjType::Rocket,
        34 => MobjType::Plasma,
        35 => MobjType::Bfg,
        36 => MobjType::Arachplaz,
        37 => MobjType::Puff,
        38 => MobjType::Blood,
        39 => MobjType::Tfog,
        40 => MobjType::Ifog,
        41 => MobjType::Teleportman,
        42 => MobjType::Extrabfg,
        43 => MobjType::Misc0,
        44 => MobjType::Misc1,
        45 => MobjType::Misc2,
        46 => MobjType::Misc3,
        47 => MobjType::Misc4,
        48 => MobjType::Misc5,
        49 => MobjType::Misc6,
        50 => MobjType::Misc7,
        51 => MobjType::Misc8,
        52 => MobjType::Misc9,
        53 => MobjType::Misc10,
        54 => MobjType::Misc11,
        55 => MobjType::Misc12,
        56 => MobjType::Inv,
        57 => MobjType::Misc13,
        58 => MobjType::Ins,
        59 => MobjType::Misc14,
        60 => MobjType::Misc15,
        61 => MobjType::Misc16,
        62 => MobjType::Mega,
        63 => MobjType::Clip,
        64 => MobjType::Misc17,
        65 => MobjType::Misc18,
        66 => MobjType::Misc19,
        67 => MobjType::Misc20,
        68 => MobjType::Misc21,
        69 => MobjType::Misc22,
        70 => MobjType::Misc23,
        71 => MobjType::Misc24,
        72 => MobjType::Misc25,
        73 => MobjType::Chaingun,
        74 => MobjType::Misc26,
        75 => MobjType::Misc27,
        76 => MobjType::Misc28,
        77 => MobjType::Shotgun,
        78 => MobjType::Supershotgun,
        79 => MobjType::Misc29,
        80 => MobjType::Misc30,
        81 => MobjType::Misc31,
        82 => MobjType::Misc32,
        83 => MobjType::Misc33,
        84 => MobjType::Misc34,
        85 => MobjType::Misc35,
        86 => MobjType::Misc36,
        87 => MobjType::Misc37,
        88 => MobjType::Misc38,
        89 => MobjType::Misc39,
        90 => MobjType::Misc40,
        91 => MobjType::Misc41,
        92 => MobjType::Misc42,
        93 => MobjType::Misc43,
        94 => MobjType::Misc44,
        95 => MobjType::Misc45,
        96 => MobjType::Misc46,
        97 => MobjType::Misc47,
        98 => MobjType::Misc48,
        99 => MobjType::Misc49,
        100 => MobjType::Misc50,
        101 => MobjType::Misc51,
        102 => MobjType::Misc52,
        103 => MobjType::Misc53,
        104 => MobjType::Misc54,
        105 => MobjType::Misc55,
        106 => MobjType::Misc56,
        107 => MobjType::Misc57,
        108 => MobjType::Misc58,
        109 => MobjType::Misc59,
        110 => MobjType::Misc60,
        111 => MobjType::Misc61,
        112 => MobjType::Misc62,
        113 => MobjType::Misc63,
        114 => MobjType::Misc64,
        115 => MobjType::Misc65,
        116 => MobjType::Misc66,
        117 => MobjType::Misc67,
        118 => MobjType::Misc68,
        119 => MobjType::Misc69,
        120 => MobjType::Misc70,
        121 => MobjType::Misc71,
        122 => MobjType::Misc72,
        123 => MobjType::Misc73,
        124 => MobjType::Misc74,
        125 => MobjType::Misc75,
        126 => MobjType::Misc76,
        127 => MobjType::Misc77,
        128 => MobjType::Misc78,
        129 => MobjType::Misc79,
        130 => MobjType::Misc80,
        131 => MobjType::Misc81,
        132 => MobjType::Misc82,
        133 => MobjType::Misc83,
        134 => MobjType::Misc84,
        135 => MobjType::Misc85,
        136 => MobjType::Misc86,
        n => panic!("invalid mobjtype {n}"),
    }
}
bitflags::bitflags! {
    /// Behaviour flags of a mobj (`MF_*` in the C source).
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
    pub struct MobjFlags: i32 {
        const SPECIAL = 1;
        const SOLID = 2;
        const SHOOTABLE = 4;
        const NOSECTOR = 8;
        const NOBLOCKMAP = 16;
        const AMBUSH = 32;
        const JUSTHIT = 64;
        const JUSTATTACKED = 128;
        const SPAWNCEILING = 256;
        const NOGRAVITY = 512;
        const DROPOFF = 1024;
        const PICKUP = 2048;
        const NOCLIP = 0x1000;
        const FLOAT = 0x4000;
        const TELEPORT = 0x8000;
        const MISSILE = 0x10000;
        const DROPPED = 0x20000;
        const SHADOW = 0x40000;
        const NOBLOOD = 0x80000;
        const CORPSE = 0x100000;
        const INFLOAT = 0x200000;
        const COUNTKILL = 0x400000;
        const COUNTITEM = 0x800000;
        const SKULLFLY = 0x1000000;
        const NOTDMATCH = 0x2000000;
        const TRANSLATION = 0xc000000;
    }
}
impl MobjFlags {
    /// Bit position of the two-bit player colour-translation field, which
    /// `TRANSLATION` masks.
    pub const TRANSLATION_SHIFT: u32 = 26;
}
bitflags::bitflags! {
    /// Flags of a map line (`ML_*` in the C source).
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
    pub struct LineFlags: i16 {
        const BLOCKING = 1;
        const BLOCKMONSTERS = 2;
        const TWOSIDED = 4;
        const DONTPEGTOP = 8;
        const DONTPEGBOTTOM = 16;
        const SECRET = 32;
        const SOUNDBLOCK = 64;
        const DONTDRAW = 128;
        const MAPPED = 256;
    }
}

#[derive(Copy, Clone)]
pub struct MobjInfo {
    pub doomednum: i32,
    pub spawnstate: StateNum,
    pub spawnhealth: i32,
    pub seestate: StateNum,
    pub seesound: SfxName,
    pub reactiontime: i32,
    pub attacksound: SfxName,
    pub painstate: StateNum,
    pub painchance: i32,
    pub painsound: SfxName,
    pub meleestate: StateNum,
    pub missilestate: StateNum,
    pub deathstate: StateNum,
    pub xdeathstate: StateNum,
    pub deathsound: SfxName,
    pub speed: i32,
    pub radius: i32,
    pub height: i32,
    pub mass: i32,
    pub damage: i32,
    pub activesound: SfxName,
    pub flags: MobjFlags,
    pub raisestate: StateNum,
}
#[derive(Copy, Clone)]
pub struct Mobj {
    pub thinker: Thinker,
    pub x: Fixed,
    pub y: Fixed,
    pub z: Fixed,
    pub snext: Option<MobjId>,
    pub sprev: Option<MobjId>,
    pub angle: Angle,
    pub sprite: SpriteNum,
    pub frame: i32,
    pub bnext: Option<MobjId>,
    pub bprev: Option<MobjId>,
    pub subsector: SubsectorId,
    pub floorz: Fixed,
    pub ceilingz: Fixed,
    pub radius: Fixed,
    pub height: Fixed,
    pub momx: Fixed,
    pub momy: Fixed,
    pub momz: Fixed,
    pub validcount: i32,
    pub kind: MobjType,
    pub tics: i32,
    pub state: Option<StateId>,
    pub flags: MobjFlags,
    pub health: i32,
    pub movedir: i32,
    pub movecount: i32,
    pub target: Option<MobjId>,
    pub reactiontime: i32,
    pub threshold: i32,
    pub player: Option<PlayerId>,
    pub lastlook: PlayerId,
    pub spawnpoint: MapThing,
    pub tracer: Option<MobjId>,
    pub id: MobjId,
}
#[derive(Copy, Clone)]
pub struct PspDef {
    pub state: Option<StateId>,
    pub tics: i32,
    pub sx: Fixed,
    pub sy: Fixed,
}
pub use crate::d_player::{PlayerId, PlayerState};
#[derive(Copy, Clone)]
pub struct Subsector {
    pub sector: SectorId,
    pub numlines: i16,
    pub firstline: i16,
}
#[derive(Clone)]
pub struct Sector {
    pub floorheight: Fixed,
    pub ceilingheight: Fixed,
    pub floorpic: i16,
    pub ceilingpic: i16,
    pub lightlevel: i16,
    pub special: i16,
    pub tag: i16,
    pub soundtraversed: i32,
    pub soundtarget: Option<MobjId>,
    pub blockbox: EnumArray<BoxIndex, i32, 4>,
    pub soundorg: DegenMobj,
    pub validcount: i32,
    pub thinglist: Option<MobjId>,
    pub specialdata: Option<SectorSpecial>,
    pub linecount: i32,
    pub lines: Vec<LineId>,
}
#[derive(Copy, Clone)]
pub struct Line {
    pub v1: VertexId,
    pub v2: VertexId,
    pub dx: Fixed,
    pub dy: Fixed,
    pub flags: LineFlags,
    pub special: i16,
    pub tag: i16,
    pub sidenum: [i16; 2],
    pub bbox: BBox,
    pub slopetype: SlopeType,
    pub frontsector: Option<SectorId>,
    pub backsector: Option<SectorId>,
    pub validcount: i32,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SlopeType {
    Horizontal,
    Vertical,
    Positive,
    Negative,
}
#[derive(Copy, Clone)]
pub struct Vertex {
    pub x: Fixed,
    pub y: Fixed,
}
#[derive(Copy, Clone)]
pub struct DegenMobj {
    pub thinker: Thinker,
    pub x: Fixed,
    pub y: Fixed,
    pub z: Fixed,
}
pub const MTF_AMBUSH: i32 = 8;
pub const FLOATSPEED: i32 = FRACUNIT * 4;
pub const GRAVITY: i32 = FRACUNIT;
pub const MAXMOVE: i32 = 30 * FRACUNIT;
pub const ONFLOORZ: i32 = INT_MIN;
pub const ONCEILINGZ: i32 = INT_MAX;
pub const ITEMQUESIZE: i32 = 128;
pub fn set_mobj_state(state: &mut GameState, mobj: MobjId, mut statenum: StateNum) -> bool {
    loop {
        if statenum == StateNum::Null {
            state.world.p_mobj.mo_mut(mobj).state = None;
            remove_mobj(state, mobj);
            return false;
        }
        let state_id = StateId(statenum as u32);
        let (tics, sprite, frame, action, nextstate) = {
            let st = state.assets.info.state_mut(state_id);
            (st.tics, st.sprite, st.frame, st.action, st.nextstate)
        };
        {
            let m = state.world.p_mobj.mo_mut(mobj);
            m.state = Some(state_id);
            m.tics = tics;
            m.sprite = sprite;
            m.frame = frame;
        }
        if let StateAction::Mobj(f) = action {
            f(state, mobj);
        }
        statenum = nextstate;
        if state.world.p_mobj.mo(mobj).tics != 0 {
            break;
        }
    }
    true
}
pub fn explode_missile(state: &mut GameState, mo: MobjId) {
    state.world.p_mobj.mo_mut(mo).momz = 0;
    state.world.p_mobj.mo_mut(mo).momy = state.world.p_mobj.mo(mo).momz;
    state.world.p_mobj.mo_mut(mo).momx = state.world.p_mobj.mo(mo).momy;
    set_mobj_state(
        state,
        mo,
        state.assets.info.mobjinfo[state.world.p_mobj.mo(mo).kind as usize].deathstate,
    );
    state.world.p_mobj.mo_mut(mo).tics -= p_random(&mut state.world.m_random) & 3;
    if state.world.p_mobj.mo(mo).tics < 1 {
        state.world.p_mobj.mo_mut(mo).tics = 1;
    }
    state.world.p_mobj.mo_mut(mo).flags &= !MobjFlags::MISSILE;
    let deathsound = state
        .assets
        .info
        .mobjinfo_mut(state.world.p_mobj.mo(mo).kind)
        .deathsound;
    if deathsound != SfxName::SfxNone {
        s_start_sound(state, SoundOrigin::Mobj(mo), deathsound);
    }
}
pub const STOPSPEED: i32 = 0x1000;
pub const FRICTION: i32 = 0xe800;
pub fn xymovement(state: &mut GameState, mo: MobjId) {
    let (momx, momy, flags) = {
        let m = state.world.p_mobj.mo(mo);
        (m.momx, m.momy, m.flags)
    };
    if momx == 0 && momy == 0 {
        if flags.contains(MobjFlags::SKULLFLY) {
            let mo_type = {
                let m = state.world.p_mobj.mo_mut(mo);
                m.flags &= !MobjFlags::SKULLFLY;
                m.momz = 0;
                m.momy = m.momz;
                m.momx = m.momy;
                m.kind
            };
            let spawnstate = state.assets.info.mobjinfo_mut(mo_type).spawnstate;
            set_mobj_state(state, mo, spawnstate);
        }
        return;
    }
    let player = state.world.p_mobj.mo(mo).player;
    let (mut xmove, mut ymove) = {
        let m = state.world.p_mobj.mo_mut(mo);
        if m.momx > MAXMOVE {
            m.momx = MAXMOVE as Fixed;
        } else if m.momx < -MAXMOVE {
            m.momx = -MAXMOVE as Fixed;
        }
        if m.momy > MAXMOVE {
            m.momy = MAXMOVE as Fixed;
        } else if m.momy < -MAXMOVE {
            m.momy = -MAXMOVE as Fixed;
        }
        (m.momx, m.momy)
    };
    loop {
        let (mx, my) = {
            let m = state.world.p_mobj.mo(mo);
            (m.x, m.y)
        };
        let (ptryx, ptryy): (Fixed, Fixed) = if xmove > MAXMOVE / 2 || ymove > MAXMOVE / 2 {
            let ptry = (mx + xmove / 2, my + ymove / 2);
            xmove >>= 1;
            ymove >>= 1;
            ptry
        } else {
            let ptry = (mx + xmove, my + ymove);
            ymove = 0;
            xmove = ymove;
            ptry
        };
        if !try_move(state, mo, ptryx, ptryy) {
            if player.is_some() {
                slide_move(state, mo);
            } else if state.world.p_mobj.mo(mo).flags.contains(MobjFlags::MISSILE) {
                if state.world.p_map.ceilingline.is_some_and(|ceilingline| {
                    state
                        .world
                        .p_setup
                        .line(ceilingline)
                        .backsector
                        .is_some_and(|backsector| {
                            i32::from(state.world.p_setup.sector_mut(backsector).ceilingpic)
                                == state.render.r_sky.skyflatnum
                        })
                }) {
                    remove_mobj(state, mo);
                    return;
                }
                explode_missile(state, mo);
            } else {
                let m = state.world.p_mobj.mo_mut(mo);
                m.momy = 0;
                m.momx = m.momy;
            }
        }
        if !(xmove != 0 || ymove != 0) {
            break;
        }
    }
    if let Some(player_id) = player {
        if state.game.g_game.players[player_id]
            .cheats
            .contains(CheatFlags::NOMOMENTUM)
        {
            let m = state.world.p_mobj.mo_mut(mo);
            m.momy = 0;
            m.momx = m.momy;
            return;
        }
    }
    let (flags, z, floorz, momx, momy, subsector) = {
        let m = state.world.p_mobj.mo(mo);
        (m.flags, m.z, m.floorz, m.momx, m.momy, m.subsector)
    };
    if flags.intersects(MobjFlags::MISSILE | MobjFlags::SKULLFLY) {
        return;
    }
    if z > floorz {
        return;
    }
    if flags.contains(MobjFlags::CORPSE)
        && (!(-FRACUNIT / 4..=FRACUNIT / 4).contains(&momx)
            || !(-FRACUNIT / 4..=FRACUNIT / 4).contains(&momy))
        && floorz
            != state
                .world
                .p_setup
                .sector_mut(state.world.p_setup.subsectors[subsector.0 as usize].sector)
                .floorheight
    {
        return;
    }
    let player_idle = match player {
        None => true,
        Some(player_id) => {
            let cmd = &state.game.g_game.players[player_id].cmd;
            i32::from(cmd.forwardmove) == 0 && i32::from(cmd.sidemove) == 0
        }
    };
    if momx > -STOPSPEED && momx < STOPSPEED && momy > -STOPSPEED && momy < STOPSPEED && player_idle
    {
        if player.is_some()
            && (state
                .world
                .p_mobj
                .mo(mo)
                .state
                .unwrap()
                .0
                .wrapping_sub(StateNum::PlayRun1 as u32))
                < 4
        {
            set_mobj_state(state, mo, StateNum::Play);
        }
        let m = state.world.p_mobj.mo_mut(mo);
        m.momx = 0;
        m.momy = 0;
    } else {
        let m = state.world.p_mobj.mo_mut(mo);
        m.momx = fixed_mul(m.momx, FRICTION);
        m.momy = fixed_mul(m.momy, FRICTION);
    }
}
pub fn zmovement(state: &mut GameState, mo: MobjId) {
    if state.world.p_mobj.mo(mo).player.is_some()
        && state.world.p_mobj.mo(mo).z < state.world.p_mobj.mo(mo).floorz
    {
        let mo_player = state
            .game
            .g_game
            .player_mut(state.world.p_mobj.mo(mo).player.unwrap());
        mo_player.viewheight -= state.world.p_mobj.mo(mo).floorz - state.world.p_mobj.mo(mo).z;
        mo_player.deltaviewheight = (VIEWHEIGHT - mo_player.viewheight) >> 3;
    }
    state.world.p_mobj.mo_mut(mo).z += state.world.p_mobj.mo(mo).momz;
    let mo_target = state
        .world
        .p_mobj
        .mo(mo)
        .target
        .filter(|&id| state.world.p_mobj.is_live(id));
    let mo_flags = state.world.p_mobj.mo(mo).flags;
    if let Some(target) = mo_target.filter(|_| {
        mo_flags.contains(MobjFlags::FLOAT)
            && !mo_flags.contains(MobjFlags::SKULLFLY)
            && !mo_flags.contains(MobjFlags::INFLOAT)
    }) {
        let dist: Fixed = aprox_distance(
            state.world.p_mobj.mo(mo).x - state.world.p_mobj.mo(target).x,
            state.world.p_mobj.mo(mo).y - state.world.p_mobj.mo(target).y,
        );
        let delta: Fixed = state.world.p_mobj.mo(target).z
            + (state.world.p_mobj.mo(mo).height >> 1)
            - state.world.p_mobj.mo(mo).z;
        if delta < 0 && dist < -(delta * 3) {
            state.world.p_mobj.mo_mut(mo).z -= FLOATSPEED;
        } else if delta > 0 && dist < delta * 3 {
            state.world.p_mobj.mo_mut(mo).z += FLOATSPEED;
        }
    }
    if state.world.p_mobj.mo(mo).z <= state.world.p_mobj.mo(mo).floorz {
        let correct_lost_soul_bounce: i32 =
            i32::from(state.game.doomstat.gameversion.is_ultimate_or_higher());
        if correct_lost_soul_bounce != 0
            && state
                .world
                .p_mobj
                .mo(mo)
                .flags
                .contains(MobjFlags::SKULLFLY)
        {
            state.world.p_mobj.mo_mut(mo).momz = -state.world.p_mobj.mo(mo).momz;
        }
        if state.world.p_mobj.mo(mo).momz < 0 {
            if state.world.p_mobj.mo(mo).player.is_some()
                && state.world.p_mobj.mo(mo).momz < -GRAVITY * 8
            {
                state
                    .game
                    .g_game
                    .player_mut(state.world.p_mobj.mo(mo).player.unwrap())
                    .deltaviewheight = state.world.p_mobj.mo(mo).momz >> 3;
                s_start_sound(state, SoundOrigin::Mobj(mo), SfxName::Oof);
            }
            state.world.p_mobj.mo_mut(mo).momz = 0;
        }
        state.world.p_mobj.mo_mut(mo).z = state.world.p_mobj.mo(mo).floorz;
        if correct_lost_soul_bounce == 0
            && state
                .world
                .p_mobj
                .mo(mo)
                .flags
                .contains(MobjFlags::SKULLFLY)
        {
            state.world.p_mobj.mo_mut(mo).momz = -state.world.p_mobj.mo(mo).momz;
        }
        if state.world.p_mobj.mo(mo).flags.contains(MobjFlags::MISSILE)
            && !state.world.p_mobj.mo(mo).flags.contains(MobjFlags::NOCLIP)
        {
            explode_missile(state, mo);
            return;
        }
    } else if !state
        .world
        .p_mobj
        .mo(mo)
        .flags
        .contains(MobjFlags::NOGRAVITY)
    {
        if state.world.p_mobj.mo(mo).momz == 0 {
            state.world.p_mobj.mo_mut(mo).momz = (-GRAVITY * 2) as Fixed;
        } else {
            state.world.p_mobj.mo_mut(mo).momz -= GRAVITY;
        }
    }
    if state.world.p_mobj.mo(mo).z + state.world.p_mobj.mo(mo).height
        > state.world.p_mobj.mo(mo).ceilingz
    {
        if state.world.p_mobj.mo(mo).momz > 0 {
            state.world.p_mobj.mo_mut(mo).momz = 0;
        }
        state.world.p_mobj.mo_mut(mo).z =
            state.world.p_mobj.mo(mo).ceilingz - state.world.p_mobj.mo(mo).height;
        if state
            .world
            .p_mobj
            .mo(mo)
            .flags
            .contains(MobjFlags::SKULLFLY)
        {
            state.world.p_mobj.mo_mut(mo).momz = -state.world.p_mobj.mo(mo).momz;
        }
        if state.world.p_mobj.mo(mo).flags.contains(MobjFlags::MISSILE)
            && !state.world.p_mobj.mo(mo).flags.contains(MobjFlags::NOCLIP)
        {
            explode_missile(state, mo);
        }
    }
}
pub fn nightmare_respawn(state: &mut GameState, mobj: MobjId) {
    let spawnpoint = state.world.p_mobj.mo(mobj).spawnpoint;
    let x = (i32::from(spawnpoint.x) << FRACBITS) as Fixed;
    let y = (i32::from(spawnpoint.y) << FRACBITS) as Fixed;
    if !check_position(state, mobj, x, y) {
        return;
    }
    let (mobj_x, mobj_y, mobj_subsector, mobj_type) = {
        let m = state.world.p_mobj.mo(mobj);
        (m.x, m.y, m.subsector, m.kind)
    };
    let floorheight1 = state
        .world
        .p_setup
        .sector_mut(state.world.p_setup.subsectors[mobj_subsector.0 as usize].sector)
        .floorheight;
    let fog = spawn_mobj(state, mobj_x, mobj_y, floorheight1, MobjType::Tfog);
    s_start_sound(state, SoundOrigin::Mobj(fog), SfxName::Telept);
    let ss = point_in_subsector(&state.world.p_setup, x, y);
    let floorheight2 = state
        .world
        .p_setup
        .sector_mut(state.world.p_setup.subsectors[ss.0 as usize].sector)
        .floorheight;
    let fog = spawn_mobj(state, x, y, floorheight2, MobjType::Tfog);
    s_start_sound(state, SoundOrigin::Mobj(fog), SfxName::Telept);
    let z = if state
        .assets
        .info
        .mobjinfo_mut(mobj_type)
        .flags
        .contains(MobjFlags::SPAWNCEILING)
    {
        ONCEILINGZ as Fixed
    } else {
        ONFLOORZ as Fixed
    };
    let mo = spawn_mobj(state, x, y, z, mobj_type);
    {
        let m = state.world.p_mobj.mo_mut(mo);
        m.spawnpoint = spawnpoint;
        m.angle = ANG45 * (i32::from(spawnpoint.angle) / 45) as u32;
        if i32::from(spawnpoint.options) & MTF_AMBUSH != 0 {
            m.flags |= MobjFlags::AMBUSH;
        }
        m.reactiontime = 18;
    }
    remove_mobj(state, mobj);
}
pub fn mobj_thinker(state: &mut GameState, id: MobjId) {
    let removed = |state: &GameState| {
        matches!(
            state.world.p_mobj.mo(id).thinker.function,
            ThinkerFn::Removed
        )
    };
    {
        let m = state.world.p_mobj.mo(id);
        if m.momx != 0 || m.momy != 0 || m.flags.contains(MobjFlags::SKULLFLY) {
            xymovement(state, id);
            if removed(state) {
                return;
            }
        }
    }
    {
        let m = state.world.p_mobj.mo(id);
        if m.z != m.floorz || m.momz != 0 {
            zmovement(state, id);
            if removed(state) {
                return;
            }
        }
    }
    if state.world.p_mobj.mo(id).tics == -1 {
        if !state
            .world
            .p_mobj
            .mo(id)
            .flags
            .contains(MobjFlags::COUNTKILL)
        {
            return;
        }
        if !state.game.g_game.respawnmonsters {
            return;
        }
        state.world.p_mobj.mo_mut(id).movecount += 1;
        if state.world.p_mobj.mo(id).movecount < 12 * TICRATE {
            return;
        }
        if state.world.p_tick.leveltime & 31 != 0 {
            return;
        }
        if p_random(&mut state.world.m_random) > 4 {
            return;
        }
        nightmare_respawn(state, id);
    } else {
        state.world.p_mobj.mo_mut(id).tics -= 1;
        if state.world.p_mobj.mo(id).tics == 0 {
            let current = state.world.p_mobj.mo(id).state.unwrap();
            let nextstate = state.assets.info.state_mut(current).nextstate;
            set_mobj_state(state, id, nextstate);
        }
    }
}
pub fn spawn_mobj(state: &mut GameState, x: Fixed, y: Fixed, z: Fixed, kind: MobjType) -> MobjId {
    // Built as a local value (starting from the same all-defaults template
    // used for PMobjState::dummy_mobj); spawn() moves it into the arena once
    // fully populated.
    let mut value = state.world.p_mobj.dummy_mobj;
    let (radius, height, flags, spawnhealth, reactiontime, spawnstate) = {
        let info = state.assets.info.mobjinfo_mut(kind);
        (
            info.radius,
            info.height,
            info.flags,
            info.spawnhealth,
            info.reactiontime,
            info.spawnstate,
        )
    };
    value.kind = kind;
    value.x = x;
    value.y = y;
    value.radius = radius as Fixed;
    value.height = height as Fixed;
    value.flags = flags;
    value.health = spawnhealth;
    if state.game.g_game.gameskill != SkillType::Nightmare {
        value.reactiontime = reactiontime;
    }
    value.lastlook = PlayerId((p_random(&mut state.world.m_random) % MAXPLAYERS as i32) as u8);
    let spawnstate_id = StateId(spawnstate as u32);
    let (tics, sprite, frame) = {
        let st = state.assets.info.state_mut(spawnstate_id);
        (st.tics, st.sprite, st.frame)
    };
    value.state = Some(spawnstate_id);
    value.tics = tics;
    value.sprite = sprite;
    value.frame = frame;
    let id = state.world.p_mobj.spawn(value);
    set_thing_position(&mut state.world.p_mobj, &mut state.world.p_setup, id);
    let subsector = state.world.p_mobj.mo(id).subsector;
    let sector = state.world.p_setup.subsectors[subsector.0 as usize].sector;
    let (floorz, ceilingz) = {
        let s = state.world.p_setup.sector_mut(sector);
        (s.floorheight, s.ceilingheight)
    };
    let final_z = if z == ONFLOORZ {
        floorz
    } else if z == ONCEILINGZ {
        (ceilingz - state.assets.info.mobjinfo_mut(kind).height) as Fixed
    } else {
        z
    };
    {
        let m = state.world.p_mobj.mo_mut(id);
        m.floorz = floorz;
        m.ceilingz = ceilingz;
        m.z = final_z;
        m.thinker.function = ThinkerFn::Mobj(mobj_thinker);
    }
    add_thinker(
        &mut state.world.p_tick,
        ThinkerPayload::Mobj(id),
        ThinkerKind::Mobj,
    );
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
    /// vanilla-demo-compatibility overrun emulation in `p_maputl.rs`.
    pub fn raw_index(self) -> u32 {
        self.index
    }
}

struct MobjSlot {
    generation: u32,
    // Owns the mobj's memory (unlike the raw pointer this replaces) -- see
    // PMobjState::spawn/retire/deallocate for why retirement and
    // deallocation are two separate steps despite that.
    mobj: Option<Box<Mobj>>,
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
    pub itemrespawnque: [MapThing; 128],
    pub itemrespawntime: [i32; 128],
    pub iquehead: i32,
    pub iquetail: i32,
    pub dummy_mobj: Mobj,
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
    // Takes a fully-constructed Mobj *value*, moves it onto the heap (a
    // fresh Box, not a Z_Malloc'd block), and hands back both a stable
    // generation-checked handle and a raw pointer for the caller's
    // remaining post-spawn field writes (set_thing_position, floorz/
    // ceilingz/z, thinker linkage -- exactly as before). The only two call
    // sites are spawn_mobj and p_saveg.rs's un_archive_thinkers
    // mobj-reconstruction branch -- the only two places that construct a
    // Mobj from scratch.
    pub fn spawn(&mut self, mut value: Mobj) -> MobjId {
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
    // raw pointer (the reaper, mid-deferred-free-schedule; remove_mobj's
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
    // from exactly the two places that used to Z_Free a mobj: run_thinkers'
    // reaper, and un_archive_thinkers' pre-load teardown loop.
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
    pub fn mobj_ref(&self, id: MobjId) -> Option<&Mobj> {
        self.mobjs
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.mobj.as_deref())
    }

    pub fn mobj_mut(&mut self, id: MobjId) -> Option<&mut Mobj> {
        self.mobjs
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.mobj.as_deref_mut())
    }

    // Panicking shorthands over mobj_ref/mobj_mut for ids that must be live.
    pub fn mo(&self, id: MobjId) -> &Mobj {
        self.mobj_ref(id).expect("stale MobjId")
    }

    pub fn mo_mut(&mut self, id: MobjId) -> &mut Mobj {
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
        Self {
            test: 0,
            itemrespawnque: [MapThing {
                x: 0,
                y: 0,
                angle: 0,
                kind: 0,
                options: 0,
            }; 128],
            itemrespawntime: [0; 128],
            iquehead: 0,
            iquetail: 0,
            mobjs: Vec::new(),
            free_list: Vec::new(),
            dummy_id: None,
            dummy_mobj: Mobj {
                thinker: Thinker {
                    function: ThinkerFn::Paused,
                },
                x: 0,
                y: 0,
                z: 0,
                snext: None,
                sprev: None,
                angle: Angle::ZERO,
                sprite: SpriteNum::Troo,
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
                kind: MobjType::Player,
                tics: 0,
                state: None,
                flags: MobjFlags::empty(),
                health: 0,
                movedir: 0,
                movecount: 0,
                target: None,
                reactiontime: 0,
                threshold: 0,
                player: None,
                lastlook: PlayerId(0),
                spawnpoint: MapThing {
                    x: 0,
                    y: 0,
                    angle: 0,
                    kind: 0,
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

pub fn remove_mobj(state: &mut GameState, mobj: MobjId) {
    state.world.p_mobj.retire(mobj);
    let (flags, kind, spawnpoint) = {
        let m = state.world.p_mobj.mo(mobj);
        (m.flags, m.kind, m.spawnpoint)
    };
    if flags.contains(MobjFlags::SPECIAL)
        && !flags.contains(MobjFlags::DROPPED)
        && kind as u32 != MobjType::Inv as i32 as u32
        && kind as u32 != MobjType::Ins as i32 as u32
    {
        state.world.p_mobj.itemrespawnque[state.world.p_mobj.iquehead as usize] = spawnpoint;
        state.world.p_mobj.itemrespawntime[state.world.p_mobj.iquehead as usize] =
            state.world.p_tick.leveltime;
        state.world.p_mobj.iquehead = (state.world.p_mobj.iquehead + 1) & (ITEMQUESIZE - 1);
        if state.world.p_mobj.iquehead == state.world.p_mobj.iquetail {
            state.world.p_mobj.iquetail = (state.world.p_mobj.iquetail + 1) & (ITEMQUESIZE - 1);
        }
    }
    unset_thing_position(&mut state.world.p_mobj, &mut state.world.p_setup, mobj);
    s_stop_sound(
        &mut state.audio.i_sound,
        &mut state.audio.s_sound,
        &mut state.audio.sounds,
        SoundOrigin::Mobj(mobj),
    );
    remove_thinker(&mut state.world.p_mobj.mo_mut(mobj).thinker);
}
pub fn respawn_specials(state: &mut GameState) {
    if state.game.g_game.deathmatch != 2 {
        return;
    }
    if state.world.p_mobj.iquehead == state.world.p_mobj.iquetail {
        return;
    }
    if state.world.p_tick.leveltime
        - state.world.p_mobj.itemrespawntime[state.world.p_mobj.iquetail as usize]
        < 30 * TICRATE
    {
        return;
    }
    let mthing = state.world.p_mobj.itemrespawnque[state.world.p_mobj.iquetail as usize];
    let x = (i32::from(mthing.x) << FRACBITS) as Fixed;
    let y = (i32::from(mthing.y) << FRACBITS) as Fixed;
    let ss = point_in_subsector(&state.world.p_setup, x, y);
    let floorheight = state
        .world
        .p_setup
        .sector_mut(state.world.p_setup.subsectors[ss.0 as usize].sector)
        .floorheight;
    let fog = spawn_mobj(state, x, y, floorheight, MobjType::Ifog);
    s_start_sound(state, SoundOrigin::Mobj(fog), SfxName::Itmbk);
    let i = state
        .assets
        .info
        .mobjinfo
        .iter()
        .position(|info| info.doomednum == i32::from(mthing.kind))
        .expect("a respawned map thing has a known type");
    let z = if state.assets.info.mobjinfo[i]
        .flags
        .contains(MobjFlags::SPAWNCEILING)
    {
        ONCEILINGZ as Fixed
    } else {
        ONFLOORZ as Fixed
    };
    let mo = spawn_mobj(state, x, y, z, mobjtype_from_raw(i as i32));
    {
        let m = state.world.p_mobj.mo_mut(mo);
        m.spawnpoint = mthing;
        m.angle = ANG45 * (i32::from(mthing.angle) / 45) as u32;
    }
    state.world.p_mobj.iquetail = (state.world.p_mobj.iquetail + 1) & (ITEMQUESIZE - 1);
}
pub fn spawn_player(state: &mut GameState, mthing: MapThing) {
    if i32::from(mthing.kind) == 0 {
        return;
    }
    let player_index = (i32::from(mthing.kind) - 1) as usize;
    if !state.game.g_game.playeringame[player_index] {
        return;
    }
    if state.game.g_game.players[player_index].playerstate == PlayerState::Reborn {
        player_reborn(&mut state.game.g_game, PlayerId(player_index as u8));
    }
    let x = (i32::from(mthing.x) << FRACBITS) as Fixed;
    let y = (i32::from(mthing.y) << FRACBITS) as Fixed;
    let z = ONFLOORZ as Fixed;
    let mobj = spawn_mobj(state, x, y, z, MobjType::Player);
    let player_health = state.game.g_game.players[player_index].health;
    {
        let m = state.world.p_mobj.mo_mut(mobj);
        if i32::from(mthing.kind) > 1 {
            m.flags |= MobjFlags::from_bits_retain(
                (i32::from(mthing.kind) - 1) << MobjFlags::TRANSLATION_SHIFT,
            );
        }
        m.angle = ANG45 * (i32::from(mthing.angle) / 45) as u32;
        m.player = Some(PlayerId(player_index as u8));
        m.health = player_health;
    }
    {
        let p = &mut state.game.g_game.players[player_index];
        p.mo = Some(mobj);
        p.playerstate = PlayerState::Live;
        p.refire = 0;
        p.message = None;
        p.damagecount = 0;
        p.bonuscount = 0;
        p.extralight = 0;
        p.fixedcolormap = 0;
        p.viewheight = VIEWHEIGHT as Fixed;
    }
    setup_psprites(state, PlayerId(player_index as u8));
    if state.game.g_game.deathmatch != 0 {
        for i in 0..NUMCARDS {
            state.game.g_game.players[player_index].cards[i] = true;
        }
    }
    if i32::from(mthing.kind) - 1 == state.game.g_game.consoleplayer.as_i32() {
        {
            st_start(state);
            hu_start(state);
        }
    }
}
pub fn spawn_map_thing(state: &mut GameState, mthing: MapThing) {
    if i32::from(mthing.kind) == 11 {
        if state.world.p_setup.deathmatch_p < 10 {
            let idx = state.world.p_setup.deathmatch_p;
            state.world.p_setup.deathmatchstarts[idx] = mthing;
            state.world.p_setup.deathmatch_p += 1;
        }
        return;
    }
    if i32::from(mthing.kind) <= 0 {
        return;
    }
    if i32::from(mthing.kind) <= 4 {
        state.world.p_setup.playerstarts[(i32::from(mthing.kind) - 1) as usize] = mthing;
        if state.game.g_game.deathmatch == 0 {
            spawn_player(state, mthing);
        }
        return;
    }
    if !state.game.g_game.netgame && i32::from(mthing.options) & 16 != 0 {
        return;
    }
    let bit: i32 = if state.game.g_game.gameskill == SkillType::Baby {
        1
    } else if state.game.g_game.gameskill == SkillType::Nightmare {
        4
    } else {
        1 << (state.game.g_game.gameskill as i32 - 1)
    };
    if i32::from(mthing.options) & bit == 0 {
        return;
    }
    let Some(i) = state
        .assets
        .info
        .mobjinfo
        .iter()
        .position(|info| info.doomednum == i32::from(mthing.kind))
    else {
        error(&format!(
            "P_SpawnMapThing: Unknown type {} at ({}, {})",
            i32::from(mthing.kind),
            i32::from(mthing.x),
            i32::from(mthing.y),
        ));
    };
    if state.game.g_game.deathmatch != 0
        && state.assets.info.mobjinfo[i]
            .flags
            .contains(MobjFlags::NOTDMATCH)
    {
        return;
    }
    if state.game.d_main.nomonsters
        && (i == MobjType::Skull as usize
            || state.assets.info.mobjinfo[i]
                .flags
                .contains(MobjFlags::COUNTKILL))
    {
        return;
    }
    let x = (i32::from(mthing.x) << FRACBITS) as Fixed;
    let y = (i32::from(mthing.y) << FRACBITS) as Fixed;
    let z = if state.assets.info.mobjinfo[i]
        .flags
        .contains(MobjFlags::SPAWNCEILING)
    {
        ONCEILINGZ as Fixed
    } else {
        ONFLOORZ as Fixed
    };
    let mobj = spawn_mobj(state, x, y, z, mobjtype_from_raw(i as i32));
    state.world.p_mobj.mo_mut(mobj).spawnpoint = mthing;
    if state.world.p_mobj.mo(mobj).tics > 0 {
        let tics = state.world.p_mobj.mo(mobj).tics;
        state.world.p_mobj.mo_mut(mobj).tics = 1 + p_random(&mut state.world.m_random) % tics;
    }
    let flags = state.world.p_mobj.mo(mobj).flags;
    if flags.contains(MobjFlags::COUNTKILL) {
        state.game.g_game.totalkills += 1;
    }
    if flags.contains(MobjFlags::COUNTITEM) {
        state.game.g_game.totalitems += 1;
    }
    let m = state.world.p_mobj.mo_mut(mobj);
    m.angle = ANG45 * (i32::from(mthing.angle) / 45) as u32;
    if i32::from(mthing.options) & MTF_AMBUSH != 0 {
        m.flags |= MobjFlags::AMBUSH;
    }
}
pub fn spawn_puff(state: &mut GameState, x: Fixed, y: Fixed, mut z: Fixed) {
    z += (p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 10;
    let th = spawn_mobj(state, x, y, z, MobjType::Puff);
    state.world.p_mobj.mo_mut(th).momz = FRACUNIT as Fixed;
    state.world.p_mobj.mo_mut(th).tics -= p_random(&mut state.world.m_random) & 3;
    if state.world.p_mobj.mo(th).tics < 1 {
        state.world.p_mobj.mo_mut(th).tics = 1;
    }
    if state.world.p_map.attackrange == MELEERANGE {
        set_mobj_state(state, th, StateNum::Puff3);
    }
}
pub fn spawn_blood(state: &mut GameState, x: Fixed, y: Fixed, mut z: Fixed, damage: i32) {
    z += (p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 10;
    let th = spawn_mobj(state, x, y, z, MobjType::Blood);
    state.world.p_mobj.mo_mut(th).momz = (FRACUNIT * 2) as Fixed;
    state.world.p_mobj.mo_mut(th).tics -= p_random(&mut state.world.m_random) & 3;
    if state.world.p_mobj.mo(th).tics < 1 {
        state.world.p_mobj.mo_mut(th).tics = 1;
    }
    if (9..=12).contains(&damage) {
        set_mobj_state(state, th, StateNum::Blood2);
    } else if damage < 9 {
        set_mobj_state(state, th, StateNum::Blood3);
    }
}
pub fn check_missile_spawn(state: &mut GameState, th: MobjId) {
    state.world.p_mobj.mo_mut(th).tics -= p_random(&mut state.world.m_random) & 3;
    let (x, y) = {
        let t = state.world.p_mobj.mo_mut(th);
        if t.tics < 1 {
            t.tics = 1;
        }
        t.x += t.momx >> 1;
        t.y += t.momy >> 1;
        t.z += t.momz >> 1;
        (t.x, t.y)
    };
    if !try_move(state, th, x, y) {
        explode_missile(state, th);
    }
}
// Stand-in for a missing target (vanilla's `dummy_mobj`): a permanent arena
// slot, so callers can treat it like any other mobj id. Its position/flags
// are reset on every substitution, as vanilla does.
pub fn subst_null_mobj(state: &mut PMobjState, mobj: Option<MobjId>) -> MobjId {
    if let Some(id) = mobj {
        return id;
    }
    let id = if let Some(id) = state.dummy_id.filter(|&id| state.mobj_ref(id).is_some()) {
        id
    } else {
        let template = state.dummy_mobj;
        let id = state.spawn(template);
        state.dummy_id = Some(id);
        id
    };
    let dummy = state.mo_mut(id);
    dummy.x = 0;
    dummy.y = 0;
    dummy.z = 0;
    dummy.flags = MobjFlags::empty();
    id
}
pub fn spawn_missile(
    state: &mut GameState,
    source: MobjId,
    dest: MobjId,
    kind: MobjType,
) -> MobjId {
    let (sx, sy, sz) = {
        let s = state.world.p_mobj.mo(source);
        (s.x, s.y, s.z)
    };
    let (dx, dy, dz, dflags) = {
        let d = state.world.p_mobj.mo(dest);
        (d.x, d.y, d.z, d.flags)
    };
    let th = spawn_mobj(state, sx, sy, sz + 4 * 8 * FRACUNIT, kind);
    let th_type = state.world.p_mobj.mo(th).kind;
    let seesound = state.assets.info.mobjinfo_mut(th_type).seesound;
    if seesound != SfxName::SfxNone {
        s_start_sound(state, SoundOrigin::Mobj(th), seesound);
    }
    state.world.p_mobj.mo_mut(th).target = Some(source);
    let mut an: Angle = point_to_angle2(sx, sy, dx, dy);
    if dflags.contains(MobjFlags::SHADOW) {
        an += Angle(
            ((p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 20)
                as u32,
        );
    }
    state.world.p_mobj.mo_mut(th).angle = an;
    let an = an.fine();
    let speed = state.assets.info.mobjinfo_mut(th_type).speed;
    {
        let t = state.world.p_mobj.mo_mut(th);
        t.momx = fixed_mul(speed as Fixed, FINECOSINE[an]);
        t.momy = fixed_mul(speed as Fixed, FINESINE[an]);
    }
    let mut dist: i32 = aprox_distance(dx - sx, dy - sy);
    dist /= speed;
    if dist < 1 {
        dist = 1;
    }
    state.world.p_mobj.mo_mut(th).momz = ((dz - sz) / dist) as Fixed;
    check_missile_spawn(state, th);
    th
}

pub fn spawn_player_missile(state: &mut GameState, source: MobjId, kind: MobjType) {
    let mut an: Angle = state.world.p_mobj.mo(source).angle;
    let mut slope = aim_line_attack(state, Some(source), an, 16 * 64 * FRACUNIT);
    if state.world.p_map.linetarget.is_none() {
        an += Angle((1 << 26) as u32);
        slope = aim_line_attack(state, Some(source), an, 16 * 64 * FRACUNIT);
        if state.world.p_map.linetarget.is_none() {
            an -= Angle((2 << 26) as u32);
            slope = aim_line_attack(state, Some(source), an, 16 * 64 * FRACUNIT);
        }
        if state.world.p_map.linetarget.is_none() {
            an = state.world.p_mobj.mo(source).angle;
            slope = 0;
        }
    }
    let (x, y, z) = {
        let s = state.world.p_mobj.mo(source);
        (s.x, s.y, (s.z + 4 * 8 * FRACUNIT) as Fixed)
    };
    let th = spawn_mobj(state, x, y, z, kind);
    let th_type = state.world.p_mobj.mo(th).kind;
    let seesound = state.assets.info.mobjinfo_mut(th_type).seesound;
    if seesound != SfxName::SfxNone {
        s_start_sound(state, SoundOrigin::Mobj(th), seesound);
    }
    let speed = state.assets.info.mobjinfo_mut(th_type).speed;
    {
        let t = state.world.p_mobj.mo_mut(th);
        t.target = Some(source);
        t.angle = an;
        t.momx = fixed_mul(speed as Fixed, FINECOSINE[an.fine()]);
        t.momy = fixed_mul(speed as Fixed, FINESINE[an.fine()]);
        t.momz = fixed_mul(speed as Fixed, slope);
    }
    check_missile_spawn(state, th);
}
