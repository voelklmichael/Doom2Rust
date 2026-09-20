use alloc::boxed::Box;
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
use crate::m_config::MConfigState;
use crate::m_controls::MControlsState;
use crate::m_menu::MMenuState;
use crate::m_random::MRandomState;
use crate::options::Options;
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

/// The simulation: level geometry, map objects, thinkers, sector specials, the random-number table.
pub struct World {
    pub p_setup: PSetupState,
    pub p_mobj: PMobjState,
    pub p_tick: PTickState,
    pub p_spec: PSpecState,
    pub p_map: PMapState,
    pub p_maputl: PMaputlState,
    pub p_sight: PSightState,
    pub p_ceilng: PCeilngState,
    pub p_doors: PDoorsState,
    pub p_lights: PLightsState,
    pub p_plats: PPlatsState,
    pub p_switch: PSwitchState,
    pub p_enemy: PEnemyState,
    pub p_pspr: PPsprState,
    pub p_user: PUserState,
    pub p_saveg: PSavegState,
    pub m_random: MRandomState,
}

impl World {
    fn new() -> Self {
        Self {
            p_setup: PSetupState::new(),
            p_mobj: PMobjState::new(),
            p_tick: PTickState::new(),
            p_spec: PSpecState::new(),
            p_map: PMapState::new(),
            p_maputl: PMaputlState::new(),
            p_sight: PSightState::new(),
            p_ceilng: PCeilngState::new(),
            p_doors: PDoorsState::new(),
            p_lights: PLightsState::new(),
            p_plats: PPlatsState::new(),
            p_switch: PSwitchState::new(),
            p_enemy: PEnemyState::new(),
            p_pspr: PPsprState::new(),
            p_user: PUserState::new(),
            p_saveg: PSavegState::new(),
            m_random: MRandomState::new(),
        }
    }
}

/// The software renderer: view setup, BSP walk, walls, flats, sprites, column/span drawers.
pub struct Render {
    pub r_main: RMainState,
    pub r_segs: RSegsState,
    pub r_draw: RDrawState,
    pub r_data: RDataState,
    pub r_plane: RPlaneState,
    pub r_bsp: RBspState,
    pub r_things: RThingsState,
    pub r_sky: RSkyState,
}

impl Render {
    fn new() -> Self {
        Self {
            r_main: RMainState::new(),
            r_segs: RSegsState::new(),
            r_draw: RDrawState::new(),
            r_data: RDataState::new(),
            r_plane: RPlaneState::new(),
            r_bsp: RBspState::new(),
            r_things: RThingsState::new(),
            r_sky: RSkyState::new(),
        }
    }
}

/// Everything drawn on top of the world: menus, HUD, status bar, automap, intermission, finale, wipe.
pub struct Ui {
    pub m_menu: MMenuState,
    pub hu_stuff: HuStuffState,
    pub st_lib: StLibState,
    pub st_stuff: StStuffState,
    pub wi_stuff: WiStuffState,
    pub am_map: AmMapState,
    pub f_finale: FFinaleState,
    pub f_wipe: FWipeState,
    pub statdump: StatDumpState,
}

impl Ui {
    fn new() -> Self {
        Self {
            m_menu: MMenuState::new(),
            hu_stuff: HuStuffState::new(),
            st_lib: StLibState::new(),
            st_stuff: StStuffState::new(),
            wi_stuff: WiStuffState::new(),
            am_map: AmMapState::new(),
            f_finale: FFinaleState::new(),
            f_wipe: FWipeState::new(),
            statdump: StatDumpState::new(),
        }
    }
}

/// Sound effects and music: channels, the sfx/music tables and the sound driver.
pub struct Audio {
    pub s_sound: SSoundState,
    pub i_sound: ISoundState,
    pub sounds: SoundsState,
}

impl Audio {
    fn new() -> Self {
        Self {
            s_sound: SSoundState::new(),
            i_sound: ISoundState::new(),
            sounds: SoundsState::new(),
        }
    }
}

/// WAD lumps and the tables loaded from them, plus the filesystem they are read through.
pub struct Assets {
    pub w_wad: WWadState,
    pub w_checksum: WChecksumState,
    pub info: InfoState,
    pub fs: Box<dyn DoomFileSystem>,
}

impl Assets {
    fn new(fs: Box<dyn DoomFileSystem>) -> Self {
        Self {
            w_wad: WWadState::new(),
            w_checksum: WChecksumState::new(),
            info: InfoState::new(),
            fs,
        }
    }
}

/// Game session: players, skill and mode, the demo/tic loop, events and configuration.
pub struct Game {
    pub g_game: GGameState,
    pub doomstat: DoomstatState,
    pub d_main: DMainState,
    pub d_loop: DLoopState,
    pub d_event: DEventState,
    pub d_iwad: DIwadState,
    pub options: Options,
    pub m_config: MConfigState,
    pub m_controls: MControlsState,
}

impl Game {
    fn new() -> Self {
        Self {
            g_game: GGameState::new(),
            doomstat: DoomstatState::new(),
            d_main: DMainState::new(),
            d_loop: DLoopState::new(),
            d_event: DEventState::new(),
            d_iwad: DIwadState::new(),
            options: Options::default(),
            m_config: MConfigState::new(),
            m_controls: MControlsState::new(),
        }
    }
}

/// Talking to the host: the platform backend, input, timing and the video output.
pub struct Io {
    pub platform: Box<dyn DoomPlatform>,
    pub i_input: IInputState,
    pub i_joystick: IJoystickState,
    pub i_system: ISystemState,
    pub i_timer: ITimerState,
    pub i_video: IVideoState,
    pub v_video: VVideoState,
}

impl Io {
    fn new(platform: Box<dyn DoomPlatform>) -> Self {
        Self {
            platform,
            i_input: IInputState::new(),
            i_joystick: IJoystickState::new(),
            i_system: ISystemState::new(),
            i_timer: ITimerState::new(),
            i_video: IVideoState::new(),
            v_video: VVideoState::new(),
        }
    }
}

pub struct GameState {
    pub world: World,
    pub render: Render,
    pub ui: Ui,
    pub audio: Audio,
    pub assets: Assets,
    pub game: Game,
    pub io: Io,
}

impl GameState {
    fn new(platform: Box<dyn DoomPlatform>, fs: Box<dyn DoomFileSystem>) -> Self {
        Self {
            world: World::new(),
            render: Render::new(),
            ui: Ui::new(),
            audio: Audio::new(),
            assets: Assets::new(fs),
            game: Game::new(),
            io: Io::new(platform),
        }
    }

    pub fn wbs(&mut self) -> &mut crate::wi_stuff::WbStartStruct {
        &mut self.game.g_game.wminfo
    }

    pub fn plyr_index(&mut self, index: usize) -> &mut crate::wi_stuff::WbPlayerStruct {
        &mut self.game.g_game.wminfo.plyr[index]
    }
}

// Self-referential pointers (e.g. sound channel links) can only be computed
// once the value is at its final, permanently-stable address -- i.e. here,
// not inside any XxxState::new(). Must run exactly once, right after the
// GameState this reference points at is constructed and will never move
// again.
pub fn finish_init(state: &mut GameState) {
    state.audio.sounds.fixup_self_links();
    fixup_numanims(&mut state.ui.wi_stuff);
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
