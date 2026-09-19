//! Turning keyboard input into `core_s3_protocol` bytes. The terminal handling lives in
//! `main.rs`; everything testable without a terminal or a socket lives here.

use core_s3_protocol::{Command, KeyEvent};
use crossterm::event::KeyCode;
use std::io::{self, Write};
use std::time::{Duration, Instant};

/// The command a key is bound to, if any.
///
/// Arrows or WASD move and turn, `q`/`e` (or `,`/`.`) strafe, Space fires, `f` uses/opens,
/// `1`-`7` pick a weapon, Tab is the map, Enter/Esc/`y`/`n` drive the menus. Holding Run is
/// handled separately in `main.rs` (a sticky toggle on `r`), because terminals cannot report
/// Shift on its own.
pub fn command_for(code: KeyCode) -> Option<Command> {
    Some(match code {
        KeyCode::Up => Command::Forward,
        KeyCode::Down => Command::Backward,
        KeyCode::Left => Command::TurnLeft,
        KeyCode::Right => Command::TurnRight,
        KeyCode::Enter => Command::Enter,
        KeyCode::Esc => Command::Escape,
        KeyCode::Tab => Command::Map,
        KeyCode::Char(c) => match c.to_ascii_lowercase() {
            'w' => Command::Forward,
            's' => Command::Backward,
            'a' => Command::TurnLeft,
            'd' => Command::TurnRight,
            'q' | ',' => Command::StrafeLeft,
            'e' | '.' => Command::StrafeRight,
            ' ' => Command::Fire,
            'f' => Command::Use,
            'y' => Command::Yes,
            'n' => Command::No,
            '1' => Command::Weapon1,
            '2' => Command::Weapon2,
            '3' => Command::Weapon3,
            '4' => Command::Weapon4,
            '5' => Command::Weapon5,
            '6' => Command::Weapon6,
            '7' => Command::Weapon7,
            _ => return None,
        },
        _ => return None,
    })
}

/// Fake key releases for terminals that only report presses.
///
/// Such a terminal repeats a held key (after its initial repeat delay), so a key counts as held
/// while presses keep arriving and is released `hold` after the last one.
pub struct HoldTracker {
    hold: Duration,
    held: Vec<(Command, Instant)>,
}

impl HoldTracker {
    pub fn new(hold: Duration) -> Self {
        Self { hold, held: Vec::new() }
    }

    /// A press (or auto-repeat) of `command` was seen. Returns the press event the first time.
    pub fn key_seen(&mut self, command: Command, now: Instant) -> Option<KeyEvent> {
        if let Some((_, last_seen)) = self.held.iter_mut().find(|(held, _)| *held == command) {
            *last_seen = now;
            return None;
        }
        self.held.push((command, now));
        Some(KeyEvent::press(command))
    }

    /// Releases every key that has not been seen for `hold`.
    pub fn expire(&mut self, now: Instant) -> Vec<KeyEvent> {
        let hold = self.hold;
        let mut released = Vec::new();
        self.held.retain(|&(command, last_seen)| {
            let expired = now.duration_since(last_seen) >= hold;
            if expired {
                released.push(KeyEvent::release(command));
            }
            !expired
        });
        released
    }

    /// Releases everything, for shutdown.
    pub fn release_all(&mut self) -> Vec<KeyEvent> {
        self.held.drain(..).map(|(command, _)| KeyEvent::release(command)).collect()
    }
}

/// Sends one event as its single byte, flushing so it leaves immediately.
pub fn send(out: &mut impl Write, event: KeyEvent) -> io::Result<()> {
    out.write_all(&[event.encode()])?;
    out.flush()
}

