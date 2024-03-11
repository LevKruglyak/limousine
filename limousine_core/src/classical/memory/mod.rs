mod layer;

// use crate::BaseComponent;
// use crate::InternalComponent;
// use crate::Key;
// use crate::NodeLayer;
// use crate::TopComponent;
// use crate::Value;
// use layer::MemoryBTreeLayer;
// use std::borrow::Borrow;
// use std::collections::BTreeMap;
use crate::common::linked_list::Index;
use crate::common::macros::impl_node_layer;
use crate::component::*;
use crate::kv::StaticBounded;
use layer::*;
use std::ops::{Bound, RangeBounds};

// -------------------------------------------------------
//                  Internal Component
// -------------------------------------------------------

pub type BTreeInternalAddress = generational_arena::Index;

pub struct BTreeInternalComponent<K, X: 'static, const FANOUT: usize, BA, PA> {
    inner: MemoryBTreeLayer<K, BA, FANOUT, PA>,
    _ph: std::marker::PhantomData<X>,
}

impl<K, X, const FANOUT: usize, BA, PA> NodeLayer<K, BTreeInternalAddress, PA>
    for BTreeInternalComponent<K, X, FANOUT, BA, PA>
where
    K: StaticBounded,
    BA: Address,
    PA: Address,
{
    type Node =
        <MemoryBTreeLayer<K, BA, FANOUT, PA> as NodeLayer<K, BTreeInternalAddress, PA>>::Node;

    impl_node_layer!(Index);
}

impl<K, X, BA, PA, B: NodeLayer<K, BA, BTreeInternalAddress>, const FANOUT: usize>
    InternalComponent<K, B, BA, BTreeInternalAddress, PA>
    for BTreeInternalComponent<K, X, FANOUT, BA, PA>
where
    K: StaticBounded,
    BA: Address,
    PA: Address,
{
    fn search(&self, _: &B, ptr: BTreeInternalAddress, key: &K) -> BA {
        let node = self.inner.deref(ptr);

        node.inner.search_lub(key).clone()
    }

    fn insert<'n>(
        &'n mut self,
        base: &mut B,
        prop: PropogateInsert<K, BA, BTreeInternalAddress>,
    ) -> Option<PropogateInsert<K, BTreeInternalAddress, PA>> {
        match prop {
            PropogateInsert::Single(key, address, ptr) => self
                .inner
                .insert_with_parent(key, address, base, ptr)
                .map(|(key, address, parent)| PropogateInsert::Single(key, address, parent)),
            PropogateInsert::Replace { .. } => {
                unimplemented!()
                // self.inner
                //     .fill(base.range(Bound::Unbounded, Bound::Unbounded));
                //
                // Some(PropogateInsert::Rebuild)
            }
        }
    }
}

impl<K, X, BA, PA, B: NodeLayer<K, BA, BTreeInternalAddress>, const FANOUT: usize>
    InternalComponentInMemoryBuild<K, B, BA, BTreeInternalAddress, PA>
    for BTreeInternalComponent<K, X, FANOUT, BA, PA>
where
    K: StaticBounded,
    BA: Address,
    PA: Address,
{
    fn build(base: &mut B) -> Self {
        let mut result = MemoryBTreeLayer::empty();
        result.fill_with_parent(base);

        Self {
            inner: result,
            _ph: std::marker::PhantomData,
        }
    }
}

// -------------------------------------------------------
//                  Base Component
// -------------------------------------------------------

pub type BTreeBaseAddress = BTreeInternalAddress;

pub struct BTreeBaseComponent<K, V, const FANOUT: usize, PA> {
    inner: MemoryBTreeLayer<K, V, FANOUT, PA>,
}

impl<K, V, const FANOUT: usize, PA: 'static> NodeLayer<K, BTreeBaseAddress, PA>
    for BTreeBaseComponent<K, V, FANOUT, PA>
where
    K: StaticBounded,
    V: 'static,
    PA: Address,
{
    type Node =
        <MemoryBTreeLayer<K, V, FANOUT, PA> as NodeLayer<K, BTreeInternalAddress, PA>>::Node;

    impl_node_layer!(Index);
}

impl<K, V, const FANOUT: usize, PA: 'static> BaseComponent<K, V, BTreeBaseAddress, PA>
    for BTreeBaseComponent<K, V, FANOUT, PA>
where
    K: StaticBounded,
    V: 'static,
    PA: Address,
{
    fn insert<'n>(
        &'n mut self,
        ptr: BTreeInternalAddress,
        key: K,
        value: V,
    ) -> Option<PropogateInsert<K, BTreeBaseAddress, PA>> {
        if let Some((key, address, parent)) = self.inner.insert(key, value, ptr) {
            Some(PropogateInsert::Single(key, address, parent))
        } else {
            None
        }
    }

    fn search(&self, ptr: BTreeInternalAddress, key: &K) -> Option<&V> {
        let node = self.inner.deref(ptr);
        node.inner.search_exact(key)
    }
}

impl<K, V, const FANOUT: usize, PA> BaseComponentInMemoryBuild<K, V>
    for BTreeBaseComponent<K, V, FANOUT, PA>
where
    K: StaticBounded,
    V: 'static,
    PA: Address,
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
