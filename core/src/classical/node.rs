use crate::traits::KeyBounded;
use crate::traits::StaticBounded;
use serde::{Deserialize, Serialize};
use sorted_array::SortedArray;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct BTreeNode<K: Ord, V, const FANOUT: usize> {
    // Serde derive has some trouble introducing the right bounds here
    #[serde(bound(
        deserialize = "K: Serialize + Deserialize<'de> + Ord, V: Serialize + Deserialize<'de>"
    ))]
    inner: SortedArray<K, V, FANOUT>,
}

impl<K: Ord, V, const FANOUT: usize> Deref for BTreeNode<K, V, FANOUT> {
    type Target = SortedArray<K, V, FANOUT>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K: Ord, V, const FANOUT: usize> DerefMut for BTreeNode<K, V, FANOUT> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<K: Ord, V, const FANOUT: usize> BTreeNode<K, V, FANOUT> {
    pub fn empty() -> Self {
        Self {
            inner: SortedArray::empty(),
        }
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

impl<K: Ord, V, const FANOUT: usize> Default for BTreeNode<K, V, FANOUT> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<K: StaticBounded, V, const FANOUT: usize> KeyBounded<K> for BTreeNode<K, V, FANOUT> {
    fn lower_bound(&self) -> &K {
        self.min()
    }
}
