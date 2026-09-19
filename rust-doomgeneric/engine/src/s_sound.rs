use crate::d_mode::GameMode;
use crate::game_state::GameState;
use crate::i_sound::get_sfx_lump_num;
use crate::i_sound::i_set_music_volume;
use crate::i_sound::i_start_sound;
use crate::i_sound::i_stop_sound;
use crate::i_sound::pause_song;
use crate::i_sound::play_song;
use crate::i_sound::precache_sounds;
use crate::i_sound::register_song;
use crate::i_sound::resume_song;
use crate::i_sound::shutdown_sound;
use crate::i_sound::sound_is_playing;
use crate::i_sound::stop_song;
use crate::i_sound::un_register_song;
use crate::i_sound::update_sound;
use crate::i_sound::update_sound_params;
use crate::i_sound::SndDevice;
use crate::i_system::at_exit;
use crate::i_system::error;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::p_mobj::MobjId;
use crate::p_setup::SectorId;
use crate::r_main::point_to_angle2;
use crate::sounds::SfxId;
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

/// Vanilla Doom's s_start_sound takes a `void *origin` that's really always
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
                if !state.p_mobj.is_live(id) {
                    return None;
                }
                let mo = state.p_mobj.mo(id);
                Some((mo.x, mo.y))
            }
            Self::Sector(id) => {
                let sec = state.p_setup.sector_mut(id);
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
    precache_sounds(&state.i_sound, &mut state.sounds.s_sfx);
    set_sfx_volume(state, sfx_volume_0);
    s_set_music_volume(state, music_volume_0);
    state.s_sound.channels = vec![
        Channel {
            sfxinfo: None,
            origin: SoundOrigin::None,
            handle: 0,
        };
        state.s_sound.snd_channels as usize
    ];
    state.s_sound.mus_paused = false;
    for i in 1..NUMSFX as usize {
        state.sounds.s_sfx[i].usefulness = -1;
        state.sounds.s_sfx[i].lumpnum = -1;
    }
    at_exit(
        &mut state.i_system,
        Some(shutdown as fn(&mut GameState) -> ()),
        true,
    );
}
pub fn shutdown(state: &mut GameState) {
    shutdown_sound(&state.i_sound);
}
fn stop_channel(state: &mut GameState, cnum: i32) {
    let c = state.s_sound.channels[cnum as usize];
    if let Some(sfxinfo) = c.sfxinfo {
        if sound_is_playing(&state.i_sound, c.handle) {
            i_stop_sound(&state.i_sound, c.handle);
        }
        state.sounds.sfx_mut(sfxinfo).usefulness -= 1;
        state.s_sound.channels[cnum as usize].sfxinfo = None;
    }
}
pub fn s_start(state: &mut GameState) {
    let mnum: i32;
    for cnum in 0..state.s_sound.snd_channels {
        if state.s_sound.channels[cnum as usize].sfxinfo.is_some() {
            stop_channel(state, cnum);
        }
    }
    state.s_sound.mus_paused = false;
    if state.doomstat.gamemode as u32 == GameMode::Commercial as i32 as u32 {
        mnum = MusicName::Runnin as i32 + state.g_game.gamemap - 1;
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
        if state.g_game.gameepisode < 4 {
            mnum =
                MusicName::E1m1 as i32 + (state.g_game.gameepisode - 1) * 9 + state.g_game.gamemap
                    - 1;
        } else {
            mnum = spmus[(state.g_game.gamemap - 1) as usize];
        }
    }
    change_music(state, mnum, true);
}
pub fn s_stop_sound(state: &mut GameState, origin: SoundOrigin) {
    for cnum in 0..state.s_sound.snd_channels {
        let c = state.s_sound.channels[cnum as usize];
        if c.sfxinfo.is_some() && c.origin == origin {
            stop_channel(state, cnum);
            break;
        }
    }
}
fn get_channel(state: &mut GameState, origin: SoundOrigin, sfxinfo: SfxId) -> i32 {
    let mut cnum: i32 = 0;
    while cnum < state.s_sound.snd_channels {
        let c = state.s_sound.channels[cnum as usize];
        if c.sfxinfo.is_none() {
            break;
        }
        if origin != SoundOrigin::None && c.origin == origin {
            stop_channel(state, cnum);
            break;
        }
        cnum += 1;
    }
    if cnum == state.s_sound.snd_channels {
        cnum = 0;
        while cnum < state.s_sound.snd_channels {
            let channel_sfx = state.s_sound.channels[cnum as usize].sfxinfo.unwrap();
            if state.sounds.sfx_mut(channel_sfx).priority
                >= state.sounds.s_sfx[sfxinfo.0 as usize].priority
            {
                break;
            }
            cnum += 1;
        }
        if cnum == state.s_sound.snd_channels {
            return -1;
        }
        stop_channel(state, cnum);
    }
    let c = &mut state.s_sound.channels[cnum as usize];
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
        let l = state.p_mobj.mo(listener);
        (l.x, l.y, l.angle)
    };
    let adx = (listener_x - source_x).abs() as Fixed;
    let ady = (listener_y - source_y).abs() as Fixed;
    let mut approx_dist = adx + ady - ((if adx < ady { adx } else { ady }) >> 1);
    if state.g_game.gamemap != 8 && approx_dist > S_CLIPPING_DIST {
        return None;
    }
    let mut angle: Angle = point_to_angle2(state, listener_x, listener_y, source_x, source_y);
    if angle > listener_angle {
        angle = angle.wrapping_sub(listener_angle);
    } else {
        angle = angle.wrapping_add((0xffffffff as Angle).wrapping_sub(listener_angle));
    }
    angle >>= ANGLETOFINESHIFT;
    let sep = 128 - (fixed_mul(S_STEREO_SWING, FINESINE[angle as usize]) >> FRACBITS);

    let vol = if approx_dist < S_CLOSE_DIST {
        state.s_sound.snd_sfx_volume
    } else if state.g_game.gamemap == 8 {
        if approx_dist > S_CLIPPING_DIST {
            approx_dist = S_CLIPPING_DIST as Fixed;
        }
        15 + (state.s_sound.snd_sfx_volume - 15) * ((S_CLIPPING_DIST - approx_dist) >> FRACBITS)
            / S_ATTENUATOR
    } else {
        state.s_sound.snd_sfx_volume * ((S_CLIPPING_DIST - approx_dist) >> FRACBITS) / S_ATTENUATOR
    };
    (vol > 0).then_some((vol, sep))
}
pub fn s_start_sound(state: &mut GameState, origin: SoundOrigin, sfx_id: i32) {
    let mut volume = state.s_sound.snd_sfx_volume;
    if !(1..=NUMSFX).contains(&sfx_id) {
        error(&format!("Bad sfx #: {sfx_id}"));
    }
    let sfx_index = sfx_id as usize;
    if state.sounds.s_sfx[sfx_index].link.is_some() {
        volume += state.sounds.s_sfx[sfx_index].volume;
        if volume < 1 {
            return;
        }
        if volume > state.s_sound.snd_sfx_volume {
            volume = state.s_sound.snd_sfx_volume;
        }
    }
    // listener_mo_id is only unwrapped when origin != None (short-circuit),
    // matching the vanilla invariant that a non-null sound origin implies
    // the console player's mobj already exists.
    let listener_mo_id = state.g_game.players[state.g_game.consoleplayer as usize].mo;
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
            let l = state.p_mobj.mo(listener);
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
    s_stop_sound(state, origin);
    let cnum = get_channel(state, origin, SfxId(sfx_id as u32));
    if cnum < 0 {
        return;
    }
    let sfx = &mut state.sounds.s_sfx[sfx_index];
    let fresh2 = sfx.usefulness;
    sfx.usefulness += 1;
    if fresh2 < 0 {
        sfx.usefulness = 1;
    }
    if sfx.lumpnum < 0 {
        sfx.lumpnum = get_sfx_lump_num(&state.i_sound, sfx);
    }
    state.s_sound.channels[cnum as usize].handle =
        i_start_sound(&state.i_sound, sfx, cnum, volume, sep);
}
pub fn pause_sound(state: &mut GameState) {
    if state.s_sound.mus_playing.is_some() && !state.s_sound.mus_paused {
        pause_song(&state.i_sound);
        state.s_sound.mus_paused = true;
    }
}
pub fn resume_sound(state: &mut GameState) {
    if state.s_sound.mus_playing.is_some() && state.s_sound.mus_paused {
        resume_song(&state.i_sound);
        state.s_sound.mus_paused = false;
    }
}
pub fn update_sounds(state: &mut GameState, listener: Option<MobjId>) {
    update_sound(&state.i_sound);
    for cnum in 0..state.s_sound.snd_channels {
        let c = state.s_sound.channels[cnum as usize];
        let Some(sfxinfo) = c.sfxinfo else {
            continue;
        };
        if !sound_is_playing(&state.i_sound, c.handle) {
            stop_channel(state, cnum);
            continue;
        }
        let (has_link, link_volume) = {
            let sfx = state.sounds.sfx_mut(sfxinfo);
            (sfx.link.is_some(), sfx.volume)
        };
        // A linked sound too quiet to hear is stopped outright; audible ones
        // get their volume from adjust_sound_params below.
        if has_link && state.s_sound.snd_sfx_volume + link_volume < 1 {
            stop_channel(state, cnum);
            continue;
        }
        if c.origin != SoundOrigin::None
            && SoundOrigin::Mobj(listener.expect("positional sound needs a listener")) != c.origin
        {
            let listener = listener.unwrap();
            match adjust_sound_params(state, listener, c.origin) {
                None => stop_channel(state, cnum),
                Some((volume, sep)) => {
                    update_sound_params(&state.i_sound, c.handle, volume, sep);
                }
            }
        }
    }
}
pub fn s_set_music_volume(state: &GameState, volume: i32) {
    if !(0..=127).contains(&volume) {
        error(&format!("Attempt to set music volume at {volume}"));
    }
    i_set_music_volume(&state.i_sound, volume);
}
pub fn set_sfx_volume(state: &mut GameState, volume: i32) {
    if !(0..=127).contains(&volume) {
        error(&format!("Attempt to set sfx volume at {volume}"));
    }
    state.s_sound.snd_sfx_volume = volume;
}
pub fn start_music(state: &mut GameState, m_id: i32) {
    change_music(state, m_id, false);
}
pub fn change_music(state: &mut GameState, mut musicnum: i32, looping: bool) {
    if musicnum == MusicName::Intro as i32
        && (state.i_sound.snd_musicdevice == SndDevice::Adlib as i32
            || state.i_sound.snd_musicdevice == SndDevice::Sb as i32)
    {
        musicnum = MusicName::Introa as i32;
    }
    if musicnum <= MusicName::MusNone as i32 || musicnum >= NUMMUSIC {
        error(&format!("Bad music number {musicnum}"));
    }
    if state.s_sound.mus_playing == Some(musicnum) {
        return;
    }
    stop_music(state);
    let music_index = musicnum as usize;
    if state.sounds.s_music[music_index].lumpnum == 0 {
        let namebuf = format!("d_{}", state.sounds.s_music[music_index].name.as_str());
        state.sounds.s_music[music_index].lumpnum = get_num_for_name(&state.w_wad, &namebuf);
    }
    let lumpnum = state.sounds.s_music[music_index].lumpnum;
    let lumplen = lump_length(&state.w_wad, lumpnum as u32) as usize;
    let data = lump_bytes(state, lumpnum);
    let handle = register_song(&state.i_sound, &data[..lumplen]);
    state.sounds.s_music[music_index].handle = handle;
    play_song(&state.i_sound, handle, looping);
    state.s_sound.mus_playing = Some(musicnum);
}
pub fn stop_music(state: &mut GameState) {
    if let Some(musicnum) = state.s_sound.mus_playing {
        if state.s_sound.mus_paused {
            resume_song(&state.i_sound);
        }
        stop_song(&state.i_sound);
        let music = &state.sounds.s_music[musicnum as usize];
        un_register_song(&state.i_sound, music.handle);
        release_lump_num(&state.w_wad, music.lumpnum);
        state.s_sound.mus_playing = None;
    }
}
