"""Turn a deferred declaration assigned in every branch into `let x = if/match ...;`.

Run from `rust-doomgeneric/` (the workspace root):

    python3 tools/if_expr.py            # dry run: list what converts
    python3 tools/if_expr.py --apply    # then `cargo build`, `cargo clippy --fix`, `cargo fmt`

C's `int x; if (a) x = 1; else x = 2;` was ported as

    let x: i32;
    if a { x = 1; } else { x = 2; }

which is `let x: i32 = if a { 1 } else { 2 };`. A declaration converts when the first statement
that mentions it is an `if`/`else if`/`else` chain (with a final `else`) or a `match`, and

  * in every branch that can fall through, the last statement is `x = <expr>;` (a `match` arm may
    be `pat => x = <expr>,`) and `x` is not mentioned anywhere else in the statement;
  * a branch that cannot fall through (it ends in `return`, `break`, `continue`, `panic!` or a
    call to the diverging `error(..)`) is left as it is.

Declarations that fail are left for a human. The compiler rejects a wrong rewrite.
"""
import glob
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rslex import code_mask

DECL = re.compile(r"(?m)^([ \t]*)let (mut )?(\w+): ([^;=\n]+);[ \t]*\n")
DIVERGES = re.compile(r"\s*(return\b|break\b|continue\b|panic!|unreachable!|error\()")


def statements(s, mask, start, end):
    depth, a, i = 0, start, start
    out = []
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
                        out.append((a, i + 1))
                        a = i + 1
            elif c == ";" and depth == 0:
                out.append((a, i + 1))
                a = i + 1
        i += 1
    if s[a:end].strip():
        out.append((a, end))
    return out


def enclosing_end(s, mask, pos):
    d = 0
    for j in range(pos, len(s)):
        if not mask[j]:
            continue
        if s[j] == "{":
            d += 1
        elif s[j] == "}":
            if d == 0:
                return j
            d -= 1
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


def branch_edit(s, mask, open_brace, word, name):
    """(edit, ok) for a block `{ ... }`: rewrite its last `name = expr;` to `expr`.

    Returns a list of (start, end, replacement) edits, [] for a diverging block, or None if the
    block neither diverges nor ends in the assignment (or mentions the name elsewhere)."""
    close = match_brace(s, mask, open_brace)
    if close is None:
        return None
    sts = statements(s, mask, open_brace + 1, close)
    if not sts:
        return None
    la, lb = sts[-1]
    last = s[la:lb]
    for a, b in sts[:-1]:
        if [x for x in word.finditer(s, a, b) if mask[x.start()]]:
            return None
    m = re.fullmatch(r"(\s*)" + name + r"\s*=(?!=)\s*(.*?);\s*", last, re.S)
    if m and not word.search(m.group(2)):
        return [(la, lb, m.group(1) + m.group(2))]
    if DIVERGES.match(last) and not [x for x in word.finditer(s, la, lb) if mask[x.start()]]:
        return []
    return None


def plan_if(s, mask, a, b, word, name):
    st = s[a:b].lstrip()
    pos = a + (len(s[a:b]) - len(st))
    edits = []
    assigned = False
    while True:
        blk = first_block(s, mask, pos + 2, b)
        if blk is None:
            return None
        if [x for x in word.finditer(s, pos, blk) if mask[x.start()]]:
            return None  # mentioned in a condition
        e = branch_edit(s, mask, blk, word, name)
        if e is None:
            return None
        assigned = assigned or bool(e)
        edits += e
        end = match_brace(s, mask, blk)
        rest = s[end + 1:b]
        m = re.match(r"\s*else\s+(if\b|\{)", rest)
        if not m:
            return None  # no final `else`: not every path assigns
        if m.group(1) == "{":
            blk2 = end + 1 + m.start(1)
            e = branch_edit(s, mask, blk2, word, name)
            end2 = match_brace(s, mask, blk2)
            if e is None or end2 is None or s[end2 + 1:b].strip():
                return None
            return edits + e if (assigned or e) else None
        pos = end + 1 + m.start(1)


def plan(s, mask, a, b, word, name):
    st = s[a:b].lstrip()
    if st.startswith("if "):
        return plan_if(s, mask, a, b, word, name)
    return None


total = 0
for f in sorted(glob.glob("engine/src/*.rs")):
    s = open(f).read()
    mask = code_mask(s)
    edits, taken = [], []
    for m in DECL.finditer(s):
        indent, mut, name, ty = m.group(1), m.group(2) or "", m.group(3), m.group(4).strip()
        if not mask[m.start() + len(indent)]:
            continue
        end = enclosing_end(s, mask, m.end())
        if end is None:
            continue
        word = re.compile(r"(?<![\w.])" + name + r"\b")
        inline = re.compile(r"\{" + name + r"[:}]")
        first = None
        for a, b in statements(s, mask, m.end(), end):
            if [x for x in word.finditer(s, a, b) if mask[x.start()]] or inline.search(s, a, b):
                first = (a, b)
                break
        if first is None:
            continue
        a, b = first
        if any(a < hi and lo < b for lo, hi in taken):
            continue
        e = plan(s, mask, a, b, word, name)
        if not e:
            continue
        taken.append((a, b))
        st = s[a:b]
        lead = len(st) - len(st.lstrip())
        edits.append((m.start(), m.end(), ""))
        edits.append((a + lead, a + lead, f"let {mut}{name}: {ty} = "))
        edits.append((b, b, ";"))
        edits += e
        total += 1
    if edits:
        print(f"{len(edits)} edits {f}")
        if "--apply" in sys.argv:
            for a, b, r in sorted(edits, key=lambda t: (t[0], t[1]), reverse=True):
                s = s[:a] + r + s[b:]
            open(f, "w").write(s)
print("total", total)
