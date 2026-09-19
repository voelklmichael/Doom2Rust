use crate::d_event::event_t;
use crate::d_event::D_PostEvent;
use crate::d_event::EvType;
use crate::game_state::GameState;
use crate::m_controls::KEY_RSHIFT;

pub struct IInputState {
    pub vanilla_keyboard_mapping: i32,
    shiftdown: i32,
}

impl Default for IInputState {
    fn default() -> Self {
        Self::new()
    }
}

impl IInputState {
    pub const fn new() -> Self {
        IInputState {
            vanilla_keyboard_mapping: 1,
            shiftdown: 0,
        }
    }
}

static SHIFTXFORM: [u8; 128] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, b' ', b'!', b'"', b'#', b'$', b'%', b'&', b'"', b'(', b')', b'*', b'+',
    b'<', b'_', b'>', b'?', b')', b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b':', b':',
    b'<', b'+', b'>', b'?', b'@', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K',
    b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'[',
    b'!', b']', b'"', b'_', b'\'', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J',
    b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z',
    b'{', b'|', b'}', b'~', 127,
];
fn TranslateKey(key: u8) -> u8 {
    key
}
fn GetTypedChar(state: &mut IInputState, mut key: u8) -> u8 {
    key = TranslateKey(key);
    if state.shiftdown > 0 {
        if key as i32 >= 0 && (key as usize) < SHIFTXFORM.len() {
            key = SHIFTXFORM[key as usize];
        } else {
            key = 0;
        }
    }
    key
}
fn UpdateShiftStatus(state: &mut IInputState, pressed: i32, key: u8) {
    let change: i32 = if pressed != 0 { 1 } else { -1 };
    if key as i32 == KEY_RSHIFT {
        state.shiftdown += change;
    }
}
pub fn I_GetEvent(state: &mut GameState) {
    let mut event: event_t = event_t {
        kind: EvType::ev_keydown,
        data1: 0,
        data2: 0,
        data3: 0,
        data4: 0,
    };
    while let Some((pressed, key)) = state.platform.get_key() {
        let pressed = pressed as i32;
        UpdateShiftStatus(&mut state.i_input, pressed, key);
        if pressed != 0 {
            event.kind = EvType::ev_keydown;
            event.data1 = TranslateKey(key) as i32;
            event.data2 = GetTypedChar(&mut state.i_input, key) as i32;
            if event.data1 != 0 {
                D_PostEvent(&mut state.d_event, event);
            }
        } else {
            event.kind = EvType::ev_keyup;
            event.data1 = TranslateKey(key) as i32;
            event.data2 = 0;
            if event.data1 != 0 {
                D_PostEvent(&mut state.d_event, event);
            }
            break;
        }
    }
}
