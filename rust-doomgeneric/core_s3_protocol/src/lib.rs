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
//! Codes 0 to 22 are the actions of [`Command`]: the game's, and one for the firmware itself
//! ([`Command::ToggleSound`]). Codes 32 to 126 are typed characters: the code is the ASCII value,
//! and the receiver presses that key (this is how cheat codes and other text reach the game). The
//! codes in between are unused.
//!
//! The other direction is used for one thing only, and only by the web controller (WebSocket text
//! messages; the plain TCP link carries nothing back): the board reports its frame rate as
//! `fps 28.4`, see [`encode_fps`] and [`decode_fps`]. A controller that does not know the message
//! ignores it.
#![no_std]

/// TCP port the CoreS3 listens on.
pub const DEFAULT_PORT: u16 = 7878;

const RELEASE_BIT: u8 = 0x80;

const FPS_PREFIX: &[u8] = b"fps ";

/// The longest [`encode_fps`] message: `fps ` and a 10-digit whole part, a point and a digit.
pub const FPS_MESSAGE_MAX: usize = FPS_PREFIX.len() + 10 + 2;

/// The message that reports a frame rate of `tenths` / 10 frames per second: `fps 28.4`. Returns
/// the number of bytes written to `out`.
pub fn encode_fps(tenths: u32, out: &mut [u8; FPS_MESSAGE_MAX]) -> usize {
    out[..FPS_PREFIX.len()].copy_from_slice(FPS_PREFIX);
    let mut length = FPS_PREFIX.len();
    let mut digits = [0u8; 10];
    let mut count = 0;
    let mut whole = tenths / 10;
    loop {
        digits[count] = b'0' + (whole % 10) as u8;
        count += 1;
        whole /= 10;
        if whole == 0 {
            break;
        }
    }
    for &digit in digits[..count].iter().rev() {
        out[length] = digit;
        length += 1;
    }
    out[length] = b'.';
    out[length + 1] = b'0' + (tenths % 10) as u8;
    length + 2
}

/// The frame rate in tenths of a frame per second that a message from [`encode_fps`] reports.
/// `None` for anything else (so a controller can ignore messages it does not know).
pub fn decode_fps(message: &[u8]) -> Option<u32> {
    let rest = message.strip_prefix(FPS_PREFIX)?;
    let (whole, tenth) = match rest {
        [whole @ .., b'.', tenth] => (whole, *tenth),
        _ => return None,
    };
    if whole.is_empty() || whole.len() > 10 || !tenth.is_ascii_digit() {
        return None;
    }
    let mut tenths: u64 = 0;
    for &digit in whole {
        if !digit.is_ascii_digit() {
            return None;
        }
        tenths = tenths * 10 + u64::from(digit - b'0');
    }
    u32::try_from(tenths * 10 + u64::from(tenth - b'0')).ok()
}

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
    /// Mutes or unmutes the speaker. Not a game key: the firmware acts on the press itself and
    /// the game never sees it.
    ToggleSound,
    /// A typed printable ASCII character. Build it with [`Command::typed`], which checks the range.
    Char(u8),
}

/// The typed characters that have a code: printable ASCII, whose code is the character itself.
const FIRST_CHAR: u8 = 32;
const LAST_CHAR: u8 = 126;

impl Command {
    /// The game actions, in code order. Typed characters are not in this list.
    pub const ALL: [Self; 23] = [
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
        Self::ToggleSound,
    ];

    /// The typed character for the ASCII byte `byte`, if it is printable.
    pub fn typed(byte: u8) -> Option<Self> {
        (FIRST_CHAR..=LAST_CHAR)
            .contains(&byte)
            .then_some(Self::Char(byte))
    }

    /// The command with wire code `code`, if there is one.
    pub fn from_code(code: u8) -> Option<Self> {
        Self::ALL
            .get(usize::from(code))
            .copied()
            .or_else(|| Self::typed(code))
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
            Self::ToggleSound => 22,
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
        Self {
            command,
            pressed: true,
        }
    }