/// The address to connect to: `host` gets the default port, `host:port` is used as given.
pub fn with_default_port(address: &str) -> String {
    if address.contains(':') {
        address.to_owned()
    } else {
        format!("{address}:{}", core_s3_protocol::DEFAULT_PORT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::{TcpListener, TcpStream};

    const HOLD: Duration = Duration::from_millis(100);

    #[test]
    fn keys_map_to_commands() {
        assert_eq!(command_for(KeyCode::Up), Some(Command::Forward));
        assert_eq!(command_for(KeyCode::Char('W')), Some(Command::Forward));
        assert_eq!(command_for(KeyCode::Char(' ')), Some(Command::Fire));
        assert_eq!(command_for(KeyCode::Char('3')), Some(Command::Weapon3));
        assert_eq!(command_for(KeyCode::Char('z')), None);
        assert_eq!(command_for(KeyCode::F(1)), None);
    }

    #[test]
    fn every_command_has_a_key() {
        let keys = [
            KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right, KeyCode::Enter,
            KeyCode::Esc, KeyCode::Tab, KeyCode::Char('q'), KeyCode::Char('e'),
            KeyCode::Char(' '), KeyCode::Char('f'), KeyCode::Char('y'), KeyCode::Char('n'),
            KeyCode::Char('1'), KeyCode::Char('2'), KeyCode::Char('3'), KeyCode::Char('4'),
            KeyCode::Char('5'), KeyCode::Char('6'), KeyCode::Char('7'),
        ];
        let bound: Vec<Command> = keys.into_iter().filter_map(command_for).collect();
        // Run is the only command without a plain key (it is the `r` toggle in main.rs).
        for command in Command::ALL.into_iter().filter(|&c| c != Command::Run) {
            assert!(bound.contains(&command), "{command:?} has no key");
        }
    }

    #[test]
    fn a_held_key_presses_once_and_releases_after_the_hold_time() {
        let mut tracker = HoldTracker::new(HOLD);
        let t0 = Instant::now();
        assert_eq!(tracker.key_seen(Command::Forward, t0), Some(KeyEvent::press(Command::Forward)));
        // Auto-repeat keeps it held without sending anything new.
        let t1 = t0 + Duration::from_millis(80);
        assert_eq!(tracker.key_seen(Command::Forward, t1), None);
        assert!(tracker.expire(t1 + Duration::from_millis(99)).is_empty());
        assert_eq!(
            tracker.expire(t1 + HOLD),
            vec![KeyEvent::release(Command::Forward)]
        );
        // Once released, the next press is a fresh press event.
        let t2 = t1 + Duration::from_secs(1);
        assert_eq!(tracker.key_seen(Command::Forward, t2), Some(KeyEvent::press(Command::Forward)));
    }

    #[test]
    fn keys_expire_independently() {
        let mut tracker = HoldTracker::new(HOLD);
        let t0 = Instant::now();
        tracker.key_seen(Command::Forward, t0);
        tracker.key_seen(Command::Fire, t0 + Duration::from_millis(60));
        assert_eq!(tracker.expire(t0 + HOLD), vec![KeyEvent::release(Command::Forward)]);
        assert_eq!(
            tracker.release_all(),
            vec![KeyEvent::release(Command::Fire)]
        );
        assert!(tracker.release_all().is_empty());
    }

    #[test]
    fn default_port_is_added_only_when_missing() {
        assert_eq!(with_default_port("192.168.1.50"), "192.168.1.50:7878");
        assert_eq!(with_default_port("doom.local:9000"), "doom.local:9000");
    }

    #[test]
    fn events_arrive_over_tcp_one_byte_each() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        client.set_nodelay(true).unwrap();
        let (mut server, _) = listener.accept().unwrap();

        send(&mut client, KeyEvent::press(Command::Fire)).unwrap();
        send(&mut client, KeyEvent::release(Command::Fire)).unwrap();
        drop(client);

        let mut received = Vec::new();
        server.read_to_end(&mut received).unwrap();
        let events: Vec<_> = received.into_iter().map(KeyEvent::decode).collect();
        assert_eq!(
            events,
            vec![Some(KeyEvent::press(Command::Fire)), Some(KeyEvent::release(Command::Fire))]
        );
    }
}
