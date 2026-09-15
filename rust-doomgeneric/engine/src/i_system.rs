use crate::game_state::GameState;
use crate::m_argv::{M_ArgvAtoi, M_CheckParmWithArgs};
use crate::m_misc::M_StrToInt;
use crate::mem_compat::malloc;
use crate::stdint_types::byte;
use crate::stdint_types::size_t;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DosMemDump {
    Dos622,
    Win98,
    DosBox,
    Custom,
}

pub struct ISystemState {
    pub exit_funcs: Vec<atexit_listentry_t>,
    pub mem_dump_custom: [u8; 10],
    pub dos_mem_dump: DosMemDump,
    pub get_memory_value_firsttime: bool,
}

impl ISystemState {
    pub fn new() -> Self {
        ISystemState {
            exit_funcs: Vec::new(),
            mem_dump_custom: [0; 10],
            dos_mem_dump: DosMemDump::Dos622,
            get_memory_value_firsttime: true,
        }
    }

    fn dos_mem_dump_bytes(&self) -> &[u8; 10] {
        match self.dos_mem_dump {
            DosMemDump::Dos622 => &mem_dump_dos622,
            DosMemDump::Win98 => &mem_dump_win98,
            DosMemDump::DosBox => &mem_dump_dosbox,
            DosMemDump::Custom => &self.mem_dump_custom,
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
pub fn I_AtExit(state: &mut ISystemState, mut func: atexit_func_t, mut run_on_error: bool) {
    state.exit_funcs.push(atexit_listentry_t {
        func,
        run_on_error,
    });
}
pub fn I_Tactile() {}
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
pub fn I_PrintBanner(msg: &str) {
    let spaces = 35usize.saturating_sub(msg.len() / 2);
    print!("{}", " ".repeat(spaces));
    println!("{}", msg);
}
pub fn I_PrintDivider() {
    println!("{}", "=".repeat(75));
}
pub fn I_PrintStartupBanner(gamedescription: &str) {
    I_PrintDivider();
    I_PrintBanner(gamedescription);
    I_PrintDivider();
    print!(
        " Doom Generic is free software, covered by the GNU General Public\n License.  There is NO warranty; not even for MERCHANTABILITY or FITNESS\n FOR A PARTICULAR PURPOSE. You are welcome to change and distribute\n copies under certain conditions. See the source for more information.\n"
    );
    I_PrintDivider();
}
pub fn I_ConsoleStdout() -> bool {
    return false;
}
pub fn I_Quit(state: &mut GameState) {
    let entries = state.i_system.exit_funcs.clone();
    for entry in entries.iter().rev() {
        unsafe {
            entry.func.expect("non-null function pointer")(state);
        }
    }
}
pub fn I_Error(message: &str) -> ! {
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
                state.i_system.dos_mem_dump = DosMemDump::Dos622;
            }
            if state.m_argv.myargv[(p + 1 as i32) as usize]
                .as_bytes()
                .eq_ignore_ascii_case(b"dos71")
            {
                state.i_system.dos_mem_dump = DosMemDump::Win98;
            } else if state.m_argv.myargv[(p + 1 as i32) as usize]
                .as_bytes()
                .eq_ignore_ascii_case(b"dosbox")
            {
                state.i_system.dos_mem_dump = DosMemDump::DosBox;
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
                state.i_system.dos_mem_dump = DosMemDump::Custom;
            }
        }
    }
    let dump = state.i_system.dos_mem_dump_bytes();
    let offset = offset as usize;
    match size {
        1 => {
            *(value as *mut u8) = dump[offset];
            return true;
        }
        2 => {
            *(value as *mut u16) = dump[offset] as u16 | (dump[offset + 1] as u16) << 8;
            return true;
        }
        4 => {
            *(value as *mut u32) = dump[offset] as u32
                | (dump[offset + 1] as u32) << 8
                | (dump[offset + 2] as u32) << 16
                | (dump[offset + 3] as u32) << 24;
            return true;
        }
        _ => {}
    }
    return false;
}
