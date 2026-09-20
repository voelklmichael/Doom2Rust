/// The difficulty levels. The values 0..=4 are what demo files, savegames and net game settings
/// store, so `Baby = 0` is spelled out.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillType {
    Baby = 0,
    Easy,
    Medium,
    Hard,
    Nightmare,
}

impl SkillType {
    /// The level a stored or typed value stands for, `None` for anything outside 0..=4. That
    /// includes vanilla's `sk_noitems = -1` "no skill": it existed but no path let it reach the
    /// game (only `-skill 0` produced it, which then shifted by a negative amount).
    pub const fn from_raw(value: i32) -> Option<Self> {
        Some(match value {
            0 => Self::Baby,
            1 => Self::Easy,
            2 => Self::Medium,
            3 => Self::Hard,
            4 => Self::Nightmare,
            _ => return None,
        })
    }
}

/// [`SkillType::from_raw`] for a value that must be a level (savegame, demo header, menu, net
/// settings): anything else is fatal.
pub fn skill_from_raw(v: i32) -> SkillType {
    SkillType::from_raw(v).unwrap_or_else(|| panic!("invalid skill level {v}"))
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

#[derive(Copy, Clone, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::SkillType;

    #[test]
    fn skill_levels_keep_the_values_the_file_formats_store() {
        for (value, level) in [
            (0, SkillType::Baby),
            (1, SkillType::Easy),
            (2, SkillType::Medium),
            (3, SkillType::Hard),
            (4, SkillType::Nightmare),
        ] {
            assert_eq!(SkillType::from_raw(value), Some(level));
            assert_eq!(level as i32, value);
        }
    }

    #[test]
    fn a_value_outside_the_five_levels_is_no_skill() {
        assert_eq!(SkillType::from_raw(-1), None); // vanilla's sk_noitems
        assert_eq!(SkillType::from_raw(5), None);
        assert_eq!(SkillType::from_raw(255), None);
    }
}
