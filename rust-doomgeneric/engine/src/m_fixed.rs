pub type Fixed = i32;
pub const INT_MAX: i32 = i32::MAX;
pub const INT_MIN: i32 = i32::MIN;
pub const FRACBITS: i32 = 16;
pub fn fixed_mul(a: Fixed, b: Fixed) -> Fixed {
    ((a as i64 * b as i64) >> FRACBITS) as Fixed
}
pub fn fixed_div(a: Fixed, b: Fixed) -> Fixed {
    if a.abs() >> 14 >= b.abs() {
        if a ^ b < 0 {
            INT_MIN
        } else {
            INT_MAX
        }
    } else {
        let result: i64 = ((a as i64) << 16) / b as i64;
        result as Fixed
    }
}
pub const FRACUNIT: i32 = 1 << FRACBITS;
