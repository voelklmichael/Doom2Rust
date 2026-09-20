"""Rewrite C's `while (count--)` / `do { } while (count--)` emulations as `for` loops.

Run from `rust-doomgeneric/` (the workspace root):

    python3 tools/count_loops.py            # dry run
    python3 tools/count_loops.py --apply    # then `cargo fmt`

The conversion leaves loops like

    loop {                                  loop {
        let fresh1 = count;                     ...
        count -= 1;                             let fresh1 = count;
        if fresh1 == 0 { break; }               count -= 1;
        ...                                     if fresh1 == 0 { break; }
    }                                       }
      (while count--: runs `count` times)     (do-while: runs `count + 1` times)

which are `for _ in 0..count` and `for _ in 0..=count`. It only rewrites a loop when the counter
is not otherwise mentioned in the body or after the loop (the original leaves it at -1), and the
body has no `continue` (in the do-while form a `continue` would have skipped the decrement).
A counter that is negative on entry would have looped ~2^32 times; the `for` runs zero times
(the callers guard against it).
"""
import glob
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rslex import code_mask, match_brace

TRIO = r"let (fresh\d+) = (\w+);\s*\n\s*\2 -= 1;\s*\n\s*if \1 == 0 \{\s*\n\s*break;\s*\n\s*\}"
START = re.compile(r"\s*" + TRIO)
END = re.compile(TRIO + r"\s*$")
n_start = n_end = 0
for f in sorted(glob.glob("engine/src/*.rs")):
    s = open(f).read()
    mask = code_mask(s)
    edits = []
    for m in re.finditer(r"\bloop \{", s):
        if not mask[m.start()]:
            continue
        b = m.end() - 1
        e = match_brace(s, mask, b)
        body = s[b + 1:e]
        kind = None
        st, en = START.match(body), END.search(body)
        if st:
            kind, counter, rest = "start", st.group(2), body[st.end():]
        elif en:
            kind, counter, rest = "end", en.group(2), body[:en.start()]
        else:
            continue
        word = re.compile(r"(?<![\w.])" + counter + r"\b")
        if word.search(rest) or re.search(r"\bcontinue\b", rest):
            continue
        # the counter must be dead after the loop: not mentioned again in the enclosing block
        depth, j, after_end = 0, e + 1, None
        while j < len(s):
            if mask[j]:
                if s[j] == "{":
                    depth += 1
                elif s[j] == "}":
                    if depth == 0:
                        after_end = j
                        break
                    depth -= 1
            j += 1
        if after_end is None or word.search(s[e + 1:after_end]):
            continue
        rng = f"0..={counter}" if kind == "end" else f"0..{counter}"
        edits.append((m.start(), e + 1, f"for _ in {rng} {{" + rest.rstrip() + "\n}"))
        if kind == "start":
            n_start += 1
        else:
            n_end += 1
    if edits:
        print(f"{len(edits):3d} {f}")
        if "--apply" in sys.argv:
            for a, b, r in sorted(edits, reverse=True):
                s = s[:a] + r + s[b:]
            open(f, "w").write(s)
print(f"while-count: {n_start}, do-while-count: {n_end}")
