use crate::am_map::am_stop;
use crate::d_items::WEAPONINFO;
use crate::d_mode::SkillType;
use crate::d_mode::{GameMode, GameVersion};
use crate::d_player::CheatFlags;
use crate::d_player::PowerType;
use crate::d_player::WeaponType;
use crate::enum_array::{ArrayIndex, EnumArray};
use crate::g_game::GGameState;
use crate::p_mobj::MobjFlags;
use crate::p_mobj::PMobjState;

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
    Bluecard,
    Yellowcard,
    Redcard,
    Blueskull,
    Yellowskull,
    Redskull,
}
impl ArrayIndex for CardType {
    #[inline(always)]
    fn slot(self) -> usize {
        self as usize
    }
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
pub static MAXAMMO: EnumArray<AmmoType, i32, 4> = EnumArray::new([200, 50, 300, 50]);
pub static CLIPAMMO: EnumArray<AmmoType, i32, 4> = EnumArray::new([10, 4, 20, 1]);
pub fn give_ammo(
    g_game: &mut GGameState,
    player_id: PlayerId,
    ammo: AmmoType,
    mut num: i32,
) -> bool {
    let player = &mut g_game.players[player_id];

    if ammo == AmmoType::Noammo {
        return false;
    }
    if ammo as u32 > NUMAMMO as u32 {
        error(&format!("P_GiveAmmo: bad type {}", ammo as u32));
    }
    if player.ammo[ammo] == player.maxammo[ammo] {
        return false;
    }
    if num != 0 {
        num *= CLIPAMMO[ammo];
    } else {
        num = CLIPAMMO[ammo] / 2;
    }
    if g_game.gameskill == SkillType::Baby || g_game.gameskill == SkillType::Nightmare {
        num <<= 1;
    }
    let oldammo: i32 = player.ammo[ammo];
    player.ammo[ammo] += num;
    if player.ammo[ammo] > player.maxammo[ammo] {
        player.ammo[ammo] = player.maxammo[ammo];
    }
    if oldammo != 0 {
        return true;
    }
    match ammo as u32 {
        0 => {
            if player.readyweapon == WeaponType::Fist {
                if player.weaponowned[WeaponType::Chaingun] {
                    player.pendingweapon = WeaponType::Chaingun;
                } else {
                    player.pendingweapon = WeaponType::Pistol;
                }
            }
        }
        1 => {
            if (player.readyweapon == WeaponType::Fist || player.readyweapon == WeaponType::Pistol)
                && player.weaponowned[WeaponType::Shotgun]
            {
                player.pendingweapon = WeaponType::Shotgun;
            }
        }
        2 => {
            if (player.readyweapon == WeaponType::Fist || player.readyweapon == WeaponType::Pistol)
                && player.weaponowned[WeaponType::Plasma]
            {
                player.pendingweapon = WeaponType::Plasma;
            }
        }
        3 if player.readyweapon == WeaponType::Fist && player.weaponowned[WeaponType::Missile] => {
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
    if state.game.g_game.netgame && state.game.g_game.deathmatch != 2 && !dropped {
        if state.game.g_game.players[player].weaponowned[weapon] {
            return false;
        }
        state.game.g_game.players[player].bonuscount += BONUSADD;
        state.game.g_game.players[player].weaponowned[weapon] = true;
        if state.game.g_game.deathmatch != 0 {
            give_ammo(&mut state.game.g_game, player, WEAPONINFO[weapon].ammo, 5);
        } else {
            give_ammo(&mut state.game.g_game, player, WEAPONINFO[weapon].ammo, 2);
        }
        state.game.g_game.players[player].pendingweapon = weapon;
        if player == state.game.g_game.consoleplayer {
            s_start_sound(state, SoundOrigin::None, SfxName::Wpnup);
        }
        return false;
    }
    if WEAPONINFO[weapon].ammo == AmmoType::Noammo {
        gaveammo = false;
    } else {
        if dropped {
            gaveammo = give_ammo(&mut state.game.g_game, player, WEAPONINFO[weapon].ammo, 1);
        } else {
            gaveammo = give_ammo(&mut state.game.g_game, player, WEAPONINFO[weapon].ammo, 2);
        }
    }
    if state.game.g_game.players[player].weaponowned[weapon] {
        gaveweapon = false;
    } else {
        gaveweapon = true;
        state.game.g_game.players[player].weaponowned[weapon] = true;
        state.game.g_game.players[player].pendingweapon = weapon;
    }
    gaveweapon || gaveammo
}
pub fn give_body(
    g_game: &mut GGameState,
    p_mobj: &mut PMobjState,
    player_id: PlayerId,
    num: i32,
) -> bool {
    let player = &mut g_game.players[player_id];
    if player.health >= MAXHEALTH {
        return false;
    }
    player.health += num;
    if player.health > MAXHEALTH {
        player.health = MAXHEALTH;
    }
    let player_mo = player.mo.unwrap();
    p_mobj.mo_mut(player_mo).health = player.health;
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
    if player.cards[card] {
        return;
    }
    player.bonuscount = BONUSADD;
    player.cards[card] = true;
}
pub fn give_power(
    g_game: &mut GGameState,
    p_mobj: &mut PMobjState,
    player: PlayerId,
    power: PowerType,
) -> bool {
    if power == PowerType::Invulnerability {
        g_game.players[player].powers[power] = INVULNTICS;
        return true;
    }
    if power == PowerType::Invisibility {
        g_game.players[player].powers[power] = INVISTICS;
        let player_mo = g_game.players[player].mo.unwrap();
        p_mobj.mo_mut(player_mo).flags |= MobjFlags::SHADOW;
        return true;
    }
    if power == PowerType::Infrared {
        g_game.players[player].powers[power] = INFRATICS;
        return true;
    }
    if power == PowerType::Ironfeet {
        g_game.players[player].powers[power] = IRONTICS;
        return true;
    }
    if power == PowerType::Strength {
        give_body(g_game, p_mobj, player, 100);
        g_game.players[player].powers[power] = 1;
        return true;
    }
    if g_game.players[player].powers[power] != 0 {
        return false;
    }
    g_game.players[player].powers[power] = 1;
    true
}
pub fn touch_special_thing(state: &mut GameState, special: MobjId, toucher: MobjId) {
    let delta: Fixed = state.world.p_mobj.mo(special).z - state.world.p_mobj.mo(toucher).z;
    if delta > state.world.p_mobj.mo(toucher).height || delta < -8 * FRACUNIT {
        return;
    }
    let mut sound: SfxName = SfxName::Itemup;
    let player = state.world.p_mobj.mo(toucher).player.unwrap();
    if state.world.p_mobj.mo(toucher).health <= 0 {
        return;
    }
    match state.world.p_mobj.mo(special).sprite as u32 {
        55 => {
            if !give_armor(
                &mut state.game.g_game.players[player],
                DEH_GREEN_ARMOR_CLASS,
            ) {
                return;
            }
            state.game.g_game.players[player].message = Some("Picked up the armor.".to_string());
        }
        56 => {
            if !give_armor(&mut state.game.g_game.players[player], DEH_BLUE_ARMOR_CLASS) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("Picked up the MegaArmor!".to_string());
        }
        60 => {
            state.game.g_game.players[player].health += 1;
            if state.game.g_game.players[player].health > DEH_MAX_HEALTH {
                state.game.g_game.players[player].health = DEH_MAX_HEALTH;
            }
            state.world.p_mobj.mo_mut(toucher).health = state.game.g_game.players[player].health;
            state.game.g_game.players[player].message =
                Some("Picked up a health bonus.".to_string());
        }
        61 => {
            state.game.g_game.players[player].armorpoints += 1;
            if state.game.g_game.players[player].armorpoints > DEH_MAX_ARMOR {
                state.game.g_game.players[player].armorpoints = DEH_MAX_ARMOR;
            }
            if state.game.g_game.players[player].armortype == 0 {
                state.game.g_game.players[player].armortype = 1;
            }
            state.game.g_game.players[player].message =
                Some("Picked up an armor bonus.".to_string());
        }
        70 => {
            state.game.g_game.players[player].health += DEH_SOULSPHERE_HEALTH;
            if state.game.g_game.players[player].health > DEH_MAX_SOULSPHERE {
                state.game.g_game.players[player].health = DEH_MAX_SOULSPHERE;
            }
            state.world.p_mobj.mo_mut(toucher).health = state.game.g_game.players[player].health;
            state.game.g_game.players[player].message = Some("Supercharge!".to_string());
            sound = SfxName::Getpow;
        }
        74 => {
            if state.game.doomstat.gamemode != GameMode::Commercial {
                return;
            }
            state.game.g_game.players[player].health = DEH_MEGASPHERE_HEALTH;
            state.world.p_mobj.mo_mut(toucher).health = state.game.g_game.players[player].health;
            give_armor(&mut state.game.g_game.players[player], 2);
            state.game.g_game.players[player].message = Some("MegaSphere!".to_string());
            sound = SfxName::Getpow;
        }
        62 => {
            if !state.game.g_game.players[player].cards[CardType::Bluecard] {
                state.game.g_game.players[player].message =
                    Some("Picked up a blue keycard.".to_string());
            }
            give_card(&mut state.game.g_game.players[player], CardType::Bluecard);
            if state.game.g_game.netgame {
                return;
            }
        }
        64 => {
            if !state.game.g_game.players[player].cards[CardType::Yellowcard] {
                state.game.g_game.players[player].message =
                    Some("Picked up a yellow keycard.".to_string());
            }
            give_card(&mut state.game.g_game.players[player], CardType::Yellowcard);
            if state.game.g_game.netgame {
                return;
            }
        }
        63 => {
            if !state.game.g_game.players[player].cards[CardType::Redcard] {
                state.game.g_game.players[player].message =
                    Some("Picked up a red keycard.".to_string());
            }
            give_card(&mut state.game.g_game.players[player], CardType::Redcard);
            if state.game.g_game.netgame {
                return;
            }
        }
        65 => {
            if !state.game.g_game.players[player].cards[CardType::Blueskull] {
                state.game.g_game.players[player].message =
                    Some("Picked up a blue skull key.".to_string());
            }
            give_card(&mut state.game.g_game.players[player], CardType::Blueskull);
            if state.game.g_game.netgame {
                return;
            }
        }
        67 => {
            if !state.game.g_game.players[player].cards[CardType::Yellowskull] {
                state.game.g_game.players[player].message =
                    Some("Picked up a yellow skull key.".to_string());
            }
            give_card(
                &mut state.game.g_game.players[player],
                CardType::Yellowskull,
            );
            if state.game.g_game.netgame {
                return;
            }
        }
        66 => {
            if !state.game.g_game.players[player].cards[CardType::Redskull] {
                state.game.g_game.players[player].message =
                    Some("Picked up a red skull key.".to_string());
            }
            give_card(&mut state.game.g_game.players[player], CardType::Redskull);
            if state.game.g_game.netgame {
                return;
            }
        }
        68 => {
            if !give_body(&mut state.game.g_game, &mut state.world.p_mobj, player, 10) {
                return;
            }
            state.game.g_game.players[player].message = Some("Picked up a stimpack.".to_string());
        }
        69 => {
            if !give_body(&mut state.game.g_game, &mut state.world.p_mobj, player, 25) {
                return;
            }
            if state.game.g_game.players[player].health < 25 {
                state.game.g_game.players[player].message =
                    Some("Picked up a medikit that you REALLY need!".to_string());
            } else {
                state.game.g_game.players[player].message =
                    Some("Picked up a medikit.".to_string());
            }
        }
        71 => {
            if !give_power(
                &mut state.game.g_game,
                &mut state.world.p_mobj,
                player,
                PowerType::Invulnerability,
            ) {
                return;
            }
            state.game.g_game.players[player].message = Some("Invulnerability!".to_string());
            sound = SfxName::Getpow;
        }
        72 => {
            if !give_power(
                &mut state.game.g_game,
                &mut state.world.p_mobj,
                player,
                PowerType::Strength,
            ) {
                return;
            }
            state.game.g_game.players[player].message = Some("Berserk!".to_string());
            if state.game.g_game.players[player].readyweapon != WeaponType::Fist {
                state.game.g_game.players[player].pendingweapon = WeaponType::Fist;
            }
            sound = SfxName::Getpow;
        }
        73 => {
            if !give_power(
                &mut state.game.g_game,
                &mut state.world.p_mobj,
                player,
                PowerType::Invisibility,
            ) {
                return;
            }
            state.game.g_game.players[player].message = Some("Partial Invisibility".to_string());
            sound = SfxName::Getpow;
        }
        75 => {
            if !give_power(
                &mut state.game.g_game,
                &mut state.world.p_mobj,
                player,
                PowerType::Ironfeet,
            ) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("Radiation Shielding Suit".to_string());
            sound = SfxName::Getpow;
        }
        76 => {
            if !give_power(
                &mut state.game.g_game,
                &mut state.world.p_mobj,
                player,
                PowerType::Allmap,
            ) {
                return;
            }
            state.game.g_game.players[player].message = Some("Computer Area Map".to_string());
            sound = SfxName::Getpow;
        }
        77 => {
            if !give_power(
                &mut state.game.g_game,
                &mut state.world.p_mobj,
                player,
                PowerType::Infrared,
            ) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("Light Amplification Visor".to_string());
            sound = SfxName::Getpow;
        }
        78 => {
            if state
                .world
                .p_mobj
                .mo(special)
                .flags
                .contains(MobjFlags::DROPPED)
            {
                if !give_ammo(&mut state.game.g_game, player, AmmoType::Clip, 0) {
                    return;
                }
            } else if !give_ammo(&mut state.game.g_game, player, AmmoType::Clip, 1) {
                return;
            }
            state.game.g_game.players[player].message = Some("Picked up a clip.".to_string());
        }
        79 => {
            if !give_ammo(&mut state.game.g_game, player, AmmoType::Clip, 5) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("Picked up a box of bullets.".to_string());
        }
        80 => {
            if !give_ammo(&mut state.game.g_game, player, AmmoType::Misl, 1) {
                return;
            }
            state.game.g_game.players[player].message = Some("Picked up a rocket.".to_string());
        }
        81 => {
            if !give_ammo(&mut state.game.g_game, player, AmmoType::Misl, 5) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("Picked up a box of rockets.".to_string());
        }
        82 => {
            if !give_ammo(&mut state.game.g_game, player, AmmoType::Cell, 1) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("Picked up an energy cell.".to_string());
        }
        83 => {
            if !give_ammo(&mut state.game.g_game, player, AmmoType::Cell, 5) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("Picked up an energy cell pack.".to_string());
        }
        84 => {
            if !give_ammo(&mut state.game.g_game, player, AmmoType::Shell, 1) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("Picked up 4 shotgun shells.".to_string());
        }
        85 => {
            if !give_ammo(&mut state.game.g_game, player, AmmoType::Shell, 5) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("Picked up a box of shotgun shells.".to_string());
        }
        86 => {
            if !state.game.g_game.players[player].backpack {
                for i in 0..(NUMAMMO as usize) {
                    state.game.g_game.players[player].maxammo[i] *= 2;
                }
                state.game.g_game.players[player].backpack = true;
            }
            for i in 0..NUMAMMO {
                give_ammo(&mut state.game.g_game, player, ammotype_from_raw(i), 1);
            }
            state.game.g_game.players[player].message =
                Some("Picked up a backpack full of ammo!".to_string());
        }
        87 => {
            if !give_weapon(state, player, WeaponType::Bfg, false) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("You got the BFG9000!  Oh, yes.".to_string());
            sound = SfxName::Wpnup;
        }
        88 => {
            if !give_weapon(
                state,
                player,
                WeaponType::Chaingun,
                state
                    .world
                    .p_mobj
                    .mo(special)
                    .flags
                    .contains(MobjFlags::DROPPED),
            ) {
                return;
            }
            state.game.g_game.players[player].message = Some("You got the chaingun!".to_string());
            sound = SfxName::Wpnup;
        }
        89 => {
            if !give_weapon(state, player, WeaponType::Chainsaw, false) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("A chainsaw!  Find some meat!".to_string());
            sound = SfxName::Wpnup;
        }
        90 => {
            if !give_weapon(state, player, WeaponType::Missile, false) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("You got the rocket launcher!".to_string());
            sound = SfxName::Wpnup;
        }
        91 => {
            if !give_weapon(state, player, WeaponType::Plasma, false) {
                return;
            }
            state.game.g_game.players[player].message = Some("You got the plasma gun!".to_string());
            sound = SfxName::Wpnup;
        }
        92 => {
            if !give_weapon(
                state,
                player,
                WeaponType::Shotgun,
                state
                    .world
                    .p_mobj
                    .mo(special)
                    .flags
                    .contains(MobjFlags::DROPPED),
            ) {
                return;
            }
            state.game.g_game.players[player].message = Some("You got the shotgun!".to_string());
            sound = SfxName::Wpnup;
        }
        93 => {
            if !give_weapon(
                state,
                player,
                WeaponType::Supershotgun,
                state
                    .world
                    .p_mobj
                    .mo(special)
                    .flags
                    .contains(MobjFlags::DROPPED),
            ) {
                return;
            }
            state.game.g_game.players[player].message =
                Some("You got the super shotgun!".to_string());
            sound = SfxName::Wpnup;
        }
        _ => {
            error("P_SpecialThing: Unknown gettable thing");
        }
    }
    if state
        .world
        .p_mobj
        .mo(special)
        .flags
        .contains(MobjFlags::COUNTITEM)
    {
        state.game.g_game.players[player].itemcount += 1;
    }
    remove_mobj(state, special);
    state.game.g_game.players[player].bonuscount += BONUSADD;
    if player == state.game.g_game.consoleplayer {
        s_start_sound(state, SoundOrigin::None, sound);
    }
}
pub fn kill_mobj(state: &mut GameState, source: Option<MobjId>, target: MobjId) {
    {
        let t = state.world.p_mobj.mo_mut(target);
        t.flags &= !(MobjFlags::SHOOTABLE | MobjFlags::FLOAT | MobjFlags::SKULLFLY);
        if t.kind as u32 != MobjType::Skull as i32 as u32 {
            t.flags &= !MobjFlags::NOGRAVITY;
        }
        t.flags |= MobjFlags::CORPSE | MobjFlags::DROPOFF;
        t.height >>= 2;
    }
    let source_player = source.and_then(|id| state.world.p_mobj.mo(id).player);
    let (target_flags, target_player) = {
        let t = state.world.p_mobj.mo(target);
        (t.flags, t.player)
    };
    if let Some(source_player_id) = source_player {
        if target_flags.contains(MobjFlags::COUNTKILL) {
            state.game.g_game.player_mut(source_player_id).killcount += 1;
        }
        if let Some(target_player_id) = target_player {
            state.game.g_game.player_mut(source_player_id).frags[target_player_id.0 as usize] += 1;
        }
    } else if !state.game.g_game.netgame && target_flags.contains(MobjFlags::COUNTKILL) {
        state.game.g_game.players[0].killcount += 1;
    }
    if let Some(target_player_id) = target_player {
        if source.is_none() {
            state.game.g_game.player_mut(target_player_id).frags[target_player_id.0 as usize] += 1;
        }
        state.world.p_mobj.mo_mut(target).flags &= !MobjFlags::SOLID;
        state.game.g_game.player_mut(target_player_id).playerstate = PlayerState::Dead;
        drop_weapon(state, target_player_id);
        if target_player_id == state.game.g_game.consoleplayer && state.ui.am_map.automapactive {
            am_stop(state);
        }
    }
    let (target_type, target_health) = {
        let t = state.world.p_mobj.mo(target);
        (t.kind, t.health)
    };
    let (spawnhealth, xdeathstate, deathstate) = {
        let info = state.assets.info.mobjinfo_mut(target_type);
        (info.spawnhealth, info.xdeathstate, info.deathstate)
    };
    if target_health < -spawnhealth && xdeathstate != StateNum::Null {
        set_mobj_state(state, target, xdeathstate);
    } else {
        set_mobj_state(state, target, deathstate);
    }
    state.world.p_mobj.mo_mut(target).tics -= p_random(&mut state.world.m_random) & 3;
    if state.world.p_mobj.mo(target).tics < 1 {
        state.world.p_mobj.mo_mut(target).tics = 1;
    }
    if state.game.doomstat.gameversion == GameVersion::Chex {
        return;
    }
    let item = match target_type as u32 {
        23 | 1 => MobjType::Clip,
        2 => MobjType::Shotgun,
        10 => MobjType::Chaingun,
        _ => return,
    };
    let (target_x, target_y) = {
        let t = state.world.p_mobj.mo(target);
        (t.x, t.y)
    };
    let mo = spawn_mobj(state, target_x, target_y, ONFLOORZ, item);
    state.world.p_mobj.mo_mut(mo).flags |= MobjFlags::DROPPED;
}
pub fn damage_mobj(
    state: &mut GameState,
    target: MobjId,
    inflictor: Option<MobjId>,
    source: Option<MobjId>,
    mut damage: i32,
) {
    let (target_flags, target_health) = {
        let t = state.world.p_mobj.mo(target);
        (t.flags, t.health)
    };
    if !target_flags.contains(MobjFlags::SHOOTABLE) {
        return;
    }
    if target_health <= 0 {
        return;
    }
    if target_flags.contains(MobjFlags::SKULLFLY) {
        let t = state.world.p_mobj.mo_mut(target);
        t.momz = 0;
        t.momy = t.momz;
        t.momx = t.momy;
    }
    let target_player_id = state.world.p_mobj.mo(target).player;
    if target_player_id.is_some() && state.game.g_game.gameskill == SkillType::Baby {
        damage >>= 1;
    }
    let source_player = source.and_then(|id| state.world.p_mobj.mo(id).player);
    let source_uses_chainsaw = source_player.is_some_and(|source_player_id| {
        state.game.g_game.players[source_player_id].readyweapon == WeaponType::Chainsaw
    });
    if let Some(inflictor) = inflictor {
        if !target_flags.contains(MobjFlags::NOCLIP) && !source_uses_chainsaw {
            let (inflictor_x, inflictor_y, inflictor_z) = {
                let i = state.world.p_mobj.mo(inflictor);
                (i.x, i.y, i.z)
            };
            let (target_x, target_y, target_z, target_type) = {
                let t = state.world.p_mobj.mo(target);
                (t.x, t.y, t.z, t.kind)
            };
            let mut ang: u32 = point_to_angle2(inflictor_x, inflictor_y, target_x, target_y);
            let mut thrust: Fixed = (damage * (FRACUNIT >> 3) * 100
                / state.assets.info.mobjinfo_mut(target_type).mass)
                as Fixed;
            if damage < 40
                && damage > target_health
                && target_z - inflictor_z > 64 * FRACUNIT
                && p_random(&mut state.world.m_random) & 1 != 0
            {
                ang = ang.wrapping_add(ANG180);
                thrust *= 4;
            }
            ang >>= ANGLETOFINESHIFT;
            let t = state.world.p_mobj.mo_mut(target);
            t.momx += fixed_mul(thrust, FINECOSINE[ang as usize]);
            t.momy += fixed_mul(thrust, FINESINE[ang as usize]);
        }
    }
    if let Some(player_id) = target_player_id {
        let target_subsector = state.world.p_mobj.mo(target).subsector;
        let sector_special = state
            .world
            .p_setup
            .sector_mut(state.world.p_setup.subsectors[target_subsector.0 as usize].sector)
            .special;
        if i32::from(sector_special) == 11 && damage >= target_health {
            damage = target_health - 1;
        }
        let player = state.game.g_game.player_mut(player_id);
        if damage < 1000
            && (player.cheats.contains(CheatFlags::GODMODE)
                || player.powers[PowerType::Invulnerability] != 0)
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
        if target_player_id == Some(state.game.g_game.consoleplayer) {
            tactile();
        }
    }
    state.world.p_mobj.mo_mut(target).health -= damage;
    if state.world.p_mobj.mo(target).health <= 0 {
        kill_mobj(state, source, target);
        return;
    }
    let target_type = state.world.p_mobj.mo(target).kind;
    if p_random(&mut state.world.m_random) < state.assets.info.mobjinfo_mut(target_type).painchance
        && !state
            .world
            .p_mobj
            .mo(target)
            .flags
            .contains(MobjFlags::SKULLFLY)
    {
        state.world.p_mobj.mo_mut(target).flags |= MobjFlags::JUSTHIT;
        let painstate = state.assets.info.mobjinfo_mut(target_type).painstate;
        set_mobj_state(state, target, painstate);
    }
    state.world.p_mobj.mo_mut(target).reactiontime = 0;
    if (state.world.p_mobj.mo(target).threshold == 0
        || target_type as u32 == MobjType::Vile as i32 as u32)
        && source.is_some_and(|source| {
            source != target
                && state.world.p_mobj.mo(source).kind as u32 != MobjType::Vile as i32 as u32
        })
    {
        {
            let t = state.world.p_mobj.mo_mut(target);
            t.target = source;
            t.threshold = BASETHRESHOLD;
        }
        let (spawnstate, seestate) = {
            let info = state.assets.info.mobjinfo_mut(target_type);
            (info.spawnstate, info.seestate)
        };
        if state.world.p_mobj.mo(target).state == Some(StateId(spawnstate as u32))
            && seestate != StateNum::Null
        {
            set_mobj_state(state, target, seestate);
        }
    }
}
