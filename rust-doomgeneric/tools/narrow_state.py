"""Narrow functions that take `state: &mut GameState` to just the subsystems they use.

Run from `rust-doomgeneric/` (the workspace root):

    python3 tools/narrow_state.py            # dry run: list candidates
    python3 tools/narrow_state.py --apply

`GameState` owns a few domain aggregates (`World`, `Render`, ...), each of which owns
per-module states, so a use is a path `state.<aggregate>.<module>`. A function is a
candidate when it never uses `state` as a whole (passing it on, calling a method on it) and
either

  * touches at most MAXF modules (default 3): it gets one `&mut`/`&` parameter per module, or
  * touches more modules but all inside a single aggregate: it gets that aggregate as its one
    parameter (`world: &mut World`).

Callers are rewritten to pass `&mut state.<aggregate>.<module>` / `&mut state.<aggregate>`.
Names used as values (callbacks), defined twice, or whose parameter would shadow a function
they call are skipped. Narrowing a leaf makes its callers narrower, so re-run until it reports
0 candidates, then `python3 tools/tighten_mut.py` (the tool keeps the declared mutability),
fix what the compiler reports (usually a call argument that reads a field the call also
borrows: hoist it into a local) and run `cargo fmt`. See docs/gamestate-decomposition.md.
"""
import collections
import glob
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rslex import code_mask, match_brace

HERE = os.path.dirname(os.path.abspath(__file__))
# Boxed trait-object fields: (parameter type, module of the trait). Call sites reborrow with `*`.
DYN_FIELDS = {"fs": ("dyn DoomFileSystem", "filesystem"), "platform": ("dyn DoomPlatform", "platform")}
SKIP_FIELDS = set(filter(None, os.environ.get("SKIP", "").split(",")))
MAXF = int(os.environ.get("MAXF", "3"))
MAXARGS = 7  # clippy::too_many_arguments

gs = open("engine/src/game_state.rs").read()
structs = {n: dict(re.findall(r"    pub (\w+): ([^,]+),", b)) for n, b in re.findall(r"pub struct (\w+) \{\n(.*?)\n\}", gs, re.S)}
AGG = structs["GameState"]  # aggregate field -> aggregate type
MOD = {}  # module field -> (aggregate field, state type)
for a, T in AGG.items():
    for m, t in structs[T].items():
        MOD[m] = (a, DYN_FIELDS[m][0] if m in DYN_FIELDS else t)
blacklist = set(json.load(open(HERE + "/narrow_blacklist.json"))) if os.path.exists(HERE + "/narrow_blacklist.json") else set()

TESTF = "engine/src/regression_tests.rs"
files = [f for f in sorted(glob.glob("engine/src/*.rs")) if not f.endswith(("regression_tests.rs", "game_state.rs"))]
src = {f: open(f).read() for f in files + [TESTF]}
allsrc = "\n".join(src.values())

# `use` statements (including multi-line brace lists) and comments never make a name a value.
valsrc = re.sub(r"(?ms)^[ \t]*(?:pub(?:\([a-z]+\))? )?use [^;]*;", lambda m: re.sub(r"[^\n]", "", m.group(0)), allsrc)
valsrc = re.sub(r"//[^\n]*", "", valsrc)


def used_as_value(name):
    """True if `name` appears as an identifier that is not called and not a fn definition."""
    for m in re.finditer(r"(?<![\w.])" + re.escape(name) + r"\b(?!\s*\()", valsrc):
        if valsrc[max(0, m.start() - 4):m.start()].endswith("fn "):
            continue
        return True
    return False


PRE = r"(?:(?<![\w.])|(?<=\.\.))"  # not a field access, but `0..state` is fine
STATE = re.compile(PRE + r"state\b(?:\s*\.\s*(\w+)(?:\s*\.\s*(\w+))?)?")


def uses(body, mask, base):
    """(set of modules, set of bare aggregates, uses_state_whole) for a fn body."""
    mods, aggs, whole = set(), set(), False
    for m in STATE.finditer(body):
        if not mask[base + m.start()]:
            continue
        a, mod = m.group(1), m.group(2)
        if a in AGG and mod in MOD and MOD[mod][0] == a:
            mods.add(mod)
        elif a in AGG:
            aggs.add(a)
        else:
            whole = True  # `state` alone, or a method such as state.screen()
    return mods, aggs, whole


