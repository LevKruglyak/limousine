use bytemuck::{Pod, Zeroable};
use mmap_buffer::Buffer;
use num::PrimInt;
use std::{
    borrow::Borrow,
    fmt::Debug,
    ops::Deref,
    path::{Path, PathBuf},
};
use trait_set::trait_set;

mod classical;
mod hybrid;
mod learned;
pub mod search;

pub use classical::BTreeLayer;
pub use hybrid::HybridIndex;
pub use hybrid::HybridIndexRangeIterator;
pub use hybrid::HybridLayer;
pub use learned::pgm_node::PGMLayer;

/// Generic error type (to avoid a dependency on anyhow)
pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

// Until `trait_alias` is stabilized, we have to use a macro
trait_set! {
    /// General value type, thread-safe and POD
    pub trait Value = Send + Sync + Pod + Debug;

    /// General key type, thread safe, POD, and primitive integer type
    pub trait Key = Value + PrimInt;
}

/// Represents an immutable tree index over a sorted slice of entries
pub trait ImmutableIndex<K: Key, V: Value>: Sized {
    /// Build an index over the given data in memory
    /// This method assumes the data is sorted, and without key repetitions
    fn build_in_memory(base: impl ExactSizeIterator<Item = (K, V)>) -> Self;

    /// Build an index over the given data, persisting to disk
    /// This method assumes the data is sorted, and without key repetitions
    fn build_on_disk(
        base: impl ExactSizeIterator<Item = (K, V)>,
        path: impl AsRef<Path>,
        threshold: usize,
    ) -> Result<Self>;

    /// Load an index from memory, rebuilding layers which weren't persisted
    fn load(path: impl AsRef<Path>, threshold: usize) -> Result<Self>;

    /// Returns `Some(entry)` if the is an entry with key `key`, otherwise `None`
    fn lookup(&self, key: &K) -> Option<V>;

    /// Returns an iterator which iterates the entries between `low` and `high`, inclusive
    fn range(&self, low: &K, high: &K) -> Self::RangeIterator<'_>;

    type RangeIterator<'e>: Iterator<Item = (&'e K, &'e V)>
    where
        Self: 'e;
}

/// Some layer of nodes on top of which an index layer can be built.
pub trait NodeLayer<K: Key> {
    type Node: Borrow<K>;

    /// Returns a slice of keys, each of which represent the
    /// minimum keys of the underlying node.
    fn nodes(&self) -> &[Self::Node];

    fn len(&self) -> usize {
        self.nodes().len()
    }
}

/// A range of potential locations for the results of a query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApproxPos {
    lo: usize,
    hi: usize,
}

/// An internal layer of nodes which can search using an `ApproxPos`
/// to return some other `ApproxPos`. Not all `InternalLayers` need
/// the full flexibility of the `ApproxPos` generality,
/// but since we must support learned index layers, we need this generality.
pub trait InternalLayer<K: Key>: NodeLayer<K> {
    fn search(&self, key: &K, range: ApproxPos) -> ApproxPos;
}

/// A set of static methods for building a particular type of internal layer.
pub trait InternalLayerBuild<K: Key>: NodeLayer<K> {
    /// Build an index layer over the given data, storing in memory
    /// Assumes the data is sorted, and without key repetitions
    fn build(base: impl ExactSizeIterator<Item = K>) -> Self
    where
        Self: Sized;

    /// Build an index layer over the given data, persisting to disk
    /// Assumes the data is sorted, and without key repetitions
    fn build_on_disk(
        base: impl ExactSizeIterator<Item = K>,
        path: impl AsRef<Path>,
    ) -> Result<Self>
    where
        Self: Sized;

    /// Load an index from memory, rebuilding layers which weren't persisted
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized;
}

// ------------------------------------------------------
// Entry
// ------------------------------------------------------

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Entry<K, V> {
    pub key: K,
    pub value: V,
}

impl<K, V> Entry<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

impl<K, V> Borrow<K> for Entry<K, V> {
    fn borrow(&self) -> &K {
        &self.key
    }
}

unsafe impl<K: Zeroable, V: Zeroable> Zeroable for Entry<K, V> {}
unsafe impl<K: Pod, V: Pod> Pod for Entry<K, V> {}

// ------------------------------------------------------
// Base Layer
// ------------------------------------------------------

pub struct BaseLayer<K: Key, V: Value> {
    data: Buffer<Entry<K, V>>,
}

impl<K: Key, V: Value> BaseLayer<K, V> {
    pub fn build(base: impl ExactSizeIterator<Item = (K, V)>) -> Self {
        Self {
            data: Buffer::from_vec_in_memory(base.map(|(k, v)| Entry::new(k, v)).collect()),
        }
    }

    pub fn build_disk(
        base: impl ExactSizeIterator<Item = (K, V)>,
        path: impl AsRef<Path>,
    ) -> Result<Self> {
        let data: Vec<Entry<K, V>> = base.map(|(k, v)| Entry::new(k, v)).collect();

        Ok(Self {
            data: Buffer::from_slice_on_disk(&data[..], path)?,
        })
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            data: Buffer::load_from_disk(path)?,
        })
    }
}

impl<K: Key, V: Value> NodeLayer<K> for BaseLayer<K, V> {
    type Node = Entry<K, V>;

    fn nodes(&self) -> &[Self::Node] {
        self.data.deref()
    }
}

impl<K: Key, V: Value> Deref for BaseLayer<K, V> {
    type Target = [Entry<K, V>];

    fn deref(&self) -> &Self::Target {
        self.data.deref()
    }
}

// ---------------------------------------------------------------------------
// Util
// ---------------------------------------------------------------------------

/// Utility function to create a new path with the given extension
fn path_with_extension(path: impl AsRef<Path>, extension: &str) -> Box<Path> {
    let mut buf = PathBuf::from(path.as_ref());
    buf.set_extension(extension);
    buf.into_boxed_path()
}
