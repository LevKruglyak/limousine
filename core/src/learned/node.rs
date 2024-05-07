use learned_index_segmentation::LinearModel;
use num::PrimInt;
use serde::{Deserialize, Serialize};

use crate::{KeyBounded, StaticBounded};
use gapped_array::GappedKVArray;

impl<K: StaticBounded, const EPSILON: usize> KeyBounded<K> for LinearModel<K, EPSILON> {
    fn lower_bound(&self) -> &K {
        self.min_key()
    }
}

#[derive(Debug)]
pub struct PGMNode<K: Ord + Copy, V: Copy, const EPSILON: usize> {
    gapped: GappedKVArray<K, V>,
    model: LinearModel<K, EPSILON>,
}

impl<K: Copy + StaticBounded, V: Copy, const EPSILON: usize> KeyBounded<K>
    for PGMNode<K, V, EPSILON>
{
    fn lower_bound(&self) -> &K {
        self.gapped.min().unwrap_or(&K::max_ref())
    }
}

impl<K: Ord + Copy + PrimInt, V: Copy, const EPSILON: usize> Default for PGMNode<K, V, EPSILON> {
    fn default() -> Self {
        Self {
            gapped: GappedKVArray::new(0),
            model: LinearModel::sentinel(),
        }
    }
}

impl<K: StaticBounded + Copy + PrimInt, V: Copy, const EPSILON: usize> PGMNode<K, V, EPSILON> {
    pub fn from_trained(model: LinearModel<K, EPSILON>, entries: Vec<(K, V)>) -> Self {
        // NOTE: Filling at 0.5 utilization is just a heuristic, eventually this should be a param
        let mut gapped = GappedKVArray::new(entries.len() * 2);
        for (key, value) in entries {
            let hint = model.hint(&key);
            gapped.initial_model_based_insert((key, value), hint);
        }
        Self { gapped, model }
    }
}
