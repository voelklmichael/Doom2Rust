use crate::d_mode::GameMission;
use crate::d_mode::GameMode;
use crate::d_mode::GameVersion;
pub struct DoomstatState {
    pub gamemode: GameMode,
    pub gamemission: GameMission,
    pub gameversion: GameVersion,
    pub gamedescription: &'static str,
    pub modifiedgame: bool,
}
impl Default for DoomstatState {
    fn default() -> Self {
        Self::new()
    }
}

impl DoomstatState {
    pub const fn new() -> Self {
        DoomstatState {
            gamemode: GameMode::Indetermined,
            gamemission: GameMission::Doom,
            gameversion: GameVersion::Final2,
            gamedescription: "",
            modifiedgame: false,
        }
    }
}
