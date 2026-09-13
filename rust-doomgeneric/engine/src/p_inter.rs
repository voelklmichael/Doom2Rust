use crate::src::am_map::AM_Stop;
use crate::src::d_items::weaponinfo;
use crate::src::d_mode::{GameMode_t, GameVersion};
use crate::src::d_mode::SkillType;
use crate::src::d_player::CF_GODMODE;
use crate::src::d_player::{ammotype_from_raw, ammotype_t, NUMAMMO};
use crate::src::d_player::{player_t, PlayerId, PlayerState};
use crate::src::d_player::PowerType;
use crate::src::d_player::weapontype_t;
use crate::src::game_state::GameState;
use crate::src::i_system::I_Error;
use crate::src::i_system::I_Tactile;
use crate::src::p_mobj::StateNum;
use crate::src::info::StateId;
use crate::src::m_fixed::fixed_t;
use crate::src::m_fixed::FixedMul;
use crate::src::m_fixed::FRACUNIT;
use crate::src::m_random::P_Random;
use crate::src::p_mobj::mobj_t;
use crate::src::p_mobj::P_RemoveMobj;
use crate::src::p_mobj::P_SetMobjState;
use crate::src::p_mobj::P_SpawnMobj;
use crate::src::p_mobj::ONFLOORZ;
use crate::src::p_mobj::MobjType;
use crate::src::p_mobj::{
    MF_CORPSE, MF_COUNTITEM, MF_COUNTKILL, MF_DROPOFF, MF_DROPPED, MF_FLOAT, MF_JUSTHIT, MF_NOCLIP,
    MF_NOGRAVITY, MF_SHADOW, MF_SHOOTABLE, MF_SKULLFLY, MF_SOLID,
};
use crate::src::p_pspr::P_DropWeapon;
use crate::src::r_main::R_PointToAngle2;
use crate::src::s_sound::S_StartSound;
use crate::src::s_sound::SoundOrigin;
use crate::src::sounds::{sfx_getpow, sfx_itemup, sfx_wpnup};
use crate::src::tables::finecosine;
use crate::src::tables::finesine;
use crate::src::tables::ANG180;
use crate::src::tables::ANGLETOFINESHIFT;

