use crate::doomdef::false_0;
use crate::doomdef::true_0;
use crate::stdint_types::size_t;
use crate::mem_compat::memcpy;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cheatseq_t {
    pub sequence: [::core::ffi::c_char; 25],
    pub sequence_len: size_t,
    pub parameter_chars: i32,
    pub chars_read: size_t,
    pub param_chars_read: i32,
    pub parameter_buf: [::core::ffi::c_char; 5],
}
fn cheat_sequence_len(sequence: &[::core::ffi::c_char; 25]) -> size_t {
    sequence.iter().position(|&c| c == 0).unwrap_or(sequence.len())
}
pub unsafe fn cht_CheckCheat(mut cht: *mut cheatseq_t, mut key: ::core::ffi::c_char) -> i32 {
    if (*cht).parameter_chars > 0_i32 && cheat_sequence_len(&(*cht).sequence) < (*cht).sequence_len
    {
        return false_0;
    }
    if (*cht).chars_read < cheat_sequence_len(&(*cht).sequence) {
        if key as i32 == (*cht).sequence[(*cht).chars_read] as i32 {
            (*cht).chars_read = (*cht).chars_read.wrapping_add(1);
        } else {
            (*cht).chars_read = 0 as size_t;
        }
        (*cht).param_chars_read = 0_i32;
    } else if (*cht).param_chars_read < (*cht).parameter_chars {
        (*cht).parameter_buf[(*cht).param_chars_read as usize] = key;
        (*cht).param_chars_read += 1;
    }
    if (*cht).chars_read >= cheat_sequence_len(&(*cht).sequence)
        && (*cht).param_chars_read >= (*cht).parameter_chars
    {
        (*cht).param_chars_read = 0_i32;
        (*cht).chars_read = (*cht).param_chars_read as size_t;
        return true_0;
    }
    false_0
}
pub unsafe fn cht_GetParam(mut cht: *mut cheatseq_t, mut buffer: *mut ::core::ffi::c_char) {
    memcpy(
        buffer as *mut ::core::ffi::c_void,
        &raw mut (*cht).parameter_buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
        (*cht).parameter_chars as size_t,
    );
}
