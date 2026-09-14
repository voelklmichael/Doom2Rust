use crate::m_fixed::fixed_t;
use crate::m_fixed::INT_MAX;
use crate::m_fixed::INT_MIN;
pub type C2RustUnnamed = u32;
pub const BOXRIGHT: C2RustUnnamed = 3;
pub const BOXLEFT: C2RustUnnamed = 2;
pub const BOXBOTTOM: C2RustUnnamed = 1;
pub const BOXTOP: C2RustUnnamed = 0;
pub unsafe fn M_ClearBox(mut box_0: *mut fixed_t) {
    let ref mut fresh0 = *box_0.offset(BOXRIGHT as i32 as isize);
    *fresh0 = INT_MIN as fixed_t;
    *box_0.offset(BOXTOP as i32 as isize) = *fresh0;
    let ref mut fresh1 = *box_0.offset(BOXLEFT as i32 as isize);
    *fresh1 = INT_MAX as fixed_t;
    *box_0.offset(BOXBOTTOM as i32 as isize) = *fresh1;
}
pub unsafe fn M_AddToBox(mut box_0: *mut fixed_t, mut x: fixed_t, mut y: fixed_t) {
    if x < *box_0.offset(BOXLEFT as i32 as isize) {
        *box_0.offset(BOXLEFT as i32 as isize) = x;
    } else if x > *box_0.offset(BOXRIGHT as i32 as isize) {
        *box_0.offset(BOXRIGHT as i32 as isize) = x;
    }
    if y < *box_0.offset(BOXBOTTOM as i32 as isize) {
        *box_0.offset(BOXBOTTOM as i32 as isize) = y;
    } else if y > *box_0.offset(BOXTOP as i32 as isize) {
        *box_0.offset(BOXTOP as i32 as isize) = y;
    }
}
