use crate::src::m_argv::M_CheckParm;
use crate::src::m_config::M_BindVariable;

use crate::src::doomdef::boolean;
use crate::src::doomdef::NULL;
use crate::src::game_state::GameState;
use crate::src::sounds::sfxinfo_t;
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
#[repr(C)]
pub struct sound_module_t {
    pub sound_devices: *mut snddevice_t,
    pub num_sound_devices: i32,
    pub Init: Option<unsafe fn(boolean) -> boolean>,
    pub Shutdown: Option<unsafe fn() -> ()>,
    pub GetSfxLumpNum: Option<unsafe fn(*mut sfxinfo_t) -> i32>,
    pub Update: Option<unsafe fn() -> ()>,
    pub UpdateSoundParams: Option<unsafe fn(i32, i32, i32) -> ()>,
    pub StartSound: Option<unsafe fn(*mut sfxinfo_t, i32, i32, i32) -> i32>,
    pub StopSound: Option<unsafe fn(i32) -> ()>,
    pub SoundIsPlaying: Option<unsafe fn(i32) -> boolean>,
    pub CacheSounds: Option<unsafe fn(*mut sfxinfo_t, i32) -> ()>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct music_module_t {
    pub sound_devices: *mut snddevice_t,
    pub num_sound_devices: i32,
    pub Init: Option<unsafe fn() -> boolean>,
    pub Shutdown: Option<unsafe fn() -> ()>,
    pub SetMusicVolume: Option<unsafe fn(i32) -> ()>,
    pub PauseMusic: Option<unsafe fn() -> ()>,
    pub ResumeMusic: Option<unsafe fn() -> ()>,
    pub RegisterSong:
        Option<unsafe fn(*mut ::core::ffi::c_void, i32) -> *mut ::core::ffi::c_void>,
    pub UnRegisterSong: Option<unsafe fn(*mut ::core::ffi::c_void) -> ()>,
    pub PlaySong: Option<unsafe fn(*mut ::core::ffi::c_void, boolean) -> ()>,
    pub StopSong: Option<unsafe fn() -> ()>,
    pub MusicIsPlaying: Option<unsafe fn() -> boolean>,
    pub Poll: Option<unsafe fn() -> ()>,
}
pub struct ISoundState {
    pub snd_samplerate: i32,
    pub snd_cachesize: i32,
    pub snd_maxslicetime_ms: i32,
    pub snd_musiccmd: Option<&'static str>,
    sound_module: *mut sound_module_t,
    music_module: *mut music_module_t,
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
    // Always a single null entry -- see InitSfxModule, which never finds a
    // real backend and always leaves sound_module null. Kept as-is (dead
    // stub), same rationale as above, rather than deleted as a drive-by.
    sound_modules: [*mut sound_module_t; 1],
}

impl ISoundState {
    pub const fn new() -> Self {
        ISoundState {
            snd_samplerate: 44100,
            snd_cachesize: 64 * 1024 * 1024,
            snd_maxslicetime_ms: 28,
            snd_musiccmd: None,
            sound_module: ::core::ptr::null::<sound_module_t>() as *mut sound_module_t,
            music_module: ::core::ptr::null::<music_module_t>() as *mut music_module_t,
            snd_musicdevice: snddevice_t::SNDDEVICE_SB as i32,
            snd_sfxdevice: snddevice_t::SNDDEVICE_SB as i32,
            snd_sbport: 0,
            snd_sbirq: 0,
            snd_sbdma: 0,
            snd_mport: 0,
            use_libsamplerate: 0,
            libsamplerate_scale: 0.65,
            sound_modules: [::core::ptr::null::<sound_module_t>() as *mut sound_module_t],
        }
    }
}
unsafe fn SndDeviceInList(
    mut device: snddevice_t,
    mut list: *mut snddevice_t,
    mut len: i32,
) -> bool {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < len {
        if device as u32 == *list.offset(i as isize) as u32 {
            return true;
        }
        i += 1;
    }
    return false;
}
unsafe fn InitSfxModule(state: &mut ISoundState, mut use_sfx_prefix: bool) {
    let mut i: i32 = 0;
    state.sound_module = ::core::ptr::null_mut::<sound_module_t>();
    i = 0 as i32;
    while !state.sound_modules[i as usize].is_null() {
        if SndDeviceInList(
            snddevice_from_raw(state.snd_sfxdevice),
            (*state.sound_modules[i as usize]).sound_devices,
            (*state.sound_modules[i as usize]).num_sound_devices,
        ) {
            if (*state.sound_modules[i as usize])
                .Init
                .expect("non-null function pointer")(use_sfx_prefix as i32 as boolean)
                != 0
            {
                state.sound_module = state.sound_modules[i as usize];
                return;
            }
        }
        i += 1;
    }
}
pub unsafe fn I_InitSound(state: &mut GameState, mut use_sfx_prefix: bool) {
    let mut nosound: bool = false;
    let mut nosfx: bool = false;
    nosound = M_CheckParm(state, "-nosound") > 0 as i32;
    nosfx = M_CheckParm(state, "-nosfx") > 0 as i32;
    if !nosound && !state.i_video.screensaver_mode {
        if !nosfx {
            InitSfxModule(&mut state.i_sound, use_sfx_prefix);
        }
    }
}
pub unsafe fn I_ShutdownSound(state: &mut ISoundState) {
    if !state.sound_module.is_null() {
        (*state.sound_module)
            .Shutdown
            .expect("non-null function pointer")();
    }
    if !state.music_module.is_null() {
        (*state.music_module)
            .Shutdown
            .expect("non-null function pointer")();
    }
}
pub unsafe fn I_GetSfxLumpNum(state: &mut ISoundState, mut sfxinfo: *mut sfxinfo_t) -> i32 {
    if !state.sound_module.is_null() {
        return (*state.sound_module)
            .GetSfxLumpNum
            .expect("non-null function pointer")(sfxinfo);
    } else {
        return 0 as i32;
    };
}
pub unsafe fn I_UpdateSound(state: &mut ISoundState) {
    if !state.sound_module.is_null() {
        (*state.sound_module)
            .Update
            .expect("non-null function pointer")();
    }
    if !state.music_module.is_null() && (*state.music_module).Poll.is_some() {
        (*state.music_module)
            .Poll
            .expect("non-null function pointer")();
    }
}
unsafe fn CheckVolumeSeparation(mut vol: *mut i32, mut sep: *mut i32) {
    if *sep < 0 as i32 {
        *sep = 0 as i32;
    } else if *sep > 254 as i32 {
        *sep = 254 as i32;
    }
    if *vol < 0 as i32 {
        *vol = 0 as i32;
    } else if *vol > 127 as i32 {
        *vol = 127 as i32;
    }
}
pub unsafe fn I_UpdateSoundParams(
    state: &mut ISoundState,
    mut channel: i32,
    mut vol: i32,
    mut sep: i32,
) {
    if !state.sound_module.is_null() {
        CheckVolumeSeparation(&raw mut vol, &raw mut sep);
        (*state.sound_module)
            .UpdateSoundParams
            .expect("non-null function pointer")(channel, vol, sep);
    }
}
pub unsafe fn I_StartSound(
    state: &mut ISoundState,
    mut sfxinfo: *mut sfxinfo_t,
    mut channel: i32,
    mut vol: i32,
    mut sep: i32,
) -> i32 {
    if !state.sound_module.is_null() {
        CheckVolumeSeparation(&raw mut vol, &raw mut sep);
        return (*state.sound_module)
            .StartSound
            .expect("non-null function pointer")(sfxinfo, channel, vol, sep);
    } else {
        return 0 as i32;
    };
}
pub unsafe fn I_StopSound(state: &mut ISoundState, mut channel: i32) {
    if !state.sound_module.is_null() {
        (*state.sound_module)
            .StopSound
            .expect("non-null function pointer")(channel);
    }
}
pub unsafe fn I_SoundIsPlaying(state: &mut ISoundState, mut channel: i32) -> bool {
    if !state.sound_module.is_null() {
        return (*state.sound_module)
            .SoundIsPlaying
            .expect("non-null function pointer")(channel)
            != 0;
    } else {
        return false;
    };
}
pub unsafe fn I_PrecacheSounds(
    state: &mut ISoundState,
    mut sounds: *mut sfxinfo_t,
    mut num_sounds: i32,
) {
    if !state.sound_module.is_null() && (*state.sound_module).CacheSounds.is_some() {
        (*state.sound_module)
            .CacheSounds
            .expect("non-null function pointer")(sounds, num_sounds);
    }
}
pub unsafe fn I_InitMusic(state: &mut ISoundState) {
    if !state.music_module.is_null() {
        (*state.music_module)
            .Init
            .expect("non-null function pointer")();
    }
}
pub unsafe fn I_SetMusicVolume(state: &mut ISoundState, mut volume: i32) {
    if !state.music_module.is_null() {
        (*state.music_module)
            .SetMusicVolume
            .expect("non-null function pointer")(volume);
    }
}
pub unsafe fn I_PauseSong(state: &mut ISoundState) {
    if !state.music_module.is_null() {
        (*state.music_module)
            .PauseMusic
            .expect("non-null function pointer")();
    }
}
pub unsafe fn I_ResumeSong(state: &mut ISoundState) {
    if !state.music_module.is_null() {
        (*state.music_module)
            .ResumeMusic
            .expect("non-null function pointer")();
    }
}
pub unsafe fn I_RegisterSong(
    state: &mut ISoundState,
    mut data: *mut ::core::ffi::c_void,
    mut len: i32,
) -> *mut ::core::ffi::c_void {
    if !state.music_module.is_null() {
        return (*state.music_module)
            .RegisterSong
            .expect("non-null function pointer")(data, len);
    } else {
        return NULL;
    };
}
pub unsafe fn I_UnRegisterSong(state: &mut ISoundState, mut handle: *mut ::core::ffi::c_void) {
    if !state.music_module.is_null() {
        (*state.music_module)
            .UnRegisterSong
            .expect("non-null function pointer")(handle);
    }
}
pub unsafe fn I_PlaySong(
    state: &mut ISoundState,
    mut handle: *mut ::core::ffi::c_void,
    mut looping: bool,
) {
    if !state.music_module.is_null() {
        (*state.music_module)
            .PlaySong
            .expect("non-null function pointer")(handle, looping as i32 as boolean);
    }
}
pub unsafe fn I_StopSong(state: &mut ISoundState) {
    if !state.music_module.is_null() {
        (*state.music_module)
            .StopSong
            .expect("non-null function pointer")();
    }
}
pub unsafe fn I_MusicIsPlaying(state: &mut ISoundState) -> bool {
    if !state.music_module.is_null() {
        return (*state.music_module)
            .MusicIsPlaying
            .expect("non-null function pointer")()
            != 0;
    } else {
        return false;
    };
}
pub unsafe fn I_BindSoundVariables(state: &mut GameState) {
    M_BindVariable(
        &mut state.m_config,
        "snd_musicdevice",
        &raw mut state.i_sound.snd_musicdevice as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "snd_sfxdevice",
        &raw mut state.i_sound.snd_sfxdevice as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "snd_sbport",
        &raw mut state.i_sound.snd_sbport as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "snd_sbirq",
        &raw mut state.i_sound.snd_sbirq as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "snd_sbdma",
        &raw mut state.i_sound.snd_sbdma as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "snd_mport",
        &raw mut state.i_sound.snd_mport as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "snd_maxslicetime_ms",
        &raw mut state.i_sound.snd_maxslicetime_ms as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "snd_musiccmd",
        &raw mut state.i_sound.snd_musiccmd as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "snd_samplerate",
        &raw mut state.i_sound.snd_samplerate as *mut ::core::ffi::c_void,
    );
    M_BindVariable(
        &mut state.m_config,
        "snd_cachesize",
        &raw mut state.i_sound.snd_cachesize as *mut ::core::ffi::c_void,
    );
}
