use crate::game_state::GameState;
use crate::m_argv::M_ParmExists;
use crate::mem_compat::memcpy;
use crate::stdint_types::size_t;
use crate::wi_stuff::{wbplayerstruct_t, wbstartstruct_t};
pub const MAX_CAPTURES: i32 = 32;

pub struct StatDumpState {
    captured_stats: [wbstartstruct_t; 32],
    num_captured_stats: i32,
}

impl StatDumpState {
    pub const fn new() -> Self {
        StatDumpState {
            captured_stats: [wbstartstruct_t {
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
                plyr: [wbplayerstruct_t {
                    in_0: false,
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

pub fn StatCopy(state: &mut GameState) {
    if M_ParmExists(state, "-statdump") && state.statdump.num_captured_stats < MAX_CAPTURES {
        state.statdump.captured_stats[state.statdump.num_captured_stats as usize] =
            state.g_game.wminfo;
        state.statdump.num_captured_stats += 1;
    }
}
pub fn StatDump(_state: &mut GameState) {}