pub const NUMCARDS: i32 = 6;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CardType {
    it_bluecard = 0,
    it_yellowcard = 1,
    it_redcard = 2,
    it_blueskull = 3,
    it_yellowskull = 4,
    it_redskull = 5,
}
pub type C2RustUnnamed_0 = u32;
pub const IRONTICS: C2RustUnnamed_0 = 2100;
pub const INFRATICS: C2RustUnnamed_0 = 4200;
pub const INVISTICS: C2RustUnnamed_0 = 2100;
pub const INVULNTICS: C2RustUnnamed_0 = 1050;
pub const DEH_DEFAULT_MAX_HEALTH: i32 = 200;
pub const DEH_DEFAULT_MAX_ARMOR: i32 = 200;
pub const DEH_DEFAULT_GREEN_ARMOR_CLASS: i32 = 1;
pub const DEH_DEFAULT_BLUE_ARMOR_CLASS: i32 = 2;
pub const DEH_DEFAULT_MAX_SOULSPHERE: i32 = 200;
pub const DEH_DEFAULT_SOULSPHERE_HEALTH: i32 = 100;
pub const DEH_DEFAULT_MEGASPHERE_HEALTH: i32 = 200;
pub const deh_max_health: i32 = DEH_DEFAULT_MAX_HEALTH;
pub const deh_max_armor: i32 = DEH_DEFAULT_MAX_ARMOR;
pub const deh_green_armor_class: i32 = DEH_DEFAULT_GREEN_ARMOR_CLASS;
pub const deh_blue_armor_class: i32 = DEH_DEFAULT_BLUE_ARMOR_CLASS;
pub const deh_max_soulsphere: i32 = DEH_DEFAULT_MAX_SOULSPHERE;
pub const deh_soulsphere_health: i32 = DEH_DEFAULT_SOULSPHERE_HEALTH;
pub const deh_megasphere_health: i32 = DEH_DEFAULT_MEGASPHERE_HEALTH;
pub const MAXHEALTH: i32 = 100;
pub const BASETHRESHOLD: i32 = 100;
pub const BONUSADD: i32 = 6;
pub static maxammo: [i32; 4] = [200 as i32, 50 as i32, 300 as i32, 50 as i32];
#[no_mangle]
pub static clipammo: [i32; 4] = [10 as i32, 4 as i32, 20 as i32, 1 as i32];
pub unsafe fn P_GiveAmmo(
    state: &mut GameState,
    mut player: *mut player_t,
    mut ammo: ammotype_t,
    mut num: i32,
) -> bool {
    let mut oldammo: i32 = 0;
    if ammo as u32 == ammotype_t::am_noammo as i32 as u32 {
        return false;
    }
    if ammo as u32 > NUMAMMO as i32 as u32 {
        I_Error(&format!("P_GiveAmmo: bad type {}", ammo as u32));
    }
    if (*player).ammo[ammo as usize] == (*player).maxammo[ammo as usize] {
        return false;
    }
    if num != 0 {
        num *= clipammo[ammo as usize];
    } else {
        num = clipammo[ammo as usize] / 2 as i32;
    }
    if state.g_game.gameskill == SkillType::sk_baby
        || state.g_game.gameskill == SkillType::sk_nightmare
    {
        num <<= 1 as i32;
    }
    oldammo = (*player).ammo[ammo as usize];
    (*player).ammo[ammo as usize] += num;
    if (*player).ammo[ammo as usize] > (*player).maxammo[ammo as usize] {
        (*player).ammo[ammo as usize] = (*player).maxammo[ammo as usize];
    }
    if oldammo != 0 {
        return true;
    }
    match ammo as u32 {
        0 => {
            if (*player).readyweapon as u32 == weapontype_t::wp_fist as i32 as u32 {
                if (*player).weaponowned[weapontype_t::wp_chaingun as usize] {
                    (*player).pendingweapon = weapontype_t::wp_chaingun;
                } else {
                    (*player).pendingweapon = weapontype_t::wp_pistol;
                }
            }
        }
        1 => {
            if (*player).readyweapon as u32 == weapontype_t::wp_fist as i32 as u32
                || (*player).readyweapon as u32 == weapontype_t::wp_pistol as i32 as u32
            {
                if (*player).weaponowned[weapontype_t::wp_shotgun as usize] {
                    (*player).pendingweapon = weapontype_t::wp_shotgun;
                }
            }
        }
        2 => {
            if (*player).readyweapon as u32 == weapontype_t::wp_fist as i32 as u32
                || (*player).readyweapon as u32 == weapontype_t::wp_pistol as i32 as u32
            {
                if (*player).weaponowned[weapontype_t::wp_plasma as usize] {
                    (*player).pendingweapon = weapontype_t::wp_plasma;
                }
            }
        }
        3 => {
            if (*player).readyweapon as u32 == weapontype_t::wp_fist as i32 as u32 {
                if (*player).weaponowned[weapontype_t::wp_missile as usize] {
                    (*player).pendingweapon = weapontype_t::wp_missile;
                }
            }
        }
        _ => {}
    }
    return true;
}
pub unsafe fn P_GiveWeapon(
    state: &mut GameState,
    mut player: *mut player_t,
    mut weapon: weapontype_t,
    mut dropped: bool,
) -> bool {
    let mut gaveammo: bool = false;
    let mut gaveweapon: bool;
    if state.g_game.netgame && state.g_game.deathmatch != 2 as i32 && !dropped {
        if (*player).weaponowned[weapon as usize] {
            return false;
        }
        (*player).bonuscount += BONUSADD;
        (*player).weaponowned[weapon as usize] = true;
        if state.g_game.deathmatch != 0 {
            P_GiveAmmo(state, player, weaponinfo[weapon as usize].ammo, 5 as i32);
        } else {
            P_GiveAmmo(state, player, weaponinfo[weapon as usize].ammo, 2 as i32);
        }
        (*player).pendingweapon = weapon;
        if player
            == (&raw mut state.g_game.players as *mut player_t)
                .offset(state.g_game.consoleplayer as isize) as *mut player_t
        {
            S_StartSound(state, SoundOrigin::None, sfx_wpnup as i32);
        }
        return false;
    }
    if weaponinfo[weapon as usize].ammo as u32 != ammotype_t::am_noammo as i32 as u32 {
        if dropped {
            gaveammo = P_GiveAmmo(state, player, weaponinfo[weapon as usize].ammo, 1 as i32);
        } else {
            gaveammo = P_GiveAmmo(state, player, weaponinfo[weapon as usize].ammo, 2 as i32);
        }
    } else {
        gaveammo = false;
    }
    if (*player).weaponowned[weapon as usize] {
        gaveweapon = false;
    } else {
        gaveweapon = true;
        (*player).weaponowned[weapon as usize] = true;
        (*player).pendingweapon = weapon;
    }
    return gaveweapon || gaveammo;
}
pub unsafe fn P_GiveBody(state: &mut GameState, mut player: *mut player_t, mut num: i32) -> bool {
    if (*player).health >= MAXHEALTH {
        return false;
    }
    (*player).health += num;
    if (*player).health > MAXHEALTH {
        (*player).health = MAXHEALTH;
    }
    let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
    (*player_mo).health = (*player).health;
    return true;
}
pub unsafe fn P_GiveArmor(mut player: *mut player_t, mut armortype: i32) -> bool {
    let mut hits: i32 = 0;
    hits = armortype * 100 as i32;
    if (*player).armorpoints >= hits {
        return false;
    }
    (*player).armortype = armortype;
    (*player).armorpoints = hits;
    return true;
}
pub unsafe fn P_GiveCard(mut player: *mut player_t, mut card: CardType) {
    if (*player).cards[card as usize] {
        return;
    }
    (*player).bonuscount = BONUSADD;
    (*player).cards[card as usize] = true;
}
pub unsafe fn P_GivePower(state: &mut GameState, mut player: *mut player_t, mut power: i32) -> bool {
    if power == PowerType::pw_invulnerability as i32 {
        (*player).powers[power as usize] = INVULNTICS as i32;
        return true;
    }
    if power == PowerType::pw_invisibility as i32 {
        (*player).powers[power as usize] = INVISTICS as i32;
        let player_mo = state.p_mobj.mobj_get((*player).mo.unwrap()).unwrap();
        (*player_mo).flags |= MF_SHADOW as i32;
        return true;
    }
    if power == PowerType::pw_infrared as i32 {
        (*player).powers[power as usize] = INFRATICS as i32;
        return true;
    }
    if power == PowerType::pw_ironfeet as i32 {
        (*player).powers[power as usize] = IRONTICS as i32;
        return true;
    }
    if power == PowerType::pw_strength as i32 {
        P_GiveBody(state, player, 100 as i32);
        (*player).powers[power as usize] = 1 as i32;
        return true;
    }
    if (*player).powers[power as usize] != 0 {
        return false;
    }
    (*player).powers[power as usize] = 1 as i32;
    return true;
}
pub unsafe fn P_TouchSpecialThing(
    state: &mut GameState,
    mut special: *mut mobj_t,
    mut toucher: *mut mobj_t,
) {
    let mut player: *mut player_t = ::core::ptr::null_mut::<player_t>();
    let mut i: i32 = 0;
    let mut delta: fixed_t = 0;
    let mut sound: i32 = 0;
    delta = (*special).z - (*toucher).z;
    if delta > (*toucher).height || delta < -(8 as i32) * FRACUNIT {
        return;
    }
    sound = sfx_itemup as i32;
    player = state.g_game.player_mut((*toucher).player.unwrap());
    if (*toucher).health <= 0 as i32 {
        return;
    }
    match (*special).sprite as u32 {
        55 => {
            if !P_GiveArmor(player, deh_green_armor_class) {
                return;
            }
            (*player).message = Some("Picked up the armor.".to_string());
        }
        56 => {
            if !P_GiveArmor(player, deh_blue_armor_class) {
                return;
            }
            (*player).message = Some("Picked up the MegaArmor!".to_string());
        }
        60 => {
            (*player).health += 1;
            if (*player).health > deh_max_health {
                (*player).health = deh_max_health;
            }
            (*toucher).health = (*player).health;
            (*player).message = Some("Picked up a health bonus.".to_string());
        }
        61 => {
            (*player).armorpoints += 1;
            if (*player).armorpoints > deh_max_armor {
                (*player).armorpoints = deh_max_armor;
            }
            if (*player).armortype == 0 {
                (*player).armortype = 1 as i32;
            }
            (*player).message = Some("Picked up an armor bonus.".to_string());
        }
        70 => {
            (*player).health += deh_soulsphere_health;
            if (*player).health > deh_max_soulsphere {
                (*player).health = deh_max_soulsphere;
            }
            (*toucher).health = (*player).health;
            (*player).message = Some("Supercharge!".to_string());
            sound = sfx_getpow as i32;
        }
        74 => {
            if state.doomstat.gamemode as u32 != GameMode_t::commercial as i32 as u32 {
                return;
            }
            (*player).health = deh_megasphere_health;
            (*toucher).health = (*player).health;
            P_GiveArmor(player, 2 as i32);
            (*player).message = Some("MegaSphere!".to_string());
            sound = sfx_getpow as i32;
        }
        62 => {
            if !(*player).cards[CardType::it_bluecard as usize] {
                (*player).message = Some("Picked up a blue keycard.".to_string());
            }
            P_GiveCard(player, CardType::it_bluecard);
            if state.g_game.netgame {
                return;
            }
        }
        64 => {
            if !(*player).cards[CardType::it_yellowcard as usize] {
                (*player).message = Some("Picked up a yellow keycard.".to_string());
            }
            P_GiveCard(player, CardType::it_yellowcard);
            if state.g_game.netgame {
                return;
            }
        }
        63 => {
            if !(*player).cards[CardType::it_redcard as usize] {
                (*player).message = Some("Picked up a red keycard.".to_string());
            }
            P_GiveCard(player, CardType::it_redcard);
            if state.g_game.netgame {
                return;
            }
        }
        65 => {
            if !(*player).cards[CardType::it_blueskull as usize] {
                (*player).message = Some("Picked up a blue skull key.".to_string());
            }
            P_GiveCard(player, CardType::it_blueskull);
            if state.g_game.netgame {
                return;
            }
        }
        67 => {
            if !(*player).cards[CardType::it_yellowskull as usize] {
                (*player).message = Some("Picked up a yellow skull key.".to_string());
            }
            P_GiveCard(player, CardType::it_yellowskull);
            if state.g_game.netgame {
                return;
            }
        }
        66 => {
            if !(*player).cards[CardType::it_redskull as usize] {
                (*player).message = Some("Picked up a red skull key.".to_string());
            }
            P_GiveCard(player, CardType::it_redskull);
            if state.g_game.netgame {
                return;
            }
        }
        68 => {
            if !P_GiveBody(state, player, 10 as i32) {
                return;
            }
            (*player).message = Some("Picked up a stimpack.".to_string());
        }
        69 => {
            if !P_GiveBody(state, player, 25 as i32) {
                return;
            }
            if (*player).health < 25 as i32 {
                (*player).message = Some("Picked up a medikit that you REALLY need!".to_string());
            } else {
                (*player).message = Some("Picked up a medikit.".to_string());
            }
        }
        71 => {
            if !P_GivePower(state, player, PowerType::pw_invulnerability as i32) {
                return;
            }
            (*player).message = Some("Invulnerability!".to_string());
            sound = sfx_getpow as i32;
        }
        72 => {
            if !P_GivePower(state, player, PowerType::pw_strength as i32) {
                return;
            }
            (*player).message = Some("Berserk!".to_string());
            if (*player).readyweapon as u32 != weapontype_t::wp_fist as i32 as u32 {
                (*player).pendingweapon = weapontype_t::wp_fist;
            }
            sound = sfx_getpow as i32;
        }
        73 => {
            if !P_GivePower(state, player, PowerType::pw_invisibility as i32) {
                return;
            }
            (*player).message = Some("Partial Invisibility".to_string());
            sound = sfx_getpow as i32;
        }
        75 => {
            if !P_GivePower(state, player, PowerType::pw_ironfeet as i32) {
                return;
            }
            (*player).message = Some("Radiation Shielding Suit".to_string());
            sound = sfx_getpow as i32;
        }
        76 => {
            if !P_GivePower(state, player, PowerType::pw_allmap as i32) {
                return;
            }
            (*player).message = Some("Computer Area Map".to_string());
            sound = sfx_getpow as i32;
        }
        77 => {
            if !P_GivePower(state, player, PowerType::pw_infrared as i32) {
                return;
            }
            (*player).message = Some("Light Amplification Visor".to_string());
            sound = sfx_getpow as i32;
        }
        78 => {
            if (*special).flags & MF_DROPPED as i32 != 0 {
                if !P_GiveAmmo(state, player, ammotype_t::am_clip, 0 as i32) {
                    return;
                }
            } else if !P_GiveAmmo(state, player, ammotype_t::am_clip, 1 as i32) {
                return;
            }
            (*player).message = Some("Picked up a clip.".to_string());
        }
        79 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_clip, 5 as i32) {
                return;
            }
            (*player).message = Some("Picked up a box of bullets.".to_string());
        }
        80 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_misl, 1 as i32) {
                return;
            }
            (*player).message = Some("Picked up a rocket.".to_string());
        }
        81 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_misl, 5 as i32) {
                return;
            }
            (*player).message = Some("Picked up a box of rockets.".to_string());
        }
        82 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_cell, 1 as i32) {
                return;
            }
            (*player).message = Some("Picked up an energy cell.".to_string());
        }
        83 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_cell, 5 as i32) {
                return;
            }
            (*player).message = Some("Picked up an energy cell pack.".to_string());
        }
        84 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_shell, 1 as i32) {
                return;
            }
            (*player).message = Some("Picked up 4 shotgun shells.".to_string());
        }
        85 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_shell, 5 as i32) {
                return;
            }
            (*player).message = Some("Picked up a box of shotgun shells.".to_string());
        }
        86 => {
            if !(*player).backpack {
                i = 0 as i32;
                while i < NUMAMMO as i32 {
                    (*player).maxammo[i as usize] *= 2 as i32;
                    i += 1;
                }
                (*player).backpack = true;
            }
            i = 0 as i32;
            while i < NUMAMMO as i32 {
                P_GiveAmmo(state, player, ammotype_from_raw(i), 1 as i32);
                i += 1;
            }
            (*player).message = Some("Picked up a backpack full of ammo!".to_string());
        }
        87 => {
            if !P_GiveWeapon(state, player, weapontype_t::wp_bfg, false) {
                return;
            }
            (*player).message = Some("You got the BFG9000!  Oh, yes.".to_string());
            sound = sfx_wpnup as i32;
        }
        88 => {
            if !P_GiveWeapon(
                state,
                player,
                weapontype_t::wp_chaingun,
                (*special).flags & MF_DROPPED as i32 != 0,
            ) {
                return;
            }
            (*player).message = Some("You got the chaingun!".to_string());
            sound = sfx_wpnup as i32;
        }
        89 => {
            if !P_GiveWeapon(state, player, weapontype_t::wp_chainsaw, false) {
                return;
            }
            (*player).message = Some("A chainsaw!  Find some meat!".to_string());
            sound = sfx_wpnup as i32;
        }
        90 => {
            if !P_GiveWeapon(state, player, weapontype_t::wp_missile, false) {
                return;
            }
            (*player).message = Some("You got the rocket launcher!".to_string());
            sound = sfx_wpnup as i32;
        }
        91 => {
            if !P_GiveWeapon(state, player, weapontype_t::wp_plasma, false) {
                return;
            }
            (*player).message = Some("You got the plasma gun!".to_string());
            sound = sfx_wpnup as i32;
        }
        92 => {
            if !P_GiveWeapon(
                state,
                player,
                weapontype_t::wp_shotgun,
                (*special).flags & MF_DROPPED as i32 != 0,
            ) {
                return;
            }
            (*player).message = Some("You got the shotgun!".to_string());
            sound = sfx_wpnup as i32;
        }
        93 => {
            if !P_GiveWeapon(
                state,
                player,
                weapontype_t::wp_supershotgun,
                (*special).flags & MF_DROPPED as i32 != 0,
            ) {
                return;
            }
            (*player).message = Some("You got the super shotgun!".to_string());
            sound = sfx_wpnup as i32;
        }
        _ => {
            I_Error("P_SpecialThing: Unknown gettable thing");
        }
    }
    if (*special).flags & MF_COUNTITEM as i32 != 0 {
        (*player).itemcount += 1;
    }
    P_RemoveMobj(state, special);
    (*player).bonuscount += BONUSADD;
    if player
        == (&raw mut state.g_game.players as *mut player_t)
            .offset(state.g_game.consoleplayer as isize) as *mut player_t
    {
        S_StartSound(state, SoundOrigin::None, sound);
    }
}
pub unsafe fn P_KillMobj(state: &mut GameState, mut source: *mut mobj_t, mut target: *mut mobj_t) {
    let mut item: MobjType = MobjType::MT_PLAYER;
    let mut mo: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    (*target).flags &= !(MF_SHOOTABLE as i32 | MF_FLOAT as i32 | MF_SKULLFLY as i32);
    if (*target).type_0 as u32 != MobjType::MT_SKULL as i32 as u32 {
        (*target).flags &= !(MF_NOGRAVITY as i32);
    }
    (*target).flags |= MF_CORPSE as i32 | MF_DROPOFF as i32;
    (*target).height >>= 2 as i32;
    if !source.is_null() && (*source).player.is_some() {
        let source_player = state.g_game.player_mut((*source).player.unwrap());
        if (*target).flags & MF_COUNTKILL as i32 != 0 {
            (*source_player).killcount += 1;
        }
        if let Some(target_player_id) = (*target).player {
            (*source_player).frags[target_player_id.0 as usize] += 1;
        }
    } else if !state.g_game.netgame && (*target).flags & MF_COUNTKILL as i32 != 0 {
        state.g_game.players[0 as usize].killcount += 1;
    }
    if let Some(target_player_id) = (*target).player {
        let target_player = state.g_game.player_mut(target_player_id);
        if source.is_null() {
            (*target_player).frags[target_player_id.0 as usize] += 1;
        }
        (*target).flags &= !(MF_SOLID as i32);
        (*target_player).playerstate = PlayerState::PST_DEAD;
        P_DropWeapon(state, target_player);
        if target_player_id.0 as i32 == state.g_game.consoleplayer && state.am_map.automapactive {
            AM_Stop(state);
        }
    }
    let target_info = state.info.mobjinfo_mut((*target).type_0);
    if (*target).health < -(*target_info).spawnhealth && (*target_info).xdeathstate != StateNum::S_NULL {
        let xdeathstate = (*target_info).xdeathstate;
        P_SetMobjState(state, target, xdeathstate);
    } else {
        let deathstate = (*target_info).deathstate;
        P_SetMobjState(state, target, deathstate);
    }
    (*target).tics -= P_Random(&mut state.m_random) & 3 as i32;
    if (*target).tics < 1 as i32 {
        (*target).tics = 1 as i32;
    }
    if state.doomstat.gameversion == GameVersion::chex {
        return;
    }
    match (*target).type_0 as u32 {
        23 | 1 => {
            item = MobjType::MT_CLIP;
        }
        2 => {
            item = MobjType::MT_SHOTGUN;
        }
        10 => {
            item = MobjType::MT_CHAINGUN;
        }
        _ => return,
    }
    mo = P_SpawnMobj(state, (*target).x, (*target).y, ONFLOORZ, item);
    (*mo).flags |= MF_DROPPED as i32;
}
pub unsafe fn P_DamageMobj(
    state: &mut GameState,
    mut target: *mut mobj_t,
    mut inflictor: *mut mobj_t,
    mut source: *mut mobj_t,
    mut damage: i32,
) {
    let mut ang: u32 = 0;
    let mut saved: i32 = 0;
    let mut player: *mut player_t = ::core::ptr::null_mut::<player_t>();
    let mut thrust: fixed_t = 0;
    if (*target).flags & MF_SHOOTABLE as i32 == 0 {
        return;
    }
    if (*target).health <= 0 as i32 {
        return;
    }
    if (*target).flags & MF_SKULLFLY as i32 != 0 {
        (*target).momz = 0 as i32 as fixed_t;
        (*target).momy = (*target).momz;
        (*target).momx = (*target).momy;
    }
    let target_player_id = (*target).player;
    player = match target_player_id {
        Some(id) => state.g_game.player_mut(id),
        None => ::core::ptr::null_mut::<player_t>(),
    };
    if !player.is_null() && state.g_game.gameskill == SkillType::sk_baby {
        damage >>= 1 as i32;
    }
    if !inflictor.is_null()
        && (*target).flags & MF_NOCLIP as i32 == 0
        && (source.is_null()
            || (*source).player.is_none()
            || (*state.g_game.player_mut((*source).player.unwrap())).readyweapon as u32
                != weapontype_t::wp_chainsaw as i32 as u32)
    {
        ang = R_PointToAngle2(
            state,
            (*inflictor).x,
            (*inflictor).y,
            (*target).x,
            (*target).y,
        ) as u32;
        thrust = (damage * (FRACUNIT >> 3 as i32) * 100 as i32 / (*state.info.mobjinfo_mut((*target).type_0)).mass) as fixed_t;
        if damage < 40 as i32
            && damage > (*target).health
            && (*target).z - (*inflictor).z > 64 as i32 * FRACUNIT
            && P_Random(&mut state.m_random) & 1 as i32 != 0
        {
            ang = ang.wrapping_add(ANG180);
            thrust *= 4 as i32;
        }
        ang >>= ANGLETOFINESHIFT;
        (*target).momx += FixedMul(thrust, finecosine[ang as isize]);
        (*target).momy += FixedMul(thrust, finesine[ang as usize]);
    }
    if !player.is_null() {
        if (*state
            .p_setup
            .sector_mut(state.p_setup.subsectors[(*target).subsector.0 as usize].sector))
        .special as i32
            == 11 as i32
            && damage >= (*target).health
        {
            damage = (*target).health - 1 as i32;
        }
        if damage < 1000 as i32
            && ((*player).cheats & CF_GODMODE as i32 != 0
                || (*player).powers[PowerType::pw_invulnerability as usize] != 0)
        {
            return;
        }
        if (*player).armortype != 0 {
            if (*player).armortype == 1 as i32 {
                saved = damage / 3 as i32;
            } else {
                saved = damage / 2 as i32;
            }
            if (*player).armorpoints <= saved {
                saved = (*player).armorpoints;
                (*player).armortype = 0 as i32;
            }
            (*player).armorpoints -= saved;
            damage -= saved;
        }
        (*player).health -= damage;
        if (*player).health < 0 as i32 {
            (*player).health = 0 as i32;
        }
        (*player).attacker = if source.is_null() {
            None
        } else {
            Some((*source).id)
        };
        (*player).damagecount += damage;
        if (*player).damagecount > 100 as i32 {
            (*player).damagecount = 100 as i32;
        }
        if target_player_id == Some(PlayerId(state.g_game.consoleplayer as u8)) {
            I_Tactile();
        }
    }
    (*target).health -= damage;
    if (*target).health <= 0 as i32 {
        P_KillMobj(state, source, target);
        return;
    }
    if P_Random(&mut state.m_random) < (*state.info.mobjinfo_mut((*target).type_0)).painchance
        && (*target).flags & MF_SKULLFLY as i32 == 0
    {
        (*target).flags |= MF_JUSTHIT as i32;
        let painstate = (*state.info.mobjinfo_mut((*target).type_0)).painstate;
        P_SetMobjState(state, target, painstate);
    }
    (*target).reactiontime = 0 as i32;
    if ((*target).threshold == 0 || (*target).type_0 as u32 == MobjType::MT_VILE as i32 as u32)
        && !source.is_null()
        && source != target
        && (*source).type_0 as u32 != MobjType::MT_VILE as i32 as u32
    {
        (*target).target = Some((*source).id);
        (*target).threshold = BASETHRESHOLD;
        let target_info = state.info.mobjinfo_mut((*target).type_0);
        if (*target).state == Some(StateId((*target_info).spawnstate as u32))
            && (*target_info).seestate != StateNum::S_NULL
        {
            let seestate = (*target_info).seestate;
            P_SetMobjState(state, target, seestate);
        }
    }
}
