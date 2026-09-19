//! Passes the Wi-Fi credentials to the firmware as `WIFI_SSID` / `WIFI_PASSWORD` compile-time
//! constants. They come from the environment, else from `wifi.env` (see `wifi.env.example`).
//! If neither has them the constants are empty and the firmware runs without Wi-Fi.

use std::{env, fs};

fn main() {
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
        let value = env::var(key).ok().or_else(|| from_file(key)).unwrap_or_default();
        println!("cargo:rustc-env={key}={value}");
    }
}
