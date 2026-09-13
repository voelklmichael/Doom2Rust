use crate::src::i_system::I_Error;
use std::alloc::{alloc, dealloc, Layout};

pub type C2RustUnnamed = u32;
pub const PU_NUM_TAGS: C2RustUnnamed = 9;
pub const PU_CACHE: C2RustUnnamed = 8;
pub const PU_PURGELEVEL: C2RustUnnamed = 7;
pub const PU_LEVSPEC: C2RustUnnamed = 6;
pub const PU_LEVEL: C2RustUnnamed = 5;
pub const PU_FREE: C2RustUnnamed = 4;
pub const PU_MUSIC: C2RustUnnamed = 3;
pub const PU_SOUND: C2RustUnnamed = 2;
pub const PU_STATIC: C2RustUnnamed = 1;

pub const MEM_ALIGN: usize = ::core::mem::size_of::<*mut ::core::ffi::c_void>();
pub const ZONEID: i32 = 0x1d4a11;

// One of these lives immediately before every payload this module hands
// out, at `payload_ptr - size_of::<BlockHeader>()` rounded up to
// `MEM_ALIGN` -- same "hidden header before the pointer" trick the original
// zone allocator used, just no longer threaded into a single pre-sized
// arena. `layout` is the exact Layout passed to `alloc`, kept around
// because `dealloc` requires the identical size+align back.
struct BlockHeader {
    id: i32,
    tag: i32,
    user: *mut *mut ::core::ffi::c_void,
    layout: Layout,
    // Position of this header's pointer in ZZoneState.blocks, kept in sync
    // by Z_Free's swap_remove so individual frees stay O(1) instead of a
    // linear search through every live allocation.
    registry_index: usize,
}

// Registry of every currently-live allocation's header, used only for the
// bulk "free everything in this tag range" operation (Z_FreeTags, called
// once per level load to tear down the previous level's dynamically
// allocated thinkers/movers) and the heap sanity check. Individual
// Z_Malloc/Z_Free/Z_ChangeTag never need to search it.
pub struct ZZoneState {
    blocks: Vec<*mut BlockHeader>,
}

impl ZZoneState {
    pub const fn new() -> Self {
        ZZoneState { blocks: Vec::new() }
    }
}

unsafe fn header_layout_for(payload_size: usize) -> (Layout, usize) {
    let header_layout = Layout::new::<BlockHeader>();
    let payload_layout = Layout::from_size_align(payload_size, MEM_ALIGN)
        .unwrap_or_else(|_| I_Error("Z_Malloc: invalid allocation size"));
    header_layout
        .extend(payload_layout)
        .unwrap_or_else(|_| I_Error("Z_Malloc: allocation size overflow"))
}

unsafe fn header_of(ptr: *mut ::core::ffi::c_void) -> *mut BlockHeader {
    let (_, payload_offset) = header_layout_for(0);
    (ptr as *mut u8).sub(payload_offset) as *mut BlockHeader
}

unsafe fn payload_of(header: *mut BlockHeader) -> *mut ::core::ffi::c_void {
    let (_, payload_offset) = header_layout_for(0);
    (header as *mut u8).add(payload_offset) as *mut ::core::ffi::c_void
}

pub unsafe fn Z_Init(state: &mut ZZoneState) {
    state.blocks.clear();
}

pub unsafe fn Z_Free(state: &mut ZZoneState, mut ptr: *mut ::core::ffi::c_void) {
    let header = header_of(ptr);
    if (*header).id != ZONEID {
        I_Error("Z_Free: freed a pointer without ZONEID");
    }
    if !(*header).user.is_null() {
        *(*header).user = ::core::ptr::null_mut::<::core::ffi::c_void>();
    }
    let layout = (*header).layout;
    let index = (*header).registry_index;
    state.blocks.swap_remove(index);
    if index < state.blocks.len() {
        (*state.blocks[index]).registry_index = index;
    }
    dealloc(header as *mut u8, layout);
}

pub unsafe fn Z_Malloc(
    state: &mut ZZoneState,
    mut size: i32,
    mut tag: i32,
    mut user: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    if user.is_null() && tag >= PU_PURGELEVEL as i32 {
        I_Error("Z_Malloc: an owner is required for purgable blocks");
    }
    let (layout, payload_offset) = header_layout_for(size.max(0) as usize);
    let base = alloc(layout);
    if base.is_null() {
        I_Error(&format!("Z_Malloc: failed on allocation of {} bytes", size));
    }
    let header = base as *mut BlockHeader;
    let index = state.blocks.len();
    (*header).id = ZONEID;
    (*header).tag = tag;
    (*header).user = user as *mut *mut ::core::ffi::c_void;
    (*header).layout = layout;
    (*header).registry_index = index;
    state.blocks.push(header);
    let result = base.add(payload_offset) as *mut ::core::ffi::c_void;
    if !(*header).user.is_null() {
        *(*header).user = result;
    }
    return result;
}

pub unsafe fn Z_FreeTags(state: &mut ZZoneState, mut lowtag: i32, mut hightag: i32) {
    let matching: Vec<*mut ::core::ffi::c_void> = state
        .blocks
        .iter()
        .filter(|h| (***h).tag >= lowtag && (***h).tag <= hightag)
        .map(|h| payload_of(*h))
        .collect();
    for ptr in matching {
        Z_Free(state, ptr);
    }
}

pub unsafe fn Z_CheckHeap(state: &mut ZZoneState) {
    for &header in &state.blocks {
        if (*header).id != ZONEID {
            I_Error("Z_CheckHeap: block without a ZONEID\n");
        }
    }
}

pub unsafe fn Z_ChangeTag2(
    mut ptr: *mut ::core::ffi::c_void,
    mut tag: i32,
    file: &str,
    mut line: i32,
) {
    let header = header_of(ptr);
    if (*header).id != ZONEID {
        I_Error(&format!(
            "{}:{}: Z_ChangeTag: block without a ZONEID!",
            file, line,
        ));
    }
    if tag >= PU_PURGELEVEL as i32 && (*header).user.is_null() {
        I_Error(&format!(
            "{}:{}: Z_ChangeTag: an owner is required for purgable blocks",
            file, line,
        ));
    }
    (*header).tag = tag;
}
