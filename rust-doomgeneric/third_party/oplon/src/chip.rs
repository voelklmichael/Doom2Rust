//! The OPL chip: up to 18 two-operator FM channels summed to stereo, plus the
//! OPL rhythm/percussion mode. [`Opl2`] is the public, register-level
//! interface — write chip registers with [`Opl2::write_reg`] (register port 0)
//! and [`Opl2::write_reg_high`] (port 1) exactly as a real AdLib / OPL3 driver
//! (or a DRO / VGM stream) would, then pull output-rate stereo frames with
//! [`Opl2::render_frame`]. A structured note/patch API ([`Opl2::set_patch`],
//! [`Opl2::key_on`], …) is also provided for callers that prefer decoded
//! patches over raw registers.
//!
//! The chip powers up in **OPL2 mode** (9 channels, mono, 2-bit waveform
//! select, second register bank ignored) and stays bit-for-bit an OPL2 until
//! the OPL3-enable bit (port-1 register `0x05`, or [`Opl2::set_opl3_enabled`])
//! is set, exactly like the real YMF262 — so OPL2 streams are unaffected while
//! OPL3 streams get 18 channels, stereo panning, waveforms 4-7, and 4-operator
//! mode (port-1 register `0x04`).

use crate::operator::Operator;

/// Native OPL sample rate for the canonical clock (`YM3812_CLOCK / 72`, equal
/// to the OPL3 `YMF262_CLOCK / 288`). F-numbers are computed against the chip's
/// native rate; the resampler then converts to the caller's output rate.
pub const OPL_NATIVE_RATE: u32 = 49716;

/// The canonical YM3812 (OPL2) master clock (Hz). The native operator rate is
/// `clock / 72`.
pub const YM3812_CLOCK: u32 = 3_579_545;

/// The canonical YMF262 (OPL3) master clock (Hz), 4× the OPL2 clock. The native
/// operator rate is `clock / 288` — the same 49716 Hz, since the YMF262 has 4×
/// the operators to service per sample.
pub const YMF262_CLOCK: u32 = 14_318_180;

/// Number of 2-operator channels on an OPL2 (YM3812).
pub const OPL2_CHANNELS: usize = 9;

/// Number of 2-operator channels on an OPL3 (YMF262): two banks of 9.
pub const OPL3_CHANNELS: usize = 18;

/// One 2-operator FM channel: modulator (op 0) → carrier (op 1), or both
/// summed in additive ("AM") mode.
struct OplChannel {
    modulator: Operator,
    carrier: Operator,
    /// Feedback register (0..7) applied to the modulator's self-FM.
    feedback: u8,
    /// Connection: `false` = FM (mod → carrier), `true` = additive.
    additive: bool,
    /// Stereo enable bits (OPL3 panning): `(left, right)`.
    pan_l: bool,
    pan_r: bool,
    /// Latched F-number low byte (reg 0xA0) and high 2 bits + block (reg
    /// 0xB0), so an 0xA0 write can recompute pitch with the current block.
    fnum_low: u8,
    fnum_high: u8,
    block: u8,
}

impl OplChannel {
    const fn new() -> Self {
        Self {
            modulator: Operator::new(),
            carrier: Operator::new(),
            feedback: 0,
            additive: false,
            pan_l: true,
            pan_r: true,
            fnum_low: 0,
            fnum_high: 0,
            block: 0,
        }
    }

    /// Recompute pitch on both operators from the latched F-number / block.
    fn refresh_freq(&mut self) {
        let fnum = (self.fnum_low as u16) | (((self.fnum_high & 0x03) as u16) << 8);
        self.modulator.set_frequency(fnum, self.block);
        self.carrier.set_frequency(fnum, self.block);
    }

    #[inline(always)]
    fn is_silent(&self) -> bool {
        self.carrier.is_silent() && (!self.additive || self.modulator.is_silent())
    }

    /// One output frame for this channel → signed mono magnitude (~±4084 per
    /// operator at full level — the chip's 12-bit output).
    #[inline(always)]
    fn next_mono(&mut self, tremolo: u16, vibpos: u8, eg_cnt: u32) -> i32 {
        let fb = self.modulator.feedback_phase(self.feedback);
        let mod_out = self.modulator.next(fb, tremolo, vibpos, eg_cnt);
        if self.additive {
            // Both operators sound; carrier runs unmodulated.
            let car = self.carrier.next(0, tremolo, vibpos, eg_cnt);
            mod_out + car
        } else {
            // FM: the modulator output bends the carrier phase directly.
            self.carrier.next(mod_out, tremolo, vibpos, eg_cnt)
        }
    }
}

// Rhythm-mode channels (bank 0) and reg 0xBD bit masks.
const CH_BD: usize = 6; // bass drum (2-operator FM)
const CH_HH_SD: usize = 7; // modulator = hi-hat (HH), carrier = snare (SD)
const CH_TT_CY: usize = 8; // modulator = tom-tom (TT), carrier = cymbal (CY)
const RM_BD: u8 = 0x10;
const RM_SD: u8 = 0x08;
const RM_TT: u8 = 0x04;
const RM_CY: u8 = 0x02;
const RM_HH: u8 = 0x01;

