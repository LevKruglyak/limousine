use serde::{Deserialize, Serialize};

mod cached_store;
mod id_alloc;
mod stable_cache;
mod store;

pub use cached_store::CachedLocalStore;
pub use id_alloc::ID;
pub use store::GlobalStore;
pub use store::LocalStore;

pub type StoreID = u64;

pub trait ObjectStore {
    fn allocate_page(&mut self) -> StoreID;

    fn free_page(&mut self, id: StoreID) -> bool;

    fn write_page<P>(&self, page: &P, id: StoreID) -> crate::Result<()>
    where
        P: Serialize;

    fn read_page<P>(&self, id: StoreID) -> crate::Result<Option<P>>
    where
        for<'de> P: Deserialize<'de>;
}