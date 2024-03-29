use crate::{Key, Persisted, Value};
use std::path::Path;

pub trait Index<K, V>
where
    K: Key,
    V: Value,
{
    fn search(&self, key: K) -> Option<V>;

    fn insert(&mut self, key: K, value: V) -> Option<V>;

    fn empty() -> Self;

    fn build(iter: impl Iterator<Item = (K, V)>) -> Self;
}

pub trait PersistedIndex<K, V>
where
    Self: Sized,
    K: Persisted + Key,
    V: Persisted + Value,
{
    fn search(&self, key: K) -> crate::Result<Option<V>>;

    fn insert(&mut self, key: K, value: V) -> crate::Result<Option<V>>;

    fn open(path: impl AsRef<Path>) -> crate::Result<Self>;

    fn build(iter: impl Iterator<Item = (K, V)>, path: impl AsRef<Path>) -> crate::Result<Self>;
}
