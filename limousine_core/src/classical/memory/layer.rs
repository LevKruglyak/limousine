use crate::classical::node::BTreeNode;
use crate::BaseComponent;
use crate::{InternalComponent, Key, NodeLayer, Value};
use bumpalo::boxed::Box;
use bumpalo::Bump;
use std::borrow::Borrow;
use std::cell::Ref;
use std::cell::RefCell;
use std::fmt::Debug;
use std::iter::Map;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::rc::Rc;
use std::slice::Iter;

/// A `BTreeLayer` is an `InternalLayer` with a constant size ratio
/// given by the `FANOUT` parameter. Each node can index a maximum of
/// `FANOUT` lower nodes, and since this is part of an immutable index, this
/// fill factor is always achieved.
pub struct BTreeLayer<K, V, const FANOUT: usize> {
    pub nodes: Vec<NonNull<BTreeNode<K, V, FANOUT>>>,
    alloc: Bump,
}

pub struct BTreeLayerNodeIterator<'n, N> {
    nodes: &'n [N],
    idx: usize,
}

impl<'n, K, V, const FANOUT: usize> BTreeLayerNodeIterator<'n, NonNull<BTreeNode<K, V, FANOUT>>> {
    fn new(layer: &'n BTreeLayer<K, V, FANOUT>) -> Self {
        Self {
            nodes: layer.nodes.as_slice(),
            idx: 0,
        }
    }
}

impl<'n, N: Clone> Iterator for BTreeLayerNodeIterator<'n, N> {
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.nodes.get(self.idx).cloned();
        self.idx += 1;
        result
    }
}

impl<K: Key, V: 'static, const FANOUT: usize> NodeLayer<K> for BTreeLayer<K, V, FANOUT> {
    type Node = BTreeNode<K, V, FANOUT>;
    type NodeRef = NonNull<Self::Node>;

    fn node_ref(&self, ptr: Self::NodeRef) -> &Self::Node {
        // TODO: safety!
        unsafe { ptr.as_ref() }
    }

    type NodeIter<'n> = BTreeLayerNodeIterator<'n, Self::NodeRef>;

    fn iter<'n>(&'n self) -> Self::NodeIter<'n> {
        Self::NodeIter::new(&self)
    }
}

impl<K: Key, V: 'static, const FANOUT: usize> BTreeLayer<K, V, FANOUT> {
    fn add_node(&mut self, node: BTreeNode<K, V, FANOUT>) -> NonNull<BTreeNode<K, V, FANOUT>> {
        let ptr = NonNull::new(Box::into_raw(Box::new_in(node, &self.alloc))).unwrap();
        self.nodes.push(ptr);
        ptr
    }

    fn insert(
        &mut self,
        key: K,
        value: V,
        mut ptr: NonNull<BTreeNode<K, V, FANOUT>>,
    ) -> Option<(K, NonNull<BTreeNode<K, V, FANOUT>>)> {
        let node = unsafe { ptr.as_mut() };
        let node_key: K = *node.borrow();

        if node.is_full() {
            // Split
            let (split_point, new_node) = node.split();

            let mut new_node_ptr = self.add_node(new_node);
            let new_node = unsafe { new_node_ptr.as_mut() };

            // Insert into the right node
            if (key < split_point) {
                node.insert(key, value);
            } else {
                new_node.insert(key, value);
            }

            return Some((*new_node.borrow(), new_node_ptr));
        } else {
            node.insert(key, value);
        }

        None
    }
}

impl<K: Key, const FANOUT: usize, B: NodeLayer<K>> InternalComponent<K, B>
    for BTreeLayer<K, B::NodeRef, FANOUT>
{
    fn new_internal(base: &B) -> Self {
        let mut layer = Self {
            nodes: Vec::new(),
            alloc: Bump::new(),
        };
        let mut node = unsafe { layer.add_node(BTreeNode::empty()).as_mut() };

        for base_ptr in base.iter() {
            if node.is_full() {
                node = unsafe { layer.add_node(BTreeNode::empty()).as_mut() };
            }

            let base_node = base.node_ref(base_ptr.clone());
            node.insert(*base_node.borrow(), base_ptr);
        }

        layer
    }

    fn search_internal(&self, key: &K, ptr: Self::NodeRef) -> B::NodeRef {
        let node = unsafe { ptr.as_ref() };

        node.search_lub(key).clone()
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

impl<K: Key, V: Value, const FANOUT: usize> BaseComponent<K, V> for BTreeLayer<K, V, FANOUT> {
    fn new_base() -> Self {
        let mut result = Self {
            nodes: Vec::new(),
            alloc: Bump::new(),
        };

        result.add_node(BTreeNode::empty());

        result
    }

    fn search_base(&self, key: &K, ptr: Self::NodeRef) -> Option<&V> {
        let node = unsafe { ptr.as_ref() };

        node.search_exact(key)
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

impl<K: Key, V: Debug + Clone, const FANOUT: usize> Debug for BTreeLayer<K, V, FANOUT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.nodes.iter().map(|node| unsafe { node.as_ref() }))
            .finish()
    }
}
