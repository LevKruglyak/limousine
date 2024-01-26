use crate::{
    common::macros::impl_node_layer, Address, BaseComponent, BaseComponentInMemoryBuild, InternalComponent,
    InternalComponentInMemoryBuild, Key, MemoryPGMLayer, NodeLayer, Value,
};

pub type PGMAddress = generational_arena::Index;

// INTERNAL COMPONENT
pub struct PGMInternalComponent<K: Key, X: 'static, const EPSILON: usize, BA: Value, PA> {
    inner: MemoryPGMLayer<K, BA, EPSILON, PA>,
    _ph: std::marker::PhantomData<X>,
}

impl<K: Key, X, const EPSILON: usize, BA, PA> NodeLayer<K, PGMAddress, PA>
    for PGMInternalComponent<K, X, EPSILON, BA, PA>
where
    BA: Address,
    PA: Address,
{
    type Node = <MemoryPGMLayer<K, BA, EPSILON, PA> as NodeLayer<K, PGMAddress, PA>>::Node;

    impl_node_layer!(PGMAddress);
}

impl<K, X, BA, PA, B: NodeLayer<K, BA, PGMAddress>, const EPSILON: usize> InternalComponent<K, B, BA, PGMAddress, PA>
    for PGMInternalComponent<K, X, EPSILON, BA, PA>
where
    K: Key,
    BA: Address,
    PA: Address,
{
    fn search(&self, base: &B, ptr: PGMAddress, key: &K) -> BA {
        let node = self.inner.deref(ptr);

        node.inner.search_lub(key).clone()
    }

    fn insert(
        &mut self,
        base: &mut B,
        prop: crate::PropogateInsert<K, BA, PGMAddress>,
    ) -> Option<crate::PropogateInsert<K, PGMAddress, PA>> {
        panic!("Unimplemented!")
    }
}

impl<K, X, BA, PA, B: NodeLayer<K, BA, PGMAddress>, const EPSILON: usize>
    InternalComponentInMemoryBuild<K, B, BA, PGMAddress, PA> for PGMInternalComponent<K, X, EPSILON, BA, PA>
where
    K: Key,
    BA: Address,
    PA: Address,
{
    fn build(base: &mut B) -> Self {
        let mut layer: MemoryPGMLayer<K, BA, EPSILON, PA> = MemoryPGMLayer::new();
        layer.fill_from_beneath(base);
        Self {
            inner: layer,
            _ph: std::marker::PhantomData,
        }
    }
}

// BASE COMPONENT
pub struct PGMBaseCopmonent<K: Key, V: Value, const EPSILON: usize, PA> {
    inner: MemoryPGMLayer<K, V, EPSILON, PA>,
}

impl<K, V, const EPSILON: usize, PA: 'static> NodeLayer<K, PGMAddress, PA> for PGMBaseCopmonent<K, V, EPSILON, PA>
where
    K: Key,
    V: Value,
    PA: Address,
{
    type Node = <MemoryPGMLayer<K, V, EPSILON, PA> as NodeLayer<K, PGMAddress, PA>>::Node;

    impl_node_layer!(PGMAddress);
}

impl<K, V, const EPSILON: usize, PA: 'static> BaseComponent<K, V, PGMAddress, PA>
    for PGMBaseCopmonent<K, V, EPSILON, PA>
where
    K: Key,
    V: Value,
    PA: Address,
{
    fn search(&self, ptr: PGMAddress, key: &K) -> Option<&V> {
        let node = self.deref(ptr);
        node.inner.search_exact(key).clone()
    }

    fn insert(&mut self, ptr: PGMAddress, key: K, value: V) -> Option<crate::PropogateInsert<K, PGMAddress, PA>> {
        panic!("Unimplemented");
    }
}

impl<K, V, const EPSILON: usize, PA> BaseComponentInMemoryBuild<K, V> for PGMBaseCopmonent<K, V, EPSILON, PA>
where
    K: Key,
    V: Value,
    PA: Address,
{
    fn empty() -> Self {
        Self {
            inner: MemoryPGMLayer::new(),
        }
    }

    fn build(iter: impl Iterator<Item = crate::Entry<K, V>>) -> Self {
        let mut layer: MemoryPGMLayer<K, V, EPSILON, PA> = MemoryPGMLayer::new();
        layer.fill(iter);
        Self { inner: layer }
    }
}
