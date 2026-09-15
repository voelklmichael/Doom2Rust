pub type fixed_t = i32;
pub const INT_MAX: i32 = i32::MAX;
pub const INT_MIN: i32 = i32::MIN;
pub const FRACBITS: i32 = 16;
pub fn FixedMul(a: fixed_t, b: fixed_t) -> fixed_t {
    ((a as i64 * b as i64) >> FRACBITS) as fixed_t
}
pub fn FixedDiv(a: fixed_t, b: fixed_t) -> fixed_t {
    if a.abs() >> 14 >= b.abs() {
        if a ^ b < 0 {
            INT_MIN
        } else {
            INT_MAX
        }
    } else {
        let result: i64 = ((a as i64) << 16_i32) / b as i64;
        result as fixed_t
    }
}
pub const FRACUNIT: i32 = 1 << FRACBITS;
