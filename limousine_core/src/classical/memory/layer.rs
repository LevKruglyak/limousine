use crate::classical::node::BTreeNode;
use crate::common::address::{Address, Arena};
use crate::component::*;
use crate::kv::StaticBounded;
use crate::kv::*;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};
use std::ptr::NonNull;

// ----------------------------------------
// Helper Types
// ----------------------------------------

type Node<K, V, const FANOUT: usize> = MemoryBTreeNode<K, V, FANOUT>;

// ----------------------------------------
// Node Type
// ----------------------------------------

pub struct MemoryBTreeNode<K, V, const FANOUT: usize> {
    pub inner: BTreeNode<K, V, FANOUT>,
    pub next: Option<Address>,
    pub parent: Option<Address>,
}

impl<K, V, const FANOUT: usize> MemoryBTreeNode<K, V, FANOUT> {
    pub fn empty() -> Self {
        Self {
            inner: BTreeNode::empty(),
            next: None,
            parent: None,
        }
    }
}

impl<K: StaticBounded, V, const FANOUT: usize> KeyBounded<K> for MemoryBTreeNode<K, V, FANOUT> {
    fn lower_bound(&self) -> &K {
        self.inner.borrow()
    }
}

impl<K: StaticBounded, V: 'static, const FANOUT: usize> LinkedNode<K>
    for MemoryBTreeNode<K, V, FANOUT>
{
    fn next(&self) -> Option<Address> {
        self.next
    }

    fn parent(&self) -> Option<Address> {
        self.parent
    }
}

// ----------------------------------------
// Layer Type
// ----------------------------------------

pub struct MemoryBTreeLayer<K, V, const FANOUT: usize> {
    pub arena: Arena<Node<K, V, FANOUT>>,
    pub first: Address,
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

impl<K, V, const FANOUT: usize> NodeLayer<K> for MemoryBTreeLayer<K, V, FANOUT>
where
    K: 'static + StaticBounded,
    V: 'static,
{
    type Node = MemoryBTreeNode<K, V, FANOUT>;

    fn deref(&self, ptr: Address) -> &Self::Node {
        self.arena.deref(ptr)
    }

    fn deref_mut(&mut self, ptr: Address) -> &mut Self::Node {
        self.arena.deref_mut(ptr)
    }

    fn first(&self) -> Address {
        self.first
    }
}

impl<K, V, const FANOUT: usize> MemoryBTreeLayer<K, V, FANOUT>
where
    K: StaticBounded,
    V: 'static,
{
    pub fn add_node(&mut self, node: Node<K, V, FANOUT>) -> Address {
        self.arena.add(node)
    }

    pub fn empty() -> Self {
        let mut arena = Arena::new();
        let first = arena.add(MemoryBTreeNode::empty());

        Self { arena, first }
    }

    pub fn fill(&mut self, iter: impl Iterator<Item = (K, V)>) {
        self.arena.reset();

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

    pub fn insert(&mut self, key: K, value: V, mut ptr: Address) -> Option<(K, Address)> {
        if self.deref_mut(ptr).inner.is_full() {
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

            return Some((*self.deref(new_node_ptr).inner.borrow(), new_node_ptr));
        } else {
            self.deref_mut(ptr).inner.insert(key, value);
        }

        None
    }
}
