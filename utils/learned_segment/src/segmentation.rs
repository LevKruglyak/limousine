use num::PrimInt;

use crate::{model::LinearModel, point::Point};

/// A data structure that will grow to incorporate points while building a PGM and eventually
/// produce a proper linear model, before moving on to the next one
pub struct LinearSimpleSegmentation<K, V, const EPSILON: usize> {
    pub first_key: Option<K>,
    pub entries: Vec<(K, V)>,
    pub max_slope: f64,
    pub min_slope: f64,
    pub num_entries: usize,

    // For sanity checking that input is increasing
    #[cfg(debug_assertions)]
    last_key: Option<K>,
}

impl<K: PrimInt, V: Clone, const EPSILON: usize> LinearSimpleSegmentation<K, V, EPSILON> {
    pub fn new() -> Self {
        Self {
            first_key: None,
            entries: Vec::new(),
            max_slope: f64::MAX,
            min_slope: f64::MIN,
            num_entries: 0,

            #[cfg(debug_assertions)]
            last_key: None,
        }
    }

    /// Tries to add an entry to this segmentor, returning a result about whether it was
    /// successful.
    fn try_add_entry(&mut self, entry: (K, V)) -> Result<(), ()> {
        if self.num_entries == 0 {
            // If it's empty just add the point
            self.first_key = Some(entry.0);
            self.entries = vec![entry.clone()];
            self.num_entries = 1;

            #[cfg(debug_assertions)]
            {
                self.last_key = Some(entry.0);
            }

            return Ok(());
        }

        // Sanity checks
        #[cfg(debug_assertions)]
        {
            debug_assert!(self.first_key.is_some());
            debug_assert!(self.entries.len() == self.num_entries);
            debug_assert!(self.last_key.is_some());
            debug_assert!(self.last_key.clone().unwrap() < entry.0);
        }

        // Get the worst case points we care about
        let base_point = Point::new(self.first_key.clone().unwrap(), 0);
        let max_point = Point::new(
            entry.0,
            self.num_entries
                .saturating_add(1) // The actual rank
                .saturating_sub(1) // To deal with floating point annoyances
                .saturating_add(EPSILON) as i32,
        );
        let min_point = Point::new(
            entry.0,
            self.num_entries
                .saturating_add(1) // The actual rank
                .saturating_add(1) // To deal with floating point annoyances
                .saturating_sub(EPSILON) as i32,
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
                // We can't fit this point in the model
                return Err(());
            }
            // SANITY TESTING
            #[cfg(debug_assertions)]
            {
                // Max slope should be monotonically decreasing
                debug_assert!(new_max_slope <= self.max_slope);
                // Min slope should be monotonically increasing
                debug_assert!(new_min_slope >= self.min_slope);
            }

            self.max_slope = new_max_slope;
            self.min_slope = new_min_slope;
        }

        // This point is fine to add, and we've already update the slope
        self.num_entries += 1;
        self.entries.push(entry.clone());

        #[cfg(debug_assertions)]
        {
            self.last_key = Some(entry.0);
        }

        Ok(())
    }

    // Outputs a linear model that fits all the points presented so far
    pub fn to_linear_model(&self) -> LinearModel<K, EPSILON> {
        assert!(self.first_key.is_some());
        assert!(self.num_entries > 0);

        let slope = if self.num_entries > 1 {
            (self.max_slope + self.min_slope) / 2.0
        } else {
            // A model that only has one point can pick any slope, we pick 1 arbitrarily
            1.0
        };

        LinearModel::new(self.first_key.unwrap(), slope, self.num_entries)
    }

    // Outputs the a vector of values that generated a linear model
    pub fn entries(&self) -> &Vec<(K, V)> {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.num_entries == 0
    }
}

