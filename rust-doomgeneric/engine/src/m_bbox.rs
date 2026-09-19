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
pub fn M_ClearBox(bbox: &mut [fixed_t; 4]) {
    bbox[BoxIndex::Right as usize] = INT_MIN as fixed_t;
    bbox[BoxIndex::Top as usize] = bbox[BoxIndex::Right as usize];
    bbox[BoxIndex::Left as usize] = INT_MAX as fixed_t;
    bbox[BoxIndex::Bottom as usize] = bbox[BoxIndex::Left as usize];
}
pub fn M_AddToBox(bbox: &mut [fixed_t; 4], x: fixed_t, y: fixed_t) {
    if x < bbox[BoxIndex::Left as usize] {
        bbox[BoxIndex::Left as usize] = x;
    } else if x > bbox[BoxIndex::Right as usize] {
        bbox[BoxIndex::Right as usize] = x;
    }
    if y < bbox[BoxIndex::Bottom as usize] {
        bbox[BoxIndex::Bottom as usize] = y;
    } else if y > bbox[BoxIndex::Top as usize] {
        bbox[BoxIndex::Top as usize] = y;
    }
}
