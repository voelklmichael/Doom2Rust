//! The LCD, driven over SPI with DMA.
//!
//! Core 0 owns the SPI bus. Before the game starts it draws text through the BSP's `Display`, which
//! reaches the bus through [`LcdSpi`] and [`LcdDc`]. While the game runs, core 1 converts each
//! finished frame into a buffer in internal RAM and hands it over ([`acquire_frame`] and
//! [`submit_frame`]); [`run_pump`] on core 0 streams it to the panel by DMA while core 1 renders the
//! next one. The CPU only sets each transfer up.
//!
//! A single DMA transfer is at most 32736 bytes and a frame is 128000, so a frame goes out as four
//! chunks of 50 rows. Between chunks chip select goes high for a moment, which the panel does not mind
//! (the BSP's own blit already sends 256-byte chunks that way).

use core::{cell::RefCell, convert::Infallible, sync::atomic::{AtomicBool, Ordering}};

use critical_section::Mutex;
use embassy_time::{Duration, Timer};
use embedded_hal::{
    digital::{ErrorType as PinErrorType, OutputPin},
    spi::{ErrorType as SpiErrorType, Operation, SpiDevice},
};
use esp_hal::{
    delay::Delay,
    dma::DmaTxBuf,
    dma_tx_buffer,
    gpio::{Level, Output, OutputConfig},
    peripherals::{DMA_CH0, GPIO3, GPIO35, GPIO36, GPIO37, SPI2},
    spi::{
        master::{Config as SpiConfig, Spi, SpiDma},
        Error, Mode,
    },
    time::Rate,
    Blocking,
};

/// DOOM's picture: 320x200, centred vertically on the 320x240 panel.
const WIDTH: usize = 320;
const HEIGHT: usize = 200;
const TOP: u16 = 20;

const CHUNKS: usize = 4;
const CHUNK_PIXELS: usize = WIDTH * HEIGHT / CHUNKS;
const CHUNK_BYTES: usize = CHUNK_PIXELS * 2;
/// Commands, parameters and the BSP's drawing all go through one small buffer.
const SCRATCH_BYTES: usize = 512;

/// The SPI clock. The panel is rated for less than this on paper; 40 MHz is what M5's own driver uses.
pub const SPI_HZ: u32 = 40_000_000;

const CMD_COLUMN_ADDRESS_SET: u8 = 0x2A;
const CMD_ROW_ADDRESS_SET: u8 = 0x2B;
const CMD_MEMORY_WRITE: u8 = 0x2C;

/// How often core 0 checks for a finished frame, and for the end of a DMA chunk.
const POLL_FRAME: Duration = Duration::from_micros(500);
const POLL_DMA: Duration = Duration::from_micros(250);

/// The SPI bus, the data/command pin and a scratch buffer for small writes.
pub struct Lcd {
    spi: Option<SpiDma<'static, Blocking>>,
    scratch: Option<DmaTxBuf>,
    dc: Output<'static>,
}

impl Lcd {
    /// Call once: the DMA buffers are statics.
    pub fn new(
        spi2: SPI2<'static>,
        sclk: GPIO36<'static>,
        mosi: GPIO37<'static>,
        cs: GPIO3<'static>,
        dc: GPIO35<'static>,
        dma: DMA_CH0<'static>,
    ) -> Self {
        static TAKEN: AtomicBool = AtomicBool::new(false);
        assert!(!TAKEN.swap(true, Ordering::SeqCst), "Lcd::new called twice");

        let spi = Spi::new(
            spi2,
            SpiConfig::default().with_frequency(Rate::from_hz(SPI_HZ)).with_mode(Mode::_0),
        )
        .expect("LCD SPI")
        .with_sck(sclk)
        .with_mosi(mosi)
        .with_cs(cs)
        .with_dma(dma);
        Self {
            spi: Some(spi),
            scratch: Some(dma_tx_buffer!(SCRATCH_BYTES).expect("LCD scratch buffer")),
            dc: Output::new(dc, Level::Low, OutputConfig::default()),
        }
    }

    /// Sends `bytes` (a command, its parameters or pixels) and waits for them to go out.
    fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        for part in bytes.chunks(SCRATCH_BYTES) {
            let mut buf = self.scratch.take().expect("scratch buffer");
            buf.as_mut_slice()[..part.len()].copy_from_slice(part);
            buf.set_length(part.len());
            let spi = self.spi.take().expect("LCD SPI");
            match spi.write(part.len(), buf) {
                Ok(transfer) => {
                    let (spi, buf) = transfer.wait();
                    self.spi = Some(spi);
                    self.scratch = Some(buf);
                }
                Err((error, spi, buf)) => {
                    self.spi = Some(spi);
                    self.scratch = Some(buf);
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    fn command(&mut self, command: u8, parameters: &[u8]) -> Result<(), Error> {
        self.dc.set_low();
        self.write(&[command])?;
        self.dc.set_high();
        self.write(parameters)
    }

    /// Points the panel's write cursor at the frame's rectangle and starts a memory write, leaving
    /// D/C high for the pixel data that follows.
    fn start_frame(&mut self) -> Result<(), Error> {
        let (x1, y0, y1) = ((WIDTH - 1) as u16, TOP, TOP + HEIGHT as u16 - 1);
        self.command(CMD_COLUMN_ADDRESS_SET, &[0, 0, (x1 >> 8) as u8, x1 as u8])?;
        self.command(CMD_ROW_ADDRESS_SET, &[(y0 >> 8) as u8, y0 as u8, (y1 >> 8) as u8, y1 as u8])?;
        self.command(CMD_MEMORY_WRITE, &[])
    }
}

/// The `SpiDevice` the BSP's `Display` draws through.
pub struct LcdSpi<'a>(pub &'a RefCell<Lcd>);

impl SpiErrorType for LcdSpi<'_> {
    type Error = Error;
}

impl SpiDevice for LcdSpi<'_> {
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Error> {
        for operation in operations {
            match operation {
                Operation::Write(bytes) => self.0.borrow_mut().write(bytes)?,
                Operation::DelayNs(ns) => Delay::new().delay_nanos(*ns),
                // The display is write-only.
                Operation::Read(_) | Operation::Transfer(..) | Operation::TransferInPlace(_) => {
                    return Err(Error::Unsupported)
                }
            }
        }
        Ok(())
    }
}

