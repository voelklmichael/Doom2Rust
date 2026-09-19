use crate::m_fixed::Fixed;
use crate::m_fixed::INT_MAX;
use crate::m_fixed::INT_MIN;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BoxIndex {
    Top = 0,
    Bottom = 1,
    Left = 2,
    Right = 3,
}
pub fn clear_box(bbox: &mut [Fixed; 4]) {
    bbox[BoxIndex::Right as usize] = INT_MIN as Fixed;
    bbox[BoxIndex::Top as usize] = bbox[BoxIndex::Right as usize];
    bbox[BoxIndex::Left as usize] = INT_MAX as Fixed;
    bbox[BoxIndex::Bottom as usize] = bbox[BoxIndex::Left as usize];
}
pub fn add_to_box(bbox: &mut [Fixed; 4], x: Fixed, y: Fixed) {
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