/// An OPL2/OPL3 chip: up to 18 two-operator channels summed to stereo.
///
/// The operators run at the chip's **native rate** (`clock / 72` for OPL2 /
/// `clock / 288` for OPL3, 49716 Hz for the canonical clocks) so their internal
/// aliasing matches real hardware; [`Opl2::render_frame`] resamples that native
/// stream down to the caller's output rate with 4-point cubic interpolation.
/// Rendering directly at 44100/48000 would fold harmonics at the *wrong*
/// Nyquist and produce inauthentic timbre on bright operators.
pub struct Opl2 {
    channels: [OplChannel; OPL3_CHANNELS],
    /// Native operator rate (Hz).
    native_rate: u32,
    /// Native→output resampler: how many native frames to advance per output
    /// frame, Q32. `> 1<<32` since native > output.
    resamp_step_q32: u64,
    /// Fractional native-stream position for the next output frame, Q32.
    resamp_cursor_q32: u64,
    /// The four most recent native frames `[n-3, n-2, n-1, n]`; the output
    /// position lies between `[1]` and `[2]` (4-point cubic interpolation).
    native_hist: [(i32, i32); 4],
    primed: bool,
    /// Global LFO counter (one tick per native frame); drives the tremolo and
    /// vibrato sub-LFOs.
    lfo_timer: u32,
    /// Tremolo triangle position (0..209), advanced every 64 native frames.
    tremolopos: u32,
    /// Global envelope counter (one tick per native frame). Every operator's
    /// envelope reads the *same* counter, so their fractional-rate dither
    /// phases stay aligned exactly as on the chip.
    eg_cnt: u32,
    /// **OPL rhythm mode** (reg 0xBD bit 5): channels 6/7/8 become BD/HH/SD/TT/CY.
    rhythm_mode: bool,
    /// Active percussion bits (reg 0xBD bits 0-4).
    rhythm_keys: u8,
    /// 23-bit noise LFSR for HH/SD/CY (rhythm mode).
    noise: u32,
    /// **OPL3 enable** (port-1 reg 0x05 bit 0): when set, all 18 channels
    /// sound, 0xC0 pan bits are authoritative, and waveforms 4-7 unlock. When
    /// clear the chip is a plain OPL2 (9 channels, mono, waveforms 0-3).
    opl3_enable: bool,
    /// **4-operator connection select** (port-1 reg 0x04, 6 bits): each bit
    /// links a channel pair (n / n+3) into one 4-operator voice. See
    /// [`Self::render_fourop`].
    fourop_mask: u8,
    /// Tremolo depth (reg 0xBD bit 7 "DAM"): `false` = shallow (≈1 dB, the
    /// default), `true` = deep (≈4.8 dB).
    deep_tremolo: bool,
}

impl Opl2 {
    /// New OPL rendering at `output_rate` Hz (stereo), assuming the canonical
    /// 3.579545 MHz YM3812 clock (native operator rate 49716 Hz). Starts in
    /// OPL2 mode.
    pub fn new(output_rate: u32) -> Self {
        Self::with_chip_clock(YM3812_CLOCK, output_rate)
    }

    /// New OPL for a specific YM3812-style master `chip_clock` (Hz); the native
    /// operator rate is `chip_clock / 72`. For an OPL3 (YMF262) clock, divide by
    /// 288 yourself and use [`Self::with_native_rate`].
    pub fn with_chip_clock(chip_clock: u32, output_rate: u32) -> Self {
        // Round to nearest: the canonical 3.579545 MHz clock → 49716 Hz (the
        // documented OPL rate), not the 49715 a truncating divide would give.
        // Widen to u64 so a near-`u32::MAX` clock can't overflow the `+ 36`.
        let native = (((chip_clock as u64 + 36) / 72).max(1)) as u32;
        Self::with_native_rate(native, output_rate)
    }

    /// New OPL running its operators at an explicit `native_rate` (Hz). Use this
    /// when the divisor is not the OPL2 `/72` — e.g. an OPL3 (YMF262) stream
    /// runs at `ymf262_clock / 288`. Pitch scales linearly with the native rate.
    pub fn with_native_rate(native_rate: u32, output_rate: u32) -> Self {
        let native = native_rate.max(1);
        let r = output_rate.max(1) as u64;
        Self {
            channels: core::array::from_fn(|_| OplChannel::new()),
            native_rate: native,
            // native frames consumed per output frame = native_rate / output.
            resamp_step_q32: ((native as u64) << 32) / r,
            resamp_cursor_q32: 0,
            native_hist: [(0, 0); 4],
            primed: false,
            lfo_timer: 0,
            tremolopos: 0,
            eg_cnt: 0,
            rhythm_mode: false,
            rhythm_keys: 0,
            noise: 1,
            opl3_enable: false,
            fourop_mask: 0,
            deep_tremolo: false,
        }
    }

    /// The native operator rate (Hz) this chip runs at.
    pub fn native_rate(&self) -> u32 {
        self.native_rate
    }

    /// Enable or disable OPL3 mode programmatically (the register equivalent is
    /// port-1 reg `0x05` bit 0). In OPL3 mode all 18 channels sound, the 0xC0
    /// pan bits are authoritative (both clear → channel muted), and waveforms
    /// 4-7 are available. In OPL2 mode (the default) only 9 channels sound,
    /// panning stays both-sides, and the waveform select is 2-bit.
    pub fn set_opl3_enabled(&mut self, on: bool) {
        self.opl3_enable = on;
        for c in self.channels.iter_mut() {
            c.modulator.set_opl3(on);
            c.carrier.set_opl3(on);
        }
    }

    /// Whether OPL3 mode is currently enabled.
    pub fn opl3_enabled(&self) -> bool {
        self.opl3_enable
    }

    /// Write a raw OPL register on **port 0** (registers 0x00-0xFF): the OPL2
    /// register space, and the OPL3 first bank (channels 0-8). This is the
    /// primitive a register-level player (DRO / VGM / a ported sound driver)
    /// needs. Registers: 0x20-0x35 (AM/VIB/EG/KSR/MULT), 0x40-0x55 (KSL/TL),
    /// 0x60-0x75 (AR/DR), 0x80-0x95 (SL/RR), 0xA0-0xA8 (F-num low), 0xB0-0xB8
    /// (F-num high / block / key-on), 0xC0-0xC8 (feedback/connection/pan),
    /// 0xE0-0xF5 (waveform), 0xBD (rhythm mode + drum key-on).
    pub fn write_reg(&mut self, reg: u8, val: u8) {
        self.write_bank(0, reg, val);
    }

