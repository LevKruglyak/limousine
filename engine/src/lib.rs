//! `limousine_engine` provides a procedural macro to automatically
//! generate an hybrid key-value store design consisting of both
//! classical and learned components.
//!
//! **As of the current version, learned components are not yet fully
//! supported.**
//!
//! ```ignore
//! use limousine_engine::prelude::*;
//!
//! create_kv_store! {
//!     name: ExampleStore,
//!     layout: [
//!         btree_top(),
//!         pgm(epsilon = 8),
//!         pgm(epsilon = 8),
//!         btree(fanout = 32),
//!         btree(fanout = 32, persist),
//!         btree(fanout = 64, persist)   
//!     ]
//! }
//! ```
//!
//! To generate a design, we provide a name for the structure and a
//! layout description which consists of a stack of components. For
//! instance in the above example, the key-value store consists of
//! a base layer of on-disk BTree nodes of fanout 64, underneath a  
//! layer of on on-disk BTree nodes with fanout 32, underneath an
//! in-memory layer of BTree nodes with fanout 32. On top of this, we
//! have two in-memory PGM learned layers with epsilon parameters of 8,
//! and a tiny in-memory BTree as a top layer.
//!
//! **Since learned components are not yet fully supported, the above example
//! will not compile. To get a working key-value store in the current version,
//! we should only use BTree components.**
//!
//! ```
//! use limousine_engine::prelude::*;
//!
//! create_kv_store! {
//!     name: ExampleStore,
//!     layout: [
//!         btree_top(),
//!         btree(fanout = 8),
//!         btree(fanout = 8),
//!         btree(fanout = 32),
//!         btree(fanout = 32, persist),
//!         btree(fanout = 64, persist)   
//!     ]
//! }
//! ```
//!
//! We can then use these generated data structures to perform queries:
//!
//! ```ignore
//! // Load the first two layer of the index from memory
//! let index: ExampleStore<u128, u128> = ExampleStore::open("data/index")?;
//!
//! index.insert(10, 50)?;
//! index.insert(20, 60)?;
//! index.insert(30, 70)?;
//! index.insert(40, 80)?;
//!
//! assert_eq!(index.search(10)?, Some(50));
//! ```
#![deny(missing_docs)]

/// Include this at the top of the file when materializing a hybrid index or using a hybrid index.
pub mod prelude {
    pub use limousine_derive::create_kv_store;

    pub use limousine_core::KVStore;
    pub use limousine_core::PersistedKVStore;
}

pub use limousine_core::Result;

#[doc(hidden)]
pub use limousine_core as private;
