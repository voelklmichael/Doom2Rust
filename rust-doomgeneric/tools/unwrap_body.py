"""Remove the redundant `{ ... }` block that some functions wrap their whole body in.

Run from `rust-doomgeneric/` (the workspace root):

    python3 tools/unwrap_body.py            # dry run
    python3 tools/unwrap_body.py --apply    # then `cargo fmt`

The C-to-Rust conversion left many functions shaped like

    pub fn chase(state: &mut GameState, id: MobjId) {
        {
            let actor = id;
            ...
        }
    }

The inner block does nothing. This drops it and, when its first statement is a plain alias
`let actor = id;` of a parameter, renames the parameter to the alias (`actor` is the name the
body already uses) instead of keeping both. A function is skipped when the rename could change
meaning: the parameter is re-bound (`let id = ...`) or the alias name is already a parameter.
"""
import glob
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rslex import code_mask, match_brace

FN = re.compile(r"(?m)^(?:pub(?:\(crate\))? )?fn (\w+)[^{;]*?\{\n")
ALIAS = re.compile(r"\s*let\s+(\w+)\s*=\s*(\w+)\s*;[ \t]*\n?")
unwrapped = renamed = skipped = 0
for f in sorted(glob.glob("engine/src/*.rs")):
    s = open(f).read()
    mask = code_mask(s)
    out, last, before = [], 0, unwrapped
    for h in FN.finditer(s):
        b = h.end() - 2
        if b < last or s[b] != "{":
            continue
        try:
            e = match_brace(s, mask, b)
        except Exception:
            continue
        inner = s[b + 1:e]
        stripped = inner.lstrip()
        if not stripped.startswith("{"):
            continue
        k = b + 1 + (len(inner) - len(stripped))
        try:
            e2 = match_brace(s, mask, k)
        except Exception:
            continue
        if s[e2 + 1:e].strip() or s[b + 1:k].strip():
            continue  # something besides the one block
        header = s[h.start():b]
        block = s[k + 1:e2]
        new_header = header
        a = ALIAS.match(block)
        if a:
            alias, param = a.group(1), a.group(2)
            has_param = re.search(r"[(,]\s*(?:mut\s+)?" + param + r"\s*:", header)
            alias_taken = re.search(r"\b" + alias + r"\s*:", header)
            rebound = re.search(r"\blet\s+(?:mut\s+)?" + param + r"\b", block[a.end():])
            if has_param and not alias_taken and not rebound:
                rest = block[a.end():]
                # rename `param` -> `alias`: identifiers, `{param}` format args, not fields/paths
                rest = re.sub(r"(?<![\w.:])" + param + r"\b(?!\s*:(?!:))", alias, rest)
                rest = re.sub(r"\{" + param + r"(?=[:}])", "{" + alias, rest)
                new_header = re.sub(r"([(,]\s*(?:mut\s+)?)" + param + r"(\s*:)", r"\g<1>" + alias + r"\2", header, count=1)
                block = rest
                renamed += 1
            else:
                skipped += 1
                continue
        out.append(s[last:h.start()])
        out.append(new_header + "{\n" + block.strip("\n") + "\n")
        last = e + 0
        unwrapped += 1
    out.append(s[last:])
    if "--apply" in sys.argv and unwrapped > before:
        open(f, "w").write("".join(out))
print(f"unwrapped {unwrapped} (renamed a parameter in {renamed}), skipped {skipped}")
