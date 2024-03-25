use crate::BenchVal;
use itertools::Itertools;
use limousine_core::learned::pgm::gapped::gapped_pgm::{GappedKey, GappedPGM};
use limousine_core::Entry;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::Instant;
use std::{collections::HashSet, fs::File};

/// Helper function to generate uniformly random inserts
pub fn generate_random_entries(size: usize, seed: u64, sorted: bool) -> Vec<Entry<GappedKey, BenchVal>> {
    let mut rng = StdRng::seed_from_u64(seed as u64);
    let mut random_numbers = Vec::with_capacity(size);
    let mut included = HashSet::with_capacity(size / 1_000);
    while random_numbers.len() < size {
        let key = rng.gen::<GappedKey>();
        let val = rng.gen::<BenchVal>();
        if !included.contains(&key) {
            included.insert(key);
            random_numbers.push(Entry::new(key, val));
        }
    }
    if sorted {
        random_numbers.sort();
    }
    random_numbers
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Workload<const VERBOSE: bool, const VERIFY: bool> {
    name: String,
    pub initial: Vec<Entry<GappedKey, BenchVal>>,
    pub upserts: Vec<Entry<GappedKey, BenchVal>>,
    pub reads: Vec<Entry<GappedKey, BenchVal>>,
}
impl<const VERBOSE: bool, const VERIFY: bool> Workload<VERBOSE, VERIFY> {
    fn get_name(seed: u64, num_initial: usize, num_upserts: usize, num_bad_reads: usize) -> String {
        format!("s={}_ni={}_nu={}_nbr={}", seed, num_initial, num_upserts, num_bad_reads)
    }

    pub fn new_uniform_workload(seed: u64, num_initial: usize, num_upserts: usize, num_bad_reads: usize) -> Self {
        // Generate the data
        if VERBOSE {
            println!(
                "GENERATING WORKLOAD: (seed: {}, ninitial: {}, nupsert: {}, nbad: {})",
                seed, num_initial, num_upserts, num_bad_reads
            );
            println!("Generating data...");
        }
        let initial = generate_random_entries(num_initial, seed + 0, true);
        let upserts = generate_random_entries(num_upserts, seed + 1, false);
        // NOTE: Not _necessarily_ bad reads (the key might exist) but is unlikely
        let mut bad_reads = generate_random_entries(num_bad_reads, seed + 2, false);
        for e in bad_reads.iter_mut() {
            e.value = i32::MIN;
        }

        // Construct the correct read map in case we also want to verify this workload
        if VERBOSE {
            println!("Correcting reads...");
        }
        let mut read_map = HashMap::with_capacity(num_initial + num_upserts + num_bad_reads);
        let mut read_order = bad_reads.iter().map(|e| e.key).collect_vec();
        for e in initial.iter() {
            read_map.insert(e.key, e.value);
            read_order.push(e.key);
        }
        for e in upserts.iter() {
            read_map.insert(e.key, e.value);
            read_order.push(e.key);
        }
        let mut reads = read_order
            .iter()
            .map(|e| Entry::new(*e, *read_map.get(e).unwrap_or(&BenchVal::MIN)))
            .collect_vec();
        if VERBOSE {
            println!("Shuffling reads...");
        }
        let mut rng = StdRng::seed_from_u64(seed + 4);
        reads.shuffle(&mut rng);

        // Lump together
        if VERBOSE {
            println!("Workload generated!\n");
        }
        Self {
            name: Self::get_name(seed, num_initial, num_upserts, num_bad_reads),
            initial,
            upserts,
            reads,
        }
    }

    pub fn save(&self) {
        if VERBOSE {
            println!("SAVING WORKLOAD");
        }
        let mut fout = File::create(format!("src/learned/pgm/gapped/bench/crystallized/{}.ron", self.name)).unwrap();
        write!(fout, "{}", ron::to_string(&self).unwrap()).unwrap();
        if VERBOSE {
            println!(
                "Workload saved to {}\n",
                format!("src/learned/pgm/gapped/bench/crystallized/{}.ron", self.name)
            );
        }
    }

    pub fn load(seed: u64, num_initial: usize, num_upserts: usize, num_bad_reads: usize) -> Result<Self, String> {
        if VERBOSE {
            println!("LOADING WORKLOAD");
        }
        let name = Self::get_name(seed, num_initial, num_upserts, num_bad_reads);
        let Ok(mut fin) = File::open(format!("src/learned/pgm/gapped/bench/crystallized/{}.ron", name)) else {
            return Err("No such file".to_string());
        };
        let mut buf = String::new();
        let Ok(_) = fin.read_to_string(&mut buf) else {
            return Err("Can't read file".to_string());
        };
        let Ok(res) = ron::from_str::<Self>(&buf) else {
            return Err("Ron can't deserialize".to_string());
        };
        if VERBOSE {
            println!(
                "WORKLOAD LOADED: (initial: {}, upserts: {}, reads: {})\n",
                res.initial.len(),
                res.upserts.len(),
                res.reads.len()
            );
        }
        Ok(res)
    }
}

#[derive(Default, Debug)]
pub struct ExecutionResult {
    build_time: u128,
    upsert_time: u128,
    read_time: u128,
}
pub trait Executor<const VERBOSE: bool, const VERIFY: bool> {
    fn measure(wk: &Workload<VERBOSE, VERIFY>) -> ExecutionResult;
}

impl<
        const VERBOSE: bool,
        const VERIFY: bool,
        const INT_EPS: usize,
        const LEAF_EPS: usize,
        const LEAF_BUFSIZE: usize,
    > Executor<VERBOSE, VERIFY> for GappedPGM<BenchVal, INT_EPS, LEAF_EPS, LEAF_BUFSIZE>
{
    fn measure(wk: &Workload<VERBOSE, VERIFY>) -> ExecutionResult {
        let mut result = ExecutionResult::default();

        if VERBOSE {
            println!("BUILDING...");
        }
        let before_build = Instant::now();
        let mut pgm = Self::build_from_slice(&wk.initial);
        result.build_time = before_build.elapsed().as_micros();
        if VERBOSE {
            println!("Finished building in {}us\n", result.build_time);
        }

        if VERBOSE {
            println!("UPSERTING...");
        }
        let before_upserts = Instant::now();
        for entry in wk.upserts.iter() {
            if VERIFY {
                assert!(pgm.upsert(entry.clone()).is_ok());
            } else {
                pgm.upsert(entry.clone()).ok();
            }
        }
        result.upsert_time = before_upserts.elapsed().as_micros();
        if VERBOSE {
            println!("Finished upserting in {}us\n", result.upsert_time);
        }

        if VERBOSE {
            println!("READING...");
        }
        let before_reads = Instant::now();
        for entry in wk.reads.iter() {
            if VERIFY {
                let val = pgm.search(entry.key);
                if entry.value == BenchVal::MIN {
                    // A bad read which we acknowledge may not exist
                    continue;
                }
                assert_eq!(val, Some(&entry.value));
            } else {
                pgm.search(entry.value);
            }
        }
        result.read_time = before_reads.elapsed().as_micros();
        if VERBOSE {
            println!("Finished reading in {}us\n", result.read_time);
        }

        result
    }
}

#[cfg(test)]
mod test_workloads {
    use super::*;

    #[test]
    fn test_gen_save_load() {
        let seed = 0;
        let num_initial = 100_000;
        let num_upserts = 100_000;
        let num_bad_reads = 10_000;
        let wk = Workload::<true, true>::new_uniform_workload(0, num_initial, num_upserts, num_bad_reads);
        wk.save();
        let wk_prime = Workload::<true, true>::load(seed, num_initial, num_upserts, num_bad_reads).unwrap();
        assert_eq!(wk, wk_prime);
    }
}
