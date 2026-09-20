use crate::d_iwad::try_find_wadby_name;
use crate::game_state::GameState;
use crate::w_wad::w_add_file;

pub fn parse_command_line(state: &mut GameState) -> bool {
    let mut modifiedgame: bool = false;
    if let Some(files) = state.game.options.file.clone() {
        modifiedgame = true;
        // Every argument up to the next option is a file to add.
        for file in files {
            let filename = try_find_wadby_name(&mut state.game.d_iwad, &*state.assets.fs, &file);
            doom_println!(state.io.platform, " adding {}", filename);
            w_add_file(
                &mut *state.assets.fs,
                &mut *state.io.platform,
                &mut state.assets.w_wad,
                &filename,
            );
        }
    }
    modifiedgame
}
