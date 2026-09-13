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
pub type GameMission_t = u32;
pub const none: GameMission_t = 9;
pub const strife: GameMission_t = 8;
pub const hexen: GameMission_t = 7;
pub const heretic: GameMission_t = 6;
pub const pack_hacx: GameMission_t = 5;
pub const pack_chex: GameMission_t = 4;
pub const pack_plut: GameMission_t = 3;
pub const pack_tnt: GameMission_t = 2;
pub const doom2: GameMission_t = 1;
pub const doom: GameMission_t = 0;
pub type GameMode_t = u32;
pub const indetermined: GameMode_t = 4;
pub const retail: GameMode_t = 3;
pub const commercial: GameMode_t = 2;
pub const registered: GameMode_t = 1;
pub const shareware: GameMode_t = 0;

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
        mission: pack_chex,
        mode: shareware,
        episode: 1 as i32,
        map: 5 as i32,
    },
    C2RustUnnamed {
        mission: doom,
        mode: shareware,
        episode: 1 as i32,
        map: 9 as i32,
    },
    C2RustUnnamed {
        mission: doom,
        mode: registered,
        episode: 3 as i32,
        map: 9 as i32,
    },
    C2RustUnnamed {
        mission: doom,
        mode: retail,
        episode: 4 as i32,
        map: 9 as i32,
    },
    C2RustUnnamed {
        mission: doom2,
        mode: commercial,
        episode: 1 as i32,
        map: 32 as i32,
    },
    C2RustUnnamed {
        mission: pack_tnt,
        mode: commercial,
        episode: 1 as i32,
        map: 32 as i32,
    },
    C2RustUnnamed {
        mission: pack_plut,
        mode: commercial,
        episode: 1 as i32,
        map: 32 as i32,
    },
    C2RustUnnamed {
        mission: pack_hacx,
        mode: commercial,
        episode: 1 as i32,
        map: 32 as i32,
    },
    C2RustUnnamed {
        mission: heretic,
        mode: shareware,
        episode: 1 as i32,
        map: 9 as i32,
    },
    C2RustUnnamed {
        mission: heretic,
        mode: registered,
        episode: 3 as i32,
        map: 9 as i32,
    },
    C2RustUnnamed {
        mission: heretic,
        mode: retail,
        episode: 5 as i32,
        map: 9 as i32,
    },
    C2RustUnnamed {
        mission: hexen,
        mode: commercial,
        episode: 1 as i32,
        map: 60 as i32,
    },
    C2RustUnnamed {
        mission: strife,
        mode: commercial,
        episode: 1 as i32,
        map: 34 as i32,
    },
];
pub fn D_ValidGameMode(mut mission: GameMission_t, mut mode: GameMode_t) -> bool {
    let mut i: i32 = 0;
    i = 0 as i32;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed; 13]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed>() as usize)
    {
        if valid_modes[i as usize].mode as u32 == mode as u32
            && valid_modes[i as usize].mission as u32 == mission as u32
        {
            return true;
        }
        i += 1;
    }
    return false;
}
pub fn D_ValidEpisodeMap(
    mut mission: GameMission_t,
    mut mode: GameMode_t,
    mut episode: i32,
    mut map: i32,
) -> bool {
    let mut i: i32 = 0;
    if mission as u32 == heretic as i32 as u32 {
        if mode as u32 == retail as i32 as u32 && episode == 6 as i32 {
            return map >= 1 as i32 && map <= 3 as i32;
        } else if mode as u32 == registered as i32 as u32 && episode == 4 as i32 {
            return map == 1 as i32;
        }
    }
    i = 0 as i32;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed; 13]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed>() as usize)
    {
        if mission as u32 == valid_modes[i as usize].mission as u32
            && mode as u32 == valid_modes[i as usize].mode as u32
        {
            return episode >= 1 as i32
                && episode <= valid_modes[i as usize].episode
                && map >= 1 as i32
                && map <= valid_modes[i as usize].map;
        }
        i += 1;
    }
    return false;
}
pub fn D_GetNumEpisodes(mut mission: GameMission_t, mut mode: GameMode_t) -> i32 {
    let mut episode: i32 = 0;
    episode = 1 as i32;
    while D_ValidEpisodeMap(mission, mode, episode, 1 as i32) {
        episode += 1;
    }
    return episode - 1 as i32;
}
static valid_versions: [C2RustUnnamed_0; 10] = [
    C2RustUnnamed_0 {
        mission: doom,
        version: GameVersion::doom_1_9,
    },
    C2RustUnnamed_0 {
        mission: doom,
        version: GameVersion::hacx,
    },
    C2RustUnnamed_0 {
        mission: doom,
        version: GameVersion::ultimate,
    },
    C2RustUnnamed_0 {
        mission: doom,
        version: GameVersion::r#final,
    },
    C2RustUnnamed_0 {
        mission: doom,
        version: GameVersion::final2,
    },
    C2RustUnnamed_0 {
        mission: doom,
        version: GameVersion::chex,
    },
    C2RustUnnamed_0 {
        mission: heretic,
        version: GameVersion::heretic_1_3,
    },
    C2RustUnnamed_0 {
        mission: hexen,
        version: GameVersion::hexen_1_1,
    },
    C2RustUnnamed_0 {
        mission: strife,
        version: GameVersion::strife_1_2,
    },
    C2RustUnnamed_0 {
        mission: strife,
        version: GameVersion::strife_1_31,
    },
];
pub fn D_ValidGameVersion(mut mission: GameMission_t, mut version: GameVersion) -> bool {
    let mut i: i32 = 0;
    if mission as u32 == doom2 as i32 as u32
        || mission as u32 == pack_plut as i32 as u32
        || mission as u32 == pack_tnt as i32 as u32
        || mission as u32 == pack_hacx as i32 as u32
        || mission as u32 == pack_chex as i32 as u32
    {
        mission = doom;
    }
    i = 0 as i32;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_0; 10]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_0>() as usize)
    {
        if valid_versions[i as usize].mission as u32 == mission as u32
            && valid_versions[i as usize].version as u32 == version as u32
        {
            return true;
        }
        i += 1;
    }
    return false;
}
pub fn D_IsEpisodeMap(mut mission: GameMission_t) -> bool {
    match mission as u32 {
        0 | 6 | 4 => return true,
        9 | 7 | 1 | 5 | 2 | 3 | 8 | _ => return false,
    };
}
pub fn D_GameMissionString(mission: GameMission_t) -> &'static str {
    match mission as u32 {
        0 => {
            return "doom";
        }
        1 => {
            return "doom2";
        }
        2 => {
            return "tnt";
        }
        3 => {
            return "plutonia";
        }
        5 => {
            return "hacx";
        }
        4 => {
            return "chex";
        }
        6 => {
            return "heretic";
        }
        7 => {
            return "hexen";
        }
        8 => {
            return "strife";
        }
        9 | _ => {
            return "none";
        }
    };
}
