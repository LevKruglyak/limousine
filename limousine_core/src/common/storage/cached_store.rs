// use std::{cell::RefCell, ops::Deref, rc::Rc};
//
// use super::{stable_cache::StableLRUCache, LocalStore, StoreID};
// use caches::{Cache, DefaultHashBuilder};
// use serde::{Deserialize, Serialize};
//
// type LRUCache<K, V> = caches::LRUCache<K, V, DefaultHashBuilder>;
//
// pub struct CachedLocalStore<Catalog, P>
// where
//     Catalog: Clone + Serialize,
// {
//     store: LocalStore<Catalog>,
//     cache: LRUCache<StoreID, Box<P>>,
// }
//
// impl<Catalog, P> CachedLocalStore<Catalog, P>
// where
//     Catalog: Clone + Serialize,
//     P: Serialize + for<'de> Deserialize<'de>,
// {
//     pub fn deref(&self, ptr: StoreID) -> crate::Result<Option<&P>> {
//         self.cache
//             .get_or_put_fallible(&ptr, move || {
//                 Ok(self.store.read_page::<P>(ptr)?.map(|page| CachedPage {
//                     page,
//                     modified: false,
//                 }))
//             })
//             .map(|opt| opt.map(|page_ref| &page_ref.page))
//         // if !self.cache.contains(&ptr) {
//         //     if let Some(page) = self.store.read_page::<P>(ptr)? {
//         //         let cached_page = CachedPage {
//         //             page,
//         //             modified: false,
//         //         };
//         //
//         //         // Flush any evicted pages to disk
//         //         if let Some((id, evicted)) = self.cache.put(ptr, cached_page) {
//         //             if evicted.modified {
//         //                 self.store.write_page(&evicted.page, id)?;
//         //             }
//         //         }
//         //     }
//         // }
//         //
//         // // Try getting a reference from the cache
//         // if let Some(deref) = self.cache.get(&ptr) {
//         //     return Ok(Some(&deref.page));
//         // }
//     }
//
//     pub fn deref_mut(&mut self, ptr: StoreID) -> crate::Result<Option<&mut P>> {
//         // if !self.cache.contains(&ptr) {
//         //     if let Some(page) = self.store.read_page::<P>(ptr)? {
//         //         let cached_page = CachedPage {
//         //             page,
//         //             modified: true,
//         //         };
//         //
//         //         // Flush any evicted pages to disk
//         //         if let Some((id, evicted)) = self.cache.put(ptr, cached_page) {
//         //             if evicted.modified {
//         //                 self.store.write_page(&evicted.page, id)?;
//         //             }
//         //         }
//         //     }
//         // }
//         //
//         // // Try getting a reference from the cache
//         // if let Some(deref) = self.cache.get_mut(&ptr) {
//         //     return Ok(Some(&mut deref.page));
//         // }
//
//         Ok(None)
//     }
// }
