//! Between the game and the speaker.
//!
//! The game (core 1) calls [`frames_wanted`] and [`write`] from the platform's audio hooks; both
//! only touch a lock-free queue and never wait. That queue carries the sound effects at their own
//! 11025 Hz. [`pump`], a task on core 0, builds the I2S DMA ring's chunks one at a time whenever
//! the DMA has played one: effects (each frame repeated to the ring's 22050 Hz) plus the music
//! that `music` has rendered ahead of time. When a queue runs short (the game is late: a level
//! load, a slow frame) it pads with silence, so a stall is a gap in the sound, not a replay of old
//! audio. The queues are `core_s3_audio` (unit tested on the host); the ring is `audio`.
//!
//! Latency: a chunk moved into the ring plays after the two chunks ahead of it (23-35 ms), and the
//! game keeps about four effect chunks (46 ms) queued, since it only refills once per tick.

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
use crate::music;

/// The rate the engine mixes the sound effects at (what the DMX samples are recorded at, so it
/// does no resampling) and asks the platform for in `audio_open`.
pub const SFX_RATE: u32 = 11_025;

/// Ring frames per effect frame.
const UPSAMPLE: usize = (audio::SAMPLE_RATE / SFX_RATE) as usize;
const _: () = assert!(audio::SAMPLE_RATE % SFX_RATE == 0);

/// Effect frames in one ring chunk.
const SFX_CHUNK_FRAMES: usize = CHUNK_FRAMES / UPSAMPLE;

/// Frames between the game and the pump (must be a power of two): 93 ms at 11025 Hz, 4 KB.
const QUEUE_FRAMES: usize = 1024;

/// What the game keeps queued: four chunks (46 ms). The game refills once per tick (28-40 ms), so
/// the queue has to hold a tick's worth plus a chunk, or the pump runs short between ticks.
const QUEUE_TARGET: usize = 4 * SFX_CHUNK_FRAMES;

/// How often the pump looks at the ring. The DMA frees a chunk every 11.6 ms and the pump has to
/// be there within two chunks of that.
const PUMP_PERIOD: Duration = Duration::from_millis(2);

/// Master volume in 256ths of what the engine mixes (256 would be the engine's own level; the
/// effects have the two channels averaged first). The CoreS3's speaker is very loud: 128 (-6 dB)
/// was "very distracting", 32 (-18 dB) was still too much, so the default is 8 (-30 dB), meant to
/// be just audible. Build with `SOUND_LEVEL=<0..=256>` for another level. The in-game volume
/// sliders work on top of it. The amp's own volume register is left at full on purpose. (A
/// multiply and a shift: this runs for every effect frame on the game core.)
const VOLUME_256THS: i32 = parse_level(option_env!("SOUND_LEVEL"), 8);

/// The music's share of the master volume, in quarters. The synthesizer's output, with the
/// engine's own gain for it, is about as loud as the effects at the same setting (that is how it
/// is balanced on the desktop), so this keeps it a bit under them: it never overpowers an effect.
const MUSIC_QUARTERS: i32 = 3;

/// `SOUND_LEVEL` as a number; a plain digit string of at most 256, or `default` if unset.
const fn parse_level(text: Option<&str>, default: i32) -> i32 {
    let Some(text) = text else { return default };
    let digits = text.as_bytes();
    assert!(!digits.is_empty(), "SOUND_LEVEL must be a number from 0 to 256");
    let (mut level, mut i) = (0, 0);
    while i < digits.len() {
        assert!(digits[i].is_ascii_digit(), "SOUND_LEVEL must be a number from 0 to 256");
        level = level * 10 + (digits[i] - b'0') as i32;
        assert!(level <= 256, "SOUND_LEVEL must be a number from 0 to 256");
        i += 1;
    }
    level
}

/// One sample of the synthesizer's output (the engine's own scale, music gain included) at the
/// music's level.
pub fn music_sample(sample: i32) -> i16 {
    // Quarters of the master in 256ths: `sample * level * quarters / 4 / 256`.
    let scaled = (sample * (VOLUME_256THS * MUSIC_QUARTERS)) >> 10;
    scaled.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}

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
        Ok(()) => {
            spawner.spawn(pump(speaker).expect("spawn sound pump"));
            spawner.spawn(music::task().expect("spawn music task"));
        }
        Err(error) => println!("audio: amp init failed ({error:?}); no sound"),
    }
}

