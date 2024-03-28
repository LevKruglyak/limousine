#![allow(dead_code)]

pub mod classical;
pub mod component;
pub mod index;
pub mod iter;

// pub mod learned;

mod common;
mod node_layer;
mod traits;

// Used by proc_macro
pub use anyhow::Result;

pub use classical::BoundaryDiskBTreeInternalAddress;
pub use classical::BoundaryDiskBTreeInternalComponent;

pub use classical::BoundaryDiskBTreeBaseAddress;
pub use classical::BoundaryDiskBTreeBaseComponent;

pub use classical::BTreeBaseAddress;
pub use classical::BTreeBaseComponent;

pub use classical::BTreeInternalAddress;
pub use classical::BTreeInternalComponent;

pub use classical::BTreeTopComponent;

pub use common::storage::GlobalStore;

pub use component::*;
pub use index::*;
pub use node_layer::*;
pub use traits::*;

pub use std::path::Path;
