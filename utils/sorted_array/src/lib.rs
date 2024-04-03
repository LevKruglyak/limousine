//! Adapted from: [StackMap](https://github.com/komora-io/stack-map)
//!
//! A zero-allocation sorted array data structure, similar to SmallVec.

#![no_std]
#![deny(missing_docs)]

mod entry;
pub use entry::SortedArrayEntry;

#[cfg(feature = "serde")]
mod serde;

use core::mem::MaybeUninit;
use slice_search::*;

/// A constant-size, zero-allocation associative container based on a sorted array.
pub struct SortedArray<K, V, const N: usize> {
    inner: [MaybeUninit<SortedArrayEntry<K, V>>; N],
    len: usize,
}

#[allow(unused)]
impl<K, V, const N: usize> SortedArray<K, V, N> {
    /// Create an empty sorted array
    pub fn empty() -> Self {
        SortedArray {
            inner: unsafe {
                MaybeUninit::<[MaybeUninit<SortedArrayEntry<K, V>>; N]>::uninit().assume_init()
            },
            len: 0,
        }
    }

    /// Utility method to search the array by key
    fn search(&self, key: &K) -> Result<usize, usize>
    where
        K: Ord,
    {
        OptimalSearch::search_by_key(self.entries(), key)
    }

    /// Return an entry which is an exact match for the key
    pub fn get_exact(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        if let Ok(index) = self.search(key) {
            Some(unsafe { &self.inner.get_unchecked(index).assume_init_ref().value })
        } else {
            None
        }
    }

    /// Inserts an item and return the previous value if it exists.
    ///
    /// # Panics
    ///
    /// This method will panic if called with a new key-value pair when
    /// already full.
    ///
    /// The `SortedArray` should be checked to ensure that it is not already
    /// full before calling this method. It is full when the `self.is_full()`
    /// method returns `true`, which happens when `self.len() == N`.
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        match self.search(&key) {
            Ok(index) => {
                let slot =
                    unsafe { &mut self.inner.get_unchecked_mut(index).assume_init_mut().value };
                Some(core::mem::replace(slot, value))
            }
            Err(index) => {
                assert!(self.len() < N);

                unsafe {
                    if index < self.len() {
                        let src = self.inner.get_unchecked(index).as_ptr();
                        let dst = self.inner.get_unchecked_mut(index + 1).as_mut_ptr();

                        core::ptr::copy(src, dst, self.len() - index);
                    }

                    self.len += 1;

                    self.inner
                        .get_unchecked_mut(index)
                        .write(SortedArrayEntry::new(key, value));
                }
                None
            }
        }
    }

    /// Remove an element from the sorted array
    pub fn remove(&mut self, key: &K) -> Option<V>
    where
        K: Ord,
    {
        // TODO: fix undefined behavior here
        if let Ok(index) = self.search(key) {
            unsafe {
                let ret = core::ptr::read(self.inner.get_unchecked(index).as_ptr()).value;

                if index + 1 < self.len() {
                    let dst = self.inner.get_unchecked_mut(index).as_mut_ptr();
                    let src = self.inner.get_unchecked(index + 1).as_ptr();

                    core::ptr::copy(src, dst, self.len() - index);
                }

                self.len -= 1;

                Some(ret)
            }
        } else {
            None
        }
    }

    /// Check if the array contains a given element.
    pub fn contains_key(&self, key: &K) -> bool
    where
        K: Ord,
    {
        self.search(key).is_ok()
    }

    /// A double ended iterator through the entries of the array.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &SortedArrayEntry<K, V>> {
        self.entries().iter()
    }

    /// Splits this `SortedArray` into two. `self` will retain
    /// all key-value pairs before the provided split index.
    /// Returns a new `SortedArray` created out of all key-value pairs
    /// at or after the provided split index.
    pub fn split_off(&mut self, split_idx: usize) -> Self {
        debug_assert!(split_idx < self.len());
        debug_assert!(split_idx < N);

        let mut rhs = Self::empty();

        for i in split_idx..self.len() {
            let src = self.inner[i].as_ptr();
            let dst = rhs.inner[i - split_idx].as_mut_ptr();
            unsafe {
                core::ptr::copy_nonoverlapping(src, dst, 1);
            }
        }

        rhs.len = self.len - split_idx;
        self.len = split_idx;

        rhs
    }

    /// Get the key-value pair that is less than or equal
    /// to the provided key.
    pub fn get_lower_bound(&self, key: &K) -> Option<&V>
    where
        K: Ord,
    {
        // binary search LUB
        if let Some(index) = lower_bound(self.search(key)) {
            return Some(&self.entries()[index].value);
        }

        None
    }

    /// Get the key-value pair that is less than or equal
    /// to the provided key, or the first key-value pair.
    pub fn get_lower_bound_always(&self, key: &K) -> &V
    where
        K: Ord,
    {
        // binary search LUB
        let index = lower_bound_always(self.search(key));
        &self.entries()[index].value
    }
}

#[allow(unused)]
impl<K, V, const N: usize> SortedArray<K, V, N> {
    /// Borrow a slice view into the entries stored in the `SortedArray`
    pub fn entries(&self) -> &[SortedArrayEntry<K, V>] {
        // SAFETY: `len` must be strictly less than `F`
        debug_assert!(self.len <= N);
        let slice = unsafe { self.inner.get_unchecked(..self.len) };

        // SAFETY: feature `maybe_uninit_slice`
        unsafe {
            &*(slice as *const [MaybeUninit<SortedArrayEntry<K, V>>]
                as *const [SortedArrayEntry<K, V>])
        }
    }

