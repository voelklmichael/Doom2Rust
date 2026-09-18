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
use crate::p_mobj::mobj_t;
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
use crate::sounds::{sfx_swtchn, sfx_swtchx};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BWhere {
    top = 0,
    middle = 1,
    bottom = 2,
}
#[derive(Copy, Clone)]
#[repr(C)]
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
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1BRN1\0\0"),
        name2: FixedCStr(*b"SW2BRN1\0\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1BRN2\0\0"),
        name2: FixedCStr(*b"SW2BRN2\0\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1BRNGN\0"),
        name2: FixedCStr(*b"SW2BRNGN\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1BROWN\0"),
        name2: FixedCStr(*b"SW2BROWN\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1COMM\0\0"),
        name2: FixedCStr(*b"SW2COMM\0\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1COMP\0\0"),
        name2: FixedCStr(*b"SW2COMP\0\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1DIRT\0\0"),
        name2: FixedCStr(*b"SW2DIRT\0\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1EXIT\0\0"),
        name2: FixedCStr(*b"SW2EXIT\0\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1GRAY\0\0"),
        name2: FixedCStr(*b"SW2GRAY\0\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1GRAY1\0"),
        name2: FixedCStr(*b"SW2GRAY1\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1METAL\0"),
        name2: FixedCStr(*b"SW2METAL\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1PIPE\0\0"),
        name2: FixedCStr(*b"SW2PIPE\0\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1SLAD\0\0"),
        name2: FixedCStr(*b"SW2SLAD\0\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1STARG\0"),
        name2: FixedCStr(*b"SW2STARG\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1STON1\0"),
        name2: FixedCStr(*b"SW2STON1\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1STON2\0"),
        name2: FixedCStr(*b"SW2STON2\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1STONE\0"),
        name2: FixedCStr(*b"SW2STONE\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1STRTN\0"),
        name2: FixedCStr(*b"SW2STRTN\0"),
        episode: 1_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1BLUE\0\0"),
        name2: FixedCStr(*b"SW2BLUE\0\0"),
        episode: 2_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1CMT\0\0\0"),
        name2: FixedCStr(*b"SW2CMT\0\0\0"),
        episode: 2_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1GARG\0\0"),
        name2: FixedCStr(*b"SW2GARG\0\0"),
        episode: 2_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1GSTON\0"),
        name2: FixedCStr(*b"SW2GSTON\0"),
        episode: 2_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1HOT\0\0\0"),
        name2: FixedCStr(*b"SW2HOT\0\0\0"),
        episode: 2_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1LION\0\0"),
        name2: FixedCStr(*b"SW2LION\0\0"),
        episode: 2_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1SATYR\0"),
        name2: FixedCStr(*b"SW2SATYR\0"),
        episode: 2_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1SKIN\0\0"),
        name2: FixedCStr(*b"SW2SKIN\0\0"),
        episode: 2_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1VINE\0\0"),
        name2: FixedCStr(*b"SW2VINE\0\0"),
        episode: 2_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1WOOD\0\0"),
        name2: FixedCStr(*b"SW2WOOD\0\0"),
        episode: 2_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1PANEL\0"),
        name2: FixedCStr(*b"SW2PANEL\0"),
        episode: 3_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1ROCK\0\0"),
        name2: FixedCStr(*b"SW2ROCK\0\0"),
        episode: 3_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1MET2\0\0"),
        name2: FixedCStr(*b"SW2MET2\0\0"),
        episode: 3_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1WDMET\0"),
        name2: FixedCStr(*b"SW2WDMET\0"),
        episode: 3_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1BRIK\0\0"),
        name2: FixedCStr(*b"SW2BRIK\0\0"),
        episode: 3_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1MOD1\0\0"),
        name2: FixedCStr(*b"SW2MOD1\0\0"),
        episode: 3_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1ZIM\0\0\0"),
        name2: FixedCStr(*b"SW2ZIM\0\0\0"),
        episode: 3_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1STON6\0"),
        name2: FixedCStr(*b"SW2STON6\0"),
        episode: 3_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1TEK\0\0\0"),
        name2: FixedCStr(*b"SW2TEK\0\0\0"),
        episode: 3_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1MARB\0\0"),
        name2: FixedCStr(*b"SW2MARB\0\0"),
        episode: 3_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"SW1SKULL\0"),
        name2: FixedCStr(*b"SW2SKULL\0"),
        episode: 3_i16,
    },
    switchlist_t {
        name1: FixedCStr(*b"\0\0\0\0\0\0\0\0\0"),
        name2: FixedCStr(*b"\0\0\0\0\0\0\0\0\0"),
        episode: 0_i16,
    },
];
pub struct PSwitchState {
    pub switchlist: [i32; 100],
    pub numswitches: i32,
    pub buttonlist: [button_t; 16],
}

