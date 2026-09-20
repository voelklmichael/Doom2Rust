use crate::d_mode::GameMode;
use crate::doomstat::DoomstatState;
use crate::fixed_cstr::FixedCStr;
use crate::g_game::exit_level;
use crate::g_game::secret_exit_level;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::p_ceilng::do_ceiling;
use crate::p_ceilng::CeilingE;
use crate::p_doors::do_door;
use crate::p_doors::do_locked_door;
use crate::p_doors::ev_vertical_door;
use crate::p_doors::VldoorE;
use crate::p_floor::build_stairs;
use crate::p_floor::do_floor;
use crate::p_floor::FloorE;
use crate::p_floor::StairE;
use crate::p_lights::light_turn_on;
use crate::p_mobj::LineFlags;
use crate::p_mobj::MobjId;
use crate::p_setup::PSetupState;
use crate::r_data::RDataState;

use crate::p_plats::do_plat;
use crate::p_plats::PlattypeE;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::do_donut;
use crate::p_spec::Button;

use crate::r_data::texture_num_for_name;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BWhere {
    Top = 0,
    Middle = 1,
    Bottom = 2,
}
#[derive(Copy, Clone)]
pub struct SwitchList {
    pub name1: FixedCStr<9>,
    pub name2: FixedCStr<9>,
    pub episode: i16,
}
pub const MAXSWITCHES: i32 = 50;
pub const MAXBUTTONS: i32 = 16;
pub const BUTTONTIME: i32 = 35;
pub static ALPH_SWITCH_LIST: [SwitchList; 41] = [
    SwitchList {
        name1: FixedCStr(*b"SW1BRCOM\0"),
        name2: FixedCStr(*b"SW2BRCOM\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1BRN1\0\0"),
        name2: FixedCStr(*b"SW2BRN1\0\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1BRN2\0\0"),
        name2: FixedCStr(*b"SW2BRN2\0\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1BRNGN\0"),
        name2: FixedCStr(*b"SW2BRNGN\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1BROWN\0"),
        name2: FixedCStr(*b"SW2BROWN\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1COMM\0\0"),
        name2: FixedCStr(*b"SW2COMM\0\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1COMP\0\0"),
        name2: FixedCStr(*b"SW2COMP\0\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1DIRT\0\0"),
        name2: FixedCStr(*b"SW2DIRT\0\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1EXIT\0\0"),
        name2: FixedCStr(*b"SW2EXIT\0\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1GRAY\0\0"),
        name2: FixedCStr(*b"SW2GRAY\0\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1GRAY1\0"),
        name2: FixedCStr(*b"SW2GRAY1\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1METAL\0"),
        name2: FixedCStr(*b"SW2METAL\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1PIPE\0\0"),
        name2: FixedCStr(*b"SW2PIPE\0\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1SLAD\0\0"),
        name2: FixedCStr(*b"SW2SLAD\0\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1STARG\0"),
        name2: FixedCStr(*b"SW2STARG\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1STON1\0"),
        name2: FixedCStr(*b"SW2STON1\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1STON2\0"),
        name2: FixedCStr(*b"SW2STON2\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1STONE\0"),
        name2: FixedCStr(*b"SW2STONE\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1STRTN\0"),
        name2: FixedCStr(*b"SW2STRTN\0"),
        episode: 1,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1BLUE\0\0"),
        name2: FixedCStr(*b"SW2BLUE\0\0"),
        episode: 2,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1CMT\0\0\0"),
        name2: FixedCStr(*b"SW2CMT\0\0\0"),
        episode: 2,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1GARG\0\0"),
        name2: FixedCStr(*b"SW2GARG\0\0"),
        episode: 2,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1GSTON\0"),
        name2: FixedCStr(*b"SW2GSTON\0"),
        episode: 2,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1HOT\0\0\0"),
        name2: FixedCStr(*b"SW2HOT\0\0\0"),
        episode: 2,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1LION\0\0"),
        name2: FixedCStr(*b"SW2LION\0\0"),
        episode: 2,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1SATYR\0"),
        name2: FixedCStr(*b"SW2SATYR\0"),
        episode: 2,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1SKIN\0\0"),
        name2: FixedCStr(*b"SW2SKIN\0\0"),
        episode: 2,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1VINE\0\0"),
        name2: FixedCStr(*b"SW2VINE\0\0"),
        episode: 2,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1WOOD\0\0"),
        name2: FixedCStr(*b"SW2WOOD\0\0"),
        episode: 2,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1PANEL\0"),
        name2: FixedCStr(*b"SW2PANEL\0"),
        episode: 3,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1ROCK\0\0"),
        name2: FixedCStr(*b"SW2ROCK\0\0"),
        episode: 3,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1MET2\0\0"),
        name2: FixedCStr(*b"SW2MET2\0\0"),
        episode: 3,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1WDMET\0"),
        name2: FixedCStr(*b"SW2WDMET\0"),
        episode: 3,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1BRIK\0\0"),
        name2: FixedCStr(*b"SW2BRIK\0\0"),
        episode: 3,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1MOD1\0\0"),
        name2: FixedCStr(*b"SW2MOD1\0\0"),
        episode: 3,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1ZIM\0\0\0"),
        name2: FixedCStr(*b"SW2ZIM\0\0\0"),
        episode: 3,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1STON6\0"),
        name2: FixedCStr(*b"SW2STON6\0"),
        episode: 3,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1TEK\0\0\0"),
        name2: FixedCStr(*b"SW2TEK\0\0\0"),
        episode: 3,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1MARB\0\0"),
        name2: FixedCStr(*b"SW2MARB\0\0"),
        episode: 3,
    },
    SwitchList {
        name1: FixedCStr(*b"SW1SKULL\0"),
        name2: FixedCStr(*b"SW2SKULL\0"),
        episode: 3,
    },
    SwitchList {
        name1: FixedCStr(*b"\0\0\0\0\0\0\0\0\0"),
        name2: FixedCStr(*b"\0\0\0\0\0\0\0\0\0"),
        episode: 0,
    },
];
pub const EMPTY_BUTTON: Button = Button {
    line: LineId(0),
    position: BWhere::Top,
    btexture: 0,
    btimer: 0,
    soundorg: SectorId(0),
};

