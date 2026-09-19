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
        Self { myargv: Vec::new() }
    }
}

/// The index in `argv` of `check`, provided at least `num_args` more
/// arguments follow it.
pub fn check_parm_with_args(state: &GameState, check: &str, num_args: usize) -> Option<usize> {
    let argv = &state.m_argv.myargv;
    (1..argv.len().saturating_sub(num_args)).find(|&i| argv[i].eq_ignore_ascii_case(check))
}
pub fn parm_exists(state: &GameState, check: &str) -> bool {
    check_parm(state, check).is_some()
}
pub fn check_parm(state: &GameState, check: &str) -> Option<usize> {
    check_parm_with_args(state, check, 0)
}
pub fn argv_atoi(arg: &str) -> i32 {
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
