// pub mod disk;
pub mod memory;
mod node;

use crate::component::*;
use std::collections::BTreeMap;

// pub use disk::*;
pub use memory::*;

// -------------------------------------------------------
//                  Top Component
// -------------------------------------------------------

/// A `TopComponent` implementation built around the BTreeMap implementation in the Rust standard
/// library.
pub struct BTreeTopComponent<K: Clone, Base: NodeLayer<K>> {
    inner: BTreeMap<K, Base::Address>,
}

impl<K: Clone, Base> TopComponent<K, Base> for BTreeTopComponent<K, Base>
where
    Base: NodeLayer<K>,
    K: Ord,
{
    fn search(&self, _: &Base, key: &K) -> Base::Address {
        self.inner.range(..=key).next_back().unwrap().1.clone()
    }

    fn insert(&mut self, base: &Base, prop: PropogateInsert<K, Base>) {
        match prop {
            PropogateInsert::Single(key, address) => {
                self.inner.insert(key, address);
            }
            PropogateInsert::Rebuild => {
                self.inner.clear();

                for entry in base.full_range() {
                    self.inner.insert(entry.key, entry.value);
                }
            }
        }
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K: Clone, Base> TopComponentInMemoryBuild<K, Base> for BTreeTopComponent<K, Base>
where
    Base: NodeLayer<K>,
    K: Ord,
{
    fn build(base: &Base) -> Self {
        let mut inner = BTreeMap::new();

        for entry in base.full_range() {
            inner.insert(entry.key, entry.value);
        }

        Self { inner }
    }
}
