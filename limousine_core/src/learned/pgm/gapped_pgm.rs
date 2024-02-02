//! This file contains experimental work incorporating ideas
//! from the ALEX index into limosuine.
//! For the sake of being able to be understood on it's own,
//! it contains some duplicated code.

use std::ops::Range;

use num::Float;

use super::gapped_array::GappedArray;
use crate::Entry;

// TODO: Use traits from lazy_static instead, they were being annoying tho
type GappedKey = i32;
type GappedValue = i32;

#[derive(PartialEq, PartialOrd, Default)]
struct GappedEntry {
    key: GappedKey,
    value: GappedValue,
}

/// Helper struct to deal with points
#[derive(Clone)]
struct Point {
    x: GappedKey,
    y: i32,
}
impl Point {
    fn new(x: GappedKey, y: i32) -> Self {
        Self { x, y }
    }

    // Verbose but understandable slope
    fn slope(&self) -> f32 {
        let run = self.x as f32;
        (self.y as f32) / run
    }

    // Verbose but understandable and quick slope to another point
    fn slope_from(&self, other: &Self) -> f32 {
        let rise = (other.y - self.y) as f32;
        let run = other.x as f32 - self.x as f32;
        rise / run
    }
}

/// A simple linear model for a key-rank segment of data.
/// NOTE: Forces intercept at origin for simplicity.
#[derive(Copy, Clone, Debug)]
pub struct LinearModel {
    pub key: GappedKey,
    pub slope: f32,
}
impl LinearModel {
    pub fn new(key: GappedKey, slope: f32) -> Self {
        Self { key, slope }
    }

    pub fn scale(&mut self, multiplier: f32) {
        self.slope *= multiplier;
    }

    pub fn approximate(&self, key: GappedKey, bound: Option<Range<usize>>) -> usize {
        let guess = (key.saturating_sub(self.key) as f32 * self.slope).round() as usize;
        match bound {
            Some(range) => guess.max(range.start).min(range.end.saturating_sub(1)),
            None => guess,
        }
    }
}

/// A data structure that will grow to incorporate points while building a PGM and eventually
/// produce a proper linear model, before moving on to the next one
pub struct SimplePGMSegmentator<const EPSILON: usize> {
    pub first_k: Option<GappedKey>,
    pub max_slope: f32,
    pub min_slope: f32,
    pub num_entries: usize,
}
impl<const EPSILON: usize> SimplePGMSegmentator<EPSILON> {
    pub fn new() -> Self {
        Self {
            first_k: None,
            max_slope: f32::MAX,
            min_slope: f32::MIN,
            num_entries: 0,
        }
    }

    /// Tries to add an entry to this segmentor, returning a result about whether it was
    /// successful.
    pub fn try_add_key(&mut self, key: &GappedKey) -> Result<(), ()> {
        if self.num_entries == 0 {
            // If it's empty just add the point
            self.first_k = Some(key.clone());
            self.num_entries = 1;
            return Ok(());
        }
        // Get the worst case points we care about
        let base_point = Point::new(self.first_k.unwrap(), 0);
        let max_point = Point::new(
            key.clone(),
            self.num_entries
                .saturating_add(1) // The actual rank
                .saturating_sub(1) // To deal with floating point annoyances
                .saturating_add(EPSILON) as i32,
        );
        let min_point = Point::new(
            key.clone(),
            self.num_entries
                .saturating_add(1) // The actual rank
                .saturating_add(1) // To deal with floating point annoyances
                .saturating_sub(EPSILON) as i32,
        );
        let this_max = max_point.slope_from(&base_point);
        let this_min = min_point.slope_from(&base_point);
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
            self.max_slope = new_max_slope;
            self.min_slope = new_min_slope;
        }
        // This point is fine to add, and we've already update the slope
        self.num_entries += 1;
        Ok(())
    }

    // Outputs a linear model that fits all the points presented so far
    pub fn to_linear_model(&self) -> LinearModel {
        let slope = if self.num_entries > 1 {
            (self.max_slope + self.min_slope) / 2.0
        } else {
            // A model that only has one point can pick any slope, we pick 1 arbitrarily
            1.0
        };
        LinearModel::new(self.first_k.unwrap(), slope)
    }

    pub fn is_empty(&self) -> bool {
        self.num_entries <= 0
    }
}

