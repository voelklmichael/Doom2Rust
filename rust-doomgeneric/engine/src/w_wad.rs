use crate::d_iwad::suggest_game_name;
use crate::d_mode::game_mission_string;
use crate::d_mode::GameMission;
use crate::d_mode::GameMode;
use crate::fixed_cstr::FixedCStr;
use crate::i_system::error;
use crate::le::le_i32;
use crate::m_misc::extract_file_base;
use crate::platform::DoomPlatform;
use alloc::vec::Vec;

use crate::filesystem::{DoomFileSystem, FileId};

/// The number of a lump in the loaded WADs (an index into `WWadState::lumpinfo`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LumpNum(pub u32);

impl LumpNum {
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// The lump `n` places after this one (flats and sprites are stored as consecutive lumps).
impl core::ops::Add<i32> for LumpNum {
    type Output = Self;
    #[inline]
    fn add(self, n: i32) -> Self {
        Self(self.0.wrapping_add_signed(n))
    }
}

/// The lump `n` places before this one.
impl core::ops::Sub<i32> for LumpNum {
    type Output = Self;
    #[inline]
    fn sub(self, n: i32) -> Self {
        Self(self.0.wrapping_add_signed(-n))
    }
}

/// How many lumps apart two lump numbers are.
impl core::ops::Sub for LumpNum {
    type Output = i32;
    #[inline]
    fn sub(self, other: Self) -> i32 {
        self.0.wrapping_sub(other.0) as i32
    }
}

impl core::fmt::Display for LumpNum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct WWadState {
    pub lumpinfo: Vec<LumpInfo>,
    pub numlumps: u32,
    pub lumphash: Vec<Option<u32>>,
}

impl Default for WWadState {
    fn default() -> Self {
        Self::new()
    }
}

impl WWadState {
    pub const fn new() -> Self {
        Self {
            lumpinfo: Vec::new(),
            numlumps: 0,
            lumphash: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct LumpInfo {
    pub name: FixedCStr<8>,
    pub wad_file: FileId,
    pub position: i32,
    pub size: i32,
    pub cache: Option<alloc::rc::Rc<[u8]>>,
    pub next: Option<u32>,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct filelump_t {
    pub filepos: i32,
    pub size: i32,
    pub name: FixedCStr<8>,
}
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct wadinfo_t {
    pub identification: FixedCStr<4>,
    pub numlumps: i32,
    pub infotableofs: i32,
}
#[derive(Copy, Clone)]
pub struct UniqueLump {
    pub mission: GameMission,
    pub lumpname: &'static str,
}
pub const PROGRAM_PREFIX: FixedCStr<12> = FixedCStr(*b"doomgeneric\0");
pub fn lump_name_hash(s: &[u8]) -> u32 {
    let mut result: u32 = 5381;
    for &b in s.iter().take(8) {
        if b == 0 {
            break;
        }
        result = result << 5 ^ result ^ u32::from(b.to_ascii_uppercase());
    }
    result
}
pub fn w_add_file(
    fs: &mut dyn DoomFileSystem,
    platform: &mut dyn DoomPlatform,
    w_wad: &mut WWadState,
    filename: &str,
) -> Option<FileId> {
    // Scratch WAD-directory buffer -- built and consumed entirely within
    // this function, so a plain owned Vec replaces the old
    // Z_Malloc-then-Z_Free-at-the-end pair with no lifetime change.

    let Some(wad_file) = fs.open(filename) else {
        doom_println!(platform, " couldn't open {}", filename);
        return None;
    };
    let wad_length = fs.len(wad_file) as u32;
    let is_wad = filename.len() >= 3 && filename[filename.len() - 3..].eq_ignore_ascii_case("wad");
    let fileinfo: Vec<filelump_t> = if is_wad {
        let mut header_buf = [0u8; ::core::mem::size_of::<wadinfo_t>()];
        fs.read_at(wad_file, 0, &mut header_buf);
        let header = wadinfo_t {
            identification: FixedCStr::from_bytes(&header_buf[0..4]),
            numlumps: le_i32(header_buf, 4),
            infotableofs: le_i32(header_buf, 8),
        };
        if header.identification.0 != *b"IWAD" && header.identification.0 != *b"PWAD" {
            error(&format!(
                "Wad file {filename} doesn't have IWAD or PWAD id\n",
            ));
        }
        let mut dir_buf =
            vec![0u8; (header.numlumps as usize) * ::core::mem::size_of::<filelump_t>()];
        fs.read_at(
            wad_file,
            u64::from(header.infotableofs as u32),
            &mut dir_buf,
        );
        dir_buf
            .as_chunks::<{ ::core::mem::size_of::<filelump_t>() }>()
            .0
            .iter()
            .map(|c| filelump_t {
                filepos: le_i32(c, 0),
                size: le_i32(c, 4),
                name: FixedCStr::from_bytes(&c[8..16]),
            })
            .collect()
    } else {
        let mut single = filelump_t {
            filepos: 0,
            size: wad_length as i32,
            name: FixedCStr([0; 8]),
        };
        extract_file_base(&mut *platform, filename, &mut single.name);
        vec![single]
    };
    w_wad
        .lumpinfo
        .extend(fileinfo.into_iter().map(|fi| LumpInfo {
            name: fi.name,
            wad_file,
            position: fi.filepos,
            size: fi.size,
            cache: None,
            next: None,
        }));
    w_wad.numlumps = w_wad.lumpinfo.len() as u32;
    w_wad.lumphash = Vec::new();
    Some(wad_file)
}
/// The number of the last lump called `name`, if there is one.
pub fn check_num_for_name(state: &WWadState, name: &str) -> Option<LumpNum> {
    if state.lumphash.is_empty() {
        // The last lump with the name wins, as in vanilla.
        return (0..state.numlumps as usize)
            .rev()
            .find(|&i| state.lumpinfo[i].name.eq_str_ignore_ascii_case(name))
            .map(|i| LumpNum(i as u32));
    }
    let hash: u32 = lump_name_hash(name.as_bytes()).wrapping_rem(state.numlumps);
    let mut cur = state.lumphash[hash as usize];
    while let Some(idx) = cur {
        if state.lumpinfo[idx as usize]
            .name
            .eq_str_ignore_ascii_case(name)
        {
            return Some(LumpNum(idx));
        }
        cur = state.lumpinfo[idx as usize].next;
    }
    None
}
pub fn get_num_for_name(state: &WWadState, name: &str) -> LumpNum {
    check_num_for_name(state, name)
        .unwrap_or_else(|| error(&format!("W_GetNumForName: {name} not found!")))
}
pub fn lump_length(state: &WWadState, lump: LumpNum) -> i32 {
    if lump.0 >= state.numlumps {
        error(&format!("W_LumpLength: {lump} >= numlumps"));
    }
    state.lumpinfo[lump.index()].size
}
pub fn read_lump(state: &WWadState, fs: &dyn DoomFileSystem, lump: LumpNum, dest: &mut [u8]) {
    if lump.0 >= state.numlumps {
        error(&format!("W_ReadLump: {lump} >= numlumps"));
    }
    let l = &state.lumpinfo[lump.index()];
    let c = fs.read_at(l.wad_file, u64::from(l.position as u32), dest) as i32;
    if c < l.size {
        error(&format!(
            "W_ReadLump: only read {} of {} on lump {}",
            c, l.size, lump
        ));
    }
}
pub fn lump_bytes(
    fs: &dyn DoomFileSystem,
    w_wad: &mut WWadState,
    lumpnum: LumpNum,
) -> alloc::rc::Rc<[u8]> {
    const CACHE_PAD: usize = 128;
    if lumpnum.0 >= w_wad.numlumps {
        error(&format!("W_CacheLumpNum: {lumpnum} >= numlumps"));
    }
    if let Some(cache) = w_wad.lumpinfo[lumpnum.index()].cache.as_ref() {
        return alloc::rc::Rc::clone(cache);
    }
    let lumplen = lump_length(w_wad, lumpnum);
    // r_draw.rs's draw_column (and friends) reproduce vanilla's
    // `dc_source[(frac>>FRACBITS) & 127]` column read verbatim, which
    // vanilla itself only gets away with because its zone allocator
    // rounds every block up, leaving slack heap bytes past a short
    // lump's real data for that `& 127` mask to wander into instead of
    // segfaulting. An exactly-sized buffer has no such slack, so a
    // short column (any post shorter than 128 rows, cached near the
    // end of its lump) makes that same read run past the end and panic
    // -- pad every cached lump by the mask's full range to give it the
    // same harmless slack vanilla relied on.
    let mut buf = vec![0u8; lumplen as usize + CACHE_PAD].into_boxed_slice();
    read_lump(w_wad, fs, lumpnum, &mut buf[..lumplen as usize]);
    let rc: alloc::rc::Rc<[u8]> = alloc::rc::Rc::from(buf);
    w_wad.lumpinfo[lumpnum.index()].cache = Some(alloc::rc::Rc::clone(&rc));
    rc
}
pub fn lump_bytes_name(
    fs: &dyn DoomFileSystem,
    w_wad: &mut WWadState,
    name: &str,
) -> alloc::rc::Rc<[u8]> {
    let lumpnum = get_num_for_name(w_wad, name);
    lump_bytes(fs, w_wad, lumpnum)
}
pub fn release_lump_num(state: &WWadState, lumpnum: LumpNum) {
    // Releasing a cached lump is a no-op now -- nothing purges cached blocks
    // under memory pressure since the zone allocator was removed entirely;
    // the owned cache buffer just stays cached until process exit either
    // way. Kept as a bounds-checked no-op rather than deleted, matching this
    // function's original validation behavior.
    if lumpnum.0 >= state.numlumps {
        error(&format!("W_ReleaseLumpNum: {lumpnum} >= numlumps"));
    }
}
pub fn release_lump_name(state: &WWadState, name: &str) {
    let lumpnum = get_num_for_name(state, name);
    release_lump_num(state, lumpnum);
}
pub fn generate_hash_table(w_wad: &mut WWadState) {
    w_wad.lumphash = Vec::new();
    if w_wad.numlumps > 0 {
        w_wad.lumphash = vec![None; w_wad.numlumps as usize];
        for i in 0..w_wad.numlumps {
            let hash: usize = lump_name_hash(w_wad.lumpinfo[i as usize].name.as_bytes())
                .wrapping_rem(w_wad.numlumps) as usize;
            let old_head = w_wad.lumphash[hash];
            w_wad.lumpinfo[i as usize].next = old_head;
            w_wad.lumphash[hash] = Some(i);
        }
    }
}
static UNIQUE_LUMPS: [UniqueLump; 4] = [
    UniqueLump {
        mission: GameMission::Doom,
        lumpname: "POSSA1",
    },
    UniqueLump {
        mission: GameMission::Heretic,
        lumpname: "IMPXA1",
    },
    UniqueLump {
        mission: GameMission::Hexen,
        lumpname: "ETTNA1",
    },
    UniqueLump {
        mission: GameMission::Strife,
        lumpname: "AGRDA1",
    },
];
pub fn check_correct_iwad(state: &WWadState, mission: GameMission) {
    for unique in &UNIQUE_LUMPS {
        if mission != unique.mission && check_num_for_name(state, unique.lumpname).is_some() {
            error(&format!(
                    "\nYou are trying to use a {} IWAD file with the {}{} binary.\nThis isn't going to work.\nYou probably want to use the {}{} binary.",
                    suggest_game_name(unique.mission, GameMode::Indetermined),
                    PROGRAM_PREFIX.as_str(),
                    game_mission_string(mission),
                    PROGRAM_PREFIX.as_str(),
                    game_mission_string(unique.mission),
                ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lump_numbers_offset_and_subtract() {
        let first = LumpNum(100);
        assert_eq!(first + 3, LumpNum(103));
        assert_eq!(first - 1, LumpNum(99));
        assert_eq!(LumpNum(110) - first, 10);
        assert_eq!(first - LumpNum(110), -10);
        assert_eq!(first.index(), 100);
    }

    fn wad_with(names: &[&str]) -> WWadState {
        let mut w_wad = WWadState::new();
        for name in names {
            w_wad.lumpinfo.push(LumpInfo {
                name: FixedCStr::from_bytes(name.as_bytes()),
                wad_file: FileId(0),
                position: 0,
                size: 0,
                cache: None,
                next: None,
            });
        }
        w_wad.numlumps = names.len() as u32;
        w_wad
    }

    #[test]
    fn name_lookup_finds_the_last_lump_with_that_name() {
        let w_wad = wad_with(&["PLAYPAL", "S_START", "TROOA1", "S_END", "PLAYPAL"]);
        assert_eq!(check_num_for_name(&w_wad, "playpal"), Some(LumpNum(4)));
        assert_eq!(check_num_for_name(&w_wad, "S_START"), Some(LumpNum(1)));
        assert_eq!(check_num_for_name(&w_wad, "NOSUCH"), None);
        assert_eq!(
            get_num_for_name(&w_wad, "S_END") - get_num_for_name(&w_wad, "S_START"),
            2
        );
    }
}
