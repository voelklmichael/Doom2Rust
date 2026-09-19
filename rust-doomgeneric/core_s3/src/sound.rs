//! Between the game and the speaker.
//!
//! The game (core 1) calls [`frames_wanted`] and [`write`] from the platform's audio hooks; both
//! only touch a lock-free queue and never wait. [`pump`], a task on core 0, moves the queued audio
//! into the I2S DMA ring one chunk at a time whenever the DMA has played one, and pads with silence
//! when the queue runs short (the game is late: a level load, a slow frame), so a stall is a gap in
//! the sound, not a replay of old audio. The queue is `core_s3_audio` (unit tested on the host); the
//! ring is `audio`.
//!
//! Latency: a chunk moved into the ring plays after the two chunks ahead of it (23-35 ms), and the
//! game keeps about four chunks (46 ms) queued, since it only refills once per tick.

use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};

use core_s3_audio::FrameQueue;
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use esp_hal::{
    delay::Delay,
    i2c::master::I2c,
    peripherals::{DMA_CH1, GPIO13, GPIO33, GPIO34, I2S1},
    Blocking,
};
use esp_println::println;

use crate::audio::{self, Speaker, CHUNK_FRAMES};

/// Frames between the game and the pump (must be a power of two): 93 ms at 11025 Hz, 4 KB.
const QUEUE_FRAMES: usize = 1024;

/// What the game keeps queued: four chunks (46 ms). The game refills once per tick (28-40 ms), so
/// the queue has to hold a tick's worth plus a chunk, or the pump runs short between ticks.
const QUEUE_TARGET: usize = 4 * CHUNK_FRAMES;

/// How often the pump looks at the ring. The DMA frees a chunk every 11.6 ms and the pump has to
/// be there within two chunks of that.
const PUMP_PERIOD: Duration = Duration::from_millis(2);

/// Master volume in 256ths of what the engine mixes, after the two channels are averaged (256 would
/// be the engine's own level). The CoreS3's speaker is loud: 128 (-6 dB) was still "very
/// distracting", so this is -12 dB below that. Change it and rebuild for another level; the
/// in-game menu's sound volume slider works on top of it. The amp's own volume register is left at
/// full on purpose. (A multiply and a shift: this runs for every frame on the game core.)
const VOLUME_256THS: i32 = 32;

/// The engine's mixed audio for the speaker.
static QUEUE: FrameQueue<QUEUE_FRAMES> = FrameQueue::new();
/// Muted at run time (the controller's sound button, or the `M` key of `core_s3_sender`). The
/// speaker keeps running and the game keeps mixing, so nothing changes for the game; the audio is
/// just thrown away and the ring gets silence.
static MUTED: AtomicBool = AtomicBool::new(false);
/// Set once the speaker works; the platform reports "no sound" to the engine until then.
static READY: AtomicBool = AtomicBool::new(false);

// Counters for the serial log (see `pump`).
static PEAK: AtomicI32 = AtomicI32::new(0);
static GAME_FRAMES: AtomicU32 = AtomicU32::new(0);

/// Brings the speaker up (I2S clock first, then the amp over `i2c`) and starts [`pump`]. If
/// anything fails it says so on the serial port and the game runs without sound.
///
/// Kept out of line: inlined into `main`, whose state machine is already huge, it makes LLVM's
/// Xtensa backend fail with "Cannot scavenge register without an emergency spill slot".
#[inline(never)]
pub fn start(
    spawner: Spawner,
    i2c: &mut I2c<'static, Blocking>,
    i2s: I2S1<'static>,
    dma: DMA_CH1<'static>,
    bclk: GPIO34<'static>,
    ws: GPIO33<'static>,
    dout: GPIO13<'static>,
) {
    // Build with SOUND=off for a firmware without any sound (the speaker is not even set up), or
    // SOUND=muted to start muted.
    match option_env!("SOUND") {
        Some("off") => return println!("audio: built with SOUND=off; no sound"),
        Some("muted") => MUTED.store(true, Ordering::Relaxed),
        _ => {}
    }
    let speaker = match Speaker::open(i2s, dma, bclk, ws, dout) {
        Ok(speaker) => speaker,
        Err(error) => return println!("audio: {error}; no sound"),
    };
    Delay::new().delay_millis(20); // the amp wants to see the clock first
    match audio::init_amp(i2c) {
        Ok(()) => spawner.spawn(pump(speaker).expect("spawn sound pump")),
        Err(error) => println!("audio: amp init failed ({error:?}); no sound"),
    }
}

