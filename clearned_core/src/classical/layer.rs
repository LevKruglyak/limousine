use std::ops::Deref;

use super::node::BTreeNode;
use crate::{Key, NodeLayer, Value};
use mmap_buffer::Buffer;

pub struct BTreeLayer<K: Key, V: Value, const FANOUT: usize> {
    nodes: Buffer<BTreeNode<K, V, FANOUT>>,
}

impl<K: Key, V: Value, const FANOUT: usize> NodeLayer<K> for BTreeLayer<K, V, FANOUT> {
    type Node = BTreeNode<K, V, FANOUT>;

    fn nodes(&self) -> &[Self::Node] {
        self.nodes.deref()
    }
}
