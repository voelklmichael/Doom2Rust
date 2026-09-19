use crate::d_iwad::suggest_game_name;
use crate::d_mode::game_mission_string;
use crate::d_mode::GameMission;
use crate::d_mode::GameMode;
use crate::fixed_cstr::FixedCStr;
use crate::game_state::GameState;
use crate::i_system::error;
use crate::m_misc::extract_file_base;
use alloc::vec::Vec;

use crate::filesystem::{DoomFileSystem, FileId};

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
        result = result << 5 ^ result ^ b.to_ascii_uppercase() as u32;
    }
    result
}
pub fn w_add_file(state: &mut GameState, filename: &str) -> Option<FileId> {
    // Scratch WAD-directory buffer -- built and consumed entirely within
    // this function, so a plain owned Vec replaces the old
    // Z_Malloc-then-Z_Free-at-the-end pair with no lifetime change.

    let Some(wad_file) = state.fs.open(filename) else {
        doom_println!(state.platform, " couldn't open {}", filename);
        return None;
    };
    let wad_length = state.fs.len(wad_file) as u32;
    let is_wad = filename.len() >= 3 && filename[filename.len() - 3..].eq_ignore_ascii_case("wad");
    let fileinfo: Vec<filelump_t> = if is_wad {
        let mut header_buf = [0u8; ::core::mem::size_of::<wadinfo_t>()];
        state.fs.read_at(wad_file, 0, &mut header_buf);
        let header = wadinfo_t {
            identification: FixedCStr::from_bytes(&header_buf[0..4]),
            numlumps: i32::from_le_bytes(header_buf[4..8].try_into().unwrap()),
            infotableofs: i32::from_le_bytes(header_buf[8..12].try_into().unwrap()),
        };
        if header.identification.0 != *b"IWAD" && header.identification.0 != *b"PWAD" {
            error(&format!(
                "Wad file {filename} doesn't have IWAD or PWAD id\n",
            ));
        }
        let mut dir_buf =
            vec![0u8; (header.numlumps as usize) * ::core::mem::size_of::<filelump_t>()];
        state
            .fs
            .read_at(wad_file, header.infotableofs as u32 as u64, &mut dir_buf);
        dir_buf
            .as_chunks::<{ ::core::mem::size_of::<filelump_t>() }>()
            .0
            .iter()
            .map(|c| filelump_t {
                filepos: i32::from_le_bytes(c[0..4].try_into().unwrap()),
                size: i32::from_le_bytes(c[4..8].try_into().unwrap()),
                name: FixedCStr::from_bytes(&c[8..16]),
            })
            .collect()
    } else {
        let mut single = filelump_t {
            filepos: 0,
            size: wad_length as i32,
            name: FixedCStr([0; 8]),
        };
        extract_file_base(&mut *state.platform, filename, &mut single.name);
        vec![single]
    };
    state
        .w_wad
        .lumpinfo
        .extend(fileinfo.into_iter().map(|fi| LumpInfo {
            name: fi.name,
            wad_file,
            position: fi.filepos,
            size: fi.size,
            cache: None,
            next: None,
        }));
    state.w_wad.numlumps = state.w_wad.lumpinfo.len() as u32;
    state.w_wad.lumphash = Vec::new();
    Some(wad_file)
}
/// The number of the last lump called `name`, if there is one.
pub fn check_num_for_name(state: &WWadState, name: &str) -> Option<i32> {
    let mut i: i32;
    if state.lumphash.is_empty() {
        i = state.numlumps.wrapping_sub(1) as i32;
        while i >= 0 {
            if state.lumpinfo[i as usize]
                .name
                .eq_str_ignore_ascii_case(name)
            {
                return Some(i);
            }
            i -= 1;
        }
    } else {
        let hash: u32 = lump_name_hash(name.as_bytes()).wrapping_rem(state.numlumps);
        let mut cur = state.lumphash[hash as usize];
        while let Some(idx) = cur {
            if state.lumpinfo[idx as usize]
                .name
                .eq_str_ignore_ascii_case(name)
            {
                return Some(idx as i32);
            }
            cur = state.lumpinfo[idx as usize].next;
        }
    }
    None
}
pub fn get_num_for_name(state: &WWadState, name: &str) -> i32 {
    check_num_for_name(state, name)
        .unwrap_or_else(|| error(&format!("W_GetNumForName: {name} not found!")))
}
pub fn lump_length(state: &WWadState, lump: u32) -> i32 {
    if lump >= state.numlumps {
        error(&format!("W_LumpLength: {lump} >= numlumps"));
    }
    state.lumpinfo[lump as usize].size
}
pub fn read_lump(state: &WWadState, fs: &dyn DoomFileSystem, lump: u32, dest: &mut [u8]) {
    if lump >= state.numlumps {
        error(&format!("W_ReadLump: {lump} >= numlumps"));
    }
    let l = &state.lumpinfo[lump as usize];
    let c = fs.read_at(l.wad_file, l.position as u32 as u64, dest) as i32;
    if c < l.size {
        error(&format!(
            "W_ReadLump: only read {} of {} on lump {}",
            c, l.size, lump
        ));
    }
}
pub fn lump_bytes(state: &mut GameState, lumpnum: i32) -> alloc::rc::Rc<[u8]> {
    const CACHE_PAD: usize = 128;
    if lumpnum as u32 >= state.w_wad.numlumps {
        error(&format!("W_CacheLumpNum: {lumpnum} >= numlumps"));
    }
    if let Some(cache) = state.w_wad.lumpinfo[lumpnum as usize].cache.as_ref() {
        return alloc::rc::Rc::clone(cache);
    }
    let lumplen = lump_length(&state.w_wad, lumpnum as u32);
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
    read_lump(
        &state.w_wad,
        &*state.fs,
        lumpnum as u32,
        &mut buf[..lumplen as usize],
    );
    let rc: alloc::rc::Rc<[u8]> = alloc::rc::Rc::from(buf);
    state.w_wad.lumpinfo[lumpnum as usize].cache = Some(alloc::rc::Rc::clone(&rc));
    rc
}
pub fn lump_bytes_name(state: &mut GameState, name: &str) -> alloc::rc::Rc<[u8]> {
    let lumpnum = get_num_for_name(&state.w_wad, name);
    lump_bytes(state, lumpnum)
}
pub fn release_lump_num(state: &WWadState, lumpnum: i32) {
    // Releasing a cached lump is a no-op now -- nothing purges cached blocks
    // under memory pressure since the zone allocator was removed entirely;
    // the owned cache buffer just stays cached until process exit either
    // way. Kept as a bounds-checked no-op rather than deleted, matching this
    // function's original validation behavior.
    if lumpnum as u32 >= state.numlumps {
        error(&format!("W_ReleaseLumpNum: {lumpnum} >= numlumps"));
    }
}
pub fn release_lump_name(state: &WWadState, name: &str) {
    let lumpnum = get_num_for_name(state, name);
    release_lump_num(state, lumpnum);
}
pub fn generate_hash_table(w_wad: &mut WWadState) {
    let mut i: u32;
    w_wad.lumphash = Vec::new();
    if w_wad.numlumps > 0 {
        w_wad.lumphash = vec![None; w_wad.numlumps as usize];
        i = 0;
        while i < w_wad.numlumps {
            let hash: u32 = lump_name_hash(w_wad.lumpinfo[i as usize].name.as_bytes())
                .wrapping_rem(w_wad.numlumps);
            let old_head = w_wad.lumphash[hash as usize];
            w_wad.lumpinfo[i as usize].next = old_head;
            w_wad.lumphash[hash as usize] = Some(i);
            i = i.wrapping_add(1);
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
    let mut i: i32;
    i = 0;
    while (i as usize)
        < ::core::mem::size_of::<[UniqueLump; 4]>()
            .wrapping_div(::core::mem::size_of::<UniqueLump>())
    {
        if mission as u32 != UNIQUE_LUMPS[i as usize].mission as u32
            && check_num_for_name(state, UNIQUE_LUMPS[i as usize].lumpname).is_some()
        {
            error(&format!(
                    "\nYou are trying to use a {} IWAD file with the {}{} binary.\nThis isn't going to work.\nYou probably want to use the {}{} binary.",
                    suggest_game_name(UNIQUE_LUMPS[i as usize].mission, GameMode::Indetermined),
                    PROGRAM_PREFIX.as_str(),
                    game_mission_string(mission),
                    PROGRAM_PREFIX.as_str(),
                    game_mission_string(UNIQUE_LUMPS[i as usize].mission),
                ));
        }
        i += 1;
    }
}
