"""Turn `for i in 0..N` loops whose variable is only ever used as `i as usize` into usize loops.

Run from `rust-doomgeneric/` (the workspace root):

    python3 tools/loop_index.py            # dry run: count per file
    python3 tools/loop_index.py --apply

A C-style `for (i = 0; i < N; i++)` port keeps `i` an `i32` and casts at every index. When
*every* use of the loop variable in the body is `i as usize`, the cast can move to the bound
(once) and the body loses all of them. Loops where `i` is also used as a number (compared,
added, passed to a function) are left alone: converting them just moves the cast elsewhere.

The bound is rewritten to `<bound> as usize` (a literal bound is left alone; a bound that is
already `as usize` means the variable is already usize, so the loop is skipped).
"""
import glob
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rslex import code_mask, match_brace

FOR = re.compile(r"\bfor\s+(\w+)\s+in\s+0\s*\.\.\s*(=?)([^{]+?)\s*\{")
total = 0
for f in sorted(glob.glob("engine/src/*.rs")):
    s = open(f).read()
    mask = code_mask(s)
    edits = []  # (start, end, replacement) on the original text
    for m in FOR.finditer(s):
        if not mask[m.start()]:
            continue
        var, eq, bound = m.group(1), m.group(2), m.group(3).strip()
        if var == "_" or eq or "as usize" in bound:
            continue  # inclusive ranges are rare here; a usize bound means the variable already is
        b = m.end() - 1
        try:
            e = match_brace(s, mask, b)
        except Exception:
            continue
        body = s[b:e]
        bmask = mask[b:e]
        uses = [u for u in re.finditer(r"(?<![\w.])" + var + r"\b", body) if bmask[u.start()]]
        casts = [u for u in re.finditer(r"(?<![\w.])" + var + r"\s+as\s+usize\b", body) if bmask[u.start()]]
        if not uses or len(uses) != len(casts):
            continue
        # A `for` whose variable is only cast: apply.
        for c in casts:
            edits.append((b + c.start(), b + c.end(), var))
        if not re.fullmatch(r"\d+", bound):  # a literal bound takes whatever type the body needs
            simple = re.fullmatch(r"[\w.]+(\([^()]*\))?", bound)
            new = (bound if simple else f"({bound})") + " as usize"
            edits.append((m.start(3), m.end(3), new))
    if not edits:
        continue
    total += len(set(edits))
    print(f"{len(edits):4d} edits {f}")
    if "--apply" in sys.argv:
        for a, b, r in sorted(set(edits), reverse=True):
            s = s[:a] + r + s[b:]
        open(f, "w").write(s)
print("total", total)
