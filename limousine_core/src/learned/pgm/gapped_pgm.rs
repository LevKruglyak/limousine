//! This file contains experimental work incorporating ideas
//! from the ALEX index into limosuine.
//! For the sake of being able to be understood on it's own,
//! it contains some duplicated code.

use super::gapped_array::GappedKVArray;
use crate::Entry;
use generational_arena::{Arena, Index};
use num::Float;
use std::ops::Range;
use trait_set::trait_set;

pub type GappedKey = i32;
#[derive(Copy, Debug)]
pub struct GappedIndex(Index);
trait_set! {
    /// General value type, thread-safe
    pub trait GappedValue = Copy + Default + PartialOrd + std::fmt::Debug + 'static;
}

#[derive(PartialEq, PartialOrd, Default)]
struct GappedEntry<V: GappedValue> {
    key: GappedKey,
    value: V,
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

impl Default for GappedIndex {
    fn default() -> Self {
        Self(Index::from_raw_parts(0, 0))
    }
}

impl Clone for GappedIndex {
    fn clone(&self) -> Self {
        let raw_parts = self.0.into_raw_parts();
        Self(Index::from_raw_parts(raw_parts.0, raw_parts.1))
    }
}

impl PartialEq for GappedIndex {
    fn eq(&self, other: &Self) -> bool {
        // Should be irrelevant
        false
    }
}

impl PartialOrd for GappedIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Should be irrelevant
        Some(std::cmp::Ordering::Equal)
    }
}

impl<V: GappedValue> Default for Entry<GappedKey, V> {
    fn default() -> Self {
        Self {
            key: GappedKey::default(),
            value: V::default(),
        }
    }
}

impl<V: GappedValue> PartialOrd for Entry<GappedKey, V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

#[derive(Debug)]
pub struct GappedPGMNode<V: GappedValue, const EPSILON: usize, const BUFSIZE: usize> {
    pub height: u32,
    pub ga: GappedKVArray<GappedKey, V>,
    pub buffer: [Option<Entry<GappedKey, V>>; BUFSIZE],
    pub model: LinearModel,
    pub density: f32,
}
impl<V: GappedValue, const EPSILON: usize, const BUFSIZE: usize> GappedPGMNode<V, EPSILON, BUFSIZE> {
    pub const fn is_leaf(&self) -> bool {
        self.height == 0
    }

    pub const fn is_branch(&self) -> bool {
        self.height == 1
    }

    pub fn to_entry(&self) -> Option<Entry<GappedKey, V>> {
        match self.ga.next_occupied_ix(0) {
            Some(ix) => Some(Entry::new(self.ga.keys[ix], self.ga.vals[ix])),
            None => None,
        }
    }
}

pub fn build_layer_from_slice<V: GappedValue, const EPSILON: usize, const BUFSIZE: usize>(
    entries: &[Entry<GappedKey, V>],
    density: f32,
    height: u32,
) -> Vec<GappedPGMNode<V, EPSILON, BUFSIZE>> {
    // Helper function
    let mut ship_new_node = |segmentor: &SimplePGMSegmentator<EPSILON>,
                             next_ix: usize,
                             built_ix: &mut usize,
                             result: &mut Vec<GappedPGMNode<V, EPSILON, BUFSIZE>>| {
        let mut model = segmentor.to_linear_model();
        model.scale(1.0 / density);
        let num_initial = next_ix - *built_ix;
        let bloated_size = ((num_initial as f32) / density).ceil() as usize;
        let mut ga = GappedKVArray::<GappedKey, V>::new(bloated_size);
        for add_ix in *built_ix..next_ix {
            let add_entry = entries[add_ix];
            let guess = model.approximate(add_entry.key, Some(0..bloated_size));
            ga.initial_model_based_insert((add_entry.key, add_entry.value), guess);
        }
        let new_node = GappedPGMNode {
            height,
            ga,
            buffer: [None; BUFSIZE],
            model,
            density,
        };
        result.push(new_node);
        *built_ix = next_ix;
    };
    // Initialize
    let mut result: Vec<GappedPGMNode<V, EPSILON, BUFSIZE>> = vec![];
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
                // The segmentor needs to break off a new model
                ship_new_node(&segmentor, next_ix, &mut built_ix, &mut result);
                segmentor = SimplePGMSegmentator::<EPSILON>::new();
                segmentor.try_add_key(&entry.key).unwrap();
            }
        }
        next_ix += 1;
    }
}

