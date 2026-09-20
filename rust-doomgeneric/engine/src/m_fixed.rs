use core::ops::{
    Add, AddAssign, BitAnd, BitXor, Div, DivAssign, Mul, MulAssign, Neg, Rem, Shl, ShlAssign, Shr,
    ShrAssign, Sub, SubAssign,
};

pub const FRACBITS: u32 = 16;
pub const INT_MAX: i32 = i32::MAX;
pub const INT_MIN: i32 = i32::MIN;

/// A 16.16 fixed-point number (`fixed_t` in the original).
///
/// Addition, subtraction and negation work on two `Fixed` values, and scaling by a plain integer
/// uses `*` and `/`. Multiplying or dividing two `Fixed` values is deliberately NOT an operator:
/// use [`fixed_mul`] / [`fixed_div`], so the shift is always visible at the call site.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Fixed(pub i32);

pub const FRACUNIT: Fixed = Fixed(1 << FRACBITS);

impl Fixed {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(i32::MAX);
    pub const MIN: Self = Self(i32::MIN);

    /// The raw 16.16 bit pattern.
    #[inline(always)]
    pub const fn to_bits(self) -> i32 {
        self.0
    }

    /// `n` as a fixed-point value (`n << FRACBITS`).
    #[inline(always)]
    pub const fn from_int(n: i32) -> Self {
        Self(n << FRACBITS)
    }

    /// The integer part, rounded towards negative infinity (`>> FRACBITS`).
    #[inline(always)]
    pub const fn to_int(self) -> i32 {
        self.0 >> FRACBITS
    }

    /// The blockmap cell this coordinate falls in (`>> MAPBLOCKSHIFT`, i.e. 7 bits above the
    /// integer part).
    #[inline(always)]
    pub const fn to_block(self) -> i32 {
        self.0 >> (FRACBITS + 7)
    }

    /// The midpoint of two values (rounding towards negative infinity).
    #[inline(always)]
    pub const fn midpoint(self, other: Self) -> Self {
        Self(i32::midpoint(self.0, other.0))
    }

    #[inline(always)]
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }
}

impl Add for Fixed {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Fixed {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Neg for Fixed {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl AddAssign for Fixed {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign for Fixed {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

/// Scale by a plain integer.
impl Mul<i32> for Fixed {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: i32) -> Self {
        Self(self.0 * rhs)
    }
}

impl Mul<Fixed> for i32 {
    type Output = Fixed;
    #[inline(always)]
    fn mul(self, rhs: Fixed) -> Fixed {
        Fixed(self * rhs.0)
    }
}

impl MulAssign<i32> for Fixed {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: i32) {
        self.0 *= rhs;
    }
}

impl Div<i32> for Fixed {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: i32) -> Self {
        Self(self.0 / rhs)
    }
}

impl DivAssign<i32> for Fixed {
    #[inline(always)]
    fn div_assign(&mut self, rhs: i32) {
        self.0 /= rhs;
    }
}

/// Truncating integer quotient of two fixed-point values (a plain number, not a `Fixed`).
impl Div for Fixed {
    type Output = i32;
    #[inline(always)]
    fn div(self, rhs: Self) -> i32 {
        self.0 / rhs.0
    }
}

impl Rem<i32> for Fixed {
    type Output = i32;
    #[inline(always)]
    fn rem(self, rhs: i32) -> i32 {
        self.0 % rhs
    }
}

impl Shl<u32> for Fixed {
    type Output = Self;
    #[inline(always)]
    fn shl(self, rhs: u32) -> Self {
        Self(self.0 << rhs)
    }
}

impl ShlAssign<u32> for Fixed {
    #[inline(always)]
    fn shl_assign(&mut self, rhs: u32) {
        self.0 <<= rhs;
    }
}

impl BitXor for Fixed {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl Shr<u32> for Fixed {
    type Output = Self;
    #[inline(always)]
    fn shr(self, rhs: u32) -> Self {
        Self(self.0 >> rhs)
    }
}

impl ShrAssign<u32> for Fixed {
    #[inline(always)]
    fn shr_assign(&mut self, rhs: u32) {
        self.0 >>= rhs;
    }
}

impl BitAnd<i32> for Fixed {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: i32) -> Self {
        Self(self.0 & rhs)
    }
}

#[inline(always)]
pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    Fixed(((i64::from(a.0) * i64::from(b.0)) >> FRACBITS) as i32)
}

pub fn fixed_div(a: Fixed, b: Fixed) -> Fixed {
    if a.0.abs() >> 14 >= b.0.abs() {
        if a.0 ^ b.0 < 0 {
            Fixed::MIN
        } else {
            Fixed::MAX
        }
    } else {
        let result: i64 = (i64::from(a.0) << 16) / i64::from(b.0);
        Fixed(result as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_conversion_rounds_towards_negative_infinity() {
        assert_eq!(Fixed::from_int(3).to_int(), 3);
        assert_eq!(Fixed::from_int(-3).to_int(), -3);
        // `>> FRACBITS` floors, unlike a division by FRACUNIT.
        assert_eq!(Fixed(-1).to_int(), -1);
        assert_eq!(Fixed(-1) / FRACUNIT, 0);
        assert_eq!((FRACUNIT * 2 + Fixed(1)).to_int(), 2);
    }

    #[test]
    fn scaling_by_an_integer_is_plain_arithmetic() {
        assert_eq!(FRACUNIT * 4, Fixed::from_int(4));
        assert_eq!(3 * FRACUNIT, Fixed::from_int(3));
        assert_eq!(FRACUNIT / 8, Fixed(0x2000));
        assert_eq!(-FRACUNIT + FRACUNIT, Fixed::ZERO);
        assert_eq!(FRACUNIT >> 1, Fixed(0x8000));
        assert_eq!(Fixed(0x2000) << 2, FRACUNIT / 2);
    }

    #[test]
    fn fixed_mul_and_div_keep_sixteen_fraction_bits() {
        let half = FRACUNIT / 2;
        assert_eq!(fixed_mul(half, half), FRACUNIT / 4);
        assert_eq!(fixed_mul(Fixed::from_int(-3), half), Fixed(-0x18000));
        assert_eq!(fixed_div(FRACUNIT, Fixed::from_int(4)), FRACUNIT / 4);
        assert_eq!(
            fixed_div(Fixed::from_int(-6), Fixed::from_int(3)),
            Fixed::from_int(-2)
        );
    }

    #[test]
    fn fixed_div_saturates_when_the_quotient_would_overflow() {
        assert_eq!(fixed_div(Fixed::from_int(1 << 14), Fixed(1)), Fixed::MAX);
        assert_eq!(fixed_div(Fixed::from_int(-(1 << 14)), Fixed(1)), Fixed::MIN);
    }

    #[test]
    fn blockmap_cell_is_seven_bits_above_the_integer_part() {
        assert_eq!(Fixed::from_int(127).to_block(), 0);
        assert_eq!(Fixed::from_int(128).to_block(), 1);
        assert_eq!(Fixed::from_int(-1).to_block(), -1);
    }

    #[test]
    fn midpoint_and_abs() {
        assert_eq!(
            Fixed::from_int(2).midpoint(Fixed::from_int(6)),
            Fixed::from_int(4)
        );
        assert_eq!(Fixed::from_int(-5).abs(), Fixed::from_int(5));
    }
}
