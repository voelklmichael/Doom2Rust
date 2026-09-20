use crate::d_main::DMainState;
use crate::d_mode::skill_from_raw;
use crate::d_player::CheatFlags;
use crate::d_player::NUMPOWERS;
use crate::d_player::NUMPSPRITES;
use crate::d_player::{weapontype_from_raw, NUMWEAPONS};
use crate::d_player::{Player, PlayerId, PlayerState};
use crate::d_ticcmd::TicCmd;
use crate::g_game::vanilla_version_code;
use crate::g_game::GGameState;
use crate::game_state::World;
use crate::i_system::error;
use crate::info::StateId;
use crate::m_fixed::Fixed;
use crate::p_ceilng::add_active_ceiling;
use crate::p_ceilng::CeilingE;
use crate::p_doors::VlDoor;
use crate::p_doors::VldoorE;
use crate::p_floor::FloorE;
use crate::p_lights::{Glow, LightFlash, Strobe};
use crate::p_maputl::set_thing_position;
use crate::p_mobj::mobjtype_from_raw;
use crate::p_mobj::remove_mobj;
use crate::p_mobj::spritenum_from_raw;
use crate::p_mobj::{LineFlags, MobjFlags};
use crate::p_mobj::{MapThing, SectorSpecial, Thinker, ThinkerFn};
use crate::p_mobj::{Mobj, PspDef};
use crate::p_plats::add_active_plat;
use crate::p_plats::PlatE;
use crate::p_plats::PlattypeE;
use crate::p_setup::PSetupState;
use crate::p_setup::SectorId;
use crate::p_setup::SideId;
use crate::p_setup::SubsectorId;
use crate::p_spec::Direction;
use crate::p_spec::{Ceiling, FloorMove, Plat};
use crate::p_tick::add_thinker;
use crate::p_tick::init_thinkers;
use crate::p_tick::thinker_function;
use crate::p_tick::ThinkerKind;
use crate::p_tick::ThinkerPayload;
use crate::platform::DoomPlatform;
use crate::tables::Angle;
use alloc::string::String;
use alloc::vec::Vec;

use crate::d_player::NUMAMMO;
use crate::doomdef::MAXPLAYERS;
use crate::game_state::GameState;
use crate::m_fixed::FRACBITS;
use crate::m_menu::SAVESTRINGSIZE;
use crate::p_ceilng::move_ceiling;
use crate::p_ceilng::MAXCEILINGS;
use crate::p_doors::t_vertical_door;
use crate::p_floor::move_floor;
use crate::p_inter::NUMCARDS;
use crate::p_lights::{glow, light_flash, strobe_flash};
use crate::p_mobj::mobj_thinker;
use crate::p_plats::plat_raise;

pub struct PSavegState {
    /// The savegame image: read from disk in full before a load, and built up
    /// in memory before being written out by a save.
    pub save_buffer: Vec<u8>,
    /// Read cursor into `save_buffer` (also the write position while saving).
    pub save_pos: usize,
    pub savegame_error: bool,
    pub temp_savegame_filename: Option<String>,
}

impl Default for PSavegState {
    fn default() -> Self {
        Self::new()
    }
}

