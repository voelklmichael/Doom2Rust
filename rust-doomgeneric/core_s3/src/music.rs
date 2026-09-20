//! The music: the engine's OPL2 synthesizer (`MusicPlayer`), run on core 0.
//!
//! The synthesizer is far too expensive for the game core: at the speaker's 22050 Hz it needs
//! 30 to 65 percent of a core at 240 MHz (see `PLAN.md`), and core 1 is already saturated by the
//! renderer. So the engine hands the music over to the platform (`DoomPlatform::music_open`): the
//! game (core 1) builds the player once and sends it, then sends every music request as a small
//! message. [`task`] on core 0 owns the player, applies the messages and keeps a queue of rendered
//! audio topped up; the pump in `sound` mixes that queue with the sound effects into the DMA ring.
//! The game never waits for the music, and the music does not care how long a game frame takes.

use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};

use alloc::vec::Vec;
use core_s3_audio::FrameQueue;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Timer};
use esp_println::println;
use rust_doomgeneric::{GenMidi, MusicCommand, MusicPlayer};
use static_cell::StaticCell;

use crate::{audio, lagprobe, sound};

/// Rendered music, mono sent as stereo like the effects (both slots equal), at the ring's rate.
/// 46 ms: the synthesizer runs ahead by this much, which is how much of a stall of core 0 (Wi-Fi,
/// a slow LCD chunk) the music survives. 4 KB of internal RAM.
pub static QUEUE: FrameQueue<1024> = FrameQueue::new();

/// Frames rendered per step. One step costs 1 ms of core 0 at most (the busiest song), and the
/// task lets the other tasks (the LCD's DMA chunks, the network) run in between.
const SLICE_FRAMES: usize = 8;

/// How long the task sleeps when the queue is full. The pump takes 256 frames every 11.6 ms.
const IDLE_PERIOD: Duration = Duration::from_millis(4);

/// What the game asks for, as owned messages for core 0.
enum Message {
    /// The synthesizer, built on the game core from the WAD's GENMIDI lump. Sent once.
    Attach(&'static mut MusicPlayer),
    Register(Vec<u8>),
    Play {
        looping: bool,
    },
    Stop,
    Pause,
    Resume,
    Volume(i32),
}

/// Requests from the game (core 1) to [`task`] (core 0). The task polls it, so nothing on core 1
/// ever sleeps or waits for a waker on the other core. Sixteen are far more than the game sends
/// between two polls (a level change is stop, register, play, and the menu slider sends one per
/// key press).
static MESSAGES: Channel<CriticalSectionRawMutex, Message, 16> = Channel::new();

// Counters for the serial log (see `take_stats`).
static BUSY_US: AtomicU32 = AtomicU32::new(0);
static RENDERED_FRAMES: AtomicU32 = AtomicU32::new(0);
static PEAK: AtomicI32 = AtomicI32::new(0);

/// True once the game has attached a player: only then is a short queue a problem.
static ATTACHED: AtomicBool = AtomicBool::new(false);

fn send(message: Message) {
    if MESSAGES.try_send(message).is_err() {
        println!("[music] request queue full; request dropped");
    }
}

/// The engine offers the GENMIDI bank: builds the synthesizer and sends it to core 0. Called on
/// the game core. Returns whether the platform plays the music (false leaves the engine without
/// music, which is right when there is no speaker).
pub fn open(genmidi: &[u8]) -> bool {
    if !sound::ready() {
        return false;
    }
    let Some(bank) = GenMidi::parse(genmidi) else {
        println!("[music] GENMIDI lump not understood; no music");
        return false;
    };
    // In internal RAM (a static): it is touched every 20 microseconds, and in PSRAM its cache
    // lines would compete with the game's.
    static PLAYER: StaticCell<MusicPlayer> = StaticCell::new();
    let player = PLAYER.init(MusicPlayer::new(bank, audio::SAMPLE_RATE));
    ATTACHED.store(true, Ordering::Relaxed);
    send(Message::Attach(player));
    true
}

/// A music request from the game (core 1).
pub fn command(command: MusicCommand<'_>) {
    send(match command {
        MusicCommand::Register(lump) => Message::Register(lump.to_vec()),
        MusicCommand::Play { looping } => Message::Play { looping },
        MusicCommand::Stop => Message::Stop,
        MusicCommand::Pause => Message::Pause,
        MusicCommand::Resume => Message::Resume,
        MusicCommand::Volume(volume) => Message::Volume(volume),
    });
}

/// How many of the `wanted` frames the pump got from the queue are missing, once a player is
/// attached (before that there is no music, and the queue is rightly empty).
pub fn missing(popped: usize, wanted: usize) -> usize {
    if ATTACHED.load(Ordering::Relaxed) {
        wanted - popped
    } else {
        0
    }
}

/// Microseconds the synthesizer ran, frames it rendered and the loudest sample (after the volume)
/// since the last call.
pub fn take_stats() -> (u32, u32, i32) {
    (
        BUSY_US.swap(0, Ordering::Relaxed),
        RENDERED_FRAMES.swap(0, Ordering::Relaxed),
        PEAK.swap(0, Ordering::Relaxed),
    )
}

/// One slice of the synthesizer's output, as queue frames.
fn render(player: &mut MusicPlayer) {
    let mut acc = [0i32; 2 * SLICE_FRAMES];
    player.render_add(&mut acc);
    let mut frames = [0i16; 2 * SLICE_FRAMES];
    let mut peak = 0;
    for (frame, sum) in frames.chunks_exact_mut(2).zip(acc.chunks_exact(2)) {
        // The OPL2 is mono: left and right are the same.
        let sample = sound::music_sample(sum[0]);
        frame.fill(sample);
        peak = peak.max(i32::from(sample).abs());
    }
    PEAK.fetch_max(peak, Ordering::Relaxed);
    QUEUE.push(&frames);
}

fn apply(player: &mut Option<&'static mut MusicPlayer>, message: Message) {
    if let Message::Attach(new) = message {
        *player = Some(new);
        return;
    }
    let Some(player) = player else { return };
    match message {
        Message::Register(lump) => {
            if !player.register(&lump) {
                println!("[music] not a MUS lump ({} bytes)", lump.len());
            }
        }
        Message::Play { looping } => player.play(looping),
        Message::Stop => player.stop(),
        Message::Pause => player.pause(),
        Message::Resume => player.resume(),
        Message::Volume(volume) => player.set_volume(volume),
        Message::Attach(_) => {}
    }
}

/// Owns the synthesizer and keeps [`QUEUE`] full. Runs on core 0.
#[embassy_executor::task]
pub async fn task() {
    let mut player: Option<&'static mut MusicPlayer> = None;
    loop {
        while let Ok(message) = MESSAGES.try_receive() {
            apply(&mut player, message);
        }
        // Muted, the song stands still (nothing is rendered, so the sequencer does not advance)
        // and core 0 is left alone; it carries on from there when the sound comes back.
        if let (Some(player), false) = (player.as_deref_mut(), sound::muted()) {
            while QUEUE.free() >= SLICE_FRAMES {
                let start = Instant::now();
                render(player);
                let spent = start.elapsed().as_micros() as u32;
                lagprobe::slice(spent);
                BUSY_US.fetch_add(spent, Ordering::Relaxed);
                RENDERED_FRAMES.fetch_add(SLICE_FRAMES as u32, Ordering::Relaxed);
                // Let the LCD's DMA chunks, the network and the pump run.
                embassy_futures::yield_now().await;
            }
        }
        Timer::after(IDLE_PERIOD).await;
    }
}
