//! Console output for the engine. `println!` and friends need `std`, so the
//! engine formats into a `String` and hands it to `DoomPlatform::print` /
//! `eprint`. The first argument is anything with those methods, normally
//! `state.io.platform`.

macro_rules! doom_print {
    ($p:expr, $($arg:tt)*) => {{
        let message = alloc::format!($($arg)*);
        $p.print(&message);
    }};
}

macro_rules! doom_println {
    ($p:expr) => {
        $p.print("\n")
    };
    ($p:expr, $($arg:tt)*) => {{
        let mut message = alloc::format!($($arg)*);
        message.push('\n');
        $p.print(&message);
    }};
}

macro_rules! doom_eprint {
    ($p:expr, $($arg:tt)*) => {{
        let message = alloc::format!($($arg)*);
        $p.eprint(&message);
    }};
}

macro_rules! doom_eprintln {
    ($p:expr) => {
        $p.eprint("\n")
    };
    ($p:expr, $($arg:tt)*) => {{
        let mut message = alloc::format!($($arg)*);
        message.push('\n');
        $p.eprint(&message);
    }};
}
