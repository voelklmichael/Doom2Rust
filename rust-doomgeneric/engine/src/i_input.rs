use crate::src::d_event::event_t;
use crate::src::d_event::D_PostEvent;
use crate::src::d_event::EvType;
use crate::src::game_state::GameState;
use crate::src::m_controls::KEY_RSHIFT;

pub struct IInputState {
    pub vanilla_keyboard_mapping: i32,
    shiftdown: i32,
}

impl IInputState {
    pub const fn new() -> Self {
        IInputState {
            vanilla_keyboard_mapping: 1,
            shiftdown: 0,
        }
    }
}

static shiftxform: [u8; 128] = [
    0 as u8,
    1 as u8,
    2 as u8,
    3 as u8,
    4 as u8,
    5 as u8,
    6 as u8,
    7 as u8,
    8 as u8,
    9 as u8,
    10 as u8,
    11 as u8,
    12 as u8,
    13 as u8,
    14 as u8,
    15 as u8,
    16 as u8,
    17 as u8,
    18 as u8,
    19 as u8,
    20 as u8,
    21 as u8,
    22 as u8,
    23 as u8,
    24 as u8,
    25 as u8,
    26 as u8,
    27 as u8,
    28 as u8,
    29 as u8,
    30 as u8,
    31 as u8,
    ' ' as u8,
    '!' as u8,
    '"' as u8,
    '#' as u8,
    '$' as u8,
    '%' as u8,
    '&' as u8,
    '"' as u8,
    '(' as u8,
    ')' as u8,
    '*' as u8,
    '+' as u8,
    '<' as u8,
    '_' as u8,
    '>' as u8,
    '?' as u8,
    ')' as u8,
    '!' as u8,
    '@' as u8,
    '#' as u8,
    '$' as u8,
    '%' as u8,
    '^' as u8,
    '&' as u8,
    '*' as u8,
    '(' as u8,
    ':' as u8,
    ':' as u8,
    '<' as u8,
    '+' as u8,
    '>' as u8,
    '?' as u8,
    '@' as u8,
    'A' as u8,
    'B' as u8,
    'C' as u8,
    'D' as u8,
    'E' as u8,
    'F' as u8,
    'G' as u8,
    'H' as u8,
    'I' as u8,
    'J' as u8,
    'K' as u8,
    'L' as u8,
    'M' as u8,
    'N' as u8,
    'O' as u8,
    'P' as u8,
    'Q' as u8,
    'R' as u8,
    'S' as u8,
    'T' as u8,
    'U' as u8,
    'V' as u8,
    'W' as u8,
    'X' as u8,
    'Y' as u8,
    'Z' as u8,
    '[' as u8,
    '!' as u8,
    ']' as u8,
    '"' as u8,
    '_' as u8,
    '\'' as u8,
    'A' as u8,
    'B' as u8,
    'C' as u8,
    'D' as u8,
    'E' as u8,
    'F' as u8,
    'G' as u8,
    'H' as u8,
    'I' as u8,
    'J' as u8,
    'K' as u8,
    'L' as u8,
    'M' as u8,
    'N' as u8,
    'O' as u8,
    'P' as u8,
    'Q' as u8,
    'R' as u8,
    'S' as u8,
    'T' as u8,
    'U' as u8,
    'V' as u8,
    'W' as u8,
    'X' as u8,
    'Y' as u8,
    'Z' as u8,
    '{' as u8,
    '|' as u8,
    '}' as u8,
    '~' as u8,
    127 as u8,
];
fn TranslateKey(mut key: u8) -> u8 {
    return key;
}
fn GetTypedChar(state: &mut IInputState, mut key: u8) -> u8 {
    key = TranslateKey(key);
    if state.shiftdown > 0 as i32 {
        if key as i32 >= 0 as i32 && (key as usize) < shiftxform.len() {
            key = shiftxform[key as usize];
        } else {
            key = 0 as u8;
        }
    }
    return key;
}
fn UpdateShiftStatus(state: &mut IInputState, mut pressed: i32, mut key: u8) {
    let mut change: i32 = 0;
    if pressed != 0 {
        change = 1 as i32;
    } else {
        change = -(1 as i32);
    }
    if key as i32 == KEY_RSHIFT {
        state.shiftdown += change;
    }
}
pub fn I_GetEvent(state: &mut GameState) {
    let mut event: event_t = event_t {
        type_0: EvType::ev_keydown,
        data1: 0,
        data2: 0,
        data3: 0,
        data4: 0,
    };
    while let Some((pressed, key)) = state.platform.get_key() {
        let pressed = pressed as i32;
        UpdateShiftStatus(&mut state.i_input, pressed, key);
        if pressed != 0 {
            event.type_0 = EvType::ev_keydown;
            event.data1 = TranslateKey(key) as i32;
            event.data2 = GetTypedChar(&mut state.i_input, key) as i32;
            if event.data1 != 0 as i32 {
                D_PostEvent(&mut state.d_event, event);
            }
        } else {
            event.type_0 = EvType::ev_keyup;
            event.data1 = TranslateKey(key) as i32;
            event.data2 = 0 as i32;
            if event.data1 != 0 as i32 {
                D_PostEvent(&mut state.d_event, event);
            }
            break;
        }
    }
}
