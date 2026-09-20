//! Music the way DOS Doom played it on an `AdLib` / Sound Blaster: the MUS score
//! drives an OPL2 FM chip, programmed from the WAD's `GENMIDI` instrument
//! patches. The chip itself is the `oplon` crate; this module is the driver in
//! front of it (what `i_oplmusic.c` is in Chocolate Doom): it keeps MIDI-style
//! channel state, hands notes to the chip's nine voices, and turns notes and
//! volumes into register writes.

use crate::genmidi::{self, GenMidi, Instrument};
use crate::mus::{Event, Sequencer, Song, PERCUSSION_CHANNEL, TICKS_PER_SECOND};
use oplon::Opl2;

const NUM_VOICES: usize = 9;
const NUM_CHANNELS: usize = 16;
/// Register offset of each voice's modulator; the carrier is 3 further on.
const OPERATOR_OFFSETS: [u8; NUM_VOICES] = [0, 1, 2, 8, 9, 10, 16, 17, 18];
const REG_TREMOLO: u8 = 0x20;
const REG_LEVEL: u8 = 0x40;
const REG_ATTACK: u8 = 0x60;
const REG_SUSTAIN: u8 = 0x80;
const REG_FREQ_LOW: u8 = 0xa0;
const REG_KEY_ON: u8 = 0xb0;
const REG_FEEDBACK: u8 = 0xc0;
const REG_WAVEFORM: u8 = 0xe0;
const KEY_ON: u8 = 0x20;
/// The chip's output is quiet next to the sound effects (about a quarter of
/// full scale for a loud passage), so it is raised to match.
const MUSIC_GAIN: i32 = 2;

/// Steps per octave in the pitch table: 64 per semitone, the resolution of the
/// MUS pitch bend (128 steps = two semitones).
const STEPS_PER_OCTAVE: usize = 12 * 64;
/// Highest pitch position the table is asked for: note 127 plus a full bend.
const MAX_POSITION: i32 = 128 * 64 + 128;

/// F-number of MIDI note 0 (8.18 Hz) at block 0, times 2^40. For a note the
/// F-number is `f * 2^20 / (3579545 / 72)`; the chip's block halves it.
const NOTE0_FNUM_Q40: u64 = 189_598_375_178_259;
/// `2^(1/768)` times 2^40: one step of the pitch table.
const STEP_Q40: u64 = 1_100_504_423_883;

/// `2^(i / 768)` times [`NOTE0_FNUM_Q40`], by repeated multiplication. Rounding
/// error over 768 steps is far below one cent.
const PITCH_TABLE: [u64; STEPS_PER_OCTAVE] = {
    let mut table = [0u64; STEPS_PER_OCTAVE];
    table[0] = NOTE0_FNUM_Q40;
    let mut i = 1;
    while i < STEPS_PER_OCTAVE {
        table[i] = ((table[i - 1] as u128 * STEP_Q40 as u128) >> 40) as u64;
        i += 1;
    }
    table
};

/// F-number and block (octave) for a pitch given in 1/64 semitones from MIDI
/// note 0.
fn f_number_and_block(position: i32) -> (u16, u8) {
    let position = position.clamp(0, MAX_POSITION) as usize;
    let at_block_0 = PITCH_TABLE[position % STEPS_PER_OCTAVE] << (position / STEPS_PER_OCTAVE);
    let mut block = 0;
    while block < 7 && (at_block_0 >> (40 + block)) > 1023 {
        block += 1;
    }
    let f_number = (at_block_0 >> (40 + block)).min(1023) as u16;
    (f_number, block as u8)
}

