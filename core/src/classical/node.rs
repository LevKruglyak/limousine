use crate::traits::KeyBounded;
use crate::traits::StaticBounded;
use serde::{Deserialize, Serialize};

use sorted_array::SortedArray;
use sorted_array::SortedArrayEntry;

use std::fmt::Debug;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BTreeNode<K, V, const FANOUT: usize> {
    // Serde derive has some trouble introducing the right bounds here
    #[serde(bound(
        deserialize = "K: Serialize + Deserialize<'de> + Ord + Copy, V: Serialize + Deserialize<'de>"
    ))]
    inner: SortedArray<K, V, FANOUT>,
}

impl<K: Debug, V: Debug, const FANOUT: usize> Debug for BTreeNode<K, V, FANOUT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.inner))
    }
}

impl<K, V, const FANOUT: usize> Default for BTreeNode<K, V, FANOUT> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<K, V, const FANOUT: usize> BTreeNode<K, V, FANOUT> {
    pub fn empty() -> Self {
        Self {
            inner: SortedArray::empty(),
        }
    }

    pub fn entries(&self) -> &[SortedArrayEntry<K, V>] {
        self.inner.entries()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    pub fn is_half_full(&self) -> bool {
        self.inner.len() >= FANOUT / 2
    }

    pub fn min(&self) -> &K
    where
        K: StaticBounded,
    {
        if self.is_empty() {
            K::min_ref()
        } else {
            &self.inner.entries()[0].key
        }
    }

    pub fn get_lower_bound_always(&self, key: &K) -> &V
    where
        K: Ord + Copy,
    {
        self.inner.get_lower_bound_always(key)
    }

    pub fn get_exact(&self, key: &K) -> Option<&V>
    where
        K: Ord + Copy,
    {
        self.inner.get_exact(key)
    }

    /// Inserts an item and return the previous value if it exists.
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord + Copy,
    {
        self.inner.insert(key, value)
    }

    // TODO: should allocation go here?
    pub fn split(&mut self) -> (K, Self)
    where
        K: Clone,
    {
        let split_idx = FANOUT / 2;

        let key = self.inner.entries()[split_idx].key.clone();
        let map = self.inner.split_off(split_idx);

        (key, Self { inner: map })
    }
}

impl<K: Copy + StaticBounded, V, const FANOUT: usize> KeyBounded<K> for BTreeNode<K, V, FANOUT> {
    fn lower_bound(&self) -> &K {
        self.min()
    }
}
