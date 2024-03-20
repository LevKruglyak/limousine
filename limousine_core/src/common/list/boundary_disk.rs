use crate::node_layer::NodeLayer;
use crate::traits::KeyBounded;
use crate::traits::*;
use crate::{common::storage::*, node_layer::Node};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct BoundaryDiskNode<N> {
    pub inner: N,
    next: StoreID,
    previous: StoreID,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct BoundaryDiskListCatalogPage {}

pub struct BoundaryDiskList<N, PA> {
    store: LocalStore<BoundaryDiskListCatalogPage>,

    // We should only persist parents when we are in a deep persisted layer, in a boundary layer we
    // keep them in transient memory
    parents: HashMap<StoreID, PA>,

    _ph: std::marker::PhantomData<N>,
}

impl<N, PA> BoundaryDiskList<N, PA>
where
    N: Serialize + for<'de> Deserialize<'de>,
{
    pub fn load(store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self> {
        let store = store.load_local_store(ident)?;
        let parents = HashMap::new();

        Ok(Self {
            store,
            parents,
            _ph: std::marker::PhantomData,
        })
    }

    fn read_node(&self, ptr: StoreID) -> crate::Result<Option<N>> {
        self.store.read_page(ptr)
    }
}

impl<K, N, PA> NodeLayer<K, StoreID, PA> for BoundaryDiskList<N, PA>
where
    K: Copy,
    N: KeyBounded<K> + 'static,
    PA: Address,
{
    fn first(&self) -> StoreID {
        // self.first
        unimplemented!()
    }

    fn last(&self) -> StoreID {
        // self.last
        unimplemented!()
    }

    fn parent(&self, ptr: StoreID) -> Option<PA> {
        unimplemented!()
    }

    fn set_parent(&mut self, ptr: StoreID, parent: PA) {
        unimplemented!()
    }

    fn lower_bound(&self, ptr: StoreID) -> K {
        unimplemented!()
    }

    fn next(&self, ptr: StoreID) -> Option<StoreID> {
        unimplemented!()
    }
}

impl<K, N> KeyBounded<K> for BoundaryDiskNode<N>
where
    N: KeyBounded<K>,
{
    fn lower_bound(&self) -> &K {
        self.inner.lower_bound()
    }
}

impl<N> AsRef<BoundaryDiskNode<N>> for &BoundaryDiskNode<N> {
    fn as_ref(&self) -> &BoundaryDiskNode<N> {
        self
    }
}

impl<K, N> Node<K, StoreID> for BoundaryDiskNode<N>
where
    N: KeyBounded<K> + 'static,
{
    fn next(&self) -> Option<StoreID> {
        self.next.to_option()
    }

    fn previous(&self) -> Option<StoreID> {
        self.previous.to_option()
    }
}

impl<N> BoundaryDiskNode<N> {
    fn new(node: N) -> Self {
        Self {
            inner: node,
            next: StoreID::none(),
            previous: StoreID::none(),
        }
    }
}

// /// Implementation for deep persistence layer
// impl<N, PA> DeepDiskLayer<N, PA>
// where
//     for<'de> PA: Serialize + Deserialize<'de>,
// {
//     fn load(store: IndexStore) -> Self {
//         unimplemented!()
//     }
// }
//
// impl<N, PA> DeepDiskNode<N, PA> {
//     fn new(node: N) -> Self {
//         Self {
//             inner: node,
//             next: STORE_ID_NONE,
//             previous: STORE_ID_NONE,
//             parent: None,
//         }
//     }
// }
// // // In the case that parents can be persisted as well, i.e. we
// // // are in a deep persisted layer, this is serialized/deserialized
// // // at startup/shutdown respectively
// // parents: HashMap<StoreId, PA>,
//
// // pub struct DiskList<N, PA> {
// //     store: TypedIndexStore<DiskListCatalogPage<PA>>,
// //     _ph: std::marker::PhantomData<N>,
// // }
// //
//
// // impl<N, PA> MemoryList<N, PA> {
// //     pub fn empty() -> Self
// //     where
// //         N: Default,
// //     {
// //         let mut arena = Arena::new();
// //         let ptr = arena.insert(MemoryNode::new(Default::default()));
// //
// //         MemoryList {
// //             arena,
// //             first: ptr,
// //             last: ptr,
// //         }
// //     }
// //
// //     pub fn insert_after(&mut self, inner: N, ptr: Index) -> Index {
// //         let next_ptr = self.arena[ptr].next;
// //
// //         let mut new_node = MemoryNode::new(inner);
// //         new_node.previous = Some(ptr);
// //         new_node.next = next_ptr;
// //
// //         let new_node_ptr = self.arena.insert(new_node);
// //         self.arena[ptr].next = Some(new_node_ptr);
// //
// //         if let Some(next_ptr) = next_ptr {
// //             self.arena[next_ptr].previous = Some(new_node_ptr);
// //         } else {
// //             self.last = new_node_ptr;
// //         }
// //
// //         new_node_ptr
// //     }
// //
// //     pub fn insert_before(&mut self, inner: N, ptr: Index) -> Index {
// //         let previous_ptr = self.arena[ptr].previous;
// //
// //         let mut new_node = MemoryNode::new(inner);
// //         new_node.previous = previous_ptr;
// //         new_node.next = Some(ptr);
// //
// //         let new_node_ptr = self.arena.insert(new_node);
// //         self.arena[ptr].previous = Some(new_node_ptr);
// //
// //         if let Some(previous_ptr) = previous_ptr {
// //             self.arena[previous_ptr].next = Some(new_node_ptr);
// //         } else {
// //             self.first = new_node_ptr;
// //         }
// //
// //         new_node_ptr
// //     }
// //
// //     pub fn clear(&mut self, inner: N) -> Index {
// //         self.arena.clear();
// //         let ptr = self.arena.insert(MemoryNode::new(inner));
// //
// //         self.first = ptr;
// //         self.last = ptr;
// //         ptr
// //     }
// //
// //     pub fn parent(&self, ptr: Index) -> Option<PA>
// //     where
// //         PA: Address,
// //     {
// //         self.arena[ptr].parent.clone()
// //     }
// //
// //     pub fn len(&self) -> usize {
// //         self.arena.len()
// //     }
// // }
// //
// // // ----------------------------------------
// // // Common implementations
// // // ----------------------------------------
// //
// // impl<K, N, PA> KeyBounded<K> for MemoryNode<N, PA>
// // where
// //     N: KeyBounded<K>,
// // {
// //     fn lower_bound(&self) -> &K {
// //         self.inner.lower_bound()
// //     }
// // }
// //
// // impl<K, N, PA> Model<K, Index, PA> for MemoryNode<N, PA>
// // where
// //     N: KeyBounded<K> + 'static,
// //     PA: Address,
// // {
// //     fn next(&self) -> Option<Index> {
// //         self.next
// //     }
// //
// //     fn previous(&self) -> Option<Index> {
// //         self.previous
// //     }
// //
// //     fn parent(&self) -> Option<PA> {
// //         self.parent.clone()
// //     }
// //
// //     fn set_parent(&mut self, parent: PA) {
// //         self.parent = Some(parent);
// //     }
// // }
// //
// // impl<N, PA> std::ops::Index<Index> for MemoryList<N, PA> {
// //     type Output = N;
// //
// //     fn index(&self, index: Index) -> &Self::Output {
// //         &self.arena[index].inner
// //     }
// // }
// //
// // impl<N, PA> std::ops::IndexMut<Index> for MemoryList<N, PA> {
// //     fn index_mut(&mut self, index: Index) -> &mut Self::Output {
// //         &mut self.arena[index].inner
// //     }
// // }
// //
// // impl<K, N, PA> NodeLayer<K, Index, PA> for MemoryList<N, PA>
// // where
// //     K: Copy,
// //     N: KeyBounded<K> + 'static,
// //     PA: Address,
// // {
// //     type Node = MemoryNode<N, PA>;
// //
// //     fn deref(&self, ptr: Index) -> &Self::Node {
// //         &self.arena[ptr]
// //     }
// //
// //     fn deref_mut(&mut self, ptr: Index) -> &mut Self::Node {
// //         &mut self.arena[ptr]
// //     }
// //
// //     unsafe fn deref_unsafe(&self, ptr: Index) -> *mut Self::Node {
// //         self.arena.get(ptr).unwrap() as *const Self::Node as *mut Self::Node
// //     }
// //
// //     fn first(&self) -> Index {
// //         self.first
// //     }
// //
// //     fn last(&self) -> Index {
// //         self.last
// //     }
// // }
//
// #[cfg(test)]
// mod tests {}