    /// Write a raw OPL register on **port 1** (the OPL3 second bank): channels
    /// 9-17 use the same register layout as port 0, plus two chip-global
    /// registers — `0x04` (4-operator connection select) and `0x05` (bit 0 =
    /// OPL3 enable). No effect on an OPL2 stream, which never touches port 1.
    pub fn write_reg_high(&mut self, reg: u8, val: u8) {
        match reg {
            0x04 => self.fourop_mask = val & 0x3F,
            0x05 => self.set_opl3_enabled(val & 0x01 != 0),
            _ => self.write_bank(OPL2_CHANNELS, reg, val),
        }
    }

    /// Decode an operator/channel register for one bank. `base` is the channel
    /// index the bank starts at (0 for port 0, 9 for port 1).
    fn write_bank(&mut self, base: usize, reg: u8, val: u8) {
        // Operator register block → (local channel 0..8, is_carrier). Offsets
        // 0x06/0x07/0x0E/0x0F unused.
        fn op_slot(off: u8) -> Option<(usize, bool)> {
            let (grp, idx) = (off / 8, off % 8);
            if idx >= 6 || grp >= 3 {
                return None;
            }
            let cbase = (grp * 3) as usize;
            match idx {
                0..=2 => Some((cbase + idx as usize, false)), // modulators
                3..=5 => Some((cbase + (idx - 3) as usize, true)), // carriers
                _ => None,
            }
        }
        fn op_mut(c: &mut OplChannel, carrier: bool) -> &mut Operator {
            if carrier {
                &mut c.carrier
            } else {
                &mut c.modulator
            }
        }
        match reg {
            0xBD if base == 0 => self.write_rhythm(val),
            0x20..=0x35 => {
                if let Some((ch, car)) = op_slot(reg - 0x20) {
                    if let Some(c) = self.channels.get_mut(base + ch) {
                        op_mut(c, car).write_reg20(val);
                    }
                }
            }
            0x40..=0x55 => {
                if let Some((ch, car)) = op_slot(reg - 0x40) {
                    if let Some(c) = self.channels.get_mut(base + ch) {
                        op_mut(c, car).write_reg40(val);
                    }
                }
            }
            0x60..=0x75 => {
                if let Some((ch, car)) = op_slot(reg - 0x60) {
                    if let Some(c) = self.channels.get_mut(base + ch) {
                        op_mut(c, car).write_reg60(val);
                    }
                }
            }
            0x80..=0x95 => {
                if let Some((ch, car)) = op_slot(reg - 0x80) {
                    if let Some(c) = self.channels.get_mut(base + ch) {
                        op_mut(c, car).write_reg80(val);
                    }
                }
            }
            0xE0..=0xF5 => {
                if let Some((ch, car)) = op_slot(reg - 0xE0) {
                    if let Some(c) = self.channels.get_mut(base + ch) {
                        op_mut(c, car).write_reg_e0(val);
                    }
                }
            }
            0xA0..=0xA8 => {
                let ch = (reg - 0xA0) as usize;
                if let Some(c) = self.channels.get_mut(base + ch) {
                    c.fnum_low = val;
                    c.refresh_freq();
                }
            }
            0xB0..=0xB8 => {
                let ch = (reg - 0xB0) as usize;
                let rhythm_ch = base == 0
                    && self.rhythm_mode
                    && (ch == CH_BD || ch == CH_HH_SD || ch == CH_TT_CY);
                if let Some(c) = self.channels.get_mut(base + ch) {
                    c.fnum_high = val & 0x03;
                    c.block = (val >> 2) & 0x07;
                    c.refresh_freq();
                    // Key-on/off (bit 5) — ignored for rhythm channels in
                    // rhythm mode (0xBD keys them).
                    if !rhythm_ch {
                        if val & 0x20 != 0 {
                            c.modulator.key_on();
                            c.carrier.key_on();
                        } else {
                            c.modulator.key_off();
                            c.carrier.key_off();
                        }
                    }
                }
            }
            0xC0..=0xC8 => {
                let opl3 = self.opl3_enable;
                let ch = (reg - 0xC0) as usize;
                if let Some(c) = self.channels.get_mut(base + ch) {
                    c.feedback = (val >> 1) & 0x07;
                    c.additive = val & 0x01 != 0;
                    // Stereo panning (bits 4/5). In OPL3 mode the bits are
                    // authoritative (both clear → muted). In plain OPL2 mode the
                    // chip powers up with both channels on and 0xC0 carries no
                    // pan info, so only *narrow* the panning when a bit is set
                    // (else a mono OPL2 stream would silence itself).
                    if opl3 || val & 0x30 != 0 {
                        c.pan_l = val & 0x10 != 0;
                        c.pan_r = val & 0x20 != 0;
                    }
                }
            }
            _ => {}
        }
    }

