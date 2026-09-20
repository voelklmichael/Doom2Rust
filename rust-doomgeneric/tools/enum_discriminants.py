"""Drop enum discriminants that Rust would assign anyway.

Run from `rust-doomgeneric/` (the workspace root):

    python3 tools/enum_discriminants.py            # dry run: what would go, what stays
    python3 tools/enum_discriminants.py --apply    # then `cargo build`, `cargo test`, `cargo fmt`

`--apply --probe` also appends a temporary test module to each changed file that asserts every
variant's OLD value against the new enum (`cargo test discriminant_probe`); `--strip-probe` removes
them again. That is an independent check that no value changed.

The C enums were ported as `Nothing = 0, LoadLevel = 1, NewGame = 2, ...`. Rust numbers variants
0, 1, 2, ... by itself, so `= n` is only information when it differs from the running value (a gap,
a negative, a bit flag, the `Noammo = 5` after `Misl = 3`). This removes each `= n` that equals the
value the variant would get without it, and keeps the rest, so an enum with a real gap ends up with
only the numbers that matter. The values of every variant are unchanged.

Skipped, so nothing is guessed: enums whose variants carry data, and everything after a
discriminant that is not a plain integer literal (`= 1 << 3`, `= OTHER as isize`).
"""
import glob
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rslex import code_mask, match_brace

# top-level enums only (column 0): one nested in a `mod`/fn is out of the probe's sight
ENUM = re.compile(r"(?m)^(?:pub(?:\([a-z]+\))? )?enum (\w+)\b[^{;]*\{")
EXPLICIT = re.compile(r"^(\s*)(\w+)\s*=\s*(-?(?:0[xX][0-9a-fA-F_]+|[0-9][0-9_]*))\s*(,)?(\s*(?://.*)?)$")
PLAIN = re.compile(r"^\s*(\w+)\s*,?\s*(?://.*)?$")
SKIPPABLE = re.compile(r"^\s*(//.*|#\[.*\]|)$")

removed = kept = enums_changed = 0
report = []
probes = {}
for f in sorted(glob.glob("engine/src/*.rs")):
    s = open(f).read()
    mask = code_mask(s)
    out, last = [], 0
    changed_here = False
    for m in ENUM.finditer(s):
        if not mask[m.start()]:
            continue
        b = m.end() - 1
        e = match_brace(s, mask, b)
        if e is None:
            continue
        body = s[b + 1:e]
        lines = body.split("\n")
        expected, known, new_lines, rem, kp = 0, True, [], 0, 0
        data_variant = False
        olds = []  # (variant, value) for every variant whose value is known
        for ln in lines:
            if SKIPPABLE.match(ln) or PLAIN.match(ln) and "(" not in ln and "{" not in ln:
                pm = PLAIN.match(ln)
                if pm and not SKIPPABLE.match(ln):
                    if known:
                        olds.append((pm.group(1), expected))
                    expected += 1
                new_lines.append(ln)
                continue
            em = EXPLICIT.match(ln)
            if em:
                value = int(em.group(3).replace("_", ""), 0)
                olds.append((em.group(2), value))
                if known and value == expected:
                    comma = em.group(4) or ""
                    new_lines.append(f"{em.group(1)}{em.group(2)}{comma}{em.group(5)}")
                    rem += 1
                else:
                    new_lines.append(ln)
                    kp += 1
                expected = value + 1
                known = True
                continue
            if "(" in ln or "{" in ln:
                data_variant = True
                break
            if "=" in ln:  # an expression discriminant: stop guessing for the rest
                known = False
                new_lines.append(ln)
                kp += 1
                continue
            new_lines.append(ln)
        if data_variant or not rem:
            continue
        out.append(s[last:b + 1])
        out.append("\n".join(new_lines))
        last = e
        removed += rem
        kept += kp
        probes.setdefault(f, []).append((m.group(1), olds))
        enums_changed += 1
        changed_here = True
        report.append((os.path.basename(f), m.group(1), rem, kp))
    if changed_here:
        out.append(s[last:])
        text = "".join(out)
        if "--probe" in sys.argv:
            checks = "\n".join(
                f"        assert_eq!({n}::{v} as i128, {val}, \"{n}::{v}\");" for n, olds in probes[f] for v, val in olds
            )
            text += f"\n#[cfg(test)]\nmod discriminant_probe_tmp {{\n    use super::*;\n    #[test]\n    fn old_values_are_unchanged() {{\n{checks}\n    }}\n}}\n"
        if "--apply" in sys.argv:
            open(f, "w").write(text)
if "--strip-probe" in sys.argv:
    for f in glob.glob("engine/src/*.rs"):
        t = open(f).read()
        t2 = re.sub(r"\n#\[cfg\(test\)\]\nmod discriminant_probe_tmp \{.*\n\}\n\Z", "\n", t, flags=re.S)
        if t2 != t:
            open(f, "w").write(t2.rstrip("\n") + "\n")
    sys.exit()
for f, name, rem, kp in report:
    if kp:
        print(f"  keeps {kp:3d} of {rem + kp:4d}: {f}::{name}")
print(f"enums changed: {enums_changed}, discriminants removed: {removed}, kept (they matter): {kept}")
