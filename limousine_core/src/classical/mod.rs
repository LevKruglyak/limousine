// pub mod disk;
pub mod memory;
mod node;

use crate::common::address::Address;
use crate::component::*;
use std::collections::BTreeMap;
use std::ops::Bound;

// pub use disk::*;
pub use memory::*;

// -------------------------------------------------------
//                  Top Component
// -------------------------------------------------------

/// A `TopComponent` implementation built around the BTreeMap implementation in the Rust standard
/// library.
pub struct BTreeTopComponent<K, X: 'static> {
    inner: BTreeMap<K, Address>,
    _ph: std::marker::PhantomData<X>,
}

impl<K, X, Base> TopComponent<K, Base> for BTreeTopComponent<K, X>
where
    Base: NodeLayer<K>,
    K: Ord + Copy,
{
    fn search(&self, _: &Base, key: &K) -> Address {
        self.inner.range(..=key).next_back().unwrap().1.clone()
    }

    fn insert(&mut self, base: &Base, prop: PropogateInsert<K>) {
        match prop {
            PropogateInsert::Single(key, address) => {
                self.inner.insert(key, address);
            }
            PropogateInsert::Rebuild => {
                self.inner.clear();

                for (key, address) in base.range(Bound::Unbounded, Bound::Unbounded) {
                    self.inner.insert(key, address);
                }
            }
        }
    }
}

impl<K, X, Base> TopComponentInMemoryBuild<K, Base> for BTreeTopComponent<K, X>
where
    Base: NodeLayer<K>,
    K: Ord + Copy,
{
    fn build(base: &Base) -> Self {
        let mut inner = BTreeMap::new();

        for (key, address) in base.range(Bound::Unbounded, Bound::Unbounded) {
            inner.insert(key, address);
        }

        Self {
            inner,
            _ph: std::marker::PhantomData,
        }
    }
}
