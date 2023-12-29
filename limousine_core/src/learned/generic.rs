//! Contains some components to build piecewise models which approximate the location of entries,
//! for use in a learned index of some form. For example, the PGM index uses linear models for nodes.

use crate::common::entry::Entry;
use crate::common::search::{BinarySearch, Search};
use crate::common::stack_map::StackMap;
use crate::component::{Key, NodeLayer, Value};
use generational_arena::{Arena, Index};
use std::ops::Bound;
use std::ptr::NonNull;
use std::{borrow::Borrow, fmt::Debug, marker::PhantomData, ops::Deref, path::Path};

/// A learned model. Nothing fancy, just means that it provides a way to
/// approximate the position of a key.
pub trait LearnedModel<K: Key>: Borrow<K> + Debug + Clone + 'static {
    /// Returns the approximate position of the specified key.
    fn approximate(&self, key: &K) -> ApproxPos;
}

/// Behavior that cretes a list of learned models from a list of entries
pub trait Segmentation<K: Key, V: Value, M: LearnedModel<K>>: Clone + 'static {
    /// Given a list of entries, return the split into models
    fn make_segmentation(data: impl Iterator<Item = Entry<K, V>>) -> Vec<(Self, Vec<Entry<K, V>>)>;
}

/// The result of an approximation search
pub struct ApproxPos {
    pub lo: usize,
    pub hi: usize,
}
