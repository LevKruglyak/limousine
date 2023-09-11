use crate::classical::node::BTreeNode;
use crate::common::entry::Entry;
use crate::component::*;
use crate::kv::StaticBounded;
use crate::kv::*;
use bumpalo::boxed::Box;
use bumpalo::Bump;
use std::borrow::Borrow;
use std::fmt::Debug;
// use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};
use std::ptr::NonNull;

// ----------------------------------------
// Helper Types
// ----------------------------------------

type Node<K, V, const FANOUT: usize> = MemoryBTreeNode<K, V, FANOUT>;
type Address<K, V, const FANOUT: usize> = NonNull<Node<K, V, FANOUT>>;
type OptAddress<K, V, const FANOUT: usize> = Option<Address<K, V, FANOUT>>;

// ----------------------------------------
// Node Type
// ----------------------------------------

pub struct MemoryBTreeNode<K, V, const FANOUT: usize> {
    pub inner: BTreeNode<K, V, FANOUT>,
    pub next: OptAddress<K, V, FANOUT>,
}

impl<K, V, const FANOUT: usize> MemoryBTreeNode<K, V, FANOUT> {
    pub fn empty() -> Self {
        Self {
            inner: BTreeNode::empty(),
            next: None,
        }
    }
}

impl<K: StaticBounded, V, const FANOUT: usize> KeyBounded<K> for MemoryBTreeNode<K, V, FANOUT> {
    fn lower_bound(&self) -> &K {
        self.inner.borrow()
    }
}

// ----------------------------------------
// Layer Type
// ----------------------------------------

pub struct MemoryBTreeLayer<K, V, const FANOUT: usize> {
    pub nodes: Vec<Address<K, V, FANOUT>>,
    pub alloc: Bump,
}

impl<K: Debug, V: Debug, const FANOUT: usize> Debug for MemoryBTreeLayer<K, V, FANOUT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.nodes.iter().map(|node| {
                (unsafe { &node.as_ref().inner }, unsafe {
                    node.as_ref().next
                })
            }))
            .finish()
    }
}

impl<K, V: Clone, const FANOUT: usize> NodeLayer<K> for MemoryBTreeLayer<K, V, FANOUT>
where
    K: 'static + StaticBounded,
    V: 'static,
{
    type Node = MemoryBTreeNode<K, V, FANOUT>;
    type Address = NonNull<Self::Node>;
    type Iter<'n> = Iter<'n, K, V, FANOUT>;

    fn deref(&self, ptr: Self::Address) -> &Self::Node {
        // TODO: safety!
        unsafe { ptr.as_ref() }
    }

    fn deref_mut(&mut self, mut ptr: Self::Address) -> &mut Self::Node {
        // TODO: safety!
        unsafe { ptr.as_mut() }
    }

    fn range<'n>(
        &'n self,
        start: Bound<Self::Address>,
        end: Bound<Self::Address>,
    ) -> Self::Iter<'n> {
        Iter::range(&self, start, end)
    }

    fn full_range<'n>(&'n self) -> Self::Iter<'n> {
        Iter::new(&self)
    }
}

impl<K, V, const FANOUT: usize> Drop for MemoryBTreeLayer<K, V, FANOUT> {
    fn drop(&mut self) {
        // Drop all of the nodes
        for node in self.nodes.iter() {
            let boxed = unsafe { Box::from_raw(node.as_ptr()) };
            drop(boxed);
        }
    }
}

impl<K, V, const FANOUT: usize> MemoryBTreeLayer<K, V, FANOUT>
where
    K: StaticBounded,
{
    pub fn add_node(&mut self, node: Node<K, V, FANOUT>) -> Address<K, V, FANOUT> {
        let ptr = NonNull::new(Box::into_raw(Box::new_in(node, &self.alloc))).unwrap();
        self.nodes.push(ptr);
        ptr
    }

    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            alloc: Bump::new(),
        }
    }

    pub fn fill(&mut self, iter: impl Iterator<Item = Entry<K, V>>) {
        // Drop all of the nodes
        for node in self.nodes.iter() {
            let boxed = unsafe { Box::from_raw(node.as_ptr()) };
            drop(boxed);
        }

        self.nodes.clear();
        self.alloc.reset();

        // Add empty cap node
        let mut node = unsafe { self.add_node(MemoryBTreeNode::empty()).as_mut() };
        let mut new_address = self.add_node(MemoryBTreeNode::empty());
        node.next = Some(new_address);
        node = unsafe { new_address.as_mut() };

        for entry in iter {
            if node.inner.is_half_full() {
                let mut new_address = self.add_node(MemoryBTreeNode::empty());
                node.next = Some(new_address);

                node = unsafe { new_address.as_mut() };
            }

            node.inner.insert(entry.key, entry.value);
        }
    }

    pub fn insert(
        &mut self,
        key: K,
        value: V,
        mut ptr: Address<K, V, FANOUT>,
    ) -> Option<(K, Address<K, V, FANOUT>)> {
        let node = unsafe { ptr.as_mut() };

        if node.inner.is_full() {
            // Split
            let (split_point, new_node) = node.inner.split();

            let mut new_node_ptr = self.add_node(MemoryBTreeNode {
                inner: new_node,
                next: None,
            });
            let new_node = unsafe { new_node_ptr.as_mut() };

            // Link to next node
            let old_next = node.next.replace(new_node_ptr);
            new_node.next = old_next;

            // Insert into the right node
            if key < split_point {
                node.inner.insert(key, value);
            } else {
                new_node.inner.insert(key, value);
            }

            return Some((*new_node.inner.borrow(), new_node_ptr));
        } else {
            node.inner.insert(key, value);
        }

        None
    }
}

// ----------------------------------------
// Iterator Type
// ----------------------------------------

#[derive(Clone)]
pub struct Iter<'n, K, V, const FANOUT: usize> {
    layer: &'n MemoryBTreeLayer<K, V, FANOUT>,
    current: OptAddress<K, V, FANOUT>,
    end: Bound<Address<K, V, FANOUT>>,
}

impl<'n, K, V, const FANOUT: usize> Iter<'n, K, V, FANOUT> {
    fn new(layer: &'n MemoryBTreeLayer<K, V, FANOUT>) -> Self {
        Self {
            layer,
            current: Some(layer.nodes[0]),
            end: Bound::Unbounded,
        }
    }

    fn range(
        layer: &'n MemoryBTreeLayer<K, V, FANOUT>,
        start: Bound<Address<K, V, FANOUT>>,
        end: Bound<Address<K, V, FANOUT>>,
    ) -> Self {
        match start {
            Bound::Excluded(start) => Self {
                layer,
                current: unsafe { start.as_ref().next },
                end,
            },

            Bound::Included(start) => Self {
                layer,
                current: Some(start.clone()),
                end,
            },

            Bound::Unbounded => Self {
                layer,
                current: Some(layer.nodes[0]),
                end,
            },
        }
    }
}

impl<'n, K, V: Clone, const FANOUT: usize> Iterator for Iter<'n, K, V, FANOUT>
where
    K: StaticBounded,
    V: 'static,
{
    type Item = Entry<K, Address<K, V, FANOUT>>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.clone()?;

        match self.end {
            Bound::Excluded(end) => {
                if current == end {
                    return None;
                }
            }

            Bound::Included(end) => {
                if current == end {
                    self.current = None;
                }
            }

            _ => (),
        }

        // Advance pointer
        if let Some(current) = self.current {
            self.current = self.layer.deref(current).next;
        }

        return Some(Entry::new(*self.layer.lower_bound(current), current));
    }
}
