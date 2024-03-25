//! This file contains experimental work incorporating ideas
//! from the ALEX index into limosuine.
//! For the sake of being able to be understood on it's own,
//! it contains some duplicated code.

use super::gapped_array::GappedKVArray;
use crate::Entry;
use generational_arena::{Arena, Index};
use itertools::Itertools;
use num::Float;
use std::{fs, ops::Range};
use trait_set::trait_set;

pub type GappedKey = i32;
#[derive(Copy, Debug, Ord, Eq)]
pub struct GappedIndex(Index);
trait_set! {
    /// General value type, thread-safe
    pub trait GappedValue = Copy + Default + PartialOrd + Ord + std::fmt::Debug + 'static;
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

#[derive(Debug)]
pub struct GappedPGMNode<V: GappedValue, const EPSILON: usize, const BUFSIZE: usize> {
    pub height: u32,
    pub ga: GappedKVArray<GappedKey, V>,
    pub buffer: [Entry<GappedKey, V>; BUFSIZE],
    pub buff_ix: usize,
    pub model: LinearModel,
    pub split_density: f32,
    pub parent: Option<GappedIndex>,
}
impl<V: GappedValue, const EPSILON: usize, const BUFSIZE: usize> GappedPGMNode<V, EPSILON, BUFSIZE> {
    pub const fn is_leaf(&self) -> bool {
        self.height == 0
    }

    pub const fn is_internal(&self) -> bool {
        !self.is_leaf()
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

    /// TODO: Make this an iterator!!!
    pub fn to_entries(&self) -> Vec<Entry<GappedKey, V>> {
        let mut result = vec![];
        let mut ix = self.ga.next_occupied_ix(0);
        while let Some(jx) = ix {
            result.push(Entry::new(self.ga.keys[jx], self.ga.vals[jx]));
            ix = self.ga.next_occupied_ix(jx + 1);
        }
        result
    }

    pub fn scale_up(&mut self, c: f32) -> Result<(), String> {
        if c <= 1.0 {
            return Err("Must scale by a constant c > 1.0".to_string());
        }
        self.model.scale(1.0 / c);
        self.ga.scale_up(c)
    }

    pub fn upsert(&mut self, entry: Entry<GappedKey, V>) -> Result<(), String> {
        if self.is_internal() && self.ga.is_full() {
            self.scale_up(2.0);
        }
        // First we need to check if this key is in the buffer to perform an update there
        for ix in 0..BUFSIZE {
            if ix >= self.buff_ix {
                break;
            }
            let buf_ent = self.buffer[ix];
            if buf_ent.key == entry.key {
                self.buffer[ix] = entry;
                return Ok(());
            }
        }
        let guess = self.model.approximate(entry.key, Some(0..self.ga.len()));
        self.ga.upsert_with_hint((entry.key, entry.value), guess)?;
        Ok(())
    }

    pub fn is_buffer_full(&self) -> bool {
        self.buff_ix >= BUFSIZE
    }

    pub fn insert_into_buffer(&mut self, entry: Entry<GappedKey, V>) -> Result<(), String> {
        if self.buff_ix >= BUFSIZE {
            Err("Buffer is full".to_string())
        } else {
            self.buffer[self.buff_ix] = entry;
            self.buff_ix += 1;
            Ok(())
        }
    }

    pub fn trim_window(&mut self, key: GappedKey, window_radius: u32) -> Result<Vec<V>, String> {
        let guess = self.model.approximate(key, Some(0..self.ga.len()));
        self.ga.trim_window(key, window_radius, guess)
    }
}
impl<const EPSILON: usize, const BUFSIZE: usize> GappedPGMNode<GappedIndex, EPSILON, BUFSIZE> {
    /// TODO: Make this an iterator!!!
    pub fn to_children(&self) -> Vec<GappedIndex> {
        let mut result = vec![];
        let mut ix = self.ga.next_occupied_ix(0);
        while let Some(jx) = ix {
            result.push(self.ga.vals[jx]);
            ix = self.ga.next_occupied_ix(jx + 1);
        }
        result
    }
}

pub fn build_layer_from_slice<V: GappedValue, const EPSILON: usize, const BUFSIZE: usize>(
    entries: &[Entry<GappedKey, V>],
    fill_density: f32,
    split_density: f32,
    height: u32,
) -> Vec<GappedPGMNode<V, EPSILON, BUFSIZE>> {
    // Helper function
    let mut ship_new_node = |segmentor: &SimplePGMSegmentator<EPSILON>,
                             next_ix: usize,
                             built_ix: &mut usize,
                             result: &mut Vec<GappedPGMNode<V, EPSILON, BUFSIZE>>| {
        let mut model = segmentor.to_linear_model();
        model.scale(1.0 / fill_density);
        let num_initial = next_ix - *built_ix;
        let bloated_size = ((num_initial as f32) / fill_density).ceil() as usize;
        let mut ga = GappedKVArray::<GappedKey, V>::new(bloated_size);
        for add_ix in *built_ix..next_ix {
            let add_entry = entries[add_ix];
            let guess = model.approximate(add_entry.key, Some(0..bloated_size));
            ga.initial_model_based_insert((add_entry.key, add_entry.value), guess);
        }
        let new_node = GappedPGMNode {
            height,
            ga,
            buffer: [Entry::default(); BUFSIZE],
            buff_ix: 0,
            model,
            split_density,
            parent: None,
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

pub struct GappedPGM<
    V: GappedValue,
    const INT_EPS: usize,
    const LEAF_EPS: usize,
    const LEAF_BUFSIZE: usize,
    const LEAF_FILL_DEC: u8,
    const LEAF_SPLIT_DEC: u8,
> {
    pub height: u32,
    pub internal_arena: Arena<GappedPGMNode<GappedIndex, INT_EPS, 0>>,
    pub leaf_arena: Arena<GappedPGMNode<V, LEAF_EPS, LEAF_BUFSIZE>>,
    pub root_ptr: Option<GappedIndex>,
    pub leaf_fill_density: f32,
    pub leaf_split_density: f32,
    pub internal_fill_density: f32,
    pub internal_split_density: f32,
    pub leaf_window_radius: u32, // TODO: Make this a generic constant, just lazy
}
struct ConnLink {
    pub node_ptr: GappedIndex,
    pub parent_ptr: Option<GappedIndex>,
    pub height: u32,
}
impl<
        V: GappedValue,
        const INT_EPS: usize,
        const LEAF_EPS: usize,
        const LEAF_BUFSIZE: usize,
        const LEAF_FILL_DEC: u8,
        const LEAF_SPLIT_DEC: u8,
    > GappedPGM<V, INT_EPS, LEAF_EPS, LEAF_BUFSIZE, LEAF_FILL_DEC, LEAF_SPLIT_DEC>
{
    pub fn to_string(&self) -> String {
        let cur_ptr = self.root_ptr.unwrap();
        if self.height < 1 {
            panic!("Can't plot degen trees");
        }
        // Initialize lol
        let mut lol = vec![];
        let root = self.get_internal_node(cur_ptr).unwrap();
        lol.push(vec![vec![(
            cur_ptr,
            Entry::new(root.to_entry().unwrap().key, V::default()),
            root.ga.to_string(),
        )]]);
        loop {
            let last_layer = lol.last().unwrap();
            let mut this_layer: Vec<Vec<(GappedIndex, Entry<GappedKey, V>, String)>> = vec![];
            let mut is_branch = false;
            for seq in last_layer {
                for (ptr, _, _) in seq {
                    let node = self.get_internal_node(*ptr).unwrap();
                    if node.is_branch() {
                        // Add the leafs properly
                        let mut ix: Option<usize> = node.ga.next_occupied_ix(0);
                        let mut this_vec = vec![];
                        while ix.is_some() {
                            let ptr = node.ga.vals[ix.unwrap()];
                            let leaf_node = self.get_leaf_node(ptr).unwrap();
                            this_vec.push((
                                ptr,
                                leaf_node.to_entry().unwrap(),
                                format!("{:?} - {}", leaf_node.parent, leaf_node.ga.to_string()),
                            ));
                            ix = node.ga.next_occupied_ix(ix.unwrap() + 1);
                        }
                        this_layer.push(this_vec);
                    } else {
                        // Add the internals properly
                        let mut ix: Option<usize> = node.ga.next_occupied_ix(0);
                        let mut this_vec = vec![];
                        while ix.is_some() {
                            let ptr = node.ga.vals[ix.unwrap()];
                            let int_node = self.get_internal_node(ptr).unwrap();
                            this_vec.push((
                                ptr,
                                Entry::new(int_node.to_entry().unwrap().key, V::default()),
                                int_node.ga.to_string(),
                            ));
                            ix = node.ga.next_occupied_ix(ix.unwrap() + 1);
                        }
                        this_layer.push(this_vec);
                    }
                    is_branch = is_branch || node.is_branch();
                }
            }
            lol.push(this_layer);
            if is_branch {
                break;
            }
        }
        let mut height = self.height as i32;
        let mut res = String::new();
        for layer in lol {
            res += &format!("HEIGHT: {}\n", height);
            for group in layer {
                res += &format!("[\n");
                for (_, _, s) in group {
                    res += &format!("  {}\n", s);
                }
                res += &format!("]\n");
            }
            res += &format!("\n");
            height -= 1;
        }
        res
    }

    pub fn to_file(&self, filename: &str) {
        fs::write(filename, self.to_string()).unwrap();
    }

    pub fn build_from_slice(entries: &[Entry<GappedKey, V>]) -> Self {
        let mut gapped_pgm = Self {
            height: 0,
            internal_arena: Arena::new(),
            leaf_arena: Arena::new(),
            root_ptr: None,
            leaf_fill_density: (LEAF_FILL_DEC as f32 / 10.0),
            leaf_split_density: (LEAF_SPLIT_DEC as f32) / 10.0,
            internal_fill_density: 0.8,
            internal_split_density: 0.9,
            leaf_window_radius: 2,
        };
        // Build the leaf layer
        let mut height = 0;
        let leaf_nodes: Vec<GappedPGMNode<V, LEAF_EPS, LEAF_BUFSIZE>> = build_layer_from_slice(
            entries,
            gapped_pgm.leaf_fill_density,
            gapped_pgm.leaf_split_density,
            height,
        );
        let mut next_entries: Vec<Entry<GappedKey, GappedIndex>> = vec![];
        for node in leaf_nodes {
            let key = node.model.key;
            let ptr = gapped_pgm.leaf_arena.insert(node);
            next_entries.push(Entry::new(key, GappedIndex(ptr)));
        }
        // Recursively build the internal layers
        while next_entries.len() > 1 {
            height += 1;
            let internal_nodes: Vec<GappedPGMNode<GappedIndex, INT_EPS, 0>> = build_layer_from_slice(
                &next_entries,
                gapped_pgm.internal_fill_density,
                gapped_pgm.internal_split_density,
                height,
            );
            next_entries.clear();
            for node in internal_nodes {
                let key = node.model.key;
                let ptr = gapped_pgm.internal_arena.insert(node);
                next_entries.push(Entry::new(key, GappedIndex(ptr)));
            }
        }
        gapped_pgm.root_ptr = Some(next_entries[0].value);
        gapped_pgm.height = height;
        // Connect parent pointers
        let mut stack: Vec<ConnLink> = vec![ConnLink {
            node_ptr: gapped_pgm.root_ptr.unwrap(),
            parent_ptr: None,
            height: gapped_pgm.height,
        }];
        while let Some(link) = stack.pop() {
            if link.height == 0 {
                // Leaf node
                let node = gapped_pgm.get_mut_leaf_node(link.node_ptr).unwrap();
                node.parent = link.parent_ptr;
            } else {
                // Internal node
                let node = gapped_pgm.get_mut_internal_node(link.node_ptr).unwrap();
                node.parent = link.parent_ptr;
                let children = node.to_children();
                for child in children {
                    stack.push(ConnLink {
                        node_ptr: child,
                        parent_ptr: Some(link.node_ptr),
                        height: link.height - 1,
                    })
                }
            }
        }
        gapped_pgm
    }

    pub fn get_internal_node(&self, ptr: GappedIndex) -> Option<&GappedPGMNode<GappedIndex, INT_EPS, 0>> {
        self.internal_arena.get(ptr.0)
    }

    pub fn get_mut_internal_node(&mut self, ptr: GappedIndex) -> Option<&mut GappedPGMNode<GappedIndex, INT_EPS, 0>> {
        self.internal_arena.get_mut(ptr.0)
    }

    pub fn get_leaf_node(&self, ptr: GappedIndex) -> Option<&GappedPGMNode<V, LEAF_EPS, LEAF_BUFSIZE>> {
        self.leaf_arena.get(ptr.0)
    }

    pub fn get_mut_leaf_node(&mut self, ptr: GappedIndex) -> Option<&mut GappedPGMNode<V, LEAF_EPS, LEAF_BUFSIZE>> {
        self.leaf_arena.get_mut(ptr.0)
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
        }
        // We're at the leaf node
        match self.get_leaf_node(ptr) {
            None => {
                panic!("Bad leaf nodes");
            }
            Some(node) => {
                let guess = node.model.approximate(needle, Some(0..node.ga.len()));
                let got = node.ga.search_exact(&needle, Some(guess));
                match got {
                    Some(v) => Some(v),
                    None => {
                        let mut result = None;
                        for ix in 0..LEAF_BUFSIZE {
                            if ix >= node.buff_ix {
                                break;
                            }
                            if node.buffer[ix].key == needle {
                                result = Some(&node.buffer[ix].value);
                            }
                        }
                        result
                    }
                }
            }
        }
    }

    pub fn upsert(&mut self, entry: Entry<GappedKey, V>) -> Result<(), String> {
        let mut ptr = self.root_ptr.unwrap();
        let mut height = self.height;
        // Recurse down to the leaf node
        while height > 0 {
            match self.get_internal_node(ptr) {
                None => {
                    panic!("Bad internal nodes");
                }
                Some(node) => {
                    let guess = node.model.approximate(entry.key, Some(0..node.ga.len()));
                    let next_ptr_option = node.ga.search_pir(&entry.key, Some(guess));
                    match next_ptr_option {
                        None => {
                            // If price-is-right returns None, it means this is smaller than everythign
                            // NOTE: We solved this by just adding a KEYMIN to every index, but this
                            // should be reflected everywhere
                            let Some(ix) = node.ga.next_occupied_ix(0) else {
                                return Err("Layer looks empty during insert".to_string());
                            };
                            ptr = node.ga.vals[ix];
                        }
                        Some(next_ptr) => ptr = *next_ptr,
                    }
                }
            }
            height -= 1;
        }
        // We're at the leaf node
        let leaf_split_density = self.leaf_split_density;
        match self.get_mut_leaf_node(ptr) {
            None => Err("Bad leaf nodes".to_string()),
            Some(node) => {
                // Can do a gapped insert
                if node.ga.density() >= leaf_split_density {
                    // Try to insert into the nodes buffer, only splitting if it's full
                    node.insert_into_buffer(entry)?;
                    if node.is_buffer_full() {
                        self.split_leaf(ptr)
                    } else {
                        Ok(())
                    }
                } else {
                    node.upsert(entry)
                }
            }
        }
    }

    fn split_leaf(&mut self, ptr: GappedIndex) -> Result<(), String> {
        let Some(leaf_node) = self.get_leaf_node(ptr) else {
            return Err("Leaf ptr doesn't exist for splitting".to_string());
        };
        let leaf_key = leaf_node.to_entry().unwrap().key;
        let Some(parent_ptr) = leaf_node.parent else {
            return Err("TODO: Single leaf splitting not yet supported".to_string());
        };
        let leaf_window_radius = self.leaf_window_radius;
        let Some(parent_node) = self.get_mut_internal_node(parent_ptr) else {
            return Err("Bad parent when splitting leaf".to_string());
        };
        // NOTE: Includes self
        let fell_ptrs = parent_node.trim_window(leaf_key, leaf_window_radius).unwrap();
        let mut entries: Vec<Entry<GappedKey, V>> = vec![];
        for ptr in fell_ptrs {
            let killed_node = self.leaf_arena.remove(ptr.0).unwrap();
            let phoenix_entries = killed_node.to_entries();
            if LEAF_BUFSIZE == 0 {
                entries.extend(phoenix_entries.into_iter());
            } else {
                let mut buf_phoenix_entries = killed_node.buffer.into_iter().take(killed_node.buff_ix).collect_vec();
                buf_phoenix_entries.sort();
                // Sorted merge
                let mut combined = Vec::with_capacity(phoenix_entries.len() + buf_phoenix_entries.len());
                let mut core_val = phoenix_entries.into_iter().peekable();
                let mut buf_val = buf_phoenix_entries.into_iter().peekable();
                while let (Some(core), Some(buf)) = (core_val.peek(), buf_val.peek()) {
                    if core < buf {
                        combined.push(core_val.next().unwrap());
                    } else {
                        combined.push(buf_val.next().unwrap());
                    }
                }
                combined.extend(core_val);
                combined.extend(buf_val);
                entries.extend(combined.into_iter());
            }
        }
        let mut new_nodes = build_layer_from_slice(&entries, self.leaf_fill_density, self.leaf_split_density, 0);
        for node in new_nodes.iter_mut() {
            node.parent = Some(parent_ptr);
        }
        let mut keys_n_ptrs = vec![];
        for node in new_nodes {
            let key = node.to_entry().unwrap().key;
            let ptr = GappedIndex(self.leaf_arena.insert(node));
            keys_n_ptrs.push((key, ptr));
        }
        // Structure is a bit weird but the borrow checker likes it
        let Some(parent_node) = self.get_mut_internal_node(parent_ptr) else {
            return Err("Bad parent when splitting leaf".to_string());
        };
        for (key, ptr) in keys_n_ptrs {
            parent_node.upsert(Entry::new(key, ptr))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod gapped_pgm_tests {
    use std::{collections::HashSet, time::Instant};

    use kdam::{tqdm, BarExt};
    use rand::{distributions::Uniform, rngs::StdRng, Rng, SeedableRng};

    use super::*;

    /// Helper function to generate uniformly random inserts
    fn generate_random_entries(size: usize, seed: Option<u64>, sorted: bool) -> Vec<Entry<i32, i32>> {
        let range = Uniform::from((GappedKey::MIN)..(GappedKey::MAX));
        let mut random_values: Vec<i32> = match seed {
            Some(val) => StdRng::seed_from_u64(val).sample_iter(&range).take(size).collect(),
            None => rand::thread_rng().sample_iter(&range).take(size).collect(),
        };
        let mut dehash = HashSet::new();
        let mut deduped = vec![];
        for v in random_values {
            if !dehash.contains(&v) {
                dehash.insert(v);
                deduped.push(v);
            }
        }
        let mut random_values = deduped;
        if sorted {
            random_values.sort();
        }
        let mut entries: Vec<Entry<GappedKey, i32>> = random_values
            .into_iter()
            .enumerate()
            .map(|(ix, key)| Entry::new(key, ix as i32 + 1))
            .collect();
        // TODO: Get rid of this quirk where we alwas need the key min
        entries.insert(0, Entry::new(i32::MIN, 0));
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
            let entries = generate_random_entries(size, seed, true);
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
            self.nodes = build_layer_from_slice::<i32, EPSILON, BUFSIZE>(&self.entries, 0.5, 0.8, 0);
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
            let entries = generate_random_entries(10_000_000, Some(seed), true);
            let gapped_pgm: GappedPGM<i32, 4, 64, 4, 5, 8> = GappedPGM::build_from_slice(&entries);
            let mut pb = tqdm!(total = entries.len());
            for entry in entries {
                let val = gapped_pgm.search(entry.key);
                assert!(*val.unwrap() == entry.value);
                pb.update(1);
            }
        }
    }

    #[test]
    fn test_gapped_pgm_parents() {
        let gen_seed = 1;
        let entries = generate_random_entries(100_000, Some(gen_seed), true);
        let mut gapped_pgm: GappedPGM<i32, 4, 4, 4, 5, 8> = GappedPGM::build_from_slice(&entries);
        for entry in entries {
            let mut ptr = gapped_pgm.root_ptr.unwrap();
            let mut height = gapped_pgm.height;
            // Recurse down to the leaf node
            while height > 0 {
                match gapped_pgm.get_internal_node(ptr) {
                    None => {
                        panic!("Nope test bad parents");
                    }
                    Some(node) => {
                        let guess = node.model.approximate(entry.key, Some(0..node.ga.len()));
                        let child_ptr = node.ga.search_pir(&entry.key, Some(guess)).unwrap();
                        if node.is_branch() {
                            let child = gapped_pgm.get_leaf_node(*child_ptr).unwrap();
                            let (x1, y1) = ptr.0.into_raw_parts();
                            let (x2, y2) = child.parent.unwrap().0.into_raw_parts();
                            assert!(x1 == x2);
                            assert!(y1 == y2);
                        } else {
                            let child = gapped_pgm.get_internal_node(*child_ptr).unwrap();
                            let (x1, y1) = ptr.0.into_raw_parts();
                            let (x2, y2) = child.parent.unwrap().0.into_raw_parts();
                            assert!(x1 == x2);
                            assert!(y1 == y2);
                        }
                        ptr = *child_ptr;
                    }
                }
                height -= 1;
            }
        }
    }

    #[test]
    fn test_gapped_pgm_update() {
        let gen_seed = 1;
        let entries = generate_random_entries(100_000, Some(gen_seed), true);
        let mut gapped_pgm: GappedPGM<i32, 4, 64, 4, 5, 8> = GappedPGM::build_from_slice(&entries);
        for entry in entries.iter() {
            gapped_pgm.upsert(Entry::new(entry.key, entry.value + 1));
        }
        for entry in entries.iter() {
            let val = gapped_pgm.search(entry.key);
            assert!(*val.unwrap() == entry.value + 1);
        }
    }

    #[test]
    fn test_basic_gapped_pgm_insert() {
        let gen_seed = 1;
        let entries = generate_random_entries(1_000, Some(gen_seed), true);
        let mut gapped_pgm: GappedPGM<i32, 4, 4, 4, 5, 8> = GappedPGM::build_from_slice(&entries);
        let ins_seed = 2;
        let additional = generate_random_entries(100_000, Some(ins_seed), false);
        println!("Inserting:");
        let mut pb = tqdm!(total = additional.len());
        for entry in additional.iter() {
            gapped_pgm.upsert(entry.clone()).unwrap();
            pb.update(1);
        }
        println!("Search additional:");
        let mut pb = tqdm!(total = additional.len());
        let mut additional_set = HashSet::new();
        for entry in additional.iter() {
            additional_set.insert(entry.key);
            let val = gapped_pgm.search(entry.key);
            assert!(*val.unwrap() == entry.value);
            pb.update(1);
        }
        println!("Search original:");
        let mut pb = tqdm!(total = entries.len());
        for entry in entries.iter() {
            pb.update(1);
            if additional_set.contains(&entry.key) {
                continue;
            }
            let val = gapped_pgm.search(entry.key);
            assert!(*val.unwrap() == entry.value);
        }
    }

    // #[test]
    // fn debug_gapped_pgm_insert() {
    //     let gen_seed = 1;
    //     let add_seed = 2;
    //     let mut entries = generate_random_entries(32, Some(gen_seed), true);
    //     let additional = generate_random_entries(100, Some(add_seed), false);
    //     let mut gapped_pgm: GappedPGM<i32, 3, 3, 0, 5, 8> = GappedPGM::build_from_slice(&entries);
    //     gapped_pgm.to_file("zDebug/_initial.out");
    //     for i in 0..100 {
    //         gapped_pgm.upsert(additional[i]);
    //         let file_name = format!("{}{}.out", "zDebug/", i);
    //         gapped_pgm.to_file(&file_name);
    //     }
    // }
}
