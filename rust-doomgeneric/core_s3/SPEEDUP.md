# core_s3: speeding up DOOM

Baseline (2026-09-19): about 3 fps. The LCD blit takes about 58 ms per frame (26 ms is the wire
minimum at 40 MHz SPI); everything else is 200-280 ms per frame. The `[perf]` line on serial prints
this every 2 s. Only the ideas marked "measured" below have been confirmed on the board; the other
numbers are estimates from reading the code.

## Measured results

`[perf]` line on serial, gameplay (attract-mode demo), 2 s windows, Wi-Fi on:

| Build | fps | blit per frame | everything else per frame |
|---|---|---|---|
| Baseline | 2.8-2.9 | 59 ms | 276-292 ms |
| D: indexed present path | 7.8-8.6 | 44.5 ms | 71-83 ms |
| E: column/span loops hoisted | 10.0-10.7 | 44.5 ms | 48-55 ms |
| H: DMA blit on core 0 | 18.4-21.8 | 6.2 ms present (0.2 ms waiting) | 40-48 ms |

From D on, "blit" is measured on the game core. For H the column is "present": the time core 1 spends
in `draw_indexed_frame`, which is waiting for the previous frame to finish going out (0.2 ms, so the
DMA is fully hidden behind rendering) plus converting the frame into the DMA buffer (6 ms).

Title and menu screens run at about 30 fps. In simple rooms the game now sits at a steady 30 fps:
the engine waits on its own 35 Hz tic clock there, so it is no longer CPU-bound. Busy scenes run at
about 18-21 fps.

- D: the blit got 15 ms cheaper because the LCD loop no longer reads half-rows out of a 1 MB PSRAM
  buffer, and the 512 KB per-frame alloc+scale+copy disappeared.
- E: host test suite 13.2 s -> 8.5 s for the same output, bit for bit.
- H: the panel is written by DMA from a 128 KB buffer in internal RAM. One SPI DMA transfer is capped
  at 32736 bytes, so a frame is four chunks of 50 rows, re-armed by core 0 (which polls every
  250 us and sleeps in between, so Wi-Fi and TCP keep running). The wire time (26 ms) no longer
  matters as long as rendering takes longer. The price: core 0's stack shrank from 230 KB to 98 KB.

## Ideas, ranked by value for effort

| # | Idea | Effect (guess) | Effort | Risk | Status |
|---|---|---|---|---|---|
| A | PSRAM to 80 MHz (`ram_frequency: Freq80m`; the default is 40 MHz) | PSRAM-bound phases up to ~2x faster | one line | low (may need a stability check) | todo |
| B | Data cache 64 KB with 64-byte lines (`ESP_HAL_CONFIG_DATA_CACHE_SIZE` / `_LINE_SIZE`); the linker script already leaves that block free | fewer PSRAM misses, no RAM cost | config | low | todo |
| C | Flash QIO at 80 MHz (`espflash --flash-mode qio --flash-freq 80mhz`), after checking what it is now | faster code and rodata cache misses | runner flag | low | todo |
| D | Indexed present path: a defaulted `DoomPlatform` method hands over the 64 KB indexed buffer and the palette; CoreS3 maps it through a 256-entry RGB565 table straight into the SPI stream. Other platforms and the tests keep the current path. | removes a 512 KB alloc+memset and ~4 MB of PSRAM traffic per frame; probably the biggest single win | medium | low | **done, measured: 2.9 -> 8.5 fps** |
| E | Hoist the per-pixel lookups out of the column, span, low-detail and fuzz loops in `r_draw.rs` (resolve source slice, `&[u8; 256]` colormap and destination once per column) | 2-3x on the loops that dominate rendering | medium | low | **done, measured: 8.5 -> 10.7 fps** |
| F | Hot code in IRAM via esp-hal's `use_rwtext_ld_hook` (move `r_*` and fixed-math functions into `.rwtext`; ~250 KB of IRAM/DRAM is spare) | fewer icache misses | medium | medium (LTO inlining changes section names) | todo |
| G | Hot buffers in internal RAM: `i_video_buffer` (64 KB), `colormaps` (9 KB), `ylookup` / `columnofs` | fewer PSRAM misses | medium | needs an allocation hook | todo |
| H | Faster, asynchronous blit: DMA SPI, an 80 MHz SPI clock (panel must tolerate it), or presenting from core 0 while core 1 renders the next frame | frees 25-58 ms per frame | high | medium | **done (DMA + core 0), measured: 10.7 -> ~20 fps.** The 80 MHz clock is not needed while the wire time stays hidden. |
| I | Visible knobs: low detail (`detailshift=1`, about half the world pixels), screen size 9 or 8, skipping the melt wipe | up to ~2x rendering | small | changes how it looks | todo |
| J | Build-profile experiments: `opt-level` 2, 3 or "s" for the engine versus the rest, given the 32 KB icache | +-10% | trivial | none | todo |

Not recommended: two-core rendering (the renderer's global state is not thread-safe and both cores
share the PSRAM bus), raising the CPU clock (already at max), dropping game tics (changes the
simulation).

## Plan

**Phase 0, measure first.** A no-op `phase()` hook on `DoomPlatform`, so `[perf]` splits time into
simulation tics, `render_player_view`, HUD/menu/status bar, `finish_update` and the blit, plus tics
per frame. Use the attract-mode demo loop as the benchmark (deterministic, no input). Calibration
runs with no code: Wi-Fi off, low detail, smaller screen. Check the flash mode and speed in the
bootloader boot log.

**Phase 1, config only** (A, B, C, J), one change per flash so the effects can be separated.

**Phase 2, engine changes** (D, then E, plus the `hu_drawer` widget clones in `hu_stuff.rs`). The
golden regression test (`cargo test --release`) checks frames bit for bit, so these are verified on
the host before flashing. Its wall time (13.0 s on the dev machine) is also a rough CPU benchmark.

**Phase 3, memory placement** (F, G), once the profile shows PSRAM or icache misses still matter.

**Phase 4, presentation** (H), only if the blit is still 10% or more of the frame after Phase 2. Done.

**Phase 5, visible knobs** (I), a decision about how the game should look.

## Where the time is suspected to go

1. `finish_update` allocates and zeroes a fresh 512 KB buffer in PSRAM every frame, fills it, copies
   it into the 1 MB `dg_screen_buffer`, and `draw_frame` reads half of it again (idea D).
2. PSRAM runs at 40 MHz and the data cache is 32 KB with 32-byte lines (ideas A, B).
3. `draw_column` / `draw_span` go through `state` for every pixel: an `Option` unwrap, `read_source`
   (lumpinfo, `Rc`, bounds check), `colormaps` and `i_video_buffer` indexing (idea E).
4. `.text` is 1.16 MB, executed from flash through a 32 KB icache (ideas C, F).
5. The blit is CPU-bound: blocking SPI writes through 64-byte FIFO refills (idea H).

## Next candidates, from the latest numbers

Per frame in a busy scene: about 6 ms converting the frame for the LCD, 40-48 ms for everything else
(simulation tics, rendering, HUD). The conversion is 12% of that and cheap to shrink: it reads 64 KB
from PSRAM and writes 2 bytes per pixel with a byte-pair copy; a `u16` table and a wider write, or
moving the conversion to core 0, would cut most of it. Beyond that the split of "everything else" is
still unmeasured (Phase 0), and A, B, C, F, G and I are untried.
