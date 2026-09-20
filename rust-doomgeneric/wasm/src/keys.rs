//! Browser keyboard to engine key codes.

/// The engine's key code for a `KeyboardEvent.code` (the physical key, so the layout does not
/// matter). `None` for a key the game has no use for. The values are the ones the engine's
/// default bindings (`m_controls.rs`) and menus expect; letters and digits are their ASCII.
pub fn doom_key(code: &str) -> Option<u8> {
    const RIGHT_ARROW: u8 = 0xae;
    const LEFT_ARROW: u8 = 0xac;
    const UP_ARROW: u8 = 0xad;
    const DOWN_ARROW: u8 = 0xaf;
    const STRAFE_LEFT: u8 = 0xa0;
    const STRAFE_RIGHT: u8 = 0xa1;
    const USE: u8 = 0xa2;
    const FIRE: u8 = 0xa3;
    const RSHIFT: u8 = 0x80 + 0x36;
    const RALT: u8 = 0x80 + 0x38;
    Some(match code {
        "ArrowLeft" => LEFT_ARROW,
        "ArrowRight" => RIGHT_ARROW,
        "ArrowUp" => UP_ARROW,
        "ArrowDown" => DOWN_ARROW,
        "ControlLeft" | "ControlRight" => FIRE,
        "Space" => USE,
        "ShiftLeft" | "ShiftRight" => RSHIFT,
        "AltLeft" | "AltRight" => RALT,
        "Comma" => STRAFE_LEFT,
        "Period" => STRAFE_RIGHT,
        "Enter" | "NumpadEnter" => 13,
        "Escape" => 27,
        "Tab" => 9,
        "Backspace" => 0x7f,
        "Pause" => 0xff,
        "Equal" => b'=',
        "Minus" => b'-',
        "Home" => 0x80 + 0x47,
        "End" => 0x80 + 0x4f,
        "PageUp" => 0x80 + 0x49,
        "PageDown" => 0x80 + 0x51,
        "Insert" => 0x80 + 0x52,
        "Delete" => 0x80 + 0x53,
        "F11" => 0x80 + 0x57,
        "F12" => 0x80 + 0x58,
        _ => return other_key(code),
    })
}

/// The keys that follow a pattern: `KeyA`..`KeyZ`, `Digit0`..`Digit9`, `F1`..`F10`.
fn other_key(code: &str) -> Option<u8> {
    if let Some(letter) = code.strip_prefix("Key") {
        let [letter @ b'A'..=b'Z'] = letter.as_bytes() else {
            return None;
        };
        return Some(letter.to_ascii_lowercase());
    }
    if let Some(digit) = code.strip_prefix("Digit") {
        let [digit @ b'0'..=b'9'] = digit.as_bytes() else {
            return None;
        };
        return Some(*digit);
    }
    let number = code.strip_prefix('F')?.parse::<u8>().ok()?;
    (1..=10).contains(&number).then(|| 0x80 + 0x3b + number - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_keys() {
        assert_eq!(doom_key("ArrowUp"), Some(0xad));
        assert_eq!(doom_key("ControlLeft"), Some(0xa3));
        assert_eq!(doom_key("Space"), Some(0xa2));
        assert_eq!(doom_key("Escape"), Some(27));
        assert_eq!(doom_key("Enter"), Some(13));
    }

    #[test]
    fn letters_digits_and_function_keys_follow_their_pattern() {
        assert_eq!(doom_key("KeyY"), Some(b'y'));
        assert_eq!(doom_key("Digit3"), Some(b'3'));
        assert_eq!(doom_key("F1"), Some(0x80 + 0x3b));
        assert_eq!(doom_key("F10"), Some(0x80 + 0x44));
        assert_eq!(doom_key("F11"), Some(0x80 + 0x57));
        assert_eq!(doom_key("F12"), Some(0x80 + 0x58));
    }

    #[test]
    fn unknown_keys_are_ignored() {
        for code in [
            "", "KeyAA", "Key1", "Digit", "F0", "F13", "MetaLeft", "Fn", "Numpad1",
        ] {
            assert_eq!(doom_key(code), None, "{code}");
        }
    }
}