impl PSavegState {
    pub const fn new() -> Self {
        Self {
            save_buffer: Vec::new(),
            save_pos: 0,
            savegame_error: false,
            temp_savegame_filename: None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ThinkerClass {
    End,
    Mobj,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SpecialThinkerClass {
    Ceiling,
    Door,
    Floor,
    Plat,
    Flash,
    Strobe,
    Glow,
    Endspecials,
}
pub const SAVEGAME_EOF: u8 = 0x1d;
pub const VERSIONSIZE: usize = 16;
pub fn temp_save_game_file(d_main: &DMainState, p_saveg: &mut PSavegState) -> String {
    if p_saveg.temp_savegame_filename.is_none() {
        p_saveg.temp_savegame_filename = Some(format!("{}temp.dsg", d_main.savegamedir));
    }
    p_saveg.temp_savegame_filename.clone().unwrap()
}
pub fn save_game_file(d_main: &DMainState, slot: i32) -> String {
    format!("{}doomsav{}.dsg", d_main.savegamedir, slot)
}
/// Prints the deferred "ran off the end of the file" diagnostic, if a
/// `saveg_read8` hit one; call once when a load finishes or is abandoned.
pub fn report_save_game_read_error(p_saveg: &PSavegState, platform: &mut dyn DoomPlatform) {
    if p_saveg.savegame_error {
        doom_eprintln!(
            platform,
            "saveg_read8: Unexpected end of file while reading save game"
        );
    }
}
fn saveg_read8(state: &mut PSavegState) -> u8 {
    if let Some(&b) = state.save_buffer.get(state.save_pos) {
        state.save_pos += 1;
        b
    } else {
        state.savegame_error = true;
        0
    }
}
fn saveg_write8(state: &mut PSavegState, value: u8) {
    state.save_buffer.push(value);
    state.save_pos += 1;
}
fn saveg_read16(state: &mut PSavegState) -> i16 {
    let mut result: i32 = i32::from(saveg_read8(state));
    result |= i32::from(saveg_read8(state)) << 8;
    result as i16
}
fn saveg_write16(state: &mut PSavegState, value: i16) {
    saveg_write8(state, (i32::from(value) & 0xff) as u8);
    saveg_write8(state, (i32::from(value) >> 8 & 0xff) as u8);
}
fn saveg_read32(state: &mut PSavegState) -> i32 {
    let mut result: i32 = i32::from(saveg_read8(state));
    result |= i32::from(saveg_read8(state)) << 8;
    result |= i32::from(saveg_read8(state)) << 16;
    result |= i32::from(saveg_read8(state)) << 24;
    result
}
fn saveg_write32(state: &mut PSavegState, value: i32) {
    saveg_write8(state, (value & 0xff) as u8);
    saveg_write8(state, (value >> 8 & 0xff) as u8);
    saveg_write8(state, (value >> 16 & 0xff) as u8);
    saveg_write8(state, (value >> 24 & 0xff) as u8);
}
fn saveg_read_pad(state: &mut PSavegState) {
    let padding: i32 = (4_usize.wrapping_sub(state.save_pos & 3) & 3) as i32;
    for _ in 0..padding {
        saveg_read8(state);
    }
}
fn saveg_write_pad(state: &mut PSavegState) {
    let padding: i32 = (4_usize.wrapping_sub(state.save_pos & 3) & 3) as i32;
    for _ in 0..padding {
        saveg_write8(state, 0_u8);
    }
}
// Vanilla wrote raw pointers here. Loading only ever checks them for null, so
// a presence flag is written instead: it keeps the format, loads vanilla
// saves (any non-zero word means "present") and makes the files reproducible
// rather than leaking process addresses.
fn saveg_read_present(state: &mut PSavegState) -> bool {
    saveg_read32(state) != 0
}
fn saveg_write_present(state: &mut PSavegState, present: bool) {
    saveg_write32(state, i32::from(present));
}
fn saveg_read_mapthing_t(state: &mut PSavegState, str: &mut MapThing) {
    str.x = saveg_read16(state);
    str.y = saveg_read16(state);
    str.angle = saveg_read16(state);
    str.kind = saveg_read16(state);
    str.options = saveg_read16(state);
}
fn saveg_write_mapthing_t(state: &mut PSavegState, str: &MapThing) {
    saveg_write16(state, str.x);
    saveg_write16(state, str.y);
    saveg_write16(state, str.angle);
    saveg_write16(state, str.kind);
    saveg_write16(state, str.options);
}
fn saveg_read_actionf_t(state: &mut PSavegState, str: &mut ThinkerFn) {
    *str = if saveg_read_present(state) {
        ThinkerFn::Unresolved
    } else {
        ThinkerFn::Paused
    };
}
fn saveg_write_actionf_t(state: &mut PSavegState, str: &ThinkerFn) {
    saveg_write_present(state, !matches!(*str, ThinkerFn::Paused));
}
fn saveg_read_thinker_t(state: &mut PSavegState, str: &mut Thinker) {
    // add_thinker (called after every payload type is reconstructed, in
    // both un_archive_thinkers and un_archive_specials below) always rebuilds
    // prev/next from scratch in PTickState's own node table, so these
    // on-disk bytes are already dead -- discard.
    saveg_read32(state);
    saveg_read32(state);
    saveg_read_actionf_t(state, &mut str.function);
}
fn saveg_write_thinker_t(state: &mut PSavegState, str: &Thinker) {
    saveg_write32(state, 0);
    saveg_write32(state, 0);
    saveg_write_actionf_t(state, &str.function);
}
fn saveg_read_mobj_t(state: &mut PSavegState, str: &mut Mobj) {
    saveg_read_thinker_t(state, &mut str.thinker);
    str.x = saveg_read32(state) as Fixed;
    str.y = saveg_read32(state) as Fixed;
    str.z = saveg_read32(state) as Fixed;
    // set_thing_position (called on every reconstructed mobj right after this,
    // see un_archive_thinkers) fully rebuilds snext/sprev from scratch, so
    // these on-disk bytes are already dead -- discard, same treatment as
    // target/tracer's inert reads just below.
    saveg_read32(state);
    str.snext = None;
    saveg_read32(state);
    str.sprev = None;
    str.angle = Angle((saveg_read32(state)) as u32);
    str.sprite = spritenum_from_raw(saveg_read32(state));
    str.frame = saveg_read32(state);
    // set_thing_position also fully rebuilds bnext/bprev from scratch --
    // same dead-bytes treatment as snext/sprev above.
    saveg_read32(state);
    str.bnext = None;
    saveg_read32(state);
    str.bprev = None;
    saveg_read32(state);
    str.subsector = SubsectorId(0);
    str.floorz = saveg_read32(state) as Fixed;
    str.ceilingz = saveg_read32(state) as Fixed;
    str.radius = saveg_read32(state) as Fixed;
    str.height = saveg_read32(state) as Fixed;
    str.momx = saveg_read32(state) as Fixed;
    str.momy = saveg_read32(state) as Fixed;
    str.momz = saveg_read32(state) as Fixed;
    str.validcount = saveg_read32(state);
    str.kind = mobjtype_from_raw(saveg_read32(state));
    saveg_read32(state);
    str.tics = saveg_read32(state);
    str.state = Some(StateId(saveg_read32(state) as u32));
    str.flags = MobjFlags::from_bits_retain(saveg_read32(state));
    str.health = saveg_read32(state);
    str.movedir = saveg_read32(state);
    str.movecount = saveg_read32(state);
    saveg_read32(state);
    str.target = None;
    str.reactiontime = saveg_read32(state);
    str.threshold = saveg_read32(state);
    let pl: i32 = saveg_read32(state);
    if pl > 0 {
        let player_id = PlayerId((pl - 1) as u8);
        str.player = Some(player_id);
    } else {
        str.player = None;
    }
    str.lastlook = PlayerId(saveg_read32(state) as u8 % MAXPLAYERS as u8);
    saveg_read_mapthing_t(state, &mut str.spawnpoint);
    saveg_read32(state);
    str.tracer = None;
}
fn saveg_write_mobj_t(state: &mut PSavegState, str: &Mobj) {
    saveg_write_thinker_t(state, &str.thinker);
    saveg_write32(state, str.x);
    saveg_write32(state, str.y);
    saveg_write32(state, str.z);
    saveg_write32(state, 0);
    saveg_write32(state, 0);
    saveg_write32(state, (str.angle).to_signed());
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
    saveg_write32(state, str.kind as i32);
    saveg_write32(state, 0);
    saveg_write32(state, str.tics);
    saveg_write32(state, str.state.unwrap().0 as i32);
    saveg_write32(state, str.flags.bits());
    saveg_write32(state, str.health);
    saveg_write32(state, str.movedir);
    saveg_write32(state, str.movecount);
    saveg_write32(state, 0);
    saveg_write32(state, str.reactiontime);
    saveg_write32(state, str.threshold);
    if let Some(player_id) = str.player {
        saveg_write32(state, i32::from(player_id.0) + 1);
    } else {
        saveg_write32(state, 0);
    }
    saveg_write32(state, str.lastlook.as_i32());
    saveg_write_mapthing_t(state, &str.spawnpoint);
    saveg_write32(state, 0);
}
fn saveg_read_ticcmd_t(state: &mut PSavegState, str: &mut TicCmd) {
    str.forwardmove = saveg_read8(state) as i8;
    str.sidemove = saveg_read8(state) as i8;
    str.angleturn = saveg_read16(state);
    str.consistancy = saveg_read16(state) as u8;
    str.chatchar = saveg_read8(state);
    str.buttons = saveg_read8(state);
}
fn saveg_write_ticcmd_t(state: &mut PSavegState, str: &TicCmd) {
    saveg_write8(state, str.forwardmove as u8);
    saveg_write8(state, str.sidemove as u8);
    saveg_write16(state, str.angleturn);
    saveg_write16(state, i16::from(str.consistancy));
    saveg_write8(state, str.chatchar);
    saveg_write8(state, str.buttons);
}
fn saveg_read_pspdef_t(state: &mut PSavegState, str: &mut PspDef) {
    let state_num: i32 = saveg_read32(state);
    if state_num > 0 {
        str.state = Some(StateId(state_num as u32));
    } else {
        str.state = None;
    }
    str.tics = saveg_read32(state);
    str.sx = saveg_read32(state) as Fixed;
    str.sy = saveg_read32(state) as Fixed;
}
fn saveg_write_pspdef_t(state: &mut PSavegState, str: &PspDef) {
    if let Some(state_id) = str.state {
        saveg_write32(state, state_id.0 as i32);
    } else {
        saveg_write32(state, 0);
    }
    saveg_write32(state, str.tics);
    saveg_write32(state, str.sx);
    saveg_write32(state, str.sy);
}
fn saveg_read_player_t(state: &mut PSavegState, str: &mut Player) {
    // Placeholder value, discarded -- see saveg_write_player_t.
    saveg_read_present(state);
    str.playerstate = match saveg_read32(state) {
        0 => PlayerState::Live,
        1 => PlayerState::Dead,
        2 => PlayerState::Reborn,
        n => panic!("P_UnArchivePlayers: invalid playerstate {n} in savegame"),
    };
    saveg_read_ticcmd_t(state, &mut str.cmd);
    str.viewz = saveg_read32(state) as Fixed;
    str.viewheight = saveg_read32(state) as Fixed;
    str.deltaviewheight = saveg_read32(state) as Fixed;
    str.bob = saveg_read32(state) as Fixed;
    str.health = saveg_read32(state);
    str.armorpoints = saveg_read32(state);
    str.armortype = saveg_read32(state);
    for i in 0..NUMPOWERS {
        str.powers[i] = saveg_read32(state);
    }
    for i in 0..NUMCARDS {
        str.cards[i] = saveg_read32(state) != 0;
    }
    str.backpack = saveg_read32(state) != 0;
    for i in 0..MAXPLAYERS {
        str.frags[i] = saveg_read32(state);
    }
    str.readyweapon = weapontype_from_raw(saveg_read32(state));
    str.pendingweapon = weapontype_from_raw(saveg_read32(state));
    for i in 0..NUMWEAPONS {
        str.weaponowned[i] = saveg_read32(state) != 0;
    }
    for i in 0..NUMAMMO {
        str.ammo[i] = saveg_read32(state);
    }
    for i in 0..NUMAMMO {
        str.maxammo[i] = saveg_read32(state);
    }
    str.attackdown = saveg_read32(state) != 0;
    str.usedown = saveg_read32(state) != 0;
    str.cheats = CheatFlags::from_bits_retain(saveg_read32(state));
    str.refire = saveg_read32(state);
    str.killcount = saveg_read32(state);
    str.itemcount = saveg_read32(state);
    str.secretcount = saveg_read32(state);
    saveg_read_present(state);
    str.message = None;
    str.damagecount = saveg_read32(state);
    str.bonuscount = saveg_read32(state);
    saveg_read32(state);
    str.attacker = None;
    str.extralight = saveg_read32(state);
    str.fixedcolormap = saveg_read32(state);
    str.colormap = saveg_read32(state);
    for i in 0..NUMPSPRITES {
        saveg_read_pspdef_t(state, &mut str.psprites[i]);
    }
    str.didsecret = saveg_read32(state) != 0;
}
fn saveg_write_player_t(state: &mut PSavegState, str: &Player) {
    // The written value is a placeholder: on load it is immediately
    // overwritten with null by un_archive_players and then correctly
    // restored from the mobj's own player backref in un_archive_thinkers.
    saveg_write_present(state, false);
    saveg_write32(state, str.playerstate as i32);
    saveg_write_ticcmd_t(state, &str.cmd);
    saveg_write32(state, str.viewz);
    saveg_write32(state, str.viewheight);
    saveg_write32(state, str.deltaviewheight);
    saveg_write32(state, str.bob);
    saveg_write32(state, str.health);
    saveg_write32(state, str.armorpoints);
    saveg_write32(state, str.armortype);
    for i in 0..NUMPOWERS {
        saveg_write32(state, str.powers[i]);
    }
    for i in 0..NUMCARDS {
        saveg_write32(state, i32::from(str.cards[i]));
    }
    saveg_write32(state, i32::from(str.backpack));
    for i in 0..MAXPLAYERS {
        saveg_write32(state, str.frags[i]);
    }
    saveg_write32(state, str.readyweapon as i32);
    saveg_write32(state, str.pendingweapon as i32);
    for i in 0..NUMWEAPONS {
        saveg_write32(state, i32::from(str.weaponowned[i]));
    }
    for i in 0..NUMAMMO {
        saveg_write32(state, str.ammo[i]);
    }
    for i in 0..NUMAMMO {
        saveg_write32(state, str.maxammo[i]);
    }
    saveg_write32(state, i32::from(str.attackdown));
    saveg_write32(state, i32::from(str.usedown));
    saveg_write32(state, str.cheats.bits());
    saveg_write32(state, str.refire);
    saveg_write32(state, str.killcount);
    saveg_write32(state, str.itemcount);
    saveg_write32(state, str.secretcount);
    saveg_write_present(state, str.message.is_some());
    saveg_write32(state, str.damagecount);
    saveg_write32(state, str.bonuscount);
    saveg_write32(state, 0);
    saveg_write32(state, str.extralight);
    saveg_write32(state, str.fixedcolormap);
    saveg_write32(state, str.colormap);
    for i in 0..NUMPSPRITES {
        saveg_write_pspdef_t(state, &str.psprites[i]);
    }
    saveg_write32(state, i32::from(str.didsecret));
}
fn saveg_read_ceiling_e(state: &mut PSavegState) -> CeilingE {
    match saveg_read32(state) {
        0 => CeilingE::LowerToFloor,
        1 => CeilingE::RaiseToHighest,
        2 => CeilingE::LowerAndCrush,
        3 => CeilingE::CrushAndRaise,
        4 => CeilingE::FastCrushAndRaise,
        5 => CeilingE::SilentCrushAndRaise,
        n => panic!("P_UnArchiveSpecials: invalid ceiling type {n} in savegame"),
    }
}
fn saveg_read_ceiling_t(state: &mut PSavegState, str: &mut Ceiling) {
    saveg_read_thinker_t(state, &mut str.thinker);
    str.kind = saveg_read_ceiling_e(state);
    let sector: i32 = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.bottomheight = saveg_read32(state) as Fixed;
    str.topheight = saveg_read32(state) as Fixed;
    str.speed = saveg_read32(state) as Fixed;
    str.crush = saveg_read32(state) != 0;
    str.direction = Direction::from_save(saveg_read32(state));
    str.tag = saveg_read32(state);
    str.olddirection = Direction::from_save(saveg_read32(state));
}
fn saveg_write_ceiling_t(state: &mut PSavegState, str: &Ceiling) {
    saveg_write_thinker_t(state, &str.thinker);
    saveg_write32(state, str.kind as i32);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.bottomheight);
    saveg_write32(state, str.topheight);
    saveg_write32(state, str.speed);
    saveg_write32(state, i32::from(str.crush));
    saveg_write32(state, str.direction.to_save());
    saveg_write32(state, str.tag);
    saveg_write32(state, str.olddirection.to_save());
}
fn saveg_read_vldoor_e(state: &mut PSavegState) -> VldoorE {
    match saveg_read32(state) {
        0 => VldoorE::Normal,
        1 => VldoorE::Close30ThenOpen,
        2 => VldoorE::Close,
        3 => VldoorE::Open,
        4 => VldoorE::RaiseIn5Mins,
        5 => VldoorE::BlazeRaise,
        6 => VldoorE::BlazeOpen,
        7 => VldoorE::BlazeClose,
        n => panic!("P_UnArchiveSpecials: invalid door type {n} in savegame"),
    }
}
fn saveg_read_vldoor_t(state: &mut PSavegState, str: &mut VlDoor) {
    saveg_read_thinker_t(state, &mut str.thinker);
    str.kind = saveg_read_vldoor_e(state);
    let sector: i32 = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.topheight = saveg_read32(state) as Fixed;
    str.speed = saveg_read32(state) as Fixed;
    str.direction = Direction::from_save(saveg_read32(state));
    str.topwait = saveg_read32(state);
    str.topcountdown = saveg_read32(state);
}
fn saveg_write_vldoor_t(state: &mut PSavegState, str: &VlDoor) {
    saveg_write_thinker_t(state, &str.thinker);
    saveg_write32(state, str.kind as i32);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.topheight);
    saveg_write32(state, str.speed);
    saveg_write32(state, str.direction.to_save());
    saveg_write32(state, str.topwait);
    saveg_write32(state, str.topcountdown);
}
fn saveg_read_floor_e(state: &mut PSavegState) -> FloorE {
    match saveg_read32(state) {
        0 => FloorE::LowerFloor,
        1 => FloorE::LowerFloorToLowest,
        2 => FloorE::TurboLower,
        3 => FloorE::RaiseFloor,
        4 => FloorE::RaiseFloorToNearest,
        5 => FloorE::RaiseToTexture,
        6 => FloorE::LowerAndChange,
        7 => FloorE::RaiseFloor24,
        8 => FloorE::RaiseFloor24AndChange,
        9 => FloorE::RaiseFloorCrush,
        10 => FloorE::RaiseFloorTurbo,
        11 => FloorE::DonutRaise,
        12 => FloorE::RaiseFloor512,
        n => panic!("P_UnArchiveSpecials: invalid floor type {n} in savegame"),
    }
}
fn saveg_read_floormove_t(state: &mut PSavegState, str: &mut FloorMove) {
    saveg_read_thinker_t(state, &mut str.thinker);
    str.kind = saveg_read_floor_e(state);
    str.crush = saveg_read32(state) != 0;
    let sector: i32 = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.direction = Direction::from_save(saveg_read32(state));
    str.newspecial = saveg_read32(state);
    str.texture = saveg_read16(state);
    str.floordestheight = saveg_read32(state) as Fixed;
    str.speed = saveg_read32(state) as Fixed;
}
fn saveg_write_floormove_t(state: &mut PSavegState, str: &FloorMove) {
    saveg_write_thinker_t(state, &str.thinker);
    saveg_write32(state, str.kind as i32);
    saveg_write32(state, i32::from(str.crush));
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.direction.to_save());
    saveg_write32(state, str.newspecial);
    saveg_write16(state, str.texture);
    saveg_write32(state, str.floordestheight);
    saveg_write32(state, str.speed);
}
fn saveg_read_plat_e(state: &mut PSavegState) -> PlatE {
    match saveg_read32(state) {
        0 => PlatE::Up,
        1 => PlatE::Down,
        2 => PlatE::Waiting,
        3 => PlatE::InStasis,
        n => panic!("P_UnArchiveSpecials: invalid plat status {n} in savegame"),
    }
}
fn saveg_read_plattype_e(state: &mut PSavegState) -> PlattypeE {
    match saveg_read32(state) {
        0 => PlattypeE::PerpetualRaise,
        1 => PlattypeE::DownWaitUpStay,
        2 => PlattypeE::RaiseAndChange,
        3 => PlattypeE::RaiseToNearestAndChange,
        4 => PlattypeE::BlazeDWUS,
        n => panic!("P_UnArchiveSpecials: invalid plat type {n} in savegame"),
    }
}
fn saveg_read_plat_t(state: &mut PSavegState, str: &mut Plat) {
    saveg_read_thinker_t(state, &mut str.thinker);
    let sector: i32 = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.speed = saveg_read32(state) as Fixed;
    str.low = saveg_read32(state) as Fixed;
    str.high = saveg_read32(state) as Fixed;
    str.wait = saveg_read32(state);
    str.count = saveg_read32(state);
    str.status = saveg_read_plat_e(state);
    str.oldstatus = saveg_read_plat_e(state);
    str.crush = saveg_read32(state) != 0;
    str.tag = saveg_read32(state);
    str.kind = saveg_read_plattype_e(state);
}
fn saveg_write_plat_t(state: &mut PSavegState, str: &Plat) {
    saveg_write_thinker_t(state, &str.thinker);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.speed);
    saveg_write32(state, str.low);
    saveg_write32(state, str.high);
    saveg_write32(state, str.wait);
    saveg_write32(state, str.count);
    saveg_write32(state, str.status as i32);
    saveg_write32(state, str.oldstatus as i32);
    saveg_write32(state, i32::from(str.crush));
    saveg_write32(state, str.tag);
    saveg_write32(state, str.kind as i32);
}
fn saveg_read_lightflash_t(state: &mut PSavegState, str: &mut LightFlash) {
    saveg_read_thinker_t(state, &mut str.thinker);
    let sector: i32 = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.count = saveg_read32(state);
    str.maxlight = saveg_read32(state);
    str.minlight = saveg_read32(state);
    str.maxtime = saveg_read32(state);
    str.mintime = saveg_read32(state);
}
fn saveg_write_lightflash_t(state: &mut PSavegState, str: &LightFlash) {
    saveg_write_thinker_t(state, &str.thinker);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.count);
    saveg_write32(state, str.maxlight);
    saveg_write32(state, str.minlight);
    saveg_write32(state, str.maxtime);
    saveg_write32(state, str.mintime);
}
fn saveg_read_strobe_t(state: &mut PSavegState, str: &mut Strobe) {
    saveg_read_thinker_t(state, &mut str.thinker);
    let sector: i32 = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.count = saveg_read32(state);
    str.minlight = saveg_read32(state);
    str.maxlight = saveg_read32(state);
    str.darktime = saveg_read32(state);
    str.brighttime = saveg_read32(state);
}
fn saveg_write_strobe_t(state: &mut PSavegState, str: &Strobe) {
    saveg_write_thinker_t(state, &str.thinker);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.count);
    saveg_write32(state, str.minlight);
    saveg_write32(state, str.maxlight);
    saveg_write32(state, str.darktime);
    saveg_write32(state, str.brighttime);
}
fn saveg_read_glow_t(state: &mut PSavegState, str: &mut Glow) {
    saveg_read_thinker_t(state, &mut str.thinker);
    let sector: i32 = saveg_read32(state);
    str.sector = SectorId(sector as u32);
    str.minlight = saveg_read32(state);
    str.maxlight = saveg_read32(state);
    str.direction = Direction::from_save(saveg_read32(state));
}
fn saveg_write_glow_t(state: &mut PSavegState, str: &Glow) {
    saveg_write_thinker_t(state, &str.thinker);
    saveg_write32(state, str.sector.0 as i32);
    saveg_write32(state, str.minlight);
    saveg_write32(state, str.maxlight);
    saveg_write32(state, str.direction.to_save());
}
pub fn write_save_game_header(state: &mut GameState, description: &str) {
    for &b in description.as_bytes() {
        saveg_write8(&mut state.world.p_saveg, b);
    }
    for _ in description.len()..SAVESTRINGSIZE {
        saveg_write8(&mut state.world.p_saveg, 0_u8);
    }
    let name = format!("version {}", vanilla_version_code(&state.game.doomstat));
    let mut name_bytes = [0u8; 16];
    let copy_len = name.len().min(16);
    name_bytes[..copy_len].copy_from_slice(&name.as_bytes()[..copy_len]);
    for &byte in name_bytes.iter().take(VERSIONSIZE) {
        saveg_write8(&mut state.world.p_saveg, byte);
    }
    saveg_write8(&mut state.world.p_saveg, state.game.g_game.gameskill as u8);
    saveg_write8(
        &mut state.world.p_saveg,
        state.game.g_game.gameepisode as u8,
    );
    saveg_write8(&mut state.world.p_saveg, state.game.g_game.gamemap as u8);
    for i in 0..MAXPLAYERS {
        saveg_write8(
            &mut state.world.p_saveg,
            u8::from(state.game.g_game.playeringame[i]),
        );
    }
    saveg_write8(
        &mut state.world.p_saveg,
        (state.world.p_tick.leveltime >> 16 & 0xff) as u8,
    );
    saveg_write8(
        &mut state.world.p_saveg,
        (state.world.p_tick.leveltime >> 8 & 0xff) as u8,
    );
    saveg_write8(
        &mut state.world.p_saveg,
        (state.world.p_tick.leveltime & 0xff) as u8,
    );
}
pub fn read_save_game_header(state: &mut GameState) -> bool {
    fn cstr_prefix(buf: &[u8]) -> &[u8] {
        let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        &buf[..len]
    }
    let mut read_vcheck: [u8; 16] = [0; 16];
    for _ in 0..SAVESTRINGSIZE {
        saveg_read8(&mut state.world.p_saveg);
    }
    for slot in read_vcheck.iter_mut().take(VERSIONSIZE) {
        *slot = saveg_read8(&mut state.world.p_saveg);
    }
    let version_name = format!("version {}", vanilla_version_code(&state.game.doomstat));
    let mut vcheck: [u8; 16] = [0; 16];
    let copy_len = version_name.len().min(16);
    vcheck[..copy_len].copy_from_slice(&version_name.as_bytes()[..copy_len]);
    if cstr_prefix(&read_vcheck) != cstr_prefix(&vcheck) {
        return false;
    }
    state.game.g_game.gameskill = skill_from_raw(i32::from(saveg_read8(&mut state.world.p_saveg)));
    state.game.g_game.gameepisode = i32::from(saveg_read8(&mut state.world.p_saveg));
    state.game.g_game.gamemap = i32::from(saveg_read8(&mut state.world.p_saveg));
    for i in 0..MAXPLAYERS {
        state.game.g_game.playeringame[i] = saveg_read8(&mut state.world.p_saveg) != 0;
    }
    let a: u8 = saveg_read8(&mut state.world.p_saveg);
    let b: u8 = saveg_read8(&mut state.world.p_saveg);
    let c: u8 = saveg_read8(&mut state.world.p_saveg);
    state.world.p_tick.leveltime = (i32::from(a) << 16) + (i32::from(b) << 8) + i32::from(c);
    true
}
pub fn read_save_game_eof(p_saveg: &mut PSavegState) -> bool {
    saveg_read8(p_saveg) == SAVEGAME_EOF
}
pub fn write_save_game_eof(p_saveg: &mut PSavegState) {
    saveg_write8(p_saveg, SAVEGAME_EOF);
}
pub fn archive_players(g_game: &GGameState, p_saveg: &mut PSavegState) {
    for i in 0..MAXPLAYERS {
        if g_game.playeringame[i] {
            saveg_write_pad(p_saveg);
            saveg_write_player_t(p_saveg, &g_game.players[i]);
        }
    }
}
pub fn un_archive_players(g_game: &mut GGameState, p_saveg: &mut PSavegState) {
    for i in 0..MAXPLAYERS {
        if g_game.playeringame[i] {
            saveg_read_pad(p_saveg);
            saveg_read_player_t(p_saveg, &mut g_game.players[i]);
            g_game.players[i].mo = None;
            g_game.players[i].message = None;
            g_game.players[i].attacker = None;
        }
    }
}
pub fn archive_world(p_saveg: &mut PSavegState, p_setup: &mut PSetupState) {
    for i in 0..p_setup.numsectors {
        let sec = p_setup.sector_mut(SectorId(i as u32));
        let (floorheight, ceilingheight, floorpic, ceilingpic, lightlevel, special, tag) = (
            sec.floorheight,
            sec.ceilingheight,
            sec.floorpic,
            sec.ceilingpic,
            sec.lightlevel,
            sec.special,
            sec.tag,
        );
        saveg_write16(p_saveg, (floorheight >> FRACBITS) as i16);
        saveg_write16(p_saveg, (ceilingheight >> FRACBITS) as i16);
        saveg_write16(p_saveg, floorpic);
        saveg_write16(p_saveg, ceilingpic);
        saveg_write16(p_saveg, lightlevel);
        saveg_write16(p_saveg, special);
        saveg_write16(p_saveg, tag);
    }
    for i in 0..(p_setup.numlines as usize) {
        let li = &p_setup.lines[i];
        let (flags, special, tag, sidenum) = (li.flags, li.special, li.tag, li.sidenum);
        saveg_write16(p_saveg, flags.bits());
        saveg_write16(p_saveg, special);
        saveg_write16(p_saveg, tag);
        for &side in &sidenum {
            if i32::from(side) != -1 {
                let si = p_setup.side_mut(SideId(side as u32));
                let (textureoffset, rowoffset, toptexture, bottomtexture, midtexture) = (
                    si.textureoffset,
                    si.rowoffset,
                    si.toptexture,
                    si.bottomtexture,
                    si.midtexture,
                );
                saveg_write16(p_saveg, (textureoffset >> FRACBITS) as i16);
                saveg_write16(p_saveg, (rowoffset >> FRACBITS) as i16);
                saveg_write16(p_saveg, toptexture);
                saveg_write16(p_saveg, bottomtexture);
                saveg_write16(p_saveg, midtexture);
            }
        }
    }
}
pub fn un_archive_world(p_saveg: &mut PSavegState, p_setup: &mut PSetupState) {
    for i in 0..p_setup.numsectors {
        let floorheight = (i32::from(saveg_read16(p_saveg)) << FRACBITS) as Fixed;
        let ceilingheight = (i32::from(saveg_read16(p_saveg)) << FRACBITS) as Fixed;
        let floorpic = saveg_read16(p_saveg);
        let ceilingpic = saveg_read16(p_saveg);
        let lightlevel = saveg_read16(p_saveg);
        let special = saveg_read16(p_saveg);
        let tag = saveg_read16(p_saveg);
        let sec = p_setup.sector_mut(SectorId(i as u32));
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
    for i in 0..(p_setup.numlines as usize) {
        let flags = saveg_read16(p_saveg);
        let special = saveg_read16(p_saveg);
        let tag = saveg_read16(p_saveg);
        let li = &mut p_setup.lines[i];
        li.flags = LineFlags::from_bits_retain(flags);
        li.special = special;
        li.tag = tag;
        let sidenum = li.sidenum;
        for &side in &sidenum {
            if i32::from(side) != -1 {
                let textureoffset = (i32::from(saveg_read16(p_saveg)) << FRACBITS) as Fixed;
                let rowoffset = (i32::from(saveg_read16(p_saveg)) << FRACBITS) as Fixed;
                let toptexture = saveg_read16(p_saveg);
                let bottomtexture = saveg_read16(p_saveg);
                let midtexture = saveg_read16(p_saveg);
                let si = p_setup.side_mut(SideId(side as u32));
                si.textureoffset = textureoffset;
                si.rowoffset = rowoffset;
                si.toptexture = toptexture;
                si.bottomtexture = bottomtexture;
                si.midtexture = midtexture;
            }
        }
    }
}
pub fn archive_thinkers(world: &mut World) {
    let mut cursor = world.p_tick.head();
    while let Some(id) = cursor {
        if let ThinkerPayload::Mobj(mobj_id) = world.p_tick.payload(id) {
            if matches!(thinker_function(world, id), ThinkerFn::Mobj(_)) {
                saveg_write8(&mut world.p_saveg, ThinkerClass::Mobj as i32 as u8);
                saveg_write_pad(&mut world.p_saveg);
                let mobj = world.p_mobj.mobj_mut(mobj_id).expect("live mobj");
                saveg_write_mobj_t(&mut world.p_saveg, mobj);
            }
        }
        cursor = world.p_tick.next(id);
    }
    saveg_write8(&mut world.p_saveg, ThinkerClass::End as i32 as u8);
}
pub fn un_archive_thinkers(state: &mut GameState) {
    let mut cursor = state.world.p_tick.head();
    while let Some(id) = cursor {
        // Unlike the raw-pointer version this replaces, `next` lives in our
        // own node table, not inside the payload memory Z_Free/deallocate
        // below may free -- capturing it first just mirrors the original
        // ordering, not a use-after-free workaround.
        let next = state.world.p_tick.next(id);
        // Dispatch on the node's recorded kind, not `.function` -- every
        // payload type's memory is now owned by its own arena (Mobj and
        // all 8 thinker specials), not the zone allocator, so each needs
        // its own dealloc/deallocate call, mirroring run_thinkers' reaper
        // dispatch exactly. (`.function` is still live/intact at this point
        // for the Mobj case specifically, which is why the original code
        // could match on it directly -- but `kind` works uniformly for all
        // 9 and doesn't depend on that.)
        match state.world.p_tick.kind(id) {
            ThinkerKind::Mobj => {
                if let ThinkerPayload::Mobj(mobj_id) = state.world.p_tick.payload(id) {
                    remove_mobj(state, mobj_id);
                    // remove_mobj only retires (see PMobjState::retire) --
                    // it never itself frees the mobj's memory, and
                    // init_thinkers just below wipes PTickState before
                    // run_thinkers' reaper ever gets a chance to run on
                    // this now-Removed node, so nothing else was ever going
                    // to deallocate it. This call closes that gap (a
                    // pre-existing leak: every live mobj at the moment a
                    // savegame is loaded used to leak its Z_Malloc'd
                    // block).
                    state.world.p_mobj.deallocate(mobj_id);
                }
            }
            ThinkerKind::Door => {
                if let ThinkerPayload::Door(door_id) = state.world.p_tick.payload(id) {
                    state.world.p_doors.dealloc(door_id);
                }
            }
            ThinkerKind::Ceiling => {
                if let ThinkerPayload::Ceiling(ceiling_id) = state.world.p_tick.payload(id) {
                    state.world.p_ceilng.dealloc(ceiling_id);
                }
            }
            ThinkerKind::Plat => {
                if let ThinkerPayload::Plat(plat_id) = state.world.p_tick.payload(id) {
                    state.world.p_plats.dealloc(plat_id);
                }
            }
            ThinkerKind::Floor => {
                if let ThinkerPayload::Floor(floor_id) = state.world.p_tick.payload(id) {
                    state.world.p_spec.dealloc_floor(floor_id);
                }
            }
            ThinkerKind::FireFlicker => {
                if let ThinkerPayload::FireFlicker(fireflicker_id) = state.world.p_tick.payload(id)
                {
                    state.world.p_lights.dealloc_fireflicker(fireflicker_id);
                }
            }
            ThinkerKind::LightFlash => {
                if let ThinkerPayload::LightFlash(lightflash_id) = state.world.p_tick.payload(id) {
                    state.world.p_lights.dealloc_lightflash(lightflash_id);
                }
            }
            ThinkerKind::Strobe => {
                if let ThinkerPayload::Strobe(strobe_id) = state.world.p_tick.payload(id) {
                    state.world.p_lights.dealloc_strobe(strobe_id);
                }
            }
            ThinkerKind::Glow => {
                if let ThinkerPayload::Glow(glow_id) = state.world.p_tick.payload(id) {
                    state.world.p_lights.dealloc_glow(glow_id);
                }
            }
        }
        cursor = next;
    }
    init_thinkers(&mut state.world.p_tick);
    loop {
        let tclass: u8 = saveg_read8(&mut state.world.p_saveg);
        match i32::from(tclass) {
            0 => return,
            1 => {
                saveg_read_pad(&mut state.world.p_saveg);
                // spawn() assigns a fresh MobjId and moves this placeholder
                // onto the heap; saveg_read_mobj_t overwrites every field
                // except `.id` (never part of the on-disk format), so the id
                // spawn() just assigned survives the read untouched below.
                let placeholder = state.world.p_mobj.dummy_mobj;
                let mobj_arena_id = state.world.p_mobj.spawn(placeholder);
                saveg_read_mobj_t(
                    &mut state.world.p_saveg,
                    state.world.p_mobj.mo_mut(mobj_arena_id),
                );
                if let Some(player_id) = state.world.p_mobj.mo(mobj_arena_id).player {
                    state.game.g_game.player_mut(player_id).mo = Some(mobj_arena_id);
                }
                {
                    let m = state.world.p_mobj.mo_mut(mobj_arena_id);
                    m.target = None;
                    m.tracer = None;
                }
                set_thing_position(
                    &mut state.world.p_mobj,
                    &mut state.world.p_setup,
                    mobj_arena_id,
                );
                let subsector = state.world.p_mobj.mo(mobj_arena_id).subsector;
                let sector = state.world.p_setup.subsectors[subsector.0 as usize].sector;
                let (floorheight, ceilingheight) = {
                    let s = state.world.p_setup.sector_mut(sector);
                    (s.floorheight, s.ceilingheight)
                };
                {
                    let m = state.world.p_mobj.mo_mut(mobj_arena_id);
                    m.floorz = floorheight;
                    m.ceilingz = ceilingheight;
                    m.thinker.function = ThinkerFn::Mobj(mobj_thinker);
                }
                add_thinker(
                    &mut state.world.p_tick,
                    ThinkerPayload::Mobj(mobj_arena_id),
                    ThinkerKind::Mobj,
                );
            }
            _ => {
                error(&format!("Unknown tclass {} in savegame", i32::from(tclass),));
            }
        }
    }
}
pub fn archive_specials(world: &mut World) {
    let mut cursor = world.p_tick.head();
    while let Some(id) = cursor {
        match thinker_function(world, id) {
            ThinkerFn::Paused => {
                let in_stasis = world
                    .p_ceilng
                    .activeceilings
                    .iter()
                    .take(MAXCEILINGS)
                    .any(|&entry| entry == Some(id));
                if in_stasis {
                    let ceiling_id = world.p_tick.ceiling_payload(id);
                    saveg_write8(
                        &mut world.p_saveg,
                        SpecialThinkerClass::Ceiling as i32 as u8,
                    );
                    saveg_write_pad(&mut world.p_saveg);
                    let c = world.p_ceilng.get_mut(ceiling_id).expect("live ceiling");
                    saveg_write_ceiling_t(&mut world.p_saveg, c);
                }
            }
            ThinkerFn::Ceiling(_) => {
                let ceiling_id = world.p_tick.ceiling_payload(id);
                saveg_write8(
                    &mut world.p_saveg,
                    SpecialThinkerClass::Ceiling as i32 as u8,
                );
                saveg_write_pad(&mut world.p_saveg);
                let c = world.p_ceilng.get_mut(ceiling_id).expect("live ceiling");
                saveg_write_ceiling_t(&mut world.p_saveg, c);
            }
            ThinkerFn::Door(_) => {
                let door_id = world.p_tick.door_payload(id);
                saveg_write8(&mut world.p_saveg, SpecialThinkerClass::Door as i32 as u8);
                saveg_write_pad(&mut world.p_saveg);
                let d = world.p_doors.get_mut(door_id).expect("live door");
                saveg_write_vldoor_t(&mut world.p_saveg, d);
            }
            ThinkerFn::Floor(_) => {
                let floor_id = world.p_tick.floor_payload(id);
                saveg_write8(&mut world.p_saveg, SpecialThinkerClass::Floor as i32 as u8);
                saveg_write_pad(&mut world.p_saveg);
                let f = world.p_spec.get_floor_mut(floor_id).expect("live floor");
                saveg_write_floormove_t(&mut world.p_saveg, f);
            }
            ThinkerFn::Plat(_) => {
                let plat_id = world.p_tick.plat_payload(id);
                saveg_write8(&mut world.p_saveg, SpecialThinkerClass::Plat as i32 as u8);
                saveg_write_pad(&mut world.p_saveg);
                let p = world.p_plats.get_mut(plat_id).expect("live plat");
                saveg_write_plat_t(&mut world.p_saveg, p);
            }
            ThinkerFn::LightFlash(_) => {
                let ThinkerPayload::LightFlash(flash_id) = world.p_tick.payload(id) else {
                    unreachable!()
                };
                saveg_write8(&mut world.p_saveg, SpecialThinkerClass::Flash as i32 as u8);
                saveg_write_pad(&mut world.p_saveg);
                let f = world
                    .p_lights
                    .get_lightflash_mut(flash_id)
                    .expect("live lightflash");
                saveg_write_lightflash_t(&mut world.p_saveg, f);
            }
            ThinkerFn::Strobe(_) => {
                let ThinkerPayload::Strobe(strobe_id) = world.p_tick.payload(id) else {
                    unreachable!()
                };
                saveg_write8(&mut world.p_saveg, SpecialThinkerClass::Strobe as i32 as u8);
                saveg_write_pad(&mut world.p_saveg);
                let s = world
                    .p_lights
                    .get_strobe_mut(strobe_id)
                    .expect("live strobe");
                saveg_write_strobe_t(&mut world.p_saveg, s);
            }
            ThinkerFn::Glow(_) => {
                let ThinkerPayload::Glow(glow_id) = world.p_tick.payload(id) else {
                    unreachable!()
                };
                saveg_write8(&mut world.p_saveg, SpecialThinkerClass::Glow as i32 as u8);
                saveg_write_pad(&mut world.p_saveg);
                let g = world.p_lights.get_glow_mut(glow_id).expect("live glow");
                saveg_write_glow_t(&mut world.p_saveg, g);
            }
            _ => {}
        }
        cursor = world.p_tick.next(id);
    }
    saveg_write8(
        &mut world.p_saveg,
        SpecialThinkerClass::Endspecials as i32 as u8,
    );
}
pub fn un_archive_specials(world: &mut World) {
    loop {
        let tclass: u8 = saveg_read8(&mut world.p_saveg);
        match i32::from(tclass) {
            7 => return,
            0 => {
                saveg_read_pad(&mut world.p_saveg);
                let ceiling_arena_id = world.p_ceilng.spawn(Ceiling::default());
                let sector = {
                    let c = world
                        .p_ceilng
                        .get_mut(ceiling_arena_id)
                        .expect("live ceiling");
                    saveg_read_ceiling_t(&mut world.p_saveg, c);
                    if matches!(c.thinker.function, ThinkerFn::Unresolved) {
                        c.thinker.function = ThinkerFn::Ceiling(move_ceiling);
                    }
                    c.sector
                };
                let ceiling_id = add_thinker(
                    &mut world.p_tick,
                    ThinkerPayload::Ceiling(ceiling_arena_id),
                    ThinkerKind::Ceiling,
                );
                world.p_setup.sector_mut(sector).specialdata =
                    Some(SectorSpecial::Ceiling(ceiling_id));
                add_active_ceiling(&mut world.p_ceilng, ceiling_id);
            }
            1 => {
                saveg_read_pad(&mut world.p_saveg);
                let door_arena_id = world.p_doors.spawn(VlDoor::default());
                let sector = {
                    let d = world.p_doors.get_mut(door_arena_id).expect("live door");
                    saveg_read_vldoor_t(&mut world.p_saveg, d);
                    d.thinker.function = ThinkerFn::Door(t_vertical_door);
                    d.sector
                };
                let door_id = add_thinker(
                    &mut world.p_tick,
                    ThinkerPayload::Door(door_arena_id),
                    ThinkerKind::Door,
                );
                world.p_setup.sector_mut(sector).specialdata = Some(SectorSpecial::Door(door_id));
            }
            2 => {
                saveg_read_pad(&mut world.p_saveg);
                let floor_arena_id = world.p_spec.spawn_floor(FloorMove::default());
                let sector = {
                    let f = world
                        .p_spec
                        .get_floor_mut(floor_arena_id)
                        .expect("live floor");
                    saveg_read_floormove_t(&mut world.p_saveg, f);
                    f.thinker.function = ThinkerFn::Floor(move_floor);
                    f.sector
                };
                let floor_id = add_thinker(
                    &mut world.p_tick,
                    ThinkerPayload::Floor(floor_arena_id),
                    ThinkerKind::Floor,
                );
                world.p_setup.sector_mut(sector).specialdata = Some(SectorSpecial::Floor(floor_id));
            }
            3 => {
                saveg_read_pad(&mut world.p_saveg);
                let plat_arena_id = world.p_plats.spawn(Plat::default());
                let sector = {
                    let p = world.p_plats.get_mut(plat_arena_id).expect("live plat");
                    saveg_read_plat_t(&mut world.p_saveg, p);
                    if matches!(p.thinker.function, ThinkerFn::Unresolved) {
                        p.thinker.function = ThinkerFn::Plat(plat_raise);
                    }
                    p.sector
                };
                let plat_id = add_thinker(
                    &mut world.p_tick,
                    ThinkerPayload::Plat(plat_arena_id),
                    ThinkerKind::Plat,
                );
                world.p_setup.sector_mut(sector).specialdata = Some(SectorSpecial::Plat(plat_id));
                add_active_plat(&mut world.p_plats, plat_id);
            }
            4 => {
                saveg_read_pad(&mut world.p_saveg);
                let flash_arena_id = world.p_lights.spawn_lightflash(LightFlash::default());
                {
                    let f = world
                        .p_lights
                        .get_lightflash_mut(flash_arena_id)
                        .expect("live lightflash");
                    saveg_read_lightflash_t(&mut world.p_saveg, f);
                    f.thinker.function = ThinkerFn::LightFlash(light_flash);
                }
                add_thinker(
                    &mut world.p_tick,
                    ThinkerPayload::LightFlash(flash_arena_id),
                    ThinkerKind::LightFlash,
                );
            }
            5 => {
                saveg_read_pad(&mut world.p_saveg);
                let strobe_arena_id = world.p_lights.spawn_strobe(Strobe::default());
                {
                    let s = world
                        .p_lights
                        .get_strobe_mut(strobe_arena_id)
                        .expect("live strobe");
                    saveg_read_strobe_t(&mut world.p_saveg, s);
                    s.thinker.function = ThinkerFn::Strobe(strobe_flash);
                }
                add_thinker(
                    &mut world.p_tick,
                    ThinkerPayload::Strobe(strobe_arena_id),
                    ThinkerKind::Strobe,
                );
            }
            6 => {
                saveg_read_pad(&mut world.p_saveg);
                let glow_arena_id = world.p_lights.spawn_glow(Glow::default());
                {
                    let g = world
                        .p_lights
                        .get_glow_mut(glow_arena_id)
                        .expect("live glow");
                    saveg_read_glow_t(&mut world.p_saveg, g);
                    g.thinker.function = ThinkerFn::Glow(glow);
                }
                add_thinker(
                    &mut world.p_tick,
                    ThinkerPayload::Glow(glow_arena_id),
                    ThinkerKind::Glow,
                );
            }
            _ => {
                error(&format!(
                    "P_UnarchiveSpecials:Unknown tclass {} in savegame",
                    i32::from(tclass),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives_round_trip_with_padding() {
        let mut w = PSavegState::new();
        saveg_write8(&mut w, 0x7f);
        saveg_write_pad(&mut w);
        saveg_write32(&mut w, -123456);
        saveg_write16(&mut w, -2);
        saveg_write_pad(&mut w);
        assert_eq!(w.save_buffer.len(), 12);

        let mut r = PSavegState::new();
        r.save_buffer = w.save_buffer;
        assert_eq!(saveg_read8(&mut r), 0x7f);
        saveg_read_pad(&mut r);
        assert_eq!(saveg_read32(&mut r), -123456);
        assert_eq!(saveg_read16(&mut r), -2);
        saveg_read_pad(&mut r);
        assert_eq!(r.save_pos, 12);
        assert!(!r.savegame_error);
    }

    #[test]
    fn reading_past_the_end_flags_an_error_and_yields_zero() {
        let mut r = PSavegState::new();
        r.save_buffer = vec![1];
        assert_eq!(saveg_read16(&mut r), 1);
        assert!(r.savegame_error);
    }
}
