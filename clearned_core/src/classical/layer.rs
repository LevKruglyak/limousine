use std::{ops::Deref, path::Path};

use super::node::BTreeNode;
use crate::Entry;
use crate::{
    path_with_extension,
    search::{lower_bound, OptimalSearch, Search},
    ApproxPos, InternalLayer, InternalLayerBuild, Key, NodeLayer, Result,
};
use mmap_buffer::Buffer;

pub struct BTreeLayer<K: Key, const FANOUT: usize> {
    nodes: Buffer<BTreeNode<K, usize, FANOUT>>,
}

impl<K: Key, const FANOUT: usize> NodeLayer<K> for BTreeLayer<K, FANOUT> {
    type Node = BTreeNode<K, usize, FANOUT>;

    fn nodes(&self) -> &[Self::Node] {
        self.nodes.deref()
    }
}

impl<K: Key, const FANOUT: usize> InternalLayer<K> for BTreeLayer<K, FANOUT> {
    fn search(&self, key: &K, range: ApproxPos) -> ApproxPos {
        // Small optimization for exact positions
        let node = if range.lo == range.hi - 1 {
            self.nodes[range.lo]
        } else {
            self.nodes[lower_bound(OptimalSearch::search_by_key_with_offset(
                &self.nodes[range.lo..range.hi],
                key,
                range.lo,
            ))]
        };

        let ptr = node.search(key);
        println!("found node with entries {:?}", node.entries());

        ApproxPos {
            lo: ptr,
            hi: ptr + 1,
        }
    }
}

impl<K: Key, const FANOUT: usize> InternalLayerBuild<K> for BTreeLayer<K, FANOUT> {
    fn build(base: impl ExactSizeIterator<Item = K>) -> Self
    where
        Self: Sized,
    {
        let data: Vec<Entry<K, usize>> = base
            .enumerate()
            .map(|(ptr, min)| Entry::new(min, ptr))
            .collect();

        let capacity = data.len() / FANOUT + 2;
        let nodes = Buffer::new_in_memory(capacity);

        Self::build_internal(&data, nodes)
    }

    fn build_on_disk(base: impl ExactSizeIterator<Item = K>, path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let data: Vec<Entry<K, usize>> = base
            .enumerate()
            .map(|(ptr, min)| Entry::new(min, ptr))
            .collect();

        let capacity = data.len() / FANOUT + 2;
        let nodes = Buffer::new_on_disk(capacity, path_with_extension(path.as_ref(), "bl"))?;

        Ok(Self::build_internal(&data, nodes))
    }

    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let nodes = Buffer::load_from_disk(path_with_extension(path.as_ref(), "bl"))?;

        Ok(Self { nodes })
    }
}

impl<K: Key, const FANOUT: usize> BTreeLayer<K, FANOUT> {
    fn build_internal(
        data: &[Entry<K, usize>],
        mut nodes: Buffer<BTreeNode<K, usize, FANOUT>>,
    ) -> Self {
        // Always add extra padding node
        // nodes[0] = BTreeNode::empty();

        let mut index = 0;
        let mut start = 0;

        // TODO: replace with slice
        while start < data.len() {
            let end = (start + FANOUT).min(data.len());
            nodes[index] = BTreeNode::empty();

            for &key_ptr in &data[start..end] {
                nodes[index].push(key_ptr);
            }

            start = end;
            index += 1;
        }

        nodes.shrink(index);

        Self { nodes }
    }
}
