use crate::d_items::weaponinfo;
use crate::d_mode::GameMode_t;
use crate::d_player::PowerType;
use crate::d_player::{ammotype_t, NUMAMMO};
use crate::d_player::{player_t, PlayerState};
use crate::d_player::{PSpriteNum, NUMPSPRITES};
use crate::d_player::weapontype_t;
use crate::d_ticcmd::BT_ATTACK;
use crate::doomdef::false_0;
use crate::doomdef::true_0;
use crate::game_state::GameState;
use crate::p_mobj::{statenum_from_raw, StateNum};
use crate::info::StateId;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::m_random::P_Random;
use crate::p_enemy::P_NoiseAlert;
use crate::p_enemy::MELEERANGE;
use crate::p_enemy::MISSILERANGE;
use crate::p_inter::P_DamageMobj;
use crate::p_map::P_AimLineAttack;
use crate::p_map::P_LineAttack;
use crate::p_mobj::MobjId;
use crate::p_mobj::P_SetMobjState;
use crate::p_mobj::P_SpawnMobj;
use crate::p_mobj::P_SpawnPlayerMissile;
use crate::p_mobj::MF_JUSTATTACKED;
use crate::p_mobj::{mobj_t, pspdef_t};
use crate::p_mobj::{state_t, StateAction};
use crate::p_mobj::MobjType;
use crate::r_main::R_PointToAngle2;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::{
    sfx_bfg, sfx_dshtgn, sfx_pistol, sfx_punch, sfx_sawful, sfx_sawhit, sfx_sawidl, sfx_sawup,
    sfx_shotgn,
};
use crate::tables::angle_t;
use crate::tables::finecosine;
use crate::tables::finesine;
use crate::tables::ANG180;
use crate::tables::ANG90;
use crate::tables::FINEANGLES;
use crate::tables::FINEMASK;

pub const DEH_DEFAULT_BFG_CELLS_PER_SHOT: i32 = 40;
pub const deh_bfg_cells_per_shot: i32 = DEH_DEFAULT_BFG_CELLS_PER_SHOT;
pub unsafe fn P_SetPsprite(
    state: &mut GameState,
    mut player: *mut player_t,
    mut position: i32,
    mut stnum: StateNum,
) {
    let mut psp: *mut pspdef_t = ::core::ptr::null_mut::<pspdef_t>();
    let mut st: *mut state_t = ::core::ptr::null_mut::<state_t>();
    psp = (&raw mut (*player).psprites as *mut pspdef_t).offset(position as isize) as *mut pspdef_t;
    loop {
        if stnum as u64 == 0 {
            (*psp).state = None;
            break;
        } else {
            let state_id = StateId(stnum as u32);
            st = state.info.state_mut(state_id);
            (*psp).state = Some(state_id);
            (*psp).tics = (*st).tics;
            if (*st).misc1 != 0 {
                (*psp).sx = ((*st).misc1 << FRACBITS) as fixed_t;
                (*psp).sy = ((*st).misc2 << FRACBITS) as fixed_t;
            }
            if let StateAction::Weapon(f) = (*st).action {
                f(state, player, psp);
                if (*psp).state.is_none() {
                    break;
                }
            }
            stnum = (*state.info.state_mut((*psp).state.unwrap())).nextstate;
            if !((*psp).tics == 0) {
                break;
            }
        }
    }
}
pub struct PPsprState {
    swingx: fixed_t,
    swingy: fixed_t,
    pub bulletslope: fixed_t,
}

impl PPsprState {
    pub const fn new() -> Self {
        PPsprState {
            swingx: 0,
            swingy: 0,
            bulletslope: 0,
        }
    }
}

