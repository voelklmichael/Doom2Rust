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
#![no_std]

/// TCP port the CoreS3 listens on.
pub const DEFAULT_PORT: u16 = 7878;

const RELEASE_BIT: u8 = 0x80;

/// A game action. The codes are part of the wire format: append new commands, never renumber.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Command {
    Forward = 0,
    Backward = 1,
    TurnLeft = 2,
    TurnRight = 3,
    StrafeLeft = 4,
    StrafeRight = 5,
    Fire = 6,
    Use = 7,
    Run = 8,
    Enter = 9,
    Escape = 10,
    Map = 11,
    Yes = 12,
    No = 13,
    Weapon1 = 14,
    Weapon2 = 15,
    Weapon3 = 16,
    Weapon4 = 17,
    Weapon5 = 18,
    Weapon6 = 19,
    Weapon7 = 20,
}

impl Command {
    pub const ALL: [Self; 21] = [
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
    ];

    /// The command with wire code `code`, if there is one.
    pub fn from_code(code: u8) -> Option<Self> {
        Self::ALL.get(usize::from(code)).copied()
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
        self.command as u8 | if self.pressed { 0 } else { RELEASE_BIT }
    }

    /// `None` for a byte whose command code is unknown.
    pub fn decode(byte: u8) -> Option<Self> {
        Some(Self {
            command: Command::from_code(byte & !RELEASE_BIT)?,
            pressed: byte & RELEASE_BIT == 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert_eq!(command as usize, code);
        }
    }

    #[test]
    fn release_sets_only_the_top_bit() {
        assert_eq!(KeyEvent::press(Command::Fire).encode(), 6);
        assert_eq!(KeyEvent::release(Command::Fire).encode(), 0x80 | 6);
    }

    #[test]
    fn unknown_codes_are_rejected_in_both_states() {
        let first_unknown = Command::ALL.len() as u8;
        assert_eq!(KeyEvent::decode(first_unknown), None);
        assert_eq!(KeyEvent::decode(RELEASE_BIT | first_unknown), None);
        assert_eq!(KeyEvent::decode(0x7f), None);
        assert_eq!(KeyEvent::decode(0xff), None);
    }
}
