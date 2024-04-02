//! ***limousine_engine*** provides a procedural macro to automatically generate an (immutable) hybrid index design:
//!
//! ```ignore
//! use limousine_engine::prelude::*;
//!
//! create_immutable_hybrid_index! {
//!     name: ExampleHybridIndex,
//!     layout: {
//!         0 | 1 => btree(16),
//!         _ => pgm(4),
//!     }
//! }
//! ```
//!
//! To generate a design, we provide a name for the structure, and a layout description, which resembles the syntax of a Rust match expression. In this example, the first two layers are BTree layers with a fanout of 16, and the rest of the layers are PGM layers with an epsilon parameter of 4. All of this is parsed and generated into a static implementation at compile time by the procedural macro. We can also generate efficient pure designs using this approach:
//!
//! ```ignore
//! use limousine_engine::prelude::*;
//!
//! create_immutable_hybrid_index! {
//!     name: BTreeIndex,
//!     layout: {
//!         _ => btree(16),
//!     }
//! }
//!
//! create_immutable_hybrid_index! {
//!     name: PGMIndex,
//!     layout: {
//!         _ => pgm(4),
//!     }
//! }
//! ```
//!
//! We can then use these generated structs to perform queries:
//!
//! ```ignore
//! // Load the first two layer of the index from memory
//! let index = ExampleHybridIndex::<i32, i32>::load("path_to_index", 2)?;
//!
//! // Range query
//! for (key, value) in index.range(&0, &100) {
//!     println!("found entry: {key:?} {value:?}");
//! }
//! ```
#![deny(missing_docs)]

/// Include this at the top of the file when materializing a hybrid index or using a hybrid index.
#[cfg(not(feature = "dev"))]
pub mod prelude {
    pub use limousine_derive::create_immutable_hybrid_index;

    pub use limousine_core::HybridIndex;
}

#[cfg(not(feature = "dev"))]
pub use limousine_core::Result;

#[doc(hidden)]
#[cfg(not(feature = "dev"))]
pub use limousine_core as private;

/// Include this at the top of the file when materializing a hybrid index or using a hybrid index.
#[cfg(feature = "dev")]
pub mod prelude {
    pub use dev_limousine_derive::create_kv_store;

    pub use dev_limousine_core::KVStore;
    pub use dev_limousine_core::PersistedKVStore;
}

#[cfg(feature = "dev")]
pub use dev_limousine_core::Result;

#[doc(hidden)]
#[cfg(feature = "dev")]
pub use dev_limousine_core as private;
