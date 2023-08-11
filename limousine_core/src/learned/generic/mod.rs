//! Contains some components to build piecewise models which approximate the location of entries,
//! for use in a learned index of some form. For example, the PGM index uses linear models for nodes.

use crate::kv::Key;
use crate::NodeLayer;
use std::{borrow::Borrow, fmt::Debug, marker::PhantomData, ops::Deref, path::Path};

pub mod pgm;
pub mod pgm_node;

pub struct ApproxPos {
    lo: usize,
    hi: usize,
}

/// An algorithm for turning a list of key-rank pairs into a piecewise model.
pub trait Segmentation<K: Key, M: Model<K>>: 'static {
    fn make_segmentation(key_ranks: impl ExactSizeIterator<Item = (usize, K)>) -> Vec<M>;
}

/// A model for approximate the location of a key, for use in a larged piecewise learned index
/// layer. Must implement `Keyed<K>`, here the `.key()` method represents the maximum key which
/// this model represents.
pub trait Model<K: Key>: Borrow<K> + Debug + 'static {
    /// Returns the approximate position of the specified key.
    fn approximate(&self, key: &K) -> ApproxPos;
}

/// A piecewise collection of models that approximates the locations a large range of keys.
pub struct PiecewiseModel<K: Key, M: Model<K>, S: Segmentation<K, M>> {
    models: Vec<M>,
    _ph: PhantomData<(K, S)>,
}

// impl<K: Key, M: Model<K>, S: Segmentation<K, M>> NodeLayer<K> for PiecewiseModel<K, M, S> {
//     type Node = M;
//     type NodeRef = usize;
//
//     fn node_ref(&self, ptr: Self::NodeRef) -> &Self::Node {
//         &self.models[ptr]
//     }
// }
