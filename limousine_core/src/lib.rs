pub mod classical;
pub mod component;
pub mod index;
pub mod iter;
// pub mod learned;

mod common;

// Used by proc_macro

pub use classical::BTreeBaseComponent;
pub use classical::BTreeInternalComponent;
pub use classical::BTreeTopComponent;

pub use classical::BTreeBaseAddress;
pub use classical::BTreeInternalAddress;

pub use component::*;
pub use index::*;

pub use std::path::Path;

pub use marble;
pub fn load_store(path: impl AsRef<Path>, clear: bool) -> marble::Marble {
    if clear {
        std::fs::remove_dir_all(path.as_ref())
            .expect(format!("Failed to clear `{:?}`!", path.as_ref()).as_str());
    }

    marble::open(path.as_ref()).expect(format!("Failed to load `{:?}`!", path.as_ref()).as_ref())
}
