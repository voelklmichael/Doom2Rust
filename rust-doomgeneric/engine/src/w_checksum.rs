use crate::filesystem::FileId;
use crate::game_state::GameState;
use alloc::vec::Vec;
use sha1_smol::Sha1;
pub type Sha1Digest = [u8; 20];
pub struct WChecksumState {
    open_wadfiles: Vec<FileId>,
}

impl Default for WChecksumState {
    fn default() -> Self {
        Self::new()
    }
}

impl WChecksumState {
    pub const fn new() -> Self {
        WChecksumState {
            open_wadfiles: Vec::new(),
        }
    }
}

fn get_file_number(state: &mut WChecksumState, handle: FileId) -> i32 {
    if let Some(pos) = state.open_wadfiles.iter().position(|&f| f == handle) {
        return pos as i32;
    }
    state.open_wadfiles.push(handle);
    (state.open_wadfiles.len() - 1) as i32
}
pub fn checksum(state: &mut GameState) -> Sha1Digest {
    let mut sha1 = Sha1::new();
    state.w_checksum.open_wadfiles.clear();
    for lump in &state.w_wad.lumpinfo {
        // Name is hashed with its NUL terminator, as the C SHA1_UpdateString did.
        sha1.update(lump.name.as_bytes());
        sha1.update(&[0]);
        let file_number = get_file_number(&mut state.w_checksum, lump.wad_file) as u32;
        sha1.update(&file_number.to_be_bytes());
        sha1.update(&(lump.position as u32).to_be_bytes());
        sha1.update(&(lump.size as u32).to_be_bytes());
    }
    sha1.digest().bytes()
}
