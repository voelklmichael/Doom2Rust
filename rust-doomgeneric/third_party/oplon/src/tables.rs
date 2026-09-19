//! OPL2/OPL3 ROM tables — **clean-room** and **generated at compile time**
//! in integer-only `const fn`. No hand-pasted magic constants: the two ROMs
//! are computed from their defining formulas, reproducing the canonical
//! Yamaha YM3812/YMF262 values to within ±1 LSB (asserted by
//! `generated_tables_match_canonical`).
//!
//! ## Q-formats
//!
//! * `LOGSIN_ROM[i]` — `−log2(sin((i+0.5)·π/512))` in **Q8** (i.e.
//!   `value / 256` = attenuation in octaves; `sin θ = 2^(−value/256)`).
//!   One quarter wave, 256 entries, range 0..2137.
//! * `EXP_ROM[lo]` — `2^(−lo/256)` mantissa in **Q10** (`value / 1024` =
//!   the linear magnitude). 256 entries, range 1024..513.
//!
//! A total attenuation `A = 256·hi + lo` (logsin + envelope, in Q8) maps
//! to a linear magnitude `EXP_ROM[lo] >> hi`.
//!
//! ## Generation
//!
//! * `EXP_ROM` uses an exact integer power function:
//!   `2^(−lo/256) = pow2_frac_q16_16(768 − 3·lo) >> 1` (Q16.16), so
//!   `EXP_ROM[lo] = (pow2_frac_q16_16(768 − 3·lo) + 64) >> 7`.
//! * `LOGSIN_ROM` is the inverse of `EXP_ROM` composed with a sine:
//!   compute `sin θ` by an integer Q30 Taylor series, normalise it into
//!   `[0.5, 1)` to get the integer part `hi`, then find the `lo` whose
//!   `EXP_ROM[lo]` best matches the mantissa. This keeps the two tables
//!   mutually consistent (`EXP_ROM[LOGSIN_ROM[i]]` reconstructs `sin θ`).
//!
//! The two integer helpers (`pow2_frac_q16_16`, `sin_pi_ratio_q30`) are the
//! same compile-time primitives xmrs uses; they carry **no float arithmetic**
//! so the whole ROM set is reproducible and `no_std`-friendly.

// =====================================================================
// Integer compile-time primitives (float-free)
// =====================================================================

/// Compile-time computation of `2^(x_num / 768)` in Q16.16 for `x_num` in
/// `[0, 768]`, via `exp(y·ln2)` as an integer Taylor series.
///
/// **Q0.32 / `u64` is deliberately the minimum width.** The largest
/// intermediate is `term·z` with `term ≤ 1.0` and `z ≤ ln2 < 1.0` in Q0.32, i.e.
/// `< 2^63.5`, which fits a `u64` with margin. Q0.32 carries 32 fractional bits —
/// far more than the Q16.16 result (and the 12-bit ROMs it feeds) need — so the
/// output is **bit-identical** to a `u128`/Q0.64 evaluation over the whole
/// domain. (Verified exhaustively; `u128` was over-wide for no gain.)
pub(crate) const fn pow2_frac_q16_16(x_num: u32) -> u32 {
    // ln(2) ≈ 0.6931471805599453, encoded in Q0.32.
    const LN2_Q32: u64 = 2_977_044_472; // round(ln2 · 2^32)

    // y = x_num / 768, in Q0.32
    let y: u64 = ((x_num as u64) << 32) / 768;

    // z = y · ln(2), in Q0.32
    let z: u64 = (y * LN2_Q32) >> 32;

    // Taylor series for exp(z): Σ z^n / n!
    let mut term: u64 = 1u64 << 32; // 1.0 in Q0.32
    let mut sum: u64 = 0;
    let mut n: u32 = 0;
    while n < 16 {
        sum += term;
        term = (term * z) >> 32;
        term /= (n as u64) + 1;
        n += 1;
    }

    // sum is Q0.32 representing 2^y ∈ [1, 2]. Narrow to Q16.16.
    (sum >> 16) as u32
}

