//! Software mixer for sound effects.
//!
//! Replaces the `SDL_mixer` / Allegro backends of the C original. It turns Doom's
//! DMX sound lumps (unsigned 8-bit mono, any rate) into interleaved stereo
//! `i16` at the output device's rate. It has no I/O of its own: the platform
//! says how many frames it wants (`DoomPlatform::audio_frames_wanted`) and the
//! engine calls [`Mixer::mix`] for exactly that many.
//!
//! The arithmetic follows `i_sdlsound.c`: nearest-neighbour rate conversion
//! (no filtering, hence the crunchy vanilla sound), 8-bit samples widened as
//! `b << 8 | b`, per-channel panning as `left = (254 - sep) * vol / 127` and
//! `right = sep * vol / 127` (each clamped to 0..=255, 255 meaning unity) and
//! saturating addition when channels overlap.

use crate::le::{le_u16, le_u32};
use alloc::rc::Rc;
use core::ops::Range;

/// Same as `NUM_CHANNELS` in `i_sdlsound.c`; Doom itself uses `snd_channels`
/// (default 8) of them.
pub const NUM_CHANNELS: usize = 16;

/// Bytes of header before the samples in a DMX sound lump.
const HEADER_LEN: usize = 8;
/// DMX drops this many bytes from both ends of the sample data.
const DMX_TRIM: usize = 16;
/// Lumps whose header length is this or less are rejected, as DMX does. The
/// length counts the trimmed bytes too, so 17 samples is the shortest that passes.
const MIN_LENGTH: usize = 48;

/// A validated sound effect: the samples are `data[samples]`, kept in place in
/// the (shared, cached) lump so starting a sound copies nothing.
#[derive(Clone)]
pub struct Sample {
    data: Rc<[u8]>,
    samples: Range<usize>,
    rate: u32,
}

impl Sample {
    /// Validates a DMX sound lump. `data` may be longer than the lump itself
    /// (the lump cache pads its buffers), so `lump_len` gives the real length.
    /// Returns `None` for anything `CacheSFX` in the C code rejects.
    pub fn from_lump(data: Rc<[u8]>, lump_len: usize) -> Option<Self> {
        let lump_len = lump_len.min(data.len());
        if lump_len < HEADER_LEN || data[0] != 0x03 || data[1] != 0x00 {
            return None;
        }
        let rate = u32::from(le_u16(&data, 2));
        let length = le_u32(&data, 4) as usize;
        if length > lump_len - HEADER_LEN || length <= MIN_LENGTH || rate == 0 {
            return None;
        }
        let start = HEADER_LEN + DMX_TRIM;
        let end = HEADER_LEN + length - DMX_TRIM;
        Some(Self {
            data,
            samples: start..end,
            rate,
        })
    }
}

/// One playing sound.
struct Voice {
    sample: Sample,
    /// Index of the next source sample.
    index: usize,
    /// 16-bit fraction of the source position.
    frac: u32,
    /// Source samples per output frame, in 16.16 fixed point.
    step: u32,
    left: i32,
    right: i32,
}

pub struct Mixer {
    sample_rate: u32,
    voices: [Option<Voice>; NUM_CHANNELS],
}

/// `(left, right)` gains, 0..=255 each, for a Doom volume (0..=127) and
/// separation (0..=254, 128 = centred).
fn pan_gains(vol: i32, sep: i32) -> (i32, i32) {
    let left = ((254 - sep) * vol) / 127;
    let right = (sep * vol) / 127;
    (left.clamp(0, 255), right.clamp(0, 255))
}

impl Mixer {
    pub fn new(sample_rate: u32) -> Self {
        assert!(sample_rate > 0, "audio sample rate must be positive");
        Self {
            sample_rate,
            voices: core::array::from_fn(|_| None),
        }
    }

