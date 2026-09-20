//! Input-latency probes (`--features lagprobe`; compiled out otherwise, `Stamp` is then an empty struct).
//!
//! There is no way to measure a real browser round trip from the development machine (it has one
//! Wi-Fi adapter and must not join the board's network), so this measures, on the device and
//! printed on the serial port every 5 s:
//!
//! - **sched**: how late a 10 ms sleep on core 0's executor wakes up. The network path (embassy-net's
//!   runner, the web and command server tasks) is woken the same way, so this is the delay any of
//!   them can suffer from whatever else runs on the executor (music slices, LCD chunks, the pump).
//! - **key**: how long a key event waits from the moment a network task decoded it to the moment the
//!   game (core 1) takes it out of the queue.
//! - **slice**: the longest single music slice.
//!
//! With `--features lagprobe-keys` a task also injects synthetic key presses (`Weapon1`, which
//! neither the menu nor the demo does anything with; never anything that could reach "Quit game")
//! at a fixed rate, so the key numbers exist without a browser.

#[cfg(feature = "lagprobe")]
mod imp {
    use core::sync::atomic::{AtomicU32, Ordering};

    use embassy_time::{Duration, Instant, Timer};
    use esp_println::println;

    /// A time stamp on a key event: microseconds, wrapping (differences only).
    pub type Stamp = u32;

    pub fn stamp() -> Stamp {
        esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_micros() as u32
    }

    const BUCKETS: usize = 64;

    /// A histogram of 64 buckets of `BUCKET_US` (the last one collects everything above), and the
    /// exact maximum.
    pub struct Hist<const BUCKET_US: u32> {
        buckets: [AtomicU32; BUCKETS],
        max: AtomicU32,
    }

    impl<const BUCKET_US: u32> Hist<BUCKET_US> {
        pub const fn new() -> Self {
            Self {
                buckets: [const { AtomicU32::new(0) }; BUCKETS],
                max: AtomicU32::new(0),
            }
        }

        pub fn record(&self, us: u32) {
            let bucket = ((us / BUCKET_US) as usize).min(BUCKETS - 1);
            self.buckets[bucket].fetch_add(1, Ordering::Relaxed);
            self.max.fetch_max(us, Ordering::Relaxed);
        }

        /// (count, median, 99th percentile, max), the two percentiles as the upper edge of their
        /// bucket, and the histogram is cleared.
        fn take(&self) -> (u32, u32, u32, u32) {
            let counts: [u32; BUCKETS] =
                core::array::from_fn(|i| self.buckets[i].swap(0, Ordering::Relaxed));
            let max = self.max.swap(0, Ordering::Relaxed);
            let total: u32 = counts.iter().sum();
            let percentile = |per_cent: u32| {
                let target = (total * per_cent).div_ceil(100);
                let mut seen = 0;
                for (i, count) in counts.iter().enumerate() {
                    seen += count;
                    if seen >= target && total > 0 {
                        return (i as u32 + 1) * BUCKET_US;
                    }
                }
                0
            };
            (total, percentile(50), percentile(99), max)
        }
    }

    pub static SCHED: Hist<250> = Hist::new();
    pub static KEY: Hist<2000> = Hist::new();
    static SLICE_MAX_US: AtomicU32 = AtomicU32::new(0);
    /// Times the game asked for events and got none (one per poll of the input), and events taken.
    static POLLS: AtomicU32 = AtomicU32::new(0);
    static TAKEN: AtomicU32 = AtomicU32::new(0);
    #[cfg(feature = "lagprobe-keys")]
    static PHASE: AtomicU32 = AtomicU32::new(0);

    /// The game took an event that was stamped `stamp` when a network task decoded it.
    pub fn key_taken(stamp: Stamp) {
        TAKEN.fetch_add(1, Ordering::Relaxed);
        KEY.record(self::stamp().wrapping_sub(stamp));
    }

    /// The game found the queue empty.
    pub fn polled_empty() {
        POLLS.fetch_add(1, Ordering::Relaxed);
    }

    pub fn slice(us: u32) {
        SLICE_MAX_US.fetch_max(us, Ordering::Relaxed);
    }

    /// Sleeps 10 ms over and over and records how late it wakes up.
    #[embassy_executor::task]
    pub async fn task() {
        const SLEEP: Duration = Duration::from_millis(10);
        let mut report_at = Instant::now() + Duration::from_secs(5);
        loop {
            let before = Instant::now();
            Timer::after(SLEEP).await;
            let late = before
                .elapsed()
                .as_micros()
                .saturating_sub(SLEEP.as_micros());
            SCHED.record(late as u32);
            if Instant::now() >= report_at {
                report_at += Duration::from_secs(5);
                let (n, p50, p99, max) = SCHED.take();
                println!(
                    "[lag] sched: {n} wakeups, late p50 <{p50} us, p99 <{p99} us, max {max} us"
                );
                let (n, p50, p99, max) = KEY.take();
                println!(
                    "[lag] key: {n} events, queue wait p50 <{p50} us, p99 <{p99} us, max {max} us"
                );
                println!(
                    "[lag] longest music slice {} us; input polls that found nothing {}, events taken {}",
                    SLICE_MAX_US.swap(0, Ordering::Relaxed),
                    POLLS.swap(0, Ordering::Relaxed),
                    TAKEN.swap(0, Ordering::Relaxed),
                );
                #[cfg(feature = "lagprobe-keys")]
                println!("[lag] injector phase {}", PHASE.load(Ordering::Relaxed));
            }
        }
    }

    /// Injects synthetic input in 10 s phases: 0 = `Weapon1` tapped every 100 ms (a person clicking
    /// a button), 1 = the typed character `z` as press + release pairs every 30 ms (the web page's
    /// Text mode at full speed), 2 = nothing, 3 = like 0 again (does the input recover?).
    #[cfg(feature = "lagprobe-keys")]
    #[embassy_executor::task]
    pub async fn keys() {
        use core_s3_protocol::{Command, HeldKeys};

        let mut held = HeldKeys::default();
        let tap = Command::Weapon1.code();
        let typed = b'z';
        let mut phase_start = Instant::now();
        let mut phase = 0u32;
        loop {
            if phase_start.elapsed() >= Duration::from_secs(10) {
                phase_start = Instant::now();
                phase = (phase + 1) % 4;
                PHASE.store(phase, Ordering::Relaxed);
            }
            match phase {
                1 => {
                    crate::net::handle_command_byte(typed, &mut held);
                    crate::net::handle_command_byte(typed | 0x80, &mut held);
                    Timer::after(Duration::from_millis(30)).await;
                }
                2 => Timer::after(Duration::from_millis(100)).await,
                _ => {
                    Timer::after(Duration::from_millis(70)).await;
                    crate::net::handle_command_byte(tap, &mut held);
                    Timer::after(Duration::from_millis(30)).await;
                    crate::net::handle_command_byte(tap | 0x80, &mut held);
                }
            }
        }
    }
}

#[cfg(not(feature = "lagprobe"))]
mod imp {
    /// Nothing to record; a struct rather than `()` so passing it around does not trip clippy's unit lints.
    #[derive(Clone, Copy)]
    pub struct Stamp;
    pub fn stamp() -> Stamp {
        Stamp
    }
    pub fn key_taken(_stamp: Stamp) {}
    pub fn slice(_us: u32) {}
    pub fn polled_empty() {}
}

pub use imp::*;