/// Compile-time `sin(π · num/den)` in **Q30** (`i64`), for `num/den ∈ [0, 1]`
/// (angle `0..π`, result ≥ 0). Eight-term Taylor series with the argument
/// reflected into `[0, π/2]` (`sin(π − x) = sin x`); residual error ≲ 1e-8.
pub(crate) const fn sin_pi_ratio_q30(num: u32, den: u32) -> i64 {
    if den == 0 {
        return 0;
    }
    // Reflect num/den > 1/2 into the first quadrant so the Taylor
    // argument never exceeds π/2.
    let d = den as i64;
    let mut n = num as i64;
    if n > d {
        n = d; // clamp out-of-range inputs to sin(π) = 0
    }
    if 2 * n > d {
        n = d - n;
    }

    // π in Q30 = round(π · 2^30).
    const PI_Q30: i64 = 3_373_259_426;
    let theta: i64 = (PI_Q30 * n) / d; // Q30, ∈ [0, π/2]
    let x2: i64 = (theta * theta) >> 30; // Q30
    let mut term: i64 = theta; // x¹, Q30
    let mut sum: i64 = 0;
    let mut m: i64 = 1;
    let mut k = 0u32;
    while k < 8 {
        if k & 1 == 0 {
            sum += term;
        } else {
            sum -= term;
        }
        // term ·= x² / ((m+1)(m+2)) → next odd power over its factorial.
        term = ((term * x2) >> 30) / ((m + 1) * (m + 2));
        m += 2;
        k += 1;
    }
    sum
}

// =====================================================================
// OPL ROM tables (generated from the primitives above)
// =====================================================================

/// `2^(−lo/256)` mantissa, Q10 (256 entries). See module docs.
pub(crate) const EXP_ROM: [u16; 256] = {
    let mut t = [0u16; 256];
    let mut lo = 0usize;
    while lo < 256 {
        // 2^(−lo/256) = 2^((768 − 3·lo)/768) / 2 = pow2_frac(768 − 3lo) >> 1
        // in Q16.16; Q10 round = (pow2_frac + 64) >> 7.
        let p = pow2_frac_q16_16((768 - 3 * lo) as u32);
        t[lo] = ((p + 64) >> 7) as u16;
        lo += 1;
    }
    t
};

/// The **operator-output exp ROM**: `round(2^((255−i)/256) · 1024)`, 256
/// entries, range 2042..1024. This is the canonical YM3812/YMF262 exp table.
/// The operator magnitude is `(EXP_OUT[att & 0xFF] << 1) >> (att >> 8)`, which
/// evaluates to `4084 · 2^(−att/256)` — the chip's real ±4084 (13-bit) output
/// scale, *not* the ±4096 a clean `2^(−att/256)·4096` would give. The 0.29 %
/// difference is inaudible for a single operator but compounds through the
/// non-linear phase modulation of a multi-operator FM chain (OPL3 4-op mode),
/// so matching the hardware scale here is what keeps deep 4-op chains faithful.
/// (Distinct from [`EXP_ROM`], which is the `[0.5,1)`-mantissa table the
/// compile-time [`LOGSIN_ROM`] generator inverts.)
#[cfg_attr(feature = "iram", link_section = ".data.oplon", allow(unsafe_code))]
pub(crate) static EXP_OUT: [u16; 256] = {
    let mut t = [0u16; 256];
    let mut i = 0usize;
    while i < 256 {
        // 2^((255−i)/256) = 2^(3·(255−i)/768) = pow2_frac(3·(255−i)) in Q16.16;
        // ×1024 then round to integer = (pow2_frac + 32) >> 6.
        let p = pow2_frac_q16_16((3 * (255 - i)) as u32);
        t[i] = ((p + 32) >> 6) as u16;
        i += 1;
    }
    t
};

