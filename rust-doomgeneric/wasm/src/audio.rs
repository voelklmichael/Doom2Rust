//! Pacing of the sound the engine mixes.

/// After a stall the game asks for at most this much catch-up audio; the rest is skipped so the
/// delay to the speaker does not grow.
const MAX_CATCH_UP_MS: f64 = 80.0;

/// Decides how many sound frames (one left and one right sample) the game should mix now: as many
/// as real time has elapsed since the start, less what was mixed already.
pub struct AudioClock {
    rate: u32,
    /// Set by the first request, so level loading before the first tick is not mixed as a burst
    /// of silence.
    started_ms: Option<f64>,
    frames_mixed: u64,
}

impl AudioClock {
    pub fn new(rate: u32) -> Self {
        Self {
            rate,
            started_ms: None,
            frames_mixed: 0,
        }
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    // Both `as u64` casts are of values that are not negative (a clamped time, a product of
    // positive numbers).
    #[allow(clippy::cast_sign_loss)]
    pub fn frames_wanted(&mut self, now_ms: f64) -> usize {
        let started = *self.started_ms.get_or_insert(now_ms);
        let per_ms = f64::from(self.rate) / 1000.0;
        let due = ((now_ms - started).max(0.0) * per_ms) as u64;
        let cap = (MAX_CATCH_UP_MS * per_ms) as u64;
        let behind = due.saturating_sub(self.frames_mixed);
        if behind > cap {
            self.frames_mixed += behind - cap;
        }
        due.saturating_sub(self.frames_mixed) as usize
    }

    pub fn mixed(&mut self, frames: usize) {
        self.frames_mixed += frames as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asks_for_the_frames_elapsed_since_the_last_write() {
        let mut clock = AudioClock::new(44100);
        // Nothing is due before the clock starts, whatever the time is.
        assert_eq!(clock.frames_wanted(5000.0), 0);
        assert_eq!(clock.frames_wanted(5020.0), 882);
        clock.mixed(882);
        assert_eq!(clock.frames_wanted(5020.0), 0);
        assert_eq!(clock.frames_wanted(5040.0), 882);
    }

    #[test]
    fn a_stall_is_caught_up_by_at_most_the_cap() {
        let mut clock = AudioClock::new(44100);
        clock.frames_wanted(0.0);
        let wanted = clock.frames_wanted(2000.0);
        assert_eq!(wanted, 3528);
        clock.mixed(wanted);
        // The skipped audio is gone for good: the next period is normal again.
        assert_eq!(clock.frames_wanted(2020.0), 882);
    }
}
