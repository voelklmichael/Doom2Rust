use crate::m_argv::M_CheckParm;
use crate::m_config::M_BindVariable_int;
use crate::m_config::M_BindVariable_string;

use crate::game_state::GameState;
use crate::sounds::sfxinfo_t;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum snddevice_t {
    SNDDEVICE_NONE = 0,
    SNDDEVICE_PCSPEAKER = 1,
    SNDDEVICE_ADLIB = 2,
    SNDDEVICE_SB = 3,
    SNDDEVICE_PAS = 4,
    SNDDEVICE_GUS = 5,
    SNDDEVICE_WAVEBLASTER = 6,
    SNDDEVICE_SOUNDCANVAS = 7,
    SNDDEVICE_GENMIDI = 8,
    SNDDEVICE_AWE32 = 9,
    SNDDEVICE_CD = 10,
}
fn snddevice_from_raw(v: i32) -> snddevice_t {
    match v {
        0 => snddevice_t::SNDDEVICE_NONE,
        1 => snddevice_t::SNDDEVICE_PCSPEAKER,
        2 => snddevice_t::SNDDEVICE_ADLIB,
        3 => snddevice_t::SNDDEVICE_SB,
        4 => snddevice_t::SNDDEVICE_PAS,
        5 => snddevice_t::SNDDEVICE_GUS,
        6 => snddevice_t::SNDDEVICE_WAVEBLASTER,
        7 => snddevice_t::SNDDEVICE_SOUNDCANVAS,
        8 => snddevice_t::SNDDEVICE_GENMIDI,
        9 => snddevice_t::SNDDEVICE_AWE32,
        10 => snddevice_t::SNDDEVICE_CD,
        n => panic!("invalid snddevice {n}"),
    }
}
#[derive(Copy, Clone)]
pub struct sound_module_t {
    pub sound_devices: &'static [snddevice_t],
    pub Init: Option<fn(bool) -> bool>,
    pub Shutdown: Option<fn()>,
    pub GetSfxLumpNum: Option<fn(*mut sfxinfo_t) -> i32>,
    pub Update: Option<fn()>,
    pub UpdateSoundParams: Option<fn(i32, i32, i32)>,
    pub StartSound: Option<fn(*mut sfxinfo_t, i32, i32, i32) -> i32>,
    pub StopSound: Option<fn(i32)>,
    pub SoundIsPlaying: Option<fn(i32) -> bool>,
    pub CacheSounds: Option<fn(*mut sfxinfo_t, i32)>,
}
#[derive(Copy, Clone)]
pub struct music_module_t {
    pub sound_devices: &'static [snddevice_t],
    pub Init: Option<fn() -> bool>,
    pub Shutdown: Option<fn()>,
    pub SetMusicVolume: Option<fn(i32)>,
    pub PauseMusic: Option<fn()>,
    pub ResumeMusic: Option<fn()>,
    pub RegisterSong: Option<fn(*mut ::core::ffi::c_void, i32) -> usize>,
    pub UnRegisterSong: Option<fn(usize)>,
    pub PlaySong: Option<fn(usize, bool)>,
    pub StopSong: Option<fn()>,
    pub MusicIsPlaying: Option<fn() -> bool>,
    pub Poll: Option<fn()>,
}
pub struct ISoundState {
    pub snd_samplerate: i32,
    pub snd_cachesize: i32,
    pub snd_maxslicetime_ms: i32,
    pub snd_musiccmd: Option<&'static str>,
    sound_module: Option<&'static sound_module_t>,
    music_module: Option<&'static music_module_t>,
    pub snd_musicdevice: i32,
    pub snd_sfxdevice: i32,
    snd_sbport: i32,
    snd_sbirq: i32,
    snd_sbdma: i32,
    snd_mport: i32,
    // Unused in this port: the M_BindVariable calls that would read/write
    // these live behind #ifdef FEATURE_SOUND in the original C, which isn't
    // defined here (sound_modules is a stub with no real backend). Kept as
    // GameState fields (rather than deleted) so the names survive if real
    // sound support is ever added.
    pub use_libsamplerate: i32,
    pub libsamplerate_scale: f32,
    // Always a single None entry -- see InitSfxModule, which never finds a
    // real backend and always leaves sound_module None. Kept as-is (dead
    // stub), same rationale as above, rather than deleted as a drive-by.
    sound_modules: [Option<&'static sound_module_t>; 1],
}

impl ISoundState {
    pub const fn new() -> Self {
        ISoundState {
            snd_samplerate: 44100,
            snd_cachesize: 64 * 1024 * 1024,
            snd_maxslicetime_ms: 28,
            snd_musiccmd: None,
            sound_module: None,
            music_module: None,
            snd_musicdevice: snddevice_t::SNDDEVICE_SB as i32,
            snd_sfxdevice: snddevice_t::SNDDEVICE_SB as i32,
            snd_sbport: 0,
            snd_sbirq: 0,
            snd_sbdma: 0,
            snd_mport: 0,
            use_libsamplerate: 0,
            libsamplerate_scale: 0.65,
            sound_modules: [None],
        }
    }
}
fn SndDeviceInList(device: snddevice_t, list: &[snddevice_t]) -> bool {
    list.contains(&device)
}
fn InitSfxModule(state: &mut ISoundState, use_sfx_prefix: bool) {
    state.sound_module = None;
    for i in 0..state.sound_modules.len() {
        let Some(module) = state.sound_modules[i] else {
            break;
        };
        if SndDeviceInList(
            snddevice_from_raw(state.snd_sfxdevice),
            module.sound_devices,
        ) && (module.Init.expect("non-null function pointer"))(use_sfx_prefix)
        {
            state.sound_module = Some(module);
            return;
        }
    }
}
pub fn I_InitSound(state: &mut GameState, mut use_sfx_prefix: bool) {
    let mut nosound: bool = false;
    let mut nosfx: bool = false;
    nosound = M_CheckParm(state, "-nosound") > 0_i32;
    nosfx = M_CheckParm(state, "-nosfx") > 0_i32;
    if !nosound && !state.i_video.screensaver_mode && !nosfx {
        InitSfxModule(&mut state.i_sound, use_sfx_prefix);
    }
}
pub fn I_ShutdownSound(state: &mut ISoundState) {
    if let Some(module) = state.sound_module {
        (module.Shutdown.expect("non-null function pointer"))();
    }
    if let Some(module) = state.music_module {
        (module.Shutdown.expect("non-null function pointer"))();
    }
}
pub fn I_GetSfxLumpNum(state: &mut ISoundState, sfxinfo: *mut sfxinfo_t) -> i32 {
    match state.sound_module {
        Some(module) => (module.GetSfxLumpNum.expect("non-null function pointer"))(sfxinfo),
        None => 0_i32,
    }
}
pub fn I_UpdateSound(state: &mut ISoundState) {
    if let Some(module) = state.sound_module {
        (module.Update.expect("non-null function pointer"))();
    }
    if let Some(module) = state.music_module {
        if let Some(poll) = module.Poll {
            poll();
        }
    }
}
fn CheckVolumeSeparation(mut vol: *mut i32, mut sep: *mut i32) {
    unsafe {
        if *sep < 0_i32 {
            *sep = 0_i32;
        } else if *sep > 254_i32 {
            *sep = 254_i32;
        }
        if *vol < 0_i32 {
            *vol = 0_i32;
        } else if *vol > 127_i32 {
            *vol = 127_i32;
        }
    }
}
pub fn I_UpdateSoundParams(state: &mut ISoundState, channel: i32, mut vol: i32, mut sep: i32) {
    if let Some(module) = state.sound_module {
        CheckVolumeSeparation(&raw mut vol, &raw mut sep);
        (module.UpdateSoundParams.expect("non-null function pointer"))(channel, vol, sep);
    }
}
pub fn I_StartSound(
    state: &mut ISoundState,
    sfxinfo: *mut sfxinfo_t,
    channel: i32,
    mut vol: i32,
    mut sep: i32,
) -> i32 {
    match state.sound_module {
        Some(module) => {
            CheckVolumeSeparation(&raw mut vol, &raw mut sep);
            (module.StartSound.expect("non-null function pointer"))(sfxinfo, channel, vol, sep)
        }
        None => 0_i32,
    }
}
pub fn I_StopSound(state: &mut ISoundState, channel: i32) {
    if let Some(module) = state.sound_module {
        (module.StopSound.expect("non-null function pointer"))(channel);
    }
}
pub fn I_SoundIsPlaying(state: &mut ISoundState, channel: i32) -> bool {
    match state.sound_module {
        Some(module) => (module.SoundIsPlaying.expect("non-null function pointer"))(channel),
        None => false,
    }
}
pub fn I_PrecacheSounds(state: &mut ISoundState, sounds: *mut sfxinfo_t, num_sounds: i32) {
    if let Some(module) = state.sound_module {
        if let Some(cache_sounds) = module.CacheSounds {
            cache_sounds(sounds, num_sounds);
        }
    }
}
pub fn I_InitMusic(state: &mut ISoundState) {
    if let Some(module) = state.music_module {
        (module.Init.expect("non-null function pointer"))();
    }
}
pub fn I_SetMusicVolume(state: &mut ISoundState, volume: i32) {
    if let Some(module) = state.music_module {
        (module.SetMusicVolume.expect("non-null function pointer"))(volume);
    }
}
pub fn I_PauseSong(state: &mut ISoundState) {
    if let Some(module) = state.music_module {
        (module.PauseMusic.expect("non-null function pointer"))();
    }
}
pub fn I_ResumeSong(state: &mut ISoundState) {
    if let Some(module) = state.music_module {
        (module.ResumeMusic.expect("non-null function pointer"))();
    }
}
pub fn I_RegisterSong(state: &mut ISoundState, data: *mut ::core::ffi::c_void, len: i32) -> usize {
    match state.music_module {
        Some(module) => (module.RegisterSong.expect("non-null function pointer"))(data, len),
        None => 0,
    }
}
pub fn I_UnRegisterSong(state: &mut ISoundState, handle: usize) {
    if let Some(module) = state.music_module {
        (module.UnRegisterSong.expect("non-null function pointer"))(handle);
    }
}
pub fn I_PlaySong(state: &mut ISoundState, handle: usize, looping: bool) {
    if let Some(module) = state.music_module {
        (module.PlaySong.expect("non-null function pointer"))(handle, looping);
    }
}
pub fn I_StopSong(state: &mut ISoundState) {
    if let Some(module) = state.music_module {
        (module.StopSong.expect("non-null function pointer"))();
    }
}
pub fn I_MusicIsPlaying(state: &mut ISoundState) -> bool {
    match state.music_module {
        Some(module) => (module.MusicIsPlaying.expect("non-null function pointer"))(),
        None => false,
    }
}
pub fn I_BindSoundVariables(state: &mut GameState) {
    M_BindVariable_int(
        &mut state.m_config,
        "snd_musicdevice",
        &mut state.i_sound.snd_musicdevice,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "snd_sfxdevice",
        &mut state.i_sound.snd_sfxdevice,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "snd_sbport",
        &mut state.i_sound.snd_sbport,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "snd_sbirq",
        &mut state.i_sound.snd_sbirq,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "snd_sbdma",
        &mut state.i_sound.snd_sbdma,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "snd_mport",
        &mut state.i_sound.snd_mport,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "snd_maxslicetime_ms",
        &mut state.i_sound.snd_maxslicetime_ms,
    );
    M_BindVariable_string(
        &mut state.m_config,
        "snd_musiccmd",
        &mut state.i_sound.snd_musiccmd,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "snd_samplerate",
        &mut state.i_sound.snd_samplerate,
    );
    M_BindVariable_int(
        &mut state.m_config,
        "snd_cachesize",
        &mut state.i_sound.snd_cachesize,
    );
}
