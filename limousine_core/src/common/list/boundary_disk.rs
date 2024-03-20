use crate::common::storage::*;
use crate::node_layer::NodeLayer;
use crate::traits::KeyBounded;
use crate::traits::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Link {
    next: Option<StoreID>,
    prev: Option<StoreID>,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct BoundaryDiskListCatalogPage {
    first: StoreID,
    last: StoreID,

    // Maps node to next and previous links
    links: HashMap<StoreID, Link>,

    // Simple flag to mark this catalog initialized
    uninit: u8,
}

pub struct BoundaryDiskList<N, PA> {
    store: LocalStore<BoundaryDiskListCatalogPage>,

    // We should only persist parents when we are in a deep persisted layer, in a boundary layer we
    // keep them in transient memory
    parents: HashMap<StoreID, PA>,

    _ph: std::marker::PhantomData<N>,
}

impl<N, PA> BoundaryDiskList<N, PA>
where
    N: Serialize + for<'de> Deserialize<'de> + Default,
{
    pub fn load(store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self> {
        let mut store: LocalStore<BoundaryDiskListCatalogPage> = store.load_local_store(ident)?;
        let parents = HashMap::new();

        if store.catalog.uninit == 0 {
            let ptr = store.allocate_page();
            store.write_page(&N::default(), ptr)?;
            store.catalog.first = ptr;
            store.catalog.last = ptr;
        }

        Ok(Self {
            store,
            parents,
            _ph: std::marker::PhantomData,
        })
    }

    fn get_node(&self, ptr: StoreID) -> crate::Result<Option<N>> {
        self.store.read_page(ptr)
    }

    fn get_next(&self, ptr: StoreID) -> Option<StoreID> {
        self.store.catalog.links.get(&ptr).unwrap().next
    }

    pub fn insert_after(&mut self, inner: N, ptr: StoreID) -> crate::Result<StoreID> {
        let next_ptr = self.get_next(ptr);
        let new_link = Link {
            next: next_ptr,
            prev: Some(ptr),
        };

        let new_node_ptr = self.store.allocate_page();

        self.store.write_page(&inner, new_node_ptr)?;
        self.store.catalog.links.insert(new_node_ptr, new_link);
        self.store.catalog.links.get_mut(&ptr).unwrap().next = Some(new_node_ptr);

        if let Some(next_ptr) = next_ptr {
            self.store.catalog.links.get_mut(&next_ptr).unwrap().prev = Some(new_node_ptr);
        } else {
            self.store.catalog.last = new_node_ptr;
        }

        Ok(new_node_ptr)
    }
    //
    //     #[allow(unused)]
    //     pub fn insert_before(&mut self, inner: N, ptr: ArenaID) -> ArenaID {
    //         let previous_ptr = self.arena[ptr].0.previous;
    //
    //         let mut new_node = MemoryNode::new(inner);
    //         new_node.previous = previous_ptr;
    //         new_node.next = Some(ptr);
    //
    //         let new_node_ptr = self.arena.insert((new_node, None));
    //         self.arena[ptr].0.previous = Some(new_node_ptr);
    //
    //         if let Some(previous_ptr) = previous_ptr {
    //             self.arena[previous_ptr].0.next = Some(new_node_ptr);
    //         } else {
    //             self.first = new_node_ptr;
    //         }
    //
    //         new_node_ptr
    //     }
    //
    //     pub fn clear(&mut self, inner: N) -> ArenaID {
    //         self.arena.clear();
    //         let ptr = self.arena.insert((MemoryNode::new(inner), None));
    //
    //         self.first = ptr;
    //         self.last = ptr;
    //         ptr
    //     }
    //
    //     #[allow(unused)]
    //     pub fn len(&self) -> usize {
    //         self.arena.len()
    //     }
    // }
}

impl<K, N, PA> NodeLayer<K, StoreID, PA> for BoundaryDiskList<N, PA>
where
    K: Copy,
    N: KeyBounded<K> + Default + Serialize + for<'de> Deserialize<'de> + 'static,
    PA: Address,
{
    fn first(&self) -> StoreID {
        self.store.catalog.first
    }

    fn last(&self) -> StoreID {
        self.store.catalog.last
    }

    fn parent(&self, ptr: StoreID) -> Option<PA> {
        self.parents.get(&ptr).cloned()
    }

    fn set_parent(&mut self, ptr: StoreID, parent: PA) {
        self.parents.insert(ptr, parent);
    }

    fn lower_bound(&self, ptr: StoreID) -> K {
        *self.get_node(ptr).unwrap().unwrap().lower_bound()
    }

    fn next(&self, ptr: StoreID) -> Option<StoreID> {
        self.get_next(ptr)
    }
}
