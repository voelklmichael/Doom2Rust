//! Reader and sequencer for Doom's MUS music format.
//!
//! A MUS lump is a header followed by a stream of events. Each event byte holds
//! a "last event of this group" flag, a channel (0..=15, where 15 is the drum
//! channel) and an event type; when the flag is set a variable-length delay in
//! ticks follows. One tick is 1/140 s. `mus2mid.c` translates this into a MIDI
//! file for SDL_mixer; the OPL driver here works from the events directly.

use alloc::vec::Vec;

/// MUS ticks per second.
pub const TICKS_PER_SECOND: u32 = 140;

const HEADER_ID: [u8; 4] = *b"MUS\x1a";
const HEADER_LEN: usize = 16;

/// The channel MUS uses for percussion.
pub const PERCUSSION_CHANNEL: u8 = 15;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Event {
    NoteOff {
        channel: u8,
        note: u8,
    },
    /// `volume` is `None` when the event keeps the channel's last note volume.
    NoteOn {
        channel: u8,
        note: u8,
        volume: Option<u8>,
    },
    /// 0..=255, 128 is centred; the full range is two semitones either way.
    PitchBend {
        channel: u8,
        value: u8,
    },
    /// Controllers 10..=14: all sounds off, all notes off, mono, poly, reset
    /// all controllers.
    System {
        channel: u8,
        controller: u8,
    },
    /// Controller 0 is a program change; 3 is the channel volume.
    Controller {
        channel: u8,
        controller: u8,
        value: u8,
    },
    EndOfMeasure,
    ScoreEnd,
}

/// A validated MUS lump: just the event stream.
pub struct Song {
    score: Vec<u8>,
}

impl Song {
    /// Returns `None` if `lump` is not a MUS file.
    pub fn parse(lump: &[u8]) -> Option<Self> {
        if lump.len() < HEADER_LEN || lump[..4] != HEADER_ID {
            return None;
        }
        let score_len = usize::from(u16::from_le_bytes([lump[4], lump[5]]));
        let score_start = usize::from(u16::from_le_bytes([lump[6], lump[7]]));
        if score_start < HEADER_LEN || score_start > lump.len() {
            return None;
        }
        // Some lumps understate or overstate the length: play what is there.
        let end = (score_start + score_len).min(lump.len());
        Some(Self {
            score: lump[score_start..end].to_vec(),
        })
    }
}

/// Position in a [`Song`]'s event stream.
#[derive(Clone, Default)]
pub struct Sequencer {
    pos: usize,
    /// Ticks until the next group of events.
    wait: u32,
    ended: bool,
}

impl Sequencer {
    pub fn new() -> Self {
        Self::default()
    }

    /// True once the score end has been reached.
    pub fn ended(&self) -> bool {
        self.ended
    }

    /// Advances one tick, calling `emit` for every event that falls on it.
    pub fn tick(&mut self, song: &Song, emit: &mut impl FnMut(Event)) {
        self.wait = self.wait.saturating_sub(1);
        while self.wait == 0 && !self.ended {
            let Some((event, last)) = self.read_event(song) else {
                // Ran off the end without a score-end event.
                self.ended = true;
                emit(Event::ScoreEnd);
                return;
            };
            emit(event);
            if event == Event::ScoreEnd {
                self.ended = true;
                return;
            }
            if last {
                self.wait = self.read_delay(song);
            }
        }
    }

    fn byte(&mut self, song: &Song) -> Option<u8> {
        let b = *song.score.get(self.pos)?;
        self.pos += 1;
        Some(b)
    }

    fn read_event(&mut self, song: &Song) -> Option<(Event, bool)> {
        let descriptor = self.byte(song)?;
        let last = descriptor & 0x80 != 0;
        let channel = descriptor & 0x0f;
        let event = match (descriptor >> 4) & 7 {
            0 => Event::NoteOff {
                channel,
                note: self.byte(song)? & 0x7f,
            },
            1 => {
                let first = self.byte(song)?;
                let volume = if first & 0x80 != 0 {
                    Some(self.byte(song)? & 0x7f)
                } else {
                    None
                };
                Event::NoteOn {
                    channel,
                    note: first & 0x7f,
                    volume,
                }
            }
            2 => Event::PitchBend {
                channel,
                value: self.byte(song)?,
            },
            3 => Event::System {
                channel,
                controller: self.byte(song)? & 0x7f,
            },
            4 => Event::Controller {
                channel,
                controller: self.byte(song)? & 0x7f,
                value: self.byte(song)? & 0x7f,
            },
            5 => Event::EndOfMeasure,
            6 => Event::ScoreEnd,
            // Type 7 is unused; skip its data byte as mus2mid does.
            _ => {
                self.byte(song)?;
                Event::EndOfMeasure
            }
        };
        Some((event, last))
    }

