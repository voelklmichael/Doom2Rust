//! The hardware-free half of the CoreS3's sound output, so it can be unit tested on the host.
//!
//! The game (core 1) mixes sound into a [`FrameQueue`]; a task on core 0 moves it from there into
//! the I2S DMA ring, one [`Pacing::plan`] at a time. The queue keeps the game from ever waiting on
//! the speaker, and lets the task on core 0 fill the ring with silence when the game is late (a
//! level load, a slow frame) instead of letting the DMA replay old audio.
//!
//! A *frame* is one left and one right sample.

#![no_std]

use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// A lock-free queue of stereo frames from one producer (the game) to one consumer (the pump).
///
/// `N` must be a power of two. The positions only ever grow (and wrap), so `head - tail` is the fill level and a slot is
/// never both written and read: the producer owns `tail + N ..= head`, the consumer `tail ..= head`.
/// Each frame is one `AtomicU32` (left in the low half, right in the high half), so no `unsafe`.
pub struct FrameQueue<const N: usize> {
    frames: [AtomicU32; N],
    /// Frames ever written; only the producer stores to it.
    head: AtomicUsize,
    /// Frames ever read; only the consumer stores to it.
    tail: AtomicUsize,
}

impl<const N: usize> FrameQueue<N> {
    pub const fn new() -> Self {
        // The positions wrap around `usize`, which only keeps slots consistent if `N` divides it.
        assert!(N.is_power_of_two(), "the queue size must be a power of two");
        Self {
            frames: [const { AtomicU32::new(0) }; N],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    /// Frames waiting to be popped.
    pub fn len(&self) -> usize {
        self.head.load(Ordering::Acquire).wrapping_sub(self.tail.load(Ordering::Acquire))
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Room for this many more frames.
    pub fn free(&self) -> usize {
        N - self.len()
    }

    /// Producer side. Queues as many frames of interleaved `samples` as fit and returns how many
    /// it took (a trailing odd sample is ignored).
    pub fn push(&self, samples: &[i16]) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let free = N - head.wrapping_sub(self.tail.load(Ordering::Acquire));
        let mut pushed = 0;
        for (frame, pair) in samples.chunks_exact(2).take(free).enumerate() {
            let packed = u32::from(pair[0] as u16) | u32::from(pair[1] as u16) << 16;
            self.frames[head.wrapping_add(frame) % N].store(packed, Ordering::Relaxed);
            pushed += 1;
        }
        self.head.store(head.wrapping_add(pushed), Ordering::Release);
        pushed
    }

    /// Consumer side. Fills `out` (interleaved) with up to `out.len() / 2` frames and returns how
    /// many it wrote.
    pub fn pop(&self, out: &mut [i16]) -> usize {
        let tail = self.tail.load(Ordering::Relaxed);
        let queued = self.head.load(Ordering::Acquire).wrapping_sub(tail);
        let mut popped = 0;
        for (frame, pair) in out.chunks_exact_mut(2).take(queued).enumerate() {
            let packed = self.frames[tail.wrapping_add(frame) % N].load(Ordering::Relaxed);
            pair[0] = packed as u16 as i16;
            pair[1] = (packed >> 16) as u16 as i16;
            popped += 1;
        }
        self.tail.store(tail.wrapping_add(popped), Ordering::Release);
        popped
    }
}

impl<const N: usize> Default for FrameQueue<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// How much audio to keep where, in frames.
#[derive(Clone, Copy, Debug)]
pub struct Pacing {
    /// The DMA ring is topped up from the queue to this level.
    pub ring_target: usize,
    /// Below this level the ring is topped up with silence, so it never runs dry (a dry ring
    /// replays old audio). It has to cover how late the pump can be.
    pub ring_min: usize,
    /// What the game keeps queued: it is asked for `queue_target - queued` frames per tick.
    pub queue_target: usize,
}

/// What one pass of the pump does.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Plan {
    /// Frames to move from the queue into the ring.
    pub from_queue: usize,
    /// Frames of silence to add after them.
    pub silence: usize,
}

impl Pacing {
    /// Frames the game should mix now, given how many are still queued.
    pub fn frames_wanted(&self, queued: usize) -> usize {
        self.queue_target.saturating_sub(queued)
    }