/// Mutes or unmutes the speaker.
pub fn toggle_mute() {
    let muted = !MUTED.fetch_xor(true, Ordering::Relaxed);
    println!("[audio] {}", if muted { "muted" } else { "unmuted" });
}

/// True once [`pump`] is running.
pub fn ready() -> bool {
    READY.load(Ordering::Acquire)
}

/// How many frames the game should mix now.
pub fn frames_wanted() -> usize {
    core_s3_audio::frames_wanted(QUEUE.len(), QUEUE_TARGET).min(QUEUE.free())
}

/// Queues interleaved stereo `samples` from the engine. The CoreS3 has a single speaker, so this
/// mixes both channels into one (the engine's panning becomes a level difference) and applies the
/// master volume.
pub fn write(samples: &[i16]) {
    if MUTED.load(Ordering::Relaxed) {
        return;
    }
    let mut mono = [0i16; 2 * 64];
    let mut peak = 0;
    for part in samples.chunks(2 * 64) {
        let out = &mut mono[..part.len() / 2 * 2];
        for (out, frame) in out.chunks_exact_mut(2).zip(part.chunks_exact(2)) {
            let sum = i32::from(frame[0]) + i32::from(frame[1]);
            // Average of the two channels, times the volume.
            let scaled = ((sum * VOLUME_256THS) >> 9) as i16;
            out.fill(scaled);
            peak = peak.max(i32::from(scaled).abs());
        }
        QUEUE.push(out);
    }
    PEAK.fetch_max(peak, Ordering::Relaxed);
    GAME_FRAMES.fetch_add((samples.len() / 2) as u32, Ordering::Relaxed);
}

/// Moves one chunk from the queue into the ring for every chunk the DMA has played. Returns how
/// many frames were real audio and how many were silence.
#[inline(never)]
fn refill(speaker: &mut Speaker) -> (u32, u32) {
    let (mut real, mut silence) = (0, 0);
    for _ in 0..speaker.free_frames() / CHUNK_FRAMES {
        let mut chunk = [0i16; 2 * CHUNK_FRAMES];
        let mut popped = QUEUE.pop_padded(&mut chunk);
        if MUTED.load(Ordering::Relaxed) {
            // Drain what was queued before the mute too.
            chunk.fill(0);
            popped = 0;
        }
        if speaker.push_chunk(&chunk) {
            real += popped as u32;
            silence += (CHUNK_FRAMES - popped) as u32;
        }
    }
    (real, silence)
}

/// Feeds the speaker, forever. Runs on core 0. `speaker` must have been opened on this core.
#[embassy_executor::task]
pub async fn pump(mut speaker: Speaker) {
    // Setting the amp up took longer than the ring lasts, so the DMA has run dry by now.
    speaker.rearm();
    READY.store(true, Ordering::Release);
    let (mut moved, mut silence) = (0u32, 0u32);
    let mut report_at = Instant::now() + Duration::from_secs(5);
    let mut last_run = Instant::now();
    let mut longest_gap = Duration::from_ticks(0);
    loop {
        let now = Instant::now();
        longest_gap = longest_gap.max(now - last_run);
        last_run = now;

        let (real, quiet) = refill(&mut speaker);
        moved += real;
        silence += quiet;

        if now >= report_at {
            report_at += Duration::from_secs(5);
            println!(
                "[audio] 5 s: game mixed {} frames (peak {}), {moved} frames played + {silence} silence, \
                 longest pump gap {} ms, {} DMA restarts",
                GAME_FRAMES.swap(0, Ordering::Relaxed),
                PEAK.swap(0, Ordering::Relaxed),
                longest_gap.as_millis(),
                speaker.restarts,
            );
            (moved, silence) = (0, 0);
            longest_gap = Duration::from_ticks(0);
        }
        Timer::after(PUMP_PERIOD).await;
    }
}
