use crate::d_mode::GameMode;
use crate::d_player::CheatFlags;
use crate::d_player::PlayerId;
use crate::d_player::PlayerState;
use crate::d_player::PowerType;
use crate::d_player::{weapontype_from_raw, WeaponType};
use crate::p_mobj::MobjFlags;
use crate::p_mobj::PMobjState;

use crate::d_ticcmd::{BT_CHANGE, BT_SPECIAL, BT_USE, BT_WEAPONMASK, BT_WEAPONSHIFT};
use crate::game_state::GameState;
use crate::info::StateId;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACUNIT;
use crate::p_map::use_lines;
use crate::p_mobj::set_mobj_state;
use crate::p_mobj::MobjId;
use crate::p_mobj::StateNum;
use crate::p_pspr::move_psprites;
use crate::p_spec::player_in_special_sector;
use crate::r_main::point_to_angle2;
use crate::tables::Angle;
use crate::tables::ANG180;
use crate::tables::ANG90;
use crate::tables::ANGLETOFINESHIFT;
use crate::tables::FINEANGLES;
use crate::tables::FINECOSINE;
use crate::tables::FINEMASK;
use crate::tables::FINESINE;

pub const VIEWHEIGHT: i32 = 41 * FRACUNIT;
pub const INVERSECOLORMAP: i32 = 32;
pub const MAXBOB: i32 = 0x100000;
pub struct PUserState {
    onground: bool,
}

impl Default for PUserState {
    fn default() -> Self {
        Self::new()
    }
}

impl PUserState {
    pub const fn new() -> Self {
        Self { onground: false }
    }
}

