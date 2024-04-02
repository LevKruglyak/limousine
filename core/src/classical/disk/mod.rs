use crate::{
    common::storage::{GlobalStore, StoreID},
    impl_node_layer, Address, BoundaryDiskBaseComponent, BoundaryDiskInternalComponent,
    DeepDiskBaseComponent, DeepDiskInternalComponent, Key, NodeLayer, Persisted, PropagateInsert,
    Value,
};

use self::boundary_layer::BoundaryDiskBTreeLayer;
use self::deep_layer::DeepDiskBTreeLayer;

mod boundary_layer;
mod deep_layer;

// -------------------------------------------------------
//                 Boundary Internal Component
// -------------------------------------------------------

pub type BoundaryDiskBTreeInternalAddress = StoreID;

pub struct BoundaryDiskBTreeInternalComponent<K, X: 'static, const FANOUT: usize, BA, PA> {
    pub inner: BoundaryDiskBTreeLayer<K, BA, FANOUT, PA>,
    _ph: std::marker::PhantomData<X>,
}

impl<K, X, const FANOUT: usize, BA, PA> NodeLayer<K, BoundaryDiskBTreeInternalAddress, PA>
    for BoundaryDiskBTreeInternalComponent<K, X, FANOUT, BA, PA>
where
    K: Key + Persisted,
    BA: Persisted + Address,
    PA: Address,
{
    impl_node_layer!(StoreID, PA);
}

impl<K, X, BA, PA, B: NodeLayer<K, BA, BoundaryDiskBTreeInternalAddress>, const FANOUT: usize>
    BoundaryDiskInternalComponent<K, B, BA, BoundaryDiskBTreeInternalAddress, PA>
    for BoundaryDiskBTreeInternalComponent<K, X, FANOUT, BA, PA>
