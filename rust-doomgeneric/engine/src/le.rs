//! Little-endian reads from the byte data of a WAD (lump directory, patches, texture tables, ...).
//!
//! Each function reads at a byte offset from anything that is a byte slice (`&[u8]`, a `Vec`, an
//! `Rc<[u8]>`, ...) and panics if the data is too short, like indexing does.

pub fn le_i16(bytes: impl AsRef<[u8]>, at: usize) -> i16 {
    let bytes = bytes.as_ref();
    i16::from_le_bytes([bytes[at], bytes[at + 1]])
}

pub fn le_u16(bytes: impl AsRef<[u8]>, at: usize) -> u16 {
    let bytes = bytes.as_ref();
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

pub fn le_i32(bytes: impl AsRef<[u8]>, at: usize) -> i32 {
    let bytes = bytes.as_ref();
    i32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

pub fn le_u32(bytes: impl AsRef<[u8]>, at: usize) -> u32 {
    let bytes = bytes.as_ref();
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_at_an_offset_in_little_endian_order() {
        let data = [0xff, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0xfe, 0xff];
        assert_eq!(le_u16(data, 1), 0x1234);
        assert_eq!(le_i32(data, 3), 0x1234_5678);
        assert_eq!(le_u32(data, 3), 0x1234_5678);
        assert_eq!(le_i16(data, 7), -2);
        assert_eq!(le_i32(data, 5), 0xfffe_1234_u32 as i32);
    }

    #[test]
    #[should_panic]
    fn a_read_past_the_end_panics() {
        le_i32([1, 2, 3], 0);
    }
}
