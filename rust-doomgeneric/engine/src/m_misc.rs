use crate::fixed_cstr::FixedCStr;
use crate::platform::DoomPlatform;
use alloc::string::String;
fn m_strtoint_digit_prefix(s: &str, radix: u32) -> Option<i32> {
    let end = s.find(|c: char| !c.is_digit(radix)).unwrap_or(s.len());
    if end == 0 {
        None
    } else {
        i32::from_str_radix(&s[..end], radix).ok()
    }
}
pub fn str_to_int(str: &str, result: &mut i32) -> bool {
    let trimmed = str.trim_start();
    let (sign, unsigned) = match trimmed.strip_prefix('-') {
        Some(rest) => (-1, rest),
        None => (1, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    let parsed = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
        .and_then(|rest| m_strtoint_digit_prefix(rest, 16))
        .or_else(|| {
            unsigned
                .strip_prefix('0')
                .and_then(|rest| m_strtoint_digit_prefix(rest, 8))
        })
        .or_else(|| m_strtoint_digit_prefix(unsigned, 10));
    match parsed {
        Some(v) => {
            *result = sign * v;
            true
        }
        None => false,
    }
}
pub fn extract_file_base(platform: &mut dyn DoomPlatform, path: &str, dest: &mut FixedCStr<8>) {
    let filename = match path.rfind('/') {
        Some(idx) => &path[idx + 1..],
        None => path,
    };
    let base = match filename.find('.') {
        Some(idx) => &filename[..idx],
        None => filename,
    };
    let mut buf = [0u8; 8];
    for (length, &b) in base.as_bytes().iter().enumerate() {
        if length >= 8 {
            let truncated = String::from_utf8_lossy(&buf[..length.min(8)]);
            doom_println!(
                platform,
                "Warning: Truncated '{}' lump name to '{:.8}'.",
                filename,
                truncated
            );
            break;
        }
        buf[length] = b.to_ascii_uppercase();
    }
    *dest = FixedCStr(buf);
}
pub fn string_ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}
