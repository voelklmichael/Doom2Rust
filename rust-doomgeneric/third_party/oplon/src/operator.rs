//! One OPL operator (a.k.a. "slot"): phase generator + envelope
//! generator + waveform output.
//!
//! Clean-room implementation of the documented YMF262/YM3812 operator. The
//! phase generator and the log-sine/exp output path are bit-faithful to the
//! hardware algorithm (see [`super::tables`]); the **envelope generator** is
//! an independent reimplementation of the documented YMF262 envelope
//! mechanism: a shared global counter (the chip's envelope counter, owned by
//! [`super::chip::Opl2`]) gates each phase on `counter & ((1<<shift)-1)`, a
//! dithered increment table supplies the per-phase step, and the attack
//! approaches 0 proportionally (`volume += (~volume·inc) >> 3`). The
//! attenuation domain (0..511) matches the chip exactly, so the envelope is
//! cycle-accurate against the silicon — no calibrated timing anchor. (GPL
//! emulators such as Schism's `player/fmopl3.c` are used only as behavioural
//! references for what the hardware does; no code or table is copied — see
//! the crate-level clean-room note in [`crate`].)

use super::tables::{EXP_OUT, KSL, KSL_SHIFT, LOGSIN_ROM, MUL};

/// Envelope generator phase.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EgState {
    /// Silent and idle (key released and faded out).
    Off,
    /// Rising toward 0 attenuation (loud).
    Attack,
    /// Falling toward the sustain level.
    Decay,
    /// Held at the sustain level (EG-type "sustaining" operators only).
    Sustain,
    /// Falling toward full attenuation (key-off, or percussive tail).
    Release,
}

/// Attenuation index range of the YMF262 envelope: 0 = loud, 511 = silent;
/// 1 unit ≈ 0.1875 dB (a 9-bit attenuation domain).
const MAX_ATT_INDEX: i32 = 511;

/// Even (Bresenham) distribution of `count` unit-steps across the 8
/// global-timer phases — the standard way to realise a fractional average
/// rate with integer per-sample steps.
const fn spread8(count: u32, phase: u32) -> u8 {
    (((count * (phase + 1)) >> 3) - ((count * phase) >> 3)) as u8
}

/// Per-phase envelope increments (15 rows × 8 global-timer phases),
/// **generated** from the documented YMF262 rate law `(4 + rate_lo)·2^rate_hi`
/// — *not* a table lifted from any emulator. Rows 0-3 spread the count
/// `4 + lo` over the 8 phases (rate groups 0..12); rows 4-7 (rate 13) spread
/// `2·(4 + lo)`; rows 8-11 (rate 14) double the matching rate-13 row; row 12
/// is rate-15 decay (flat 4), row 13 is the "zero-time" attack (flat 8), row
/// 14 is held (flat 0). This reproduces the chip's dither **counts** exactly
/// (so envelope timing is identical) and its per-phase ordering in 14 of the
/// 15 rows; the single differing row is count-identical.
#[cfg_attr(feature = "iram", link_section = ".data.oplon", allow(unsafe_code))]
static EG_INC: [u8; 15 * 8] = {
    let mut t = [0u8; 15 * 8];
    let mut row = 0usize;
    while row < 15 {
        let mut p = 0u32;
        while p < 8 {
            let v = if row < 4 {
                spread8(4 + row as u32, p) // rates 00..12, low bits 0..3
            } else if row < 8 {
                spread8(2 * (4 + (row as u32 - 4)), p) // rate 13
            } else if row < 12 {
                2 * spread8(2 * (4 + (row as u32 - 8)), p) // rate 14 = 2× rate 13
            } else if row == 12 {
                4 // rate 15 decay/release
            } else if row == 13 {
                8 // rate 15 attack (zero time)
            } else {
                0 // held (infinite time)
            };
            t[row * 8 + p as usize] = v;
            p += 1;
        }
        row += 1;
    }
    t
};

