# Decomposing `GameState`

## Where things stand (2026-09-19)

`GameState` (`engine/src/game_state.rs`) bundles 57 per-module state structs
plus the boxed platform and filesystem. It replaced the C globals one module at
a time (see `track16-gamestate-plan.md`), and almost every function started out
taking `state: &mut GameState`, which hides what it really needs and forces
borrow-checker workarounds (copy fields into locals before calling anything).

A function only needs `&mut GameState` if it needs *several* subsystems at once
or passes the whole state to something that does. Measured on the engine:

| | functions taking the whole `GameState` |
|---|---|
| before the narrowing pass | 631 |
| after (this change) | 453 |

178 functions now take only the subsystems they use, for example
`activate_in_stasis(p_plats: &mut PPlatsState, p_tick: &PTickState, tag: i32)`
or `point_to_angle(r_main: &RMainState, ...)`. `m_controls`, `m_argv`, `r_sky`,
`i_joystick` and `m_random` no longer mention `GameState` at all.

## How it was done

`tools/narrow_state.py` (run from `rust-doomgeneric/`) finds functions whose
body touches at most three `state.<field>` paths and never passes `state`
itself onwards, rewrites the signature to one parameter per field, and rewrites
every caller. It works bottom-up: narrowing a leaf makes its callers narrower,
so it is re-run until it finds nothing (single-field pass: 71, 15, 2
candidates; up-to-three-field pass: 70, 12). Then `cargo clippy --fix` tightens
parameters that turned out to be read-only from `&mut` to `&`, and the
compiler's remaining complaints are fixed by hand. In this pass those were:
a call argument that read a field the same call borrows mutably (hoist it into
a local, 10 sites), and a parameter named `m_random` shadowing the function
`m_random`.

## What is left, and why the script stops

The 453 remaining functions mostly *pass `state` on*, and a few hubs are called
from everywhere and genuinely need several subsystems:

| hub | call sites | needs |
|---|---|---|
| `s_start_sound` | 155 | sound state, mobj positions, platform, WAD |
| `saveg_write32` / `saveg_read32` | 124 / 121 | already narrow; callers are the save-game readers/writers |
| `cache_patch_num` / `draw_patch` | 58 / 57 | WAD + lump cache + video buffers + filesystem |
| `change_switch_texture` | 53 | level geometry, switch table, sound |
| `set_mobj_state`, `do_floor`, `do_door` | 33 / 44 / 31 | level, mobjs, thinkers, RNG, sound |

Going further means giving those a *context* instead of the whole state.
Suggested order, each step verifiable with the golden tests:

1. **Group the 57 structs into a few domain aggregates** owned by `GameState`:
   for example `world` (p_setup, p_mobj, p_tick, p_spec, p_map, p_maputl,
   p_sight, the sector-special modules), `render` (r_*), `ui` (m_menu, hu_*,
   st_*, wi_*, am_map, f_*), `audio` (s_sound, i_sound, sounds) and `sys`
   (platform, fs, d_loop, i_*). Functions then take `&mut World` and so on.
2. **Give `s_start_sound` an `Audio` plus a `Listener`/position argument**
   instead of the whole state; that alone frees ~155 call sites.
3. **Split the WAD and lump cache from the rest of `w_wad`** so patch and lump
   access takes `&Wad` instead of `&mut GameState` (58 + 57 sites).
4. Only then re-run `tools/narrow_state.py`; each of the steps above unlocks a
   new wave of leaf functions.

Verification for every step is the same: `cargo test --release` (the golden
simulation/frame/save tests must not change), clippy, and `cargo fmt --check`.
