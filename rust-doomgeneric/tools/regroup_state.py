"""Rewrite `state.<module>` to `state.<aggregate>.<module>` after grouping GameState's fields.

Run once from `rust-doomgeneric/` (the workspace root) after changing GROUPS and the
`GameState` struct in engine/src/game_state.rs by hand:

    python3 tools/regroup_state.py            # dry run: per-file counts
    python3 tools/regroup_state.py --apply

Only `state.` and `gs.` receivers are rewritten (code and comments; string literals are left
alone). Anything else that holds a `GameState` (`self` inside `impl GameState`, ...) is left
to the compiler, which names every remaining site.
"""
import glob
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rslex import region_mask

GROUPS = {
    "world": "p_setup p_mobj p_tick p_spec p_map p_maputl p_sight p_ceilng p_doors p_lights "
             "p_plats p_switch p_enemy p_pspr p_user p_saveg m_random",
    "render": "r_main r_segs r_draw r_data r_plane r_bsp r_things r_sky",
    "ui": "m_menu hu_stuff st_lib st_stuff wi_stuff am_map f_finale f_wipe statdump",
    "audio": "s_sound i_sound sounds",
    "assets": "w_wad w_checksum info fs",
    "game": "g_game doomstat d_main d_loop d_event d_iwad m_argv m_config m_controls",
    "io": "platform i_input i_joystick i_system i_timer i_video v_video",
}
GROUP_OF = {m: g for g, ms in GROUPS.items() for m in ms.split()}
# Other receivers (`s` in bind_variable closures, `self` in `impl GameState`) are only safe in
# the files where the compiler reports them: RECEIVERS=s FILES="d_main m_controls" ...
RECEIVERS = tuple(os.environ.get("RECEIVERS", "state,gs").split(","))
ONLY = os.environ.get("FILES", "").split()

gs_src = open("engine/src/game_state.rs").read()
declared = re.findall(r"^    pub (\w+): ", gs_src.split("pub struct GameState")[1].split("\n}")[0], re.M)
unassigned = [d for d in declared if d not in GROUP_OF and d not in GROUPS]
if unassigned:
    sys.exit(f"fields of GameState with no group: {unassigned}")

# rustfmt splits long chains (`state\n    .p_mobj\n    .mo(..)`), so allow whitespace around the dot.
rx = re.compile(r"(?:(?<![\w.])|(?<=\.\.))(" + "|".join(RECEIVERS) + r")(\s*)\.(\s*)(" + "|".join(sorted(GROUP_OF, key=len, reverse=True)) + r")\b")
total = 0
for f in sorted(glob.glob("engine/src/*.rs")):
    if ONLY and os.path.basename(f)[:-3] not in ONLY:
        continue
    s = open(f).read()
    reg = region_mask(s)
    out, last, n = [], 0, 0
    for m in rx.finditer(s):
        if reg[m.start()] == 2:  # inside a string/char literal
            continue
        out.append(s[last:m.start()])
        ws1, ws2 = m.group(2), m.group(3)
        out.append(f"{m.group(1)}{ws1}.{ws2}{GROUP_OF[m.group(4)]}{ws1}.{ws2}{m.group(4)}")
        last = m.end()
        n += 1
    if n:
        total += n
        print(f"{n:5d} {f}")
        if "--apply" in sys.argv:
            open(f, "w").write("".join(out) + s[last:])
print("total", total)
