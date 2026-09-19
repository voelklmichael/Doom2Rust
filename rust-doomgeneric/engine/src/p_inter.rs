use crate::am_map::AM_Stop;
use crate::d_items::weaponinfo;
use crate::d_mode::SkillType;
use crate::d_mode::{GameMode_t, GameVersion};
use crate::d_player::weapontype_t;
use crate::d_player::PowerType;
use crate::d_player::CF_GODMODE;
use crate::d_player::{ammotype_from_raw, ammotype_t, NUMAMMO};
use crate::d_player::{player_t, PlayerId, PlayerState};
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::i_system::I_Tactile;
use crate::info::StateId;
use crate::m_fixed::fixed_t;
use crate::m_fixed::FixedMul;
use crate::m_fixed::FRACUNIT;
use crate::m_random::P_Random;
use crate::p_mobj::MobjId;
use alloc::string::ToString;

use crate::p_mobj::MobjType;
use crate::p_mobj::P_RemoveMobj;
use crate::p_mobj::P_SetMobjState;
use crate::p_mobj::P_SpawnMobj;
use crate::p_mobj::StateNum;
use crate::p_mobj::ONFLOORZ;
use crate::p_mobj::{
    MF_CORPSE, MF_COUNTITEM, MF_COUNTKILL, MF_DROPOFF, MF_DROPPED, MF_FLOAT, MF_JUSTHIT, MF_NOCLIP,
    MF_NOGRAVITY, MF_SHADOW, MF_SHOOTABLE, MF_SKULLFLY, MF_SOLID,
};
use crate::p_pspr::P_DropWeapon;
use crate::r_main::R_PointToAngle2;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::tables::finecosine;
use crate::tables::finesine;
use crate::tables::ANG180;
use crate::tables::ANGLETOFINESHIFT;

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
pub const IRONTICS: i32 = 2100;
pub const INFRATICS: i32 = 4200;
pub const INVISTICS: i32 = 2100;
pub const INVULNTICS: i32 = 1050;
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
pub static maxammo: [i32; 4] = [200, 50, 300, 50];
pub static clipammo: [i32; 4] = [10, 4, 20, 1];
pub fn P_GiveAmmo(
    state: &mut GameState,
    player_id: PlayerId,
    ammo: ammotype_t,
    mut num: i32,
) -> bool {
    let player = &mut state.g_game.players[player_id.0 as usize];

    if ammo as u32 == ammotype_t::am_noammo as i32 as u32 {
        return false;
    }
    if ammo as u32 > NUMAMMO as u32 {
        I_Error(&format!("P_GiveAmmo: bad type {}", ammo as u32));
    }
    if player.ammo[ammo as usize] == player.maxammo[ammo as usize] {
        return false;
    }
    if num != 0 {
        num *= clipammo[ammo as usize];
    } else {
        num = clipammo[ammo as usize] / 2;
    }
    if state.g_game.gameskill == SkillType::sk_baby
        || state.g_game.gameskill == SkillType::sk_nightmare
    {
        num <<= 1;
    }
    let oldammo: i32 = player.ammo[ammo as usize];
    player.ammo[ammo as usize] += num;
    if player.ammo[ammo as usize] > player.maxammo[ammo as usize] {
        player.ammo[ammo as usize] = player.maxammo[ammo as usize];
    }
    if oldammo != 0 {
        return true;
    }
    match ammo as u32 {
        0 => {
            if player.readyweapon as u32 == weapontype_t::wp_fist as i32 as u32 {
                if player.weaponowned[weapontype_t::wp_chaingun as usize] {
                    player.pendingweapon = weapontype_t::wp_chaingun;
                } else {
                    player.pendingweapon = weapontype_t::wp_pistol;
                }
            }
        }
        1 => {
            if (player.readyweapon as u32 == weapontype_t::wp_fist as i32 as u32
                || player.readyweapon as u32 == weapontype_t::wp_pistol as i32 as u32)
                && player.weaponowned[weapontype_t::wp_shotgun as usize]
            {
                player.pendingweapon = weapontype_t::wp_shotgun;
            }
        }
        2 => {
            if (player.readyweapon as u32 == weapontype_t::wp_fist as i32 as u32
                || player.readyweapon as u32 == weapontype_t::wp_pistol as i32 as u32)
                && player.weaponowned[weapontype_t::wp_plasma as usize]
            {
                player.pendingweapon = weapontype_t::wp_plasma;
            }
        }
        3 if player.readyweapon as u32 == weapontype_t::wp_fist as i32 as u32
            && player.weaponowned[weapontype_t::wp_missile as usize] =>
        {
            player.pendingweapon = weapontype_t::wp_missile;
        }
        _ => {}
    }
    true
}
pub fn P_GiveWeapon(
    state: &mut GameState,
    player: PlayerId,
    weapon: weapontype_t,
    dropped: bool,
) -> bool {
    let gaveammo: bool;
    let gaveweapon: bool;
    if state.g_game.netgame && state.g_game.deathmatch != 2 && !dropped {
        if state.g_game.players[player.0 as usize].weaponowned[weapon as usize] {
            return false;
        }
        state.g_game.players[player.0 as usize].bonuscount += BONUSADD;
        state.g_game.players[player.0 as usize].weaponowned[weapon as usize] = true;
        if state.g_game.deathmatch != 0 {
            P_GiveAmmo(state, player, weaponinfo[weapon as usize].ammo, 5);
        } else {
            P_GiveAmmo(state, player, weaponinfo[weapon as usize].ammo, 2);
        }
        state.g_game.players[player.0 as usize].pendingweapon = weapon;
        if player.0 as i32 == state.g_game.consoleplayer {
            S_StartSound(state, SoundOrigin::None, SfxName::sfx_wpnup as i32);
        }
        return false;
    }
    if weaponinfo[weapon as usize].ammo as u32 != ammotype_t::am_noammo as i32 as u32 {
        if dropped {
            gaveammo = P_GiveAmmo(state, player, weaponinfo[weapon as usize].ammo, 1);
        } else {
            gaveammo = P_GiveAmmo(state, player, weaponinfo[weapon as usize].ammo, 2);
        }
    } else {
        gaveammo = false;
    }
    if state.g_game.players[player.0 as usize].weaponowned[weapon as usize] {
        gaveweapon = false;
    } else {
        gaveweapon = true;
        state.g_game.players[player.0 as usize].weaponowned[weapon as usize] = true;
        state.g_game.players[player.0 as usize].pendingweapon = weapon;
    }
    gaveweapon || gaveammo
}
pub fn P_GiveBody(state: &mut GameState, player_id: PlayerId, num: i32) -> bool {
    let player = &mut state.g_game.players[player_id.0 as usize];
    if player.health >= MAXHEALTH {
        return false;
    }
    player.health += num;
    if player.health > MAXHEALTH {
        player.health = MAXHEALTH;
    }
    let player_mo = player.mo.unwrap();
    state.p_mobj.mo_mut(player_mo).health = player.health;
    true
}
pub fn P_GiveArmor(player: &mut player_t, armortype: i32) -> bool {
    let hits: i32 = armortype * 100;
    if player.armorpoints >= hits {
        return false;
    }
    player.armortype = armortype;
    player.armorpoints = hits;
    true
}
pub fn P_GiveCard(player: &mut player_t, card: CardType) {
    if player.cards[card as usize] {
        return;
    }
    player.bonuscount = BONUSADD;
    player.cards[card as usize] = true;
}
pub fn P_GivePower(state: &mut GameState, player: PlayerId, power: i32) -> bool {
    if power == PowerType::pw_invulnerability as i32 {
        state.g_game.players[player.0 as usize].powers[power as usize] = INVULNTICS;
        return true;
    }
    if power == PowerType::pw_invisibility as i32 {
        state.g_game.players[player.0 as usize].powers[power as usize] = INVISTICS;
        let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
        state.p_mobj.mo_mut(player_mo).flags |= MF_SHADOW;
        return true;
    }
    if power == PowerType::pw_infrared as i32 {
        state.g_game.players[player.0 as usize].powers[power as usize] = INFRATICS;
        return true;
    }
    if power == PowerType::pw_ironfeet as i32 {
        state.g_game.players[player.0 as usize].powers[power as usize] = IRONTICS;
        return true;
    }
    if power == PowerType::pw_strength as i32 {
        P_GiveBody(state, player, 100);
        state.g_game.players[player.0 as usize].powers[power as usize] = 1;
        return true;
    }
    if state.g_game.players[player.0 as usize].powers[power as usize] != 0 {
        return false;
    }
    state.g_game.players[player.0 as usize].powers[power as usize] = 1;
    true
}
pub fn P_TouchSpecialThing(state: &mut GameState, special: MobjId, toucher: MobjId) {
    let mut sound: i32;
    let delta: fixed_t = state.p_mobj.mo(special).z - state.p_mobj.mo(toucher).z;
    if delta > state.p_mobj.mo(toucher).height || delta < -8 * FRACUNIT {
        return;
    }
    sound = SfxName::sfx_itemup as i32;
    let player = state.p_mobj.mo(toucher).player.unwrap();
    if state.p_mobj.mo(toucher).health <= 0 {
        return;
    }
    match state.p_mobj.mo(special).sprite as u32 {
        55 => {
            if !P_GiveArmor(
                &mut state.g_game.players[player.0 as usize],
                deh_green_armor_class,
            ) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up the armor.".to_string());
        }
        56 => {
            if !P_GiveArmor(
                &mut state.g_game.players[player.0 as usize],
                deh_blue_armor_class,
            ) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up the MegaArmor!".to_string());
        }
        60 => {
            state.g_game.players[player.0 as usize].health += 1;
            if state.g_game.players[player.0 as usize].health > deh_max_health {
                state.g_game.players[player.0 as usize].health = deh_max_health;
            }
            state.p_mobj.mo_mut(toucher).health = state.g_game.players[player.0 as usize].health;
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a health bonus.".to_string());
        }
        61 => {
            state.g_game.players[player.0 as usize].armorpoints += 1;
            if state.g_game.players[player.0 as usize].armorpoints > deh_max_armor {
                state.g_game.players[player.0 as usize].armorpoints = deh_max_armor;
            }
            if state.g_game.players[player.0 as usize].armortype == 0 {
                state.g_game.players[player.0 as usize].armortype = 1;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up an armor bonus.".to_string());
        }
        70 => {
            state.g_game.players[player.0 as usize].health += deh_soulsphere_health;
            if state.g_game.players[player.0 as usize].health > deh_max_soulsphere {
                state.g_game.players[player.0 as usize].health = deh_max_soulsphere;
            }
            state.p_mobj.mo_mut(toucher).health = state.g_game.players[player.0 as usize].health;
            state.g_game.players[player.0 as usize].message = Some("Supercharge!".to_string());
            sound = SfxName::sfx_getpow as i32;
        }
        74 => {
            if state.doomstat.gamemode as u32 != GameMode_t::commercial as i32 as u32 {
                return;
            }
            state.g_game.players[player.0 as usize].health = deh_megasphere_health;
            state.p_mobj.mo_mut(toucher).health = state.g_game.players[player.0 as usize].health;
            P_GiveArmor(&mut state.g_game.players[player.0 as usize], 2);
            state.g_game.players[player.0 as usize].message = Some("MegaSphere!".to_string());
            sound = SfxName::sfx_getpow as i32;
        }
        62 => {
            if !state.g_game.players[player.0 as usize].cards[CardType::it_bluecard as usize] {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a blue keycard.".to_string());
            }
            P_GiveCard(
                &mut state.g_game.players[player.0 as usize],
                CardType::it_bluecard,
            );
            if state.g_game.netgame {
                return;
            }
        }
        64 => {
            if !state.g_game.players[player.0 as usize].cards[CardType::it_yellowcard as usize] {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a yellow keycard.".to_string());
            }
            P_GiveCard(
                &mut state.g_game.players[player.0 as usize],
                CardType::it_yellowcard,
            );
            if state.g_game.netgame {
                return;
            }
        }
        63 => {
            if !state.g_game.players[player.0 as usize].cards[CardType::it_redcard as usize] {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a red keycard.".to_string());
            }
            P_GiveCard(
                &mut state.g_game.players[player.0 as usize],
                CardType::it_redcard,
            );
            if state.g_game.netgame {
                return;
            }
        }
        65 => {
            if !state.g_game.players[player.0 as usize].cards[CardType::it_blueskull as usize] {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a blue skull key.".to_string());
            }
            P_GiveCard(
                &mut state.g_game.players[player.0 as usize],
                CardType::it_blueskull,
            );
            if state.g_game.netgame {
                return;
            }
        }
        67 => {
            if !state.g_game.players[player.0 as usize].cards[CardType::it_yellowskull as usize] {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a yellow skull key.".to_string());
            }
            P_GiveCard(
                &mut state.g_game.players[player.0 as usize],
                CardType::it_yellowskull,
            );
            if state.g_game.netgame {
                return;
            }
        }
        66 => {
            if !state.g_game.players[player.0 as usize].cards[CardType::it_redskull as usize] {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a red skull key.".to_string());
            }
            P_GiveCard(
                &mut state.g_game.players[player.0 as usize],
                CardType::it_redskull,
            );
            if state.g_game.netgame {
                return;
            }
        }
        68 => {
            if !P_GiveBody(state, player, 10) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a stimpack.".to_string());
        }
        69 => {
            if !P_GiveBody(state, player, 25) {
                return;
            }
            if state.g_game.players[player.0 as usize].health < 25 {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a medikit that you REALLY need!".to_string());
            } else {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a medikit.".to_string());
            }
        }
        71 => {
            if !P_GivePower(state, player, PowerType::pw_invulnerability as i32) {
                return;
            }
            state.g_game.players[player.0 as usize].message = Some("Invulnerability!".to_string());
            sound = SfxName::sfx_getpow as i32;
        }
        72 => {
            if !P_GivePower(state, player, PowerType::pw_strength as i32) {
                return;
            }
            state.g_game.players[player.0 as usize].message = Some("Berserk!".to_string());
            if state.g_game.players[player.0 as usize].readyweapon as u32
                != weapontype_t::wp_fist as i32 as u32
            {
                state.g_game.players[player.0 as usize].pendingweapon = weapontype_t::wp_fist;
            }
            sound = SfxName::sfx_getpow as i32;
        }
        73 => {
            if !P_GivePower(state, player, PowerType::pw_invisibility as i32) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Partial Invisibility".to_string());
            sound = SfxName::sfx_getpow as i32;
        }
        75 => {
            if !P_GivePower(state, player, PowerType::pw_ironfeet as i32) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Radiation Shielding Suit".to_string());
            sound = SfxName::sfx_getpow as i32;
        }
        76 => {
            if !P_GivePower(state, player, PowerType::pw_allmap as i32) {
                return;
            }
            state.g_game.players[player.0 as usize].message = Some("Computer Area Map".to_string());
            sound = SfxName::sfx_getpow as i32;
        }
        77 => {
            if !P_GivePower(state, player, PowerType::pw_infrared as i32) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Light Amplification Visor".to_string());
            sound = SfxName::sfx_getpow as i32;
        }
        78 => {
            if state.p_mobj.mo(special).flags & MF_DROPPED != 0 {
                if !P_GiveAmmo(state, player, ammotype_t::am_clip, 0) {
                    return;
                }
            } else if !P_GiveAmmo(state, player, ammotype_t::am_clip, 1) {
                return;
            }
            state.g_game.players[player.0 as usize].message = Some("Picked up a clip.".to_string());
        }
        79 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_clip, 5) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a box of bullets.".to_string());
        }
        80 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_misl, 1) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a rocket.".to_string());
        }
        81 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_misl, 5) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a box of rockets.".to_string());
        }
        82 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_cell, 1) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up an energy cell.".to_string());
        }
        83 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_cell, 5) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up an energy cell pack.".to_string());
        }
        84 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_shell, 1) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up 4 shotgun shells.".to_string());
        }
        85 => {
            if !P_GiveAmmo(state, player, ammotype_t::am_shell, 5) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a box of shotgun shells.".to_string());
        }
        86 => {
            if !state.g_game.players[player.0 as usize].backpack {
                for i in 0..(NUMAMMO as usize) {
                    state.g_game.players[player.0 as usize].maxammo[i] *= 2;
                }
                state.g_game.players[player.0 as usize].backpack = true;
            }
            for i in 0..NUMAMMO {
                P_GiveAmmo(state, player, ammotype_from_raw(i), 1);
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a backpack full of ammo!".to_string());
        }
        87 => {
            if !P_GiveWeapon(state, player, weapontype_t::wp_bfg, false) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("You got the BFG9000!  Oh, yes.".to_string());
            sound = SfxName::sfx_wpnup as i32;
        }
        88 => {
            if !P_GiveWeapon(
                state,
                player,
                weapontype_t::wp_chaingun,
                state.p_mobj.mo(special).flags & MF_DROPPED != 0,
            ) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("You got the chaingun!".to_string());
            sound = SfxName::sfx_wpnup as i32;
        }
        89 => {
            if !P_GiveWeapon(state, player, weapontype_t::wp_chainsaw, false) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("A chainsaw!  Find some meat!".to_string());
            sound = SfxName::sfx_wpnup as i32;
        }
        90 => {
            if !P_GiveWeapon(state, player, weapontype_t::wp_missile, false) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("You got the rocket launcher!".to_string());
            sound = SfxName::sfx_wpnup as i32;
        }
        91 => {
            if !P_GiveWeapon(state, player, weapontype_t::wp_plasma, false) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("You got the plasma gun!".to_string());
            sound = SfxName::sfx_wpnup as i32;
        }
        92 => {
            if !P_GiveWeapon(
                state,
                player,
                weapontype_t::wp_shotgun,
                state.p_mobj.mo(special).flags & MF_DROPPED != 0,
            ) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("You got the shotgun!".to_string());
            sound = SfxName::sfx_wpnup as i32;
        }
        93 => {
            if !P_GiveWeapon(
                state,
                player,
                weapontype_t::wp_supershotgun,
                state.p_mobj.mo(special).flags & MF_DROPPED != 0,
            ) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("You got the super shotgun!".to_string());
            sound = SfxName::sfx_wpnup as i32;
        }
        _ => {
            I_Error("P_SpecialThing: Unknown gettable thing");
        }
    }
    if state.p_mobj.mo(special).flags & MF_COUNTITEM != 0 {
        state.g_game.players[player.0 as usize].itemcount += 1;
    }
    P_RemoveMobj(state, special);
    state.g_game.players[player.0 as usize].bonuscount += BONUSADD;
    if player.0 as i32 == state.g_game.consoleplayer {
        S_StartSound(state, SoundOrigin::None, sound);
    }
}
pub fn P_KillMobj(state: &mut GameState, source: Option<MobjId>, target: MobjId) {
    {
        let t = state.p_mobj.mo_mut(target);
        t.flags &= !(MF_SHOOTABLE | MF_FLOAT | MF_SKULLFLY);
        if t.kind as u32 != MobjType::MT_SKULL as i32 as u32 {
            t.flags &= !MF_NOGRAVITY;
        }
        t.flags |= MF_CORPSE | MF_DROPOFF;
        t.height >>= 2;
    }
    let source_player = source.and_then(|id| state.p_mobj.mo(id).player);
    let (target_flags, target_player) = {
        let t = state.p_mobj.mo(target);
        (t.flags, t.player)
    };
    if let Some(source_player_id) = source_player {
        if target_flags & MF_COUNTKILL != 0 {
            state.g_game.player_mut(source_player_id).killcount += 1;
        }
        if let Some(target_player_id) = target_player {
            state.g_game.player_mut(source_player_id).frags[target_player_id.0 as usize] += 1;
        }
    } else if !state.g_game.netgame && target_flags & MF_COUNTKILL != 0 {
        state.g_game.players[0].killcount += 1;
    }
    if let Some(target_player_id) = target_player {
        if source.is_none() {
            state.g_game.player_mut(target_player_id).frags[target_player_id.0 as usize] += 1;
        }
        state.p_mobj.mo_mut(target).flags &= !MF_SOLID;
        state.g_game.player_mut(target_player_id).playerstate = PlayerState::PST_DEAD;
        P_DropWeapon(state, target_player_id);
        if target_player_id.0 as i32 == state.g_game.consoleplayer && state.am_map.automapactive {
            AM_Stop(state);
        }
    }
    let (target_type, target_health) = {
        let t = state.p_mobj.mo(target);
        (t.kind, t.health)
    };
    let (spawnhealth, xdeathstate, deathstate) = {
        let info = state.info.mobjinfo_mut(target_type);
        (info.spawnhealth, info.xdeathstate, info.deathstate)
    };
    if target_health < -spawnhealth && xdeathstate != StateNum::S_NULL {
        P_SetMobjState(state, target, xdeathstate);
    } else {
        P_SetMobjState(state, target, deathstate);
    }
    state.p_mobj.mo_mut(target).tics -= P_Random(&mut state.m_random) & 3;
    if state.p_mobj.mo(target).tics < 1 {
        state.p_mobj.mo_mut(target).tics = 1;
    }
    if state.doomstat.gameversion == GameVersion::chex {
        return;
    }
    let item = match target_type as u32 {
        23 | 1 => MobjType::MT_CLIP,
        2 => MobjType::MT_SHOTGUN,
        10 => MobjType::MT_CHAINGUN,
        _ => return,
    };
    let (target_x, target_y) = {
        let t = state.p_mobj.mo(target);
        (t.x, t.y)
    };
    let mo = P_SpawnMobj(state, target_x, target_y, ONFLOORZ, item);
    state.p_mobj.mo_mut(mo).flags |= MF_DROPPED;
}
pub fn P_DamageMobj(
    state: &mut GameState,
    target: MobjId,
    inflictor: Option<MobjId>,
    source: Option<MobjId>,
    mut damage: i32,
) {
    let (target_flags, target_health) = {
        let t = state.p_mobj.mo(target);
        (t.flags, t.health)
    };
    if target_flags & MF_SHOOTABLE == 0 {
        return;
    }
    if target_health <= 0 {
        return;
    }
    if target_flags & MF_SKULLFLY != 0 {
        let t = state.p_mobj.mo_mut(target);
        t.momz = 0;
        t.momy = t.momz;
        t.momx = t.momy;
    }
    let target_player_id = state.p_mobj.mo(target).player;
    if target_player_id.is_some() && state.g_game.gameskill == SkillType::sk_baby {
        damage >>= 1;
    }
    let source_player = source.and_then(|id| state.p_mobj.mo(id).player);
    let source_uses_chainsaw = source_player.is_some_and(|source_player_id| {
        state.g_game.players[source_player_id.0 as usize].readyweapon as u32
            == weapontype_t::wp_chainsaw as i32 as u32
    });
    if let Some(inflictor) = inflictor {
        if target_flags & MF_NOCLIP == 0 && !source_uses_chainsaw {
            let (inflictor_x, inflictor_y, inflictor_z) = {
                let i = state.p_mobj.mo(inflictor);
                (i.x, i.y, i.z)
            };
            let (target_x, target_y, target_z, target_type) = {
                let t = state.p_mobj.mo(target);
                (t.x, t.y, t.z, t.kind)
            };
            let mut ang: u32 = R_PointToAngle2(state, inflictor_x, inflictor_y, target_x, target_y);
            let mut thrust: fixed_t = (damage * (FRACUNIT >> 3) * 100
                / state.info.mobjinfo_mut(target_type).mass)
                as fixed_t;
            if damage < 40
                && damage > target_health
                && target_z - inflictor_z > 64 * FRACUNIT
                && P_Random(&mut state.m_random) & 1 != 0
            {
                ang = ang.wrapping_add(ANG180);
                thrust *= 4;
            }
            ang >>= ANGLETOFINESHIFT;
            let t = state.p_mobj.mo_mut(target);
            t.momx += FixedMul(thrust, finecosine[ang as usize]);
            t.momy += FixedMul(thrust, finesine[ang as usize]);
        }
    }
    if let Some(player_id) = target_player_id {
        let target_subsector = state.p_mobj.mo(target).subsector;
        let sector_special = state
            .p_setup
            .sector_mut(state.p_setup.subsectors[target_subsector.0 as usize].sector)
            .special;
        if sector_special as i32 == 11 && damage >= target_health {
            damage = target_health - 1;
        }
        let player = state.g_game.player_mut(player_id);
        if damage < 1000
            && (player.cheats & CF_GODMODE != 0
                || player.powers[PowerType::pw_invulnerability as usize] != 0)
        {
            return;
        }
        if player.armortype != 0 {
            let mut saved: i32 = if player.armortype == 1 {
                damage / 3
            } else {
                damage / 2
            };
            if player.armorpoints <= saved {
                saved = player.armorpoints;
                player.armortype = 0;
            }
            player.armorpoints -= saved;
            damage -= saved;
        }
        player.health -= damage;
        if player.health < 0 {
            player.health = 0;
        }
        player.attacker = source;
        player.damagecount += damage;
        if player.damagecount > 100 {
            player.damagecount = 100;
        }
        if target_player_id == Some(PlayerId(state.g_game.consoleplayer as u8)) {
            I_Tactile();
        }
    }
    state.p_mobj.mo_mut(target).health -= damage;
    if state.p_mobj.mo(target).health <= 0 {
        P_KillMobj(state, source, target);
        return;
    }
    let target_type = state.p_mobj.mo(target).kind;
    if P_Random(&mut state.m_random) < state.info.mobjinfo_mut(target_type).painchance
        && state.p_mobj.mo(target).flags & MF_SKULLFLY == 0
    {
        state.p_mobj.mo_mut(target).flags |= MF_JUSTHIT;
        let painstate = state.info.mobjinfo_mut(target_type).painstate;
        P_SetMobjState(state, target, painstate);
    }
    state.p_mobj.mo_mut(target).reactiontime = 0;
    if (state.p_mobj.mo(target).threshold == 0
        || target_type as u32 == MobjType::MT_VILE as i32 as u32)
        && source.is_some_and(|source| {
            source != target
                && state.p_mobj.mo(source).kind as u32 != MobjType::MT_VILE as i32 as u32
        })
    {
        {
            let t = state.p_mobj.mo_mut(target);
            t.target = source;
            t.threshold = BASETHRESHOLD;
        }
        let (spawnstate, seestate) = {
            let info = state.info.mobjinfo_mut(target_type);
            (info.spawnstate, info.seestate)
        };
        if state.p_mobj.mo(target).state == Some(StateId(spawnstate as u32))
            && seestate != StateNum::S_NULL
        {
            P_SetMobjState(state, target, seestate);
        }
    }
}
