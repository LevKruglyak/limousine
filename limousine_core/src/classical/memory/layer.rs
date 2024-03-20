use crate::classical::node::BTreeNode;
use crate::common::list::memory::*;
use crate::node_layer::{impl_node_layer, NodeLayer};
use crate::traits::{Address, KeyBounded, StaticBounded};
use generational_arena::Index;
use std::ops::Bound;

// ----------------------------------------
// Layer Type
// ----------------------------------------

pub struct MemoryBTreeLayer<K, V, const FANOUT: usize, PA> {
    inner: MemoryList<BTreeNode<K, V, FANOUT>, PA>,
}

impl<K, V, const FANOUT: usize, PA> MemoryBTreeLayer<K, V, FANOUT, PA> {
    pub fn empty() -> Self {
        Self {
            inner: MemoryList::new(),
        }
    }

    pub fn fill(&mut self, iter: impl Iterator<Item = (K, V)>)
    where
        K: Copy + Ord,
    {
        // Add empty cap node
        let mut ptr = self.inner.clear(BTreeNode::empty());

        for (key, address) in iter {
            // If node too full, carry over to next
            if self.inner[ptr].is_half_full() {
                ptr = self.inner.insert_after(BTreeNode::empty(), ptr);
            }

            self.inner[ptr].insert(key, address);
        }
    }

    pub fn fill_with_parent<B>(&mut self, base: &mut B)
    where
        K: Copy + Ord,
        V: Address,
        B: NodeLayer<K, V, Index>,
    {
        // Add empty cap node
        let mut ptr = self.inner.clear(BTreeNode::empty());
        let mut iter = base.range_mut(Bound::Unbounded, Bound::Unbounded);

        while let Some((key, address, parent)) = iter.next() {
            // If node too full, carry over to next
            if self.inner[ptr].is_half_full() {
                ptr = self.inner.insert_after(BTreeNode::empty(), ptr);
            }

            self.inner[ptr].insert(key, address.clone());
            parent.set(ptr);
        }
    }

    pub fn insert(&mut self, key: K, value: V, ptr: Index) -> Option<(K, Index, PA)>
    where
        K: Copy + Ord + StaticBounded,
        V: 'static,
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
                *self.inner[new_node_ptr].lower_bound(),
                new_node_ptr,
                parent,
            ));
        } else {
            self.inner[ptr].insert(key, value);
        }

        None
    }

    pub fn insert_with_parent<B>(
        &mut self,
        key: K,
        value: V,
        base: &mut B,
        ptr: Index,
    ) -> Option<(K, Index, PA)>
    where
        K: Copy + Ord + StaticBounded,
        V: Address,
        PA: Address,
        B: NodeLayer<K, V, Index>,
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
                *self.inner[new_node_ptr].lower_bound(),
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

impl<K, V, const FANOUT: usize, PA> core::ops::Index<Index> for MemoryBTreeLayer<K, V, FANOUT, PA> {
    type Output = BTreeNode<K, V, FANOUT>;

    fn index(&self, index: Index) -> &Self::Output {
        &self.inner[index]
    }
}

impl<K, V, const FANOUT: usize, PA> NodeLayer<K, Index, PA> for MemoryBTreeLayer<K, V, FANOUT, PA>
where
    K: Copy + StaticBounded + 'static,
    V: 'static,
    PA: Address,
{
    impl_node_layer!(Index, PA);
}
