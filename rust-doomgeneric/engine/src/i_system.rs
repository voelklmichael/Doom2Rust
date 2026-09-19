use crate::game_state::GameState;
use crate::m_argv::check_parm_with_args;
use crate::m_argv::MArgvState;
use crate::m_misc::str_to_int;
use crate::platform::DoomPlatform;
use alloc::vec::Vec;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DosMemDump {
    Dos622,
    Win98,
    DosBox,
    Custom,
}

pub struct ISystemState {
    pub exit_funcs: Vec<AtexitListentry>,
    pub mem_dump_custom: [u8; 10],
    pub dos_mem_dump: DosMemDump,
    pub get_memory_value_firsttime: bool,
}

impl Default for ISystemState {
    fn default() -> Self {
        Self::new()
    }
}

impl ISystemState {
    pub fn new() -> Self {
        Self {
            exit_funcs: Vec::new(),
            mem_dump_custom: [0; 10],
            dos_mem_dump: DosMemDump::Dos622,
            get_memory_value_firsttime: true,
        }
    }

    fn dos_mem_dump_bytes(&self) -> &[u8; 10] {
        match self.dos_mem_dump {
            DosMemDump::Dos622 => &MEM_DUMP_DOS622,
            DosMemDump::Win98 => &MEM_DUMP_WIN98,
            DosMemDump::DosBox => &MEM_DUMP_DOSBOX,
            DosMemDump::Custom => &self.mem_dump_custom,
        }
    }
}

pub type AtexitFunc = Option<fn(&mut GameState) -> ()>;
#[derive(Copy, Clone)]
pub struct AtexitListentry {
    pub func: AtexitFunc,
    pub run_on_error: bool,
}
pub fn at_exit(state: &mut ISystemState, func: AtexitFunc, run_on_error: bool) {
    state
        .exit_funcs
        .push(AtexitListentry { func, run_on_error });
}
pub fn tactile() {}
pub fn print_banner(platform: &mut dyn DoomPlatform, msg: &str) {
    let spaces = 35usize.saturating_sub(msg.len() / 2);
    doom_print!(platform, "{}", " ".repeat(spaces));
    doom_println!(platform, "{}", msg);
}
pub fn print_divider(platform: &mut dyn DoomPlatform) {
    doom_println!(platform, "{}", "=".repeat(75));
}
pub fn print_startup_banner(platform: &mut dyn DoomPlatform, gamedescription: &str) {
    print_divider(platform);
    print_banner(platform, gamedescription);
    print_divider(platform);
    doom_print!(platform,
        " Doom Generic is free software, covered by the GNU General Public\n License.  There is NO warranty; not even for MERCHANTABILITY or FITNESS\n FOR A PARTICULAR PURPOSE. You are welcome to change and distribute\n copies under certain conditions. See the source for more information.\n"
    );
    print_divider(platform);
}
pub fn console_stdout() -> bool {
    false
}
pub fn i_quit(state: &mut GameState) {
    let entries = state.i_system.exit_funcs.clone();
    for entry in entries.iter().rev() {
        entry.func.expect("non-null function pointer")(state);
    }
}
pub fn error(message: &str) -> ! {
    panic!("{}", message)
}
pub const DOS_MEM_DUMP_SIZE: i32 = 10;
static MEM_DUMP_DOS622: [u8; 10] = [0x57, 0x92, 0x19, 0, 0xf4, 0x6, 0x70, 0, 0x16, 0];
static MEM_DUMP_WIN98: [u8; 10] = [0x9e, 0xf, 0xc9, 0, 0x65, 0x4, 0x70, 0, 0x16, 0];
static MEM_DUMP_DOSBOX: [u8; 10] = [0, 0, 0, 0xf1, 0, 0, 0, 0, 0x7, 0];
pub fn get_memory_value(
    i_system: &mut ISystemState,
    m_argv: &MArgvState,
    offset: u32,
    size: i32,
) -> Option<u32> {
    if i_system.get_memory_value_firsttime {
        let _p: i32;
        let mut i: i32;
        let mut val: i32 = 0;
        i_system.get_memory_value_firsttime = false;
        if let Some(mut p) = check_parm_with_args(m_argv, "-setmem", 1) {
            if m_argv.myargv[p + 1]
                .as_bytes()
                .eq_ignore_ascii_case(b"dos622")
            {
                i_system.dos_mem_dump = DosMemDump::Dos622;
            }
            if m_argv.myargv[p + 1]
                .as_bytes()
                .eq_ignore_ascii_case(b"dos71")
            {
                i_system.dos_mem_dump = DosMemDump::Win98;
            } else if m_argv.myargv[p + 1]
                .as_bytes()
                .eq_ignore_ascii_case(b"dosbox")
            {
                i_system.dos_mem_dump = DosMemDump::DosBox;
            } else {
                i = 0;
                while i < DOS_MEM_DUMP_SIZE {
                    p += 1;
                    if p >= m_argv.myargv.len()
                        || m_argv.myargv[p].as_bytes().first() == Some(&b'-')
                    {
                        break;
                    }
                    str_to_int(m_argv.myargv[p].as_str(), &mut val);
                    let fresh0 = i;
                    i += 1;
                    i_system.mem_dump_custom[fresh0 as usize] = val as u8;
                    i += 1;
                }
                i_system.dos_mem_dump = DosMemDump::Custom;
            }
        }
    }
    let dump = i_system.dos_mem_dump_bytes();
    let offset = offset as usize;
    match size {
        1 => Some(dump[offset] as u32),
        2 => Some(dump[offset] as u32 | (dump[offset + 1] as u32) << 8),
        4 => Some(
            dump[offset] as u32
                | (dump[offset + 1] as u32) << 8
                | (dump[offset + 2] as u32) << 16
                | (dump[offset + 3] as u32) << 24,
        ),
        _ => None,
    }
}