/// Attenuation in 0.75 dB steps (the chip's total level unit) of a MIDI volume
/// or velocity, for a `40 * log10(v / 127)` dB curve; 63 is silence.
const ATTENUATION: [u8; 128] = [
    63, 63, 63, 63, 63, 63, 63, 63, 63, 61, 59, 57, 55, 53, 51, 49, //
    48, 47, 45, 44, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 33, //
    32, 31, 31, 30, 29, 29, 28, 27, 27, 26, 26, 25, 25, 24, 24, 23, //
    23, 22, 22, 21, 21, 20, 20, 19, 19, 19, 18, 18, 17, 17, 17, 16, //
    16, 16, 15, 15, 14, 14, 14, 13, 13, 13, 13, 12, 12, 12, 11, 11, //
    11, 10, 10, 10, 10, 9, 9, 9, 8, 8, 8, 8, 7, 7, 7, 7, //
    6, 6, 6, 6, 6, 5, 5, 5, 5, 4, 4, 4, 4, 4, 3, 3, //
    3, 3, 3, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 0, //
];

fn attenuation(volume: u8) -> u32 {
    u32::from(ATTENUATION[usize::from(volume.min(127))])
}

#[derive(Copy, Clone)]
struct Channel {
    program: u8,
    volume: u8,
    /// Volume of the last note on this channel, for notes that give none.
    note_volume: u8,
    /// Pitch bend in 1/64 semitones, -128..=127.
    bend: i32,
}

impl Channel {
    const RESET: Self = Self {
        program: 0,
        volume: 100,
        note_volume: 127,
        bend: 0,
    };
}

/// A note held by one of the chip's voices.
#[derive(Copy, Clone)]
struct Playing {
    channel: u8,
    key: u8,
    instrument: Instrument,
    /// 0 for the first voice of an instrument, 1 for the second.
    part: u8,
    note_volume: u8,
    /// Pitch without bend, in 1/64 semitones.
    position: i32,
    /// Sequence number, to tell the older of two notes.
    started: u32,
}

#[derive(Copy, Clone)]
struct Voice {
    playing: Option<Playing>,
    /// When the voice was last released; the one released longest ago is reused
    /// first, so a note's release tail is cut as late as possible.
    released: u32,
    f_number: u16,
    block: u8,
}

pub struct OplDriver {
    opl: Opl2,
    bank: GenMidi,
    output_rate: u32,
    channels: [Channel; NUM_CHANNELS],
    voices: [Voice; NUM_VOICES],
    /// Counts note starts and releases.
    clock: u32,
    /// 0..=127
    music_volume: u8,
}

impl OplDriver {
    pub fn new(bank: GenMidi, output_rate: u32) -> Self {
        let mut driver = Self {
            opl: Opl2::new(output_rate),
            bank,
            output_rate,
            channels: [Channel::RESET; NUM_CHANNELS],
            voices: [Voice {
                playing: None,
                released: 0,
                f_number: 0,
                block: 0,
            }; NUM_VOICES],
            clock: 0,
            music_volume: 127,
        };
        driver.reset();
        driver
    }

    /// Silences everything and restores the power-on channel state.
    pub fn reset(&mut self) {
        self.opl = Opl2::new(self.output_rate);
        // Enable the waveform-select register.
        self.opl.write_reg(0x01, 0x20);
        self.channels = [Channel::RESET; NUM_CHANNELS];
        for voice in &mut self.voices {
            voice.playing = None;
        }
    }

    pub fn set_music_volume(&mut self, volume: i32) {
        self.music_volume = volume.clamp(0, 127) as u8;
        for v in 0..NUM_VOICES {
            if self.voices[v].playing.is_some() {
                self.write_volume(v);
            }
        }
    }

    /// True while any voice still makes sound, including release tails.
    pub fn is_sounding(&self) -> bool {
        self.opl.any_active()
    }

    /// One stereo frame of chip output.
    pub fn render_frame(&mut self) -> (i32, i32) {
        self.opl.render_frame()
    }

