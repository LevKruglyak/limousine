use crate::BenchVal;
use itertools::Itertools;
use limousine_core::learned::pgm::gapped::gapped_pgm::{GappedKey, GappedPGM};
use limousine_core::Entry;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
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

    fn new_uniform_workload(seed: u64, num_initial: usize, num_upserts: usize, num_bad_reads: usize) -> Self {
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
        let folder = format!("src/learned/pgm/gapped/bench/crystallized/{}", self.name);
        fs::create_dir(&folder).ok();

        for (filename, data) in [
            ("initial", &self.initial),
            ("upserts", &self.upserts),
            ("reads", &self.reads),
        ] {
            let mut fout = File::create(format!("{}/{}.out", &folder, filename)).unwrap();
            let bulk_size = 100_000;
            let mut dit = data.iter().peekable();
            while dit.peek().is_some() {
                let mut writing = String::new();
                for _ in 0..bulk_size {
                    match dit.next() {
                        Some(val) => {
                            writing += &format!("{},{}\n", val.key, val.value);
                        }
                        None => break,
                    };
                }
                write!(fout, "{}", writing).ok();
            }
        }

        if VERBOSE {
            println!(
                "Workload saved to {}\n",
                format!("src/learned/pgm/gapped/bench/crystallized/{}", folder)
            );
        }
    }

    pub fn load(seed: u64, num_initial: usize, num_upserts: usize, num_bad_reads: usize) -> Result<Self, String> {
        if VERBOSE {
            println!("LOADING WORKLOAD");
        }
        let name = Self::get_name(seed, num_initial, num_upserts, num_bad_reads);
        let folder = format!("src/learned/pgm/gapped/bench/crystallized/{}", name);

        let Ok(initial_fin) = File::open(format!("{}/{}", folder, "initial.out")) else {
            return Err("No initial".to_string());
        };

        let Ok(upserts_fin) = File::open(format!("{}/{}", folder, "upserts.out")) else {
            return Err("No upserts".to_string());
        };

        let Ok(reads_fin) = File::open(format!("{}/{}", folder, "reads.out")) else {
            return Err("No reads".to_string());
        };

        let mut res = Self {
            name: name.clone(),
            initial: vec![],
            upserts: vec![],
            reads: vec![],
        };

        let mut initial_reader = BufReader::new(initial_fin).lines();
        while let Some(Ok(line)) = initial_reader.next() {
            let mut parts = line.split(',');
            let key = parts.next().unwrap().parse::<GappedKey>().unwrap();
            let val = parts.next().unwrap().parse::<BenchVal>().unwrap();
            res.initial.push(Entry::new(key, val));
        }

        let mut upserts_reader = BufReader::new(upserts_fin).lines();
        while let Some(Ok(line)) = upserts_reader.next() {
            let mut parts = line.split(',');
            let key = parts.next().unwrap().parse::<GappedKey>().unwrap();
            let val = parts.next().unwrap().parse::<BenchVal>().unwrap();
            res.upserts.push(Entry::new(key, val));
        }

        let mut reads_reader = BufReader::new(reads_fin).lines();
        while let Some(Ok(line)) = reads_reader.next() {
            let mut parts = line.split(',');
            let key = parts.next().unwrap().parse::<GappedKey>().unwrap();
            let val = parts.next().unwrap().parse::<BenchVal>().unwrap();
            res.reads.push(Entry::new(key, val));
        }

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

    /// Attempts to get a uniform workload first by loading from filesystem, creating it
    /// if it doesn't exist
    pub fn get_uniform_workload(seed: u64, num_initial: usize, num_upserts: usize, num_bad_reads: usize) -> Self {
        match Self::load(seed, num_initial, num_upserts, num_bad_reads) {
            Ok(wk) => wk,
            Err(_) => {
                let result = Self::new_uniform_workload(seed, num_initial, num_upserts, num_bad_reads);
                result.save();
                result
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct ExecutionResult {
    initial_size: u128,
    build_time: u128,
    upsert_time: u128,
    read_time: u128,
    final_size: u128,
}
impl ExecutionResult {
    pub fn help_fill_row(&self, map: &mut HashMap<String, u128>) {
        map.insert("initial_size".to_string(), self.initial_size);
        map.insert("build_time".to_string(), self.build_time);
        map.insert("upsert_time".to_string(), self.build_time);
        map.insert("read_time".to_string(), self.build_time);
        map.insert("final_size".to_string(), self.final_size);
    }
}

pub trait Executor<const VERBOSE: bool, const VERIFY: bool> {
    #[must_use]
    fn measure(&mut self, wk: &Workload<VERBOSE, VERIFY>) -> ExecutionResult;

    fn help_fill_row(&self, map: &mut HashMap<String, u128>);
}

impl<
        const VERBOSE: bool,
        const VERIFY: bool,
        const INT_EPS: usize,
        const LEAF_EPS: usize,
        const LEAF_BUFSIZE: usize,
        const LEAF_FILL_DEC: u8,
        const LEAF_SPLIT_DEC: u8,
    > Executor<VERBOSE, VERIFY>
    for GappedPGM<BenchVal, INT_EPS, LEAF_EPS, LEAF_BUFSIZE, LEAF_FILL_DEC, LEAF_SPLIT_DEC>
{
    #[must_use]
    fn measure(&mut self, wk: &Workload<VERBOSE, VERIFY>) -> ExecutionResult {
        let mut result = ExecutionResult::default();

        if VERBOSE {
            println!("BUILDING...");
        }
        let before_build = Instant::now();
        *self = Self::build_from_slice(&wk.initial);
        result.build_time = before_build.elapsed().as_micros();
        result.initial_size = self.size_in_bytes();
        if VERBOSE {
            println!("Finished building in {}us\n", result.build_time);
        }

        if VERBOSE {
            println!("UPSERTING...");
        }
        let before_upserts = Instant::now();
        for entry in wk.upserts.iter() {
            if VERIFY {
                assert!(self.upsert(entry.clone()).is_ok());
            } else {
                self.upsert(entry.clone()).ok();
            }
        }
        result.upsert_time = before_upserts.elapsed().as_micros();
        result.final_size = self.size_in_bytes();
        if VERBOSE {
            println!("Finished upserting in {}us\n", result.upsert_time);
        }

        if VERBOSE {
            println!("READING...");
        }
        let before_reads = Instant::now();
        for entry in wk.reads.iter() {
            if VERIFY {
                let val = self.search(entry.key);
                if entry.value == BenchVal::MIN {
                    // A bad read which we acknowledge may not exist
                    continue;
                }
                assert_eq!(val, Some(&entry.value));
            } else {
                self.search(entry.value);
            }
        }
        result.read_time = before_reads.elapsed().as_micros();
        if VERBOSE {
            println!("Finished reading in {}us\n", result.read_time);
        }

        result
    }

    fn help_fill_row(&self, map: &mut HashMap<String, u128>) {
        map.insert("INT_EPS".to_string(), INT_EPS as u128);
        map.insert("LEAF_EPS".to_string(), LEAF_EPS as u128);
        map.insert("LEAF_BUFSIZE".to_string(), LEAF_BUFSIZE as u128);
        map.insert("LEAF_FILL_DEC".to_string(), LEAF_FILL_DEC as u128);
        map.insert("LEAF_SPLIT_DEC".to_string(), LEAF_SPLIT_DEC as u128);
    }
}

#[cfg(test)]
mod test_workloads {
    use super::*;

    #[test]
    fn test_gen_save_load() {
        let seed = 0;
        let num_initial = 50_000_000;
        let num_upserts = 50_000_000;
        let num_bad_reads = 500_000;
        let wk = Workload::<true, true>::get_uniform_workload(seed, num_initial, num_upserts, num_bad_reads);
        wk.save();
        let wk_prime = Workload::<true, true>::load(seed, num_initial, num_upserts, num_bad_reads).unwrap();
        assert_eq!(wk, wk_prime);
    }

    #[test]
    fn test_workload_execute() {
        let seed = 0;
        let num_initial = 100_000;
        let num_upserts = 100_000;
        let num_bad_reads = 10_000;
        let wk = Workload::<true, true>::get_uniform_workload(seed, num_initial, num_upserts, num_bad_reads);
        let mut model = GappedPGM::<BenchVal, 8, 64, 16, 5, 8>::blank();
        let result = model.measure(&wk);
        println!("Measured: {:?}", result);
    }
}
