use std::{ops::Deref, path::Path};

use mmap_buffer::Buffer;

use crate::{entry::Entry, Key, NodeLayer, Result, Value};

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
