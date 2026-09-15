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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed {
    pub mission: GameMission_t,
    pub mode: GameMode_t,
    pub episode: i32,
    pub map: i32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub mission: GameMission_t,
    pub version: GameVersion,
}
static valid_modes: [C2RustUnnamed; 13] = [
    C2RustUnnamed {
        mission: GameMission_t::pack_chex,
        mode: GameMode_t::shareware,
        episode: 1_i32,
        map: 5_i32,
    },
    C2RustUnnamed {
        mission: GameMission_t::doom,
        mode: GameMode_t::shareware,
        episode: 1_i32,
        map: 9_i32,
    },
    C2RustUnnamed {
        mission: GameMission_t::doom,
        mode: GameMode_t::registered,
        episode: 3_i32,
        map: 9_i32,
    },
    C2RustUnnamed {
        mission: GameMission_t::doom,
        mode: GameMode_t::retail,
        episode: 4_i32,
        map: 9_i32,
    },
    C2RustUnnamed {
        mission: GameMission_t::doom2,
        mode: GameMode_t::commercial,
        episode: 1_i32,
        map: 32_i32,
    },
    C2RustUnnamed {
        mission: GameMission_t::pack_tnt,
        mode: GameMode_t::commercial,
        episode: 1_i32,
        map: 32_i32,
    },
    C2RustUnnamed {
        mission: GameMission_t::pack_plut,
        mode: GameMode_t::commercial,
        episode: 1_i32,
        map: 32_i32,
    },
    C2RustUnnamed {
        mission: GameMission_t::pack_hacx,
        mode: GameMode_t::commercial,
        episode: 1_i32,
        map: 32_i32,
    },
    C2RustUnnamed {
        mission: GameMission_t::heretic,
        mode: GameMode_t::shareware,
        episode: 1_i32,
        map: 9_i32,
    },
    C2RustUnnamed {
        mission: GameMission_t::heretic,
        mode: GameMode_t::registered,
        episode: 3_i32,
        map: 9_i32,
    },
    C2RustUnnamed {
        mission: GameMission_t::heretic,
        mode: GameMode_t::retail,
        episode: 5_i32,
        map: 9_i32,
    },
    C2RustUnnamed {
        mission: GameMission_t::hexen,
        mode: GameMode_t::commercial,
        episode: 1_i32,
        map: 60_i32,
    },
    C2RustUnnamed {
        mission: GameMission_t::strife,
        mode: GameMode_t::commercial,
        episode: 1_i32,
        map: 34_i32,
    },
];
pub fn D_ValidGameMode(mut mission: GameMission_t, mut mode: GameMode_t) -> bool {
    let mut i: i32 = 0;
    i = 0_i32;
    while (i as usize)
        < ::core::mem::size_of::<[C2RustUnnamed; 13]>()
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed>())
    {
        if valid_modes[i as usize].mode as u32 == mode as u32
            && valid_modes[i as usize].mission as u32 == mission as u32
        {
            return true;
        }
        i += 1;
    }
    false
}
pub fn D_ValidEpisodeMap(
    mut mission: GameMission_t,
    mut mode: GameMode_t,
    mut episode: i32,
    mut map: i32,
) -> bool {
    let mut i: i32 = 0;
    if mission as u32 == GameMission_t::heretic as i32 as u32 {
        if mode as u32 == GameMode_t::retail as i32 as u32 && episode == 6_i32 {
            return map >= 1_i32 && map <= 3_i32;
        } else if mode as u32 == GameMode_t::registered as i32 as u32 && episode == 4_i32 {
            return map == 1_i32;
        }
    }
    i = 0_i32;
    while (i as usize)
        < ::core::mem::size_of::<[C2RustUnnamed; 13]>()
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed>())
    {
        if mission as u32 == valid_modes[i as usize].mission as u32
            && mode as u32 == valid_modes[i as usize].mode as u32
        {
            return episode >= 1_i32
                && episode <= valid_modes[i as usize].episode
                && map >= 1_i32
                && map <= valid_modes[i as usize].map;
        }
        i += 1;
    }
    false
}
pub fn D_GetNumEpisodes(mut mission: GameMission_t, mut mode: GameMode_t) -> i32 {
    let mut episode: i32 = 0;
    episode = 1_i32;
    while D_ValidEpisodeMap(mission, mode, episode, 1_i32) {
        episode += 1;
    }
    return episode - 1_i32;
}
static valid_versions: [C2RustUnnamed_0; 10] = [
    C2RustUnnamed_0 {
        mission: GameMission_t::doom,
        version: GameVersion::doom_1_9,
    },
    C2RustUnnamed_0 {
        mission: GameMission_t::doom,
        version: GameVersion::hacx,
    },
    C2RustUnnamed_0 {
        mission: GameMission_t::doom,
        version: GameVersion::ultimate,
    },
    C2RustUnnamed_0 {
        mission: GameMission_t::doom,
        version: GameVersion::r#final,
    },
    C2RustUnnamed_0 {
        mission: GameMission_t::doom,
        version: GameVersion::final2,
    },
    C2RustUnnamed_0 {
        mission: GameMission_t::doom,
        version: GameVersion::chex,
    },
    C2RustUnnamed_0 {
        mission: GameMission_t::heretic,
        version: GameVersion::heretic_1_3,
    },
    C2RustUnnamed_0 {
        mission: GameMission_t::hexen,
        version: GameVersion::hexen_1_1,
    },
    C2RustUnnamed_0 {
        mission: GameMission_t::strife,
        version: GameVersion::strife_1_2,
    },
    C2RustUnnamed_0 {
        mission: GameMission_t::strife,
        version: GameVersion::strife_1_31,
    },
];
pub fn D_ValidGameVersion(mut mission: GameMission_t, mut version: GameVersion) -> bool {
    let mut i: i32 = 0;
    if mission as u32 == GameMission_t::doom2 as i32 as u32
        || mission as u32 == GameMission_t::pack_plut as i32 as u32
        || mission as u32 == GameMission_t::pack_tnt as i32 as u32
        || mission as u32 == GameMission_t::pack_hacx as i32 as u32
        || mission as u32 == GameMission_t::pack_chex as i32 as u32
    {
        mission = GameMission_t::doom;
    }
    i = 0_i32;
    while (i as usize)
        < ::core::mem::size_of::<[C2RustUnnamed_0; 10]>()
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_0>())
    {
        if valid_versions[i as usize].mission as u32 == mission as u32
            && valid_versions[i as usize].version as u32 == version as u32
        {
            return true;
        }
        i += 1;
    }
    false
}
pub fn D_IsEpisodeMap(mut mission: GameMission_t) -> bool {
    match mission {
        GameMission_t::doom | GameMission_t::heretic | GameMission_t::pack_chex => true,
        GameMission_t::none
        | GameMission_t::hexen
        | GameMission_t::doom2
        | GameMission_t::pack_hacx
        | GameMission_t::pack_tnt
        | GameMission_t::pack_plut
        | GameMission_t::strife => false,
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
