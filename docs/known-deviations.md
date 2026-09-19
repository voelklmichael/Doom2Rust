# Known Deviations from Upstream doomgeneric

## Context

`rust-doomgeneric` is a c2rust transpile of doomgeneric, then converted phase by phase
toward idiomatic Rust (see `track16-gamestate-plan.md`, `track17-arena-indices-idea.md`).
Almost all of that work is behavior-preserving: same logic, different representation.
This document tracks the exceptions — places where a conversion phase deliberately
changed *observable runtime behavior*, not just internal structure, usually because
reproducing the original C behavior exactly would have meant fighting the type system
or threading global state through code that has no real need for it.

Each entry should say what changed, why, and what was explicitly given up, so nobody
mistakes a deviation for a bug or tries to "fix" it back to C parity without knowing
the tradeoff was intentional.

## `I_Error`: replaced with a bare Rust panic (2026-09-11)

**What changed**: `I_Error(message)` used to reimplement C's fatal-error path by hand —
print to `stderr`, run every `atexit`-registered handler flagged `run_on_error`, show a
Zenity GUI error dialog on Linux desktops (unless `-nogui`), guard against recursive
calls via an `already_quitting` flag, then `exit(-1)`. It now does exactly this:

```rust
pub unsafe fn I_Error(message: &str) -> ! {
    panic!("{}", message);
}
```

All ~110 call sites across the codebase are untouched — every one of them just calls
`I_Error(&format!(...))` or `I_Error("literal")` exactly as before.

**Why**: `I_Error` was the last of the codebase's three whole-codebase-fanout utility
hubs left over from the `GameState` bridge-collapse track (see
`track16-gamestate-plan.md`). Converting it to thread a real `&mut GameState` parameter
like its two
siblings (`S_StartSound`, `W_CacheLumpNum`/`W_CacheLumpName`) turned out to be much
more tractable than earlier tracked (82% of call sites already had `state` in scope
after those two phases), but the remaining handful required either widening more leaf
functions or accepting a real structural blocker: `G_CheckDemoStatus` is registered via
`I_AtExit` as a raw C-ABI `unsafe extern "C" fn() -> ()`, which cannot carry a `&mut
GameState` argument without breaking that callback contract. Rather than solve that,
the user asked whether `I_Error` could just become a panic instead — which sidesteps
the whole problem, since `panic!` doesn't need any state at all.

**What was given up**:
- The 3 `run_on_error`-flagged `atexit` cleanup handlers (`D_QuitNetGame` — network
  disconnect notice, `S_Shutdown` — audio device teardown, `StatDump` — crash-time
  stats dump) no longer run when `I_Error` fires. Normal quit (`I_Quit`, unrelated
  code path) still runs all registered handlers unconditionally — only the
  crash/error path lost this.
- The Zenity GUI error popup (and the `-nogui` flag that suppressed it) is gone. A
  crash now only prints Rust's standard panic message to stderr.
- The recursive-`I_Error`-call warning guard is gone (a panic inside a panic's unwind
  path aborts the process instead, which is arguably a reasonable outcome on its own).

