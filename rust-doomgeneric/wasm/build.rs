//! Sets the WebAssembly stack size.
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32" {
        // `init_game_state` builds the engine's state (hundreds of KB) by value before boxing it,
        // which does not fit the default 1 MiB stack with the rest of the start-up.
        println!("cargo:rustc-link-arg=-zstack-size=8388608");
    }
}
