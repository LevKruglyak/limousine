use crate::classical::node::BTreeNode;
use crate::common::entry::Entry;
use crate::component::*;
use crate::kv::StaticBounded;
use crate::kv::*;
use generational_arena::{Arena, Index};
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};
use std::ptr::NonNull;

// ----------------------------------------
// Helper Types
// ----------------------------------------

type Node<K, V, const FANOUT: usize, PA> = MemoryBTreeNode<K, V, FANOUT, PA>;

// ----------------------------------------
// Node Type
// ----------------------------------------

pub struct MemoryBTreeNode<K, V, const FANOUT: usize, PA> {
    pub inner: BTreeNode<K, V, FANOUT>,
    pub next: Option<Index>,
    pub parent: Option<PA>,
}

impl<K, V, const FANOUT: usize, PA> MemoryBTreeNode<K, V, FANOUT, PA> {
    pub fn empty() -> Self {
        Self {
            inner: BTreeNode::empty(),
            next: None,
            parent: None,
        }
    }
}

impl<K: StaticBounded, V, const FANOUT: usize, PA> KeyBounded<K>
    for MemoryBTreeNode<K, V, FANOUT, PA>
{
    fn lower_bound(&self) -> &K {
        self.inner.borrow()
    }
}

impl<K: StaticBounded, V: 'static, const FANOUT: usize, PA: 'static> LinkedNode<K, Index, PA>
    for MemoryBTreeNode<K, V, FANOUT, PA>
where
    PA: Address,
{
    fn next(&self) -> Option<Index> {
        self.next
    }

    fn parent(&self) -> Option<PA> {
        self.parent.clone()
    }

    fn set_parent(&mut self, parent: PA) {
        self.parent = Some(parent);
    }
}

// ----------------------------------------
// Layer Type
// ----------------------------------------

pub struct MemoryBTreeLayer<K, V, const FANOUT: usize, PA> {
    pub arena: Arena<Node<K, V, FANOUT, PA>>,
    pub first: Index,
}

// impl<K: Debug, V: Debug, const FANOUT: usize> Debug for MemoryBTreeLayer<K, V, FANOUT> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_list()
//             .entries(self.nodes.iter().map(|node| {
//                 (unsafe { &node.as_ref().inner }, unsafe {
//                     node.as_ref().next
//                 })
//             }))
//             .finish()
//     }
// }

impl<K, V, const FANOUT: usize, PA> NodeLayer<K, Index, PA> for MemoryBTreeLayer<K, V, FANOUT, PA>
where
    K: 'static + StaticBounded,
    V: 'static,
    PA: Address,
{
    type Node = MemoryBTreeNode<K, V, FANOUT, PA>;

    fn deref(&self, ptr: Index) -> &Self::Node {
        self.arena.get(ptr).unwrap()
    }

    fn deref_mut(&mut self, ptr: Index) -> &mut Self::Node {
        self.arena.get_mut(ptr).unwrap()
    }

    unsafe fn deref_unsafe(&self, ptr: Index) -> *mut Self::Node {
        self.arena.get(ptr).unwrap() as *const Self::Node as *mut Self::Node
    }

    fn first(&self) -> Index {
        self.first
    }
}

impl<K, V, const FANOUT: usize, PA> MemoryBTreeLayer<K, V, FANOUT, PA>
where
    K: StaticBounded,
    V: 'static,
    PA: Address,
{
    pub fn add_node(&mut self, node: Node<K, V, FANOUT, PA>) -> Index {
        self.arena.insert(node)
    }

    pub fn empty() -> Self {
        let mut arena = Arena::new();
        let first = arena.insert(MemoryBTreeNode::empty());

        Self { arena, first }
    }

    pub fn fill(&mut self, iter: impl Iterator<Item = (K, V)>) {
        self.arena.clear();

        // Add empty cap node
        let mut ptr = self.add_node(MemoryBTreeNode::empty());
        self.first = ptr;

        for (key, address) in iter {
            // If node too full, carry over to next
            if self.deref(ptr).inner.is_half_full() {
                let mut new_address = self.add_node(MemoryBTreeNode::empty());
                self.deref_mut(ptr).next = Some(new_address);

                ptr = new_address;
            }

            self.deref_mut(ptr).inner.insert(key, address);
        }
    }

    pub fn fill_with_parent<B>(&mut self, base: &mut B)
    where
        V: Address,
        B: NodeLayer<K, V, Index>,
    {
        self.arena.clear();

        // Add empty cap node
        let mut ptr = self.add_node(MemoryBTreeNode::empty());
        self.first = ptr;

        for view in base.mut_range(Bound::Unbounded, Bound::Unbounded) {
            // If node too full, carry over to next
            if self.deref(ptr).inner.is_half_full() {
                let mut new_address = self.add_node(MemoryBTreeNode::empty());
                self.deref_mut(ptr).next = Some(new_address);

                ptr = new_address;
            }

            self.deref_mut(ptr).inner.insert(view.key(), view.address());
            view.set_parent(ptr);
        }
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

    pub fn insert(&mut self, key: K, value: V, mut ptr: Index) -> Option<(K, Index, PA)> {
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

            // Insert into the right node
            if key < split_point {
                self.deref_mut(ptr).inner.insert(key, value);
            } else {
                self.deref_mut(new_node_ptr).inner.insert(key, value);
            }

            return Some((
                *self.deref(new_node_ptr).inner.borrow(),
                new_node_ptr,
                parent,
            ));
        } else {
            self.deref_mut(ptr).inner.insert(key, value);
        }

        None
    }
}
