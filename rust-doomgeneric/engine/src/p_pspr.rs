use crate::d_items::WEAPONINFO;
use crate::d_mode::GameMode;
use crate::d_player::PlayerId;
use crate::d_player::PlayerState;
use crate::d_player::PowerType;
use crate::d_player::WeaponType;
use crate::d_player::{AmmoType, NUMAMMO};
use crate::d_player::{PSpriteNum, NUMPSPRITES};
use crate::d_ticcmd::BT_ATTACK;
use crate::g_game::GGameState;
use crate::game_state::GameState;
use crate::info::StateId;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACBITS;
use crate::m_fixed::FRACUNIT;
use crate::m_random::p_random;
use crate::p_enemy::noise_alert;
use crate::p_enemy::MELEERANGE;
use crate::p_enemy::MISSILERANGE;
use crate::p_inter::damage_mobj;
use crate::p_map::aim_line_attack;
use crate::p_map::line_attack;
use crate::p_mobj::set_mobj_state;
use crate::p_mobj::spawn_mobj;
use crate::p_mobj::spawn_player_missile;
use crate::p_mobj::MobjFlags;
use crate::p_mobj::MobjId;
use crate::p_mobj::MobjType;

use crate::p_mobj::StateAction;
use crate::p_mobj::{statenum_from_raw, StateNum};
use crate::r_main::point_to_angle2;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::tables::Angle;
use crate::tables::ANG180;
use crate::tables::ANG90;
use crate::tables::FINEANGLES;
use crate::tables::FINECOSINE;
use crate::tables::FINEMASK;
use crate::tables::FINESINE;

pub const DEH_DEFAULT_BFG_CELLS_PER_SHOT: i32 = 40;
pub const DEH_BFG_CELLS_PER_SHOT: i32 = DEH_DEFAULT_BFG_CELLS_PER_SHOT;
pub fn set_psprite(state: &mut GameState, player_id: PlayerId, position: i32, mut stnum: StateNum) {
    let pos = position as usize;
    loop {
        if stnum as u64 == 0 {
            state.game.g_game.player_mut(player_id).psprites[pos].state = None;
            break;
        }
        let state_id = StateId(stnum as u32);
        let (tics, misc1, misc2, action) = {
            let st = state.assets.info.state_mut(state_id);
            (st.tics, st.misc1, st.misc2, st.action)
        };
        {
            let psp = &mut state.game.g_game.player_mut(player_id).psprites[pos];
            psp.state = Some(state_id);
            psp.tics = tics;
            if misc1 != 0 {
                psp.sx = (misc1 << FRACBITS) as Fixed;
                psp.sy = (misc2 << FRACBITS) as Fixed;
            }
        }
        if let StateAction::Weapon(f) = action {
            f(state, player_id, position);
            if state.game.g_game.player_mut(player_id).psprites[pos]
                .state
                .is_none()
            {
                break;
            }
        }
        let current = state.game.g_game.player_mut(player_id).psprites[pos]
            .state
            .unwrap();
        stnum = state.assets.info.state_mut(current).nextstate;
        if state.game.g_game.player_mut(player_id).psprites[pos].tics != 0 {
            break;
        }
    }
}
pub struct PPsprState {
    pub bulletslope: Fixed,
}

impl Default for PPsprState {
    fn default() -> Self {
        Self::new()
    }
}

impl PPsprState {
    pub const fn new() -> Self {
        Self { bulletslope: 0 }
    }
}

