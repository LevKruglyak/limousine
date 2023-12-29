use crate::common::bounded::KeyBounded;
use crate::common::bounded::StaticBounded;
use crate::common::entry::Entry;
use crate::common::stack_map::StackMap;
use std::fmt::Debug;

#[derive(Clone, Default)]
pub struct BTreeNode<K, V, const FANOUT: usize> {
    inner: StackMap<K, V, FANOUT>,
}

impl<K: Debug, V: Debug, const FANOUT: usize> Debug for BTreeNode<K, V, FANOUT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.inner))
    }
}

impl<K, V, const FANOUT: usize> BTreeNode<K, V, FANOUT> {
    pub fn empty() -> Self {
        Self {
            inner: StackMap::empty(),
        }
    }

    pub fn entries(&self) -> &[Entry<K, V>] {
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

    pub fn search_lub(&self, key: &K) -> &V
    where
        K: Ord + Copy,
    {
        self.inner.get_always(key)
    }

    pub fn search_exact(&self, key: &K) -> Option<&V>
    where
        K: Ord + Copy,
    {
        self.inner.get(key)
    }

    /// Inserts an item and return the previous value if it exists.
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord + Copy,
    {
        self.inner.insert(key, value)
    }

    pub fn split(&mut self) -> (K, Self)
    where
        K: Clone,
    {
        let (key, map) = self.inner.split();
        (key, Self { inner: map })
    }
}

impl<K: Copy + StaticBounded, V, const FANOUT: usize> KeyBounded<K> for BTreeNode<K, V, FANOUT> {
    fn lower_bound(&self) -> &K {
        self.min()
    }
}
