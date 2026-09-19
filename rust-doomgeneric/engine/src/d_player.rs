use crate::d_ticcmd::TicCmd;
use crate::m_fixed::Fixed;
use crate::p_mobj::{MobjId, PspDef};
use alloc::string::String;
pub const NUMAMMO: i32 = 4;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AmmoType {
    Clip = 0,
    Shell = 1,
    Cell = 2,
    Misl = 3,
    Noammo = 5,
}
pub fn ammotype_from_raw(v: i32) -> AmmoType {
    match v {
        0 => AmmoType::Clip,
        1 => AmmoType::Shell,
        2 => AmmoType::Cell,
        3 => AmmoType::Misl,
        5 => AmmoType::Noammo,
        n => panic!("invalid ammotype {n}"),
    }
}
pub const NUMPSPRITES: i32 = 2;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PSpriteNum {
    Weapon = 0,
    Flash = 1,
}
// CheatFlags::NOCLIP/CheatFlags::GODMODE/CheatFlags::NOMOMENTUM are bit flags (1/2/4) combined with
// bitwise OR/AND/XOR into a single `cheats` field, not mutually-exclusive
// enum variants - not a candidate for enum conversion.
pub const NUMPOWERS: i32 = 6;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PowerType {
    Invulnerability = 0,
    Strength = 1,
    Invisibility = 2,
    Ironfeet = 3,
    Allmap = 4,
    Infrared = 5,
}

pub const NUMWEAPONS: i32 = 9;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WeaponType {
    Fist = 0,
    Pistol = 1,
    Shotgun = 2,
    Chaingun = 3,
    Missile = 4,
    Plasma = 5,
    Bfg = 6,
    Chainsaw = 7,
    Supershotgun = 8,
    Nochange = 10,
}
pub fn weapontype_from_raw(v: i32) -> WeaponType {
    match v {
        0 => WeaponType::Fist,
        1 => WeaponType::Pistol,
        2 => WeaponType::Shotgun,
        3 => WeaponType::Chaingun,
        4 => WeaponType::Missile,
        5 => WeaponType::Plasma,
        6 => WeaponType::Bfg,
        7 => WeaponType::Chainsaw,
        8 => WeaponType::Supershotgun,
        10 => WeaponType::Nochange,
        n => panic!("invalid weapontype {n}"),
    }
}

bitflags::bitflags! {
    /// A player's active cheats (`CF_*` in the C source).
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
    pub struct CheatFlags: i32 {
        const NOCLIP = 1;
        const GODMODE = 2;
        const NOMOMENTUM = 4;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PlayerId(pub u8);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlayerState {
    Live = 0,
    Dead = 1,
    Reborn = 2,
}

#[derive(Clone)]
pub struct Player {
    pub mo: Option<MobjId>,
    pub playerstate: PlayerState,
    pub cmd: TicCmd,
    pub viewz: Fixed,
    pub viewheight: Fixed,
    pub deltaviewheight: Fixed,
    pub bob: Fixed,
    pub health: i32,
    pub armorpoints: i32,
    pub armortype: i32,
    pub powers: [i32; 6],
    pub cards: [bool; 6],
    pub backpack: bool,
    pub frags: [i32; 4],
    pub readyweapon: WeaponType,
    pub pendingweapon: WeaponType,
    pub weaponowned: [bool; 9],
    pub ammo: [i32; 4],
    pub maxammo: [i32; 4],
    pub attackdown: bool,
    pub usedown: bool,
    pub cheats: CheatFlags,
    pub refire: i32,
    pub killcount: i32,
    pub itemcount: i32,
    pub secretcount: i32,
    pub message: Option<String>,
    pub damagecount: i32,
    pub bonuscount: i32,
    pub attacker: Option<MobjId>,
    pub extralight: i32,
    pub fixedcolormap: i32,
    pub colormap: i32,
    pub psprites: [PspDef; 2],
    pub didsecret: bool,
}
