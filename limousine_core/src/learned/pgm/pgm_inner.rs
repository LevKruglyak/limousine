//! The "inner" of a PGM node is what contains the data we care about.
//! That is, it contains:
//! - The data itself (a variable number of entries)
//! - A linear model (which is used to approximate the data)

use crate::common::bounded::{KeyBounded, StaticBounded};
use crate::common::entry::Entry;
use crate::common::stack_map::StackMap;
use crate::component::{Key, Value};
use crate::learned::generic::{ApproxPos, LearnedModel};
use crate::learned::pgm::pgm_model::LinearModel;
use std::borrow::Borrow;
use std::fmt::Debug;

/// Store the data and model separate to keep model logic clean and interchangeable
#[derive(Clone)]
pub struct PGMInner<K: Key, V: Value, const EPSILON: usize> {
    pub data: Vec<Entry<K, V>>,
    pub model: LinearModel<K, EPSILON>,
}
impl<K: Key, V: Value, const EPSILON: usize> Debug for PGMInner<K, V, EPSILON> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", &self.data))
    }
}

/// For using the data/model pair, likely inside a LinkedNode
impl<K: Key, V: Value, const EPSILON: usize> PGMInner<K, V, EPSILON> {
    /// Get an empty inner which should sit at the end of the layer and act as a sentinel
    pub fn sentinel() -> Self {
        Self {
            data: vec![],
            model: LinearModel::sentinel(),
        }
    }

    /// Create a new inner from a model and data
    pub fn from_model_n_vec(model: LinearModel<K, EPSILON>, data: Vec<Entry<K, V>>) -> Self {
        Self { data, model }
    }

    /// Return the data as a slice
    pub fn entries(&self) -> &[Entry<K, V>] {
        self.data.as_slice()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the smallest key in this inner.
    /// NOTE: Empty models (sentinels) return the max key
    pub fn min(&self) -> &K {
        if self.is_empty() {
            &K::max_ref()
        } else {
            &self.data[0].key
        }
    }

    /// A wrapper to the approximate function on the model which will
    /// always return things with bounds that make sense
    pub fn approximate(&self, key: &K) -> ApproxPos {
        let mut guess = self.model.approximate(key);
        guess.lo = guess.lo.min(self.data.len() - 1).max(0);
        guess.hi = guess.hi.min(self.data.len()).max(0);
        guess
    }

    /// Search for the lower bound of the key in the data
    pub fn search_lub(&self, key: &K) -> &V {
        unimplemented!("PGMNode::search_lub")
    }

    /// Get the exact key in the data
    pub fn search_exact(&self, key: &K) -> Option<&V> {
        unimplemented!("PGMNode::search_lub")
    }
}

impl<K: Key, V: Value, const EPSILON: usize> KeyBounded<K> for PGMInner<K, V, EPSILON> {
    fn lower_bound(&self) -> &K {
        self.min()
    }
}

impl<K: Key, V: Value, const EPSILON: usize> Borrow<K> for PGMInner<K, V, EPSILON> {
    fn borrow(&self) -> &K {
        self.min()
    }
}

impl<K: Key, V: Value, const EPSILON: usize> Borrow<K> for &PGMInner<K, V, EPSILON> {
    fn borrow(&self) -> &K {
        self.min()
    }
}

impl<K: Key, V: Value, const EPSILON: usize> Borrow<K> for &mut PGMInner<K, V, EPSILON> {
    fn borrow(&self) -> &K {
        self.min()
    }
}
