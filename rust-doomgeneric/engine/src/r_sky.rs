use crate::m_fixed::Fixed;
use crate::m_fixed::FRACUNIT;

#[derive(Default)]
pub struct RSkyState {
    pub skyflatnum: i32,
    pub skytexture: i32,
    pub skytexturemid: Fixed,
}

pub fn init_sky_map(r_sky: &mut RSkyState) {
    r_sky.skytexturemid = 100 * FRACUNIT;
}
