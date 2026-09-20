"""Merge a deferred declaration with its first assignment.

Run from `rust-doomgeneric/` (the workspace root):

    python3 tools/late_init.py            # dry run: count per file
    python3 tools/late_init.py --apply    # then `cargo clippy --fix` (drops now-unneeded `mut`) and `cargo fmt`

C declares every local at the top of the function; the port kept that as

    let mut delta: i32;
    ...
    delta = a - b;

When the first statement after the declaration that mentions the variable is a plain
`delta = <expr>;` at the same block level (and `<expr>` does not mention it), the two are one
`let mut delta: i32 = a - b;`. Anything else (assigned inside an `if`, read first, declared in one
block and assigned in another) is left for a human, and the compiler rejects any merge that
would be wrong. The variable keeps its type annotation and its `mut`; rustc's `unused_mut`
fix removes the `mut` where it turns out not to be needed.
"""
import glob
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rslex import code_mask

DECL = re.compile(r"(?m)^([ \t]*)let (mut )?(\w+): ([^;=\n]+);[ \t]*\n")


def statements(s, mask, start, end):
    """Yield (a, b) spans of top-level statements in s[start:end] (b is exclusive)."""
    depth, a, i = 0, start, start
    while i < end:
        if mask[i]:
            c = s[i]
            if c in "([{":
                depth += 1
            elif c in ")]}":
                depth -= 1
                if depth == 0 and c == "}":
                    # a block-bodied statement (if/while/for/match/loop) ends at its closing brace
                    # unless it continues with `else` or is followed by `.method()`/`;`/`)`
                    rest = s[i + 1:end].lstrip()
                    if not (rest.startswith("else") or rest.startswith(".") or rest.startswith(";") or rest.startswith(")") or rest.startswith("?") or rest.startswith(",")):
                        yield (a, i + 1)
                        a = i + 1
            elif c == ";" and depth == 0:
                yield (a, i + 1)
                a = i + 1
        i += 1
    if s[a:end].strip():
        yield (a, end)


def enclosing_end(s, mask, pos):
    depth = 0
    for j in range(pos, len(s)):
        if not mask[j]:
            continue
        if s[j] == "{":
            depth += 1
        elif s[j] == "}":
            if depth == 0:
                return j
            depth -= 1
    return None


total = 0
for f in sorted(glob.glob("engine/src/*.rs")):
    s = open(f).read()
    mask = code_mask(s)
    edits = []  # (start, end, replacement); non-overlapping, applied back to front
    for m in DECL.finditer(s):
        indent, mut, name, ty = m.group(1), m.group(2) or "", m.group(3), m.group(4).strip()
        if not mask[m.start() + len(indent)]:
            continue
        end = enclosing_end(s, mask, m.end())
        if end is None:
            continue
        word = re.compile(r"(?<![\w.])" + name + r"\b")
        inline = re.compile(r"\{" + name + r"[:}]")
        for a, b in statements(s, mask, m.end(), end):
            text = s[a:b]
            mentions = [x for x in word.finditer(text) if mask[a + x.start()]] or list(inline.finditer(text))
            if not mentions:
                continue
            assign = re.match(r"(\s*)" + name + r"\s*=(?!=)\s*(.*);\s*$", text, re.S)
            if assign and not word.search(assign.group(2)) and not inline.search(assign.group(2)):
                lead = assign.group(1)
                new = f"{lead}let {mut}{name}: {ty} = {assign.group(2)};"
                edits.append((m.start(), m.end(), ""))
                edits.append((a, b, new))
            break
    if not edits:
        continue
    total += len(edits) // 2
    print(f"{len(edits) // 2:4d} {f}")
    if "--apply" in sys.argv:
        for a, b, r in sorted(edits, reverse=True):
            s = s[:a] + r + s[b:]
        open(f, "w").write(s)
print("total", total)
