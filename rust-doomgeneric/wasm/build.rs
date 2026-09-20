//! Tells the crate where the IWAD it embeds is (`DOOM_WAD`), and sets the WebAssembly stack size.
use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let wad = manifest_dir.join("assets/doom1.wad");
    println!("cargo:rerun-if-changed={}", wad.display());
    println!("cargo:rerun-if-changed=build.rs");

    let wasm = env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32";
    let wad = if wad.exists() {
        wad
    } else if wasm {
        panic!(
            "{} is missing: put the shareware IWAD there (see assets/README.md)",
            wad.display()
        );
    } else {
        // Host builds (cargo check, cargo test, rust-analyzer) do not need the game data.
        let empty = PathBuf::from(env::var("OUT_DIR").unwrap()).join("empty.wad");
        fs::write(&empty, []).unwrap();
        empty
    };
    println!("cargo:rustc-env=DOOM_WAD={}", wad.display());

    if wasm {
        // `init_game_state` builds the engine's state (hundreds of KB) by value before boxing it,
        // which does not fit the default 1 MiB stack with the rest of the start-up.
        println!("cargo:rustc-link-arg=-zstack-size=8388608");
    }
}