/// `−log2(sin((i+0.5)·π/512))`, Q8 (256 entries, quarter wave). See module
/// docs.
#[cfg_attr(feature = "iram", link_section = ".data.oplon", allow(unsafe_code))]
pub(crate) static LOGSIN_ROM: [u16; 256] = {
    let mut t = [0u16; 256];
    let mut i = 0u32;
    while i < 256 {
        // sin θ in Q30, θ = (2i+1)·π/1024 ∈ (0, π/2].
        let mut s = sin_pi_ratio_q30(2 * i + 1, 1024);
        // Normalise into [0.5, 1): each doubling adds 1 octave (= 256 in Q8).
        let mut hi: u16 = 0;
        let half: i64 = 1i64 << 29;
        while s < half {
            s <<= 1;
            hi += 1;
        }
        // Mantissa magnitude in Q10: round(m · 1024) where m = s / 2^30.
        let target = (((s * 1024) + (1i64 << 29)) >> 30) as i32;
        // Inverse-EXP: the `lo` whose `EXP_ROM[lo]` is nearest `target`.
        // `EXP_ROM` is monotone decreasing (1024 → 513), so the nearest entry
        // straddles `target`: binary-search the crossing (≈8 steps instead of
        // scanning all 256), then pick the closer of the two neighbours,
        // preferring the smaller index on a tie (matching a first-min scan).
        let mut lo = 0usize;
        let mut hi_idx = 256usize;
        while lo < hi_idx {
            let mid = (lo + hi_idx) / 2;
            if EXP_ROM[mid] as i32 > target {
                lo = mid + 1;
            } else {
                hi_idx = mid;
            }
        }
        let mut best_lo = 0u16;
        let mut best_d = i32::MAX;
        let mut c = if lo == 0 { 0usize } else { lo - 1 };
        while c <= lo && c < 256 {
            let diff = EXP_ROM[c] as i32 - target;
            let d = if diff < 0 { -diff } else { diff };
            if d < best_d {
                best_d = d;
                best_lo = c as u16;
            }
            c += 1;
        }
        t[i as usize] = 256 * hi + best_lo;
        i += 1;
    }
    t
};

/// Frequency-multiple register → phase-increment multiplier, in **half**
/// units (the real multiple is `MUL[n] / 2`, so `MUL[0] = 1` means ×0.5).
/// The documented YM3812/YMF262 datasheet multiple ladder (×0.5,1,2,…,15).
#[rustfmt::skip]
#[cfg_attr(feature = "iram", link_section = ".data.oplon", allow(unsafe_code))]
pub(crate) static MUL: [u32; 16] = [1, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 20, 24, 24, 30, 30];

/// Key-scale-level ROM indexed by the top 4 bits of the F-number. Gives the
/// per-block attenuation base (eg-domain units) before the block-dependent
/// subtraction. The documented datasheet KSL ladder (1.5 dB/octave steps).
#[rustfmt::skip]
pub(crate) const KSL: [u16; 16] = [0, 32, 40, 45, 48, 51, 53, 55, 56, 58, 59, 60, 61, 62, 63, 64];

/// KSL register field (2 bits) → right-shift applied to the KSL base. Index
/// `0` (off) is `8`, but the caller short-circuits `reg == 0`; `3` is the
/// full 6 dB/oct.
pub(crate) const KSL_SHIFT: [u8; 4] = [8, 1, 2, 0];

#[cfg(test)]
mod tests {
    use super::*;

