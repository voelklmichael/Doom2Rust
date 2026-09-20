pub type Fixed = i32;
pub const INT_MAX: i32 = i32::MAX;
pub const INT_MIN: i32 = i32::MIN;
pub const FRACBITS: i32 = 16;
pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    ((i64::from(a) * i64::from(b)) >> FRACBITS) as Fixed
}
pub fn fixed_div(a: Fixed, b: Fixed) -> Fixed {
    if a.abs() >> 14 >= b.abs() {
        if a ^ b < 0 {
            INT_MIN
        } else {
            INT_MAX
        }
    } else {
        let result: i64 = (i64::from(a) << 16) / i64::from(b);
        result as Fixed
    }
}
pub const FRACUNIT: i32 = 1 << FRACBITS;