    pub fn release(command: Command) -> Self {
        Self {
            command,
            pressed: false,
        }
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

/// Sits between the queue of received events and the game's input polls, so that every press is
/// seen by the game before its release.
///
/// The game (the engine's `I_GetEvent`) drains the events once per tic and stops after the first
/// release it sees. A tap whose press and release were both waiting in the queue at that moment
/// (they arrived close together, or the frame was slow) was therefore over before the game's next
/// tic looked at which keys are held, and the tap was lost. `PollGate::next` holds such a release
/// back: it ends the poll (returns `None`) and hands the release out first thing in the next poll,
/// so the key is down for at least one tic. Typed characters are not held keys (the game acts on
/// the press), so their releases pass straight through.
///
/// `pop` returns the next queued event, if any. A poll is the run of calls up to a `None` or a
/// release.
#[derive(Clone, Copy, Debug, Default)]
pub struct PollGate {
    /// A release held back for the next poll.
    held_back: Option<KeyEvent>,
    /// The commands (bit per code) pressed earlier in this poll.
    pressed_this_poll: u32,
}

impl PollGate {
    pub const fn new() -> Self {
        Self {
            held_back: None,
            pressed_this_poll: 0,
        }
    }

    /// The next event for the game, or `None` to end this poll.
    pub fn next(&mut self, mut pop: impl FnMut() -> Option<KeyEvent>) -> Option<KeyEvent> {
        let Some(event) = self.held_back.take().or_else(&mut pop) else {
            self.pressed_this_poll = 0;
            return None;
        };
        let bit = match event.command {
            Command::Char(_) => 0,
            command => 1 << u32::from(command.code()),
        };
        if event.pressed {
            self.pressed_this_poll |= bit;
            return Some(event);
        }
        if self.pressed_this_poll & bit != 0 {
            // Pressed in this very poll: the game has not had its tic with the key down yet.
            self.held_back = Some(event);
            self.pressed_this_poll = 0;
            return None;
        }
        // A release ends the game's poll.
        self.pressed_this_poll = 0;
        Some(event)
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
            assert_eq!(
                command.code(),
                byte,
                "the code of a character is its ASCII value"
            );
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
            [
                KeyEvent::release(Command::Forward),
                KeyEvent::release(Command::Fire)
            ]
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

    fn fps(tenths: u32) -> Vec<u8> {
        let mut out = [0u8; FPS_MESSAGE_MAX];
        let length = encode_fps(tenths, &mut out);
        out[..length].to_vec()
    }

    #[test]
    fn a_frame_rate_is_written_as_text_with_one_decimal() {
        assert_eq!(fps(284), b"fps 28.4");
        assert_eq!(fps(300), b"fps 30.0");
        assert_eq!(fps(5), b"fps 0.5");
        assert_eq!(fps(0), b"fps 0.0");
        assert_eq!(fps(1234), b"fps 123.4");
    }

    #[test]
    fn a_frame_rate_round_trips_and_always_fits() {
        for tenths in [0, 1, 9, 10, 99, 284, 999, 6553, 100_000, u32::MAX] {
            let message = fps(tenths);
            assert!(message.len() <= FPS_MESSAGE_MAX);
            assert_eq!(decode_fps(&message), Some(tenths));
        }
    }

    #[test]
    fn other_messages_are_not_a_frame_rate() {
        for message in [
            &b""[..],
            b"fps",
            b"fps ",
            b"fps 28",
            b"fps 28.",
            b"fps .4",
            b"fps 28.44",
            b"fps -1.0",
            b"fps 2x.4",
            b"fps 28,4",
            b"FPS 28.4",
            b"fps 28.4 ",
            b"ping",
            b"fps 99999999999.0",
            b"fps 429496729.6",
        ] {
            assert_eq!(
                decode_fps(message),
                None,
                "{:?}",
                core::str::from_utf8(message)
            );
        }
    }

    /// What the game sees over several polls: each inner list is one poll, ended by `None` or by
    /// a release (which the game's loop stops at).
    fn polls(gate: &mut PollGate, queue: &mut Vec<KeyEvent>, polls: usize) -> Vec<Vec<KeyEvent>> {
        let mut out = Vec::new();
        for _ in 0..polls {
            let mut seen = Vec::new();
            while let Some(event) = gate.next(|| (!queue.is_empty()).then(|| queue.remove(0))) {
                seen.push(event);
                if !event.pressed {
                    break;
                }
            }
            out.push(seen);
        }
        out
    }

    #[test]
    fn a_tap_that_arrived_whole_is_seen_held_for_a_poll() {
        let mut gate = PollGate::new();
        let mut queue = std::vec![
            KeyEvent::press(Command::Fire),
            KeyEvent::release(Command::Fire)
        ];
        assert_eq!(
            polls(&mut gate, &mut queue, 3),
            [
                std::vec![KeyEvent::press(Command::Fire)],
                std::vec![KeyEvent::release(Command::Fire)],
                std::vec![],
            ]
        );
    }

    #[test]
    fn a_release_after_an_earlier_poll_goes_straight_through() {
        let mut gate = PollGate::new();
        let mut queue = std::vec![KeyEvent::press(Command::Forward)];
        assert_eq!(
            polls(&mut gate, &mut queue, 1),
            [std::vec![KeyEvent::press(Command::Forward)]]
        );
        queue.push(KeyEvent::release(Command::Forward));
        assert_eq!(
            polls(&mut gate, &mut queue, 1),
            [std::vec![KeyEvent::release(Command::Forward)]]
        );
    }

    #[test]
    fn releasing_another_key_is_not_held_back() {
        let mut gate = PollGate::new();
        let mut queue = std::vec![
            KeyEvent::press(Command::Fire),
            KeyEvent::release(Command::Forward),
            KeyEvent::release(Command::Fire),
        ];
        assert_eq!(
            polls(&mut gate, &mut queue, 2),
            [
                std::vec![
                    KeyEvent::press(Command::Fire),
                    KeyEvent::release(Command::Forward)
                ],
                std::vec![KeyEvent::release(Command::Fire)],
            ]
        );
    }

    #[test]
    fn typed_characters_pass_with_their_releases() {
        let mut gate = PollGate::new();
        let mut queue = std::vec![
            KeyEvent::press(Command::Char(b'i')),
            KeyEvent::release(Command::Char(b'i')),
            KeyEvent::press(Command::Char(b'd')),
            KeyEvent::release(Command::Char(b'd')),
        ];
        assert_eq!(
            polls(&mut gate, &mut queue, 2),
            [
                std::vec![
                    KeyEvent::press(Command::Char(b'i')),
                    KeyEvent::release(Command::Char(b'i'))
                ],
                std::vec![
                    KeyEvent::press(Command::Char(b'd')),
                    KeyEvent::release(Command::Char(b'd'))
                ],
            ]
        );
    }

    #[test]
    fn a_held_back_release_keeps_its_place_and_nothing_is_lost() {
        let mut gate = PollGate::new();
        let mut queue = std::vec![
            KeyEvent::press(Command::Use),
            KeyEvent::release(Command::Use),
            KeyEvent::press(Command::Use),
            KeyEvent::release(Command::Use),
        ];
        let seen: Vec<KeyEvent> = polls(&mut gate, &mut queue, 6)
            .into_iter()
            .flatten()
            .collect();
        assert_eq!(
            seen,
            [
                KeyEvent::press(Command::Use),
                KeyEvent::release(Command::Use),
                KeyEvent::press(Command::Use),
                KeyEvent::release(Command::Use),
            ]
        );
    }
}
