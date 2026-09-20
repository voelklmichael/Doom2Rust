use crate::d_mode::GameMode;
use crate::doomstat::DoomstatState;
use crate::fixed_cstr::FixedCStr;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::p_mobj::LineFlags;
use crate::p_mobj::MobjId;
use crate::p_setup::PSetupState;
use crate::r_data::RDataState;

use crate::line_effects::use_rule;
use crate::p_setup::LineId;
use crate::p_setup::SectorId;
use crate::p_spec::Button;

use crate::r_data::texture_num_for_name;
use crate::s_sound::s_start_sound;
use crate::s_sound::SoundOrigin;
use crate::sounds::SfxName;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BWhere {
    Top,
    Middle,
    Bottom,
}
#[derive(Copy, Clone)]
pub struct SwitchList {
    pub name1: FixedCStr<9>,
    pub name2: FixedCStr<9>,
    pub episode: i16,
}
pub const MAXSWITCHES: usize = 50;
pub const MAXBUTTONS: usize = 16;
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
    for entry in ALPH_SWITCH_LIST.iter().take(MAXSWITCHES) {
        if entry.episode == 0 {
            p_switch.numswitches = index / 2;
            p_switch.switchlist[index as usize] = -1;
            break;
        }
        if i32::from(entry.episode) <= episode {
            p_switch.switchlist[index as usize] =
                texture_num_for_name(r_data, &entry.name1.as_str());
            index += 1;
            p_switch.switchlist[index as usize] =
                texture_num_for_name(r_data, &entry.name2.as_str());
            index += 1;
        }
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
    for i in 0..MAXBUTTONS {
        if p_switch.buttonlist[i].btimer != 0 && p_switch.buttonlist[i].line == line {
            return;
        }
    }
    for i in 0..MAXBUTTONS {
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
    let sidenum0 = state.world.p_setup.line(line).front_side().0;
    let tex_top: i32 = i32::from(state.world.p_setup.sides[sidenum0 as usize].toptexture);
    let tex_mid: i32 = i32::from(state.world.p_setup.sides[sidenum0 as usize].midtexture);
    let tex_bot: i32 = i32::from(state.world.p_setup.sides[sidenum0 as usize].bottomtexture);
    let sound = if i32::from(state.world.p_setup.line(line).special) == 11 {
        SfxName::Swtchx
    } else {
        SfxName::Swtchn
    };
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
    if side != 0 && i32::from(linev.special) != 124 {
        return false;
    }
    if state.world.p_mobj.mo(thing).player.is_none() {
        if linev.flags.contains(LineFlags::SECRET) {
            return false;
        }
        if !matches!(i32::from(linev.special), 1 | 32 | 33 | 34) {
            return false;
        }
    }
    let Some(rule) = use_rule(linev.special) else {
        return true;
    };
    if rule.effect.ends_the_level() {
        change_switch_texture(state, line, rule.repeatable);
        rule.effect.run(state, line, side, thing);
    } else if rule.effect.run(state, line, side, thing) {
        change_switch_texture(state, line, rule.repeatable);
    }
    true
}
