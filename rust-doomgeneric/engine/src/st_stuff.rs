use crate::am_map::{AM_MSGENTERED, AM_MSGEXITED, AM_MSGHEADER};
use crate::d_event::EvType;
use crate::d_event::Event;
use crate::d_items::WEAPONINFO;
use crate::d_mode::GameMission;
use crate::d_mode::GameMode;
use crate::d_mode::{GameVersion, SkillType};
use crate::d_player::CheatFlags;
use crate::d_player::PlayerId;
use crate::d_player::PowerType;
use crate::d_player::{AmmoType, NUMAMMO};
use crate::d_player::{WeaponType, NUMWEAPONS};
use crate::doomdef::MAXPLAYERS;
use crate::doomdef::SCREENHEIGHT;
use crate::doomdef::SCREENWIDTH;
use crate::doomdef::TICRATE;
use crate::g_game::defered_init_new;
use crate::g_game::GGameState;
use crate::game_state::GameState;
use crate::i_video::set_palette;
use crate::m_cheat::cht_check_cheat;
use crate::m_cheat::CheatSeq;
use crate::m_random::m_random;
use crate::p_inter::give_power;
use crate::p_inter::NUMCARDS;
use crate::p_mobj::PMobjState;
use crate::r_main::point_to_angle2;
use crate::s_sound::change_music;
use crate::sounds::MusicName;
use crate::st_lib::stlib_init;
use crate::st_lib::stlib_init_bin_icon;
use crate::st_lib::stlib_init_mult_icon;
use crate::st_lib::stlib_init_num;
use crate::st_lib::stlib_init_percent;
use crate::st_lib::stlib_update_bin_icon;
use crate::st_lib::stlib_update_mult_icon;
use crate::st_lib::stlib_update_num;
use crate::st_lib::stlib_update_percent;
use crate::st_lib::StDigitSet;
use crate::st_lib::{StBinIcon, StMultiIcon, StNumber, StPercent};
use crate::v_video::Screen;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::tables::Angle;
use crate::tables::ANG180;
use crate::tables::ANG45;
use crate::v_video::cache_patch_num;
use crate::v_video::copy_rect;
use crate::v_video::draw_patch;
use crate::w_wad::get_num_for_name;
use crate::w_wad::lump_bytes;

pub struct StStuffState {
    pub st_backing_screen: Vec<u8>,
    pub plyr: PlayerId,
    pub st_firsttime: bool,
    pub lu_palette: i32,
    pub st_clock: u32,
    pub st_msgcounter: i32,
    pub st_chatstate: StChatStateEnum,
    pub st_gamestate: StStateEnum,
    pub st_statusbaron: bool,
    pub st_chat: bool,
    pub st_oldchat: bool,
    pub st_cursoron: bool,
    pub st_notdeathmatch: bool,
    pub st_armson: bool,
    pub st_fragson: bool,
    pub sbar: i32,
    pub tallnum: [i32; 10],
    pub tallpercent: i32,
    pub shortnum: [i32; 10],
    pub keys: [i32; 6],
    pub faces: [i32; 42],
    pub faceback: i32,
    pub armsbg: i32,
    pub arms: [[i32; 2]; 6],
    pub w_ready: StNumber,
    pub w_frags: StNumber,
    pub w_health: StPercent,
    pub w_armsbg: StBinIcon,
    pub w_arms_owned: [i32; 6],
    pub w_arms: [StMultiIcon; 6],
    pub w_faces: StMultiIcon,
    pub w_keyboxes: [StMultiIcon; 3],
    pub w_armor: StPercent,
    pub w_ammo: [StNumber; 4],
    pub w_maxammo: [StNumber; 4],
    pub st_fragscount: i32,
    pub st_oldhealth: i32,
    pub oldweaponsowned: [bool; 9],
    pub st_facecount: i32,
    pub st_faceindex: i32,
    pub keyboxes: [i32; 3],
    pub st_randomnumber: i32,
    pub cheat_mus: CheatSeq,
    pub cheat_god: CheatSeq,
    pub cheat_ammo: CheatSeq,
    pub cheat_ammonokey: CheatSeq,
    pub cheat_noclip: CheatSeq,
    pub cheat_commercial_noclip: CheatSeq,
    pub cheat_powerup: [CheatSeq; 7],
    pub cheat_choppers: CheatSeq,
    pub cheat_clev: CheatSeq,
    pub cheat_mypos: CheatSeq,
    pub st_calcpainoffset_lastcalc: i32,
    pub st_calcpainoffset_oldhealth: i32,
    pub st_updatefacewidget_lastattackdown: i32,
    pub st_updatefacewidget_priority: i32,
    pub st_palette: i32,
    pub st_stopped: bool,
}

impl Default for StStuffState {
    fn default() -> Self {
        Self::new()
    }
}

