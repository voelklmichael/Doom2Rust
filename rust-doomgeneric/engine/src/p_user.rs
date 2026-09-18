use crate::d_mode::GameMode_t;
use crate::d_player::PlayerId;
use crate::d_player::PlayerState;
use crate::d_player::PowerType;
use crate::d_player::{weapontype_from_raw, weapontype_t};
use crate::d_player::{CF_NOCLIP, CF_NOMOMENTUM};

use crate::d_ticcmd::{BT_CHANGE, BT_SPECIAL, BT_USE, BT_WEAPONMASK, BT_WEAPONSHIFT};
use crate::doomdef::false_0;
use crate::doomdef::true_0;
use crate::game_state::GameState;
use crate::info::StateId;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FRACUNIT;
use crate::p_map::P_UseLines;
use crate::p_mobj::MobjId;
use crate::p_mobj::P_SetMobjState;
use crate::p_mobj::StateNum;
use crate::p_mobj::{MF_JUSTATTACKED, MF_NOCLIP, MF_SHADOW};
use crate::p_pspr::P_MovePsprites;
use crate::p_spec::P_PlayerInSpecialSector;
use crate::r_main::R_PointToAngle2;
use crate::stdint_types::byte;
use crate::tables::angle_t;
use crate::tables::finecosine;
use crate::tables::finesine;
use crate::tables::ANG180;
use crate::tables::ANG90;
use crate::tables::ANGLETOFINESHIFT;
use crate::tables::FINEANGLES;
use crate::tables::FINEMASK;

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
        PUserState { onground: false }
    }
}

