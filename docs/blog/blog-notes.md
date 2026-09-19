# Doom in Rust: numbers and story for the blog post

Source material, not a finished post. Every number below was measured on 2026-09-19 from
the git history (`rust-doomgeneric`, `doom_v4`, `doom_v3`) and from the Claude Code session
transcripts in `~/.claude/projects/`. I do not remember the earlier sessions; this is
reconstructed from those records and from my memory notes. Method notes are at the end.

Chart: `loc_and_unsafe_over_time.png`. Raw per-commit data: `loc_unsafe_series.csv`.

## 1. Headline numbers

| | |
|---|---|
| Result | Doom (shareware, E1) runs from a Rust engine with **zero `unsafe`**, `#![deny(unsafe_code)]`, `#![no_std]` + `alloc` |
| Also runs on | an ESP32-S3 (M5Stack CoreS3 Lite), 2.9 fps at first bring-up, ~29 fps after tuning |
| Elapsed working time (final approach, v5) | **about 89 h** over 12 calendar days (Sep 5 to Sep 19); range 82 to 98 h depending on how idle gaps are counted |
| Elapsed working time, whole effort (measured) | **about 140 h** over 37 days (Aug 14 to Sep 19), including ~53 h in two approaches that were abandoned. Excludes the January to April attempts, for which there are no records |
| Commits / merged PRs | 765 commits, 358 merged PRs (highest PR number #501) |
| Rust churn | +378k / -320k lines of `.rs` over the history (net +58k) |
| Unsafe lines | 48,430 (c2rust output) -> **199** today (198 in the X11 window glue, 1 in firmware); **0 in the engine** |
| Safety net | 15 engine tests, incl. a **regression oracle** that hashes the whole simulation and rendered frames (section 7). Runs as `cargo test --release` in about 10 s |
| Speed vs. C | Was ~13 % slower than C on `-timedemo demo1`. Profiling found two per-frame copies (PR #502); now about 6.5 G user cycles vs 10 to 11 G for C, i.e. **faster than C on this benchmark** (section 5) |

## 2. How long did I work?

Measured as elapsed time in which a session (or one of its subagents) was active, counting a
gap of at most 10 minutes as continuous work. It is elapsed time, not effort: several
sessions and subagents often ran in parallel, and long stretches ran autonomously ("continue
until done") while you were away. You typed 432 prompts in v5, many of them just "continue".

| Approach | Dates | Active time | Notes |
|---|---|---|---|
| v3: hand-written C parser | Aug 14 to 18 | ~9 to 15 h (estimate) | No transcripts survive; estimated from commit timestamps. 103 commits |
| v4: hand-written transpiler + per-function codegen | Aug 18 to Sep 5 | **~41 h** | 295 commits, 41.7k lines of transpiler Rust. Never produced a running game |
| v5: c2rust, then refactor | Sep 5 to 19 | **~89 h** | 765 commits in this repo |

Small side experiments (`RDoom`, `doom_c99`, `doom_c99_rust`) exist as directories without git
history, so I cannot give times for them.

v5 by day (union of all sessions, 10-minute gaps):

| Date | Hours | What was going on |
|---|---|---|
| Sep 5 | 8.1 | c2rust, game running, strings/bool conversion starts |
| Sep 6 | 12.7 | type/constant dedup, bool, action enums; GameState (Track 16) begins |
| Sep 7 | 12.4 | GameState "bridge collapse" BC1 to ~BC65 |
| Sep 8 | 3.9 | bridge collapse continues |
| Sep 11 | 5.8 | `c_char` elimination, libc `FILE*` removal |
| Sep 12 | 10.3 | bridge collapse finished, DoomPlatform trait, callbacks, pointer fields |
| Sep 13 | 11.0 | enums, `Vec` buffers, zone allocator, mobj arena |
| Sep 14 | 6.0 | unsafe reduction, wad file, patch lump indices |
| Sep 15 | 2.8 | pointer fields pass 3 |
| Sep 17 | 2.1 | safe fn pointers |
| Sep 18 | 5.4 | crash fixes, the unsafe-elimination sprint |
| Sep 19 | 8.4 | fs crate, `no_std`, idiomatic review, ESP32 firmware |

Sep 9, 10 and 16 have no sessions.

### My own estimates

These are judgment, not measurement.

- **What a person would need for the same end state:** roughly 400 to 800 hours (about 3 to 5
  person-months). The work is 358 reviewable PRs, ~700k lines of churn, most of it mechanical
  but each needing verification. Low confidence; you know better what you would have done
  yourself. I would not claim a precise speed-up factor, only "an order of magnitude in
  elapsed time, if you accept that a person would probably have stopped short of zero unsafe".
- **What v4 would have needed:** on Sep 3 (after 34 automated rounds) you asked how many
  sessions to a working Doom. I said 4 to 8 more sessions just to finish the gameplay function
  bodies (168 of 210 done), and that a running game would be "many times" that, likely dozens
  of sessions, because the renderer, WAD/texture layer, sound, menus, HUD and main loop were
  untouched. The c2rust route had a running game 14 minutes after its first prompt
  (12:06 first prompt, 12:12 transpile committed, 12:20 "fix ... so the game runs").
- **What is left:** removing the unsafe in the X11 crate (needs a safe Xlib binding, e.g.
  `x11rb`; a few hours); ~225 `unwrap()` calls and ~90 `error()` panics that are really
  vanilla `I_Error`; the deliberately deferred items listed in the idiomatic-review notes.

## 3. Lines of code

### The original C

| Set | Files | Lines (`wc -l`) |
|---|---|---|
| `.c` files compiled in the X11 build | 81 | 55,913 |
| all `.c` in the tree (other platforms too) | 95 | 62,458 |
| all `.h` | 97 | 10,637 |
| compiled `.c` + all `.h`, **non-blank, non-comment** | | **~42,200** |

### The Rust codebase

Non-blank, non-comment lines (approximate counter, see method notes):

| Point in history | Rust lines | Comment |
|---|---|---|
| Sep 5, c2rust output (PR #1) | **145,160** | 83 files, literal translation |
| Sep 6 morning | 131,600 | after bool and casts work |
| Sep 6 afternoon | 117,000 | type dedup, extern-fn-pointer work |
| Sep 6 evening | ~60,000 | most of the c2rust boilerplate gone |
| Sep 12 | ~59,000 | |
| Today | **56,036** | engine 53,312 (83 files), X11 1,124, firmware + Wi-Fi/DHCP/sender crates 1,528, fs 72 |

The four biggest one-PR drops explain most of the 145k -> 60k fall: `extern-c-removal-phase3`
(-35.5k), `minor_event_references` (-21k), `type-dedup-phase10-stdint-types` (-14k),
`bool-phase19` (-13.5k). c2rust repeats every extern declaration and type alias in every file,
so this was mostly redundancy, not game logic.

Today's engine is larger than the C it came from (53k vs 42k non-blank lines), mainly because
rustfmt puts one item per line, plus 38 tests and the golden-test harness.

## 4. Unsafe lines

"Unsafe lines" = lines inside an `unsafe { }` block or the body of an `unsafe fn`.

| Point | Unsafe lines | Share of code | `unsafe fn` | `static mut` variables | raw-pointer tokens | `c_int`-style C types |
|---|---|---|---|---|---|---|
| c2rust output (Sep 5) | 48,430 | 33 % | ~1,400 | 985 | 10,456 | 52,321 |
| Sep 6 evening | 41,557 | 69 % | 1,142 | 851 | 5,543 | 2,905 |
| Sep 12 | 27,255 | 46 % | 888 | 0 | 2,832 | 614 |
| Sep 14 | 22,873 | 39 % | 715 | 0 | 1,667 | 239 |
| Sep 17 | 21,436 | 36 % | 543 | 0 | 1,233 | 186 |
| Sep 18 (after the sprint) | **198** | 0.4 % | 10 | 0 | 122 | 32 |
| Today | **199** | 0.36 % | 10 (all X11) | 0 | 110 (109 in X11) | 23 |

Things worth a sentence in the post:

- The share of unsafe code **went up** on Sep 6 (33 % -> 69 %) because the boilerplate was
  deleted and what was left was the real, entirely-unsafe game logic.
- It then fell in steps, each from a different technique: global state into one `GameState`
  struct (-8.8k lines in one Sep 12 commit that removed unnecessary unsafe blocks around statics), raw pointers to
  typed arena indices, Rust `Vec`s instead of the zone allocator, and finally a one-day sprint
  (Sep 18, PRs #460 to #469) that took ~21k unsafe lines to 198.
- `as` casts: ~50k on Sep 6 -> 5.3k now.

## 5. Other numbers I measured (or could)

Measured:

- **Performance:** `-timedemo demo1`, 5,026 gametics (identical in every build), shareware
  `Doom1.WAD`, Xvfb, `perf stat` counting user-space cycles, pinned to one core. Two sessions
  measured independently and agree:

  | Build | User cycles | Instructions | L1 load misses |
  |---|---|---|---|
  | Rust before #502 | 12.5 to 13.0 G | 29.5 to 31.3 G | 0.51 to 0.53 G |
  | Rust after #502 | 6.4 to 6.8 G | 19.4 G | 0.13 G |
  | C (GCC 13 `-O2`) | 9.6 to 11.1 G | 30.1 G | 0.14 G |

  The first measurement (wall clock, `realtics`) said Rust was ~13 % slower than C. Profiling
  showed why: `i_video::finish_update` allocated and zeroed a fresh ~1 MB `Vec` per frame,
  converted into it, repacked it into the screen buffer, and the X11 platform then copied it
  once more. The C code converts straight into the screen buffer. PR #502 removed both copies;
  the frame conversion itself was never the problem. It only shows in an uncapped timedemo:
  in real play at 35 fps the copies cost about 0.1 ms per frame. The framebuffer golden hashes
  passed unchanged, which is what showed the change did not alter a pixel (section 7).
  Noise is large on this machine: the same C binary measured 9.6 G and 15.6 G cycles at an
  identical instruction count when other work was running, so use the best of several runs.
  Not explained by `-C target-cpu=native` or fat LTO (no reliable gain). Still open: the other
  session's profile has `draw_column` ~30 % slower than C's `R_DrawColumn` (not re-checked).
- **Binary size** (`size`, text section): C `-O2` 409 KB, Rust 1,266 KB (Rust statically
  includes std; the C binary links libc and X11 dynamically). Not a fair comparison; mention
  only with that caveat.
- **Correctness safety net:** 15 engine tests (38 in the whole workspace), 4 of them driven by a
  real WAD: golden hashes of the simulation state of demo1 to demo3, frame hashes of a scripted
  tour, save/load round trips and a sweep of every line special. Every one of the 11
  idiomatic-review PRs kept the goldens byte-identical. See section 7.
- **Effort proxies:** 25,822 assistant turns and ~25,000 tool calls (v5, including 24
  subagent transcripts); 13.5 M output tokens; 10.7 B cache-read tokens (that is mostly the
  same context being re-read, not text I wrote). v4 for comparison: 12.0k turns, 6.6 M output
  tokens.
- **Renames and mechanical passes:** ~2,600 identifiers renamed to Rust style, ~9.3k literal
  suffixes removed, 191 `while` loops to `for`, 178 functions narrowed off `&mut GameState`.
- **Hardware:** ESP32-S3 2.9 -> 8.5 -> 10.7 -> 20 -> 28 -> 29 fps through five measured steps
  (skip the 512 KB scaled-frame build, hoist per-column lookups, own SPI+DMA driver on the
  second core, PSRAM at 80 MHz, bigger data cache).

Not measured, possible: clean build time then vs now, incremental rebuild time, compiler
warning count (a 113-warning baseline early in the refactor -> 0 warnings in a release build today, checked), dollar
cost (check your billing page; I do not have reliable pricing to hand), number of distinct
PR "tracks" (about 25 named ones in my notes).

## 6. Failed previous attempts

Three approaches were tried and dropped before the one that worked.

| When | Approach | Outcome |
|---|---|---|
| January to April | Ask an AI to translate the C codebase to Rust directly. No c2rust, no transpiler | Worked for small, independent files; **failed completely** on the codebase as a whole |
| Aug 14 to 18 (v3) | Own C parser for `linuxdoom-1.10` | Parser only; function bodies were opaque. ~9 to 15 h, 103 commits |
| Aug 18 to Sep 5 (v4) | Own type checker + Rust code generator, function by function | 168 of 210 gameplay function bodies translated, no running game. ~41 h, 295 commits |

I have no records of the January to April attempts (the earliest session transcript on this
machine is from Aug 20), so everything about them here is what you told me. `~/rust_ideas/`
holds an upstream doomgeneric checkout and `room4doom` (an existing Rust Doom port), both
dated Feb 12; they may be from that period, but I cannot tell.

*To fill in before publishing: which model and tool were used, how many files were tried, and
what specifically broke.*

### Why direct translation probably failed (my inference, not from records)

"Fine on small independent files, hopeless on the whole" is the pattern I would expect, and this
project found concrete reasons for it later:

- **Shared mutable state.** Doom is ~80 files sharing ~1,000 mutable globals (985 `static mut`
  variables in the c2rust output). A file translated in isolation has to guess the shape of everything it
  touches, and Rust makes you decide ownership, which C never asked for.
- **Pointer-based data structures.** Intrusive linked lists, a custom zone allocator, WAD lumps
  as raw memory, pointers stored in structs and compared by address. There is no local,
  file-sized translation of these; the design has to change across files.
- **C behaviour that is technically undefined but relied on.** Masked reads past the end of a
  buffer, a `memset(-1)` producing a tri-state flag, signed overflow that wraps. Each of these
  compiled fine and misbehaved or panicked only when run.
- **No way to know it is right.** A translated file compiles or it does not, but "the same
  game" is a property of the whole program running. Without something to run and compare, the
  errors accumulate silently.

### Would a newer model make the direct route feasible now?

My assessment, with the caveat that I have not tried it and have **no measurements for Fable
(or any other model) on this task**. This is a reasoned guess, not a result.

**More feasible than in spring, but I would not expect "ask a better model to translate the
codebase" to be what makes it work.** The reasons come from this project:

- v4 used a strong model with a large test harness for ~41 h and still had no running game.
  v5 had a running game 14 minutes in, but through a mechanical tool, not through translation
  skill. What carried v5 was a running program from minute 14, hashes of the simulation that
  had to stay identical after every change, and one small PR per step. A better model can use
  those; it cannot replace them.
- Newer models should help with the parts that are about model quality: keeping a whole
  subsystem in view at once, fewer subtle semantic slips, and running the build-run-compare
  loop on their own for hours. That plausibly fixes some of the "small files work, whole
  fails" gap.
- They do not fix the part that is about the problem: the coupling between files and the lack
  of an oracle.

**What I would do if asked to try the direct route today:** build the oracle first (the C game
instrumented to print a hash of the world state per tic for the three demos, which this project
only added later and that is now a test, see section 7), then translate module by module in dependency order with the Rust and C
linked together, so the game runs after every step, and accept a module only when the hashes
match. With that harness I think a current model could plausibly do it without c2rust. Without
it I would expect the same failure as in spring, whichever model is used.

**Two further caveats.** The end state here (zero `unsafe`, indices instead of pointers, no
globals) took 358 PRs after the first running version; a direct translation would either need to
do all of that in one pass or hit the same sequence of steps afterwards. And cost matters: this
project used ~13.5 M output tokens and ~25,000 tool calls in v5 alone.

**A cheap experiment for the post:** pick one tightly coupled subsystem (for example `p_enemy.c`
or `r_bsp.c`), give a model the oracle and the neighbouring C, and see whether it produces a
hash-identical translation. That would turn this section from opinion into a measurement.

## 7. The oracle: the most important thing in the repo

An oracle is a program that answers one question: "does it still behave exactly as before?"
Everything else in this project (358 PRs, ~700k lines of churn, 985 globals and 48k unsafe lines
removed) was only trustworthy because of it. If the post has one lesson about working with an AI
on a big refactor, it is this one.

**What it is.** The real engine, run headlessly (no window, a virtual clock, input only from a
script) against a WAD. It hashes:

- the full simulation state (every map object, every sector, the player, both random-number
  indices, the tic counter) at the end of demo1, demo2 and demo3;
- frames rendered during a scripted tour of the menus, automap, intermission and finale, plus
  frames of the visual effects (invisibility fuzz, player-colour translation, low detail);
- a save/load round trip, and that two saves of the same state are byte-identical;
- a 6,000-tic trace of the cast-call finale;
- a sweep of every line special, on every side, for every actor type, restoring a save
  between cases.

The expected values live in `engine/golden/regression.txt` (1,502 lines). Doom is
deterministic, so a hash that stays the same means nothing observable changed, and one wrong
pixel or one extra random-number call changes it.

**It is a test now.** It started as an external script kept outside the repo: patch
`G_CheckDemoStatus` to print a hash, build into a separate directory, run under Xvfb, and diff
the output of `main` against the branch. That was the rule for every gameplay-touching PR in the
later tracks. On Sep 19, after the script stopped building when the engine went `no_std`, it
became part of the repo: PR #484, the first of the 11 idiomatic-review PRs, added
`engine/src/regression_tests.rs`. Now it is `cargo test --release`: 4 WAD-driven tests among 15,
about 10 s. Anyone, and any AI, can run it, and `UPDATE_GOLDEN=1` re-records the values when a
behaviour change is intended and understood.

**Why it matters.**

- **It replaces trust with a check.** Each PR either reproduces the hashes or it does not. Nobody
  has to read 700k lines of diff to believe the result.
- **It gives an AI a fast, objective feedback loop.** Run the tests, get yes or no in seconds.
  That is what lets an agent work autonomously for hours and land dozens of PRs without
  supervision, and it is what was missing in the January to April attempts (section 6).
- **It makes risky rewrites possible.** The technique: record the expected values against the OLD
  code first, then require the new code to reproduce them; then mutation-check the test by
  breaking one arm on purpose and confirming the test fails. That check found that the first
  version of the line-special sweep was too weak (the movers never ticked).
- **It shows performance work is safe too.** PR #502 removed two frame copies from the display
  path (section 5). The framebuffer hashes passing unchanged is the evidence that the speed-up
  did not change a single pixel.

**Limits, worth saying in the post.**

1. **The expected values were recorded from this Rust engine, not from the original C.** The
   oracle guards against drift from the c2rust-derived baseline; it does not prove equality with
   C. I have not checked the values against the C game. The stronger version is a per-tic hash
   dumped from an instrumented C build and compared directly. Not done.
2. **Without a WAD it passes vacuously.** If neither `$DOOM_IWAD` nor `~/Downloads/doom1.wad`
   exists, the tests print "skipping" and succeed, so a green run on a machine without the WAD
   verifies nothing.
3. **Which WAD.** The `doom1.wad` on this machine is a 14 MB, 36-map PWAD, while the shareware
   IWAD is a different 4.2 MB file (`Doom1.WAD`). The test's comments call it "shareware". Worth
   confirming which one the post should say.
4. **Coverage is what the demos and scripts reach.** Not covered: sound and music (nothing
   audible is produced or hashed, see section 11), Doom II-only paths (`G_WorldDone`), most other
   levels and games, and the X11 and ESP32 front ends themselves (the tests use a null platform).

## 8. What was done, at a high level

**Before the pivot (three failed approaches, see section 6).** After the January to April direct-translation attempts, Aug 14 to Sep 5 went into two more: write our own C parser, then a
transpiler with a type checker and a Rust code generator for `linuxdoom-1.10` (54.6k lines,
no Makefile). Good engineering, wrong shape of problem: each C function needed a new
"mechanism" (switch fallthrough, constructors, pointer-as-cursor patterns), and the
subsystems (WAD/texture, blockmap, renderer, sound, menus) were untouched.

**Step 1: let c2rust do the mechanical part (Sep 5).** Transpile the X11 build of doomgeneric.
145k unreadable lines, but it ran within minutes after fixing two C idioms Rust rejects:
masked out-of-bounds column reads and signed-overflow arithmetic (`--release` wraps like C).

**Step 2: shrink and clean (Sep 5 to 7).** Strings to `&str`, `boolean` to `bool`,
type/constant dedup, action enums, dead-code removal. One PR per phase, each verified against
a fixed warning baseline.

**Step 3: kill global state (Sep 6 to 12).** All `static mut` globals into one `GameState`
threaded as `&mut`, over ~96 small "bridge collapse" PRs. 985 `static mut` variables -> 0.

**Step 4: pointers to indices (Sep 11 to 15).** Struct-field pointers to typed ids and
`Vec`s, intrusive linked lists to index-backed lists, mobjs and thinkers into arenas, the zone
allocator deleted entirely, lump-number indices instead of `*mut patch_t`.

**Step 5: callbacks and enums (Sep 12 to 18).** All `extern "C"` callbacks to safe `fn`
types, C enums to Rust enums (including the three big tables: sprites, states, mobj types).

**Step 6: the unsafe sprint (Sep 18).** Ten PRs, ~21k unsafe lines to 198, SHA-1 swapped for a
crate, `deny(unsafe_code)` switched on.

**Step 7: platform boundary (Sep 12 and 19).** `DoomPlatform` trait, `x11` and `fs` crates,
engine `no_std` + `alloc`.

**Step 8: idiomatic review with a real test net (Sep 19).** Eleven stacked PRs: the oracle
became a test first (section 7), then literals, bool, loops, naming, clippy lints, `bitflags`, `Option` lookups.

**Step 9: hardware (Sep 19).** The unchanged engine crate on an ESP32-S3: display bring-up,
demos, Wi-Fi keyboard input, own DMA display driver, its own Wi-Fi access point, power-off on quit.

Why it worked: the oracle (section 7), one small PR per step, a verification bar that never moved (no new warnings,
identical simulation hashes on demo1 to demo3), and stacked PRs merged in order. The standing
"continue until done" authorization let phases chain without waiting.

## 9. What did not work

- **Direct AI translation (January to April).** Fine for small independent files, a complete
  failure on the codebase as a whole (see section 6 for why I think so).
- **The own-transpiler route (v3, v4): ~53 h, no running game.** The most honest "did not
  work". It was building tooling to avoid a problem that c2rust had already solved.
- **A wrong "this can never be done" verdict.** On Sep 11 I closed the GameState track,
  declaring the last `game_state()` accessor and its hub types "permanent" (real C-ABI
  constraints). That was wrong: none of them were real FFI boundaries. You asked to reopen it,
  the next session re-audited from scratch, and it was finished (`OnceLock` deleted, `Box::leak`
  instead). Lesson: a closure is only as good as the audit behind it.
- **A wrong size estimate the other way.** The mobj/thinker zone-allocator removal was
  estimated "an order of magnitude bigger, hundreds of call sites". Only ~30 real
  construction/destruction sites existed. It took 7 PRs.
- **The tri-state boolean.** `spriteframe_t.rotate` looked like a bool but was set with
  `memset(..., -1)` and read as -1/0/1. Converting it made a bogus error dialog appear on every
  boot. Caught before merge; the check "grep for `memset` with a non-zero fill before making a
  field `bool`" came out of it.
- **Things C got away with silently.** Once bounds checks existed, two real crashes showed up
  in ordinary demo playback: a reaper reading an already-retired mobj, and masked column reads
  running past exactly-sized buffers (vanilla relies on slack bytes in its zone allocator).
  Fixed by padding buffers and a reaper-specific accessor. Also `I_Error` became `panic!`, a
  documented deviation.
- **Plan ordering.** A safe `fn` pointer type can only hold safe callees, so callback types
  could not be converted ahead of their bodies; the unsafe-elimination plan had to be reordered
  mid-way.
- **The external regression oracle rotted.** The hash-the-world script stopped building when
  the engine went `no_std`; it was replaced by in-repo golden tests, which are better because
  they run under `cargo test`.
- **Text-based renames.** A global find-and-replace over plain lowercase names (`ok`, `top`,
  `none`) is unsafe; the final rename was driven by compiler lints instead.
- **Note rot.** My own status file grew to 134 KB and went stale; it is now marked outdated.
  Session summaries kept the work going across context resets, but only the PR trail and
  short per-track notes stayed trustworthy.
- **Hardware limits.** QIO flash mode boot-loops this board. Wi-Fi join, DHCP and TCP end to end
  are **still untested** on the access-point build (the dev laptop has one Wi-Fi adapter,
  so joining would cut the session's internet).
- **Deliberately not done.** 65 post-increment temporaries in renderer column loops, ~83 non-counting
  `while` loops, `Result`-based error handling, and the remaining `c_char` uses in cheat
  sequences and dead code.

## 10. Method notes and caveats

- **Time:** first-to-last event timestamps in the transcripts, union across all sessions and
  subagents, gaps over 10 minutes ignored. With a 5-minute gap the v5 total is 82 h, with 30
  minutes it is 98 h. v3 has no transcripts (estimated from commit gaps of 30 to 120 minutes:
  9 to 15 h). Transcripts record every session as Sonnet 5.
- **Lines:** counted per first-parent commit with a small script (`git grep` over `*.rs`),
  skipping blank lines and `//` and `/* */` comments. Approximate: string literals are
  stripped before brace matching, and edge cases (raw strings, macros) are not handled.
  The C figure uses the same idea via `grep`, over the 81 compiled `.c` files plus all headers.
- **Unsafe lines:** brace tracking from each `unsafe {` and `unsafe fn ... {` to its matching
  `}`. Overcounts nothing but can miss `unsafe` inside macros. The engine result is backed up
  by the compiler: `#![deny(unsafe_code)]` is on for the whole crate.
- **`static mut`:** the column counts variable definitions (a `static mut` with an initializer,
  via `git grep`). c2rust also emits 653 repeated `extern` re-declarations of the same globals;
  counting those too gives 1,639 at the start, which overstates the number of globals. It does
  not count `&'static mut` types. The per-commit CSV column `static_mut` also matches `&'static mut`, so
  it shows 5 to 7 at the end; ignore those.
- **Performance:** two sessions measured independently with `perf stat -e
  cycles:u,instructions:u,L1-dcache-load-misses:u`, pinned to one core with `taskset`, on the same
  machine (loaded by other sessions, so cycle counts vary by up to ~50 % between runs; the
  instruction counts do not). Ranges in section 5 are best-of-several. The C game was built with
  GCC 13 `-O2` in a scratch directory; nothing in the repo was touched. The first, wall-clock
  numbers (1/35 s `realtics`) are superseded.
- **Human-time estimates** in section 2 are my judgment.

## 11. Open items (help needed)

- **Speed-up: how to measure.** There is now a repeatable method: `perf stat` on user cycles,
  pinned core, `-timedemo demo1`, `Doom1.WAD`, C built with GCC `-O2`, compare absolute cycles
  rather than percentages. It needs `kernel.perf_event_paranoid <= 2` (set to 1 here; it resets on
  reboot). Still to decide: how many runs, and whether to quote best-of-N given the noise. The
  ESP32 figures (2.9 to 29 fps) come from the `[perf]` serial lines and have not been re-verified.
- **Sound and music.** Neither exists yet, on X11 or on the ESP32.
  - `DoomPlatform` has no audio hooks. `i_sound.rs` has slots for a sound module and a music
    module that are always empty (its own comments call the module table a stub).
  - The game logic in `s_sound.rs` (channels, volume, stereo separation) runs, but nothing is
    output.
  - Music would need more than effects: MUS lumps converted to MIDI and a synthesizer. There is
    no MUS-to-MIDI module in the tree.
  - A separate worktree, `sound-sfx-linux`, has one unmerged commit, "Play sound effects: built-in
    mixer, DoomPlatform audio hooks, Linux sink". I have only read its title, not the code, and
    the title does not mention music.
  - The oracle does not cover audio. One option is to hash the mixed sample stream per tic, so
    sound work gets the same kind of check as the rest.
- **Testing of more games and levels.** The oracle's coverage is what its demos and scripts
  reach, on one WAD file (see the WAD caveat in section 7). Other games (Doom II, Ultimate Doom,
  Final Doom), most levels and mods are untested.