    pub fn event(&mut self, event: Event) {
        match event {
            Event::NoteOff { channel, note } => self.note_off(channel, note),
            Event::NoteOn {
                channel,
                note,
                volume,
            } => {
                let ch = &mut self.channels[usize::from(channel)];
                if let Some(volume) = volume {
                    ch.note_volume = volume;
                }
                let volume = ch.note_volume;
                self.note_on(channel, note, volume);
            }
            Event::PitchBend { channel, value } => {
                self.channels[usize::from(channel)].bend = i32::from(value) - 128;
                for v in 0..NUM_VOICES {
                    if self.voices[v].playing.is_some_and(|p| p.channel == channel) {
                        self.write_frequency(v, true);
                    }
                }
            }
            Event::System {
                channel,
                controller,
            } => match controller {
                // All sounds off, all notes off.
                10 | 11 => self.release_channel(channel),
                // Reset all controllers.
                14 => {
                    self.channels[usize::from(channel)] = Channel {
                        program: self.channels[usize::from(channel)].program,
                        ..Channel::RESET
                    };
                }
                _ => {}
            },
            Event::Controller {
                channel,
                controller,
                value,
            } => match controller {
                0 => self.channels[usize::from(channel)].program = value,
                3 => {
                    self.channels[usize::from(channel)].volume = value;
                    for v in 0..NUM_VOICES {
                        if self.voices[v].playing.is_some_and(|p| p.channel == channel) {
                            self.write_volume(v);
                        }
                    }
                }
                _ => {}
            },
            Event::EndOfMeasure | Event::ScoreEnd => {}
        }
    }

    fn note_on(&mut self, channel: u8, key: u8, volume: u8) {
        let instrument = if channel == PERCUSSION_CHANNEL {
            match self.bank.percussion(key) {
                Some(instrument) => *instrument,
                None => return,
            }
        } else {
            *self
                .bank
                .melodic(self.channels[usize::from(channel)].program)
        };
        let first = self.start_voice(channel, key, instrument, 0, volume, None);
        if instrument.flags & genmidi::FLAG_TWO_VOICE != 0 {
            self.start_voice(channel, key, instrument, 1, volume, first);
        }
    }

    fn start_voice(
        &mut self,
        channel: u8,
        key: u8,
        instrument: Instrument,
        part: u8,
        note_volume: u8,
        keep: Option<usize>,
    ) -> Option<usize> {
        let v = self.allocate_voice(keep)?;
        let patch = instrument.voices[usize::from(part)];
        let mut note = if instrument.flags & genmidi::FLAG_FIXED != 0 {
            i32::from(instrument.fixed_note)
        } else if channel == PERCUSSION_CHANNEL {
            60
        } else {
            i32::from(key)
        };
        note += i32::from(patch.base_note_offset);
        while note < 0 {
            note += 12;
        }
        while note > 95 {
            note -= 12;
        }
        let mut position = note * 64;
        if part == 1 {
            // The second voice is detuned in 1/32 semitones around 128.
            position += (i32::from(instrument.fine_tuning) / 2 - 64) * 2;
        }
        self.clock += 1;
        self.voices[v].playing = Some(Playing {
            channel,
            key,
            instrument,
            part,
            note_volume,
            position,
            started: self.clock,
        });
        // A voice that was still ringing must be silent before it is reprogrammed.
        self.write_key(v, false);
        self.write_instrument(v);
        self.write_volume(v);
        self.write_frequency(v, true);
        Some(v)
    }

    /// A free voice (the one released longest ago), else the voice to steal:
    /// second voices go first, then the highest channel number.
    fn allocate_voice(&mut self, keep: Option<usize>) -> Option<usize> {
        let free = (0..NUM_VOICES)
            .filter(|&v| self.voices[v].playing.is_none())
            .min_by_key(|&v| self.voices[v].released);
        if free.is_some() {
            return free;
        }
        let mut victim: Option<(usize, Playing)> = None;
        for v in (0..NUM_VOICES).filter(|&v| Some(v) != keep) {
            let Some(p) = self.voices[v].playing else {
                continue;
            };
            let better = match victim {
                None => true,
                Some((_, best)) => {
                    (p.part, p.channel, core::cmp::Reverse(p.started))
                        > (best.part, best.channel, core::cmp::Reverse(best.started))
                }
            };
            if better {
                victim = Some((v, p));
            }
        }
        let (v, _) = victim?;
        self.release_voice(v);
        Some(v)
    }

    fn note_off(&mut self, channel: u8, key: u8) {
        for v in 0..NUM_VOICES {
            if self.voices[v]
                .playing
                .is_some_and(|p| p.channel == channel && p.key == key)
            {
                self.release_voice(v);
            }
        }
    }

