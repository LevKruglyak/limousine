pub mod pgm_node;
pub mod pgm_segmentation;

use crate::common::search::*;
use crate::component::*;
use crate::kv::Key;
use generic::pgm::{LinearModel, PGMSegmentation};
// use generic::pgm_node::LinearModel;
// use generic::pgm_node::PGMLayer;
use generic::*;
use std::collections::HashMap;
use std::ops::Bound;
use std::ops::RangeBounds;

// -------------------------------------------------------
//                  PGM Internal Component
// -------------------------------------------------------

type PGMLayer<K, const EPSILON: usize> =
    PiecewiseModel<K, LinearModel<K, EPSILON>, PGMSegmentation>;

pub struct PGMInternalComponent<K, Base: NodeLayer<K>, const EPSILON: usize> {
    inner: PGMLayer<K, EPSILON>,
    mapping: Vec<Base::Address>,
}

impl<K: Key, Base: NodeLayer<K>, const EPSILON: usize> NodeLayer<K>
    for PGMInternalComponent<K, Base, EPSILON>
{
    type Node = <PGMLayer<K, EPSILON> as NodeLayer<K>>::Node;
    type Address = <PGMLayer<K, EPSILON> as NodeLayer<K>>::Address;

    fn deref(&self, ptr: Self::Address) -> &Self::Node {
        self.inner.deref(ptr)
    }

    fn deref_mut(&mut self, ptr: Self::Address) -> &mut Self::Node {
        self.inner.deref_mut(ptr)
    }

    type Iter<'n> = <PGMLayer<K, EPSILON> as NodeLayer<K>>::Iter<'n>;

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
        let mut approx_pos = self.inner.approximate(ptr, key);

        // Adjust to avoid out-of-bounds
        if approx_pos.lo >= self.mapping.len() {
            approx_pos.lo = self.mapping.len() - 1;
        }

        if approx_pos.hi > self.mapping.len() {
            approx_pos.hi = self.mapping.len();
        }

        let start = Bound::Included(self.mapping[approx_pos.lo].clone());
        let end = Bound::Included(self.mapping[approx_pos.hi - 1].clone());

        for (base_key, base_address) in base.range(start, end) {
            if key >= &base_key {
                return base_address;
            }
        }

        unreachable!()
    }

    fn insert<'n>(
        &'n mut self,
        base: &Base,
        ptr: Self::Address,
        prop: PropogateInsert<K, Base>,
    ) -> Option<PropogateInsert<K, Self>> {
        // Don't care what prop is, we always rebuild
        self.mapping = Vec::new();

        // TODO: move
        let models = PGMSegmentation::make_segmentation(base.full_range().enumerate().map(
            |(rank, (key, address))| {
                self.mapping.push(address);
                return (key, rank);
            },
        ));

        self.inner = PGMLayer::new(models);

        Some(PropogateInsert::Rebuild)
    }

    fn memory_size(&self) -> usize {
        self.inner.len() * std::mem::size_of::<LinearModel<K, EPSILON>>()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K: Key, Base: NodeLayer<K>, const EPSILON: usize> InternalComponentInMemoryBuild<K, Base>
    for PGMInternalComponent<K, Base, EPSILON>
{
    fn build(base: &Base) -> Self {
        let mapping = base.full_range().map(|(_, address)| address).collect();

        // TODO: move
        let models = PGMSegmentation::make_segmentation(
            base.full_range()
                .enumerate()
                .map(|(rank, (key, address))| (key, rank)),
        );

        Self {
            inner: PiecewiseModel::new(models),
            mapping,
        }
    }
}
