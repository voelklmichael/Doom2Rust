use crate::m_fixed::fixed_t;
use crate::m_fixed::INT_MAX;
use crate::m_fixed::INT_MIN;
pub type C2RustUnnamed = u32;
pub const BOXRIGHT: C2RustUnnamed = 3;
pub const BOXLEFT: C2RustUnnamed = 2;
pub const BOXBOTTOM: C2RustUnnamed = 1;
pub const BOXTOP: C2RustUnnamed = 0;
pub fn M_ClearBox(box_0: &mut [fixed_t; 4]) {
    box_0[BOXRIGHT as usize] = INT_MIN as fixed_t;
    box_0[BOXTOP as usize] = box_0[BOXRIGHT as usize];
    box_0[BOXLEFT as usize] = INT_MAX as fixed_t;
    box_0[BOXBOTTOM as usize] = box_0[BOXLEFT as usize];
}
pub fn M_AddToBox(box_0: &mut [fixed_t; 4], x: fixed_t, y: fixed_t) {
    if x < box_0[BOXLEFT as usize] {
        box_0[BOXLEFT as usize] = x;
    } else if x > box_0[BOXRIGHT as usize] {
        box_0[BOXRIGHT as usize] = x;
    }
    if y < box_0[BOXBOTTOM as usize] {
        box_0[BOXBOTTOM as usize] = y;
    } else if y > box_0[BOXTOP as usize] {
        box_0[BOXTOP as usize] = y;
    }
}