pub struct PSwitchState {
    pub switchlist: [i32; 100],
    pub numswitches: i32,
    pub buttonlist: [Button; 16],
}

impl Default for PSwitchState {
    fn default() -> Self {
        Self::new()
    }
}

impl PSwitchState {
    pub const fn new() -> Self {
        Self {
            switchlist: [0; 100],
            numswitches: 0,
            buttonlist: [EMPTY_BUTTON; 16],
        }
    }
}

pub fn init_switch_list(
    doomstat: &DoomstatState,
    p_switch: &mut PSwitchState,
    r_data: &RDataState,
) {
    let mut episode: i32 = 1;
    if doomstat.gamemode == GameMode::Registered || doomstat.gamemode == GameMode::Retail {
        episode = 2;
    } else if doomstat.gamemode == GameMode::Commercial {
        episode = 3;
    }
    let mut index: i32 = 0;
    let mut i: i32 = 0;
    while i < MAXSWITCHES {
        if ALPH_SWITCH_LIST[i as usize].episode == 0 {
            p_switch.numswitches = index / 2;
            p_switch.switchlist[index as usize] = -1;
            break;
        }
        if ALPH_SWITCH_LIST[i as usize].episode as i32 <= episode {
            p_switch.switchlist[index as usize] =
                texture_num_for_name(r_data, &ALPH_SWITCH_LIST[i as usize].name1.as_str());
            index += 1;
            p_switch.switchlist[index as usize] =
                texture_num_for_name(r_data, &ALPH_SWITCH_LIST[i as usize].name2.as_str());
            index += 1;
        }
        i += 1;
    }
}
pub fn start_button(
    p_setup: &PSetupState,
    p_switch: &mut PSwitchState,
    line: LineId,
    w: BWhere,
    texture: i32,
    time: i32,
) {
    for i in 0..(MAXBUTTONS as usize) {
        if p_switch.buttonlist[i].btimer != 0 && p_switch.buttonlist[i].line == line {
            return;
        }
    }
    for i in 0..(MAXBUTTONS as usize) {
        if p_switch.buttonlist[i].btimer == 0 {
            p_switch.buttonlist[i].line = line;
            p_switch.buttonlist[i].position = w;
            p_switch.buttonlist[i].btexture = texture;
            p_switch.buttonlist[i].btimer = time;
            p_switch.buttonlist[i].soundorg = p_setup.line(line).frontsector.unwrap();
            return;
        }
    }
    error("P_StartButton: no button slots left!");
}
pub fn change_switch_texture(state: &mut GameState, line: LineId, use_again: bool) {
    if !use_again {
        state.world.p_setup.line_mut(line).special = 0;
    }
    let sidenum0 = state.world.p_setup.line(line).sidenum[0];
    let tex_top: i32 = state.world.p_setup.sides[sidenum0 as usize].toptexture as i32;
    let tex_mid: i32 = state.world.p_setup.sides[sidenum0 as usize].midtexture as i32;
    let tex_bot: i32 = state.world.p_setup.sides[sidenum0 as usize].bottomtexture as i32;
    let mut sound: SfxName = SfxName::Swtchn;
    if state.world.p_setup.line(line).special as i32 == 11 {
        sound = SfxName::Swtchx;
    }
    for i in 0..state.world.p_switch.numswitches * 2 {
        if state.world.p_switch.switchlist[i as usize] == tex_top {
            s_start_sound(
                state,
                SoundOrigin::Sector(state.world.p_switch.buttonlist[0].soundorg),
                sound,
            );
            state.world.p_setup.sides[sidenum0 as usize].toptexture =
                state.world.p_switch.switchlist[(i ^ 1) as usize] as i16;
            if use_again {
                let texture = state.world.p_switch.switchlist[i as usize];
                start_button(
                    &state.world.p_setup,
                    &mut state.world.p_switch,
                    line,
                    BWhere::Top,
                    texture,
                    BUTTONTIME,
                );
            }
            return;
        } else if state.world.p_switch.switchlist[i as usize] == tex_mid {
            s_start_sound(
                state,
                SoundOrigin::Sector(state.world.p_switch.buttonlist[0].soundorg),
                sound,
            );
            state.world.p_setup.sides[sidenum0 as usize].midtexture =
                state.world.p_switch.switchlist[(i ^ 1) as usize] as i16;
            if use_again {
                let texture = state.world.p_switch.switchlist[i as usize];
                start_button(
                    &state.world.p_setup,
                    &mut state.world.p_switch,
                    line,
                    BWhere::Middle,
                    texture,
                    BUTTONTIME,
                );
            }
            return;
        } else if state.world.p_switch.switchlist[i as usize] == tex_bot {
            s_start_sound(
                state,
                SoundOrigin::Sector(state.world.p_switch.buttonlist[0].soundorg),
                sound,
            );
            state.world.p_setup.sides[sidenum0 as usize].bottomtexture =
                state.world.p_switch.switchlist[(i ^ 1) as usize] as i16;
            if use_again {
                let texture = state.world.p_switch.switchlist[i as usize];
                start_button(
                    &state.world.p_setup,
                    &mut state.world.p_switch,
                    line,
                    BWhere::Bottom,
                    texture,
                    BUTTONTIME,
                );
            }
            return;
        }
    }
}
pub fn use_special_line(state: &mut GameState, thing: MobjId, line: LineId, side: i32) -> bool {
    let linev = state.world.p_setup.line(line);
    if side != 0 && linev.special as i32 != 124 {
        return false;
    }
    if state.world.p_mobj.mo(thing).player.is_none() {
        if linev.flags.contains(LineFlags::SECRET) {
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
            if build_stairs(
                &mut state.world.p_setup,
                &mut state.world.p_spec,
                &mut state.world.p_tick,
                line,
                StairE::Build8,
            ) {
                change_switch_texture(state, line, false);
            }
        }
        9 => {
            if do_donut(state, line) {
                change_switch_texture(state, line, false);
            }
        }
        11 => {
            change_switch_texture(state, line, false);
            exit_level(&mut state.game.g_game);
        }
        14 => {
            if do_plat(state, line, PlattypeE::RaiseAndChange, 32) {
                change_switch_texture(state, line, false);
            }
        }
        15 => {
            if do_plat(state, line, PlattypeE::RaiseAndChange, 24) {
                change_switch_texture(state, line, false);
            }
        }
        18 => {
            if do_floor(state, line, FloorE::RaiseFloorToNearest) {
                change_switch_texture(state, line, false);
            }
        }
        20 => {
            if do_plat(state, line, PlattypeE::RaiseToNearestAndChange, 0) {
                change_switch_texture(state, line, false);
            }
        }
        21 => {
            if do_plat(state, line, PlattypeE::DownWaitUpStay, 0) {
                change_switch_texture(state, line, false);
            }
        }
        23 => {
            if do_floor(state, line, FloorE::LowerFloorToLowest) {
                change_switch_texture(state, line, false);
            }
        }
        29 => {
            if do_door(state, line, VldoorE::Normal) {
                change_switch_texture(state, line, false);
            }
        }
        41 => {
            if do_ceiling(
                &mut state.world.p_ceilng,
                &mut state.world.p_setup,
                &mut state.world.p_tick,
                line,
                CeilingE::LowerToFloor,
            ) {
                change_switch_texture(state, line, false);
            }
        }
        71 => {
            if do_floor(state, line, FloorE::TurboLower) {
                change_switch_texture(state, line, false);
            }
        }
        49 => {
            if do_ceiling(
                &mut state.world.p_ceilng,
                &mut state.world.p_setup,
                &mut state.world.p_tick,
                line,
                CeilingE::CrushAndRaise,
            ) {
                change_switch_texture(state, line, false);
            }
        }
        50 => {
            if do_door(state, line, VldoorE::Close) {
                change_switch_texture(state, line, false);
            }
        }
        51 => {
            change_switch_texture(state, line, false);
            secret_exit_level(
                &state.game.doomstat,
                &mut state.game.g_game,
                &state.assets.w_wad,
            );
        }
        55 => {
            if do_floor(state, line, FloorE::RaiseFloorCrush) {
                change_switch_texture(state, line, false);
            }
        }
        101 => {
            if do_floor(state, line, FloorE::RaiseFloor) {
                change_switch_texture(state, line, false);
            }
        }
        102 => {
            if do_floor(state, line, FloorE::LowerFloor) {
                change_switch_texture(state, line, false);
            }
        }
        103 => {
            if do_door(state, line, VldoorE::Open) {
                change_switch_texture(state, line, false);
            }
        }
        111 => {
            if do_door(state, line, VldoorE::BlazeRaise) {
                change_switch_texture(state, line, false);
            }
        }
        112 => {
            if do_door(state, line, VldoorE::BlazeOpen) {
                change_switch_texture(state, line, false);
            }
        }
        113 => {
            if do_door(state, line, VldoorE::BlazeClose) {
                change_switch_texture(state, line, false);
            }
        }
        122 => {
            if do_plat(state, line, PlattypeE::BlazeDWUS, 0) {
                change_switch_texture(state, line, false);
            }
        }
        127 => {
            if build_stairs(
                &mut state.world.p_setup,
                &mut state.world.p_spec,
                &mut state.world.p_tick,
                line,
                StairE::Turbo16,
            ) {
                change_switch_texture(state, line, false);
            }
        }
        131 => {
            if do_floor(state, line, FloorE::RaiseFloorTurbo) {
                change_switch_texture(state, line, false);
            }
        }
        140 => {
            if do_floor(state, line, FloorE::RaiseFloor512) {
                change_switch_texture(state, line, false);
            }
        }
        42 => {
            if do_door(state, line, VldoorE::Close) {
                change_switch_texture(state, line, true);
            }
        }
        43 => {
            if do_ceiling(
                &mut state.world.p_ceilng,
                &mut state.world.p_setup,
                &mut state.world.p_tick,
                line,
                CeilingE::LowerToFloor,
            ) {
                change_switch_texture(state, line, true);
            }
        }
        45 => {
            if do_floor(state, line, FloorE::LowerFloor) {
                change_switch_texture(state, line, true);
            }
        }
        60 => {
            if do_floor(state, line, FloorE::LowerFloorToLowest) {
                change_switch_texture(state, line, true);
            }
        }
        61 => {
            if do_door(state, line, VldoorE::Open) {
                change_switch_texture(state, line, true);
            }
        }
        62 => {
            if do_plat(state, line, PlattypeE::DownWaitUpStay, 1) {
                change_switch_texture(state, line, true);
            }
        }
        63 => {
            if do_door(state, line, VldoorE::Normal) {
                change_switch_texture(state, line, true);
            }
        }
        64 => {
            if do_floor(state, line, FloorE::RaiseFloor) {
                change_switch_texture(state, line, true);
            }
        }
        66 => {
            if do_plat(state, line, PlattypeE::RaiseAndChange, 24) {
                change_switch_texture(state, line, true);
            }
        }
        67 => {
            if do_plat(state, line, PlattypeE::RaiseAndChange, 32) {
                change_switch_texture(state, line, true);
            }
        }
        65 => {
            if do_floor(state, line, FloorE::RaiseFloorCrush) {
                change_switch_texture(state, line, true);
            }
        }
        68 => {
            if do_plat(state, line, PlattypeE::RaiseToNearestAndChange, 0) {
                change_switch_texture(state, line, true);
            }
        }
        69 => {
            if do_floor(state, line, FloorE::RaiseFloorToNearest) {
                change_switch_texture(state, line, true);
            }
        }
        70 => {
            if do_floor(state, line, FloorE::TurboLower) {
                change_switch_texture(state, line, true);
            }
        }
        114 => {
            if do_door(state, line, VldoorE::BlazeRaise) {
                change_switch_texture(state, line, true);
            }
        }
        115 => {
            if do_door(state, line, VldoorE::BlazeOpen) {
                change_switch_texture(state, line, true);
            }
        }
        116 => {
            if do_door(state, line, VldoorE::BlazeClose) {
                change_switch_texture(state, line, true);
            }
        }
        123 => {
            if do_plat(state, line, PlattypeE::BlazeDWUS, 0) {
                change_switch_texture(state, line, true);
            }
        }
        132 => {
            if do_floor(state, line, FloorE::RaiseFloorTurbo) {
                change_switch_texture(state, line, true);
            }
        }
        138 => {
            light_turn_on(&mut state.world.p_setup, line, 255);
            change_switch_texture(state, line, true);
        }
        139 => {
            light_turn_on(&mut state.world.p_setup, line, 35);
            change_switch_texture(state, line, true);
        }
        1 | 26 | 27 | 28 | 31 | 32 | 33 | 34 | 117 | 118 => {
            ev_vertical_door(state, line, thing);
        }
        133 | 135 | 137 => {
            if do_locked_door(state, line, VldoorE::BlazeOpen, thing) {
                change_switch_texture(state, line, false);
            }
        }
        99 | 134 | 136 => {
            if do_locked_door(state, line, VldoorE::BlazeOpen, thing) {
                change_switch_texture(state, line, true);
            }
        }
        _ => {}
    }
    true
}
