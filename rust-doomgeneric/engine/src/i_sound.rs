use crate::game_state::GameState;
use crate::i_video::IVideoState;
use crate::m_argv::parm_exists;
use crate::m_argv::MArgvState;
use crate::m_config::bind_variable_int;
use crate::m_config::bind_variable_string;
use crate::m_config::MConfigState;
use crate::platform::DoomPlatform;
use crate::sfx_mixer::Mixer;
use crate::sfx_mixer::Sample;
use crate::sounds::SfxId;
use crate::sounds::SoundsState;
use crate::w_wad::check_num_for_name;
use crate::w_wad::lump_bytes;
use crate::w_wad::lump_length;
use crate::w_wad::WWadState;
use alloc::format;
use alloc::string::String;
/// Values of the `snd_musicdevice` / `snd_sfxdevice` config variables. Only
/// `Adlib` and `Sb` are compared against; the rest document the config values.
#[allow(dead_code)]
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
    /// `Some` once the platform has opened an audio device.
    mixer: Option<Mixer>,
    use_sfx_prefix: bool,
    music_module: Option<&'static MusicModule>,
    pub snd_musicdevice: i32,
    pub snd_sfxdevice: i32,
    snd_sbport: i32,
    snd_sbirq: i32,
    snd_sbdma: i32,
    snd_mport: i32,
    // Unused in this port: libsamplerate is not supported (the mixer does its
    // own nearest-neighbour rate conversion, as the SDL_mixer backend does
    // without it). Kept as GameState fields (rather than deleted) so the names
    // survive.
    pub use_libsamplerate: i32,
    pub libsamplerate_scale: f32,
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
            mixer: None,
            use_sfx_prefix: true,
            music_module: None,
            snd_musicdevice: SndDevice::Sb as i32,
            snd_sfxdevice: SndDevice::Sb as i32,
            snd_sbport: 0,
            snd_sbirq: 0,
            snd_sbdma: 0,
            snd_mport: 0,
            use_libsamplerate: 0,
            libsamplerate_scale: 0.65,
        }
    }
}
/// Opens the platform's audio device (unless `-nosound` / `-nosfx`) and starts
/// the mixer at whatever rate the platform settles on.
pub fn init_sound(
    i_sound: &mut ISoundState,
    i_video: &IVideoState,
    m_argv: &MArgvState,
    platform: &mut dyn DoomPlatform,
    use_sfx_prefix: bool,
) {
    let nosound: bool = parm_exists(m_argv, "-nosound");
    let nosfx: bool = parm_exists(m_argv, "-nosfx");
    if !nosound && !i_video.screensaver_mode && !nosfx {
        let preferred = u32::try_from(i_sound.snd_samplerate)
            .ok()
            .filter(|&rate| rate > 0)
            .unwrap_or(44100);
        i_sound.mixer = platform.audio_open(preferred).map(Mixer::new);
        i_sound.use_sfx_prefix = use_sfx_prefix;
    }
}
pub fn shutdown_sound(state: &mut ISoundState) {
    state.mixer = None;
    if let Some(module) = state.music_module {
        (module.shutdown.expect("non-null function pointer"))();
    }
}
/// Name of the lump holding `sfx`'s samples: Doom prefixes its lumps with
/// `ds`, and a linked sfx shares the lump of the sfx it links to.
fn sfx_lump_name(sounds: &SoundsState, sfx: SfxId, use_sfx_prefix: bool) -> String {
    let info = &sounds.s_sfx[sfx.0 as usize];
    let name = match info.link {
        Some(link) => sounds.s_sfx[link.0 as usize].name.as_str(),
        None => info.name.as_str(),
    };
    if use_sfx_prefix {
        format!("ds{name}")
    } else {
        String::from(name)
    }
}
/// Lump number of `sfx`'s samples, or -1 if the WAD has none (that sound is
/// then silent). Without an audio device nothing is looked up.
pub fn get_sfx_lump_num(
    state: &ISoundState,
    w_wad: &WWadState,
    sounds: &SoundsState,
    sfx: SfxId,
) -> i32 {
    if state.mixer.is_none() {
        return 0;
    }
    check_num_for_name(w_wad, &sfx_lump_name(sounds, sfx, state.use_sfx_prefix)).unwrap_or(-1)
}
/// Renders as much audio as the platform currently wants and hands it over.
pub fn update_sound(state: &mut ISoundState, platform: &mut dyn DoomPlatform) {
    const CHUNK_FRAMES: usize = 512;
    if let Some(mixer) = state.mixer.as_mut() {
        let mut wanted = platform.audio_frames_wanted();
        let mut chunk = [0i16; CHUNK_FRAMES * 2];
        while wanted > 0 {
            let frames = wanted.min(CHUNK_FRAMES);
            mixer.mix(&mut chunk[..frames * 2]);
            platform.audio_write(&chunk[..frames * 2]);
            wanted -= frames;
        }
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
pub fn update_sound_params(state: &mut ISoundState, channel: i32, mut vol: i32, mut sep: i32) {
    if let Some(mixer) = state.mixer.as_mut() {
        check_volume_separation(&mut vol, &mut sep);
        mixer.set_params(channel, vol, sep);
    }
}
/// Starts `sfx` on `channel`. Returns the channel as the sound's handle, or -1
/// if it could not be played. Without an audio device it returns 0.
pub fn i_start_sound(
    state: &mut GameState,
    sfx: SfxId,
    channel: i32,
    mut vol: i32,
    mut sep: i32,
) -> i32 {
    if state.i_sound.mixer.is_none() {
        return 0;
    }
    check_volume_separation(&mut vol, &mut sep);
    let lumpnum = state.sounds.s_sfx[sfx.0 as usize].lumpnum;
    if lumpnum < 0 {
        return -1;
    }
    let lump_len = lump_length(&state.w_wad, lumpnum as u32) as usize;
    let Some(sample) = Sample::from_lump(lump_bytes(state, lumpnum), lump_len) else {
        return -1;
    };
    let started = state
        .i_sound
        .mixer
        .as_mut()
        .is_some_and(|mixer| mixer.start(channel, sample, vol, sep));
    if started {
        channel
    } else {
        -1
    }
}
pub fn i_stop_sound(state: &mut ISoundState, channel: i32) {
    if let Some(mixer) = state.mixer.as_mut() {
        mixer.stop(channel);
    }
}
pub fn sound_is_playing(state: &ISoundState, channel: i32) -> bool {
    state
        .mixer
        .as_ref()
        .is_some_and(|mixer| mixer.is_playing(channel))
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
