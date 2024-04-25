use learned_index_segmentation::LinearModel;

use crate::{KeyBounded, StaticBounded};

impl<K: StaticBounded, const EPSILON: usize> KeyBounded<K> for LinearModel<K, EPSILON> {
    fn lower_bound(&self) -> &K {
        self.min_key()
    }
}
