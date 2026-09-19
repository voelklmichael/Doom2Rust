# Filesystem crate extraction, and the path to `#![no_std]` + `alloc`

## Goal

Long term: the `engine` crate (`rust_doomgeneric`) builds as `#![no_std]` with
`extern crate alloc`. Everything that needs an operating system lives in the host
crates. Two boundaries already exist or are being added:

| Concern | Engine-side trait | Implementation |
|---|---|---|
| Window, input, timing | `DoomPlatform` (`engine/src/platform.rs`) | `x11` crate |
| Files | `DoomFileSystem` (`engine/src/filesystem.rs`) | `fs` crate (`doomgeneric_fs`) |

Dependency direction: `x11 -> doomgeneric_fs -> rust_doomgeneric`. The engine defines
the trait, so it never depends on a `std` crate; `doomgeneric_fs` sits between the
engine and `x11` and implements it as `StdFileSystem`. The host passes both backends
to `init_game_state(platform, fs)`, and the engine stores them as
`GameState.platform` / `GameState.fs`.

## Track A: filesystem extraction (DONE, PRs #478-#481)

Status: all four phases landed as a stack. `engine/src` no longer contains `std::fs` or
`std::io`; a unit test (`filesystem::tests::engine_sources_do_not_use_std_fs_or_io`) fails
if either comes back.

Where the engine touched the filesystem before this track:

| Area | Sites |
|---|---|
| WAD open / length / `read_at` | `w_file.rs`, `M_FileLength` |
| Save/load streams, save-slot strings | `g_game.rs` (`G_DoLoadGame`, `G_DoSaveGame`), `p_saveg.rs`, `m_menu.rs` (`M_ReadSaveStrings`) |
| Save-file move/delete | `g_game.rs` (`remove_file`, `rename`) |
| Demo / screenshot writes | `M_WriteFile` (`g_game.rs`, `v_video.rs`) |
| Exists checks | `M_FileExists` (`d_iwad.rs`, `v_video.rs`) |
| Directories, temp path | `M_MakeDirectory` (`m_config.rs`), `M_TempFile` |

Config-file loading/saving and response files are stubs in the engine (`M_SaveDefaults`,
`M_FindResponseFile`), so nothing to extract there.

Phases, one PR each (stacked):

1. **Scaffold**: `DoomFileSystem` trait + `FileId`, the `doomgeneric_fs` crate with
   `StdFileSystem`, `GameState.fs`, x11 wiring, and a test-only `MemFileSystem`.
2. **WADs**: `wad_file_t` / `W_OpenFile` / `W_Read` replaced by `FileId`. `lumpinfo_t`
   stores the id (which is `Copy`), so the `Box::leak` and `&'static` go away;
   `w_checksum` numbers files by id instead of pointer identity; `w_file.rs` is deleted.
3. **Savegames**: load reads the whole file into a `Vec<u8>` with a cursor; save builds
   a `Vec<u8>` and writes it via `fs.write_file`, then `remove_file` + `rename`.
   `PSavegState.save_stream` is deleted. Failure timing changes (a write error surfaces
   at flush instead of at open); see `known-deviations.md`.
4. **Remaining callers**: `M_WriteFile`, `M_FileExists`, `M_MakeDirectory`,
   `M_TempFile` go through `state.fs`; the `std::fs` code in `m_misc.rs` is deleted.

Verification for each phase: build, `cargo test --release`, and the demo1/2/3 mobj+sector
hash oracle (byte-identical to main). Phase 3 additionally compares a written `.dsg`
byte-for-byte against main's.

## Track B: `no_std` + `alloc` (future)

Not started. Blockers left after Track A, in rough order of effort:

1. **Console output**: ~60 `println!`/`eprintln!`/`print!` sites (`d_main`, `d_net`,
   `i_video`, `m_config`, `i_system`, ...). Route them through the platform trait
   (for example `DoomPlatform::log(&str)`) or a `doom_print!` macro that calls it.
2. **Process exit**: `std::process::exit(0)` in `D_Endoom` (`d_main.rs`). Becomes a
   platform call (`DoomPlatform::quit`) or a returned "quit requested" state that the
   host loop checks.
3. **Command line**: `m_argv` stores `Vec<std::ffi::CString>` and takes `&CStr`. Both
   types exist in `alloc::ffi` / `core::ffi`, so this is an import swap, but a plain
   `Vec<String>` would be simpler still. `doomgeneric_Create` builds the `CString`s.
4. **Mechanical `std` -> `core`/`alloc` swaps**: `std::rc::Rc` (9 uses, `m_config`,
   `w_wad`) -> `alloc::rc::Rc`; `std::borrow::Cow` (`fixed_cstr`) -> `alloc::borrow`;
   `std::mem::take` and `std::ptr::dangling_mut` -> `core::`; `Vec`, `String`, `Box`,
   `format!` and `vec!` need explicit `alloc` imports (there is no prelude for them).
5. **`sha1_smol`**: switch to `default-features = false` (it is `no_std` capable).
6. **Panics and allocation**: `I_Error` is a bare `panic!`, which is fine in `no_std`;
   the host binary supplies the panic handler and global allocator (on `x11` these are
   already `std`'s). Test-only code (`MemFileSystem`, `p_tick` tests) stays behind
   `#[cfg(test)]`, where `std` is available.
7. **Floating point**: the `f64::abs` in `v_video.rs` is in `core` on the current
   toolchain; check for any `f32`/`f64` method that is `std`-only (`sqrt`, `powi`, ...)
   when the switch is made. A grep today finds none.
8. **Enforcement**: flip `#![no_std]` on, add `extern crate alloc;`, and let the compiler
   list what is left. A CI grep for `std::` in `engine/src` outside `#[cfg(test)]`
   keeps it from regressing in the meantime.

Related open question: `platform.init` passes a `*mut pixel_t` framebuffer pointer
across the trait boundary. That is `no_std`-compatible but is the last raw pointer in
the engine's public surface; it could become a `&'static mut [pixel_t]` later.

## Later ideas (not part of either track)

- Config-file persistence (`M_LoadDefaults` / `M_SaveDefaults`) is stubbed out. If it is
  ever implemented, it should use `DoomFileSystem` (read whole file, write whole file)
  rather than adding new trait methods.
- `StdFileSystem` uses `std::os::unix::fs::{FileExt, DirBuilderExt}`, so the `fs` crate
  is Unix-only, as the engine was before. A Windows port would `cfg` those two calls.
- A second `DoomFileSystem` implementation (embedded WAD via `include_bytes!`, or a
  browser/wasm one) would exercise the trait boundary and is the natural first consumer
  of a `no_std` engine.
