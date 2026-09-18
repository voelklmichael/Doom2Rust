use crate::m_fixed::fixed_t;
use crate::m_fixed::INT_MAX;
use crate::m_fixed::INT_MIN;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BoxIndex {
    Top = 0,
    Bottom = 1,
    Left = 2,
    Right = 3,
}
pub fn M_ClearBox(box_0: &mut [fixed_t; 4]) {
    box_0[BoxIndex::Right as usize] = INT_MIN as fixed_t;
    box_0[BoxIndex::Top as usize] = box_0[BoxIndex::Right as usize];
    box_0[BoxIndex::Left as usize] = INT_MAX as fixed_t;
    box_0[BoxIndex::Bottom as usize] = box_0[BoxIndex::Left as usize];
}
pub fn M_AddToBox(box_0: &mut [fixed_t; 4], x: fixed_t, y: fixed_t) {
    if x < box_0[BoxIndex::Left as usize] {
        box_0[BoxIndex::Left as usize] = x;
    } else if x > box_0[BoxIndex::Right as usize] {
        box_0[BoxIndex::Right as usize] = x;
    }
    if y < box_0[BoxIndex::Bottom as usize] {
        box_0[BoxIndex::Bottom as usize] = y;
    } else if y > box_0[BoxIndex::Top as usize] {
        box_0[BoxIndex::Top as usize] = y;
    }
}