**Removed as a result** (dead once `I_Error`'s body no longer needs them):
`ZenityErrorBox`/`EscapeShellString`/`ZenityAvailable`/`ZENITY_BINARY`,
`ISystemState::already_quitting`, `ISystemState::zenity_errorboxpath_size`, and a
handful of now-unreachable `return`/assignment statements immediately after
`I_Error(...)` calls in `g_game.rs`/`m_misc.rs`/`p_enemy.rs` (the compiler's own
`unreachable_code` lint caught these once `I_Error`'s return type became `!`).

## `player_t.message`: savegame field no longer round-trips a raw pointer (2026-09-11)

**What changed**: `player_t.message` (the pending-HUD-message slot — "Picked up a
clip.", "game saved.", cheat feedback, etc.) was `*mut c_char`, `NULL` meaning "no
message pending." Track 18 converted it to `Option<String>`. The field is also part
of the `.dsg` savegame binary format: `saveg_write_player_t`/`saveg_read_player_t`
write/read it as a raw 4-byte slot (`saveg_writep`/`saveg_readp`, the same
pointer-sized-placeholder mechanism used for several other pointer fields that get
properly relinked after load via a separate fixup pass). `message` was never one of
the fields with a real fixup pass — chocolate-doom's original C saved literally
whatever the in-process pointer bit pattern happened to be at save time, and reading
it back in a *different* process (after a load) reconstructs a bit pattern with no
relationship to any valid memory in the new process. Nothing has ever safely
dereferenced a loaded `.message` value: `P_UnArchivePlayers` (the only caller of
`saveg_read_player_t` for live players) unconditionally overwrites it to `NULL`
immediately afterward, every time. The field's 4 bytes in the file have therefore
always been meaningless padding in practice, not restorable content.

The new code preserves the file's byte *layout* exactly (still reads and writes
exactly 4 bytes for this field, so `.dsg` file size and every other field's offset is
unchanged) but writes a fixed placeholder (`0` for `None`, `1` for `Some`) instead of
a real pointer bit pattern, and unconditionally sets `message = None` on read.

**Why**: `Option<String>` cannot represent an arbitrary saved pointer value, and
there was never a real value to preserve in the first place — `P_UnArchivePlayers`
discarding it immediately confirms the original design already treated this slot as
disposable. Reconstructing "faithful" garbage would add complexity for a value that
was never observable.

**What was given up**: nothing observable. The one theoretical difference: the
original code, had some future caller ever read `.message` right after a load
without the existing unconditional reset, would have dereferenced a wild pointer
(undefined behavior, likely a crash). The new code can't do that — a loaded message
is always safely `None`. Strictly safer, not a behavior loss for any code path that
currently exists.

## `SetVariable` (m_config.rs) string duplication: intentional one-time leak (2026-09-11)

**What it does**: when a config file line sets a `DEFAULT_STRING`-typed variable
(e.g. `back_flat`, `savedir`, `video_driver`), `SetVariable` duplicates the parsed
value and stores the new pointer into the variable's bound location:
`CStr::from_ptr(value).to_owned().into_raw()`. This replaced a raw libc
`strdup(value)` call — same shape (heap-allocate a nul-terminated copy, hand back
an owned `*mut c_char`), just Rust's allocator instead of libc's.

**Why this is safe to leak**: `into_raw()` (like the `strdup` it replaced) forgets
the allocation — nothing ever calls `CString::from_raw` on these pointers to
reclaim and drop them, and nothing in the codebase calls `free()` on them either
(the only `free(` call anywhere is `w_wad.rs`'s unrelated `lumpinfo` cleanup).
Every config variable overwrite therefore leaks its previous string value for the
life of the process. This has always been true of the original C (`strdup` without
a matching `free` on reload), so it's not a new deviation — noted here so the
`into_raw()` call isn't mistaken for a bug (a "leaked memory" clippy/reviewer flag)
or "fixed" by adding a `from_raw`/drop that would double-free or free a
libc-vs-Rust-allocator-mismatched pointer.

## `EV_VerticalDoor`: dropped a byte-layout-coincidence fallback when retriggering an active mover (2026-09-12)

**What changed**: `sector_t.specialdata` was `*mut c_void`, and `EV_VerticalDoor`
(the handler for a player re-pressing/re-walking-into a door-type line while some
mover is already running on that sector) unconditionally reinterpreted it as `*mut
vldoor_t` *before* checking what it actually pointed at:

```c
door = sec->specialdata;
if (door->direction == -1) { door->direction = 1; }   // read/write through the WRONG type if door isn't really a vldoor_t
else { /* real type check via thinker->function, dispatch correctly */ }
```

The initial `door->direction == -1` check only reads the right field when the
active mover genuinely is a door. If it's actually a `plat_t`/`ceiling_t`/
`floormove_t`, this reads/writes whatever field happens to sit at the same byte
offset as `vldoor_t.direction` in that other struct's layout — a real (if
long-standing, vanilla-Doom-inherited) type-punning shortcut, and one that had
already been silently altered by an earlier track: `sector: SectorId` (a 4-byte
newtype) replaced the original `sector_t*` (an 8-byte pointer) in all four mover
structs, shifting every field after it — so this offset "coincidence" was already
producing different results than upstream well before this phase touched it.

This phase converted `specialdata` to `Option<SectorSpecial>` (a real enum:
`Door`/`Ceiling`/`Floor`/`Plat`, each holding its own typed pointer), which makes
the speculative wrong-type read impossible to express — `match`ing the enum tells
you the real type before you can touch any field. The rewrite now checks the real
type *first* in all cases, including the door-reopen check, then acts on that
type's own real field.

**Why**: the pre-check was undefined behavior in Rust regardless of layout
(reading a `*mut vldoor_t` when the pointee is actually a different type violates
Rust's aliasing rules even when the bytes happen to line up), and the "coincidence"
it depended on had already been broken by the `SectorId` shrink — so there was no
still-working behavior left to faithfully preserve, only a subtly-already-wrong one
to knowingly replace with a type-safe equivalent.

**What was given up**: the specific case of re-triggering a door-type line while an
active **Plat** is running on that sector previously reopened the "door" (in
practice, wrote into whatever plat field the coincidence landed on, likely
`plat_t.wait`) *without* requiring a player-initiated activation, before falling
through to the real Plat dispatch (`plat.wait = -1`) on the *next* retrigger. The
new code always requires a player activation and always dispatches directly to
`plat.wait = -1` for a real Plat, `ceiling.direction = -1` for a Ceiling, and
`floor.direction = -1` for a Floor (matching the pre-existing, always-type-correct
`eprintln!("...wasn't a door.")` fallback branch's intent) — same practical
end state for the common case (something ends up set to `-1`), but reached through
one consistent, type-checked path instead of two different ones depending on
whether the coincidental pre-check happened to trip first.

## `z_zone.rs`: the zone allocator no longer purges `PU_CACHE` blocks under memory pressure (2026-09-13)

**What changed**: Doom's zone allocator (`Z_Malloc`/`Z_Free`/`Z_FreeTags`/`Z_ChangeTag`)
managed one fixed-size arena (`memzone_t`, ~600KB by default, `-mb` to resize) as a
single hand-rolled coalescing free-list: every allocation lived inside that one block,
addressed via manual pointer arithmetic and an intrusive `memblock_s.next/prev` chain.
Critically, `Z_Malloc` **actively evicted** other allocations when it couldn't find enough
contiguous free space: it walked the free list looking for any block tagged
`PU_CACHE`/`>= PU_PURGELEVEL` and force-freed it (via `Z_Free`, which also null'd out that
block's owner through the `user: *mut *mut c_void` back-pointer) to make room. This is how
vanilla Doom kept its WAD lump cache (`lumpinfo_s.cache`, populated by
`W_CacheLumpNum`) bounded on 90s-era hardware — texture/sprite/sound data silently got
evicted and transparently reloaded from the WAD file the next time it was needed.

The allocator now uses `std::alloc::{alloc, dealloc}` for each allocation individually
(still with a small header — id/tag/user/`Layout` — placed immediately before the
returned pointer, same "hidden header" trick as before, just no longer threaded into one
shared arena), plus a `Vec<*mut BlockHeader>` registry used only for `Z_FreeTags`'
bulk-free-by-tag-range operation (called once per level load, to free the previous
level's thinkers/movers in one shot) and a basic heap sanity check. Every external
function signature is unchanged — all ~50 `Z_Malloc`/`Z_Free` call sites across the
codebase needed no changes at all.

**Why**: the fixed-size arena and its eviction dance exist purely to survive an
artificially small memory budget that has no reason to exist in a modern process — the
game's total dynamic allocation across a level is a tiny fraction of what a contemporary
machine has to spare. `std::alloc` is fast, well-tested, and doesn't fragment the way a
bump-and-coalesce free list does under Doom's allocation pattern (many small
same-size mover/mobj structs coming and going).

**What was given up**: `Z_Malloc` no longer tries to evict anything — if the system
allocator itself returns null (genuine OOM), it calls `I_Error` immediately instead of
first searching for something purgeable to reclaim. In practice this means WAD lumps
cached via `W_CacheLumpNum` are never silently evicted and reloaded once loaded, and (at
the time of this phase) any existing `Z_ChangeTag2(ptr, PU_CACHE, ...)` calls
(`W_ReleaseLumpNum`, `r_data.rs`'s patch composite step) still recorded `PU_CACHE` as a
tag on the block's header, but tagging something `PU_CACHE` was already inert — it
changed no observable behavior, since nothing ever scans for purgeable blocks anymore.
Also removed as dead code in the same phase (all had zero callers): `Z_ClearZone`,
`Z_DumpHeap`, `Z_FileDumpHeap`, `Z_FreeMemory`, `Z_ZoneSize`, and the
`memzone_t`/`memblock_s` structs themselves. `I_ZoneBase`, `AutoAllocMemory`, and the
`-mb` command-line option in `i_system.rs` are now vestigial (nothing calls
`I_ZoneBase` anymore) but were left in place rather than bundled into this phase's diff.

**Follow-up (2026-09-13, same day)**: since the tag/purge bookkeeping above was already
fully inert, a later phase took it a step further for the WAD lump cache specifically —
`lumpinfo_s.cache` changed from a zone-allocated `*mut c_void` to a plain owned
`Option<Box<[u8]>>`, so `W_CacheLumpNum`/`W_ReleaseLumpNum` no longer call into the zone
allocator for lump data at all (the `mmap`'d-file fast path, and every downstream
caller's return-pointer usage, are unchanged). `W_ReleaseLumpNum`'s `Z_ChangeTag2` call
was accordingly deleted (it had nothing left to do), and `Z_ChangeUser` — whose only real
caller was this same cache's regrow-fixup path — was deleted from `z_zone.rs` entirely as
dead code.

**Follow-up 2 (2026-09-13, same day)**: `r_data.rs`'s texture composite cache
(`RDataState.texturecomposite`) got the same treatment — `Vec<*mut byte>` changed to
`Vec<Option<Box<[u8]>>>`, so `R_GenerateComposite` builds the composite in a local `Vec<u8>`
and stores it once fully drawn instead of handing `Z_Malloc` a `user` back-pointer into the
slot up front. This was the last real caller of `Z_ChangeTag2` project-wide, so it was
deleted from `z_zone.rs` entirely as dead code too. `R_GetColumn`'s rendering-path read of
this cache is unchanged in effect (same bytes, same offsets) — screenshot-diffed against
unmodified `main` to confirm, given this feeds pixel data directly to the screen. No further
observable behavior change beyond what this deviation already covers.

## Resolved: `snd_musiccmd`/`chatmacro*` config bindings (2026-09-18)

An earlier version of this document recorded a dormant bug where `snd_musiccmd` and
`chatmacro*` were bound into the config system through an untyped pointer whose
`&'static str` (16-byte) representation didn't match the 8-byte write `SetVariable`
performed. That mechanism no longer exists: `M_BindVariable_int/_f32/_string` now take a
typed accessor closure (`|s| &mut s.i_sound.snd_musiccmd`), `DefaultLocation` stores it
as an `Rc<dyn Fn(&mut GameState) -> &mut T>`, and `SetVariable`/`M_Get*Variable` go
through the accessor with the whole `GameState`. There is no raw pointer, and no size
mismatch, left to corrupt.

## `unsafe` in the engine crate (2026-09-18)

Every file in `engine/src` is `unsafe`-free (`#![deny(unsafe_code)]` in `lib.rs`). The two
exceptions originally recorded here are gone: the hand-written `sha1.rs` was replaced by the
`sha1_smol` crate (`w_checksum.rs` calls it directly), and `mem_compat.rs` — whose only
remaining caller was `sha1.rs` — was deleted (PR #469).

The `x11` crate is outside this policy.

## Intermission: no filler graphic for map numbers beyond `NUMCMAPS`

`WI_drawLF` (intermission "finished" header) used to draw, for a level number
greater than the episode's map count, a stack-built fake `patch_t` with all-zero
column offsets — upstream doomgeneric/Chocolate Doom's "MAP33+ filler", whose pixels
were whatever bytes followed that struct in memory (undefined behavior). The Rust
port draws nothing there. It cannot occur with any stock IWAD (only a PWAD that
warps to a map above the episode's last would reach it).

## Wall texture column: `finetangent[]` index clamped (2026-09-19)

`R_RenderSegLoop` computes `angle = (rw_centerangle + xtoviewangle[x]) >> ANGLETOFINESHIFT`
and indexes `finetangent[4096]` with it. For a column at a clipped seg edge the angle can
fall just outside the front half-plane (e.g. 8188), which upstream doomgeneric silently
reads past the end of the array (undefined behavior; whatever the linker placed next). The
Rust port panicked with an out-of-bounds index during normal play on `DOOM.WAD`. The index
is now clamped to `finetangent.len() - 1`, as PrBoom does. Only the texture column of that
single edge pixel column can differ, and in-range angles are unaffected.

## Savegames: built and read in memory, written in one go (2026-09-19)

`G_DoSaveGame` / `G_DoLoadGame` used to stream the save file byte by byte through an open
`std::fs::File` (`PSavegState.save_stream`). The engine no longer touches the filesystem
directly (see `fs-crate-and-no-std-plan.md`), so a savegame is now a `Vec<u8>`
(`PSavegState.save_buffer`) that is read from disk in full before a load and handed to
`DoomFileSystem::write_file` in one call after a save. The on-disk format is unchanged: a
save written by the new code is byte-identical to one from the streaming code, apart from
the raw thinker-function pointer words that `saveg_writep` has always serialized (they vary
from run to run even on the old code).

Observable differences:

- A save-file open failure is now detected when the finished image is written, not before
  serialization starts. The recovery-file fallback (`<temp>/recovery.dsg`) and both error
  messages are unchanged.
- The "Savegame buffer overrun" check (`vanilla_savegame_limit`) now happens before anything
  is written, so an over-limit save leaves no partial `temp.dsg` behind.
- A write error can no longer surface halfway through serialization (`saveg_write8` cannot
  fail), so the `saveg_write8: Error while writing save game` message is gone.

## Savegames: pointer words are a 0/1 presence flag (2026-09-19)

Vanilla serialized raw pointers in a few slots (`player_t` placeholder pointers, `message`,
and the thinker function pointer of every thinker). Loading only ever tests them for
null/non-null, so the engine now writes `0` or `1` instead of a process address
(`saveg_write_present`). Saves written by the engine are therefore byte-for-byte
reproducible (previously the thinker-function words varied from run to run), and saves from
vanilla or from older builds still load, because any non-zero word reads as "present".
What is given up: a save no longer carries the (meaningless) original pointer values, so
byte-comparing a save against one written by the C program will differ in those words,
exactly as it already did between two runs of the C program.

## Sound effects: a built-in mixer instead of SDL_mixer (2026-09-19)

Upstream doomgeneric only has sound in its SDL/Allegro ports (`i_sdlsound.c`,
`i_allegrosound.c`, built with `FEATURE_SOUND`); the X11 port is silent. The engine now
mixes sound effects itself (`sfx_mixer.rs`) and hands stereo PCM to the platform through
`DoomPlatform::audio_open` / `audio_frames_wanted` / `audio_write`. The mixer follows
`i_sdlsound.c`: nearest-neighbour rate conversion, the `b | b << 8` 8-to-16-bit widening,
pan `left = (254 - sep) * vol / 127`, `right = sep * vol / 127`, saturating overlap.

Differences from the SDL backend:

- A sound whose `ds*` lump is missing from the WAD is silent. Vanilla stops with
  `W_GetNumForName: ... not found!`.
- Sounds are loaded when first played, not at start-up (`I_PrecacheSounds` is gone), so the
  first play of an effect reads its lump then. The lump is shared with the lump cache, not
  copied and expanded to the output rate.
- `use_libsamplerate` / `libsamplerate_scale` stay unused; there is no libsamplerate path.
- Music is not implemented yet (`MusicModule` has no backend), so `S_ChangeMusic` still
  does nothing audible.
- A platform that does not implement `audio_open` (the default) plays nothing, and the
  engine then behaves exactly as before: no lumps are looked up for sound.
- The Linux build sends its audio to `aplay` or `paplay` over a pipe (`DOOM_AUDIO=off` or
  `DOOM_AUDIO=file:PATH` override it).
