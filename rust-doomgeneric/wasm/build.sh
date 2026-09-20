#!/usr/bin/env bash
# Builds the game for the browser into www/pkg. Needs the wasm32 target
# (rustup target add wasm32-unknown-unknown) and the wasm-bindgen command line tool at the
# version pinned in Cargo.toml (cargo install wasm-bindgen-cli --version 0.2.121).
#
#   wasm/build.sh          build
#   wasm/build.sh serve    build, then serve www/ on http://localhost:8000
set -euo pipefail
cd "$(dirname "$0")"

# What the page shows as its version: when it was built, and from which commit. It is baked into
# the wasm (`build_version`) and written to www/version.js and www/version.json, so that the page
# can tell whether it is running one build's code together with another's.
commit=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
if [ -n "$(git status --porcelain 2>/dev/null)" ]; then commit="$commit-dirty"; fi
export DOOM_BUILD="$(date -u '+%Y-%m-%d %H:%M UTC') $commit"

# The `wasm` profile (see the workspace Cargo.toml) optimizes for size; the name section is only
# useful for debugging and is 12% of the file.
cargo build --profile wasm --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir www/pkg --no-typescript \
    --remove-name-section --remove-producers-section \
    ../target/wasm32-unknown-unknown/wasm/doomgeneric_wasm.wasm

# `files` is what the page's "Update" button fetches afresh: everything the page loads.
files=$(cd www && find . -type f ! -name 'version.js*' | sed 's|^\./||' | sort | sed 's|.*|  "&",|')
printf 'export const VERSION = "%s";\nexport const FILES = [\n%s\n  "version.js",\n];\n' \
    "$DOOM_BUILD" "$files" > www/version.js
printf '{"version": "%s"}\n' "$DOOM_BUILD" > www/version.json

if [ "${1:-}" = serve ]; then
    echo "http://localhost:8000"
    exec python3 -m http.server 8000 --directory www
fi
