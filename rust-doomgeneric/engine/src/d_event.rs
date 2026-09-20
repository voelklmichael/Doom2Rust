#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameAction {
    Nothing,
    LoadLevel,
    NewGame,
    LoadGame,
    SaveGame,
    PlayDemo,
    Completed,
    Victory,
    WorldDone,
    Screenshot,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameScreenState {
    Level,
    Intermission,
    Finale,
    Demoscreen,
    Wipped,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EvType {
    Keydown,
    Keyup,
    Mouse,
    Joystick,
    Quit,
}
#[derive(Copy, Clone)]
pub struct Event {
    pub kind: EvType,
    pub data1: i32,
    pub data2: i32,
    pub data3: i32,
    pub data4: i32,
}
const MAXEVENTS: usize = 64;

pub struct DEventState {
    events: [Event; MAXEVENTS],
    eventhead: usize,
    eventtail: usize,
}

impl Default for DEventState {
    fn default() -> Self {
        Self {
            events: [Event {
                kind: EvType::Keydown,
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

pub fn post_event(state: &mut DEventState, ev: Event) {
    state.events[state.eventhead] = ev;
    state.eventhead = (state.eventhead + 1) % MAXEVENTS;
}
pub fn pop_event(state: &mut DEventState) -> Option<Event> {
    if state.eventtail == state.eventhead {
        return None;
    }
    let event = state.events[state.eventtail];

    state.eventtail = (state.eventtail + 1) % MAXEVENTS;
    Some(event)
}
