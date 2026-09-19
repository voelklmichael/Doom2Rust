use crate::game_state::GameState;
use crate::m_fixed::FRACUNIT;

pub struct RSkyState {
    pub skyflatnum: i32,
    pub skytexture: i32,
    pub skytexturemid: i32,
}

impl Default for RSkyState {
    fn default() -> Self {
        Self::new()
    }
}

impl RSkyState {
    pub const fn new() -> Self {
        RSkyState {
            skyflatnum: 0,
            skytexture: 0,
            skytexturemid: 0,
        }
    }
}

pub fn init_sky_map(state: &mut GameState) {
    state.r_sky.skytexturemid = 100 * FRACUNIT;
}
