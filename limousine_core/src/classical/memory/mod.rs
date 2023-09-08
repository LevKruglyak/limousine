mod layer;

// use crate::BaseComponent;
// use crate::InternalComponent;
// use crate::Key;
// use crate::NodeLayer;
// use crate::TopComponent;
// use crate::Value;
// use layer::MemoryBTreeLayer;
// use std::borrow::Borrow;
use crate::common::address::Address;
// use std::collections::BTreeMap;
use crate::component::*;
use crate::kv::StaticBounded;
use layer::*;
use std::ops::{Bound, RangeBounds};

// -------------------------------------------------------
//                  Internal Component
// -------------------------------------------------------

pub struct BTreeInternalComponent<K, X: 'static, const FANOUT: usize> {
    inner: MemoryBTreeLayer<K, Address, FANOUT>,
    _ph: std::marker::PhantomData<X>,
}

impl<K, X, const FANOUT: usize> NodeLayer<K> for BTreeInternalComponent<K, X, FANOUT>
where
    K: StaticBounded,
{
    type Node = <MemoryBTreeLayer<K, Address, FANOUT> as NodeLayer<K>>::Node;

    fn deref(&self, ptr: Address) -> &Self::Node {
        self.inner.deref(ptr)
    }

    fn deref_mut(&mut self, ptr: Address) -> &mut Self::Node {
        self.inner.deref_mut(ptr)
    }

    fn first(&self) -> Address {
        self.inner.first()
    }
}

impl<K, X, B: NodeLayer<K>, const FANOUT: usize> InternalComponent<K, B>
    for BTreeInternalComponent<K, X, FANOUT>
where
    K: StaticBounded,
{
    fn search(&self, _: &B, ptr: Address, key: &K) -> Address {
        let node = self.inner.deref(ptr);

        node.inner.search_lub(key).clone()
    }

    fn insert<'n>(
        &'n mut self,
        base: &B,
        ptr: Address,
        prop: PropogateInsert<K>,
    ) -> Option<PropogateInsert<K>> {
        match prop {
            PropogateInsert::Single(key, address) => self
                .inner
                .insert(key, address, ptr)
                .map(|(key, address)| PropogateInsert::Single(key, address)),
            PropogateInsert::Rebuild => {
                self.inner
                    .fill(base.range(Bound::Unbounded, Bound::Unbounded));

                Some(PropogateInsert::Rebuild)
            }
        }
    }
}

impl<K, X, B: NodeLayer<K>, const FANOUT: usize> InternalComponentInMemoryBuild<K, B>
    for BTreeInternalComponent<K, X, FANOUT>
where
    K: StaticBounded,
{
    fn build(base: &B) -> Self {
        let mut result = MemoryBTreeLayer::empty();
        result.fill(base.range(Bound::Unbounded, Bound::Unbounded));

        Self {
            inner: result,
            _ph: std::marker::PhantomData,
        }
    }
}

// -------------------------------------------------------
//                  Base Component
// -------------------------------------------------------

pub struct BTreeBaseComponent<K, V, const FANOUT: usize> {
    inner: MemoryBTreeLayer<K, V, FANOUT>,
}

impl<K, V, const FANOUT: usize> NodeLayer<K> for BTreeBaseComponent<K, V, FANOUT>
where
    K: StaticBounded,
    V: 'static,
{
    type Node = <MemoryBTreeLayer<K, V, FANOUT> as NodeLayer<K>>::Node;

    fn deref(&self, ptr: Address) -> &Self::Node {
        self.inner.deref(ptr)
    }

    fn deref_mut(&mut self, ptr: Address) -> &mut Self::Node {
        self.inner.deref_mut(ptr)
    }

    fn first(&self) -> Address {
        self.inner.first()
    }
}

impl<K, V, const FANOUT: usize> BaseComponent<K, V, Self> for BTreeBaseComponent<K, V, FANOUT>
where
    K: StaticBounded,
    V: 'static,
{
    fn insert<'n>(&'n mut self, ptr: Address, key: K, value: V) -> Option<PropogateInsert<K>> {
        if let Some((key, address)) = self.inner.insert(key, value, ptr) {
            Some(PropogateInsert::Single(key, address))
        } else {
            None
        }
    }

    fn search(&self, ptr: Address, key: &K) -> Option<&V> {
        let node = self.inner.deref(ptr);
        node.inner.search_exact(key)
    }
}

impl<K, V, const FANOUT: usize> BaseComponentInMemoryBuild<K, V>
    for BTreeBaseComponent<K, V, FANOUT>
where
    K: StaticBounded,
    V: 'static,
{
    fn empty() -> Self {
        let mut result = MemoryBTreeLayer::empty();
        result.add_node(MemoryBTreeNode::empty());

        Self { inner: result }
    }

    fn build(iter: impl Iterator<Item = (K, V)>) -> Self {
        let mut result = MemoryBTreeLayer::empty();
        result.fill(iter);

        Self { inner: result }
    }
}
