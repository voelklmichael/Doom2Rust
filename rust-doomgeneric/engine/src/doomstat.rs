use crate::src::d_mode::GameVersion;
use crate::src::d_mode::GameMission_t;
use crate::src::d_mode::GameMode_t;
pub struct DoomstatState {
    pub gamemode: GameMode_t,
    pub gamemission: GameMission_t,
    pub gameversion: GameVersion,
    pub gamedescription: &'static str,
    pub modifiedgame: bool,
}
impl DoomstatState {
    pub const fn new() -> Self {
        DoomstatState {
            gamemode: GameMode_t::indetermined,
            gamemission: GameMission_t::doom,
            gameversion: GameVersion::final2,
            gamedescription: "",
            modifiedgame: false,
        }
    }
}
