use crate::d_mode::skill_from_raw;
use crate::d_player::NUMPOWERS;
use crate::d_player::NUMPSPRITES;
use crate::d_player::{player_t, PlayerId, PlayerState};
use crate::d_player::{weapontype_from_raw, NUMWEAPONS};
use crate::d_ticcmd::ticcmd_t;
use crate::g_game::G_VanillaVersionCode;
use crate::i_system::I_Error;
use crate::info::StateId;
use crate::m_fixed::fixed_t;
use crate::p_ceilng::CeilingE;
use crate::p_ceilng::P_AddActiveCeiling;
use crate::p_doors::vldoor_t;
use crate::p_doors::VldoorE;
use crate::p_floor::FloorE;
use crate::p_lights::{glow_t, lightflash_t, strobe_t};
use crate::p_maputl::P_SetThingPosition;
use crate::p_mobj::mobjtype_from_raw;
use crate::p_mobj::spritenum_from_raw;
use crate::p_mobj::P_RemoveMobj;
use crate::p_mobj::{mapthing_t, thinker_t, SectorSpecial, ThinkerFn};
use crate::p_mobj::{mobj_t, pspdef_t};
use crate::p_plats::P_AddActivePlat;
use crate::p_plats::PlatE;
use crate::p_plats::PlattypeE;
use crate::p_setup::SectorId;
use crate::p_setup::SideId;
use crate::p_setup::SubsectorId;
use crate::p_spec::{ceiling_t, floormove_t, plat_t};
use crate::p_tick::P_AddThinker;
use crate::p_tick::P_InitThinkers;
use crate::p_tick::P_ThinkerFunction;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::stdint_types::byte;
use crate::tables::angle_t;
use std::io::{Read, Seek, Write};

use crate::d_player::NUMAMMO;
use crate::doomdef::MAXPLAYERS;
use crate::game_state::GameState;
use crate::m_fixed::FRACBITS;
use crate::m_menu::SAVESTRINGSIZE;
use crate::p_ceilng::T_MoveCeiling;
use crate::p_ceilng::MAXCEILINGS;
use crate::p_doors::T_VerticalDoor;
use crate::p_floor::T_MoveFloor;
use crate::p_inter::NUMCARDS;
use crate::p_lights::{T_Glow, T_LightFlash, T_StrobeFlash};
use crate::p_mobj::P_MobjThinker;
use crate::p_plats::T_PlatRaise;

pub struct PSavegState {
    pub save_stream: Option<std::fs::File>,
    pub savegame_error: bool,
    pub temp_savegame_filename: Option<String>,
}

impl PSavegState {
    pub const fn new() -> Self {
        PSavegState {
            save_stream: None,
            savegame_error: false,
            temp_savegame_filename: None,
        }
    }
}

