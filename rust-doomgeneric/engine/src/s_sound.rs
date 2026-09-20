use crate::d_mode::GameMode;
use crate::game_state::GameState;
use crate::i_sound::get_sfx_lump_num;
use crate::i_sound::i_set_music_volume;
use crate::i_sound::i_start_sound;
use crate::i_sound::i_stop_sound;
use crate::i_sound::pause_song;
use crate::i_sound::play_song;
use crate::i_sound::register_song;
use crate::i_sound::resume_song;
use crate::i_sound::shutdown_sound;
use crate::i_sound::sound_is_playing;
use crate::i_sound::stop_song;
use crate::i_sound::un_register_song;
use crate::i_sound::update_sound;
use crate::i_sound::update_sound_params;
use crate::i_sound::ISoundState;
use crate::i_sound::SndDevice;
use crate::i_system::at_exit;
use crate::i_system::error;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::p_mobj::MobjId;
use crate::p_setup::SectorId;
use crate::platform::DoomPlatform;
use crate::r_main::point_to_angle2;
use crate::sounds::SfxId;
use crate::sounds::SfxName;
use crate::sounds::SoundsState;
use crate::sounds::NUMSFX;
use crate::sounds::{MusicName, NUMMUSIC};
use crate::tables::Angle;
use crate::tables::ANGLETOFINESHIFT;
use crate::tables::FINESINE;
use crate::w_wad::get_num_for_name;
use crate::w_wad::lump_bytes;
use crate::w_wad::lump_length;
use crate::w_wad::release_lump_num;
use alloc::vec::Vec;

pub struct SSoundState {
    pub channels: Vec<Channel>,
    pub sfx_volume: i32,
    pub music_volume: i32,
    pub snd_sfx_volume: i32,
    pub mus_paused: bool,
    pub mus_playing: Option<i32>,
    pub snd_channels: i32,
}

impl Default for SSoundState {
    fn default() -> Self {
        Self::new()
    }
}

impl SSoundState {
    pub const fn new() -> Self {
        Self {
            channels: Vec::new(),
            sfx_volume: 8,
            music_volume: 8,
            snd_sfx_volume: 0,
            mus_paused: false,
            mus_playing: None,
            snd_channels: 8,
        }
    }
}

