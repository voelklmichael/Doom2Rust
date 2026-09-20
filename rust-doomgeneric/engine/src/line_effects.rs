//! What a special line does when something crosses it or a player uses it.
//!
//! Vanilla wrote this twice, as two 300-line `switch` statements (`P_CrossSpecialLine` for the
//! lines you walk over, `P_UseSpecialLine` for switches and doors). Both are the same handful of
//! effects (open a door, lower a floor, start a platform, ...) chosen by the line's special
//! number, so here the special number is looked up in a table of [`LineEffect`]s and the two
//! callers only differ in what they do around the effect.

use crate::g_game::{exit_level, secret_exit_level};
use crate::game_state::GameState;
use crate::p_ceilng::{ceiling_crush_stop, do_ceiling, CeilingE};
use crate::p_doors::{do_door, do_locked_door, ev_vertical_door, VldoorE};
use crate::p_floor::{build_stairs, do_floor, FloorE, StairE};
use crate::p_lights::{light_turn_on, start_light_strobing, turn_tag_lights_off};
use crate::p_mobj::MobjId;
use crate::p_plats::{do_plat, stop_plat, PlattypeE};
use crate::p_setup::LineId;
use crate::p_spec::do_donut;
use crate::p_telept::teleport;

/// One thing a special line can do.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineEffect {
    Door(VldoorE),
    /// A door that needs a key the toucher has.
    LockedDoor(VldoorE),
    /// The door in the line's sector, opened by the player who used the line.
    ManualDoor,
    Floor(FloorE),
    Ceiling(CeilingE),
    /// A platform, and the height it raises by (for the change-texture platforms).
    Plat(PlattypeE, i32),
    Stairs(StairE),
    Donut,
    /// Set the tagged sectors' light to a level (0 = the darkest neighbour).
    LightOn(i32),
    LightsOff,
    LightStrobe,
    /// Stop the platforms with the line's tag.
    StopPlat,
    /// Stop the crushers with the line's tag.
    CrushStop,
    Teleport,
    Exit,
    SecretExit,
}

impl LineEffect {
    /// Does the effect. `true` if it did something (a door that is already moving, a floor
    /// already at its destination and so on make it `false`), which is when a switch flips.
    pub fn run(self, state: &mut GameState, line: LineId, side: i32, thing: MobjId) -> bool {
        let tag = i32::from(state.world.p_setup.line(line).tag);
        match self {
            Self::Door(kind) => do_door(state, line, kind),
            Self::LockedDoor(kind) => do_locked_door(state, line, kind, thing),
            Self::ManualDoor => {
                ev_vertical_door(state, line, thing);
                false
            }
            Self::Floor(kind) => do_floor(state, line, kind),
            Self::Ceiling(kind) => do_ceiling(
                &mut state.world.p_ceilng,
                &mut state.world.p_setup,
                &mut state.world.p_tick,
                line,
                kind,
            ),
            Self::Plat(kind, amount) => do_plat(state, line, kind, amount),
            Self::Stairs(kind) => build_stairs(
                &mut state.world.p_setup,
                &mut state.world.p_spec,
                &mut state.world.p_tick,
                line,
                kind,
            ),
            Self::Donut => do_donut(state, line),
            Self::LightOn(level) => {
                light_turn_on(&mut state.world.p_setup, line, level);
                true
            }
            Self::LightsOff => {
                turn_tag_lights_off(&mut state.world.p_setup, line);
                true
            }
            Self::LightStrobe => {
                start_light_strobing(&mut state.world, line);
                true
            }
            Self::StopPlat => {
                stop_plat(&mut state.world.p_plats, &state.world.p_tick, tag);
                true
            }
            Self::CrushStop => {
                ceiling_crush_stop(&mut state.world.p_ceilng, &state.world.p_tick, tag);
                true
            }
            Self::Teleport => teleport(state, line, side, thing),
            Self::Exit => {
                exit_level(&mut state.game.g_game);
                true
            }
            Self::SecretExit => {
                secret_exit_level(
                    &state.game.doomstat,
                    &mut state.game.g_game,
                    &state.assets.w_wad,
                );
                true
            }
        }
    }

    /// A switch that ends the level flips before the level ends, not after.
    pub const fn ends_the_level(self) -> bool {
        matches!(self, Self::Exit | Self::SecretExit)
    }
}

/// What happens when a thing walks over a line with a given special.
pub struct WalkRule {
    pub effects: &'static [LineEffect],
    /// The line loses its special afterwards (it works once), whatever the effect did.
    pub clears: bool,
    /// Only monsters and other non-players set it off.
    pub monsters_only: bool,
}

const fn walk(effects: &'static [LineEffect], clears: bool) -> Option<WalkRule> {
    Some(WalkRule {
        effects,
        clears,
        monsters_only: false,
    })
}

const fn monsters_walk(effects: &'static [LineEffect], clears: bool) -> Option<WalkRule> {
    Some(WalkRule {
        effects,
        clears,
        monsters_only: true,
    })
}

