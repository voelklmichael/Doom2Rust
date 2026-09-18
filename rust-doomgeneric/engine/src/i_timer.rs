use crate::doomdef::TICRATE;
use crate::game_state::GameState;
use crate::stdint_types::uint32_t;
pub struct ITimerState {
    basetime: uint32_t,
}

impl Default for ITimerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ITimerState {
    pub const fn new() -> Self {
        ITimerState { basetime: 0 }
    }
}

pub fn I_GetTime(state: &mut GameState) -> i32 {
    let mut ticks: uint32_t = state.platform.get_ticks_ms();
    if state.i_timer.basetime == 0 as uint32_t {
        state.i_timer.basetime = ticks;
    }
    ticks = ticks.wrapping_sub(state.i_timer.basetime);
    ticks
        .wrapping_mul(TICRATE as uint32_t)
        .wrapping_div(1000 as uint32_t) as i32
}
pub fn I_GetTimeMS(state: &mut GameState) -> i32 {
    let mut ticks: uint32_t = state.platform.get_ticks_ms();
    if state.i_timer.basetime == 0 as uint32_t {
        state.i_timer.basetime = ticks;
    }
    ticks.wrapping_sub(state.i_timer.basetime) as i32
}
pub fn I_Sleep(state: &mut GameState, ms: i32) {
    state.platform.sleep_ms(ms as uint32_t);
}
