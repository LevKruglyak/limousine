use crate::common::storage::*;
use crate::node_layer::NodeLayer;
use crate::traits::KeyBounded;
use crate::traits::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Link<PA> {
    next: Option<StoreID>,
    prev: Option<StoreID>,
    parent: Option<PA>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct DeepDiskListCatalogPage<PA> {
    first: StoreID,
    last: StoreID,

    // Maps node to next, previous, and parent links
    links: HashMap<StoreID, Link<PA>>,
}

pub struct DeepDiskList<N, PA>
where
    PA: DiskAddress,
{
    store: LocalStore<DeepDiskListCatalogPage<PA>>,
    _ph: std::marker::PhantomData<N>,
}

impl<N, PA> DeepDiskList<N, PA>
where
    N: Serialize + for<'de> Deserialize<'de>,
    PA: DiskAddress,
{
    pub fn load(store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self> {
        let store = store.load_local_store(ident)?;

        Ok(Self {
            store,
            _ph: std::marker::PhantomData,
        })
    }

    fn get_node(&self, ptr: StoreID) -> crate::Result<Option<N>> {
        self.store.read_page(ptr)
    }
}

impl<K, N, PA> NodeLayer<K, StoreID, PA> for DeepDiskList<N, PA>
where
    K: Copy,
    N: KeyBounded<K> + Serialize + for<'de> Deserialize<'de> + 'static,
    PA: DiskAddress,
{
    fn first(&self) -> StoreID {
        self.store.catalog.first
    }

    fn last(&self) -> StoreID {
        self.store.catalog.last
    }

    fn parent(&self, ptr: StoreID) -> Option<PA> {
        self.store.catalog.links.get(&ptr).unwrap().clone().parent
    }

    fn set_parent(&mut self, ptr: StoreID, parent: PA) {
        self.store.catalog.links.get_mut(&ptr).unwrap().parent = Some(parent);
    }

    fn lower_bound(&self, ptr: StoreID) -> K {
        *self.get_node(ptr).unwrap().unwrap().lower_bound()
    }

    fn next(&self, ptr: StoreID) -> Option<StoreID> {
        self.store.catalog.links.get(&ptr).unwrap().next
    }
}
