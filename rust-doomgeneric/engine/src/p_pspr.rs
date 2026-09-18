use crate::d_items::weaponinfo;
use crate::d_mode::GameMode_t;
use crate::d_player::weapontype_t;
use crate::d_player::PlayerId;
use crate::d_player::PlayerState;
use crate::d_player::PowerType;
use crate::d_player::{ammotype_t, NUMAMMO};
use crate::d_player::{PSpriteNum, NUMPSPRITES};
use crate::d_ticcmd::BT_ATTACK;
use crate::doomdef::false_0;
use crate::doomdef::true_0;
use crate::game_state::GameState;
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
use crate::p_mobj::MobjType;
use crate::p_mobj::P_SetMobjState;
use crate::p_mobj::P_SpawnMobj;
use crate::p_mobj::P_SpawnPlayerMissile;
use crate::p_mobj::MF_JUSTATTACKED;

use crate::p_mobj::StateAction;
use crate::p_mobj::{statenum_from_raw, StateNum};
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
pub fn P_SetPsprite(
    state: &mut GameState,
    player_id: PlayerId,
    position: i32,
    mut stnum: StateNum,
) {
    let pos = position as usize;
    loop {
        if stnum as u64 == 0 {
            state.g_game.player_mut(player_id).psprites[pos].state = None;
            break;
        } else {
            let state_id = StateId(stnum as u32);
            let (tics, misc1, misc2, action) = {
                let st = state.info.state_mut(state_id);
                (st.tics, st.misc1, st.misc2, st.action)
            };
            {
                let psp = &mut state.g_game.player_mut(player_id).psprites[pos];
                psp.state = Some(state_id);
                psp.tics = tics;
                if misc1 != 0 {
                    psp.sx = (misc1 << FRACBITS) as fixed_t;
                    psp.sy = (misc2 << FRACBITS) as fixed_t;
                }
            }
            if let StateAction::Weapon(f) = action {
                f(state, player_id, position);
                if state.g_game.player_mut(player_id).psprites[pos]
                    .state
                    .is_none()
                {
                    break;
                }
            }
            let current = state.g_game.player_mut(player_id).psprites[pos]
                .state
                .unwrap();
            stnum = state.info.state_mut(current).nextstate;
            if state.g_game.player_mut(player_id).psprites[pos].tics != 0 {
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

impl Default for PPsprState {
    fn default() -> Self {
        Self::new()
    }
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

pub fn P_CalcSwing(state: &mut GameState, player: PlayerId) {
    let mut swing: fixed_t = 0;
    let mut angle: i32 = 0;
    swing = state.g_game.players[player.0 as usize].bob;
    angle = (FINEANGLES / 70_i32 * state.p_tick.leveltime) & FINEMASK;
    state.p_pspr.swingx = FixedMul(swing, finesine[angle as usize]);
    angle = (FINEANGLES / 70_i32 * state.p_tick.leveltime + FINEANGLES / 2_i32) & FINEMASK;
    state.p_pspr.swingy = -FixedMul(state.p_pspr.swingx, finesine[angle as usize]);
}
pub fn P_BringUpWeapon(state: &mut GameState, player_id: PlayerId) {
    let player = player_id;
    let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
    let mut newstate: StateNum = StateNum::S_NULL;
    if state.g_game.players[player.0 as usize].pendingweapon as u32
        == weapontype_t::wp_nochange as i32 as u32
    {
        state.g_game.players[player.0 as usize].pendingweapon =
            state.g_game.players[player.0 as usize].readyweapon;
    }
    if state.g_game.players[player.0 as usize].pendingweapon as u32
        == weapontype_t::wp_chainsaw as i32 as u32
    {
        S_StartSound(state, SoundOrigin::Mobj(player_mo), sfx_sawup as i32);
    }
    newstate = weaponinfo[state.g_game.players[player.0 as usize].pendingweapon as usize].upstate;
    state.g_game.players[player.0 as usize].pendingweapon = weapontype_t::wp_nochange;
    state.g_game.players[player.0 as usize].psprites[PSpriteNum::ps_weapon as usize].sy =
        (128_i32 * FRACUNIT) as fixed_t;
    P_SetPsprite(state, player_id, PSpriteNum::ps_weapon as i32, newstate);
}
pub fn P_CheckAmmo(state: &mut GameState, player_id: PlayerId) -> bool {
    let player = player_id;
    let mut ammo: ammotype_t = ammotype_t::am_clip;
    let mut count: i32 = 0;
    ammo = weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].ammo;
    if state.g_game.players[player.0 as usize].readyweapon as u32
        == weapontype_t::wp_bfg as i32 as u32
    {
        count = deh_bfg_cells_per_shot;
    } else if state.g_game.players[player.0 as usize].readyweapon as u32
        == weapontype_t::wp_supershotgun as i32 as u32
    {
        count = 2_i32;
    } else {
        count = 1_i32;
    }
    if ammo as u32 == ammotype_t::am_noammo as i32 as u32
        || state.g_game.players[player.0 as usize].ammo[ammo as usize] >= count
    {
        return true;
    }
    loop {
        if state.g_game.players[player.0 as usize].weaponowned[weapontype_t::wp_plasma as usize]
            && state.g_game.players[player.0 as usize].ammo[ammotype_t::am_cell as usize] != 0
            && state.doomstat.gamemode as u32 != GameMode_t::shareware as i32 as u32
        {
            state.g_game.players[player.0 as usize].pendingweapon = weapontype_t::wp_plasma;
        } else if state.g_game.players[player.0 as usize].weaponowned
            [weapontype_t::wp_supershotgun as usize]
            && state.g_game.players[player.0 as usize].ammo[ammotype_t::am_shell as usize] > 2_i32
            && state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32
        {
            state.g_game.players[player.0 as usize].pendingweapon = weapontype_t::wp_supershotgun;
        } else if state.g_game.players[player.0 as usize].weaponowned
            [weapontype_t::wp_chaingun as usize]
            && state.g_game.players[player.0 as usize].ammo[ammotype_t::am_clip as usize] != 0
        {
            state.g_game.players[player.0 as usize].pendingweapon = weapontype_t::wp_chaingun;
        } else if state.g_game.players[player.0 as usize].weaponowned
            [weapontype_t::wp_shotgun as usize]
            && state.g_game.players[player.0 as usize].ammo[ammotype_t::am_shell as usize] != 0
        {
            state.g_game.players[player.0 as usize].pendingweapon = weapontype_t::wp_shotgun;
        } else if state.g_game.players[player.0 as usize].ammo[ammotype_t::am_clip as usize] != 0 {
            state.g_game.players[player.0 as usize].pendingweapon = weapontype_t::wp_pistol;
        } else if state.g_game.players[player.0 as usize].weaponowned
            [weapontype_t::wp_chainsaw as usize]
        {
            state.g_game.players[player.0 as usize].pendingweapon = weapontype_t::wp_chainsaw;
        } else if state.g_game.players[player.0 as usize].weaponowned
            [weapontype_t::wp_missile as usize]
            && state.g_game.players[player.0 as usize].ammo[ammotype_t::am_misl as usize] != 0
        {
            state.g_game.players[player.0 as usize].pendingweapon = weapontype_t::wp_missile;
        } else if state.g_game.players[player.0 as usize].weaponowned[weapontype_t::wp_bfg as usize]
            && state.g_game.players[player.0 as usize].ammo[ammotype_t::am_cell as usize] > 40_i32
            && state.doomstat.gamemode as u32 != GameMode_t::shareware as i32 as u32
        {
            state.g_game.players[player.0 as usize].pendingweapon = weapontype_t::wp_bfg;
        } else {
            state.g_game.players[player.0 as usize].pendingweapon = weapontype_t::wp_fist;
        }
        if state.g_game.players[player.0 as usize].pendingweapon as u32
            != weapontype_t::wp_nochange as i32 as u32
        {
            break;
        }
    }
    P_SetPsprite(
        state,
        player_id,
        PSpriteNum::ps_weapon as i32,
        weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].downstate,
    );
    false
}
pub fn P_FireWeapon(state: &mut GameState, player_id: PlayerId) {
    let player = player_id;
    let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
    let mut newstate: StateNum = StateNum::S_NULL;
    if !P_CheckAmmo(state, player_id) {
        return;
    }
    P_SetMobjState(state, player_mo, StateNum::S_PLAY_ATK1);
    newstate = weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].atkstate;
    P_SetPsprite(state, player_id, PSpriteNum::ps_weapon as i32, newstate);
    P_NoiseAlert(state, player_mo, player_mo);
}
pub fn P_DropWeapon(state: &mut GameState, player_id: PlayerId) {
    let player = player_id;
    P_SetPsprite(
        state,
        player_id,
        PSpriteNum::ps_weapon as i32,
        weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].downstate,
    );
}
pub fn A_WeaponReady(state: &mut GameState, player_id: PlayerId, position: i32) {
    {
        let player = player_id;
        let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
        let mut newstate: StateNum = StateNum::S_NULL;
        let mut angle: i32 = 0;
        if state.p_mobj.mo(player_mo).state == Some(StateId(StateNum::S_PLAY_ATK1 as u32))
            || state.p_mobj.mo(player_mo).state == Some(StateId(StateNum::S_PLAY_ATK2 as u32))
        {
            P_SetMobjState(state, player_mo, StateNum::S_PLAY);
        }
        if state.g_game.players[player.0 as usize].readyweapon as u32
            == weapontype_t::wp_chainsaw as i32 as u32
            && state.g_game.players[player_id.0 as usize].psprites[position as usize].state
                == Some(StateId(StateNum::S_SAW as u32))
        {
            S_StartSound(state, SoundOrigin::Mobj(player_mo), sfx_sawidl as i32);
        }
        if state.g_game.players[player.0 as usize].pendingweapon as u32
            != weapontype_t::wp_nochange as i32 as u32
            || state.g_game.players[player.0 as usize].health == 0
        {
            newstate =
                weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].downstate;
            P_SetPsprite(state, player_id, PSpriteNum::ps_weapon as i32, newstate);
            return;
        }
        if state.g_game.players[player.0 as usize].cmd.buttons as i32 & BT_ATTACK as i32 != 0 {
            if state.g_game.players[player.0 as usize].attackdown == 0
                || state.g_game.players[player.0 as usize].readyweapon as u32
                    != weapontype_t::wp_missile as i32 as u32
                    && state.g_game.players[player.0 as usize].readyweapon as u32
                        != weapontype_t::wp_bfg as i32 as u32
            {
                state.g_game.players[player.0 as usize].attackdown = true_0;
                P_FireWeapon(state, player_id);
                return;
            }
        } else {
            state.g_game.players[player.0 as usize].attackdown = false_0;
        }
        angle = (128_i32 * state.p_tick.leveltime) & FINEMASK;
        state.g_game.players[player_id.0 as usize].psprites[position as usize].sx = FRACUNIT
            + FixedMul(
                state.g_game.players[player.0 as usize].bob,
                finecosine[angle as isize],
            );
        angle &= FINEANGLES / 2_i32 - 1_i32;
        state.g_game.players[player_id.0 as usize].psprites[position as usize].sy = 32 as fixed_t
            * FRACUNIT
            + FixedMul(
                state.g_game.players[player.0 as usize].bob,
                finesine[angle as usize],
            );
    }
}
pub fn A_ReFire(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        if state.g_game.players[player.0 as usize].cmd.buttons as i32 & BT_ATTACK as i32 != 0
            && state.g_game.players[player.0 as usize].pendingweapon as u32
                == weapontype_t::wp_nochange as i32 as u32
            && state.g_game.players[player.0 as usize].health != 0
        {
            state.g_game.players[player.0 as usize].refire += 1;
            P_FireWeapon(state, player_id);
        } else {
            state.g_game.players[player.0 as usize].refire = 0_i32;
            P_CheckAmmo(state, player_id);
        };
    }
}
pub fn A_CheckReload(state: &mut GameState, player_id: PlayerId, _position: i32) {
    P_CheckAmmo(state, player_id);
}
pub fn A_Lower(state: &mut GameState, player_id: PlayerId, position: i32) {
    {
        let player = player_id;
        state.g_game.players[player_id.0 as usize].psprites[position as usize].sy +=
            FRACUNIT * 6_i32;
        if state.g_game.players[player_id.0 as usize].psprites[position as usize].sy
            < 128_i32 * FRACUNIT
        {
            return;
        }
        if state.g_game.players[player.0 as usize].playerstate == PlayerState::PST_DEAD {
            state.g_game.players[player_id.0 as usize].psprites[position as usize].sy =
                (128_i32 * FRACUNIT) as fixed_t;
            return;
        }
        if state.g_game.players[player.0 as usize].health == 0 {
            P_SetPsprite(
                state,
                player_id,
                PSpriteNum::ps_weapon as i32,
                StateNum::S_NULL,
            );
            return;
        }
        state.g_game.players[player.0 as usize].readyweapon =
            state.g_game.players[player.0 as usize].pendingweapon;
        P_BringUpWeapon(state, player_id);
    }
}
pub fn A_Raise(state: &mut GameState, player_id: PlayerId, position: i32) {
    {
        let player = player_id;
        let mut newstate: StateNum = StateNum::S_NULL;
        state.g_game.players[player_id.0 as usize].psprites[position as usize].sy -=
            FRACUNIT * 6_i32;
        if state.g_game.players[player_id.0 as usize].psprites[position as usize].sy
            > 32_i32 * FRACUNIT
        {
            return;
        }
        state.g_game.players[player_id.0 as usize].psprites[position as usize].sy =
            (32_i32 * FRACUNIT) as fixed_t;
        newstate =
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].readystate;
        P_SetPsprite(state, player_id, PSpriteNum::ps_weapon as i32, newstate);
    }
}
pub fn A_GunFlash(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
        P_SetMobjState(state, player_mo, StateNum::S_PLAY_ATK2);
        P_SetPsprite(
            state,
            player_id,
            PSpriteNum::ps_flash as i32,
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].flashstate,
        );
    }
}
pub fn A_Punch(state: &mut GameState, player_id: PlayerId, _position: i32) {
    let player = player_id;
    let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
    let mut angle: angle_t = 0;
    let mut damage: i32 = 0;
    let mut slope: i32 = 0;
    damage = (P_Random(&mut state.m_random) % 10_i32 + 1_i32) << 1_i32;
    if state.g_game.players[player.0 as usize].powers[PowerType::pw_strength as usize] != 0 {
        damage *= 10_i32;
    }
    angle = state.p_mobj.mo(player_mo).angle;
    angle = angle.wrapping_add(
        ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 18_i32) as angle_t,
    );
    slope = P_AimLineAttack(state, Some(player_mo), angle, MELEERANGE);
    P_LineAttack(
        state,
        player_mo,
        angle,
        MELEERANGE,
        slope as fixed_t,
        damage,
    );
    if let Some(linetarget) = state.p_map.linetarget {
        let linetarget = state.p_mobj.mo(linetarget);
        let (linetarget_x, linetarget_y) = (linetarget.x, linetarget.y);
        S_StartSound(state, SoundOrigin::Mobj(player_mo), sfx_punch as i32);
        state.p_mobj.mo_mut(player_mo).angle = R_PointToAngle2(
            state,
            state.p_mobj.mo(player_mo).x,
            state.p_mobj.mo(player_mo).y,
            linetarget_x,
            linetarget_y,
        );
    }
}
pub fn A_Saw(state: &mut GameState, player_id: PlayerId, _position: i32) {
    let player = player_id;
    let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
    let mut angle: angle_t = 0;
    let mut damage: i32 = 0;
    let mut slope: i32 = 0;
    damage = 2_i32 * (P_Random(&mut state.m_random) % 10_i32 + 1_i32);
    angle = state.p_mobj.mo(player_mo).angle;
    angle = angle.wrapping_add(
        ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 18_i32) as angle_t,
    );
    slope = P_AimLineAttack(state, Some(player_mo), angle, MELEERANGE + 1 as fixed_t);
    P_LineAttack(
        state,
        player_mo,
        angle,
        MELEERANGE + 1 as fixed_t,
        slope as fixed_t,
        damage,
    );
    if state.p_map.linetarget.is_none() {
        S_StartSound(state, SoundOrigin::Mobj(player_mo), sfx_sawful as i32);
        return;
    }
    S_StartSound(state, SoundOrigin::Mobj(player_mo), sfx_sawhit as i32);
    let linetarget = state.p_mobj.mo(state.p_map.linetarget.unwrap());
    let (linetarget_x, linetarget_y) = (linetarget.x, linetarget.y);
    angle = R_PointToAngle2(
        state,
        state.p_mobj.mo(player_mo).x,
        state.p_mobj.mo(player_mo).y,
        linetarget_x,
        linetarget_y,
    );
    if angle.wrapping_sub(state.p_mobj.mo(player_mo).angle) > ANG180 {
        if (angle.wrapping_sub(state.p_mobj.mo(player_mo).angle) as i32) < -ANG90 / 20_i32 {
            state.p_mobj.mo_mut(player_mo).angle = angle.wrapping_add((ANG90 / 21_i32) as angle_t);
        } else {
            state.p_mobj.mo_mut(player_mo).angle = state
                .p_mobj
                .mo(player_mo)
                .angle
                .wrapping_sub((ANG90 / 20_i32) as angle_t);
        }
    } else if angle.wrapping_sub(state.p_mobj.mo(player_mo).angle) > (ANG90 / 20_i32) as angle_t {
        state.p_mobj.mo_mut(player_mo).angle = angle.wrapping_sub((ANG90 / 21_i32) as angle_t);
    } else {
        state.p_mobj.mo_mut(player_mo).angle = state
            .p_mobj
            .mo(player_mo)
            .angle
            .wrapping_add((ANG90 / 20_i32) as angle_t);
    }
    state.p_mobj.mo_mut(player_mo).flags |= MF_JUSTATTACKED as i32;
}
fn DecreaseAmmo(state: &mut GameState, player: PlayerId, ammonum: i32, amount: i32) {
    let player = state.g_game.player_mut(player);
    if ammonum < NUMAMMO {
        player.ammo[ammonum as usize] -= amount;
    } else {
        player.maxammo[(ammonum - NUMAMMO) as usize] -= amount;
    };
}
pub fn A_FireMissile(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
        DecreaseAmmo(
            state,
            player_id,
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].ammo as i32,
            1_i32,
        );
        P_SpawnPlayerMissile(state, player_mo, MobjType::MT_ROCKET);
    }
}
pub fn A_FireBFG(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
        DecreaseAmmo(
            state,
            player_id,
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].ammo as i32,
            deh_bfg_cells_per_shot,
        );
        P_SpawnPlayerMissile(state, player_mo, MobjType::MT_BFG);
    }
}
pub fn A_FirePlasma(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
        DecreaseAmmo(
            state,
            player_id,
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].ammo as i32,
            1_i32,
        );
        let flashstate = statenum_from_raw(
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].flashstate
                as i32
                + (P_Random(&mut state.m_random) & 1_i32),
        );
        P_SetPsprite(state, player_id, PSpriteNum::ps_flash as i32, flashstate);
        P_SpawnPlayerMissile(state, player_mo, MobjType::MT_PLASMA);
    }
}
pub fn P_BulletSlope(state: &mut GameState, mo: MobjId) {
    let mut an: angle_t = 0;
    an = state.p_mobj.mo(mo).angle;
    state.p_pspr.bulletslope = P_AimLineAttack(
        state,
        Some(mo),
        an,
        16 as fixed_t * 64 as fixed_t * FRACUNIT,
    );
    if state.p_map.linetarget.is_none() {
        an = an.wrapping_add((1_i32 << 26_i32) as angle_t);
        state.p_pspr.bulletslope = P_AimLineAttack(
            state,
            Some(mo),
            an,
            16 as fixed_t * 64 as fixed_t * FRACUNIT,
        );
        if state.p_map.linetarget.is_none() {
            an = an.wrapping_sub((2_i32 << 26_i32) as angle_t);
            state.p_pspr.bulletslope = P_AimLineAttack(
                state,
                Some(mo),
                an,
                16 as fixed_t * 64 as fixed_t * FRACUNIT,
            );
        }
    }
}
pub fn P_GunShot(state: &mut GameState, mo: MobjId, mut accurate: bool) {
    let mut angle: angle_t = 0;
    let mut damage: i32 = 0;
    damage = 5_i32 * (P_Random(&mut state.m_random) % 3_i32 + 1_i32);
    angle = state.p_mobj.mo(mo).angle;
    if !accurate {
        angle = angle.wrapping_add(
            ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 18_i32) as angle_t,
        );
    }
    let bulletslope = state.p_pspr.bulletslope;
    P_LineAttack(state, mo, angle, MISSILERANGE, bulletslope, damage);
}
pub fn A_FirePistol(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
        S_StartSound(state, SoundOrigin::Mobj(player_mo), sfx_pistol as i32);
        P_SetMobjState(state, player_mo, StateNum::S_PLAY_ATK2);
        DecreaseAmmo(
            state,
            player_id,
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].ammo as i32,
            1_i32,
        );
        P_SetPsprite(
            state,
            player_id,
            PSpriteNum::ps_flash as i32,
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].flashstate,
        );
        P_BulletSlope(state, player_mo);
        P_GunShot(
            state,
            player_mo,
            state.g_game.players[player.0 as usize].refire == 0,
        );
    }
}
pub fn A_FireShotgun(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
        let mut i: i32 = 0;
        S_StartSound(state, SoundOrigin::Mobj(player_mo), sfx_shotgn as i32);
        P_SetMobjState(state, player_mo, StateNum::S_PLAY_ATK2);
        DecreaseAmmo(
            state,
            player_id,
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].ammo as i32,
            1_i32,
        );
        P_SetPsprite(
            state,
            player_id,
            PSpriteNum::ps_flash as i32,
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].flashstate,
        );
        P_BulletSlope(state, player_mo);
        i = 0_i32;
        while i < 7_i32 {
            P_GunShot(state, player_mo, false);
            i += 1;
        }
    }
}
pub fn A_FireShotgun2(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
        let mut i: i32 = 0;
        let mut angle: angle_t = 0;
        let mut damage: i32 = 0;
        S_StartSound(state, SoundOrigin::Mobj(player_mo), sfx_dshtgn as i32);
        P_SetMobjState(state, player_mo, StateNum::S_PLAY_ATK2);
        DecreaseAmmo(
            state,
            player_id,
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].ammo as i32,
            2_i32,
        );
        P_SetPsprite(
            state,
            player_id,
            PSpriteNum::ps_flash as i32,
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].flashstate,
        );
        P_BulletSlope(state, player_mo);
        i = 0_i32;
        while i < 20_i32 {
            damage = 5_i32 * (P_Random(&mut state.m_random) % 3_i32 + 1_i32);
            angle = state.p_mobj.mo(player_mo).angle;
            angle = angle.wrapping_add(
                ((P_Random(&mut state.m_random) - P_Random(&mut state.m_random)) << 19_i32)
                    as angle_t,
            );
            let slope = state.p_pspr.bulletslope
                + ((P_Random(&mut state.m_random) as fixed_t
                    - P_Random(&mut state.m_random) as fixed_t)
                    << 5_i32);
            P_LineAttack(state, player_mo, angle, MISSILERANGE, slope, damage);
            i += 1;
        }
    }
}
pub fn A_FireCGun(state: &mut GameState, player_id: PlayerId, position: i32) {
    {
        let player = player_id;
        let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
        S_StartSound(state, SoundOrigin::Mobj(player_mo), sfx_pistol as i32);
        if state.g_game.players[player.0 as usize].ammo
            [weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].ammo as usize]
            == 0
        {
            return;
        }
        P_SetMobjState(state, player_mo, StateNum::S_PLAY_ATK2);
        DecreaseAmmo(
            state,
            player_id,
            weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].ammo as i32,
            1_i32,
        );
        P_SetPsprite(
            state,
            player_id,
            PSpriteNum::ps_flash as i32,
            statenum_from_raw(
                (weaponinfo[state.g_game.players[player.0 as usize].readyweapon as usize].flashstate
                    as i64
                    + state.g_game.players[player_id.0 as usize].psprites[position as usize]
                        .state
                        .unwrap()
                        .0 as i64
                    - StateNum::S_CHAIN1 as i64) as i32,
            ),
        );
        P_BulletSlope(state, player_mo);
        P_GunShot(
            state,
            player_mo,
            state.g_game.players[player.0 as usize].refire == 0,
        );
    }
}
pub fn A_Light0(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        state.g_game.players[player.0 as usize].extralight = 0_i32;
    }
}
pub fn A_Light1(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        state.g_game.players[player.0 as usize].extralight = 1_i32;
    }
}
pub fn A_Light2(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        state.g_game.players[player.0 as usize].extralight = 2_i32;
    }
}
pub fn A_BFGSpray(state: &mut GameState, id: MobjId) {
    let mo = id;
    let mo_target = state
        .p_mobj
        .mo(mo)
        .target
        .filter(|&target| state.p_mobj.is_live(target));
    for i in 0..40_i32 {
        let an: angle_t = state
            .p_mobj
            .mo(mo)
            .angle
            .wrapping_sub((ANG90 / 2_i32) as angle_t)
            .wrapping_add((ANG90 / 40_i32 * i) as angle_t);
        P_AimLineAttack(
            state,
            mo_target,
            an,
            16 as fixed_t * 64 as fixed_t * FRACUNIT,
        );
        if let Some(linetarget) = state.p_map.linetarget {
            let (lx, ly, lz, lheight) = {
                let l = state.p_mobj.mo(linetarget);
                (l.x, l.y, l.z, l.height)
            };
            P_SpawnMobj(
                state,
                lx,
                ly,
                lz + (lheight >> 2_i32),
                MobjType::MT_EXTRABFG,
            );
            let mut damage: i32 = 0_i32;
            for _ in 0..15_i32 {
                damage += (P_Random(&mut state.m_random) & 7_i32) + 1_i32;
            }
            P_DamageMobj(state, linetarget, mo_target, mo_target, damage);
        }
    }
}
pub fn A_BFGsound(state: &mut GameState, player_id: PlayerId, _position: i32) {
    {
        let player = player_id;
        let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
        S_StartSound(state, SoundOrigin::Mobj(player_mo), sfx_bfg as i32);
    }
}
pub fn P_SetupPsprites(state: &mut GameState, player_id: PlayerId) {
    let player = player_id;
    let mut i: i32 = 0;
    i = 0_i32;
    while i < NUMPSPRITES {
        state.g_game.players[player.0 as usize].psprites[i as usize].state = None;
        i += 1;
    }
    state.g_game.players[player.0 as usize].pendingweapon =
        state.g_game.players[player.0 as usize].readyweapon;
    P_BringUpWeapon(state, player_id);
}
pub fn P_MovePsprites(state: &mut GameState, player_id: PlayerId) {
    for i in 0..NUMPSPRITES {
        let psp_state = state.g_game.player_mut(player_id).psprites[i as usize].state;
        if let Some(psp_state) = psp_state
            .filter(|_| state.g_game.player_mut(player_id).psprites[i as usize].tics != -1_i32)
        {
            let psp = &mut state.g_game.player_mut(player_id).psprites[i as usize];
            psp.tics -= 1;
            if psp.tics == 0 {
                let nextstate = state.info.state_mut(psp_state).nextstate;
                P_SetPsprite(state, player_id, i, nextstate);
            }
        }
    }
    let player = state.g_game.player_mut(player_id);
    player.psprites[PSpriteNum::ps_flash as usize].sx =
        player.psprites[PSpriteNum::ps_weapon as usize].sx;
    player.psprites[PSpriteNum::ps_flash as usize].sy =
        player.psprites[PSpriteNum::ps_weapon as usize].sy;
}
