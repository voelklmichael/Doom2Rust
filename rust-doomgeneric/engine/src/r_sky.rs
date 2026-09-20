use crate::m_fixed::Fixed;
use crate::m_fixed::FRACUNIT;

pub struct RSkyState {
    pub skyflatnum: i32,
    pub skytexture: i32,
    pub skytexturemid: Fixed,
}

impl Default for RSkyState {
    fn default() -> Self {
        Self::new()
    }
}

impl RSkyState {
    pub const fn new() -> Self {
        Self {
            skyflatnum: 0,
            skytexture: 0,
            skytexturemid: Fixed::ZERO,
        }
    }
}

pub fn init_sky_map(r_sky: &mut RSkyState) {
    r_sky.skytexturemid = 100 * FRACUNIT;
}