/// Vanilla Doom's `s_start_sound` takes a `void *origin` that's really always
/// either a `Mobj*` or a `Sector::soundorg` (a `DegenMobj`, which
/// shares the `{thinker, x, y, z}` prefix of Mobj by construction) --
/// callers rely on that layout pun to pass a sector's position as if it
/// were a thing. This enum replaces the pun with an explicit tag.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SoundOrigin {
    None,
    Mobj(MobjId),
    Sector(SectorId),
}
impl SoundOrigin {
    fn xy(&self, state: &mut GameState) -> Option<(Fixed, Fixed)> {
        match *self {
            Self::None => None,
            Self::Mobj(id) => {
                if !state.world.p_mobj.is_live(id) {
                    return None;
                }
                let mo = state.world.p_mobj.mo(id);
                Some((mo.x, mo.y))
            }
            Self::Sector(id) => {
                let sec = state.world.p_setup.sector_mut(id);
                Some((sec.soundorg.x, sec.soundorg.y))
            }
        }
    }
}
#[derive(Copy, Clone)]
pub struct Channel {
    pub sfxinfo: Option<SfxId>,
    pub origin: SoundOrigin,
    pub handle: i32,
}
pub const S_CLIPPING_DIST: i32 = 1200 * FRACUNIT;
pub const S_CLOSE_DIST: i32 = 200 * FRACUNIT;
pub const S_ATTENUATOR: i32 = (S_CLIPPING_DIST - S_CLOSE_DIST) >> FRACBITS;
pub const S_STEREO_SWING: i32 = 96 * FRACUNIT;
pub const NORM_SEP: i32 = 128;
pub fn s_init(state: &mut GameState, sfx_volume_0: i32, music_volume_0: i32) {
    set_sfx_volume(&mut state.audio.s_sound, sfx_volume_0);
    s_set_music_volume(
        &mut state.audio.i_sound,
        &mut *state.io.platform,
        music_volume_0,
    );
    state.audio.s_sound.channels = vec![
        Channel {
            sfxinfo: None,
            origin: SoundOrigin::None,
            handle: 0,
        };
        state.audio.s_sound.snd_channels as usize
    ];
    state.audio.s_sound.mus_paused = false;
    for i in 1..NUMSFX as usize {
        state.audio.sounds.s_sfx[i].usefulness = -1;
        state.audio.sounds.s_sfx[i].lumpnum = -1;
    }
    at_exit(
        &mut state.io.i_system,
        Some(shutdown as fn(&mut GameState) -> ()),
        true,
    );
}
pub fn shutdown(state: &mut GameState) {
    shutdown_sound(&mut state.audio.i_sound);
}
fn stop_channel(
    i_sound: &mut ISoundState,
    s_sound: &mut SSoundState,
    sounds: &mut SoundsState,
    cnum: i32,
) {
    let c = s_sound.channels[cnum as usize];
    if let Some(sfxinfo) = c.sfxinfo {
        if sound_is_playing(i_sound, c.handle) {
            i_stop_sound(i_sound, c.handle);
        }
        sounds.sfx_mut(sfxinfo).usefulness -= 1;
        s_sound.channels[cnum as usize].sfxinfo = None;
    }
}
pub fn s_start(state: &mut GameState) {
    let mnum: i32;
    for cnum in 0..state.audio.s_sound.snd_channels {
        if state.audio.s_sound.channels[cnum as usize]
            .sfxinfo
            .is_some()
        {
            stop_channel(
                &mut state.audio.i_sound,
                &mut state.audio.s_sound,
                &mut state.audio.sounds,
                cnum,
            );
        }
    }
    state.audio.s_sound.mus_paused = false;
    if state.game.doomstat.gamemode == GameMode::Commercial {
        mnum = MusicName::Runnin as i32 + state.game.g_game.gamemap - 1;
    } else {
        let spmus: [i32; 9] = [
            MusicName::E3m4 as i32,
            MusicName::E3m2 as i32,
            MusicName::E3m3 as i32,
            MusicName::E1m5 as i32,
            MusicName::E2m7 as i32,
            MusicName::E2m4 as i32,
            MusicName::E2m6 as i32,
            MusicName::E2m5 as i32,
            MusicName::E1m9 as i32,
        ];
        if state.game.g_game.gameepisode < 4 {
            mnum = MusicName::E1m1 as i32
                + (state.game.g_game.gameepisode - 1) * 9
                + state.game.g_game.gamemap
                - 1;
        } else {
            mnum = spmus[(state.game.g_game.gamemap - 1) as usize];
        }
    }
    change_music(state, mnum, true);
}
pub fn s_stop_sound(
    i_sound: &mut ISoundState,
    s_sound: &mut SSoundState,
    sounds: &mut SoundsState,
    origin: SoundOrigin,
) {
    for cnum in 0..s_sound.snd_channels {
        let c = s_sound.channels[cnum as usize];
        if c.sfxinfo.is_some() && c.origin == origin {
            stop_channel(i_sound, s_sound, sounds, cnum);
            break;
        }
    }
}
fn get_channel(
    i_sound: &mut ISoundState,
    s_sound: &mut SSoundState,
    sounds: &mut SoundsState,
    origin: SoundOrigin,
    sfxinfo: SfxId,
) -> i32 {
    let mut cnum: i32 = 0;
    while cnum < s_sound.snd_channels {
        let c = s_sound.channels[cnum as usize];
        if c.sfxinfo.is_none() {
            break;
        }
        if origin != SoundOrigin::None && c.origin == origin {
            stop_channel(i_sound, s_sound, sounds, cnum);
            break;
        }
        cnum += 1;
    }
    if cnum == s_sound.snd_channels {
        cnum = 0;
        while cnum < s_sound.snd_channels {
            let channel_sfx = s_sound.channels[cnum as usize].sfxinfo.unwrap();
            if sounds.sfx_mut(channel_sfx).priority >= sounds.s_sfx[sfxinfo.0 as usize].priority {
                break;
            }
            cnum += 1;
        }
        if cnum == s_sound.snd_channels {
            return -1;
        }
        stop_channel(i_sound, s_sound, sounds, cnum);
    }
    let c = &mut s_sound.channels[cnum as usize];
    c.sfxinfo = Some(sfxinfo);
    c.origin = origin;
    cnum
}
/// Volume and stereo separation for a sound heard from `source`, or `None` if
/// it is too far away (or too quiet) to be audible.
fn adjust_sound_params(
    state: &mut GameState,
    listener: MobjId,
    source: SoundOrigin,
) -> Option<(i32, i32)> {
    let (source_x, source_y) = source
        .xy(state)
        .expect("sound source is always resolvable here");
    let (listener_x, listener_y, listener_angle) = {
        let l = state.world.p_mobj.mo(listener);
        (l.x, l.y, l.angle)
    };
    let adx = (listener_x - source_x).abs() as Fixed;
    let ady = (listener_y - source_y).abs() as Fixed;
    let mut approx_dist = adx + ady - ((if adx < ady { adx } else { ady }) >> 1);
    if state.game.g_game.gamemap != 8 && approx_dist > S_CLIPPING_DIST {
        return None;
    }
    let mut angle: Angle = point_to_angle2(listener_x, listener_y, source_x, source_y);
    if angle > listener_angle {
        angle = angle.wrapping_sub(listener_angle);
    } else {
        angle = angle.wrapping_add((0xffffffff as Angle).wrapping_sub(listener_angle));
    }
    angle >>= ANGLETOFINESHIFT;
    let sep = 128 - (fixed_mul(S_STEREO_SWING, FINESINE[angle as usize]) >> FRACBITS);

    let vol = if approx_dist < S_CLOSE_DIST {
        state.audio.s_sound.snd_sfx_volume
    } else if state.game.g_game.gamemap == 8 {
        if approx_dist > S_CLIPPING_DIST {
            approx_dist = S_CLIPPING_DIST as Fixed;
        }
        15 + (state.audio.s_sound.snd_sfx_volume - 15)
            * ((S_CLIPPING_DIST - approx_dist) >> FRACBITS)
            / S_ATTENUATOR
    } else {
        state.audio.s_sound.snd_sfx_volume * ((S_CLIPPING_DIST - approx_dist) >> FRACBITS)
            / S_ATTENUATOR
    };
    (vol > 0).then_some((vol, sep))
}
pub fn s_start_sound(state: &mut GameState, origin: SoundOrigin, sfx_id: SfxName) {
    let mut volume = state.audio.s_sound.snd_sfx_volume;
    if sfx_id == SfxName::SfxNone {
        error("Bad sfx #: 0");
    }
    let sfx_index = sfx_id as usize;
    if state.audio.sounds.s_sfx[sfx_index].link.is_some() {
        volume += state.audio.sounds.s_sfx[sfx_index].volume;
        if volume < 1 {
            return;
        }
        if volume > state.audio.s_sound.snd_sfx_volume {
            volume = state.audio.s_sound.snd_sfx_volume;
        }
    }
    // listener_mo_id is only unwrapped when origin != None (short-circuit),
    // matching the vanilla invariant that a non-null sound origin implies
    // the console player's mobj already exists.
    let listener_mo_id = state.game.g_game.players[state.game.g_game.consoleplayer].mo;
    let sep: i32 = if origin != SoundOrigin::None
        && origin != SoundOrigin::Mobj(listener_mo_id.unwrap())
    {
        let listener = listener_mo_id.unwrap();
        let Some((adjusted_volume, adjusted_sep)) = adjust_sound_params(state, listener, origin)
        else {
            return;
        };
        volume = adjusted_volume;
        let (origin_x, origin_y) = origin.xy(state).unwrap();
        let (listener_x, listener_y) = {
            let l = state.world.p_mobj.mo(listener);
            (l.x, l.y)
        };
        if origin_x == listener_x && origin_y == listener_y {
            NORM_SEP
        } else {
            adjusted_sep
        }
    } else {
        NORM_SEP
    };
    s_stop_sound(
        &mut state.audio.i_sound,
        &mut state.audio.s_sound,
        &mut state.audio.sounds,
        origin,
    );
    let cnum = get_channel(
        &mut state.audio.i_sound,
        &mut state.audio.s_sound,
        &mut state.audio.sounds,
        origin,
        SfxId(sfx_id as u32),
    );
    if cnum < 0 {
        return;
    }
    let sfx = &mut state.audio.sounds.s_sfx[sfx_index];
    if sfx.usefulness < 0 {
        sfx.usefulness = 1;
    } else {
        sfx.usefulness += 1;
    }
    let sfx_id = SfxId(sfx_id as u32);
    if state.audio.sounds.s_sfx[sfx_index].lumpnum < 0 {
        let lumpnum = get_sfx_lump_num(
            &state.audio.i_sound,
            &state.assets.w_wad,
            &state.audio.sounds,
            sfx_id,
        );
        state.audio.sounds.s_sfx[sfx_index].lumpnum = lumpnum;
    }
    state.audio.s_sound.channels[cnum as usize].handle =
        i_start_sound(state, sfx_id, cnum, volume, sep);
}
pub fn pause_sound(
    i_sound: &mut ISoundState,
    platform: &mut dyn DoomPlatform,
    s_sound: &mut SSoundState,
) {
    if s_sound.mus_playing.is_some() && !s_sound.mus_paused {
        pause_song(i_sound, platform);
        s_sound.mus_paused = true;
    }
}
pub fn resume_sound(
    i_sound: &mut ISoundState,
    platform: &mut dyn DoomPlatform,
    s_sound: &mut SSoundState,
) {
    if s_sound.mus_playing.is_some() && s_sound.mus_paused {
        resume_song(i_sound, platform);
        s_sound.mus_paused = false;
    }
}
pub fn update_sounds(state: &mut GameState, listener: Option<MobjId>) {
    update_sound(&mut state.audio.i_sound, &mut *state.io.platform);
    for cnum in 0..state.audio.s_sound.snd_channels {
        let c = state.audio.s_sound.channels[cnum as usize];
        let Some(sfxinfo) = c.sfxinfo else {
            continue;
        };
        if !sound_is_playing(&state.audio.i_sound, c.handle) {
            stop_channel(
                &mut state.audio.i_sound,
                &mut state.audio.s_sound,
                &mut state.audio.sounds,
                cnum,
            );
            continue;
        }
        let (has_link, link_volume) = {
            let sfx = state.audio.sounds.sfx_mut(sfxinfo);
            (sfx.link.is_some(), sfx.volume)
        };
        // A linked sound too quiet to hear is stopped outright; audible ones
        // get their volume from adjust_sound_params below.
        if has_link && state.audio.s_sound.snd_sfx_volume + link_volume < 1 {
            stop_channel(
                &mut state.audio.i_sound,
                &mut state.audio.s_sound,
                &mut state.audio.sounds,
                cnum,
            );
            continue;
        }
        if c.origin != SoundOrigin::None
            && SoundOrigin::Mobj(listener.expect("positional sound needs a listener")) != c.origin
        {
            let listener = listener.unwrap();
            match adjust_sound_params(state, listener, c.origin) {
                None => stop_channel(
                    &mut state.audio.i_sound,
                    &mut state.audio.s_sound,
                    &mut state.audio.sounds,
                    cnum,
                ),
                Some((volume, sep)) => {
                    update_sound_params(&mut state.audio.i_sound, c.handle, volume, sep);
                }
            }
        }
    }
}
pub fn s_set_music_volume(i_sound: &mut ISoundState, platform: &mut dyn DoomPlatform, volume: i32) {
    if !(0..=127).contains(&volume) {
        error(&format!("Attempt to set music volume at {volume}"));
    }
    i_set_music_volume(i_sound, platform, volume);
}
pub fn set_sfx_volume(s_sound: &mut SSoundState, volume: i32) {
    if !(0..=127).contains(&volume) {
        error(&format!("Attempt to set sfx volume at {volume}"));
    }
    s_sound.snd_sfx_volume = volume;
}
pub fn start_music(state: &mut GameState, m_id: i32) {
    change_music(state, m_id, false);
}
pub fn change_music(state: &mut GameState, mut musicnum: i32, looping: bool) {
    if musicnum == MusicName::Intro as i32
        && (state.audio.i_sound.snd_musicdevice == SndDevice::Adlib as i32
            || state.audio.i_sound.snd_musicdevice == SndDevice::Sb as i32)
    {
        musicnum = MusicName::Introa as i32;
    }
    if musicnum <= MusicName::MusNone as i32 || musicnum >= NUMMUSIC {
        error(&format!("Bad music number {musicnum}"));
    }
    if state.audio.s_sound.mus_playing == Some(musicnum) {
        return;
    }
    stop_music(state);
    let music_index = musicnum as usize;
    if state.audio.sounds.s_music[music_index].lumpnum == 0 {
        let namebuf = format!(
            "d_{}",
            state.audio.sounds.s_music[music_index].name.as_str()
        );
        state.audio.sounds.s_music[music_index].lumpnum =
            get_num_for_name(&state.assets.w_wad, &namebuf);
    }
    let lumpnum = state.audio.sounds.s_music[music_index].lumpnum;
    let lumplen = lump_length(&state.assets.w_wad, lumpnum as u32) as usize;
    let data = lump_bytes(&*state.assets.fs, &mut state.assets.w_wad, lumpnum);
    let handle = register_song(
        &mut state.audio.i_sound,
        &mut *state.io.platform,
        &data[..lumplen],
    );
    state.audio.sounds.s_music[music_index].handle = handle;
    play_song(
        &mut state.audio.i_sound,
        &mut *state.io.platform,
        handle,
        looping,
    );
    state.audio.s_sound.mus_playing = Some(musicnum);
}
pub fn stop_music(state: &mut GameState) {
    if let Some(musicnum) = state.audio.s_sound.mus_playing {
        if state.audio.s_sound.mus_paused {
            resume_song(&mut state.audio.i_sound, &mut *state.io.platform);
        }
        stop_song(&mut state.audio.i_sound, &mut *state.io.platform);
        let music = &state.audio.sounds.s_music[musicnum as usize];
        un_register_song(&mut state.audio.i_sound, music.handle);
        release_lump_num(&state.assets.w_wad, music.lumpnum);
        state.audio.s_sound.mus_playing = None;
    }
}