    /// The pump's move. `level` is how many frames the ring holds (an upper estimate is fine),
    /// `free` how many it can take, `queued` how many wait in the queue.
    pub fn plan(&self, level: usize, free: usize, queued: usize) -> Plan {
        let room = free.min(self.ring_target.saturating_sub(level));
        let from_queue = room.min(queued);
        let silence = self.ring_min.saturating_sub(level + from_queue).min(free - from_queue);
        Plan { from_queue, silence }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[test]
    fn queue_is_first_in_first_out_across_the_wrap() {
        let q = FrameQueue::<4>::new();
        let mut out = [0i16; 8];
        // Push and pop in odd sizes so the positions wrap the four slots several times.
        let mut next = 0i16;
        let mut expected = 0i16;
        for round in 0..20 {
            let n = 1 + round % 3;
            let input: Vec<i16> = (0..n * 2).map(|_| { next += 1; next }).collect();
            assert_eq!(q.push(&input), n);
            let popped = q.pop(&mut out[..n * 2]);
            assert_eq!(popped, n);
            for sample in &out[..n * 2] {
                expected += 1;
                assert_eq!(*sample, expected);
            }
        }
        assert!(q.is_empty());
    }

    #[test]
    fn queue_keeps_the_sign_and_the_channels_apart() {
        let q = FrameQueue::<2>::new();
        assert_eq!(q.push(&[-32768, 32767, -1, 1]), 2);
        let mut out = [0i16; 4];
        assert_eq!(q.pop(&mut out), 2);
        assert_eq!(out, [-32768, 32767, -1, 1]);
    }

    #[test]
    fn a_full_queue_takes_what_fits_and_reports_it() {
        let q = FrameQueue::<4>::new();
        assert_eq!(q.push(&[1; 10]), 4);
        assert_eq!((q.len(), q.free()), (4, 0));
        assert_eq!(q.push(&[1; 2]), 0);
        let mut out = [0i16; 2];
        assert_eq!(q.pop(&mut out), 1);
        assert_eq!((q.len(), q.free()), (3, 1));
        // A trailing odd sample is not a frame.
        assert_eq!(q.push(&[7, 7, 7]), 1);
    }

    #[test]
    fn an_empty_queue_pops_nothing() {
        let q = FrameQueue::<4>::new();
        let mut out = [9i16; 4];
        assert_eq!(q.pop(&mut out), 0);
        assert_eq!(out, [9; 4]);
    }

    #[test]
    fn the_positions_may_wrap_around_usize() {
        let q = FrameQueue::<4>::new();
        q.head.store(usize::MAX - 1, Ordering::Relaxed);
        q.tail.store(usize::MAX - 1, Ordering::Relaxed);
        assert_eq!(q.push(&[1, 2, 3, 4, 5, 6]), 3);
        assert_eq!(q.len(), 3);
        let mut out = [0i16; 6];
        assert_eq!(q.pop(&mut out), 3);
        assert_eq!(out, [1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn works_across_threads() {
        static Q: FrameQueue<64> = FrameQueue::new();
        const TOTAL: i16 = 5000;
        let producer = std::thread::spawn(|| {
            let mut sent = 0;
            while sent < TOTAL {
                if Q.push(&[sent, -sent]) == 1 {
                    sent += 1;
                }
            }
        });
        let mut got = 0;
        while got < TOTAL {
            let mut out = [0i16; 2];
            if Q.pop(&mut out) == 1 {
                assert_eq!(out, [got, -got]);
                got += 1;
            }
        }
        producer.join().unwrap();
    }

    const PACING: Pacing = Pacing { ring_target: 600, ring_min: 300, queue_target: 800 };

    #[test]
    fn the_game_is_asked_for_what_the_queue_lacks() {
        assert_eq!(PACING.frames_wanted(0), 800);
        assert_eq!(PACING.frames_wanted(500), 300);
        assert_eq!(PACING.frames_wanted(800), 0);
        assert_eq!(PACING.frames_wanted(2000), 0);
    }

    #[test]
    fn the_pump_tops_the_ring_up_from_the_queue() {
        // Ring at 400 of 600: 200 frames of room, plenty queued.
        assert_eq!(PACING.plan(400, 1000, 800), Plan { from_queue: 200, silence: 0 });
        // Little queued: take what there is.
        assert_eq!(PACING.plan(400, 1000, 50), Plan { from_queue: 50, silence: 0 });
        // The ring's free space limits it.
        assert_eq!(PACING.plan(400, 128, 800), Plan { from_queue: 128, silence: 0 });
        // A full ring takes nothing.
        assert_eq!(PACING.plan(600, 1000, 800), Plan { from_queue: 0, silence: 0 });
    }

    #[test]
    fn a_starved_ring_gets_silence_up_to_the_minimum() {
        // Nothing queued and the level fell below the minimum.
        assert_eq!(PACING.plan(100, 1000, 0), Plan { from_queue: 0, silence: 200 });
        // Exactly at the minimum: leave it (real audio may arrive any moment).
        assert_eq!(PACING.plan(300, 1000, 0), Plan { from_queue: 0, silence: 0 });
        // A little queued audio counts towards the minimum before silence is added.
        assert_eq!(PACING.plan(100, 1000, 50), Plan { from_queue: 50, silence: 150 });
        // Silence never exceeds the room that is left.
        assert_eq!(PACING.plan(0, 128, 0), Plan { from_queue: 0, silence: 128 });
        assert_eq!(PACING.plan(0, 128, 100), Plan { from_queue: 100, silence: 28 });
    }
}
