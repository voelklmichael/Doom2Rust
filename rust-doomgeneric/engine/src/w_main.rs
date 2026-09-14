use crate::d_iwad::D_TryFindWADByName;
use crate::game_state::GameState;
use crate::m_argv::M_CheckParmWithArgs;
use crate::w_wad::W_AddFile;

pub unsafe fn W_ParseCommandLine(state: &mut GameState) -> bool {
    let mut modifiedgame: bool = false;
    let mut p: i32 = 0;
    p = M_CheckParmWithArgs(state, "-file", 1 as i32);
    if p != 0 {
        modifiedgame = true;
        loop {
            p += 1;
            if !(p != state.m_argv.myargv.len() as i32
                && state.m_argv.myargv[p as usize].as_bytes().first() != Some(&b'-'))
            {
                break;
            }
            let filename = D_TryFindWADByName(
                &mut state.d_iwad,
                state.m_argv.myargv[p as usize].to_str().unwrap(),
            );
            println!(" adding {}", filename);
            W_AddFile(state, &filename);
        }
    }
    return modifiedgame;
}
