use crate::i_video::IVideoState;
use crate::m_argv::parm_exists;
use crate::m_argv::MArgvState;
use crate::m_config::bind_variable_int;
use crate::m_config::bind_variable_string;
use crate::m_config::MConfigState;

use crate::sounds::SfxInfo;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SndDevice {
    SnddeviceNone = 0,
    Pcspeaker = 1,
    Adlib = 2,
    Sb = 3,
    Pas = 4,
    Gus = 5,
    Waveblaster = 6,
    Soundcanvas = 7,
    Genmidi = 8,
    Awe32 = 9,
    Cd = 10,
}
fn snddevice_from_raw(v: i32) -> SndDevice {
    match v {
        0 => SndDevice::SnddeviceNone,
        1 => SndDevice::Pcspeaker,
        2 => SndDevice::Adlib,
        3 => SndDevice::Sb,
        4 => SndDevice::Pas,
        5 => SndDevice::Gus,
        6 => SndDevice::Waveblaster,
        7 => SndDevice::Soundcanvas,
        8 => SndDevice::Genmidi,
        9 => SndDevice::Awe32,
        10 => SndDevice::Cd,
        n => panic!("invalid snddevice {n}"),
    }
}
type StartSoundFn = fn(&mut SfxInfo, i32, i32, i32) -> i32;
#[derive(Copy, Clone)]
pub struct SoundModule {
    pub sound_devices: &'static [SndDevice],
    pub init: Option<fn(bool) -> bool>,
    pub shutdown: Option<fn()>,
    pub get_sfx_lump_num: Option<fn(&mut SfxInfo) -> i32>,
    pub update: Option<fn()>,
    pub update_sound_params: Option<fn(i32, i32, i32)>,
    pub start_sound: Option<StartSoundFn>,
    pub stop_sound: Option<fn(i32)>,
    pub sound_is_playing: Option<fn(i32) -> bool>,
    pub cache_sounds: Option<fn(&mut [SfxInfo])>,
}
#[derive(Copy, Clone)]
pub struct MusicModule {
    pub init: Option<fn() -> bool>,
    pub shutdown: Option<fn()>,
    pub set_music_volume: Option<fn(i32)>,
    pub pause_music: Option<fn()>,
    pub resume_music: Option<fn()>,
    pub register_song: Option<fn(&[u8]) -> usize>,
    pub un_register_song: Option<fn(usize)>,
    pub play_song: Option<fn(usize, bool)>,
    pub stop_song: Option<fn()>,
    pub poll: Option<fn()>,
}
pub struct ISoundState {
    pub snd_samplerate: i32,
    pub snd_cachesize: i32,
    pub snd_maxslicetime_ms: i32,
    pub snd_musiccmd: Option<&'static str>,
    sound_module: Option<&'static SoundModule>,
    music_module: Option<&'static MusicModule>,
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
    // Always a single None entry -- see init_sfx_module, which never finds a
    // real backend and always leaves sound_module None. Kept as-is (dead
    // stub), same rationale as above, rather than deleted as a drive-by.
    sound_modules: [Option<&'static SoundModule>; 1],
}

impl Default for ISoundState {
    fn default() -> Self {
        Self::new()
    }
}

impl ISoundState {
    pub const fn new() -> Self {
        Self {
            snd_samplerate: 44100,
            snd_cachesize: 64 * 1024 * 1024,
            snd_maxslicetime_ms: 28,
            snd_musiccmd: None,
            sound_module: None,
            music_module: None,
            snd_musicdevice: SndDevice::Sb as i32,
            snd_sfxdevice: SndDevice::Sb as i32,
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
fn snd_device_in_list(device: SndDevice, list: &[SndDevice]) -> bool {
    list.contains(&device)
}
fn init_sfx_module(state: &mut ISoundState, use_sfx_prefix: bool) {
    state.sound_module = None;
    for i in 0..state.sound_modules.len() {
        let Some(module) = state.sound_modules[i] else {
            break;
        };
        if snd_device_in_list(
            snddevice_from_raw(state.snd_sfxdevice),
            module.sound_devices,
        ) && (module.init.expect("non-null function pointer"))(use_sfx_prefix)
        {
            state.sound_module = Some(module);
            return;
        }
    }
}
pub fn init_sound(
    i_sound: &mut ISoundState,
    i_video: &IVideoState,
    m_argv: &MArgvState,
    use_sfx_prefix: bool,
) {
    let nosound: bool = parm_exists(m_argv, "-nosound");
    let nosfx: bool = parm_exists(m_argv, "-nosfx");
    if !nosound && !i_video.screensaver_mode && !nosfx {
        init_sfx_module(i_sound, use_sfx_prefix);
    }
}
pub fn shutdown_sound(state: &ISoundState) {
    if let Some(module) = state.sound_module {
        (module.shutdown.expect("non-null function pointer"))();
    }
    if let Some(module) = state.music_module {
        (module.shutdown.expect("non-null function pointer"))();
    }
}
pub fn get_sfx_lump_num(state: &ISoundState, sfxinfo: &mut SfxInfo) -> i32 {
    match state.sound_module {
        Some(module) => (module.get_sfx_lump_num.expect("non-null function pointer"))(sfxinfo),
        None => 0,
    }
}
pub fn update_sound(state: &ISoundState) {
    if let Some(module) = state.sound_module {
        (module.update.expect("non-null function pointer"))();
    }
    if let Some(module) = state.music_module {
        if let Some(poll) = module.poll {
            poll();
        }
    }
}
fn check_volume_separation(vol: &mut i32, sep: &mut i32) {
    *sep = (*sep).clamp(0, 254);
    *vol = (*vol).clamp(0, 127);
}
pub fn update_sound_params(state: &ISoundState, channel: i32, mut vol: i32, mut sep: i32) {
    if let Some(module) = state.sound_module {
        check_volume_separation(&mut vol, &mut sep);
        (module
            .update_sound_params
            .expect("non-null function pointer"))(channel, vol, sep);
    }
}
pub fn i_start_sound(
    state: &ISoundState,
    sfxinfo: &mut SfxInfo,
    channel: i32,
    mut vol: i32,
    mut sep: i32,
) -> i32 {
    match state.sound_module {
        Some(module) => {
            check_volume_separation(&mut vol, &mut sep);
            (module.start_sound.expect("non-null function pointer"))(sfxinfo, channel, vol, sep)
        }
        None => 0,
    }
}
pub fn i_stop_sound(state: &ISoundState, channel: i32) {
    if let Some(module) = state.sound_module {
        (module.stop_sound.expect("non-null function pointer"))(channel);
    }
}
pub fn sound_is_playing(state: &ISoundState, channel: i32) -> bool {
    match state.sound_module {
        Some(module) => (module.sound_is_playing.expect("non-null function pointer"))(channel),
        None => false,
    }
}
pub fn precache_sounds(state: &ISoundState, sounds: &mut [SfxInfo]) {
    if let Some(module) = state.sound_module {
        if let Some(cache_sounds) = module.cache_sounds {
            cache_sounds(sounds);
        }
    }
}
pub fn init_music(state: &ISoundState) {
    if let Some(module) = state.music_module {
        (module.init.expect("non-null function pointer"))();
    }
}
pub fn i_set_music_volume(state: &ISoundState, volume: i32) {
    if let Some(module) = state.music_module {
        (module.set_music_volume.expect("non-null function pointer"))(volume);
    }
}
pub fn pause_song(state: &ISoundState) {
    if let Some(module) = state.music_module {
        (module.pause_music.expect("non-null function pointer"))();
    }
}
pub fn resume_song(state: &ISoundState) {
    if let Some(module) = state.music_module {
        (module.resume_music.expect("non-null function pointer"))();
    }
}
pub fn register_song(state: &ISoundState, data: &[u8]) -> usize {
    match state.music_module {
        Some(module) => (module.register_song.expect("non-null function pointer"))(data),
        None => 0,
    }
}
pub fn un_register_song(state: &ISoundState, handle: usize) {
    if let Some(module) = state.music_module {
        (module.un_register_song.expect("non-null function pointer"))(handle);
    }
}
pub fn play_song(state: &ISoundState, handle: usize, looping: bool) {
    if let Some(module) = state.music_module {
        (module.play_song.expect("non-null function pointer"))(handle, looping);
    }
}
pub fn stop_song(state: &ISoundState) {
    if let Some(module) = state.music_module {
        (module.stop_song.expect("non-null function pointer"))();
    }
}
pub fn bind_sound_variables(m_config: &mut MConfigState) {
    bind_variable_int(m_config, "snd_musicdevice", |s| {
        &mut s.i_sound.snd_musicdevice
    });
    bind_variable_int(m_config, "snd_sfxdevice", |s| &mut s.i_sound.snd_sfxdevice);
    bind_variable_int(m_config, "snd_sbport", |s| &mut s.i_sound.snd_sbport);
    bind_variable_int(m_config, "snd_sbirq", |s| &mut s.i_sound.snd_sbirq);
    bind_variable_int(m_config, "snd_sbdma", |s| &mut s.i_sound.snd_sbdma);
    bind_variable_int(m_config, "snd_mport", |s| &mut s.i_sound.snd_mport);
    bind_variable_int(m_config, "snd_maxslicetime_ms", |s| {
        &mut s.i_sound.snd_maxslicetime_ms
    });
    bind_variable_string(m_config, "snd_musiccmd", |s| &mut s.i_sound.snd_musiccmd);
    bind_variable_int(m_config, "snd_samplerate", |s| {
        &mut s.i_sound.snd_samplerate
    });
    bind_variable_int(m_config, "snd_cachesize", |s| &mut s.i_sound.snd_cachesize);
}