    /// Write the OPL rhythm register (0xBD): bit 5 = enable, bits 0-4 = drum
    /// key-ons. A rising bit → key-on the drum's operator(s); a falling bit →
    /// key-off.
    pub fn write_rhythm(&mut self, bd: u8) {
        // Vibrato depth (DVB, bit 6): `1` = shallow (default), `0` = deep. This
        // is a chip-global control independent of rhythm mode, so it is applied
        // to every operator on any 0xBD write.
        let vib_shift = ((bd >> 6) & 1) ^ 1;
        for c in self.channels.iter_mut() {
            c.modulator.set_vib_shift(vib_shift);
            c.carrier.set_vib_shift(vib_shift);
        }
        // Tremolo depth (DAM, bit 7): also chip-global.
        self.deep_tremolo = bd & 0x80 != 0;
        let was_enabled = self.rhythm_mode;
        self.rhythm_mode = bd & 0x20 != 0;
        // On the rising edge of rhythm mode, treat currently-set drum bits as
        // fresh key-ons (compare against 0), so a driver that arms a drum in
        // one write and enables rhythm mode in a later write still triggers it.
        let old = if was_enabled { self.rhythm_keys } else { 0 };
        let new = bd & 0x1F;
        self.rhythm_keys = new;
        if !self.rhythm_mode {
            return;
        }
        let edge = |mask: u8| (new & mask != 0, (new & mask) != (old & mask));
        for &(ch, modu, mask) in &[
            (CH_BD, true, RM_BD),
            (CH_BD, false, RM_BD),
            (CH_HH_SD, true, RM_HH),
            (CH_HH_SD, false, RM_SD),
            (CH_TT_CY, true, RM_TT),
            (CH_TT_CY, false, RM_CY),
        ] {
            let (on, changed) = edge(mask);
            if !changed {
                continue;
            }
            if let Some(c) = self.channels.get_mut(ch) {
                let op = if modu { &mut c.modulator } else { &mut c.carrier };
                if on {
                    op.key_on();
                } else {
                    op.key_off();
                }
            }
        }
    }

    pub fn rhythm_enabled(&self) -> bool {
        self.rhythm_mode
    }

    /// Number of channels currently sounding: 18 in OPL3 mode, 9 in OPL2 mode.
    pub fn channel_count(&self) -> usize {
        if self.opl3_enable {
            OPL3_CHANNELS
        } else {
            OPL2_CHANNELS
        }
    }

    /// Program a channel's two operators + routing from a decoded patch.
    pub fn set_patch(&mut self, ch: usize, patch: &ChannelPatch) {
        let Some(c) = self.channels.get_mut(ch) else {
            return;
        };
        c.feedback = patch.feedback & 0x07;
        c.additive = patch.additive;
        let m = &patch.modulator;
        c.modulator.set_patch(
            m.mul, m.waveform, m.tl, m.ksl, m.ksr, m.sustaining, m.attack, m.decay, m.sustain,
            m.release, m.am, m.vib,
        );
        let cr = &patch.carrier;
        c.carrier.set_patch(
            cr.mul, cr.waveform, cr.tl, cr.ksl, cr.ksr, cr.sustaining, cr.attack, cr.decay,
            cr.sustain, cr.release, cr.am, cr.vib,
        );
    }

    /// Program a **single** operator (modulator or carrier) of a channel from
    /// an [`OpPatch`] — used to load the rhythm-mode percussion voices (HH/SD
    /// on ch7, TT/CY on ch8, which are single operators; BD on ch6 uses the
    /// full channel via [`Self::set_patch`]).
    pub fn set_rhythm_operator(&mut self, ch: usize, carrier: bool, p: &OpPatch) {
        let Some(c) = self.channels.get_mut(ch) else {
            return;
        };
        let op = if carrier { &mut c.carrier } else { &mut c.modulator };
        op.set_patch(
            p.mul, p.waveform, p.tl, p.ksl, p.ksr, p.sustaining, p.attack, p.decay, p.sustain,
            p.release, p.am, p.vib,
        );
    }

    /// Set the pitch of a single rhythm operator (ch7/ch8 carry two drums
    /// sharing the channel F-number on hardware, but each drum's operator can
    /// be phase-driven independently here).
    pub fn set_operator_frequency(&mut self, ch: usize, carrier: bool, fnum: u16, block: u8) {
        if let Some(c) = self.channels.get_mut(ch) {
            let op = if carrier { &mut c.carrier } else { &mut c.modulator };
            op.set_frequency(fnum, block);
        }
    }

    /// Set a channel's F-number / block (pitch). Recomputes both operators'
    /// phase increments.
    pub fn set_frequency(&mut self, ch: usize, fnum: u16, block: u8) {
        if let Some(c) = self.channels.get_mut(ch) {
            c.modulator.set_frequency(fnum, block);
            c.carrier.set_frequency(fnum, block);
        }
    }

    /// Override the carrier total level (per-note volume). `tl` is the 0..63
    /// register value.
    pub fn set_carrier_tl(&mut self, ch: usize, tl: u8) {
        if let Some(c) = self.channels.get_mut(ch) {
            c.carrier.set_total_level(tl);
            if c.additive {
                c.modulator.set_total_level(tl);
            }
        }
    }

    /// Override a single rhythm operator's total level (per-note volume for a
    /// percussion voice, which shares its channel with another drum and so
    /// can't use [`Self::set_carrier_tl`]).
    pub fn set_rhythm_operator_tl(&mut self, ch: usize, carrier: bool, tl: u8) {
        if let Some(c) = self.channels.get_mut(ch) {
            let op = if carrier { &mut c.carrier } else { &mut c.modulator };
            op.set_total_level(tl);
        }
    }

    pub fn set_pan(&mut self, ch: usize, left: bool, right: bool) {
        if let Some(c) = self.channels.get_mut(ch) {
            c.pan_l = left;
            c.pan_r = right;
        }
    }

    pub fn key_on(&mut self, ch: usize) {
        if let Some(c) = self.channels.get_mut(ch) {
            c.modulator.key_on();
            c.carrier.key_on();
        }
    }

    pub fn key_off(&mut self, ch: usize) {
        if let Some(c) = self.channels.get_mut(ch) {
            c.modulator.key_off();
            c.carrier.key_off();
        }
    }

    pub fn channel_is_silent(&self, ch: usize) -> bool {
        self.channels.get(ch).map(|c| c.is_silent()).unwrap_or(true)
    }

    /// Any channel still sounding? (Cheap activity gate for the mixer.)
    pub fn any_active(&self) -> bool {
        self.channels.iter().any(|c| !c.is_silent())
    }

