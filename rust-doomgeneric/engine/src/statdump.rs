use crate::doomdef::MAXPLAYERS;
use crate::g_game::GGameState;
use crate::game_state::GameState;
use crate::options::Options;

use crate::wi_stuff::{WbPlayerStruct, WbStartStruct};
pub const MAX_CAPTURES: i32 = 32;

pub struct StatDumpState {
    captured_stats: [WbStartStruct; 32],
    num_captured_stats: i32,
}

impl Default for StatDumpState {
    fn default() -> Self {
        Self::new()
    }
}

impl StatDumpState {
    pub const fn new() -> Self {
        Self {
            captured_stats: [WbStartStruct {
                epsd: 0,
                didsecret: false,
                last: 0,
                next: 0,
                maxkills: 0,
                maxitems: 0,
                maxsecret: 0,
                maxfrags: 0,
                partime: 0,
                pnum: 0,
                plyr: [WbPlayerStruct {
                    intercept: false,
                    skills: 0,
                    sitems: 0,
                    ssecret: 0,
                    stime: 0,
                    frags: [0; MAXPLAYERS],
                    score: 0,
                }; MAXPLAYERS],
            }; 32],
            num_captured_stats: 0,
        }
    }
}

pub fn stat_copy(g_game: &GGameState, options: &Options, statdump: &mut StatDumpState) {
    if options.statdump && statdump.num_captured_stats < MAX_CAPTURES {
        statdump.captured_stats[statdump.num_captured_stats as usize] = g_game.wminfo;
        statdump.num_captured_stats += 1;
    }
}
pub fn stat_dump(_state: &mut GameState) {}
