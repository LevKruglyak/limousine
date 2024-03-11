use average::{Estimate, MeanWithError};
use clap::ValueEnum;
use itertools::Itertools;
use limousine_engine::prelude::*;
use mmap_buffer::Buffer;
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};
use rand_distr::Uniform;
use rayon::prelude::*;
use std::{fs, hint::black_box, path::Path, time::Instant};

use crate::baseline::{BTreeMapBaseline, SortedBaseline};

pub mod baseline;

limousine_macros::create_immutable_hybrid_index! {
    name: HybridIndex1,
    layout: {
        _ => btree(8),
    }
}

limousine_macros::create_immutable_hybrid_index! {
    name: HybridIndex2,
    layout: {
        0 => btree(8),
        _ => pgm(32),
    }
}

limousine_macros::create_immutable_hybrid_index! {
    name: HybridIndex3,
    layout: {
        0 => pgm(32),
        _ => btree(8),
    }
}

limousine_macros::create_immutable_hybrid_index! {
    name: HybridIndex4,
    layout: {
        _ => pgm(32),
    }
}

fn main() {
    println!("generating data... (will be cached for future runs)");

    if fs::metadata("data/main.dat").is_err() {
        gen_data(200_000_000, "data/main.dat", Distributions::Uniform);
    }

    let num_entries = 100_000_000;
    println!("benchmaring average point query latency in ns over 100_000_000 uniform random u64.");

    print!("std btree: ");
    query_benchmark_generic::<BTreeMapBaseline<_, _>>(num_entries, "data/main.dat", 100_000);
    println!();

    print!("sorted: ");
    query_benchmark_generic::<SortedBaseline<_, _>>(num_entries, "data/main.dat", 100_000);
    println!();

    println!();
    print!("pure btree: ");
    query_benchmark_generic::<HybridIndex1<_, _>>(num_entries, "data/main.dat", 100_000);
    println!();

    print!("btree then pgm: ");
    query_benchmark_generic::<HybridIndex2<_, _>>(num_entries, "data/main.dat", 100_000);
    println!();

    print!("pgm then btree: ");
    query_benchmark_generic::<HybridIndex3<_, _>>(num_entries, "data/main.dat", 100_000);
    println!();

    print!("pure pgm: ");
    query_benchmark_generic::<HybridIndex4<_, _>>(num_entries, "data/main.dat", 100_000);
    println!();
}

#[derive(Debug, ValueEnum, Copy, Clone)]
enum Distributions {
    Uniform,
}

impl std::fmt::Display for Distributions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

fn gen_data(num_entries: usize, path: impl AsRef<Path>, distribution: Distributions) {
    let mut data = match distribution {
        Distributions::Uniform => StdRng::from_entropy()
            .sample_iter(Uniform::new(0, u64::MAX))
            .take(num_entries)
            .unique()
            // .map(|x| (x, ()))
            .collect_vec(),
    };
    println!("generated data: {:?}", &data[..10]);

    data.par_sort_unstable();

    Buffer::<u64>::from_slice_on_disk(&data, path).expect("failed to write to file");
}

fn query_benchmark_generic<I: ImmutableIndex<u64, ()>>(
    num_entries: usize,
    path: impl AsRef<Path>,
    num_trials: usize,
) {
    let buffer = Buffer::<u64>::load_from_disk(path).expect("failed to read file");
    let ratio = num_entries as f64 / buffer.len() as f64;
    assert!(ratio <= 1.0);

    let index = I::build_in_memory(buffer[0..num_entries].into_iter().copied().map(|x| (x, ())));

    let dist = Uniform::new(0, (u64::MAX as f64 * ratio) as u64);

    // Warm up the tree
    for _ in 0..num_trials {
        black_box(index.lookup(&thread_rng().sample(dist)));
    }

    let searches = thread_rng()
        .sample_iter(Uniform::new(0, num_entries))
        .take(num_trials)
        .map(|i| buffer[i])
        .collect_vec();

    let mut average = MeanWithError::new();
    for search in searches {
        let start = Instant::now();
        black_box(index.lookup(&search));
        let end = start.elapsed();
        average.add(end.as_nanos() as f64);
    }

    print!("{} ", average.mean() as u64);
}
