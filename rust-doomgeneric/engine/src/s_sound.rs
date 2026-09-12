use crate::src::i_sound::I_GetSfxLumpNum;
use crate::src::i_sound::I_MusicIsPlaying;
use crate::src::i_sound::I_PauseSong;
use crate::src::i_sound::I_PlaySong;
use crate::src::i_sound::I_PrecacheSounds;
use crate::src::i_sound::I_RegisterSong;
use crate::src::i_sound::I_ResumeSong;
use crate::src::i_sound::I_SetMusicVolume;
use crate::src::i_sound::I_ShutdownSound;
use crate::src::i_sound::I_SoundIsPlaying;
use crate::src::i_sound::I_StartSound;
use crate::src::i_sound::I_StopSong;
use crate::src::i_sound::I_StopSound;
use crate::src::i_sound::I_UnRegisterSong;
use crate::src::i_sound::I_UpdateSound;
use crate::src::i_sound::I_UpdateSoundParams;
use crate::src::i_system::I_AtExit;
use crate::src::i_system::I_Error;
use crate::src::m_fixed::FixedMul;
use crate::src::p_mobj::mobj_t;
use crate::src::r_main::R_PointToAngle2;
use crate::src::tables::finesine;
use crate::src::w_wad::W_CacheLumpNum;
use crate::src::w_wad::W_LumpLength;
use crate::src::w_wad::W_ReleaseLumpNum;
use crate::src::w_wad::W_GetNumForName;
use crate::src::z_zone::Z_Malloc;

use crate::src::d_mode::commercial;
use crate::src::doomdef::false_0;
use crate::src::doomdef::true_0;
use crate::src::doomdef::NULL;
use crate::src::game_state::GameState;
use crate::src::i_sound::{SNDDEVICE_ADLIB, SNDDEVICE_SB};
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FRACBITS;
use crate::src::m_fixed::FRACUNIT;
use crate::src::sounds::NUMSFX;
use crate::src::sounds::{
    mus_None, mus_e1m1, mus_e1m5, mus_e1m9, mus_e2m4, mus_e2m5, mus_e2m6, mus_e2m7, mus_e3m2,
    mus_e3m3, mus_e3m4, mus_intro, mus_introa, mus_runnin, NUMMUSIC,
};
use crate::src::sounds::{musicinfo_t, sfxinfo_t, SfxId};
use crate::src::tables::angle_t;
use crate::src::tables::ANGLETOFINESHIFT;
use crate::src::z_zone::PU_STATIC;

pub struct SSoundState {
    pub channels: *mut channel_t,
    pub sfxVolume: i32,
    pub musicVolume: i32,
    pub snd_SfxVolume: i32,
    pub mus_paused: bool,
    pub mus_playing: *mut musicinfo_t,
    pub snd_channels: i32,
}

