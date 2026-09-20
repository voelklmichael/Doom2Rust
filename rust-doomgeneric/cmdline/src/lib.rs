//! Turns a command line into the engine's [`Options`].
//!
//! Vanilla scans `myargv` with `M_CheckParm("-flag")` wherever it needs an option. Here the
//! arguments are parsed once, by the frontend, into the engine's `Options`: a `bool` per switch
//! and an `Option` per option that takes a value, so every option the engine understands is listed
//! in one place (`rust_doomgeneric::Options`) and the engine never sees a string it has to
//! interpret.
//!
//! The rules of the C lookup are kept exactly: option names match case-insensitively, the program
//! name (`argv[0]`) is never an option, the *first* occurrence wins, and an option that needs `n`
//! values only counts if at least `n` arguments follow it (an option that is last on the line is
//! ignored, not an error). Unknown arguments are ignored. Numbers are read like C's `atoi`, and
//! `-skill` / `-episode` / `-warp` use the first character of the argument, as vanilla did.
#![no_std]
extern crate alloc;
#[cfg(test)]
extern crate std;

use alloc::string::String;
use alloc::vec::Vec;
use rust_doomgeneric::{Options, Warp};

/// The index in `argv` of `check`, provided at least `num_args` more arguments follow it. The
/// program name at index 0 is not an option.
fn find(argv: &[String], check: &str, num_args: usize) -> Option<usize> {
    (1..argv.len().saturating_sub(num_args)).find(|&i| argv[i].eq_ignore_ascii_case(check))
}

/// Reads the options out of `argv` (whose first element is the program name).
pub fn parse(argv: &[String]) -> Options {
    let switch = |name: &str| find(argv, name, 0).is_some();
    let value = |name: &str| find(argv, name, 1).map(|p| argv[p + 1].clone());
    let number = |name: &str| find(argv, name, 1).map(|p| argv_atoi(&argv[p + 1]));
    let digit = |name: &str| find(argv, name, 1).map(|p| first_digit(&argv[p + 1]));
    Options {
        altdeath: switch("-altdeath"),
        avg: switch("-avg"),
        deathmatch: switch("-deathmatch"),
        devparm: switch("-devparm"),
        fast: switch("-fast"),
        left: switch("-left"),
        longtics: switch("-longtics"),
        netdemo: switch("-netdemo"),
        nodraw: switch("-nodraw"),
        nomonsters: switch("-nomonsters"),
        nomusic: switch("-nomusic"),
        nosfx: switch("-nosfx"),
        nosound: switch("-nosound"),
        record: switch("-record"),
        reject_pad_with_ff: switch("-reject_pad_with_ff"),
        respawn: switch("-respawn"),
        right: switch("-right"),
        solo_net: switch("-solo-net"),
        statdump: switch("-statdump"),
        testcontrols: switch("-testcontrols"),

        config: value("-config"),
        donut: find(argv, "-donut", 2).map(|p| (argv[p + 1].clone(), argv[p + 2].clone())),
        episode: digit("-episode"),
        extraconfig: value("-extraconfig"),
        // every argument after `-file` up to the next option
        file: find(argv, "-file", 1).map(|p| {
            argv[p + 1..]
                .iter()
                .take_while(|arg| !arg.starts_with('-'))
                .cloned()
                .collect::<Vec<_>>()
        }),
        gameversion: value("-gameversion"),
        gfxmode: value("-gfxmode"),
        iwad: value("-iwad"),
        loadgame: number("-loadgame"),
        maxdemo: number("-maxdemo"),
        pack: value("-pack"),
        playdemo: value("-playdemo"),
        record_file: value("-record"),
        scaling: number("-scaling"),
        setmem: find(argv, "-setmem", 1).map(|p| argv[p + 1..].to_vec()),
        skill: digit("-skill"),
        spechit: value("-spechit"),
        statdump_file: value("-statdump"),
        timedemo: value("-timedemo"),
        timer: number("-timer"),
        // `-turbo` with an optional value after it
        turbo: find(argv, "-turbo", 0).map(|p| argv.get(p + 1).map(|arg| argv_atoi(arg))),
        // read both ways: the engine picks by game mode
        warp: find(argv, "-warp", 1).map(|p| Warp {
            map_number: argv_atoi(&argv[p + 1]),
            episode: first_digit(&argv[p + 1]),
            episode_map: argv.get(p + 2).map_or(1, |arg| first_digit(arg)),
        }),
    }
}

/// The digit a level or episode argument starts with (`'0'` is 0), from its first character; 0
/// for an empty argument, as the C code did with a NUL byte.
fn first_digit(arg: &str) -> i32 {
    i32::from(arg.as_bytes().first().copied().unwrap_or(0)) - i32::from(b'0')
}

/// C's `atoi`: leading whitespace, an optional sign, then digits; anything else stops the number
/// (0 if there are none). Overflow wraps.
fn argv_atoi(arg: &str) -> i32 {
    let bytes = arg.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let negative = i < bytes.len() && bytes[i] == b'-';
    if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
        i += 1;
    }
    let mut value: i32 = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        value = value
            .wrapping_mul(10)
            .wrapping_add(i32::from(bytes[i] - b'0'));
        i += 1;
    }
    if negative {
        -value
    } else {
        value
    }
}

#[cfg(test)]
mod tests;
