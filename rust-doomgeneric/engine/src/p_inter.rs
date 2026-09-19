use crate::am_map::am_stop;
use crate::d_items::WEAPONINFO;
use crate::d_mode::SkillType;
use crate::d_mode::{GameMode, GameVersion};
use crate::d_player::CheatFlags;
use crate::d_player::PowerType;
use crate::d_player::WeaponType;
use crate::p_mobj::MobjFlags;

use crate::d_player::{ammotype_from_raw, AmmoType, NUMAMMO};
use crate::d_player::{Player, PlayerId, PlayerState};
use crate::game_state::GameState;
use crate::i_system::error;
use crate::i_system::tactile;
use crate::info::StateId;
use crate::m_fixed::fixed_mul;
use crate::m_fixed::Fixed;
use crate::m_fixed::FRACUNIT;
use crate::m_random::p_random;
use crate::p_mobj::MobjId;
use alloc::string::ToString;

use crate::p_mobj::remove_mobj;
use crate::p_mobj::set_mobj_state;
use crate::p_mobj::spawn_mobj;
use crate::p_mobj::MobjType;
use crate::p_mobj::StateNum;
use crate::p_mobj::ONFLOORZ;
use crate::p_pspr::drop_weapon;
use crate::r_main::point_to_angle2;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::tables::ANG180;
use crate::tables::ANGLETOFINESHIFT;
use crate::tables::FINECOSINE;
use crate::tables::FINESINE;

