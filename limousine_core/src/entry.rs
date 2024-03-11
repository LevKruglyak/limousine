use std::borrow::Borrow;

use bytemuck::{Pod, Zeroable};

/// Simple entry type containing a key and a value
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Entry<K, V> {
    /// Key field
    pub key: K,
    /// Value field
    pub value: V,
}

impl<K, V> Entry<K, V> {
    /// Create an entry from a key and a value
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
