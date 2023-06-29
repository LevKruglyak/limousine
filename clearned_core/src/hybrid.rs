use crate::{
    search::{lower_bound, upper_bound, OptimalSearch, Search},
    ApproxPos, BaseLayer, HybridIndexRangeIterator, ImmutableIndex, Key, NodeLayer, Value,
};
use std::path::Path;

pub trait HybridLayer<K>: 'static {
    fn len(&self) -> usize;

    fn search(&self, key: &K, range: ApproxPos) -> ApproxPos;

    fn build(layer: usize, base: impl ExactSizeIterator<Item = K>) -> Self;

    fn build_on_disk(
        layer: usize,
        base: impl ExactSizeIterator<Item = K>,
        path: impl AsRef<Path>,
    ) -> crate::Result<Self>
    where
        Self: Sized;

    fn load(layer: usize, path: impl AsRef<Path>) -> crate::Result<Self>
    where
        Self: Sized;

    fn key_iter<'e>(&'e self) -> Box<dyn ExactSizeIterator<Item = K> + 'e>;
}

pub struct HybridIndex<K: Key, V: Value, I: HybridLayer<K>> {
    layers: Vec<I>,
    base: BaseLayer<K, V>,
}

impl<K: Key, V: Value, I: HybridLayer<K>> HybridIndex<K, V, I> {
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

impl<K: Key, V: Value, I: HybridLayer<K>> ImmutableIndex<K, V> for HybridIndex<K, V, I> {
    fn build_in_memory(base: impl ExactSizeIterator<Item = (K, V)>) -> Self {
        let base = BaseLayer::build(base);
        let mut layers: Vec<I> = Vec::new();
        let mut layer = 0;

        // Build first layer
        layers.push(I::build(layer, base.nodes().into_iter().map(|x| x.key)));
        layer += 1;

        while layers.last().unwrap().len() > 2 {
            layers.push(I::build(layer, layers.last().unwrap().key_iter()));

            layer += 1;
        }

        Self { base, layers }
    }

    fn build_on_disk(
        base: impl ExactSizeIterator<Item = (K, V)>,
        path: impl AsRef<Path>,
        threshold: usize,
    ) -> crate::Result<Self> {
        // Create index directory
        std::fs::create_dir(path.as_ref())?;
        let base = BaseLayer::build_disk(base, path.as_ref().join("base"))?;
        let mut layers: Vec<I> = Vec::new();
        let mut layer = 0;

        // Build first layer
        layers.push(I::build_on_disk(
            layer,
            base.nodes().into_iter().map(|x| x.key),
            path.as_ref().join(format!("layer{}", layer)),
        )?);

        layer += 1;

        while layers.last().unwrap().len() > 2 {
            if layer < threshold {
                layers.push(I::build_on_disk(
                    layer,
                    layers.last().unwrap().key_iter(),
                    path.as_ref().join(format!("layer{}", layer)),
                )?);
            } else {
                layers.push(I::build(layer, layers.last().unwrap().key_iter()));
            }
            layer += 1;
        }

        Ok(Self { base, layers })
    }

    fn load(path: impl AsRef<Path>, threshold: usize) -> crate::Result<Self> {
        let base = BaseLayer::load(path.as_ref().join("base"))?;
        let mut layers: Vec<I> = Vec::new();
        let mut layer = 0;

        while layers.last().is_none() || layers.last().unwrap().len() > 2 {
            if layer < threshold {
                layers.push(I::load(
                    layer,
                    path.as_ref().join(format!("layer{}", layer)),
                )?);
            } else {
                layers.push(I::build(layer, layers.last().unwrap().key_iter()));
            }

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
