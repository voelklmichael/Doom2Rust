//! What the game is started with.
//!
//! The engine does not read a command line: the frontend (the `x11` program, the ESP firmware, a
//! test) fills in an [`Options`] and passes it to `doomgeneric_create`. `Options::default()` is
//! "no options"; each field says which vanilla switch it stands for, so a frontend that does have
//! a command line (`doomgeneric_cmdline` parses one) maps the switches onto it.
use alloc::string::String;
use alloc::vec::Vec;

/// `-warp`, read both ways the game mode may need. In Doom II the first argument is a map
/// number; in the other games it is an episode digit and the second argument a map digit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Warp {
    /// The first argument as a number (the map in Doom II).
    pub map_number: i32,
    /// The digit the first argument starts with (the episode elsewhere).
    pub episode: i32,
    /// The digit the second argument starts with, 1 if there is none (the map elsewhere).
    pub episode_map: i32,
}

/// Everything the engine can be told at start-up.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Options {
    // ---- switches ----
    /// `-altdeath`: deathmatch 2.0.
    pub altdeath: bool,
    /// `-avg`: Austin Virtual Gaming, a 20 minute limit.
    pub avg: bool,
    /// `-deathmatch`
    pub deathmatch: bool,
    /// `-devparm`: developer mode.
    pub devparm: bool,
    /// `-fast`: fast monsters.
    pub fast: bool,
    /// `-left`: a drone looking left.
    pub left: bool,
    /// `-longtics`: 16-bit angle turns in demos.
    pub longtics: bool,
    /// `-netdemo`: treat a demo as a net game.
    pub netdemo: bool,
    /// `-nodraw`: time demos without drawing.
    pub nodraw: bool,
    /// `-nomonsters`
    pub nomonsters: bool,
    /// `-nomusic`
    pub nomusic: bool,
    /// `-nosfx`
    pub nosfx: bool,
    /// `-nosound`
    pub nosound: bool,
    /// `-record` was given (with or without a name; see also `record_file`).
    pub record: bool,
    /// `-reject_pad_with_ff`: pad a short REJECT lump with 0xff (vanilla-compatibility switch).
    pub reject_pad_with_ff: bool,
    /// `-respawn`: respawning monsters.
    pub respawn: bool,
    /// `-right`: a drone looking right.
    pub right: bool,
    /// `-solo-net`: a net game with one player.
    pub solo_net: bool,
    /// `-statdump` was given (with or without a name; see also `statdump_file`).
    pub statdump: bool,
    /// `-testcontrols`: the input test mode.
    pub testcontrols: bool,

    // ---- options with a value ----
    /// `-config file`: the main configuration file.
    pub config: Option<String>,
    /// `-donut floorheight floorpic`: the two values (vanilla-compatibility), as typed: the
    /// overflow emulation reads them with the engine's own number reader.
    pub donut: Option<(String, String)>,
    /// `-episode n`: the episode number.
    pub episode: Option<i32>,
    /// `-extraconfig file`: a second configuration file.
    pub extraconfig: Option<String>,
    /// `-file a b c`: the files to add; `Some(empty)` if none were given. Also makes the game
    /// count as modified.
    pub file: Option<Vec<String>>,
    /// `-gameversion name`
    pub gameversion: Option<String>,
    /// `-gfxmode mode`
    pub gfxmode: Option<String>,
    /// `-iwad file`
    pub iwad: Option<String>,
    /// `-loadgame slot`
    pub loadgame: Option<i32>,
    /// `-maxdemo kilobytes`: the demo buffer size while recording.
    pub maxdemo: Option<i32>,
    /// `-pack name`: which Doom 2 mission pack the IWAD is.
    pub pack: Option<String>,
    /// `-playdemo name`
    pub playdemo: Option<String>,
    /// `-record name`: the demo to record (see also `record`).
    pub record_file: Option<String>,
    /// `-scaling n`
    pub scaling: Option<i32>,
    /// `-setmem ...`: a keyword (`dos71`, `dosbox`), or a run of numbers; as typed, since the
    /// memory-dump emulation reads them with the engine's own number reader.
    pub setmem: Option<Vec<String>>,
    /// `-skill n`: the level as typed, 1 (baby) to 5 (nightmare).
    pub skill: Option<i32>,
    /// `-spechit address`: the base address for the spechit overflow emulation, as typed.
    pub spechit: Option<String>,
    /// `-statdump file`: the file to write (see also `statdump`).
    pub statdump_file: Option<String>,
    /// `-timedemo name`
    pub timedemo: Option<String>,
    /// `-timer minutes`
    pub timer: Option<i32>,
    /// `-turbo [percent]`: `Some(None)` if no value was given.
    pub turbo: Option<Option<i32>>,
    /// `-warp`
    pub warp: Option<Warp>,
}
