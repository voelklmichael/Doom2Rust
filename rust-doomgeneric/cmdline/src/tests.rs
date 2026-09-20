use super::*;
use alloc::string::ToString;
use alloc::vec;

/// The lookup the engine used before it took an `Options`, kept here as the oracle.
fn old_check_parm_with_args(argv: &[String], check: &str, num_args: usize) -> Option<usize> {
    (1..argv.len().saturating_sub(num_args)).find(|&i| argv[i].eq_ignore_ascii_case(check))
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

/// Every field, computed the way the engine's call sites used to.
fn expected(argv: &[String]) -> Options {
    let exists = |n: &str| old_check_parm_with_args(argv, n, 0).is_some();
    let val = |n: &str| old_check_parm_with_args(argv, n, 1).map(|p| argv[p + 1].clone());
    let num = |n: &str| old_check_parm_with_args(argv, n, 1).map(|p| argv_atoi(&argv[p + 1]));
    // d_main's old code: `(first byte of the argument) - '0'`
    let digit = |n: &str| val(n).map(|a| first_digit(&a));
    Options {
        altdeath: exists("-altdeath"),
        avg: exists("-avg"),
        deathmatch: exists("-deathmatch"),
        devparm: exists("-devparm"),
        fast: exists("-fast"),
        left: exists("-left"),
        longtics: exists("-longtics"),
        netdemo: exists("-netdemo"),
        nodraw: exists("-nodraw"),
        nomonsters: exists("-nomonsters"),
        nomusic: exists("-nomusic"),
        nosfx: exists("-nosfx"),
        nosound: exists("-nosound"),
        record: exists("-record"),
        reject_pad_with_ff: exists("-reject_pad_with_ff"),
        respawn: exists("-respawn"),
        right: exists("-right"),
        solo_net: exists("-solo-net"),
        statdump: exists("-statdump"),
        testcontrols: exists("-testcontrols"),
        config: val("-config"),
        donut: old_check_parm_with_args(argv, "-donut", 2)
            .map(|p| (argv[p + 1].clone(), argv[p + 2].clone())),
        episode: digit("-episode"),
        extraconfig: val("-extraconfig"),
        // w_main's old code: every argument after -file up to the next option
        file: old_check_parm_with_args(argv, "-file", 1).map(|p| {
            let mut files = Vec::new();
            for a in &argv[p + 1..] {
                if a.starts_with('-') {
                    break;
                }
                files.push(a.clone());
            }
            files
        }),
        gameversion: val("-gameversion"),
        gfxmode: val("-gfxmode"),
        iwad: val("-iwad"),
        loadgame: num("-loadgame"),
        maxdemo: num("-maxdemo"),
        pack: val("-pack"),
        playdemo: val("-playdemo"),
        record_file: val("-record"),
        scaling: num("-scaling"),
        setmem: old_check_parm_with_args(argv, "-setmem", 1).map(|p| argv[p + 1..].to_vec()),
        skill: digit("-skill"),
        spechit: val("-spechit"),
        statdump_file: val("-statdump"),
        timedemo: val("-timedemo"),
        timer: num("-timer"),
        // d_main's old code: `-turbo` anywhere, then `argv.get(p + 1)` if there is one
        turbo: old_check_parm_with_args(argv, "-turbo", 0)
            .map(|p| argv.get(p + 1).map(|a| argv_atoi(a))),
        // d_main's old code: `p + 2 < argv.len()` decided whether there was a second argument
        warp: old_check_parm_with_args(argv, "-warp", 1).map(|p| Warp {
            map_number: argv_atoi(&argv[p + 1]),
            episode: first_digit(&argv[p + 1]),
            episode_map: if p + 2 < argv.len() {
                first_digit(&argv[p + 2])
            } else {
                1
            },
        }),
    }
}

#[test]
fn an_empty_command_line_is_no_options() {
    assert_eq!(parse(&args(&["doom"])), Options::default());
    assert_eq!(parse(&[]), Options::default());
}

#[test]
fn switches_and_values_are_found() {
    let o = parse(&args(&[
        "doom",
        "-nomonsters",
        "-skill",
        "4",
        "-warp",
        "2",
        "5",
        "-iwad",
        "doom.wad",
    ]));
    assert!(o.nomonsters && !o.respawn);
    assert_eq!(o.skill, Some(4));
    assert_eq!(o.iwad.as_deref(), Some("doom.wad"));
    assert_eq!(
        o.warp,
        Some(Warp {
            map_number: 2,
            episode: 2,
            episode_map: 5
        })
    );
}

#[test]
fn a_warp_with_one_argument_defaults_the_episode_map_to_one() {
    let o = parse(&args(&["doom", "-warp", "12"]));
    assert_eq!(
        o.warp,
        Some(Warp {
            map_number: 12,
            episode: 1,
            episode_map: 1
        })
    );
}

#[test]
fn matching_is_case_insensitive_and_skips_the_program_name() {
    assert!(parse(&args(&["doom", "-NoMonsters"])).nomonsters);
    assert!(!parse(&args(&["-nomonsters"])).nomonsters);
}

#[test]
fn an_option_that_needs_a_value_is_ignored_when_it_is_last() {
    assert_eq!(parse(&args(&["doom", "-skill"])).skill, None);
    let o = parse(&args(&["doom", "-record"]));
    assert!(o.record && o.record_file.is_none());
    assert_eq!(parse(&args(&["doom", "-donut", "1"])).donut, None);
}

#[test]
fn the_first_occurrence_wins() {
    assert_eq!(
        parse(&args(&["doom", "-skill", "1", "-skill", "5"])).skill,
        Some(1)
    );
}

#[test]
fn file_takes_arguments_up_to_the_next_option() {
    let o = parse(&args(&["doom", "-file", "a.wad", "b.wad", "-nosound", "c"]));
    assert_eq!(o.file, Some(vec!["a.wad".to_string(), "b.wad".to_string()]));
    assert_eq!(
        parse(&args(&["doom", "-file", "-nosound"])).file,
        Some(vec![])
    );
}

#[test]
fn turbo_has_an_optional_value() {
    assert_eq!(parse(&args(&["doom", "-turbo"])).turbo, Some(None));
    assert_eq!(
        parse(&args(&["doom", "-turbo", "150"])).turbo,
        Some(Some(150))
    );
}

#[test]
fn numbers_read_like_atoi_and_levels_like_the_first_character() {
    let o = parse(&args(&[
        "doom", "-timer", " 12abc", "-scaling", "x", "-skill", "3x",
    ]));
    assert_eq!(o.timer, Some(12));
    assert_eq!(o.scaling, Some(0));
    assert_eq!(o.skill, Some(3));
    assert_eq!(parse(&args(&["doom", "-episode", ""])).episode, Some(-48));
}

/// Every option name, mixed with values and with things that are not options: every combination up
/// to length 3, and a long deterministic pseudo-random sweep, must parse exactly as the old
/// lookups answered.
#[test]
fn parse_agrees_with_the_old_lookups_on_every_short_command_line() {
    let names = [
        "-altdeath",
        "-avg",
        "-config",
        "-deathmatch",
        "-devparm",
        "-donut",
        "-episode",
        "-extraconfig",
        "-fast",
        "-file",
        "-gameversion",
        "-gfxmode",
        "-iwad",
        "-left",
        "-loadgame",
        "-longtics",
        "-maxdemo",
        "-netdemo",
        "-nodraw",
        "-nomonsters",
        "-nomusic",
        "-nosfx",
        "-nosound",
        "-pack",
        "-playdemo",
        "-record",
        "-reject_pad_with_ff",
        "-respawn",
        "-right",
        "-scaling",
        "-setmem",
        "-skill",
        "-solo-net",
        "-spechit",
        "-statdump",
        "-testcontrols",
        "-timedemo",
        "-timer",
        "-turbo",
        "-warp",
    ];
    assert_eq!(names.len(), 40);
    let mut pool: Vec<String> = names.iter().map(|s| (*s).to_string()).collect();
    pool.extend(["-WARP", "-Skill", "3", "x.wad", "-unknown", "", "-"].map(String::from));
    for len in 0..=3usize {
        let mut idx = vec![0usize; len];
        loop {
            let mut argv = vec!["doom".to_string()];
            argv.extend(idx.iter().map(|&i| pool[i].clone()));
            assert_eq!(parse(&argv), expected(&argv), "{argv:?}");
            let mut k = 0;
            while k < len {
                idx[k] += 1;
                if idx[k] < pool.len() {
                    break;
                }
                idx[k] = 0;
                k += 1;
            }
            if k == len {
                break;
            }
        }
    }
    let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
    for _ in 0..20_000 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let len = (seed % 9) as usize;
        let mut argv = vec!["doom".to_string()];
        for _ in 0..len {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            argv.push(pool[(seed % pool.len() as u64) as usize].clone());
        }
        assert_eq!(parse(&argv), expected(&argv), "{argv:?}");
    }
}
