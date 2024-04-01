#![allow(dead_code)]

pub mod classical;
pub mod component;
pub mod iter;
pub mod kv_store;

// pub mod learned;

mod common;
mod node_layer;
mod traits;

// Used by proc_macro
pub use anyhow::Result;

pub use classical::DeepDiskBTreeInternalAddress;
pub use classical::DeepDiskBTreeInternalComponent;

pub use classical::DeepDiskBTreeBaseAddress;
pub use classical::DeepDiskBTreeBaseComponent;

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
pub use kv_store::*;
pub use node_layer::*;
pub use traits::*;

pub use std::path::Path;
use std::path::PathBuf;

pub fn add_prefix_to_path<P: AsRef<Path>>(
    path: P,
    prefix: String,
) -> Result<PathBuf, std::io::Error> {
    let path = path.as_ref();

    if let Some(last_component) = path.iter().last() {
        let new_last_component = format!("{}_{}", prefix, last_component.to_str().unwrap());

        let mut new_path = PathBuf::new();
        for component in path.iter().take(path.iter().count() - 1) {
            new_path.push(component);
        }
        new_path.push(new_last_component);

        Ok(new_path)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Path is empty",
        ))
    }
}
