use crate::classical::node::BTreeNode;
use crate::BaseComponent;
use crate::{InternalComponent, Key, NodeLayer, Value};
use std::borrow::Borrow;
use std::cell::Ref;
use std::cell::RefCell;
use std::fmt::Debug;
use std::iter::Map;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;
use std::slice::Iter;

pub struct DiskBTreeLayer<K, V, const FANOUT: usize> {
    pub nodes: Vec<BTreeNode<K, V, FANOUT>>,
}

pub struct DiskBTreeLayerNodeIterator {
    len: usize,
    idx: usize,
}

impl DiskBTreeLayerNodeIterator {
    fn new(len: usize) -> Self {
        Self { len, idx: 0 }
    }
}

impl Iterator for DiskBTreeLayerNodeIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.len {
            self.idx += 1;
            Some(self.idx - 1)
        } else {
            None
        }
    }
}

impl<K: Key, V: 'static, const FANOUT: usize> NodeLayer<K> for DiskBTreeLayer<K, V, FANOUT> {
    type Node = BTreeNode<K, V, FANOUT>;
    type NodeRef = usize;

    fn node_ref(&self, ptr: Self::NodeRef) -> &Self::Node {
        &self.nodes[ptr]
    }

    type NodeIter<'n> = DiskBTreeLayerNodeIterator;

    fn iter<'n>(&'n self) -> Self::NodeIter<'n> {
        Self::NodeIter::new(self.nodes.len())
    }

    fn range<'n>(&'n self, lo_ptr: Self::NodeRef, hi_ptr: Self::NodeRef) -> Self::NodeIter<'n> {
        unimplemented!()
    }
}

impl<K: Key, V: 'static, const FANOUT: usize> DiskBTreeLayer<K, V, FANOUT> {
    fn add_node(&mut self, node: BTreeNode<K, V, FANOUT>) -> usize {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    fn insert(&mut self, key: K, value: V, mut ptr: usize) -> Option<(K, usize)> {
        let node_key: K = *self.nodes[ptr].borrow();

        if self.nodes[ptr].is_full() {
            // Split
            let (split_point, new_node) = self.nodes[ptr].split();

            let mut new_node_ptr = self.add_node(new_node);

            // Insert into the right node
            if (key < split_point) {
                self.nodes[ptr].insert(key, value);
            } else {
                self.nodes[new_node_ptr].insert(key, value);
            }

            return Some((*self.nodes[new_node_ptr].borrow(), new_node_ptr));
        } else {
            self.nodes[ptr].insert(key, value);
        }

        None
    }
}

impl<K: Key, const FANOUT: usize, B: NodeLayer<K>> InternalComponent<K, B>
    for DiskBTreeLayer<K, B::NodeRef, FANOUT>
{
    fn new_internal(base: &B) -> Self {
        let mut layer = Self { nodes: Vec::new() };
        let mut node_ptr = layer.add_node(BTreeNode::empty());

        for base_ptr in base.iter() {
            if layer.nodes[node_ptr].is_full() {
                node_ptr = layer.add_node(BTreeNode::empty());
            }

            let base_node = base.node_ref(base_ptr.clone());
            layer.nodes[node_ptr].insert(*base_node.borrow(), base_ptr);
        }

        layer
    }

    fn search_internal(&self, key: &K, ptr: Self::NodeRef) -> B::NodeRef {
        self.nodes[ptr].search_lub(key).clone()
    }

    fn insert_internal(
        &mut self,
        key: K,
        value: B::NodeRef,
        mut ptr: Self::NodeRef,
    ) -> Option<(K, Self::NodeRef)> {
        self.insert(key, value, ptr)
    }
}

impl<K: Key, V: Value, const FANOUT: usize> BaseComponent<K, V> for DiskBTreeLayer<K, V, FANOUT> {
    fn new_base() -> Self {
        let mut result = Self { nodes: Vec::new() };

        result.add_node(BTreeNode::empty());

        result
    }

    fn search_base(&self, key: &K, ptr: Self::NodeRef) -> Option<&V> {
        self.nodes[ptr].search_exact(key)
    }

    fn insert_base(
        &mut self,
        key: K,
        value: V,
        mut ptr: Self::NodeRef,
    ) -> Option<(K, Self::NodeRef)> {
        self.insert(key, value, ptr)
    }
}

impl<K: Key, V: Debug + Clone, const FANOUT: usize> Debug for DiskBTreeLayer<K, V, FANOUT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.nodes.iter()).finish()
    }
}
