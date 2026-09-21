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
use crate::p_mobj::SpriteNum;
use crate::p_mobj::StateNum;
use crate::p_mobj::ONFLOORZ;
use crate::p_pspr::drop_weapon;
use crate::r_main::point_to_angle2;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;
use crate::tables::fine_cosine;
use crate::tables::fine_sine;
use crate::tables::Angle;
use crate::tables::ANG180;

pub const NUMCARDS: usize = 6;
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
    let gaveammo: bool = WEAPONINFO[weapon].ammo != AmmoType::Noammo
        && give_ammo(
            &mut state.game.g_game,
            player,
            WEAPONINFO[weapon].ammo,
            if dropped { 1 } else { 2 },
        );
    let gaveweapon: bool = !state.game.g_game.players[player].weaponowned[weapon];
    if gaveweapon {
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
    let player_mo = player.mobj();
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
        let player_mo = g_game.players[player].mobj();
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
/// The key (and the message for a key the player did not have yet) that a sprite stands for.
const fn key_pickup(sprite: SpriteNum) -> Option<(CardType, &'static str)> {
    Some(match sprite {
        SpriteNum::Bkey => (CardType::Bluecard, "Picked up a blue keycard."),
        SpriteNum::Ykey => (CardType::Yellowcard, "Picked up a yellow keycard."),
        SpriteNum::Rkey => (CardType::Redcard, "Picked up a red keycard."),
        SpriteNum::Bsku => (CardType::Blueskull, "Picked up a blue skull key."),
        SpriteNum::Ysku => (CardType::Yellowskull, "Picked up a yellow skull key."),
        SpriteNum::Rsku => (CardType::Redskull, "Picked up a red skull key."),
        _ => return None,
    })
}

/// The ammo, the amount (in clips or boxes) and the message of an ammo pickup. A clip dropped by
/// a monster gives half a clip, which is what `give_ammo` makes of an amount of 0.
const fn ammo_pickup(sprite: SpriteNum, dropped: bool) -> Option<(AmmoType, i32, &'static str)> {
    Some(match sprite {
        SpriteNum::Clip => (
            AmmoType::Clip,
            if dropped { 0 } else { 1 },
            "Picked up a clip.",
        ),
        SpriteNum::Ammo => (AmmoType::Clip, 5, "Picked up a box of bullets."),
        SpriteNum::Rock => (AmmoType::Misl, 1, "Picked up a rocket."),
        SpriteNum::Brok => (AmmoType::Misl, 5, "Picked up a box of rockets."),
        SpriteNum::Cell => (AmmoType::Cell, 1, "Picked up an energy cell."),
        SpriteNum::Celp => (AmmoType::Cell, 5, "Picked up an energy cell pack."),
        SpriteNum::Shel => (AmmoType::Shell, 1, "Picked up 4 shotgun shells."),
        SpriteNum::Sbox => (AmmoType::Shell, 5, "Picked up a box of shotgun shells."),
        _ => return None,
    })
}

/// The power and the message of a power-up.
const fn power_pickup(sprite: SpriteNum) -> Option<(PowerType, &'static str)> {
    Some(match sprite {
        SpriteNum::Pinv => (PowerType::Invulnerability, "Invulnerability!"),
        SpriteNum::Pstr => (PowerType::Strength, "Berserk!"),
        SpriteNum::Pins => (PowerType::Invisibility, "Partial Invisibility"),
        SpriteNum::Suit => (PowerType::Ironfeet, "Radiation Shielding Suit"),
        SpriteNum::Pmap => (PowerType::Allmap, "Computer Area Map"),
        SpriteNum::Pvis => (PowerType::Infrared, "Light Amplification Visor"),
        _ => return None,
    })
}

/// The weapon and the message of a weapon pickup, and whether a weapon dropped by a monster is
/// worth less ammo than one lying on the level.
const fn weapon_pickup(sprite: SpriteNum) -> Option<(WeaponType, bool, &'static str)> {
    Some(match sprite {
        SpriteNum::Bfug => (WeaponType::Bfg, false, "You got the BFG9000!  Oh, yes."),
        SpriteNum::Mgun => (WeaponType::Chaingun, true, "You got the chaingun!"),
        SpriteNum::Csaw => (WeaponType::Chainsaw, false, "A chainsaw!  Find some meat!"),
        SpriteNum::Laun => (WeaponType::Missile, false, "You got the rocket launcher!"),
        SpriteNum::Plas => (WeaponType::Plasma, false, "You got the plasma gun!"),
        SpriteNum::Shot => (WeaponType::Shotgun, true, "You got the shotgun!"),
        SpriteNum::Sgn2 => (WeaponType::Supershotgun, true, "You got the super shotgun!"),
        _ => return None,
    })
}

fn set_message(state: &mut GameState, player: PlayerId, message: &str) {
    state.game.g_game.players[player].message = Some(message.to_string());
}

/// Applies a pickup to the player. `None` means the thing is left where it lies (the player could
/// not use it, or is in a netgame where keys stay); `Some(sound)` means it was taken.
fn pick_up(
    state: &mut GameState,
    special: MobjId,
    toucher: MobjId,
    player: PlayerId,
) -> Option<SfxName> {
    let sprite = state.world.p_mobj.mo(special).sprite;
    let dropped = state
        .world
        .p_mobj
        .mo(special)
        .flags
        .contains(MobjFlags::DROPPED);
    if let Some((card, message)) = key_pickup(sprite) {
        if !state.game.g_game.players[player].cards[card] {
            set_message(state, player, message);
        }
        give_card(&mut state.game.g_game.players[player], card);
        return (!state.game.g_game.netgame).then_some(SfxName::Itemup);
    }
    if let Some((ammo, amount, message)) = ammo_pickup(sprite, dropped) {
        if !give_ammo(&mut state.game.g_game, player, ammo, amount) {
            return None;
        }
        set_message(state, player, message);
        return Some(SfxName::Itemup);
    }
    if let Some((power, message)) = power_pickup(sprite) {
        if !give_power(
            &mut state.game.g_game,
            &mut state.world.p_mobj,
            player,
            power,
        ) {
            return None;
        }
        set_message(state, player, message);
        let p = &mut state.game.g_game.players[player];
        if power == PowerType::Strength && p.readyweapon != WeaponType::Fist {
            p.pendingweapon = WeaponType::Fist;
        }
        return Some(SfxName::Getpow);
    }
    if let Some((weapon, dropped_matters, message)) = weapon_pickup(sprite) {
        if !give_weapon(state, player, weapon, dropped_matters && dropped) {
            return None;
        }
        set_message(state, player, message);
        return Some(SfxName::Wpnup);
    }
    match sprite {
        SpriteNum::Arm1 | SpriteNum::Arm2 => {
            let class = if sprite == SpriteNum::Arm1 {
                DEH_GREEN_ARMOR_CLASS
            } else {
                DEH_BLUE_ARMOR_CLASS
            };
            if !give_armor(&mut state.game.g_game.players[player], class) {
                return None;
            }
            let message = if sprite == SpriteNum::Arm1 {
                "Picked up the armor."
            } else {
                "Picked up the MegaArmor!"
            };
            set_message(state, player, message);
            Some(SfxName::Itemup)
        }
        SpriteNum::Bon1 => {
            let p = &mut state.game.g_game.players[player];
            p.health = (p.health + 1).min(DEH_MAX_HEALTH);
            let health = p.health;
            state.world.p_mobj.mo_mut(toucher).health = health;
            set_message(state, player, "Picked up a health bonus.");
            Some(SfxName::Itemup)
        }
        SpriteNum::Bon2 => {
            let p = &mut state.game.g_game.players[player];
            p.armorpoints = (p.armorpoints + 1).min(DEH_MAX_ARMOR);
            if p.armortype == 0 {
                p.armortype = 1;
            }
            set_message(state, player, "Picked up an armor bonus.");
            Some(SfxName::Itemup)
        }
        SpriteNum::Soul => {
            let p = &mut state.game.g_game.players[player];
            p.health = (p.health + DEH_SOULSPHERE_HEALTH).min(DEH_MAX_SOULSPHERE);
            let health = p.health;
            state.world.p_mobj.mo_mut(toucher).health = health;
            set_message(state, player, "Supercharge!");
            Some(SfxName::Getpow)
        }
        SpriteNum::Mega => {
            if state.game.doomstat.gamemode != GameMode::Commercial {
                return None;
            }
            let p = &mut state.game.g_game.players[player];
            p.health = DEH_MEGASPHERE_HEALTH;
            state.world.p_mobj.mo_mut(toucher).health = DEH_MEGASPHERE_HEALTH;
            give_armor(&mut state.game.g_game.players[player], 2);
            set_message(state, player, "MegaSphere!");
            Some(SfxName::Getpow)
        }
        SpriteNum::Stim => {
            if !give_body(&mut state.game.g_game, &mut state.world.p_mobj, player, 10) {
                return None;
            }
            set_message(state, player, "Picked up a stimpack.");
            Some(SfxName::Itemup)
        }
        SpriteNum::Medi => {
            if !give_body(&mut state.game.g_game, &mut state.world.p_mobj, player, 25) {
                return None;
            }
            let message = if state.game.g_game.players[player].health < 25 {
                "Picked up a medikit that you REALLY need!"
            } else {
                "Picked up a medikit."
            };
            set_message(state, player, message);
            Some(SfxName::Itemup)
        }
        SpriteNum::Bpak => {
            let p = &mut state.game.g_game.players[player];
            if !p.backpack {
                for i in 0..NUMAMMO {
                    p.maxammo[i] *= 2;
                }
                p.backpack = true;
            }
            for i in 0..NUMAMMO {
                give_ammo(
                    &mut state.game.g_game,
                    player,
                    ammotype_from_raw(i as i32),
                    1,
                );
            }
            set_message(state, player, "Picked up a backpack full of ammo!");
            Some(SfxName::Itemup)
        }
        _ => error("P_SpecialThing: Unknown gettable thing"),
    }
}

pub fn touch_special_thing(state: &mut GameState, special: MobjId, toucher: MobjId) {
    let delta: Fixed = state.world.p_mobj.mo(special).z - state.world.p_mobj.mo(toucher).z;
    if delta > state.world.p_mobj.mo(toucher).height || delta < -8 * FRACUNIT {
        return;
    }
    let player = state
        .world
        .p_mobj
        .mo(toucher)
        .player
        .expect("only players touch specials");
    if state.world.p_mobj.mo(toucher).health <= 0 {
        return;
    }
    let Some(sound) = pick_up(state, special, toucher, player) else {
        return;
    };
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
        if t.kind as u32 != (MobjType::Skull as i32).cast_unsigned() {
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
    let mo = spawn_mobj(state, target_x, target_y, Fixed(ONFLOORZ), item);
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
        t.momz = Fixed::ZERO;
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
            let mut ang: Angle = point_to_angle2(inflictor_x, inflictor_y, target_x, target_y);
            let mut thrust: Fixed =
                damage * (FRACUNIT >> 3) * 100 / state.assets.info.mobjinfo_mut(target_type).mass;
            if damage < 40
                && damage > target_health
                && target_z - inflictor_z > 64 * FRACUNIT
                && p_random(&mut state.world.m_random) & 1 != 0
            {
                ang += ANG180;
                thrust *= 4;
            }
            let ang = ang.fine();
            let t = state.world.p_mobj.mo_mut(target);
            t.momx += fixed_mul(thrust, fine_cosine(ang));
            t.momy += fixed_mul(thrust, fine_sine(ang));
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
        || target_type as u32 == (MobjType::Vile as i32).cast_unsigned())
        && source.is_some_and(|source| {
            source != target
                && state.world.p_mobj.mo(source).kind as u32
                    != (MobjType::Vile as i32).cast_unsigned()
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
