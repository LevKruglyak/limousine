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
pub struct BTreeTopComponent<K, Base: NodeLayer<K>> {
    inner: BTreeMap<K, Base::Address>,
}

impl<K, Base> TopComponent<K, Base> for BTreeTopComponent<K, Base>
where
    Base: NodeLayer<K>,
    K: Ord,
{
    fn search(&self, key: &K) -> Base::Address {
        self.inner.range(..=key).next_back().unwrap().1.clone()
    }

    fn insert(&mut self, prop: PropogateInsert<'_, K, Base>) {
        match prop {
            PropogateInsert::Single(key, address) => {
                self.inner.insert(key, address);
            }
            PropogateInsert::Rebuild(base) => {
                self.inner.clear();

                for (key, address) in base.full_range() {
                    self.inner.insert(key, address);
                }
            }
        }
    }

    fn size(&self) -> usize {
        self.inner.len()
    }
}

impl<K, Base> TopComponentInMemoryBuild<K, Base> for BTreeTopComponent<K, Base>
where
    Base: NodeLayer<K>,
    K: Ord,
{
    fn build(base: &Base) -> Self {
        let mut inner = BTreeMap::new();

        for (key, address) in base.full_range() {
            inner.insert(key, address);
        }

        Self { inner }
    }
}
