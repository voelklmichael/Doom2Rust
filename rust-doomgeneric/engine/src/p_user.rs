use crate::src::d_mode::GameMode_t;
use crate::src::d_player::{player_t, PlayerState};
use crate::src::d_player::{
    pw_infrared, pw_invisibility, pw_invulnerability, pw_ironfeet, pw_strength,
};
use crate::src::d_player::{
    weapontype_t, wp_bfg, wp_chainsaw, wp_fist, wp_plasma, wp_shotgun, wp_supershotgun,
};
use crate::src::d_player::{CF_NOCLIP, CF_NOMOMENTUM};
use crate::src::d_ticcmd::ticcmd_t;
use crate::src::d_ticcmd::{BT_CHANGE, BT_SPECIAL, BT_USE, BT_WEAPONMASK, BT_WEAPONSHIFT};
use crate::src::doomdef::false_0;
use crate::src::doomdef::true_0;
use crate::src::game_state::GameState;
use crate::src::info::{StateId, S_PLAY, S_PLAY_RUN1};
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FixedMul;
use crate::src::m_fixed::FRACUNIT;
use crate::src::p_map::P_UseLines;
use crate::src::p_mobj::P_SetMobjState;
use crate::src::p_mobj::{MF_JUSTATTACKED, MF_NOCLIP, MF_SHADOW};
use crate::src::p_pspr::P_MovePsprites;
use crate::src::p_spec::P_PlayerInSpecialSector;
use crate::src::r_main::R_PointToAngle2;
use crate::src::stdint_types::byte;
use crate::src::tables::angle_t;
use crate::src::tables::finecosine;
use crate::src::tables::finesine;
use crate::src::tables::ANG180;
use crate::src::tables::ANG90;
use crate::src::tables::ANGLETOFINESHIFT;
use crate::src::tables::FINEANGLES;
use crate::src::tables::FINEMASK;

pub const VIEWHEIGHT: i32 = 41 * FRACUNIT;
pub const INVERSECOLORMAP: i32 = 32;
pub const MAXBOB: i32 = 0x100000;
pub struct PUserState {
    onground: bool,
}

impl PUserState {
    pub const fn new() -> Self {
        PUserState { onground: false }
    }
}

