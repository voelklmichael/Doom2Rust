use crate::fixed_cstr::FixedCStr;
use std::io::Write;
pub const EISDIR: i32 = 21;
pub const DIR_SEPARATOR: i32 = '/' as i32;
pub const DIR_SEPARATOR_S: &str = "/";
pub fn M_MakeDirectory(path: &str) {
    use std::os::unix::fs::DirBuilderExt;
    let _ = std::fs::DirBuilder::new().mode(0o755).create(path);
}
pub fn M_FileExists(filename: &str) -> bool {
    match std::fs::File::open(filename) {
        Ok(_) => true,
        Err(e) => e.raw_os_error() == Some(EISDIR),
    }
}
pub fn M_FileLength(file: &std::fs::File) -> i64 {
    file.metadata().map(|m| m.len() as i64).unwrap_or(0)
}
pub fn M_WriteFile(name: &str, source: &[u8]) -> bool {
    let mut handle = match std::fs::File::create(name) {
        Ok(handle) => handle,
        Err(_) => return false,
    };
    handle.write_all(source).is_ok()
}
pub fn M_TempFile(s: &str) -> String {
    format!("/tmp{}{}", DIR_SEPARATOR_S, s)
}
fn m_strtoint_digit_prefix(s: &str, radix: u32) -> Option<i32> {
    let end = s
        .find(|c: char| !c.is_digit(radix))
        .unwrap_or(s.len());
    if end == 0 {
        None
    } else {
        i32::from_str_radix(&s[..end], radix).ok()
    }
}
pub fn M_StrToInt(str: &str, result: &mut i32) -> bool {
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
pub fn M_ExtractFileBase(path: &str, dest: &mut FixedCStr<8>) {
    let filename = match path.rfind('/') {
        Some(idx) => &path[idx + 1..],
        None => path,
    };
    let base = match filename.find('.') {
        Some(idx) => &filename[..idx],
        None => filename,
    };
    let mut buf = [0u8; 8];
    let mut length = 0usize;
    for &b in base.as_bytes() {
        if length >= 8 {
            let truncated = String::from_utf8_lossy(&buf[..length.min(8)]);
            println!("Warning: Truncated '{}' lump name to '{:.8}'.", filename, truncated);
            break;
        }
        buf[length] = b.to_ascii_uppercase();
        length += 1;
    }
    *dest = FixedCStr(buf);
}
pub fn M_ForceUppercase(text: &str) -> String {
    text.to_uppercase()
}
pub fn M_StrCaseStr<'a>(haystack: &'a str, needle: &str) -> Option<&'a str> {
    let haystack_lower = haystack.to_lowercase();
    let needle_lower = needle.to_lowercase();
    haystack_lower.find(&needle_lower).map(|i| &haystack[i..])
}
pub fn M_StringDuplicate(orig: &str) -> String {
    orig.to_string()
}
pub fn M_StringStartsWith(s: &str, prefix: &str) -> bool {
    s.len() > prefix.len() && s.starts_with(prefix)
}
pub fn M_StringEndsWith(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}
