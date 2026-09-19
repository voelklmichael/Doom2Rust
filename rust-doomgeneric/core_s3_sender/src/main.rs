//! Keyboard controller for DOOM running on the CoreS3: connects to the board over TCP and sends
//! one byte per key press or release (see `core_s3_protocol`).
//!
//! ```text
//! cargo run -p core_s3_sender -- <board-ip>[:port] [--hold-ms N]
//! ```
//!
//! Keys: arrows or WASD move/turn, Q/E strafe, Space fire, F use, 1-7 weapons, Tab map,
//! Enter/Esc/Y/N menus, R toggles run, Ctrl-C quits.
//!
//! Terminals that support the kitty keyboard protocol (kitty, WezTerm, foot, Ghostty, ...) report
//! real key releases. On other terminals a key counts as released `--hold-ms` (default 150) after
//! its last press or auto-repeat, so holding a key briefly stutters until auto-repeat starts.

use core_s3_protocol::{Command, KeyEvent};
use core_s3_sender::{command_for, send, with_default_port, HoldTracker, KEY_MAP};
use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode};
use crossterm::execute;
use std::io::{self, Write};
use std::net::TcpStream;
use std::process::ExitCode;
use std::time::{Duration, Instant};

const DEFAULT_HOLD_MS: u64 = 150;
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Puts the terminal in raw mode (and, if supported, key-release reporting), and undoes it on drop.
struct RawTerminal {
    reports_releases: bool,
}

impl RawTerminal {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let reports_releases = terminal::supports_keyboard_enhancement().unwrap_or(false);
        if reports_releases {
            execute!(
                io::stdout(),
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                )
            )?;
        }
        Ok(Self { reports_releases })
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        if self.reports_releases {
            let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
        }
        let _ = disable_raw_mode();
    }
}

struct Args {
    address: String,
    hold: Duration,
}

fn parse_args() -> Result<Args, String> {
    let mut address = None;
    let mut hold = Duration::from_millis(DEFAULT_HOLD_MS);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--hold-ms" {
            let value = args.next().ok_or("--hold-ms needs a value")?;
            let ms = value.parse().map_err(|_| format!("bad --hold-ms value: {value}"))?;
            hold = Duration::from_millis(ms);
        } else if address.is_none() {
            address = Some(with_default_port(&arg));
        } else {
            return Err(format!("unexpected argument: {arg}"));
        }
    }
    let address = address.ok_or("usage: core_s3_sender <board-ip>[:port] [--hold-ms N]")?;
    Ok(Args { address, hold })
}

fn run(args: &Args) -> io::Result<()> {
    let mut stream = TcpStream::connect(&args.address)?;
    // One byte per event: never let Nagle's algorithm batch them up.
    stream.set_nodelay(true)?;
    println!("connected to {}", args.address);
    // Before raw mode, while "\n" still returns the cursor to the start of the line.
    println!("{KEY_MAP}");

    let terminal = RawTerminal::enter()?;
    let mut tracker = HoldTracker::new(args.hold);
    let mut run_on = false;
    let result = (|| -> io::Result<()> {
        loop {
            if event::poll(POLL_INTERVAL)? {
                let Event::Key(key) = event::read()? else { continue };
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(());
                }
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('r') {
                    run_on = !run_on;
                    send(&mut stream, KeyEvent { command: Command::Run, pressed: run_on })?;
                    continue;
                }
                let Some(command) = command_for(key.code) else { continue };
                if terminal.reports_releases {
                    match key.kind {
                        KeyEventKind::Press => send(&mut stream, KeyEvent::press(command))?,
                        KeyEventKind::Release => send(&mut stream, KeyEvent::release(command))?,
                        KeyEventKind::Repeat => {}
                    }
                } else if key.kind != KeyEventKind::Release {
                    if let Some(press) = tracker.key_seen(command, Instant::now()) {
                        send(&mut stream, press)?;
                    }
                }
            }
            for release in tracker.expire(Instant::now()) {
                send(&mut stream, release)?;
            }
        }
    })();
    // Leave nothing stuck down on the board, even if we are quitting on an error.
    for release in tracker.release_all() {
        let _ = send(&mut stream, release);
    }
    if run_on {
        let _ = send(&mut stream, KeyEvent::release(Command::Run));
    }
    let _ = stream.flush();
    result
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("core_s3_sender: {err}");
            ExitCode::FAILURE
        }
    }
}
