use crate::src::game_state::GameState;
use crate::src::m_argv::M_CheckParm;
use crate::src::stdint_types::byte;
use crate::src::stdint_types::size_t;
use crate::src::w_file_stdc::STDC_WAD_FILE;
use crate::src::z_zone::ZZoneState;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _wad_file_s {
    pub file_class: wad_file_class_t,
    pub mapped: *mut byte,
    pub length: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wad_file_class_t {
    pub OpenFile: Option<unsafe fn(&mut ZZoneState, &str) -> *mut wad_file_t>,
    pub CloseFile: Option<unsafe fn(&mut ZZoneState, *mut wad_file_t) -> ()>,
    pub Read: Option<unsafe fn(*mut wad_file_t, u32, *mut ::core::ffi::c_void, size_t) -> size_t>,
}
pub type wad_file_t = _wad_file_s;

pub struct WFileState {
    wad_file_classes: [wad_file_class_t; 1],
}

impl WFileState {
    pub const fn new() -> Self {
        WFileState {
            wad_file_classes: [STDC_WAD_FILE],
        }
    }
}

pub unsafe fn W_OpenFile(state: &mut GameState, path: &str) -> *mut wad_file_t {
    let mut result: *mut wad_file_t = ::core::ptr::null_mut::<wad_file_t>();
    let mut i: i32 = 0;
    if M_CheckParm(state, "-mmap") == 0 {
        return STDC_WAD_FILE.OpenFile.expect("non-null function pointer")(
            &mut state.z_zone,
            path,
        );
    }
    result = ::core::ptr::null_mut::<wad_file_t>();
    i = 0 as i32;
    while (i as usize) < state.w_file.wad_file_classes.len() {
        result = state.w_file.wad_file_classes[i as usize]
            .OpenFile
            .expect("non-null function pointer")(&mut state.z_zone, path);
        if !result.is_null() {
            break;
        }
        i += 1;
    }
    return result;
}
pub unsafe fn W_CloseFile(zone: &mut ZZoneState, mut wad: *mut wad_file_t) {
    (*wad)
        .file_class
        .CloseFile
        .expect("non-null function pointer")(zone, wad);
}
pub unsafe fn W_Read(
    mut wad: *mut wad_file_t,
    mut offset: u32,
    mut buffer: *mut ::core::ffi::c_void,
    mut buffer_len: size_t,
) -> size_t {
    return (*wad)
        .file_class
        .Read
        .expect("non-null function pointer")(wad, offset, buffer, buffer_len);
}
