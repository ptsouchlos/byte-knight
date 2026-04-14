// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use arrayvec::ArrayVec;

/// A fixed-capacity list backed by [`ArrayVec`]. Panics on overflow.
pub struct ArrayVecList<T, const N: usize> {
    items: ArrayVec<T, N>,
}

impl<T, const N: usize> Default for ArrayVecList<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> ArrayVecList<T, N> {
    /// Create a new, empty list.
    #[inline]
    pub fn new() -> Self {
        Self {
            items: ArrayVec::new(),
        }
    }

    /// Returns the number of items in the list.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Push an item to the list. Panics if the list is full.
    #[inline]
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Get an iterator over the items in the list.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    /// Get the item at the given index. Returns `None` if out of bounds.
    #[inline]
    pub fn at(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    /// Clear the list.
    #[inline]
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Return the list as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.items.as_slice()
    }

    /// Return the list as a mutable slice.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.items.as_mut_slice()
    }
}
