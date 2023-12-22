//! Contains some components to build piecewise models which approximate the location of entries,
//! for use in a learned index of some form. For example, the PGM index uses linear models for nodes.

use crate::common::entry::Entry;
use crate::common::search::{BinarySearch, Search};
use crate::common::stack_map::StackMap;
use crate::component::NodeLayer;
use crate::{component, Key};
use generational_arena::{Arena, Index};
use std::ops::Bound;
use std::ptr::NonNull;
use std::{borrow::Borrow, fmt::Debug, marker::PhantomData, ops::Deref, path::Path};

/// A learned model for approximate the location of a key, for use in a larged piecewise learned index
/// layer. Must implement `Keyed<K>`, here the `.key()` method represents the maximum key which
/// this model represents.
pub trait LearnedModel<K: Key>: Borrow<K> + Debug + Clone + 'static {
    /// Returns the approximate position of the specified key.
    fn approximate(&self, key: &K) -> ApproxPos;
}

/// An algorithm for turning a list of key-rank pairs into a piecewise model.
pub trait Segmentation<K: Key, V, M: LearnedModel<K>>: Clone + 'static {
    /// Given a list of entries, return the split into models
    /// ? Does make_segmentation work better as just a method on model?
    /// Pro:
    ///     Less traits
    /// Con:
    ///     Each model needs to carry around knowledge of what value type it's indexing,
    ///     which seems unnecessary and potentially bad
    fn make_segmentation(data: impl Iterator<Item = Entry<K, V>> + Clone) -> Vec<(Self, Vec<Entry<K, V>>)>;
}

/// The result of an approximation search
pub struct ApproxPos {
    pub lo: usize,
    pub hi: usize,
}
