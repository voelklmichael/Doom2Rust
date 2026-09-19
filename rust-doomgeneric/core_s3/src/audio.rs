//! The CoreS3's speaker: an AW88298 amplifier fed by I2S1 with a circular DMA ring.
//!
//! Hardware facts (M5Unified's CoreS3 code, the `core-s3` BSP and its `sound_wav` example):
//! * The amp is on the internal I2C (0x36, SDA GPIO12 / SCL GPIO11) and is held in reset by the
//!   AW9523B expander's P0_2, which the BSP's `CoreS3::init_core_s3_power` already releases.
//! * Audio goes ESP32-S3 I2S1 -> AW88298 over BCLK = GPIO34, WS = GPIO33, DOUT = GPIO13. The amp is
//!   an I2S *slave* and derives its clocks from BCLK/WS with its own PLL, so no MCLK (GPIO0) is
//!   needed. The format is Philips I2S, 16 bit slots, 32 BCLK per frame.
//! * The amp needs a running BCLK before it is enabled, so [`Speaker::open`] starts the I2S clock
//!   (sending silence) first and [`init_amp`] comes second.
//! * The amp's own volume register stays at "full" (M5Unified does the same and scales the samples
//!   instead); volume is the caller's business.
//!
//! The ring is [`RING_FRAMES`] frames of [`CHUNK_FRAMES`]-frame DMA descriptors, in internal RAM
//! (DMA cannot read PSRAM here). esp-hal only frees ring space one descriptor at a time, hence the
//! small chunks: the pacing granularity is 128 frames = 5.8 ms.
//!
//! This file is shared with the `sound_test` example (`#[path]`), so it must not depend on the
//! rest of the firmware.

use core::sync::atomic::{AtomicBool, Ordering};

use core_s3::aw9523b::{Aw9523b, ExpanderPin, Port};
use embedded_hal::i2c::I2c;
use esp_hal::{
    delay::Delay,
    dma::DmaTransferTxCircular,
    dma_circular_buffers_chunk_size,
    i2s::master::{Channels, Config, DataFormat, I2s, I2sTx},
    peripherals::{DMA_CH1, GPIO13, GPIO33, GPIO34, I2S1},
    time::Rate,
    Blocking,
};
use esp_println::println;
use static_cell::StaticCell;

/// Doom's sound effects are 11025 Hz; 22050 is twice that (cheap nearest-neighbour resampling in
/// the engine) and the amp has a matching rate setting.
pub const SAMPLE_RATE: u32 = 22_050;

/// One DMA descriptor: 128 stereo frames, 512 bytes.
pub const CHUNK_FRAMES: usize = 128;
/// The ring: 2048 frames (93 ms), 8 KB of internal RAM.
pub const RING_FRAMES: usize = 16 * CHUNK_FRAMES;
const FRAME_BYTES: usize = 4;
const RING_BYTES: usize = RING_FRAMES * FRAME_BYTES;

const AMP_ADDRESS: u8 = 0x36;

type Tx = I2sTx<'static, Blocking>;
type Transfer = DmaTransferTxCircular<'static, Tx>;

/// I2S output into the DMA ring. Not `Send` (esp-hal's DMA state holds raw pointers): create it
/// on the core that uses it.
pub struct Speaker {
    /// The leaked I2S transmitter, kept as a pointer so the transfer can be started again after it
    /// was stopped (esp-hal's transfer swallows the `&mut` it borrows).
    tx: *mut Tx,
    ring: &'static [u8; RING_BYTES],
    /// `None` only if restarting after an underrun failed.
    transfer: Option<Transfer>,
    /// How often the DMA ran dry so far and had to be restarted (should stay 0).
    pub restarts: u32,
}

impl Speaker {
    /// Starts I2S1 on DMA channel 1 sending silence. Call once.
    pub fn open(
        i2s: I2S1<'static>,
        dma: DMA_CH1<'static>,
        bclk: GPIO34<'static>,
        ws: GPIO33<'static>,
        dout: GPIO13<'static>,
    ) -> Result<Self, &'static str> {
        static TAKEN: AtomicBool = AtomicBool::new(false);
        assert!(!TAKEN.swap(true, Ordering::SeqCst), "Speaker::open called twice");
        static TX: StaticCell<Tx> = StaticCell::new();

