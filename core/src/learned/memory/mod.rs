use num::PrimInt;

use crate::{
    common::list::memory::ArenaID, impl_node_layer, Address, InternalComponent, Key, NodeLayer,
    StaticBounded,
};

use self::layer::MemoryPGMLayer;

mod layer;

// -------------------------------------------------------
//                  Internal Component
// -------------------------------------------------------

pub type PGMInternalAddress = ArenaID;

pub struct PGMInternalComponent<K: Ord + Copy, X: 'static, const EPSILON: usize, BA: Copy, PA> {
    inner: MemoryPGMLayer<K, BA, EPSILON, PA>,
    _ph: std::marker::PhantomData<X>,
}

impl<K, X, const EPSILON: usize, BA, PA> NodeLayer<K, PGMInternalAddress, PA>
    for PGMInternalComponent<K, X, EPSILON, BA, PA>
where
    K: Clone + Copy + Ord + StaticBounded + PrimInt,
    BA: Address + Copy,
    PA: Address,
{
    impl_node_layer!(ArenaID, PA);
}

impl<K, X, BA, PA, B: NodeLayer<K, BA, PGMInternalAddress>, const EPSILON: usize>
    InternalComponent<K, B, BA, PGMInternalAddress, PA>
    for PGMInternalComponent<K, X, EPSILON, BA, PA>
where
    K: Key + PrimInt,
    BA: Address + Copy,
    PA: Address,
{
    fn search(&self, base: &B, ptr: PGMInternalAddress, key: &K) -> BA {
        unimplemented!()
    }

    fn insert(
        &mut self,
        base: &mut B,
        prop: crate::PropagateInsert<K, BA, PGMInternalAddress>,
    ) -> Option<crate::PropagateInsert<K, PGMInternalAddress, PA>> {
        unimplemented!()
    }

    fn build(base: &mut B) -> Self {
        unimplemented!()
    }
}
