use learned_index_segmentation::LinearModel;

use crate::{Key, KeyBounded, StaticBounded};
use gapped_array::GappedEntryArray;

impl<K: StaticBounded, const EPSILON: usize> KeyBounded<K> for LinearModel<K, EPSILON> {
    fn lower_bound(&self) -> &K {
        self.min_key()
    }
}

#[derive(Debug)]
pub struct PGMNode<K, V, const EPSILON: usize> {
    gapped: GappedEntryArray<K, V>,
    model: LinearModel<K, EPSILON>,
}

impl<K: Key, V: Clone, const EPSILON: usize> KeyBounded<K> for PGMNode<K, V, EPSILON> {
    fn lower_bound(&self) -> &K {
        self.gapped.min().unwrap_or(&K::max_ref())
    }
}

impl<K: Key, V: Clone, const EPSILON: usize> Default for PGMNode<K, V, EPSILON> {
    fn default() -> Self {
        Self {
            gapped: GappedEntryArray::new(0),
            model: LinearModel::sentinel(),
        }
    }
}

impl<K: Key, V: Clone, const EPSILON: usize> PGMNode<K, V, EPSILON> {
    pub fn from_trained(model: LinearModel<K, EPSILON>, entries: Vec<(K, V)>) -> Self {
        // NOTE: Filling at 0.5 utilization is just a heuristic, eventually this should be a param
        let mut gapped = GappedEntryArray::new(entries.len() * 2);
        for (key, value) in entries {
            let hint = model.hint(&key).min(gapped.len() - 1);
            gapped
                .initial_model_based_insert((key, value), hint)
                .unwrap();
        }
        Self { gapped, model }
    }

    pub fn search_exact(&self, key: &K) -> Option<&V> {
        let hint = self.model.hint(key);
        self.gapped.search_exact(key, Some(hint))
    }

    pub fn search_pir(&self, key: &K) -> &V {
        let hint = self.model.hint(key);
        match self.gapped.search_pir(key, Some(hint)) {
            Some(val) => val,
            None => self.gapped.min_val().unwrap(),
        }
    }

    pub fn grow_insert(&mut self, entry: (K, V)) {
        if self.gapped.density() >= 0.8 {
            let scale_factor = 2.0;
            self.gapped.rescale(scale_factor).unwrap();
            self.model.rescale(scale_factor as f64);
        }
        let hint = self.model.hint(&entry.0);
        self.gapped.upsert_with_hint(entry, hint).unwrap();
    }
}