    /// One **native-rate** frame: advance the global LFO, then sum all active
    /// channels to stereo with the current tremolo / vibrato.
    #[inline(always)]
    fn render_native(&mut self) -> (i32, i32) {
        self.lfo_timer = self.lfo_timer.wrapping_add(1);
        self.eg_cnt = self.eg_cnt.wrapping_add(1);
        let eg_cnt = self.eg_cnt;
        // Tremolo (AM): a 0→26→0 triangle stepped once every 64 native frames
        // over 210 steps. The 0..26 eg-unit swing is the deep (DAM = 1)
        // depth; S3M/IT never set DAM, so the default `>> 2` gives the shallow
        // ≈1 dB depth.
        if self.lfo_timer & 0x3f == 0x3f {
            self.tremolopos = (self.tremolopos + 1) % 210;
        }
        let ramp = if self.tremolopos < 105 {
            self.tremolopos
        } else {
            209 - self.tremolopos
        };
        let am26 = (ramp * 26) / 104;
        // DAM = 0 → shallow (≈1 dB, `>> 2`); DAM = 1 → deep (the full 0..26 swing).
        let tremolo = (if self.deep_tremolo { am26 } else { am26 >> 2 }) as u16;
        // Vibrato (PM): an 8-step LFO advanced every 1024 frames (≈6 Hz).
        let vibpos = ((self.lfo_timer >> 10) & 7) as u8;

        let mut l: i32 = 0;
        let mut r: i32 = 0;
        let rhythm = self.rhythm_mode;
        let opl3 = self.opl3_enable;
        // OPL2 mode sounds only bank 0 (9 channels); OPL3 sounds all 18.
        let nch = self.channel_count();
        for ch in 0..nch {
            // In rhythm mode, channels 7/8 are rendered separately; channel 6
            // (BD) stays a normal FM channel (keyed by `write_rhythm`).
            if rhythm && (ch == CH_HH_SD || ch == CH_TT_CY) {
                continue;
            }
            // OPL3 4-operator groups (reg 0x104): a secondary channel is
            // consumed by its primary (skipped here); a primary renders the
            // whole 4-op pair.
            if opl3 {
                if let Some(bit) = fourop_secondary_bit(ch) {
                    if self.fourop_mask & (1 << bit) != 0 {
                        continue;
                    }
                }
                if let Some(bit) = fourop_primary_bit(ch) {
                    if self.fourop_mask & (1 << bit) != 0 {
                        let (dl, dr) = self.render_fourop(ch, tremolo, vibpos, eg_cnt);
                        l += dl;
                        r += dr;
                        continue;
                    }
                }
            }
            let c = &mut self.channels[ch];
            if c.is_silent() {
                continue;
            }
            let s = c.next_mono(tremolo, vibpos, eg_cnt);
            if c.pan_l {
                l += s;
            }
            if c.pan_r {
                r += s;
            }
        }

        if rhythm {
            let (dl, dr) = self.render_rhythm_percussion(tremolo, vibpos, eg_cnt);
            l += dl;
            r += dr;
        }
        // 23-bit noise LFSR (advances every native frame). The documented
        // YMF262 generator: the new top bit (22) is `bit14 XOR bit0`.
        let n_bit = ((self.noise >> 14) ^ self.noise) & 1;
        self.noise = (self.noise >> 1) | (n_bit << 22);
        (l, r)
    }

    /// Render an OPL3 **4-operator** group whose primary channel is `n` (its
    /// secondary is `n + 3`). The two channels' connection bits (0xC0 bit 0)
    /// select one of four algorithms; feedback applies only to operator 1 (the
    /// primary modulator), from the primary channel's feedback register. Each
    /// operator that reaches the output is routed through *its own* channel's
    /// pan bits (drivers normally set both channels' pan the same). All four
    /// operators are always advanced, matching the hardware.
    fn render_fourop(&mut self, n: usize, tremolo: u16, vibpos: u8, eg_cnt: u32) -> (i32, i32) {
        let sec = n + 3;
        // Skip a fully-idle group (all four operators released to silence).
        if self.channels[n].modulator.is_silent()
            && self.channels[n].carrier.is_silent()
            && self.channels[sec].modulator.is_silent()
            && self.channels[sec].carrier.is_silent()
        {
            return (0, 0);
        }
        // Disjoint mutable access to channels `n` and `n + 3`.
        let (lo, hi) = self.channels.split_at_mut(sec);
        let c0 = &mut lo[n];
        let c3 = &mut hi[0];
        let (cnt0, cnt3) = (c0.additive, c3.additive);
        let (p0l, p0r) = (c0.pan_l, c0.pan_r);
        let (p3l, p3r) = (c3.pan_l, c3.pan_r);

        // Operator 1 (primary modulator) with its self-feedback.
        let fb = c0.modulator.feedback_phase(c0.feedback);
        let op1 = c0.modulator.next(fb, tremolo, vibpos, eg_cnt);

        let mut l = 0i32;
        let mut r = 0i32;
        // Accumulate one operator's output through a channel's pan bits.
        macro_rules! emit {
            ($v:expr, $pl:expr, $pr:expr) => {{
                let v = $v;
                if $pl {
                    l += v;
                }
                if $pr {
                    r += v;
                }
            }};
        }
        match (cnt0, cnt3) {
            // 1 → 2 → 3 → 4   (out = op4)
            (false, false) => {
                let op2 = c0.carrier.next(op1, tremolo, vibpos, eg_cnt);
                let op3 = c3.modulator.next(op2, tremolo, vibpos, eg_cnt);
                let op4 = c3.carrier.next(op3, tremolo, vibpos, eg_cnt);
                emit!(op4, p3l, p3r);
            }
            // 1 + (2 → 3 → 4)   (out = op1 + op4)
            (true, false) => {
                let op2 = c0.carrier.next(0, tremolo, vibpos, eg_cnt);
                let op3 = c3.modulator.next(op2, tremolo, vibpos, eg_cnt);
                let op4 = c3.carrier.next(op3, tremolo, vibpos, eg_cnt);
                emit!(op1, p0l, p0r);
                emit!(op4, p3l, p3r);
            }
            // (1 → 2) + (3 → 4)   (out = op2 + op4)
            (false, true) => {
                let op2 = c0.carrier.next(op1, tremolo, vibpos, eg_cnt);
                let op3 = c3.modulator.next(0, tremolo, vibpos, eg_cnt);
                let op4 = c3.carrier.next(op3, tremolo, vibpos, eg_cnt);
                emit!(op2, p0l, p0r);
                emit!(op4, p3l, p3r);
            }
            // 1 + (2 → 3) + 4   (out = op1 + op3 + op4)
            (true, true) => {
                let op2 = c0.carrier.next(0, tremolo, vibpos, eg_cnt);
                let op3 = c3.modulator.next(op2, tremolo, vibpos, eg_cnt);
                let op4 = c3.carrier.next(0, tremolo, vibpos, eg_cnt);
                emit!(op1, p0l, p0r);
                emit!(op3, p3l, p3r);
                emit!(op4, p3l, p3r);
            }
        }
        (l, r)
    }

