use crate::classical::node::BTreeNode;
use crate::common::linked_list::*;
use crate::common::macros::impl_node_layer;
use crate::component::*;
use crate::kv::StaticBounded;
use crate::kv::*;
use generational_arena::{Arena, Index};
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};
use std::ptr::NonNull;

// ----------------------------------------
// Layer Type
// ----------------------------------------

pub struct MemoryBTreeLayer<K, V, const FANOUT: usize, PA> {
    inner: LinkedList<BTreeNode<K, V, FANOUT>, PA>,
}

impl<K, V, const FANOUT: usize, PA> MemoryBTreeLayer<K, V, FANOUT, PA> {
    pub fn empty() -> Self {
        Self {
            inner: LinkedList::new(BTreeNode::empty()),
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

        for view in base.mut_range(Bound::Unbounded, Bound::Unbounded) {
            // If node too full, carry over to next
            if self.inner[ptr].is_half_full() {
                ptr = self.inner.insert_after(BTreeNode::empty(), ptr);
            }

            self.inner[ptr].insert(view.key(), view.address());
            view.set_parent(ptr);
        }
    }

    pub fn insert(&mut self, key: K, value: V, mut ptr: Index) -> Option<(K, Index, PA)>
    where
        K: Copy + Ord + StaticBounded,
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
        mut ptr: Index,
    ) -> Option<(K, Index, PA)>
    where
        V: Address,
        B: NodeLayer<K, V, Index>,
    {
        if self.deref_mut(ptr).inner.is_full() {
            let parent = self.deref(ptr).parent().unwrap();

            // Split
            let (split_point, new_node) = self.deref_mut(ptr).inner.split();

            let mut new_node_ptr = self.add_node(MemoryBTreeNode {
                inner: new_node,
                next: None,
                parent: None,
            });

            // Link to next node
            let old_next = self.deref_mut(ptr).next.replace(new_node_ptr);
            self.deref_mut(new_node_ptr).next = old_next;

            // Update all of the parents for the split node
            for entry in self.deref(new_node_ptr).inner.entries() {
                base.deref_mut(entry.value.clone()).set_parent(new_node_ptr);
            }

            // Insert into the right node
            if key < split_point {
                self.deref_mut(ptr).inner.insert(key, value.clone());
                base.deref_mut(value).set_parent(ptr);
            } else {
                self.deref_mut(new_node_ptr)
                    .inner
                    .insert(key, value.clone());
                base.deref_mut(value).set_parent(new_node_ptr);
            }

            return Some((
                *self.deref(new_node_ptr).inner.borrow(),
                new_node_ptr,
                parent,
            ));
        } else {
            self.deref_mut(ptr).inner.insert(key, value.clone());
            base.deref_mut(value).set_parent(ptr);
        }

        None
    }
}

impl<K, V, const FANOUT: usize, PA> NodeLayer<K, Index, PA> for MemoryBTreeLayer<K, V, FANOUT, PA>
where
    K: Copy + StaticBounded + 'static,
    V: 'static,
    PA: Address,
{
    type Node = <LinkedList<BTreeNode<K, V, FANOUT>, PA> as NodeLayer<K, Index, PA>>::Node;

    impl_node_layer!(Index);
}

//
//     pub fn insert(&mut self, key: K, value: V, mut ptr: Index) -> Option<(K, Index, PA)> {
//         if self.deref_mut(ptr).inner.is_full() {
//             let parent = self.deref(ptr).parent().unwrap();
//
//             // Split
//             let (split_point, new_node) = self.deref_mut(ptr).inner.split();
//
//             let mut new_node_ptr = self.add_node(MemoryBTreeNode {
//                 inner: new_node,
//                 next: None,
//                 parent: None,
//             });
//
//             // Link to next node
//             let old_next = self.deref_mut(ptr).next.replace(new_node_ptr);
//             self.deref_mut(new_node_ptr).next = old_next;
//
//             // Insert into the right node
//             if key < split_point {
//                 self.deref_mut(ptr).inner.insert(key, value);
//             } else {
//                 self.deref_mut(new_node_ptr).inner.insert(key, value);
//             }
//
//             return Some((
//                 *self.deref(new_node_ptr).inner.borrow(),
//                 new_node_ptr,
//                 parent,
//             ));
//         } else {
//             self.deref_mut(ptr).inner.insert(key, value);
//         }
//
//         None
//     }
// }
