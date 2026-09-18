use crate::d_mode::GameMode_t;
use crate::doomdef::false_0;
use crate::doomdef::true_0;
use crate::game_state::GameState;
use crate::i_sound::snddevice_t;
use crate::i_sound::I_GetSfxLumpNum;
use crate::i_sound::I_MusicIsPlaying;
use crate::i_sound::I_PauseSong;
use crate::i_sound::I_PlaySong;
use crate::i_sound::I_PrecacheSounds;
use crate::i_sound::I_RegisterSong;
use crate::i_sound::I_ResumeSong;
use crate::i_sound::I_SetMusicVolume;
use crate::i_sound::I_ShutdownSound;
use crate::i_sound::I_SoundIsPlaying;
use crate::i_sound::I_StartSound;
use crate::i_sound::I_StopSong;
use crate::i_sound::I_StopSound;
use crate::i_sound::I_UnRegisterSong;
use crate::i_sound::I_UpdateSound;
use crate::i_sound::I_UpdateSoundParams;
use crate::i_system::I_AtExit;
use crate::i_system::I_Error;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::p_mobj::MobjId;
use crate::p_setup::SectorId;
use crate::r_main::R_PointToAngle2;
use crate::sounds::SfxId;
use crate::sounds::NUMSFX;
use crate::sounds::{MusicName, NUMMUSIC};
use crate::tables::angle_t;
use crate::tables::finesine;
use crate::tables::ANGLETOFINESHIFT;
use crate::w_wad::W_GetNumForName;
use crate::w_wad::W_LumpBytes;
use crate::w_wad::W_LumpLength;
use crate::w_wad::W_ReleaseLumpNum;

pub struct SSoundState {
    pub channels: Vec<channel_t>,
    pub sfxVolume: i32,
    pub musicVolume: i32,
    pub snd_SfxVolume: i32,
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
        SSoundState {
            channels: Vec::new(),
            sfxVolume: 8,
            musicVolume: 8,
            snd_SfxVolume: 0,
            mus_paused: false,
            mus_playing: None,
            snd_channels: 8,
        }
    }
}