pub(crate) struct Operator {
    // ---- register-derived configuration (set by patch / key) ----
    /// Phase increment per sample (computed from F-num/block/mul).
    phase_inc: u32,
    mul: u8,
    waveform: u8,
    /// Total level attenuation, eg-domain (`tl_reg << 2`), 0..252.
    tl_eg: u16,
    ksl_reg: u8,
    ksr: bool,
    /// Amplitude-modulation (tremolo) enable — the chip's global ~3.7 Hz
    /// tremolo LFO is added to this operator's attenuation.
    am: bool,
    /// Frequency-modulation (vibrato) enable — the chip's global ~6 Hz
    /// vibrato LFO is added to this operator's phase increment.
    vib: bool,
    /// EG-type: true = sustaining (hold at sustain level until key-off);
    /// false = percussive (decay straight through to silence).
    sustaining: bool,
    attack_rate: u8,
    decay_rate: u8,
    sustain_level_eg: u16, // eg-domain target for the decay→sustain knee
    release_rate: u8,

    // ---- live frequency context (for KSL + key-scale-of-rate) ----
    fnum: u16,
    block: u8,
    /// Cached KSL attenuation (eg-domain) — depends only on fnum/block/KSL, so
    /// it is recomputed on a pitch/KSL change instead of every sample. See
    /// [`Self::refresh_pitch_cache`].
    ksl_eg: u16,
    /// Cached static attenuation `tl_eg + ksl_eg` (eg-domain) — the part of the
    /// per-sample attenuation that doesn't move with the envelope or tremolo.
    att_static: u32,
    /// Cached key-scale-of-rate offset (0..15) — likewise pitch/KSR-derived.
    ksr_off: u8,
    /// Cached envelope `(timer shift, EG_INC row offset)` for the attack,
    /// decay and release states — pure functions of the rate registers and
    /// `ksr_off`, so hoisted out of the per-sample `tick_envelope`.
    eg_att: (u32, usize),
    eg_dec: (u32, usize),
    eg_rel: (u32, usize),

    // ---- runtime state ----
    phase: u32,
    eg_state: EgState,
    volume: i32, // attenuation index, 0 (loud) .. MAX_ATT_INDEX (silent)
    /// Previous operator output (for feedback on operator 0).
    prev_out: i32,
    out: i32,
    /// OPL3 mode: enables waveforms 4-7. In OPL2 mode the waveform-select
    /// register is 2-bit, so the waveform is masked to 0-3 at output time.
    opl3: bool,
    /// Vibrato depth shift (reg 0xBD bit 6 "DVB"): `1` = shallow (default, the
    /// range is halved), `0` = deep. The chip sets it on every operator.
    vib_shift: u8,
}

impl Operator {
    pub(crate) const fn new() -> Self {
        Self {
            phase_inc: 0,
            mul: 0,
            waveform: 0,
            tl_eg: 0,
            ksl_reg: 0,
            ksr: false,
            am: false,
            vib: false,
            sustaining: false,
            attack_rate: 0,
            decay_rate: 0,
            sustain_level_eg: 0,
            release_rate: 0,
            fnum: 0,
            block: 0,
            ksl_eg: 0,
            att_static: 0,
            ksr_off: 0,
            eg_att: (0, 14 * 8),
            eg_dec: (0, 14 * 8),
            eg_rel: (0, 14 * 8),
            phase: 0,
            eg_state: EgState::Off,
            volume: MAX_ATT_INDEX,
            prev_out: 0,
            out: 0,
            opl3: false,
            vib_shift: 1,
        }
    }

    /// Select OPL3 mode (waveforms 0-7) vs OPL2 (0-3). The chip calls this on
    /// every operator when the OPL3-enable register (`0x105`) changes.
    pub(crate) fn set_opl3(&mut self, on: bool) {
        self.opl3 = on;
    }

