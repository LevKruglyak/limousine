//! Contains some components to build piecewise models which approximate the location of entries,
//! for use in a learned index of some form. For example, the PGM index uses linear models for nodes.

use crate::common::entry::Entry;
use crate::common::heap_map::HeapMap;
use crate::common::search::{BinarySearch, Search};
use crate::common::stack_map::StackMap;
use crate::component;
use crate::kv::{Key, KeyBounded, Value};
use crate::{component::NodeLayer, kv::StaticBounded};
use generational_arena::{Arena, Index};
use std::ops::Bound;
use std::ptr::NonNull;
use std::{borrow::Borrow, fmt::Debug, marker::PhantomData, ops::Deref, path::Path};

// ----------------------------------------
// Helper Types
// ----------------------------------------

pub type Node<K, V, M> = PiecewiseNode<K, V, M>;
pub type Address = Index;
pub type OptAddress = Option<Index>;

// ----------------------------------------
// Iteration Types
// ----------------------------------------

/// A struct to iterate over learned nodes in the same layer
#[derive(Clone)]
pub struct Iter<'n, K: Key, V, M: Model<K>, S: Segmentation<K, V, M>> {
    layer: &'n PiecewiseLayer<K, V, M, S>,
    current: OptAddress,
    end: Bound<Address>,
    _entry_marker: PhantomData<(K, V, M, S)>,
}

impl<'n, K: Key, V, M: Model<K>, S: Segmentation<K, V, M>> Iter<'n, K, V, M, S> {
    fn new(layer: &'n PiecewiseLayer<K, V, M, S>) -> Self {
        Self {
            layer,
            current: layer.head,
            end: Bound::Unbounded,
            _entry_marker: Default::default(),
        }
    }

    fn range(
        layer: &'n PiecewiseLayer<K, V, M, S>,
        start: Bound<Address>,
        end: Bound<Address>,
    ) -> Self {
        match start {
            Bound::Included(id) => Self {
                layer,
                current: Some(id),
                end,
                _entry_marker: Default::default(),
            },
            Bound::Excluded(id) => {
                let node = layer.arena.get(id);
                if node.is_none() || node.unwrap().next.is_none() {
                    Self {
                        layer,
                        current: None,
                        end,
                        _entry_marker: Default::default(),
                    }
                } else {
                    Self {
                        layer,
                        current: node.unwrap().next,
                        end,
                        _entry_marker: Default::default(),
                    }
                }
            }
            Bound::Unbounded => Self {
                layer,
                current: layer.head,
                end,
                _entry_marker: Default::default(),
            },
        }
    }
}

impl<'n, K: Key, V, M: Model<K>, S: Segmentation<K, V, M>> Iterator for Iter<'n, K, V, M, S>
where
    K: StaticBounded,
    V: 'static,
{
    type Item = Entry<K, Address>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(cur_id) => {
                let Some(node) = self.layer.arena.get(cur_id) else { return None; };
                let Some(next_id) = node.next else { return None; };
                if Bound::Excluded(next_id) == self.end {
                    return None;
                }
                self.current = Some(next_id);
                let Some(node) = self.layer.arena.get(next_id) else {return None;};
                Some(Entry::new(node.lower_bound().clone(), next_id))
            }
        }
    }
}

// ----------------------------------------
// Node Type
// ----------------------------------------

pub struct PiecewiseNode<K: Key, V, M: Model<K>> {
    pub model: M,
    pub data: Vec<Entry<K, V>>, // TODO: Eventually replace with heapmap, or something more optimized
    pub next: Option<Index>,
}

impl<K: Key, V, M: Model<K>> KeyBounded<K> for PiecewiseNode<K, V, M> {
    fn lower_bound(&self) -> &K {
        self.model.borrow()
    }
}

// ----------------------------------------
// Model Type
// ----------------------------------------

/// An algorithm for turning a list of key-rank pairs into a piecewise model.
pub trait Segmentation<K: Key, V, M: Model<K>>: Clone + 'static {
    /// Given a list of entries and an arena to allocate nodes into, constructs a flat learned layer
    fn make_segmentation(
        data: impl Iterator<Item = Entry<K, V>> + Clone,
        arena: &mut Arena<PiecewiseNode<K, V, M>>,
    ) -> Index;
}

pub struct ApproxPos {
    pub lo: usize,
    pub hi: usize,
}

/// A model for approximate the location of a key, for use in a larged piecewise learned index
/// layer. Must implement `Keyed<K>`, here the `.key()` method represents the maximum key which
/// this model represents.
pub trait Model<K: Key>: Borrow<K> + Debug + Clone + 'static {
    /// Returns the approximate position of the specified key.
    fn approximate(&self, key: &K) -> ApproxPos;
}

// ----------------------------------------
// Layer Types
// ----------------------------------------

/// Implement the node layer abstractions
pub struct PiecewiseLayer<K: Key, V, M: Model<K>, S: Segmentation<K, V, M>> {
    pub arena: Arena<PiecewiseNode<K, V, M>>,
    pub head: Option<Index>,
    _seg_marker: PhantomData<S>,
}

impl<K: Key, V: Clone, M: Model<K>, S: Segmentation<K, V, M>> PiecewiseLayer<K, V, M, S>
where
    K: 'static + StaticBounded,
    V: 'static,
{
    pub fn build(data: impl Iterator<Item = Entry<K, V>> + Clone) -> Self {
        let mut arena = Arena::new();
        let head = Some(S::make_segmentation(data, &mut arena));

        Self {
            arena,
            head,
            _seg_marker: PhantomData,
        }
    }
}

impl<K: Key, V: Clone, M: Model<K>, S: Segmentation<K, V, M>> NodeLayer<K>
    for PiecewiseLayer<K, V, M, S>
where
    K: 'static + StaticBounded,
    V: 'static,
{
    type Node = Node<K, V, M>;
    type Address = Address;
    type Iter<'n> = Iter<'n, K, V, M, S>;

    fn deref(&self, id: Self::Address) -> &Self::Node {
        &self.arena.get(id).unwrap()
    }

    fn deref_mut(&mut self, mut id: Self::Address) -> &mut Self::Node {
        self.arena.get_mut(id).unwrap()
    }

    fn range<'n>(
        &'n self,
        start: std::ops::Bound<Self::Address>,
        end: std::ops::Bound<Self::Address>,
    ) -> Self::Iter<'n> {
        Self::Iter::range(self, start, end)
    }

    fn full_range<'n>(&'n self) -> Self::Iter<'n> {
        Self::Iter::range(self, Bound::Unbounded, Bound::Unbounded)
    }
}

/// Basic implementations for common functions on a layer
impl<K: Key, V, M: Model<K>, S: Segmentation<K, V, M>> PiecewiseLayer<K, V, M, S>
where
    K: 'static + StaticBounded,
    V: 'static + Clone,
{
    pub fn search(
        &self,
        id: <PiecewiseLayer<K, V, M, S> as component::NodeLayer<K>>::Address,
        key: &K,
    ) -> &Entry<K, V> {
        let node = self.arena.get(id).unwrap();
        let approx_pos = node.model.approximate(key);
        let found_ix = BinarySearch::search_by_key(&node.data[approx_pos.lo..approx_pos.hi], key);
        match found_ix {
            Ok(ix) => &node.data[ix],
            Err(ix) => {
                if ix > 0 {
                    &node.data[ix - 1]
                } else {
                    &node.data[ix]
                }
            }
        }
    }
}