pub struct GappedPGM<V: GappedValue, const INT_EPS: usize, const LEAF_EPS: usize, const LEAF_BUFSIZE: usize> {
    pub height: u32,
    pub internal_arena: Arena<GappedPGMNode<GappedIndex, INT_EPS, 0>>,
    pub leaf_arena: Arena<GappedPGMNode<V, LEAF_EPS, LEAF_BUFSIZE>>,
    pub root_ptr: Option<GappedIndex>,
}
impl<V: GappedValue, const INT_EPS: usize, const LEAF_EPS: usize, const LEAF_BUFSIZE: usize>
    GappedPGM<V, INT_EPS, LEAF_EPS, LEAF_BUFSIZE>
{
    pub fn build_from_slice(entries: &[Entry<GappedKey, V>]) -> Self {
        let mut gapped_pgm = Self {
            height: 0,
            internal_arena: Arena::new(),
            leaf_arena: Arena::new(),
            root_ptr: None,
        };
        // Build the leaf layer
        let mut height = 0;
        let leaf_nodes: Vec<GappedPGMNode<V, LEAF_EPS, LEAF_BUFSIZE>> = build_layer_from_slice(entries, 0.5, height);
        let mut next_entries: Vec<Entry<GappedKey, GappedIndex>> = vec![];
        for node in leaf_nodes {
            let key = node.model.key;
            let ptr = gapped_pgm.leaf_arena.insert(node);
            next_entries.push(Entry::new(key, GappedIndex(ptr)));
        }
        // Recursively build the internal layers
        while next_entries.len() > 1 {
            height += 1;
            let internal_nodes: Vec<GappedPGMNode<GappedIndex, INT_EPS, 0>> =
                build_layer_from_slice(&next_entries, 0.9, height);
            next_entries.clear();
            for node in internal_nodes {
                let key = node.model.key;
                let ptr = gapped_pgm.internal_arena.insert(node);
                next_entries.push(Entry::new(key, GappedIndex(ptr)));
            }
        }
        gapped_pgm.root_ptr = Some(next_entries[0].value);
        gapped_pgm.height = height;
        gapped_pgm
    }

    pub fn get_internal_node(&self, ptr: GappedIndex) -> Option<&GappedPGMNode<GappedIndex, INT_EPS, 0>> {
        self.internal_arena.get(ptr.0)
    }

    pub fn get_leaf_node(&self, ptr: GappedIndex) -> Option<&GappedPGMNode<V, LEAF_EPS, LEAF_BUFSIZE>> {
        self.leaf_arena.get(ptr.0)
    }

    pub fn search(&self, needle: GappedKey) -> Option<&V> {
        let mut ptr = self.root_ptr.unwrap();
        let mut height = self.height;
        // Recurse down to the leaf node
        while height > 0 {
            match self.get_internal_node(ptr) {
                None => {
                    panic!("Bad internal nodes");
                }
                Some(node) => {
                    let guess = node.model.approximate(needle, Some(0..node.ga.len()));
                    let next_ptr_option = node.ga.search_pir(&needle, Some(guess));
                    match next_ptr_option {
                        None => return None,
                        Some(next_ptr) => ptr = *next_ptr,
                    }
                }
            }
            height -= 1;
            // let ten_millis = std::time::Duration::from_millis(500);
            // let now = std::time::Instant::now();
            // std::thread::sleep(ten_millis);
        }
        assert!(height == 0);
        // We're at the leaf node
        match self.get_leaf_node(ptr) {
            None => {
                panic!("Bad leaf nodes");
            }
            Some(node) => {
                let guess = node.model.approximate(needle, Some(0..node.ga.len()));
                let got = node.ga.search_exact(&needle, Some(guess));
                got
            }
        }
    }
}

#[cfg(test)]
mod gapped_pgm_tests {
    use std::time::Instant;

    use kdam::{tqdm, BarExt};
    use rand::{distributions::Uniform, rngs::StdRng, Rng, SeedableRng};

    use super::*;

    fn generate_random_entries(size: usize, seed: Option<u64>) -> Vec<Entry<i32, i32>> {
        let range = Uniform::from((GappedKey::MIN)..(GappedKey::MAX));
        let mut random_values: Vec<i32> = match seed {
            Some(val) => StdRng::seed_from_u64(val).sample_iter(&range).take(size).collect(),
            None => rand::thread_rng().sample_iter(&range).take(size).collect(),
        };
        random_values.sort();
        random_values.dedup();
        let entries: Vec<Entry<GappedKey, i32>> = random_values
            .into_iter()
            .enumerate()
            .map(|(ix, key)| Entry::new(key, ix as i32))
            .collect();
        entries
    }

    /// To test with different epsilon we need a struct that can handle that generic
    struct PGMSegTestCase<const EPSILON: usize, const BUFSIZE: usize> {
        verbose: bool,
        entries: Vec<Entry<GappedKey, i32>>,
        nodes: Vec<GappedPGMNode<i32, EPSILON, BUFSIZE>>,
        last_model_ix: usize,
        last_base_rank: usize,
    }
    impl<const EPSILON: usize, const BUFSIZE: usize> PGMSegTestCase<EPSILON, BUFSIZE> {
        /// Generates a test key, meaning make the entries, sort + dedup them
        fn generate(size: usize, verbose: Option<bool>, seed: Option<u64>) -> Self {
            let verbose = verbose.unwrap_or(true);
            if verbose {
                println!("Generating {} entries with eps={}", size, EPSILON);
            }
            let entries = generate_random_entries(size, seed);
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
            let start_time = Instant::now();
            self.nodes = build_layer_from_slice::<i32, EPSILON, BUFSIZE>(&self.entries, 0.5, 0);
            let elapsed_time = start_time.elapsed();
            if self.verbose {
                println!("Training completed in {} ms.", elapsed_time.as_millis());
            }
        }

        /// Assuming data has already been generated and trained on, tests that every key is correctly approximated
        fn test(&mut self) {
            let mut cumulative_value = 0;
            for node in self.nodes.iter() {
                let mut ix = node.ga.next_occupied_ix(0);
                while ix.is_some() {
                    assert!(node.ga.vals[ix.unwrap()] == cumulative_value);
                    cumulative_value += 1;
                    ix = node.ga.next_occupied_ix(ix.unwrap() + 1);
                }
            }
        }
    }

    #[test]
    fn test_seg_eps64() {
        let mut test_case: PGMSegTestCase<64, 0> = PGMSegTestCase::generate(1_000_000, None, None);
        test_case.train();
        test_case.test();
    }

    #[test]
    fn test_gapped_pgm_build() {
        for seed in 0..10 {
            println!("seed: {:?}", seed);
            let entries = generate_random_entries(10_000_000, Some(seed));
            let gapped_pgm: GappedPGM<i32, 4, 64, 4> = GappedPGM::build_from_slice(&entries);
            let mut pb = tqdm!(total = entries.len());
            for entry in entries {
                let val = gapped_pgm.search(entry.key);
                assert!(*val.unwrap() == entry.value);
                pb.update(1);
            }
        }
    }
}
