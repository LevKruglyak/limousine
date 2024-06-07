use crate::classical::node::BTreeNode;
use crate::common::list::memory::*;
use crate::node_layer::{impl_node_layer, NodeLayer};
use crate::traits::{Address, KeyBounded};
use crate::Key;
use std::ops::Bound;

// ----------------------------------------
// Layer Type
// ----------------------------------------

pub struct MemoryBTreeLayer<K: Ord, V, const FANOUT: usize, PA> {
    inner: MemoryList<BTreeNode<K, V, FANOUT>, PA>,
}

impl<K, V, const FANOUT: usize, PA> MemoryBTreeLayer<K, V, FANOUT, PA>
where
    K: Key,
{
    pub fn empty() -> Self {
        Self {
            inner: MemoryList::empty(),
        }
    }

    pub fn fill(&mut self, iter: impl Iterator<Item = (K, V)>) {
        // Add empty cap node
        let mut ptr = self.inner.clear();

        for (key, address) in iter {
            // If node too full, carry over to next
            if self.inner[ptr].is_half_full() {
                ptr = self.inner.insert_after(BTreeNode::empty(), ptr);
            }

            self.inner[ptr].insert(key, address);
        }
    }

    pub fn fill_with_parent<B: NodeLayer<K, V, ArenaID>>(&mut self, base: &mut B)
    where
        V: Address,
    {
        // Add empty cap node
        let mut ptr = self.inner.clear();
        let mut iter = base.range_mut(Bound::Unbounded, Bound::Unbounded);

        while let Some((key, address, parent)) = iter.next() {
            // If node too full, carry over to next
            if self.inner[ptr].is_half_full() {
                ptr = self.inner.insert_after(BTreeNode::empty(), ptr);
            }

            self.inner[ptr].insert(key.clone(), address.clone());
            parent.set(ptr);
        }
    }

    pub fn insert(&mut self, key: K, value: V, ptr: ArenaID) -> Option<(K, ArenaID, PA)>
    where
        PA: Address,
    {
        if self.inner[ptr].is_full() {
            let parent = self.inner.parent(ptr).unwrap();

            // Split
            let (split_point, new_node) = self.inner[ptr].split();
            let new_node_ptr = self.inner.insert_after(new_node, ptr);

            // Insert into the right node
            if key < split_point {
                self.inner[ptr].insert(key, value);
            } else {
                self.inner[new_node_ptr].insert(key, value);
            }

            return Some((
                self.inner[new_node_ptr].lower_bound().clone(),
                new_node_ptr,
                parent,
            ));
        } else {
            self.inner[ptr].insert(key, value);
        }

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
        if self.inner[ptr].is_full() {
            let parent = self.inner.parent(ptr).unwrap();

            // Split
            let (split_point, new_node) = self.inner[ptr].split();
            let new_node_ptr = self.inner.insert_after(new_node, ptr);

            // Update all of the parents for the split node
            for entry in self.inner[new_node_ptr].entries() {
                base.set_parent(entry.value.clone(), new_node_ptr)
            }

            // Insert into the right node
            if key < split_point {
                self.inner[ptr].insert(key, value.clone());
                base.set_parent(value, ptr);
            } else {
                self.inner[new_node_ptr].insert(key, value.clone());
                base.set_parent(value, new_node_ptr);
            }

            return Some((
                self.inner[new_node_ptr].lower_bound().clone(),
                new_node_ptr,
                parent,
            ));
        } else {
            self.inner[ptr].insert(key, value.clone());
            base.set_parent(value, ptr);
        }

        None
    }
}

impl<K: Ord, V, const FANOUT: usize, PA> core::ops::Index<ArenaID>
    for MemoryBTreeLayer<K, V, FANOUT, PA>
{
    type Output = BTreeNode<K, V, FANOUT>;

    fn index(&self, index: ArenaID) -> &Self::Output {
        &self.inner[index]
    }
}

impl<K, V, const FANOUT: usize, PA> NodeLayer<K, ArenaID, PA> for MemoryBTreeLayer<K, V, FANOUT, PA>
where
    K: Key,
    PA: Address,
{
    impl_node_layer!(ArenaID, PA);
}