pub fn linear_simple_segmentation<K: PrimInt, V: Clone, const EPSILON: usize>(
    data: impl Iterator<Item = (K, V)>,
) -> Vec<(LinearModel<K, EPSILON>, Vec<(K, V)>)> {
    let mut result: Vec<(LinearModel<K, EPSILON>, Vec<(K, V)>)> = vec![];

    let mut cur_segment: LinearSimpleSegmentation<K, V, EPSILON> = LinearSimpleSegmentation::new();

    for entry in data {
        match cur_segment.try_add_entry(entry.clone()) {
            Ok(_) => {
                // Nothing to do, entry added successfully
            }
            Err(_) => {
                // Export the model currently specified by the segmentor
                result.push((cur_segment.to_linear_model(), cur_segment.entries().clone()));
                // Reset current segmentor
                cur_segment = LinearSimpleSegmentation::new();

                // Should be fine to unwrap here since adding first entry always works
                cur_segment.try_add_entry(entry).unwrap();
            }
        }
    }

    // Handle last segment
    if !cur_segment.is_empty() {
        result.push((cur_segment.to_linear_model(), cur_segment.entries().clone()));
    }

    result
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
        entries: Vec<(Key, Value)>,
        models: Vec<LinearModel<Key, EPSILON>>,
        values: Vec<Value>,
        last_model_ix: usize,
        last_base_rank: usize,
    }

    impl<const EPSILON: usize> PGMSegTestCase<EPSILON> {
        /// Generates a test key, meaning make the entries, sort + dedup them
        fn generate(size: usize, verbose: Option<bool>) -> Self {
            let verbose = verbose.unwrap_or(true);
            if verbose {
                println!("Generating {} entries with eps={}", size, EPSILON);
            }
            let range = Uniform::from((Key::MIN)..(Key::MAX));
            let mut random_values: Vec<usize> =
                rand::thread_rng().sample_iter(&range).take(size).collect();
            random_values.sort();
            random_values.dedup();
            let entries: Vec<(Key, Value)> = random_values
                .into_iter()
                .enumerate()
                .map(|(ix, key)| (key, ix))
                .collect();

            Self {
                entries,
                verbose,
                models: vec![],
                values: vec![],
                last_model_ix: 0,
                last_base_rank: 0,
            }
        }

        /// Assuming data has already been generated, segments it as a layer
        fn train(&mut self) {
            if self.verbose {
                println!(
                    "Training on {} entries with eps={}",
                    self.entries.len(),
                    EPSILON
                );
            }
            let trained: Vec<(LinearModel<Key, EPSILON>, Vec<(Key, Value)>)> =
                linear_simple_segmentation(self.entries.clone().into_iter());

            self.models.clear();
            self.values.clear();
            trained.into_iter().for_each(|(model, values)| {
                self.models.push(model);
                self.values.push(values[0].0);
            });
        }

        /// Helper function for determining if a single entry is approximated within bounds
        fn is_entry_well_approximated(&mut self, entry: (Key, Value)) -> bool {
            let mut model_ix = self.last_model_ix;
            let mut base_rank = self.last_base_rank;
            while model_ix < self.models.len().saturating_sub(1) {
                if self.models[model_ix + 1].key > entry.0 {
                    break;
                }

                base_rank += self.models[model_ix].size;
                model_ix += 1;
            }
            let range = self.models[model_ix].approximate(&entry.0);
            self.last_base_rank = base_rank;
            self.last_model_ix = model_ix;
            return base_rank + range.0 <= entry.1 && entry.1 < base_rank + range.1;
        }

        /// Assuming data has already been generated and trained on, tests that every key is correctly approximated
        fn test(&mut self) {
            for entry in self.entries.clone() {
                assert!(self.is_entry_well_approximated(entry));
            }
        }
    }

    /// Test with different epsilons
    macro_rules! test_eps {
        ($fname: ident, $val: expr) => {
            #[test]
            fn $fname() {
                let mut test_case: PGMSegTestCase<$val> =
                    PGMSegTestCase::generate(10_000_000, None);
                test_case.train();
                test_case.test();
            }
        };
    }
    test_eps!(test_eps4, 4);
    test_eps!(test_eps8, 8);
    test_eps!(test_eps16, 16);
    test_eps!(test_eps64, 64);
}
