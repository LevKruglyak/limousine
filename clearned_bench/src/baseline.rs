use clearned_core::{
    search::{lower_bound, upper_bound, OptimalSearch, Search},
    BaseLayer, HybridIndexRangeIterator, ImmutableIndex, Key, NodeLayer, Value,
};
use std::collections::{btree_map::Range, BTreeMap};

pub struct BTreeMapBaseline<K: Key, V: Value>(BTreeMap<K, V>);

impl<K: Key, V: Value> ImmutableIndex<K, V> for BTreeMapBaseline<K, V> {
    fn lookup(&self, key: &K) -> Option<V> {
        self.0.get(key).map(|x| *x)
    }

    fn build_in_memory(base: impl ExactSizeIterator<Item = (K, V)>) -> Self {
        let mut btree = BTreeMap::new();

        for entry in base {
            btree.insert(entry.0, entry.1);
        }

        Self(btree)
    }

    fn build_on_disk(
        _: impl ExactSizeIterator<Item = (K, V)>,
        _: impl AsRef<std::path::Path>,
        _: usize,
    ) -> clearned_core::Result<Self> {
        unimplemented!()
    }

    fn load(_: impl AsRef<std::path::Path>, _: usize) -> clearned_core::Result<Self> {
        unimplemented!()
    }

    fn range(&self, low: &K, high: &K) -> Self::RangeIterator<'_> {
        self.0.range(low..high)
    }

    type RangeIterator<'e> = Range<'e, K, V>;
}

pub struct SortedBaseline<K: Key, V: Value>(BaseLayer<K, V>);

impl<K: Key, V: Value> ImmutableIndex<K, V> for SortedBaseline<K, V> {
    fn lookup(&self, key: &K) -> Option<V> {
        OptimalSearch::search_by_key(self.0.nodes(), key)
            .ok()
            .map(|index| self.0[index].value)
    }

    fn build_in_memory(base: impl ExactSizeIterator<Item = (K, V)>) -> Self {
        Self(BaseLayer::build(base))
    }

    fn build_on_disk(
        _: impl ExactSizeIterator<Item = (K, V)>,
        _: impl AsRef<std::path::Path>,
        _: usize,
    ) -> clearned_core::Result<Self> {
        unimplemented!()
    }

    fn load(_: impl AsRef<std::path::Path>, _: usize) -> clearned_core::Result<Self> {
        unimplemented!()
    }

    fn range(&self, low: &K, high: &K) -> Self::RangeIterator<'_> {
        let low = lower_bound(OptimalSearch::search_by_key(self.0.nodes(), low));
        let high = upper_bound(
            OptimalSearch::search_by_key(self.0.nodes(), high),
            self.0.len(),
        );

        Self::RangeIterator::new(&self.0, low, high)
    }

    type RangeIterator<'e> = HybridIndexRangeIterator<'e, K, V>;
}
