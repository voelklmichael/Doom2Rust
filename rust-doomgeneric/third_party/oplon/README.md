# oplon

**Yamaha YM3812 / YMF262 (OPL2 / OPL3) FM synthesis core for Rust — `no_std`, integer-only, clean-room**

`oplon` reproduces the *sound* of the AdLib / Sound Blaster OPL chip: up to 18 two-operator FM channels plus the rhythm/percussion mode, summed to stereo. It powers up as an **OPL2** (9 channels, mono, waveforms 0-3) and becomes a full **OPL3** — 18 channels, stereo panning, waveforms 0-7 — the moment OPL3 mode is enabled (port-1 register `0x05`, or `set_opl3_enabled`), exactly like the real YMF262. So OPL2 code paths stay bit-for-bit unchanged while OPL3 streams get the extra features. It is designed to run on embedded targets, real-time audio threads, and any context where heap allocation and floating-point are unavailable or undesirable.

The operators run at the chip's **native rate** (OPL2 `clock / 72`, OPL3 `clock / 288`, ≈49716 Hz for the canonical clocks), and the output is 4-point-cubic-resampled to your sample rate — so the chip's own harmonic aliasing matches the silicon instead of folding at the wrong Nyquist.

It is the OPL counterpart of [`sidera`](https://codeberg.org/sbechet/sidera) (the SID engine), and ships the same way: a `no_std` core library, plus an optional CLI player behind a feature flag (`sidplay` there → `vgmplay` here).

**Fidelity:** verified against two independent OPL3 emulators used purely as black-box acoustic oracles — the DOSBox core (PyOPL/dbopl) and **Nuked-OPL3** (the cycle-accurate reference). Driving both with the same register stream from real Doom (id Software) VGMs gives a **median spectral cosine ≈ 0.99** and **RMS-envelope correlation ≈ 0.998**; the operator output is calibrated to the hardware ±4084 scale, so levels sit within a few tenths of a dB of Nuked. The **4-operator** algorithms match the reference near-exactly for parallel/single-modulation connections (correlation ≈ 0.994–0.999); a deep serial FM chain (algorithm 0) is bit-faithful sample-for-sample at onset but, being a 3-stage non-linear chain, decorrelates from the reference over time under sub-LSB rounding differences while staying perceptually equivalent (matched envelope, brightness and level). The OPL2 path is exercised by an exhaustive waveform equality test and a render golden hash.

## Properties

| Property            | Value                                              |
|---------------------|----------------------------------------------------|
| `no_std`            | ✅                                                 |
| Heap allocation     | ❌ none                                            |
| Floating-point      | ❌ none                                            |
| `unsafe`            | ❌ `#![forbid(unsafe_code)]`                       |
| Channels            | up to 18 × 2-operator FM (two banks) + rhythm mode |
| Waveforms           | all 8 OPL3 (sine, half, abs, quarter, even ×2, square, log-saw) |
| Envelope            | 8-bit ADSR, cycle-accurate rate table + dither     |
| LFOs                | tremolo (AM) + vibrato (PM), documented shapes      |
| Stereo panning      | OPL3, register 0xC0 bits 4/5                        |
| 4-operator mode     | ✅ all 4 connection algorithms (reg 0x104)           |
| ROM tables          | **generated at compile time**, integer-only, ±1 LSB |
| Output              | stereo `i32` per frame (per-operator ±4084)         |

## Two interfaces

`oplon::Opl2` exposes both layers; use whichever fits.

### Register level — what a VGM / DRO / ported driver needs

```rust
use oplon::Opl2;

let mut opl = Opl2::new(44_100);          // output sample rate
opl.write_reg(0x20, 0x01);                // op0 mult = 1
opl.write_reg(0x40, 0x00);                // op0 total level = loud
opl.write_reg(0x60, 0xF0);                // op0 fast attack
opl.write_reg(0x80, 0x00);                // op0 sustain/release
opl.write_reg(0xA0, 0x98);                // F-number low
opl.write_reg(0xB0, 0x31);                // key-on + block 4 + F-number high
let (left, right) = opl.render_frame();   // one stereo frame
```

For a non-standard master clock (VGM streams may declare one), use
`Opl2::with_chip_clock(clock_hz, output_rate)` — pitch scales with the clock.

### Structured note/patch level

```rust
use oplon::{Opl2, ChannelPatch, OpPatch};

let mut opl = Opl2::new(48_000);
opl.set_patch(0, &ChannelPatch {
    modulator: OpPatch { mul: 1, tl: 63, attack: 15, release: 7, sustaining: true, ..Default::default() },
    carrier:   OpPatch { mul: 1, attack: 15, release: 7, sustaining: true, ..Default::default() },
    ..Default::default()
});
opl.set_frequency(0, 0x2AE, 4);   // F-number / block
opl.key_on(0);
let (l, r) = opl.render_frame();
```

## The `vgmplay` binary

An optional CLI that plays [VGM](https://vgmrips.net/) and **DRO** (DOSBox Raw OPL) register logs through the engine — the OPL analogue of `sidera`'s `sidplay`.

```sh
cargo run --release --features player --bin vgmplay -- tune.vgm         # live (cpal)
cargo run --release --features player --bin vgmplay -- tune.vgz --wav out.wav
cargo run --release --features player --bin vgmplay -- tune.vgm --regdump regs.csv
```

Gzip-compressed `.vgz` files are decompressed transparently, and `.dro` files (DOSBox Raw OPL, both v1 and v2) are transcoded to VGM in memory so the same player path handles them. Scope is the **OPL family**: YM3812 (OPL2, VGM command `0x5A`), YM3526 (OPL1, `0x5B`), Y8950 (`0x5C`, melodic part) and the YMF262 (OPL3, both banks — port 0 `0x5E` and port 1 `0x5F`) — they share the register map. A YMF262 stream enables OPL3 mode (18 channels, stereo, waveforms 4-7, 4-operator linking) and is clocked at `clock / 288`. Other chips' commands are only skipped for timing.

`--regdump` writes the OPL register-write timeline as CSV (`sample,reg,val`) — the ground truth for comparing a reimplemented driver against the log.

## Clean-room & attribution

MIT-licensed. The OPL emulators `oplon` is *validated against* are not: Schism Tracker's `player/fmopl3.c` (a ymf262 core) is **GPL-2.0+, © Jarek Burczynski**, and Nuked-OPL3 is LGPL. **No code or data table is copied from either.** The log-sin / exp ROMs are generated from their defining formulas, the envelope-increment table from the documented `(4 + lo)·2^rate_hi` rate law, the tremolo/vibrato LFOs from their documented shapes, and the `MUL` / `KSL` / key-scale values from the Yamaha YM3812/YMF262 datasheet. Those references are consulted only to know what the silicon *does*; the implementation — structure, code and tables — is original.

## Licence

MIT © 2026 Sébastien Béchet.
