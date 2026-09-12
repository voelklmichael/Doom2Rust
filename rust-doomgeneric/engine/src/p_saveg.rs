use crate::src::d_mode::skill_t;
use crate::src::d_player::NUMPOWERS;
use crate::src::info::StateId;
use crate::src::d_player::NUMPSPRITES;
use crate::src::d_player::{player_t, playerstate_t, PlayerId};
use crate::src::d_player::{weapontype_t, NUMWEAPONS};
use crate::src::d_ticcmd::ticcmd_t;
use crate::src::doomdef::boolean;
use crate::src::g_game::G_VanillaVersionCode;
use crate::src::i_system::I_Error;
use std::io::{Read, Seek, Write};
use crate::src::m_fixed::fixed_t;
use crate::src::p_ceilng::ceiling_e;
use crate::src::p_ceilng::P_AddActiveCeiling;
use crate::src::p_doors::vldoor_e;
use crate::src::p_doors::vldoor_t;
use crate::src::p_floor::floor_e;
use crate::src::p_lights::{glow_t, lightflash_t, strobe_t};
use crate::src::p_maputl::P_SetThingPosition;
use crate::src::p_mobj::mobjtype_t;
use crate::src::p_mobj::spritenum_t;
use crate::src::p_mobj::P_RemoveMobj;
use crate::src::p_mobj::{
    line_t, mapthing_t, sector_t, thinker_s, thinker_t, SectorSpecial, ThinkerFn,
};
use crate::src::p_mobj::{mobj_t, pspdef_t};
use crate::src::p_plats::plat_e;
use crate::src::p_plats::plattype_e;
use crate::src::p_plats::P_AddActivePlat;
use crate::src::p_setup::SectorId;
use crate::src::p_setup::SideId;
use crate::src::p_setup::SubsectorId;
use crate::src::p_spec::{ceiling_t, floormove_t, plat_t};
use crate::src::p_tick::P_AddThinker;
use crate::src::p_tick::P_InitThinkers;
use crate::src::r_defs::side_t;
use crate::src::stdint_types::byte;
use crate::src::tables::angle_t;
use crate::src::z_zone::Z_Free;
use crate::src::z_zone::Z_Malloc;
use crate::src::z_zone::PU_LEVEL;

