mod layer;

use crate::common::list::memory::ArenaID;
use crate::component::*;
use crate::node_layer::{impl_node_layer, NodeLayer};
use crate::traits::{Address, StaticBounded};
use layer::*;

// -------------------------------------------------------
//                  Internal Component
// -------------------------------------------------------

pub type BTreeInternalAddress = ArenaID;

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
    impl_node_layer!(ArenaID, PA);
}

impl<K, X, BA, PA, B: NodeLayer<K, BA, BTreeInternalAddress>, const FANOUT: usize>
    InternalComponent<K, B, BA, BTreeInternalAddress, PA>
    for BTreeInternalComponent<K, X, FANOUT, BA, PA>
where
    K: StaticBounded,
    BA: Address,
    PA: Address,
{
    fn search(&self, _: &B, ptr: BTreeInternalAddress, key: K) -> BA {
        self.inner[ptr].get_lower_bound_always(&key).clone()
    }

    fn insert(
        &mut self,
        base: &mut B,
        prop: PropagateInsert<K, BA, BTreeInternalAddress>,
    ) -> Option<PropagateInsert<K, BTreeInternalAddress, PA>> {
        match prop {
            PropagateInsert::Single(key, address, ptr) => self
                .inner
                .insert_with_parent(key, address, base, ptr)
                .map(|(key, address, parent)| PropagateInsert::Single(key, address, parent)),
            PropagateInsert::Replace { .. } => {
                unimplemented!()
            }
        }
    }

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
    impl_node_layer!(ArenaID, PA);
}

impl<K, V, const FANOUT: usize, PA: 'static> BaseComponent<K, V, BTreeBaseAddress, PA>
    for BTreeBaseComponent<K, V, FANOUT, PA>
where
    K: StaticBounded,
    V: 'static + Clone,
    PA: Address,
{
    fn insert(
        &mut self,
        ptr: BTreeInternalAddress,
        key: K,
        value: V,
    ) -> Option<PropagateInsert<K, BTreeBaseAddress, PA>> {
        if let Some((key, address, parent)) = self.inner.insert(key, value, ptr) {
            Some(PropagateInsert::Single(key, address, parent))
        } else {
            None
        }
    }

    fn search(&self, ptr: BTreeInternalAddress, key: K) -> Option<V> {
        self.inner[ptr].get_exact(&key).cloned()
    }

    fn empty() -> Self {
        let result = MemoryBTreeLayer::empty();

        Self { inner: result }
    }

    fn build(iter: impl Iterator<Item = (K, V)>) -> Self {
        let mut result = MemoryBTreeLayer::empty();
        result.fill(iter);

        Self { inner: result }
    }
}
