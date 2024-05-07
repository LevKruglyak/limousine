use num::PrimInt;

use crate::{
    common::list::memory::ArenaID, impl_node_layer, Address, BaseComponent, InternalComponent, Key,
    NodeLayer, PropagateInsert, StaticBounded, Value,
};

use self::layer::MemoryPGMLayer;

mod layer;

// -------------------------------------------------------
//                  Internal Component
// -------------------------------------------------------

pub type PGMInternalAddress = ArenaID;

pub struct PGMInternalComponent<K: Ord, X: 'static, const EPSILON: usize, BA, PA> {
    inner: MemoryPGMLayer<K, BA, EPSILON, PA>,
    _ph: std::marker::PhantomData<X>,
}

impl<K, X, const EPSILON: usize, BA, PA> NodeLayer<K, PGMInternalAddress, PA>
    for PGMInternalComponent<K, X, EPSILON, BA, PA>
where
    K: Clone + Ord + StaticBounded + PrimInt,
    BA: Address,
    PA: Address,
{
    impl_node_layer!(ArenaID, PA);
}

impl<K, X, BA, PA, B: NodeLayer<K, BA, PGMInternalAddress>, const EPSILON: usize>
    InternalComponent<K, B, BA, PGMInternalAddress, PA>
    for PGMInternalComponent<K, X, EPSILON, BA, PA>
where
    K: Key + PrimInt,
    BA: Address,
    PA: Address,
{
    fn search(&self, _: &B, ptr: PGMInternalAddress, key: &K) -> BA {
        let node = &self.inner[ptr];
        node.search_pir(key).clone()
    }

    fn insert(
        &mut self,
        base: &mut B,
        prop: crate::PropagateInsert<K, BA, PGMInternalAddress>,
    ) -> Option<crate::PropagateInsert<K, PGMInternalAddress, PA>> {
        match prop {
            PropagateInsert::Single(key, address, ptr) => {
                let result = self.inner.insert_with_parent(key, address, base, ptr);
                result.map(|(key, address, parent)| PropagateInsert::Single(key, address, parent))
            }
            PropagateInsert::Replace(_, _) => {
                unimplemented!()
            }
        }
    }

    fn build(base: &mut B) -> Self {
        let mut result = MemoryPGMLayer::empty();
        result.fill_will_parent(base);

        Self {
            inner: result,
            _ph: std::marker::PhantomData,
        }
    }
}

// -------------------------------------------------------
//                  Base Component
// -------------------------------------------------------

pub type PGMBaseAddress = PGMInternalAddress;

pub struct PGMBaseComponent<K: Ord, V, const EPSILON: usize, PA> {
    inner: MemoryPGMLayer<K, V, EPSILON, PA>,
}

impl<K, V, const EPSILON: usize, PA: 'static> NodeLayer<K, PGMBaseAddress, PA>
    for PGMBaseComponent<K, V, EPSILON, PA>
where
    K: Key + PrimInt,
    V: Value,
    PA: Address,
{
    impl_node_layer!(ArenaID, PA);
}

impl<K, V, const EPSILON: usize, PA: 'static> BaseComponent<K, V, PGMBaseAddress, PA>
    for PGMBaseComponent<K, V, EPSILON, PA>
where
    K: Key + PrimInt,
    V: Value,
    PA: Address,
{
    fn insert(
        &mut self,
        ptr: PGMBaseAddress,
        key: K,
        value: V,
    ) -> Option<PropagateInsert<K, PGMBaseAddress, PA>> {
        if let Some((key, address, parent)) = self.inner.insert(key, value, ptr) {
            Some(PropagateInsert::Single(key, address, parent))
        } else {
            None
        }
    }

    fn search(&self, ptr: PGMBaseAddress, key: &K) -> Option<V> {
        self.inner[ptr].search_exact(key).cloned()
    }

    fn empty() -> Self {
        let result = MemoryPGMLayer::empty();

        Self { inner: result }
    }

    fn build(iter: impl Iterator<Item = (K, V)>) -> Self {
        let mut result = MemoryPGMLayer::empty();
        result.fill(iter);

        Self { inner: result }
    }
}