    /// A delay is a big-endian number of 7-bit groups, high bit = more follow.
    fn read_delay(&mut self, song: &Song) -> u32 {
        let mut delay = 0u32;
        while let Some(b) = self.byte(song) {
            delay = (delay << 7) | u32::from(b & 0x7f);
            if b & 0x80 == 0 {
                return delay;
            }
        }
        // Truncated: the next read_event ends the song.
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn lump(score: &[u8]) -> Vec<u8> {
        let mut v = b"MUS\x1a".to_vec();
        v.extend_from_slice(&(score.len() as u16).to_le_bytes());
        v.extend_from_slice(&16u16.to_le_bytes());
        v.extend_from_slice(&[0; 8]);
        v.extend_from_slice(score);
        v
    }

    /// Runs `ticks` ticks and returns `(tick, event)` pairs.
    fn run(score: &[u8], ticks: u32) -> Vec<(u32, Event)> {
        let song = Song::parse(&lump(score)).unwrap();
        let mut seq = Sequencer::new();
        let mut events = Vec::new();
        for t in 0..ticks {
            seq.tick(&song, &mut |e| events.push((t, e)));
        }
        events
    }

    #[test]
    fn rejects_other_data() {
        assert!(Song::parse(b"MThd\0\0\0\x06\0\0\0\x01\0\x60\0\0").is_none());
        assert!(Song::parse(b"MUS\x1a").is_none());
        // Score start beyond the end of the lump.
        let mut bad = lump(&[0x60]);
        bad[6] = 200;
        assert!(Song::parse(&bad).is_none());
    }

    #[test]
    fn events_land_on_their_ticks() {
        let score = [
            0x12,
            0x80 | 60,
            100, // note on ch 2, key 60, volume 100 (not last)
            0xa2,
            128, // pitch bend ch 2 = 128 (last), so a delay follows
            3,   // 3 ticks
            0x02,
            60,   // note off ch 2, key 60 (not last)
            0x60, // score end
        ];
        let events = run(&score, 10);
        assert_eq!(
            events,
            vec![
                (
                    0,
                    Event::NoteOn {
                        channel: 2,
                        note: 60,
                        volume: Some(100)
                    }
                ),
                (
                    0,
                    Event::PitchBend {
                        channel: 2,
                        value: 128
                    }
                ),
                (
                    3,
                    Event::NoteOff {
                        channel: 2,
                        note: 60
                    }
                ),
                (3, Event::ScoreEnd),
            ]
        );
    }

    #[test]
    fn multi_byte_delay_and_note_without_volume() {
        let score = [
            0x90, 60, // note on ch 0, key 60, no volume, last
            0x81, 0x01, // delay = (1 << 7) | 1 = 129
            0x60, // score end
        ];
        let events = run(&score, 200);
        assert_eq!(
            events,
            vec![
                (
                    0,
                    Event::NoteOn {
                        channel: 0,
                        note: 60,
                        volume: None
                    }
                ),
                (129, Event::ScoreEnd),
            ]
        );
    }

    #[test]
    fn controllers_and_system_events() {
        let score = [
            0x4f, 3, 90, // controller 3 (volume) = 90 on the drum channel
            0xb1, 11, // all notes off, ch 1, last
            0,  // no delay
            0x60,
        ];
        let events = run(&score, 1);
        assert_eq!(
            events,
            vec![
                (
                    0,
                    Event::Controller {
                        channel: 15,
                        controller: 3,
                        value: 90
                    }
                ),
                (
                    0,
                    Event::System {
                        channel: 1,
                        controller: 11
                    }
                ),
                (0, Event::ScoreEnd),
            ]
        );
    }

    #[test]
    fn truncated_score_ends_instead_of_panicking() {
        let events = run(&[0x12, 0x80 | 60], 5); // note on missing its volume
        assert_eq!(events.last().map(|e| e.1), Some(Event::ScoreEnd));
        let events = run(&[], 5);
        assert_eq!(events, vec![(0, Event::ScoreEnd)]);
    }

    #[test]
    fn score_length_is_clamped_to_the_lump() {
        let mut l = lump(&[0x60]);
        l[4] = 0xff; // claims 255 bytes
        let song = Song::parse(&l).unwrap();
        assert_eq!(song.score.len(), 1);
    }
}
