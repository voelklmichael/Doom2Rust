"""Tighten `&mut T` parameters that clippy::needless_pass_by_ref_mut says are only read.

Run from `rust-doomgeneric/` (the workspace root):

    python3 tools/tighten_mut.py

`cargo clippy --fix` does not apply this lint, so after `narrow_state.py` (which keeps the
declared mutability) this applies clippy's own suggestion (drop the `mut`), then rewrites
the matching argument at every call site from `&mut x` to `&x`, and repeats until clippy is
quiet: tightening a callee often lets its callers tighten too.

Only free functions with a unique name get their call sites rewritten; a parameter of a
method (or of a name defined twice) is tightened and its callers are left to the compiler,
which accepts `&mut` where `&` is expected.
"""
import glob
import json
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rslex import code_mask

PKG = os.environ.get("PKG", "rust_doomgeneric")
FILES = sorted(glob.glob("engine/src/*.rs"))


def clippy_suggestions():
    out = subprocess.run(
        ["cargo", "clippy", "-p", PKG, "--release", "--message-format=json"],
        capture_output=True, text=True,
    ).stdout
    found = {}
    for line in out.splitlines():
        try:
            msg = json.loads(line)
        except ValueError:
            continue
        if msg.get("reason") != "compiler-message":
            continue
        m = msg["message"]
        if (m.get("code") or {}).get("code") != "clippy::needless_pass_by_ref_mut":
            continue
        for child in m.get("children", []):
            for sp in child.get("spans", []):
                if sp.get("suggested_replacement") is not None:
                    found[(sp["file_name"], sp["byte_start"], sp["byte_end"])] = sp["suggested_replacement"]
    return found


def split_args(s, mask, start):
    """Given `s[start]` == '(', return (end, [(arg_start, arg_end), ...]) of its top-level args."""
    depth, args, cur = 0, [], start + 1
    for j in range(start, len(s)):
        if not mask[j]:
            continue
        c = s[j]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                if s[cur:j].strip():
                    args.append((cur, j))
                return j, args
        elif c == "," and depth == 1:
            args.append((cur, j))
            cur = j + 1
    raise ValueError("unbalanced")


def param_index(text, span_start):
    """Index of the parameter containing `span_start` in the fn whose header precedes it,
    plus the fn's name (None for methods)."""
    head = None
    for m in re.finditer(r"\bfn (\w+)(?:<[^>]*>)?\(", text[:span_start]):
        head = m
    if head is None:
        return None, None
    params = text[head.end():span_start]
    depth = idx = 0
    for c in params:
        if c in "(<[":
            depth += 1
        elif c in ")>]":
            depth -= 1
        elif c == "," and depth == 0:
            idx += 1
    if re.match(r"\s*&?\s*(mut )?self\b", text[head.end():]):
        return None, None
    return idx, head.group(1)


def main():
    rounds = 0
    while True:
        sugg = clippy_suggestions()
        if not sugg:
            print(f"clean after {rounds} round(s)")
            return
        rounds += 1
        by_file = {}
        for (f, a, b), rep in sugg.items():
            by_file.setdefault(f, []).append((a, b, rep))
        tightened = []  # (fn name, arg index)
        for f, edits in by_file.items():
            raw = open(f, "rb").read()
            text = raw.decode()
            for a, b, rep in sorted(edits, reverse=True):
                char_a = len(raw[:a].decode())
                idx, name = param_index(text, char_a)
                if name:
                    tightened.append((name, idx))
                raw = raw[:a] + rep.encode() + raw[b:]
                text = raw.decode()
            open(f, "wb").write(raw)
        src = {f: open(f).read() for f in FILES}
        defs = {}
        for s in src.values():
            for n in re.findall(r"(?m)^\s*(?:pub(?:\([a-z]+\))? )?fn (\w+)", s):
                defs[n] = defs.get(n, 0) + 1
        for name, idx in set(tightened):
            if defs.get(name) != 1:
                continue
            rx = re.compile(r"(?<![\w.])" + re.escape(name) + r"\s*\(")
            for f in FILES:
                s = src[f]
                mask = code_mask(s)
                edits = []
                for m in rx.finditer(s):
                    if not mask[m.start()] or re.search(r"\bfn\s+$", s[max(0, m.start() - 4):m.start()]):
                        continue
                    end, args = split_args(s, mask, m.end() - 1)
                    if idx < len(args):
                        a, b = args[idx]
                        seg = s[a:b]
                        k = re.match(r"\s*&mut ", seg)
                        if k:
                            edits.append((a + k.end() - 4, a + k.end()))
                for a, b in sorted(edits, reverse=True):
                    s = s[:a] + s[b:]
                if edits:
                    src[f] = s
        for f, s in src.items():
            open(f, "w").write(s)
        print(f"round {rounds}: tightened {len(sugg)} parameter(s)")


main()
