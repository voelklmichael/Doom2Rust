#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillType {
    Noitems = -1,
    Baby,
    Easy,
    Medium,
    Hard,
    Nightmare,
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
    Doom,
    Doom2,
    PackTnt,
    PackPlut,
    PackChex,
    PackHacx,
    Heretic,
    Hexen,
    Strife,
    None,
}

impl GameMission {
    /// The game whose rules a mission pack plays by: Chex Quest is Doom, Hacx is Doom II.
    pub const fn base(self) -> Self {
        match self {
            Self::PackChex => Self::Doom,
            Self::PackHacx => Self::Doom2,
            other => other,
        }
    }
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameMode {
    Shareware,
    Registered,
    Commercial,
    Retail,
    Indetermined,
}

#[derive(Copy, Clone, PartialEq)]
pub enum GameVersion {
    Doom12,
    Doom1666,
    Doom17,
    Doom18,
    Doom19,
    Hacx,
    Ultimate,
    Final,
    Final2,
    Chex,
    Heretic13,
    Hexen11,
    Strife12,
    Strife131,
}
impl GameVersion {
    pub(crate) fn is_ultimate_or_higher(self) -> bool {
        match self {
            Self::Doom12
            | Self::Doom1666
            | Self::Doom17
            | Self::Doom18
            | Self::Doom19
            | Self::Hacx => false,
            Self::Ultimate
            | Self::Final
            | Self::Final2
            | Self::Chex
            | Self::Heretic13
            | Self::Hexen11
            | Self::Strife12
            | Self::Strife131 => true,
        }
    }

    pub(crate) fn below_1_9(self) -> bool {
        match self {
            Self::Doom12 | Self::Doom1666 | Self::Doom17 | Self::Doom18 | Self::Doom19 => true,
            Self::Hacx
            | Self::Ultimate
            | Self::Final
            | Self::Final2
            | Self::Chex
            | Self::Heretic13
            | Self::Hexen11
            | Self::Strife12
            | Self::Strife131 => false,
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
