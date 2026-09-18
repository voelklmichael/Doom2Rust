use crate::game_state::GameState;

pub struct MArgvState {
    pub myargv: Vec<::std::ffi::CString>,
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
    let mut i: i32 = 1_i32;
    while i < state.m_argv.myargv.len() as i32 - num_args {
        if state.m_argv.myargv[i as usize]
            .to_str()
            .is_ok_and(|arg| arg.eq_ignore_ascii_case(check))
        {
            return i;
        }
        i += 1;
    }
    0_i32
}
pub fn M_ParmExists(state: &mut GameState, check: &str) -> bool {
    M_CheckParm(state, check) != 0_i32
}
pub fn M_CheckParm(state: &mut GameState, check: &str) -> i32 {
    M_CheckParmWithArgs(state, check, 0_i32)
}
pub fn M_FindResponseFile(state: &mut GameState) {
    let mut i: i32 = 1_i32;
    while i < state.m_argv.myargv.len() as i32 {
        i += 1;
    }
}
pub fn M_ArgvAtoi(arg: &::std::ffi::CStr) -> i32 {
    let bytes = arg.to_bytes();
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
