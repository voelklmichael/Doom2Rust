use crate::d_mode::GameMode_t;
use crate::fixed_cstr::FixedCStr;
use crate::g_game::G_ExitLevel;
use crate::g_game::G_SecretExitLevel;
use crate::game_state::GameState;
use crate::i_system::I_Error;
use crate::p_ceilng::CeilingE;
use crate::p_ceilng::EV_DoCeiling;
use crate::p_doors::EV_DoDoor;
use crate::p_doors::EV_DoLockedDoor;
use crate::p_doors::EV_VerticalDoor;
use crate::p_doors::VldoorE;
use crate::p_floor::EV_BuildStairs;
use crate::p_floor::EV_DoFloor;
use crate::p_floor::FloorE;
use crate::p_floor::StairE;
use crate::p_lights::EV_LightTurnOn;
use crate::p_mobj::MobjId;

use crate::p_plats::EV_DoPlat;
use crate::p_plats::PlattypeE;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::button_t;
use crate::p_spec::EV_DoDonut;
use crate::p_spec::ML_SECRET;
use crate::r_data::R_TextureNumForName;
use crate::s_sound::S_StartSound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BWhere {
    top = 0,
    middle = 1,
    bottom = 2,
}
#[derive(Copy, Clone)]
pub struct switchlist_t {
    pub name1: FixedCStr<9>,
    pub name2: FixedCStr<9>,
    pub episode: i16,
}
pub const MAXSWITCHES: i32 = 50;
pub const MAXBUTTONS: i32 = 16;
pub const BUTTONTIME: i32 = 35;
pub static alphSwitchList: [switchlist_t; 41] = [
    switchlist_t {
        name1: FixedCStr(*b"SW1BRCOM\0"),
        name2: FixedCStr(*b"SW2BRCOM\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1BRN1\0\0"),
        name2: FixedCStr(*b"SW2BRN1\0\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1BRN2\0\0"),
        name2: FixedCStr(*b"SW2BRN2\0\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1BRNGN\0"),
        name2: FixedCStr(*b"SW2BRNGN\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1BROWN\0"),
        name2: FixedCStr(*b"SW2BROWN\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1COMM\0\0"),
        name2: FixedCStr(*b"SW2COMM\0\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1COMP\0\0"),
        name2: FixedCStr(*b"SW2COMP\0\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1DIRT\0\0"),
        name2: FixedCStr(*b"SW2DIRT\0\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1EXIT\0\0"),
        name2: FixedCStr(*b"SW2EXIT\0\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1GRAY\0\0"),
        name2: FixedCStr(*b"SW2GRAY\0\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1GRAY1\0"),
        name2: FixedCStr(*b"SW2GRAY1\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1METAL\0"),
        name2: FixedCStr(*b"SW2METAL\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1PIPE\0\0"),
        name2: FixedCStr(*b"SW2PIPE\0\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1SLAD\0\0"),
        name2: FixedCStr(*b"SW2SLAD\0\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1STARG\0"),
        name2: FixedCStr(*b"SW2STARG\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1STON1\0"),
        name2: FixedCStr(*b"SW2STON1\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1STON2\0"),
        name2: FixedCStr(*b"SW2STON2\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1STONE\0"),
        name2: FixedCStr(*b"SW2STONE\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1STRTN\0"),
        name2: FixedCStr(*b"SW2STRTN\0"),
        episode: 1,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1BLUE\0\0"),
        name2: FixedCStr(*b"SW2BLUE\0\0"),
        episode: 2,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1CMT\0\0\0"),
        name2: FixedCStr(*b"SW2CMT\0\0\0"),
        episode: 2,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1GARG\0\0"),
        name2: FixedCStr(*b"SW2GARG\0\0"),
        episode: 2,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1GSTON\0"),
        name2: FixedCStr(*b"SW2GSTON\0"),
        episode: 2,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1HOT\0\0\0"),
        name2: FixedCStr(*b"SW2HOT\0\0\0"),
        episode: 2,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1LION\0\0"),
        name2: FixedCStr(*b"SW2LION\0\0"),
        episode: 2,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1SATYR\0"),
        name2: FixedCStr(*b"SW2SATYR\0"),
        episode: 2,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1SKIN\0\0"),
        name2: FixedCStr(*b"SW2SKIN\0\0"),
        episode: 2,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1VINE\0\0"),
        name2: FixedCStr(*b"SW2VINE\0\0"),
        episode: 2,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1WOOD\0\0"),
        name2: FixedCStr(*b"SW2WOOD\0\0"),
        episode: 2,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1PANEL\0"),
        name2: FixedCStr(*b"SW2PANEL\0"),
        episode: 3,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1ROCK\0\0"),
        name2: FixedCStr(*b"SW2ROCK\0\0"),
        episode: 3,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1MET2\0\0"),
        name2: FixedCStr(*b"SW2MET2\0\0"),
        episode: 3,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1WDMET\0"),
        name2: FixedCStr(*b"SW2WDMET\0"),
        episode: 3,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1BRIK\0\0"),
        name2: FixedCStr(*b"SW2BRIK\0\0"),
        episode: 3,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1MOD1\0\0"),
        name2: FixedCStr(*b"SW2MOD1\0\0"),
        episode: 3,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1ZIM\0\0\0"),
        name2: FixedCStr(*b"SW2ZIM\0\0\0"),
        episode: 3,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1STON6\0"),
        name2: FixedCStr(*b"SW2STON6\0"),
        episode: 3,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1TEK\0\0\0"),
        name2: FixedCStr(*b"SW2TEK\0\0\0"),
        episode: 3,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1MARB\0\0"),
        name2: FixedCStr(*b"SW2MARB\0\0"),
        episode: 3,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1SKULL\0"),
        name2: FixedCStr(*b"SW2SKULL\0"),
        episode: 3,
    },
    switchlist_t {
        name1: FixedCStr(*b"\0\0\0\0\0\0\0\0\0"),
        name2: FixedCStr(*b"\0\0\0\0\0\0\0\0\0"),
        episode: 0,
    },
];
pub const EMPTY_BUTTON: button_t = button_t {
    line: LineId(0),
    position: BWhere::top,
    btexture: 0,
    btimer: 0,
    soundorg: SectorId(0),
};

