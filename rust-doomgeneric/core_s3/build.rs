//! Passes the Wi-Fi credentials to the firmware as `WIFI_SSID` / `WIFI_PASSWORD` compile-time
//! constants. They come from the environment, else from `wifi.env` (see `wifi.env.example`).
//! If neither has them the constants are empty and the firmware makes its own open network instead
//! of joining one (see `src/net.rs`).
//!
//! It also gzips the web page (`assets/controller.html`) into `OUT_DIR`, so the board sends about a
//! third of the bytes through its small TCP buffers (see `src/web.rs`).

use std::{env, fs, io::Write, path::PathBuf};

fn main() {
    gzip_page();
    println!("cargo:rerun-if-changed=wifi.env");
    println!("cargo:rerun-if-env-changed=WIFI_SSID");
    println!("cargo:rerun-if-env-changed=WIFI_PASSWORD");

    let file = fs::read_to_string("wifi.env").unwrap_or_default();
    let from_file = |key: &str| {
        file.lines()
            .filter_map(|line| line.split_once('='))
            .find(|(name, _)| name.trim() == key)
            .map(|(_, value)| value.trim().trim_matches('"').to_owned())
    };
    for key in ["WIFI_SSID", "WIFI_PASSWORD"] {
        let value = env::var(key)
            .ok()
            .or_else(|| from_file(key))
            .unwrap_or_default();
        println!("cargo:rustc-env={key}={value}");
    }
}

/// Writes `controller.html.gz` (maximum compression, no timestamp, so the build is reproducible).
fn gzip_page() {
    println!("cargo:rerun-if-changed=assets/controller.html");
    let page = fs::read("assets/controller.html").expect("assets/controller.html");
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(&page).expect("gzip the page");
    let gz = encoder.finish().expect("gzip the page");
    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("controller.html.gz");
    fs::write(out, gz).expect("write controller.html.gz");
}