        let (_, _, ring, descriptors) =
            dma_circular_buffers_chunk_size!(0, RING_BYTES, CHUNK_FRAMES * FRAME_BYTES);
        ring.fill(0);
        let i2s = I2s::new(
            i2s,
            dma,
            Config::new_tdm_philips()
                .with_sample_rate(Rate::from_hz(SAMPLE_RATE))
                .with_data_format(DataFormat::Data16Channel16)
                .with_channels(Channels::STEREO),
        )
        .map_err(|_| "I2S configuration rejected")?;
        let tx = TX.init(i2s.i2s_tx.with_bclk(bclk).with_ws(ws).with_dout(dout).build(descriptors));
        let tx: *mut Tx = tx;
        let mut speaker = Self { tx, ring, transfer: None, restarts: 0 };
        if !speaker.start() {
            return Err("I2S DMA did not start");
        }
        Ok(speaker)
    }

    /// (Re)starts the circular transfer over the whole ring. The ring counts as full, so nothing
    /// can be pushed until the DMA has played some of it.
    fn start(&mut self) -> bool {
        // Stop the old transfer (dropping it does) before borrowing the transmitter again.
        drop(self.transfer.take());
        // SAFETY: `tx` points to the transmitter leaked in `open`, which nothing else uses; the
        // only other borrow of it was held by the previous transfer, dropped just above.
        let tx: &'static mut Tx = unsafe { &mut *self.tx };
        self.transfer = tx.write_dma_circular(self.ring).ok();
        self.transfer.is_some()
    }

    /// Frames the ring can take right now. If the DMA ran dry (nobody pushed for a whole ring's
    /// time) esp-hal's bookkeeping is dead, so the transfer is restarted: a short glitch, then it
    /// carries on with 0 free frames until the DMA has played some silence.
    pub fn free_frames(&mut self) -> usize {
        let free = match self.transfer.as_mut() {
            Some(transfer) => transfer.available().map_err(|_| ()),
            None => Err(()),
        };
        match free {
            Ok(bytes) => bytes / FRAME_BYTES,
            Err(()) => {
                self.restarts += 1;
                let started = self.start();
                if self.restarts.is_power_of_two() {
                    println!("[audio] DMA ran dry, restart #{} (ok: {started})", self.restarts);
                }
                0
            }
        }
    }

    /// Pushes interleaved stereo `samples` (no more than [`free_frames`](Self::free_frames) worth)
    /// into the ring and returns how many frames went in.
    pub fn push(&mut self, samples: &[i16]) -> usize {
        let Some(transfer) = self.transfer.as_mut() else { return 0 };
        let mut pushed = 0;
        // Convert to bytes (little endian, left slot first) through a small stack buffer.
        let mut bytes = [0u8; 64 * FRAME_BYTES];
        for part in samples.chunks(64 * 2) {
            let bytes = &mut bytes[..part.len() * 2];
            for (out, sample) in bytes.chunks_exact_mut(2).zip(part) {
                out.copy_from_slice(&sample.to_le_bytes());
            }
            match transfer.push(bytes) {
                Ok(_) => pushed += part.len() / 2,
                Err(_) => break,
            }
        }
        pushed
    }

    /// Pushes `frames` frames of silence (as far as they fit). The game's pump uses it, the
    /// `sound_test` example does not.
    #[allow(dead_code)]
    pub fn push_silence(&mut self, mut frames: usize) {
        while frames > 0 {
            let now = frames.min(64);
            if self.push(&[0i16; 64 * 2][..now * 2]) < now {
                break;
            }
            frames -= now;
        }
    }
}

/// The AW88298's sample-rate setting for `rate` Hz (register 0x06's low bits), the way M5Unified
/// works it out: 0 = 8k, 1 = 11k, 2 = 12k, 3 = 16k, 4 = 22.05k, 5 = 24k, 6 = 32k, 7 = 44.1k, 8 = 48k.
fn rate_code(rate: u32) -> u16 {
    const LIMITS: [u32; 10] = [4, 5, 6, 8, 10, 11, 15, 20, 22, 44];
    let units = (rate + 1102) / 2205;
    LIMITS.iter().position(|&limit| units <= limit).unwrap_or(LIMITS.len() - 1) as u16
}

fn read16<I: I2c>(i2c: &mut I, register: u8) -> Option<u16> {
    let mut data = [0u8; 2];
    i2c.write_read(AMP_ADDRESS, &[register], &mut data).ok()?;
    Some(u16::from_be_bytes(data))
}

fn write16<I: I2c>(i2c: &mut I, register: u8, value: u16) -> Result<(), I::Error> {
    let [high, low] = value.to_be_bytes();
    i2c.write(AMP_ADDRESS, &[register, high, low])
}

/// Prints the amp registers that say something about its state.
pub fn log_amp<I: I2c>(i2c: &mut I, when: &str) {
    println!(
        "[audio] AW88298 {when}: id(00)={:04X?} status(01)={:04X?} sysctrl(04)={:04X?} \
         sysctrl2(05)={:04X?} i2s(06)={:04X?} volume(0C)={:04X?}",
        read16(i2c, 0x00),
        read16(i2c, 0x01),
        read16(i2c, 0x04),
        read16(i2c, 0x05),
        read16(i2c, 0x06),
        read16(i2c, 0x0C),
    );
}

/// Sets the amp up for [`SAMPLE_RATE`] (M5Unified's sequence) and unmutes it. Start the I2S clock
/// ([`Speaker::open`]) first. Call it while the caller still owns the internal I2C.
///
/// The amp is reset first (AW9523B P0_2 low, then high): it keeps its registers across an ESP32
/// reset, so without this a working setup could hide a broken sequence.
pub fn init_amp<I: I2c>(i2c: &mut I) -> Result<(), I::Error> {
    log_amp(i2c, "before reset");
    let mut expander = Aw9523b::new(&mut *i2c);
    let reset = ExpanderPin { port: Port::P0, index: 2 };
    expander.set_output(reset, false)?;
    Delay::new().delay_millis(5);
    expander.set_output(reset, true)?;
    Delay::new().delay_millis(10);
    log_amp(i2c, "after reset");
    write16(i2c, 0x61, 0x0673)?; // boost mode off
    write16(i2c, 0x04, 0x4040)?; // I2S on, amplifier on, not powered down
    write16(i2c, 0x05, 0x0008)?; // no mute, no AGC
    write16(i2c, 0x06, 0x14C0 | rate_code(SAMPLE_RATE))?; // 16 bit slots, sample rate bucket
    write16(i2c, 0x0C, 0x0064)?; // full volume; the caller scales the samples
    Delay::new().delay_millis(10);
    log_amp(i2c, "after init");
    Ok(())
}
