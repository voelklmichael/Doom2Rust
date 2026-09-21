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
use crate::p_setup::SideId;
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

use crate::tables::fine_cosine;
use crate::tables::fine_sine;
use crate::tables::Angle;
use crate::tables::ANG45;

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
raw_enum! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum SpriteNum, spritenum_from_raw, "spritenum" {
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
}
raw_enum! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum StateNum, statenum_from_raw, "statenum" {
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
raw_enum! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub enum MobjType, mobjtype_from_raw, "mobjtype" {
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
    pub radius: Fixed,
    pub height: Fixed,
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
    /// The sides in front of and behind the line (`None` where the line has no such side).
    pub sidenum: [Option<SideId>; 2],
    pub bbox: BBox,
    pub slopetype: SlopeType,
    pub frontsector: Option<SectorId>,
    pub backsector: Option<SectorId>,
    pub validcount: i32,
}
impl Line {
    /// The side in front of the line, which every line of a valid map has.
    pub fn front_side(&self) -> SideId {
        self.sidenum[0].expect("line without a front side")
    }
    /// The sector in front of the line, which every line of a valid map has.
    pub fn front_sector(&self) -> SectorId {
        self.frontsector.expect("line without a front sector")
    }
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
pub const FLOATSPEED: Fixed = Fixed::from_int(4);
pub const GRAVITY: Fixed = FRACUNIT;
pub const MAXMOVE: Fixed = Fixed::from_int(30);
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
    state.world.p_mobj.mo_mut(mo).momz = Fixed::ZERO;
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
    if momx == Fixed::ZERO && momy == Fixed::ZERO {
        if flags.contains(MobjFlags::SKULLFLY) {
            let mo_type = {
                let m = state.world.p_mobj.mo_mut(mo);
                m.flags &= !MobjFlags::SKULLFLY;
                m.momz = Fixed::ZERO;
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
            m.momx = MAXMOVE;
        } else if m.momx < (-MAXMOVE) {
            m.momx = -MAXMOVE;
        }
        if m.momy > MAXMOVE {
            m.momy = MAXMOVE;
        } else if m.momy < (-MAXMOVE) {
            m.momy = -MAXMOVE;
        }
        (m.momx, m.momy)
    };
    loop {
        let (mx, my) = {
            let m = state.world.p_mobj.mo(mo);
            (m.x, m.y)
        };
        let (ptryx, ptryy): (Fixed, Fixed) = if xmove > (MAXMOVE / 2) || ymove > (MAXMOVE / 2) {
            let ptry = (mx + xmove / 2, my + ymove / 2);
            xmove >>= 1;
            ymove >>= 1;
            ptry
        } else {
            let ptry = (mx + xmove, my + ymove);
            ymove = Fixed::ZERO;
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
                m.momy = Fixed::ZERO;
                m.momx = m.momy;
            }
        }
        if !(xmove != Fixed::ZERO || ymove != Fixed::ZERO) {
            break;
        }
    }
    if let Some(player_id) = player {
        if state.game.g_game.players[player_id]
            .cheats
            .contains(CheatFlags::NOMOMENTUM)
        {
            let m = state.world.p_mobj.mo_mut(mo);
            m.momy = Fixed::ZERO;
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
    if momx > Fixed(-STOPSPEED)
        && momx < Fixed(STOPSPEED)
        && momy > Fixed(-STOPSPEED)
        && momy < Fixed(STOPSPEED)
        && player_idle
    {
        if player.is_some()
            && (state
                .world
                .p_mobj
                .mo(mo)
                .state
                .expect("a thinking mobj has a state")
                .0
                .wrapping_sub(StateNum::PlayRun1 as u32))
                < 4
        {
            set_mobj_state(state, mo, StateNum::Play);
        }
        let m = state.world.p_mobj.mo_mut(mo);
        m.momx = Fixed::ZERO;
        m.momy = Fixed::ZERO;
    } else {
        let m = state.world.p_mobj.mo_mut(mo);
        m.momx = fixed_mul(m.momx, Fixed(FRICTION));
        m.momy = fixed_mul(m.momy, Fixed(FRICTION));
    }
}
pub fn zmovement(state: &mut GameState, mo: MobjId) {
    if state.world.p_mobj.mo(mo).player.is_some()
        && state.world.p_mobj.mo(mo).z < state.world.p_mobj.mo(mo).floorz
    {
        let mo_player = state.game.g_game.player_mut(
            state
                .world
                .p_mobj
                .mo(mo)
                .player
                .expect("a mobj that lands hard is a player"),
        );
        mo_player.viewheight -= state.world.p_mobj.mo(mo).floorz - state.world.p_mobj.mo(mo).z;
        mo_player.deltaviewheight = (VIEWHEIGHT - mo_player.viewheight) >> 3;
    }
    let momz = state.world.p_mobj.mo(mo).momz;
    state.world.p_mobj.mo_mut(mo).z += momz;
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
        if delta < Fixed::ZERO && dist < -(delta * 3) {
            state.world.p_mobj.mo_mut(mo).z -= FLOATSPEED;
        } else if delta > Fixed::ZERO && dist < delta * 3 {
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
        if state.world.p_mobj.mo(mo).momz < Fixed::ZERO {
            if state.world.p_mobj.mo(mo).player.is_some()
                && state.world.p_mobj.mo(mo).momz < (-GRAVITY * 8)
            {
                state
                    .game
                    .g_game
                    .player_mut(
                        state
                            .world
                            .p_mobj
                            .mo(mo)
                            .player
                            .expect("a mobj that lands hard is a player"),
                    )
                    .deltaviewheight = state.world.p_mobj.mo(mo).momz >> 3;
                s_start_sound(state, SoundOrigin::Mobj(mo), SfxName::Oof);
            }
            state.world.p_mobj.mo_mut(mo).momz = Fixed::ZERO;
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
        if state.world.p_mobj.mo(mo).momz == Fixed::ZERO {
            state.world.p_mobj.mo_mut(mo).momz = -GRAVITY * 2;
        } else {
            state.world.p_mobj.mo_mut(mo).momz -= GRAVITY;
        }
    }
    if state.world.p_mobj.mo(mo).z + state.world.p_mobj.mo(mo).height
        > state.world.p_mobj.mo(mo).ceilingz
    {
        if state.world.p_mobj.mo(mo).momz > Fixed::ZERO {
            state.world.p_mobj.mo_mut(mo).momz = Fixed::ZERO;
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
    let x = Fixed::from_int(i32::from(spawnpoint.x));
    let y = Fixed::from_int(i32::from(spawnpoint.y));
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
        ONCEILINGZ
    } else {
        ONFLOORZ
    };
    let mo = spawn_mobj(state, x, y, Fixed(z), mobj_type);
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
        if m.momx != Fixed::ZERO || m.momy != Fixed::ZERO || m.flags.contains(MobjFlags::SKULLFLY) {
            xymovement(state, id);
            if removed(state) {
                return;
            }
        }
    }
    {
        let m = state.world.p_mobj.mo(id);
        if m.z != m.floorz || m.momz != Fixed::ZERO {
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
            let current = state
                .world
                .p_mobj
                .mo(id)
                .state
                .expect("a thinking mobj has a state");
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
    value.radius = radius;
    value.height = height;
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
    let final_z = if z == Fixed(ONFLOORZ) {
        floorz
    } else if z == Fixed(ONCEILINGZ) {
        ceilingz - state.assets.info.mobjinfo_mut(kind).height
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
                x: Fixed::ZERO,
                y: Fixed::ZERO,
                z: Fixed::ZERO,
                snext: None,
                sprev: None,
                angle: Angle::ZERO,
                sprite: SpriteNum::Troo,
                frame: 0,
                bnext: None,
                bprev: None,
                subsector: SubsectorId(0),
                floorz: Fixed::ZERO,
                ceilingz: Fixed::ZERO,
                radius: Fixed::ZERO,
                height: Fixed::ZERO,
                momx: Fixed::ZERO,
                momy: Fixed::ZERO,
                momz: Fixed::ZERO,
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
    let x = Fixed::from_int(i32::from(mthing.x));
    let y = Fixed::from_int(i32::from(mthing.y));
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
        ONCEILINGZ
    } else {
        ONFLOORZ
    };
    let mo = spawn_mobj(state, x, y, Fixed(z), mobjtype_from_raw(i as i32));
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
    let x = Fixed::from_int(i32::from(mthing.x));
    let y = Fixed::from_int(i32::from(mthing.y));
    let z = ONFLOORZ;
    let mobj = spawn_mobj(state, x, y, Fixed(z), MobjType::Player);
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
        p.viewheight = VIEWHEIGHT;
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
    let x = Fixed::from_int(i32::from(mthing.x));
    let y = Fixed::from_int(i32::from(mthing.y));
    let z = if state.assets.info.mobjinfo[i]
        .flags
        .contains(MobjFlags::SPAWNCEILING)
    {
        ONCEILINGZ
    } else {
        ONFLOORZ
    };
    let mobj = spawn_mobj(state, x, y, Fixed(z), mobjtype_from_raw(i as i32));
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
    z += Fixed((p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 10);
    let th = spawn_mobj(state, x, y, z, MobjType::Puff);
    state.world.p_mobj.mo_mut(th).momz = FRACUNIT;
    state.world.p_mobj.mo_mut(th).tics -= p_random(&mut state.world.m_random) & 3;
    if state.world.p_mobj.mo(th).tics < 1 {
        state.world.p_mobj.mo_mut(th).tics = 1;
    }
    if state.world.p_map.attackrange == MELEERANGE {
        set_mobj_state(state, th, StateNum::Puff3);
    }
}
pub fn spawn_blood(state: &mut GameState, x: Fixed, y: Fixed, mut z: Fixed, damage: i32) {
    z += Fixed((p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 10);
    let th = spawn_mobj(state, x, y, z, MobjType::Blood);
    state.world.p_mobj.mo_mut(th).momz = FRACUNIT * 2;
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
    dummy.x = Fixed::ZERO;
    dummy.y = Fixed::ZERO;
    dummy.z = Fixed::ZERO;
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
        t.momx = fixed_mul(Fixed(speed), fine_cosine(an));
        t.momy = fixed_mul(Fixed(speed), fine_sine(an));
    }
    let mut dist: Fixed = aprox_distance(dx - sx, dy - sy);
    dist /= speed;
    if dist < Fixed(1) {
        dist = Fixed(1);
    }
    state.world.p_mobj.mo_mut(th).momz = Fixed((dz - sz) / dist);
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
            slope = Fixed::ZERO;
        }
    }
    let (x, y, z) = {
        let s = state.world.p_mobj.mo(source);
        (s.x, s.y, (s.z + 4 * 8 * FRACUNIT))
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
        t.momx = fixed_mul(Fixed(speed), fine_cosine(an.fine()));
        t.momy = fixed_mul(Fixed(speed), fine_sine(an.fine()));
        t.momz = fixed_mul(Fixed(speed), slope);
    }
    check_missile_spawn(state, th);
}
