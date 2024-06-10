// ----------------------------------------
// Layer Type
// ----------------------------------------

use std::ops::Bound;

use learned_index_segmentation::linear_simple_segmentation;

use crate::common::list::memory::*;
use crate::iter::Iter;
use crate::learned::node::PGMNode;
use crate::{impl_node_layer, Address, Key, NodeLayer};

pub struct MemoryPGMLayer<K: Key, V: Clone, const EPSILON: usize, PA> {
    inner: MemoryList<PGMNode<K, V, EPSILON>, PA>,
}

struct FillerIter<'a, K, B, SA, PA>
where
    SA: Address,
    PA: Address,
{
    iter: Iter<'a, K, B, SA, PA>,
}
impl<'a, K, B, SA, PA> Iterator for FillerIter<'a, K, B, SA, PA>
where
    K: Clone,
    B: NodeLayer<K, SA, PA>,
    SA: Address,
    PA: Address,
{
    type Item = (K, SA);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some((key, address)) => Some((key, address)),
            None => None,
        }
    }
}

impl<K, V, const EPSILON: usize, PA> MemoryPGMLayer<K, V, EPSILON, PA>
where
    K: Key,
    V: Clone,
{
    pub fn empty() -> Self {
        Self {
            inner: MemoryList::empty(),
        }
    }

    pub fn fill(&mut self, iter: impl Iterator<Item = (K, V)>) {
        let trained = linear_simple_segmentation::<_, _, EPSILON>(iter);

        let mut ptr = self.inner.clear();

        for (model, entries) in trained.into_iter().rev() {
            let node = PGMNode::from_trained(model, entries);
            ptr = self.inner.insert_before(node, ptr);
        }
    }

    pub fn fill_will_parent<B: NodeLayer<K, V, ArenaID>>(&mut self, base: &mut B)
    where
        V: Address,
    {
        let iter = base.range(Bound::Unbounded, Bound::Unbounded);
        let iter = FillerIter { iter };

        let trained = linear_simple_segmentation::<_, _, EPSILON>(iter);

        let mut ptr = self.inner.clear();

        for (model, entries) in trained.into_iter().rev() {
            let node = PGMNode::from_trained(model, entries.clone());
            ptr = self.inner.insert_before(node, ptr);
            for (_, value) in entries.iter() {
                base.set_parent(value.clone(), ptr);
            }
        }
    }

    pub fn insert(&mut self, key: K, value: V, ptr: ArenaID) -> Option<(K, ArenaID, PA)>
    where
        PA: Address,
    {
        self.inner[ptr].grow_insert((key, value));
        None
    }

    pub fn insert_with_parent<B: NodeLayer<K, V, ArenaID>>(
        &mut self,
        key: K,
        value: V,
        base: &mut B,
        ptr: ArenaID,
    ) -> Option<(K, ArenaID, PA)>
    where
        V: Address,
        PA: Address,
    {
        self.inner[ptr].grow_insert((key, value.clone()));
        base.set_parent(value, ptr);
        None
    }
}

impl<K: Key, V: Clone, const EPSILON: usize, PA> core::ops::Index<ArenaID>
    for MemoryPGMLayer<K, V, EPSILON, PA>
{
    type Output = PGMNode<K, V, EPSILON>;

    fn index(&self, index: ArenaID) -> &Self::Output {
        &self.inner[index]
    }
}

impl<K, V, const EPSILON: usize, PA> NodeLayer<K, ArenaID, PA> for MemoryPGMLayer<K, V, EPSILON, PA>
where
    K: Key,
    V: Clone,
    PA: Address,
{
    impl_node_layer!(ArenaID, PA);
}
