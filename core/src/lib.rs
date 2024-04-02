#![allow(dead_code)]

pub mod classical;
pub mod component;
pub mod iter;
pub mod kv_store;

mod common;
mod node_layer;
mod traits;

// Used by proc_macro
pub use anyhow::Result;

pub use classical::*;
pub use common::storage::GlobalStore;

pub use component::*;
pub use kv_store::*;
pub use node_layer::*;
pub use traits::*;

pub use std::path::Path;

pub fn add_prefix_to_path<P: AsRef<Path>>(
    path: P,
    prefix: String,
) -> Result<std::path::PathBuf, std::io::Error> {
    let path = path.as_ref();

    if let Some(last_component) = path.iter().last() {
        let new_last_component = format!("{}_{}", prefix, last_component.to_str().unwrap());

        let mut new_path = std::path::PathBuf::new();
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