    /// Starts `sample` on `channel`, replacing whatever it was playing.
    /// Returns `false` if `channel` is out of range.
    pub fn start(&mut self, channel: i32, sample: Sample, vol: i32, sep: i32) -> bool {
        let Some(slot) = usize::try_from(channel)
            .ok()
            .and_then(|c| self.voices.get_mut(c))
        else {
            return false;
        };
        let (left, right) = pan_gains(vol, sep);
        let step = ((u64::from(sample.rate) << 16) / u64::from(self.sample_rate)) as u32;
        *slot = Some(Voice {
            sample,
            index: 0,
            frac: 0,
            step,
            left,
            right,
        });
        true
    }

    pub fn set_params(&mut self, channel: i32, vol: i32, sep: i32) {
        if let Some(Some(voice)) = usize::try_from(channel)
            .ok()
            .and_then(|c| self.voices.get_mut(c))
        {
            (voice.left, voice.right) = pan_gains(vol, sep);
        }
    }

    pub fn stop(&mut self, channel: i32) {
        if let Some(slot) = usize::try_from(channel)
            .ok()
            .and_then(|c| self.voices.get_mut(c))
        {
            *slot = None;
        }
    }

    pub fn is_playing(&self, channel: i32) -> bool {
        usize::try_from(channel)
            .ok()
            .and_then(|c| self.voices.get(c))
            .is_some_and(Option::is_some)
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Adds the next `acc.len() / 2` frames of every playing voice to `acc`
    /// (interleaved stereo, unclipped). Voices that run out of samples are
    /// dropped, which is what makes [`is_playing`](Self::is_playing) turn false.
    ///
    /// The voices add `sample * gain` (gain 0..=255, unity 255) into a scratch
    /// buffer and the division by 255 happens once per output sample, not once
    /// per voice and sample: on the ESP32 the divide was the mixer's main cost.
    pub fn mix_add(&mut self, acc: &mut [i32]) {
        const CHUNK_FRAMES: usize = 256;
        for out in acc.chunks_mut(CHUNK_FRAMES * 2) {
            let mut scaled = [0i32; CHUNK_FRAMES * 2];
            let scaled = &mut scaled[..out.len() / 2 * 2];
            for slot in &mut self.voices {
                let Some(voice) = slot else { continue };
                if voice.render_into(scaled) {
                    *slot = None;
                }
            }
            for (sum, scaled) in out.iter_mut().zip(scaled.iter()) {
                *sum += scaled / 255;
            }
        }
    }

    /// Renders `out.len() / 2` frames of interleaved stereo, clipped to 16 bits.
    #[cfg(test)]
    pub fn mix(&mut self, out: &mut [i16]) {
        const CHUNK_FRAMES: usize = 256;
        for chunk in out.chunks_mut(CHUNK_FRAMES * 2) {
            let mut acc = [0i32; CHUNK_FRAMES * 2];
            let acc = &mut acc[..chunk.len() / 2 * 2];
            self.mix_add(acc);
            for (o, a) in chunk.iter_mut().zip(acc.iter()) {
                *o = (*a).clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
            }
        }
    }
}

impl Voice {
    /// Adds this voice, scaled by its gains but not yet divided by 255, to `acc`
    /// (interleaved stereo). Returns `true` once the sample has been played to
    /// its end.
    #[allow(clippy::chunks_exact_to_as_chunks)] // as_chunks needs a newer toolchain than the ESP one
    fn render_into(&mut self, acc: &mut [i32]) -> bool {
        let pcm = &self.sample.data[self.sample.samples.clone()];
        for frame in acc.chunks_exact_mut(2) {
            let Some(&byte) = pcm.get(self.index) else {
                return true;
            };
            // Widen 8 -> 16 bits the way the C code does: `b | b << 8`, then
            // recentre from unsigned.
            let s = i32::from(byte) * 257 - 32768;
            frame[0] += s * self.left;
            frame[1] += s * self.right;
            let pos = self.frac + self.step;
            self.index += (pos >> 16) as usize;
            self.frac = pos & 0xffff;
        }
        self.index >= pcm.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    /// A DMX lump: header, 16 bytes of padding either side, then `pcm`.
    fn lump(rate: u16, pcm: &[u8]) -> Vec<u8> {
        let length = (pcm.len() + 2 * DMX_TRIM) as u32;
        let mut v = vec![0x03, 0x00];
        v.extend_from_slice(&rate.to_le_bytes());
        v.extend_from_slice(&length.to_le_bytes());
        v.extend_from_slice(&[128; DMX_TRIM]);
        v.extend_from_slice(pcm);
        v.extend_from_slice(&[128; DMX_TRIM]);
        v
    }

    fn sample(rate: u16, pcm: &[u8]) -> Sample {
        let bytes = lump(rate, pcm);
        let len = bytes.len();
        Sample::from_lump(Rc::from(bytes), len).expect("valid lump")
    }

    #[test]
    fn from_lump_trims_the_dmx_padding() {
        let pcm: Vec<u8> = (0..100).collect();
        let s = sample(11025, &pcm);
        assert_eq!(s.samples.len(), 100);
        assert_eq!(s.rate, 11025);
        assert_eq!(&s.data[s.samples.clone()], &pcm[..]);
    }

    #[test]
    fn from_lump_ignores_cache_padding() {
        let mut bytes = lump(11025, &[200; 100]);
        let len = bytes.len();
        bytes.extend_from_slice(&[0xee; 128]);
        let s = Sample::from_lump(Rc::from(bytes), len).unwrap();
        assert_eq!(s.samples.len(), 100);
    }

    #[test]
    fn from_lump_rejects_what_the_c_code_rejects() {
        let good = lump(11025, &[128; 100]);
        let n = good.len();
        let check = |bytes: Vec<u8>, len: usize| Sample::from_lump(Rc::from(bytes), len);
        assert!(check(good.clone(), n).is_some());
        // Too short for a header.
        assert!(check(good[..7].to_vec(), 7).is_none());
        // Wrong format tag.
        let mut bad = good.clone();
        bad[0] = 0x02;
        assert!(check(bad, n).is_none());
        // The header claims more samples than the lump holds.
        assert!(check(good.clone(), n - 1).is_none());
        // A header length of 48 or less (16 samples plus the 32 trimmed bytes).
        assert!(check(lump(11025, &[128; 16]), 8 + 48).is_none());
        assert!(check(lump(11025, &[128; 17]), 8 + 49).is_some());
        // A rate of zero cannot be resampled.
        assert!(check(lump(0, &[128; 100]), n).is_none());
    }

    #[test]
    fn eight_bit_samples_widen_like_the_c_code() {
        // 0 -> -32768, 128 -> 128*257-32768 = 128, 255 -> 32767.
        let mut m = Mixer::new(11025);
        let mut pcm = [128; 20];
        pcm[..3].copy_from_slice(&[0, 128, 255]);
        assert!(m.start(0, sample(11025, &pcm), 127, 127));
        let mut out = [0i16; 6];
        m.mix(&mut out);
        // sep 127, vol 127: left = 127, right = 127 (about half of 255).
        assert_eq!(out[0], (-32768 * 127 / 255) as i16);
        assert_eq!(out[2], (128 * 127 / 255) as i16);
        assert_eq!(out[4], (32767 * 127 / 255) as i16);
        assert_eq!(out[0], out[1]);
    }

    #[test]
    fn panning_follows_doom_separation() {
        // Full left (sep 0) and full right (sep 254) at full volume.
        assert_eq!(pan_gains(127, 0), (254, 0));
        assert_eq!(pan_gains(127, 254), (0, 254));
        // Centred, silent, and clamped.
        assert_eq!(pan_gains(127, 128), (126, 128));
        assert_eq!(pan_gains(0, 128), (0, 0));
        assert_eq!(pan_gains(127, 300), (0, 255));
        let mut m = Mixer::new(11025);
        m.start(3, sample(11025, &[255; 60]), 127, 254);
        let mut out = [0i16; 4];
        m.mix(&mut out);
        assert_eq!(out[0], 0);
        assert!(out[1] > 30000);
    }

    #[test]
    fn upsampling_repeats_source_samples() {
        // 11025 -> 44100 is a plain 4x repeat.
        let mut m = Mixer::new(44100);
        let mut pcm = [0; 20];
        pcm[1..].fill(255);
        m.start(0, sample(11025, &pcm), 127, 127);
        let mut out = [0i16; 16];
        m.mix(&mut out);
        let left: Vec<i16> = out.iter().step_by(2).copied().collect();
        assert_eq!(left[0], left[3]);
        assert_eq!(left[4], left[7]);
        assert!(left[0] < 0 && left[4] > 0);
    }

    #[test]
    fn sample_ends_and_channel_frees() {
        let mut m = Mixer::new(11025);
        m.start(0, sample(11025, &[255; 50]), 127, 128);
        assert!(m.is_playing(0));
        let mut out = [0i16; 2 * 49];
        m.mix(&mut out);
        assert!(m.is_playing(0), "one sample still to go");
        let mut out = [1i16; 2 * 10];
        m.mix(&mut out);
        assert!(!m.is_playing(0));
        // Past the end it renders silence.
        assert_eq!(out[2..], [0; 18]);
    }

    #[test]
    fn voices_share_one_division() {
        // Two voices with gains 127 and 90: the sum of the scaled samples is
        // divided by 255 once, so it can differ from the sum of two separate
        // divisions by less than one step per voice.
        let mut m = Mixer::new(11025);
        let mut a = [128u8; 20];
        a[0] = 200;
        let mut b = [128u8; 20];
        b[0] = 40;
        m.start(0, sample(11025, &a), 127, 254); // right gain 254
        m.start(1, sample(11025, &b), 90, 254); // right gain 180
        let mut out = [0i16; 2];
        m.mix(&mut out);
        let s = |byte: i32| byte * 257 - 32768;
        assert_eq!(i32::from(out[1]), (s(200) * 254 + s(40) * 180) / 255);
        assert_eq!(out[0], 0);
    }

    #[test]
    fn overlapping_channels_saturate() {
        let mut m = Mixer::new(11025);
        for c in 0..3 {
            m.start(c, sample(11025, &[255; 60]), 127, 254);
        }
        let mut out = [0i16; 2];
        m.mix(&mut out);
        assert_eq!(out[1], i16::MAX);
    }

    #[test]
    fn stop_and_bad_channels() {
        let mut m = Mixer::new(11025);
        assert!(!m.start(-1, sample(11025, &[0; 60]), 127, 128));
        assert!(!m.start(NUM_CHANNELS as i32, sample(11025, &[0; 60]), 127, 128));
        assert!(m.start(5, sample(11025, &[0; 60]), 127, 128));
        assert!(m.is_playing(5));
        m.stop(5);
        assert!(!m.is_playing(5));
        m.stop(-3);
        m.set_params(99, 1, 1);
        assert!(!m.is_playing(-1));
    }

    #[test]
    fn mixing_across_chunk_boundaries_matches_one_call() {
        let pcm: Vec<u8> = (0..2000u32).map(|i| (i * 7) as u8).collect();
        let render = |split: usize| {
            let mut m = Mixer::new(22050);
            m.start(0, sample(11025, &pcm), 100, 90);
            m.start(1, sample(11025, &pcm[10..]), 60, 200);
            let mut all = vec![0i16; 2 * 4000];
            let (a, b) = all.split_at_mut(split);
            m.mix(a);
            m.mix(b);
            all
        };
        assert_eq!(render(2 * 300), render(2 * 1234));
    }
}
