pub mod disk;
pub mod memory;
mod node;

use crate::BaseComponent;
use crate::InternalComponent;
use crate::Key;
use crate::NodeLayer;
use crate::TopComponent;
use crate::Value;
use std::borrow::Borrow;
use std::collections::BTreeMap;

pub use disk::*;
pub use memory::*;

// -------------------------------------------------------
//                  Top Component
// -------------------------------------------------------

pub struct BTreeTopComponent<K: Key, B: NodeLayer<K>> {
    map: BTreeMap<K, B::NodeRef>,
}

impl<K: Key, B: NodeLayer<K>> TopComponent<K, B> for BTreeTopComponent<K, B> {
    fn new_top(base: &B) -> Self {
        let mut component = Self {
            map: BTreeMap::new(),
        };

        for base_ptr in base.iter() {
            let base_node = base.node_ref(base_ptr.clone());
            component.map.insert(*base_node.borrow(), base_ptr);
        }

        component
    }

    fn search_top(&self, key: &K) -> B::NodeRef {
        self.map.range(..=key).next_back().unwrap().1.clone()
    }

    fn insert_top(&mut self, key: K, value: B::NodeRef) {
        self.map.insert(key, value);
    }
}
