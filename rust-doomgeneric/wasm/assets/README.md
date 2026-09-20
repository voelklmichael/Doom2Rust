# assets

`build.rs` embeds `doom1.wad` from this directory into the WebAssembly module (with
`include_bytes!`), so the file has to be here before a build for `wasm32-unknown-unknown`. It is
gitignored.

Use the shareware IWAD (`doom1.wad`, 4,196,020 bytes, E1M1-E1M9 and three demos). It is freely
distributable, but the full game's WAD is not, and it would make the download about 11 MB.

On any other target (`cargo check`/`cargo test` for the host) a missing WAD is fine: the crate is
built with an empty one.
