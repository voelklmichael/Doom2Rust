use crate::d_event::post_event;
use crate::d_event::DEventState;
use crate::d_event::EvType;
use crate::d_event::Event;
use crate::m_controls::KEY_RSHIFT;
use crate::platform::DoomPlatform;

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
        Self {
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
fn translate_key(key: u8) -> u8 {
    key
}
fn get_typed_char(state: &IInputState, mut key: u8) -> u8 {
    key = translate_key(key);
    if state.shiftdown > 0 {
        if i32::from(key) >= 0 && (key as usize) < SHIFTXFORM.len() {
            key = SHIFTXFORM[key as usize];
        } else {
            key = 0;
        }
    }
    key
}
fn update_shift_status(state: &mut IInputState, pressed: i32, key: u8) {
    let change: i32 = if pressed != 0 { 1 } else { -1 };
    if i32::from(key) == KEY_RSHIFT {
        state.shiftdown += change;
    }
}
pub fn get_event(
    d_event: &mut DEventState,
    i_input: &mut IInputState,
    platform: &mut dyn DoomPlatform,
) {
    let mut event: Event = Event {
        kind: EvType::Keydown,
        data1: 0,
        data2: 0,
        data3: 0,
        data4: 0,
    };
    while let Some((pressed, key)) = platform.get_key() {
        let pressed = i32::from(pressed);
        update_shift_status(i_input, pressed, key);
        if pressed != 0 {
            event.kind = EvType::Keydown;
            event.data1 = i32::from(translate_key(key));
            event.data2 = i32::from(get_typed_char(i_input, key));
            if event.data1 != 0 {
                post_event(d_event, event);
            }
        } else {
            event.kind = EvType::Keyup;
            event.data1 = i32::from(translate_key(key));
            event.data2 = 0;
            if event.data1 != 0 {
                post_event(d_event, event);
            }
            break;
        }
    }
}
