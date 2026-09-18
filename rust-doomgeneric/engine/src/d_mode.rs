#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillType {
    sk_noitems = -1,
    sk_baby = 0,
    sk_easy = 1,
    sk_medium = 2,
    sk_hard = 3,
    sk_nightmare = 4,
}
pub fn skill_from_raw(v: i32) -> SkillType {
    match v {
        -1 => SkillType::sk_noitems,
        0 => SkillType::sk_baby,
        1 => SkillType::sk_easy,
        2 => SkillType::sk_medium,
        3 => SkillType::sk_hard,
        4 => SkillType::sk_nightmare,
        n => panic!("invalid skill level {n}"),
    }
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameMission_t {
    doom = 0,
    doom2 = 1,
    pack_tnt = 2,
    pack_plut = 3,
    pack_chex = 4,
    pack_hacx = 5,
    heretic = 6,
    hexen = 7,
    strife = 8,
    none = 9,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameMode_t {
    shareware = 0,
    registered = 1,
    commercial = 2,
    retail = 3,
    indetermined = 4,
}

#[derive(Copy, Clone, PartialEq)]
pub enum GameVersion {
    doom_1_2 = 0,
    doom_1_666 = 1,
    doom_1_7 = 2,
    doom_1_8 = 3,
    doom_1_9 = 4,
    hacx = 5,
    ultimate = 6,
    r#final = 7,
    final2 = 8,
    chex = 9,
    heretic_1_3 = 10,
    hexen_1_1 = 11,
    strife_1_2 = 12,
    strife_1_31 = 13,
}
impl GameVersion {
    pub(crate) fn is_ultimate_or_higher(&self) -> bool {
        match self {
            GameVersion::doom_1_2
            | GameVersion::doom_1_666
            | GameVersion::doom_1_7
            | GameVersion::doom_1_8
            | GameVersion::doom_1_9
            | GameVersion::hacx => false,
            GameVersion::ultimate
            | GameVersion::r#final
            | GameVersion::final2
            | GameVersion::chex
            | GameVersion::heretic_1_3
            | GameVersion::hexen_1_1
            | GameVersion::strife_1_2
            | GameVersion::strife_1_31 => true,
        }
    }

    pub(crate) fn below_1_9(&self) -> bool {
        match self {
            GameVersion::doom_1_2
            | GameVersion::doom_1_666
            | GameVersion::doom_1_7
            | GameVersion::doom_1_8
            | GameVersion::doom_1_9 => true,
            GameVersion::hacx
            | GameVersion::ultimate
            | GameVersion::r#final
            | GameVersion::final2
            | GameVersion::chex
            | GameVersion::heretic_1_3
            | GameVersion::hexen_1_1
            | GameVersion::strife_1_2
            | GameVersion::strife_1_31 => false,
        }
    }
}
pub fn D_GameMissionString(mission: GameMission_t) -> &'static str {
    match mission {
        GameMission_t::doom => "doom",
        GameMission_t::doom2 => "doom2",
        GameMission_t::pack_tnt => "tnt",
        GameMission_t::pack_plut => "plutonia",
        GameMission_t::pack_hacx => "hacx",
        GameMission_t::pack_chex => "chex",
        GameMission_t::heretic => "heretic",
        GameMission_t::hexen => "hexen",
        GameMission_t::strife => "strife",
        GameMission_t::none => "none",
    }
}
