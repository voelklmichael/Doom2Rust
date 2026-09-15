pub struct ICdMusState {
    cd_Error: i32,
}

impl ICdMusState {
    pub const fn new() -> Self {
        ICdMusState { cd_Error: 0 }
    }
}

pub fn I_CDMusInit() -> i32 {
    0_i32
}
pub fn I_CDMusPlay() -> i32 {
    0_i32
}
pub fn I_CDMusStop() -> i32 {
    0_i32
}
pub fn I_CDMusResume() -> i32 {
    0_i32
}
pub fn I_CDMusSetVolume(state: &mut ICdMusState) -> i32 {
    state.cd_Error = 0_i32;
    0_i32
}
pub fn I_CDMusFirstTrack() -> i32 {
    0_i32
}
pub fn I_CDMusLastTrack() -> i32 {
    0_i32
}
pub fn I_CDMusTrackLength() -> i32 {
    0_i32
}
