use crate::game_state::GameState;
use crate::m_argv::parm_exists;

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
                    frags: [0; 4],
                    score: 0,
                }; 4],
            }; 32],
            num_captured_stats: 0,
        }
    }
}

pub fn stat_copy(state: &mut GameState) {
    if parm_exists(state, "-statdump") && state.statdump.num_captured_stats < MAX_CAPTURES {
        state.statdump.captured_stats[state.statdump.num_captured_stats as usize] =
            state.g_game.wminfo;
        state.statdump.num_captured_stats += 1;
    }
}
pub fn stat_dump(_state: &mut GameState) {}
