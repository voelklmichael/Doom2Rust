use crate::game_state::GameState;
use alloc::string::String;
use alloc::vec::Vec;

pub struct MArgvState {
    pub myargv: Vec<String>,
}

impl Default for MArgvState {
    fn default() -> Self {
        Self::new()
    }
}

impl MArgvState {
    pub const fn new() -> Self {
        MArgvState { myargv: Vec::new() }
    }
}

pub fn M_CheckParmWithArgs(state: &mut GameState, check: &str, num_args: i32) -> i32 {
    for i in 1..state.m_argv.myargv.len() as i32 - num_args {
        if state.m_argv.myargv[i as usize].eq_ignore_ascii_case(check) {
            return i;
        }
    }
    0
}
pub fn M_ParmExists(state: &mut GameState, check: &str) -> bool {
    M_CheckParm(state, check) != 0
}
pub fn M_CheckParm(state: &mut GameState, check: &str) -> i32 {
    M_CheckParmWithArgs(state, check, 0)
}
pub fn M_ArgvAtoi(arg: &str) -> i32 {
    let bytes = arg.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let negative = i < bytes.len() && bytes[i] == b'-';
    if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
        i += 1;
    }
    let mut value: i32 = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        value = value
            .wrapping_mul(10)
            .wrapping_add((bytes[i] - b'0') as i32);
        i += 1;
    }
    if negative {
        -value
    } else {
        value
    }
}
