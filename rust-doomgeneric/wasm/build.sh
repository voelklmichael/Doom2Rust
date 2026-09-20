#!/usr/bin/env bash
# Builds the game for the browser into www/pkg. Needs the wasm32 target
# (rustup target add wasm32-unknown-unknown) and the wasm-bindgen command line tool at the
# version pinned in Cargo.toml (cargo install wasm-bindgen-cli --version 0.2.121), and the IWAD
# in assets/ (see assets/README.md).
#
#   wasm/build.sh          build
#   wasm/build.sh serve    build, then serve www/ on http://localhost:8000
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir www/pkg --no-typescript \
    ../target/wasm32-unknown-unknown/release/doomgeneric_wasm.wasm

if [ "${1:-}" = serve ]; then
    echo "http://localhost:8000"
    exec python3 -m http.server 8000 --directory www
fi
