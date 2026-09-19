//! Wire format between the controller (`core_s3_sender`) and the CoreS3 firmware (`core_s3`).
//!
//! The link is a plain TCP stream and **every byte is one command**:
//!
//! ```text
//! bit 7      : 0 = key pressed, 1 = key released
//! bits 6..=0 : command code (see `Command`)
//! ```
//!
//! Press and release are separate events so a held key (walking, firing) stays held for exactly
//! as long as the sender says. Bytes with an unknown command code are ignored by the receiver, so
//! new commands can be added without breaking older firmware.
//!
//! Codes 0 to 21 are the game actions of [`Command`]. Codes 32 to 126 are typed characters: the
//! code is the ASCII value, and the receiver presses that key (this is how cheat codes and other
//! text reach the game). The codes in between are unused.
#![no_std]

/// TCP port the CoreS3 listens on.
pub const DEFAULT_PORT: u16 = 7878;

const RELEASE_BIT: u8 = 0x80;

/// What a byte asks for: a game action, or a typed character. The codes are part of the wire
/// format: append new actions, never renumber.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Command {
    Forward,
    Backward,
    TurnLeft,
    TurnRight,
    StrafeLeft,
    StrafeRight,
    Fire,
    Use,
    Run,
    Enter,
    Escape,
    Map,
    Yes,
    No,
    Weapon1,
    Weapon2,
    Weapon3,
    Weapon4,
    Weapon5,
    Weapon6,
    Weapon7,
    Backspace,
    /// A typed printable ASCII character. Build it with [`Command::typed`], which checks the range.
    Char(u8),
}

/// The typed characters that have a code: printable ASCII, whose code is the character itself.
const FIRST_CHAR: u8 = 32;
const LAST_CHAR: u8 = 126;

impl Command {
    /// The game actions, in code order. Typed characters are not in this list.
    pub const ALL: [Self; 22] = [
        Self::Forward,
        Self::Backward,
        Self::TurnLeft,
        Self::TurnRight,
        Self::StrafeLeft,
        Self::StrafeRight,
        Self::Fire,
        Self::Use,
        Self::Run,
        Self::Enter,
        Self::Escape,
        Self::Map,
        Self::Yes,
        Self::No,
        Self::Weapon1,
        Self::Weapon2,
        Self::Weapon3,
        Self::Weapon4,
        Self::Weapon5,
        Self::Weapon6,
        Self::Weapon7,
        Self::Backspace,
    ];

    /// The typed character for the ASCII byte `byte`, if it is printable.
    pub fn typed(byte: u8) -> Option<Self> {
        (FIRST_CHAR..=LAST_CHAR).contains(&byte).then_some(Self::Char(byte))
    }

    /// The command with wire code `code`, if there is one.
    pub fn from_code(code: u8) -> Option<Self> {
        Self::ALL.get(usize::from(code)).copied().or_else(|| Self::typed(code))
    }

    /// The wire code, in bits 6..=0.
    pub fn code(self) -> u8 {
        match self {
            Self::Char(byte) => byte,
            Self::Forward => 0,
            Self::Backward => 1,
            Self::TurnLeft => 2,
            Self::TurnRight => 3,
            Self::StrafeLeft => 4,
            Self::StrafeRight => 5,
            Self::Fire => 6,
            Self::Use => 7,
            Self::Run => 8,
            Self::Enter => 9,
            Self::Escape => 10,
            Self::Map => 11,
            Self::Yes => 12,
            Self::No => 13,
            Self::Weapon1 => 14,
            Self::Weapon2 => 15,
            Self::Weapon3 => 16,
            Self::Weapon4 => 17,
            Self::Weapon5 => 18,
            Self::Weapon6 => 19,
            Self::Weapon7 => 20,
            Self::Backspace => 21,
        }
    }
}

/// One byte on the wire: a command being pressed or released.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub command: Command,
    pub pressed: bool,
}

impl KeyEvent {
    pub fn press(command: Command) -> Self {
        Self { command, pressed: true }
    }

    pub fn release(command: Command) -> Self {
        Self { command, pressed: false }
    }

    pub fn encode(self) -> u8 {
        self.command.code() | if self.pressed { 0 } else { RELEASE_BIT }
    }

    /// `None` for a byte whose command code is unknown.
    pub fn decode(byte: u8) -> Option<Self> {
        Some(Self {
            command: Command::from_code(byte & !RELEASE_BIT)?,
            pressed: byte & RELEASE_BIT == 0,
        })
    }
}

