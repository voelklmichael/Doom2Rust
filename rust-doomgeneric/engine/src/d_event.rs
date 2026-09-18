#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameAction {
    ga_nothing = 0,
    ga_loadlevel = 1,
    ga_newgame = 2,
    ga_loadgame = 3,
    ga_savegame = 4,
    ga_playdemo = 5,
    ga_completed = 6,
    ga_victory = 7,
    ga_worlddone = 8,
    ga_screenshot = 9,
}
#[derive(Copy, Clone, PartialEq)]
pub enum GameScreenState {
    GS_LEVEL,
    GS_INTERMISSION,
    GS_FINALE,
    GS_DEMOSCREEN,
    GS_WIPPED,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EvType {
    ev_keydown = 0,
    ev_keyup = 1,
    ev_mouse = 2,
    ev_joystick = 3,
    ev_quit = 4,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_t {
    pub type_0: EvType,
    pub data1: i32,
    pub data2: i32,
    pub data3: i32,
    pub data4: i32,
}
const MAXEVENTS: usize = 64;

pub struct DEventState {
    events: [event_t; MAXEVENTS],
    eventhead: usize,
    eventtail: usize,
}

impl Default for DEventState {
    fn default() -> Self {
        Self::new()
    }
}

impl DEventState {
    pub const fn new() -> Self {
        DEventState {
            events: [event_t {
                type_0: EvType::ev_keydown,
                data1: 0,
                data2: 0,
                data3: 0,
                data4: 0,
            }; 64],
            eventhead: 0,
            eventtail: 0,
        }
    }
}

pub fn D_PostEvent(state: &mut DEventState, ev: event_t) {
    state.events[state.eventhead] = ev;
    state.eventhead = (state.eventhead + 1) % MAXEVENTS;
}
pub fn D_PopEvent(state: &mut DEventState) -> Option<event_t> {
    if state.eventtail == state.eventhead {
        return None;
    }
    let event = state.events[state.eventtail];

    state.eventtail = (state.eventtail + 1) % MAXEVENTS;
    Some(event)
}
