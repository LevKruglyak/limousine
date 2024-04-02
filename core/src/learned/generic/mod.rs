//! Contains some components to build piecewise models which approximate the location of entries,
//! for use in a learned index of some form. For example, the PGM index uses linear models for nodes.

use crate::component::NodeLayer;
use crate::kv::Key;
use crate::kv::KeyBounded;
use std::ops::{Bound, RangeBounds};
use std::{borrow::Borrow, fmt::Debug, marker::PhantomData, ops::Deref, path::Path};

pub mod pgm;

#[derive(Debug)]
pub struct ApproxPos {
    pub lo: usize,
    pub hi: usize,
}

/// A model for approximate the location of a key, for use in a larged piecewise learned index
/// layer. Must implement `KeyBounded<K>`, here the `.lower_bound()` method represents the minimum
/// key which this model represents.
pub trait Model<K>: KeyBounded<K> + 'static {
    /// Returns the approximate position of the specified key.
    fn approximate(&self, key: &K) -> ApproxPos;
}

/// An algorithm for turning a list of key-rank pairs into a piecewise model.
pub trait Segmentation<K, M>: 'static {
    fn make_segmentation(key_ranks: impl Iterator<Item = (K, usize)>) -> Vec<M>;
}

/// A piecewise collection of models that approximates the rank of a range of keys.
pub struct PiecewiseModel<K, Model, Segmentation> {
    models: Vec<Model>,

    /// Keep the segmentation as a generic parameter so that we can automatically rebuild segments
    /// of the piecewise model
    _ph: PhantomData<(K, Segmentation)>,
}

impl<K, M, S> PiecewiseModel<K, M, S> {
    pub fn new(models: Vec<M>) -> Self {
        Self {
            models,
            _ph: PhantomData,
        }
    }

    pub fn build(key_ranks: impl Iterator<Item = (K, usize)>) -> Self
    where
        S: Segmentation<K, M>,
    {
        Self {
            models: S::make_segmentation(key_ranks),
            _ph: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.models.len()
    }

    pub fn models(&self) -> &[M] {
        self.models.as_slice()
    }

    pub fn approximate(&self, address: usize, key: &K) -> ApproxPos
    where
        M: Model<K>,
    {
        self.models[address].approximate(key)
    }
}

pub struct Iter<'n, K, M, S> {
    model: &'n PiecewiseModel<K, M, S>,
    current: usize,
    end: usize,
}

impl<'n, K, M, S> Iter<'n, K, M, S> {
    fn new(model: &'n PiecewiseModel<K, M, S>) -> Self {
        Self {
            model,
            current: 0,
            end: model.len(),
        }
    }

    fn range(model: &'n PiecewiseModel<K, M, S>, start: Bound<usize>, end: Bound<usize>) -> Self {
        let current = match start {
            Bound::Included(start) => start,
            Bound::Excluded(start) => start + 1,
            Bound::Unbounded => 0,
        };

        let end = match end {
            Bound::Included(start) => start,
            Bound::Excluded(start) => start + 1,
            Bound::Unbounded => model.len(),
        };

        Self {
            model,
            current,
            end,
        }
    }
}

impl<'n, K, M, S> Iterator for Iter<'n, K, M, S>
where
    K: Copy,
    M: KeyBounded<K>,
{
    type Item = (K, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            return None;
        }

        let result = self
            .model
            .models()
            .get(self.current)
            .map(|node| (*node.lower_bound(), self.current));
        self.current += 1;
        result
    }
}

impl<K, M, S> NodeLayer<K> for PiecewiseModel<K, M, S>
where
    K: Key,
    M: Model<K>,
    S: 'static,
{
    type Node = M;
    type Address = usize;

    fn deref(&self, ptr: Self::Address) -> &Self::Node {
        &self.models[ptr]
    }

    fn deref_mut(&mut self, ptr: Self::Address) -> &mut Self::Node {
        &mut self.models[ptr]
    }

    type Iter<'n> = Iter<'n, K, M, S>;

    fn range<'n>(
        &'n self,
        start: Bound<Self::Address>,
        end: Bound<Self::Address>,
    ) -> Self::Iter<'n> {
        Self::Iter::range(&self, start, end)
    }

    fn full_range<'n>(&'n self) -> Self::Iter<'n> {
        Self::Iter::new(&self)
    }
}