impl Default for Entry<GappedKey, GappedValue> {
    fn default() -> Self {
        Self {
            key: GappedKey::default(),
            value: GappedValue::default(),
        }
    }
}

impl PartialOrd for Entry<GappedKey, GappedValue> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

pub struct GappedPGM<const EPSILON: usize, const BUFSIZE: usize> {
    pub ga: GappedArray<Entry<GappedKey, GappedValue>>,
    pub buffer: [Option<Entry<GappedKey, GappedValue>>; BUFSIZE],
    pub model: LinearModel,
    pub density: f32,
}

pub fn build_from_slice<const EPSILON: usize, const BUFSIZE: usize>(
    entries: &[Entry<GappedKey, GappedValue>],
    density: f32,
) -> Vec<GappedPGM<EPSILON, BUFSIZE>> {
    // Helper function
    let mut ship_new_node = |segmentor: &SimplePGMSegmentator<EPSILON>,
                             next_ix: usize,
                             built_ix: &mut usize,
                             result: &mut Vec<GappedPGM<EPSILON, BUFSIZE>>| {
        let mut model = segmentor.to_linear_model();
        model.scale(1.0 / density);
        let num_initial = next_ix - *built_ix;
        let bloated_size = ((num_initial as f32) / density).ceil() as usize;
        let mut ga = GappedArray::<Entry<GappedKey, GappedValue>>::new(bloated_size);
        for add_ix in *built_ix..next_ix {
            let add_entry = entries[add_ix];
            let guess = model.approximate(add_entry.key, Some(0..bloated_size));
            ga.initial_model_based_insert(add_entry, guess);
        }
        let new_node = GappedPGM {
            ga,
            buffer: [None; BUFSIZE],
            model,
            density,
        };
        result.push(new_node);
        *built_ix = next_ix;
    };
    // Initialize
    let mut result: Vec<GappedPGM<EPSILON, BUFSIZE>> = vec![];
    let mut segmentor = SimplePGMSegmentator::<EPSILON>::new();
    let mut built_ix = 0;
    let mut next_ix = 0;
    loop {
        // Build
        if next_ix >= entries.len() {
            // We ran out of items, stop building
            if segmentor.is_empty() {
                // It's our lucky day, nothing left to build
                return result;
            }
            ship_new_node(&segmentor, next_ix, &mut built_ix, &mut result);
            return result;
        }
        let entry = entries[next_ix];
        let status = segmentor.try_add_key(&entry.key);
        match status {
            Ok(_) => {
                // All set, keep going
            }
            Err(_) => {
                ship_new_node(&segmentor, next_ix, &mut built_ix, &mut result);
                segmentor = SimplePGMSegmentator::<EPSILON>::new();
                segmentor.try_add_key(&entry.key).unwrap();
            }
        }
        next_ix += 1;
    }
}

#[cfg(test)]
mod gapped_pgm_tests {
    use kdam::{tqdm, BarExt};
    use rand::{distributions::Uniform, Rng};

    use super::*;