/// The rule for a line special that is triggered by walking over the line (the `W1` and `WR`
/// lines of the Doom specials list), if it is one.
pub const fn walk_rule(special: i16) -> Option<WalkRule> {
    use LineEffect::*;
    match special {
        2 => walk(&[Door(VldoorE::Open)], true),
        3 => walk(&[Door(VldoorE::Close)], true),
        4 => walk(&[Door(VldoorE::Normal)], true),
        5 => walk(&[Floor(FloorE::RaiseFloor)], true),
        6 => walk(&[Ceiling(CeilingE::FastCrushAndRaise)], true),
        8 => walk(&[Stairs(StairE::Build8)], true),
        10 => walk(&[Plat(PlattypeE::DownWaitUpStay, 0)], true),
        12 => walk(&[LightOn(0)], true),
        13 => walk(&[LightOn(255)], true),
        16 => walk(&[Door(VldoorE::Close30ThenOpen)], true),
        17 => walk(&[LightStrobe], true),
        19 => walk(&[Floor(FloorE::LowerFloor)], true),
        22 => walk(&[Plat(PlattypeE::RaiseToNearestAndChange, 0)], true),
        25 => walk(&[Ceiling(CeilingE::CrushAndRaise)], true),
        30 => walk(&[Floor(FloorE::RaiseToTexture)], true),
        35 => walk(&[LightOn(35)], true),
        36 => walk(&[Floor(FloorE::TurboLower)], true),
        37 => walk(&[Floor(FloorE::LowerAndChange)], true),
        38 => walk(&[Floor(FloorE::LowerFloorToLowest)], true),
        39 => walk(&[Teleport], true),
        40 => walk(
            &[
                Ceiling(CeilingE::RaiseToHighest),
                Floor(FloorE::LowerFloorToLowest),
            ],
            true,
        ),
        44 => walk(&[Ceiling(CeilingE::LowerAndCrush)], true),
        52 => walk(&[Exit], false),
        53 => walk(&[Plat(PlattypeE::PerpetualRaise, 0)], true),
        54 => walk(&[StopPlat], true),
        56 => walk(&[Floor(FloorE::RaiseFloorCrush)], true),
        57 => walk(&[CrushStop], true),
        58 => walk(&[Floor(FloorE::RaiseFloor24)], true),
        59 => walk(&[Floor(FloorE::RaiseFloor24AndChange)], true),
        72 => walk(&[Ceiling(CeilingE::LowerAndCrush)], false),
        73 => walk(&[Ceiling(CeilingE::CrushAndRaise)], false),
        74 => walk(&[CrushStop], false),
        75 => walk(&[Door(VldoorE::Close)], false),
        76 => walk(&[Door(VldoorE::Close30ThenOpen)], false),
        77 => walk(&[Ceiling(CeilingE::FastCrushAndRaise)], false),
        79 => walk(&[LightOn(35)], false),
        80 => walk(&[LightOn(0)], false),
        81 => walk(&[LightOn(255)], false),
        82 => walk(&[Floor(FloorE::LowerFloorToLowest)], false),
        83 => walk(&[Floor(FloorE::LowerFloor)], false),
        84 => walk(&[Floor(FloorE::LowerAndChange)], false),
        86 => walk(&[Door(VldoorE::Open)], false),
        87 => walk(&[Plat(PlattypeE::PerpetualRaise, 0)], false),
        88 => walk(&[Plat(PlattypeE::DownWaitUpStay, 0)], false),
        89 => walk(&[StopPlat], false),
        90 => walk(&[Door(VldoorE::Normal)], false),
        91 => walk(&[Floor(FloorE::RaiseFloor)], false),
        92 => walk(&[Floor(FloorE::RaiseFloor24)], false),
        93 => walk(&[Floor(FloorE::RaiseFloor24AndChange)], false),
        94 => walk(&[Floor(FloorE::RaiseFloorCrush)], false),
        95 => walk(&[Plat(PlattypeE::RaiseToNearestAndChange, 0)], false),
        96 => walk(&[Floor(FloorE::RaiseToTexture)], false),
        97 => walk(&[Teleport], false),
        98 => walk(&[Floor(FloorE::TurboLower)], false),
        100 => walk(&[Stairs(StairE::Turbo16)], true),
        104 => walk(&[LightsOff], true),
        105 => walk(&[Door(VldoorE::BlazeRaise)], false),
        106 => walk(&[Door(VldoorE::BlazeOpen)], false),
        107 => walk(&[Door(VldoorE::BlazeClose)], false),
        108 => walk(&[Door(VldoorE::BlazeRaise)], true),
        109 => walk(&[Door(VldoorE::BlazeOpen)], true),
        110 => walk(&[Door(VldoorE::BlazeClose)], true),
        119 => walk(&[Floor(FloorE::RaiseFloorToNearest)], true),
        120 => walk(&[Plat(PlattypeE::BlazeDWUS, 0)], false),
        121 => walk(&[Plat(PlattypeE::BlazeDWUS, 0)], true),
        124 => walk(&[SecretExit], false),
        125 => monsters_walk(&[Teleport], true),
        126 => monsters_walk(&[Teleport], false),
        128 => walk(&[Floor(FloorE::RaiseFloorToNearest)], false),
        129 => walk(&[Floor(FloorE::RaiseFloorTurbo)], false),
        130 => walk(&[Floor(FloorE::RaiseFloorTurbo)], true),
        141 => walk(&[Ceiling(CeilingE::SilentCrushAndRaise)], true),
        _ => None,
    }
}

