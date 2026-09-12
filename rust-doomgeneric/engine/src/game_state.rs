// The aggregate state struct that replaces the codebase's original `static
// mut` globals. Built once by `init_game_state`, which leaks it to a
// `&'static mut GameState` and threads it explicitly through every function
// from then on. See /docs/track16-gamestate-plan.md for the history of how
// this replaced the globals module by module.

use crate::src::am_map::AmMapState;
use crate::src::d_event::DEventState;
use crate::src::d_iwad::DIwadState;
use crate::src::d_loop::DLoopState;
use crate::src::d_main::DMainState;
use crate::src::d_net::DNetState;
use crate::src::doomstat::DoomstatState;
use crate::src::f_finale::FFinaleState;
use crate::src::f_wipe::FWipeState;
use crate::src::g_game::GGameState;
use crate::src::hu_stuff::HuStuffState;
use crate::src::i_cdmus::ICdMusState;
use crate::src::i_input::IInputState;
use crate::src::i_joystick::IJoystickState;
use crate::src::i_sound::ISoundState;
use crate::src::i_system::ISystemState;
use crate::src::i_timer::ITimerState;
use crate::src::i_video::IVideoState;
use crate::src::info::InfoState;
use crate::src::m_argv::MArgvState;
use crate::src::m_config::MConfigState;
use crate::src::m_controls::MControlsState;
use crate::src::m_menu::MMenuState;
use crate::src::m_random::MRandomState;
use crate::src::p_ceilng::PCeilngState;
use crate::src::p_enemy::PEnemyState;
use crate::src::p_map::PMapState;
use crate::src::p_maputl::fixup_intercepts_overrun;
use crate::src::p_maputl::PMaputlState;
use crate::src::p_mobj::PMobjState;
use crate::src::p_plats::PPlatsState;
use crate::src::p_pspr::PPsprState;
use crate::src::p_saveg::PSavegState;
use crate::src::p_setup::PSetupState;
use crate::src::p_sight::PSightState;
use crate::src::p_spec::PSpecState;
use crate::src::p_switch::PSwitchState;
use crate::src::p_tick::PTickState;
use crate::src::p_user::PUserState;
use crate::src::platform::DoomPlatform;
use crate::src::r_bsp::RBspState;
use crate::src::r_data::RDataState;
use crate::src::r_draw::RDrawState;
use crate::src::r_main::RMainState;
use crate::src::r_plane::RPlaneState;
use crate::src::r_segs::RSegsState;
use crate::src::r_sky::RSkyState;
use crate::src::r_things::RThingsState;
use crate::src::s_sound::SSoundState;
use crate::src::sounds::SoundsState;
use crate::src::st_lib::StLibState;
use crate::src::st_stuff::{fixup_cheat_sequences, StStuffState};
use crate::src::statdump::StatDumpState;
use crate::src::v_video::VVideoState;
use crate::src::w_checksum::WChecksumState;
use crate::src::w_file::WFileState;
use crate::src::w_wad::WWadState;
use crate::src::wi_stuff::{fixup_numanims, WiStuffState};
use crate::src::z_zone::ZZoneState;

pub struct GameState {
    pub am_map: AmMapState,
    pub d_event: DEventState,
    pub d_iwad: DIwadState,
    pub d_loop: DLoopState,
    pub d_main: DMainState,
    pub d_net: DNetState,
    pub doomstat: DoomstatState,
    pub f_finale: FFinaleState,
    pub f_wipe: FWipeState,
    pub g_game: GGameState,
    pub hu_stuff: HuStuffState,
    pub i_cdmus: ICdMusState,
    pub i_input: IInputState,
    pub i_joystick: IJoystickState,
    pub i_sound: ISoundState,
    pub i_video: IVideoState,
    pub i_system: ISystemState,
    pub i_timer: ITimerState,
    pub info: InfoState,
    pub m_argv: MArgvState,
    pub m_config: MConfigState,
    pub m_controls: MControlsState,
    pub m_menu: MMenuState,
    pub m_random: MRandomState,
    pub p_ceilng: PCeilngState,
    pub p_enemy: PEnemyState,
    pub p_map: PMapState,
    pub p_maputl: PMaputlState,
    pub r_sky: RSkyState,
    pub p_mobj: PMobjState,
    pub p_plats: PPlatsState,
    pub p_pspr: PPsprState,
    pub p_saveg: PSavegState,
    pub p_setup: PSetupState,
    pub p_sight: PSightState,
    pub p_spec: PSpecState,
    pub p_switch: PSwitchState,
    pub p_tick: PTickState,
    pub p_user: PUserState,
    pub r_main: RMainState,
    pub r_segs: RSegsState,
    pub r_draw: RDrawState,
    pub r_data: RDataState,
    pub r_plane: RPlaneState,
    pub r_bsp: RBspState,
    pub r_things: RThingsState,
    pub s_sound: SSoundState,
    pub sounds: SoundsState,
    pub st_lib: StLibState,
    pub st_stuff: StStuffState,
    pub statdump: StatDumpState,
    pub v_video: VVideoState,
    pub w_checksum: WChecksumState,
    pub w_file: WFileState,
    pub w_wad: WWadState,
    pub wi_stuff: WiStuffState,
    pub z_zone: ZZoneState,
    pub platform: Box<dyn DoomPlatform>,
}