pub unsafe fn P_Thrust(mut player: *mut player_t, mut angle: angle_t, mut move_0: fixed_t) {
    angle >>= ANGLETOFINESHIFT;
    (*(*player).mo).momx += FixedMul(move_0, finecosine[angle as isize]);
    (*(*player).mo).momy += FixedMul(move_0, finesine[angle as usize]);
}
pub unsafe fn P_CalcHeight(state: &mut GameState, mut player: *mut player_t) {
    let mut angle: i32 = 0;
    let mut bob: fixed_t = 0;
    (*player).bob = FixedMul((*(*player).mo).momx, (*(*player).mo).momx)
        + FixedMul((*(*player).mo).momy, (*(*player).mo).momy);
    (*player).bob >>= 2 as i32;
    if (*player).bob > MAXBOB {
        (*player).bob = MAXBOB as fixed_t;
    }
    if (*player).cheats & CF_NOMOMENTUM as i32 != 0 || !state.p_user.onground {
        (*player).viewz = ((*(*player).mo).z as i32 + VIEWHEIGHT) as fixed_t;
        if (*player).viewz > (*(*player).mo).ceilingz as i32 - 4 as i32 * FRACUNIT {
            (*player).viewz = ((*(*player).mo).ceilingz as i32 - 4 as i32 * FRACUNIT) as fixed_t;
        }
        (*player).viewz = (*(*player).mo).z + (*player).viewheight;
        return;
    }
    angle = FINEANGLES / 20 as i32 * state.p_tick.leveltime & FINEMASK;
    bob = FixedMul((*player).bob / 2 as fixed_t, finesine[angle as usize]);
    if (*player).playerstate == PlayerState::PST_LIVE {
        (*player).viewheight += (*player).deltaviewheight;
        if (*player).viewheight > VIEWHEIGHT {
            (*player).viewheight = VIEWHEIGHT as fixed_t;
            (*player).deltaviewheight = 0 as i32 as fixed_t;
        }
        if (*player).viewheight < VIEWHEIGHT / 2 as i32 {
            (*player).viewheight = (VIEWHEIGHT / 2 as i32) as fixed_t;
            if (*player).deltaviewheight <= 0 as i32 {
                (*player).deltaviewheight = 1 as i32 as fixed_t;
            }
        }
        if (*player).deltaviewheight != 0 {
            (*player).deltaviewheight += FRACUNIT / 4 as i32;
            if (*player).deltaviewheight == 0 {
                (*player).deltaviewheight = 1 as i32 as fixed_t;
            }
        }
    }
    (*player).viewz = (*(*player).mo).z + (*player).viewheight + bob;
    if (*player).viewz > (*(*player).mo).ceilingz as i32 - 4 as i32 * FRACUNIT {
        (*player).viewz = ((*(*player).mo).ceilingz as i32 - 4 as i32 * FRACUNIT) as fixed_t;
    }
}
pub unsafe fn P_MovePlayer(state: &mut GameState, mut player: *mut player_t) {
    let mut cmd: *mut ticcmd_t = ::core::ptr::null_mut::<ticcmd_t>();
    cmd = &raw mut (*player).cmd;
    (*(*player).mo).angle = (*(*player).mo)
        .angle
        .wrapping_add((((*cmd).angleturn as i32) << 16 as i32) as angle_t);
    state.p_user.onground = (*(*player).mo).z <= (*(*player).mo).floorz;
    if (*cmd).forwardmove as i32 != 0 && state.p_user.onground {
        P_Thrust(
            player,
            (*(*player).mo).angle,
            (*cmd).forwardmove as fixed_t * 2048 as fixed_t,
        );
    }
    if (*cmd).sidemove as i32 != 0 && state.p_user.onground {
        P_Thrust(
            player,
            (*(*player).mo).angle.wrapping_sub(ANG90 as angle_t),
            (*cmd).sidemove as fixed_t * 2048 as fixed_t,
        );
    }
    if ((*cmd).forwardmove as i32 != 0 || (*cmd).sidemove as i32 != 0)
        && (*(*player).mo).state == Some(StateId(S_PLAY))
    {
        P_SetMobjState(state, (*player).mo, S_PLAY_RUN1);
    }
}
pub const ANG5: i32 = ANG90 / 18 as i32;
pub unsafe fn P_DeathThink(state: &mut GameState, mut player: *mut player_t) {
    let mut angle: angle_t = 0;
    let mut delta: angle_t = 0;
    P_MovePsprites(state, player);
    if (*player).viewheight > 6 as i32 * FRACUNIT {
        (*player).viewheight -= FRACUNIT;
    }
    if (*player).viewheight < 6 as i32 * FRACUNIT {
        (*player).viewheight = (6 as i32 * FRACUNIT) as fixed_t;
    }
    (*player).deltaviewheight = 0 as i32 as fixed_t;
    state.p_user.onground = (*(*player).mo).z <= (*(*player).mo).floorz;
    P_CalcHeight(state, player);
    let attacker = (*player).attacker.and_then(|id| state.p_mobj.mobj_get(id));
    if attacker.is_some() && attacker != Some((*player).mo) {
        let attacker = attacker.unwrap();
        angle = R_PointToAngle2(
            state,
            (*(*player).mo).x,
            (*(*player).mo).y,
            (*attacker).x,
            (*attacker).y,
        );
        delta = angle.wrapping_sub((*(*player).mo).angle);
        if delta < ANG5 as angle_t || delta > -ANG5 as u32 {
            (*(*player).mo).angle = angle;
            if (*player).damagecount != 0 {
                (*player).damagecount -= 1;
            }
        } else if delta < ANG180 {
            (*(*player).mo).angle = (*(*player).mo).angle.wrapping_add(ANG5 as angle_t);
        } else {
            (*(*player).mo).angle = (*(*player).mo).angle.wrapping_sub(ANG5 as angle_t);
        }
    } else if (*player).damagecount != 0 {
        (*player).damagecount -= 1;
    }
    if (*player).cmd.buttons as i32 & BT_USE as i32 != 0 {
        (*player).playerstate = PlayerState::PST_REBORN;
    }
}
pub unsafe fn P_PlayerThink(state: &mut GameState, mut player: *mut player_t) {
    let mut cmd: *mut ticcmd_t = ::core::ptr::null_mut::<ticcmd_t>();
    let mut newweapon: weapontype_t = wp_fist;
    if (*player).cheats & CF_NOCLIP as i32 != 0 {
        (*(*player).mo).flags |= MF_NOCLIP as i32;
    } else {
        (*(*player).mo).flags &= !(MF_NOCLIP as i32);
    }
    cmd = &raw mut (*player).cmd;
    if (*(*player).mo).flags & MF_JUSTATTACKED as i32 != 0 {
        (*cmd).angleturn = 0 as i16;
        (*cmd).forwardmove = (0xc800 as i32 / 512 as i32) as i8;
        (*cmd).sidemove = 0 as i8;
        (*(*player).mo).flags &= !(MF_JUSTATTACKED as i32);
    }
    if (*player).playerstate == PlayerState::PST_DEAD {
        P_DeathThink(state, player);
        return;
    }
    if (*(*player).mo).reactiontime != 0 {
        (*(*player).mo).reactiontime -= 1;
    } else {
        P_MovePlayer(state, player);
    }
    P_CalcHeight(state, player);
    if (*state.p_setup.sector_mut(
        state.p_setup.subsectors[(*(*player).mo).subsector.0 as usize].sector,
    ))
    .special
        != 0
    {
        P_PlayerInSpecialSector(state, player);
    }
    if (*cmd).buttons as i32 & BT_SPECIAL as i32 != 0 {
        (*cmd).buttons = 0 as byte;
    }
    if (*cmd).buttons as i32 & BT_CHANGE as i32 != 0 {
        newweapon = (((*cmd).buttons as i32 & BT_WEAPONMASK as i32) >> BT_WEAPONSHIFT as i32)
            as weapontype_t;
        if newweapon as u32 == wp_fist as i32 as u32
            && (*player).weaponowned[wp_chainsaw as i32 as usize]
            && !((*player).readyweapon as u32 == wp_chainsaw as i32 as u32
                && (*player).powers[pw_strength as i32 as usize] != 0)
        {
            newweapon = wp_chainsaw;
        }
        if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32
            && newweapon as u32 == wp_shotgun as i32 as u32
            && (*player).weaponowned[wp_supershotgun as i32 as usize]
            && (*player).readyweapon as u32 != wp_supershotgun as i32 as u32
        {
            newweapon = wp_supershotgun;
        }
        if (*player).weaponowned[newweapon as usize]
            && newweapon as u32 != (*player).readyweapon as u32
        {
            if newweapon as u32 != wp_plasma as i32 as u32
                && newweapon as u32 != wp_bfg as i32 as u32
                || state.doomstat.gamemode as u32 != GameMode_t::shareware as i32 as u32
            {
                (*player).pendingweapon = newweapon;
            }
        }
    }
    if (*cmd).buttons as i32 & BT_USE as i32 != 0 {
        if (*player).usedown == 0 {
            P_UseLines(state, player);
            (*player).usedown = true_0;
        }
    } else {
        (*player).usedown = false_0;
    }
    P_MovePsprites(state, player);
    if (*player).powers[pw_strength as i32 as usize] != 0 {
        (*player).powers[pw_strength as i32 as usize] += 1;
    }
    if (*player).powers[pw_invulnerability as i32 as usize] != 0 {
        (*player).powers[pw_invulnerability as i32 as usize] -= 1;
    }
    if (*player).powers[pw_invisibility as i32 as usize] != 0 {
        (*player).powers[pw_invisibility as i32 as usize] -= 1;
        if (*player).powers[pw_invisibility as i32 as usize] == 0 {
            (*(*player).mo).flags &= !(MF_SHADOW as i32);
        }
    }
    if (*player).powers[pw_infrared as i32 as usize] != 0 {
        (*player).powers[pw_infrared as i32 as usize] -= 1;
    }
    if (*player).powers[pw_ironfeet as i32 as usize] != 0 {
        (*player).powers[pw_ironfeet as i32 as usize] -= 1;
    }
    if (*player).damagecount != 0 {
        (*player).damagecount -= 1;
    }
    if (*player).bonuscount != 0 {
        (*player).bonuscount -= 1;
    }
    if (*player).powers[pw_invulnerability as i32 as usize] != 0 {
        if (*player).powers[pw_invulnerability as i32 as usize] > 4 as i32 * 32 as i32
            || (*player).powers[pw_invulnerability as i32 as usize] & 8 as i32 != 0
        {
            (*player).fixedcolormap = INVERSECOLORMAP;
        } else {
            (*player).fixedcolormap = 0 as i32;
        }
    } else if (*player).powers[pw_infrared as i32 as usize] != 0 {
        if (*player).powers[pw_infrared as i32 as usize] > 4 as i32 * 32 as i32
            || (*player).powers[pw_infrared as i32 as usize] & 8 as i32 != 0
        {
            (*player).fixedcolormap = 1 as i32;
        } else {
            (*player).fixedcolormap = 0 as i32;
        }
    } else {
        (*player).fixedcolormap = 0 as i32;
    };
}
