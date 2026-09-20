# Decomposing `GameState`

## Where things stand (2026-09-20)

`GameState` (`engine/src/game_state.rs`) owns seven domain aggregates plus nothing else:

| aggregate | holds |
|---|---|
| `World` | the simulation: p_setup, p_mobj, p_tick, p_spec, p_map, p_maputl, p_sight, p_ceilng/doors/lights/plats/switch, p_enemy, p_pspr, p_user, p_saveg, m_random |
| `Render` | r_main, r_segs, r_draw, r_data, r_plane, r_bsp, r_things, r_sky |
| `Ui` | m_menu, hu_stuff, st_lib, st_stuff, wi_stuff, am_map, f_finale, f_wipe, statdump |
| `Audio` | s_sound, i_sound, sounds |
| `Assets` | w_wad, w_checksum, info, and the boxed `DoomFileSystem` |
| `Game` | g_game, doomstat, d_main, d_loop, d_event, d_iwad, m_argv, m_config, m_controls |
| `Io` | the boxed `DoomPlatform`, i_input, i_joystick, i_system, i_timer, i_video, v_video |

A use is a path `state.<aggregate>.<module>`. The grouping was chosen by measuring, over every
function that took `&mut GameState`, how many touch exactly one aggregate; the alternatives tried
(merging io into game, render into world, moving v_video or audio) scored within a few functions
of it.

A function only needs `&mut GameState` if it needs *several* aggregates at once or passes the
whole state to something that does. Functions taking the whole state:

| | count |
|---|---|
| original flat `GameState` (before the 2026-09-19 narrowing) | 631 |
| after the 2026-09-19 narrowing pass | 455 |
| after aggregates + the tools below (this change) | 404 |

## Tools (all in `tools/`, run from `rust-doomgeneric/`)

- `narrow_state.py` - gives a function that never uses `state` as a whole one parameter per
  module it touches (<= 3), or its aggregate (`world: &mut World`) when it touches more modules of
  one aggregate; rewrites every caller; skips callbacks, duplicate names and anything that would
  exceed clippy's 7-argument limit. Handles the boxed `fs`/`platform` fields (`&dyn
  DoomFileSystem`, `&mut dyn DoomPlatform`). Run to a fixpoint: narrowing a leaf makes its callers
  narrower.
- `tighten_mut.py` - applies clippy's `needless_pass_by_ref_mut` suggestions (which `cargo clippy
  --fix` does not) and the matching call-site `&mut` -> `&`, to a fixpoint. Run after
  `narrow_state.py`.
- `regroup_state.py` - the one-off `state.<module>` -> `state.<aggregate>.<module>` rewrite; keeps
  the membership table, so a different grouping is a table edit plus a re-run.
- Then `cargo fmt`, fix what the compiler reports (usually a call argument that reads a field the
  same call borrows mutably: hoist it into a local), and `cargo test --release`.

## Why it stops at ~400

Measured on the stack tip (a by-name transitive closure of "who passes `state` to whom", plus
direct aggregate use):

- Only 46 of the remaining state-taking functions transitively need a single aggregate; 65 need
  two, 89 need three, 52 need four, and 160 need five or more.
- **Sound is the linchpin.** `s_start_sound` needs assets, audio, game, render and world at once
  (origin position, listener, `gamemap`, `point_to_angle2`, lump loading, the platform mixer), and
  it is called from ~155 sites in door/floor/plat/enemy/switch code that otherwise touch only
  `World`. Every one of those inherits its 5-aggregate need. A "context struct" bundling what it
  reads would be the whole `GameState` again, so the bundle is not a way out.
- **Callbacks pin the whole state.** 150 state-taking functions are used as values
  (`StateAction`, weapon actions, menu routines, `p_map` traversal callbacks); `narrow_state.py`
  cannot change a function whose type is fixed by a `fn(&mut GameState, ...)` alias. 39 of them
  touch only `World` directly, but all of those reach `s_start_sound`.
- `GameState::screen()`/`screen_mut()` spans three aggregates (io, ui, render): the five C
  `screens[]` live in `i_video`, `st_stuff` and `r_draw`, so every drawing helper needs all three.

## Possible next steps (not done)

1. **Decouple sound from the simulation.** Have simulation code push a `SoundRequest { origin,
   sfx, resolved volume/separation }` onto a queue owned by `World`, drained into `Audio` by the
   game loop. That removes `Audio`/`Assets`/`Io`/`Render` from every mover and monster function
   and would let the ~39 world-only callbacks retype to `fn(&mut World, ..)`. It changes *when* a
   sound starts relative to other world code, so it needs an audio-trace golden first (record
   `(origin, sfx, volume, sep, channel)` for every `s_start_sound` over demo1-3 against the old
   code).
2. **Consolidate the five screens** into `VVideoState` (in `Io`), so drawing helpers take
   `&mut VVideoState` instead of the whole state.
3. Only then re-run `narrow_state.py`; each step unlocks a new wave of leaf functions.

## A performance trap found along the way

Grouping the fields changed no instruction, but `-timedemo demo1` got 5-8% slower (identical
instruction counts, identical L1 misses): `i_video::finish_update`, 45% of all cycles, read its
palette from an inline table whose address moved with the regroup. Padding the old `GameState` did
not reproduce it, so it is specific to relative field placement. Packing the palette once per
frame (one table load per pixel) removed the dependence and made the demo 19% faster than before
(6.36G -> 5.15G cycles). Lesson: after moving state around, re-run the timedemo (recipe in the
perf notes: `perf stat -e cycles:u,instructions:u`, `taskset -c 4`, `Doom1.WAD`, `-nosound`) and
compare *cycles against instructions* - equal instructions with more cycles means a layout effect,
not a code change.
