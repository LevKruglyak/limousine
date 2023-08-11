mod layer;

use crate::BaseComponent;
use crate::InternalComponent;
use crate::Key;
use crate::NodeLayer;
use crate::TopComponent;
use crate::Value;
use layer::DiskBTreeLayer;
use std::borrow::Borrow;
use std::collections::BTreeMap;

// -------------------------------------------------------
//                  Internal Component
// -------------------------------------------------------

pub struct DiskBTreeInternalComponent<K: Key, B: NodeLayer<K>, const FANOUT: usize> {
    layer: DiskBTreeLayer<K, B::NodeRef, FANOUT>,
}

impl<K: Key, B: NodeLayer<K>, const FANOUT: usize> NodeLayer<K>
    for DiskBTreeInternalComponent<K, B, FANOUT>
{
    type Node = <DiskBTreeLayer<K, B::NodeRef, FANOUT> as NodeLayer<K>>::Node;
    type NodeRef = <DiskBTreeLayer<K, B::NodeRef, FANOUT> as NodeLayer<K>>::NodeRef;

    fn node_ref(&self, ptr: Self::NodeRef) -> &Self::Node {
        self.layer.node_ref(ptr)
    }

    type NodeIter<'n> = <DiskBTreeLayer<K, B::NodeRef, FANOUT> as NodeLayer<K>>::NodeIter<'n>;

    fn iter<'n>(&'n self) -> Self::NodeIter<'n> {
        self.layer.iter()
    }

    fn range<'n>(&'n self, lo_ptr: Self::NodeRef, hi_ptr: Self::NodeRef) -> Self::NodeIter<'n> {
        self.layer.range(lo_ptr, hi_ptr)
    }
}

impl<K: Key, B: NodeLayer<K>, const FANOUT: usize> InternalComponent<K, B>
    for DiskBTreeInternalComponent<K, B, FANOUT>
{
    fn new_internal(base: &B) -> Self {
        Self {
            layer: DiskBTreeLayer::new_internal(base),
        }
    }

    fn search_internal(&self, key: &K, ptr: Self::NodeRef) -> B::NodeRef {
        <DiskBTreeLayer<K, B::NodeRef, FANOUT> as InternalComponent<K, B>>::search_internal(
            &self.layer,
            key,
            ptr,
        )
    }

    fn insert_internal(
        &mut self,
        key: K,
        value: B::NodeRef,
        ptr: Self::NodeRef,
    ) -> Option<(K, Self::NodeRef)> {
        <DiskBTreeLayer<K, B::NodeRef, FANOUT> as InternalComponent<K, B>>::insert_internal(
            &mut self.layer,
            key,
            value,
            ptr,
        )
    }
}

// -------------------------------------------------------
//                  Base Component
// -------------------------------------------------------

pub struct DiskBTreeBaseComponent<K: Key, V: Value, const FANOUT: usize> {
    layer: DiskBTreeLayer<K, V, FANOUT>,
}

impl<K: Key, V: Value, const FANOUT: usize> NodeLayer<K> for DiskBTreeBaseComponent<K, V, FANOUT> {
    type Node = <DiskBTreeLayer<K, V, FANOUT> as NodeLayer<K>>::Node;
    type NodeRef = <DiskBTreeLayer<K, V, FANOUT> as NodeLayer<K>>::NodeRef;

    fn node_ref(&self, ptr: Self::NodeRef) -> &Self::Node {
        self.layer.node_ref(ptr)
    }

    type NodeIter<'n> = <DiskBTreeLayer<K, V, FANOUT> as NodeLayer<K>>::NodeIter<'n>;

    fn iter<'n>(&'n self) -> Self::NodeIter<'n> {
        self.layer.iter()
    }

    fn range<'n>(&'n self, lo_ptr: Self::NodeRef, hi_ptr: Self::NodeRef) -> Self::NodeIter<'n> {
        self.layer.range(lo_ptr, hi_ptr)
    }
}

impl<K: Key, V: Value, const FANOUT: usize> BaseComponent<K, V>
    for DiskBTreeBaseComponent<K, V, FANOUT>
{
    fn new_base() -> Self {
        Self {
            layer: DiskBTreeLayer::new_base(),
        }
    }

    fn search_base(&self, key: &K, ptr: Self::NodeRef) -> Option<&V> {
        <DiskBTreeLayer<K, V, FANOUT> as BaseComponent<K, V>>::search_base(&self.layer, key, ptr)
    }

    fn insert_base(&mut self, key: K, value: V, ptr: Self::NodeRef) -> Option<(K, Self::NodeRef)> {
        <DiskBTreeLayer<K, V, FANOUT> as BaseComponent<K, V>>::insert_base(
            &mut self.layer,
            key,
            value,
            ptr,
        )
    }
}
