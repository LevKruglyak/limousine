//! Code defining how to create a layer of PGMs given data

use std::ops::Sub;

use crate::{
    learned::generic::{Model, Segmentation},
    Entry, Key,
};

use super::pgm_model::LinearModel;

/// Helper struct to deal with points
#[derive(Clone)]
struct Point<K: Key> {
    x: K,
    y: i32,
}
impl<K: Key> Point<K> {
    fn new(x: K, y: i32) -> Self {
        Self { x, y }
    }

    /// Verbose but understandable slope
    fn slope(self) -> f64 {
        let run = num::cast::<K, f64>(self.x).unwrap();
        // For simplicity let's just make sure it's nowhere near 0
        assert!(run.abs() > 0.00001);
        (self.y as f64) / run
    }
}
impl<K: Key> Sub<Self> for Point<K> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Point::new(self.x.saturating_sub(rhs.x), self.y.saturating_sub(rhs.y))
    }
}

/// A data structure that will grow to incorporate points while building a PGM and eventually
/// produce a proper linear model, before moving on to the next one
pub struct SimplePGMSegmentator<K: Key, V, const EPSILON: usize> {
    pub first_k: Option<K>,
    pub first_v: Option<V>,
    pub max_slope: f64,
    pub min_slope: f64,
    pub num_entries: usize,
    // For sanity checking that input is increasing
    _last_k: Option<K>,
}
impl<K: Key, V, const EPSILON: usize> SimplePGMSegmentator<K, V, EPSILON> {
    pub fn new() -> Self {
        Self {
            first_k: None,
            first_v: None,
            max_slope: f64::MAX,
            min_slope: f64::MIN,
            num_entries: 0,
            _last_k: None,
        }
    }

    /// Tries to add an entry to this segmentor, returning a result about whether it was
    /// successful.
    pub fn try_add_entry(&mut self, entry: Entry<K, V>) -> Result<(), ()> {
        if self.num_entries == 0 {
            // If it's empty just add the point
            self.first_k = Some(entry.key);
            self.first_v = Some(entry.value);
            self._last_k = Some(entry.key);
            self.num_entries = 1;
            return Ok(());
        }
        // Sanity checks
        assert!(self.first_k.is_some());
        assert!(self.first_v.is_some());
        assert!(self._last_k.is_some());
        assert!(self._last_k.unwrap() < entry.key);
        // Get the worst case points we care about
        let base_point = Point::new(self.first_k.unwrap(), 0);
        let max_point = Point::new(
            entry.key,
            self.num_entries.saturating_add(1).saturating_add(EPSILON) as i32,
        );
        let min_point = Point::new(
            entry.key,
            self.num_entries.saturating_add(1).saturating_sub(EPSILON) as i32,
        );
        let this_max = (max_point - base_point.clone()).slope();
        let this_min = (min_point - base_point.clone()).slope();
        if self.num_entries == 1 {
            self.max_slope = this_max;
            self.min_slope = this_min;
        } else {
            let new_max_slope = this_max.min(self.max_slope);
            let new_min_slope = this_min.max(self.min_slope);
            if new_min_slope >= new_max_slope {
                return Err(());
            }
            self.max_slope = new_max_slope;
            self.min_slope = new_min_slope;
        }
        // This point is fine to add, and we've already update the slope
        self.num_entries += 1;
        self._last_k = Some(entry.key);
        Ok(())
    }

    // Outputs a linear model that fits all the points presented so far
    pub fn to_linear_model(&self) -> LinearModel<K, EPSILON> {
        assert!(self.first_k.is_some());
        assert!(self.num_entries > 0);
        let slope = if self.num_entries > 1 {
            (self.max_slope + self.min_slope) / 2.0
        } else {
            // A model that only has one point can pick any slope, we pick 1 arbitrarily
            1.0
        };
        LinearModel::new(self.first_k.unwrap(), slope, self.num_entries)
    }

    pub fn is_empty(&self) -> bool {
        self.num_entries <= 0
    }
}

impl<K: Key, V: Clone, const EPSILON: usize> Segmentation<K, V, LinearModel<K, EPSILON>> for LinearModel<K, EPSILON> {
    fn make_segmentation(data: impl Iterator<Item = crate::Entry<K, V>> + Clone) -> Vec<(Self, V)> {
        let mut result: Vec<(Self, V)> = vec![];

        let mut cur_segment: SimplePGMSegmentator<K, V, EPSILON> = SimplePGMSegmentator::new();
        for entry in data.into_iter() {
            match cur_segment.try_add_entry(entry) {
                Ok(_) => {
                    // Nothing to do, entry added successfully
                }
                Err(_) => {
                    // Export the model currently specified by the segmentor
                    result.push((cur_segment.to_linear_model(), cur_segment.first_v.clone().unwrap()));
                    // Reset current segmentor
                    cur_segment = SimplePGMSegmentator::new();
                }
            }
        }

        // Handle last segment
        if !cur_segment.is_empty() {
            result.push((cur_segment.to_linear_model(), cur_segment.first_v.clone().unwrap()));
        }

        result
    }
}

/// We'll test this part just by initializing TONS of indexes and making sure every key in every index is
/// properly indexed
#[cfg(test)]
mod pgm_segmentation_tests {
    use rand::{distributions::Uniform, Rng};

    use super::*;

    type Key = usize;
    type Value = usize;

    /// To test with different epsilon we need a struct that can handle that generic
    struct PGMSegTestCase<const EPSILON: usize> {
        verbose: bool,
        entries: Vec<Entry<Key, Value>>,
        models: Vec<LinearModel<Key, EPSILON>>,
    }
    impl<const EPSILON: usize> PGMSegTestCase<EPSILON> {
        /// Generates a test key, meaning make the entries, sort + dedup them
        fn generate(size: usize, verbose: Option<bool>) -> Self {
            let verbose = verbose.unwrap_or(true);
            let range = Uniform::from((Key::MIN)..(Key::MAX));
            let mut random_values: Vec<usize> = rand::thread_rng().sample_iter(&range).take(size).collect();
            random_values.sort();
            random_values.dedup();
            let entries: Vec<Entry<Key, Value>> = random_values
                .into_iter()
                .enumerate()
                .map(|(ix, key)| Entry::new(key, ix))
                .collect();
            Self {
                entries,
                verbose,
                models: vec![],
            }
        }

        /// Assuming data has already been generated, segments it as a layer
        fn train(&mut self) {}

        /// Assuming data has already been generated and trained on, tests that every key is correctly approximated
        fn test(&self) {}
    }
}
