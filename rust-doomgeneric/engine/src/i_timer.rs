use crate::doomdef::TICRATE;
use crate::game_state::GameState;
pub struct ITimerState {
    basetime: u32,
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
    let mut ticks: u32 = state.platform.get_ticks_ms();
    if state.i_timer.basetime == 0_u32 {
        state.i_timer.basetime = ticks;
    }
    ticks = ticks.wrapping_sub(state.i_timer.basetime);
    ticks.wrapping_mul(TICRATE as u32).wrapping_div(1000_u32) as i32
}
pub fn I_GetTimeMS(state: &mut GameState) -> i32 {
    let ticks: u32 = state.platform.get_ticks_ms();
    if state.i_timer.basetime == 0_u32 {
        state.i_timer.basetime = ticks;
    }
    ticks.wrapping_sub(state.i_timer.basetime) as i32
}
pub fn I_Sleep(state: &mut GameState, ms: i32) {
    state.platform.sleep_ms(ms as u32);
}
