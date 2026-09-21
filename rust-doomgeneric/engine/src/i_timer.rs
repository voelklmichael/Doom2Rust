use crate::doomdef::TICRATE;
use crate::platform::DoomPlatform;
#[derive(Default)]
pub struct ITimerState {
    basetime: u32,
}

pub fn get_time(i_timer: &mut ITimerState, platform: &mut dyn DoomPlatform) -> i32 {
    let mut ticks: u32 = platform.get_ticks_ms();
    if i_timer.basetime == 0_u32 {
        i_timer.basetime = ticks;
    }
    ticks = ticks.wrapping_sub(i_timer.basetime);
    ticks.wrapping_mul(TICRATE as u32).wrapping_div(1000_u32) as i32
}
pub fn get_time_ms(i_timer: &mut ITimerState, platform: &mut dyn DoomPlatform) -> i32 {
    let ticks: u32 = platform.get_ticks_ms();
    if i_timer.basetime == 0_u32 {
        i_timer.basetime = ticks;
    }
    ticks.wrapping_sub(i_timer.basetime) as i32
}
pub fn sleep(platform: &mut dyn DoomPlatform, ms: i32) {
    platform.sleep_ms(ms.cast_unsigned());
}
