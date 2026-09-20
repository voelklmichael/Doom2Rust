# Transpiling the X11 Version to Rust with c2rust

This documents the exact commands used to transpile the C sources for the
Linux X11 build of doomgeneric (`doomgeneric/doomgeneric/`) into Rust, using
[c2rust](https://github.com/immunant/c2rust). This is a mechanical,
line-by-line translation — the output is not idiomatic Rust and does not
build without further work. Getting it to compile is tracked in a separate
doc/change.

The result was written to `rust-doomgeneric/` at the repository root.

## Prerequisites

```bash
cargo install c2rust
```

This installed `c2rust` 0.22.1 and `c2rust-transpile` 0.22.1 into
`~/.cargo/bin`. It uses the system Clang/LLVM (Clang 18.1.3 on this machine)
to parse the C sources.

c2rust needs a `compile_commands.json` describing how each translation unit
is compiled. Tools like `bear` or `intercept-build` normally generate this by
wrapping a real build, but neither was installed and installing `bear` via
`apt` required interactive sudo, which was not available non-interactively.
Since the Makefile compiles every source file with the same flags, the
compilation database was instead generated directly from the Makefile's
`SRC_DOOM` file list.

## Step 1: Sanity-check the C build

From `doomgeneric/doomgeneric/`:

```bash
make clean
make -j"$(nproc)"
```

This confirms the C sources still build before attempting a translation (see
`docs/build-and-run-c.md`).

## Step 2: Generate compile_commands.json

From `doomgeneric/doomgeneric/`, extract the source file list from the
Makefile and emit one compile-command entry per file, using the same flags
the Makefile passes to `clang`:

```bash
grep "^SRC_DOOM" Makefile | tr ' ' '\n' | grep '\.o$' | sed 's/\.o$//' > /tmp/doom_src_list.txt

python3 - <<'EOF'
import json, os

srcdir = os.path.abspath(".")
with open("/tmp/doom_src_list.txt") as fh:
    files = [l.strip() for l in fh if l.strip()]

flags = ["clang", "-ggdb3", "-Os", "-ggdb3", "-Wall", "-DNORMALUNIX", "-DLINUX",
         "-DSNDSERV", "-D_DEFAULT_SOURCE"]

entries = []
for f in files:
    src = f"{f}.c"
    obj = f"build/{f}.o"
    entries.append({
        "directory": srcdir,
        "arguments": flags + ["-c", src, "-o", obj],
        "file": os.path.join(srcdir, src),
        "output": os.path.join(srcdir, obj),
    })

with open("compile_commands.json", "w") as fh:
    json.dump(entries, fh, indent=2)

print(f"wrote {len(entries)} entries")
EOF
```

This produced 81 entries, one per file in the Makefile's `SRC_DOOM` list
(the directory contains other, unused `doomgeneric_*.c` backends —
SDL, Allegro, Emscripten, Windows, etc. — that are correctly excluded).

## Step 3: Run c2rust transpile

The X11 backend's `main()` lives in `doomgeneric_xlib.c` (not
`doomgeneric.c`, despite the crate name), so that is the file passed to
`-b/--binary`. Note that `-b` takes the module name **without** the `.c`
extension — passing `doomgeneric_xlib.c` or an absolute path silently fails
to attach a `main` and only emits a library crate.

From `doomgeneric/doomgeneric/`:

```bash
c2rust transpile compile_commands.json \
  --emit-build-files \
  -b doomgeneric_xlib \
  -o ../../rust-doomgeneric
```

This:

- Parses all 81 translation units with Clang and translates each to a
  `.rs` file of the same name under `rust-doomgeneric/src/`.
- Emits `rust-doomgeneric/lib.rs` re-exporting every translated module.
- Emits `rust-doomgeneric/build.rs` (an empty per-platform stub).
- Emits `rust-doomgeneric/Cargo.toml` with a `[lib]` target
  (`staticlib` + `rlib`) and a `[[bin]]` target named `doomgeneric_xlib`
  pointing at `src/doomgeneric_xlib.rs`, which contains the translated
  `main()`.
- Emits `rust-doomgeneric/rust-toolchain.toml`, pinning
  `nightly-2023-04-15` — the translated code uses unstable features
  (`c_variadic`, `extern_types`, `raw_ref_op`) that require a nightly
  compiler.
  (Historical: the pin was later removed and the crate builds on stable Rust; see
  `docs/rust-build.md`.)
- Depends on the `libc` and `c2rust-bitfields` crates.

`rustfmt` printed `rustfmt failed, code may not be well-formatted` /
`Error: file ... does not exist` warnings for every emitted file during this
run; these are cosmetic (rustfmt just failed to format the fresh output
in-place) and every `.rs` file was written correctly.

## Step 4: Clean up

The `compile_commands.json` and C build artifacts from Step 1/2 are not
committed:

```bash
cd doomgeneric/doomgeneric
make clean
rm -f compile_commands.json
```

`rust-doomgeneric/target/` (Cargo's build output) is excluded via
`.gitignore`.

## Result

```
rust-doomgeneric/
├── Cargo.toml
├── Cargo.lock
├── build.rs
├── lib.rs
├── rust-toolchain.toml
└── src/
    ├── doomgeneric_xlib.rs   # contains translated main()
    ├── doomgeneric.rs
    ├── d_main.rs
    ├── p_mobj.rs
    ├── r_bsp.rs
    ├── w_wad.rs
    └── ... (81 files total, one per C translation unit)
```

At this point the crate has not yet been built (`cargo build`/`cargo run`);
that is tracked separately.
