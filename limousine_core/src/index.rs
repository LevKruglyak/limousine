use std::path::Path;

pub trait Index<K, V> {
    fn search(&self, key: &K) -> Option<&V>;

    fn insert(&mut self, key: K, value: V) -> Option<V>;
}

pub trait IndexBuild<K, V>: Index<K, V> {
    fn empty() -> Self;

    fn build(iter: impl Iterator<Item = (K, V)>) -> Self;
}

pub trait IndexBuildDisk<K, V>: Index<K, V> {
    fn empty(path: impl AsRef<Path>) -> Self;

    fn build(iter: impl Iterator<Item = (K, V)>, path: impl AsRef<Path>) -> Self;
}
