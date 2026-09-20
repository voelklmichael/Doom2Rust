use crate::game_state::GameState;
use crate::genmidi::GenMidi;
use crate::i_video::IVideoState;
use crate::m_config::bind_variable_int;
use crate::m_config::bind_variable_string;
use crate::m_config::MConfigState;
use crate::opl_music::MusicPlayer;
use crate::options::Options;
use crate::platform::{DoomPlatform, MusicCommand};
use crate::sfx_mixer::Mixer;
use crate::sfx_mixer::Sample;
use crate::sounds::SfxId;
use crate::sounds::SoundsState;
use crate::w_wad::check_num_for_name;
use crate::w_wad::lump_bytes;
use crate::w_wad::lump_length;
use crate::w_wad::LumpNum;
use crate::w_wad::WWadState;
use alloc::format;
use alloc::string::String;
/// Values of the `snd_musicdevice` / `snd_sfxdevice` config variables. Only
/// `Adlib` and `Sb` are compared against; the rest document the config values.
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SndDevice {
    SnddeviceNone,
    Pcspeaker,
    Adlib,
    Sb,
    Pas,
    Gus,
    Waveblaster,
    Soundcanvas,
    Genmidi,
    Awe32,
    Cd,
}
pub struct ISoundState {
    pub snd_samplerate: i32,
    pub snd_cachesize: i32,
    pub snd_maxslicetime_ms: i32,
    pub snd_musiccmd: Option<&'static str>,
    /// `Some` once the platform has opened an audio device.
    mixer: Option<Mixer>,
    use_sfx_prefix: bool,
    /// `Some` once the audio device is open and the WAD has a `GENMIDI` lump.
    music: Option<MusicPlayer>,
    /// The platform plays the music (`DoomPlatform::music_open`); `music` stays `None`.
    platform_music: bool,
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
            music: None,
            platform_music: false,
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
    options: &Options,
    platform: &mut dyn DoomPlatform,
    use_sfx_prefix: bool,
) {
    let nosound: bool = options.nosound;
    let nosfx: bool = options.nosfx;
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
    state.music = None;
}
/// Sends `command` to the platform if it plays the music, else to the engine's own player.
fn music_command(
    state: &mut ISoundState,
    platform: &mut dyn DoomPlatform,
    command: MusicCommand<'_>,
) -> bool {
    if state.platform_music {
        platform.music_command(command);
        return true;
    }
    let Some(music) = state.music.as_mut() else {
        return false;
    };
    match command {
        MusicCommand::Register(data) => return music.register(data),
        MusicCommand::Play { looping } => music.play(looping),
        MusicCommand::Stop => music.stop(),
        MusicCommand::Pause => music.pause(),
        MusicCommand::Resume => music.resume(),
        MusicCommand::Volume(volume) => music.set_volume(volume),
    }
    true
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
/// Lump number of `sfx`'s samples, or `None` if the WAD has none (that sound is
/// then silent). Without an audio device nothing is looked up.
pub fn get_sfx_lump_num(
    state: &ISoundState,
    w_wad: &WWadState,
    sounds: &SoundsState,
    sfx: SfxId,
) -> Option<LumpNum> {
    state.mixer.as_ref()?;
    check_num_for_name(w_wad, &sfx_lump_name(sounds, sfx, state.use_sfx_prefix))
}
/// Renders as much audio as the platform currently wants and hands it over.
pub fn update_sound(state: &mut ISoundState, platform: &mut dyn DoomPlatform) {
    const CHUNK_FRAMES: usize = 512;
    if let Some(mixer) = state.mixer.as_mut() {
        let mut wanted = platform.audio_frames_wanted();
        while wanted > 0 {
            let frames = wanted.min(CHUNK_FRAMES);
            let mut acc = [0i32; CHUNK_FRAMES * 2];
            let acc = &mut acc[..frames * 2];
            mixer.mix_add(acc);
            if let Some(music) = state.music.as_mut() {
                music.render_add(acc);
            }
            let mut chunk = [0i16; CHUNK_FRAMES * 2];
            for (out, sum) in chunk.iter_mut().zip(acc.iter()) {
                *out = (*sum).clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
            }
            platform.audio_write(&chunk[..frames * 2]);
            wanted -= frames;
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
    if state.audio.i_sound.mixer.is_none() {
        return 0;
    }
    check_volume_separation(&mut vol, &mut sep);
    let lumpnum = state.audio.sounds.s_sfx[sfx.0 as usize].lumpnum;
    let Some(lumpnum) = lumpnum else {
        return -1;
    };
    let lump_len = lump_length(&state.assets.w_wad, lumpnum) as usize;
    let Some(sample) = Sample::from_lump(
        lump_bytes(&*state.assets.fs, &mut state.assets.w_wad, lumpnum),
        lump_len,
    ) else {
        return -1;
    };
    let started = state
        .audio
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
/// Starts the music synthesizer, if there is an audio device and the WAD has
/// the `GENMIDI` instrument lump (`-nomusic` leaves it off).
pub fn init_music(state: &mut GameState) {
    let Some(rate) = state.audio.i_sound.mixer.as_ref().map(Mixer::sample_rate) else {
        return;
    };
    if state.game.options.nomusic {
        return;
    }
    let Some(lumpnum) = check_num_for_name(&state.assets.w_wad, "GENMIDI") else {
        return;
    };
    let lump_len = lump_length(&state.assets.w_wad, lumpnum) as usize;
    let lump = lump_bytes(&*state.assets.fs, &mut state.assets.w_wad, lumpnum);
    if state.io.platform.music_open(&lump[..lump_len]) {
        state.audio.i_sound.platform_music = true;
    } else if let Some(bank) = GenMidi::parse(&lump[..lump_len]) {
        state.audio.i_sound.music = Some(MusicPlayer::new(bank, rate));
    }
}
pub fn i_set_music_volume(state: &mut ISoundState, platform: &mut dyn DoomPlatform, volume: i32) {
    music_command(state, platform, MusicCommand::Volume(volume));
}
pub fn pause_song(state: &mut ISoundState, platform: &mut dyn DoomPlatform) {
    music_command(state, platform, MusicCommand::Pause);
}
pub fn resume_song(state: &mut ISoundState, platform: &mut dyn DoomPlatform) {
    music_command(state, platform, MusicCommand::Resume);
}
/// Loads a MUS lump. The handle is 1 if it was accepted, else 0.
pub fn register_song(
    state: &mut ISoundState,
    platform: &mut dyn DoomPlatform,
    data: &[u8],
) -> usize {
    usize::from(music_command(state, platform, MusicCommand::Register(data)))
}
pub fn un_register_song(state: &mut ISoundState, _handle: usize) {
    // A platform's player is told to stop by `stop_song` and drops the song when the next one is
    // registered.
    if let Some(music) = state.music.as_mut() {
        music.unregister();
    }
}
pub fn play_song(
    state: &mut ISoundState,
    platform: &mut dyn DoomPlatform,
    _handle: usize,
    looping: bool,
) {
    music_command(state, platform, MusicCommand::Play { looping });
}
pub fn stop_song(state: &mut ISoundState, platform: &mut dyn DoomPlatform) {
    music_command(state, platform, MusicCommand::Stop);
}
pub fn bind_sound_variables(m_config: &mut MConfigState) {
    bind_variable_int(m_config, "snd_musicdevice", |s| {
        &mut s.audio.i_sound.snd_musicdevice
    });
    bind_variable_int(m_config, "snd_sfxdevice", |s| {
        &mut s.audio.i_sound.snd_sfxdevice
    });
    bind_variable_int(m_config, "snd_sbport", |s| &mut s.audio.i_sound.snd_sbport);
    bind_variable_int(m_config, "snd_sbirq", |s| &mut s.audio.i_sound.snd_sbirq);
    bind_variable_int(m_config, "snd_sbdma", |s| &mut s.audio.i_sound.snd_sbdma);
    bind_variable_int(m_config, "snd_mport", |s| &mut s.audio.i_sound.snd_mport);
    bind_variable_int(m_config, "snd_maxslicetime_ms", |s| {
        &mut s.audio.i_sound.snd_maxslicetime_ms
    });
    bind_variable_string(m_config, "snd_musiccmd", |s| {
        &mut s.audio.i_sound.snd_musiccmd
    });
    bind_variable_int(m_config, "snd_samplerate", |s| {
        &mut s.audio.i_sound.snd_samplerate
    });
    bind_variable_int(m_config, "snd_cachesize", |s| {
        &mut s.audio.i_sound.snd_cachesize
    });
}
