// Adapted from: [StackMap](https://github.com/komora-io/stack-map)

use crate::common::entry::Entry;
use crate::common::search::*;
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::mem::MaybeUninit;

/// `StackMap` is a constant-size, zero-allocation associative container
/// backed by an array.
pub struct StackMap<K, V, const FANOUT: usize> {
    inner: [MaybeUninit<Entry<K, V>>; FANOUT],
    len: usize,
}

impl<K, V, const FANOUT: usize> PartialEq for StackMap<K, V, FANOUT>
where
    K: PartialEq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        // TODO: check if this ``optimization'' is necessary
        if self.len == other.len {
            return self.entries().eq(other.entries());
        }

        false
    }
}

impl<K, V, const FANOUT: usize> Eq for StackMap<K, V, FANOUT>
where
    K: PartialEq,
    V: PartialEq,
{
}

impl<K, V, const FANOUT: usize> Serialize for StackMap<K, V, FANOUT>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len))?;
        for entry in self.entries() {
            seq.serialize_element(entry)?;
        }
        seq.end()
    }
}

struct StackMapDeserializer<K, V, const FANOUT: usize>(
    std::marker::PhantomData<(K, V, [(); FANOUT])>,
);

impl<'de, K, V, const FANOUT: usize> Visitor<'de> for StackMapDeserializer<K, V, FANOUT>
where
    K: Deserialize<'de> + Ord + Copy,
    V: Deserialize<'de>,
{
    type Value = StackMap<K, V, FANOUT>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence of entries for StackMap")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut map = StackMap::<K, V, FANOUT>::empty();

        while let Some(entry) = seq.next_element::<Entry<K, V>>()? {
            if map.len() >= FANOUT {
                return Err(serde::de::Error::custom(
                    "StackMap exceeded its capacity during deserialization",
                ));
            }
            map.insert(entry.key, entry.value);
        }

        Ok(map)
    }
}

impl<'de, K, V, const FANOUT: usize> Deserialize<'de> for StackMap<K, V, FANOUT>
where
    K: Serialize + Deserialize<'de> + Ord + Copy,
    V: Serialize + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(StackMapDeserializer(std::marker::PhantomData))
    }
}

impl<K: Debug, V: Debug, const FANOUT: usize> Debug for StackMap<K, V, FANOUT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.entries().iter()).finish()
    }
}

impl<K: Clone, V: Clone, const FANOUT: usize> Clone for StackMap<K, V, FANOUT> {
    fn clone(&self) -> Self {
        let mut inner: [MaybeUninit<Entry<K, V>>; FANOUT] =
            unsafe { MaybeUninit::<[MaybeUninit<Entry<K, V>>; FANOUT]>::uninit().assume_init() };

        for (i, item) in self.iter().cloned().enumerate() {
            inner[i].write(item);
        }

        StackMap {
            inner,
            len: self.len,
        }
    }
}

impl<K, V, const FANOUT: usize> Default for StackMap<K, V, FANOUT> {
    fn default() -> Self {
        Self::empty()
    }
}

#[allow(unused)]
impl<K, V, const FANOUT: usize> StackMap<K, V, FANOUT> {
    pub fn empty() -> Self {
        StackMap {
            inner: unsafe {
                MaybeUninit::<[MaybeUninit<Entry<K, V>>; FANOUT]>::uninit().assume_init()
            },
            len: 0,
        }
    }