pub fn p_thrust(p_mobj: &mut PMobjState, mo: MobjId, mut angle: Angle, amount: Fixed) {
    angle >>= ANGLETOFINESHIFT;
    let mo = p_mobj.mo_mut(mo);
    mo.momx += fixed_mul(amount, FINECOSINE[angle as usize]);
    mo.momy += fixed_mul(amount, FINESINE[angle as usize]);
}
pub fn calc_height(state: &mut GameState, player_id: PlayerId) {
    let player = &mut state.game.g_game.players[player_id];

    let player_mo = player.mo.unwrap();
    player.bob = fixed_mul(
        state.world.p_mobj.mo(player_mo).momx,
        state.world.p_mobj.mo(player_mo).momx,
    ) + fixed_mul(
        state.world.p_mobj.mo(player_mo).momy,
        state.world.p_mobj.mo(player_mo).momy,
    );
    player.bob >>= 2;
    if player.bob > MAXBOB {
        player.bob = MAXBOB as Fixed;
    }
    if player.cheats.contains(CheatFlags::NOMOMENTUM) || !state.world.p_user.onground {
        player.viewz = (state.world.p_mobj.mo(player_mo).z + VIEWHEIGHT) as Fixed;
        if player.viewz > state.world.p_mobj.mo(player_mo).ceilingz - 4 * FRACUNIT {
            player.viewz = (state.world.p_mobj.mo(player_mo).ceilingz - 4 * FRACUNIT) as Fixed;
        }
        player.viewz = state.world.p_mobj.mo(player_mo).z + player.viewheight;
        return;
    }
    let angle: i32 = (FINEANGLES / 20 * state.world.p_tick.leveltime) & FINEMASK;
    let bob: Fixed = fixed_mul(player.bob / 2, FINESINE[angle as usize]);
    if player.playerstate == PlayerState::Live {
        player.viewheight += player.deltaviewheight;
        if player.viewheight > VIEWHEIGHT {
            player.viewheight = VIEWHEIGHT as Fixed;
            player.deltaviewheight = 0;
        }
        if player.viewheight < VIEWHEIGHT / 2 {
            player.viewheight = (VIEWHEIGHT / 2) as Fixed;
            if player.deltaviewheight <= 0 {
                player.deltaviewheight = 1;
            }
        }
        if player.deltaviewheight != 0 {
            player.deltaviewheight += FRACUNIT / 4;
            if player.deltaviewheight == 0 {
                player.deltaviewheight = 1;
            }
        }
    }
    player.viewz = state.world.p_mobj.mo(player_mo).z + player.viewheight + bob;
    if player.viewz > state.world.p_mobj.mo(player_mo).ceilingz - 4 * FRACUNIT {
        player.viewz = (state.world.p_mobj.mo(player_mo).ceilingz - 4 * FRACUNIT) as Fixed;
    }
}
pub fn move_player(state: &mut GameState, player_id: PlayerId) {
    let cmd = state.game.g_game.players[player_id].cmd;
    let player_mo = state.game.g_game.players[player_id].mo.unwrap();
    {
        let mo = state.world.p_mobj.mo_mut(player_mo);
        mo.angle = mo
            .angle
            .wrapping_add(((cmd.angleturn as i32) << 16) as Angle);
    }
    let (z, floorz, angle) = {
        let mo = state.world.p_mobj.mo(player_mo);
        (mo.z, mo.floorz, mo.angle)
    };
    state.world.p_user.onground = z <= floorz;
    if cmd.forwardmove as i32 != 0 && state.world.p_user.onground {
        p_thrust(
            &mut state.world.p_mobj,
            player_mo,
            angle,
            cmd.forwardmove as Fixed * 2048,
        );
    }
    if cmd.sidemove as i32 != 0 && state.world.p_user.onground {
        p_thrust(
            &mut state.world.p_mobj,
            player_mo,
            angle.wrapping_sub(ANG90 as Angle),
            cmd.sidemove as Fixed * 2048,
        );
    }
    if (cmd.forwardmove as i32 != 0 || cmd.sidemove as i32 != 0)
        && state.world.p_mobj.mo(player_mo).state == Some(StateId(StateNum::Play as u32))
    {
        set_mobj_state(state, player_mo, StateNum::PlayRun1);
    }
}
pub const ANG5: i32 = ANG90 / 18;
pub fn death_think(state: &mut GameState, player_id: PlayerId) {
    let player = player_id;
    let angle: Angle;
    let delta: Angle;
    move_psprites(state, player_id);
    if state.game.g_game.players[player].viewheight > 6 * FRACUNIT {
        state.game.g_game.players[player].viewheight -= FRACUNIT;
    }
    if state.game.g_game.players[player].viewheight < 6 * FRACUNIT {
        state.game.g_game.players[player].viewheight = (6 * FRACUNIT) as Fixed;
    }
    state.game.g_game.players[player].deltaviewheight = 0;
    let player_mo = state.game.g_game.players[player].mo.unwrap();
    state.world.p_user.onground =
        state.world.p_mobj.mo(player_mo).z <= state.world.p_mobj.mo(player_mo).floorz;
    calc_height(state, player);
    if state.game.g_game.players[player].attacker.is_some()
        && state.game.g_game.players[player].attacker != state.game.g_game.players[player].mo
    {
        let attacker = state.game.g_game.players[player].attacker.unwrap();
        angle = point_to_angle2(
            &mut state.render.r_main,
            state.world.p_mobj.mo(player_mo).x,
            state.world.p_mobj.mo(player_mo).y,
            state.world.p_mobj.mo(attacker).x,
            state.world.p_mobj.mo(attacker).y,
        );
        delta = angle.wrapping_sub(state.world.p_mobj.mo(player_mo).angle);
        if delta < ANG5 as Angle || delta > -ANG5 as u32 {
            state.world.p_mobj.mo_mut(player_mo).angle = angle;
            if state.game.g_game.players[player].damagecount != 0 {
                state.game.g_game.players[player].damagecount -= 1;
            }
        } else if delta < ANG180 {
            state.world.p_mobj.mo_mut(player_mo).angle = state
                .world
                .p_mobj
                .mo(player_mo)
                .angle
                .wrapping_add(ANG5 as Angle);
        } else {
            state.world.p_mobj.mo_mut(player_mo).angle = state
                .world
                .p_mobj
                .mo(player_mo)
                .angle
                .wrapping_sub(ANG5 as Angle);
        }
    } else if state.game.g_game.players[player].damagecount != 0 {
        state.game.g_game.players[player].damagecount -= 1;
    }
    if state.game.g_game.players[player].cmd.buttons as i32 & BT_USE != 0 {
        state.game.g_game.players[player].playerstate = PlayerState::Reborn;
    }
}
pub fn player_think(state: &mut GameState, player_id: PlayerId) {
    let player = player_id;
    let mut newweapon: WeaponType;
    let player_mo = state.game.g_game.players[player].mo.unwrap();
    if state.game.g_game.players[player]
        .cheats
        .contains(CheatFlags::NOCLIP)
    {
        state.world.p_mobj.mo_mut(player_mo).flags |= MobjFlags::NOCLIP;
    } else {
        state.world.p_mobj.mo_mut(player_mo).flags &= !MobjFlags::NOCLIP;
    }
    if state
        .world
        .p_mobj
        .mo(player_mo)
        .flags
        .contains(MobjFlags::JUSTATTACKED)
    {
        state.game.g_game.players[player_id].cmd.angleturn = 0;
        state.game.g_game.players[player_id].cmd.forwardmove = (0xc800 / 512) as i8;
        state.game.g_game.players[player_id].cmd.sidemove = 0;
        state.world.p_mobj.mo_mut(player_mo).flags &= !MobjFlags::JUSTATTACKED;
    }
    if state.game.g_game.players[player].playerstate == PlayerState::Dead {
        death_think(state, player_id);
        return;
    }
    if state.world.p_mobj.mo(player_mo).reactiontime != 0 {
        state.world.p_mobj.mo_mut(player_mo).reactiontime -= 1;
    } else {
        move_player(state, player_id);
    }
    calc_height(state, player_id);
    if state
        .world
        .p_setup
        .sector_mut(
            state.world.p_setup.subsectors[state.world.p_mobj.mo(player_mo).subsector.0 as usize]
                .sector,
        )
        .special
        != 0
    {
        player_in_special_sector(state, player_id);
    }
    if state.game.g_game.players[player_id].cmd.buttons as i32 & BT_SPECIAL != 0 {
        state.game.g_game.players[player_id].cmd.buttons = 0_u8;
    }
    if state.game.g_game.players[player_id].cmd.buttons as i32 & BT_CHANGE != 0 {
        newweapon = weapontype_from_raw(
            (state.game.g_game.players[player_id].cmd.buttons as i32 & BT_WEAPONMASK)
                >> BT_WEAPONSHIFT,
        );
        if newweapon as u32 == WeaponType::Fist as i32 as u32
            && state.game.g_game.players[player].weaponowned[WeaponType::Chainsaw as usize]
            && !(state.game.g_game.players[player].readyweapon as u32
                == WeaponType::Chainsaw as i32 as u32
                && state.game.g_game.players[player].powers[PowerType::Strength as usize] != 0)
        {
            newweapon = WeaponType::Chainsaw;
        }
        if state.game.doomstat.gamemode == GameMode::Commercial
            && newweapon as u32 == WeaponType::Shotgun as i32 as u32
            && state.game.g_game.players[player].weaponowned[WeaponType::Supershotgun as usize]
            && state.game.g_game.players[player].readyweapon as u32
                != WeaponType::Supershotgun as i32 as u32
        {
            newweapon = WeaponType::Supershotgun;
        }
        if state.game.g_game.players[player].weaponowned[newweapon as usize]
            && newweapon as u32 != state.game.g_game.players[player].readyweapon as u32
            && (newweapon as u32 != WeaponType::Plasma as i32 as u32
                && newweapon as u32 != WeaponType::Bfg as i32 as u32
                || state.game.doomstat.gamemode != GameMode::Shareware)
        {
            state.game.g_game.players[player].pendingweapon = newweapon;
        }
    }
    if state.game.g_game.players[player_id].cmd.buttons as i32 & BT_USE != 0 {
        if !state.game.g_game.players[player].usedown {
            use_lines(state, player_id);
            state.game.g_game.players[player].usedown = true;
        }
    } else {
        state.game.g_game.players[player].usedown = false;
    }
    move_psprites(state, player_id);
    if state.game.g_game.players[player].powers[PowerType::Strength as usize] != 0 {
        state.game.g_game.players[player].powers[PowerType::Strength as usize] += 1;
    }
    if state.game.g_game.players[player].powers[PowerType::Invulnerability as usize] != 0 {
        state.game.g_game.players[player].powers[PowerType::Invulnerability as usize] -= 1;
    }
    if state.game.g_game.players[player].powers[PowerType::Invisibility as usize] != 0 {
        state.game.g_game.players[player].powers[PowerType::Invisibility as usize] -= 1;
        if state.game.g_game.players[player].powers[PowerType::Invisibility as usize] == 0 {
            state.world.p_mobj.mo_mut(player_mo).flags &= !MobjFlags::SHADOW;
        }
    }
    if state.game.g_game.players[player].powers[PowerType::Infrared as usize] != 0 {
        state.game.g_game.players[player].powers[PowerType::Infrared as usize] -= 1;
    }
    if state.game.g_game.players[player].powers[PowerType::Ironfeet as usize] != 0 {
        state.game.g_game.players[player].powers[PowerType::Ironfeet as usize] -= 1;
    }
    if state.game.g_game.players[player].damagecount != 0 {
        state.game.g_game.players[player].damagecount -= 1;
    }
    if state.game.g_game.players[player].bonuscount != 0 {
        state.game.g_game.players[player].bonuscount -= 1;
    }
    if state.game.g_game.players[player].powers[PowerType::Invulnerability as usize] != 0 {
        if state.game.g_game.players[player].powers[PowerType::Invulnerability as usize] > 4 * 32
            || state.game.g_game.players[player].powers[PowerType::Invulnerability as usize] & 8
                != 0
        {
            state.game.g_game.players[player].fixedcolormap = INVERSECOLORMAP;
        } else {
            state.game.g_game.players[player].fixedcolormap = 0;
        }
    } else if state.game.g_game.players[player].powers[PowerType::Infrared as usize] != 0 {
        if state.game.g_game.players[player].powers[PowerType::Infrared as usize] > 4 * 32
            || state.game.g_game.players[player].powers[PowerType::Infrared as usize] & 8 != 0
        {
            state.game.g_game.players[player].fixedcolormap = 1;
        } else {
            state.game.g_game.players[player].fixedcolormap = 0;
        }
    } else {
        state.game.g_game.players[player].fixedcolormap = 0;
    }
}