impl StStuffState {
    pub const fn new() -> Self {
        Self {
            st_backing_screen: Vec::new(),
            plyr: PlayerId(0),
            st_firsttime: false,
            lu_palette: 0,
            st_clock: 0,
            st_msgcounter: 0,
            st_chatstate: StChatStateEnum::Start,
            st_gamestate: StStateEnum::AutomapState,
            st_statusbaron: false,
            st_chat: false,
            st_oldchat: false,
            st_cursoron: false,
            st_notdeathmatch: false,
            st_armson: false,
            st_fragson: false,
            sbar: -1,
            tallnum: [-1; 10],
            tallpercent: -1,
            shortnum: [-1; 10],
            keys: [-1; 6],
            faces: [-1; 42],
            faceback: -1,
            armsbg: -1,
            arms: [[-1; 2]; 6],
            w_ready: StNumber {
                x: 0,
                y: 0,
                width: 0,
                oldnum: 0,
                p: StDigitSet::TallNum,
                data: 0,
            },
            w_frags: StNumber {
                x: 0,
                y: 0,
                width: 0,
                oldnum: 0,
                p: StDigitSet::TallNum,
                data: 0,
            },
            w_health: StPercent {
                n: StNumber {
                    x: 0,
                    y: 0,
                    width: 0,
                    oldnum: 0,
                    p: StDigitSet::TallNum,
                    data: 0,
                },
                p: -1,
            },
            w_armsbg: StBinIcon {
                x: 0,
                y: 0,
                oldval: false,
                p: -1,
                data: 0,
            },
            w_arms_owned: [0; 6],
            w_arms: [StMultiIcon {
                x: 0,
                y: 0,
                oldinum: 0,
                p: StDigitSet::TallNum,
                data: 0,
            }; 6],
            w_faces: StMultiIcon {
                x: 0,
                y: 0,
                oldinum: 0,
                p: StDigitSet::TallNum,
                data: 0,
            },
            w_keyboxes: [StMultiIcon {
                x: 0,
                y: 0,
                oldinum: 0,
                p: StDigitSet::TallNum,
                data: 0,
            }; 3],
            w_armor: StPercent {
                n: StNumber {
                    x: 0,
                    y: 0,
                    width: 0,
                    oldnum: 0,
                    p: StDigitSet::TallNum,
                    data: 0,
                },
                p: -1,
            },
            w_ammo: [StNumber {
                x: 0,
                y: 0,
                width: 0,
                oldnum: 0,
                p: StDigitSet::TallNum,
                data: 0,
            }; 4],
            w_maxammo: [StNumber {
                x: 0,
                y: 0,
                width: 0,
                oldnum: 0,
                p: StDigitSet::TallNum,
                data: 0,
            }; 4],
            st_fragscount: 0,
            st_oldhealth: -1,
            oldweaponsowned: [false; 9],
            st_facecount: 0,
            st_faceindex: 0,
            keyboxes: [0; 3],
            st_randomnumber: 0,
            cheat_mus: CheatSeq::new("idmus", 2),
            cheat_god: CheatSeq::new("iddqd", 0),
            cheat_ammo: CheatSeq::new("idkfa", 0),
            cheat_ammonokey: CheatSeq::new("idfa", 0),
            cheat_noclip: CheatSeq::new("idspispopd", 0),
            cheat_commercial_noclip: CheatSeq::new("idclip", 0),
            cheat_powerup: [
                CheatSeq::new("idbeholdv", 0),
                CheatSeq::new("idbeholds", 0),
                CheatSeq::new("idbeholdi", 0),
                CheatSeq::new("idbeholdr", 0),
                CheatSeq::new("idbeholda", 0),
                CheatSeq::new("idbeholdl", 0),
                CheatSeq::new("idbehold", 0),
            ],
            cheat_choppers: CheatSeq::new("idchoppers", 0),
            cheat_clev: CheatSeq::new("idclev", 2),
            cheat_mypos: CheatSeq::new("idmypos", 0),
            st_calcpainoffset_lastcalc: 0,
            st_calcpainoffset_oldhealth: -1,
            st_updatefacewidget_lastattackdown: -1,
            st_updatefacewidget_priority: 0,
            st_palette: 0,
            st_stopped: true,
        }
    }

