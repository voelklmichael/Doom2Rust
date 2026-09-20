"""Turn counting `while` loops into `for` loops when that is provably the same.

Run from `rust-doomgeneric/` (the workspace root):

    python3 tools/while_to_for.py            # dry run: list what converts
    python3 tools/while_to_for.py --apply    # then `cargo build`, `cargo fmt`

Converts

    i = 0;                          for i in 0..n {
    while i < n {           ->          ...
        ...                         }
        i += 1;
    }

(also `<=` -> `..=`, and a preceding `let mut i: T = 0;`) when ALL of:

  * the statement just before the loop is `i = init;` / `let mut i[: T] = init;` in the same block;
  * the loop's last statement is `i += 1;` and there is no other write to `i` and no `continue`
    in the body (a `continue` would skip the increment);
  * `i` is not read again after the loop before being re-assigned;
  * the bound is a literal, an ALL_CAPS constant, or a local (optionally `as T`) that the body does
    not write, so evaluating it once is the same as evaluating it every iteration;
  * `i` is not mentioned in the bound.

A loop that fails any test is left alone. The type of `i` becomes whatever the range infers; the
compiler rejects the rare case where that differs.
"""
import glob
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rslex import code_mask, match_brace

WHILE = re.compile(r"(?m)^([ \t]*)while (\w+) (<=|<) ([^{;\n]+?) \{\n")
total = 0


def stmts(s, mask, start, end):
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


def enclosing(s, mask, pos):
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


def open_of(s, mask, pos):
    d = 0
    for j in range(pos - 1, -1, -1):
        if not mask[j]:
            continue
        if s[j] == "}":
            d += 1
        elif s[j] == "{":
            if d == 0:
                return j
            d -= 1
    return None


for f in sorted(glob.glob("engine/src/*.rs")):
    s = open(f).read()
    mask = code_mask(s)
    edits = []
    for m in WHILE.finditer(s):
        indent, var, op, bound = m.group(1), m.group(2), m.group(3), m.group(4).strip()
        if not mask[m.start() + len(indent)]:
            continue
        b = m.end() - 2
        e = match_brace(s, mask, b)
        if e is None:
            continue
        body = s[b + 1:e]
        word = re.compile(r"(?<![\w.])" + var + r"\b")
        # last statement is `var += 1;`, nothing else writes var, no continue
        body_stmts = stmts(s, mask, b + 1, e)
        if not body_stmts:
            continue
        la, lb = body_stmts[-1]
        if not re.fullmatch(r"\s*" + var + r"\s*(\+=\s*1|=\s*" + var + r"\.wrapping_add\(1\))\s*;", s[la:lb]):
            continue
        # `for` is not allowed in a `const fn`
        head_fn = list(re.finditer(r"(?m)^\s*(?:pub(?:\([a-z]+\))? )?(const )?fn \w+", s[:m.start()]))
        if head_fn and head_fn[-1].group(1):
            continue
        rest_body = s[b + 1:la]
        if re.search(r"(?<![\w.])" + var + r"\s*(\+|-|\*|/|%|<<|>>|&|\||\^)?=(?!=)", rest_body) or re.search(r"&mut\s+" + var + r"\b", rest_body) or re.search(r"\bcontinue\b", rest_body):
            continue
        # bound: literal / CONST / plain local (optionally cast), not mentioning var, not written in the body
        core = re.sub(r"\s+as\s+\w+$", "", bound).strip()
        if word.search(bound):
            continue
        if not (re.fullmatch(r"-?\d+", core) or re.fullmatch(r"[A-Z][A-Z0-9_]*", core) or re.fullmatch(r"[a-z_]\w*", core)):
            continue
        if re.fullmatch(r"[a-z_]\w*", core) and re.search(r"(?<![\w.])" + core + r"\s*(\+|-|\*|/|%)?=(?!=)|&mut\s+" + core + r"\b", body):
            continue
        # the statement just before the loop initialises var
        block_open = open_of(s, mask, m.start())
        block_end = enclosing(s, mask, m.start())
        if block_open is None or block_end is None:
            continue
        # `for` is not allowed in a `const` / `static` initialiser either
        outer, in_const = block_open, False
        while outer is not None:
            line_start = s.rfind("\n", 0, outer) + 1
            if re.match(r"\s*(pub(?:\([a-z]+\))? )?(const|static)\s+[A-Z_]", s[line_start:outer]):
                in_const = True
                break
            outer = open_of(s, mask, outer)
        if in_const:
            continue
        sts = stmts(s, mask, block_open + 1, block_end)
        idx = next((k for k, (a, bb) in enumerate(sts) if a <= m.start() < bb or s[a:bb].lstrip().startswith("while") and a <= m.start() + len(indent) <= bb), None)
        if idx is None or idx == 0:
            continue
        pa, pb = sts[idx - 1]
        prev = s[pa:pb]
        init = re.fullmatch(r"\s*(?:let\s+mut\s+" + var + r"(?::\s*[\w<>\[\]; ]+)?|" + var + r")\s*=\s*([^;]+?)\s*;", prev, re.S)
        if not init:
            continue
        # var must not be read after the loop before being re-assigned
        after_ok = True
        for a, bb in sts[idx + 1:]:
            t = s[a:bb]
            if not [x for x in word.finditer(t) if mask[a + x.start()]]:
                continue
            if not re.match(r"\s*" + var + r"\s*=(?!=)", t) or word.search(re.sub(r"^\s*" + var + r"\s*=", "", t, count=1)):
                after_ok = False
            break
        if not after_ok:
            continue
        rng = f"{init.group(1)}..{'=' if op == '<=' else ''}{bound}"
        head = f"{indent}for {var} in {rng} {{\n"
        edits.append((pa, pb, ""))  # drop the init statement
        edits.append((m.start(), m.end(), head))
        edits.append((la, lb, ""))  # drop `i += 1;`
        total += 1
    if edits:
        n = len(edits) // 3
        print(f"{n:3d} {f}")
        if "--apply" in sys.argv:
            for a, b, r in sorted(edits, reverse=True):
                s = s[:a] + r + s[b:]
            open(f, "w").write(s)
print("total", total)
