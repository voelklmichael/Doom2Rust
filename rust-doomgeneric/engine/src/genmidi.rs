//! The `GENMIDI` lump: the OPL2 patches Doom plays its music with.
//!
//! Layout (from the DMX / Chocolate Doom documentation): the ASCII tag
//! `#OPL_II#`, then 175 instruments of 36 bytes (128 General MIDI melodic ones
//! followed by 47 percussion ones, keys 35..=81), then 175 32-byte names that
//! are not needed for playback.

use crate::le::{le_i16, le_u16};
pub const NUM_INSTRUMENTS: usize = 175;
/// Index of the first percussion instrument; key 35 plays this one.
const PERCUSSION_BASE: usize = 128;
/// First and last General MIDI percussion key that has an instrument.
const PERCUSSION_KEYS: core::ops::RangeInclusive<u8> = 35..=81;

const TAG: &[u8; 8] = b"#OPL_II#";
const INSTRUMENT_LEN: usize = 36;
const VOICE_LEN: usize = 16;

/// The instrument plays one fixed note whatever key is pressed.
pub const FLAG_FIXED: u16 = 0x01;
/// The instrument uses both of its voices.
pub const FLAG_TWO_VOICE: u16 = 0x04;

/// One operator's register values, in the units the OPL registers take.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Operator {
    /// Register 0x20: tremolo, vibrato, sustain, key-scale rate, multiplier.
    pub tremolo: u8,
    /// Register 0x60: attack rate (high nibble), decay rate.
    pub attack: u8,
    /// Register 0x80: sustain level (high nibble), release rate.
    pub sustain: u8,
    /// Register 0xE0.
    pub waveform: u8,
    /// Key-scale level, already in bits 6-7 of register 0x40.
    pub scale: u8,
    /// Total level, 0 (loudest) to 63.
    pub level: u8,
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Voice {
    pub modulator: Operator,
    /// Register 0xC0: feedback and connection (bit 0: additive instead of FM).
    pub feedback: u8,
    pub carrier: Operator,
    /// Semitones added to the note.
    pub base_note_offset: i16,
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Instrument {
    pub flags: u16,
    /// 128 is no detune; only the second voice of a two-voice instrument uses it.
    pub fine_tuning: u8,
    pub fixed_note: u8,
    pub voices: [Voice; 2],
}

pub struct GenMidi {
    instruments: [Instrument; NUM_INSTRUMENTS],
}

fn operator(b: &[u8]) -> Operator {
    Operator {
        tremolo: b[0],
        attack: b[1],
        sustain: b[2],
        waveform: b[3],
        scale: b[4],
        level: b[5],
    }
}

fn voice(b: &[u8]) -> Voice {
    Voice {
        modulator: operator(&b[0..6]),
        feedback: b[6],
        carrier: operator(&b[7..13]),
        // b[13] is unused.
        base_note_offset: le_i16(b, 14),
    }
}

impl GenMidi {
    /// Parses the lump, or `None` if it is not a `GENMIDI` lump.
    #[allow(clippy::chunks_exact_to_as_chunks)] // as_chunks needs a newer toolchain than the ESP one
    pub fn parse(lump: &[u8]) -> Option<Self> {
        let body = lump.strip_prefix(TAG)?;
        if body.len() < NUM_INSTRUMENTS * INSTRUMENT_LEN {
            return None;
        }
        let mut instruments = [Instrument::default(); NUM_INSTRUMENTS];
        for (instrument, b) in instruments
            .iter_mut()
            .zip(body.chunks_exact(INSTRUMENT_LEN))
        {
            *instrument = Instrument {
                flags: le_u16(b, 0),
                fine_tuning: b[2],
                fixed_note: b[3],
                voices: [voice(&b[4..4 + VOICE_LEN]), voice(&b[4 + VOICE_LEN..])],
            };
        }
        Some(Self { instruments })
    }

    /// The instrument for MIDI program `program` (0..=127).
    pub fn melodic(&self, program: u8) -> &Instrument {
        &self.instruments[usize::from(program & 0x7f)]
    }

    /// The instrument for percussion `key`, or `None` outside keys 35..=81.
    pub fn percussion(&self, key: u8) -> Option<&Instrument> {
        PERCUSSION_KEYS.contains(&key).then(|| {
            &self.instruments[PERCUSSION_BASE + usize::from(key - PERCUSSION_KEYS.start())]
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    /// A well-formed `GENMIDI` lump with recognisable values: instrument `n`
    /// has tremolo byte `n`, fixed-note flag when `n` is odd, fixed note `n`,
    /// base note offset `-n`. Instrument 0 is single-voice.
    pub fn sample_lump() -> Vec<u8> {
        let mut lump = TAG.to_vec();
        for n in 0..NUM_INSTRUMENTS {
            let flags: u16 = if n % 2 == 1 { FLAG_FIXED } else { 0 };
            lump.extend_from_slice(&flags.to_le_bytes());
            lump.push(128);
            lump.push(n as u8);
            for part in 0..2u8 {
                // modulator: tremolo, attack, sustain, waveform, scale, level
                lump.extend_from_slice(&[n as u8, 0xf0, 0x0f, part, 0x40, 20]);
                lump.push(0x06); // feedback
                lump.extend_from_slice(&[0x21, 0xf1, 0x14, 1, 0, 3]);
                lump.push(0); // unused
                lump.extend_from_slice(&(-(n as i16)).to_le_bytes());
            }
        }
        // The names, which the parser does not look at.
        lump.extend(vec![b'x'; NUM_INSTRUMENTS * 32]);
        lump
    }

    #[test]
    fn parses_every_field() {
        let bank = GenMidi::parse(&sample_lump()).unwrap();
        let i = bank.melodic(3);
        assert_eq!(i.flags, FLAG_FIXED);
        assert_eq!(i.fine_tuning, 128);
        assert_eq!(i.fixed_note, 3);
        assert_eq!(
            i.voices[0].modulator,
            Operator {
                tremolo: 3,
                attack: 0xf0,
                sustain: 0x0f,
                waveform: 0,
                scale: 0x40,
                level: 20
            }
        );
        assert_eq!(i.voices[1].modulator.waveform, 1);
        assert_eq!(i.voices[0].feedback, 0x06);
        assert_eq!(
            i.voices[0].carrier,
            Operator {
                tremolo: 0x21,
                attack: 0xf1,
                sustain: 0x14,
                waveform: 1,
                scale: 0,
                level: 3
            }
        );
        assert_eq!(i.voices[0].base_note_offset, -3);
        assert_eq!(bank.melodic(0).flags, 0);
    }

    #[test]
    fn percussion_starts_at_key_35() {
        let bank = GenMidi::parse(&sample_lump()).unwrap();
        assert!(bank.percussion(34).is_none());
        assert_eq!(bank.percussion(35).unwrap().fixed_note, 128);
        assert_eq!(bank.percussion(81).unwrap().fixed_note, 174);
        assert!(bank.percussion(82).is_none());
    }

    #[test]
    fn rejects_other_lumps() {
        assert!(GenMidi::parse(b"").is_none());
        assert!(GenMidi::parse(b"#OPL_II#").is_none());
        let mut bad = sample_lump();
        bad[0] = b'X';
        assert!(GenMidi::parse(&bad).is_none());
        let short = &sample_lump()[..8 + 174 * INSTRUMENT_LEN];
        assert!(GenMidi::parse(short).is_none());
    }
}
