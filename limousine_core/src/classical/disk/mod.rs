use crate::classical::node::BTreeNode;
use crate::common::bounded::*;
use crate::{Address, BaseComponentBuildDisk, IndexStore, NodeLayer, StoreId};

// -------------------------------------------------------
//                  Base Component
// -------------------------------------------------------

pub type BTreeBaseAddressDisk = StoreId;

pub struct BTreeBaseComponentDisk<K, V, const FANOUT: usize, PA> {
    store: IndexStore,
    _ph: std::marker::PhantomData<(K, V, PA)>,
}

impl<K, V, const FANOUT: usize, PA: 'static> NodeLayer<K, BTreeBaseAddressDisk, PA>
    for BTreeBaseComponentDisk<K, V, FANOUT, PA>
where
    K: StaticBounded,
    V: 'static,
    PA: Address,
{
    type Node = BTreeNode<K, V, FANOUT>;

    fn deref(&self, ptr: BTreeBaseAddressDisk) -> &Self::Node {
        todo!()
    }
}
//
// impl<K, V, const FANOUT: usize, PA: 'static> BaseComponent<K, V, BTreeBaseAddress, PA>
//     for BTreeBaseComponent<K, V, FANOUT, PA>
// where
//     K: StaticBounded,
//     V: 'static,
//     PA: Address,
// {
//     fn insert(
//         &mut self,
//         ptr: BTreeInternalAddress,
//         key: K,
//         value: V,
//     ) -> Option<PropogateInsert<K, BTreeBaseAddress, PA>> {
//         if let Some((key, address, parent)) = self.inner.insert(key, value, ptr) {
//             Some(PropogateInsert::Single(key, address, parent))
//         } else {
//             None
//         }
//     }
//
//     fn search(&self, ptr: BTreeInternalAddress, key: &K) -> Option<&V> {
//         let node = self.inner.deref(ptr);
//         node.inner.search_exact(key)
//     }
// }
//
impl<K, V, const FANOUT: usize, PA> BaseComponentBuildDisk<K, V>
    for BTreeBaseComponentDisk<K, V, FANOUT, PA>
where
    K: StaticBounded,
    V: 'static,
    PA: Address,
{
    fn load(store: IndexStore) -> Self {
        Self {
            store,
            _ph: std::marker::PhantomData,
        }
    }

    fn build(iter: impl Iterator<Item = (K, V)>, store: IndexStore) -> Self {
        Self {
            store,
            _ph: std::marker::PhantomData,
        }
    }
}
