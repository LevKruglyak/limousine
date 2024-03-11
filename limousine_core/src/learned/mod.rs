//! Contains some components to build piecewise models which approximate the location of entries,
//! for use in a learned index of some form. For example, the PGM index uses linear models for nodes.

use super::{ApproxPos, InternalLayer, InternalLayerBuild, NodeLayer};
use crate::{
    path_with_extension,
    search::{lower_bound, OptimalSearch, Search},
    Key, Result,
};
use bytemuck::Pod;
use mmap_buffer::Buffer;
use std::{borrow::Borrow, fmt::Debug, marker::PhantomData, ops::Deref, path::Path};

pub mod pgm;
pub mod pgm_node;

/// An algorithm for turning a list of key-rank pairs into a piecewise model.
pub trait Segmentation<K: Key, M: Model<K>> {
    fn make_segmentation(key_ranks: impl ExactSizeIterator<Item = (usize, K)>) -> Vec<M>;
}

/// A model for approximate the location of a key, for use in a larged piecewise learned index
/// layer. Must implement `Keyed<K>`, here the `.key()` method represents the maximum key which
/// this model represents.
pub trait Model<K: Key>: Pod + Borrow<K> + Debug {
    /// Returns the approximate position of the specified key.
    fn approximate(&self, key: &K) -> ApproxPos;
}

/// A piecewise collection of models that approximates the locations a large range of keys.
pub struct PiecewiseModel<K: Key, M: Model<K>, S: Segmentation<K, M>> {
    models: Buffer<M>,
    _ph: PhantomData<(K, S)>,
}

impl<K: Key, M: Model<K>, S: Segmentation<K, M>> InternalLayer<K> for PiecewiseModel<K, M, S> {
    fn search(&self, key: &K, range: ApproxPos) -> ApproxPos {
        // First pass
        let model = self.models[lower_bound(OptimalSearch::search_by_key_with_offset(
            &self.models[range.lo..range.hi],
            key,
            range.lo,
        ))];

        // println!("found model {:?}", model);

        let pos = model.approximate(key);
        pos
    }
}

impl<K: Key, M: Model<K>, S: Segmentation<K, M>> InternalLayerBuild<K> for PiecewiseModel<K, M, S> {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        let models = Buffer::load_from_disk(path_with_extension(path.as_ref(), "models"))?;

        Ok(Self {
            models,
            _ph: PhantomData,
        })
    }

    fn build(base: impl ExactSizeIterator<Item = K>) -> Self
    where
        Self: Sized,
    {
        let models = Buffer::from_vec_in_memory(S::make_segmentation(base.enumerate()));

        Self {
            models,
            _ph: PhantomData,
        }
    }

    fn build_on_disk(
        base: impl ExactSizeIterator<Item = K>,
        path: impl AsRef<Path>,
    ) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let models = Buffer::from_slice_on_disk(
            &S::make_segmentation(base.enumerate())[..],
            path_with_extension(path.as_ref(), "models"),
        )?;

        Ok(Self {
            models,
            _ph: PhantomData,
        })
    }
}

impl<K: Key, M: Model<K>, S: Segmentation<K, M>> NodeLayer<K> for PiecewiseModel<K, M, S> {
    type Node = M;

    fn nodes(&self) -> &[Self::Node] {
        &self.models.deref()
    }
}
