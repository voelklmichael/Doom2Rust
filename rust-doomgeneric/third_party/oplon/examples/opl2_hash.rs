//! Deterministic OPL2 render checksum (used to pin bit-identity of the OPL2
//! path across refactors). Prints an FNV-1a hash of a fixed register sequence.
#![forbid(unsafe_code)]
use oplon::Opl2;
fn main() {
    let mut opl = Opl2::new(44100);
    for (r, v) in [
        (0x20u8, 0x21u8), (0x23, 0x21), (0x40, 0x10), (0x43, 0x00),
        (0x60, 0xF2), (0x63, 0xF2), (0x80, 0x05), (0x83, 0x05),
        (0xC0, 0x06), (0xA0, 0x98), (0xB0, 0x31),
        // a second voice with a different waveform (WF3) + feedback
        (0x28, 0x22), (0x2B, 0x22), (0x48, 0x00), (0x4B, 0x00),
        (0x68, 0xF4), (0x6B, 0xF4), (0x88, 0x03), (0x8B, 0x03),
        (0xE8, 0x02), (0xEB, 0x03), (0xC8, 0x0A), (0xA8, 0x40), (0xB8, 0x25),
    ] { opl.write_reg(r, v); }
    let mut h: u64 = 0xcbf29ce484222325;
    for i in 0..44100u32 {
        if i == 33000 { opl.write_reg(0xB0, 0x11); opl.write_reg(0xB8, 0x05); }
        let (l, r) = opl.render_frame();
        for b in l.to_le_bytes().iter().chain(r.to_le_bytes().iter()) {
            h ^= *b as u64; h = h.wrapping_mul(0x100000001b3);
        }
    }
    println!("{:016x}", h);
}
