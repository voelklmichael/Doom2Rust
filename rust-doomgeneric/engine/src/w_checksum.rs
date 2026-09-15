use crate::game_state::GameState;
use crate::sha1::{
    sha1_context_s, sha1_context_t, SHA1_Final, SHA1_Init, SHA1_UpdateInt32, SHA1_UpdateString,
};
use crate::stdint_types::byte;
use crate::w_file::wad_file_t;
use crate::w_wad::lumpinfo_t;
use std::alloc::{alloc, realloc, Layout};
pub struct WChecksumState {
    open_wadfiles: *mut *mut wad_file_t,
    num_open_wadfiles: i32,
}

impl WChecksumState {
    pub const fn new() -> Self {
        WChecksumState {
            open_wadfiles: ::core::ptr::null::<*mut wad_file_t>() as *mut *mut wad_file_t,
            num_open_wadfiles: 0,
        }
    }
}

unsafe fn GetFileNumber(state: &mut WChecksumState, handle: &'static wad_file_t) -> i32 {
    let handle = handle as *const wad_file_t as *mut wad_file_t;
    let mut i: i32 = 0;
    let mut result: i32 = 0;
    i = 0_i32;
    while i < state.num_open_wadfiles {
        if *state.open_wadfiles.offset(i as isize) == handle {
            return i;
        }
        i += 1;
    }
    let new_layout =
        Layout::array::<*mut wad_file_t>((state.num_open_wadfiles + 1) as usize).unwrap();
    state.open_wadfiles = if state.open_wadfiles.is_null() {
        alloc(new_layout)
    } else {
        let old_layout =
            Layout::array::<*mut wad_file_t>(state.num_open_wadfiles as usize).unwrap();
        realloc(
            state.open_wadfiles as *mut u8,
            old_layout,
            new_layout.size(),
        )
    } as *mut *mut wad_file_t;
    let fresh0 = &mut (*state.open_wadfiles.offset(state.num_open_wadfiles as isize));
    *fresh0 = handle;
    result = state.num_open_wadfiles;
    state.num_open_wadfiles += 1;
    result
}
unsafe fn ChecksumAddLump(
    state: &mut WChecksumState,
    mut sha1_context: *mut sha1_context_t,
    mut lump: *mut lumpinfo_t,
) {
    let mut buf: [u8; 9] = [0; 9];
    let name_len = (*lump).name.len();
    buf[..name_len].copy_from_slice(&(*lump).name.as_bytes()[..name_len]);
    SHA1_UpdateString(sha1_context, buf.as_mut_ptr() as *mut ::core::ffi::c_char);
    SHA1_UpdateInt32(sha1_context, GetFileNumber(state, (*lump).wad_file) as u32);
    SHA1_UpdateInt32(sha1_context, (*lump).position as u32);
    SHA1_UpdateInt32(sha1_context, (*lump).size as u32);
}
pub unsafe fn W_Checksum(state: &mut GameState, mut digest: *mut byte) {
    let mut sha1_context: sha1_context_t = sha1_context_s {
        h0: 0,
        h1: 0,
        h2: 0,
        h3: 0,
        h4: 0,
        nblocks: 0,
        buf: [0; 64],
        count: 0,
    };
    let mut i: u32 = 0;
    SHA1_Init(&raw mut sha1_context);
    state.w_checksum.num_open_wadfiles = 0_i32;
    i = 0_u32;
    while i < state.w_wad.numlumps {
        let lump = state.w_wad.lumpinfo.as_mut_ptr().offset(i as isize);
        ChecksumAddLump(&mut state.w_checksum, &raw mut sha1_context, lump);
        i = i.wrapping_add(1);
    }
    SHA1_Final(digest, &raw mut sha1_context);
}