    pub fn digit_set(&self, id: StDigitSet) -> &[i32] {
        match id {
            StDigitSet::TallNum => &self.tallnum,
            StDigitSet::ShortNum => &self.shortnum,
            StDigitSet::Faces => &self.faces,
            StDigitSet::Arms(i) => &self.arms[i],
            StDigitSet::Keys => &self.keys,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StStateEnum {
    AutomapState = 0,
    FirstPersonState = 1,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum StChatStateEnum {
    Start = 0,
    WaitDest = 1,
    Get = 2,
}
pub type LoadCallback = fn(&mut GameState, &str) -> i32;
pub const DEH_DEFAULT_GOD_MODE_HEALTH: i32 = 100;
pub const DEH_DEFAULT_IDFA_ARMOR: i32 = 200;
pub const DEH_DEFAULT_IDFA_ARMOR_CLASS: i32 = 2;
pub const DEH_DEFAULT_IDKFA_ARMOR: i32 = 200;
pub const DEH_DEFAULT_IDKFA_ARMOR_CLASS: i32 = 2;
pub const DEH_GOD_MODE_HEALTH: i32 = DEH_DEFAULT_GOD_MODE_HEALTH;
pub const DEH_IDFA_ARMOR: i32 = DEH_DEFAULT_IDFA_ARMOR;
pub const DEH_IDFA_ARMOR_CLASS: i32 = DEH_DEFAULT_IDFA_ARMOR_CLASS;
pub const DEH_IDKFA_ARMOR: i32 = DEH_DEFAULT_IDKFA_ARMOR;
pub const DEH_IDKFA_ARMOR_CLASS: i32 = DEH_DEFAULT_IDKFA_ARMOR_CLASS;
pub const ST_HEIGHT: i32 = 32;
pub const ST_WIDTH: i32 = SCREENWIDTH;
pub const ST_Y: i32 = SCREENHEIGHT - ST_HEIGHT;
pub const STARTREDPALS: i32 = 1;
pub const STARTBONUSPALS: i32 = 9;
pub const NUMREDPALS: i32 = 8;
pub const NUMBONUSPALS: i32 = 4;
pub const RADIATIONPAL: i32 = 13;
pub const ST_X: i32 = 0;
pub const ST_FX: i32 = 143;
pub const ST_NUMPAINFACES: i32 = 5;
pub const ST_NUMSTRAIGHTFACES: i32 = 3;
pub const ST_NUMTURNFACES: i32 = 2;
pub const ST_NUMSPECIALFACES: i32 = 3;
pub const ST_FACESTRIDE: i32 = ST_NUMSTRAIGHTFACES + ST_NUMTURNFACES + ST_NUMSPECIALFACES;
pub const ST_TURNOFFSET: i32 = 3;
pub const ST_OUCHOFFSET: i32 = ST_TURNOFFSET + ST_NUMTURNFACES;
pub const ST_EVILGRINOFFSET: i32 = ST_OUCHOFFSET + 1;
pub const ST_RAMPAGEOFFSET: i32 = ST_EVILGRINOFFSET + 1;
pub const ST_GODFACE: i32 = ST_NUMPAINFACES * ST_FACESTRIDE;
pub const ST_DEADFACE: i32 = ST_GODFACE + 1;
pub const ST_FACESX: i32 = 143;
pub const ST_FACESY: i32 = 168;
pub const ST_EVILGRINCOUNT: i32 = 2 * TICRATE;
pub const ST_STRAIGHTFACECOUNT: i32 = TICRATE / 2;
pub const ST_TURNCOUNT: i32 = TICRATE;
pub const ST_RAMPAGEDELAY: i32 = 2 * TICRATE;
pub const ST_MUCHPAIN: i32 = 20;
pub const ST_AMMOWIDTH: i32 = 3;
pub const ST_AMMOX: i32 = 44;
pub const ST_AMMOY: i32 = 171;
pub const ST_HEALTHX: i32 = 90;
pub const ST_HEALTHY: i32 = 171;
pub const ST_ARMSX: i32 = 111;
pub const ST_ARMSY: i32 = 172;
pub const ST_ARMSBGX: i32 = 104;
pub const ST_ARMSBGY: i32 = 168;
pub const ST_ARMSXSPACE: i32 = 12;
pub const ST_ARMSYSPACE: i32 = 10;
pub const ST_FRAGSX: i32 = 138;
pub const ST_FRAGSY: i32 = 171;
pub const ST_FRAGSWIDTH: i32 = 2;
pub const ST_ARMORX: i32 = 221;
pub const ST_ARMORY: i32 = 171;
pub const ST_KEY0X: i32 = 239;
pub const ST_KEY0Y: i32 = 171;
pub const ST_KEY1X: i32 = 239;
pub const ST_KEY1Y: i32 = 181;
pub const ST_KEY2X: i32 = 239;
pub const ST_KEY2Y: i32 = 191;
pub const ST_AMMO0WIDTH: i32 = 3;
pub const ST_AMMO0X: i32 = 288;
pub const ST_AMMO0Y: i32 = 173;
pub const ST_AMMO1WIDTH: i32 = ST_AMMO0WIDTH;
pub const ST_AMMO1X: i32 = 288;
pub const ST_AMMO1Y: i32 = 179;
pub const ST_AMMO2WIDTH: i32 = ST_AMMO0WIDTH;
pub const ST_AMMO2X: i32 = 288;
pub const ST_AMMO2Y: i32 = 191;
pub const ST_AMMO3WIDTH: i32 = ST_AMMO0WIDTH;
pub const ST_AMMO3X: i32 = 288;
pub const ST_AMMO3Y: i32 = 185;
pub const ST_MAXAMMO0WIDTH: i32 = 3;
pub const ST_MAXAMMO0X: i32 = 314;
pub const ST_MAXAMMO0Y: i32 = 173;
pub const ST_MAXAMMO1WIDTH: i32 = ST_MAXAMMO0WIDTH;
pub const ST_MAXAMMO1X: i32 = 314;
pub const ST_MAXAMMO1Y: i32 = 179;
pub const ST_MAXAMMO2WIDTH: i32 = ST_MAXAMMO0WIDTH;
pub const ST_MAXAMMO2X: i32 = 314;
pub const ST_MAXAMMO2Y: i32 = 191;
pub const ST_MAXAMMO3WIDTH: i32 = ST_MAXAMMO0WIDTH;
pub const ST_MAXAMMO3X: i32 = 314;
pub const ST_MAXAMMO3Y: i32 = 185;
pub fn refresh_background(state: &mut GameState) {
    if state.ui.st_stuff.st_statusbaron {
        let st_backing_screen = Screen::StatusBar;
        let sbar_patch = cache_patch_num(
            &*state.assets.fs,
            &mut state.assets.w_wad,
            state.ui.st_stuff.sbar,
        );
        draw_patch(state, st_backing_screen, ST_X, 0, &sbar_patch);
        if state.game.g_game.netgame {
            let faceback_patch = cache_patch_num(
                &*state.assets.fs,
                &mut state.assets.w_wad,
                state.ui.st_stuff.faceback,
            );
            draw_patch(state, st_backing_screen, ST_FX, 0, &faceback_patch);
        }
        let dest_screen = Screen::Video;
        copy_rect(
            state,
            dest_screen,
            ST_X,
            0,
            st_backing_screen,
            ST_WIDTH,
            ST_HEIGHT,
            ST_X,
            ST_Y,
        );
    }
}
pub fn st_responder(state: &mut GameState, ev: &Event) -> bool {
    if ev.kind == EvType::Keyup && ev.data1 as u32 & 0xffff0000 == AM_MSGHEADER as u32 {
        match ev.data1 {
            AM_MSGENTERED => {
                state.ui.st_stuff.st_gamestate = StStateEnum::AutomapState;
                state.ui.st_stuff.st_firsttime = true;
            }
            AM_MSGEXITED => {
                state.ui.st_stuff.st_gamestate = StStateEnum::FirstPersonState;
            }
            _ => {}
        }
    } else if ev.kind == EvType::Keydown {
        if !state.game.g_game.netgame && state.game.g_game.gameskill != SkillType::Nightmare {
            if cht_check_cheat(&mut state.ui.st_stuff.cheat_god, ev.data2 as u8) {
                state.game.g_game.player_mut(state.ui.st_stuff.plyr).cheats ^= CheatFlags::GODMODE;
                if state
                    .game
                    .g_game
                    .player_mut(state.ui.st_stuff.plyr)
                    .cheats
                    .contains(CheatFlags::GODMODE)
                {
                    if let Some(mo_id) = state.game.g_game.player_mut(state.ui.st_stuff.plyr).mo {
                        state.world.p_mobj.mo_mut(mo_id).health = 100;
                    }
                    state.game.g_game.player_mut(state.ui.st_stuff.plyr).health =
                        DEH_GOD_MODE_HEALTH;
                    state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                        Some("Degreelessness Mode On".to_string());
                } else {
                    state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                        Some("Degreelessness Mode Off".to_string());
                }
            } else if cht_check_cheat(&mut state.ui.st_stuff.cheat_ammonokey, ev.data2 as u8) {
                state
                    .game
                    .g_game
                    .player_mut(state.ui.st_stuff.plyr)
                    .armorpoints = DEH_IDFA_ARMOR;
                state
                    .game
                    .g_game
                    .player_mut(state.ui.st_stuff.plyr)
                    .armortype = DEH_IDFA_ARMOR_CLASS;
                for i in 0..(NUMWEAPONS as usize) {
                    state
                        .game
                        .g_game
                        .player_mut(state.ui.st_stuff.plyr)
                        .weaponowned[i] = true;
                }
                for i in 0..(NUMAMMO as usize) {
                    state.game.g_game.player_mut(state.ui.st_stuff.plyr).ammo[i] =
                        state.game.g_game.player_mut(state.ui.st_stuff.plyr).maxammo[i];
                }
                state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                    Some("Ammo (no keys) Added".to_string());
            } else if cht_check_cheat(&mut state.ui.st_stuff.cheat_ammo, ev.data2 as u8) {
                state
                    .game
                    .g_game
                    .player_mut(state.ui.st_stuff.plyr)
                    .armorpoints = DEH_IDKFA_ARMOR;
                state
                    .game
                    .g_game
                    .player_mut(state.ui.st_stuff.plyr)
                    .armortype = DEH_IDKFA_ARMOR_CLASS;
                for i in 0..(NUMWEAPONS as usize) {
                    state
                        .game
                        .g_game
                        .player_mut(state.ui.st_stuff.plyr)
                        .weaponowned[i] = true;
                }
                for i in 0..(NUMAMMO as usize) {
                    state.game.g_game.player_mut(state.ui.st_stuff.plyr).ammo[i] =
                        state.game.g_game.player_mut(state.ui.st_stuff.plyr).maxammo[i];
                }
                for i in 0..(NUMCARDS as usize) {
                    state.game.g_game.player_mut(state.ui.st_stuff.plyr).cards[i] = true;
                }
                state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                    Some("Very Happy Ammo Added".to_string());
            } else if cht_check_cheat(&mut state.ui.st_stuff.cheat_mus, ev.data2 as u8) {
                state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                    Some("Music Change".to_string());
                let buf: [u8; 2] = [
                    state.ui.st_stuff.cheat_mus.param()[0],
                    state.ui.st_stuff.cheat_mus.param()[1],
                ];
                if state.game.doomstat.gamemode == GameMode::Commercial
                    || !state.game.doomstat.gameversion.is_ultimate_or_higher()
                {
                    let musnum: i32 = MusicName::Runnin as i32
                        + (buf[0] as i32 - '0' as i32) * 10
                        + buf[1] as i32
                        - '0' as i32
                        - 1;
                    if (buf[0] as i32 - '0' as i32) * 10 + buf[1] as i32 - '0' as i32 > 35 {
                        state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                            Some("IMPOSSIBLE SELECTION".to_string());
                    } else {
                        change_music(state, musnum, true);
                    }
                } else {
                    let musnum: i32 = MusicName::E1m1 as i32
                        + (buf[0] as i32 - '1' as i32) * 9
                        + (buf[1] as i32 - '1' as i32);
                    if (buf[0] as i32 - '1' as i32) * 9 + buf[1] as i32 - '1' as i32 > 31 {
                        state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                            Some("IMPOSSIBLE SELECTION".to_string());
                    } else {
                        change_music(state, musnum, true);
                    }
                }
            } else if state.game.doomstat.gamemission.base() == GameMission::Doom
                && cht_check_cheat(&mut state.ui.st_stuff.cheat_noclip, ev.data2 as u8)
                || state.game.doomstat.gamemission.base() != GameMission::Doom
                    && cht_check_cheat(
                        &mut state.ui.st_stuff.cheat_commercial_noclip,
                        ev.data2 as u8,
                    )
            {
                state.game.g_game.player_mut(state.ui.st_stuff.plyr).cheats ^= CheatFlags::NOCLIP;
                if state
                    .game
                    .g_game
                    .player_mut(state.ui.st_stuff.plyr)
                    .cheats
                    .contains(CheatFlags::NOCLIP)
                {
                    state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                        Some("No Clipping Mode ON".to_string());
                } else {
                    state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                        Some("No Clipping Mode OFF".to_string());
                }
            }
            for (i, power) in PowerType::ALL.into_iter().enumerate() {
                if cht_check_cheat(&mut state.ui.st_stuff.cheat_powerup[i], ev.data2 as u8) {
                    if state.game.g_game.player_mut(state.ui.st_stuff.plyr).powers[power] == 0 {
                        give_power(
                            &mut state.game.g_game,
                            &mut state.world.p_mobj,
                            state.ui.st_stuff.plyr,
                            power,
                        );
                    } else if power != PowerType::Strength {
                        state.game.g_game.player_mut(state.ui.st_stuff.plyr).powers[power] = 1;
                    } else {
                        state.game.g_game.player_mut(state.ui.st_stuff.plyr).powers[power] = 0;
                    }
                    state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                        Some("Power-up Toggled".to_string());
                }
            }
            if cht_check_cheat(&mut state.ui.st_stuff.cheat_powerup[6], ev.data2 as u8) {
                state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                    Some("inVuln, Str, Inviso, Rad, Allmap, or Lite-amp".to_string());
            } else if cht_check_cheat(&mut state.ui.st_stuff.cheat_choppers, ev.data2 as u8) {
                state
                    .game
                    .g_game
                    .player_mut(state.ui.st_stuff.plyr)
                    .weaponowned[WeaponType::Chainsaw] = true;
                state.game.g_game.player_mut(state.ui.st_stuff.plyr).powers
                    [PowerType::Invulnerability as usize] = 1;
                state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                    Some("... doesn't suck - GM".to_string());
            } else if cht_check_cheat(&mut state.ui.st_stuff.cheat_mypos, ev.data2 as u8) {
                let cp_mo_id = state.game.g_game.players[state.game.g_game.consoleplayer]
                    .mo
                    .unwrap();
                let cp_mo = state.world.p_mobj.mo(cp_mo_id);
                state.game.g_game.player_mut(state.ui.st_stuff.plyr).message = Some(format!(
                    "ang=0x{:x};x,y=(0x{:x},0x{:x})",
                    cp_mo.angle, cp_mo.x, cp_mo.y,
                ));
            }
        }
        if !state.game.g_game.netgame
            && cht_check_cheat(&mut state.ui.st_stuff.cheat_clev, ev.data2 as u8)
        {
            let mut epsd: i32;
            let digits: [u8; 2] = [
                state.ui.st_stuff.cheat_clev.param()[0],
                state.ui.st_stuff.cheat_clev.param()[1],
            ];
            let map: i32 = if state.game.doomstat.gamemode == GameMode::Commercial {
                epsd = 1;
                (digits[0] as i32 - '0' as i32) * 10 + digits[1] as i32 - '0' as i32
            } else {
                epsd = digits[0] as i32 - '0' as i32;
                digits[1] as i32 - '0' as i32
            };
            if state.game.doomstat.gameversion == GameVersion::Chex {
                epsd = 1;
            }
            if epsd < 1 {
                return false;
            }
            if map < 1 {
                return false;
            }
            if state.game.doomstat.gamemode == GameMode::Retail && (epsd > 4 || map > 9) {
                return false;
            }
            if state.game.doomstat.gamemode == GameMode::Registered && (epsd > 3 || map > 9) {
                return false;
            }
            if state.game.doomstat.gamemode == GameMode::Shareware && (epsd > 1 || map > 9) {
                return false;
            }
            if state.game.doomstat.gamemode == GameMode::Commercial && (epsd > 1 || map > 40) {
                return false;
            }
            state.game.g_game.player_mut(state.ui.st_stuff.plyr).message =
                Some("Changing Level...".to_string());
            let gameskill = state.game.g_game.gameskill;
            defered_init_new(&mut state.game.g_game, gameskill, epsd, map);
        }
    }
    false
}
pub fn calc_pain_offset(g_game: &mut GGameState, st_stuff: &mut StStuffState) -> i32 {
    let health: i32 = if g_game.player_mut(st_stuff.plyr).health > 100 {
        100
    } else {
        g_game.player_mut(st_stuff.plyr).health
    };
    if health != st_stuff.st_calcpainoffset_oldhealth {
        st_stuff.st_calcpainoffset_lastcalc =
            ST_FACESTRIDE * ((100 - health) * ST_NUMPAINFACES / 101);
        st_stuff.st_calcpainoffset_oldhealth = health;
    }
    st_stuff.st_calcpainoffset_lastcalc
}
pub fn update_face_widget(
    g_game: &mut GGameState,
    p_mobj: &PMobjState,
    st_stuff: &mut StStuffState,
) {
    let i: i32;
    if st_stuff.st_updatefacewidget_priority < 10 && g_game.player_mut(st_stuff.plyr).health == 0 {
        st_stuff.st_updatefacewidget_priority = 9;
        st_stuff.st_faceindex = ST_DEADFACE;
        st_stuff.st_facecount = 1;
    }
    if st_stuff.st_updatefacewidget_priority < 9 && g_game.player_mut(st_stuff.plyr).bonuscount != 0
    {
        let mut doevilgrin: bool = false;
        for i in 0..(NUMWEAPONS as usize) {
            if st_stuff.oldweaponsowned[i] != g_game.player_mut(st_stuff.plyr).weaponowned[i] {
                doevilgrin = true;
                st_stuff.oldweaponsowned[i] = g_game.player_mut(st_stuff.plyr).weaponowned[i];
            }
        }
        if doevilgrin {
            st_stuff.st_updatefacewidget_priority = 8;
            st_stuff.st_facecount = ST_EVILGRINCOUNT;
            st_stuff.st_faceindex = calc_pain_offset(g_game, st_stuff) + ST_EVILGRINOFFSET;
        }
    }
    if st_stuff.st_updatefacewidget_priority < 8 {
        let plyr_attacker_id = g_game.player_mut(st_stuff.plyr).attacker;
        if let Some(attacker_id) = plyr_attacker_id.filter(|&a| {
            g_game.player_mut(st_stuff.plyr).damagecount != 0
                && Some(a) != g_game.player_mut(st_stuff.plyr).mo
        }) {
            st_stuff.st_updatefacewidget_priority = 7;
            let plyr_mo = p_mobj.mo(g_game.player_mut(st_stuff.plyr).mo.unwrap());
            let (plyr_mo_x, plyr_mo_y, plyr_mo_angle) = (plyr_mo.x, plyr_mo.y, plyr_mo.angle);
            if g_game.player_mut(st_stuff.plyr).health - st_stuff.st_oldhealth > ST_MUCHPAIN {
                st_stuff.st_facecount = ST_TURNCOUNT;
                st_stuff.st_faceindex = calc_pain_offset(g_game, st_stuff) + ST_OUCHOFFSET;
            } else {
                let diffang: Angle;

                let plyr_attacker = p_mobj.mo(attacker_id);
                let (attacker_x, attacker_y) = (plyr_attacker.x, plyr_attacker.y);
                let badguyangle: Angle =
                    point_to_angle2(plyr_mo_x, plyr_mo_y, attacker_x, attacker_y);
                if badguyangle > plyr_mo_angle {
                    diffang = badguyangle.wrapping_sub(plyr_mo_angle);
                    i = (diffang > ANG180) as i32;
                } else {
                    diffang = plyr_mo_angle.wrapping_sub(badguyangle);
                    i = (diffang <= ANG180) as i32;
                }
                st_stuff.st_facecount = ST_TURNCOUNT;
                st_stuff.st_faceindex = calc_pain_offset(g_game, st_stuff);
                if diffang < ANG45 as Angle {
                    st_stuff.st_faceindex += ST_RAMPAGEOFFSET;
                } else if i != 0 {
                    st_stuff.st_faceindex += ST_TURNOFFSET;
                } else {
                    st_stuff.st_faceindex += ST_TURNOFFSET + 1;
                }
            }
        }
    }
    if st_stuff.st_updatefacewidget_priority < 7
        && g_game.player_mut(st_stuff.plyr).damagecount != 0
    {
        if g_game.player_mut(st_stuff.plyr).health - st_stuff.st_oldhealth > ST_MUCHPAIN {
            st_stuff.st_updatefacewidget_priority = 7;
            st_stuff.st_facecount = ST_TURNCOUNT;
            st_stuff.st_faceindex = calc_pain_offset(g_game, st_stuff) + ST_OUCHOFFSET;
        } else {
            st_stuff.st_updatefacewidget_priority = 6;
            st_stuff.st_facecount = ST_TURNCOUNT;
            st_stuff.st_faceindex = calc_pain_offset(g_game, st_stuff) + ST_RAMPAGEOFFSET;
        }
    }
    if st_stuff.st_updatefacewidget_priority < 6 {
        if g_game.player_mut(st_stuff.plyr).attackdown {
            if st_stuff.st_updatefacewidget_lastattackdown == -1 {
                st_stuff.st_updatefacewidget_lastattackdown = ST_RAMPAGEDELAY;
            } else {
                st_stuff.st_updatefacewidget_lastattackdown -= 1;
                if st_stuff.st_updatefacewidget_lastattackdown == 0 {
                    st_stuff.st_updatefacewidget_priority = 5;
                    st_stuff.st_faceindex = calc_pain_offset(g_game, st_stuff) + ST_RAMPAGEOFFSET;
                    st_stuff.st_facecount = 1;
                    st_stuff.st_updatefacewidget_lastattackdown = 1;
                }
            }
        } else {
            st_stuff.st_updatefacewidget_lastattackdown = -1;
        }
    }
    if st_stuff.st_updatefacewidget_priority < 5
        && (g_game
            .player_mut(st_stuff.plyr)
            .cheats
            .contains(CheatFlags::GODMODE)
            || g_game.player_mut(st_stuff.plyr).powers[PowerType::Invulnerability as usize] != 0)
    {
        st_stuff.st_updatefacewidget_priority = 4;
        st_stuff.st_faceindex = ST_GODFACE;
        st_stuff.st_facecount = 1;
    }
    if st_stuff.st_facecount == 0 {
        st_stuff.st_faceindex = calc_pain_offset(g_game, st_stuff) + st_stuff.st_randomnumber % 3;
        st_stuff.st_facecount = ST_STRAIGHTFACECOUNT;
        st_stuff.st_updatefacewidget_priority = 0;
    }
    st_stuff.st_facecount -= 1;
}
pub fn update_widgets(g_game: &mut GGameState, p_mobj: &PMobjState, st_stuff: &mut StStuffState) {
    st_stuff.w_ready.data = g_game.player_mut(st_stuff.plyr).readyweapon as i32;
    for i in 0..6 {
        st_stuff.w_arms_owned[i as usize] =
            g_game.player_mut(st_stuff.plyr).weaponowned[(i + 1) as usize] as i32;
    }
    for i in 0..3 {
        st_stuff.keyboxes[i as usize] = if g_game.player_mut(st_stuff.plyr).cards[i as usize] {
            i
        } else {
            -1
        };
        if g_game.player_mut(st_stuff.plyr).cards[(i + 3) as usize] {
            st_stuff.keyboxes[i as usize] = i + 3;
        }
    }
    update_face_widget(g_game, p_mobj, st_stuff);
    st_stuff.st_notdeathmatch = g_game.deathmatch == 0;
    st_stuff.st_armson = st_stuff.st_statusbaron && g_game.deathmatch == 0;
    st_stuff.st_fragson = g_game.deathmatch != 0 && st_stuff.st_statusbaron;
    st_stuff.st_fragscount = 0;
    for i in 0..MAXPLAYERS {
        if i == g_game.consoleplayer.as_i32() {
            st_stuff.st_fragscount -= g_game.player_mut(st_stuff.plyr).frags[i as usize];
        } else {
            st_stuff.st_fragscount += g_game.player_mut(st_stuff.plyr).frags[i as usize];
        }
    }
    st_stuff.st_msgcounter -= 1;
    if st_stuff.st_msgcounter == 0 {
        st_stuff.st_chat = st_stuff.st_oldchat;
    }
}
pub fn st_ticker(state: &mut GameState) {
    state.ui.st_stuff.st_clock = state.ui.st_stuff.st_clock.wrapping_add(1);
    state.ui.st_stuff.st_randomnumber = m_random(&mut state.world.m_random);
    update_widgets(
        &mut state.game.g_game,
        &state.world.p_mobj,
        &mut state.ui.st_stuff,
    );
    state.ui.st_stuff.st_oldhealth = state.game.g_game.player_mut(state.ui.st_stuff.plyr).health;
}
pub fn do_palette_stuff(state: &mut GameState) {
    let mut palette: i32;
    let mut cnt: i32 = state
        .game
        .g_game
        .player_mut(state.ui.st_stuff.plyr)
        .damagecount;
    if state.game.g_game.player_mut(state.ui.st_stuff.plyr).powers[PowerType::Strength] != 0 {
        let bzc: i32 = 12
            - (state.game.g_game.player_mut(state.ui.st_stuff.plyr).powers
                [PowerType::Strength as usize]
                >> 6);
        if bzc > cnt {
            cnt = bzc;
        }
    }
    if cnt != 0 {
        palette = (cnt + 7) >> 3;
        if palette >= NUMREDPALS {
            palette = NUMREDPALS - 1;
        }
        palette += STARTREDPALS;
    } else if state
        .game
        .g_game
        .player_mut(state.ui.st_stuff.plyr)
        .bonuscount
        != 0
    {
        palette = (state
            .game
            .g_game
            .player_mut(state.ui.st_stuff.plyr)
            .bonuscount
            + 7)
            >> 3;
        if palette >= NUMBONUSPALS {
            palette = NUMBONUSPALS - 1;
        }
        palette += STARTBONUSPALS;
    } else if state.game.g_game.player_mut(state.ui.st_stuff.plyr).powers
        [PowerType::Ironfeet as usize]
        > 4 * 32
        || state.game.g_game.player_mut(state.ui.st_stuff.plyr).powers[PowerType::Ironfeet] & 8 != 0
    {
        palette = RADIATIONPAL;
    } else {
        palette = 0;
    }
    if state.game.doomstat.gameversion == GameVersion::Chex
        && (STARTREDPALS..STARTREDPALS + NUMREDPALS).contains(&palette)
    {
        palette = RADIATIONPAL;
    }
    if palette != state.ui.st_stuff.st_palette {
        state.ui.st_stuff.st_palette = palette;
        let pal = lump_bytes(
            &*state.assets.fs,
            &mut state.assets.w_wad,
            state.ui.st_stuff.lu_palette,
        );
        let offset = (palette * 768) as usize;
        set_palette(&mut state.io.i_video, &pal[offset..offset + 768]);
    }
}
pub fn draw_widgets(state: &mut GameState, refresh: bool) {
    state.ui.st_stuff.st_armson =
        state.ui.st_stuff.st_statusbaron && state.game.g_game.deathmatch == 0;
    state.ui.st_stuff.st_fragson =
        state.game.g_game.deathmatch != 0 && state.ui.st_stuff.st_statusbaron;
    let statusbaron = state.ui.st_stuff.st_statusbaron;
    let ready_weapon_ammo = WEAPONINFO[state
        .game
        .g_game
        .player_mut(state.ui.st_stuff.plyr)
        .readyweapon as usize]
        .ammo;
    let ready_ammo_num = if ready_weapon_ammo as u32 == AmmoType::Noammo as i32 as u32 {
        1994
    } else {
        state.game.g_game.player_mut(state.ui.st_stuff.plyr).ammo[ready_weapon_ammo]
    };
    let mut w_ready = state.ui.st_stuff.w_ready;
    stlib_update_num(state, &mut w_ready, ready_ammo_num, statusbaron);
    state.ui.st_stuff.w_ready = w_ready;
    for i in 0..4 {
        let ammo_num = state.game.g_game.player_mut(state.ui.st_stuff.plyr).ammo[i];
        let mut w_ammo = state.ui.st_stuff.w_ammo[i];
        stlib_update_num(state, &mut w_ammo, ammo_num, statusbaron);
        state.ui.st_stuff.w_ammo[i] = w_ammo;
        let maxammo_num = state.game.g_game.player_mut(state.ui.st_stuff.plyr).maxammo[i];
        let mut w_maxammo = state.ui.st_stuff.w_maxammo[i];
        stlib_update_num(state, &mut w_maxammo, maxammo_num, statusbaron);
        state.ui.st_stuff.w_maxammo[i] = w_maxammo;
    }
    let health_num = state.game.g_game.player_mut(state.ui.st_stuff.plyr).health;
    let mut w_health = state.ui.st_stuff.w_health;
    stlib_update_percent(
        state,
        &mut w_health,
        health_num,
        statusbaron,
        refresh as i32,
    );
    state.ui.st_stuff.w_health = w_health;
    let armor_num = state
        .game
        .g_game
        .player_mut(state.ui.st_stuff.plyr)
        .armorpoints;
    let mut w_armor = state.ui.st_stuff.w_armor;
    stlib_update_percent(state, &mut w_armor, armor_num, statusbaron, refresh as i32);
    state.ui.st_stuff.w_armor = w_armor;
    let notdeathmatch = state.ui.st_stuff.st_notdeathmatch;
    let mut w_armsbg = state.ui.st_stuff.w_armsbg;
    stlib_update_bin_icon(state, &mut w_armsbg, notdeathmatch, statusbaron, refresh);
    state.ui.st_stuff.w_armsbg = w_armsbg;
    let armson = state.ui.st_stuff.st_armson;
    for i in 0..6 {
        let arms_owned = state.ui.st_stuff.w_arms_owned[i];
        let mut w_arms = state.ui.st_stuff.w_arms[i];
        stlib_update_mult_icon(state, &mut w_arms, arms_owned, armson, refresh);
        state.ui.st_stuff.w_arms[i] = w_arms;
    }
    let faceindex = state.ui.st_stuff.st_faceindex;
    let mut w_faces = state.ui.st_stuff.w_faces;
    stlib_update_mult_icon(state, &mut w_faces, faceindex, statusbaron, refresh);
    state.ui.st_stuff.w_faces = w_faces;
    for i in 0..3 {
        let keybox = state.ui.st_stuff.keyboxes[i];
        let mut w_keyboxes = state.ui.st_stuff.w_keyboxes[i];
        stlib_update_mult_icon(state, &mut w_keyboxes, keybox, statusbaron, refresh);
        state.ui.st_stuff.w_keyboxes[i] = w_keyboxes;
    }
    let fragscount = state.ui.st_stuff.st_fragscount;
    let fragson = state.ui.st_stuff.st_fragson;
    let mut w_frags = state.ui.st_stuff.w_frags;
    stlib_update_num(state, &mut w_frags, fragscount, fragson);
    state.ui.st_stuff.w_frags = w_frags;
}
pub fn do_refresh(state: &mut GameState) {
    state.ui.st_stuff.st_firsttime = false;
    refresh_background(state);
    draw_widgets(state, true);
}
pub fn diff_draw(state: &mut GameState) {
    draw_widgets(state, false);
}
pub fn st_drawer(state: &mut GameState, fullscreen: bool, refresh: bool) {
    state.ui.st_stuff.st_statusbaron = !fullscreen || state.ui.am_map.automapactive;
    state.ui.st_stuff.st_firsttime = state.ui.st_stuff.st_firsttime || refresh;
    do_palette_stuff(state);
    if state.ui.st_stuff.st_firsttime {
        do_refresh(state);
    } else {
        diff_draw(state);
    }
}
fn load_unload_graphics(state: &mut GameState, callback: LoadCallback) {
    for i in 0..10_usize {
        state.ui.st_stuff.tallnum[i] = callback(state, &format!("STTNUM{i}"));
        state.ui.st_stuff.shortnum[i] = callback(state, &format!("STYSNUM{i}"));
    }
    state.ui.st_stuff.tallpercent = callback(state, "STTPRCNT");
    for i in 0..NUMCARDS as usize {
        state.ui.st_stuff.keys[i] = callback(state, &format!("STKEYS{i}"));
    }
    state.ui.st_stuff.armsbg = callback(state, "STARMS");
    for i in 0..6_usize {
        state.ui.st_stuff.arms[i][0] = callback(state, &format!("STGNUM{}", i + 2));
        state.ui.st_stuff.arms[i][1] = state.ui.st_stuff.shortnum[i + 2];
    }
    state.ui.st_stuff.faceback =
        callback(state, &format!("STFB{}", state.game.g_game.consoleplayer.0));
    state.ui.st_stuff.sbar = callback(state, "STBAR");
    let mut facenum = 0_usize;
    for i in 0..ST_NUMPAINFACES {
        for j in 0..ST_NUMSTRAIGHTFACES {
            state.ui.st_stuff.faces[facenum] = callback(state, &format!("STFST{i}{j}"));
            facenum += 1;
        }
        for name in [
            format!("STFTR{i}0"),
            format!("STFTL{i}0"),
            format!("STFOUCH{i}"),
            format!("STFEVL{i}"),
            format!("STFKILL{i}"),
        ] {
            state.ui.st_stuff.faces[facenum] = callback(state, &name);
            facenum += 1;
        }
    }
    state.ui.st_stuff.faces[facenum] = callback(state, "STFGOD0");
    facenum += 1;
    state.ui.st_stuff.faces[facenum] = callback(state, "STFDEAD0");
}
fn st_load_callback(state: &mut GameState, lumpname: &str) -> i32 {
    let lumpnum = get_num_for_name(&state.assets.w_wad, lumpname);
    lump_bytes(&*state.assets.fs, &mut state.assets.w_wad, lumpnum);
    lumpnum
}
pub fn load_graphics(state: &mut GameState) {
    load_unload_graphics(state, st_load_callback);
}
pub fn st_load_data(state: &mut GameState) {
    state.ui.st_stuff.lu_palette = get_num_for_name(&state.assets.w_wad, "PLAYPAL");
    load_graphics(state);
}
pub fn st_init_data(state: &mut GameState) {
    state.ui.st_stuff.st_firsttime = true;
    state.ui.st_stuff.plyr = state.game.g_game.consoleplayer;
    state.ui.st_stuff.st_clock = 0;
    state.ui.st_stuff.st_chatstate = StChatStateEnum::Start;
    state.ui.st_stuff.st_gamestate = StStateEnum::FirstPersonState;
    state.ui.st_stuff.st_statusbaron = true;
    state.ui.st_stuff.st_chat = false;
    state.ui.st_stuff.st_oldchat = state.ui.st_stuff.st_chat;
    state.ui.st_stuff.st_cursoron = false;
    state.ui.st_stuff.st_faceindex = 0;
    state.ui.st_stuff.st_palette = -1;
    state.ui.st_stuff.st_oldhealth = -1;
    for i in 0..(NUMWEAPONS as usize) {
        state.ui.st_stuff.oldweaponsowned[i] = state
            .game
            .g_game
            .player_mut(state.ui.st_stuff.plyr)
            .weaponowned[i];
    }
    for i in 0..3 {
        state.ui.st_stuff.keyboxes[i] = -1;
    }
    stlib_init(
        &*state.assets.fs,
        &mut state.ui.st_lib,
        &mut state.assets.w_wad,
    );
}
pub fn create_widgets(g_game: &mut GGameState, st_stuff: &mut StStuffState) {
    stlib_init_num(
        &mut st_stuff.w_ready,
        ST_AMMOX,
        ST_AMMOY,
        StDigitSet::TallNum,
        ST_AMMOWIDTH,
    );
    st_stuff.w_ready.data = g_game.player_mut(st_stuff.plyr).readyweapon as i32;
    stlib_init_percent(
        &mut st_stuff.w_health,
        ST_HEALTHX,
        ST_HEALTHY,
        StDigitSet::TallNum,
        st_stuff.tallpercent,
    );
    stlib_init_bin_icon(
        &mut st_stuff.w_armsbg,
        ST_ARMSBGX,
        ST_ARMSBGY,
        st_stuff.armsbg,
    );
    for i in 0..6 {
        stlib_init_mult_icon(
            &mut st_stuff.w_arms[i as usize],
            ST_ARMSX + i % 3 * ST_ARMSXSPACE,
            ST_ARMSY + i / 3 * ST_ARMSYSPACE,
            StDigitSet::Arms(i as usize),
        );
    }
    stlib_init_num(
        &mut st_stuff.w_frags,
        ST_FRAGSX,
        ST_FRAGSY,
        StDigitSet::TallNum,
        ST_FRAGSWIDTH,
    );
    stlib_init_mult_icon(
        &mut st_stuff.w_faces,
        ST_FACESX,
        ST_FACESY,
        StDigitSet::Faces,
    );
    stlib_init_percent(
        &mut st_stuff.w_armor,
        ST_ARMORX,
        ST_ARMORY,
        StDigitSet::TallNum,
        st_stuff.tallpercent,
    );
    stlib_init_mult_icon(
        &mut st_stuff.w_keyboxes[0],
        ST_KEY0X,
        ST_KEY0Y,
        StDigitSet::Keys,
    );
    stlib_init_mult_icon(
        &mut st_stuff.w_keyboxes[1],
        ST_KEY1X,
        ST_KEY1Y,
        StDigitSet::Keys,
    );
    stlib_init_mult_icon(
        &mut st_stuff.w_keyboxes[2],
        ST_KEY2X,
        ST_KEY2Y,
        StDigitSet::Keys,
    );
    stlib_init_num(
        &mut st_stuff.w_ammo[0],
        ST_AMMO0X,
        ST_AMMO0Y,
        StDigitSet::ShortNum,
        ST_AMMO0WIDTH,
    );
    stlib_init_num(
        &mut st_stuff.w_ammo[1],
        ST_AMMO1X,
        ST_AMMO1Y,
        StDigitSet::ShortNum,
        ST_AMMO1WIDTH,
    );
    stlib_init_num(
        &mut st_stuff.w_ammo[2],
        ST_AMMO2X,
        ST_AMMO2Y,
        StDigitSet::ShortNum,
        ST_AMMO2WIDTH,
    );
    stlib_init_num(
        &mut st_stuff.w_ammo[3],
        ST_AMMO3X,
        ST_AMMO3Y,
        StDigitSet::ShortNum,
        ST_AMMO3WIDTH,
    );
    stlib_init_num(
        &mut st_stuff.w_maxammo[0],
        ST_MAXAMMO0X,
        ST_MAXAMMO0Y,
        StDigitSet::ShortNum,
        ST_MAXAMMO0WIDTH,
    );
    stlib_init_num(
        &mut st_stuff.w_maxammo[1],
        ST_MAXAMMO1X,
        ST_MAXAMMO1Y,
        StDigitSet::ShortNum,
        ST_MAXAMMO1WIDTH,
    );
    stlib_init_num(
        &mut st_stuff.w_maxammo[2],
        ST_MAXAMMO2X,
        ST_MAXAMMO2Y,
        StDigitSet::ShortNum,
        ST_MAXAMMO2WIDTH,
    );
    stlib_init_num(
        &mut st_stuff.w_maxammo[3],
        ST_MAXAMMO3X,
        ST_MAXAMMO3Y,
        StDigitSet::ShortNum,
        ST_MAXAMMO3WIDTH,
    );
}
pub fn st_start(state: &mut GameState) {
    if !state.ui.st_stuff.st_stopped {
        st_stop(state);
    }
    st_init_data(state);
    create_widgets(&mut state.game.g_game, &mut state.ui.st_stuff);
    state.ui.st_stuff.st_stopped = false;
}
pub fn st_stop(state: &mut GameState) {
    if state.ui.st_stuff.st_stopped {
        return;
    }
    let pal = lump_bytes(
        &*state.assets.fs,
        &mut state.assets.w_wad,
        state.ui.st_stuff.lu_palette,
    );
    set_palette(&mut state.io.i_video, &pal[..768]);
    state.ui.st_stuff.st_stopped = true;
}
pub fn st_init(state: &mut GameState) {
    st_load_data(state);
    state.ui.st_stuff.st_backing_screen = vec![0u8; (ST_WIDTH * ST_HEIGHT) as usize];
}
