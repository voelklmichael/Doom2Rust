use crate::enum_array::{ArrayIndex, EnumArray};
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
/// A bounding box: the four `BoxIndex` extents.
pub type BBox = EnumArray<BoxIndex, Fixed, 4>;

impl ArrayIndex for BoxIndex {
    #[inline(always)]
    fn slot(self) -> usize {
        self as usize
    }
}

pub fn clear_box(bbox: &mut BBox) {
    bbox[BoxIndex::Right] = INT_MIN as Fixed;
    bbox[BoxIndex::Top] = bbox[BoxIndex::Right];
    bbox[BoxIndex::Left] = INT_MAX as Fixed;
    bbox[BoxIndex::Bottom] = bbox[BoxIndex::Left];
}
pub fn add_to_box(bbox: &mut BBox, x: Fixed, y: Fixed) {
    if x < bbox[BoxIndex::Left] {
        bbox[BoxIndex::Left] = x;
    } else if x > bbox[BoxIndex::Right] {
        bbox[BoxIndex::Right] = x;
    }
    if y < bbox[BoxIndex::Bottom] {
        bbox[BoxIndex::Bottom] = y;
    } else if y > bbox[BoxIndex::Top] {
        bbox[BoxIndex::Top] = y;
    }
}