impl PSwitchState {
    pub const fn new() -> Self {
        PSwitchState {
            switchlist: [0; 100],
            numswitches: 0,
            buttonlist: [button_t {
                line: LineId(0),
                where_0: BWhere::top,
                btexture: 0,
                btimer: 0,
                soundorg: SectorId(0),
            }; 16],
        }
    }
}

pub fn P_InitSwitchList(state: &mut GameState) {
    let mut i: i32 = 0;
    let mut index: i32 = 0;
    let mut episode: i32 = 0;
    episode = 1_i32;
    if state.doomstat.gamemode as u32 == GameMode_t::registered as i32 as u32
        || state.doomstat.gamemode as u32 == GameMode_t::retail as i32 as u32
    {
        episode = 2_i32;
    } else if state.doomstat.gamemode as u32 == GameMode_t::commercial as i32 as u32 {
        episode = 3_i32;
    }
    index = 0_i32;
    i = 0_i32;
    while i < MAXSWITCHES {
        if alphSwitchList[i as usize].episode == 0 {
            state.p_switch.numswitches = index / 2_i32;
            state.p_switch.switchlist[index as usize] = -1_i32;
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
pub fn P_StartButton(
    state: &mut GameState,
    mut line: LineId,
    mut w: BWhere,
    mut texture: i32,
    mut time: i32,
) {
    let mut i: i32 = 0;
    i = 0_i32;
    while i < MAXBUTTONS {
        if state.p_switch.buttonlist[i as usize].btimer != 0
            && state.p_switch.buttonlist[i as usize].line == line
        {
            return;
        }
        i += 1;
    }
    i = 0_i32;
    while i < MAXBUTTONS {
        if state.p_switch.buttonlist[i as usize].btimer == 0 {
            state.p_switch.buttonlist[i as usize].line = line;
            state.p_switch.buttonlist[i as usize].where_0 = w;
            state.p_switch.buttonlist[i as usize].btexture = texture;
            state.p_switch.buttonlist[i as usize].btimer = time;
            state.p_switch.buttonlist[i as usize].soundorg =
                state.p_setup.line(line).frontsector.unwrap();
            return;
        }
        i += 1;
    }
    I_Error("P_StartButton: no button slots left!");
}
pub fn P_ChangeSwitchTexture(state: &mut GameState, mut line: LineId, mut useAgain: i32) {
    let mut texTop: i32 = 0;
    let mut texMid: i32 = 0;
    let mut texBot: i32 = 0;
    let mut i: i32 = 0;
    let mut sound: i32 = 0;
    if useAgain == 0 {
        state.p_setup.line_mut(line).special = 0_i16;
    }
    let sidenum0 = state.p_setup.line(line).sidenum[0];
    texTop = state.p_setup.sides[sidenum0 as usize].toptexture as i32;
    texMid = state.p_setup.sides[sidenum0 as usize].midtexture as i32;
    texBot = state.p_setup.sides[sidenum0 as usize].bottomtexture as i32;
    sound = sfx_swtchn as i32;
    if state.p_setup.line(line).special as i32 == 11_i32 {
        sound = sfx_swtchx as i32;
    }
    i = 0_i32;
    while i < state.p_switch.numswitches * 2_i32 {
        if state.p_switch.switchlist[i as usize] == texTop {
            S_StartSound(
                state,
                SoundOrigin::Sector(state.p_switch.buttonlist[0].soundorg),
                sound,
            );
            state.p_setup.sides[sidenum0 as usize].toptexture =
                state.p_switch.switchlist[(i ^ 1_i32) as usize] as i16;
            if useAgain != 0 {
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
                state.p_switch.switchlist[(i ^ 1_i32) as usize] as i16;
            if useAgain != 0 {
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
                state.p_switch.switchlist[(i ^ 1_i32) as usize] as i16;
            if useAgain != 0 {
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
        i += 1;
    }
}
pub unsafe fn P_UseSpecialLine(
    state: &mut GameState,
    mut thing: *mut mobj_t,
    mut line: LineId,
    mut side: i32,
) -> bool {
    let linev = state.p_setup.line(line);
    if side != 0 {
        match linev.special as i32 {
            124 => {}
            _ => return false,
        }
    }
    if (*thing).player.is_none() {
        if linev.flags as i32 & ML_SECRET != 0 {
            return false;
        }
        let mut current_block_6: u64;
        match linev.special as i32 {
            1 => {
                current_block_6 = 3640593987805443782;
            }
            32 => {
                current_block_6 = 12497116384748537712;
            }
            33 => {
                current_block_6 = 12497116384748537712;
            }
            34 => {
                current_block_6 = 3514215265213398008;
            }
            _ => return false,
        }
        match current_block_6 {
            12497116384748537712 => {
                current_block_6 = 3514215265213398008;
            }
            _ => {}
        }
        match current_block_6 {
            3514215265213398008 => {}
            _ => {}
        }
    }
    let mut current_block_108: u64;
    match linev.special as i32 {
        1 => {
            current_block_108 = 9599537275859496075;
        }
        26 => {
            current_block_108 = 9599537275859496075;
        }
        27 => {
            current_block_108 = 4020771665460505868;
        }
        28 => {
            current_block_108 = 14541717319785412967;
        }
        31 => {
            current_block_108 = 9823790708098527527;
        }
        32 => {
            current_block_108 = 16744586327905341259;
        }
        33 => {
            current_block_108 = 346168964044254087;
        }
        34 => {
            current_block_108 = 7551167036031974558;
        }
        117 => {
            current_block_108 = 16004491317120851547;
        }
        118 => {
            current_block_108 = 6634390297149606533;
        }
        7 => {
            if EV_BuildStairs(state, line, StairE::build8) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        9 => {
            if EV_DoDonut(state, line) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        11 => {
            P_ChangeSwitchTexture(state, line, 0_i32);
            G_ExitLevel(state);
            current_block_108 = 16981061190961355901;
        }
        14 => {
            if EV_DoPlat(state, line, PlattypeE::raiseAndChange, 32_i32) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        15 => {
            if EV_DoPlat(state, line, PlattypeE::raiseAndChange, 24_i32) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        18 => {
            if EV_DoFloor(state, line, FloorE::raiseFloorToNearest) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        20 => {
            if EV_DoPlat(state, line, PlattypeE::raiseToNearestAndChange, 0_i32) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        21 => {
            if EV_DoPlat(state, line, PlattypeE::downWaitUpStay, 0_i32) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        23 => {
            if EV_DoFloor(state, line, FloorE::lowerFloorToLowest) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        29 => {
            if EV_DoDoor(state, line, VldoorE::vld_normal) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        41 => {
            if EV_DoCeiling(state, line, CeilingE::lowerToFloor) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        71 => {
            if EV_DoFloor(state, line, FloorE::turboLower) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        49 => {
            if EV_DoCeiling(state, line, CeilingE::crushAndRaise) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        50 => {
            if EV_DoDoor(state, line, VldoorE::vld_close) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        51 => {
            P_ChangeSwitchTexture(state, line, 0_i32);
            G_SecretExitLevel(state);
            current_block_108 = 16981061190961355901;
        }
        55 => {
            if EV_DoFloor(state, line, FloorE::raiseFloorCrush) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        101 => {
            if EV_DoFloor(state, line, FloorE::raiseFloor) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        102 => {
            if EV_DoFloor(state, line, FloorE::lowerFloor) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        103 => {
            if EV_DoDoor(state, line, VldoorE::vld_open) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        111 => {
            if EV_DoDoor(state, line, VldoorE::vld_blazeRaise) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        112 => {
            if EV_DoDoor(state, line, VldoorE::vld_blazeOpen) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        113 => {
            if EV_DoDoor(state, line, VldoorE::vld_blazeClose) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        122 => {
            if EV_DoPlat(state, line, PlattypeE::blazeDWUS, 0_i32) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        127 => {
            if EV_BuildStairs(state, line, StairE::turbo16) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        131 => {
            if EV_DoFloor(state, line, FloorE::raiseFloorTurbo) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        133 => {
            current_block_108 = 6707790765423050264;
        }
        135 | 137 => {
            current_block_108 = 6707790765423050264;
        }
        140 => {
            if EV_DoFloor(state, line, FloorE::raiseFloor512) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        42 => {
            if EV_DoDoor(state, line, VldoorE::vld_close) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        43 => {
            if EV_DoCeiling(state, line, CeilingE::lowerToFloor) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        45 => {
            if EV_DoFloor(state, line, FloorE::lowerFloor) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        60 => {
            if EV_DoFloor(state, line, FloorE::lowerFloorToLowest) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        61 => {
            if EV_DoDoor(state, line, VldoorE::vld_open) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        62 => {
            if EV_DoPlat(state, line, PlattypeE::downWaitUpStay, 1_i32) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        63 => {
            if EV_DoDoor(state, line, VldoorE::vld_normal) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        64 => {
            if EV_DoFloor(state, line, FloorE::raiseFloor) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        66 => {
            if EV_DoPlat(state, line, PlattypeE::raiseAndChange, 24_i32) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        67 => {
            if EV_DoPlat(state, line, PlattypeE::raiseAndChange, 32_i32) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        65 => {
            if EV_DoFloor(state, line, FloorE::raiseFloorCrush) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        68 => {
            if EV_DoPlat(state, line, PlattypeE::raiseToNearestAndChange, 0_i32) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        69 => {
            if EV_DoFloor(state, line, FloorE::raiseFloorToNearest) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        70 => {
            if EV_DoFloor(state, line, FloorE::turboLower) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        114 => {
            if EV_DoDoor(state, line, VldoorE::vld_blazeRaise) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        115 => {
            if EV_DoDoor(state, line, VldoorE::vld_blazeOpen) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        116 => {
            if EV_DoDoor(state, line, VldoorE::vld_blazeClose) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        123 => {
            if EV_DoPlat(state, line, PlattypeE::blazeDWUS, 0_i32) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        132 => {
            if EV_DoFloor(state, line, FloorE::raiseFloorTurbo) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        99 => {
            current_block_108 = 16848555411549253182;
        }
        134 | 136 => {
            current_block_108 = 16848555411549253182;
        }
        138 => {
            EV_LightTurnOn(state, line, 255_i32);
            P_ChangeSwitchTexture(state, line, 1_i32);
            current_block_108 = 16981061190961355901;
        }
        139 => {
            EV_LightTurnOn(state, line, 35_i32);
            P_ChangeSwitchTexture(state, line, 1_i32);
            current_block_108 = 16981061190961355901;
        }
        _ => {
            current_block_108 = 16981061190961355901;
        }
    }
    match current_block_108 {
        9599537275859496075 => {
            current_block_108 = 4020771665460505868;
        }
        6707790765423050264 => {
            if EV_DoLockedDoor(state, line, VldoorE::vld_blazeOpen, (*thing).id) != 0 {
                P_ChangeSwitchTexture(state, line, 0_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        16848555411549253182 => {
            if EV_DoLockedDoor(state, line, VldoorE::vld_blazeOpen, (*thing).id) != 0 {
                P_ChangeSwitchTexture(state, line, 1_i32);
            }
            current_block_108 = 16981061190961355901;
        }
        _ => {}
    }
    match current_block_108 {
        4020771665460505868 => {
            current_block_108 = 14541717319785412967;
        }
        _ => {}
    }
    match current_block_108 {
        14541717319785412967 => {
            current_block_108 = 9823790708098527527;
        }
        _ => {}
    }
    match current_block_108 {
        9823790708098527527 => {
            current_block_108 = 16744586327905341259;
        }
        _ => {}
    }
    match current_block_108 {
        16744586327905341259 => {
            current_block_108 = 346168964044254087;
        }
        _ => {}
    }
    match current_block_108 {
        346168964044254087 => {
            current_block_108 = 7551167036031974558;
        }
        _ => {}
    }
    match current_block_108 {
        7551167036031974558 => {
            current_block_108 = 16004491317120851547;
        }
        _ => {}
    }
    match current_block_108 {
        16004491317120851547 => {
            current_block_108 = 6634390297149606533;
        }
        _ => {}
    }
    match current_block_108 {
        6634390297149606533 => {
            EV_VerticalDoor(state, line, (*thing).id);
        }
        _ => {}
    }
    true
}