impl GameState {
    fn new(platform: Box<dyn DoomPlatform>) -> Self {
        GameState {
            am_map: AmMapState::new(),
            d_event: DEventState::new(),
            d_iwad: DIwadState::new(),
            d_loop: DLoopState::new(),
            d_main: DMainState::new(),
            d_net: DNetState::new(),
            doomstat: DoomstatState::new(),
            f_finale: FFinaleState::new(),
            f_wipe: FWipeState::new(),
            g_game: GGameState::new(),
            hu_stuff: HuStuffState::new(),
            i_cdmus: ICdMusState::new(),
            i_input: IInputState::new(),
            i_joystick: IJoystickState::new(),
            i_sound: ISoundState::new(),
            i_video: IVideoState::new(),
            i_system: ISystemState::new(),
            i_timer: ITimerState::new(),
            info: InfoState::new(),
            m_argv: MArgvState::new(),
            m_config: MConfigState::new(),
            m_controls: MControlsState::new(),
            m_menu: MMenuState::new(),
            m_random: MRandomState::new(),
            p_ceilng: PCeilngState::new(),
            p_enemy: PEnemyState::new(),
            p_map: PMapState::new(),
            p_maputl: PMaputlState::new(),
            r_sky: RSkyState::new(),
            p_mobj: PMobjState::new(),
            p_plats: PPlatsState::new(),
            p_pspr: PPsprState::new(),
            p_saveg: PSavegState::new(),
            p_setup: PSetupState::new(),
            p_sight: PSightState::new(),
            p_spec: PSpecState::new(),
            p_switch: PSwitchState::new(),
            p_tick: PTickState::new(),
            p_user: PUserState::new(),
            r_main: RMainState::new(),
            r_segs: RSegsState::new(),
            r_draw: RDrawState::new(),
            r_data: RDataState::new(),
            r_plane: RPlaneState::new(),
            r_bsp: RBspState::new(),
            r_things: RThingsState::new(),
            s_sound: SSoundState::new(),
            sounds: SoundsState::new(),
            st_lib: StLibState::new(),
            st_stuff: StStuffState::new(),
            statdump: StatDumpState::new(),
            v_video: VVideoState::new(),
            w_checksum: WChecksumState::new(),
            w_file: WFileState::new(),
            w_wad: WWadState::new(),
            wi_stuff: WiStuffState::new(),
            z_zone: ZZoneState::new(),
            platform,
        }
    }
}

// Self-referential pointers (e.g. p_maputl's intercepts_overrun addresses)
// can only be computed once the value is at its final, permanently-stable
// address -- i.e. here, not inside any XxxState::new(). Must run exactly
// once, right after the GameState this reference points at is constructed
// and will never move again.
pub fn finish_init(state: &mut GameState) {
    unsafe {
        state.sounds.fixup_self_links();
        fixup_intercepts_overrun(state);
        state.m_config.fixup_defaults();
        state.m_controls.fixup_weapon_keys();
        state.m_menu.fixup_menu_links();
        state.m_menu.fixup_menu_routines();
        state.wi_stuff.fixup_anims();
        state.g_game.fixup_button_pointers();
        fixup_cheat_sequences(state);
        fixup_numanims(state);
    }
}

/// Constructs the single `GameState`, wired to the given platform backend,
/// and leaks it to obtain a `&'static mut` -- the state is meant to live for
/// the remainder of the process, so this is not actually a leak in practice.
pub fn init_game_state(platform: Box<dyn DoomPlatform>) -> &'static mut GameState {
    let state = Box::leak(Box::new(GameState::new(platform)));
    finish_init(state);
    state
}