/// Which actions are currently held, so a receiver can let go of everything when the connection
/// drops (otherwise the player would keep walking forever). Typed characters are not tracked: they
/// are sent as a press and a release together.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeldKeys {
    bits: u32,
}

const _: () = assert!(Command::ALL.len() <= u32::BITS as usize);

impl HeldKeys {
    pub fn update(&mut self, event: KeyEvent) {
        if matches!(event.command, Command::Char(_)) {
            return;
        }
        let bit = 1 << u32::from(event.command.code());
        if event.pressed {
            self.bits |= bit;
        } else {
            self.bits &= !bit;
        }
    }

    /// Release events for everything currently held; afterwards nothing is held.
    pub fn take_releases(&mut self) -> impl Iterator<Item = KeyEvent> {
        let bits = core::mem::take(&mut self.bits);
        Command::ALL
            .into_iter()
            .filter(move |&command| bits & (1 << u32::from(command.code())) != 0)
            .map(KeyEvent::release)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    use std::vec::Vec;

    #[test]
    fn every_command_round_trips_as_press_and_release() {
        for command in Command::ALL {
            for pressed in [true, false] {
                let event = KeyEvent { command, pressed };
                assert_eq!(KeyEvent::decode(event.encode()), Some(event));
            }
        }
    }

    #[test]
    fn all_lists_commands_in_code_order() {
        for (code, command) in Command::ALL.into_iter().enumerate() {
            assert_eq!(usize::from(command.code()), code);
            assert_eq!(Command::from_code(code as u8), Some(command));
        }
    }

    #[test]
    fn every_printable_character_round_trips_as_press_and_release() {
        for byte in b' '..=b'~' {
            let command = Command::typed(byte).unwrap();
            assert_eq!(command.code(), byte, "the code of a character is its ASCII value");
            for pressed in [true, false] {
                let event = KeyEvent { command, pressed };
                assert_eq!(KeyEvent::decode(event.encode()), Some(event));
            }
        }
        assert_eq!(KeyEvent::press(Command::Char(b'i')).encode(), b'i');
        assert_eq!(KeyEvent::release(Command::Char(b'i')).encode(), 0x80 | b'i');
    }

    #[test]
    fn characters_outside_printable_ascii_have_no_code() {
        assert_eq!(Command::typed(0), None);
        assert_eq!(Command::typed(31), None);
        assert_eq!(Command::typed(127), None);
        assert_eq!(Command::typed(200), None);
    }

    #[test]
    fn typed_characters_are_not_tracked_as_held() {
        let mut held = HeldKeys::default();
        held.update(KeyEvent::press(Command::Char(b'w')));
        held.update(KeyEvent::press(Command::Forward));
        let releases: Vec<_> = held.take_releases().collect();
        assert_eq!(releases, [KeyEvent::release(Command::Forward)]);
    }

    #[test]
    fn release_sets_only_the_top_bit() {
        assert_eq!(KeyEvent::press(Command::Fire).encode(), 6);
        assert_eq!(KeyEvent::release(Command::Fire).encode(), 0x80 | 6);
    }

    #[test]
    fn unknown_codes_are_rejected_in_both_states() {
        // Between the actions and the printable characters, and after them.
        for code in (Command::ALL.len() as u8)..FIRST_CHAR {
            assert_eq!(KeyEvent::decode(code), None);
            assert_eq!(KeyEvent::decode(RELEASE_BIT | code), None);
        }
        assert_eq!(KeyEvent::decode(0x7f), None);
        assert_eq!(KeyEvent::decode(0xff), None);
    }

    #[test]
    fn held_keys_are_released_once_after_a_disconnect() {
        let mut held = HeldKeys::default();
        held.update(KeyEvent::press(Command::Forward));
        held.update(KeyEvent::press(Command::Fire));
        held.update(KeyEvent::press(Command::Use));
        held.update(KeyEvent::release(Command::Use));
        let releases: Vec<_> = held.take_releases().collect();
        assert_eq!(
            releases,
            [KeyEvent::release(Command::Forward), KeyEvent::release(Command::Fire)]
        );
        assert_eq!(held.take_releases().count(), 0);
    }

    #[test]
    fn pressing_twice_still_releases_once() {
        let mut held = HeldKeys::default();
        held.update(KeyEvent::press(Command::Run));
        held.update(KeyEvent::press(Command::Run));
        assert_eq!(held.take_releases().count(), 1);
    }
}
