use crate::m_fixed::fixed_t;
use crate::m_fixed::INT_MAX;
use crate::m_fixed::INT_MIN;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BoxIndex {
    BOXTOP = 0,
    BOXBOTTOM = 1,
    BOXLEFT = 2,
    BOXRIGHT = 3,
}
pub fn M_ClearBox(box_0: &mut [fixed_t; 4]) {
    box_0[BoxIndex::BOXRIGHT as usize] = INT_MIN as fixed_t;
    box_0[BoxIndex::BOXTOP as usize] = box_0[BoxIndex::BOXRIGHT as usize];
    box_0[BoxIndex::BOXLEFT as usize] = INT_MAX as fixed_t;
    box_0[BoxIndex::BOXBOTTOM as usize] = box_0[BoxIndex::BOXLEFT as usize];
}
pub fn M_AddToBox(box_0: &mut [fixed_t; 4], x: fixed_t, y: fixed_t) {
    if x < box_0[BoxIndex::BOXLEFT as usize] {
        box_0[BoxIndex::BOXLEFT as usize] = x;
    } else if x > box_0[BoxIndex::BOXRIGHT as usize] {
        box_0[BoxIndex::BOXRIGHT as usize] = x;
    }
    if y < box_0[BoxIndex::BOXBOTTOM as usize] {
        box_0[BoxIndex::BOXBOTTOM as usize] = y;
    } else if y > box_0[BoxIndex::BOXTOP as usize] {
        box_0[BoxIndex::BOXTOP as usize] = y;
    }
}
