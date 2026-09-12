use crate::src::m_misc::M_FileLength;
use crate::src::stdint_types::byte;
use crate::src::stdint_types::size_t;
use crate::src::w_file::{wad_file_class_t, wad_file_t};
use crate::src::z_zone::Z_Free;
use crate::src::z_zone::Z_Malloc;
use crate::src::z_zone::ZZoneState;
use crate::src::z_zone::PU_STATIC;
use std::io::{Read, Seek, SeekFrom};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct stdc_wad_file_t {
    pub wad: wad_file_t,
    pub fstream: *mut std::fs::File,
}
unsafe fn W_StdC_OpenFile(zone: &mut ZZoneState, path: &str) -> *mut wad_file_t {
    let mut result: *mut stdc_wad_file_t = ::core::ptr::null_mut::<stdc_wad_file_t>();
    let fstream = match std::fs::File::open(path) {
        Ok(fstream) => fstream,
        Err(_) => return ::core::ptr::null_mut::<wad_file_t>(),
    };
    let length = M_FileLength(&fstream) as u32;
    result = Z_Malloc(
        zone,
        ::core::mem::size_of::<stdc_wad_file_t>() as i32,
        PU_STATIC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut stdc_wad_file_t;
    (*result).wad.file_class = STDC_WAD_FILE;
    (*result).wad.mapped = ::core::ptr::null_mut::<byte>();
    (*result).wad.length = length;
    (*result).fstream = Box::into_raw(Box::new(fstream));
    return &raw mut (*result).wad;
}
unsafe fn W_StdC_CloseFile(zone: &mut ZZoneState, mut wad: *mut wad_file_t) {
    let mut stdc_wad: *mut stdc_wad_file_t = ::core::ptr::null_mut::<stdc_wad_file_t>();
    stdc_wad = wad as *mut stdc_wad_file_t;
    drop(Box::from_raw((*stdc_wad).fstream));
    Z_Free(zone, stdc_wad as *mut ::core::ffi::c_void);
}
pub unsafe fn W_StdC_Read(
    mut wad: *mut wad_file_t,
    offset: u32,
    buffer: *mut ::core::ffi::c_void,
    buffer_len: size_t,
) -> size_t {
    let mut stdc_wad: *mut stdc_wad_file_t = ::core::ptr::null_mut::<stdc_wad_file_t>();
    stdc_wad = wad as *mut stdc_wad_file_t;
    let fstream = &mut *(*stdc_wad).fstream;
    let _ = fstream.seek(SeekFrom::Start(offset as u64));
    let slice = ::core::slice::from_raw_parts_mut(buffer as *mut u8, buffer_len as usize);
    fstream.read(slice).unwrap_or(0) as size_t
}
pub const STDC_WAD_FILE: wad_file_class_t = wad_file_class_t {
    OpenFile: Some(W_StdC_OpenFile as unsafe fn(&mut ZZoneState, &str) -> *mut wad_file_t),
    CloseFile: Some(W_StdC_CloseFile as unsafe fn(&mut ZZoneState, *mut wad_file_t) -> ()),
    Read: Some(
        W_StdC_Read as unsafe fn(*mut wad_file_t, u32, *mut ::core::ffi::c_void, size_t) -> size_t,
    ),
};
