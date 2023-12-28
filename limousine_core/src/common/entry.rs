use crate::common::bounded::KeyBounded;
use std::borrow::Borrow;
use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};

/// Simple entry type containing a key and a value
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Entry<K, V> {
    /// Key field
    pub key: K,

    /// Value field
    pub value: V,
}

/// SAFETY: should only cause uninitialized memory, eventually we should
/// use a macro to add padding bytes or something.
unsafe impl<K: Zeroable, V: Zeroable> Zeroable for Entry<K, V> {}
unsafe impl<K: Pod, V: Pod> Pod for Entry<K, V> {}

impl<K, V> Entry<K, V> {
    /// Create an entry from a key and a value
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

impl<K, V> KeyBounded<K> for Entry<K, V> {
    fn lower_bound(&self) -> &K {
        &self.key
    }
}

impl<K, V> Borrow<K> for Entry<K, V> {
    fn borrow(&self) -> &K {
        &self.key
    }
}

impl<K: Debug, V: Debug> Debug for Entry<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({:?}, {:?})", &self.key, &self.value))
    }
}