/// True while muted.
pub fn muted() -> bool {
    MUTED.load(Ordering::Relaxed)
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

/// Queues interleaved stereo effect `samples` (at [`SFX_RATE`]) from the engine. The CoreS3 has a single speaker, so this
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

/// Builds one ring chunk from the effects and the music: every effect frame is repeated
/// [`UPSAMPLE`] times (the effects' own rate is what the mixer used, so this is what the engine
/// would have done itself at the ring's rate) and the music frames are added. Returns the effect
/// frames that were real audio (the rest is silence) and the music frames missing.
fn mix_chunk(chunk: &mut [i16; 2 * CHUNK_FRAMES]) -> (usize, usize) {
    let mut effects = [0i16; 2 * SFX_CHUNK_FRAMES];
    let effects_popped = QUEUE.pop_padded(&mut effects);
    let mut music = [0i16; 2 * CHUNK_FRAMES];
    let music_popped = music::QUEUE.pop_padded(&mut music);
    if MUTED.load(Ordering::Relaxed) {
        // What was queued before the mute is dropped too.
        chunk.fill(0);
        return (0, 0);
    }
    for (i, frame) in chunk.chunks_exact_mut(2).enumerate() {
        // Both slots of a frame are equal in both queues.
        let sum = i32::from(effects[2 * (i / UPSAMPLE)]) + i32::from(music[2 * i]);
        frame.fill(sum.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);
    }
    (effects_popped, music::missing(music_popped, CHUNK_FRAMES))
}

/// Moves one chunk into the ring for every chunk the DMA has played. Returns how many effect
/// frames were real audio and how many were silence, and how many music frames were missing.
#[inline(never)]
fn refill(speaker: &mut Speaker) -> (u32, u32, u32) {
    let (mut real, mut silence, mut music_late) = (0, 0, 0);
    for _ in 0..speaker.free_frames() / CHUNK_FRAMES {
        let mut chunk = [0i16; 2 * CHUNK_FRAMES];
        let (popped, late) = mix_chunk(&mut chunk);
        if speaker.push_chunk(&chunk) {
            real += (popped * UPSAMPLE) as u32;
            silence += ((SFX_CHUNK_FRAMES - popped) * UPSAMPLE) as u32;
            music_late += late as u32;
        }
    }
    (real, silence, music_late)
}

/// Feeds the speaker, forever. Runs on core 0. `speaker` must have been opened on this core.
#[embassy_executor::task]
pub async fn pump(mut speaker: Speaker) {
    // Setting the amp up took longer than the ring lasts, so the DMA has run dry by now.
    speaker.rearm();
    READY.store(true, Ordering::Release);
    let (mut moved, mut silence, mut music_late) = (0u32, 0u32, 0u32);
    let mut report_at = Instant::now() + Duration::from_secs(5);
    let mut last_run = Instant::now();
    let mut longest_gap = Duration::from_ticks(0);
    loop {
        let now = Instant::now();
        longest_gap = longest_gap.max(now - last_run);
        last_run = now;

        let (real, quiet, late) = refill(&mut speaker);
        moved += real;
        silence += quiet;
        music_late += late;

        if now >= report_at {
            report_at += Duration::from_secs(5);
            let (music_us, music_frames, music_peak) = music::take_stats();
            println!(
                "[audio] 5 s: game mixed {} effect frames (peak {}), {moved} frames played + {silence} silence, \
                 longest pump gap {} ms, {} DMA restarts; music: {} ms of core 0 for {music_frames} frames \
                 ({} per mille, peak {music_peak}), {music_late} frames late",
                GAME_FRAMES.swap(0, Ordering::Relaxed),
                PEAK.swap(0, Ordering::Relaxed),
                longest_gap.as_millis(),
                speaker.restarts,
                music_us / 1000,
                music_us / 5000,
            );
            (moved, silence, music_late) = (0, 0, 0);
            longest_gap = Duration::from_ticks(0);
        }
        Timer::after(PUMP_PERIOD).await;
    }
}
