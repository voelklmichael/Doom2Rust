//! Telling which game a WAD file is.

/// The name the engine should know the IWAD by, judged by what the WAD contains rather than what
/// the file is called (a download may be named anything). The engine decides the game from the
/// name for the Doom II expansions, and from the levels for the others, so the names of the
/// original files are the ones to use. `Err` says why the file is not an IWAD the game can run.
pub fn identify(wad: &[u8]) -> Result<&'static str, String> {
    let lumps = lump_names(wad)?;
    let has = |name: &str| lumps.contains(&name.as_bytes());
    if has("MAP01") {
        // The two expansions are Doom II with different extra lumps.
        Ok(if has("CAMO1") {
            "plutonia.wad"
        } else if has("REDTNT2") {
            "tnt.wad"
        } else {
            "doom2.wad"
        })
    } else if has("E1M1") {
        // The shareware IWAD has the first episode only.
        Ok(if has("E2M1") { "doom.wad" } else { "doom1.wad" })
    } else {
        Err("this WAD has no levels (neither E1M1 nor MAP01): it is not a DOOM IWAD".to_string())
    }
}

/// The lump names of the directory, without the NUL padding.
fn lump_names(wad: &[u8]) -> Result<Vec<&[u8]>, String> {
    let word = |at: usize| -> Option<usize> {
        let bytes = wad.get(at..at + 4)?;
        Some(u32::from_le_bytes(bytes.try_into().ok()?) as usize)
    };
    match wad.get(..4) {
        Some(b"IWAD") => {}
        Some(b"PWAD") => {
            return Err("this is a PWAD, which only adds to a game: drop an IWAD \
                        (doom1.wad, doom.wad, doom2.wad, ...)"
                .to_string())
        }
        _ => return Err("this is not a WAD file".to_string()),
    }
    let truncated = || "this WAD file is truncated".to_string();
    let count = word(4).ok_or_else(truncated)?;
    let table = word(8).ok_or_else(truncated)?;
    let end = count
        .checked_mul(16)
        .and_then(|size| size.checked_add(table))
        .ok_or_else(truncated)?;
    let directory = wad.get(table..end).ok_or_else(truncated)?;
    Ok(directory
        .as_chunks::<16>()
        .0
        .iter()
        .map(|entry| {
            let name = &entry[8..16];
            &name[..name.iter().position(|&b| b == 0).unwrap_or(8)]
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A WAD with empty lumps of the given names.
    fn wad(magic: [u8; 4], lumps: &[&str]) -> Vec<u8> {
        let mut bytes = magic.to_vec();
        bytes.extend((lumps.len() as u32).to_le_bytes());
        bytes.extend(12u32.to_le_bytes());
        for name in lumps {
            bytes.extend([0; 8]);
            let mut padded = name.as_bytes().to_vec();
            padded.resize(8, 0);
            bytes.extend(padded);
        }
        bytes
    }

    #[test]
    fn games_are_told_apart_by_their_levels() {
        let iwad = |lumps: &[&str]| identify(&wad(*b"IWAD", lumps));
        assert_eq!(iwad(&["PLAYPAL", "E1M1", "E1M2"]), Ok("doom1.wad"));
        assert_eq!(iwad(&["E1M1", "E2M1", "E3M1"]), Ok("doom.wad"));
        assert_eq!(iwad(&["E1M1", "E2M1", "E3M1", "E4M1"]), Ok("doom.wad"));
        assert_eq!(iwad(&["MAP01", "MAP02"]), Ok("doom2.wad"));
        assert_eq!(iwad(&["MAP01", "CAMO1"]), Ok("plutonia.wad"));
        assert_eq!(iwad(&["MAP01", "REDTNT2"]), Ok("tnt.wad"));
    }

    #[test]
    fn full_lump_names_are_not_confused_with_longer_ones() {
        // A lump called E1M1X is not E1M1.
        assert!(identify(&wad(*b"IWAD", &["E1M1X"])).is_err());
        assert_eq!(
            identify(&wad(*b"IWAD", &["E1M12345"])),
            Err(NO_LEVELS.to_string())
        );
    }

    const NO_LEVELS: &str =
        "this WAD has no levels (neither E1M1 nor MAP01): it is not a DOOM IWAD";

    #[test]
    fn files_that_are_not_runnable_iwads_say_why() {
        assert!(identify(&wad(*b"PWAD", &["MAP01"]))
            .unwrap_err()
            .contains("PWAD"));
        assert!(identify(b"hello, world").unwrap_err().contains("not a WAD"));
        assert!(identify(b"").unwrap_err().contains("not a WAD"));
        assert_eq!(
            identify(&wad(*b"IWAD", &["PLAYPAL"])),
            Err(NO_LEVELS.to_string())
        );
        let mut cut = wad(*b"IWAD", &["MAP01", "MAP02"]);
        cut.truncate(cut.len() - 1);
        assert!(identify(&cut).unwrap_err().contains("truncated"));
        // A directory that claims to be enormous does not overflow anything.
        let mut huge = wad(*b"IWAD", &[]);
        huge[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(identify(&huge).unwrap_err().contains("truncated"));
        assert!(identify(b"IWAD").unwrap_err().contains("truncated"));
    }
}
