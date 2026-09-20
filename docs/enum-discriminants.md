# Enums that still spell out numbers, and why

Rust numbers the variants of an enum 0, 1, 2, ... by itself, so `= n` says something only where it *differs* from that running value. `tools/enum_discriminants.py` removed every redundant number from the engine (1,643 of them), and two more turned out never to be observed and were dropped by hand (`StateEnum::NoState = -1`, and `SpriteRotate::Unset = -1`, whose one integer `match` is now a match on the variants); `SkillType::Noitems = -1` was replaced by an `Option`. What is left are the enums below, where a number does work.

**How the sites were found.** `tools/enum_number_uses.py` temporarily gives an enum a data-carrying variant, which turns every `enum as i32` / `as usize` / `as u8` cast of that type into a compile error, and reads the positions off the compiler. So the "where the number is used" lists are exact for casts. The integer-to-enum direction (`*_from_raw`) is listed separately, and comparing or matching variants is not a use of the number. Line numbers are as of the commit that added this file; regenerate with the tool if the code has moved.

In every case below the number is required for one of two reasons: it is **written to or read from an external format** that vanilla Doom defines (savegames, demo files, the ticcmd byte, net game settings), or it is **used as arithmetic** (an index or a step).

---

## `SkillType` — `Baby = 0` ([d_mode.rs:5](../rust-doomgeneric/engine/src/d_mode.rs#L5))

**Why it is spelled out.** This one is a choice, not a necessity: `Baby` is 0 by Rust's numbering anyway. It stays written because the five values 0..=4 are what every external format stores, and the enum no longer has vanilla's `sk_noitems = -1` "no skill" variant that used to make `Baby` 0 (see below). The `skill_levels_keep_the_values_the_file_formats_store` test in `d_mode.rs` pins them.

