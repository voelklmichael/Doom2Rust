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

static shiftxform: [u8; 128] = [
    0_u8, 1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8, 9_u8, 10_u8, 11_u8, 12_u8, 13_u8, 14_u8,
    15_u8, 16_u8, 17_u8, 18_u8, 19_u8, 20_u8, 21_u8, 22_u8, 23_u8, 24_u8, 25_u8, 26_u8, 27_u8,
    28_u8, 29_u8, 30_u8, 31_u8, b' ', b'!', b'"', b'#', b'$', b'%', b'&', b'"', b'(', b')', b'*',
    b'+', b'<', b'_', b'>', b'?', b')', b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b':',
    b':', b'<', b'+', b'>', b'?', b'@', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J',
    b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z',
    b'[', b'!', b']', b'"', b'_', b'\'', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I',
    b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y',
    b'Z', b'{', b'|', b'}', b'~', 127_u8,
];
fn TranslateKey(mut key: u8) -> u8 {
    key
}
fn GetTypedChar(state: &mut IInputState, mut key: u8) -> u8 {
    key = TranslateKey(key);
    if state.shiftdown > 0_i32 {
        if key as i32 >= 0_i32 && (key as usize) < shiftxform.len() {
            key = shiftxform[key as usize];
        } else {
            key = 0_u8;
        }
    }
    key
}
fn UpdateShiftStatus(state: &mut IInputState, mut pressed: i32, mut key: u8) {
    let mut change: i32 = 0;
    if pressed != 0 {
        change = 1_i32;
    } else {
        change = -1_i32;
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
            if event.data1 != 0_i32 {
                D_PostEvent(&mut state.d_event, event);
            }
        } else {
            event.type_0 = EvType::ev_keyup;
            event.data1 = TranslateKey(key) as i32;
            event.data2 = 0_i32;
            if event.data1 != 0_i32 {
                D_PostEvent(&mut state.d_event, event);
            }
            break;
        }
    }
}
