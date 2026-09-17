use crate::game_state::GameState;
use crate::sha1::{
    sha1_context_s, sha1_context_t, SHA1_Final, SHA1_Init, SHA1_UpdateInt32, SHA1_UpdateString,
};
use crate::stdint_types::byte;
use crate::w_file::wad_file_t;
use crate::w_wad::lumpinfo_t;
pub struct WChecksumState {
    open_wadfiles: Vec<&'static wad_file_t>,
}

impl WChecksumState {
    pub const fn new() -> Self {
        WChecksumState {
            open_wadfiles: Vec::new(),
        }
    }
}

fn GetFileNumber(state: &mut WChecksumState, handle: &'static wad_file_t) -> i32 {
    if let Some(pos) = state
        .open_wadfiles
        .iter()
        .position(|&f| ::core::ptr::eq(f, handle))
    {
        return pos as i32;
    }
    state.open_wadfiles.push(handle);
    (state.open_wadfiles.len() - 1) as i32
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
    state.w_checksum.open_wadfiles.clear();
    i = 0_u32;
    while i < state.w_wad.numlumps {
        let lump = state.w_wad.lumpinfo.as_mut_ptr().offset(i as isize);
        ChecksumAddLump(&mut state.w_checksum, &raw mut sha1_context, lump);
        i = i.wrapping_add(1);
    }
    SHA1_Final(digest, &raw mut sha1_context);
}
