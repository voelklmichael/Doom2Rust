//! Sound output for the Linux build.
//!
//! The engine mixes; this only delivers. Mixed stereo 16-bit PCM goes to a
//! system player (`aplay` or `paplay`, whichever starts first) over a
//! pipe, so the build needs no audio library. A writer thread owns the pipe:
//! the game loop must never wait on the sound server, and if the player stalls
//! the audio is dropped rather than the game.
//!
//! `DOOM_AUDIO` overrides the choice of output:
//! * `off` - no sound
//! * `file:PATH` - write raw signed 16-bit little-endian stereo to `PATH`
//!   (what the tests and `sox`/`ffmpeg -f s16le -ac 2` want)

use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use std::thread::JoinHandle;
use std::time::Instant;

/// Length of the buffer the player is asked for. Together with what sits in
/// the pipe this is the delay between the game and the speaker.
const PLAYER_LATENCY_MS: u32 = 60;
/// After a stall the game loop asks for at most this much catch-up audio; the
/// rest is skipped so the delay does not grow.
const MAX_CATCH_UP_MS: u64 = 80;
/// Chunks queued for the writer thread before new ones are dropped.
const QUEUE_CHUNKS: usize = 16;

pub struct AudioSink {
    rate: u32,
    /// Set by the first `frames_wanted`, so level loading before the first tick
    /// is not fed to the player as a burst of silence.
    clock: Option<Instant>,
    frames_sent: u64,
    tx: Option<SyncSender<Vec<i16>>>,
    writer: Option<JoinHandle<()>>,
}

impl AudioSink {
    /// Starts the output at `rate` Hz, or returns `None` (after saying why on
    /// stderr) if there is nothing to play on.
    pub fn open(rate: u32) -> Option<Self> {
        let (output, child): (Box<dyn Write + Send>, Option<Child>) =
            match std::env::var("DOOM_AUDIO").ok().as_deref() {
                Some("off") => return None,
                Some(spec) if spec.starts_with("file:") => {
                    let path = &spec["file:".len()..];
                    match std::fs::File::create(path) {
                        Ok(file) => (Box::new(file), None),
                        Err(e) => {
                            eprintln!("sound disabled: cannot create {path}: {e}");
                            return None;
                        }
                    }
                }
                _ => {
                    let mut child = spawn_player(rate)?;
                    let stdin = child.stdin.take()?;
                    (Box::new(stdin), Some(child))
                }
            };
        let (tx, rx) = sync_channel::<Vec<i16>>(QUEUE_CHUNKS);
        let writer = std::thread::spawn(move || {
            let mut output = output;
            let mut child = child;
            for chunk in rx {
                let bytes: Vec<u8> = chunk.iter().flat_map(|s| s.to_le_bytes()).collect();
                if output.write_all(&bytes).is_err() {
                    // The player went away; stop feeding it.
                    break;
                }
            }
            drop(output);
            if let Some(child) = child.as_mut() {
                // Closing stdin lets the player play out what it has buffered.
                let _ = child.wait();
            }
        });
        Some(Self {
            rate,
            clock: None,
            frames_sent: 0,
            tx: Some(tx),
            writer: Some(writer),
        })
    }

    /// Frames elapsed in real time since the last call, capped at
    /// [`MAX_CATCH_UP_MS`].
    pub fn frames_wanted(&mut self) -> usize {
        let clock = *self.clock.get_or_insert_with(Instant::now);
        let due = clock.elapsed().as_micros() as u64 * u64::from(self.rate) / 1_000_000;
        let cap = u64::from(self.rate) * MAX_CATCH_UP_MS / 1000;
        let behind = due.saturating_sub(self.frames_sent);
        if behind > cap {
            self.frames_sent += behind - cap;
        }
        due.saturating_sub(self.frames_sent) as usize
    }

    /// Hands `samples` (interleaved stereo) to the writer thread.
    pub fn write(&mut self, samples: &[i16]) {
        self.frames_sent += (samples.len() / 2) as u64;
        if let Some(tx) = &self.tx {
            if let Err(TrySendError::Disconnected(_)) = tx.try_send(samples.to_vec()) {
                // The writer gave up (dead player): stop producing audio.
                self.tx = None;
            }
        }
    }
}

impl Drop for AudioSink {
    fn drop(&mut self) {
        self.tx = None;
        if let Some(writer) = self.writer.take() {
            let _ = writer.join();
        }
    }
}

/// Starts the first player that exists, reading raw PCM on stdin.
fn spawn_player(rate: u32) -> Option<Child> {
    let rate = rate.to_string();
    let latency = PLAYER_LATENCY_MS;
    let players: [(&str, Vec<String>); 2] = [
        (
            "aplay",
            [
                "-q",
                "-t",
                "raw",
                "-f",
                "S16_LE",
                "-c",
                "2",
                "-r",
                &rate,
                &format!("--buffer-time={}", latency * 1000),
                "-",
            ]
            .map(String::from)
            .to_vec(),
        ),
        (
            "paplay",
            [
                "--raw",
                "--format=s16le",
                "--channels=2",
                &format!("--rate={rate}"),
                &format!("--latency-msec={latency}"),
            ]
            .map(String::from)
            .to_vec(),
        ),
    ];
    for (program, args) in players {
        let spawned = Command::new(program)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Ok(child) = spawned {
            return Some(child);
        }
    }
    eprintln!("sound disabled: neither aplay nor paplay could be started");
    None
}