    /// Render the 4 percussion voices on channels 7/8 (HH/SD on ch7, TT/CY on
    /// ch8) using the OPL correlated-noise algorithm.
    fn render_rhythm_percussion(&mut self, tremolo: u16, vibpos: u8, eg_cnt: u32) -> (i32, i32) {
        let noise_bit = self.noise & 1;
        self.channels[CH_HH_SD].modulator.advance(vibpos, eg_cnt); // HH
        self.channels[CH_HH_SD].carrier.advance(vibpos, eg_cnt); // SD
        self.channels[CH_TT_CY].modulator.advance(vibpos, eg_cnt); // TT
        self.channels[CH_TT_CY].carrier.advance(vibpos, eg_cnt); // CY

        let hh_p = self.channels[CH_HH_SD].modulator.phase_out();
        let cy_p = self.channels[CH_TT_CY].carrier.phase_out();
        let bit = |v: u32, n: u32| (v >> n) & 1;
        let rm = (bit(hh_p, 2) ^ bit(hh_p, 7))
            | (bit(hh_p, 3) ^ bit(cy_p, 5))
            | (bit(cy_p, 3) ^ bit(cy_p, 5));

        // Hi-hat (HH)
        let hh_phase = (rm << 9) | if (rm ^ noise_bit) != 0 { 0xD0 } else { 0x34 };
        let hh_out = self.channels[CH_HH_SD].modulator.output_at(hh_phase, tremolo);
        // Snare drum (SD)
        let hh_bit8 = bit(hh_p, 8);
        let sd_phase = (hh_bit8 << 9) | ((hh_bit8 ^ noise_bit) << 8);
        let sd_out = self.channels[CH_HH_SD].carrier.output_at(sd_phase, tremolo);
        // Tom-tom (TT): normal operator (own phase, no modulation)
        let tt_phase = self.channels[CH_TT_CY].modulator.phase_out() & 0x3FF;
        let tt_out = self.channels[CH_TT_CY].modulator.output_at(tt_phase, tremolo);
        // Cymbal (CY)
        let cy_phase = (rm << 9) | 0x80;
        let cy_out = self.channels[CH_TT_CY].carrier.output_at(cy_phase, tremolo);

        // Percussion is summed at double gain (hardware), with each voice's
        // panning.
        let d7 = (hh_out + sd_out) * 2;
        let d8 = (tt_out + cy_out) * 2;
        let mut l = 0;
        let mut r = 0;
        if self.channels[CH_HH_SD].pan_l {
            l += d7;
        }
        if self.channels[CH_HH_SD].pan_r {
            r += d7;
        }
        if self.channels[CH_TT_CY].pan_l {
            l += d8;
        }
        if self.channels[CH_TT_CY].pan_r {
            r += d8;
        }
        (l, r)
    }

    /// Render one **output-rate** stereo frame by resampling the native stream
    /// with **4-point cubic (Catmull-Rom)** interpolation. The chip advances
    /// `resamp_step_q32` native frames per call, so its operators always run at
    /// the authentic chip rate.
    #[cfg_attr(feature = "iram", allow(unsafe_code), link_section = ".rwtext", inline(never))]
    pub fn render_frame(&mut self) -> (i32, i32) {
        if !self.primed {
            for i in 0..4 {
                self.native_hist[i] = self.render_native();
            }
            self.primed = true;
        }
        self.resamp_cursor_q32 += self.resamp_step_q32;
        while self.resamp_cursor_q32 >= (1u64 << 32) {
            self.resamp_cursor_q32 -= 1u64 << 32;
            self.native_hist[0] = self.native_hist[1];
            self.native_hist[1] = self.native_hist[2];
            self.native_hist[2] = self.native_hist[3];
            self.native_hist[3] = self.render_native();
        }
        let t = ((self.resamp_cursor_q32 & 0xFFFF_FFFF) >> 16) as i64;
        let h = self.native_hist;
        (
            cubic(h[0].0, h[1].0, h[2].0, h[3].0, t),
            cubic(h[0].1, h[1].1, h[2].1, h[3].1, t),
        )
    }
}

/// The reg-0x104 mask bit that makes channel `ch` a 4-op **primary**, if it can
/// be one. Primaries are channels 0/1/2 (bank 0) and 9/10/11 (bank 1); each
/// pairs with `ch + 3`.
fn fourop_primary_bit(ch: usize) -> Option<u8> {
    match ch {
        0..=2 => Some(ch as u8),
        9..=11 => Some((ch - 6) as u8),
        _ => None,
    }
}

