use crate::common::entry::Entry;
use crate::common::stack_map::StackMap;
use crate::kv::StaticBounded;
use crate::learned::generic::{ApproxPos, Model};
use crate::Key;
use std::borrow::Borrow;
use std::fmt::Debug;

use super::pgm_model::LinearModel;

#[derive(Clone)]
pub struct PGMNode<K: Key, V, const EPSILON: usize> {
    pub data: Vec<Entry<K, V>>,
    pub model: LinearModel<K, EPSILON>,
}

impl<K: Key + Debug, V: Debug, const EPSILON: usize> Debug for PGMNode<K, V, EPSILON> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.data))
    }
}

impl<K: Key, V: Clone, const EPSILON: usize> PGMNode<K, V, EPSILON> {
    pub fn from_model_n_vec(model: LinearModel<K, EPSILON>, data: Vec<Entry<K, V>>) -> Self {
        Self {
            data: data.clone(),
            model: model.clone(),
        }
    }

    pub fn entries(&self) -> &[Entry<K, V>] {
        self.data.as_slice()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn min(&self) -> &K
    where
        K: StaticBounded,
    {
        if self.is_empty() {
            &K::min_ref()
        } else {
            &self.data[0].key
        }
    }

    /// A wrapper to the approximate function on the model which will always return things with bounds that make sense
    pub fn approximate(&self, key: &K) -> ApproxPos {
        let mut guess = self.model.approximate(key);
        guess.lo = guess.lo.min(self.data.len() - 1);
        guess.hi = guess.hi.min(self.data.len());
        guess
    }

    pub fn search_lub(&self, key: &K) -> &V
    where
        K: Ord + Copy,
    {
        unimplemented!("PGMNode::search_lub")
    }

    pub fn search_exact(&self, key: &K) -> Option<&V>
    where
        K: Ord + Copy,
    {
        unimplemented!("PGMNode::search_lub")
    }
}

impl<K: Copy + StaticBounded + Key, V: Clone, const EPSILON: usize> Borrow<K> for PGMNode<K, V, EPSILON> {
    fn borrow(&self) -> &K {
        self.min()
    }
}

impl<K: Copy + StaticBounded + Key, V: Clone, const EPSILON: usize> Borrow<K> for &PGMNode<K, V, EPSILON> {
    fn borrow(&self) -> &K {
        self.min()
    }
}

impl<K: Copy + StaticBounded + Key, V: Clone, const EPSILON: usize> Borrow<K> for &mut PGMNode<K, V, EPSILON> {
    fn borrow(&self) -> &K {
        self.min()
    }
}