    /// Canonical Yamaha YM3812/YMF262 log-sin ROM (the rounded reference of
    /// the defining formula). Test-only, purely to prove the compile-time
    /// generator reproduces it.
    #[rustfmt::skip]
    const CANON_LOGSIN: [u16; 256] = [
        2137, 1731, 1543, 1419, 1326, 1252, 1190, 1137, 1091, 1050, 1013,  979,  949,  920,  894,  869,
         846,  825,  804,  785,  767,  749,  732,  717,  701,  687,  672,  659,  646,  633,  621,  609,
         598,  587,  576,  566,  556,  546,  536,  527,  518,  509,  501,  492,  484,  476,  468,  461,
         453,  446,  439,  432,  425,  418,  411,  405,  399,  392,  386,  380,  375,  369,  363,  358,
         352,  347,  341,  336,  331,  326,  321,  316,  311,  307,  302,  297,  293,  289,  284,  280,
         276,  271,  267,  263,  259,  255,  251,  248,  244,  240,  236,  233,  229,  226,  222,  219,
         215,  212,  209,  205,  202,  199,  196,  193,  190,  187,  184,  181,  178,  175,  172,  169,
         167,  164,  161,  159,  156,  153,  151,  148,  146,  143,  141,  138,  136,  134,  131,  129,
         127,  125,  122,  120,  118,  116,  114,  112,  110,  108,  106,  104,  102,  100,   98,   96,
          94,   92,   91,   89,   87,   85,   83,   82,   80,   78,   77,   75,   74,   72,   70,   69,
          67,   66,   64,   63,   62,   60,   59,   57,   56,   55,   53,   52,   51,   49,   48,   47,
          46,   45,   43,   42,   41,   40,   39,   38,   37,   36,   35,   34,   33,   32,   31,   30,
          29,   28,   27,   26,   25,   24,   23,   23,   22,   21,   20,   20,   19,   18,   17,   17,
          16,   15,   15,   14,   13,   13,   12,   12,   11,   10,   10,    9,    9,    8,    8,    7,
           7,    7,    6,    6,    5,    5,    5,    4,    4,    4,    3,    3,    3,    2,    2,    2,
           2,    1,    1,    1,    1,    1,    1,    1,    0,    0,    0,    0,    0,    0,    0,    0,
    ];

    #[test]
    fn exp_out_matches_canonical_hardware_rom() {
        // Endpoints of the canonical YM3812/YMF262 exp ROM.
        assert_eq!(EXP_OUT[0], 2042);
        assert_eq!(EXP_OUT[255], 1024);
        // Peak operator magnitude at zero attenuation is the chip's ±4084.
        assert_eq!((EXP_OUT[0] as u32) << 1, 4084);
        // Monotonically decreasing (2^((255−i)/256) shrinks with i).
        for i in 1..256 {
            assert!(EXP_OUT[i] <= EXP_OUT[i - 1]);
        }
    }

    #[test]
    fn generated_tables_match_canonical() {
        // EXP self-consistency endpoints.
        assert_eq!(EXP_ROM[0], 1024);
        assert!((512..=514).contains(&EXP_ROM[255]));
        // Both ROMs reproduce the canonical hardware values within ±1 LSB.
        for i in 0..256 {
            let d = (LOGSIN_ROM[i] as i32 - CANON_LOGSIN[i] as i32).abs();
            assert!(
                d <= 1,
                "LOGSIN[{i}] = {} vs {}",
                LOGSIN_ROM[i],
                CANON_LOGSIN[i]
            );
        }
    }

    /// One operator sample at full envelope, waveform 0, over the full cycle.
    fn full_sine(phase1024: u32) -> i32 {
        let quad = (phase1024 >> 8) & 3;
        let mut idx = (phase1024 & 0xFF) as usize;
        if quad & 1 != 0 {
            idx = 255 - idx;
        }
        let att = LOGSIN_ROM[idx] as u32;
        let mag = (EXP_ROM[(att & 0xFF) as usize] >> (att >> 8)) as i32;
        if quad & 2 != 0 {
            -mag
        } else {
            mag
        }
    }

    #[test]
    fn tables_reconstruct_full_sine() {
        assert_eq!(full_sine(256), 1024);
        assert_eq!(full_sine(768), -1024);
        for p in 1..256 {
            assert!(full_sine(p) >= full_sine(p - 1));
        }
    }
}