pub fn P_Thrust(state: &mut GameState, mo: MobjId, mut angle: angle_t, move_0: fixed_t) {
    angle >>= ANGLETOFINESHIFT;
    let mo = state.p_mobj.mo_mut(mo);
    mo.momx += FixedMul(move_0, finecosine[angle as isize]);
    mo.momy += FixedMul(move_0, finesine[angle as usize]);
}
pub fn P_CalcHeight(state: &mut GameState, player_id: PlayerId) {
    let player = &mut state.g_game.players[player_id.0 as usize];
    let mut angle: i32 = 0;
    let mut bob: fixed_t = 0;
    let player_mo = player.mo.unwrap();
    player.bob = FixedMul(
        state.p_mobj.mo(player_mo).momx,
        state.p_mobj.mo(player_mo).momx,
    ) + FixedMul(
        state.p_mobj.mo(player_mo).momy,
        state.p_mobj.mo(player_mo).momy,
    );
    player.bob >>= 2_i32;
    if player.bob > MAXBOB {
        player.bob = MAXBOB as fixed_t;
    }
    if player.cheats & CF_NOMOMENTUM != 0 || !state.p_user.onground {
        player.viewz = (state.p_mobj.mo(player_mo).z + VIEWHEIGHT) as fixed_t;
        if player.viewz > state.p_mobj.mo(player_mo).ceilingz - 4_i32 * FRACUNIT {
            player.viewz = (state.p_mobj.mo(player_mo).ceilingz - 4_i32 * FRACUNIT) as fixed_t;
        }
        player.viewz = state.p_mobj.mo(player_mo).z + player.viewheight;
        return;
    }
    angle = (FINEANGLES / 20_i32 * state.p_tick.leveltime) & FINEMASK;
    bob = FixedMul(player.bob / 2 as fixed_t, finesine[angle as usize]);
    if player.playerstate == PlayerState::PST_LIVE {
        player.viewheight += player.deltaviewheight;
        if player.viewheight > VIEWHEIGHT {
            player.viewheight = VIEWHEIGHT as fixed_t;
            player.deltaviewheight = 0_i32 as fixed_t;
        }
        if player.viewheight < VIEWHEIGHT / 2_i32 {
            player.viewheight = (VIEWHEIGHT / 2_i32) as fixed_t;
            if player.deltaviewheight <= 0_i32 {
                player.deltaviewheight = 1_i32 as fixed_t;
            }
        }
        if player.deltaviewheight != 0 {
            player.deltaviewheight += FRACUNIT / 4_i32;
            if player.deltaviewheight == 0 {
                player.deltaviewheight = 1_i32 as fixed_t;
            }
        }
    }
    player.viewz = state.p_mobj.mo(player_mo).z + player.viewheight + bob;
    if player.viewz > state.p_mobj.mo(player_mo).ceilingz - 4_i32 * FRACUNIT {
        player.viewz = (state.p_mobj.mo(player_mo).ceilingz - 4_i32 * FRACUNIT) as fixed_t;
    }
}
pub fn P_MovePlayer(state: &mut GameState, player_id: PlayerId) {
    let cmd = state.g_game.players[player_id.0 as usize].cmd;
    let player_mo = state.g_game.players[player_id.0 as usize].mo.unwrap();
    {
        let mo = state.p_mobj.mo_mut(player_mo);
        mo.angle = mo
            .angle
            .wrapping_add(((cmd.angleturn as i32) << 16_i32) as angle_t);
    }
    let (z, floorz, angle) = {
        let mo = state.p_mobj.mo(player_mo);
        (mo.z, mo.floorz, mo.angle)
    };
    state.p_user.onground = z <= floorz;
    if cmd.forwardmove as i32 != 0 && state.p_user.onground {
        P_Thrust(
            state,
            player_mo,
            angle,
            cmd.forwardmove as fixed_t * 2048 as fixed_t,
        );
    }
    if cmd.sidemove as i32 != 0 && state.p_user.onground {
        P_Thrust(
            state,
            player_mo,
            angle.wrapping_sub(ANG90 as angle_t),
            cmd.sidemove as fixed_t * 2048 as fixed_t,
        );
    }
    if (cmd.forwardmove as i32 != 0 || cmd.sidemove as i32 != 0)
        && state.p_mobj.mo(player_mo).state == Some(StateId(StateNum::S_PLAY as u32))
    {
        P_SetMobjState(state, player_mo, StateNum::S_PLAY_RUN1);
    }
}
pub const ANG5: i32 = ANG90 / 18_i32;
pub fn P_DeathThink(state: &mut GameState, player_id: PlayerId) {
    let player = player_id;
    let mut angle: angle_t = 0;
    let mut delta: angle_t = 0;
    P_MovePsprites(state, player_id);
    if state.g_game.players[player.0 as usize].viewheight > 6_i32 * FRACUNIT {
        state.g_game.players[player.0 as usize].viewheight -= FRACUNIT;
    }
    if state.g_game.players[player.0 as usize].viewheight < 6_i32 * FRACUNIT {
        state.g_game.players[player.0 as usize].viewheight = (6_i32 * FRACUNIT) as fixed_t;
    }
    state.g_game.players[player.0 as usize].deltaviewheight = 0_i32 as fixed_t;
    let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
    state.p_user.onground = state.p_mobj.mo(player_mo).z <= state.p_mobj.mo(player_mo).floorz;
    P_CalcHeight(state, player);
    if state.g_game.players[player.0 as usize].attacker.is_some()
        && state.g_game.players[player.0 as usize].attacker
            != state.g_game.players[player.0 as usize].mo
    {
        let attacker = state.g_game.players[player.0 as usize].attacker.unwrap();
        angle = R_PointToAngle2(
            state,
            state.p_mobj.mo(player_mo).x,
            state.p_mobj.mo(player_mo).y,
            state.p_mobj.mo(attacker).x,
            state.p_mobj.mo(attacker).y,
        );
        delta = angle.wrapping_sub(state.p_mobj.mo(player_mo).angle);
        if delta < ANG5 as angle_t || delta > -ANG5 as u32 {
            state.p_mobj.mo_mut(player_mo).angle = angle;
            if state.g_game.players[player.0 as usize].damagecount != 0 {
                state.g_game.players[player.0 as usize].damagecount -= 1;
            }
        } else if delta < ANG180 {
            state.p_mobj.mo_mut(player_mo).angle = state
                .p_mobj
                .mo(player_mo)
                .angle
                .wrapping_add(ANG5 as angle_t);
        } else {
            state.p_mobj.mo_mut(player_mo).angle = state
                .p_mobj
                .mo(player_mo)
                .angle
                .wrapping_sub(ANG5 as angle_t);
        }
    } else if state.g_game.players[player.0 as usize].damagecount != 0 {
        state.g_game.players[player.0 as usize].damagecount -= 1;
    }
    if state.g_game.players[player.0 as usize].cmd.buttons as i32 & BT_USE as i32 != 0 {
        state.g_game.players[player.0 as usize].playerstate = PlayerState::PST_REBORN;
    }
}
pub fn P_PlayerThink(state: &mut GameState, player_id: PlayerId) {
    let player = player_id;
    let mut newweapon: weapontype_t = weapontype_t::wp_fist;
    let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
    if state.g_game.players[player.0 as usize].cheats & CF_NOCLIP != 0 {
        state.p_mobj.mo_mut(player_mo).flags |= MF_NOCLIP as i32;
    } else {
        state.p_mobj.mo_mut(player_mo).flags &= !(MF_NOCLIP as i32);
    }
    if state.p_mobj.mo(player_mo).flags & MF_JUSTATTACKED as i32 != 0 {
        state.g_game.players[player_id.0 as usize].cmd.angleturn = 0_i16;
        state.g_game.players[player_id.0 as usize].cmd.forwardmove = (0xc800_i32 / 512_i32) as i8;
        state.g_game.players[player_id.0 as usize].cmd.sidemove = 0_i8;
        state.p_mobj.mo_mut(player_mo).flags &= !(MF_JUSTATTACKED as i32);
    }
    if state.g_game.players[player.0 as usize].playerstate == PlayerState::PST_DEAD {
        P_DeathThink(state, player_id);
        return;
    }
    if state.p_mobj.mo(player_mo).reactiontime != 0 {
        state.p_mobj.mo_mut(player_mo).reactiontime -= 1;
    } else {
        P_MovePlayer(state, player_id);
    }
    P_CalcHeight(state, player_id);
    if state
        .p_setup
        .sector_mut(
            state.p_setup.subsectors[state.p_mobj.mo(player_mo).subsector.0 as usize].sector,
        )
        .special
        != 0
    {
        P_PlayerInSpecialSector(state, player_id);
    }
    if state.g_game.players[player_id.0 as usize].cmd.buttons as i32 & BT_SPECIAL as i32 != 0 {
        state.g_game.players[player_id.0 as usize].cmd.buttons = 0 as byte;
    }
    if state.g_game.players[player_id.0 as usize].cmd.buttons as i32 & BT_CHANGE as i32 != 0 {
        newweapon = weapontype_from_raw(
            (state.g_game.players[player_id.0 as usize].cmd.buttons as i32 & BT_WEAPONMASK as i32)
                >> BT_WEAPONSHIFT as i32,
        );
        if newweapon as u32 == weapontype_t::wp_fist as i32 as u32
            && state.g_game.players[player.0 as usize].weaponowned
                [weapontype_t::wp_chainsaw as usize]
            && !(state.g_game.players[player.0 as usize].readyweapon as u32
                == weapontype_t::wp_chainsaw as i32 as u32
                && state.g_game.players[player.0 as usize].powers[PowerType::pw_strength as usize]
                    != 0)
        {
            newweapon = weapontype_t::wp_chainsaw;
        }
        if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32
            && newweapon as u32 == weapontype_t::wp_shotgun as i32 as u32
            && state.g_game.players[player.0 as usize].weaponowned
                [weapontype_t::wp_supershotgun as usize]
            && state.g_game.players[player.0 as usize].readyweapon as u32
                != weapontype_t::wp_supershotgun as i32 as u32
        {
            newweapon = weapontype_t::wp_supershotgun;
        }
        if state.g_game.players[player.0 as usize].weaponowned[newweapon as usize]
            && newweapon as u32 != state.g_game.players[player.0 as usize].readyweapon as u32
            && (newweapon as u32 != weapontype_t::wp_plasma as i32 as u32
                && newweapon as u32 != weapontype_t::wp_bfg as i32 as u32
                || state.doomstat.gamemode as u32 != GameMode_t::shareware as i32 as u32)
        {
            state.g_game.players[player.0 as usize].pendingweapon = newweapon;
        }
    }
    if state.g_game.players[player_id.0 as usize].cmd.buttons as i32 & BT_USE as i32 != 0 {
        if state.g_game.players[player.0 as usize].usedown == 0 {
            P_UseLines(state, player_id);
            state.g_game.players[player.0 as usize].usedown = true_0;
        }
    } else {
        state.g_game.players[player.0 as usize].usedown = false_0;
    }
    P_MovePsprites(state, player_id);
    if state.g_game.players[player.0 as usize].powers[PowerType::pw_strength as usize] != 0 {
        state.g_game.players[player.0 as usize].powers[PowerType::pw_strength as usize] += 1;
    }
    if state.g_game.players[player.0 as usize].powers[PowerType::pw_invulnerability as usize] != 0 {
        state.g_game.players[player.0 as usize].powers[PowerType::pw_invulnerability as usize] -= 1;
    }
    if state.g_game.players[player.0 as usize].powers[PowerType::pw_invisibility as usize] != 0 {
        state.g_game.players[player.0 as usize].powers[PowerType::pw_invisibility as usize] -= 1;
        if state.g_game.players[player.0 as usize].powers[PowerType::pw_invisibility as usize] == 0
        {
            state.p_mobj.mo_mut(player_mo).flags &= !(MF_SHADOW as i32);
        }
    }
    if state.g_game.players[player.0 as usize].powers[PowerType::pw_infrared as usize] != 0 {
        state.g_game.players[player.0 as usize].powers[PowerType::pw_infrared as usize] -= 1;
    }
    if state.g_game.players[player.0 as usize].powers[PowerType::pw_ironfeet as usize] != 0 {
        state.g_game.players[player.0 as usize].powers[PowerType::pw_ironfeet as usize] -= 1;
    }
    if state.g_game.players[player.0 as usize].damagecount != 0 {
        state.g_game.players[player.0 as usize].damagecount -= 1;
    }
    if state.g_game.players[player.0 as usize].bonuscount != 0 {
        state.g_game.players[player.0 as usize].bonuscount -= 1;
    }
    if state.g_game.players[player.0 as usize].powers[PowerType::pw_invulnerability as usize] != 0 {
        if state.g_game.players[player.0 as usize].powers[PowerType::pw_invulnerability as usize]
            > 4_i32 * 32_i32
            || state.g_game.players[player.0 as usize].powers
                [PowerType::pw_invulnerability as usize]
                & 8_i32
                != 0
        {
            state.g_game.players[player.0 as usize].fixedcolormap = INVERSECOLORMAP;
        } else {
            state.g_game.players[player.0 as usize].fixedcolormap = 0_i32;
        }
    } else if state.g_game.players[player.0 as usize].powers[PowerType::pw_infrared as usize] != 0 {
        if state.g_game.players[player.0 as usize].powers[PowerType::pw_infrared as usize]
            > 4_i32 * 32_i32
            || state.g_game.players[player.0 as usize].powers[PowerType::pw_infrared as usize]
                & 8_i32
                != 0
        {
            state.g_game.players[player.0 as usize].fixedcolormap = 1_i32;
        } else {
            state.g_game.players[player.0 as usize].fixedcolormap = 0_i32;
        }
    } else {
        state.g_game.players[player.0 as usize].fixedcolormap = 0_i32;
    };
}