    fn release_channel(&mut self, channel: u8) {
        for v in 0..NUM_VOICES {
            if self.voices[v].playing.is_some_and(|p| p.channel == channel) {
                self.release_voice(v);
            }
        }
    }

    fn release_voice(&mut self, v: usize) {
        self.write_key(v, false);
        self.clock += 1;
        self.voices[v].playing = None;
        self.voices[v].released = self.clock;
    }

    fn operator_offsets(v: usize) -> (u8, u8) {
        let modulator = OPERATOR_OFFSETS[v];
        (modulator, modulator + 3)
    }

    /// Programs the voice's operators with its instrument patch. The carrier's
    /// level is left to [`write_volume`](Self::write_volume).
    fn write_instrument(&mut self, v: usize) {
        let Some(p) = self.voices[v].playing else {
            return;
        };
        let patch = p.instrument.voices[usize::from(p.part)];
        let (m, c) = Self::operator_offsets(v);
        let opl = &mut self.opl;
        for (offset, op) in [(m, patch.modulator), (c, patch.carrier)] {
            opl.write_reg(REG_TREMOLO + offset, op.tremolo);
            opl.write_reg(REG_ATTACK + offset, op.attack);
            opl.write_reg(REG_SUSTAIN + offset, op.sustain);
            opl.write_reg(REG_WAVEFORM + offset, op.waveform);
        }
        opl.write_reg(
            REG_LEVEL + m,
            (patch.modulator.scale & 0xc0) | (patch.modulator.level & 0x3f),
        );
        opl.write_reg(REG_FEEDBACK + v as u8, patch.feedback);
    }

    /// Sets the levels from the note velocity, the channel volume and the music
    /// volume. Only the carrier is attenuated, except in additive mode, where
    /// the modulator is a second output and follows it.
    fn write_volume(&mut self, v: usize) {
        let Some(p) = self.voices[v].playing else {
            return;
        };
        let patch = p.instrument.voices[usize::from(p.part)];
        let (m, c) = Self::operator_offsets(v);
        let attenuation = attenuation(p.note_volume)
            + attenuation(self.channels[usize::from(p.channel)].volume)
            + attenuation(self.music_volume);
        let level = |op: genmidi::Operator| {
            (op.scale & 0xc0) | (u32::from(op.level & 0x3f) + attenuation).min(63) as u8
        };
        self.opl.write_reg(REG_LEVEL + c, level(patch.carrier));
        if patch.feedback & 1 != 0 {
            self.opl.write_reg(REG_LEVEL + m, level(patch.modulator));
        }
    }

    /// Writes the voice's pitch, with the key held if `key_on`.
    fn write_frequency(&mut self, v: usize, key_on: bool) {
        let Some(p) = self.voices[v].playing else {
            return;
        };
        let position = p.position + self.channels[usize::from(p.channel)].bend;
        let (f_number, block) = f_number_and_block(position);
        self.voices[v].f_number = f_number;
        self.voices[v].block = block;
        self.opl.write_reg(REG_FREQ_LOW + v as u8, f_number as u8);
        self.write_key(v, key_on);
    }

    fn write_key(&mut self, v: usize, on: bool) {
        let voice = self.voices[v];
        let high = (voice.block << 2) | (voice.f_number >> 8) as u8;
        self.opl
            .write_reg(REG_KEY_ON + v as u8, if on { KEY_ON | high } else { high });
    }
}

/// Plays a registered song through an [`OplDriver`], on the output's clock.
pub struct MusicPlayer {
    driver: OplDriver,
    song: Option<Song>,
    sequencer: Sequencer,
    output_rate: u32,
    /// Fraction of a MUS tick elapsed, in output frames.
    tick_phase: u32,
    playing: bool,
    paused: bool,
    looping: bool,
}

impl MusicPlayer {
    pub fn new(bank: GenMidi, output_rate: u32) -> Self {
        Self {
            driver: OplDriver::new(bank, output_rate),
            song: None,
            sequencer: Sequencer::new(),
            output_rate,
            tick_phase: 0,
            playing: false,
            paused: false,
            looping: false,
        }
    }

