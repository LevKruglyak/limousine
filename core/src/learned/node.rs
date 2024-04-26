use learned_index_segmentation::LinearModel;
use serde::{Deserialize, Serialize};

use crate::{KeyBounded, StaticBounded};
use gapped_array::GappedKVArray;

impl<K: StaticBounded, const EPSILON: usize> KeyBounded<K> for LinearModel<K, EPSILON> {
    fn lower_bound(&self) -> &K {
        self.min_key()
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct PGMNode<K: Ord + Default + Copy, V: Default + Copy, const EPSILON: usize> {
    gapped: GappedKVArray<K, V>,
    model: LinearModel<K, EPSILON>,
}

impl<K: StaticBounded + Default + Copy, V: Default + Copy, const EPSILON: usize> KeyBounded<K>
    for PGMNode<K, V, EPSILON>
{
    fn lower_bound(&self) -> &K {
        self.gapped.min().unwrap_or(K::min_ref())
    }
}
