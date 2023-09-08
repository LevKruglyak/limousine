pub mod pgm_node;
pub mod pgm_segmentation;

use self::pgm_node::LinearModel;
use self::pgm_segmentation::PGMSegmentation;
use super::generic::{PiecewiseLayer, Segmentation};
use crate::common::search::*;
use crate::component::*;
use crate::kv::Key;
use crate::kv::StaticBounded;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::Bound;
use std::ops::RangeBounds;

// -------------------------------------------------------
//                  PGM Internal Component
// -------------------------------------------------------

pub struct PGMInternalComponent<K: Key, Base: NodeLayer<K>, const EPSILON: usize> {
    inner: PiecewiseLayer<K, Base::Address, LinearModel<K, EPSILON>, PGMSegmentation>,
}

impl<K: Key, Base: NodeLayer<K>, const EPSILON: usize> NodeLayer<K>
    for PGMInternalComponent<K, Base, EPSILON>
where
    K: StaticBounded,
{
    type Node =
        <PiecewiseLayer<K, Base::Address, LinearModel<K, EPSILON>, PGMSegmentation> as NodeLayer<
            K,
        >>::Node;
    type Address =
        <PiecewiseLayer<K, Base::Address, LinearModel<K, EPSILON>, PGMSegmentation> as NodeLayer<
            K,
        >>::Address;

    fn deref(&self, ptr: Self::Address) -> &Self::Node {
        self.inner.deref(ptr)
    }

    fn deref_mut(&mut self, ptr: Self::Address) -> &mut Self::Node {
        self.inner.deref_mut(ptr)
    }

    type Iter<'n> =
        <PiecewiseLayer<K, Base::Address, LinearModel<K, EPSILON>, PGMSegmentation> as NodeLayer<
            K,
        >>::Iter<'n>;

    fn full_range<'n>(&'n self) -> Self::Iter<'n> {
        self.inner.full_range()
    }

    fn range<'n>(
        &'n self,
        start: Bound<Self::Address>,
        end: Bound<Self::Address>,
    ) -> Self::Iter<'n> {
        self.inner.range(start, end)
    }
}

impl<K: Key, Base: NodeLayer<K>, const EPSILON: usize> InternalComponent<K, Base>
    for PGMInternalComponent<K, Base, EPSILON>
where
    Base::Address: std::fmt::Debug,
{
    fn search(&self, base: &Base, ptr: Self::Address, key: &K) -> Base::Address {
        unreachable!()
    }

    fn insert<'n>(
        &'n mut self,
        base: &Base,
        ptr: Self::Address,
        prop: PropogateInsert<K, Base>,
    ) -> Option<PropogateInsert<K, Self>> {
        Some(PropogateInsert::Rebuild)
    }

    fn memory_size(&self) -> usize {
        unreachable!()
    }

    fn len(&self) -> usize {
        self.inner.nodes.len()
    }
}

impl<K: Key, Base: NodeLayer<K>, const EPSILON: usize> InternalComponentInMemoryBuild<K, Base>
    for PGMInternalComponent<K, Base, EPSILON>
{
    fn build(base: &Base) -> Self {
        unimplemented!();
    }
}