/// Vanilla Doom's S_StartSound takes a `void *origin` that's really always
/// either a `mobj_t*` or a `sector_t::soundorg` (a `degenmobj_t`, which
/// shares the `{thinker, x, y, z}` prefix of mobj_t by construction) --
/// callers rely on that layout pun to pass a sector's position as if it
/// were a thing. This enum replaces the pun with an explicit tag.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SoundOrigin {
    None,
    Mobj(MobjId),
    Sector(SectorId),
}
impl SoundOrigin {
    fn xy(&self, state: &mut GameState) -> Option<(fixed_t, fixed_t)> {
        match *self {
            SoundOrigin::None => None,
            SoundOrigin::Mobj(id) => {
                if !state.p_mobj.is_live(id) {
                    return None;
                }
                let mo = state.p_mobj.mo(id);
                Some((mo.x, mo.y))
            }
            SoundOrigin::Sector(id) => {
                let sec = state.p_setup.sector_mut(id);
                Some((sec.soundorg.x, sec.soundorg.y))
            }
        }
    }
}
#[derive(Copy, Clone)]
pub struct channel_t {
    pub sfxinfo: Option<SfxId>,
    pub origin: SoundOrigin,
    pub handle: i32,
}
pub const S_CLIPPING_DIST: i32 = 1200 * FRACUNIT;
pub const S_CLOSE_DIST: i32 = 200 * FRACUNIT;
pub const S_ATTENUATOR: i32 = (S_CLIPPING_DIST - S_CLOSE_DIST) >> FRACBITS;
pub const S_STEREO_SWING: i32 = 96 * FRACUNIT;
pub const NORM_SEP: i32 = 128;
pub fn S_Init(state: &mut GameState, sfxVolume_0: i32, musicVolume_0: i32) {
    I_PrecacheSounds(&mut state.i_sound, &mut state.sounds.S_sfx);
    S_SetSfxVolume(state, sfxVolume_0);
    S_SetMusicVolume(state, musicVolume_0);
    state.s_sound.channels = vec![
        channel_t {
            sfxinfo: None,
            origin: SoundOrigin::None,
            handle: 0,
        };
        state.s_sound.snd_channels as usize
    ];
    state.s_sound.mus_paused = false;
    for i in 1..NUMSFX as usize {
        state.sounds.S_sfx[i].usefulness = -1_i32;
        state.sounds.S_sfx[i].lumpnum = -1_i32;
    }
    I_AtExit(
        &mut state.i_system,
        Some(S_Shutdown as fn(&mut GameState) -> ()),
        true,
    );
}
pub fn S_Shutdown(state: &mut GameState) {
    I_ShutdownSound(&mut state.i_sound);
}
fn S_StopChannel(state: &mut GameState, cnum: i32) {
    let c = state.s_sound.channels[cnum as usize];
    if let Some(sfxinfo) = c.sfxinfo {
        if I_SoundIsPlaying(&mut state.i_sound, c.handle) {
            I_StopSound(&mut state.i_sound, c.handle);
        }
        state.sounds.sfx_mut(sfxinfo).usefulness -= 1;
        state.s_sound.channels[cnum as usize].sfxinfo = None;
    }
}
pub fn S_Start(state: &mut GameState) {
    let mut cnum: i32;
    let mnum: i32;
    cnum = 0_i32;
    while cnum < state.s_sound.snd_channels {
        if state.s_sound.channels[cnum as usize].sfxinfo.is_some() {
            S_StopChannel(state, cnum);
        }
        cnum += 1;
    }
    state.s_sound.mus_paused = false;
    if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        mnum = MusicName::mus_runnin as i32 + state.g_game.gamemap - 1_i32;
    } else {
        let spmus: [i32; 9] = [
            MusicName::mus_e3m4 as i32,
            MusicName::mus_e3m2 as i32,
            MusicName::mus_e3m3 as i32,
            MusicName::mus_e1m5 as i32,
            MusicName::mus_e2m7 as i32,
            MusicName::mus_e2m4 as i32,
            MusicName::mus_e2m6 as i32,
            MusicName::mus_e2m5 as i32,
            MusicName::mus_e1m9 as i32,
        ];
        if state.g_game.gameepisode < 4_i32 {
            mnum = MusicName::mus_e1m1 as i32
                + (state.g_game.gameepisode - 1_i32) * 9_i32
                + state.g_game.gamemap
                - 1_i32;
        } else {
            mnum = spmus[(state.g_game.gamemap - 1_i32) as usize];
        }
    }
    S_ChangeMusic(state, mnum, true_0);
}
pub fn S_StopSound(state: &mut GameState, origin: SoundOrigin) {
    for cnum in 0..state.s_sound.snd_channels {
        let c = state.s_sound.channels[cnum as usize];
        if c.sfxinfo.is_some() && c.origin == origin {
            S_StopChannel(state, cnum);
            break;
        }
    }
}
fn S_GetChannel(state: &mut GameState, origin: SoundOrigin, sfxinfo: SfxId) -> i32 {
    let mut cnum: i32 = 0;
    while cnum < state.s_sound.snd_channels {
        let c = state.s_sound.channels[cnum as usize];
        if c.sfxinfo.is_none() {
            break;
        }
        if origin != SoundOrigin::None && c.origin == origin {
            S_StopChannel(state, cnum);
            break;
        } else {
            cnum += 1;
        }
    }
    if cnum == state.s_sound.snd_channels {
        cnum = 0_i32;
        while cnum < state.s_sound.snd_channels {
            let channel_sfx = state.s_sound.channels[cnum as usize].sfxinfo.unwrap();
            if state.sounds.sfx_mut(channel_sfx).priority
                >= state.sounds.S_sfx[sfxinfo.0 as usize].priority
            {
                break;
            }
            cnum += 1;
        }
        if cnum == state.s_sound.snd_channels {
            return -1_i32;
        } else {
            S_StopChannel(state, cnum);
        }
    }
    let c = &mut state.s_sound.channels[cnum as usize];
    c.sfxinfo = Some(sfxinfo);
    c.origin = origin;
    cnum
}
fn S_AdjustSoundParams(
    state: &mut GameState,
    listener: MobjId,
    source: SoundOrigin,
    vol: &mut i32,
    sep: &mut i32,
) -> i32 {
    let (source_x, source_y) = source
        .xy(state)
        .expect("sound source is always resolvable here");
    let (listener_x, listener_y, listener_angle) = {
        let l = state.p_mobj.mo(listener);
        (l.x, l.y, l.angle)
    };
    let adx = (listener_x - source_x).abs() as fixed_t;
    let ady = (listener_y - source_y).abs() as fixed_t;
    let mut approx_dist = adx + ady - ((if adx < ady { adx } else { ady }) >> 1_i32);
    if state.g_game.gamemap != 8_i32 && approx_dist > S_CLIPPING_DIST {
        return 0_i32;
    }
    let mut angle: angle_t = R_PointToAngle2(state, listener_x, listener_y, source_x, source_y);
    if angle > listener_angle {
        angle = angle.wrapping_sub(listener_angle);
    } else {
        angle = angle.wrapping_add((0xffffffff as angle_t).wrapping_sub(listener_angle));
    }
    angle >>= ANGLETOFINESHIFT;
    *sep = 128 as fixed_t - (FixedMul(S_STEREO_SWING, finesine[angle as usize]) >> FRACBITS);
    if approx_dist < S_CLOSE_DIST {
        *vol = state.s_sound.snd_SfxVolume;
    } else if state.g_game.gamemap == 8_i32 {
        if approx_dist > S_CLIPPING_DIST {
            approx_dist = S_CLIPPING_DIST as fixed_t;
        }
        *vol = 15_i32
            + (state.s_sound.snd_SfxVolume - 15_i32)
                * ((S_CLIPPING_DIST - approx_dist) >> FRACBITS)
                / S_ATTENUATOR;
    } else {
        *vol = state.s_sound.snd_SfxVolume * ((S_CLIPPING_DIST - approx_dist) >> FRACBITS)
            / S_ATTENUATOR;
    }
    (*vol > 0_i32) as i32
}
pub fn S_StartSound(state: &mut GameState, origin: SoundOrigin, sfx_id: i32) {
    let mut sep: i32 = 0;
    let mut volume = state.s_sound.snd_SfxVolume;
    if !(1_i32..=NUMSFX).contains(&sfx_id) {
        I_Error(&format!("Bad sfx #: {}", sfx_id));
    }
    let sfx_index = sfx_id as usize;
    if state.sounds.S_sfx[sfx_index].link.is_some() {
        volume += state.sounds.S_sfx[sfx_index].volume;
        if volume < 1_i32 {
            return;
        }
        if volume > state.s_sound.snd_SfxVolume {
            volume = state.s_sound.snd_SfxVolume;
        }
    }
    // listener_mo_id is only unwrapped when origin != None (short-circuit),
    // matching the vanilla invariant that a non-null sound origin implies
    // the console player's mobj already exists.
    let listener_mo_id = state.g_game.players[state.g_game.consoleplayer as usize].mo;
    if origin != SoundOrigin::None && origin != SoundOrigin::Mobj(listener_mo_id.unwrap()) {
        let listener = listener_mo_id.unwrap();
        let rc = S_AdjustSoundParams(state, listener, origin, &mut volume, &mut sep);
        let (origin_x, origin_y) = origin.xy(state).unwrap();
        let (listener_x, listener_y) = {
            let l = state.p_mobj.mo(listener);
            (l.x, l.y)
        };
        if origin_x == listener_x && origin_y == listener_y {
            sep = NORM_SEP;
        }
        if rc == 0 {
            return;
        }
    } else {
        sep = NORM_SEP;
    }
    S_StopSound(state, origin);
    let cnum = S_GetChannel(state, origin, SfxId(sfx_id as u32));
    if cnum < 0_i32 {
        return;
    }
    let sfx = &mut state.sounds.S_sfx[sfx_index];
    let fresh2 = sfx.usefulness;
    sfx.usefulness += 1;
    if fresh2 < 0_i32 {
        sfx.usefulness = 1_i32;
    }
    if sfx.lumpnum < 0_i32 {
        sfx.lumpnum = I_GetSfxLumpNum(&mut state.i_sound, sfx);
    }
    state.s_sound.channels[cnum as usize].handle =
        I_StartSound(&mut state.i_sound, sfx, cnum, volume, sep);
}
pub fn S_PauseSound(state: &mut GameState) {
    if state.s_sound.mus_playing.is_some() && !state.s_sound.mus_paused {
        I_PauseSong(&mut state.i_sound);
        state.s_sound.mus_paused = true;
    }
}
pub fn S_ResumeSound(state: &mut GameState) {
    if state.s_sound.mus_playing.is_some() && state.s_sound.mus_paused {
        I_ResumeSong(&mut state.i_sound);
        state.s_sound.mus_paused = false;
    }
}
pub fn S_UpdateSounds(state: &mut GameState, listener: Option<MobjId>) {
    I_UpdateSound(&mut state.i_sound);
    for cnum in 0..state.s_sound.snd_channels {
        let c = state.s_sound.channels[cnum as usize];
        let Some(sfxinfo) = c.sfxinfo else {
            continue;
        };
        if !I_SoundIsPlaying(&mut state.i_sound, c.handle) {
            S_StopChannel(state, cnum);
            continue;
        }
        let mut volume = state.s_sound.snd_SfxVolume;
        let mut sep = NORM_SEP;
        let (has_link, link_volume) = {
            let sfx = state.sounds.sfx_mut(sfxinfo);
            (sfx.link.is_some(), sfx.volume)
        };
        if has_link {
            volume += link_volume;
            if volume < 1_i32 {
                S_StopChannel(state, cnum);
                continue;
            }
            if volume > state.s_sound.snd_SfxVolume {
                volume = state.s_sound.snd_SfxVolume;
            }
        }
        if c.origin != SoundOrigin::None
            && SoundOrigin::Mobj(listener.expect("positional sound needs a listener")) != c.origin
        {
            let listener = listener.unwrap();
            let audible = S_AdjustSoundParams(state, listener, c.origin, &mut volume, &mut sep);
            if audible == 0 {
                S_StopChannel(state, cnum);
            } else {
                I_UpdateSoundParams(&mut state.i_sound, c.handle, volume, sep);
            }
        }
    }
}
pub fn S_SetMusicVolume(state: &mut GameState, volume: i32) {
    if !(0_i32..=127_i32).contains(&volume) {
        I_Error(&format!("Attempt to set music volume at {}", volume));
    }
    I_SetMusicVolume(&mut state.i_sound, volume);
}
pub fn S_SetSfxVolume(state: &mut GameState, volume: i32) {
    if !(0_i32..=127_i32).contains(&volume) {
        I_Error(&format!("Attempt to set sfx volume at {}", volume));
    }
    state.s_sound.snd_SfxVolume = volume;
}
pub fn S_StartMusic(state: &mut GameState, m_id: i32) {
    S_ChangeMusic(state, m_id, false_0);
}
pub fn S_ChangeMusic(state: &mut GameState, mut musicnum: i32, looping: i32) {
    if musicnum == MusicName::mus_intro as i32
        && (state.i_sound.snd_musicdevice == snddevice_t::SNDDEVICE_ADLIB as i32
            || state.i_sound.snd_musicdevice == snddevice_t::SNDDEVICE_SB as i32)
    {
        musicnum = MusicName::mus_introa as i32;
    }
    if musicnum <= MusicName::mus_None as i32 || musicnum >= NUMMUSIC {
        I_Error(&format!("Bad music number {}", musicnum));
    }
    if state.s_sound.mus_playing == Some(musicnum) {
        return;
    }
    S_StopMusic(state);
    let music_index = musicnum as usize;
    if state.sounds.S_music[music_index].lumpnum == 0 {
        let namebuf = format!("d_{}", state.sounds.S_music[music_index].name.as_str());
        state.sounds.S_music[music_index].lumpnum = W_GetNumForName(&mut state.w_wad, &namebuf);
    }
    let lumpnum = state.sounds.S_music[music_index].lumpnum;
    let lumplen = W_LumpLength(&mut state.w_wad, lumpnum as u32) as usize;
    let data = W_LumpBytes(state, lumpnum);
    let handle = I_RegisterSong(&mut state.i_sound, &data[..lumplen]);
    state.sounds.S_music[music_index].handle = handle;
    I_PlaySong(&mut state.i_sound, handle, looping != 0);
    state.s_sound.mus_playing = Some(musicnum);
}
pub fn S_MusicPlaying(state: &mut GameState) -> bool {
    I_MusicIsPlaying(&mut state.i_sound)
}
pub fn S_StopMusic(state: &mut GameState) {
    if let Some(musicnum) = state.s_sound.mus_playing {
        if state.s_sound.mus_paused {
            I_ResumeSong(&mut state.i_sound);
        }
        I_StopSong(&mut state.i_sound);
        let music = &state.sounds.S_music[musicnum as usize];
        I_UnRegisterSong(&mut state.i_sound, music.handle);
        W_ReleaseLumpNum(&mut state.w_wad, music.lumpnum);
        state.s_sound.mus_playing = None;
    }
}