pub fn bring_up_weapon(state: &mut GameState, player_id: PlayerId) {
    let player = player_id;
    let player_mo = state.game.g_game.players[player].mo.unwrap();

    if state.game.g_game.players[player].pendingweapon as u32 == WeaponType::Nochange as i32 as u32
    {
        state.game.g_game.players[player].pendingweapon =
            state.game.g_game.players[player].readyweapon;
    }
    if state.game.g_game.players[player].pendingweapon as u32 == WeaponType::Chainsaw as i32 as u32
    {
        s_start_sound(state, SoundOrigin::Mobj(player_mo), SfxName::Sawup);
    }
    let newstate: StateNum =
        WEAPONINFO[state.game.g_game.players[player].pendingweapon as usize].upstate;
    state.game.g_game.players[player].pendingweapon = WeaponType::Nochange;
    state.game.g_game.players[player].psprites[PSpriteNum::Weapon as usize].sy =
        (128 * FRACUNIT) as Fixed;
    set_psprite(state, player_id, PSpriteNum::Weapon as i32, newstate);
}
pub fn check_ammo(state: &mut GameState, player_id: PlayerId) -> bool {
    let player = player_id;

    let ammo: AmmoType = WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].ammo;
    let count: i32 =
        if state.game.g_game.players[player].readyweapon as u32 == WeaponType::Bfg as i32 as u32 {
            DEH_BFG_CELLS_PER_SHOT
        } else if state.game.g_game.players[player].readyweapon as u32
            == WeaponType::Supershotgun as i32 as u32
        {
            2
        } else {
            1
        };
    if ammo as u32 == AmmoType::Noammo as i32 as u32
        || state.game.g_game.players[player].ammo[ammo] >= count
    {
        return true;
    }
    loop {
        if state.game.g_game.players[player].weaponowned[WeaponType::Plasma]
            && state.game.g_game.players[player].ammo[AmmoType::Cell] != 0
            && state.game.doomstat.gamemode != GameMode::Shareware
        {
            state.game.g_game.players[player].pendingweapon = WeaponType::Plasma;
        } else if state.game.g_game.players[player].weaponowned[WeaponType::Supershotgun]
            && state.game.g_game.players[player].ammo[AmmoType::Shell] > 2
            && state.game.doomstat.gamemode == GameMode::Commercial
        {
            state.game.g_game.players[player].pendingweapon = WeaponType::Supershotgun;
        } else if state.game.g_game.players[player].weaponowned[WeaponType::Chaingun]
            && state.game.g_game.players[player].ammo[AmmoType::Clip] != 0
        {
            state.game.g_game.players[player].pendingweapon = WeaponType::Chaingun;
        } else if state.game.g_game.players[player].weaponowned[WeaponType::Shotgun]
            && state.game.g_game.players[player].ammo[AmmoType::Shell] != 0
        {
            state.game.g_game.players[player].pendingweapon = WeaponType::Shotgun;
        } else if state.game.g_game.players[player].ammo[AmmoType::Clip] != 0 {
            state.game.g_game.players[player].pendingweapon = WeaponType::Pistol;
        } else if state.game.g_game.players[player].weaponowned[WeaponType::Chainsaw] {
            state.game.g_game.players[player].pendingweapon = WeaponType::Chainsaw;
        } else if state.game.g_game.players[player].weaponowned[WeaponType::Missile]
            && state.game.g_game.players[player].ammo[AmmoType::Misl] != 0
        {
            state.game.g_game.players[player].pendingweapon = WeaponType::Missile;
        } else if state.game.g_game.players[player].weaponowned[WeaponType::Bfg]
            && state.game.g_game.players[player].ammo[AmmoType::Cell] > 40
            && state.game.doomstat.gamemode != GameMode::Shareware
        {
            state.game.g_game.players[player].pendingweapon = WeaponType::Bfg;
        } else {
            state.game.g_game.players[player].pendingweapon = WeaponType::Fist;
        }
        if state.game.g_game.players[player].pendingweapon as u32
            != WeaponType::Nochange as i32 as u32
        {
            break;
        }
    }
    set_psprite(
        state,
        player_id,
        PSpriteNum::Weapon as i32,
        WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].downstate,
    );
    false
}
pub fn fire_weapon(state: &mut GameState, player_id: PlayerId) {
    let player = player_id;
    let player_mo = state.game.g_game.players[player].mo.unwrap();

    if !check_ammo(state, player_id) {
        return;
    }
    set_mobj_state(state, player_mo, StateNum::PlayAtk1);
    let newstate: StateNum =
        WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].atkstate;
    set_psprite(state, player_id, PSpriteNum::Weapon as i32, newstate);
    noise_alert(state, player_mo, player_mo);
}
pub fn drop_weapon(state: &mut GameState, player_id: PlayerId) {
    let player = player_id;
    set_psprite(
        state,
        player_id,
        PSpriteNum::Weapon as i32,
        WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].downstate,
    );
}
pub fn weapon_ready(state: &mut GameState, player: PlayerId, position: i32) {
    let player_mo = state.game.g_game.players[player].mo.unwrap();
    let newstate: StateNum;
    if state.world.p_mobj.mo(player_mo).state == Some(StateId(StateNum::PlayAtk1 as u32))
        || state.world.p_mobj.mo(player_mo).state == Some(StateId(StateNum::PlayAtk2 as u32))
    {
        set_mobj_state(state, player_mo, StateNum::Play);
    }
    if state.game.g_game.players[player].readyweapon as u32 == WeaponType::Chainsaw as i32 as u32
        && state.game.g_game.players[player].psprites[position as usize].state
            == Some(StateId(StateNum::Saw as u32))
    {
        s_start_sound(state, SoundOrigin::Mobj(player_mo), SfxName::Sawidl);
    }
    if state.game.g_game.players[player].pendingweapon as u32 != WeaponType::Nochange as i32 as u32
        || state.game.g_game.players[player].health == 0
    {
        newstate = WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].downstate;
        set_psprite(state, player, PSpriteNum::Weapon as i32, newstate);
        return;
    }
    if state.game.g_game.players[player].cmd.buttons as i32 & BT_ATTACK != 0 {
        if !state.game.g_game.players[player].attackdown
            || state.game.g_game.players[player].readyweapon as u32
                != WeaponType::Missile as i32 as u32
                && state.game.g_game.players[player].readyweapon as u32
                    != WeaponType::Bfg as i32 as u32
        {
            state.game.g_game.players[player].attackdown = true;
            fire_weapon(state, player);
            return;
        }
    } else {
        state.game.g_game.players[player].attackdown = false;
    }
    let mut angle: i32 = (128 * state.world.p_tick.leveltime) & FINEMASK;
    state.game.g_game.players[player].psprites[position as usize].sx = FRACUNIT
        + fixed_mul(
            state.game.g_game.players[player].bob,
            FINECOSINE[angle as usize],
        );
    angle &= FINEANGLES / 2 - 1;
    state.game.g_game.players[player].psprites[position as usize].sy = 32 * FRACUNIT
        + fixed_mul(
            state.game.g_game.players[player].bob,
            FINESINE[angle as usize],
        );
}
pub fn re_fire(state: &mut GameState, player: PlayerId, _position: i32) {
    if state.game.g_game.players[player].cmd.buttons as i32 & BT_ATTACK != 0
        && state.game.g_game.players[player].pendingweapon as u32
            == WeaponType::Nochange as i32 as u32
        && state.game.g_game.players[player].health != 0
    {
        state.game.g_game.players[player].refire += 1;
        fire_weapon(state, player);
    } else {
        state.game.g_game.players[player].refire = 0;
        check_ammo(state, player);
    }
}
pub fn check_reload(state: &mut GameState, player_id: PlayerId, _position: i32) {
    check_ammo(state, player_id);
}
pub fn lower(state: &mut GameState, player: PlayerId, position: i32) {
    state.game.g_game.players[player].psprites[position as usize].sy += FRACUNIT * 6;
    if state.game.g_game.players[player].psprites[position as usize].sy < 128 * FRACUNIT {
        return;
    }
    if state.game.g_game.players[player].playerstate == PlayerState::Dead {
        state.game.g_game.players[player].psprites[position as usize].sy =
            (128 * FRACUNIT) as Fixed;
        return;
    }
    if state.game.g_game.players[player].health == 0 {
        set_psprite(state, player, PSpriteNum::Weapon as i32, StateNum::Null);
        return;
    }
    state.game.g_game.players[player].readyweapon = state.game.g_game.players[player].pendingweapon;
    bring_up_weapon(state, player);
}
pub fn raise(state: &mut GameState, player: PlayerId, position: i32) {
    state.game.g_game.players[player].psprites[position as usize].sy -= FRACUNIT * 6;
    if state.game.g_game.players[player].psprites[position as usize].sy > 32 * FRACUNIT {
        return;
    }
    state.game.g_game.players[player].psprites[position as usize].sy = (32 * FRACUNIT) as Fixed;
    let newstate: StateNum =
        WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].readystate;
    set_psprite(state, player, PSpriteNum::Weapon as i32, newstate);
}
pub fn gun_flash(state: &mut GameState, player: PlayerId, _position: i32) {
    let player_mo = state.game.g_game.players[player].mo.unwrap();
    set_mobj_state(state, player_mo, StateNum::PlayAtk2);
    set_psprite(
        state,
        player,
        PSpriteNum::Flash as i32,
        WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].flashstate,
    );
}
pub fn punch(state: &mut GameState, player_id: PlayerId, _position: i32) {
    let player = player_id;
    let player_mo = state.game.g_game.players[player].mo.unwrap();

    let mut damage: i32 = (p_random(&mut state.world.m_random) % 10 + 1) << 1;
    if state.game.g_game.players[player].powers[PowerType::Strength] != 0 {
        damage *= 10;
    }
    let mut angle: Angle = state.world.p_mobj.mo(player_mo).angle;
    angle = angle.wrapping_add(
        ((p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 18)
            as Angle,
    );
    let slope: i32 = aim_line_attack(state, Some(player_mo), angle, MELEERANGE);
    line_attack(state, player_mo, angle, MELEERANGE, slope as Fixed, damage);
    if let Some(linetarget) = state.world.p_map.linetarget {
        let linetarget = state.world.p_mobj.mo(linetarget);
        let (linetarget_x, linetarget_y) = (linetarget.x, linetarget.y);
        s_start_sound(state, SoundOrigin::Mobj(player_mo), SfxName::Punch);
        state.world.p_mobj.mo_mut(player_mo).angle = point_to_angle2(
            &mut state.render.r_main,
            state.world.p_mobj.mo(player_mo).x,
            state.world.p_mobj.mo(player_mo).y,
            linetarget_x,
            linetarget_y,
        );
    }
}
pub fn saw(state: &mut GameState, player_id: PlayerId, _position: i32) {
    let player = player_id;
    let player_mo = state.game.g_game.players[player].mo.unwrap();

    let damage: i32 = 2 * (p_random(&mut state.world.m_random) % 10 + 1);
    let mut angle: Angle = state.world.p_mobj.mo(player_mo).angle;
    angle = angle.wrapping_add(
        ((p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 18)
            as Angle,
    );
    let slope: i32 = aim_line_attack(state, Some(player_mo), angle, MELEERANGE + 1);
    line_attack(
        state,
        player_mo,
        angle,
        MELEERANGE + 1,
        slope as Fixed,
        damage,
    );
    if state.world.p_map.linetarget.is_none() {
        s_start_sound(state, SoundOrigin::Mobj(player_mo), SfxName::Sawful);
        return;
    }
    s_start_sound(state, SoundOrigin::Mobj(player_mo), SfxName::Sawhit);
    let linetarget = state.world.p_mobj.mo(state.world.p_map.linetarget.unwrap());
    let (linetarget_x, linetarget_y) = (linetarget.x, linetarget.y);
    angle = point_to_angle2(
        &mut state.render.r_main,
        state.world.p_mobj.mo(player_mo).x,
        state.world.p_mobj.mo(player_mo).y,
        linetarget_x,
        linetarget_y,
    );
    if angle.wrapping_sub(state.world.p_mobj.mo(player_mo).angle) > ANG180 {
        if (angle.wrapping_sub(state.world.p_mobj.mo(player_mo).angle) as i32) < -ANG90 / 20 {
            state.world.p_mobj.mo_mut(player_mo).angle = angle.wrapping_add((ANG90 / 21) as Angle);
        } else {
            state.world.p_mobj.mo_mut(player_mo).angle = state
                .world
                .p_mobj
                .mo(player_mo)
                .angle
                .wrapping_sub((ANG90 / 20) as Angle);
        }
    } else if angle.wrapping_sub(state.world.p_mobj.mo(player_mo).angle) > (ANG90 / 20) as Angle {
        state.world.p_mobj.mo_mut(player_mo).angle = angle.wrapping_sub((ANG90 / 21) as Angle);
    } else {
        state.world.p_mobj.mo_mut(player_mo).angle = state
            .world
            .p_mobj
            .mo(player_mo)
            .angle
            .wrapping_add((ANG90 / 20) as Angle);
    }
    state.world.p_mobj.mo_mut(player_mo).flags |= MobjFlags::JUSTATTACKED;
}
fn decrease_ammo(g_game: &mut GGameState, player: PlayerId, ammonum: i32, amount: i32) {
    let player = g_game.player_mut(player);
    if ammonum < NUMAMMO {
        player.ammo[ammonum as usize] -= amount;
    } else {
        player.maxammo[(ammonum - NUMAMMO) as usize] -= amount;
    }
}
pub fn fire_missile(state: &mut GameState, player: PlayerId, _position: i32) {
    let player_mo = state.game.g_game.players[player].mo.unwrap();
    let ammo_type = WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].ammo as i32;
    decrease_ammo(&mut state.game.g_game, player, ammo_type, 1);
    spawn_player_missile(state, player_mo, MobjType::Rocket);
}
pub fn fire_bfg(state: &mut GameState, player: PlayerId, _position: i32) {
    let player_mo = state.game.g_game.players[player].mo.unwrap();
    let ammo_type = WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].ammo as i32;
    decrease_ammo(
        &mut state.game.g_game,
        player,
        ammo_type,
        DEH_BFG_CELLS_PER_SHOT,
    );
    spawn_player_missile(state, player_mo, MobjType::Bfg);
}
pub fn fire_plasma(state: &mut GameState, player: PlayerId, _position: i32) {
    let player_mo = state.game.g_game.players[player].mo.unwrap();
    let ammo_type = WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].ammo as i32;
    decrease_ammo(&mut state.game.g_game, player, ammo_type, 1);
    let flashstate = statenum_from_raw(
        WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].flashstate as i32
            + (p_random(&mut state.world.m_random) & 1),
    );
    set_psprite(state, player, PSpriteNum::Flash as i32, flashstate);
    spawn_player_missile(state, player_mo, MobjType::Plasma);
}
pub fn bullet_slope(state: &mut GameState, mo: MobjId) {
    let mut an: Angle = state.world.p_mobj.mo(mo).angle;
    state.world.p_pspr.bulletslope = aim_line_attack(state, Some(mo), an, 16 * 64 * FRACUNIT);
    if state.world.p_map.linetarget.is_none() {
        an = an.wrapping_add((1 << 26) as Angle);
        state.world.p_pspr.bulletslope = aim_line_attack(state, Some(mo), an, 16 * 64 * FRACUNIT);
        if state.world.p_map.linetarget.is_none() {
            an = an.wrapping_sub((2 << 26) as Angle);
            state.world.p_pspr.bulletslope =
                aim_line_attack(state, Some(mo), an, 16 * 64 * FRACUNIT);
        }
    }
}
pub fn gun_shot(state: &mut GameState, mo: MobjId, accurate: bool) {
    let damage: i32 = 5 * (p_random(&mut state.world.m_random) % 3 + 1);
    let mut angle: Angle = state.world.p_mobj.mo(mo).angle;
    if !accurate {
        angle = angle.wrapping_add(
            ((p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 18)
                as Angle,
        );
    }
    let bulletslope = state.world.p_pspr.bulletslope;
    line_attack(state, mo, angle, MISSILERANGE, bulletslope, damage);
}
pub fn fire_pistol(state: &mut GameState, player: PlayerId, _position: i32) {
    let player_mo = state.game.g_game.players[player].mo.unwrap();
    s_start_sound(state, SoundOrigin::Mobj(player_mo), SfxName::Pistol);
    set_mobj_state(state, player_mo, StateNum::PlayAtk2);
    let ammo_type = WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].ammo as i32;
    decrease_ammo(&mut state.game.g_game, player, ammo_type, 1);
    set_psprite(
        state,
        player,
        PSpriteNum::Flash as i32,
        WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].flashstate,
    );
    bullet_slope(state, player_mo);
    gun_shot(
        state,
        player_mo,
        state.game.g_game.players[player].refire == 0,
    );
}
pub fn fire_shotgun(state: &mut GameState, player: PlayerId, _position: i32) {
    let player_mo = state.game.g_game.players[player].mo.unwrap();
    s_start_sound(state, SoundOrigin::Mobj(player_mo), SfxName::Shotgn);
    set_mobj_state(state, player_mo, StateNum::PlayAtk2);
    let ammo_type = WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].ammo as i32;
    decrease_ammo(&mut state.game.g_game, player, ammo_type, 1);
    set_psprite(
        state,
        player,
        PSpriteNum::Flash as i32,
        WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].flashstate,
    );
    bullet_slope(state, player_mo);
    for _ in 0..7 {
        gun_shot(state, player_mo, false);
    }
}
pub fn fire_shotgun2(state: &mut GameState, player: PlayerId, _position: i32) {
    let player_mo = state.game.g_game.players[player].mo.unwrap();
    let mut angle: Angle;
    let mut damage: i32;
    s_start_sound(state, SoundOrigin::Mobj(player_mo), SfxName::Dshtgn);
    set_mobj_state(state, player_mo, StateNum::PlayAtk2);
    let ammo_type = WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].ammo as i32;
    decrease_ammo(&mut state.game.g_game, player, ammo_type, 2);
    set_psprite(
        state,
        player,
        PSpriteNum::Flash as i32,
        WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].flashstate,
    );
    bullet_slope(state, player_mo);
    for _ in 0..20 {
        damage = 5 * (p_random(&mut state.world.m_random) % 3 + 1);
        angle = state.world.p_mobj.mo(player_mo).angle;
        angle = angle.wrapping_add(
            ((p_random(&mut state.world.m_random) - p_random(&mut state.world.m_random)) << 19)
                as Angle,
        );
        let slope = state.world.p_pspr.bulletslope
            + ((p_random(&mut state.world.m_random) as Fixed
                - p_random(&mut state.world.m_random) as Fixed)
                << 5);
        line_attack(state, player_mo, angle, MISSILERANGE, slope, damage);
    }
}
pub fn fire_cgun(state: &mut GameState, player: PlayerId, position: i32) {
    let player_mo = state.game.g_game.players[player].mo.unwrap();
    s_start_sound(state, SoundOrigin::Mobj(player_mo), SfxName::Pistol);
    if state.game.g_game.players[player].ammo
        [WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].ammo as usize]
        == 0
    {
        return;
    }
    set_mobj_state(state, player_mo, StateNum::PlayAtk2);
    let ammo_type = WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].ammo as i32;
    decrease_ammo(&mut state.game.g_game, player, ammo_type, 1);
    set_psprite(
        state,
        player,
        PSpriteNum::Flash as i32,
        statenum_from_raw(
            (WEAPONINFO[state.game.g_game.players[player].readyweapon as usize].flashstate as i64
                + state.game.g_game.players[player].psprites[position as usize]
                    .state
                    .unwrap()
                    .0 as i64
                - StateNum::Chain1 as i64) as i32,
        ),
    );
    bullet_slope(state, player_mo);
    gun_shot(
        state,
        player_mo,
        state.game.g_game.players[player].refire == 0,
    );
}
pub fn light0(state: &mut GameState, player: PlayerId, _position: i32) {
    state.game.g_game.players[player].extralight = 0;
}
pub fn light1(state: &mut GameState, player: PlayerId, _position: i32) {
    state.game.g_game.players[player].extralight = 1;
}
pub fn light2(state: &mut GameState, player: PlayerId, _position: i32) {
    state.game.g_game.players[player].extralight = 2;
}
pub fn bfgspray(state: &mut GameState, id: MobjId) {
    let mo = id;
    let mo_target = state
        .world
        .p_mobj
        .mo(mo)
        .target
        .filter(|&target| state.world.p_mobj.is_live(target));
    for i in 0..40_i32 {
        let an: Angle = state
            .world
            .p_mobj
            .mo(mo)
            .angle
            .wrapping_sub((ANG90 / 2) as Angle)
            .wrapping_add((ANG90 / 40 * i) as Angle);
        aim_line_attack(state, mo_target, an, 16 * 64 * FRACUNIT);
        if let Some(linetarget) = state.world.p_map.linetarget {
            let (lx, ly, lz, lheight) = {
                let l = state.world.p_mobj.mo(linetarget);
                (l.x, l.y, l.z, l.height)
            };
            spawn_mobj(state, lx, ly, lz + (lheight >> 2), MobjType::Extrabfg);
            let mut damage: i32 = 0;
            for _ in 0..15_i32 {
                damage += (p_random(&mut state.world.m_random) & 7) + 1;
            }
            damage_mobj(state, linetarget, mo_target, mo_target, damage);
        }
    }
}
pub fn bfgsound(state: &mut GameState, player: PlayerId, _position: i32) {
    let player_mo = state.game.g_game.players[player].mo.unwrap();
    s_start_sound(state, SoundOrigin::Mobj(player_mo), SfxName::Bfg);
}
pub fn setup_psprites(state: &mut GameState, player_id: PlayerId) {
    let player = player_id;
    for i in 0..(NUMPSPRITES as usize) {
        state.game.g_game.players[player].psprites[i].state = None;
    }
    state.game.g_game.players[player].pendingweapon = state.game.g_game.players[player].readyweapon;
    bring_up_weapon(state, player_id);
}
pub fn move_psprites(state: &mut GameState, player_id: PlayerId) {
    for i in 0..NUMPSPRITES {
        let psp_state = state.game.g_game.player_mut(player_id).psprites[i as usize].state;
        if let Some(psp_state) = psp_state
            .filter(|_| state.game.g_game.player_mut(player_id).psprites[i as usize].tics != -1)
        {
            let psp = &mut state.game.g_game.player_mut(player_id).psprites[i as usize];
            psp.tics -= 1;
            if psp.tics == 0 {
                let nextstate = state.assets.info.state_mut(psp_state).nextstate;
                set_psprite(state, player_id, i, nextstate);
            }
        }
    }
    let player = state.game.g_game.player_mut(player_id);
    player.psprites[PSpriteNum::Flash as usize].sx =
        player.psprites[PSpriteNum::Weapon as usize].sx;
    player.psprites[PSpriteNum::Flash as usize].sy =
        player.psprites[PSpriteNum::Weapon as usize].sy;
}