    fn search(&self, key: &K) -> Result<usize, usize>
    where
        K: Ord + Copy,
    {
        // TODO: Experimentally determine
        if std::mem::size_of::<Entry<K, V>>() * FANOUT > 8 * 64 {
            BinarySearch::search_by_key(self.entries(), key)
        } else {
            LinearSearch::search_by_key(self.entries(), key)
        }
    }

    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Ord + Copy,
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
    /// The `StackMap` should be checked to ensure that it is not already
    /// full before calling this method. It is full when the `self.is_full()`
    /// method returns `true`, which happens when `self.len() == FANOUT`.
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord + Copy,
    {
        match self.search(&key) {
            Ok(index) => {
                let slot =
                    unsafe { &mut self.inner.get_unchecked_mut(index).assume_init_mut().value };
                Some(std::mem::replace(slot, value))
            }
            Err(index) => {
                assert!(self.len() < FANOUT);

                unsafe {
                    if index < self.len() {
                        let src = self.inner.get_unchecked(index).as_ptr();
                        let dst = self.inner.get_unchecked_mut(index + 1).as_mut_ptr();

                        std::ptr::copy(src, dst, self.len() - index);
                    }

                    self.len += 1;

                    self.inner
                        .get_unchecked_mut(index)
                        .write(Entry::new(key, value));
                }
                None
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V>
    where
        K: Ord + Copy,
    {
        // TODO: fix undefined behavior here
        if let Ok(index) = self.search(key) {
            unsafe {
                let ret = std::ptr::read(self.inner.get_unchecked(index).as_ptr()).value;

                if index + 1 < self.len() {
                    let dst = self.inner.get_unchecked_mut(index).as_mut_ptr();
                    let src = self.inner.get_unchecked(index + 1).as_ptr();

                    std::ptr::copy(src, dst, self.len() - index);
                }

                self.len -= 1;

                Some(ret)
            }
        } else {
            None
        }
    }

    pub fn contains_key(&self, key: &K) -> bool
    where
        K: Ord + Copy,
    {
        self.search(key).is_ok()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Entry<K, V>> {
        self.inner[..self.len()]
            .iter()
            .map(|slot| unsafe { slot.assume_init_ref() })
    }

    /// Splits this `StackMap` into two. `self` will retain
    /// all key-value pairs before the provided split index.
    /// Returns a new `StackMap` created out of all key-value pairs
    /// at or after the provided split index.
    pub fn split_off(&mut self, split_idx: usize) -> Self {
        assert!(split_idx < self.len());
        assert!(split_idx < FANOUT);

        let mut rhs = Self::empty();

        for i in split_idx..self.len() {
            let src = self.inner[i].as_ptr();
            let dst = rhs.inner[i - split_idx].as_mut_ptr();
            unsafe {
                std::ptr::copy_nonoverlapping(src, dst, 1);
            }
        }

        rhs.len = self.len - split_idx;
        self.len = split_idx;

        rhs
    }

    pub fn split(&mut self) -> (K, Self)
    where
        K: Clone,
    {
        let split_idx = FANOUT / 2;

        (
            self.entries()[split_idx].key.clone(),
            self.split_off(split_idx),
        )
    }

    /// Get the key-value pair that is less than or equal
    /// to the provided key. Useful for any least upper
    /// bound operation, such as MVCC lookups where the
    /// key is suffixed by a version or an internal b-tree
    /// index lookup where you are looking for the next
    /// node based on a node's low key.
    pub fn get_less_than_or_equal(&self, key: &K) -> Option<&Entry<K, V>>
    where
        K: Ord + Copy,
    {
        // binary search LUB
        let index = match self.search(key) {
            Ok(i) => i,
            Err(0) => return None,
            Err(i) => i - 1,
        };

        self.get_index(index)
    }

    /// Gets a kv pair that has a key that is less than the provided key.
    pub fn get_less_than(&self, key: &K) -> Option<&Entry<K, V>>
    where
        K: Ord + Copy,
    {
        // binary search LUB
        let index = match self.search(key) {
            Ok(0) | Err(0) => return None,
            Ok(i) => i - 1,
            Err(i) => i - 1,
        };

        self.get_index(index)
    }

    pub fn get_always(&self, key: &K) -> &V
    where
        K: Ord + Copy,
    {
        // binary search LUB
        let index = match self.search(key) {
            Ok(i) => i,
            Err(0) => 0,
            Err(i) => i - 1,
        };

        &self.entries()[index].value
    }
}

#[allow(unused)]
impl<K, V, const FANOUT: usize> StackMap<K, V, FANOUT> {
    /// Borrow a slice view into the entries stored in the `StackMap`
    pub fn entries(&self) -> &[Entry<K, V>] {
        // SAFETY: `len` must be strictly less than `F`
        debug_assert!(self.len <= FANOUT);
        let slice = unsafe { self.inner.get_unchecked(..self.len) };

        // SAFETY: feature `maybe_uninit_slice`
        unsafe { &*(slice as *const [MaybeUninit<Entry<K, V>>] as *const [Entry<K, V>]) }
    }

    /// Get a key-value pair based on its internal relative
    /// index in the backing array.
    pub fn get_index(&self, index: usize) -> Option<&Entry<K, V>> {
        if index < self.len() {
            Some(unsafe { self.inner.get_unchecked(index).assume_init_ref() })
        } else {
            None
        }
    }

    /// Returns the first kv pair in the StackMap, if any exists
    pub fn first(&self) -> Option<&Entry<K, V>> {
        self.get_index(0)
    }

    /// Returns the last kv pair in the StackMap, if any exists
    pub fn last(&self) -> Option<&Entry<K, V>> {
        if self.is_empty() {
            None
        } else {
            self.get_index(self.len - 1)
        }
    }

    /// Returns `true` if this `StackMap` is at its maximum capacity and
    /// unable to receive additional data.
    pub const fn is_full(&self) -> bool {
        self.len == FANOUT
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use super::StackMap;
    use crate::common::entry::Entry;

    #[test]
    fn test_insert_and_get() {
        let mut stack_map: StackMap<u32, &str, 3> = StackMap::empty();
        assert!(stack_map.insert(1, "one").is_none());
        assert!(stack_map.insert(2, "two").is_none());
        assert!(stack_map.insert(3, "three").is_none());

        assert_eq!(stack_map.get(&1), Some(&"one"));
        assert_eq!(stack_map.get(&2), Some(&"two"));
        assert_eq!(stack_map.get(&3), Some(&"three"));
        assert_eq!(stack_map.get(&4), None);
    }

    #[test]
    fn test_remove() {
        let mut stack_map: StackMap<u32, &str, 3> = StackMap::empty();
        stack_map.insert(1, "one");
        stack_map.insert(2, "two");
        stack_map.insert(3, "three");

        assert_eq!(stack_map.remove(&2), Some("two"));
        assert_eq!(stack_map.remove(&2), None);
        assert_eq!(stack_map.get(&2), None);
    }

    #[test]
    fn test_contains_key() {
        let mut stack_map: StackMap<u32, &str, 3> = StackMap::empty();
        stack_map.insert(1, "one");
        stack_map.insert(2, "two");

        assert!(stack_map.contains_key(&1));
        assert!(stack_map.contains_key(&2));
        assert!(!stack_map.contains_key(&3));
    }

    #[test]
    fn test_iter() {
        let mut stack_map: StackMap<u32, &str, 3> = StackMap::empty();
        stack_map.insert(1, "one");
        stack_map.insert(2, "two");
        stack_map.insert(3, "three");

        let mut iter = stack_map.iter();
        assert_eq!(iter.next(), Some(&Entry::new(1, "one")));
        assert_eq!(iter.next_back(), Some(&Entry::new(3, "three")));
        assert_eq!(iter.next(), Some(&Entry::new(2, "two")));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_split_off() {
        let mut stack_map: StackMap<u32, &str, 4> = StackMap::empty();
        stack_map.insert(1, "one");
        stack_map.insert(2, "two");
        stack_map.insert(3, "three");
        stack_map.insert(4, "four");

        let split_map = stack_map.split_off(2);
        assert_eq!(stack_map.len(), 2);
        assert_eq!(split_map.len(), 2);
        assert_eq!(stack_map.get(&1), Some(&"one"));
        assert_eq!(stack_map.get(&2), Some(&"two"));
        assert_eq!(split_map.get(&3), Some(&"three"));
        assert_eq!(split_map.get(&4), Some(&"four"));
    }

    #[test]
    fn test_get_less_than() {
        let mut stack_map: StackMap<u32, &str, 4> = StackMap::empty();
        stack_map.insert(1, "one");
        stack_map.insert(3, "three");
        stack_map.insert(5, "five");
        stack_map.insert(7, "seven");

        assert_eq!(stack_map.get_less_than(&2), Some(&Entry::new(1, "one")));
        assert_eq!(stack_map.get_less_than(&4), Some(&Entry::new(3, "three")));
        assert_eq!(stack_map.get_less_than(&6), Some(&Entry::new(5, "five")));
        assert_eq!(stack_map.get_less_than(&8), Some(&Entry::new(7, "seven")));
        assert_eq!(stack_map.get_less_than(&0), None);
    }
}
