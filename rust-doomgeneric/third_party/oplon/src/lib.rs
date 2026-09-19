//! # oplon
//!
//! A `no_std`, integer-only **OPL2 / OPL3 (Yamaha YM3812 / YMF262)** FM
//! synthesis core, with a clean-room ROM set generated entirely at compile time
//! (no float, no hand-pasted magic tables).
//!
//! The chip powers up as an OPL2 (9 channels, mono, waveforms 0-3) and becomes
//! a full OPL3 — 18 channels, stereo panning, waveforms 4-7 — once OPL3 mode is
//! enabled (port-1 register `0x05`, or [`Opl2::set_opl3_enabled`]), exactly like
//! the real hardware. So OPL2 code paths are unchanged while OPL3 streams get
//! the extra features (including 4-operator mode).
//!
//! ## Properties
//!
//! - `no_std`, zero heap allocation, zero floating-point, fully deterministic.
//! - Up to 18 two-operator FM channels (two banks) + the OPL rhythm mode.
//! - Register-level interface ([`Opl2::write_reg`]) *and* a structured
//!   note/patch interface ([`Opl2::set_patch`], [`Opl2::key_on`], …).
//! - Operators run at the chip's native rate (`clock / 72`); the output is
//!   4-point-cubic-resampled to your sample rate, so the chip's own aliasing
//!   matches the silicon.
//! - Log-sine / exp ROMs and the envelope-increment table are **generated**
//!   from their defining hardware formulas in integer `const fn`, verified
//!   within ±1 LSB of the canonical values.
//!
//! ## Minimal usage (register level — what a VGM/DRO player needs)
//!
//! ```rust
//! use oplon::Opl2;
//!
//! let mut opl = Opl2::new(44_100); // output sample rate
//! // Program a simple carrier tone on channel 0 (raw AdLib registers):
//! opl.write_reg(0x20, 0x01); // op0 mult = 1
//! opl.write_reg(0x40, 0x00); // op0 total level = loud
//! opl.write_reg(0x60, 0xF0); // op0 fast attack
//! opl.write_reg(0x80, 0x00); // op0 sustain/release
//! opl.write_reg(0xA0, 0x98); // F-number low
//! opl.write_reg(0xB0, 0x31); // key-on + block 4 + F-number high
//! for _ in 0..44_100 {
//!     let (left, right): (i32, i32) = opl.render_frame();
//!     // feed `left`/`right` to your DAC or audio buffer
//!     let _ = (left, right);
//! }
//! ```
//!
//! ## Scope
//!
//! 2-operator voices on both register banks (18 channels), OPL3 stereo panning
//! (register 0xC0 bits 4/5), the 8 OPL3 waveforms, and **4-operator linking**
//! (port-1 register `0x04`, the four connection algorithms).
//!
//! ## Clean-room & attribution
//!
//! This crate is MIT-licensed. The OPL emulators it is *validated against* are
//! not: Schism Tracker's `player/fmopl3.c` (a ymf262 core) is **GPL-2.0+,
//! © Jarek Burczynski**, and Nuked-OPL3 is LGPL. **No code or data table is
//! copied from either.** Every constant here is independently generated from
//! the documented hardware behaviour and integer math: the log-sin/exp ROMs
//! from their defining formulas, the envelope increments from the documented
//! `(4+lo)·2^rate_hi` rate law, the tremolo/vibrato LFOs from their documented
//! triangle/8-step shapes, and the `MUL`/`KSL`/key-scale values from the
//! Yamaha YM3812/YMF262 datasheet. Those references are consulted only as
//! *behavioural* references (to know what the silicon does); the
//! implementation — structure, code and tables — is original.

#![no_std]
#![cfg_attr(not(feature = "iram"), forbid(unsafe_code))]
#![cfg_attr(feature = "iram", deny(unsafe_code))]
#![allow(missing_docs)]

mod chip;
mod operator;
mod tables;

pub use chip::{
    ChannelPatch, OpPatch, Opl2, OPL2_CHANNELS, OPL3_CHANNELS, OPL_NATIVE_RATE, YM3812_CLOCK,
    YMF262_CLOCK,
};

/// Alias for [`Opl2`]: the same chip type, which becomes a full OPL3 (18
/// channels, stereo, waveforms 4-7) once [`Opl2::set_opl3_enabled`] — or the
/// port-1 `0x05` register — turns OPL3 mode on.
pub use chip::Opl2 as Opl3;
