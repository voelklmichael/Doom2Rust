#[derive(Copy, Clone)]
pub struct cheatseq_t {
    sequence: &'static [u8],
    parameter_chars: usize,
    chars_read: usize,
    param_chars_read: usize,
    parameter_buf: [u8; 5],
}
impl cheatseq_t {
    pub const fn new(sequence: &'static str, parameter_chars: usize) -> Self {
        cheatseq_t {
            sequence: sequence.as_bytes(),
            parameter_chars,
            chars_read: 0,
            param_chars_read: 0,
            parameter_buf: [0; 5],
        }
    }

    /// The parameter characters typed after the sequence (valid right after a match).
    pub fn param(&self) -> &[u8] {
        &self.parameter_buf[..self.parameter_chars]
    }
}
pub fn cht_CheckCheat(cht: &mut cheatseq_t, key: u8) -> bool {
    if cht.chars_read < cht.sequence.len() {
        if key == cht.sequence[cht.chars_read] {
            cht.chars_read += 1;
        } else {
            cht.chars_read = 0;
        }
        cht.param_chars_read = 0;
    } else if cht.param_chars_read < cht.parameter_chars {
        cht.parameter_buf[cht.param_chars_read] = key;
        cht.param_chars_read += 1;
    }
    if cht.chars_read >= cht.sequence.len() && cht.param_chars_read >= cht.parameter_chars {
        cht.param_chars_read = 0;
        cht.chars_read = 0;
        return true;
    }
    false
}