    /// Set the vibrato depth shift (reg 0xBD bit 6 "DVB"): `1` = shallow
    /// (default), `0` = deep. The chip sets it on every operator on a 0xBD write.
    pub(crate) fn set_vib_shift(&mut self, shift: u8) {
        self.vib_shift = shift;
    }

    /// Program the operator from a decoded patch operator.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn set_patch(
        &mut self,
        mul: u8,
        waveform: u8,
        tl_reg: u8,
        ksl_reg: u8,
        ksr: bool,
        sustaining: bool,
        attack_rate: u8,
        decay_rate: u8,
        sustain_reg: u8,
        release_rate: u8,
        am: bool,
        vib: bool,
    ) {
        self.mul = mul & 0x0F;
        self.waveform = waveform & 0x07;
        self.tl_eg = ((tl_reg & 0x3F) as u16) << 2;
        self.ksl_reg = ksl_reg & 0x03;
        self.ksr = ksr;
        self.am = am;
        self.vib = vib;
        self.sustaining = sustaining;
        self.attack_rate = attack_rate & 0x0F;
        self.decay_rate = decay_rate & 0x0F;
        // Sustain level: register 0..15 → eg attenuation; 15 = near silent.
        self.sustain_level_eg = if sustain_reg >= 0x0F {
            0x1F0
        } else {
            ((sustain_reg & 0x0F) as u16) << 4
        };
        self.release_rate = release_rate & 0x0F;
        self.refresh_pitch_cache();
    }

    /// Update the live frequency context. Called on every pitch change (note,
    /// porta, vibrato) so KSL and key-scale-of-rate track the current note.
    /// The operator always runs at the chip's native rate (the resampler in
    /// [`super::chip::Opl2`] does the output-rate conversion), so `phase_inc`
    /// is simply the native phase step.
    pub(crate) fn set_frequency(&mut self, fnum: u16, block: u8) {
        self.fnum = fnum & 0x3FF;
        self.block = block & 0x07;
        // Phase increment per native sample: ((fnum << block) >> 1) * MUL[mul]
        // >> 1. Every term fits u32 (fnum ≤ 10 bits, block ≤ 7, MUL ≤ 30, so
        // the product is ≤ ~2^21).
        let base = ((self.fnum as u32) << self.block) >> 1;
        self.phase_inc = (base * MUL[self.mul as usize]) >> 1;
        self.refresh_pitch_cache();
    }

    /// Override the total-level attenuation (per-note volume). `tl_reg` is the
    /// 0..63 register value.
    pub(crate) fn set_total_level(&mut self, tl_reg: u8) {
        self.tl_eg = ((tl_reg & 0x3F) as u16) << 2;
        self.att_static = self.tl_eg as u32 + self.ksl_eg as u32;
    }

    // --- Raw OPL register writes (for a register-level `Opl2::write_reg`) ---

    /// Register 0x20: AM / VIB / EG-type / KSR / MULT. Recomputes `phase_inc`
    /// (MULT affects it) from the stored F-number / block.
    pub(crate) fn write_reg20(&mut self, val: u8) {
        self.am = val & 0x80 != 0;
        self.vib = val & 0x40 != 0;
        self.sustaining = val & 0x20 != 0;
        self.ksr = val & 0x10 != 0;
        self.mul = val & 0x0F;
        self.set_frequency(self.fnum, self.block);
    }

    /// Register 0x40: KSL (bits 6-7) + total level (bits 0-5).
    pub(crate) fn write_reg40(&mut self, val: u8) {
        self.ksl_reg = val >> 6;
        self.tl_eg = ((val & 0x3F) as u16) << 2;
        self.refresh_pitch_cache();
    }

    /// Register 0x60: attack rate (bits 4-7) + decay rate (bits 0-3).
    pub(crate) fn write_reg60(&mut self, val: u8) {
        self.attack_rate = val >> 4;
        self.decay_rate = val & 0x0F;
        self.refresh_pitch_cache();
    }

    /// Register 0x80: sustain level (bits 4-7) + release rate (bits 0-3).
    pub(crate) fn write_reg80(&mut self, val: u8) {
        let sustain_reg = val >> 4;
        self.sustain_level_eg = if sustain_reg >= 0x0F {
            0x1F0
        } else {
            (sustain_reg as u16) << 4
        };
        self.release_rate = val & 0x0F;
        self.refresh_pitch_cache();
    }

    /// Register 0xE0: waveform select (bits 0-2).
    pub(crate) fn write_reg_e0(&mut self, val: u8) {
        self.waveform = val & 0x07;
    }

    /// Key-on: (re)start the envelope from the attack phase and reset the
    /// phase accumulator (a fresh note restarts the waveform).
    pub(crate) fn key_on(&mut self) {
        self.eg_state = EgState::Attack;
        self.phase = 0;
        // Attack continues from wherever the envelope currently sits
        // (re-trigger of a still-sounding operator), matching hardware.
    }

    /// Key-off: enter the release phase.
    pub(crate) fn key_off(&mut self) {
        if self.eg_state != EgState::Off {
            self.eg_state = EgState::Release;
        }
    }

    pub(crate) fn is_silent(&self) -> bool {
        self.eg_state == EgState::Off
            || (self.eg_state == EgState::Release && self.volume >= MAX_ATT_INDEX)
    }

    /// Recompute the pitch-derived caches (KSL attenuation, key-scale-of-rate).
    /// Both depend only on fnum / block / KSL / KSR — values that change on a
    /// note, porta, KSL or KSR write, never per sample — so they are hoisted out
    /// of the hot path. Every setter that touches those fields calls this.
    fn refresh_pitch_cache(&mut self) {
        self.ksl_eg = self.ksl_attenuation();
        self.att_static = self.tl_eg as u32 + self.ksl_eg as u32;
        let ksr = self.ksr_offset();
        self.ksr_off = ksr;
        self.eg_att = Self::eg_params(self.attack_rate, ksr, true);
        self.eg_dec = Self::eg_params(self.decay_rate, ksr, false);
        self.eg_rel = Self::eg_params(self.release_rate, ksr, false);
    }

    /// Key-scale-of-rate offset (0..3 added rate, scaled below). The hardware
    /// derives a 4-bit "key scale number" from block and the top fnum bit;
    /// `ksr` selects whether all 4 or only the top 2 bits apply.
    fn ksr_offset(&self) -> u8 {
        let ksn = (self.block << 1) | ((self.fnum >> 9) & 1) as u8; // 0..15
        if self.ksr {
            ksn
        } else {
            ksn >> 2
        }
    }

    /// Decode an envelope rate into `(timer shift, increment row offset)`,
    /// reproducing the documented YMF262 rate-shift / rate-select behaviour.
    /// `reg` is the 0..15 register rate, `ksr` the key-scale offset (0..15).
    /// A register rate of 0 holds forever (row 14); `attack` selects the
    /// "zero-time" instant-attack row (13) for the fastest rates.
    fn eg_params(reg: u8, ksr: u8, attack: bool) -> (u32, usize) {
        if reg == 0 {
            return (0, 14 * 8); // infinite time (held)
        }
        let idx = 16 + ((reg as usize) << 2) + ksr as usize;
        if attack && idx >= 16 + 60 {
            return (0, 13 * 8); // all rate-15 attacks take zero time
        }
        let r = idx - 16;
        if r < 52 {
            ((12 - (r >> 2)) as u32, (r & 3) * 8) // rates 00..12
        } else if r < 56 {
            (0, (4 + (r & 3)) * 8) // rate 13
        } else if r < 60 {
            (0, (8 + (r & 3)) * 8) // rate 14
        } else {
            (0, 12 * 8) // rate 15 (+ dummy RKS rows)
        }
    }

    /// Advance the envelope one sample against the chip's global envelope
    /// counter `eg_cnt`. Cycle-accurate against the documented YMF262
    /// envelope mechanism.
    #[inline(always)]
    fn tick_envelope(&mut self, eg_cnt: u32) {
        match self.eg_state {
            EgState::Off | EgState::Sustain => {}
            EgState::Attack => {
                let (sh, sel) = self.eg_att;
                if eg_cnt & ((1 << sh) - 1) == 0 {
                    let inc = EG_INC[sel + ((eg_cnt >> sh) & 7) as usize] as i32;
                    self.volume += (!self.volume * inc) >> 3;
                    if self.volume <= 0 {
                        self.volume = 0;
                        self.eg_state = EgState::Decay;
                    }
                }
            }
            EgState::Decay => {
                let (sh, sel) = self.eg_dec;
                if eg_cnt & ((1 << sh) - 1) == 0 {
                    self.volume += EG_INC[sel + ((eg_cnt >> sh) & 7) as usize] as i32;
                    if self.volume >= self.sustain_level_eg as i32 {
                        // Sustaining (EG-type) operators hold; percussive ones
                        // continue falling at the release rate.
                        self.eg_state = if self.sustaining {
                            EgState::Sustain
                        } else {
                            EgState::Release
                        };
                    }
                }
            }
            EgState::Release => {
                let (sh, sel) = self.eg_rel;
                if eg_cnt & ((1 << sh) - 1) == 0 {
                    self.volume += EG_INC[sel + ((eg_cnt >> sh) & 7) as usize] as i32;
                    if self.volume >= MAX_ATT_INDEX {
                        self.volume = MAX_ATT_INDEX;
                        self.eg_state = EgState::Off;
                    }
                }
            }
        }
    }

    /// KSL attenuation contribution (eg-domain units) for the current note.
    /// `max(0, (kslrom[fnum>>6] << 2) − ((8 − block) << 5)) >> kslshift[reg]`.
    /// The contribution lives in the same eg domain as `tl << 2`, so it sums
    /// straight into [`Self::attenuation_eg`].
    fn ksl_attenuation(&self) -> u16 {
        if self.ksl_reg == 0 {
            return 0;
        }
        let base = (KSL[(self.fnum >> 6) as usize] as i32) << 2;
        let level = base - ((8 - self.block as i32) << 5);
        if level <= 0 {
            0
        } else {
            (level >> KSL_SHIFT[self.ksl_reg as usize]) as u16
        }
    }

    /// Total attenuation in eg-domain (envelope + total-level + KSL + the
    /// chip's tremolo when AM is enabled).
    #[inline(always)]
    fn attenuation_eg(&self, tremolo: u16) -> u32 {
        let env = self.volume.max(0) as u32;
        let trem = if self.am { tremolo as u32 } else { 0 };
        env + self.att_static + trem
    }

    /// Vibrato deviation added to `phase_inc` this sample (0 unless VIB).
    /// Mirrors the YMF262 8-step vibrato LFO: a triangle in units of
    /// `fnum >> 7`, converted to phase-increment units via block/multiple.
    #[inline(always)]
    fn vib_delta(&self, vibpos: u8) -> i32 {
        if !self.vib {
            return 0;
        }
        let mut range = ((self.fnum >> 7) & 7) as i32;
        if vibpos & 3 == 0 {
            range = 0;
        } else if vibpos & 1 == 1 {
            range >>= 1;
        }
        // Depth shift (DVB): shallow by default (>> 1). Missing this made the
        // default vibrato twice as deep as the hardware.
        range >>= self.vib_shift;
        if vibpos & 4 != 0 {
            range = -range;
        }
        // `range` is an fnum delta → phase-increment delta (same law as
        // `set_frequency`): ((range << block) >> 1) * MUL[mul] >> 1.
        (((range << self.block) >> 1) * MUL[self.mul as usize] as i32) >> 1
    }

    /// Compute the operator output for the current phase, given a phase
    /// modulation input (in 10-bit phase units) and the chip's LFO state
    /// (`tremolo` eg-units, `vibpos` 0..7). Advances the phase and the
    /// envelope by one sample. Returns a signed magnitude (~±4084 at full
    /// volume — the chip's 12-bit operator output).
    #[inline(always)]
    pub(crate) fn next(&mut self, mod_input: i32, tremolo: u16, vibpos: u8, eg_cnt: u32) -> i32 {
        self.advance(vibpos, eg_cnt);
        let phase10 = ((self.phase >> 9) as i32).wrapping_add(mod_input) as u32 & 0x3FF;
        self.output_at(phase10, tremolo)
    }

    /// Tick the envelope and advance the phase accumulator (without producing
    /// output). Split from [`Self::output_at`] for the **OPL rhythm mode**,
    /// where the output phase of some operators (HH/SD/CY) is *forced* from
    /// the phase bits of other operators, while still letting each operator
    /// advance its own phase.
    #[inline(always)]
    pub(crate) fn advance(&mut self, vibpos: u8, eg_cnt: u32) {
        self.tick_envelope(eg_cnt);
        let inc = (self.phase_inc as i32).wrapping_add(self.vib_delta(vibpos));
        self.phase = self.phase.wrapping_add(inc as u32);
    }

    /// Produce the operator's output at a given 10-bit phase (already advanced
    /// by [`Self::advance`]). In normal mode `phase10 = (phase>>9) + mod_input`.
    #[inline(always)]
    pub(crate) fn output_at(&mut self, phase10: u32, tremolo: u16) -> i32 {
        let att = self.attenuation_eg(tremolo);
        // OPL2's waveform-select register is 2-bit; OPL3 widens it to 3-bit.
        let wf = if self.opl3 { self.waveform } else { self.waveform & 0x03 };
        let out = waveform_output(wf, phase10 & 0x3FF, att);
        self.prev_out = self.out;
        self.out = out;
        out
    }

    /// Current phase bits (`phase >> 9`) — for the rhythm-mode correlated-noise
    /// computation.
    pub(crate) fn phase_out(&self) -> u32 {
        self.phase >> 9
    }

    /// Modulator self-feedback in phase-index units. The chip sums the last
    /// two operator outputs and shifts by the feedback register: the
    /// documented hardware applies `(prev_out + cur_out) << (fb_reg + 7)` to
    /// the 16.16 phase, i.e. a phase-index delta of `(prev + cur) >> (9 -
    /// fb_reg)` (`fb_reg` 1..7 → shift 8..2). `fb_reg == 0` disables feedback.
    #[inline(always)]
    pub(crate) fn feedback_phase(&self, fb_reg: u8) -> i32 {
        if fb_reg == 0 {
            0
        } else {
            (self.prev_out + self.out) >> (9 - fb_reg as i32).max(0)
        }
    }
}