/// The reg-0x104 mask bit that makes channel `ch` a 4-op **secondary**
/// (consumed by its primary `ch - 3`). Secondaries are channels 3/4/5 and
/// 12/13/14.
fn fourop_secondary_bit(ch: usize) -> Option<u8> {
    match ch {
        3..=5 => Some((ch - 3) as u8),
        12..=14 => Some((ch - 9) as u8),
        _ => None,
    }
}

/// Catmull-Rom cubic interpolation at fraction `t` (Q16, between `p1` and
/// `p2`).
#[inline]
fn cubic(p0: i32, p1: i32, p2: i32, p3: i32, t: i64) -> i32 {
    let (p0, p1, p2, p3) = (p0 as i64, p1 as i64, p2 as i64, p3 as i64);
    let a = -p0 + 3 * p1 - 3 * p2 + p3;
    let b = 2 * p0 - 5 * p1 + 4 * p2 - p3;
    let c = p2 - p0;
    let inner = b + ((a * t) >> 16);
    let inner = c + ((inner * t) >> 16);
    let half = (inner * t) >> 16;
    (p1 + (half >> 1)) as i32
}

/// One operator's decoded patch fields.
#[derive(Clone, Copy, Default)]
pub struct OpPatch {
    pub mul: u8,
    pub waveform: u8,
    pub tl: u8,
    pub ksl: u8,
    pub ksr: bool,
    pub sustaining: bool,
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
    /// Tremolo (AM) / vibrato enable bits.
    pub am: bool,
    pub vib: bool,
}

/// A full 2-operator channel patch.
#[derive(Clone, Copy, Default)]
pub struct ChannelPatch {
    pub modulator: OpPatch,
    pub carrier: OpPatch,
    pub feedback: u8,
    pub additive: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opl2_raw_registers_render_a_tone() {
        let mut opl = Opl2::new(44100);
        opl.write_reg(0x20, 0x01); // mod: mult=1
        opl.write_reg(0x23, 0x01); // car: mult=1
        opl.write_reg(0x40, 0x3F); // mod: TL max attenuation (silent modulator)
        opl.write_reg(0x43, 0x00); // car: TL loud
        opl.write_reg(0x60, 0xF0); // mod: fast attack
        opl.write_reg(0x63, 0xF0); // car: fast attack
        opl.write_reg(0x80, 0x00); // mod: sustain/release
        opl.write_reg(0x83, 0x00); // car
        opl.write_reg(0xC0, 0x00); // FM, no feedback
        opl.write_reg(0xA0, 0x98); // fnum low
        opl.write_reg(0xB0, 0x20 | (4 << 2) | 0x01); // key-on + block 4 + fnum high
        let mut peak = 0i32;
        for _ in 0..8820 {
            let (l, _r) = opl.render_frame();
            peak = peak.max(l.abs());
        }
        assert!(peak > 100, "raw-register tone silent, peak={peak}");
    }

    #[test]
    fn chip_renders_and_releases_a_tone() {
        let mut chip = Opl2::new(44100);
        let patch = ChannelPatch {
            modulator: OpPatch {
                mul: 1,
                tl: 63, // modulator silent → carrier ≈ pure sine
                attack: 15,
                release: 7,
                sustaining: true,
                ..Default::default()
            },
            carrier: OpPatch {
                mul: 1,
                tl: 0,
                attack: 15,
                release: 7,
                sustaining: true,
                ..Default::default()
            },
            ..Default::default()
        };
        chip.set_patch(0, &patch);
        chip.set_frequency(0, 0x2AE, 4);
        chip.key_on(0);

        let mut peak = 0i32;
        for _ in 0..4410 {
            let (l, _r) = chip.render_frame();
            peak = peak.max(l.abs());
        }
        assert!(peak > 100, "expected an audible tone, peak={peak}");

        chip.key_off(0);
        for _ in 0..44100 {
            chip.render_frame();
            if chip.channel_is_silent(0) {
                break;
            }
        }
        assert!(chip.channel_is_silent(0), "channel should release to silence");
    }

    #[test]
    fn non_standard_clock_scales_pitch() {
        // Half clock → half native rate → an octave down (pitch ∝ clock).
        let a = Opl2::new(44100);
        let b = Opl2::with_chip_clock(YM3812_CLOCK / 2, 44100);
        assert_eq!(a.native_rate(), 49716);
        assert_eq!(b.native_rate(), 49716 / 2);
    }

    #[test]
    fn opl3_clock_and_opl2_clock_agree_on_native_rate() {
        // OPL3 divides its 4× clock by 288 → the same 49716 Hz native rate.
        let opl2 = Opl2::with_chip_clock(YM3812_CLOCK, 44100);
        let opl3 = Opl2::with_native_rate((YMF262_CLOCK + 144) / 288, 44100);
        assert_eq!(opl2.native_rate(), opl3.native_rate());
        assert_eq!(opl3.native_rate(), 49716);
    }

    #[test]
    fn opl2_mode_ignores_second_bank_and_stays_9_channels() {
        let mut opl = Opl2::new(44100);
        assert_eq!(opl.channel_count(), 9);
        // Program a loud tone on a port-1 channel (channel 9). In OPL2 mode it
        // must stay silent (the second bank is dead).
        for (r, v) in [(0x43u8, 0x00u8), (0x63, 0xF0), (0xA0, 0x98), (0xB0, 0x31)] {
            opl.write_reg_high(r, v);
        }
        let mut peak = 0i32;
        for _ in 0..4410 {
            let (l, r) = opl.render_frame();
            peak = peak.max(l.abs()).max(r.abs());
        }
        assert_eq!(peak, 0, "port-1 channel sounded in OPL2 mode (peak={peak})");
    }

