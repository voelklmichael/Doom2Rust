use crate::src::d_player::ammotype_t;
use crate::src::info::{
    S_BFG, S_BFG1, S_BFGDOWN, S_BFGFLASH1, S_BFGUP, S_CHAIN, S_CHAIN1, S_CHAINDOWN, S_CHAINFLASH1,
    S_CHAINUP, S_DSGUN, S_DSGUN1, S_DSGUNDOWN, S_DSGUNFLASH1, S_DSGUNUP, S_MISSILE, S_MISSILE1,
    S_MISSILEDOWN, S_MISSILEFLASH1, S_MISSILEUP, S_NULL, S_PISTOL, S_PISTOL1, S_PISTOLDOWN,
    S_PISTOLFLASH, S_PISTOLUP, S_PLASMA, S_PLASMA1, S_PLASMADOWN, S_PLASMAFLASH1, S_PLASMAUP,
    S_PUNCH, S_PUNCH1, S_PUNCHDOWN, S_PUNCHUP, S_SAW, S_SAW1, S_SAWDOWN, S_SAWUP, S_SGUN, S_SGUN1,
    S_SGUNDOWN, S_SGUNFLASH1, S_SGUNUP,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct weaponinfo_t {
    pub ammo: ammotype_t,
    pub upstate: i32,
    pub downstate: i32,
    pub readystate: i32,
    pub atkstate: i32,
    pub flashstate: i32,
}
#[no_mangle]
pub static weaponinfo: [weaponinfo_t; 9] = [
    weaponinfo_t {
        ammo: ammotype_t::am_noammo,
        upstate: S_PUNCHUP as i32,
        downstate: S_PUNCHDOWN as i32,
        readystate: S_PUNCH as i32,
        atkstate: S_PUNCH1 as i32,
        flashstate: S_NULL as i32,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_clip,
        upstate: S_PISTOLUP as i32,
        downstate: S_PISTOLDOWN as i32,
        readystate: S_PISTOL as i32,
        atkstate: S_PISTOL1 as i32,
        flashstate: S_PISTOLFLASH as i32,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_shell,
        upstate: S_SGUNUP as i32,
        downstate: S_SGUNDOWN as i32,
        readystate: S_SGUN as i32,
        atkstate: S_SGUN1 as i32,
        flashstate: S_SGUNFLASH1 as i32,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_clip,
        upstate: S_CHAINUP as i32,
        downstate: S_CHAINDOWN as i32,
        readystate: S_CHAIN as i32,
        atkstate: S_CHAIN1 as i32,
        flashstate: S_CHAINFLASH1 as i32,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_misl,
        upstate: S_MISSILEUP as i32,
        downstate: S_MISSILEDOWN as i32,
        readystate: S_MISSILE as i32,
        atkstate: S_MISSILE1 as i32,
        flashstate: S_MISSILEFLASH1 as i32,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_cell,
        upstate: S_PLASMAUP as i32,
        downstate: S_PLASMADOWN as i32,
        readystate: S_PLASMA as i32,
        atkstate: S_PLASMA1 as i32,
        flashstate: S_PLASMAFLASH1 as i32,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_cell,
        upstate: S_BFGUP as i32,
        downstate: S_BFGDOWN as i32,
        readystate: S_BFG as i32,
        atkstate: S_BFG1 as i32,
        flashstate: S_BFGFLASH1 as i32,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_noammo,
        upstate: S_SAWUP as i32,
        downstate: S_SAWDOWN as i32,
        readystate: S_SAW as i32,
        atkstate: S_SAW1 as i32,
        flashstate: S_NULL as i32,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_shell,
        upstate: S_DSGUNUP as i32,
        downstate: S_DSGUNDOWN as i32,
        readystate: S_DSGUN as i32,
        atkstate: S_DSGUN1 as i32,
        flashstate: S_DSGUNFLASH1 as i32,
    },
];
