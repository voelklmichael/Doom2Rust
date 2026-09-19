// The aggregate state struct that replaces the codebase's original `static
// mut` globals. Built once by `init_game_state`, which leaks it to a
// `&'static mut GameState` and threads it explicitly through every function
// from then on. See /docs/track16-gamestate-plan.md for the history of how
// this replaced the globals module by module.

use crate::am_map::AmMapState;
use crate::d_event::DEventState;
use crate::d_iwad::DIwadState;
use crate::d_loop::DLoopState;
use crate::d_main::DMainState;
use crate::doomstat::DoomstatState;
use crate::f_finale::FFinaleState;
use crate::f_wipe::FWipeState;
use crate::filesystem::DoomFileSystem;
use crate::g_game::GGameState;
use crate::hu_stuff::HuStuffState;
use crate::i_input::IInputState;
use crate::i_joystick::IJoystickState;
use crate::i_sound::ISoundState;
use crate::i_system::ISystemState;
use crate::i_timer::ITimerState;
use crate::i_video::IVideoState;
use crate::info::InfoState;
use crate::m_argv::MArgvState;
use crate::m_config::MConfigState;
use crate::m_controls::MControlsState;
use crate::m_menu::MMenuState;
use crate::m_random::MRandomState;
use crate::p_ceilng::PCeilngState;
use crate::p_doors::PDoorsState;
use crate::p_enemy::PEnemyState;
use crate::p_lights::PLightsState;
use crate::p_map::PMapState;
use crate::p_maputl::PMaputlState;
use crate::p_mobj::PMobjState;
use crate::p_plats::PPlatsState;
use crate::p_pspr::PPsprState;
use crate::p_saveg::PSavegState;
use crate::p_setup::PSetupState;
use crate::p_sight::PSightState;
use crate::p_spec::PSpecState;
use crate::p_switch::PSwitchState;
use crate::p_tick::PTickState;
use crate::p_user::PUserState;
use crate::platform::DoomPlatform;
use crate::r_bsp::RBspState;
use crate::r_data::RDataState;
use crate::r_draw::RDrawState;
use crate::r_main::RMainState;
use crate::r_plane::RPlaneState;
use crate::r_segs::RSegsState;
use crate::r_sky::RSkyState;
use crate::r_things::RThingsState;
use crate::s_sound::SSoundState;
use crate::sounds::SoundsState;
use crate::st_lib::StLibState;
use crate::st_stuff::StStuffState;
use crate::statdump::StatDumpState;
use crate::v_video::VVideoState;
use crate::w_checksum::WChecksumState;
use crate::w_wad::WWadState;
use crate::wi_stuff::{fixup_numanims, WiStuffState};

pub struct GameState {
    pub am_map: AmMapState,
    pub d_event: DEventState,
    pub d_iwad: DIwadState,
    pub d_loop: DLoopState,
    pub d_main: DMainState,
    pub doomstat: DoomstatState,
    pub f_finale: FFinaleState,
    pub f_wipe: FWipeState,
    pub g_game: GGameState,
    pub hu_stuff: HuStuffState,
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
    pub p_doors: PDoorsState,
    pub p_enemy: PEnemyState,
    pub p_lights: PLightsState,
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
    pub w_wad: WWadState,
    pub wi_stuff: WiStuffState,
    pub platform: Box<dyn DoomPlatform>,
    pub fs: Box<dyn DoomFileSystem>,
}

impl GameState {
    fn new(platform: Box<dyn DoomPlatform>, fs: Box<dyn DoomFileSystem>) -> Self {
        GameState {
            am_map: AmMapState::new(),
            d_event: DEventState::new(),
            d_iwad: DIwadState::new(),
            d_loop: DLoopState::new(),
            d_main: DMainState::new(),
            doomstat: DoomstatState::new(),
            f_finale: FFinaleState::new(),
            f_wipe: FWipeState::new(),
            g_game: GGameState::new(),
            hu_stuff: HuStuffState::new(),
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
            p_doors: PDoorsState::new(),
            p_enemy: PEnemyState::new(),
            p_lights: PLightsState::new(),
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
            w_wad: WWadState::new(),
            wi_stuff: WiStuffState::new(),
            platform,
            fs,
        }
    }

    pub fn wbs(&mut self) -> &mut crate::wi_stuff::wbstartstruct_t {
        &mut self.g_game.wminfo
    }

    pub fn plyr_index(&mut self, index: i32) -> &mut crate::wi_stuff::wbplayerstruct_t {
        &mut self.g_game.wminfo.plyr[index as usize]
    }
}

// Self-referential pointers (e.g. sound channel links) can only be computed
// once the value is at its final, permanently-stable address -- i.e. here,
// not inside any XxxState::new(). Must run exactly once, right after the
// GameState this reference points at is constructed and will never move
// again.
pub fn finish_init(state: &mut GameState) {
    {
        state.sounds.fixup_self_links();
        fixup_numanims(state);
    }
}

/// Constructs the single `GameState`, wired to the given platform and
/// filesystem backends, and leaks it to obtain a `&'static mut` -- the state is meant to live for
/// the remainder of the process, so this is not actually a leak in practice.
pub fn init_game_state(
    platform: Box<dyn DoomPlatform>,
    fs: Box<dyn DoomFileSystem>,
) -> &'static mut GameState {
    let state = Box::leak(Box::new(GameState::new(platform, fs)));
    finish_init(state);
    state
}
