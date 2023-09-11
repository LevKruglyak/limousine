pub mod pgm_model;
pub mod pgm_segmentation;

use self::pgm_model::LinearModel;
use self::pgm_segmentation::PGMSegmentation;
use super::generic::PiecewiseNode;
use super::generic::{PiecewiseLayer, Segmentation};
use crate::common::entry::Entry;
use crate::common::search::*;
use crate::component::*;
use crate::kv::Key;
use crate::kv::StaticBounded;
use crate::Value;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Bound;
use std::ops::RangeBounds;

// -------------------------------------------------------
//                  PGM Top Component
// -------------------------------------------------------

pub struct PGMTopComponent<K: Key, Base: NodeLayer<K>, const EPSILON: usize> {
    inner: PiecewiseLayer<K, Base::Address, LinearModel<K, EPSILON>, PGMSegmentation>,
}

impl<K: Key, Base: NodeLayer<K>, const EPSILON: usize> NodeLayer<K>
    for PGMTopComponent<K, Base, EPSILON>
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

impl<K: Key, Base: NodeLayer<K>, const EPSILON: usize> TopComponent<K, Base>
    for PGMTopComponent<K, Base, EPSILON>
where
    Base::Address: std::fmt::Debug,
{
    fn search(&self, base: &Base, key: &K) -> <Base as NodeLayer<K>>::Address {
        // Linear scan to find the right thing to enter in
        let mut next_iter = self.inner.full_range();
        let mut last_val = next_iter.next();
        let mut next_val = next_iter.next();
        while next_val.is_some() && key < next_val.unwrap().borrow() {
            last_val = next_val;
            next_val = next_iter.next();
        }
        let last_id = last_val.unwrap().value;
        self.inner.search(last_id, key).value.clone()
    }

    fn insert(&mut self, base: &Base, prop: PropogateInsert<K, Base>) {
        // For now, no matter what kind of rebuild is coming from the layer below (full rebuild, single)
        // we're going to retrain this entire layer.
        self.inner.arena.clear();
        PGMSegmentation::make_segmentation(base.full_range(), &mut self.inner.arena);
    }

    fn len(&self) -> usize {
        self.inner.arena.len()
    }
}

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
        self.inner.search(ptr, key).value.clone()
    }

    fn insert<'n>(
        &'n mut self,
        base: &Base,
        ptr: Self::Address,
        prop: PropogateInsert<K, Base>,
    ) -> Option<PropogateInsert<K, Self>> {
        // For now, no matter what kind of rebuild is coming from the layer below (full rebuild, single)
        // we're going to retrain this entire layer.
        self.inner.arena.clear();
        PGMSegmentation::make_segmentation(base.full_range(), &mut self.inner.arena);
        Some(PropogateInsert::Rebuild)
    }

    fn memory_size(&self) -> usize {
        unimplemented!()
    }

    fn len(&self) -> usize {
        self.inner.arena.len()
    }
}

// -------------------------------------------------------
//                  PGM Base Component
// -------------------------------------------------------

pub struct PGMBaseComponent<K: Key, V: Value, const EPSILON: usize> {
    inner: PiecewiseLayer<K, V, LinearModel<K, EPSILON>, PGMSegmentation>,
}

impl<K: Key, V: Value, const EPSILON: usize> NodeLayer<K> for PGMBaseComponent<K, V, EPSILON>
where
    K: StaticBounded,
{
    type Node =
        <PiecewiseLayer<K, V, LinearModel<K, EPSILON>, PGMSegmentation> as NodeLayer<K>>::Node;
    type Address =
        <PiecewiseLayer<K, V, LinearModel<K, EPSILON>, PGMSegmentation> as NodeLayer<K>>::Address;

    fn deref(&self, ptr: Self::Address) -> &Self::Node {
        self.inner.deref(ptr)
    }

    fn deref_mut(&mut self, ptr: Self::Address) -> &mut Self::Node {
        self.inner.deref_mut(ptr)
    }

    type Iter<'n> =
        <PiecewiseLayer<K, V, LinearModel<K, EPSILON>, PGMSegmentation> as NodeLayer<K>>::Iter<'n>;

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

impl<K: Key, V: Value, const EPSILON: usize> BaseComponent<K, V, Self>
    for PGMBaseComponent<K, V, EPSILON>
{
    fn search(&self, ptr: Self::Address, key: &K) -> Option<&V> {
        let res = self.inner.search(ptr, key);
        if res.key == *key {
            Some(&res.value)
        } else {
            None
        }
    }

    fn insert(&mut self, ptr: Self::Address, key: K, value: V) -> Option<PropogateInsert<K, Self>> {
        // For now we just reconstruct the entire base layer as a vector and then rebuild
        // NOTE: This is incredibly inefficient, just wanted to get something working
        self.inner.arena.clear();
        let mut blind_base: Vec<Entry<K, V>> = vec![];
        let mut node_iter = self.inner.full_range();
        for node_id in node_iter {
            let Some(node) = self.inner.arena.get(node_id.value) else { continue; };
            blind_base.append(&mut node.data.clone());
        }
        match blind_base.binary_search_by(|a| {
            if a.key >= key {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }) {
            Ok(pos) => {
                blind_base[pos] = Entry::new(key, value); // Update
            }
            Err(pos) => blind_base.insert(pos, Entry::new(key, value)),
        };
        PGMSegmentation::make_segmentation(blind_base.into_iter(), &mut self.inner.arena);
        Some(PropogateInsert::Rebuild)
    }

    fn len(&self) -> usize {
        self.inner.arena.len()
    }

    fn memory_size(&self) -> usize {
        unimplemented!()
    }
}
