//! Contains some components to build piecewise models which approximate the location of entries,
//! for use in a learned index of some form. For example, the PGM index uses linear models for nodes.

use crate::common::heap_map::HeapMap;
use crate::common::stack_map::StackMap;
use crate::kv::{Key, KeyBounded, Value};
use crate::{component::NodeLayer, kv::StaticBounded};
use std::ops::Bound;
use std::ptr::NonNull;
use std::{borrow::Borrow, fmt::Debug, marker::PhantomData, ops::Deref, path::Path};

// ----------------------------------------
// Helper Types
// ----------------------------------------

type Node<K, V, M> = PiecewiseNode<K, V, M>;
type Address = usize;
type OptAddress = Option<usize>;

// ----------------------------------------
// Iteration Types
// ----------------------------------------

/// A struct to iterate over learned nodes in the same layer
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
            current: Some(0),
            end: Bound::Unbounded,
            _entry_marker: Default::default(),
        }
    }

    fn range(
        layer: &'n PiecewiseLayer<K, V, M, S>,
        start: Bound<Address>,
        end: Bound<Address>,
    ) -> Self {
        let mut start_ix = match start {
            Bound::Included(ix) => ix,
            Bound::Excluded(ix) => ix + 1,
            Bound::Unbounded => 0,
        };
        if start_ix >= layer.nodes.len() {
            Self {
                layer,
                current: None,
                end,
                _entry_marker: Default::default(),
            }
        } else {
            Self {
                layer,
                current: Some(start_ix),
                end,
                _entry_marker: Default::default(),
            }
        }
    }
}

impl<'n, K: Key, V, M: Model<K>, S: Segmentation<K, V, M>> Iterator for Iter<'n, K, V, M, S>
where
    K: StaticBounded,
    V: 'static,
{
    type Item = (K, Address);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(cur_ix) => {
                let mut ix = cur_ix + 1;
                let mut end_ix = self.layer.nodes.len(); // Index of first thing _not_ included
                match self.end {
                    Bound::Included(jx) => {
                        if jx + 1 < end_ix {
                            end_ix = jx + 1;
                        }
                    }
                    Bound::Excluded(jx) => {
                        if jx < end_ix {
                            end_ix = jx;
                        }
                    }
                    _ => (),
                }
                if ix >= end_ix {
                    self.current = None;
                    None
                } else {
                    self.current = Some(ix);
                    Some((self.layer.nodes[ix].lower_bound().clone(), ix))
                }
            }
        }
    }
}

// ----------------------------------------
// Node Type
// ----------------------------------------

pub struct PiecewiseNode<K: Key, V, M: Model<K>> {
    pub model: M,
    pub data: Vec<(K, V)>, // TODO: Eventually replace with heapmap, or something more optimized
                           // pub next: OptAddress<K, V, M>, Don't think we need for this implementation?
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
pub trait Segmentation<K: Key, V, M: Model<K>>: 'static {
    fn make_segmentation(data: impl Iterator<Item = (K, V)>) -> Vec<PiecewiseNode<K, V, M>>;
}

pub struct ApproxPos {
    pub lo: usize,
    pub hi: usize,
}

/// A model for approximate the location of a key, for use in a larged piecewise learned index
/// layer. Must implement `Keyed<K>`, here the `.key()` method represents the maximum key which
/// this model represents.
pub trait Model<K: Key>: Borrow<K> + Debug + 'static {
    /// Returns the approximate position of the specified key.
    fn approximate(&self, key: &K) -> ApproxPos;
}

// ----------------------------------------
// Layer Types
// ----------------------------------------

pub struct PiecewiseLayer<K: Key, V, M: Model<K>, S: Segmentation<K, V, M>> {
    pub nodes: Vec<PiecewiseNode<K, V, M>>,
    _seg_marker: PhantomData<S>,
}

impl<K: Key, V: Value, M: Model<K>, S: Segmentation<K, V, M>> NodeLayer<K>
    for PiecewiseLayer<K, V, M, S>
where
    K: 'static + StaticBounded,
    V: 'static,
{
    type Node = Node<K, V, M>;
    type Address = Address;
    type Iter<'n> = Iter<'n, K, V, M, S>;

    fn deref(&self, ix: Self::Address) -> &Self::Node {
        &self.nodes[ix]
    }

    fn deref_mut(&mut self, mut ix: Self::Address) -> &mut Self::Node {
        &mut self.nodes[ix]
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
