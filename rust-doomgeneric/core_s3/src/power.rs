//! Switching the board off, for when the game quits. The AXP2101 power chip has a soft power-off
//! bit (`core_s3::power::Axp2101::shutdown`), so this keeps the internal I2C bus after the panel
//! power-up for that one write. Nothing else uses the bus once the game is running.

use core::cell::RefCell;

use core_s3::power::Axp2101;
use critical_section::Mutex;
use esp_hal::{delay::Delay, i2c::master::I2c, Blocking};
use esp_println::println;

type Pmu = Axp2101<I2c<'static, Blocking>>;

static PMU: Mutex<RefCell<Option<Pmu>>> = Mutex::new(RefCell::new(None));

/// Keeps the internal I2C bus for [`power_off`].
pub fn init(i2c: I2c<'static, Blocking>) {
    critical_section::with(|cs| *PMU.borrow_ref_mut(cs) = Some(Axp2101::new(i2c)));
}

/// Asks the power chip to switch the board off. If that works this never returns; if it comes
/// back the board is still powered (for instance from USB) and the caller has nothing left to do.
pub fn power_off() {
    let Some(mut pmu) = critical_section::with(|cs| PMU.borrow_ref_mut(cs).take()) else {
        println!("power off: no power chip handle");
        return;
    };
    println!("powering off");
    // Let the message leave over USB before the power goes.
    Delay::new().delay_millis(50);
    match pmu.shutdown() {
        Ok(()) => println!("power off: the power chip did not switch the board off"),
        Err(error) => println!("power off: I2C write failed: {error:?}"),
    }
}
