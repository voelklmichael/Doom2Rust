# Filesystem crate extraction, and the path to `#![no_std]` + `alloc`

## Goal

The `engine` crate (`rust_doomgeneric`) builds as `#![no_std]` with `extern crate alloc`.
Everything that needs an operating system lives in the host crates, behind two traits:

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

Status: all four phases landed (main at 62dfccd). `engine/src` no longer contains `std::fs` or
`std::io`; since Track B the compiler enforces that (`#![no_std]`).

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

## Track B: `no_std` + `alloc` (DONE)

The engine crate is `#![no_std]` with `extern crate alloc`. Everything the engine needs
from the OS goes through the two traits above. What it took:

1. **Console output**: `DoomPlatform` gained `print` / `eprint`. The engine's ~85
   `println!`/`print!`/`eprintln!`/`eprint!` sites became `doom_println!` etc.
   (`engine/src/console.rs`), which format into a `String` and call the platform. Helpers
   that had no `state` in scope (`I_PrintBanner`, `M_ExtractFileBase`, `I_GetPaletteIndex`,
   `M_SetConfigDir`, ...) take a `&mut dyn DoomPlatform`. `saveg_read8` (narrow
   `PSavegState`) only sets its error flag now, and `G_DoLoadGame` prints the message.
2. **Process exit**: `D_Endoom` calls `DoomPlatform::quit()` (`-> !`) instead of
   `std::process::exit`.
3. **Command line**: `myargv` is `Vec<String>` (was `Vec<CString>`); `M_ArgvAtoi` takes `&str`.
4. **`std` -> `core`/`alloc`**: `Rc`, `Cow`, `mem::take`, `ptr::dangling_mut`; `Vec`, `String`,
   `Box`, `ToString`, `ToOwned` are imported from `alloc` where used (`vec!`/`format!` come
   from `#[macro_use] extern crate alloc`).
5. **`sha1_smol`**: nothing to do; it is `no_std` with no default features.
6. **Panics and allocation**: `I_Error` is a bare `panic!`, which is fine in `no_std`. The host
   binary supplies the panic handler and global allocator (on `x11` they are `std`'s).
7. **Floating point**: the few `f64` operations in the engine (`abs`) are in `core`.
8. **Tests** link `std` (`#[cfg(test)] extern crate std;`) for `MemFileSystem`'s `BTreeMap` and
   nothing else; non-test code cannot name `std`, so the compiler now enforces what the
   Track A source-scanning test used to.

Deviations from vanilla output: none on stdout; `PrintDehackedBanners` (dead code whose
condition compared a string to itself) was deleted, and the "Unexpected end of file" savegame
diagnostic is printed once when a load finishes rather than at the first bad read.

Not verified: the crate has only been compiled for the host target (`x86_64-unknown-linux-gnu`,
where `#![no_std]` is still enforced by the compiler). Building it for a bare-metal or wasm
target would additionally need a global allocator and panic handler in that target's binary.

## Later ideas (not part of either track)

- `platform.init` passes a `*mut pixel_t` framebuffer pointer across the trait boundary. That
  is `no_std`-compatible but is the last raw pointer in the engine's public surface; it could
  become a `&'static mut [pixel_t]` later.
- Config-file persistence (`M_LoadDefaults` / `M_SaveDefaults`) is stubbed out. If it is
  ever implemented, it should use `DoomFileSystem` (read whole file, write whole file)
  rather than adding new trait methods.
- `StdFileSystem` uses `std::os::unix::fs::{FileExt, DirBuilderExt}`, so the `fs` crate
  is Unix-only, as the engine was before. A Windows port would `cfg` those two calls.
- A second `DoomFileSystem` implementation (embedded WAD via `include_bytes!`, or a
  browser/wasm one) would exercise the trait boundary and is the natural first consumer
  of a `no_std` engine.