    /// Get a key-value pair based on its internal relative
    /// index in the backing array.
    pub fn get_index(&self, index: usize) -> Option<&SortedArrayEntry<K, V>> {
        if index < self.len() {
            Some(unsafe { self.inner.get_unchecked(index).assume_init_ref() })
        } else {
            None
        }
    }

    /// Returns the first key-value pair in the array, if any exists.
    pub fn first(&self) -> Option<&SortedArrayEntry<K, V>> {
        self.get_index(0)
    }

    /// Returns the last key-value pair in the array, if any exists.
    pub fn last(&self) -> Option<&SortedArrayEntry<K, V>> {
        if self.is_empty() {
            None
        } else {
            self.get_index(self.len - 1)
        }
    }

    /// Returns whether this array is at its maximum capacity and
    /// unable to receive additional data.
    pub fn is_full(&self) -> bool {
        self.len == N
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the array has any elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<K, V, const N: usize> PartialEq for SortedArray<K, V, N>
where
    K: PartialEq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.entries().eq(other.entries())
    }
}

use core::fmt::Debug;
impl<K, V, const N: usize> Eq for SortedArray<K, V, N>
where
    K: PartialEq,
    V: PartialEq,
{
}

impl<K: Debug, V: Debug, const N: usize> Debug for SortedArray<K, V, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.entries().iter()).finish()
    }
}

impl<K: Clone, V: Clone, const N: usize> Clone for SortedArray<K, V, N> {
    fn clone(&self) -> Self {
        let mut inner: [MaybeUninit<SortedArrayEntry<K, V>>; N] = unsafe {
            MaybeUninit::<[MaybeUninit<SortedArrayEntry<K, V>>; N]>::uninit().assume_init()
        };

        for (i, item) in self.iter().cloned().enumerate() {
            inner[i].write(item);
        }

        SortedArray {
            inner,
            len: self.len,
        }
    }
}

impl<K, V, const N: usize> Default for SortedArray<K, V, N> {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::entry::SortedArrayEntry;
    use crate::SortedArray;

    #[test]
    fn test_insert_and_get() {
        let mut stack_map: SortedArray<u32, &str, 3> = SortedArray::empty();
        assert!(stack_map.insert(1, "one").is_none());
        assert!(stack_map.insert(2, "two").is_none());
        assert!(stack_map.insert(3, "three").is_none());

        assert_eq!(stack_map.get_exact(&1), Some(&"one"));
        assert_eq!(stack_map.get_exact(&2), Some(&"two"));
        assert_eq!(stack_map.get_exact(&3), Some(&"three"));
        assert_eq!(stack_map.get_exact(&4), None);
    }

    #[test]
    fn test_remove() {
        let mut stack_map: SortedArray<u32, &str, 3> = SortedArray::empty();
        stack_map.insert(1, "one");
        stack_map.insert(2, "two");
        stack_map.insert(3, "three");

        assert_eq!(stack_map.remove(&2), Some("two"));
        assert_eq!(stack_map.remove(&2), None);
        assert_eq!(stack_map.get_exact(&2), None);
    }

    #[test]
    fn test_contains_key() {
        let mut stack_map: SortedArray<u32, &str, 3> = SortedArray::empty();
        stack_map.insert(1, "one");
        stack_map.insert(2, "two");

        assert!(stack_map.contains_key(&1));
        assert!(stack_map.contains_key(&2));
        assert!(!stack_map.contains_key(&3));
    }

    #[test]
    fn test_iter() {
        let mut stack_map: SortedArray<u32, &str, 3> = SortedArray::empty();
        stack_map.insert(1, "one");
        stack_map.insert(2, "two");
        stack_map.insert(3, "three");

        let mut iter = stack_map.iter();
        assert_eq!(iter.next(), Some(&SortedArrayEntry::new(1, "one")));
        assert_eq!(iter.next_back(), Some(&SortedArrayEntry::new(3, "three")));
        assert_eq!(iter.next(), Some(&SortedArrayEntry::new(2, "two")));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_split_off() {
        let mut stack_map: SortedArray<u32, &str, 4> = SortedArray::empty();
        stack_map.insert(1, "one");
        stack_map.insert(2, "two");
        stack_map.insert(3, "three");
        stack_map.insert(4, "four");

        let split_map = stack_map.split_off(2);
        assert_eq!(stack_map.len(), 2);
        assert_eq!(split_map.len(), 2);
        assert_eq!(stack_map.get_exact(&1), Some(&"one"));
        assert_eq!(stack_map.get_exact(&2), Some(&"two"));
        assert_eq!(split_map.get_exact(&3), Some(&"three"));
        assert_eq!(split_map.get_exact(&4), Some(&"four"));
    }

    #[test]
    fn test_get_lower_bound() {
        let mut stack_map: SortedArray<u32, &str, 4> = SortedArray::empty();
        stack_map.insert(1, "one");
        stack_map.insert(3, "three");
        stack_map.insert(5, "five");
        stack_map.insert(7, "seven");

        assert_eq!(stack_map.get_lower_bound(&2), Some(&"one"));
        assert_eq!(stack_map.get_lower_bound(&4), Some(&"three"));
        assert_eq!(stack_map.get_lower_bound(&6), Some(&"five"));
        assert_eq!(stack_map.get_lower_bound(&8), Some(&"seven"));
        assert_eq!(stack_map.get_lower_bound(&0), None);
    }
}
