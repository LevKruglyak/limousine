//! Contains some components to build piecewise models which approximate the location of entries,
//! for use in a learned index of some form. For example, the PGM index uses linear models for nodes.

use crate::kv::{Key, Value};
use crate::{component::NodeLayer, kv::StaticBounded};
use std::{borrow::Borrow, fmt::Debug, marker::PhantomData, ops::Deref, path::Path};

// ----------------------------------------
// Helper Types
// ----------------------------------------

type Address = usize;

pub struct ApproxPos {
    pub lo: usize,
    pub hi: usize,
}

// ----------------------------------------
// Node Type
// ----------------------------------------

pub struct PiecewiseNode<K: Key, V, M: Model<K>> {
    pub model: M,
    pub data: Vec<(K, V)>,
}

// ----------------------------------------
// Model Type
// ----------------------------------------

/// An algorithm for turning a list of key-rank pairs into a piecewise model.
pub trait Segmentation<K: Key, V: Value, M: Model<K>>: 'static {
    fn make_segmentation(data: impl Iterator<Item = (K, V)>) -> Vec<PiecewiseNode<K, V, M>>;
}

/// A model for approximate the location of a key, for use in a larged piecewise learned index
/// layer. Must implement `Keyed<K>`, here the `.key()` method represents the maximum key which
/// this model represents.
pub trait Model<K: Key>: Borrow<K> + Debug + 'static {
    /// Returns the approximate position of the specified key.
    fn approximate(&self, key: &K) -> ApproxPos;
}

// ----------------------------------------
// Iterator Type
// ----------------------------------------

pub struct Iter<'n, K: Key, V, M: Model<K>, S> {
    layer: &'n PiecewiseLayer<K, V, M, S>,
    ix: Address,
}

impl<'n, K: Key, V, M: Model<K>, S> Iter<'n, K, V, M, S> {
    fn new(layer: &'n PiecewiseLayer<K, V, M, S>, ix: usize) -> Self {
        Self { layer, ix }
    }
}

impl<'n, K: Key, V, M: Model<K>, S> Iterator for Iter<'n, K, V, M, S>
where
    K: StaticBounded,
    V: 'static,
{
    type Item = (K, Address);

    fn next(&mut self) -> Option<Self::Item> {
        self.ix += 1;
        if self.ix < self.layer.nodes.len() {
            Some((*self.layer.nodes[self.ix].model.borrow(), self.ix))
        } else {
            None
        }
    }
}

// ----------------------------------------
// Layer Types
// ----------------------------------------

pub struct PiecewiseLayer<K: Key, V, M: Model<K>, S> {
    pub nodes: Vec<PiecewiseNode<K, V, M>>,
    _seg_marker: PhantomData<S>,
}

/*impl<K: Key, V: Value, M: Model<K>, S: Segmentation<K, V, M>> NodeLayer<K>
    for PiecewiseLayer<K, V, M, S>
where
    K: 'static + StaticBounded,
    V: 'static,
{
    type Node = M;
    type Address = usize;
    type Iter<'n> = Iter<'n, K, V, M, S>;

    fn deref(&self, address: Self::Address) -> &Self::Node {
        &self.nodes[address]
    }
}
*/