HEAD = r"(?m)^(pub(?:\(crate\))? )?fn (\w+)(?:<[^>]*>)?\(\s*state: &(mut )?GameState"
cands = {}  # name -> (file, kind, names, is_mut)
for f in files:
    s = src[f]
    m = code_mask(s)
    for h in re.finditer(HEAD, s):
        name = h.group(2)
        if name in blacklist:
            continue
        b = s.index("{", s.index(")", h.end()))
        try:
            e = match_brace(s, m, b)
        except Exception:
            continue
        body = s[b:e]
        mods, aggs, whole = uses(body, m, b)
        if whole or not (mods or aggs):
            continue
        used_aggs = {MOD[x][0] for x in mods} | aggs
        if any(x in SKIP_FIELDS for x in mods):
            continue
        if len(mods) <= MAXF and not aggs:
            kind, names = "modules", tuple(sorted(mods))
        elif len(used_aggs) == 1:
            kind, names = "aggregate", tuple(used_aggs)
        else:
            continue
        # A parameter named like a module would shadow a function of that name the body calls,
        # or clash with an existing parameter.
        header = s[h.start():b]
        if kind == "modules" and any(re.search(r"(?<![\w.])" + n + r"\s*\(", body) or re.search(r"\b" + n + r"\s*:", header) for n in names):
            continue
        # clippy::too_many_arguments: keep the whole state rather than trip it. Counting commas
        # over-counts generic parameters, which only makes this more conservative.
        extra = [x for x in s[h.end():s.index(")", h.end())].split(",") if x.strip()]
        if len(extra) + len(names) > MAXARGS:
            continue
        cands[name] = (f, kind, names, bool(h.group(3)))
# drop those used as values, and those defined twice
cands = {n: v for n, v in cands.items() if not used_as_value(n)}
cnt = collections.Counter(re.findall(r"(?m)^(?:pub(?:\(crate\))? )?fn (\w+)", allsrc))
cands = {n: v for n, v in cands.items() if cnt[n] == 1}
print("candidates", len(cands), "(aggregate:", sum(v[1] == "aggregate" for v in cands.values()) + 0, ")")
if "--apply" not in sys.argv:
    for n, (f, kind, names, mu) in sorted(cands.items()):
        print(" ", f.split("/")[-1], n, kind[:3], ",".join(names), "mut" if mu else "")
    sys.exit()


def param_type(kind, name):
    return AGG[name] if kind == "aggregate" else MOD[name][1]


needed = set()  # (file, type, import module)
for n, (f, kind, names, mu) in cands.items():
    s = src[f]
    m = code_mask(s)
    h = re.search(r"(?m)^(pub(?:\(crate\))? )?fn " + n + r"(?:<[^>]*>)?\(\s*state: &(mut )?GameState", s)
    b = s.index("{", s.index(")", h.end()))
    e = match_brace(s, m, b)
    body = s[b:e + 1]
    if kind == "aggregate":
        a = names[0]
        end = r"(?![\w]|\s*\.)"
        body = re.sub(r"&mut\s+state\s*\.\s*" + a + end, a, body)
        body = re.sub(r"&\s*state\s*\.\s*" + a + end, a, body)
        body = re.sub(PRE + r"state\s*\.\s*" + a + r"\b", a, body)
    else:
        for x in names:
            a = MOD[x][0]
            # `&mut state.a.m` is exactly the parameter, but `&state.a.m.field` borrows the field.
            end = r"(?![\w]|\s*\.)"
            body = re.sub(r"&mut\s+state\s*\.\s*" + a + r"\s*\.\s*" + x + end, x, body)
            body = re.sub(r"&\s*state\s*\.\s*" + a + r"\s*\.\s*" + x + end, x, body)
            body = re.sub(PRE + r"state\s*\.\s*" + a + r"\s*\.\s*" + x + r"(?![\w])", x, body)
    kw = "&mut " if mu else "&"
    params = ", ".join(f"{x}: {kw}{param_type(kind, x)}" for x in names)
    head = s[h.start():h.end()].replace("state: &mut GameState", params).replace("state: &GameState", params)
    src[f] = s[:h.start()] + head + s[h.end():b] + body + s[e + 1:]
    for x in names:
        needed.add((f, param_type(kind, x), "game_state" if kind == "aggregate" else (DYN_FIELDS[x][1] if x in DYN_FIELDS else x)))

# callers
for n, (f0, kind, names, mu) in cands.items():
    def arg(x):
        if kind == "aggregate":
            return ("&mut " if mu else "&") + "state." + x
        return ("&mut " if mu else "&") + ("*" if x in DYN_FIELDS else "") + "state." + MOD[x][0] + "." + x
    args = ", ".join(arg(x) for x in names)
    rx = re.compile(r"(?<![\w.])" + n + r"\(\s*state\b(?!\s*\.)")
    for f in files + [TESTF]:
        s = src[f]
        s2 = rx.sub(lambda m: m.group(0)[:-5] + args, s)
        if s2 != s:
            src[f] = s2

# imports for the parameter types
for f, T, mod in sorted(needed):
    s = src[f]
    T = T.split(" ")[-1]  # `dyn DoomFileSystem` -> `DoomFileSystem`
    already = re.search(r"use crate::\w+::[^;]*\b" + T + r"\b", s) or re.search(r"(?m)^(pub )?(struct|trait) " + T + r"\b", s)
    if not already:
        s = re.sub(r"(?m)^use [^\n]*\n", lambda mm: f"use crate::{mod}::{T};\n" + mm.group(0), s, count=1)
        src[f] = s
for f, s in src.items():
    open(f, "w").write(s)
print("applied", len(cands))