pub unsafe fn P_CalcSwing(state: &mut GameState, mut player: *mut player_t) {
    let mut swing: fixed_t = 0;
    let mut angle: i32 = 0;
    swing = (*player).bob;
    angle = FINEANGLES / 70 as i32 * state.p_tick.leveltime & FINEMASK;
    state.p_pspr.swingx = FixedMul(swing, finesine[angle as usize]);
    angle = FINEANGLES / 70 as i32 * state.p_tick.leveltime + FINEANGLES / 2 as i32 & FINEMASK;
    state.p_pspr.swingy = -FixedMul(state.p_pspr.swingx, finesine[angle as usize]);
}
pub unsafe fn P_BringUpWeapon(state: &mut GameState, mut player: *mut player_t) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    let mut newstate: StateNum = StateNum::S_NULL;
    if (*player).pendingweapon as u32 == weapontype_t::wp_nochange as i32 as u32 {
        (*player).pendingweapon = (*player).readyweapon;
    }
    if (*player).pendingweapon as u32 == weapontype_t::wp_chainsaw as i32 as u32 {
        S_StartSound(state, SoundOrigin::Mobj((*(player_mo)).id), sfx_sawup as i32);
    }
    newstate = weaponinfo[(*player).pendingweapon as usize].upstate;
    (*player).pendingweapon = weapontype_t::wp_nochange;
    (*player).psprites[PSpriteNum::ps_weapon as usize].sy = (128 as i32 * FRACUNIT) as fixed_t;
    P_SetPsprite(state, player, PSpriteNum::ps_weapon as i32, newstate);
}
pub unsafe fn P_CheckAmmo(state: &mut GameState, mut player: *mut player_t) -> bool {
    let mut ammo: ammotype_t = ammotype_t::am_clip;
    let mut count: i32 = 0;
    ammo = weaponinfo[(*player).readyweapon as usize].ammo;
    if (*player).readyweapon as u32 == weapontype_t::wp_bfg as i32 as u32 {
        count = deh_bfg_cells_per_shot;
    } else if (*player).readyweapon as u32 == weapontype_t::wp_supershotgun as i32 as u32 {
        count = 2 as i32;
    } else {
        count = 1 as i32;
    }
    if ammo as u32 == ammotype_t::am_noammo as i32 as u32 || (*player).ammo[ammo as usize] >= count {
        return true;
    }
    loop {
        if (*player).weaponowned[weapontype_t::wp_plasma as usize]
            && (*player).ammo[ammotype_t::am_cell as usize] != 0
            && state.doomstat.gamemode as u32 != GameMode_t::shareware as i32 as u32
        {
            (*player).pendingweapon = weapontype_t::wp_plasma;
        } else if (*player).weaponowned[weapontype_t::wp_supershotgun as usize]
            && (*player).ammo[ammotype_t::am_shell as usize] > 2 as i32
            && state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32
        {
            (*player).pendingweapon = weapontype_t::wp_supershotgun;
        } else if (*player).weaponowned[weapontype_t::wp_chaingun as usize]
            && (*player).ammo[ammotype_t::am_clip as usize] != 0
        {
            (*player).pendingweapon = weapontype_t::wp_chaingun;
        } else if (*player).weaponowned[weapontype_t::wp_shotgun as usize]
            && (*player).ammo[ammotype_t::am_shell as usize] != 0
        {
            (*player).pendingweapon = weapontype_t::wp_shotgun;
        } else if (*player).ammo[ammotype_t::am_clip as usize] != 0 {
            (*player).pendingweapon = weapontype_t::wp_pistol;
        } else if (*player).weaponowned[weapontype_t::wp_chainsaw as usize] {
            (*player).pendingweapon = weapontype_t::wp_chainsaw;
        } else if (*player).weaponowned[weapontype_t::wp_missile as usize]
            && (*player).ammo[ammotype_t::am_misl as usize] != 0
        {
            (*player).pendingweapon = weapontype_t::wp_missile;
        } else if (*player).weaponowned[weapontype_t::wp_bfg as usize]
            && (*player).ammo[ammotype_t::am_cell as usize] > 40 as i32
            && state.doomstat.gamemode as u32 != GameMode_t::shareware as i32 as u32
        {
            (*player).pendingweapon = weapontype_t::wp_bfg;
        } else {
            (*player).pendingweapon = weapontype_t::wp_fist;
        }
        if !((*player).pendingweapon as u32 == weapontype_t::wp_nochange as i32 as u32) {
            break;
        }
    }
    P_SetPsprite(
        state,
        player,
        PSpriteNum::ps_weapon as i32,
        weaponinfo[(*player).readyweapon as usize].downstate,
    );
    return false;
}
pub unsafe fn P_FireWeapon(state: &mut GameState, mut player: *mut player_t) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    let mut newstate: StateNum = StateNum::S_NULL;
    if !P_CheckAmmo(state, player) {
        return;
    }
    P_SetMobjState(state, player_mo, StateNum::S_PLAY_ATK1);
    newstate = weaponinfo[(*player).readyweapon as usize].atkstate;
    P_SetPsprite(state, player, PSpriteNum::ps_weapon as i32, newstate);
    P_NoiseAlert(state, player_mo, player_mo);
}
pub unsafe fn P_DropWeapon(state: &mut GameState, mut player: *mut player_t) {
    P_SetPsprite(
        state,
        player,
        PSpriteNum::ps_weapon as i32,
        weaponinfo[(*player).readyweapon as usize].downstate,
    );
}
pub unsafe fn A_WeaponReady(
    state: &mut GameState,
    mut player: *mut player_t,
    mut psp: *mut pspdef_t,
) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    let mut newstate: StateNum = StateNum::S_NULL;
    let mut angle: i32 = 0;
    if (*player_mo).state == Some(StateId(StateNum::S_PLAY_ATK1 as u32))
        || (*player_mo).state == Some(StateId(StateNum::S_PLAY_ATK2 as u32))
    {
        P_SetMobjState(state, player_mo, StateNum::S_PLAY);
    }
    if (*player).readyweapon as u32 == weapontype_t::wp_chainsaw as i32 as u32
        && (*psp).state == Some(StateId(StateNum::S_SAW as u32))
    {
        S_StartSound(state, SoundOrigin::Mobj((*(player_mo)).id), sfx_sawidl as i32);
    }
    if (*player).pendingweapon as u32 != weapontype_t::wp_nochange as i32 as u32 || (*player).health == 0 {
        newstate = weaponinfo[(*player).readyweapon as usize].downstate;
        P_SetPsprite(state, player, PSpriteNum::ps_weapon as i32, newstate);
        return;
    }
    if (*player).cmd.buttons as i32 & BT_ATTACK as i32 != 0 {
        if (*player).attackdown == 0
            || (*player).readyweapon as u32 != weapontype_t::wp_missile as i32 as u32
                && (*player).readyweapon as u32 != weapontype_t::wp_bfg as i32 as u32
        {
            (*player).attackdown = true_0;
            P_FireWeapon(state, player);
            return;
        }
    } else {
        (*player).attackdown = false_0;
    }
    angle = 128 as i32 * state.p_tick.leveltime & FINEMASK;
    (*psp).sx = FRACUNIT + FixedMul((*player).bob, finecosine[angle as isize]);
    angle &= FINEANGLES / 2 as i32 - 1 as i32;
    (*psp).sy = 32 as fixed_t * FRACUNIT + FixedMul((*player).bob, finesine[angle as usize]);
}
pub unsafe fn A_ReFire(state: &mut GameState, mut player: *mut player_t, _psp: *mut pspdef_t) {
    if (*player).cmd.buttons as i32 & BT_ATTACK as i32 != 0
        && (*player).pendingweapon as u32 == weapontype_t::wp_nochange as i32 as u32
        && (*player).health != 0
    {
        (*player).refire += 1;
        P_FireWeapon(state, player);
    } else {
        (*player).refire = 0 as i32;
        P_CheckAmmo(state, player);
    };
}
pub fn A_CheckReload(
    state: &mut GameState,
    mut player: *mut player_t,
    _psp: *mut pspdef_t,
) {
    unsafe { P_CheckAmmo(state, player) };
}
pub unsafe fn A_Lower(state: &mut GameState, mut player: *mut player_t, mut psp: *mut pspdef_t) {
    (*psp).sy += FRACUNIT * 6 as i32;
    if (*psp).sy < 128 as i32 * FRACUNIT {
        return;
    }
    if (*player).playerstate == PlayerState::PST_DEAD {
        (*psp).sy = (128 as i32 * FRACUNIT) as fixed_t;
        return;
    }
    if (*player).health == 0 {
        P_SetPsprite(state, player, PSpriteNum::ps_weapon as i32, StateNum::S_NULL);
        return;
    }
    (*player).readyweapon = (*player).pendingweapon;
    P_BringUpWeapon(state, player);
}
pub unsafe fn A_Raise(state: &mut GameState, mut player: *mut player_t, mut psp: *mut pspdef_t) {
    let mut newstate: StateNum = StateNum::S_NULL;
    (*psp).sy -= FRACUNIT * 6 as i32;
    if (*psp).sy > 32 as i32 * FRACUNIT {
        return;
    }
    (*psp).sy = (32 as i32 * FRACUNIT) as fixed_t;
    newstate = weaponinfo[(*player).readyweapon as usize].readystate;
    P_SetPsprite(state, player, PSpriteNum::ps_weapon as i32, newstate);
}
pub unsafe fn A_GunFlash(state: &mut GameState, mut player: *mut player_t, _psp: *mut pspdef_t) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    P_SetMobjState(state, player_mo, StateNum::S_PLAY_ATK2);
    P_SetPsprite(
        state,
        player,
        PSpriteNum::ps_flash as i32,
        weaponinfo[(*player).readyweapon as usize].flashstate,
    );
}
pub unsafe fn A_Punch(state: &mut GameState, mut player: *mut player_t, _psp: *mut pspdef_t) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    let mut angle: angle_t = 0;
    let mut damage: i32 = 0;
    let mut slope: i32 = 0;
    damage = (P_Random(&mut state.m_random) % 10 as i32 + 1 as i32) << 1 as i32;
    if (*player).powers[PowerType::pw_strength as usize] != 0 {
        damage *= 10 as i32;
    }
    angle = (*player_mo).angle;
    angle = angle.wrapping_add(
        (P_Random(&mut state.m_random) - P_Random(&mut state.m_random) << 18 as i32) as angle_t,
    );
    slope = P_AimLineAttack(state, player_mo, angle, MELEERANGE) as i32;
    P_LineAttack(
        state,
        player_mo,
        angle,
        MELEERANGE,
        slope as fixed_t,
        damage,
    );
    if let Some(linetarget) = state.p_map.linetarget {
        let linetarget = state.p_mobj.mobj_get(linetarget).unwrap();
        S_StartSound(state, SoundOrigin::Mobj((*(player_mo)).id), sfx_punch as i32);
        (*player_mo).angle = R_PointToAngle2(
            state,
            (*player_mo).x,
            (*player_mo).y,
            (*linetarget).x,
            (*linetarget).y,
        );
    }
}
pub unsafe fn A_Saw(state: &mut GameState, mut player: *mut player_t, _psp: *mut pspdef_t) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    let mut angle: angle_t = 0;
    let mut damage: i32 = 0;
    let mut slope: i32 = 0;
    damage = 2 as i32 * (P_Random(&mut state.m_random) % 10 as i32 + 1 as i32);
    angle = (*player_mo).angle;
    angle = angle.wrapping_add(
        (P_Random(&mut state.m_random) - P_Random(&mut state.m_random) << 18 as i32) as angle_t,
    );
    slope = P_AimLineAttack(state, player_mo, angle, MELEERANGE + 1 as fixed_t) as i32;
    P_LineAttack(
        state,
        player_mo,
        angle,
        MELEERANGE + 1 as fixed_t,
        slope as fixed_t,
        damage,
    );
    if state.p_map.linetarget.is_none() {
        S_StartSound(state, SoundOrigin::Mobj((*(player_mo)).id), sfx_sawful as i32);
        return;
    }
    S_StartSound(state, SoundOrigin::Mobj((*(player_mo)).id), sfx_sawhit as i32);
    let linetarget = state.p_mobj.mobj_get(state.p_map.linetarget.unwrap()).unwrap();
    angle = R_PointToAngle2(
        state,
        (*player_mo).x,
        (*player_mo).y,
        (*linetarget).x,
        (*linetarget).y,
    );
    if angle.wrapping_sub((*player_mo).angle) > ANG180 {
        if (angle.wrapping_sub((*player_mo).angle) as i32) < -ANG90 / 20 as i32 {
            (*player_mo).angle = angle.wrapping_add((ANG90 / 21 as i32) as angle_t);
        } else {
            (*player_mo).angle = (*player_mo)
                .angle
                .wrapping_sub((ANG90 / 20 as i32) as angle_t);
        }
    } else if angle.wrapping_sub((*player_mo).angle) > (ANG90 / 20 as i32) as angle_t {
        (*player_mo).angle = angle.wrapping_sub((ANG90 / 21 as i32) as angle_t);
    } else {
        (*player_mo).angle = (*player_mo)
            .angle
            .wrapping_add((ANG90 / 20 as i32) as angle_t);
    }
    (*player_mo).flags |= MF_JUSTATTACKED as i32;
}
unsafe fn DecreaseAmmo(mut player: *mut player_t, mut ammonum: i32, mut amount: i32) {
    if ammonum < NUMAMMO as i32 {
        (*player).ammo[ammonum as usize] -= amount;
    } else {
        (*player).maxammo[(ammonum - NUMAMMO as i32) as usize] -= amount;
    };
}
pub unsafe fn A_FireMissile(
    state: &mut GameState,
    mut player: *mut player_t,
    _psp: *mut pspdef_t,
) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    DecreaseAmmo(
        player,
        weaponinfo[(*player).readyweapon as usize].ammo as i32,
        1 as i32,
    );
    P_SpawnPlayerMissile(state, player_mo, MobjType::MT_ROCKET);
}
pub unsafe fn A_FireBFG(state: &mut GameState, mut player: *mut player_t, _psp: *mut pspdef_t) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    DecreaseAmmo(
        player,
        weaponinfo[(*player).readyweapon as usize].ammo as i32,
        deh_bfg_cells_per_shot,
    );
    P_SpawnPlayerMissile(state, player_mo, MobjType::MT_BFG);
}
pub unsafe fn A_FirePlasma(
    state: &mut GameState,
    mut player: *mut player_t,
    _psp: *mut pspdef_t,
) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    DecreaseAmmo(
        player,
        weaponinfo[(*player).readyweapon as usize].ammo as i32,
        1 as i32,
    );
    let flashstate = statenum_from_raw(
        weaponinfo[(*player).readyweapon as usize].flashstate as i32
            + (P_Random(&mut state.m_random) & 1 as i32),
    );
    P_SetPsprite(state, player, PSpriteNum::ps_flash as i32, flashstate);
    P_SpawnPlayerMissile(state, player_mo, MobjType::MT_PLASMA);
}
pub unsafe fn P_BulletSlope(state: &mut GameState, mut mo: *mut mobj_t) {
    let mut an: angle_t = 0;
    an = (*mo).angle;
    state.p_pspr.bulletslope =
        P_AimLineAttack(state, mo, an, 16 as fixed_t * 64 as fixed_t * FRACUNIT);
    if state.p_map.linetarget.is_none() {
        an = an.wrapping_add(((1 as i32) << 26 as i32) as angle_t);
        state.p_pspr.bulletslope =
            P_AimLineAttack(state, mo, an, 16 as fixed_t * 64 as fixed_t * FRACUNIT);
        if state.p_map.linetarget.is_none() {
            an = an.wrapping_sub(((2 as i32) << 26 as i32) as angle_t);
            state.p_pspr.bulletslope =
                P_AimLineAttack(state, mo, an, 16 as fixed_t * 64 as fixed_t * FRACUNIT);
        }
    }
}
pub unsafe fn P_GunShot(state: &mut GameState, mut mo: *mut mobj_t, mut accurate: bool) {
    let mut angle: angle_t = 0;
    let mut damage: i32 = 0;
    damage = 5 as i32 * (P_Random(&mut state.m_random) % 3 as i32 + 1 as i32);
    angle = (*mo).angle;
    if !accurate {
        angle = angle.wrapping_add(
            (P_Random(&mut state.m_random) - P_Random(&mut state.m_random) << 18 as i32) as angle_t,
        );
    }
    let bulletslope = state.p_pspr.bulletslope;
    P_LineAttack(state, mo, angle, MISSILERANGE, bulletslope, damage);
}
pub unsafe fn A_FirePistol(
    state: &mut GameState,
    mut player: *mut player_t,
    _psp: *mut pspdef_t,
) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    S_StartSound(state, SoundOrigin::Mobj((*(player_mo)).id), sfx_pistol as i32);
    P_SetMobjState(state, player_mo, StateNum::S_PLAY_ATK2);
    DecreaseAmmo(
        player,
        weaponinfo[(*player).readyweapon as usize].ammo as i32,
        1 as i32,
    );
    P_SetPsprite(
        state,
        player,
        PSpriteNum::ps_flash as i32,
        weaponinfo[(*player).readyweapon as usize].flashstate,
    );
    P_BulletSlope(state, player_mo);
    P_GunShot(state, player_mo, (*player).refire == 0);
}
pub unsafe fn A_FireShotgun(
    state: &mut GameState,
    mut player: *mut player_t,
    _psp: *mut pspdef_t,
) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    let mut i: i32 = 0;
    S_StartSound(state, SoundOrigin::Mobj((*(player_mo)).id), sfx_shotgn as i32);
    P_SetMobjState(state, player_mo, StateNum::S_PLAY_ATK2);
    DecreaseAmmo(
        player,
        weaponinfo[(*player).readyweapon as usize].ammo as i32,
        1 as i32,
    );
    P_SetPsprite(
        state,
        player,
        PSpriteNum::ps_flash as i32,
        weaponinfo[(*player).readyweapon as usize].flashstate,
    );
    P_BulletSlope(state, player_mo);
    i = 0 as i32;
    while i < 7 as i32 {
        P_GunShot(state, player_mo, false);
        i += 1;
    }
}
pub unsafe fn A_FireShotgun2(
    state: &mut GameState,
    mut player: *mut player_t,
    _psp: *mut pspdef_t,
) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    let mut i: i32 = 0;
    let mut angle: angle_t = 0;
    let mut damage: i32 = 0;
    S_StartSound(state, SoundOrigin::Mobj((*(player_mo)).id), sfx_dshtgn as i32);
    P_SetMobjState(state, player_mo, StateNum::S_PLAY_ATK2);
    DecreaseAmmo(
        player,
        weaponinfo[(*player).readyweapon as usize].ammo as i32,
        2 as i32,
    );
    P_SetPsprite(
        state,
        player,
        PSpriteNum::ps_flash as i32,
        weaponinfo[(*player).readyweapon as usize].flashstate,
    );
    P_BulletSlope(state, player_mo);
    i = 0 as i32;
    while i < 20 as i32 {
        damage = 5 as i32 * (P_Random(&mut state.m_random) % 3 as i32 + 1 as i32);
        angle = (*player_mo).angle;
        angle = angle.wrapping_add(
            (P_Random(&mut state.m_random) - P_Random(&mut state.m_random) << 19 as i32) as angle_t,
        );
        let slope = state.p_pspr.bulletslope
            + ((P_Random(&mut state.m_random) as fixed_t
                - P_Random(&mut state.m_random) as fixed_t)
                << 5 as i32);
        P_LineAttack(state, player_mo, angle, MISSILERANGE, slope, damage);
        i += 1;
    }
}
pub unsafe fn A_FireCGun(state: &mut GameState, mut player: *mut player_t, mut psp: *mut pspdef_t) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    S_StartSound(state, SoundOrigin::Mobj((*(player_mo)).id), sfx_pistol as i32);
    if (*player).ammo[weaponinfo[(*player).readyweapon as usize].ammo as usize] == 0 {
        return;
    }
    P_SetMobjState(state, player_mo, StateNum::S_PLAY_ATK2);
    DecreaseAmmo(
        player,
        weaponinfo[(*player).readyweapon as usize].ammo as i32,
        1 as i32,
    );
    P_SetPsprite(
        state,
        player,
        PSpriteNum::ps_flash as i32,
        statenum_from_raw(
            (weaponinfo[(*player).readyweapon as usize].flashstate as i64
                + (*psp).state.unwrap().0 as i64
                - StateNum::S_CHAIN1 as i64) as i32,
        ),
    );
    P_BulletSlope(state, player_mo);
    P_GunShot(state, player_mo, (*player).refire == 0);
}
pub unsafe fn A_Light0(_state: &mut GameState, mut player: *mut player_t, _psp: *mut pspdef_t) {
    (*player).extralight = 0 as i32;
}
pub unsafe fn A_Light1(_state: &mut GameState, mut player: *mut player_t, _psp: *mut pspdef_t) {
    (*player).extralight = 1 as i32;
}
pub unsafe fn A_Light2(_state: &mut GameState, mut player: *mut player_t, _psp: *mut pspdef_t) {
    (*player).extralight = 2 as i32;
}
pub unsafe fn A_BFGSpray(state: &mut GameState, id: MobjId) {
    let mo = state.p_mobj.mobj_get(id).unwrap();
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut damage: i32 = 0;
    let mut an: angle_t = 0;
    let mo_target = (*mo)
        .target
        .and_then(|id| state.p_mobj.mobj_get(id))
        .unwrap_or(::core::ptr::null_mut());
    i = 0 as i32;
    while i < 40 as i32 {
        an = (*mo)
            .angle
            .wrapping_sub((ANG90 / 2 as i32) as angle_t)
            .wrapping_add((ANG90 / 40 as i32 * i) as angle_t);
        P_AimLineAttack(
            state,
            mo_target,
            an,
            16 as fixed_t * 64 as fixed_t * FRACUNIT,
        );
        if let Some(linetarget) = state.p_map.linetarget {
            let linetarget = state.p_mobj.mobj_get(linetarget).unwrap();
            P_SpawnMobj(
                state,
                (*linetarget).x,
                (*linetarget).y,
                (*linetarget).z + ((*linetarget).height >> 2 as i32),
                MobjType::MT_EXTRABFG,
            );
            damage = 0 as i32;
            j = 0 as i32;
            while j < 15 as i32 {
                damage += (P_Random(&mut state.m_random) & 7 as i32) + 1 as i32;
                j += 1;
            }
            P_DamageMobj(state, linetarget, mo_target, mo_target, damage);
        }
        i += 1;
    }
}
pub unsafe fn A_BFGsound(state: &mut GameState, mut player: *mut player_t, _psp: *mut pspdef_t) {
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    S_StartSound(state, SoundOrigin::Mobj((*(player_mo)).id), sfx_bfg as i32);
}
pub unsafe fn P_SetupPsprites(state: &mut GameState, mut player: *mut player_t) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < NUMPSPRITES as i32 {
        (*player).psprites[i as usize].state = None;
        i += 1;
    }
    (*player).pendingweapon = (*player).readyweapon;
    P_BringUpWeapon(state, player);
}
pub unsafe fn P_MovePsprites(state: &mut GameState, mut player: *mut player_t) {
    let mut i: i32 = 0;
    let mut psp: *mut pspdef_t = ::core::ptr::null_mut::<pspdef_t>();
    let mut psp_state: Option<StateId> = None;
    psp = (&raw mut (*player).psprites as *mut pspdef_t).offset(0 as i32 as isize) as *mut pspdef_t;
    i = 0 as i32;
    while i < NUMPSPRITES as i32 {
        psp_state = (*psp).state;
        if psp_state.is_some() {
            if (*psp).tics != -(1 as i32) {
                (*psp).tics -= 1;
                if (*psp).tics == 0 {
                    let nextstate = (*state.info.state_mut(psp_state.unwrap())).nextstate;
                    P_SetPsprite(state, player, i, nextstate);
                }
            }
        }
        i += 1;
        psp = psp.offset(1);
    }
    (*player).psprites[PSpriteNum::ps_flash as usize].sx =
        (*player).psprites[PSpriteNum::ps_weapon as usize].sx;
    (*player).psprites[PSpriteNum::ps_flash as usize].sy =
        (*player).psprites[PSpriteNum::ps_weapon as usize].sy;
}
