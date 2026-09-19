use crate::d_iwad::try_find_wadby_name;
use crate::game_state::GameState;
use crate::m_argv::check_parm_with_args;
use crate::w_wad::w_add_file;

pub fn parse_command_line(state: &mut GameState) -> bool {
    let mut modifiedgame: bool = false;
    let mut p: i32;
    p = check_parm_with_args(state, "-file", 1);
    if p != 0 {
        modifiedgame = true;
        loop {
            p += 1;
            if !(p != state.m_argv.myargv.len() as i32
                && state.m_argv.myargv[p as usize].as_bytes().first() != Some(&b'-'))
            {
                break;
            }
            let filename = try_find_wadby_name(
                &mut state.d_iwad,
                &*state.fs,
                state.m_argv.myargv[p as usize].as_str(),
            );
            doom_println!(state.platform, " adding {}", filename);
            w_add_file(state, &filename);
        }
    }
    modifiedgame
}
