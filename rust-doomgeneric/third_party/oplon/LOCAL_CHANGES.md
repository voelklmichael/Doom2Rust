# Local changes to oplon 1.0.0

This is the crates.io release of `oplon` 1.0.0 (MIT, (c) 2026 Sebastien Bechet, see `LICENSE`) with
a few changes for the CoreS3 firmware (`../../core_s3`), which uses it through `[patch.crates-io]`.
The host build and the engine tests keep using the crates.io release. Output is bit-identical:
`cargo run --release --example opl2_hash` (with or without `--features iram`) prints the same hash,
`a17a333af7d2d119`, as the release, and the in-crate unit tests pass.

* `src/operator.rs`, `src/chip.rs`: `#[inline]` on the per-operator path became `#[inline(always)]`
  (and `render_native`, `next_mono`, `is_silent` were added to it), so the whole synthesis step is
  one function, `Opl2::render_frame`.
* Feature `iram` (off by default): `render_frame` is linked into `.rwtext` (esp-hal's internal-RAM
  code section) and the four tables the loop reads every sample (`EXP_OUT`, `LOGSIN_ROM`, `MUL`,
  `EG_INC`, now `static`s) into `.data.oplon`. Nothing of the hot loop is then fetched through the
  flash cache, which the ESP32-S3's two cores share with the PSRAM: with the loop in flash the game
  on the other core lost 1.8 fps of 28 to it (see `core_s3/PLAN.md`, Music).
* `Cargo.toml`: no binary or dev-dependencies, an empty `[workspace]` (this copy is not a member of
  the host workspace), the `iram` feature.

`examples/opl2_hash.rs` is the release's own example, kept as the regression check.
