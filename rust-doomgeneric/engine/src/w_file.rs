use crate::m_misc::M_FileLength;
use crate::stdint_types::size_t;
use std::os::unix::fs::FileExt;

pub struct _wad_file_s {
    file: std::fs::File,
    pub length: u32,
}
pub type wad_file_t = _wad_file_s;

// SAFETY invariant relied on below: nothing in this codebase ever closes a
// wad_file_t once opened, so every one returned here lives for the rest of
// the process -- Box::leak just makes that existing fact visible to the type
// system, matching how `lumpinfo_t.wad_file` already stores it as `&'static`.
pub fn W_OpenFile(path: &str) -> Option<&'static wad_file_t> {
    let file = std::fs::File::open(path).ok()?;
    let length = M_FileLength(&file) as u32;
    Some(Box::leak(Box::new(wad_file_t { file, length })))
}
pub fn W_Read(wad: &wad_file_t, offset: u32, buffer: &mut [u8]) -> size_t {
    wad.file.read_at(buffer, offset as u64).unwrap_or(0) as size_t
}