use crate::src::d_player::NUMAMMO;
use crate::src::doomdef::MAXPLAYERS;
use crate::src::doomdef::NULL;
use crate::src::game_state::GameState;
use crate::src::m_fixed::FRACBITS;
use crate::src::m_menu::SAVESTRINGSIZE;
use crate::src::p_ceilng::T_MoveCeiling;
use crate::src::p_ceilng::MAXCEILINGS;
use crate::src::p_doors::T_VerticalDoor;
use crate::src::p_floor::T_MoveFloor;
use crate::src::p_inter::NUMCARDS;
use crate::src::p_lights::{T_Glow, T_LightFlash, T_StrobeFlash};
use crate::src::p_mobj::P_MobjThinker;
use crate::src::p_plats::T_PlatRaise;

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
#[no_mangle]
pub static savegamelength: i32 = 0;
pub unsafe fn P_TempSaveGameFile(state: &mut GameState) -> String {
    if state.p_saveg.temp_savegame_filename.is_none() {
        state.p_saveg.temp_savegame_filename =
            Some(format!("{}temp.dsg", state.d_main.savegamedir));
    }
    state.p_saveg.temp_savegame_filename.clone().unwrap()
}
pub unsafe fn P_SaveGameFile(state: &mut GameState, slot: i32) -> String {
    format!("{}doomsav{}.dsg", state.d_main.savegamedir, slot)
}
unsafe fn saveg_read8(state: &mut GameState) -> byte {
    let mut result: [byte; 1] = [0];
    if state
        .p_saveg
        .save_stream
        .as_mut()
        .unwrap()
        .read(&mut result)
        .unwrap_or(0)
        < 1
    {
        if !state.p_saveg.savegame_error {
            eprintln!("saveg_read8: Unexpected end of file while reading save game");
            state.p_saveg.savegame_error = true;
        }
    }
    return result[0];
}
unsafe fn saveg_write8(state: &mut GameState, value: byte) {
    if state
        .p_saveg
        .save_stream
        .as_mut()
        .unwrap()
        .write(&[value])
        .unwrap_or(0)
        < 1
    {
        if !state.p_saveg.savegame_error {
            eprintln!("saveg_write8: Error while writing save game");
            state.p_saveg.savegame_error = true;
        }
    }
}
unsafe fn saveg_read16(state: &mut GameState) -> i16 {
    let mut result: i32 = 0;
    result = saveg_read8(state) as i32;
    result |= (saveg_read8(state) as i32) << 8 as i32;
    return result as i16;
}
unsafe fn saveg_write16(state: &mut GameState, mut value: i16) {
    saveg_write8(state, (value as i32 & 0xff as i32) as byte);
    saveg_write8(state, (value as i32 >> 8 as i32 & 0xff as i32) as byte);
}
unsafe fn saveg_read32(state: &mut GameState) -> i32 {
    let mut result: i32 = 0;
    result = saveg_read8(state) as i32;
    result |= (saveg_read8(state) as i32) << 8 as i32;
    result |= (saveg_read8(state) as i32) << 16 as i32;
    result |= (saveg_read8(state) as i32) << 24 as i32;
    return result;
}
unsafe fn saveg_write32(state: &mut GameState, mut value: i32) {
    saveg_write8(state, (value & 0xff as i32) as byte);
    saveg_write8(state, (value >> 8 as i32 & 0xff as i32) as byte);
    saveg_write8(state, (value >> 16 as i32 & 0xff as i32) as byte);
    saveg_write8(state, (value >> 24 as i32 & 0xff as i32) as byte);
}
unsafe fn saveg_read_pad(state: &mut GameState) {
    let mut padding: i32 = 0;
    let mut i: i32 = 0;
    let pos = state
        .p_saveg
        .save_stream
        .as_mut()
        .unwrap()
        .stream_position()
        .unwrap_or(0);
    padding = ((4 as u64).wrapping_sub(pos & 3 as u64) & 3 as u64) as i32;
    i = 0 as i32;
    while i < padding {
        saveg_read8(state);
        i += 1;
    }
}
unsafe fn saveg_write_pad(state: &mut GameState) {
    let mut padding: i32 = 0;
    let mut i: i32 = 0;
    let pos = state
        .p_saveg
        .save_stream
        .as_mut()
        .unwrap()
        .stream_position()
        .unwrap_or(0);
    padding = ((4 as u64).wrapping_sub(pos & 3 as u64) & 3 as u64) as i32;
    i = 0 as i32;
    while i < padding {
        saveg_write8(state, 0 as byte);
        i += 1;
    }
}
unsafe fn saveg_readp(state: &mut GameState) -> *mut ::core::ffi::c_void {
    return saveg_read32(state) as intptr_t as *mut ::core::ffi::c_void;
}
unsafe fn saveg_writep(state: &mut GameState, mut p: *mut ::core::ffi::c_void) {
    saveg_write32(state, p as intptr_t as i32);
}
unsafe fn saveg_read_mapthing_t(state: &mut GameState, mut str: *mut mapthing_t) {
    (*str).x = saveg_read16(state);
    (*str).y = saveg_read16(state);
    (*str).angle = saveg_read16(state);
    (*str).type_0 = saveg_read16(state);
    (*str).options = saveg_read16(state);
}
unsafe fn saveg_write_mapthing_t(state: &mut GameState, mut str: *mut mapthing_t) {
    saveg_write16(state, (*str).x);
    saveg_write16(state, (*str).y);
    saveg_write16(state, (*str).angle);
    saveg_write16(state, (*str).type_0);
    saveg_write16(state, (*str).options);
}
unsafe fn saveg_read_actionf_t(state: &mut GameState, mut str: *mut ThinkerFn) {
    let word = saveg_readp(state);
    *str = if word.is_null() {
        ThinkerFn::Paused
    } else {
        ThinkerFn::Unresolved
    };
}
unsafe fn saveg_write_actionf_t(state: &mut GameState, mut str: *mut ThinkerFn) {
    let word: *mut ::core::ffi::c_void = if matches!(*str, ThinkerFn::Paused) {
        ::core::ptr::null_mut()
    } else {
        str as *mut ::core::ffi::c_void
    };
    saveg_writep(state, word);
}
unsafe fn saveg_read_thinker_t(state: &mut GameState, mut str: *mut thinker_t) {
    (*str).prev = saveg_readp(state) as *mut thinker_s;
    (*str).next = saveg_readp(state) as *mut thinker_s;
    saveg_read_actionf_t(state, &raw mut (*str).function);
}
unsafe fn saveg_write_thinker_t(state: &mut GameState, mut str: *mut thinker_t) {
    saveg_writep(state, (*str).prev as *mut ::core::ffi::c_void);
    saveg_writep(state, (*str).next as *mut ::core::ffi::c_void);
    saveg_write_actionf_t(state, &raw mut (*str).function);
}
unsafe fn saveg_read_mobj_t(state: &mut GameState, mut str: *mut mobj_t) {
    let mut pl: i32 = 0;
    saveg_read_thinker_t(state, &raw mut (*str).thinker);
    (*str).x = saveg_read32(state) as fixed_t;
    (*str).y = saveg_read32(state) as fixed_t;
    (*str).z = saveg_read32(state) as fixed_t;
    // P_SetThingPosition (called on every reconstructed mobj right after this,
    // see P_UnArchiveThinkers) fully rebuilds snext/sprev from scratch, so
    // these on-disk bytes are already dead -- discard, same treatment as
    // target/tracer's inert reads just below.
    saveg_read32(state);
    (*str).snext = None;
    saveg_read32(state);
    (*str).sprev = None;
    (*str).angle = saveg_read32(state) as angle_t;
    (*str).sprite = saveg_read32(state) as spritenum_t;
    (*str).frame = saveg_read32(state);
    // P_SetThingPosition also fully rebuilds bnext/bprev from scratch --
    // same dead-bytes treatment as snext/sprev above.
    saveg_read32(state);
    (*str).bnext = None;
    saveg_read32(state);
    (*str).bprev = None;
    saveg_read32(state);
    (*str).subsector = SubsectorId(0);
    (*str).floorz = saveg_read32(state) as fixed_t;
    (*str).ceilingz = saveg_read32(state) as fixed_t;
    (*str).radius = saveg_read32(state) as fixed_t;
    (*str).height = saveg_read32(state) as fixed_t;
    (*str).momx = saveg_read32(state) as fixed_t;
    (*str).momy = saveg_read32(state) as fixed_t;
    (*str).momz = saveg_read32(state) as fixed_t;
    (*str).validcount = saveg_read32(state);
    (*str).type_0 = saveg_read32(state) as mobjtype_t;
    saveg_read32(state);
    (*str).tics = saveg_read32(state);
    (*str).state = Some(StateId(saveg_read32(state) as u32));
    (*str).flags = saveg_read32(state);
    (*str).health = saveg_read32(state);
    (*str).movedir = saveg_read32(state);
    (*str).movecount = saveg_read32(state);
    saveg_read32(state);
    (*str).target = None;
    (*str).reactiontime = saveg_read32(state);
    (*str).threshold = saveg_read32(state);
    pl = saveg_read32(state);
    if pl > 0 as i32 {
        let player_id = PlayerId((pl - 1 as i32) as u8);
        (*str).player = Some(player_id);
        (*state.g_game.player_mut(player_id)).mo = str;
    } else {
        (*str).player = None;
    }
    (*str).lastlook = saveg_read32(state);
    saveg_read_mapthing_t(state, &raw mut (*str).spawnpoint);
    saveg_read32(state);
    (*str).tracer = None;
}
unsafe fn saveg_write_mobj_t(state: &mut GameState, mut str: *mut mobj_t) {
    saveg_write_thinker_t(state, &raw mut (*str).thinker);
    saveg_write32(state, (*str).x as i32);
    saveg_write32(state, (*str).y as i32);
    saveg_write32(state, (*str).z as i32);
    saveg_write32(state, 0);
    saveg_write32(state, 0);
    saveg_write32(state, (*str).angle as i32);
    saveg_write32(state, (*str).sprite as i32);
    saveg_write32(state, (*str).frame);
    saveg_write32(state, 0);
    saveg_write32(state, 0);
    saveg_write32(state, 0);
    saveg_write32(state, (*str).floorz as i32);
    saveg_write32(state, (*str).ceilingz as i32);
    saveg_write32(state, (*str).radius as i32);
    saveg_write32(state, (*str).height as i32);
    saveg_write32(state, (*str).momx as i32);
    saveg_write32(state, (*str).momy as i32);
    saveg_write32(state, (*str).momz as i32);
    saveg_write32(state, (*str).validcount);
    saveg_write32(state, (*str).type_0 as i32);
    saveg_write32(state, 0);
    saveg_write32(state, (*str).tics);
    saveg_write32(state, (*str).state.unwrap().0 as i32);
    saveg_write32(state, (*str).flags);
    saveg_write32(state, (*str).health);
    saveg_write32(state, (*str).movedir);
    saveg_write32(state, (*str).movecount);
    saveg_write32(state, 0 as i32);
    saveg_write32(state, (*str).reactiontime);
    saveg_write32(state, (*str).threshold);
    if let Some(player_id) = (*str).player {
        saveg_write32(state, player_id.0 as i32 + 1 as i32);
    } else {
        saveg_write32(state, 0 as i32);
    }
    saveg_write32(state, (*str).lastlook);
    saveg_write_mapthing_t(state, &raw mut (*str).spawnpoint);
    saveg_write32(state, 0 as i32);
}
unsafe fn saveg_read_ticcmd_t(state: &mut GameState, mut str: *mut ticcmd_t) {
    (*str).forwardmove = saveg_read8(state) as i8;
    (*str).sidemove = saveg_read8(state) as i8;
    (*str).angleturn = saveg_read16(state);
    (*str).consistancy = saveg_read16(state) as byte;
    (*str).chatchar = saveg_read8(state);
    (*str).buttons = saveg_read8(state);
}
unsafe fn saveg_write_ticcmd_t(state: &mut GameState, mut str: *mut ticcmd_t) {
    saveg_write8(state, (*str).forwardmove as byte);
    saveg_write8(state, (*str).sidemove as byte);
    saveg_write16(state, (*str).angleturn);
    saveg_write16(state, (*str).consistancy as i16);
    saveg_write8(state, (*str).chatchar);
    saveg_write8(state, (*str).buttons);
}
unsafe fn saveg_read_pspdef_t(state: &mut GameState, mut str: *mut pspdef_t) {
    let mut state_num: i32 = 0;
    state_num = saveg_read32(state);
    if state_num > 0 as i32 {
        (*str).state = Some(StateId(state_num as u32));
    } else {
        (*str).state = None;
    }
    (*str).tics = saveg_read32(state);
    (*str).sx = saveg_read32(state) as fixed_t;
    (*str).sy = saveg_read32(state) as fixed_t;
}
unsafe fn saveg_write_pspdef_t(state: &mut GameState, mut str: *mut pspdef_t) {
    if let Some(state_id) = (*str).state {
        saveg_write32(state, state_id.0 as i32);
    } else {
        saveg_write32(state, 0 as i32);
    }
    saveg_write32(state, (*str).tics);
    saveg_write32(state, (*str).sx as i32);
    saveg_write32(state, (*str).sy as i32);
}
unsafe fn saveg_read_player_t(state: &mut GameState, mut str: *mut player_t) {
    let mut i: i32 = 0;
    (*str).mo = saveg_readp(state) as *mut mobj_t;
    (*str).playerstate = saveg_read32(state) as playerstate_t;
    saveg_read_ticcmd_t(state, &raw mut (*str).cmd);
    (*str).viewz = saveg_read32(state) as fixed_t;
    (*str).viewheight = saveg_read32(state) as fixed_t;
    (*str).deltaviewheight = saveg_read32(state) as fixed_t;
    (*str).bob = saveg_read32(state) as fixed_t;
    (*str).health = saveg_read32(state);
    (*str).armorpoints = saveg_read32(state);
    (*str).armortype = saveg_read32(state);
    i = 0 as i32;
    while i < NUMPOWERS as i32 {
        (*str).powers[i as usize] = saveg_read32(state);
        i += 1;
    }
    i = 0 as i32;
    while i < NUMCARDS as i32 {
        (*str).cards[i as usize] = saveg_read32(state) != 0;
        i += 1;
    }
    (*str).backpack = saveg_read32(state) != 0;
    i = 0 as i32;
    while i < MAXPLAYERS {
        (*str).frags[i as usize] = saveg_read32(state);
        i += 1;
    }
    (*str).readyweapon = saveg_read32(state) as weapontype_t;
    (*str).pendingweapon = saveg_read32(state) as weapontype_t;
    i = 0 as i32;
    while i < NUMWEAPONS as i32 {
        (*str).weaponowned[i as usize] = saveg_read32(state) != 0;
        i += 1;
    }
    i = 0 as i32;
    while i < NUMAMMO as i32 {
        (*str).ammo[i as usize] = saveg_read32(state);
        i += 1;
    }
    i = 0 as i32;
    while i < NUMAMMO as i32 {
        (*str).maxammo[i as usize] = saveg_read32(state);
        i += 1;
    }
    (*str).attackdown = saveg_read32(state);
    (*str).usedown = saveg_read32(state);
    (*str).cheats = saveg_read32(state);
    (*str).refire = saveg_read32(state);
    (*str).killcount = saveg_read32(state);
    (*str).itemcount = saveg_read32(state);
    (*str).secretcount = saveg_read32(state);
    saveg_readp(state);
    (*str).message = None;
    (*str).damagecount = saveg_read32(state);
    (*str).bonuscount = saveg_read32(state);
    saveg_read32(state);
    (*str).attacker = None;
    (*str).extralight = saveg_read32(state);
    (*str).fixedcolormap = saveg_read32(state);
    (*str).colormap = saveg_read32(state);
    i = 0 as i32;
    while i < NUMPSPRITES as i32 {
        saveg_read_pspdef_t(
            state,
            (&raw mut (*str).psprites as *mut pspdef_t).offset(i as isize) as *mut pspdef_t,
        );
        i += 1;
    }
    (*str).didsecret = saveg_read32(state) != 0;
}
unsafe fn saveg_write_player_t(state: &mut GameState, mut str: *mut player_t) {
    let mut i: i32 = 0;
    saveg_writep(state, (*str).mo as *mut ::core::ffi::c_void);
    saveg_write32(state, (*str).playerstate as i32);
    saveg_write_ticcmd_t(state, &raw mut (*str).cmd);
    saveg_write32(state, (*str).viewz as i32);
    saveg_write32(state, (*str).viewheight as i32);
    saveg_write32(state, (*str).deltaviewheight as i32);
    saveg_write32(state, (*str).bob as i32);
    saveg_write32(state, (*str).health);
    saveg_write32(state, (*str).armorpoints);
    saveg_write32(state, (*str).armortype);
    i = 0 as i32;
    while i < NUMPOWERS as i32 {
        saveg_write32(state, (*str).powers[i as usize]);
        i += 1;
    }
    i = 0 as i32;
    while i < NUMCARDS as i32 {
        saveg_write32(state, (*str).cards[i as usize] as i32);
        i += 1;
    }
    saveg_write32(state, (*str).backpack as i32);
    i = 0 as i32;
    while i < MAXPLAYERS {
        saveg_write32(state, (*str).frags[i as usize]);
        i += 1;
    }
    saveg_write32(state, (*str).readyweapon as i32);
    saveg_write32(state, (*str).pendingweapon as i32);
    i = 0 as i32;
    while i < NUMWEAPONS as i32 {
        saveg_write32(state, (*str).weaponowned[i as usize] as i32);
        i += 1;
    }
    i = 0 as i32;
    while i < NUMAMMO as i32 {
        saveg_write32(state, (*str).ammo[i as usize]);
        i += 1;
    }
    i = 0 as i32;
    while i < NUMAMMO as i32 {
        saveg_write32(state, (*str).maxammo[i as usize]);
        i += 1;
    }
    saveg_write32(state, (*str).attackdown);
    saveg_write32(state, (*str).usedown);
    saveg_write32(state, (*str).cheats);
    saveg_write32(state, (*str).refire);
    saveg_write32(state, (*str).killcount);
    saveg_write32(state, (*str).itemcount);
    saveg_write32(state, (*str).secretcount);
    saveg_writep(
        state,
        if (*str).message.is_some() {
            1 as *mut ::core::ffi::c_void
        } else {
            ::core::ptr::null_mut()
        },
    );
    saveg_write32(state, (*str).damagecount);
    saveg_write32(state, (*str).bonuscount);
    saveg_write32(state, 0 as i32);
    saveg_write32(state, (*str).extralight);
    saveg_write32(state, (*str).fixedcolormap);
    saveg_write32(state, (*str).colormap);
    i = 0 as i32;
    while i < NUMPSPRITES as i32 {
        saveg_write_pspdef_t(
            state,
            (&raw mut (*str).psprites as *mut pspdef_t).offset(i as isize) as *mut pspdef_t,
        );
        i += 1;
    }
    saveg_write32(state, (*str).didsecret as i32);
}
unsafe fn saveg_read_ceiling_t(state: &mut GameState, mut str: *mut ceiling_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &raw mut (*str).thinker);
    (*str).type_0 = saveg_read32(state) as ceiling_e;
    sector = saveg_read32(state);
    (*str).sector = SectorId(sector as u32);
    (*str).bottomheight = saveg_read32(state) as fixed_t;
    (*str).topheight = saveg_read32(state) as fixed_t;
    (*str).speed = saveg_read32(state) as fixed_t;
    (*str).crush = saveg_read32(state) != 0;
    (*str).direction = saveg_read32(state);
    (*str).tag = saveg_read32(state);
    (*str).olddirection = saveg_read32(state);
}
unsafe fn saveg_write_ceiling_t(state: &mut GameState, mut str: *mut ceiling_t) {
    saveg_write_thinker_t(state, &raw mut (*str).thinker);
    saveg_write32(state, (*str).type_0 as i32);
    saveg_write32(state, (*str).sector.0 as i32);
    saveg_write32(state, (*str).bottomheight as i32);
    saveg_write32(state, (*str).topheight as i32);
    saveg_write32(state, (*str).speed as i32);
    saveg_write32(state, (*str).crush as i32);
    saveg_write32(state, (*str).direction);
    saveg_write32(state, (*str).tag);
    saveg_write32(state, (*str).olddirection);
}
unsafe fn saveg_read_vldoor_t(state: &mut GameState, mut str: *mut vldoor_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &raw mut (*str).thinker);
    (*str).type_0 = saveg_read32(state) as vldoor_e;
    sector = saveg_read32(state);
    (*str).sector = SectorId(sector as u32);
    (*str).topheight = saveg_read32(state) as fixed_t;
    (*str).speed = saveg_read32(state) as fixed_t;
    (*str).direction = saveg_read32(state);
    (*str).topwait = saveg_read32(state);
    (*str).topcountdown = saveg_read32(state);
}
unsafe fn saveg_write_vldoor_t(state: &mut GameState, mut str: *mut vldoor_t) {
    saveg_write_thinker_t(state, &raw mut (*str).thinker);
    saveg_write32(state, (*str).type_0 as i32);
    saveg_write32(state, (*str).sector.0 as i32);
    saveg_write32(state, (*str).topheight as i32);
    saveg_write32(state, (*str).speed as i32);
    saveg_write32(state, (*str).direction);
    saveg_write32(state, (*str).topwait);
    saveg_write32(state, (*str).topcountdown);
}
unsafe fn saveg_read_floormove_t(state: &mut GameState, mut str: *mut floormove_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &raw mut (*str).thinker);
    (*str).type_0 = saveg_read32(state) as floor_e;
    (*str).crush = saveg_read32(state) != 0;
    sector = saveg_read32(state);
    (*str).sector = SectorId(sector as u32);
    (*str).direction = saveg_read32(state);
    (*str).newspecial = saveg_read32(state);
    (*str).texture = saveg_read16(state);
    (*str).floordestheight = saveg_read32(state) as fixed_t;
    (*str).speed = saveg_read32(state) as fixed_t;
}
unsafe fn saveg_write_floormove_t(state: &mut GameState, mut str: *mut floormove_t) {
    saveg_write_thinker_t(state, &raw mut (*str).thinker);
    saveg_write32(state, (*str).type_0 as i32);
    saveg_write32(state, (*str).crush as i32);
    saveg_write32(state, (*str).sector.0 as i32);
    saveg_write32(state, (*str).direction);
    saveg_write32(state, (*str).newspecial);
    saveg_write16(state, (*str).texture);
    saveg_write32(state, (*str).floordestheight as i32);
    saveg_write32(state, (*str).speed as i32);
}
unsafe fn saveg_read_plat_t(state: &mut GameState, mut str: *mut plat_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &raw mut (*str).thinker);
    sector = saveg_read32(state);
    (*str).sector = SectorId(sector as u32);
    (*str).speed = saveg_read32(state) as fixed_t;
    (*str).low = saveg_read32(state) as fixed_t;
    (*str).high = saveg_read32(state) as fixed_t;
    (*str).wait = saveg_read32(state);
    (*str).count = saveg_read32(state);
    (*str).status = saveg_read32(state) as plat_e;
    (*str).oldstatus = saveg_read32(state) as plat_e;
    (*str).crush = saveg_read32(state) != 0;
    (*str).tag = saveg_read32(state);
    (*str).type_0 = saveg_read32(state) as plattype_e;
}
unsafe fn saveg_write_plat_t(state: &mut GameState, mut str: *mut plat_t) {
    saveg_write_thinker_t(state, &raw mut (*str).thinker);
    saveg_write32(state, (*str).sector.0 as i32);
    saveg_write32(state, (*str).speed as i32);
    saveg_write32(state, (*str).low as i32);
    saveg_write32(state, (*str).high as i32);
    saveg_write32(state, (*str).wait);
    saveg_write32(state, (*str).count);
    saveg_write32(state, (*str).status as i32);
    saveg_write32(state, (*str).oldstatus as i32);
    saveg_write32(state, (*str).crush as i32);
    saveg_write32(state, (*str).tag);
    saveg_write32(state, (*str).type_0 as i32);
}
unsafe fn saveg_read_lightflash_t(state: &mut GameState, mut str: *mut lightflash_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &raw mut (*str).thinker);
    sector = saveg_read32(state);
    (*str).sector = SectorId(sector as u32);
    (*str).count = saveg_read32(state);
    (*str).maxlight = saveg_read32(state);
    (*str).minlight = saveg_read32(state);
    (*str).maxtime = saveg_read32(state);
    (*str).mintime = saveg_read32(state);
}
unsafe fn saveg_write_lightflash_t(state: &mut GameState, mut str: *mut lightflash_t) {
    saveg_write_thinker_t(state, &raw mut (*str).thinker);
    saveg_write32(state, (*str).sector.0 as i32);
    saveg_write32(state, (*str).count);
    saveg_write32(state, (*str).maxlight);
    saveg_write32(state, (*str).minlight);
    saveg_write32(state, (*str).maxtime);
    saveg_write32(state, (*str).mintime);
}
unsafe fn saveg_read_strobe_t(state: &mut GameState, mut str: *mut strobe_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &raw mut (*str).thinker);
    sector = saveg_read32(state);
    (*str).sector = SectorId(sector as u32);
    (*str).count = saveg_read32(state);
    (*str).minlight = saveg_read32(state);
    (*str).maxlight = saveg_read32(state);
    (*str).darktime = saveg_read32(state);
    (*str).brighttime = saveg_read32(state);
}
unsafe fn saveg_write_strobe_t(state: &mut GameState, mut str: *mut strobe_t) {
    saveg_write_thinker_t(state, &raw mut (*str).thinker);
    saveg_write32(state, (*str).sector.0 as i32);
    saveg_write32(state, (*str).count);
    saveg_write32(state, (*str).minlight);
    saveg_write32(state, (*str).maxlight);
    saveg_write32(state, (*str).darktime);
    saveg_write32(state, (*str).brighttime);
}
unsafe fn saveg_read_glow_t(state: &mut GameState, mut str: *mut glow_t) {
    let mut sector: i32 = 0;
    saveg_read_thinker_t(state, &raw mut (*str).thinker);
    sector = saveg_read32(state);
    (*str).sector = SectorId(sector as u32);
    (*str).minlight = saveg_read32(state);
    (*str).maxlight = saveg_read32(state);
    (*str).direction = saveg_read32(state);
}
unsafe fn saveg_write_glow_t(state: &mut GameState, mut str: *mut glow_t) {
    saveg_write_thinker_t(state, &raw mut (*str).thinker);
    saveg_write32(state, (*str).sector.0 as i32);
    saveg_write32(state, (*str).minlight);
    saveg_write32(state, (*str).maxlight);
    saveg_write32(state, (*str).direction);
}
pub unsafe fn P_WriteSaveGameHeader(state: &mut GameState, description: &str) {
    let mut i: i32 = 0;
    for &b in description.as_bytes() {
        saveg_write8(state, b);
        i += 1;
    }
    while i < SAVESTRINGSIZE {
        saveg_write8(state, 0 as byte);
        i += 1;
    }
    let name = format!("version {}", G_VanillaVersionCode(&mut state.doomstat));
    let mut name_bytes = [0u8; 16];
    let copy_len = name.len().min(16);
    name_bytes[..copy_len].copy_from_slice(&name.as_bytes()[..copy_len]);
    i = 0 as i32;
    while i < VERSIONSIZE {
        saveg_write8(state, name_bytes[i as usize]);
        i += 1;
    }
    saveg_write8(state, state.g_game.gameskill as byte);
    saveg_write8(state, state.g_game.gameepisode as byte);
    saveg_write8(state, state.g_game.gamemap as byte);
    i = 0 as i32;
    while i < MAXPLAYERS {
        saveg_write8(state, state.g_game.playeringame[i as usize] as byte);
        i += 1;
    }
    saveg_write8(
        state,
        (state.p_tick.leveltime >> 16 as i32 & 0xff as i32) as byte,
    );
    saveg_write8(
        state,
        (state.p_tick.leveltime >> 8 as i32 & 0xff as i32) as byte,
    );
    saveg_write8(state, (state.p_tick.leveltime & 0xff as i32) as byte);
}
pub unsafe fn P_ReadSaveGameHeader(state: &mut GameState) -> bool {
    let mut i: i32 = 0;
    let mut a: byte = 0;
    let mut b: byte = 0;
    let mut c: byte = 0;
    let mut read_vcheck: [u8; 16] = [0; 16];
    i = 0 as i32;
    while i < SAVESTRINGSIZE {
        saveg_read8(state);
        i += 1;
    }
    i = 0 as i32;
    while i < VERSIONSIZE {
        read_vcheck[i as usize] = saveg_read8(state);
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
    state.g_game.gameskill = saveg_read8(state) as skill_t;
    state.g_game.gameepisode = saveg_read8(state) as i32;
    state.g_game.gamemap = saveg_read8(state) as i32;
    i = 0 as i32;
    while i < MAXPLAYERS {
        state.g_game.playeringame[i as usize] = saveg_read8(state) as boolean;
        i += 1;
    }
    a = saveg_read8(state);
    b = saveg_read8(state);
    c = saveg_read8(state);
    state.p_tick.leveltime = ((a as i32) << 16 as i32) + ((b as i32) << 8 as i32) + c as i32;
    return true;
}
pub unsafe fn P_ReadSaveGameEOF(state: &mut GameState) -> bool {
    let mut value: i32 = 0;
    value = saveg_read8(state) as i32;
    return value == SAVEGAME_EOF;
}
pub unsafe fn P_WriteSaveGameEOF(state: &mut GameState) {
    saveg_write8(state, SAVEGAME_EOF as byte);
}
pub unsafe fn P_ArchivePlayers(state: &mut GameState) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < MAXPLAYERS {
        if !(state.g_game.playeringame[i as usize] == 0) {
            saveg_write_pad(state);
            let player = (&raw mut state.g_game.players as *mut player_t).offset(i as isize)
                as *mut player_t;
            saveg_write_player_t(state, player);
        }
        i += 1;
    }
}
pub unsafe fn P_UnArchivePlayers(state: &mut GameState) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < MAXPLAYERS {
        if !(state.g_game.playeringame[i as usize] == 0) {
            saveg_read_pad(state);
            let player = (&raw mut state.g_game.players as *mut player_t).offset(i as isize)
                as *mut player_t;
            saveg_read_player_t(state, player);
            state.g_game.players[i as usize].mo = ::core::ptr::null_mut::<mobj_t>();
            state.g_game.players[i as usize].message = None;
            state.g_game.players[i as usize].attacker = None;
        }
        i += 1;
    }
}
pub unsafe fn P_ArchiveWorld(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut li: *mut line_t = ::core::ptr::null_mut::<line_t>();
    let mut si: *mut side_t = ::core::ptr::null_mut::<side_t>();
    i = 0 as i32;
    while i < state.p_setup.numsectors {
        sec = state.p_setup.sector_mut(SectorId(i as u32));
        saveg_write16(state, ((*sec).floorheight >> FRACBITS) as i16);
        saveg_write16(state, ((*sec).ceilingheight >> FRACBITS) as i16);
        saveg_write16(state, (*sec).floorpic);
        saveg_write16(state, (*sec).ceilingpic);
        saveg_write16(state, (*sec).lightlevel);
        saveg_write16(state, (*sec).special);
        saveg_write16(state, (*sec).tag);
        i += 1;
    }
    i = 0 as i32;
    li = state.p_setup.lines.as_mut_ptr();
    while i < state.p_setup.numlines {
        saveg_write16(state, (*li).flags);
        saveg_write16(state, (*li).special);
        saveg_write16(state, (*li).tag);
        j = 0 as i32;
        while j < 2 as i32 {
            if !((*li).sidenum[j as usize] as i32 == -(1 as i32)) {
                si = state.p_setup.side_mut(SideId(
                    *(&raw mut (*li).sidenum as *mut i16).offset(j as isize) as u32,
                ));
                saveg_write16(state, ((*si).textureoffset >> FRACBITS) as i16);
                saveg_write16(state, ((*si).rowoffset >> FRACBITS) as i16);
                saveg_write16(state, (*si).toptexture);
                saveg_write16(state, (*si).bottomtexture);
                saveg_write16(state, (*si).midtexture);
            }
            j += 1;
        }
        i += 1;
        li = li.offset(1);
    }
}
pub unsafe fn P_UnArchiveWorld(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut sec: *mut sector_t = ::core::ptr::null_mut::<sector_t>();
    let mut li: *mut line_t = ::core::ptr::null_mut::<line_t>();
    let mut si: *mut side_t = ::core::ptr::null_mut::<side_t>();
    i = 0 as i32;
    while i < state.p_setup.numsectors {
        sec = state.p_setup.sector_mut(SectorId(i as u32));
        (*sec).floorheight = ((saveg_read16(state) as i32) << FRACBITS) as fixed_t;
        (*sec).ceilingheight = ((saveg_read16(state) as i32) << FRACBITS) as fixed_t;
        (*sec).floorpic = saveg_read16(state);
        (*sec).ceilingpic = saveg_read16(state);
        (*sec).lightlevel = saveg_read16(state);
        (*sec).special = saveg_read16(state);
        (*sec).tag = saveg_read16(state);
        (*sec).specialdata = None;
        (*sec).soundtarget = None;
        i += 1;
    }
    i = 0 as i32;
    li = state.p_setup.lines.as_mut_ptr();
    while i < state.p_setup.numlines {
        (*li).flags = saveg_read16(state);
        (*li).special = saveg_read16(state);
        (*li).tag = saveg_read16(state);
        j = 0 as i32;
        while j < 2 as i32 {
            if !((*li).sidenum[j as usize] as i32 == -(1 as i32)) {
                si = state.p_setup.side_mut(SideId(
                    *(&raw mut (*li).sidenum as *mut i16).offset(j as isize) as u32,
                ));
                (*si).textureoffset = ((saveg_read16(state) as i32) << FRACBITS) as fixed_t;
                (*si).rowoffset = ((saveg_read16(state) as i32) << FRACBITS) as fixed_t;
                (*si).toptexture = saveg_read16(state);
                (*si).bottomtexture = saveg_read16(state);
                (*si).midtexture = saveg_read16(state);
            }
            j += 1;
        }
        i += 1;
        li = li.offset(1);
    }
}
pub unsafe fn P_ArchiveThinkers(state: &mut GameState) {
    let mut th: *mut thinker_t = ::core::ptr::null_mut::<thinker_t>();
    th = state.p_tick.thinkercap.next as *mut thinker_t;
    while th != &raw mut state.p_tick.thinkercap {
        if matches!((*th).function, ThinkerFn::Mobj(_)) {
            saveg_write8(state, tc_mobj as i32 as byte);
            saveg_write_pad(state);
            saveg_write_mobj_t(state, th as *mut mobj_t);
        }
        th = (*th).next as *mut thinker_t;
    }
    saveg_write8(state, tc_end as i32 as byte);
}
pub unsafe fn P_UnArchiveThinkers(state: &mut GameState) {
    let mut tclass: byte = 0;
    let mut currentthinker: *mut thinker_t = ::core::ptr::null_mut::<thinker_t>();
    let mut next: *mut thinker_t = ::core::ptr::null_mut::<thinker_t>();
    let mut mobj: *mut mobj_t = ::core::ptr::null_mut::<mobj_t>();
    currentthinker = state.p_tick.thinkercap.next as *mut thinker_t;
    while currentthinker != &raw mut state.p_tick.thinkercap {
        next = (*currentthinker).next as *mut thinker_t;
        if matches!((*currentthinker).function, ThinkerFn::Mobj(_)) {
            P_RemoveMobj(state, currentthinker as *mut mobj_t);
        } else {
            Z_Free(
                &mut state.z_zone,
                currentthinker as *mut ::core::ffi::c_void,
            );
        }
        currentthinker = next;
    }
    P_InitThinkers(state);
    loop {
        tclass = saveg_read8(state);
        match tclass as i32 {
            0 => return,
            1 => {
                saveg_read_pad(state);
                mobj = Z_Malloc(
                    &mut state.z_zone,
                    ::core::mem::size_of::<mobj_t>() as i32,
                    PU_LEVEL as i32,
                    NULL,
                ) as *mut mobj_t;
                saveg_read_mobj_t(state, mobj);
                (*mobj).id = state.p_mobj.register(mobj);
                (*mobj).target = None;
                (*mobj).tracer = None;
                P_SetThingPosition(state, mobj);
                (*mobj).floorz = (*state
                    .p_setup
                    .sector_mut(state.p_setup.subsectors[(*mobj).subsector.0 as usize].sector))
                .floorheight;
                (*mobj).ceilingz = (*state
                    .p_setup
                    .sector_mut(state.p_setup.subsectors[(*mobj).subsector.0 as usize].sector))
                .ceilingheight;
                (*mobj).thinker.function = ThinkerFn::Mobj(P_MobjThinker);
                P_AddThinker(state, &raw mut (*mobj).thinker);
            }
            _ => {
                I_Error(&format!("Unknown tclass {} in savegame", tclass as i32,));
            }
        }
    }
}
#[no_mangle]
pub static specials_e: C2RustUnnamed_5 = tc_ceiling;
pub unsafe fn P_ArchiveSpecials(state: &mut GameState) {
    let mut th: *mut thinker_t = ::core::ptr::null_mut::<thinker_t>();
    let mut i: i32 = 0;
    th = state.p_tick.thinkercap.next as *mut thinker_t;
    while th != &raw mut state.p_tick.thinkercap {
        match (*th).function {
            ThinkerFn::Paused => {
                i = 0 as i32;
                while i < MAXCEILINGS {
                    if state.p_ceilng.activeceilings[i as usize] == th as *mut ceiling_t {
                        break;
                    }
                    i += 1;
                }
                if i < MAXCEILINGS {
                    saveg_write8(state, tc_ceiling as i32 as byte);
                    saveg_write_pad(state);
                    saveg_write_ceiling_t(state, th as *mut ceiling_t);
                }
            }
            ThinkerFn::Ceiling(_) => {
                saveg_write8(state, tc_ceiling as i32 as byte);
                saveg_write_pad(state);
                saveg_write_ceiling_t(state, th as *mut ceiling_t);
            }
            ThinkerFn::Door(_) => {
                saveg_write8(state, tc_door as i32 as byte);
                saveg_write_pad(state);
                saveg_write_vldoor_t(state, th as *mut vldoor_t);
            }
            ThinkerFn::Floor(_) => {
                saveg_write8(state, tc_floor as i32 as byte);
                saveg_write_pad(state);
                saveg_write_floormove_t(state, th as *mut floormove_t);
            }
            ThinkerFn::Plat(_) => {
                saveg_write8(state, tc_plat as i32 as byte);
                saveg_write_pad(state);
                saveg_write_plat_t(state, th as *mut plat_t);
            }
            ThinkerFn::LightFlash(_) => {
                saveg_write8(state, tc_flash as i32 as byte);
                saveg_write_pad(state);
                saveg_write_lightflash_t(state, th as *mut lightflash_t);
            }
            ThinkerFn::Strobe(_) => {
                saveg_write8(state, tc_strobe as i32 as byte);
                saveg_write_pad(state);
                saveg_write_strobe_t(state, th as *mut strobe_t);
            }
            ThinkerFn::Glow(_) => {
                saveg_write8(state, tc_glow as i32 as byte);
                saveg_write_pad(state);
                saveg_write_glow_t(state, th as *mut glow_t);
            }
            _ => {}
        }
        th = (*th).next as *mut thinker_t;
    }
    saveg_write8(state, tc_endspecials as i32 as byte);
}
pub unsafe fn P_UnArchiveSpecials(state: &mut GameState) {
    let mut tclass: byte = 0;
    let mut ceiling: *mut ceiling_t = ::core::ptr::null_mut::<ceiling_t>();
    let mut door: *mut vldoor_t = ::core::ptr::null_mut::<vldoor_t>();
    let mut floor: *mut floormove_t = ::core::ptr::null_mut::<floormove_t>();
    let mut plat: *mut plat_t = ::core::ptr::null_mut::<plat_t>();
    let mut flash: *mut lightflash_t = ::core::ptr::null_mut::<lightflash_t>();
    let mut strobe: *mut strobe_t = ::core::ptr::null_mut::<strobe_t>();
    let mut glow: *mut glow_t = ::core::ptr::null_mut::<glow_t>();
    loop {
        tclass = saveg_read8(state);
        match tclass as i32 {
            7 => return,
            0 => {
                saveg_read_pad(state);
                ceiling = Z_Malloc(
                    &mut state.z_zone,
                    ::core::mem::size_of::<ceiling_t>() as i32,
                    PU_LEVEL as i32,
                    NULL,
                ) as *mut ceiling_t;
                saveg_read_ceiling_t(state, ceiling);
                (*state.p_setup.sector_mut((*ceiling).sector)).specialdata =
                    Some(SectorSpecial::Ceiling(ceiling));
                if matches!((*ceiling).thinker.function, ThinkerFn::Unresolved) {
                    (*ceiling).thinker.function = ThinkerFn::Ceiling(T_MoveCeiling);
                }
                P_AddThinker(state, &raw mut (*ceiling).thinker);
                P_AddActiveCeiling(&mut state.p_ceilng, ceiling);
            }
            1 => {
                saveg_read_pad(state);
                door = Z_Malloc(
                    &mut state.z_zone,
                    ::core::mem::size_of::<vldoor_t>() as i32,
                    PU_LEVEL as i32,
                    NULL,
                ) as *mut vldoor_t;
                saveg_read_vldoor_t(state, door);
                (*state.p_setup.sector_mut((*door).sector)).specialdata =
                    Some(SectorSpecial::Door(door));
                (*door).thinker.function = ThinkerFn::Door(T_VerticalDoor);
                P_AddThinker(state, &raw mut (*door).thinker);
            }
            2 => {
                saveg_read_pad(state);
                floor = Z_Malloc(
                    &mut state.z_zone,
                    ::core::mem::size_of::<floormove_t>() as i32,
                    PU_LEVEL as i32,
                    NULL,
                ) as *mut floormove_t;
                saveg_read_floormove_t(state, floor);
                (*state.p_setup.sector_mut((*floor).sector)).specialdata =
                    Some(SectorSpecial::Floor(floor));
                (*floor).thinker.function = ThinkerFn::Floor(T_MoveFloor);
                P_AddThinker(state, &raw mut (*floor).thinker);
            }
            3 => {
                saveg_read_pad(state);
                plat = Z_Malloc(
                    &mut state.z_zone,
                    ::core::mem::size_of::<plat_t>() as i32,
                    PU_LEVEL as i32,
                    NULL,
                ) as *mut plat_t;
                saveg_read_plat_t(state, plat);
                (*state.p_setup.sector_mut((*plat).sector)).specialdata =
                    Some(SectorSpecial::Plat(plat));
                if matches!((*plat).thinker.function, ThinkerFn::Unresolved) {
                    (*plat).thinker.function = ThinkerFn::Plat(T_PlatRaise);
                }
                P_AddThinker(state, &raw mut (*plat).thinker);
                P_AddActivePlat(&mut state.p_plats, plat);
            }
            4 => {
                saveg_read_pad(state);
                flash = Z_Malloc(
                    &mut state.z_zone,
                    ::core::mem::size_of::<lightflash_t>() as i32,
                    PU_LEVEL as i32,
                    NULL,
                ) as *mut lightflash_t;
                saveg_read_lightflash_t(state, flash);
                (*flash).thinker.function = ThinkerFn::LightFlash(T_LightFlash);
                P_AddThinker(state, &raw mut (*flash).thinker);
            }
            5 => {
                saveg_read_pad(state);
                strobe = Z_Malloc(
                    &mut state.z_zone,
                    ::core::mem::size_of::<strobe_t>() as i32,
                    PU_LEVEL as i32,
                    NULL,
                ) as *mut strobe_t;
                saveg_read_strobe_t(state, strobe);
                (*strobe).thinker.function = ThinkerFn::Strobe(T_StrobeFlash);
                P_AddThinker(state, &raw mut (*strobe).thinker);
            }
            6 => {
                saveg_read_pad(state);
                glow = Z_Malloc(
                    &mut state.z_zone,
                    ::core::mem::size_of::<glow_t>() as i32,
                    PU_LEVEL as i32,
                    NULL,
                ) as *mut glow_t;
                saveg_read_glow_t(state, glow);
                (*glow).thinker.function = ThinkerFn::Glow(T_Glow);
                P_AddThinker(state, &raw mut (*glow).thinker);
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
