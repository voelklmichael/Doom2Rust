use crate::src::d_ticcmd::ticcmd_t;
use crate::src::m_fixed::fixed_t;
use crate::src::p_mobj::{mobj_t, pspdef_t, MobjId};
pub type ammotype_t = u32;
pub const am_noammo: ammotype_t = 5;
pub const NUMAMMO: ammotype_t = 4;
pub const am_misl: ammotype_t = 3;
pub const am_cell: ammotype_t = 2;
pub const am_shell: ammotype_t = 1;
pub const am_clip: ammotype_t = 0;
pub type C2RustUnnamed_2 = u32;
pub const NUMPSPRITES: C2RustUnnamed_2 = 2;
pub const ps_flash: C2RustUnnamed_2 = 1;
pub const ps_weapon: C2RustUnnamed_2 = 0;
pub type C2RustUnnamed_3 = u32;
pub const CF_NOMOMENTUM: C2RustUnnamed_3 = 4;
pub const CF_GODMODE: C2RustUnnamed_3 = 2;
pub const CF_NOCLIP: C2RustUnnamed_3 = 1;
pub type C2RustUnnamed = u32;
pub const NUMPOWERS: C2RustUnnamed = 6;
pub const pw_infrared: C2RustUnnamed = 5;
pub const pw_allmap: C2RustUnnamed = 4;
pub const pw_ironfeet: C2RustUnnamed = 3;
pub const pw_invisibility: C2RustUnnamed = 2;
pub const pw_strength: C2RustUnnamed = 1;
pub const pw_invulnerability: C2RustUnnamed = 0;

pub type weapontype_t = u32;
pub const wp_nochange: weapontype_t = 10;
pub const NUMWEAPONS: weapontype_t = 9;
pub const wp_supershotgun: weapontype_t = 8;
pub const wp_chainsaw: weapontype_t = 7;
pub const wp_bfg: weapontype_t = 6;
pub const wp_plasma: weapontype_t = 5;
pub const wp_missile: weapontype_t = 4;
pub const wp_chaingun: weapontype_t = 3;
pub const wp_shotgun: weapontype_t = 2;
pub const wp_pistol: weapontype_t = 1;
pub const wp_fist: weapontype_t = 0;

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
    pub mo: *mut mobj_t,
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
