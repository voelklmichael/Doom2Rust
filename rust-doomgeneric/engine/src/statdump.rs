use crate::src::game_state::GameState;
use crate::src::m_argv::M_ParmExists;
use crate::src::stdint_types::size_t;
use crate::src::wi_stuff::{wbplayerstruct_t, wbstartstruct_t};
use crate::src::mem_compat::memcpy;
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

pub unsafe fn StatCopy(state: &mut GameState, mut stats: *mut wbstartstruct_t) {
    if M_ParmExists(state, "-statdump") && state.statdump.num_captured_stats < MAX_CAPTURES {
        memcpy(
            (&raw mut state.statdump.captured_stats as *mut wbstartstruct_t)
                .offset(state.statdump.num_captured_stats as isize) as *mut wbstartstruct_t
                as *mut ::core::ffi::c_void,
            stats as *const ::core::ffi::c_void,
            ::core::mem::size_of::<wbstartstruct_t>() as size_t,
        );
        state.statdump.num_captured_stats += 1;
    }
}
pub fn StatDump(_state: &mut GameState) {}