pub const NUMCARDS: i32 = 6;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CardType {
    Bluecard = 0,
    Yellowcard = 1,
    Redcard = 2,
    Blueskull = 3,
    Yellowskull = 4,
    Redskull = 5,
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
pub const DEH_MAX_HEALTH: i32 = DEH_DEFAULT_MAX_HEALTH;
pub const DEH_MAX_ARMOR: i32 = DEH_DEFAULT_MAX_ARMOR;
pub const DEH_GREEN_ARMOR_CLASS: i32 = DEH_DEFAULT_GREEN_ARMOR_CLASS;
pub const DEH_BLUE_ARMOR_CLASS: i32 = DEH_DEFAULT_BLUE_ARMOR_CLASS;
pub const DEH_MAX_SOULSPHERE: i32 = DEH_DEFAULT_MAX_SOULSPHERE;
pub const DEH_SOULSPHERE_HEALTH: i32 = DEH_DEFAULT_SOULSPHERE_HEALTH;
pub const DEH_MEGASPHERE_HEALTH: i32 = DEH_DEFAULT_MEGASPHERE_HEALTH;
pub const MAXHEALTH: i32 = 100;
pub const BASETHRESHOLD: i32 = 100;
pub const BONUSADD: i32 = 6;
pub static MAXAMMO: [i32; 4] = [200, 50, 300, 50];
pub static CLIPAMMO: [i32; 4] = [10, 4, 20, 1];
pub fn give_ammo(state: &mut GameState, player_id: PlayerId, ammo: AmmoType, mut num: i32) -> bool {
    let player = &mut state.g_game.players[player_id.0 as usize];

    if ammo as u32 == AmmoType::Noammo as i32 as u32 {
        return false;
    }
    if ammo as u32 > NUMAMMO as u32 {
        error(&format!("P_GiveAmmo: bad type {}", ammo as u32));
    }
    if player.ammo[ammo as usize] == player.maxammo[ammo as usize] {
        return false;
    }
    if num != 0 {
        num *= CLIPAMMO[ammo as usize];
    } else {
        num = CLIPAMMO[ammo as usize] / 2;
    }
    if state.g_game.gameskill == SkillType::Baby || state.g_game.gameskill == SkillType::Nightmare {
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
            if player.readyweapon as u32 == WeaponType::Fist as i32 as u32 {
                if player.weaponowned[WeaponType::Chaingun as usize] {
                    player.pendingweapon = WeaponType::Chaingun;
                } else {
                    player.pendingweapon = WeaponType::Pistol;
                }
            }
        }
        1 => {
            if (player.readyweapon as u32 == WeaponType::Fist as i32 as u32
                || player.readyweapon as u32 == WeaponType::Pistol as i32 as u32)
                && player.weaponowned[WeaponType::Shotgun as usize]
            {
                player.pendingweapon = WeaponType::Shotgun;
            }
        }
        2 => {
            if (player.readyweapon as u32 == WeaponType::Fist as i32 as u32
                || player.readyweapon as u32 == WeaponType::Pistol as i32 as u32)
                && player.weaponowned[WeaponType::Plasma as usize]
            {
                player.pendingweapon = WeaponType::Plasma;
            }
        }
        3 if player.readyweapon as u32 == WeaponType::Fist as i32 as u32
            && player.weaponowned[WeaponType::Missile as usize] =>
        {
            player.pendingweapon = WeaponType::Missile;
        }
        _ => {}
    }
    true
}
pub fn give_weapon(
    state: &mut GameState,
    player: PlayerId,
    weapon: WeaponType,
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
            give_ammo(state, player, WEAPONINFO[weapon as usize].ammo, 5);
        } else {
            give_ammo(state, player, WEAPONINFO[weapon as usize].ammo, 2);
        }
        state.g_game.players[player.0 as usize].pendingweapon = weapon;
        if player.0 as i32 == state.g_game.consoleplayer {
            s_start_sound(state, SoundOrigin::None, SfxName::Wpnup as i32);
        }
        return false;
    }
    if WEAPONINFO[weapon as usize].ammo as u32 == AmmoType::Noammo as i32 as u32 {
        gaveammo = false;
    } else {
        if dropped {
            gaveammo = give_ammo(state, player, WEAPONINFO[weapon as usize].ammo, 1);
        } else {
            gaveammo = give_ammo(state, player, WEAPONINFO[weapon as usize].ammo, 2);
        }
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
pub fn give_body(state: &mut GameState, player_id: PlayerId, num: i32) -> bool {
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
pub fn give_armor(player: &mut Player, armortype: i32) -> bool {
    let hits: i32 = armortype * 100;
    if player.armorpoints >= hits {
        return false;
    }
    player.armortype = armortype;
    player.armorpoints = hits;
    true
}
pub fn give_card(player: &mut Player, card: CardType) {
    if player.cards[card as usize] {
        return;
    }
    player.bonuscount = BONUSADD;
    player.cards[card as usize] = true;
}
pub fn give_power(state: &mut GameState, player: PlayerId, power: i32) -> bool {
    if power == PowerType::Invulnerability as i32 {
        state.g_game.players[player.0 as usize].powers[power as usize] = INVULNTICS;
        return true;
    }
    if power == PowerType::Invisibility as i32 {
        state.g_game.players[player.0 as usize].powers[power as usize] = INVISTICS;
        let player_mo = state.g_game.players[player.0 as usize].mo.unwrap();
        state.p_mobj.mo_mut(player_mo).flags |= MobjFlags::SHADOW;
        return true;
    }
    if power == PowerType::Infrared as i32 {
        state.g_game.players[player.0 as usize].powers[power as usize] = INFRATICS;
        return true;
    }
    if power == PowerType::Ironfeet as i32 {
        state.g_game.players[player.0 as usize].powers[power as usize] = IRONTICS;
        return true;
    }
    if power == PowerType::Strength as i32 {
        give_body(state, player, 100);
        state.g_game.players[player.0 as usize].powers[power as usize] = 1;
        return true;
    }
    if state.g_game.players[player.0 as usize].powers[power as usize] != 0 {
        return false;
    }
    state.g_game.players[player.0 as usize].powers[power as usize] = 1;
    true
}
pub fn touch_special_thing(state: &mut GameState, special: MobjId, toucher: MobjId) {
    let mut sound: i32;
    let delta: Fixed = state.p_mobj.mo(special).z - state.p_mobj.mo(toucher).z;
    if delta > state.p_mobj.mo(toucher).height || delta < -8 * FRACUNIT {
        return;
    }
    sound = SfxName::Itemup as i32;
    let player = state.p_mobj.mo(toucher).player.unwrap();
    if state.p_mobj.mo(toucher).health <= 0 {
        return;
    }
    match state.p_mobj.mo(special).sprite as u32 {
        55 => {
            if !give_armor(
                &mut state.g_game.players[player.0 as usize],
                DEH_GREEN_ARMOR_CLASS,
            ) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up the armor.".to_string());
        }
        56 => {
            if !give_armor(
                &mut state.g_game.players[player.0 as usize],
                DEH_BLUE_ARMOR_CLASS,
            ) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up the MegaArmor!".to_string());
        }
        60 => {
            state.g_game.players[player.0 as usize].health += 1;
            if state.g_game.players[player.0 as usize].health > DEH_MAX_HEALTH {
                state.g_game.players[player.0 as usize].health = DEH_MAX_HEALTH;
            }
            state.p_mobj.mo_mut(toucher).health = state.g_game.players[player.0 as usize].health;
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a health bonus.".to_string());
        }
        61 => {
            state.g_game.players[player.0 as usize].armorpoints += 1;
            if state.g_game.players[player.0 as usize].armorpoints > DEH_MAX_ARMOR {
                state.g_game.players[player.0 as usize].armorpoints = DEH_MAX_ARMOR;
            }
            if state.g_game.players[player.0 as usize].armortype == 0 {
                state.g_game.players[player.0 as usize].armortype = 1;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up an armor bonus.".to_string());
        }
        70 => {
            state.g_game.players[player.0 as usize].health += DEH_SOULSPHERE_HEALTH;
            if state.g_game.players[player.0 as usize].health > DEH_MAX_SOULSPHERE {
                state.g_game.players[player.0 as usize].health = DEH_MAX_SOULSPHERE;
            }
            state.p_mobj.mo_mut(toucher).health = state.g_game.players[player.0 as usize].health;
            state.g_game.players[player.0 as usize].message = Some("Supercharge!".to_string());
            sound = SfxName::Getpow as i32;
        }
        74 => {
            if state.doomstat.gamemode as u32 != GameMode::Commercial as i32 as u32 {
                return;
            }
            state.g_game.players[player.0 as usize].health = DEH_MEGASPHERE_HEALTH;
            state.p_mobj.mo_mut(toucher).health = state.g_game.players[player.0 as usize].health;
            give_armor(&mut state.g_game.players[player.0 as usize], 2);
            state.g_game.players[player.0 as usize].message = Some("MegaSphere!".to_string());
            sound = SfxName::Getpow as i32;
        }
        62 => {
            if !state.g_game.players[player.0 as usize].cards[CardType::Bluecard as usize] {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a blue keycard.".to_string());
            }
            give_card(
                &mut state.g_game.players[player.0 as usize],
                CardType::Bluecard,
            );
            if state.g_game.netgame {
                return;
            }
        }
        64 => {
            if !state.g_game.players[player.0 as usize].cards[CardType::Yellowcard as usize] {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a yellow keycard.".to_string());
            }
            give_card(
                &mut state.g_game.players[player.0 as usize],
                CardType::Yellowcard,
            );
            if state.g_game.netgame {
                return;
            }
        }
        63 => {
            if !state.g_game.players[player.0 as usize].cards[CardType::Redcard as usize] {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a red keycard.".to_string());
            }
            give_card(
                &mut state.g_game.players[player.0 as usize],
                CardType::Redcard,
            );
            if state.g_game.netgame {
                return;
            }
        }
        65 => {
            if !state.g_game.players[player.0 as usize].cards[CardType::Blueskull as usize] {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a blue skull key.".to_string());
            }
            give_card(
                &mut state.g_game.players[player.0 as usize],
                CardType::Blueskull,
            );
            if state.g_game.netgame {
                return;
            }
        }
        67 => {
            if !state.g_game.players[player.0 as usize].cards[CardType::Yellowskull as usize] {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a yellow skull key.".to_string());
            }
            give_card(
                &mut state.g_game.players[player.0 as usize],
                CardType::Yellowskull,
            );
            if state.g_game.netgame {
                return;
            }
        }
        66 => {
            if !state.g_game.players[player.0 as usize].cards[CardType::Redskull as usize] {
                state.g_game.players[player.0 as usize].message =
                    Some("Picked up a red skull key.".to_string());
            }
            give_card(
                &mut state.g_game.players[player.0 as usize],
                CardType::Redskull,
            );
            if state.g_game.netgame {
                return;
            }
        }
        68 => {
            if !give_body(state, player, 10) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a stimpack.".to_string());
        }
        69 => {
            if !give_body(state, player, 25) {
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
            if !give_power(state, player, PowerType::Invulnerability as i32) {
                return;
            }
            state.g_game.players[player.0 as usize].message = Some("Invulnerability!".to_string());
            sound = SfxName::Getpow as i32;
        }
        72 => {
            if !give_power(state, player, PowerType::Strength as i32) {
                return;
            }
            state.g_game.players[player.0 as usize].message = Some("Berserk!".to_string());
            if state.g_game.players[player.0 as usize].readyweapon as u32
                != WeaponType::Fist as i32 as u32
            {
                state.g_game.players[player.0 as usize].pendingweapon = WeaponType::Fist;
            }
            sound = SfxName::Getpow as i32;
        }
        73 => {
            if !give_power(state, player, PowerType::Invisibility as i32) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Partial Invisibility".to_string());
            sound = SfxName::Getpow as i32;
        }
        75 => {
            if !give_power(state, player, PowerType::Ironfeet as i32) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Radiation Shielding Suit".to_string());
            sound = SfxName::Getpow as i32;
        }
        76 => {
            if !give_power(state, player, PowerType::Allmap as i32) {
                return;
            }
            state.g_game.players[player.0 as usize].message = Some("Computer Area Map".to_string());
            sound = SfxName::Getpow as i32;
        }
        77 => {
            if !give_power(state, player, PowerType::Infrared as i32) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Light Amplification Visor".to_string());
            sound = SfxName::Getpow as i32;
        }
        78 => {
            if state.p_mobj.mo(special).flags.contains(MobjFlags::DROPPED) {
                if !give_ammo(state, player, AmmoType::Clip, 0) {
                    return;
                }
            } else if !give_ammo(state, player, AmmoType::Clip, 1) {
                return;
            }
            state.g_game.players[player.0 as usize].message = Some("Picked up a clip.".to_string());
        }
        79 => {
            if !give_ammo(state, player, AmmoType::Clip, 5) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a box of bullets.".to_string());
        }
        80 => {
            if !give_ammo(state, player, AmmoType::Misl, 1) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a rocket.".to_string());
        }
        81 => {
            if !give_ammo(state, player, AmmoType::Misl, 5) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a box of rockets.".to_string());
        }
        82 => {
            if !give_ammo(state, player, AmmoType::Cell, 1) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up an energy cell.".to_string());
        }
        83 => {
            if !give_ammo(state, player, AmmoType::Cell, 5) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up an energy cell pack.".to_string());
        }
        84 => {
            if !give_ammo(state, player, AmmoType::Shell, 1) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up 4 shotgun shells.".to_string());
        }
        85 => {
            if !give_ammo(state, player, AmmoType::Shell, 5) {
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
                give_ammo(state, player, ammotype_from_raw(i), 1);
            }
            state.g_game.players[player.0 as usize].message =
                Some("Picked up a backpack full of ammo!".to_string());
        }
        87 => {
            if !give_weapon(state, player, WeaponType::Bfg, false) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("You got the BFG9000!  Oh, yes.".to_string());
            sound = SfxName::Wpnup as i32;
        }
        88 => {
            if !give_weapon(
                state,
                player,
                WeaponType::Chaingun,
                state.p_mobj.mo(special).flags.contains(MobjFlags::DROPPED),
            ) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("You got the chaingun!".to_string());
            sound = SfxName::Wpnup as i32;
        }
        89 => {
            if !give_weapon(state, player, WeaponType::Chainsaw, false) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("A chainsaw!  Find some meat!".to_string());
            sound = SfxName::Wpnup as i32;
        }
        90 => {
            if !give_weapon(state, player, WeaponType::Missile, false) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("You got the rocket launcher!".to_string());
            sound = SfxName::Wpnup as i32;
        }
        91 => {
            if !give_weapon(state, player, WeaponType::Plasma, false) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("You got the plasma gun!".to_string());
            sound = SfxName::Wpnup as i32;
        }
        92 => {
            if !give_weapon(
                state,
                player,
                WeaponType::Shotgun,
                state.p_mobj.mo(special).flags.contains(MobjFlags::DROPPED),
            ) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("You got the shotgun!".to_string());
            sound = SfxName::Wpnup as i32;
        }
        93 => {
            if !give_weapon(
                state,
                player,
                WeaponType::Supershotgun,
                state.p_mobj.mo(special).flags.contains(MobjFlags::DROPPED),
            ) {
                return;
            }
            state.g_game.players[player.0 as usize].message =
                Some("You got the super shotgun!".to_string());
            sound = SfxName::Wpnup as i32;
        }
        _ => {
            error("P_SpecialThing: Unknown gettable thing");
        }
    }
    if state
        .p_mobj
        .mo(special)
        .flags
        .contains(MobjFlags::COUNTITEM)
    {
        state.g_game.players[player.0 as usize].itemcount += 1;
    }
    remove_mobj(state, special);
    state.g_game.players[player.0 as usize].bonuscount += BONUSADD;
    if player.0 as i32 == state.g_game.consoleplayer {
        s_start_sound(state, SoundOrigin::None, sound);
    }
}
pub fn kill_mobj(state: &mut GameState, source: Option<MobjId>, target: MobjId) {
    {
        let t = state.p_mobj.mo_mut(target);
        t.flags &= !(MobjFlags::SHOOTABLE | MobjFlags::FLOAT | MobjFlags::SKULLFLY);
        if t.kind as u32 != MobjType::Skull as i32 as u32 {
            t.flags &= !MobjFlags::NOGRAVITY;
        }
        t.flags |= MobjFlags::CORPSE | MobjFlags::DROPOFF;
        t.height >>= 2;
    }
    let source_player = source.and_then(|id| state.p_mobj.mo(id).player);
    let (target_flags, target_player) = {
        let t = state.p_mobj.mo(target);
        (t.flags, t.player)
    };
    if let Some(source_player_id) = source_player {
        if target_flags.contains(MobjFlags::COUNTKILL) {
            state.g_game.player_mut(source_player_id).killcount += 1;
        }
        if let Some(target_player_id) = target_player {
            state.g_game.player_mut(source_player_id).frags[target_player_id.0 as usize] += 1;
        }
    } else if !state.g_game.netgame && target_flags.contains(MobjFlags::COUNTKILL) {
        state.g_game.players[0].killcount += 1;
    }
    if let Some(target_player_id) = target_player {
        if source.is_none() {
            state.g_game.player_mut(target_player_id).frags[target_player_id.0 as usize] += 1;
        }
        state.p_mobj.mo_mut(target).flags &= !MobjFlags::SOLID;
        state.g_game.player_mut(target_player_id).playerstate = PlayerState::Dead;
        drop_weapon(state, target_player_id);
        if target_player_id.0 as i32 == state.g_game.consoleplayer && state.am_map.automapactive {
            am_stop(state);
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
    if target_health < -spawnhealth && xdeathstate != StateNum::Null {
        set_mobj_state(state, target, xdeathstate);
    } else {
        set_mobj_state(state, target, deathstate);
    }
    state.p_mobj.mo_mut(target).tics -= p_random(&mut state.m_random) & 3;
    if state.p_mobj.mo(target).tics < 1 {
        state.p_mobj.mo_mut(target).tics = 1;
    }
    if state.doomstat.gameversion == GameVersion::Chex {
        return;
    }
    let item = match target_type as u32 {
        23 | 1 => MobjType::Clip,
        2 => MobjType::Shotgun,
        10 => MobjType::Chaingun,
        _ => return,
    };
    let (target_x, target_y) = {
        let t = state.p_mobj.mo(target);
        (t.x, t.y)
    };
    let mo = spawn_mobj(state, target_x, target_y, ONFLOORZ, item);
    state.p_mobj.mo_mut(mo).flags |= MobjFlags::DROPPED;
}
pub fn damage_mobj(
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
    if !target_flags.contains(MobjFlags::SHOOTABLE) {
        return;
    }
    if target_health <= 0 {
        return;
    }
    if target_flags.contains(MobjFlags::SKULLFLY) {
        let t = state.p_mobj.mo_mut(target);
        t.momz = 0;
        t.momy = t.momz;
        t.momx = t.momy;
    }
    let target_player_id = state.p_mobj.mo(target).player;
    if target_player_id.is_some() && state.g_game.gameskill == SkillType::Baby {
        damage >>= 1;
    }
    let source_player = source.and_then(|id| state.p_mobj.mo(id).player);
    let source_uses_chainsaw = source_player.is_some_and(|source_player_id| {
        state.g_game.players[source_player_id.0 as usize].readyweapon as u32
            == WeaponType::Chainsaw as i32 as u32
    });
    if let Some(inflictor) = inflictor {
        if !target_flags.contains(MobjFlags::NOCLIP) && !source_uses_chainsaw {
            let (inflictor_x, inflictor_y, inflictor_z) = {
                let i = state.p_mobj.mo(inflictor);
                (i.x, i.y, i.z)
            };
            let (target_x, target_y, target_z, target_type) = {
                let t = state.p_mobj.mo(target);
                (t.x, t.y, t.z, t.kind)
            };
            let mut ang: u32 = point_to_angle2(state, inflictor_x, inflictor_y, target_x, target_y);
            let mut thrust: Fixed = (damage * (FRACUNIT >> 3) * 100
                / state.info.mobjinfo_mut(target_type).mass)
                as Fixed;
            if damage < 40
                && damage > target_health
                && target_z - inflictor_z > 64 * FRACUNIT
                && p_random(&mut state.m_random) & 1 != 0
            {
                ang = ang.wrapping_add(ANG180);
                thrust *= 4;
            }
            ang >>= ANGLETOFINESHIFT;
            let t = state.p_mobj.mo_mut(target);
            t.momx += fixed_mul(thrust, FINECOSINE[ang as usize]);
            t.momy += fixed_mul(thrust, FINESINE[ang as usize]);
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
            && (player.cheats.contains(CheatFlags::GODMODE)
                || player.powers[PowerType::Invulnerability as usize] != 0)
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
            tactile();
        }
    }
    state.p_mobj.mo_mut(target).health -= damage;
    if state.p_mobj.mo(target).health <= 0 {
        kill_mobj(state, source, target);
        return;
    }
    let target_type = state.p_mobj.mo(target).kind;
    if p_random(&mut state.m_random) < state.info.mobjinfo_mut(target_type).painchance
        && !state.p_mobj.mo(target).flags.contains(MobjFlags::SKULLFLY)
    {
        state.p_mobj.mo_mut(target).flags |= MobjFlags::JUSTHIT;
        let painstate = state.info.mobjinfo_mut(target_type).painstate;
        set_mobj_state(state, target, painstate);
    }
    state.p_mobj.mo_mut(target).reactiontime = 0;
    if (state.p_mobj.mo(target).threshold == 0
        || target_type as u32 == MobjType::Vile as i32 as u32)
        && source.is_some_and(|source| {
            source != target && state.p_mobj.mo(source).kind as u32 != MobjType::Vile as i32 as u32
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
            && seestate != StateNum::Null
        {
            set_mobj_state(state, target, seestate);
        }
    }
}
