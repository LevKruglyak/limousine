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
type Address<K, V, M> = NonNull<Node<K, V, M>>;
type OptAddress<K, V, M> = Option<Address<K, V, M>>;

// ----------------------------------------
// Iteration Types
// ----------------------------------------

/// A struct to stay organized while iterating over learned nodes in the same layer
pub struct Iter<'n, K: Key, V, M: Model<K>, S: Segmentation<K, V, M>> {
    layer: &'n PiecewiseLayer<K, V, M, S>,
    current: OptAddress<K, V, M>,
    end: Bound<Address<K, V, M>>,
}

impl<'n, K: Key, V, M: Model<K>, S: Segmentation<K, V, M>> Iter<'n, K, V, M, S> {
    fn new(layer: &'n PiecewiseLayer<K, V, M, S>) -> Self {
        Self {
            layer,
            current: Some(layer.nodes[0]),
            end: Bound::Unbounded,
        }
    }
}

impl<'n, K: Key, V, M: Model<K>, S: Segmentation<K, V, M>> Iterator for Iter<'n, K, V, M, S>
where
    K: StaticBounded,
    V: 'static,
{
    type Item = (K, Address<K, V, M>);

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

// ----------------------------------------
// Node Type
// ----------------------------------------

pub struct PiecewiseNode<K: Key, V, M: Model<K>> {
    pub model: M,
    pub data: Vec<(K, V)>,
    pub next: OptAddress<K, V, M>,
    _ph: PhantomData<(K, V)>,
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
    pub nodes: Vec<Address<K, V, M>>,
    _seg_marker: PhantomData<S>,
}

impl<K: Key, V: Value, M: Model<K>, S: Segmentation<K, V, M>> NodeLayer<K>
    for PiecewiseLayer<K, V, M, S>
where
    K: 'static + StaticBounded,
    V: 'static,
{
    type Node = Node<K, V, M>;
    type Address = Address<K, V, M>;
    type Iter<'n> = Iter<'n, K, V, M, S>;

    fn deref(&self, ptr: Self::Address) -> &Self::Node {
        unsafe { ptr.as_ref() }
    }

    fn deref_mut(&mut self, mut ptr: Self::Address) -> &mut Self::Node {
        unsafe { ptr.as_mut() }
    }

    fn range<'n>(
        &'n self,
        start: std::ops::Bound<Self::Address>,
        end: std::ops::Bound<Self::Address>,
    ) -> Self::Iter<'n> {
        Self::Iter::new(self)
    }

    fn full_range<'n>(&'n self) -> Self::Iter<'n> {
        Self::Iter::new(self)
    }
}
