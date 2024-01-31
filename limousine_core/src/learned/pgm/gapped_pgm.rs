// //! This file contains experimental work incorporating ideas
// //! from the ALEX index into limosuine.
// //! For the sake of being able to be understood on it's own,
// //! it contains some duplicated code.

// use crate::{Entry, Key, Value};
// use std::ops::Sub;

// /// Helper struct to deal with points
// #[derive(Clone)]
// struct Point<K: Key> {
//     x: K,
//     y: i32,
// }
// impl<K: Key> Point<K> {
//     fn new(x: K, y: i32) -> Self {
//         Self { x, y }
//     }

//     // Verbose but understandable slope
//     fn slope(&self) -> f32 {
//         let run = num::cast::<K, f32>(self.x).unwrap();
//         (self.y as f32) / run
//     }

//     // Verbose but understandable and quick slope to another point
//     fn slope_from(&self, other: &Self) -> f32 {
//         let rise = (self.y - other.y) as f32;
//         let run = num::cast::<K, f32>(self.x - other.x).unwrap();
//         rise / run
//     }
// }

// /// A simple linear model for a key-rank segment of data.
// /// NOTE: Forces intercept at origin for simplicity.
// #[derive(Copy, Clone, Debug)]
// pub struct LinearModel<K: Key> {
//     pub key: K,
//     pub slope: f32,
// }
// impl<K: Key> LinearModel<K> {
//     pub fn new(key: K, slope: f32) -> Self {
//         Self { key, slope }
//     }

//     pub fn scale(&mut self, density: f32) {
//         self.slope *= density;
//     }
// }

// /// A data structure that will grow to incorporate points while building a PGM and eventually
// /// produce a proper linear model, before moving on to the next one
// pub struct SimplePGMSegmentator<K: Key, const EPSILON: usize> {
//     pub first_k: Option<K>,
//     pub max_slope: f32,
//     pub min_slope: f32,
//     pub num_entries: usize,
// }
// impl<K: Key, const EPSILON: usize> SimplePGMSegmentator<K, EPSILON> {
//     pub fn new() -> Self {
//         Self {
//             first_k: None,
//             max_slope: f32::MAX,
//             min_slope: f32::MIN,
//             num_entries: 0,
//         }
//     }

//     /// Tries to add an entry to this segmentor, returning a result about whether it was
//     /// successful.
//     pub fn try_add_key(&mut self, key: &K) -> Result<(), ()> {
//         if self.num_entries == 0 {
//             // If it's empty just add the point
//             self.first_k = Some(key.clone());
//             self.num_entries = 1;
//             return Ok(());
//         }
//         // Get the worst case points we care about
//         let base_point = Point::new(self.first_k.unwrap(), 0);
//         let max_point = Point::new(
//             key.clone(),
//             self.num_entries
//                 .saturating_add(1) // The actual rank
//                 .saturating_sub(1) // To deal with floating point annoyances
//                 .saturating_add(EPSILON) as i32,
//         );
//         let min_point = Point::new(
//             key.clone(),
//             self.num_entries
//                 .saturating_add(1) // The actual rank
//                 .saturating_add(1) // To deal with floating point annoyances
//                 .saturating_sub(EPSILON) as i32,
//         );
//         let this_max = max_point.slope_from(&base_point);
//         let this_min = min_point.slope_from(&base_point);
//         if self.num_entries == 1 {
//             self.max_slope = this_max;
//             self.min_slope = this_min;
//         } else {
//             let new_max_slope = this_max.min(self.max_slope);
//             let new_min_slope = this_min.max(self.min_slope);
//             if new_min_slope >= new_max_slope {
//                 // We can't fit this point in the model
//                 return Err(());
//             }
//             self.max_slope = new_max_slope;
//             self.min_slope = new_min_slope;
//         }
//         // This point is fine to add, and we've already update the slope
//         self.num_entries += 1;
//         Ok(())
//     }

//     // Outputs a linear model that fits all the points presented so far
//     pub fn to_linear_model(&self) -> LinearModel<K> {
//         let slope = if self.num_entries > 1 {
//             (self.max_slope + self.min_slope) / 2.0
//         } else {
//             // A model that only has one point can pick any slope, we pick 1 arbitrarily
//             1.0
//         };
//         LinearModel::new(self.first_k.unwrap(), slope)
//     }

//     pub fn is_empty(&self) -> bool {
//         self.num_entries <= 0
//     }
// }

// pub struct GappedPGM<K: Key, V: Value, const EPSILON: usize, const BUFSIZE: usize> {
//     pub entries: Box<[Entry<K, V>]>,
//     pub buffer: [Entry<K, V>; BUFSIZE],
//     pub model: LinearModel<K>,
//     pub density: f32,
// }

// impl<K: Key, V: Value, const EPSILON: usize, const BUFSIE: usize> GappedPGM<K, V, EPSILON, BUFSIE> {
//     /// Creates an empty model with the given model and s
//     pub fn empty_from_model_n_size(model: LinearModel<K>, size: usize) {
//         let vec: Vec<Entry<K, V>> = vec![]
//         panic!("Unimplemented")
//     }

//     // pub fn from_entries_with_density(entries: &[Entry<K, V>], density: f32) -> Self {
//     //     panic!("unimplemented")
//     // }
// }

// pub fn build_from_slice<K: Key, V: Value, const EPSILON: usize, const BUFSIZE: usize>(
//     entries: &[Entry<K, V>],
//     density: f32,
// ) -> Vec<GappedPGM<K, V, EPSILON, BUFSIZE>> {
//     // Initialize
//     let result: Vec<GappedPGM<K, V, EPSILON, BUFSIZE>>;
//     let mut segmentor = SimplePGMSegmentator::<K, EPSILON>::new();
//     let built_ix = 0;
//     let next_ix = 0;
//     // Helper lambda function
//     let

//     loop {
//         if next_ix >= entries.len() {
//             // We ran out of items, stop building
//             if segmentor.is_empty() {
//                 // It's our lucky day, nothing left to build
//                 return result;
//             }

//             return result;
//         }

//         let status = segmentor.try_add_key();
//     }
// }

// /*
// What are the possibiliies here?

// Alex-inspired data nodes
// - Gapped array
// - Use simple linear regression + scaling to size
// - How to determine size? (List of options)
//     - Fixed size and density, i.e. take 20 nodes, linearly fit them, then scale to say 30 nodes
//     - Fixed error and density, i.e. train like PGM, then scale (FEELS GOOD)
//     - ^GOING WITH THE SECOND ONE

// So let's enumerate the parameters:

// density: (0, 1]
// - 1 is just a PGM
// - 0.5 means a PGM with a GAP of 2

// buffer size (BYTES): [0, \inf)
// - In practice 1KB or something would be highest number remotely reasonable
// - 0 is just standard alex
// - Non-zero is crammed

// merge granularity: [0, \inf)
// - 0 is standard alex
// - ~2 is crammed PGM
// - In practice 16 or something would be highest number remotely reasonable

// when to use buffer?
// - density based, i.e. when density is above some number
// - distance based, i.e. when nearest gap is above some number
//     - ^could be done efficiently with bithacks

// */
