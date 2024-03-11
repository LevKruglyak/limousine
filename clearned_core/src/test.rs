use std::borrow::Borrow;
use std::path::Path;

use crate::classical::BTreeLayer;
use crate::search::{lower_bound, upper_bound, OptimalSearch, Search};
use crate::{ApproxPos, BaseLayer, ImmutableIndex, Key, Value};
use crate::{InternalLayer, InternalLayerBuild, NodeLayer};

type _InternalLayer<K> = BTreeLayer<K, 6>;

pub struct HybridIndex<K: Key, V: Value> {
    layers: Vec<_InternalLayer<K>>,
    base: BaseLayer<K, V>,
}

impl<K: Key, V: Value> HybridIndex<K, V> {
    /// Return the index of the base entry which should contain the
    /// current value if it exists, otherwise a lower bound
    fn search(&self, key: &K) -> Result<usize, usize> {
        // Start by searching top layer
        let mut pos = ApproxPos {
            lo: 0,
            hi: self.layers.last().unwrap().len(),
        };

        for layer in self.layers.iter().rev() {
            // Small adjustment since previous layer doesn't know how big this one is
            pos.hi = pos.hi.min(layer.len());
            pos.lo = pos.lo.min(pos.hi);

            pos = layer.search(key, pos);
        }

        pos.hi = pos.hi.min(self.base.len());
        pos.lo = pos.lo.min(pos.hi);

        OptimalSearch::search_by_key_with_offset(&self.base[pos.lo..pos.hi], key, pos.lo)
    }
}

impl<K: Key, V: Value> ImmutableIndex<K, V> for HybridIndex<K, V> {
    fn build(base: impl ExactSizeIterator<Item = (K, V)>) -> Self {
        let base = BaseLayer::build(base);
        let mut layers: Vec<_InternalLayer<K>> = Vec::new();

        // Build first layer
        layers.push(_InternalLayer::<K>::build(
            base.nodes().into_iter().map(|x| x.key),
        ));

        while layers.last().unwrap().len() > 2 {
            layers.push(_InternalLayer::<K>::build(
                layers
                    .last()
                    .unwrap()
                    .nodes()
                    .into_iter()
                    .map(|x| *x.borrow()),
            ));
        }

        Self { base, layers }
    }

    fn build_disk(
        base: impl ExactSizeIterator<Item = (K, V)>,
        path: impl AsRef<Path>,
    ) -> crate::Result<Self> {
        // Create index directory
        std::fs::create_dir(path.as_ref())?;
        let base = BaseLayer::build_disk(base, path.as_ref().join("base"))?;
        let mut layers: Vec<_InternalLayer<K>> = Vec::new();
        let mut layer = 0;

        // Build first layer
        layers.push(_InternalLayer::<K>::build_on_disk(
            base.nodes().into_iter().map(|x| x.key),
            path.as_ref().join(format!("layer{}", layer)),
        )?);

        layer += 1;

        while layers.last().unwrap().len() > 2 {
            layers.push(_InternalLayer::<K>::build_on_disk(
                layers
                    .last()
                    .unwrap()
                    .nodes()
                    .into_iter()
                    .map(|x| *x.borrow()),
                path.as_ref().join(format!("layer{}", layer)),
            )?);
            layer += 1;
        }

        Ok(Self { base, layers })
    }

    fn load(path: impl AsRef<Path>) -> crate::Result<Self> {
        let base = BaseLayer::load(path.as_ref().join("base"))?;
        let mut layers: Vec<_InternalLayer<K>> = Vec::new();
        let mut layer = 0;

        while layers.last().is_none() || layers.last().unwrap().len() > 2 {
            layers.push(_InternalLayer::<K>::load(
                path.as_ref().join(format!("layer{}", layer)),
            )?);
            layer += 1;
        }

        Ok(Self { base, layers })
    }

    fn range(&self, low: &K, high: &K) -> Self::RangeIterator<'_> {
        let low = lower_bound(self.search(low));
        let high = upper_bound(self.search(high), self.base.len());

        Self::RangeIterator::new(&self.base, low, high)
    }

    fn lookup(&self, key: &K) -> Option<V> {
        self.search(key).ok().map(|index| self.base[index].value)
    }

    type RangeIterator<'e> = HybridIndexRangeIterator<'e, K, V>;
}

pub struct HybridIndexRangeIterator<'e, K: Key, V: Value> {
    data: &'e BaseLayer<K, V>,
    low: usize,
    high: usize,
}

impl<'e, K: Key, V: Value> HybridIndexRangeIterator<'e, K, V> {
    fn new(data: &'e BaseLayer<K, V>, low: usize, high: usize) -> Self {
        Self { data, low, high }
    }
}

impl<'e, K: Key, V: Value> Iterator for HybridIndexRangeIterator<'e, K, V> {
    type Item = (&'e K, &'e V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.low < self.high {
            let result = Some((
                &self.data.nodes()[self.low].key,
                &self.data.nodes()[self.low].value,
            ));

            self.low += 1;
            return result;
        }

        None
    }
}
