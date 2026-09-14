use crate::d_player::ammotype_t;
use crate::p_mobj::StateNum;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct weaponinfo_t {
    pub ammo: ammotype_t,
    pub upstate: StateNum,
    pub downstate: StateNum,
    pub readystate: StateNum,
    pub atkstate: StateNum,
    pub flashstate: StateNum,
}
#[no_mangle]
pub static weaponinfo: [weaponinfo_t; 9] = [
    weaponinfo_t {
        ammo: ammotype_t::am_noammo,
        upstate: StateNum::S_PUNCHUP,
        downstate: StateNum::S_PUNCHDOWN,
        readystate: StateNum::S_PUNCH,
        atkstate: StateNum::S_PUNCH1,
        flashstate: StateNum::S_NULL,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_clip,
        upstate: StateNum::S_PISTOLUP,
        downstate: StateNum::S_PISTOLDOWN,
        readystate: StateNum::S_PISTOL,
        atkstate: StateNum::S_PISTOL1,
        flashstate: StateNum::S_PISTOLFLASH,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_shell,
        upstate: StateNum::S_SGUNUP,
        downstate: StateNum::S_SGUNDOWN,
        readystate: StateNum::S_SGUN,
        atkstate: StateNum::S_SGUN1,
        flashstate: StateNum::S_SGUNFLASH1,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_clip,
        upstate: StateNum::S_CHAINUP,
        downstate: StateNum::S_CHAINDOWN,
        readystate: StateNum::S_CHAIN,
        atkstate: StateNum::S_CHAIN1,
        flashstate: StateNum::S_CHAINFLASH1,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_misl,
        upstate: StateNum::S_MISSILEUP,
        downstate: StateNum::S_MISSILEDOWN,
        readystate: StateNum::S_MISSILE,
        atkstate: StateNum::S_MISSILE1,
        flashstate: StateNum::S_MISSILEFLASH1,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_cell,
        upstate: StateNum::S_PLASMAUP,
        downstate: StateNum::S_PLASMADOWN,
        readystate: StateNum::S_PLASMA,
        atkstate: StateNum::S_PLASMA1,
        flashstate: StateNum::S_PLASMAFLASH1,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_cell,
        upstate: StateNum::S_BFGUP,
        downstate: StateNum::S_BFGDOWN,
        readystate: StateNum::S_BFG,
        atkstate: StateNum::S_BFG1,
        flashstate: StateNum::S_BFGFLASH1,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_noammo,
        upstate: StateNum::S_SAWUP,
        downstate: StateNum::S_SAWDOWN,
        readystate: StateNum::S_SAW,
        atkstate: StateNum::S_SAW1,
        flashstate: StateNum::S_NULL,
    },
    weaponinfo_t {
        ammo: ammotype_t::am_shell,
        upstate: StateNum::S_DSGUNUP,
        downstate: StateNum::S_DSGUNDOWN,
        readystate: StateNum::S_DSGUN,
        atkstate: StateNum::S_DSGUN1,
        flashstate: StateNum::S_DSGUNFLASH1,
    },
];