/// The data/command pin as the BSP's `Display` sees it.
pub struct LcdDc<'a>(pub &'a RefCell<Lcd>);

impl PinErrorType for LcdDc<'_> {
    type Error = Infallible;
}

impl OutputPin for LcdDc<'_> {
    fn set_low(&mut self) -> Result<(), Infallible> {
        self.0.borrow_mut().dc.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Infallible> {
        self.0.borrow_mut().dc.set_high();
        Ok(())
    }
}

/// One frame of RGB565 pixels in DMA buffers, big-endian as the panel wants them.
pub struct Frame {
    chunks: [Option<DmaTxBuf>; CHUNKS],
}

impl Frame {
    /// Call once: the buffers are statics.
    fn new() -> Self {
        Self {
            chunks: [
                Some(dma_tx_buffer!(CHUNK_BYTES).expect("frame buffer")),
                Some(dma_tx_buffer!(CHUNK_BYTES).expect("frame buffer")),
                Some(dma_tx_buffer!(CHUNK_BYTES).expect("frame buffer")),
                Some(dma_tx_buffer!(CHUNK_BYTES).expect("frame buffer")),
            ],
        }
    }

    /// Fills the frame from the engine's 320x200 palette indices; `colors` maps an index to the
    /// two bytes of its RGB565 pixel.
    pub fn fill(&mut self, indices: &[u8], colors: &[[u8; 2]; 256]) {
        assert_eq!(indices.len(), WIDTH * HEIGHT);
        for (chunk, pixels) in self.chunks.iter_mut().zip(indices.chunks_exact(CHUNK_PIXELS)) {
            let bytes = chunk.as_mut().expect("frame chunk").as_mut_slice();
            for (out, &index) in bytes.chunks_exact_mut(2).zip(pixels) {
                out.copy_from_slice(&colors[usize::from(index)]);
            }
        }
    }
}

/// The frame the game can fill now (there is one, so it is only there once the last one has gone out).
static FREE: Mutex<RefCell<Option<Frame>>> = Mutex::new(RefCell::new(None));
/// A filled frame waiting for the pump.
static READY: Mutex<RefCell<Option<Frame>>> = Mutex::new(RefCell::new(None));

/// Makes the frame buffer available to the game. Call once, before the game starts.
pub fn init_frames() {
    static TAKEN: AtomicBool = AtomicBool::new(false);
    assert!(!TAKEN.swap(true, Ordering::SeqCst), "init_frames called twice");
    let frame = Frame::new();
    critical_section::with(|cs| *FREE.borrow_ref_mut(cs) = Some(frame));
}

/// Takes the frame buffer, waiting for the previous frame to finish going out if it has not yet.
pub fn acquire_frame() -> Frame {
    loop {
        if let Some(frame) = critical_section::with(|cs| FREE.borrow_ref_mut(cs).take()) {
            return frame;
        }
        Delay::new().delay_micros(100);
    }
}

/// Hands a filled frame to the pump.
pub fn submit_frame(frame: Frame) {
    critical_section::with(|cs| *READY.borrow_ref_mut(cs) = Some(frame));
}

/// Streams submitted frames to the panel, forever. Runs on core 0; it sleeps between DMA chunks, so
/// the network tasks sharing the executor keep running.
pub async fn run_pump(lcd: &RefCell<Lcd>) -> ! {
    loop {
        let ready = critical_section::with(|cs| READY.borrow_ref_mut(cs).take());
        let Some(frame) = ready else {
            Timer::after(POLL_FRAME).await;
            continue;
        };
        let frame = send(lcd, frame).await;
        critical_section::with(|cs| *FREE.borrow_ref_mut(cs) = Some(frame));
    }
}

async fn send(lcd: &RefCell<Lcd>, mut frame: Frame) -> Frame {
    lcd.borrow_mut().start_frame().expect("LCD window");
    for slot in &mut frame.chunks {
        let buf = slot.take().expect("frame chunk");
        let spi = lcd.borrow_mut().spi.take().expect("LCD SPI");
        let transfer = match spi.write(CHUNK_BYTES, buf) {
            Ok(transfer) => transfer,
            Err((error, _, _)) => panic!("LCD DMA: {error:?}"),
        };
        while !transfer.is_done() {
            Timer::after(POLL_DMA).await;
        }
        let (spi, buf) = transfer.wait();
        lcd.borrow_mut().spi = Some(spi);
        *slot = Some(buf);
    }
    frame
}
