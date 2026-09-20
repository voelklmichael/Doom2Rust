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

The whole-state functions are not stuck because of one bad edge: the call graph is entangled. Measured
on the merged tree (a by-name transitive closure of "who passes `state` to whom", plus direct module
use; callbacks are not even counted, so this is a lower bound):

- Only 46 of the 412 state-taking functions transitively need a single aggregate; 65 need two, 89
  three, 52 four, and 160 need five or more.
- The extra aggregates come from a handful of roots: `remove_mobj` stops the mobj's sound (`audio`, 56
  functions inherit it), `change_music` and `draw_patch` (`io`), `check_position`/`check_sight`/
  `damage_mobj` read `RMainState` (`render`), `kill_mobj` reaches the HUD (`ui`), and `spawn_mobj`/
  `set_psprite` read the `info` tables that live in `Assets`.
- 150 state-taking functions are callbacks (`StateAction`, weapon actions, menu routines, `p_map`
  traversals) whose `fn(&mut GameState, ..)` type pins them.
- `GameState::screen()`/`screen_mut()` spans three aggregates (io, ui, render).

### Simulated payoff of specific decouplings (current grouping)

| change | fns needing one aggregate | fns needing only `World` |
|---|---|---|
| today | 46 | 20 |
| sound calls removed from the graph (e.g. a `SoundRequest` queue) | 48 | 21 |
| ... and `info` tables moved into `World` | 49 | 22 |
| ... and `p_*` code not needing `RMainState` | 64 | 37 |
| ... and `change_music`/`draw_patch` cut too | 69 | 37 |

An earlier version of this document called sound "the linchpin" and proposed the queue as the way
forward. **That was wrong**: `s_start_sound` needs five aggregates, but removing it from the graph
frees only two functions, because they reach the other roots above anyway. A queue would change when
sounds start relative to other world code, and would need mirrored listener state to stay
bit-exact, for that return. It is not worth doing for narrowing.

A hill-climb over the module-to-aggregate assignment (minimising mean transitive exposure) only
reaches ~19 of 57 modules by lumping unrelated modules together (`p_map` with audio, `p_enemy` with
UI), so the grouping is not the problem either; regrouping by statistics would hide the coupling, not
remove it.

## Possible next steps (not done)

1. ~~Make what is really pure into free functions~~ **Done** (`point_to_angle2` is pure, `validcount`
   lives in `PSetupState`): world code no longer touches `RMainState`. Simulated +15 single-aggregate
   functions, actual payoff only 3 narrowed functions (404 -> 401), because the tool can only narrow
   a function that does not pass `state` on, and the callbacks and hubs still pin most of them. The
   "fns needing one aggregate" figures above are an upper bound on what the tool can free, not a
   forecast.
2. Consolidate the five screens into `VVideoState` (in `Io`), so drawing helpers take
   `&mut VVideoState` instead of the whole state.
3. Re-run `narrow_state.py` after each; leaf functions unlock in waves.

Do not expect the whole-state count to fall much below ~350 without changing what a "tic" is
(e.g. one explicit context struct); it is a global-state program at heart.

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
