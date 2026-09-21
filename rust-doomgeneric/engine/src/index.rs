//! `.idx()`: an integer that is an array index, as an explicit conversion instead of `as usize`.
//!
//! The engine keeps many counts, coordinates and table positions as `i32` (or `i16`), the way
//! the C original did, and indexes with them. Those values are non-negative by construction; a
//! negative one turns into a huge `usize` and stops at the slice's bounds check. `.idx()` says
//! so, and checks it in debug builds where the panic can name the value.

/// An integer used as an index into a slice, `Vec` or array.
pub trait ToIndex: Copy + core::fmt::Display {
    /// The value as a `usize`. Must not be negative.
    fn idx(self) -> usize;
}

macro_rules! to_index {
    ($($t:ty),*) => {$(
        impl ToIndex for $t {
            #[inline(always)]
            #[allow(clippy::cast_sign_loss)]
            fn idx(self) -> usize {
                debug_assert!(self >= 0, "negative index {self}");
                self as usize
            }
        }
    )*};
}

to_index!(i8, i16, i32, i64, isize);