impl SSoundState {
    pub const fn new() -> Self {
        SSoundState {
            channels: ::core::ptr::null::<channel_t>() as *mut channel_t,
            sfxVolume: 8,
            musicVolume: 8,
            snd_SfxVolume: 0,
            mus_paused: false,
            mus_playing: ::core::ptr::null::<musicinfo_t>() as *mut musicinfo_t,
            snd_channels: 8,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct channel_t {
    pub sfxinfo: Option<SfxId>,
    pub origin: *mut mobj_t,
    pub handle: i32,
}
pub const S_CLIPPING_DIST: i32 = 1200 * FRACUNIT;
pub const S_CLOSE_DIST: i32 = 200 * FRACUNIT;
pub const S_ATTENUATOR: i32 = S_CLIPPING_DIST - S_CLOSE_DIST >> FRACBITS;
pub const S_STEREO_SWING: i32 = 96 * FRACUNIT;
pub const NORM_SEP: i32 = 128;
pub unsafe fn S_Init(state: &mut GameState, mut sfxVolume_0: i32, mut musicVolume_0: i32) {
    let mut i: i32 = 0;
    I_PrecacheSounds(
        &mut state.i_sound,
        &raw mut state.sounds.S_sfx as *mut sfxinfo_t,
        NUMSFX as i32,
    );
    S_SetSfxVolume(state, sfxVolume_0);
    S_SetMusicVolume(state, musicVolume_0);
    state.s_sound.channels = Z_Malloc(
        &mut state.z_zone,
        (state.s_sound.snd_channels as usize)
            .wrapping_mul(::core::mem::size_of::<channel_t>() as usize) as i32,
        PU_STATIC as i32,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ) as *mut channel_t;
    i = 0 as i32;
    while i < state.s_sound.snd_channels {
        let ref mut fresh0 = (*state.s_sound.channels.offset(i as isize)).sfxinfo;
        *fresh0 = None;
        i += 1;
    }
    state.s_sound.mus_paused = false;
    i = 1 as i32;
    while i < NUMSFX as i32 {
        let ref mut fresh1 =
            (*(&raw mut state.sounds.S_sfx as *mut sfxinfo_t).offset(i as isize)).usefulness;
        *fresh1 = -(1 as i32);
        (*(&raw mut state.sounds.S_sfx as *mut sfxinfo_t).offset(i as isize)).lumpnum = *fresh1;
        i += 1;
    }
    I_AtExit(&mut state.i_system, Some(S_Shutdown as unsafe fn(&mut GameState) -> ()), true);
}
pub unsafe fn S_Shutdown(state: &mut GameState) {
    I_ShutdownSound(&mut state.i_sound);
}
unsafe fn S_StopChannel(state: &mut GameState, mut cnum: i32) {
    let mut i: i32 = 0;
    let mut c: *mut channel_t = ::core::ptr::null_mut::<channel_t>();
    c = state.s_sound.channels.offset(cnum as isize) as *mut channel_t;
    if (*c).sfxinfo.is_some() {
        if I_SoundIsPlaying(&mut state.i_sound, (*c).handle) {
            I_StopSound(&mut state.i_sound, (*c).handle);
        }
        i = 0 as i32;
        while i < state.s_sound.snd_channels {
            if cnum != i && (*c).sfxinfo == (*state.s_sound.channels.offset(i as isize)).sfxinfo {
                break;
            }
            i += 1;
        }
        (*state.sounds.sfx_mut((*c).sfxinfo.unwrap())).usefulness -= 1;
        (*c).sfxinfo = None;
    }
}
pub unsafe fn S_Start(state: &mut GameState) {
    let mut cnum: i32 = 0;
    let mut mnum: i32 = 0;
    cnum = 0 as i32;
    while cnum < state.s_sound.snd_channels {
        if !(*state.s_sound.channels.offset(cnum as isize))
            .sfxinfo
            .is_none()
        {
            S_StopChannel(state, cnum);
        }
        cnum += 1;
    }
    state.s_sound.mus_paused = false;
    if state.doomstat.gamemode as u32 == commercial as i32 as u32 {
        mnum = mus_runnin as i32 + state.g_game.gamemap - 1 as i32;
    } else {
        let mut spmus: [i32; 9] = [
            mus_e3m4 as i32,
            mus_e3m2 as i32,
            mus_e3m3 as i32,
            mus_e1m5 as i32,
            mus_e2m7 as i32,
            mus_e2m4 as i32,
            mus_e2m6 as i32,
            mus_e2m5 as i32,
            mus_e1m9 as i32,
        ];
        if state.g_game.gameepisode < 4 as i32 {
            mnum = mus_e1m1 as i32
                + (state.g_game.gameepisode - 1 as i32) * 9 as i32
                + state.g_game.gamemap
                - 1 as i32;
        } else {
            mnum = spmus[(state.g_game.gamemap - 1 as i32) as usize];
        }
    }
    S_ChangeMusic(state, mnum, true_0);
}
pub unsafe fn S_StopSound(state: &mut GameState, mut origin: *mut mobj_t) {
    let mut cnum: i32 = 0;
    cnum = 0 as i32;
    while cnum < state.s_sound.snd_channels {
        if !(*state.s_sound.channels.offset(cnum as isize))
            .sfxinfo
            .is_none()
            && (*state.s_sound.channels.offset(cnum as isize)).origin == origin
        {
            S_StopChannel(state, cnum);
            break;
        } else {
            cnum += 1;
        }
    }
}
unsafe fn S_GetChannel(state: &mut GameState, mut origin: *mut mobj_t, sfxinfo: SfxId) -> i32 {
    let mut cnum: i32 = 0;
    let mut c: *mut channel_t = ::core::ptr::null_mut::<channel_t>();
    cnum = 0 as i32;
    while cnum < state.s_sound.snd_channels {
        if (*state.s_sound.channels.offset(cnum as isize))
            .sfxinfo
            .is_none()
        {
            break;
        }
        if !origin.is_null() && (*state.s_sound.channels.offset(cnum as isize)).origin == origin {
            S_StopChannel(state, cnum);
            break;
        } else {
            cnum += 1;
        }
    }
    if cnum == state.s_sound.snd_channels {
        cnum = 0 as i32;
        while cnum < state.s_sound.snd_channels {
            if (*state
                .sounds
                .sfx_mut((*state.s_sound.channels.offset(cnum as isize)).sfxinfo.unwrap()))
            .priority
                >= state.sounds.S_sfx[sfxinfo.0 as usize].priority
            {
                break;
            }
            cnum += 1;
        }
        if cnum == state.s_sound.snd_channels {
            return -(1 as i32);
        } else {
            S_StopChannel(state, cnum);
        }
    }
    c = state.s_sound.channels.offset(cnum as isize) as *mut channel_t;
    (*c).sfxinfo = Some(sfxinfo);
    (*c).origin = origin;
    return cnum;
}
unsafe fn S_AdjustSoundParams(
    state: &mut GameState,
    mut listener: *mut mobj_t,
    mut source: *mut mobj_t,
    mut vol: *mut i32,
    mut sep: *mut i32,
) -> i32 {
    let mut approx_dist: fixed_t = 0;
    let mut adx: fixed_t = 0;
    let mut ady: fixed_t = 0;
    let mut angle: angle_t = 0;
    adx = ((*listener).x as i32 - (*source).x as i32).abs() as fixed_t;
    ady = ((*listener).y as i32 - (*source).y as i32).abs() as fixed_t;
    approx_dist = adx + ady - ((if adx < ady { adx } else { ady }) >> 1 as i32);
    if state.g_game.gamemap != 8 as i32 && approx_dist > S_CLIPPING_DIST {
        return 0 as i32;
    }
    angle = R_PointToAngle2(
        state,
        (*listener).x,
        (*listener).y,
        (*source).x,
        (*source).y,
    );
    if angle > (*listener).angle {
        angle = angle.wrapping_sub((*listener).angle);
    } else {
        angle = angle.wrapping_add((0xffffffff as angle_t).wrapping_sub((*listener).angle));
    }
    angle >>= ANGLETOFINESHIFT;
    *sep =
        (128 as fixed_t - (FixedMul(S_STEREO_SWING, finesine[angle as usize]) >> FRACBITS)) as i32;
    if approx_dist < S_CLOSE_DIST {
        *vol = state.s_sound.snd_SfxVolume;
    } else if state.g_game.gamemap == 8 as i32 {
        if approx_dist > S_CLIPPING_DIST {
            approx_dist = S_CLIPPING_DIST as fixed_t;
        }
        *vol = 15 as i32
            + (state.s_sound.snd_SfxVolume - 15 as i32)
                * (S_CLIPPING_DIST - approx_dist as i32 >> FRACBITS)
                / S_ATTENUATOR;
    } else {
        *vol = state.s_sound.snd_SfxVolume * (S_CLIPPING_DIST - approx_dist as i32 >> FRACBITS)
            / S_ATTENUATOR;
    }
    return (*vol > 0 as i32) as i32;
}
pub unsafe fn S_StartSound(
    state: &mut GameState,
    mut origin_p: *mut ::core::ffi::c_void,
    mut sfx_id: i32,
) {
    let mut sfx: *mut sfxinfo_t = ::core::ptr::null_mut::<sfxinfo_t>();
    let mut origin: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    let mut rc: i32 = 0;
    let mut sep: i32 = 0;
    let mut cnum: i32 = 0;
    let mut volume: i32 = 0;
    origin = origin_p as *mut mobj_t;
    volume = state.s_sound.snd_SfxVolume;
    if sfx_id < 1 as i32 || sfx_id > NUMSFX as i32 {
        I_Error(&format!("Bad sfx #: {}", sfx_id));
    }
    sfx =
        (&raw mut state.sounds.S_sfx as *mut sfxinfo_t).offset(sfx_id as isize) as *mut sfxinfo_t;
    if (*sfx).link.is_some() {
        volume += (*sfx).volume;
        if volume < 1 as i32 {
            return;
        }
        if volume > state.s_sound.snd_SfxVolume {
            volume = state.s_sound.snd_SfxVolume;
        }
    }
    if !origin.is_null()
        && origin != state.g_game.players[state.g_game.consoleplayer as usize].mo
    {
        let listener = state.g_game.players[state.g_game.consoleplayer as usize].mo;
        rc = S_AdjustSoundParams(state, listener, origin, &raw mut volume, &raw mut sep);
        if (*origin).x == (*listener).x && (*origin).y == (*listener).y {
            sep = NORM_SEP;
        }
        if rc == 0 {
            return;
        }
    } else {
        sep = NORM_SEP;
    }
    S_StopSound(state, origin);
    cnum = S_GetChannel(state, origin, SfxId(sfx_id as u32));
    if cnum < 0 as i32 {
        return;
    }
    let fresh2 = (*sfx).usefulness;
    (*sfx).usefulness = (*sfx).usefulness + 1;
    if fresh2 < 0 as i32 {
        (*sfx).usefulness = 1 as i32;
    }
    if (*sfx).lumpnum < 0 as i32 {
        (*sfx).lumpnum = I_GetSfxLumpNum(&mut state.i_sound, sfx);
    }
    (*state.s_sound.channels.offset(cnum as isize)).handle =
        I_StartSound(&mut state.i_sound, sfx, cnum, volume, sep);
}
pub unsafe fn S_PauseSound(state: &mut GameState) {
    if !state.s_sound.mus_playing.is_null() && !state.s_sound.mus_paused {
        I_PauseSong(&mut state.i_sound);
        state.s_sound.mus_paused = true;
    }
}
pub unsafe fn S_ResumeSound(state: &mut GameState) {
    if !state.s_sound.mus_playing.is_null() && state.s_sound.mus_paused {
        I_ResumeSong(&mut state.i_sound);
        state.s_sound.mus_paused = false;
    }
}
pub unsafe fn S_UpdateSounds(state: &mut GameState, mut listener: *mut mobj_t) {
    let mut audible: i32 = 0;
    let mut cnum: i32 = 0;
    let mut volume: i32 = 0;
    let mut sep: i32 = 0;
    let mut sfx: *mut sfxinfo_t = ::core::ptr::null_mut::<sfxinfo_t>();
    let mut c: *mut channel_t = ::core::ptr::null_mut::<channel_t>();
    I_UpdateSound(&mut state.i_sound);
    let mut current_block_20: u64;
    cnum = 0 as i32;
    while cnum < state.s_sound.snd_channels {
        c = state.s_sound.channels.offset(cnum as isize) as *mut channel_t;
        sfx = match (*c).sfxinfo {
            Some(id) => state.sounds.sfx_mut(id),
            None => ::core::ptr::null_mut::<sfxinfo_t>(),
        };
        if (*c).sfxinfo.is_some() {
            if I_SoundIsPlaying(&mut state.i_sound, (*c).handle) {
                volume = state.s_sound.snd_SfxVolume;
                sep = NORM_SEP;
                if (*sfx).link.is_some() {
                    volume += (*sfx).volume;
                    if volume < 1 as i32 {
                        S_StopChannel(state, cnum);
                        current_block_20 = 10680521327981672866;
                    } else {
                        if volume > state.s_sound.snd_SfxVolume {
                            volume = state.s_sound.snd_SfxVolume;
                        }
                        current_block_20 = 17860125682698302841;
                    }
                } else {
                    current_block_20 = 17860125682698302841;
                }
                match current_block_20 {
                    10680521327981672866 => {}
                    _ => {
                        if !(*c).origin.is_null() && listener != (*c).origin {
                            audible = S_AdjustSoundParams(
                                state,
                                listener,
                                (*c).origin,
                                &raw mut volume,
                                &raw mut sep,
                            );
                            if audible == 0 {
                                S_StopChannel(state, cnum);
                            } else {
                                I_UpdateSoundParams(&mut state.i_sound, (*c).handle, volume, sep);
                            }
                        }
                    }
                }
            } else {
                S_StopChannel(state, cnum);
            }
        }
        cnum += 1;
    }
}
pub unsafe fn S_SetMusicVolume(state: &mut GameState, mut volume: i32) {
    if volume < 0 as i32 || volume > 127 as i32 {
        I_Error(&format!("Attempt to set music volume at {}", volume));
    }
    I_SetMusicVolume(&mut state.i_sound, volume);
}
pub unsafe fn S_SetSfxVolume(state: &mut GameState, mut volume: i32) {
    if volume < 0 as i32 || volume > 127 as i32 {
        I_Error(&format!("Attempt to set sfx volume at {}", volume));
    }
    state.s_sound.snd_SfxVolume = volume;
}
pub unsafe fn S_StartMusic(state: &mut GameState, mut m_id: i32) {
    S_ChangeMusic(state, m_id, false_0);
}
pub unsafe fn S_ChangeMusic(state: &mut GameState, mut musicnum: i32, mut looping: i32) {
    let mut music: *mut musicinfo_t = ::core::ptr::null_mut::<musicinfo_t>();
    let mut handle: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if musicnum == mus_intro as i32
        && (state.i_sound.snd_musicdevice == SNDDEVICE_ADLIB as i32
            || state.i_sound.snd_musicdevice == SNDDEVICE_SB as i32)
    {
        musicnum = mus_introa as i32;
    }
    if musicnum <= mus_None as i32 || musicnum >= NUMMUSIC as i32 {
        I_Error(&format!("Bad music number {}", musicnum));
    } else {
        music = (&raw mut state.sounds.S_music as *mut musicinfo_t).offset(musicnum as isize)
            as *mut musicinfo_t;
    }
    if state.s_sound.mus_playing == music {
        return;
    }
    S_StopMusic(state);
    if (*music).lumpnum == 0 {
        let namebuf = format!("d_{}", (*music).name.as_str());
        (*music).lumpnum = W_GetNumForName(&mut state.w_wad, &namebuf);
    }
    (*music).data = W_CacheLumpNum(state, (*music).lumpnum, PU_STATIC as i32);
    let lumplen = W_LumpLength(&mut state.w_wad, (*music).lumpnum as u32);
    handle = I_RegisterSong(&mut state.i_sound, (*music).data, lumplen);
    (*music).handle = handle;
    I_PlaySong(&mut state.i_sound, handle, looping != 0);
    state.s_sound.mus_playing = music;
}
pub unsafe fn S_MusicPlaying(state: &mut GameState) -> bool {
    return I_MusicIsPlaying(&mut state.i_sound);
}
pub unsafe fn S_StopMusic(state: &mut GameState) {
    if !state.s_sound.mus_playing.is_null() {
        if state.s_sound.mus_paused {
            I_ResumeSong(&mut state.i_sound);
        }
        I_StopSong(&mut state.i_sound);
        I_UnRegisterSong(&mut state.i_sound, (*state.s_sound.mus_playing).handle);
        W_ReleaseLumpNum(&mut state.w_wad, (*state.s_sound.mus_playing).lumpnum);
        (*state.s_sound.mus_playing).data = NULL;
        state.s_sound.mus_playing = ::core::ptr::null_mut::<musicinfo_t>();
    }
}
