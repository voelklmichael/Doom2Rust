use crate::game_state::GameState;
use sha1_smol::Sha1;
use crate::w_file::wad_file_t;
pub type sha1_digest_t = [u8; 20];
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
pub fn W_Checksum(state: &mut GameState) -> sha1_digest_t {
    let mut sha1 = Sha1::new();
    state.w_checksum.open_wadfiles.clear();
    for lump in &state.w_wad.lumpinfo {
        // Name is hashed with its NUL terminator, as the C SHA1_UpdateString did.
        sha1.update(lump.name.as_bytes());
        sha1.update(&[0]);
        let file_number = GetFileNumber(&mut state.w_checksum, lump.wad_file) as u32;
        sha1.update(&file_number.to_be_bytes());
        sha1.update(&(lump.position as u32).to_be_bytes());
        sha1.update(&(lump.size as u32).to_be_bytes());
    }
    sha1.digest().bytes()
}
