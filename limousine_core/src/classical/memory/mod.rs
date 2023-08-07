mod layer;

use crate::BaseComponent;
use crate::InternalComponent;
use crate::Key;
use crate::NodeLayer;
use crate::TopComponent;
use crate::Value;
use layer::BTreeLayer;
use std::borrow::Borrow;
use std::collections::BTreeMap;

// -------------------------------------------------------
//                  Internal Component
// -------------------------------------------------------

pub struct BTreeInternalComponent<K: Key, B: NodeLayer<K>, const FANOUT: usize> {
    layer: BTreeLayer<K, B::NodeRef, FANOUT>,
}

impl<K: Key, B: NodeLayer<K>, const FANOUT: usize> NodeLayer<K>
    for BTreeInternalComponent<K, B, FANOUT>
{
    type Node = <BTreeLayer<K, B::NodeRef, FANOUT> as NodeLayer<K>>::Node;
    type NodeRef = <BTreeLayer<K, B::NodeRef, FANOUT> as NodeLayer<K>>::NodeRef;

    fn node_ref(&self, ptr: Self::NodeRef) -> &Self::Node {
        self.layer.node_ref(ptr)
    }

    type NodeIter<'n> = <BTreeLayer<K, B::NodeRef, FANOUT> as NodeLayer<K>>::NodeIter<'n>;

    fn iter<'n>(&'n self) -> Self::NodeIter<'n> {
        self.layer.iter()
    }
}

impl<K: Key, B: NodeLayer<K>, const FANOUT: usize> InternalComponent<K, B>
    for BTreeInternalComponent<K, B, FANOUT>
{
    fn new_internal(base: &B) -> Self {
        Self {
            layer: BTreeLayer::new_internal(base),
        }
    }

    fn search_internal(&self, key: &K, ptr: Self::NodeRef) -> B::NodeRef {
        <BTreeLayer<K, B::NodeRef, FANOUT> as InternalComponent<K, B>>::search_internal(
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
        <BTreeLayer<K, B::NodeRef, FANOUT> as InternalComponent<K, B>>::insert_internal(
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

pub struct BTreeBaseComponent<K: Key, V: Value, const FANOUT: usize> {
    layer: BTreeLayer<K, V, FANOUT>,
}

impl<K: Key, V: Value, const FANOUT: usize> NodeLayer<K> for BTreeBaseComponent<K, V, FANOUT> {
    type Node = <BTreeLayer<K, V, FANOUT> as NodeLayer<K>>::Node;
    type NodeRef = <BTreeLayer<K, V, FANOUT> as NodeLayer<K>>::NodeRef;

    fn node_ref(&self, ptr: Self::NodeRef) -> &Self::Node {
        self.layer.node_ref(ptr)
    }

    type NodeIter<'n> = <BTreeLayer<K, V, FANOUT> as NodeLayer<K>>::NodeIter<'n>;

    fn iter<'n>(&'n self) -> Self::NodeIter<'n> {
        self.layer.iter()
    }
}

impl<K: Key, V: Value, const FANOUT: usize> BaseComponent<K, V>
    for BTreeBaseComponent<K, V, FANOUT>
{
    fn new_base() -> Self {
        Self {
            layer: BTreeLayer::new_base(),
        }
    }

    fn search_base(&self, key: &K, ptr: Self::NodeRef) -> Option<&V> {
        <BTreeLayer<K, V, FANOUT> as BaseComponent<K, V>>::search_base(&self.layer, key, ptr)
    }

    fn insert_base(&mut self, key: K, value: V, ptr: Self::NodeRef) -> Option<(K, Self::NodeRef)> {
        <BTreeLayer<K, V, FANOUT> as BaseComponent<K, V>>::insert_base(
            &mut self.layer,
            key,
            value,
            ptr,
        )
    }
}
