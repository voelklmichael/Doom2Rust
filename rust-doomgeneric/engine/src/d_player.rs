use crate::src::d_ticcmd::ticcmd_t;
use crate::src::m_fixed::fixed_t;
use crate::src::p_mobj::{pspdef_t, MobjId};
pub const NUMAMMO: i32 = 4;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ammotype_t {
    am_clip = 0,
    am_shell = 1,
    am_cell = 2,
    am_misl = 3,
    am_noammo = 5,
}
pub fn ammotype_from_raw(v: i32) -> ammotype_t {
    match v {
        0 => ammotype_t::am_clip,
        1 => ammotype_t::am_shell,
        2 => ammotype_t::am_cell,
        3 => ammotype_t::am_misl,
        5 => ammotype_t::am_noammo,
        n => panic!("invalid ammotype {n}"),
    }
}
pub const NUMPSPRITES: i32 = 2;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PSpriteNum {
    ps_weapon = 0,
    ps_flash = 1,
}
// CF_NOCLIP/CF_GODMODE/CF_NOMOMENTUM are bit flags (1/2/4) combined with
// bitwise OR/AND/XOR into a single `cheats` field, not mutually-exclusive
// enum variants - not a candidate for enum conversion.
pub const CF_NOCLIP: i32 = 1;
pub const CF_GODMODE: i32 = 2;
pub const CF_NOMOMENTUM: i32 = 4;
pub const NUMPOWERS: i32 = 6;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PowerType {
    pw_invulnerability = 0,
    pw_strength = 1,
    pw_invisibility = 2,
    pw_ironfeet = 3,
    pw_allmap = 4,
    pw_infrared = 5,
}

pub const NUMWEAPONS: i32 = 9;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum weapontype_t {
    wp_fist = 0,
    wp_pistol = 1,
    wp_shotgun = 2,
    wp_chaingun = 3,
    wp_missile = 4,
    wp_plasma = 5,
    wp_bfg = 6,
    wp_chainsaw = 7,
    wp_supershotgun = 8,
    wp_nochange = 10,
}
pub fn weapontype_from_raw(v: i32) -> weapontype_t {
    match v {
        0 => weapontype_t::wp_fist,
        1 => weapontype_t::wp_pistol,
        2 => weapontype_t::wp_shotgun,
        3 => weapontype_t::wp_chaingun,
        4 => weapontype_t::wp_missile,
        5 => weapontype_t::wp_plasma,
        6 => weapontype_t::wp_bfg,
        7 => weapontype_t::wp_chainsaw,
        8 => weapontype_t::wp_supershotgun,
        10 => weapontype_t::wp_nochange,
        n => panic!("invalid weapontype {n}"),
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PlayerId(pub u8);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlayerState {
    PST_LIVE = 0,
    PST_DEAD = 1,
    PST_REBORN = 2,
}

#[derive(Clone)]
#[repr(C)]
pub struct player_s {
    pub mo: Option<MobjId>,
    pub playerstate: PlayerState,
    pub cmd: ticcmd_t,
    pub viewz: fixed_t,
    pub viewheight: fixed_t,
    pub deltaviewheight: fixed_t,
    pub bob: fixed_t,
    pub health: i32,
    pub armorpoints: i32,
    pub armortype: i32,
    pub powers: [i32; 6],
    pub cards: [bool; 6],
    pub backpack: bool,
    pub frags: [i32; 4],
    pub readyweapon: weapontype_t,
    pub pendingweapon: weapontype_t,
    pub weaponowned: [bool; 9],
    pub ammo: [i32; 4],
    pub maxammo: [i32; 4],
    pub attackdown: i32,
    pub usedown: i32,
    pub cheats: i32,
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
    pub psprites: [pspdef_t; 2],
    pub didsecret: bool,
}
pub type player_t = player_s;
