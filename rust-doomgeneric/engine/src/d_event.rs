pub type gameaction_t = u32;
pub const ga_screenshot: gameaction_t = 9;
pub const ga_worlddone: gameaction_t = 8;
pub const ga_victory: gameaction_t = 7;
pub const ga_completed: gameaction_t = 6;
pub const ga_playdemo: gameaction_t = 5;
pub const ga_savegame: gameaction_t = 4;
pub const ga_loadgame: gameaction_t = 3;
pub const ga_newgame: gameaction_t = 2;
pub const ga_loadlevel: gameaction_t = 1;
pub const ga_nothing: gameaction_t = 0;
#[derive(Copy, Clone, PartialEq)]
pub enum GameScreenState {
    GS_LEVEL = 0,
    GS_INTERMISSION = 1,
    GS_FINALE = 2,
    GS_DEMOSCREEN = 3,
    GS_WIPPED = 4294967295,
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
    let event = state.events[state.eventtail].clone();

    state.eventtail = (state.eventtail + 1) % MAXEVENTS;
    return Some(event);
}