**The `Noitems` sentinel is gone.** It was only ever produced by `-skill 0` (`argv[p+1][0] - '1'` = -1), which then reached `gameskill` and made `P_SpawnMapThing` shift by a negative amount. "No skill" is now `None` from [`SkillType::from_raw`](../rust-doomgeneric/engine/src/d_mode.rs#L16), and `-skill` outside 1..=5 stops with an error (`docs/known-deviations.md`).

**Where the number is used:**

- [g_game.rs:1638](../rust-doomgeneric/engine/src/g_game.rs#L1638) — the demo file header byte (`gameskill as u8`).
- [p_saveg.rs:696](../rust-doomgeneric/engine/src/p_saveg.rs#L696) — the savegame header byte (`gameskill as u8`).
- [d_net.rs:84](../rust-doomgeneric/engine/src/d_net.rs#L84) — the net game settings `skill` field (`startskill as i32`); [d_net.rs:169](../rust-doomgeneric/engine/src/d_net.rs#L169) prints it.
- [p_mobj.rs:3669](../rust-doomgeneric/engine/src/p_mobj.rs#L3669) — `1 << (gameskill - 1)` builds the "spawn on this skill" bit of a map thing's options (Easy/Medium/Hard are bits 0..2; `Baby` and `Nightmare` are special-cased just above).
- Integer → enum, [`skill_from_raw`](../rust-doomgeneric/engine/src/d_mode.rs#L30) (panics on an invalid value; a thin wrapper over `SkillType::from_raw`). Callers: [p_saveg.rs:740](../rust-doomgeneric/engine/src/p_saveg.rs#L740), [g_game.rs:1695](../rust-doomgeneric/engine/src/g_game.rs#L1695), [d_net.rs:63](../rust-doomgeneric/engine/src/d_net.rs#L63), [m_menu.rs:994](../rust-doomgeneric/engine/src/m_menu.rs#L994), [m_menu.rs:1012](../rust-doomgeneric/engine/src/m_menu.rs#L1012).
- The `-skill` argument uses `SkillType::from_raw` directly: [d_main.rs:975](../rust-doomgeneric/engine/src/d_main.rs#L975).

## `AmmoType` — `Noammo = 5` ([d_player.rs:14](../rust-doomgeneric/engine/src/d_player.rs#L14))

**Why the `5` is necessary.** In C, `NUMAMMO` (= 4) is an enumerator *between* `am_misl` and `am_noammo`, so `am_noammo` is 5, not 4. The number is observed as an out-of-range value in three ways, and one of them is a vanilla quirk the port reproduces exactly:

- **`decrease_ammo` overflow.** [`decrease_ammo`](../rust-doomgeneric/engine/src/p_pspr.rs#L380) does `if ammonum < NUMAMMO { ammo[ammonum] } else { maxammo[ammonum - NUMAMMO] }`. For `Noammo` (5) that is `maxammo[1]`: in the C `player_t`, `ammo[5]` overlaps `maxammo[1]`, and vanilla really does decrement it. The call sites pass `weapon.ammo as i32`: [p_pspr.rs:390](../rust-doomgeneric/engine/src/p_pspr.rs#L390), [p_pspr.rs:396](../rust-doomgeneric/engine/src/p_pspr.rs#L396), [p_pspr.rs:407](../rust-doomgeneric/engine/src/p_pspr.rs#L407), [p_pspr.rs:445](../rust-doomgeneric/engine/src/p_pspr.rs#L445), [p_pspr.rs:464](../rust-doomgeneric/engine/src/p_pspr.rs#L464), [p_pspr.rs:481](../rust-doomgeneric/engine/src/p_pspr.rs#L481), [p_pspr.rs:508](../rust-doomgeneric/engine/src/p_pspr.rs#L508), [p_pspr.rs:514](../rust-doomgeneric/engine/src/p_pspr.rs#L514).
- **The bad-type check.** [p_inter.rs:92](../rust-doomgeneric/engine/src/p_inter.rs#L92) `if ammo > NUMAMMO { error("P_GiveAmmo: bad type ...") }` (the number is printed at [p_inter.rs:93](../rust-doomgeneric/engine/src/p_inter.rs#L93)); [p_inter.rs:114](../rust-doomgeneric/engine/src/p_inter.rs#L114) `match ammo { 0 => .., 1 => .., 2 => .., 3 => .. }` picks the weapon to switch to by ammo number.
- **Array index.** [d_player.rs:19](../rust-doomgeneric/engine/src/d_player.rs#L19) (`ArrayIndex::slot`) indexes the 4-slot `ammo` / `maxammo` tables with the discriminant, so `Noammo` is out of range there, as it is in C.
- Integer → enum, [`ammotype_from_raw`](../rust-doomgeneric/engine/src/d_player.rs#L22) (only caller: [p_inter.rs:553](../rust-doomgeneric/engine/src/p_inter.rs#L553), the "give every ammo" cheat loop).

## `WeaponType` — `Nochange = 10` ([d_player.rs:82](../rust-doomgeneric/engine/src/d_player.rs#L82))

**Why the `10` is necessary.** Same story as ammo: `NUMWEAPONS` (= 9) sits between `wp_supershotgun` and `wp_nochange`, so "no pending weapon change" is 10 and 0..8 are the nine weapons. Those numbers are external:

- **Savegame.** [p_saveg.rs:437](../rust-doomgeneric/engine/src/p_saveg.rs#L437) and [p_saveg.rs:438](../rust-doomgeneric/engine/src/p_saveg.rs#L438) write `readyweapon` / `pendingweapon` as 32-bit ints; `pendingweapon` is `Nochange` almost always, so a vanilla-compatible save has a `10` there. Read back with [`weapontype_from_raw`](../rust-doomgeneric/engine/src/d_player.rs#L90) ([p_saveg.rs:381](../rust-doomgeneric/engine/src/p_saveg.rs#L381), [p_saveg.rs:382](../rust-doomgeneric/engine/src/p_saveg.rs#L382); `10 → Nochange`).
- **The ticcmd.** [g_game.rs:478](../rust-doomgeneric/engine/src/g_game.rs#L478) returns the chosen weapon's number from [`g_next_weapon`](../rust-doomgeneric/engine/src/g_game.rs#L452); the caller puts it in the buttons byte as `weapon << BT_WEAPONSHIFT` ([g_game.rs:594](../rust-doomgeneric/engine/src/g_game.rs#L594)), which is the demo and network wire format. [p_user.rs:244](../rust-doomgeneric/engine/src/p_user.rs#L244) reads the weapon number back out of the ticcmd with `weapontype_from_raw`.
- **Status bar.** [st_stuff.rs:755](../rust-doomgeneric/engine/src/st_stuff.rs#L755) and [st_stuff.rs:1048](../rust-doomgeneric/engine/src/st_stuff.rs#L1048) store the ready weapon's number in the "ready weapon" widget (`w_ready.data`).
- **Array index.** [d_player.rs:87](../rust-doomgeneric/engine/src/d_player.rs#L87) (`ArrayIndex::slot`) indexes `weaponowned[9]` and `WEAPONINFO[9]` with the discriminant; `Nochange` is out of range there, as in C.

## `WeaponCycle` — `Previous = -1`, `Next = 1` ([g_game.rs:448](../rust-doomgeneric/engine/src/g_game.rs#L448))

**Why the numbers are necessary.** They are arithmetic: [g_game.rs:468](../rust-doomgeneric/engine/src/g_game.rs#L468) does `i += direction as i32` to step through the weapon order table, then wraps with `% len`. `-1` and `+1` *are* the step.

**Could it go?** Yes, by replacing the cast with a small `fn step(self) -> i32` and dropping the numbers. It is kept as is because here the numbers are the meaning.

## `Direction` — `Down = -1` ([p_spec.rs:252](../rust-doomgeneric/engine/src/p_spec.rs#L252))

**Why the `-1` is necessary.** `Direction` is the `direction` of a door, ceiling, floor or glow. Making `Down` -1 gives `Still = 0`, `Up = 1`, `InitialWait = 2`, exactly the values vanilla writes into a savegame.

- [`Direction::to_save`](../rust-doomgeneric/engine/src/p_spec.rs#L265) (`self as i32`) is the only place the value is observed; the movers' savegame writers call it (`saveg_write32(state, str.direction.to_save())` in `p_saveg.rs`).
- Integer → enum, [`Direction::from_save`](../rust-doomgeneric/engine/src/p_spec.rs#L270): `-1 → Down`, `1 → Up`, `2 → InitialWait`, anything else `Still` (a deliberate deviation, see `docs/known-deviations.md`).
- A unit test, `direction_tests` at the bottom of `p_spec.rs`, pins all four values.

## `Colour` (test only) — `Red = 0`, `Blue = 2` ([enum_array.rs:132](../rust-doomgeneric/engine/src/enum_array.rs#L132))

A throwaway enum inside the tests of `enum_array.rs`. The gap is the point: the test indexes an `EnumArray` by `Colour::Blue` and expects slot 2, and checks that a key beyond the table panics like a bad index.

---

## Appendix A — enums with no number written, but whose order is still observed

Removing the `= n` did not remove the dependence where the value is cast: these enums have no explicit number, so **the order of their variants is the number**. Do not reorder them without checking the sites. (Most are C index tables: the `info.rs` `StateNum` / `MobjType` / `SpriteNum`, `SfxName`, the menu enums, `GameMode`, ...)

| enum | defined | cast sites | first sites |
|---|---|---|---|
| `MobjType` | [p_mobj.rs:2338](../rust-doomgeneric/engine/src/p_mobj.rs#L2338) | 64 | [f_finale.rs:515](../rust-doomgeneric/engine/src/f_finale.rs#L515), [f_finale.rs:549](../rust-doomgeneric/engine/src/f_finale.rs#L549), [f_finale.rs:557](../rust-doomgeneric/engine/src/f_finale.rs#L557) … |
| `StateNum` | [p_mobj.rs:386](../rust-doomgeneric/engine/src/p_mobj.rs#L386) | 28 | [f_finale.rs:515](../rust-doomgeneric/engine/src/f_finale.rs#L515), [f_finale.rs:539](../rust-doomgeneric/engine/src/f_finale.rs#L539), [f_finale.rs:563](../rust-doomgeneric/engine/src/f_finale.rs#L563) … |
| `MusicName` | [sounds.rs:4](../rust-doomgeneric/engine/src/sounds.rs#L4) | 25 | [d_main.rs:524](../rust-doomgeneric/engine/src/d_main.rs#L524), [d_main.rs:526](../rust-doomgeneric/engine/src/d_main.rs#L526), [d_main.rs:545](../rust-doomgeneric/engine/src/d_main.rs#L545) … |
| `PSpriteNum` | [d_player.rs:34](../rust-doomgeneric/engine/src/d_player.rs#L34) | 18 | [p_pspr.rs:115](../rust-doomgeneric/engine/src/p_pspr.rs#L115), [p_pspr.rs:117](../rust-doomgeneric/engine/src/p_pspr.rs#L117), [p_pspr.rs:175](../rust-doomgeneric/engine/src/p_pspr.rs#L175) … |
| `DirType` | [p_enemy.rs:103](../rust-doomgeneric/engine/src/p_enemy.rs#L103) | 16 | [p_enemy.rs:288](../rust-doomgeneric/engine/src/p_enemy.rs#L288), [p_enemy.rs:331](../rust-doomgeneric/engine/src/p_enemy.rs#L331), [p_enemy.rs:367](../rust-doomgeneric/engine/src/p_enemy.rs#L367) … |
| `SpriteNum` | [p_mobj.rs:102](../rust-doomgeneric/engine/src/p_mobj.rs#L102) | 12 | [f_finale.rs:719](../rust-doomgeneric/engine/src/f_finale.rs#L719), [p_inter.rs:267](../rust-doomgeneric/engine/src/p_inter.rs#L267), [p_saveg.rs:284](../rust-doomgeneric/engine/src/p_saveg.rs#L284) … |
| `GameMission` | [d_mode.rs:34](../rust-doomgeneric/engine/src/d_mode.rs#L34) | 10 | [d_iwad.rs:165](../rust-doomgeneric/engine/src/d_iwad.rs#L165), [d_iwad.rs:181](../rust-doomgeneric/engine/src/d_iwad.rs#L181), [d_main.rs:827](../rust-doomgeneric/engine/src/d_main.rs#L827) … |
| `MapLump` | [p_setup.rs:233](../rust-doomgeneric/engine/src/p_setup.rs#L233) | 10 | [p_setup.rs:734](../rust-doomgeneric/engine/src/p_setup.rs#L734), [p_setup.rs:736](../rust-doomgeneric/engine/src/p_setup.rs#L736), [p_setup.rs:737](../rust-doomgeneric/engine/src/p_setup.rs#L737) … |
| `SpecialThinkerClass` | [p_saveg.rs:94](../rust-doomgeneric/engine/src/p_saveg.rs#L94) | 9 | [p_saveg.rs:1029](../rust-doomgeneric/engine/src/p_saveg.rs#L1029), [p_saveg.rs:1040](../rust-doomgeneric/engine/src/p_saveg.rs#L1040), [p_saveg.rs:1048](../rust-doomgeneric/engine/src/p_saveg.rs#L1048) … |
| `PowerType` | [d_player.rs:43](../rust-doomgeneric/engine/src/d_player.rs#L43) | 6 | [am_map.rs:1149](../rust-doomgeneric/engine/src/am_map.rs#L1149), [d_player.rs:66](../rust-doomgeneric/engine/src/d_player.rs#L66), [st_stuff.rs:571](../rust-doomgeneric/engine/src/st_stuff.rs#L571) … |
| `GameVersion` | [d_mode.rs:67](../rust-doomgeneric/engine/src/d_mode.rs#L67) | 4 | [d_main.rs:729](../rust-doomgeneric/engine/src/d_main.rs#L729), [d_net.rs:86](../rust-doomgeneric/engine/src/d_net.rs#L86), [g_game.rs:1617](../rust-doomgeneric/engine/src/g_game.rs#L1617) … |
| `OptionsMenu` | [m_menu.rs:584](../rust-doomgeneric/engine/src/m_menu.rs#L584) | 4 | [m_menu.rs:1059](../rust-doomgeneric/engine/src/m_menu.rs#L1059), [m_menu.rs:1072](../rust-doomgeneric/engine/src/m_menu.rs#L1072), [m_menu.rs:1078](../rust-doomgeneric/engine/src/m_menu.rs#L1078) … |
| `SfxName` | [sounds.rs:95](../rust-doomgeneric/engine/src/sounds.rs#L95) | 4 | [s_sound.rs:299](../rust-doomgeneric/engine/src/s_sound.rs#L299), [s_sound.rs:346](../rust-doomgeneric/engine/src/s_sound.rs#L346), [s_sound.rs:357](../rust-doomgeneric/engine/src/s_sound.rs#L357) … |
| `SndDevice` | [i_sound.rs:25](../rust-doomgeneric/engine/src/i_sound.rs#L25) | 4 | [i_sound.rs:81](../rust-doomgeneric/engine/src/i_sound.rs#L81), [i_sound.rs:82](../rust-doomgeneric/engine/src/i_sound.rs#L82), [s_sound.rs:456](../rust-doomgeneric/engine/src/s_sound.rs#L456) … |
| `NewGameMenu` | [m_menu.rs:615](../rust-doomgeneric/engine/src/m_menu.rs#L615) | 3 | [m_menu.rs:210](../rust-doomgeneric/engine/src/m_menu.rs#L210), [m_menu.rs:994](../rust-doomgeneric/engine/src/m_menu.rs#L994), [m_menu.rs:1001](../rust-doomgeneric/engine/src/m_menu.rs#L1001) |
| `SoundMenu` | [m_menu.rs:597](../rust-doomgeneric/engine/src/m_menu.rs#L597) | 3 | [m_menu.rs:901](../rust-doomgeneric/engine/src/m_menu.rs#L901), [m_menu.rs:907](../rust-doomgeneric/engine/src/m_menu.rs#L907), [m_menu.rs:1590](../rust-doomgeneric/engine/src/m_menu.rs#L1590) |
| `EvType` | [d_event.rs:23](../rust-doomgeneric/engine/src/d_event.rs#L23) | 2 | [am_map.rs:648](../rust-doomgeneric/engine/src/am_map.rs#L648), [g_game.rs:835](../rust-doomgeneric/engine/src/g_game.rs#L835) |
| `MainMenu` | [m_menu.rs:625](../rust-doomgeneric/engine/src/m_menu.rs#L625) | 2 | [m_menu.rs:1852](../rust-doomgeneric/engine/src/m_menu.rs#L1852), [m_menu.rs:1853](../rust-doomgeneric/engine/src/m_menu.rs#L1853) |
| `PlatE` | [p_plats.rs:34](../rust-doomgeneric/engine/src/p_plats.rs#L34) | 2 | [p_saveg.rs:620](../rust-doomgeneric/engine/src/p_saveg.rs#L620), [p_saveg.rs:621](../rust-doomgeneric/engine/src/p_saveg.rs#L621) |
| `ThinkerClass` | [p_saveg.rs:89](../rust-doomgeneric/engine/src/p_saveg.rs#L89) | 2 | [p_saveg.rs:874](../rust-doomgeneric/engine/src/p_saveg.rs#L874), [p_saveg.rs:882](../rust-doomgeneric/engine/src/p_saveg.rs#L882) |
| `AnimEnum` | [wi_stuff.rs:531](../rust-doomgeneric/engine/src/wi_stuff.rs#L531) | 1 | [wi_stuff.rs:731](../rust-doomgeneric/engine/src/wi_stuff.rs#L731) |
| `BoxIndex` | [m_bbox.rs:6](../rust-doomgeneric/engine/src/m_bbox.rs#L6) | 1 | [m_bbox.rs:18](../rust-doomgeneric/engine/src/m_bbox.rs#L18) |
| `CardType` | [p_inter.rs:44](../rust-doomgeneric/engine/src/p_inter.rs#L44) | 1 | [p_inter.rs:55](../rust-doomgeneric/engine/src/p_inter.rs#L55) |
| `CeilingE` | [p_ceilng.rs:29](../rust-doomgeneric/engine/src/p_ceilng.rs#L29) | 1 | [p_saveg.rs:493](../rust-doomgeneric/engine/src/p_saveg.rs#L493) |
| `EpisodeMenu` | [m_menu.rs:606](../rust-doomgeneric/engine/src/m_menu.rs#L606) | 1 | [m_menu.rs:170](../rust-doomgeneric/engine/src/m_menu.rs#L170) |
| `FloorE` | [p_floor.rs:36](../rust-doomgeneric/engine/src/p_floor.rs#L36) | 1 | [p_saveg.rs:569](../rust-doomgeneric/engine/src/p_saveg.rs#L569) |
| `GameMode` | [d_mode.rs:58](../rust-doomgeneric/engine/src/d_mode.rs#L58) | 1 | [d_net.rs:105](../rust-doomgeneric/engine/src/d_net.rs#L105) |
| `PlattypeE` | [p_plats.rs:41](../rust-doomgeneric/engine/src/p_plats.rs#L41) | 1 | [p_saveg.rs:624](../rust-doomgeneric/engine/src/p_saveg.rs#L624) |
| `PlayerState` | [d_player.rs:153](../rust-doomgeneric/engine/src/d_player.rs#L153) | 1 | [p_saveg.rs:418](../rust-doomgeneric/engine/src/p_saveg.rs#L418) |
| `SlopeType` | [p_mobj.rs:2786](../rust-doomgeneric/engine/src/p_mobj.rs#L2786) | 1 | [p_maputl.rs:241](../rust-doomgeneric/engine/src/p_maputl.rs#L241) |
| `VldoorE` | [p_doors.rs:32](../rust-doomgeneric/engine/src/p_doors.rs#L32) | 1 | [p_saveg.rs:529](../rust-doomgeneric/engine/src/p_saveg.rs#L529) |

## Appendix B — enums whose number is never observed

No `as` cast of these exists anywhere, so neither a number nor the order matters: they are only ever compared or matched by variant, and could be reordered freely.

`BWhere` ([p_switch.rs:37](../rust-doomgeneric/engine/src/p_switch.rs#L37)), `DefaultType` ([m_config.rs:13](../rust-doomgeneric/engine/src/m_config.rs#L13)), `DosMemDump` ([i_system.rs:9](../rust-doomgeneric/engine/src/i_system.rs#L9)), `FinaleStage` ([f_finale.rs:38](../rust-doomgeneric/engine/src/f_finale.rs#L38)), `GameAction` ([d_event.rs:2](../rust-doomgeneric/engine/src/d_event.rs#L2)), `GameScreenState` ([d_event.rs:15](../rust-doomgeneric/engine/src/d_event.rs#L15)), `MenuId` ([m_menu.rs:55](../rust-doomgeneric/engine/src/m_menu.rs#L55)), `OverrunTarget` ([p_maputl.rs:179](../rust-doomgeneric/engine/src/p_maputl.rs#L179)), `Plane` ([p_spec.rs:282](../rust-doomgeneric/engine/src/p_spec.rs#L282)), `ResultE` ([p_floor.rs:57](../rust-doomgeneric/engine/src/p_floor.rs#L57)), `Screen` ([v_video.rs:21](../rust-doomgeneric/engine/src/v_video.rs#L21)), `SpriteRotate` ([r_defs.rs:147](../rust-doomgeneric/engine/src/r_defs.rs#L147)), `StChatStateEnum` ([st_stuff.rs:282](../rust-doomgeneric/engine/src/st_stuff.rs#L282)), `StStateEnum` ([st_stuff.rs:277](../rust-doomgeneric/engine/src/st_stuff.rs#L277)), `StairE` ([p_floor.rs:52](../rust-doomgeneric/engine/src/p_floor.rs#L52)), `StateEnum` ([wi_stuff.rs:506](../rust-doomgeneric/engine/src/wi_stuff.rs#L506)), `ThinkerKind` ([p_tick.rs:41](../rust-doomgeneric/engine/src/p_tick.rs#L41)).