    /// Loads a MUS lump. Returns `false` if it is not one.
    pub fn register(&mut self, lump: &[u8]) -> bool {
        self.stop();
        self.song = Song::parse(lump);
        self.song.is_some()
    }

    pub fn unregister(&mut self) {
        self.stop();
        self.song = None;
    }

    pub fn play(&mut self, looping: bool) {
        if self.song.is_none() {
            return;
        }
        self.driver.reset();
        self.sequencer = Sequencer::new();
        self.tick_phase = 0;
        self.playing = true;
        self.paused = false;
        self.looping = looping;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.paused = false;
        self.driver.reset();
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn set_volume(&mut self, volume: i32) {
        self.driver.set_music_volume(volume);
    }

    /// Adds the next `acc.len() / 2` frames of music to `acc` (interleaved
    /// stereo, 32-bit so it can be summed with the effects before clipping).
    #[allow(clippy::chunks_exact_to_as_chunks)] // as_chunks needs a newer toolchain than the ESP one
    pub fn render_add(&mut self, acc: &mut [i32]) {
        if self.paused || !(self.playing || self.driver.is_sounding()) {
            return;
        }
        for frame in acc.chunks_exact_mut(2) {
            if self.playing {
                self.tick_phase += TICKS_PER_SECOND;
                if self.tick_phase >= self.output_rate {
                    self.tick_phase -= self.output_rate;
                    self.tick();
                }
            }
            let (left, right) = self.driver.render_frame();
            frame[0] += left * MUSIC_GAIN;
            frame[1] += right * MUSIC_GAIN;
        }
    }

    fn tick(&mut self) {
        let Some(song) = self.song.as_ref() else {
            self.playing = false;
            return;
        };
        let driver = &mut self.driver;
        self.sequencer.tick(song, &mut |event| driver.event(event));
        if self.sequencer.ended() {
            if self.looping {
                // Let the last notes ring out into the repeat, as the score
                // end usually follows the last note off anyway.
                self.sequencer = Sequencer::new();
                self.tick_phase = 0;
            } else {
                self.playing = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genmidi::tests::sample_lump;
    use alloc::vec;
    use alloc::vec::Vec;

    fn driver() -> OplDriver {
        OplDriver::new(GenMidi::parse(&sample_lump()).unwrap(), 44100)
    }

    #[test]
    fn pitch_table_is_equal_tempered() {
        // A4 = 440 Hz: note 69. F-number * 49715.9 / 2^(20 - block) is the pitch.
        let hz = |position: i32| {
            let (f, b) = f_number_and_block(position);
            f64::from(f) * (3_579_545.0 / 72.0) / f64::from(1u32 << (20 - u32::from(b)))
        };
        assert!((hz(69 * 64) - 440.0).abs() < 0.5, "{}", hz(69 * 64));
        assert!((hz(60 * 64) - 261.63).abs() < 0.4);
        assert!((hz(81 * 64) - 880.0).abs() < 1.0);
        // A semitone is a factor 2^(1/12), a bend step 1/64 of that.
        let cents = |a: f64, b: f64| 1200.0 * (b / a).log2();
        assert!((cents(hz(60 * 64), hz(61 * 64)) - 100.0).abs() < 3.0);
        assert!((cents(hz(60 * 64), hz(60 * 64 + 128)) - 200.0).abs() < 3.0);
        // The table wraps into the next block exactly once per octave.
        for position in (0..127 * 64).step_by(37) {
            let (f, _) = f_number_and_block(position);
            assert!(f <= 1023);
        }
        assert_eq!(f_number_and_block(-500), f_number_and_block(0));
    }

    #[test]
    fn attenuation_falls_with_volume() {
        assert_eq!(attenuation(127), 0);
        assert_eq!(attenuation(0), 63);
        assert!(ATTENUATION.windows(2).all(|w| w[0] >= w[1]));
        // Half velocity is about -12 dB = 16 steps of 0.75 dB.
        assert!((15..=17).contains(&attenuation(64)));
    }

    #[test]
    fn notes_take_and_release_voices() {
        let mut d = driver();
        d.event(Event::NoteOn {
            channel: 0,
            note: 60,
            volume: Some(100),
        });
        assert_eq!(d.voices.iter().filter(|v| v.playing.is_some()).count(), 1);
        d.event(Event::NoteOff {
            channel: 0,
            note: 60,
        });
        assert_eq!(d.voices.iter().filter(|v| v.playing.is_some()).count(), 0);
        // The freed voice is the last one to be reused.
        d.event(Event::NoteOn {
            channel: 0,
            note: 62,
            volume: None,
        });
        assert!(d.voices[0].playing.is_none());
    }

    #[test]
    fn a_full_chip_steals_the_highest_channel_first() {
        let mut d = driver();
        // Instrument 0 of the sample bank is single-voice (flags 0).
        for ch in 0..9 {
            d.event(Event::NoteOn {
                channel: ch,
                note: 60,
                volume: Some(100),
            });
        }
        d.event(Event::NoteOn {
            channel: 3,
            note: 64,
            volume: Some(100),
        });
        let channels: Vec<u8> = d
            .voices
            .iter()
            .map(|v| v.playing.expect("all nine voices busy").channel)
            .collect();
        assert!(!channels.contains(&8), "channel 8 was the lowest priority");
        assert_eq!(channels.iter().filter(|&&c| c == 3).count(), 2);
    }

    #[test]
    fn two_voice_instruments_use_two_voices() {
        let mut d = driver();
        // Sample-bank instruments with odd numbers have flag 1 (fixed note);
        // make one two-voice by choosing program 0 with the flag patched.
        let mut lump = sample_lump();
        lump[8] = 0x04; // instrument 0: two voices
        d.bank = GenMidi::parse(&lump).unwrap();
        d.event(Event::NoteOn {
            channel: 0,
            note: 60,
            volume: Some(100),
        });
        let parts: Vec<u8> = d
            .voices
            .iter()
            .filter_map(|v| v.playing.map(|p| p.part))
            .collect();
        assert_eq!(parts, [0, 1]);
    }

    /// Renders `frames` frames and returns the left channel.
    fn render(d: &mut OplDriver, frames: usize) -> Vec<i32> {
        (0..frames).map(|_| d.render_frame().0).collect()
    }

    /// A bank whose every instrument is a plain sine (modulator silent).
    fn sine_bank() -> GenMidi {
        let mut lump = sample_lump();
        for n in 0..genmidi::NUM_INSTRUMENTS {
            let base = 8 + n * 36;
            lump[base..base + 4].copy_from_slice(&[0, 0, 128, 60]);
            let voice = &mut lump[base + 4..base + 20];
            voice.fill(0);
            // modulator: silent
            voice[5] = 63;
            voice[1] = 0xf0;
            // feedback 0, FM connection
            // carrier: multiplier 1, sustained, instant attack, no decay
            voice[7] = 0x21;
            voice[8] = 0xf0;
            voice[9] = 0x0f;
            voice[10] = 0;
            voice[12] = 0;
            let second = &mut lump[base + 20..base + 36];
            second.fill(0);
            second[5] = 63;
            second[12] = 63;
        }
        GenMidi::parse(&lump).unwrap()
    }

    fn crossings_per_second(samples: &[i32], rate: usize) -> f64 {
        let crossings = samples.windows(2).filter(|w| w[0] < 0 && w[1] >= 0).count();
        crossings as f64 * rate as f64 / samples.len() as f64
    }

    #[test]
    fn a_note_sounds_at_its_pitch() {
        for (note, hz) in [(69u8, 440.0), (57, 220.0), (81, 880.0), (48, 130.81)] {
            let mut d = OplDriver::new(sine_bank(), 44100);
            d.event(Event::NoteOn {
                channel: 0,
                note,
                volume: Some(127),
            });
            let _attack = render(&mut d, 2000);
            let samples = render(&mut d, 44100);
            let measured = crossings_per_second(&samples, 44100);
            assert!(
                (measured - hz).abs() < hz * 0.01,
                "note {note}: {measured} Hz, wanted {hz}"
            );
        }
    }

    #[test]
    fn pitch_bend_shifts_the_note() {
        let mut d = OplDriver::new(sine_bank(), 44100);
        d.event(Event::NoteOn {
            channel: 0,
            note: 69,
            volume: Some(127),
        });
        // Full bend up = two semitones: 440 * 2^(2/12) = 493.9 Hz.
        d.event(Event::PitchBend {
            channel: 0,
            value: 255,
        });
        let _ = render(&mut d, 2000);
        let measured = crossings_per_second(&render(&mut d, 44100), 44100);
        assert!((measured - 493.9 * 127.0 / 128.0).abs() < 8.0, "{measured}");
    }

    #[test]
    fn volume_controls_loudness() {
        let peak = |velocity: u8, channel_volume: u8, music: i32| {
            let mut d = OplDriver::new(sine_bank(), 44100);
            d.set_music_volume(music);
            d.event(Event::Controller {
                channel: 0,
                controller: 3,
                value: channel_volume,
            });
            d.event(Event::NoteOn {
                channel: 0,
                note: 69,
                volume: Some(velocity),
            });
            let _ = render(&mut d, 1000);
            render(&mut d, 4000)
                .into_iter()
                .map(i32::abs)
                .max()
                .unwrap()
        };
        let full = peak(127, 127, 127);
        assert!(full > 1000, "silent: {full}");
        assert!(peak(64, 127, 127) < full);
        assert!(peak(127, 64, 127) < full);
        assert!(peak(127, 127, 64) < full);
        assert!(peak(127, 127, 0) < full / 20);
    }

    #[test]
    fn a_released_note_dies_away() {
        let mut d = OplDriver::new(sine_bank(), 44100);
        d.event(Event::NoteOn {
            channel: 0,
            note: 69,
            volume: Some(127),
        });
        let _ = render(&mut d, 5000);
        assert!(d.is_sounding());
        d.event(Event::NoteOff {
            channel: 0,
            note: 69,
        });
        // Release rate 15 (sample bank): a few ms.
        let _ = render(&mut d, 44100);
        assert!(!d.is_sounding());
    }

    #[test]
    fn player_follows_the_sequencer_clock() {
        // Note on at tick 0, off after 140 ticks (one second), score end.
        let score = [
            0x90,
            0x80 | 69,
            127, // note on (last)
            0x81,
            0x0c, // delay 140
            0x80, // release-note event (type 0) with the last-event flag
            69,
            0x00, // note off (last), delay 0
            0x60,
        ];
        let mut lump = b"MUS\x1a".to_vec();
        lump.extend_from_slice(&(score.len() as u16).to_le_bytes());
        lump.extend_from_slice(&16u16.to_le_bytes());
        lump.extend_from_slice(&[0; 8]);
        lump.extend_from_slice(&score);
        let mut p = MusicPlayer::new(sine_bank(), 44100);
        assert!(p.register(&lump));
        p.play(false);
        let mut acc = vec![0i32; 2 * 44100 / 2];
        p.render_add(&mut acc);
        let half_second_peak = acc.iter().map(|s| s.abs()).max().unwrap();
        assert!(half_second_peak > 1000);
        assert!(p.playing, "still before the note off");
        let mut acc = vec![0i32; 2 * 44100];
        p.render_add(&mut acc);
        assert!(!p.playing, "score ended");
        // After the release tail the player is silent and stops rendering.
        let mut acc = vec![0i32; 2 * 4410];
        p.render_add(&mut acc);
        assert!(acc.iter().all(|&s| s == 0));
    }
}

/// Tests against the real WAD (skipped without one; see `iwad_bytes`).
#[cfg(test)]
mod wad_tests {
    use super::*;
    use crate::regression_tests::iwad_bytes;
    use alloc::string::String;
    use alloc::vec::Vec;

    /// The lump called `name` in a WAD image.
    fn lump<'a>(wad: &'a [u8], name: &str) -> Option<&'a [u8]> {
        let count = u32::from_le_bytes(wad[4..8].try_into().unwrap()) as usize;
        let dir = u32::from_le_bytes(wad[8..12].try_into().unwrap()) as usize;
        (0..count).find_map(|i| {
            let e = &wad[dir + 16 * i..dir + 16 * i + 16];
            let entry_name = e[8..16].split(|&b| b == 0).next().unwrap();
            (entry_name == name.as_bytes()).then(|| {
                let pos = u32::from_le_bytes(e[0..4].try_into().unwrap()) as usize;
                let len = u32::from_le_bytes(e[4..8].try_into().unwrap()) as usize;
                &wad[pos..pos + len]
            })
        })
    }

    fn song_names(wad: &[u8]) -> Vec<String> {
        let count = u32::from_le_bytes(wad[4..8].try_into().unwrap()) as usize;
        let dir = u32::from_le_bytes(wad[8..12].try_into().unwrap()) as usize;
        (0..count)
            .filter_map(|i| {
                let e = &wad[dir + 16 * i..dir + 16 * i + 16];
                let name = e[8..16].split(|&b| b == 0).next().unwrap();
                let name = core::str::from_utf8(name).ok()?;
                let data = lump(wad, name)?;
                (name.starts_with("D_") && data.starts_with(b"MUS\x1a")).then(|| name.into())
            })
            .collect()
    }

    #[test]
    fn genmidi_of_the_iwad_parses() {
        let Some(wad) = iwad_bytes() else {
            std::eprintln!("skipping: no IWAD");
            return;
        };
        let bank = GenMidi::parse(lump(&wad, "GENMIDI").unwrap()).unwrap();
        // Instrument 0 (Acoustic Grand Piano) is single-voice with a base note of 0.
        let piano = bank.melodic(0);
        assert_eq!(piano.flags & genmidi::FLAG_TWO_VOICE, 0);
        assert_eq!(piano.voices[0].base_note_offset, 0);
        assert_eq!(piano.voices[0].carrier.level & 0xc0, 0);
        // Bass drum is a fixed-note percussion instrument.
        let kick = bank.percussion(36).unwrap();
        assert_ne!(kick.flags & genmidi::FLAG_FIXED, 0);
        // The name table follows the 175 instruments.
        let lump = lump(&wad, "GENMIDI").unwrap();
        assert!(lump[8 + 175 * 36..].starts_with(b"Acoustic Grand Piano"));
    }

    /// Plays the first ten seconds of every song of the WAD, checks they are
    /// audible, and runs the sequencer alone to the score end.
    #[test]
    fn every_song_is_audible_and_ends() {
        let Some(wad) = iwad_bytes() else {
            std::eprintln!("skipping: no IWAD");
            return;
        };
        let bank_lump = lump(&wad, "GENMIDI").unwrap();
        let names = song_names(&wad);
        assert!(names.len() >= 9, "only {} songs", names.len());
        for name in names {
            let data = lump(&wad, &name).unwrap();
            let mut player = MusicPlayer::new(GenMidi::parse(bank_lump).unwrap(), 44100);
            assert!(player.register(data), "{name}");
            player.play(false);
            let mut peak = 0i32;
            let mut chunk = [0i32; 2 * 441];
            for _ in 0..1000 {
                chunk.fill(0);
                player.render_add(&mut chunk);
                peak = peak.max(chunk.iter().map(|s| s.abs()).max().unwrap());
            }
            assert!(peak > 2000, "{name} is nearly silent: peak {peak}");
            assert!(peak < 200_000, "{name} is absurdly loud: peak {peak}");

            let song = Song::parse(data).unwrap();
            let mut sequencer = Sequencer::new();
            let mut ticks = 0u32;
            while !sequencer.ended() && ticks < TICKS_PER_SECOND * 600 {
                sequencer.tick(&song, &mut |_| {});
                ticks += 1;
            }
            assert!(sequencer.ended(), "{name} never ended");
            assert!(
                ticks > TICKS_PER_SECOND * 5,
                "{name} ended after {ticks} ticks"
            );
        }
    }
}
