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
//! The ring is [`RING_CHUNKS`] = 3 DMA descriptors of [`CHUNK_FRAMES`] stereo frames each, in
//! internal RAM (DMA cannot read PSRAM here). esp-hal only tells us about ring space one whole
//! descriptor at a time, and its I2S driver always builds 4092-byte descriptors, except that a
//! circular buffer of at most 8184 bytes is cut into exactly three equal ones. So the finest ring
//! there is has three chunks: [`Speaker::free_frames`] is always a whole number of chunks and
//! [`Speaker::push`] wants whole chunks. (Smaller descriptors declared with `dma_buffers!` are
//! silently ignored by the I2S driver: an earlier version did that and got nonsense free counts.)
//!
//! This file is shared with the `sound_test` example (`#[path]`), so it must not depend on the
//! rest of the firmware.

use core::sync::atomic::{AtomicBool, Ordering};

use esp_hal::{
    delay::Delay,
    dma::DmaTransferTxCircular,
    dma_circular_buffers,
    i2c::master::{Error as I2cError, I2c},
    i2s::master::{Channels, Config, DataFormat, I2s, I2sTx},
    peripherals::{DMA_CH1, GPIO13, GPIO33, GPIO34, I2S1},
    time::Rate,
    Blocking,
};
use esp_println::println;
use static_cell::StaticCell;

/// Doom's sound effects are 11025 Hz; 22050 is twice that (cheap nearest-neighbour resampling in
/// the engine) and the amp has a matching rate setting.
pub const SAMPLE_RATE: u32 = 11_025;

/// One DMA descriptor: 256 stereo frames (11.6 ms), 1 KB.
pub const CHUNK_FRAMES: usize = 128;
pub const RING_CHUNKS: usize = 3;
/// The ring: 768 frames (35 ms), 3 KB of internal RAM. With the DMA a chunk into the ring at all
/// times, a sound reaches the speaker after two to three chunks (23-35 ms).
pub const RING_FRAMES: usize = RING_CHUNKS * CHUNK_FRAMES;
const FRAME_BYTES: usize = 4;
const RING_BYTES: usize = RING_FRAMES * FRAME_BYTES;
// esp-hal cuts a circular buffer of up to two default chunks (4092 bytes) into three descriptors.
const _: () = assert!(RING_BYTES <= 2 * 4092 && RING_BYTES % 3 == 0);

const AMP_ADDRESS: u8 = 0x36;
/// The AW9523B I/O expander, whose output port 0 bit 2 is the amp's reset line (high = running).
const AW9523B_ADDRESS: u8 = 0x58;
const AW9523B_OUTPUT_P0: u8 = 0x02;
const AMP_RESET_BIT: u8 = 1 << 2;

type Tx = I2sTx<'static, Blocking>;
type Transfer = DmaTransferTxCircular<'static, Tx>;

/// I2S output into the DMA ring. Not `Send` (esp-hal's DMA state holds raw pointers): create it
/// on the core that uses it.
pub struct Speaker {
    /// The leaked I2S transmitter, kept as a pointer so the transfer can be started again after it
    /// was stopped (esp-hal's transfer swallows the `&mut` it borrows).
    tx: *mut Tx,
    ring: *mut [u8; RING_BYTES],
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