    #[test]
    fn opl3_fourop_group_sounds_and_consumes_secondary() {
        let mut opl = Opl2::new(44100);
        opl.write_reg_high(0x05, 0x01); // OPL3 enable
        opl.write_reg_high(0x04, 0x01); // 4-op for pair (0, 3)
        assert_eq!(fourop_primary_bit(0), Some(0));
        assert_eq!(fourop_secondary_bit(3), Some(0));

        // Serial 4-op (both CNT = 0): op1→op2→op3→op4, op4 the audible carrier.
        // Channel 0 holds op1 (reg 0x00 slot) / op2 (0x03 slot).
        for (r, v) in [
            (0x20u8, 0x01u8),
            (0x23, 0x01),
            (0x40, 0x1F),
            (0x43, 0x1F),
            (0x60, 0xF0),
            (0x63, 0xF0),
            (0x80, 0x00),
            (0x83, 0x00),
            (0xC0, 0x30), // CNT=0, panned both L+R (OPL3 pan bits)
            (0xA0, 0x98),
            (0xB0, 0x31), // key-on ch0 (op1/op2)
        ] {
            opl.write_reg(r, v);
        }
        // Channel 3 holds op3 (reg 0x28 slot) / op4 (0x2B slot).
        for (r, v) in [
            (0x28u8, 0x01u8),
            (0x2B, 0x01),
            (0x48, 0x1F),
            (0x4B, 0x00), // op4 loud
            (0x68, 0xF0),
            (0x6B, 0xF0),
            (0x88, 0x00),
            (0x8B, 0x00),
            (0xC3, 0x30), // CNT=0, panned both L+R
            (0xA3, 0x98),
            (0xB3, 0x31), // key-on ch3 (op3/op4)
        ] {
            opl.write_reg(r, v);
        }
        let mut peak = 0i32;
        for _ in 0..8820 {
            let (l, _r) = opl.render_frame();
            peak = peak.max(l.abs());
        }
        assert!(peak > 100, "4-op group silent (peak={peak})");
        assert_eq!(opl.channel_count(), 18);
    }

    #[test]
    fn fourop_pairing_map_is_correct() {
        // Primaries 0/1/2 (bank 0) and 9/10/11 (bank 1); secondaries n+3.
        for (prim, bit) in [(0, 0), (1, 1), (2, 2), (9, 3), (10, 4), (11, 5)] {
            assert_eq!(fourop_primary_bit(prim), Some(bit));
            assert_eq!(fourop_secondary_bit(prim + 3), Some(bit));
            assert_eq!(fourop_secondary_bit(prim), None);
            assert_eq!(fourop_primary_bit(prim + 3), None);
        }
        // Channels 6/7/8 (rhythm) and 15/16/17 are never 4-op.
        for ch in [6, 7, 8, 15, 16, 17] {
            assert_eq!(fourop_primary_bit(ch), None);
            assert_eq!(fourop_secondary_bit(ch), None);
        }
    }

    #[test]
    fn opl2_render_golden_hash() {
        // Pins the OPL2 (9-channel) render bit-for-bit. Two voices (WF0 + WF3),
        // feedback, additive routing, a key-off partway — exercises the OPL2
        // path. Baseline set when the operator output was recalibrated to the
        // hardware ±4084 exp scale + one's-complement negation (see
        // tables::EXP_OUT and waveform_output); it must stay fixed for OPL2
        // input now. (Same sequence as examples/opl2_hash.rs.)
        let mut opl = Opl2::new(44100);
        for (r, v) in [
            (0x20u8, 0x21u8), (0x23, 0x21), (0x40, 0x10), (0x43, 0x00),
            (0x60, 0xF2), (0x63, 0xF2), (0x80, 0x05), (0x83, 0x05),
            (0xC0, 0x06), (0xA0, 0x98), (0xB0, 0x31),
            (0x28, 0x22), (0x2B, 0x22), (0x48, 0x00), (0x4B, 0x00),
            (0x68, 0xF4), (0x6B, 0xF4), (0x88, 0x03), (0x8B, 0x03),
            (0xE8, 0x02), (0xEB, 0x03), (0xC8, 0x0A), (0xA8, 0x40), (0xB8, 0x25),
        ] {
            opl.write_reg(r, v);
        }
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for i in 0..44100u32 {
            if i == 33000 {
                opl.write_reg(0xB0, 0x11);
                opl.write_reg(0xB8, 0x05);
            }
            let (l, r) = opl.render_frame();
            for b in l.to_le_bytes().iter().chain(r.to_le_bytes().iter()) {
                h ^= *b as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        assert_eq!(h, 0xa17a_333a_f7d2_d119, "OPL2 render changed (hash {h:016x})");
    }

    #[test]
    fn opl3_second_bank_channel_sounds_and_pans() {
        let mut opl = Opl2::new(44100);
        opl.write_reg_high(0x05, 0x01); // OPL3 enable
        assert!(opl.opl3_enabled());
        assert_eq!(opl.channel_count(), 18);
        // Loud tone on channel 9 (port 1), panned hard right.
        opl.write_reg_high(0x40, 0x3F); // modulator silent
        opl.write_reg_high(0x43, 0x00); // carrier loud
        opl.write_reg_high(0x60, 0xF0);
        opl.write_reg_high(0x63, 0xF0);
        opl.write_reg_high(0x80, 0x0F);
        opl.write_reg_high(0x83, 0x0F);
        opl.write_reg_high(0xC0, 0x20); // right only (bit 5)
        opl.write_reg_high(0xA0, 0x98);
        opl.write_reg_high(0xB0, 0x31); // key-on, block 4
        let (mut lpk, mut rpk) = (0i32, 0i32);
        for _ in 0..8820 {
            let (l, r) = opl.render_frame();
            lpk = lpk.max(l.abs());
            rpk = rpk.max(r.abs());
        }
        assert!(rpk > 100, "port-1 channel silent on the right (peak={rpk})");
        assert_eq!(lpk, 0, "right-panned channel leaked to the left ({lpk})");
    }
}
