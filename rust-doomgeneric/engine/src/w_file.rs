use crate::game_state::GameState;
use crate::m_argv::M_CheckParm;
use crate::stdint_types::byte;
use crate::stdint_types::size_t;
use crate::w_file_stdc::STDC_WAD_FILE;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _wad_file_s {
    pub file_class: wad_file_class_t,
    pub mapped: Option<&'static [byte]>,
    pub length: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wad_file_class_t {
    pub OpenFile: Option<unsafe fn(&str) -> *mut wad_file_t>,
    pub CloseFile: Option<unsafe fn(*mut wad_file_t) -> ()>,
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

// SAFETY invariant relied on below: `W_CloseFile` has no callers anywhere in
// the engine, so every `wad_file_t` a class's `OpenFile` hands back lives for
// the rest of the process -- turning the raw pointer into `&'static` just
// makes that existing fact visible to the type system.
pub fn W_OpenFile(state: &mut GameState, path: &str) -> Option<&'static wad_file_t> {
    let mut result: *mut wad_file_t = ::core::ptr::null_mut::<wad_file_t>();
    let mut i: i32 = 0;
    if M_CheckParm(state, "-mmap") == 0 {
        result = unsafe { STDC_WAD_FILE.OpenFile.expect("non-null function pointer")(path) };
    } else {
        i = 0 as i32;
        while (i as usize) < state.w_file.wad_file_classes.len() {
            result = unsafe {
                state.w_file.wad_file_classes[i as usize]
                    .OpenFile
                    .expect("non-null function pointer")(path)
            };
            if !result.is_null() {
                break;
            }
            i += 1;
        }
    }
    if result.is_null() {
        None
    } else {
        Some(unsafe { &*result })
    }
}
pub unsafe fn W_CloseFile(mut wad: *mut wad_file_t) {
    (*wad)
        .file_class
        .CloseFile
        .expect("non-null function pointer")(wad);
}
// `buffer`/`buffer_len` are a raw pointer-and-length pair by nature (a
// destination byte buffer for a `read`-style call) -- unrelated to the
// `wad_file_t` reference-safety this pass is about, and out of scope here.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn W_Read(
    wad: &wad_file_t,
    offset: u32,
    buffer: *mut ::core::ffi::c_void,
    buffer_len: size_t,
) -> size_t {
    let wad_ptr = wad as *const wad_file_t as *mut wad_file_t;
    unsafe {
        wad.file_class.Read.expect("non-null function pointer")(wad_ptr, offset, buffer, buffer_len)
    }
}
