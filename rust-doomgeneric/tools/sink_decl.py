"""Sink a deferred declaration into the block that is the only place that uses it.

Run from `rust-doomgeneric/` (the workspace root):

    python3 tools/sink_decl.py            # dry run: count per file
    python3 tools/sink_decl.py --apply    # then `python3 tools/late_init.py --apply`, `cargo build`, fmt

C declares every local at the top of the function; the port kept that:

    let mut dx: i32;
    ...
    for i in 0..n {
        dx = a[i] - b[i];
        ...
    }

When exactly one statement after the declaration mentions the variable, and that statement is a
`for`/`while`/`loop` (mentions only in the body) or an `if`/`else` chain (mentions only in the
branch blocks, never in a condition), the declaration moves to the start of that body / of each
branch that uses it. It repeats until nothing moves, then `late_init.py` merges each with its first
assignment. Moving an *uninitialised* declaration inward changes no behaviour: a variable that is
carried from one iteration to the next (read before it is assigned) stops compiling with "possibly
uninitialised", and such a case goes in tools/sink_decl_skip.json (`"file.rs:name"`) and is left.
"""
import glob
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rslex import code_mask

HERE = os.path.dirname(os.path.abspath(__file__))
SKIP = set(json.load(open(HERE + "/sink_decl_skip.json"))) if os.path.exists(HERE + "/sink_decl_skip.json") else set()
DECL = re.compile(r"(?m)^([ \t]*)let (mut )?(\w+): ([^;=\n]+);[ \t]*\n")


def statements(s, mask, start, end):
    depth, a, i = 0, start, start
    while i < end:
        if mask[i]:
            c = s[i]
            if c in "([{":
                depth += 1
            elif c in ")]}":
                depth -= 1
                if depth == 0 and c == "}":
                    rest = s[i + 1:end].lstrip()
                    if not rest.startswith(("else", ".", ";", ")", "?", ",")):
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


def match_brace(s, mask, i):
    d = 0
    for j in range(i, len(s)):
        if not mask[j]:
            continue
        if s[j] == "{":
            d += 1
        elif s[j] == "}":
            d -= 1
            if d == 0:
                return j
    return None


def first_block(s, mask, start, end):
    """Position of the first `{` at paren depth 0 in s[start:end] (skipping strings), else None."""
    depth = 0
    for j in range(start, end):
        if not mask[j]:
            continue
        c = s[j]
        if c in "([":
            depth += 1
        elif c in ")]":
            depth -= 1
        elif c == "{" and depth == 0:
            return j
    return None


def targets(s, mask, a, b, word, inline):
    """Block openers `{` (positions) a declaration must be sunk to, or None if it cannot sink."""

    def mentions(x, y):
        return [m for m in word.finditer(s, x, y) if mask[m.start()]] or list(inline.finditer(s, x, y))

    st = s[a:b].lstrip()
    off = a + (len(s[a:b]) - len(st))
    if re.match(r"(for|while|loop)\b", st):
        blk = first_block(s, mask, off, b)
        if blk is None:
            return None
        end = match_brace(s, mask, blk)
        if end is None or s[end + 1:b].strip() or mentions(off, blk):
            return None
        return [blk]
    if st.startswith("if "):
        out, pos = [], off
        while True:
            blk = first_block(s, mask, pos + 2, b)
            if blk is None:
                return None
            cond_mentions = mentions(pos, blk)
            end = match_brace(s, mask, blk)
            if end is None or cond_mentions:
                return None
            if mentions(blk, end):
                out.append(blk)
            rest = s[end + 1:b]
            m = re.match(r"\s*else\s+(if\b|\{)", rest)
            if not m:
                if rest.strip():
                    return None
                return out or None
            pos = end + 1 + m.start(1) - 0
            if m.group(1) == "{":
                blk = end + 1 + m.start(1)
                end2 = match_brace(s, mask, blk)
                if end2 is None or s[end2 + 1:b].strip():
                    return None
                if mentions(blk, end2):
                    out.append(blk)
                return out or None
    return None


def pass_once(s, fname):
    mask = code_mask(s)
    for m in DECL.finditer(s):
        indent, mut, name, ty = m.group(1), m.group(2) or "", m.group(3), m.group(4).strip()
        if not mask[m.start() + len(indent)] or f"{fname}:{name}" in SKIP:
            continue
        end = enclosing_end(s, mask, m.end())
        if end is None:
            continue
        word = re.compile(r"(?<![\w.])" + name + r"\b")
        inline = re.compile(r"\{" + name + r"[:}]")
        hits = []
        for a, b in statements(s, mask, m.end(), end):
            if [x for x in word.finditer(s, a, b) if mask[x.start()]] or inline.search(s, a, b):
                hits.append((a, b))
        if len(hits) != 1:
            continue
        tg = targets(s, mask, hits[0][0], hits[0][1], word, inline)
        if not tg:
            continue
        ins = f"let {mut}{name}: {ty};\n"
        edits = [(m.start(), m.end(), "")] + [(t + 1, t + 1, "\n" + indent + "    " + ins) for t in tg]
        for a, b, r in sorted(edits, reverse=True):
            s = s[:a] + r + s[b:]
        return s, name
    return s, None


total = 0
for f in sorted(glob.glob("engine/src/*.rs")):
    s = open(f).read()
    n = 0
    while True:
        s2, name = pass_once(s, os.path.basename(f))
        if name is None:
            break
        s = s2
        n += 1
    if n:
        total += n
        print(f"{n:4d} sinks {f}")
        if "--apply" in sys.argv:
            open(f, "w").write(s)
print("total", total)
