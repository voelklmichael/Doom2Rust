//! A fixed-width, C-string-flavored byte buffer, used everywhere this codebase
//! has a `char name[N]`-shaped field transpiled from C (WAD lump names, texture
//! names, sound/switch lump names, savegame version strings, ...).
//!
//! Mirrors the C field's exact on-disk/in-memory layout (`#[repr(transparent)]`
//! over `[u8; N]`), truncating/padding at construction the way the original
//! fixed-width `char[N]` field did, rather than growing like a `String`. Some of
//! these widths are load-bearing for file format compatibility (WAD lump
//! directory entries are exactly 8 bytes on disk), so this type intentionally
//! never becomes a `String` outright.
//!
//! Not every `char[N]` field in this codebase uses this type -- only ones acting
//! as a name/identifier compared or hashed as a unit. A buffer that's just
//! scratch space for `snprintf`-style formatting stays a plain array.

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FixedCStr<const N: usize>(pub [u8; N]);

impl<const N: usize> FixedCStr<N> {
    pub const fn from_array(bytes: [u8; N]) -> Self {
        FixedCStr(bytes)
    }

    /// Truncates to N bytes if longer; pads with NUL bytes if shorter -- the
    /// same behavior the original `char[N]` field had under `strncpy`/manual
    /// byte-copy construction.
    pub fn new(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    pub fn from_bytes(b: &[u8]) -> Self {
        let mut buf = [0u8; N];
        let len = b.len().min(N);
        buf[..len].copy_from_slice(&b[..len]);
        FixedCStr(buf)
    }

    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.0
    }

    pub const fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    /// The logical length: up to the first NUL byte, or N if there is none
    /// (matches how the C code treated an exactly-N-byte name with no room
    /// left for a terminator).
    pub fn len(&self) -> usize {
        self.0.iter().position(|&b| b == 0).unwrap_or(N)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Lossy conversion for display/formatting -- WAD data isn't guaranteed to
    /// be valid UTF-8 (or even ASCII), so this never panics.
    pub fn as_str(&self) -> ::std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.0[..self.len()])
    }

    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }

    /// Matches C's `strncasecmp(self, other, N) == 0` exactly: compares up to N
    /// bytes, stopping early (as equal) if both sides hit a NUL at the same
    /// position, and never looking at `other` past N bytes even if it's longer.
    pub fn eq_bytes_ignore_ascii_case(&self, other_bytes: &[u8]) -> bool {
        for i in 0..N {
            let a = self.0[i];
            let b = other_bytes.get(i).copied().unwrap_or(0);
            if a == 0 && b == 0 {
                return true;
            }
            if a.to_ascii_lowercase() != b.to_ascii_lowercase() {
                return false;
            }
        }
        true
    }

    pub fn eq_str_ignore_ascii_case(&self, other: &str) -> bool {
        self.eq_bytes_ignore_ascii_case(other.as_bytes())
    }

    /// Matches C's `strncasecmp(self, other, n) == 0` for n < N (e.g. comparing
    /// just a sprite name's 4-character prefix, ignoring the frame/rotation
    /// bytes that follow it).
    pub fn eq_bytes_ignore_ascii_case_n(&self, other_bytes: &[u8], n: usize) -> bool {
        for i in 0..n.min(N) {
            let a = self.0[i];
            let b = other_bytes.get(i).copied().unwrap_or(0);
            if a == 0 && b == 0 {
                return true;
            }
            if a.to_ascii_lowercase() != b.to_ascii_lowercase() {
                return false;
            }
        }
        true
    }
}

impl<const N: usize> ::core::ops::Index<usize> for FixedCStr<N> {
    type Output = u8;
    fn index(&self, i: usize) -> &u8 {
        &self.0[i]
    }
}

impl<const N: usize> ::core::ops::IndexMut<usize> for FixedCStr<N> {
    fn index_mut(&mut self, i: usize) -> &mut u8 {
        &mut self.0[i]
    }
}

impl<const N: usize> ::core::fmt::Debug for FixedCStr<N> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl<const N: usize> Default for FixedCStr<N> {
    fn default() -> Self {
        FixedCStr([0u8; N])
    }
}
