//! Fixed-size arrays indexed by a C-style enum instead of a bare integer.
//!
//! The C code indexes tables with enum constants (`weaponowned[wp_shotgun]`); the port kept that
//! as `weaponowned[WeaponType::Shotgun]`. [`EnumArray`] lets the enum itself be the index
//! (`weaponowned[WeaponType::Shotgun]`), so a `PowerType` cannot index an ammo table. A plain
//! `usize` still works for the loops that walk every slot.
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

/// A type that names a slot of an [`EnumArray`].
pub trait ArrayIndex: Copy {
    /// The array position this value stands for.
    fn slot(self) -> usize;
}

/// An array of `N` values of type `V`, indexed by `K`.
pub struct EnumArray<K, V, const N: usize>([V; N], PhantomData<fn(K)>);

impl<K, V, const N: usize> EnumArray<K, V, N> {
    pub const fn new(values: [V; N]) -> Self {
        Self(values, PhantomData)
    }

    pub const fn len(&self) -> usize {
        N
    }

    pub const fn is_empty(&self) -> bool {
        N == 0
    }

    pub fn iter(&self) -> core::slice::Iter<'_, V> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, V> {
        self.0.iter_mut()
    }

    pub fn as_slice(&self) -> &[V] {
        &self.0
    }

    pub fn as_mut_slice(&mut self) -> &mut [V] {
        &mut self.0
    }
}

impl<K: ArrayIndex, V, const N: usize> Index<K> for EnumArray<K, V, N> {
    type Output = V;
    fn index(&self, key: K) -> &V {
        &self.0[key.slot()]
    }
}

impl<K: ArrayIndex, V, const N: usize> IndexMut<K> for EnumArray<K, V, N> {
    fn index_mut(&mut self, key: K) -> &mut V {
        &mut self.0[key.slot()]
    }
}

impl<K, V, const N: usize> Index<usize> for EnumArray<K, V, N> {
    type Output = V;
    fn index(&self, slot: usize) -> &V {
        &self.0[slot]
    }
}

impl<K, V, const N: usize> IndexMut<usize> for EnumArray<K, V, N> {
    fn index_mut(&mut self, slot: usize) -> &mut V {
        &mut self.0[slot]
    }
}

impl<'a, K, V, const N: usize> IntoIterator for &'a EnumArray<K, V, N> {
    type Item = &'a V;
    type IntoIter = core::slice::Iter<'a, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, K, V, const N: usize> IntoIterator for &'a mut EnumArray<K, V, N> {
    type Item = &'a mut V;
    type IntoIter = core::slice::IterMut<'a, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

// Manual impls: deriving would demand `K: Clone` etc. of the phantom key type.
impl<K, V: Clone, const N: usize> Clone for EnumArray<K, V, N> {
    fn clone(&self) -> Self {
        Self::new(self.0.clone())
    }
}

impl<K, V: Copy, const N: usize> Copy for EnumArray<K, V, N> {}

impl<K, V: PartialEq, const N: usize> PartialEq for EnumArray<K, V, N> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K, V: core::fmt::Debug, const N: usize> core::fmt::Debug for EnumArray<K, V, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<K, V: Default, const N: usize> Default for EnumArray<K, V, N>
where
    [V; N]: Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    enum Colour {
        Red = 0,
        Blue = 2,
    }

    impl ArrayIndex for Colour {
        fn slot(self) -> usize {
            self as usize
        }
    }

    #[test]
    fn an_enum_or_a_slot_number_indexes() {
        let mut table: EnumArray<Colour, i32, 3> = EnumArray::new([10, 20, 30]);
        assert_eq!(table[Colour::Red], 10);
        table[Colour::Blue] += 5;
        assert_eq!(table[2usize], 35);
        assert_eq!(table.iter().sum::<i32>(), 65);
        assert_eq!(table.len(), 3);
    }

    #[test]
    #[should_panic]
    fn a_key_outside_the_table_panics_like_a_bad_index() {
        let table: EnumArray<Colour, u8, 2> = EnumArray::new([1, 2]);
        let _ = table[Colour::Blue];
    }
}