/// What using a line with a given special does.
pub struct UseRule {
    pub effect: LineEffect,
    /// A switch that can be used again (`SR`), rather than once (`S1`).
    pub repeatable: bool,
}

const fn once(effect: LineEffect) -> Option<UseRule> {
    Some(UseRule {
        effect,
        repeatable: false,
    })
}

const fn repeatable(effect: LineEffect) -> Option<UseRule> {
    Some(UseRule {
        effect,
        repeatable: true,
    })
}

/// The rule for a line special that a player triggers by using the line (the `S1`, `SR`, `D1` and
/// `DR` lines of the Doom specials list), if it is one.
pub const fn use_rule(special: i16) -> Option<UseRule> {
    use LineEffect::*;
    match special {
        1 | 26 | 27 | 28 | 31 | 32 | 33 | 34 | 117 | 118 => once(ManualDoor),
        7 => once(Stairs(StairE::Build8)),
        9 => once(Donut),
        11 => once(Exit),
        14 => once(Plat(PlattypeE::RaiseAndChange, 32)),
        15 => once(Plat(PlattypeE::RaiseAndChange, 24)),
        18 => once(Floor(FloorE::RaiseFloorToNearest)),
        20 => once(Plat(PlattypeE::RaiseToNearestAndChange, 0)),
        21 => once(Plat(PlattypeE::DownWaitUpStay, 0)),
        23 => once(Floor(FloorE::LowerFloorToLowest)),
        29 => once(Door(VldoorE::Normal)),
        41 => once(Ceiling(CeilingE::LowerToFloor)),
        42 => repeatable(Door(VldoorE::Close)),
        43 => repeatable(Ceiling(CeilingE::LowerToFloor)),
        45 => repeatable(Floor(FloorE::LowerFloor)),
        49 => once(Ceiling(CeilingE::CrushAndRaise)),
        50 => once(Door(VldoorE::Close)),
        51 => once(SecretExit),
        55 => once(Floor(FloorE::RaiseFloorCrush)),
        60 => repeatable(Floor(FloorE::LowerFloorToLowest)),
        61 => repeatable(Door(VldoorE::Open)),
        62 => repeatable(Plat(PlattypeE::DownWaitUpStay, 1)),
        63 => repeatable(Door(VldoorE::Normal)),
        64 => repeatable(Floor(FloorE::RaiseFloor)),
        65 => repeatable(Floor(FloorE::RaiseFloorCrush)),
        66 => repeatable(Plat(PlattypeE::RaiseAndChange, 24)),
        67 => repeatable(Plat(PlattypeE::RaiseAndChange, 32)),
        68 => repeatable(Plat(PlattypeE::RaiseToNearestAndChange, 0)),
        69 => repeatable(Floor(FloorE::RaiseFloorToNearest)),
        70 => repeatable(Floor(FloorE::TurboLower)),
        71 => once(Floor(FloorE::TurboLower)),
        99 | 134 | 136 => repeatable(LockedDoor(VldoorE::BlazeOpen)),
        101 => once(Floor(FloorE::RaiseFloor)),
        102 => once(Floor(FloorE::LowerFloor)),
        103 => once(Door(VldoorE::Open)),
        111 => once(Door(VldoorE::BlazeRaise)),
        112 => once(Door(VldoorE::BlazeOpen)),
        113 => once(Door(VldoorE::BlazeClose)),
        114 => repeatable(Door(VldoorE::BlazeRaise)),
        115 => repeatable(Door(VldoorE::BlazeOpen)),
        116 => repeatable(Door(VldoorE::BlazeClose)),
        122 => once(Plat(PlattypeE::BlazeDWUS, 0)),
        123 => repeatable(Plat(PlattypeE::BlazeDWUS, 0)),
        127 => once(Stairs(StairE::Turbo16)),
        131 => once(Floor(FloorE::RaiseFloorTurbo)),
        132 => repeatable(Floor(FloorE::RaiseFloorTurbo)),
        133 | 135 | 137 => once(LockedDoor(VldoorE::BlazeOpen)),
        138 => repeatable(LightOn(255)),
        139 => repeatable(LightOn(35)),
        140 => once(Floor(FloorE::RaiseFloor512)),
        _ => None,
    }
}