pub type intptr_t = isize;
pub const tc_end: C2RustUnnamed_4 = 0;
pub const tc_mobj: C2RustUnnamed_4 = 1;
pub const tc_endspecials: C2RustUnnamed_5 = 7;
pub const tc_glow: C2RustUnnamed_5 = 6;
pub const tc_strobe: C2RustUnnamed_5 = 5;
pub const tc_flash: C2RustUnnamed_5 = 4;
pub const tc_plat: C2RustUnnamed_5 = 3;
pub const tc_floor: C2RustUnnamed_5 = 2;
pub const tc_door: C2RustUnnamed_5 = 1;
pub const tc_ceiling: C2RustUnnamed_5 = 0;
pub type C2RustUnnamed_4 = u32;
pub type C2RustUnnamed_5 = u32;
pub const SAVEGAME_EOF: i32 = 0x1d;
pub const VERSIONSIZE: i32 = 16;
pub static savegamelength: i32 = 0;
pub fn P_TempSaveGameFile(state: &mut GameState) -> String {
    if state.p_saveg.temp_savegame_filename.is_none() {
        state.p_saveg.temp_savegame_filename =
            Some(format!("{}temp.dsg", state.d_main.savegamedir));
    }
    state.p_saveg.temp_savegame_filename.clone().unwrap()
}
pub fn P_SaveGameFile(state: &mut GameState, slot: i32) -> String {
    format!("{}doomsav{}.dsg", state.d_main.savegamedir, slot)
}
fn saveg_read8(state: &mut PSavegState) -> byte {
    let mut result: [byte; 1] = [0];
    if state.save_stream.as_mut().unwrap().read(&mut result).unwrap_or(0) < 1
        && !state.savegame_error
    {
        eprintln!("saveg_read8: Unexpected end of file while reading save game");
        state.savegame_error = true;
    }
    result[0]
}
fn saveg_write8(state: &mut PSavegState, value: byte) {
    if state.save_stream.as_mut().unwrap().write(&[value]).unwrap_or(0) < 1
        && !state.savegame_error
    {
        eprintln!("saveg_write8: Error while writing save game");
        state.savegame_error = true;
    }
}
fn saveg_read16(state: &mut PSavegState) -> i16 {
    let mut result: i32 = 0;
    result = saveg_read8(state) as i32;
    result |= (saveg_read8(state) as i32) << 8_i32;
    result as i16
}
fn saveg_write16(state: &mut PSavegState, mut value: i16) {
    saveg_write8(state, (value as i32 & 0xff_i32) as byte);
    saveg_write8(state, (value as i32 >> 8_i32 & 0xff_i32) as byte);
}
fn saveg_read32(state: &mut PSavegState) -> i32 {
    let mut result: i32 = 0;
    result = saveg_read8(state) as i32;
    result |= (saveg_read8(state) as i32) << 8_i32;
    result |= (saveg_read8(state) as i32) << 16_i32;
    result |= (saveg_read8(state) as i32) << 24_i32;
    result
}
fn saveg_write32(state: &mut PSavegState, mut value: i32) {
    saveg_write8(state, (value & 0xff_i32) as byte);
    saveg_write8(state, (value >> 8_i32 & 0xff_i32) as byte);
    saveg_write8(state, (value >> 16_i32 & 0xff_i32) as byte);
    saveg_write8(state, (value >> 24_i32 & 0xff_i32) as byte);
}
fn saveg_read_pad(state: &mut PSavegState) {
    let mut padding: i32 = 0;
    let mut i: i32 = 0;
    let pos = state
        .save_stream
        .as_mut()
        .unwrap()
        .stream_position()
        .unwrap_or(0);
    padding = (4_u64.wrapping_sub(pos & 3_u64) & 3_u64) as i32;
    i = 0_i32;
    while i < padding {
        saveg_read8(state);
        i += 1;
    }
}
fn saveg_write_pad(state: &mut PSavegState) {
    let mut padding: i32 = 0;
    let mut i: i32 = 0;
    let pos = state
        .save_stream
        .as_mut()
        .unwrap()
        .stream_position()
        .unwrap_or(0);
    padding = (4_u64.wrapping_sub(pos & 3_u64) & 3_u64) as i32;
    i = 0_i32;
    while i < padding {
        saveg_write8(state, 0 as byte);
        i += 1;
    }
}
fn saveg_readp(state: &mut PSavegState) -> *mut ::core::ffi::c_void {
    saveg_read32(state) as intptr_t as *mut ::core::ffi::c_void
}
fn saveg_writep(state: &mut PSavegState, mut p: *mut ::core::ffi::c_void) {
    saveg_write32(state, p as intptr_t as i32);
}
fn saveg_read_mapthing_t(state: &mut PSavegState, str: &mut mapthing_t) {
    str.x = saveg_read16(state);
    str.y = saveg_read16(state);
    str.angle = saveg_read16(state);
    str.type_0 = saveg_read16(state);
    str.options = saveg_read16(state);
}
fn saveg_write_mapthing_t(state: &mut PSavegState, str: &mut mapthing_t) {
    saveg_write16(state, str.x);
    saveg_write16(state, str.y);
    saveg_write16(state, str.angle);
    saveg_write16(state, str.type_0);
    saveg_write16(state, str.options);
}
fn saveg_read_actionf_t(state: &mut PSavegState, str: &mut ThinkerFn) {
    let word = saveg_readp(state);
    *str = if word.is_null() {
        ThinkerFn::Paused
    } else {
        ThinkerFn::Unresolved
    };
}
fn saveg_write_actionf_t(state: &mut PSavegState, str: &mut ThinkerFn) {
    let word: *mut ::core::ffi::c_void = if matches!(*str, ThinkerFn::Paused) {
        ::core::ptr::null_mut()
    } else {
        str as *mut ThinkerFn as *mut ::core::ffi::c_void
    };
    saveg_writep(state, word);
}
fn saveg_read_thinker_t(state: &mut PSavegState, str: &mut thinker_t) {
    // P_AddThinker (called after every payload type is reconstructed, in
    // both P_UnArchiveThinkers and P_UnArchiveSpecials below) always rebuilds
    // prev/next from scratch in PTickState's own node table, so these
    // on-disk bytes are already dead -- discard.
    saveg_read32(state);
    saveg_read32(state);
    saveg_read_actionf_t(state, &mut str.function);
}
fn saveg_write_thinker_t(state: &mut PSavegState, str: &mut thinker_t) {
    saveg_write32(state, 0);
    saveg_write32(state, 0);
    saveg_write_actionf_t(state, &mut str.function);
}
fn saveg_read_mobj_t(state: &mut PSavegState, str: &mut mobj_t) {
    let mut pl: i32 = 0;
    saveg_read_thinker_t(state, &mut str.thinker);
    str.x = saveg_read32(state) as fixed_t;
    str.y = saveg_read32(state) as fixed_t;
    str.z = saveg_read32(state) as fixed_t;
    // P_SetThingPosition (called on every reconstructed mobj right after this,
    // see P_UnArchiveThinkers) fully rebuilds snext/sprev from scratch, so
    // these on-disk bytes are already dead -- discard, same treatment as
    // target/tracer's inert reads just below.
    saveg_read32(state);
    str.snext = None;
    saveg_read32(state);
    str.sprev = None;
    str.angle = saveg_read32(state) as angle_t;
    str.sprite = spritenum_from_raw(saveg_read32(state));
    str.frame = saveg_read32(state);
    // P_SetThingPosition also fully rebuilds bnext/bprev from scratch --
    // same dead-bytes treatment as snext/sprev above.
    saveg_read32(state);
    str.bnext = None;
    saveg_read32(state);
    str.bprev = None;
    saveg_read32(state);
    str.subsector = SubsectorId(0);
    str.floorz = saveg_read32(state) as fixed_t;
    str.ceilingz = saveg_read32(state) as fixed_t;
    str.radius = saveg_read32(state) as fixed_t;
    str.height = saveg_read32(state) as fixed_t;
    str.momx = saveg_read32(state) as fixed_t;
    str.momy = saveg_read32(state) as fixed_t;
    str.momz = saveg_read32(state) as fixed_t;
    str.validcount = saveg_read32(state);
    str.type_0 = mobjtype_from_raw(saveg_read32(state));
    saveg_read32(state);
    str.tics = saveg_read32(state);
    str.state = Some(StateId(saveg_read32(state) as u32));
    str.flags = saveg_read32(state);
    str.health = saveg_read32(state);
    str.movedir = saveg_read32(state);
    str.movecount = saveg_read32(state);
    saveg_read32(state);
    str.target = None;
    str.reactiontime = saveg_read32(state);
    str.threshold = saveg_read32(state);
    pl = saveg_read32(state);
    if pl > 0_i32 {
        let player_id = PlayerId((pl - 1_i32) as u8);
        str.player = Some(player_id);
    } else {
        str.player = None;
    }
    str.lastlook = saveg_read32(state);
    saveg_read_mapthing_t(state, &mut str.spawnpoint);
    saveg_read32(state);
    str.tracer = None;
}
fn saveg_write_mobj_t(state: &mut PSavegState, str: &mut mobj_t) {
    saveg_write_thinker_t(state, &mut str.thinker);
    saveg_write32(state, str.x);
    saveg_write32(state, str.y);
    saveg_write32(state, str.z);
    saveg_write32(state, 0);
    saveg_write32(state, 0);
    saveg_write32(state, str.angle as i32);
    saveg_write32(state, str.sprite as i32);
    saveg_write32(state, str.frame);
    saveg_write32(state, 0);
    saveg_write32(state, 0);
    saveg_write32(state, 0);
    saveg_write32(state, str.floorz);
    saveg_write32(state, str.ceilingz);
    saveg_write32(state, str.radius);
    saveg_write32(state, str.height);
    saveg_write32(state, str.momx);
    saveg_write32(state, str.momy);
    saveg_write32(state, str.momz);
    saveg_write32(state, str.validcount);
    saveg_write32(state, str.type_0 as i32);
    saveg_write32(state, 0);
    saveg_write32(state, str.tics);
    saveg_write32(state, str.state.unwrap().0 as i32);
    saveg_write32(state, str.flags);
    saveg_write32(state, str.health);
    saveg_write32(state, str.movedir);
    saveg_write32(state, str.movecount);
    saveg_write32(state, 0_i32);
    saveg_write32(state, str.reactiontime);
    saveg_write32(state, str.threshold);
    if let Some(player_id) = str.player {
        saveg_write32(state, player_id.0 as i32 + 1_i32);
    } else {
        saveg_write32(state, 0_i32);
    }
    saveg_write32(state, str.lastlook);
    saveg_write_mapthing_t(state, &mut str.spawnpoint);
    saveg_write32(state, 0_i32);
}
fn saveg_read_ticcmd_t(state: &mut PSavegState, str: &mut ticcmd_t) {
    str.forwardmove = saveg_read8(state) as i8;
    str.sidemove = saveg_read8(state) as i8;
    str.angleturn = saveg_read16(state);
    str.consistancy = saveg_read16(state) as byte;
    str.chatchar = saveg_read8(state);
    str.buttons = saveg_read8(state);
}
fn saveg_write_ticcmd_t(state: &mut PSavegState, str: &mut ticcmd_t) {
    saveg_write8(state, str.forwardmove as byte);
    saveg_write8(state, str.sidemove as byte);
    saveg_write16(state, str.angleturn);
    saveg_write16(state, str.consistancy as i16);
    saveg_write8(state, str.chatchar);
    saveg_write8(state, str.buttons);
}
fn saveg_read_pspdef_t(state: &mut PSavegState, str: &mut pspdef_t) {
    let mut state_num: i32 = 0;
    state_num = saveg_read32(state);
    if state_num > 0_i32 {
        str.state = Some(StateId(state_num as u32));
    } else {
        str.state = None;
    }
    str.tics = saveg_read32(state);
    str.sx = saveg_read32(state) as fixed_t;
    str.sy = saveg_read32(state) as fixed_t;
}
fn saveg_write_pspdef_t(state: &mut PSavegState, str: &mut pspdef_t) {
    if let Some(state_id) = str.state {
        saveg_write32(state, state_id.0 as i32);
    } else {
        saveg_write32(state, 0_i32);
    }
    saveg_write32(state, str.tics);
    saveg_write32(state, str.sx);
    saveg_write32(state, str.sy);
}
fn saveg_read_player_t(state: &mut PSavegState, str: &mut player_t) {
    let mut i: i32 = 0;
    // Placeholder value, discarded -- see saveg_write_player_t.
    saveg_readp(state);
    str.playerstate = match saveg_read32(state) {
        0 => PlayerState::PST_LIVE,
        1 => PlayerState::PST_DEAD,
        2 => PlayerState::PST_REBORN,
        n => panic!("P_UnArchivePlayers: invalid playerstate {n} in savegame"),
    };
    saveg_read_ticcmd_t(state, &mut str.cmd);
    str.viewz = saveg_read32(state) as fixed_t;
    str.viewheight = saveg_read32(state) as fixed_t;
    str.deltaviewheight = saveg_read32(state) as fixed_t;
    str.bob = saveg_read32(state) as fixed_t;
    str.health = saveg_read32(state);
    str.armorpoints = saveg_read32(state);
    str.armortype = saveg_read32(state);
    i = 0_i32;
    while i < NUMPOWERS {
        str.powers[i as usize] = saveg_read32(state);
        i += 1;
    }
    i = 0_i32;
    while i < NUMCARDS {
        str.cards[i as usize] = saveg_read32(state) != 0;
        i += 1;
    }
    str.backpack = saveg_read32(state) != 0;
    i = 0_i32;
    while i < MAXPLAYERS {
        str.frags[i as usize] = saveg_read32(state);
        i += 1;
    }
    str.readyweapon = weapontype_from_raw(saveg_read32(state));
    str.pendingweapon = weapontype_from_raw(saveg_read32(state));
    i = 0_i32;
    while i < NUMWEAPONS {
        str.weaponowned[i as usize] = saveg_read32(state) != 0;
        i += 1;
    }
    i = 0_i32;
    while i < NUMAMMO {
        str.ammo[i as usize] = saveg_read32(state);
        i += 1;
    }
    i = 0_i32;
    while i < NUMAMMO {
        str.maxammo[i as usize] = saveg_read32(state);
        i += 1;
    }
    str.attackdown = saveg_read32(state);
    str.usedown = saveg_read32(state);
    str.cheats = saveg_read32(state);
    str.refire = saveg_read32(state);
    str.killcount = saveg_read32(state);
    str.itemcount = saveg_read32(state);
    str.secretcount = saveg_read32(state);
    saveg_readp(state);
    str.message = None;
    str.damagecount = saveg_read32(state);
    str.bonuscount = saveg_read32(state);
    saveg_read32(state);
    str.attacker = None;
    str.extralight = saveg_read32(state);
    str.fixedcolormap = saveg_read32(state);
    str.colormap = saveg_read32(state);
    i = 0_i32;
    while i < NUMPSPRITES {
        saveg_read_pspdef_t(state, &mut str.psprites[i as usize]);
        i += 1;
    }
    str.didsecret = saveg_read32(state) != 0;
}
fn saveg_write_player_t(state: &mut PSavegState, str: &mut player_t) {
    let mut i: i32 = 0;
    // The written value is a placeholder: on load it is immediately
    // overwritten with null by P_UnArchivePlayers and then correctly
    // restored from the mobj's own player backref in P_UnArchiveThinkers.
    saveg_writep(state, ::core::ptr::null_mut());
    saveg_write32(state, str.playerstate as i32);
    saveg_write_ticcmd_t(state, &mut str.cmd);
    saveg_write32(state, str.viewz);
    saveg_write32(state, str.viewheight);
    saveg_write32(state, str.deltaviewheight);
    saveg_write32(state, str.bob);
    saveg_write32(state, str.health);
    saveg_write32(state, str.armorpoints);
    saveg_write32(state, str.armortype);
    i = 0_i32;
    while i < NUMPOWERS {
        saveg_write32(state, str.powers[i as usize]);
        i += 1;
    }
    i = 0_i32;
    while i < NUMCARDS {
        saveg_write32(state, str.cards[i as usize] as i32);
        i += 1;
    }
    saveg_write32(state, str.backpack as i32);
    i = 0_i32;
    while i < MAXPLAYERS {
        saveg_write32(state, str.frags[i as usize]);
        i += 1;
    }
    saveg_write32(state, str.readyweapon as i32);
    saveg_write32(state, str.pendingweapon as i32);
    i = 0_i32;
    while i < NUMWEAPONS {
        saveg_write32(state, str.weaponowned[i as usize] as i32);
        i += 1;
    }
    i = 0_i32;
    while i < NUMAMMO {
        saveg_write32(state, str.ammo[i as usize]);
        i += 1;
    }
    i = 0_i32;
    while i < NUMAMMO {
        saveg_write32(state, str.maxammo[i as usize]);
        i += 1;
    }
    saveg_write32(state, str.attackdown);
    saveg_write32(state, str.usedown);
    saveg_write32(state, str.cheats);
    saveg_write32(state, str.refire);
    saveg_write32(state, str.killcount);
    saveg_write32(state, str.itemcount);
    saveg_write32(state, str.secretcount);
    saveg_writep(
        state,
        if str.message.is_some() {
            1 as *mut ::core::ffi::c_void
        } else {
            ::core::ptr::null_mut()
        },
    );
    saveg_write32(state, str.damagecount);
    saveg_write32(state, str.bonuscount);
    saveg_write32(state, 0_i32);
    saveg_write32(state, str.extralight);
    saveg_write32(state, str.fixedcolormap);
    saveg_write32(state, str.colormap);
    i = 0_i32;
    while i < NUMPSPRITES {
        saveg_write_pspdef_t(state, &mut str.psprites[i as usize]);
        i += 1;
    }
    saveg_write32(state, str.didsecret as i32);
}
fn saveg_read_ceiling_e(state: &mut PSavegState) -> CeilingE {
    match saveg_read32(state) {
        0 => CeilingE::lowerToFloor,
        1 => CeilingE::raiseToHighest,
        2 => CeilingE::lowerAndCrush,
        3 => CeilingE::crushAndRaise,
        4 => CeilingE::fastCrushAndRaise,
        5 => CeilingE::silentCrushAndRaise,
        n => panic!("P_UnArchiveSpecials: invalid ceiling type {n} in savegame"),
    }
}
fn saveg_read_ceiling_t(state: &mut PSavegState, str: &mut ceiling_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &mut str.thinker);
    str.type_0 = saveg_read_ceiling_e(state);
    sector = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.bottomheight = saveg_read32(state) as fixed_t;
    str.topheight = saveg_read32(state) as fixed_t;
    str.speed = saveg_read32(state) as fixed_t;
    str.crush = saveg_read32(state) != 0;
    str.direction = saveg_read32(state);
    str.tag = saveg_read32(state);
    str.olddirection = saveg_read32(state);
}
fn saveg_write_ceiling_t(state: &mut PSavegState, str: &mut ceiling_t) {
    saveg_write_thinker_t(state, &mut str.thinker);
    saveg_write32(state, str.type_0 as i32);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.bottomheight);
    saveg_write32(state, str.topheight);
    saveg_write32(state, str.speed);
    saveg_write32(state, str.crush as i32);
    saveg_write32(state, str.direction);
    saveg_write32(state, str.tag);
    saveg_write32(state, str.olddirection);
}
fn saveg_read_vldoor_e(state: &mut PSavegState) -> VldoorE {
    match saveg_read32(state) {
        0 => VldoorE::vld_normal,
        1 => VldoorE::vld_close30ThenOpen,
        2 => VldoorE::vld_close,
        3 => VldoorE::vld_open,
        4 => VldoorE::vld_raiseIn5Mins,
        5 => VldoorE::vld_blazeRaise,
        6 => VldoorE::vld_blazeOpen,
        7 => VldoorE::vld_blazeClose,
        n => panic!("P_UnArchiveSpecials: invalid door type {n} in savegame"),
    }
}
fn saveg_read_vldoor_t(state: &mut PSavegState, str: &mut vldoor_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &mut str.thinker);
    str.type_0 = saveg_read_vldoor_e(state);
    sector = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.topheight = saveg_read32(state) as fixed_t;
    str.speed = saveg_read32(state) as fixed_t;
    str.direction = saveg_read32(state);
    str.topwait = saveg_read32(state);
    str.topcountdown = saveg_read32(state);
}
fn saveg_write_vldoor_t(state: &mut PSavegState, str: &mut vldoor_t) {
    saveg_write_thinker_t(state, &mut str.thinker);
    saveg_write32(state, str.type_0 as i32);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.topheight);
    saveg_write32(state, str.speed);
    saveg_write32(state, str.direction);
    saveg_write32(state, str.topwait);
    saveg_write32(state, str.topcountdown);
}
fn saveg_read_floor_e(state: &mut PSavegState) -> FloorE {
    match saveg_read32(state) {
        0 => FloorE::lowerFloor,
        1 => FloorE::lowerFloorToLowest,
        2 => FloorE::turboLower,
        3 => FloorE::raiseFloor,
        4 => FloorE::raiseFloorToNearest,
        5 => FloorE::raiseToTexture,
        6 => FloorE::lowerAndChange,
        7 => FloorE::raiseFloor24,
        8 => FloorE::raiseFloor24AndChange,
        9 => FloorE::raiseFloorCrush,
        10 => FloorE::raiseFloorTurbo,
        11 => FloorE::donutRaise,
        12 => FloorE::raiseFloor512,
        n => panic!("P_UnArchiveSpecials: invalid floor type {n} in savegame"),
    }
}
fn saveg_read_floormove_t(state: &mut PSavegState, str: &mut floormove_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &mut str.thinker);
    str.type_0 = saveg_read_floor_e(state);
    str.crush = saveg_read32(state) != 0;
    sector = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.direction = saveg_read32(state);
    str.newspecial = saveg_read32(state);
    str.texture = saveg_read16(state);
    str.floordestheight = saveg_read32(state) as fixed_t;
    str.speed = saveg_read32(state) as fixed_t;
}
fn saveg_write_floormove_t(state: &mut PSavegState, str: &mut floormove_t) {
    saveg_write_thinker_t(state, &mut str.thinker);
    saveg_write32(state, str.type_0 as i32);
    saveg_write32(state, str.crush as i32);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.direction);
    saveg_write32(state, str.newspecial);
    saveg_write16(state, str.texture);
    saveg_write32(state, str.floordestheight);
    saveg_write32(state, str.speed);
}
fn saveg_read_plat_e(state: &mut PSavegState) -> PlatE {
    match saveg_read32(state) {
        0 => PlatE::up,
        1 => PlatE::down,
        2 => PlatE::waiting,
        3 => PlatE::in_stasis,
        n => panic!("P_UnArchiveSpecials: invalid plat status {n} in savegame"),
    }
}
fn saveg_read_plattype_e(state: &mut PSavegState) -> PlattypeE {
    match saveg_read32(state) {
        0 => PlattypeE::perpetualRaise,
        1 => PlattypeE::downWaitUpStay,
        2 => PlattypeE::raiseAndChange,
        3 => PlattypeE::raiseToNearestAndChange,
        4 => PlattypeE::blazeDWUS,
        n => panic!("P_UnArchiveSpecials: invalid plat type {n} in savegame"),
    }
}
fn saveg_read_plat_t(state: &mut PSavegState, str: &mut plat_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &mut str.thinker);
    sector = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.speed = saveg_read32(state) as fixed_t;
    str.low = saveg_read32(state) as fixed_t;
    str.high = saveg_read32(state) as fixed_t;
    str.wait = saveg_read32(state);
    str.count = saveg_read32(state);
    str.status = saveg_read_plat_e(state);
    str.oldstatus = saveg_read_plat_e(state);
    str.crush = saveg_read32(state) != 0;
    str.tag = saveg_read32(state);
    str.type_0 = saveg_read_plattype_e(state);
}
fn saveg_write_plat_t(state: &mut PSavegState, str: &mut plat_t) {
    saveg_write_thinker_t(state, &mut str.thinker);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.speed);
    saveg_write32(state, str.low);
    saveg_write32(state, str.high);
    saveg_write32(state, str.wait);
    saveg_write32(state, str.count);
    saveg_write32(state, str.status as i32);
    saveg_write32(state, str.oldstatus as i32);
    saveg_write32(state, str.crush as i32);
    saveg_write32(state, str.tag);
    saveg_write32(state, str.type_0 as i32);
}
fn saveg_read_lightflash_t(state: &mut PSavegState, str: &mut lightflash_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &mut str.thinker);
    sector = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.count = saveg_read32(state);
    str.maxlight = saveg_read32(state);
    str.minlight = saveg_read32(state);
    str.maxtime = saveg_read32(state);
    str.mintime = saveg_read32(state);
}
fn saveg_write_lightflash_t(state: &mut PSavegState, str: &mut lightflash_t) {
    saveg_write_thinker_t(state, &mut str.thinker);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.count);
    saveg_write32(state, str.maxlight);
    saveg_write32(state, str.minlight);
    saveg_write32(state, str.maxtime);
    saveg_write32(state, str.mintime);
}
fn saveg_read_strobe_t(state: &mut PSavegState, str: &mut strobe_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &mut str.thinker);
    sector = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.count = saveg_read32(state);
    str.minlight = saveg_read32(state);
    str.maxlight = saveg_read32(state);
    str.darktime = saveg_read32(state);
    str.brighttime = saveg_read32(state);
}
fn saveg_write_strobe_t(state: &mut PSavegState, str: &mut strobe_t) {
    saveg_write_thinker_t(state, &mut str.thinker);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.count);
    saveg_write32(state, str.minlight);
    saveg_write32(state, str.maxlight);
    saveg_write32(state, str.darktime);
    saveg_write32(state, str.brighttime);
}
fn saveg_read_glow_t(state: &mut PSavegState, str: &mut glow_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &mut str.thinker);
    sector = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.minlight = saveg_read32(state);
    str.maxlight = saveg_read32(state);
    str.direction = saveg_read32(state);
}
fn saveg_write_glow_t(state: &mut PSavegState, str: &mut glow_t) {
    saveg_write_thinker_t(state, &mut str.thinker);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.minlight);
    saveg_write32(state, str.maxlight);
    saveg_write32(state, str.direction);
}
pub fn P_WriteSaveGameHeader(state: &mut GameState, description: &str) {
    let mut i: i32 = 0;
    for &b in description.as_bytes() {
        saveg_write8(&mut state.p_saveg, b);
        i += 1;
    }
    while i < SAVESTRINGSIZE {
        saveg_write8(&mut state.p_saveg, 0 as byte);
        i += 1;
    }
    let name = format!("version {}", G_VanillaVersionCode(&mut state.doomstat));
    let mut name_bytes = [0u8; 16];
    let copy_len = name.len().min(16);
    name_bytes[..copy_len].copy_from_slice(&name.as_bytes()[..copy_len]);
    i = 0_i32;
    while i < VERSIONSIZE {
        saveg_write8(&mut state.p_saveg, name_bytes[i as usize]);
        i += 1;
    }
    saveg_write8(&mut state.p_saveg, state.g_game.gameskill as byte);
    saveg_write8(&mut state.p_saveg, state.g_game.gameepisode as byte);
    saveg_write8(&mut state.p_saveg, state.g_game.gamemap as byte);
    i = 0_i32;
    while i < MAXPLAYERS {
        saveg_write8(&mut state.p_saveg, state.g_game.playeringame[i as usize] as byte);
        i += 1;
    }
    saveg_write8(
        &mut state.p_saveg,
        (state.p_tick.leveltime >> 16_i32 & 0xff_i32) as byte,
    );
    saveg_write8(
        &mut state.p_saveg,
        (state.p_tick.leveltime >> 8_i32 & 0xff_i32) as byte,
    );
    saveg_write8(&mut state.p_saveg, (state.p_tick.leveltime & 0xff_i32) as byte);
}
pub fn P_ReadSaveGameHeader(state: &mut GameState) -> bool {
    let mut i: i32 = 0;
    let mut a: byte = 0;
    let mut b: byte = 0;
    let mut c: byte = 0;
    let mut read_vcheck: [u8; 16] = [0; 16];
    i = 0_i32;
    while i < SAVESTRINGSIZE {
        saveg_read8(&mut state.p_saveg);
        i += 1;
    }
    i = 0_i32;
    while i < VERSIONSIZE {
        read_vcheck[i as usize] = saveg_read8(&mut state.p_saveg);
        i += 1;
    }
    let version_name = format!("version {}", G_VanillaVersionCode(&mut state.doomstat));
    let mut vcheck: [u8; 16] = [0; 16];
    let copy_len = version_name.len().min(16);
    vcheck[..copy_len].copy_from_slice(&version_name.as_bytes()[..copy_len]);
    fn cstr_prefix(buf: &[u8]) -> &[u8] {
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        &buf[..len]
    }
    if cstr_prefix(&read_vcheck) != cstr_prefix(&vcheck) {
        return false;
    }
    state.g_game.gameskill = skill_from_raw(saveg_read8(&mut state.p_saveg) as i32);
    state.g_game.gameepisode = saveg_read8(&mut state.p_saveg) as i32;
    state.g_game.gamemap = saveg_read8(&mut state.p_saveg) as i32;
    i = 0_i32;
    while i < MAXPLAYERS {
        state.g_game.playeringame[i as usize] = saveg_read8(&mut state.p_saveg) != 0;
        i += 1;
    }
    a = saveg_read8(&mut state.p_saveg);
    b = saveg_read8(&mut state.p_saveg);
    c = saveg_read8(&mut state.p_saveg);
    state.p_tick.leveltime = ((a as i32) << 16_i32) + ((b as i32) << 8_i32) + c as i32;
    true
}
pub fn P_ReadSaveGameEOF(state: &mut GameState) -> bool {
    let mut value: i32 = 0;
    value = saveg_read8(&mut state.p_saveg) as i32;
    value == SAVEGAME_EOF
}
pub fn P_WriteSaveGameEOF(state: &mut GameState) {
    saveg_write8(&mut state.p_saveg, SAVEGAME_EOF as byte);
}
pub fn P_ArchivePlayers(state: &mut GameState) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            saveg_write_pad(&mut state.p_saveg);
            saveg_write_player_t(&mut state.p_saveg, &mut state.g_game.players[i as usize]);
        }
        i += 1;
    }
}
pub fn P_UnArchivePlayers(state: &mut GameState) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < MAXPLAYERS {
        if state.g_game.playeringame[i as usize] {
            saveg_read_pad(&mut state.p_saveg);
            saveg_read_player_t(&mut state.p_saveg, &mut state.g_game.players[i as usize]);
            state.g_game.players[i as usize].mo = None;
            state.g_game.players[i as usize].message = None;
            state.g_game.players[i as usize].attacker = None;
        }
        i += 1;
    }
}
pub fn P_ArchiveWorld(state: &mut GameState) {
    for i in 0..state.p_setup.numsectors {
        let sec = state.p_setup.sector_mut(SectorId(i as u32));
        let (floorheight, ceilingheight, floorpic, ceilingpic, lightlevel, special, tag) = (
            sec.floorheight,
            sec.ceilingheight,
            sec.floorpic,
            sec.ceilingpic,
            sec.lightlevel,
            sec.special,
            sec.tag,
        );
        saveg_write16(&mut state.p_saveg, (floorheight >> FRACBITS) as i16);
        saveg_write16(&mut state.p_saveg, (ceilingheight >> FRACBITS) as i16);
        saveg_write16(&mut state.p_saveg, floorpic);
        saveg_write16(&mut state.p_saveg, ceilingpic);
        saveg_write16(&mut state.p_saveg, lightlevel);
        saveg_write16(&mut state.p_saveg, special);
        saveg_write16(&mut state.p_saveg, tag);
    }
    for i in 0..state.p_setup.numlines {
        let li = &state.p_setup.lines[i as usize];
        let (flags, special, tag, sidenum) = (li.flags, li.special, li.tag, li.sidenum);
        saveg_write16(&mut state.p_saveg, flags);
        saveg_write16(&mut state.p_saveg, special);
        saveg_write16(&mut state.p_saveg, tag);
        for &side in sidenum.iter() {
            if side as i32 != -1_i32 {
                let si = state.p_setup.side_mut(SideId(side as u32));
                let (textureoffset, rowoffset, toptexture, bottomtexture, midtexture) = (
                    si.textureoffset,
                    si.rowoffset,
                    si.toptexture,
                    si.bottomtexture,
                    si.midtexture,
                );
                saveg_write16(&mut state.p_saveg, (textureoffset >> FRACBITS) as i16);
                saveg_write16(&mut state.p_saveg, (rowoffset >> FRACBITS) as i16);
                saveg_write16(&mut state.p_saveg, toptexture);
                saveg_write16(&mut state.p_saveg, bottomtexture);
                saveg_write16(&mut state.p_saveg, midtexture);
            }
        }
    }
}
pub fn P_UnArchiveWorld(state: &mut GameState) {
    for i in 0..state.p_setup.numsectors {
        let floorheight = ((saveg_read16(&mut state.p_saveg) as i32) << FRACBITS) as fixed_t;
        let ceilingheight = ((saveg_read16(&mut state.p_saveg) as i32) << FRACBITS) as fixed_t;
        let floorpic = saveg_read16(&mut state.p_saveg);
        let ceilingpic = saveg_read16(&mut state.p_saveg);
        let lightlevel = saveg_read16(&mut state.p_saveg);
        let special = saveg_read16(&mut state.p_saveg);
        let tag = saveg_read16(&mut state.p_saveg);
        let sec = state.p_setup.sector_mut(SectorId(i as u32));
        sec.floorheight = floorheight;
        sec.ceilingheight = ceilingheight;
        sec.floorpic = floorpic;
        sec.ceilingpic = ceilingpic;
        sec.lightlevel = lightlevel;
        sec.special = special;
        sec.tag = tag;
        sec.specialdata = None;
        sec.soundtarget = None;
    }
    for i in 0..state.p_setup.numlines {
        let flags = saveg_read16(&mut state.p_saveg);
        let special = saveg_read16(&mut state.p_saveg);
        let tag = saveg_read16(&mut state.p_saveg);
        let li = &mut state.p_setup.lines[i as usize];
        li.flags = flags;
        li.special = special;
        li.tag = tag;
        let sidenum = li.sidenum;
        for &side in sidenum.iter() {
            if side as i32 != -1_i32 {
                let textureoffset = ((saveg_read16(&mut state.p_saveg) as i32) << FRACBITS) as fixed_t;
                let rowoffset = ((saveg_read16(&mut state.p_saveg) as i32) << FRACBITS) as fixed_t;
                let toptexture = saveg_read16(&mut state.p_saveg);
                let bottomtexture = saveg_read16(&mut state.p_saveg);
                let midtexture = saveg_read16(&mut state.p_saveg);
                let si = state.p_setup.side_mut(SideId(side as u32));
                si.textureoffset = textureoffset;
                si.rowoffset = rowoffset;
                si.toptexture = toptexture;
                si.bottomtexture = bottomtexture;
                si.midtexture = midtexture;
            }
        }
    }
}
pub fn P_ArchiveThinkers(state: &mut GameState) {
    let mut cursor = state.p_tick.head();
    while let Some(id) = cursor {
        if let ThinkerPayload::Mobj(mobj_id) = state.p_tick.payload(id) {
            if matches!(P_ThinkerFunction(state, id), ThinkerFn::Mobj(_)) {
                saveg_write8(&mut state.p_saveg, tc_mobj as i32 as byte);
                saveg_write_pad(&mut state.p_saveg);
                let mobj = state.p_mobj.mobj_mut(mobj_id).expect("live mobj");
                saveg_write_mobj_t(&mut state.p_saveg, mobj);
            }
        }
        cursor = state.p_tick.next(id);
    }
    saveg_write8(&mut state.p_saveg, tc_end as i32 as byte);
}
pub fn P_UnArchiveThinkers(state: &mut GameState) {
    let mut tclass: byte = 0;
    let mut cursor = state.p_tick.head();
    while let Some(id) = cursor {
        // Unlike the raw-pointer version this replaces, `next` lives in our
        // own node table, not inside the payload memory Z_Free/deallocate
        // below may free -- capturing it first just mirrors the original
        // ordering, not a use-after-free workaround.
        let next = state.p_tick.next(id);
        // Dispatch on the node's recorded kind, not `.function` -- every
        // payload type's memory is now owned by its own arena (mobj_t and
        // all 8 thinker specials), not the zone allocator, so each needs
        // its own dealloc/deallocate call, mirroring P_RunThinkers' reaper
        // dispatch exactly. (`.function` is still live/intact at this point
        // for the Mobj case specifically, which is why the original code
        // could match on it directly -- but `kind` works uniformly for all
        // 9 and doesn't depend on that.)
        match state.p_tick.kind(id) {
            ThinkerKind::Mobj => {
                if let ThinkerPayload::Mobj(mobj_id) = state.p_tick.payload(id) {
                    P_RemoveMobj(state, mobj_id);
                    // P_RemoveMobj only retires (see PMobjState::retire) --
                    // it never itself frees the mobj's memory, and
                    // P_InitThinkers just below wipes PTickState before
                    // P_RunThinkers' reaper ever gets a chance to run on
                    // this now-Removed node, so nothing else was ever going
                    // to deallocate it. This call closes that gap (a
                    // pre-existing leak: every live mobj at the moment a
                    // savegame is loaded used to leak its Z_Malloc'd
                    // block).
                    state.p_mobj.deallocate(mobj_id);
                }
            }
            ThinkerKind::Door => {
                if let ThinkerPayload::Door(door_id) = state.p_tick.payload(id) {
                    state.p_doors.dealloc(door_id);
                }
            }
            ThinkerKind::Ceiling => {
                if let ThinkerPayload::Ceiling(ceiling_id) = state.p_tick.payload(id) {
                    state.p_ceilng.dealloc(ceiling_id);
                }
            }
            ThinkerKind::Plat => {
                if let ThinkerPayload::Plat(plat_id) = state.p_tick.payload(id) {
                    state.p_plats.dealloc(plat_id);
                }
            }
            ThinkerKind::Floor => {
                if let ThinkerPayload::Floor(floor_id) = state.p_tick.payload(id) {
                    state.p_spec.dealloc_floor(floor_id);
                }
            }
            ThinkerKind::FireFlicker => {
                if let ThinkerPayload::FireFlicker(fireflicker_id) = state.p_tick.payload(id) {
                    state.p_lights.dealloc_fireflicker(fireflicker_id);
                }
            }
            ThinkerKind::LightFlash => {
                if let ThinkerPayload::LightFlash(lightflash_id) = state.p_tick.payload(id) {
                    state.p_lights.dealloc_lightflash(lightflash_id);
                }
            }
            ThinkerKind::Strobe => {
                if let ThinkerPayload::Strobe(strobe_id) = state.p_tick.payload(id) {
                    state.p_lights.dealloc_strobe(strobe_id);
                }
            }
            ThinkerKind::Glow => {
                if let ThinkerPayload::Glow(glow_id) = state.p_tick.payload(id) {
                    state.p_lights.dealloc_glow(glow_id);
                }
            }
        }
        cursor = next;
    }
    P_InitThinkers(state);
    loop {
        tclass = saveg_read8(&mut state.p_saveg);
        match tclass as i32 {
            0 => return,
            1 => {
                saveg_read_pad(&mut state.p_saveg);
                // spawn() assigns a fresh MobjId and moves this placeholder
                // onto the heap; saveg_read_mobj_t overwrites every field
                // except `.id` (never part of the on-disk format), so the id
                // spawn() just assigned survives the read untouched below.
                let placeholder = state.p_mobj.dummy_mobj;
                let (mobj_arena_id, _) = state.p_mobj.spawn(placeholder);
                saveg_read_mobj_t(&mut state.p_saveg, state.p_mobj.mo_mut(mobj_arena_id));
                if let Some(player_id) = state.p_mobj.mo(mobj_arena_id).player {
                    state.g_game.player_mut(player_id).mo = Some(mobj_arena_id);
                }
                {
                    let m = state.p_mobj.mo_mut(mobj_arena_id);
                    m.target = None;
                    m.tracer = None;
                }
                P_SetThingPosition(state, mobj_arena_id);
                let subsector = state.p_mobj.mo(mobj_arena_id).subsector;
                let sector = state.p_setup.subsectors[subsector.0 as usize].sector;
                let (floorheight, ceilingheight) = {
                    let s = state.p_setup.sector_mut(sector);
                    (s.floorheight, s.ceilingheight)
                };
                {
                    let m = state.p_mobj.mo_mut(mobj_arena_id);
                    m.floorz = floorheight;
                    m.ceilingz = ceilingheight;
                    m.thinker.function = ThinkerFn::Mobj(P_MobjThinker);
                }
                P_AddThinker(state, ThinkerPayload::Mobj(mobj_arena_id), ThinkerKind::Mobj);
            }
            _ => {
                I_Error(&format!("Unknown tclass {} in savegame", tclass as i32,));
            }
        }
    }
}
pub static specials_e: C2RustUnnamed_5 = tc_ceiling;
pub fn P_ArchiveSpecials(state: &mut GameState) {
    let mut cursor = state.p_tick.head();
    while let Some(id) = cursor {
        match P_ThinkerFunction(state, id) {
            ThinkerFn::Paused => {
                let in_stasis = state
                    .p_ceilng
                    .activeceilings
                    .iter()
                    .take(MAXCEILINGS as usize)
                    .any(|&entry| entry == Some(id));
                if in_stasis {
                    let ceiling_id = state.p_tick.ceiling_payload(id);
                    saveg_write8(&mut state.p_saveg, tc_ceiling as i32 as byte);
                    saveg_write_pad(&mut state.p_saveg);
                    let c = state.p_ceilng.get_mut(ceiling_id).expect("live ceiling");
                    saveg_write_ceiling_t(&mut state.p_saveg, c);
                }
            }
            ThinkerFn::Ceiling(_) => {
                let ceiling_id = state.p_tick.ceiling_payload(id);
                saveg_write8(&mut state.p_saveg, tc_ceiling as i32 as byte);
                saveg_write_pad(&mut state.p_saveg);
                let c = state.p_ceilng.get_mut(ceiling_id).expect("live ceiling");
                saveg_write_ceiling_t(&mut state.p_saveg, c);
            }
            ThinkerFn::Door(_) => {
                let door_id = state.p_tick.door_payload(id);
                saveg_write8(&mut state.p_saveg, tc_door as i32 as byte);
                saveg_write_pad(&mut state.p_saveg);
                let d = state.p_doors.get_mut(door_id).expect("live door");
                saveg_write_vldoor_t(&mut state.p_saveg, d);
            }
            ThinkerFn::Floor(_) => {
                let floor_id = state.p_tick.floor_payload(id);
                saveg_write8(&mut state.p_saveg, tc_floor as i32 as byte);
                saveg_write_pad(&mut state.p_saveg);
                let f = state.p_spec.get_floor_mut(floor_id).expect("live floor");
                saveg_write_floormove_t(&mut state.p_saveg, f);
            }
            ThinkerFn::Plat(_) => {
                let plat_id = state.p_tick.plat_payload(id);
                saveg_write8(&mut state.p_saveg, tc_plat as i32 as byte);
                saveg_write_pad(&mut state.p_saveg);
                let p = state.p_plats.get_mut(plat_id).expect("live plat");
                saveg_write_plat_t(&mut state.p_saveg, p);
            }
            ThinkerFn::LightFlash(_) => {
                let ThinkerPayload::LightFlash(flash_id) = state.p_tick.payload(id) else {
                    unreachable!()
                };
                saveg_write8(&mut state.p_saveg, tc_flash as i32 as byte);
                saveg_write_pad(&mut state.p_saveg);
                let f = state.p_lights.get_lightflash_mut(flash_id).expect("live lightflash");
                saveg_write_lightflash_t(&mut state.p_saveg, f);
            }
            ThinkerFn::Strobe(_) => {
                let ThinkerPayload::Strobe(strobe_id) = state.p_tick.payload(id) else {
                    unreachable!()
                };
                saveg_write8(&mut state.p_saveg, tc_strobe as i32 as byte);
                saveg_write_pad(&mut state.p_saveg);
                let s = state.p_lights.get_strobe_mut(strobe_id).expect("live strobe");
                saveg_write_strobe_t(&mut state.p_saveg, s);
            }
            ThinkerFn::Glow(_) => {
                let ThinkerPayload::Glow(glow_id) = state.p_tick.payload(id) else {
                    unreachable!()
                };
                saveg_write8(&mut state.p_saveg, tc_glow as i32 as byte);
                saveg_write_pad(&mut state.p_saveg);
                let g = state.p_lights.get_glow_mut(glow_id).expect("live glow");
                saveg_write_glow_t(&mut state.p_saveg, g);
            }
            _ => {}
        }
        cursor = state.p_tick.next(id);
    }
    saveg_write8(&mut state.p_saveg, tc_endspecials as i32 as byte);
}
pub fn P_UnArchiveSpecials(state: &mut GameState) {
    let mut tclass: byte = 0;
    loop {
        tclass = saveg_read8(&mut state.p_saveg);
        match tclass as i32 {
            7 => return,
            0 => {
                saveg_read_pad(&mut state.p_saveg);
                let (ceiling_arena_id, _) = state.p_ceilng.spawn(ceiling_t::default());
                let sector = {
                    let c = state.p_ceilng.get_mut(ceiling_arena_id).expect("live ceiling");
                    saveg_read_ceiling_t(&mut state.p_saveg, c);
                    if matches!(c.thinker.function, ThinkerFn::Unresolved) {
                        c.thinker.function = ThinkerFn::Ceiling(T_MoveCeiling);
                    }
                    c.sector
                };
                let ceiling_id = P_AddThinker(
                    state,
                    ThinkerPayload::Ceiling(ceiling_arena_id),
                    ThinkerKind::Ceiling,
                );
                state.p_setup.sector_mut(sector).specialdata =
                    Some(SectorSpecial::Ceiling(ceiling_id));
                P_AddActiveCeiling(&mut state.p_ceilng, ceiling_id);
            }
            1 => {
                saveg_read_pad(&mut state.p_saveg);
                let (door_arena_id, _) = state.p_doors.spawn(vldoor_t::default());
                let sector = {
                    let d = state.p_doors.get_mut(door_arena_id).expect("live door");
                    saveg_read_vldoor_t(&mut state.p_saveg, d);
                    d.thinker.function = ThinkerFn::Door(T_VerticalDoor);
                    d.sector
                };
                let door_id = P_AddThinker(state, ThinkerPayload::Door(door_arena_id), ThinkerKind::Door);
                state.p_setup.sector_mut(sector).specialdata = Some(SectorSpecial::Door(door_id));
            }
            2 => {
                saveg_read_pad(&mut state.p_saveg);
                let (floor_arena_id, _) = state.p_spec.spawn_floor(floormove_t::default());
                let sector = {
                    let f = state.p_spec.get_floor_mut(floor_arena_id).expect("live floor");
                    saveg_read_floormove_t(&mut state.p_saveg, f);
                    f.thinker.function = ThinkerFn::Floor(T_MoveFloor);
                    f.sector
                };
                let floor_id =
                    P_AddThinker(state, ThinkerPayload::Floor(floor_arena_id), ThinkerKind::Floor);
                state.p_setup.sector_mut(sector).specialdata = Some(SectorSpecial::Floor(floor_id));
            }
            3 => {
                saveg_read_pad(&mut state.p_saveg);
                let (plat_arena_id, _) = state.p_plats.spawn(plat_t::default());
                let sector = {
                    let p = state.p_plats.get_mut(plat_arena_id).expect("live plat");
                    saveg_read_plat_t(&mut state.p_saveg, p);
                    if matches!(p.thinker.function, ThinkerFn::Unresolved) {
                        p.thinker.function = ThinkerFn::Plat(T_PlatRaise);
                    }
                    p.sector
                };
                let plat_id =
                    P_AddThinker(state, ThinkerPayload::Plat(plat_arena_id), ThinkerKind::Plat);
                state.p_setup.sector_mut(sector).specialdata = Some(SectorSpecial::Plat(plat_id));
                P_AddActivePlat(&mut state.p_plats, plat_id);
            }
            4 => {
                saveg_read_pad(&mut state.p_saveg);
                let (flash_arena_id, _) = state.p_lights.spawn_lightflash(lightflash_t::default());
                {
                    let f = state
                        .p_lights
                        .get_lightflash_mut(flash_arena_id)
                        .expect("live lightflash");
                    saveg_read_lightflash_t(&mut state.p_saveg, f);
                    f.thinker.function = ThinkerFn::LightFlash(T_LightFlash);
                }
                P_AddThinker(
                    state,
                    ThinkerPayload::LightFlash(flash_arena_id),
                    ThinkerKind::LightFlash,
                );
            }
            5 => {
                saveg_read_pad(&mut state.p_saveg);
                let (strobe_arena_id, _) = state.p_lights.spawn_strobe(strobe_t::default());
                {
                    let s = state
                        .p_lights
                        .get_strobe_mut(strobe_arena_id)
                        .expect("live strobe");
                    saveg_read_strobe_t(&mut state.p_saveg, s);
                    s.thinker.function = ThinkerFn::Strobe(T_StrobeFlash);
                }
                P_AddThinker(
                    state,
                    ThinkerPayload::Strobe(strobe_arena_id),
                    ThinkerKind::Strobe,
                );
            }
            6 => {
                saveg_read_pad(&mut state.p_saveg);
                let (glow_arena_id, _) = state.p_lights.spawn_glow(glow_t::default());
                {
                    let g = state.p_lights.get_glow_mut(glow_arena_id).expect("live glow");
                    saveg_read_glow_t(&mut state.p_saveg, g);
                    g.thinker.function = ThinkerFn::Glow(T_Glow);
                }
                P_AddThinker(state, ThinkerPayload::Glow(glow_arena_id), ThinkerKind::Glow);
            }
            _ => {
                I_Error(&format!(
                    "P_UnarchiveSpecials:Unknown tclass {} in savegame",
                    tclass as i32,
                ));
            }
        }
    }
}