pub struct PSwitchState {
    pub switchlist: [i32; 100],
    pub numswitches: i32,
    pub buttonlist: [button_t; 16],
}

impl Default for PSwitchState {
    fn default() -> Self {
        Self::new()
    }
}

impl PSwitchState {
    pub const fn new() -> Self {
        PSwitchState {
            switchlist: [0; 100],
            numswitches: 0,
            buttonlist: [EMPTY_BUTTON; 16],
        }
    }
}

pub fn P_InitSwitchList(state: &mut GameState) {
    let mut i: i32;
    let mut index: i32;
    let mut episode: i32;
    episode = 1;
    if state.doomstat.gamemode as u32 == GameMode_t::registered as i32 as u32
        || state.doomstat.gamemode as u32 == GameMode_t::retail as i32 as u32
    {
        episode = 2;
    } else if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        episode = 3;
    }
    index = 0;
    i = 0;
    while i < MAXSWITCHES {
        if alphSwitchList[i as usize].episode == 0 {
            state.p_switch.numswitches = index / 2;
            state.p_switch.switchlist[index as usize] = -1;
            break;
        } else {
            if alphSwitchList[i as usize].episode as i32 <= episode {
                let fresh0 = index;
                index += 1;
                state.p_switch.switchlist[fresh0 as usize] = R_TextureNumForName(
                    &mut state.r_data,
                    &alphSwitchList[i as usize].name1.as_str(),
                );
                let fresh1 = index;
                index += 1;
                state.p_switch.switchlist[fresh1 as usize] = R_TextureNumForName(
                    &mut state.r_data,
                    &alphSwitchList[i as usize].name2.as_str(),
                );
            }
            i += 1;
        }
    }
}
pub fn P_StartButton(state: &mut GameState, line: LineId, w: BWhere, texture: i32, time: i32) {
    for i in 0..(MAXBUTTONS as usize) {
        if state.p_switch.buttonlist[i].btimer != 0 && state.p_switch.buttonlist[i].line == line {
            return;
        }
    }
    for i in 0..(MAXBUTTONS as usize) {
        if state.p_switch.buttonlist[i].btimer == 0 {
            state.p_switch.buttonlist[i].line = line;
            state.p_switch.buttonlist[i].position = w;
            state.p_switch.buttonlist[i].btexture = texture;
            state.p_switch.buttonlist[i].btimer = time;
            state.p_switch.buttonlist[i].soundorg = state.p_setup.line(line).frontsector.unwrap();
            return;
        }
    }
    I_Error("P_StartButton: no button slots left!");
}
pub fn P_ChangeSwitchTexture(state: &mut GameState, line: LineId, useAgain: bool) {
    let mut sound: i32;
    if !useAgain {
        state.p_setup.line_mut(line).special = 0;
    }
    let sidenum0 = state.p_setup.line(line).sidenum[0];
    let texTop: i32 = state.p_setup.sides[sidenum0 as usize].toptexture as i32;
    let texMid: i32 = state.p_setup.sides[sidenum0 as usize].midtexture as i32;
    let texBot: i32 = state.p_setup.sides[sidenum0 as usize].bottomtexture as i32;
    sound = SfxName::sfx_swtchn as i32;
    if state.p_setup.line(line).special as i32 == 11 {
        sound = SfxName::sfx_swtchx as i32;
    }
    for i in 0..state.p_switch.numswitches * 2 {
        if state.p_switch.switchlist[i as usize] == texTop {
            S_StartSound(
                state,
                SoundOrigin::Sector(state.p_switch.buttonlist[0].soundorg),
                sound,
            );
            state.p_setup.sides[sidenum0 as usize].toptexture =
                state.p_switch.switchlist[(i ^ 1) as usize] as i16;
            if useAgain {
                P_StartButton(
                    state,
                    line,
                    BWhere::top,
                    state.p_switch.switchlist[i as usize],
                    BUTTONTIME,
                );
            }
            return;
        } else if state.p_switch.switchlist[i as usize] == texMid {
            S_StartSound(
                state,
                SoundOrigin::Sector(state.p_switch.buttonlist[0].soundorg),
                sound,
            );
            state.p_setup.sides[sidenum0 as usize].midtexture =
                state.p_switch.switchlist[(i ^ 1) as usize] as i16;
            if useAgain {
                P_StartButton(
                    state,
                    line,
                    BWhere::middle,
                    state.p_switch.switchlist[i as usize],
                    BUTTONTIME,
                );
            }
            return;
        } else if state.p_switch.switchlist[i as usize] == texBot {
            S_StartSound(
                state,
                SoundOrigin::Sector(state.p_switch.buttonlist[0].soundorg),
                sound,
            );
            state.p_setup.sides[sidenum0 as usize].bottomtexture =
                state.p_switch.switchlist[(i ^ 1) as usize] as i16;
            if useAgain {
                P_StartButton(
                    state,
                    line,
                    BWhere::bottom,
                    state.p_switch.switchlist[i as usize],
                    BUTTONTIME,
                );
            }
            return;
        }
    }
}
pub fn P_UseSpecialLine(state: &mut GameState, thing: MobjId, line: LineId, side: i32) -> bool {
    let linev = state.p_setup.line(line);
    if side != 0 && linev.special as i32 != 124 {
        return false;
    }
    if state.p_mobj.mo(thing).player.is_none() {
        if linev.flags as i32 & ML_SECRET != 0 {
            return false;
        }
        if !matches!(linev.special as i32, 1 | 32 | 33 | 34) {
            return false;
        }
    }
    // Arms call the EV_* function for its effect and then flip the switch if it
    // did anything; folding that call into a guard would hide the side effect.
    #[allow(clippy::collapsible_match)]
    match linev.special as i32 {
        7 => {
            if EV_BuildStairs(state, line, StairE::build8) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        9 => {
            if EV_DoDonut(state, line) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        11 => {
            P_ChangeSwitchTexture(state, line, false);
            G_ExitLevel(state);
        }
        14 => {
            if EV_DoPlat(state, line, PlattypeE::raiseAndChange, 32) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        15 => {
            if EV_DoPlat(state, line, PlattypeE::raiseAndChange, 24) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        18 => {
            if EV_DoFloor(state, line, FloorE::raiseFloorToNearest) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        20 => {
            if EV_DoPlat(state, line, PlattypeE::raiseToNearestAndChange, 0) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        21 => {
            if EV_DoPlat(state, line, PlattypeE::downWaitUpStay, 0) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        23 => {
            if EV_DoFloor(state, line, FloorE::lowerFloorToLowest) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        29 => {
            if EV_DoDoor(state, line, VldoorE::vld_normal) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        41 => {
            if EV_DoCeiling(state, line, CeilingE::lowerToFloor) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        71 => {
            if EV_DoFloor(state, line, FloorE::turboLower) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        49 => {
            if EV_DoCeiling(state, line, CeilingE::crushAndRaise) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        50 => {
            if EV_DoDoor(state, line, VldoorE::vld_close) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        51 => {
            P_ChangeSwitchTexture(state, line, false);
            G_SecretExitLevel(state);
        }
        55 => {
            if EV_DoFloor(state, line, FloorE::raiseFloorCrush) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        101 => {
            if EV_DoFloor(state, line, FloorE::raiseFloor) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        102 => {
            if EV_DoFloor(state, line, FloorE::lowerFloor) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        103 => {
            if EV_DoDoor(state, line, VldoorE::vld_open) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        111 => {
            if EV_DoDoor(state, line, VldoorE::vld_blazeRaise) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        112 => {
            if EV_DoDoor(state, line, VldoorE::vld_blazeOpen) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        113 => {
            if EV_DoDoor(state, line, VldoorE::vld_blazeClose) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        122 => {
            if EV_DoPlat(state, line, PlattypeE::blazeDWUS, 0) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        127 => {
            if EV_BuildStairs(state, line, StairE::turbo16) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        131 => {
            if EV_DoFloor(state, line, FloorE::raiseFloorTurbo) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        140 => {
            if EV_DoFloor(state, line, FloorE::raiseFloor512) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        42 => {
            if EV_DoDoor(state, line, VldoorE::vld_close) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        43 => {
            if EV_DoCeiling(state, line, CeilingE::lowerToFloor) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        45 => {
            if EV_DoFloor(state, line, FloorE::lowerFloor) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        60 => {
            if EV_DoFloor(state, line, FloorE::lowerFloorToLowest) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        61 => {
            if EV_DoDoor(state, line, VldoorE::vld_open) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        62 => {
            if EV_DoPlat(state, line, PlattypeE::downWaitUpStay, 1) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        63 => {
            if EV_DoDoor(state, line, VldoorE::vld_normal) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        64 => {
            if EV_DoFloor(state, line, FloorE::raiseFloor) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        66 => {
            if EV_DoPlat(state, line, PlattypeE::raiseAndChange, 24) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        67 => {
            if EV_DoPlat(state, line, PlattypeE::raiseAndChange, 32) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        65 => {
            if EV_DoFloor(state, line, FloorE::raiseFloorCrush) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        68 => {
            if EV_DoPlat(state, line, PlattypeE::raiseToNearestAndChange, 0) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        69 => {
            if EV_DoFloor(state, line, FloorE::raiseFloorToNearest) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        70 => {
            if EV_DoFloor(state, line, FloorE::turboLower) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        114 => {
            if EV_DoDoor(state, line, VldoorE::vld_blazeRaise) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        115 => {
            if EV_DoDoor(state, line, VldoorE::vld_blazeOpen) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        116 => {
            if EV_DoDoor(state, line, VldoorE::vld_blazeClose) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        123 => {
            if EV_DoPlat(state, line, PlattypeE::blazeDWUS, 0) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        132 => {
            if EV_DoFloor(state, line, FloorE::raiseFloorTurbo) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        138 => {
            EV_LightTurnOn(state, line, 255);
            P_ChangeSwitchTexture(state, line, true);
        }
        139 => {
            EV_LightTurnOn(state, line, 35);
            P_ChangeSwitchTexture(state, line, true);
        }
        1 | 26 | 27 | 28 | 31 | 32 | 33 | 34 | 117 | 118 => {
            EV_VerticalDoor(state, line, thing);
        }
        133 | 135 | 137 => {
            if EV_DoLockedDoor(state, line, VldoorE::vld_blazeOpen, thing) {
                P_ChangeSwitchTexture(state, line, false);
            }
        }
        99 | 134 | 136 => {
            if EV_DoLockedDoor(state, line, VldoorE::vld_blazeOpen, thing) {
                P_ChangeSwitchTexture(state, line, true);
            }
        }
        _ => {}
    }
    true
}