    /// To test with different epsilon we need a struct that can handle that generic
    struct PGMSegTestCase<const EPSILON: usize, const BUFSIZE: usize> {
        verbose: bool,
        entries: Vec<Entry<GappedKey, GappedValue>>,
        nodes: Vec<GappedPGM<EPSILON, BUFSIZE>>,
        last_model_ix: usize,
        last_base_rank: usize,
    }
    impl<const EPSILON: usize, const BUFSIZE: usize> PGMSegTestCase<EPSILON, BUFSIZE> {
        /// Generates a test key, meaning make the entries, sort + dedup them
        fn generate(size: usize, verbose: Option<bool>) -> Self {
            let verbose = verbose.unwrap_or(true);
            if verbose {
                println!("Generating {} entries with eps={}", size, EPSILON);
            }
            let range = Uniform::from((GappedKey::MIN)..(GappedKey::MAX));
            let mut random_values: Vec<i32> = rand::thread_rng().sample_iter(&range).take(size).collect();
            random_values.sort();
            random_values.dedup();
            let entries: Vec<Entry<GappedKey, GappedValue>> = random_values
                .into_iter()
                .enumerate()
                .map(|(ix, key)| Entry::new(key, ix as GappedValue))
                .collect();
            Self {
                entries,
                verbose,
                nodes: vec![],
                last_model_ix: 0,
                last_base_rank: 0,
            }
        }

        /// Assuming data has already been generated, segments it as a layer
        fn train(&mut self) {
            if self.verbose {
                println!("Training on {} entries with eps={}", self.entries.len(), EPSILON);
            }
            self.nodes = build_from_slice::<EPSILON, BUFSIZE>(&self.entries, 0.5);
        }

        // /// Helper function for determining if a single entry is approximated within bounds
        // fn is_entry_well_approximated(&mut self, entry: Entry<Key, Value>) -> bool {
        //     let mut model_ix = self.last_model_ix;
        //     let mut base_rank = self.last_base_rank;
        //     while model_ix < self.models.len().saturating_sub(1) {
        //         if self.models[model_ix + 1].key > entry.key {
        //             break;
        //         }
        //         base_rank += self.models[model_ix].size;
        //         model_ix += 1;
        //     }
        //     let range = self.models[model_ix].approximate(&entry.key);
        //     self.last_base_rank = base_rank;
        //     self.last_model_ix = model_ix;
        //     return base_rank + range.lo <= entry.value && entry.value < base_rank + range.hi;
        // }

        /// Assuming data has already been generated and trained on, tests that every key is correctly approximated
        fn test(&mut self) {
            let mut cumulative_value = 0;
            for node in self.nodes.iter() {
                println!("NEW NODE!");
                let mut ix = node.ga.next_occupied_ix(0);
                while ix.is_some() {
                    println!("{}, {}", cumulative_value, node.ga.data[ix.unwrap()].value);
                    assert!(node.ga.data[ix.unwrap()].value == cumulative_value);
                    if node.ga.data[ix.unwrap()].value != cumulative_value {
                        println!("BAD BAD BAD");
                        println!("BAD BAD BAD");
                        println!("BAD BAD BAD");
                        println!("BAD BAD BAD");
                    }
                    cumulative_value += 1;
                    ix = node.ga.next_occupied_ix(ix.unwrap() + 1);
                }
            }
        }
    }

    #[test]
    fn test_eps4() {
        let mut test_case: PGMSegTestCase<64, 4> = PGMSegTestCase::generate(1_000_000, None);
        test_case.train();
        test_case.test();
    }
}

/*
What are the possibiliies here?

Alex-inspired data nodes
- Gapped array
- Use simple linear regression + scaling to size
- How to determine size? (List of options)
    - Fixed size and density, i.e. take 20 nodes, linearly fit them, then scale to say 30 nodes
    - Fixed error and density, i.e. train like PGM, then scale (FEELS GOOD)
    - ^GOING WITH THE SECOND ONE

So let's enumerate the parameters:

density: (0, 1]
- 1 is just a PGM
- 0.5 means a PGM with a GAP of 2

buffer size (BYTES): [0, \inf)
- In practice 1KB or something would be highest number remotely reasonable
- 0 is just standard alex
- Non-zero is crammed

merge granularity: [0, \inf)
- 0 is standard alex
- ~2 is crammed PGM
- In practice 16 or something would be highest number remotely reasonable

when to use buffer?
- density based, i.e. when density is above some number
- distance based, i.e. when nearest gap is above some number
    - ^could be done efficiently with bithacks

*/
