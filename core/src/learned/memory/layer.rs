// ----------------------------------------
// Layer Type
// ----------------------------------------

use learned_index_segmentation::linear_simple_segmentation;
use num::PrimInt;

use crate::common::list::memory::*;
use crate::learned::node::PGMNode;
use crate::{impl_node_layer, Address, Key, NodeLayer};

pub struct MemoryPGMLayer<K: Ord + Copy, V: Copy, const EPSILON: usize, PA> {
    inner: MemoryList<PGMNode<K, V, EPSILON>, PA>,
}

impl<K, V, const EPSILON: usize, PA> MemoryPGMLayer<K, V, EPSILON, PA>
where
    K: Ord + Copy + Key + PrimInt,
    V: Copy,
{
    pub fn empty() -> Self {
        Self {
            inner: MemoryList::empty(),
        }
    }

    pub fn fill(&mut self, iter: impl Iterator<Item = (K, V)>) {
        let trained = linear_simple_segmentation::<K, V, EPSILON>(iter);

        let mut ptr = self.inner.clear();

        for (model, entries) in trained.into_iter().rev() {
            let node = PGMNode::from_trained(model, entries);
            ptr = self.inner.insert_before(node, ptr);
        }
    }
}

impl<K: Ord + Copy, V: Copy, const EPSILON: usize, PA> core::ops::Index<ArenaID>
    for MemoryPGMLayer<K, V, EPSILON, PA>
{
    type Output = PGMNode<K, V, EPSILON>;

    fn index(&self, index: ArenaID) -> &Self::Output {
        &self.inner[index]
    }
}

impl<K, V, const EPSILON: usize, PA> NodeLayer<K, ArenaID, PA> for MemoryPGMLayer<K, V, EPSILON, PA>
where
    K: Ord + Copy + Key + PrimInt,
    V: Copy,
    PA: Address,
{
    impl_node_layer!(ArenaID, PA);
}
