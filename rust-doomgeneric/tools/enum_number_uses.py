"""List every place where the integer value of an enum is used.

Run from `rust-doomgeneric/` (the workspace root):

    python3 tools/enum_number_uses.py > /tmp/enum_uses.json     # slow: one build per enum

For each fieldless top-level enum this temporarily adds a data-carrying variant (`__Probe(u8)`),
which makes every `enum as i32` / `as usize` / `as u8` cast of that type a compile error
("non-primitive cast"), builds, collects the error positions, and restores the file. Those positions
are exactly the sites that observe the discriminant. A `match` on the enum is not a cast and does
not show up; neither does anything that only compares variants.

The probe adds one line to the enum's own file, so sites in that file after the definition are
shifted back by one. Output is JSON: {enum: {"file", "line", "sites": [[file, line], ...]}}.
docs/enum-discriminants.md was written from this output.
"""
import glob
import json
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rslex import code_mask, match_brace

ENUM = re.compile(r"(?m)^(?:#\[[^\n]*\]\s*)*(?:pub(?:\([a-z]+\))? )?enum (\w+)\b[^{;]*\{\n")
PKG = os.environ.get("PKG", "rust_doomgeneric")

enums = []
for f in sorted(glob.glob("engine/src/*.rs")):
    s = open(f).read()
    mask = code_mask(s)
    for m in ENUM.finditer(s):
        b = m.end() - 2
        e = match_brace(s, mask, b)
        if e is None or re.search(r"^\s*\w+\s*[({]", s[b + 1:e], re.M):
            continue  # data-carrying: no integer value to observe
        enums.append((f, m.group(1), s[:m.end()].count("\n")))

result = {}
for f, name, line in enums:
    orig = open(f).read()
    m = re.search(r"(?m)^(?:pub(?:\([a-z]+\))? )?enum " + name + r"\b[^{;]*\{\n", orig)
    open(f, "w").write(orig[:m.end()] + "    __Probe(u8),\n" + orig[m.end():])
    try:
        r = subprocess.run(
            ["cargo", "build", "-p", PKG, "--release", "--message-format=short"],
            capture_output=True, text=True,
        )
    finally:
        open(f, "w").write(orig)
    sites = set()
    for l in (r.stdout + r.stderr).splitlines():
        if "non-primitive cast" in l and "`" + name + "`" in l:
            mm = re.match(r"^(engine/src/[\w.]+):(\d+):(\d+):", l)
            if mm:
                fl, ln = mm.group(1), int(mm.group(2))
                if fl == f and ln > line:
                    ln -= 1
                sites.add((fl, ln))
    result[name] = {"file": f, "line": line, "sites": sorted(sites)}
    print(f"{name}: {len(sites)} sites", file=sys.stderr)
json.dump(result, sys.stdout, indent=1)
