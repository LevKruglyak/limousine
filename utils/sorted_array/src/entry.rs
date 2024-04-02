use core::{borrow::Borrow, fmt::Debug};

/// Simple entry type containing a key and a value
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct SortedArrayEntry<K, V> {
    /// Key
    pub key: K,

    /// Value
    pub value: V,
}

impl<K, V> SortedArrayEntry<K, V> {
    /// Create a new entry
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

impl<K, V> Borrow<K> for SortedArrayEntry<K, V> {
    fn borrow(&self) -> &K {
        &self.key
    }
}

impl<K: Debug, V: Debug> Debug for SortedArrayEntry<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("({:?}, {:?})", &self.key, &self.value))
    }
}