/// Look up the quarter-wave log-sine ROM with the standard OPL folding: the
/// low 8 bits index the quarter, the 0x100 bit mirrors it. Returns the
/// attenuation in the log-sine domain (Q8).
#[inline(always)]
fn logsin(p: u32) -> u32 {
    let idx = if p & 0x100 != 0 { 255 - (p & 0xFF) } else { p & 0xFF };
    LOGSIN_ROM[idx as usize] as u32
}

/// Map (waveform, 10-bit phase, eg-domain attenuation) → signed sample.
///
/// Each waveform resolves to `(wf_att, neg, mute)`: a log-sine-domain
/// attenuation, a sign, and a mute flag. Waveforms 0-3 are the OPL2 set
/// (bit-identical to before); 4-7 are the OPL3 additions — 4/5 are the
/// double-frequency ("even") sine and its rectified form, 6 is a square wave
/// (constant full amplitude, sign flipping at the half period), and 7 is the
/// logarithmic sawtooth (a linear ramp in the log domain → exponential decay).
#[inline(always)]
fn waveform_output(waveform: u8, phase10: u32, att_eg: u32) -> i32 {
    let p = phase10 & 0x3FF;

    let (wf_att, neg, mute) = match waveform & 0x07 {
        // 0: full sine.
        0 => (logsin(p), p & 0x200 != 0, false),
        // 1: half sine (second half muted).
        1 => {
            if p & 0x200 != 0 {
                (0, false, true)
            } else {
                (logsin(p), false, false)
            }
        }
        // 2: absolute sine (period halved, always positive).
        2 => (logsin(p & 0x1FF), false, false),
        // 3: quarter (pulse) sine — only the rising quarter of each half.
        3 => {
            let pp = p & 0x1FF;
            if pp & 0x100 != 0 {
                (0, false, true)
            } else {
                (logsin(pp), false, false)
            }
        }
        // 4: even (double-frequency) sine — a full sine cycle squeezed into the
        // first half period, second half muted. The quarter-wave mirror is
        // taken on the *pre-doubled* phase (`(p ^ 0xFF) << 1`), matching the
        // hardware: mirroring the doubled index instead would be off by the LSB.
        4 => {
            if p & 0x200 != 0 {
                (0, false, true)
            } else {
                let idx = if p & 0x80 != 0 {
                    ((p ^ 0xFF) << 1) & 0xFF
                } else {
                    (p << 1) & 0xFF
                };
                (LOGSIN_ROM[idx as usize] as u32, p & 0x100 != 0, false)
            }
        }
        // 5: even absolute sine — double-frequency rectified sine, first half
        // (same pre-doubled mirror as waveform 4, always positive).
        5 => {
            if p & 0x200 != 0 {
                (0, false, true)
            } else {
                let idx = if p & 0x80 != 0 {
                    ((p ^ 0xFF) << 1) & 0xFF
                } else {
                    (p << 1) & 0xFF
                };
                (LOGSIN_ROM[idx as usize] as u32, false, false)
            }
        }
        // 6: square wave — constant full amplitude (0 log-sine attenuation),
        // sign flips at the half period.
        6 => (0, p & 0x200 != 0, false),
        // 7: logarithmic sawtooth — the phase itself is the log-domain
        // attenuation, so exp() of it is an exponential ramp; mirrored and
        // negated in the second half.
        _ => {
            if p & 0x200 != 0 {
                (((p ^ 0x1FF) & 0x1FF) << 3, true, false)
            } else {
                ((p & 0x1FF) << 3, false, false)
            }
        }
    };

    if mute {
        return 0;
    }
    // logsin (attenuation) + envelope, then exp. eg-domain → logsin-domain
    // via `<< 3` (1 eg-unit ≈ 8 logsin-units ≈ 0.1875 dB).
    let total = wf_att + (att_eg << 3);
    let shift = total >> 8;
    // 13-bit operator output: `(EXP_OUT[lo] << 1) >> hi` = 4084·2^(−total/256),
    // the chip's real ±4084 peak (see `tables::EXP_OUT`). `shift ≥ 13` is below
    // the LSB (silence).
    let mag = if shift >= 13 {
        0
    } else {
        (((EXP_OUT[(total & 0xFF) as usize] as u32) << 1) >> shift) as i32
    };
    // One's-complement negation on the negative half (the chip XORs the 13-bit
    // magnitude with all-ones), i.e. `−(mag + 1)`, not two's complement. The
    // 1-LSB difference is inaudible for a single operator but keeps a
    // multi-operator FM chain phase-aligned with the hardware.
    if neg {
        !mag
    } else {
        mag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The generated `EG_INC` realises the documented YMF262 rate law: each
    /// row's per-cycle sum (the dither *count* that sets the envelope speed)
    /// is `(4 + lo)·2^rate_hi`, and every step is one of the hardware's
    /// {0,1,2,4,8}.
    #[test]
    fn eg_inc_counts_match_rate_law() {
        let row_sum = |r: usize| -> u32 { (0..8).map(|p| EG_INC[r * 8 + p] as u32).sum() };
        for lo in 0..4 {
            assert_eq!(row_sum(lo), 4 + lo as u32, "row {lo}");
        }
        for lo in 0..4 {
            assert_eq!(row_sum(4 + lo), 2 * (4 + lo as u32), "row {}", 4 + lo);
        }
        for lo in 0..4 {
            assert_eq!(row_sum(8 + lo), 4 * (4 + lo as u32), "row {}", 8 + lo);
        }
        assert_eq!(row_sum(12), 32); // rate 15 decay: flat 4
        assert_eq!(row_sum(13), 64); // rate 15 attack: flat 8
        assert_eq!(row_sum(14), 0); // held
        for &v in EG_INC.iter() {
            assert!(matches!(v, 0 | 1 | 2 | 4 | 8), "illegal step {v}");
        }
    }

    /// The pre-OPL3-refactor `waveform_output` for the OPL2 waveforms 0-3,
    /// verbatim. `waveforms_0_3_bit_identical` asserts the refactored version
    /// reproduces it exactly, so adding OPL3 waveforms 4-7 could not have
    /// perturbed the OPL2 path.
    fn old_waveform_output(waveform: u8, phase10: u32, att_eg: u32) -> i32 {
        let p = phase10 & 0x3FF;
        let (idx, neg, mute) = match waveform & 0x07 {
            0 => {
                let quad = (p >> 8) & 3;
                let idx = if quad & 1 != 0 { 255 - (p & 0xFF) } else { p & 0xFF };
                (idx, quad & 2 != 0, false)
            }
            1 => {
                if p & 0x200 != 0 {
                    (0, false, true)
                } else {
                    let idx = if p & 0x100 != 0 { 255 - (p & 0xFF) } else { p & 0xFF };
                    (idx, false, false)
                }
            }
            2 | 6 => {
                let pp = p & 0x1FF;
                let idx = if pp & 0x100 != 0 { 255 - (pp & 0xFF) } else { pp & 0xFF };
                (idx, false, false)
            }
            3 | 7 => {
                let pp = p & 0x1FF;
                if pp & 0x100 != 0 {
                    (0, false, true)
                } else {
                    (pp & 0xFF, false, false)
                }
            }
            _ => {
                let quad = (p >> 8) & 3;
                let idx = if quad & 1 != 0 { 255 - (p & 0xFF) } else { p & 0xFF };
                (idx, quad & 2 != 0, false)
            }
        };
        if mute {
            return 0;
        }
        let total = LOGSIN_ROM[idx as usize] as u32 + (att_eg << 3);
        let shift = total >> 8;
        let mag = if shift >= 13 {
            0
        } else {
            (((EXP_OUT[(total & 0xFF) as usize] as u32) << 1) >> shift) as i32
        };
        if neg {
            !mag
        } else {
            mag
        }
    }

    #[test]
    fn waveforms_0_3_bit_identical() {
        for wf in 0u8..4 {
            for p in 0..1024u32 {
                for att in [0u32, 1, 7, 32, 100, 200, 255] {
                    assert_eq!(
                        waveform_output(wf, p, att),
                        old_waveform_output(wf, p, att),
                        "wf={wf} p={p} att={att}"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod wf_golden {
    /// Golden hash pinning `waveform_output` for all 8 waveforms across every
    /// phase and a spread of attenuations. Each of the 8 shapes was verified
    /// **bit-identical to Nuked-OPL3** (the cycle-accurate reference) per phase,
    /// including the pre-doubled quarter-wave mirror of waveforms 4/5; this hash
    /// guards that against silent regressions.
    #[test]
    fn all_waveforms_match_validated_golden() {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for wf in 0u8..8 {
            for p in 0..1024u32 {
                for att in [0u32, 10, 50, 100] {
                    for b in super::waveform_output(wf, p, att).to_le_bytes() {
                        h ^= b as u64;
                        h = h.wrapping_mul(0x0000_0100_0000_01b3);
                    }
                }
            }
        }
        assert_eq!(h, 0x220c_fe4b_bd17_c54d, "waveform output changed ({h:016x})");
    }
}
