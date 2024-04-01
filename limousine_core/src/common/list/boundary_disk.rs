use crate::common::storage::*;
use crate::node_layer::NodeLayer;
use crate::traits::KeyBounded;
use crate::traits::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Link {
    next: Option<StoreID>,
    prev: Option<StoreID>,
}

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum BoundaryDiskListState {
    #[default]
    Uninitialized,
    Initialized,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct BoundaryDiskListCatalogPage {
    first: StoreID,
    last: StoreID,

    // Maps node to next and previous links
    links: HashMap<StoreID, Link>,

    // Simple flag to mark the state of this list
    state: BoundaryDiskListState,
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
    N: Persisted + Default + Eq,
{
    pub fn load(store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self> {
        let mut store: LocalStore<BoundaryDiskListCatalogPage> = store.load_local_store(ident)?;
        let parents = HashMap::new();

        if store.catalog.state == BoundaryDiskListState::Uninitialized {
            store.catalog.state = BoundaryDiskListState::Initialized;
            let ptr = store.allocate_page();
            store.write_page(&N::default(), ptr)?;
            store.catalog.first = ptr;
            store.catalog.last = ptr;
            store.catalog.links.insert(ptr, Default::default());
        }

        Ok(Self {
            store,
            parents,
            _ph: std::marker::PhantomData,
        })
    }

    pub fn is_empty(&self) -> crate::Result<Option<StoreID>> {
        if self.store.catalog.first == self.store.catalog.last {
            if self.get_node(self.store.catalog.first)?.unwrap() == N::default() {
                return Ok(Some(self.store.catalog.first));
            }
        }

        return Ok(None);
    }

    pub fn transform_node<T>(
        &mut self,
        ptr: StoreID,
        closure: impl Fn(&mut N) -> T,
    ) -> crate::Result<T> {
        let mut node = self.get_node(ptr)?.unwrap();
        let result = closure(&mut node);
        self.store.write_page(&node, ptr)?;

        Ok(result)
    }

    pub fn get_node(&self, ptr: StoreID) -> crate::Result<Option<N>> {
        self.store.read_page(ptr)
    }

    fn get_next(&self, ptr: StoreID) -> Option<StoreID> {
        self.store.catalog.links.get(&ptr).unwrap().next
    }

    fn get_prev(&self, ptr: StoreID) -> Option<StoreID> {
        self.store.catalog.links.get(&ptr).unwrap().prev
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

    pub fn clear(&mut self) -> crate::Result<StoreID> {
        self.store.clear()?;

        let ptr = self.store.allocate_page();
        self.store.write_page(&N::default(), ptr)?;
        self.store.catalog.first = ptr;
        self.store.catalog.last = ptr;
        self.store.catalog.links.insert(ptr, Default::default());

        Ok(ptr)
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
    N: KeyBounded<K> + Persisted + Eq,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linked_list_new() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = GlobalStore::load(&dir).unwrap();
        let list: BoundaryDiskList<i32, ()> = BoundaryDiskList::load(&mut store, "test").unwrap();

        assert_eq!(
            list.get_node(list.first()).unwrap(),
            Some(Default::default())
        );
    }

    #[test]
    fn linked_list_insert_after() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = GlobalStore::load(&dir).unwrap();
        let mut list: BoundaryDiskList<u32, ()> =
            BoundaryDiskList::load(&mut store, "test").unwrap();

        let first_ptr = list.first();
        let second_ptr = list.insert_after(2, first_ptr).unwrap();

        assert_eq!(list.get_next(first_ptr), Some(second_ptr));
        assert_eq!(list.get_prev(second_ptr), Some(first_ptr));
        assert_eq!(list.last(), second_ptr);
    }
    //
    //     #[test]
    //     fn linked_list_insert_before() {
    //         let mut list: MemoryList<u32, ()> = MemoryList::new();
    //
    //         let first_ptr = list.first;
    //         let zero_ptr = list.insert_before(0, first_ptr);
    //
    //         assert_eq!(list.arena[first_ptr].0.previous, Some(zero_ptr));
    //         assert_eq!(list.arena[zero_ptr].0.next, Some(first_ptr));
    //         assert_eq!(list.first, zero_ptr);
    //     }
    //
    //     #[test]
    //     fn test_linked_list_clear() {
    //         let mut list: MemoryList<i32, ()> = MemoryList::new();
    //         list.insert_after(2, list.first);
    //
    //         assert_eq!(list.arena.len(), 2);
    //
    //         list.clear(5);
    //
    //         assert_eq!(list.len(), 1);
    //         assert_eq!(list[list.first], 5);
    //         assert_eq!(list.first, list.last);
    //     }
    //
    //     #[test]
    //     fn linked_node_new() {
    //         let node: MemoryNode<i32> = MemoryNode::new(10);
    //
    //         assert_eq!(node.inner, 10);
    //         assert_eq!(node.next, None);
    //         assert_eq!(node.previous, None);
    //     }
}