where
    K: Persisted + Key,
    BA: Persisted + Address,
    PA: Address,
{
    fn search(&self, _: &B, ptr: BoundaryDiskBTreeInternalAddress, key: K) -> crate::Result<BA> {
        Ok(self.inner.get_node(ptr)?.search_lub(&key).clone())
    }

    fn insert(
        &mut self,
        base: &mut B,
        prop: PropagateInsert<K, BA, BoundaryDiskBTreeInternalAddress>,
    ) -> crate::Result<Option<PropagateInsert<K, BoundaryDiskBTreeInternalAddress, PA>>> {
        Ok(match prop {
            PropagateInsert::Single(key, address, ptr) => self
                .inner
                .insert_with_parent(key, address, base, ptr)?
                .map(|(key, address, parent)| PropagateInsert::Single(key, address, parent)),
            PropagateInsert::Replace { .. } => {
                unimplemented!()
            }
        })
    }

    fn load(base: &mut B, store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self> {
        let mut result = BoundaryDiskBTreeLayer::load(store, ident)?;
        result.fill_with_parent(base)?;

        Ok(Self {
            inner: result,
            _ph: std::marker::PhantomData,
        })
    }
}

// -------------------------------------------------------
//                 Boundary Base Component
// -------------------------------------------------------

pub type BoundaryDiskBTreeBaseAddress = StoreID;

pub struct BoundaryDiskBTreeBaseComponent<K, V, const FANOUT: usize, PA> {
    pub inner: BoundaryDiskBTreeLayer<K, V, FANOUT, PA>,
}

impl<K, V, const FANOUT: usize, PA: 'static> NodeLayer<K, BoundaryDiskBTreeBaseAddress, PA>
    for BoundaryDiskBTreeBaseComponent<K, V, FANOUT, PA>
where
    K: Key + Persisted,
    V: Persisted,
    PA: Address,
{
    impl_node_layer!(StoreID, PA);
}

impl<K, V, const FANOUT: usize, PA: 'static>
    BoundaryDiskBaseComponent<K, V, BoundaryDiskBTreeBaseAddress, PA>
    for BoundaryDiskBTreeBaseComponent<K, V, FANOUT, PA>
where
    K: Persisted + Key,
    V: Persisted + Value,
    PA: Address,
{
    fn insert(
        &mut self,
        ptr: BoundaryDiskBTreeInternalAddress,
        key: K,
        value: V,
    ) -> crate::Result<Option<PropagateInsert<K, BoundaryDiskBTreeBaseAddress, PA>>> {
        if let Some((key, address, parent)) = self.inner.insert(key, value, ptr)? {
            Ok(Some(PropagateInsert::Single(key, address, parent)))
        } else {
            Ok(None)
        }
    }

    fn search(&self, ptr: BoundaryDiskBTreeInternalAddress, key: K) -> crate::Result<Option<V>> {
        Ok(self.inner.get_node(ptr)?.search_exact(&key).cloned())
    }

    fn load(store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self> {
        Ok(Self {
            inner: BoundaryDiskBTreeLayer::load(store, ident)?,
        })
    }
}

// -------------------------------------------------------
//                 Deep disk Internal Component
// -------------------------------------------------------

pub type DeepDiskBTreeInternalAddress = StoreID;

pub struct DeepDiskBTreeInternalComponent<K, X: 'static, const FANOUT: usize, BA, PA>
where
    PA: Persisted + Address,
{
    pub inner: DeepDiskBTreeLayer<K, BA, FANOUT, PA>,
    _ph: std::marker::PhantomData<X>,
}

impl<K, X, const FANOUT: usize, BA, PA> NodeLayer<K, DeepDiskBTreeInternalAddress, PA>
    for DeepDiskBTreeInternalComponent<K, X, FANOUT, BA, PA>
where
    K: Key + Persisted,
    BA: Persisted + Address,
    PA: Persisted + Address,
{
    impl_node_layer!(StoreID, PA);
}

impl<K, X, BA, PA, B: NodeLayer<K, BA, DeepDiskBTreeInternalAddress>, const FANOUT: usize>
    DeepDiskInternalComponent<K, B, BA, DeepDiskBTreeInternalAddress, PA>
    for DeepDiskBTreeInternalComponent<K, X, FANOUT, BA, PA>
where
    K: Persisted + Key,
    BA: Persisted + Address,
    PA: Persisted + Address,
{
    fn search(&self, _: &B, ptr: DeepDiskBTreeInternalAddress, key: K) -> crate::Result<BA> {
        Ok(self.inner.get_node(ptr)?.search_lub(&key).clone())
    }

    fn insert(
        &mut self,
        base: &mut B,
        prop: PropagateInsert<K, BA, DeepDiskBTreeInternalAddress>,
    ) -> crate::Result<Option<PropagateInsert<K, DeepDiskBTreeInternalAddress, PA>>> {
        Ok(match prop {
            PropagateInsert::Single(key, address, ptr) => self
                .inner
                .insert_with_parent(key, address, base, ptr)?
                .map(|(key, address, parent)| PropagateInsert::Single(key, address, parent)),
            PropagateInsert::Replace { .. } => {
                unimplemented!()
            }
        })
    }

    fn load(base: &mut B, store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self> {
        let mut result = DeepDiskBTreeLayer::load(store, ident)?;
        result.fill_with_parent(base)?;

        Ok(Self {
            inner: result,
            _ph: std::marker::PhantomData,
        })
    }
}

// -------------------------------------------------------
//                 Deep Base Component
// -------------------------------------------------------

pub type DeepDiskBTreeBaseAddress = StoreID;

pub struct DeepDiskBTreeBaseComponent<K, V, const FANOUT: usize, PA>
where
    PA: Persisted + Address,
{
    pub inner: DeepDiskBTreeLayer<K, V, FANOUT, PA>,
}

impl<K, V, const FANOUT: usize, PA: 'static> NodeLayer<K, DeepDiskBTreeBaseAddress, PA>
    for DeepDiskBTreeBaseComponent<K, V, FANOUT, PA>
where
    K: Key + Persisted,
    V: Persisted,
    PA: Persisted + Address,
{
    impl_node_layer!(StoreID, PA);
}

impl<K, V, const FANOUT: usize, PA: 'static>
    DeepDiskBaseComponent<K, V, BoundaryDiskBTreeBaseAddress, PA>
    for DeepDiskBTreeBaseComponent<K, V, FANOUT, PA>
where
    K: Persisted + Key,
    V: Persisted + Value,
    PA: Persisted + Address,
{
    fn insert(
        &mut self,
        ptr: BoundaryDiskBTreeInternalAddress,
        key: K,
        value: V,
    ) -> crate::Result<Option<PropagateInsert<K, BoundaryDiskBTreeBaseAddress, PA>>> {
        if let Some((key, address, parent)) = self.inner.insert(key, value, ptr)? {
            Ok(Some(PropagateInsert::Single(key, address, parent)))
        } else {
            Ok(None)
        }
    }

    fn search(&self, ptr: BoundaryDiskBTreeInternalAddress, key: K) -> crate::Result<Option<V>> {
        Ok(self.inner.get_node(ptr)?.search_exact(&key).cloned())
    }

    fn load(store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self> {
        Ok(Self {
            inner: DeepDiskBTreeLayer::load(store, ident)?,
        })
    }
}
