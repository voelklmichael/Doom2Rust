use crate::d_ticcmd::TicCmd;
use crate::enum_array::{ArrayIndex, EnumArray};
use crate::m_fixed::Fixed;
use crate::p_inter::CardType;
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
impl ArrayIndex for AmmoType {
    #[inline(always)]
    fn slot(self) -> usize {
        self as usize
    }
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
impl PowerType {
    /// Every power-up, in slot order.
    pub const ALL: [Self; 6] = [
        Self::Invulnerability,
        Self::Strength,
        Self::Invisibility,
        Self::Ironfeet,
        Self::Allmap,
        Self::Infrared,
    ];
}

impl ArrayIndex for PowerType {
    #[inline(always)]
    fn slot(self) -> usize {
        self as usize
    }
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
impl ArrayIndex for WeaponType {
    #[inline(always)]
    fn slot(self) -> usize {
        self as usize
    }
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

impl PlayerId {
    /// Every player slot, in order.
    pub fn all() -> impl Iterator<Item = Self> {
        (0..4).map(Self)
    }

    /// The slot number as an array index.
    pub const fn slot(self) -> usize {
        self.0 as usize
    }

    /// The slot number where the C code used a plain `int`.
    pub const fn as_i32(self) -> i32 {
        self.0 as i32
    }

    /// The next slot, wrapping from the last player back to the first.
    pub const fn next_wrapping(self) -> Self {
        Self((self.0 + 1) % 4)
    }
}

/// One `T` per player slot, indexed by [`PlayerId`] (or by a plain slot number in the loops
/// that walk every slot).
pub type PerPlayer<T> = EnumArray<PlayerId, T, 4>;

impl ArrayIndex for PlayerId {
    #[inline(always)]
    fn slot(self) -> usize {
        usize::from(self.0)
    }
}

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
    pub powers: EnumArray<PowerType, i32, 6>,
    pub cards: EnumArray<CardType, bool, 6>,
    pub backpack: bool,
    pub frags: [i32; 4],
    pub readyweapon: WeaponType,
    pub pendingweapon: WeaponType,
    pub weaponowned: EnumArray<WeaponType, bool, 9>,
    pub ammo: EnumArray<AmmoType, i32, 4>,
    pub maxammo: EnumArray<AmmoType, i32, 4>,
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