        let (_, _, ring, descriptors) = dma_circular_buffers!(0, RING_BYTES);
        let ring: *mut [u8; RING_BYTES] = ring;
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
        // Stop the old transfer (dropping it does) before touching the ring or the transmitter.
        drop(self.transfer.take());
        // SAFETY: `ring` and `tx` point to the static ring buffer and the transmitter leaked in
        // `open`, which nothing else uses; the DMA that read the ring and the borrow of `tx` held
        // by the previous transfer ended with the drop just above.
        let (ring, tx): (&'static mut [u8; RING_BYTES], &'static mut Tx) =
            unsafe { (&mut *self.ring, &mut *self.tx) };
        // A fresh start plays the whole ring first: make that silence, not old audio.
        ring.fill(0);
        self.transfer = tx.write_dma_circular(&*ring).ok();
        self.transfer.is_some()
    }

    /// Starts over with a ring of silence and forgets earlier restarts. For after setup work that
    /// took longer than the ring lasts.
    pub fn rearm(&mut self) {
        self.start();
        self.restarts = 0;
    }

    /// Frames the ring can take right now, a whole number of chunks. If the DMA ran dry (nobody pushed for a whole ring's
    /// time) esp-hal's bookkeeping is dead, so the transfer is restarted: a short glitch, then it
    /// carries on with 0 free frames until the DMA has played some silence.
    pub fn free_frames(&mut self) -> usize {
        let free = match self.transfer.as_mut() {
            Some(transfer) => transfer.available().map_err(|_| ()),
            None => Err(()),
        };
        match free {
            Ok(bytes) => bytes.min(RING_BYTES) / FRAME_BYTES / CHUNK_FRAMES * CHUNK_FRAMES,
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

    /// Pushes one chunk of interleaved stereo samples into the ring; only call it while
    /// [`free_frames`](Self::free_frames) is at least a chunk (esp-hal's bookkeeping goes wrong
    /// with partial chunks, so it is always exactly one). Returns whether it went in.
    pub fn push_chunk(&mut self, samples: &[i16; 2 * CHUNK_FRAMES]) -> bool {
        let Some(transfer) = self.transfer.as_mut() else { return false };
        // Little endian, left slot first.
        let mut bytes = [0u8; CHUNK_FRAMES * FRAME_BYTES];
        for (out, sample) in bytes.chunks_exact_mut(2).zip(samples) {
            out.copy_from_slice(&sample.to_le_bytes());
        }
        transfer.push(&bytes).is_ok()
    }
}

/// The AW88298's sample-rate setting for `rate` Hz (register 0x06's low bits), the way M5Unified
/// works it out: 0 = 8k, 1 = 11k, 2 = 12k, 3 = 16k, 4 = 22.05k, 5 = 24k, 6 = 32k, 7 = 44.1k, 8 = 48k.
fn rate_code(rate: u32) -> u16 {
    const LIMITS: [u32; 10] = [4, 5, 6, 8, 10, 11, 15, 20, 22, 44];
    let units = (rate + 1102) / 2205;
    LIMITS.iter().position(|&limit| units <= limit).unwrap_or(LIMITS.len() - 1) as u16
}

// The I2C helpers use esp-hal's own I2C type and are kept out of line: written generically over
// `embedded_hal::i2c::I2c` they made LLVM's Xtensa backend fail on esp-hal's `transaction` with
// "Cannot scavenge register without an emergency spill slot" in the game build.

#[inline(never)]
fn read16(i2c: &mut I2c<'_, Blocking>, register: u8) -> Option<u16> {
    let mut data = [0u8; 2];
    i2c.write_read(AMP_ADDRESS, &[register], &mut data).ok()?;
    Some(u16::from_be_bytes(data))
}

#[inline(never)]
fn write16(i2c: &mut I2c<'_, Blocking>, register: u8, value: u16) -> Result<(), I2cError> {
    let [high, low] = value.to_be_bytes();
    i2c.write(AMP_ADDRESS, &[register, high, low])
}

/// Drives the amp's reset line (AW9523B P0_2, output register 0x02 bit 2).
#[inline(never)]
fn amp_reset_line(i2c: &mut I2c<'_, Blocking>, high: bool) -> Result<(), I2cError> {
    let mut port = [0u8];
    i2c.write_read(AW9523B_ADDRESS, &[AW9523B_OUTPUT_P0], &mut port)?;
    let port = if high { port[0] | AMP_RESET_BIT } else { port[0] & !AMP_RESET_BIT };
    i2c.write(AW9523B_ADDRESS, &[AW9523B_OUTPUT_P0, port])
}

/// Prints the amp registers that say something about its state.
pub fn log_amp(i2c: &mut I2c<'_, Blocking>, when: &str) {
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
pub fn init_amp(i2c: &mut I2c<'_, Blocking>) -> Result<(), I2cError> {
    log_amp(i2c, "before reset");
    amp_reset_line(i2c, false)?;
    Delay::new().delay_millis(5);
    amp_reset_line(i2c, true)?;
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
