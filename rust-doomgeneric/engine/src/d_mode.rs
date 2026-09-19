#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillType {
    Noitems = -1,
    Baby = 0,
    Easy = 1,
    Medium = 2,
    Hard = 3,
    Nightmare = 4,
}
pub fn skill_from_raw(v: i32) -> SkillType {
    match v {
        -1 => SkillType::Noitems,
        0 => SkillType::Baby,
        1 => SkillType::Easy,
        2 => SkillType::Medium,
        3 => SkillType::Hard,
        4 => SkillType::Nightmare,
        n => panic!("invalid skill level {n}"),
    }
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameMission {
    Doom = 0,
    Doom2 = 1,
    PackTnt = 2,
    PackPlut = 3,
    PackChex = 4,
    PackHacx = 5,
    Heretic = 6,
    Hexen = 7,
    Strife = 8,
    None = 9,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameMode {
    Shareware = 0,
    Registered = 1,
    Commercial = 2,
    Retail = 3,
    Indetermined = 4,
}

#[derive(Copy, Clone, PartialEq)]
pub enum GameVersion {
    Doom12 = 0,
    Doom1666 = 1,
    Doom17 = 2,
    Doom18 = 3,
    Doom19 = 4,
    Hacx = 5,
    Ultimate = 6,
    Final = 7,
    Final2 = 8,
    Chex = 9,
    Heretic13 = 10,
    Hexen11 = 11,
    Strife12 = 12,
    Strife131 = 13,
}
impl GameVersion {
    pub(crate) fn is_ultimate_or_higher(&self) -> bool {
        match self {
            GameVersion::Doom12
            | GameVersion::Doom1666
            | GameVersion::Doom17
            | GameVersion::Doom18
            | GameVersion::Doom19
            | GameVersion::Hacx => false,
            GameVersion::Ultimate
            | GameVersion::Final
            | GameVersion::Final2
            | GameVersion::Chex
            | GameVersion::Heretic13
            | GameVersion::Hexen11
            | GameVersion::Strife12
            | GameVersion::Strife131 => true,
        }
    }

    pub(crate) fn below_1_9(&self) -> bool {
        match self {
            GameVersion::Doom12
            | GameVersion::Doom1666
            | GameVersion::Doom17
            | GameVersion::Doom18
            | GameVersion::Doom19 => true,
            GameVersion::Hacx
            | GameVersion::Ultimate
            | GameVersion::Final
            | GameVersion::Final2
            | GameVersion::Chex
            | GameVersion::Heretic13
            | GameVersion::Hexen11
            | GameVersion::Strife12
            | GameVersion::Strife131 => false,
        }
    }
}
pub fn game_mission_string(mission: GameMission) -> &'static str {
    match mission {
        GameMission::Doom => "doom",
        GameMission::Doom2 => "doom2",
        GameMission::PackTnt => "tnt",
        GameMission::PackPlut => "plutonia",
        GameMission::PackHacx => "hacx",
        GameMission::PackChex => "chex",
        GameMission::Heretic => "heretic",
        GameMission::Hexen => "hexen",
        GameMission::Strife => "strife",
        GameMission::None => "none",
    }
}
