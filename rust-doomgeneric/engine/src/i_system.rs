use crate::src::game_state::GameState;
use crate::src::m_argv::{M_ArgvAtoi, M_CheckParmWithArgs};
use crate::src::m_misc::M_StrToInt;
use crate::src::mem_compat::malloc;
use crate::src::stdint_types::byte;
use crate::src::stdint_types::size_t;

pub struct ISystemState {
    pub exit_funcs: Vec<atexit_listentry_t>,
    pub mem_dump_custom: [u8; 10],
    pub dos_mem_dump: *const u8,
    pub get_memory_value_firsttime: bool,
}

impl ISystemState {
    pub fn new() -> Self {
        ISystemState {
            exit_funcs: Vec::new(),
            mem_dump_custom: [0; 10],
            dos_mem_dump: &raw const mem_dump_dos622 as *const u8,
            get_memory_value_firsttime: true,
        }
    }
}

pub type atexit_func_t = Option<unsafe fn(&mut GameState) -> ()>;
pub type atexit_listentry_t = atexit_listentry_s;
#[derive(Copy, Clone)]
pub struct atexit_listentry_s {
    pub func: atexit_func_t,
    pub run_on_error: bool,
}
pub const DEFAULT_RAM: i32 = 6;
pub const MIN_RAM: i32 = 6;
pub unsafe fn I_AtExit(state: &mut ISystemState, mut func: atexit_func_t, mut run_on_error: bool) {
    state.exit_funcs.push(atexit_listentry_t {
        func,
        run_on_error,
    });
}
pub unsafe fn I_Tactile() {}
unsafe fn AutoAllocMemory(mut size: *mut i32, mut default_ram: i32, mut min_ram: i32) -> *mut byte {
    let mut zonemem: *mut byte = ::core::ptr::null_mut::<byte>();
    zonemem = ::core::ptr::null_mut::<byte>();
    while zonemem.is_null() {
        if default_ram < min_ram {
            I_Error(&format!(
                "Unable to allocate {} MiB of RAM for zone",
                default_ram
            ));
        }
        *size = default_ram * 1024 as i32 * 1024 as i32;
        zonemem = malloc(*size as size_t) as *mut byte;
        if zonemem.is_null() {
            default_ram -= 1 as i32;
        }
    }
    return zonemem;
}
pub unsafe fn I_ZoneBase(state: &mut GameState, mut size: *mut i32) -> *mut byte {
    let mut zonemem: *mut byte = ::core::ptr::null_mut::<byte>();
    let mut min_ram: i32 = 0;
    let mut default_ram: i32 = 0;
    let mut p: i32 = 0;
    p = M_CheckParmWithArgs(state, "-mb", 1 as i32);
    if p > 0 as i32 {
        default_ram = M_ArgvAtoi(&state.m_argv.myargv[(p + 1 as i32) as usize]);
        min_ram = default_ram;
    } else {
        default_ram = DEFAULT_RAM;
        min_ram = MIN_RAM;
    }
    zonemem = AutoAllocMemory(size, default_ram, min_ram);
    println!("zone memory: {:p}, {:x} allocated for zone", zonemem, *size);
    return zonemem;
}
pub unsafe fn I_PrintBanner(msg: &str) {
    let spaces = 35usize.saturating_sub(msg.len() / 2);
    print!("{}", " ".repeat(spaces));
    println!("{}", msg);
}
pub unsafe fn I_PrintDivider() {
    println!("{}", "=".repeat(75));
}
pub unsafe fn I_PrintStartupBanner(gamedescription: &str) {
    I_PrintDivider();
    I_PrintBanner(gamedescription);
    I_PrintDivider();
    print!(
        " Doom Generic is free software, covered by the GNU General Public\n License.  There is NO warranty; not even for MERCHANTABILITY or FITNESS\n FOR A PARTICULAR PURPOSE. You are welcome to change and distribute\n copies under certain conditions. See the source for more information.\n"
    );
    I_PrintDivider();
}
pub unsafe fn I_ConsoleStdout() -> bool {
    return false;
}
pub unsafe fn I_Quit(state: &mut GameState) {
    let entries = state.i_system.exit_funcs.clone();
    for entry in entries.iter().rev() {
        entry.func.expect("non-null function pointer")(state);
    }
}
pub unsafe fn I_Error(message: &str) -> ! {
    panic!("{}", message)
}
pub const DOS_MEM_DUMP_SIZE: i32 = 10;
static mem_dump_dos622: [u8; 10] = [
    0x57 as i32 as u8,
    0x92 as i32 as u8,
    0x19 as i32 as u8,
    0 as i32 as u8,
    0xf4 as i32 as u8,
    0x6 as i32 as u8,
    0x70 as i32 as u8,
    0 as i32 as u8,
    0x16 as i32 as u8,
    0 as i32 as u8,
];
static mem_dump_win98: [u8; 10] = [
    0x9e as i32 as u8,
    0xf as i32 as u8,
    0xc9 as i32 as u8,
    0 as i32 as u8,
    0x65 as i32 as u8,
    0x4 as i32 as u8,
    0x70 as i32 as u8,
    0 as i32 as u8,
    0x16 as i32 as u8,
    0 as i32 as u8,
];
static mem_dump_dosbox: [u8; 10] = [
    0 as i32 as u8,
    0 as i32 as u8,
    0 as i32 as u8,
    0xf1 as i32 as u8,
    0 as i32 as u8,
    0 as i32 as u8,
    0 as i32 as u8,
    0 as i32 as u8,
    0x7 as i32 as u8,
    0 as i32 as u8,
];
pub unsafe fn I_GetMemoryValue(
    state: &mut GameState,
    mut offset: u32,
    mut value: *mut ::core::ffi::c_void,
    mut size: i32,
) -> bool {
    if state.i_system.get_memory_value_firsttime {
        let mut p: i32 = 0;
        let mut i: i32 = 0;
        let mut val: i32 = 0;
        state.i_system.get_memory_value_firsttime = false;
        i = 0 as i32;
        p = M_CheckParmWithArgs(state, "-setmem", 1 as i32);
        if p > 0 as i32 {
            if state.m_argv.myargv[(p + 1 as i32) as usize]
                .as_bytes()
                .eq_ignore_ascii_case(b"dos622")
            {
                state.i_system.dos_mem_dump = &raw const mem_dump_dos622 as *const u8;
            }
            if state.m_argv.myargv[(p + 1 as i32) as usize]
                .as_bytes()
                .eq_ignore_ascii_case(b"dos71")
            {
                state.i_system.dos_mem_dump = &raw const mem_dump_win98 as *const u8;
            } else if state.m_argv.myargv[(p + 1 as i32) as usize]
                .as_bytes()
                .eq_ignore_ascii_case(b"dosbox")
            {
                state.i_system.dos_mem_dump = &raw const mem_dump_dosbox as *const u8;
            } else {
                i = 0 as i32;
                while i < DOS_MEM_DUMP_SIZE {
                    p += 1;
                    if p >= state.m_argv.myargv.len() as i32
                        || state.m_argv.myargv[p as usize].as_bytes().first() == Some(&b'-')
                    {
                        break;
                    }
                    M_StrToInt(
                        state.m_argv.myargv[p as usize].to_str().unwrap(),
                        &mut val,
                    );
                    let fresh0 = i;
                    i = i + 1;
                    state.i_system.mem_dump_custom[fresh0 as usize] = val as u8;
                    i += 1;
                }
                state.i_system.dos_mem_dump = &raw mut state.i_system.mem_dump_custom as *mut u8;
            }
        }
    }
    match size {
        1 => {
            *(value as *mut u8) = *state.i_system.dos_mem_dump.offset(offset as isize);
            return true;
        }
        2 => {
            *(value as *mut u16) = (*state.i_system.dos_mem_dump.offset(offset as isize) as i32
                | (*state
                    .i_system
                    .dos_mem_dump
                    .offset(offset.wrapping_add(1 as u32) as isize) as i32)
                    << 8 as i32) as u16;
            return true;
        }
        4 => {
            *(value as *mut u32) = (*state.i_system.dos_mem_dump.offset(offset as isize) as i32
                | (*state
                    .i_system
                    .dos_mem_dump
                    .offset(offset.wrapping_add(1 as u32) as isize) as i32)
                    << 8 as i32
                | (*state
                    .i_system
                    .dos_mem_dump
                    .offset(offset.wrapping_add(2 as u32) as isize) as i32)
                    << 16 as i32
                | (*state
                    .i_system
                    .dos_mem_dump
                    .offset(offset.wrapping_add(3 as u32) as isize) as i32)
                    << 24 as i32) as u32;
            return true;
        }
        _ => {}
    }
    return false;
}
