#[cfg(feature = "key-U32")]
type Key = u32;
#[cfg(feature = "key-I32")]
type Key = i32;
#[cfg(feature = "key-U64")]
type Key = u64;
#[cfg(feature = "key-I64")]
type Key = i64;
#[cfg(feature = "key-I128")]
type Key = u128;
#[cfg(feature = "key-I128")]
type Key = i128;

use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use average::{Estimate, Variance};
use limousine_engine::prelude::*;

#[cfg(feature = "instance")]
create_kv_store! {
    name: Instance,
    path: ".layout"
}

#[cfg(not(feature = "instance"))]
create_kv_store! {
    name: Instance,
    layout: [
        btree_top(),
        btree(fanout = 32, persist)
    ]
}

/// Blackbox function for benchmarking
#[inline(never)]
pub fn black_box<D>(dummy: D) -> D {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

fn random(n: u32) -> u32 {
    use std::cell::Cell;
    use std::num::Wrapping;

    thread_local! {
        static RNG: Cell<Wrapping<u32>> = Cell::new(Wrapping(1406868647));
    }

    RNG.with(|rng| {
        // This is the 32-bit variant of Xorshift.
        //
        // Source: https://en.wikipedia.org/wiki/Xorshift
        let mut x = rng.get();
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        rng.set(x);

        // This is a fast alternative to `x % n`.
        //
        // Author: Daniel Lemire
        // Source: https://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/
        ((x.0 as u64).wrapping_mul(n as u64) >> 32) as u32
    })
}

pub fn bench_function<T>(gen_params: impl Fn() -> T, mut func: impl FnMut(T) -> ()) -> Variance {
    let samples = 100;
    let warmup = Duration::from_secs(3);
    let trial = Duration::from_secs(3);

    // Measure how long warmup will take
    let mut mean_time = average::MeanWithError::new();
    for _ in 0..samples {
        let params = gen_params();
        let start = Instant::now();
        black_box(func(params));
        mean_time.add(start.elapsed().as_secs_f64());
    }
    let num_samples_for_second = (1.0 / mean_time.mean()) as usize;

    // Warm up
    let start = Instant::now();
    while start.elapsed() < warmup {
        for _ in 0..num_samples_for_second {
            let params = gen_params();
            black_box(func(params));
        }
    }

    // Run trial
    let mut mean_time = average::MeanWithError::new();

    let start = Instant::now();
    while start.elapsed() < trial {
        for _ in 0..num_samples_for_second {
            let params = gen_params();

            let start = Instant::now();
            black_box(func(params));
            let elapsed = start.elapsed().as_secs_f64();

            mean_time.add(elapsed);
        }
    }

    mean_time
}

fn main() {
    let value_size: usize = str::parse(std::env!("VALUE_SIZE")).unwrap_or(1024);
    let size: usize = str::parse(std::env!("SIZE")).unwrap_or(0);

    let path: PathBuf = PathBuf::from(std::env!("STORE_PATH"));
    let path = path.join("data");

    println!("Running benchmark with parameters:");
    println!("    Key Type: {} bytes", Key::BITS / 8);
    println!("    Value Size: {} bytes", value_size);
    println!("    Size: {} entries", size);
    println!();

    print!("Loading stores...    ");
    {
        std::fs::remove_dir_all(path.clone()).unwrap();
        let mut search_store: Instance<Key, ()> = Instance::open(path.join("search")).unwrap();
        let mut insert_store: Instance<Key, ()> = Instance::open(path.join("insert")).unwrap();
        for _ in 0..size {
            search_store.insert(random(Key::MAX), ()).unwrap();
            insert_store.insert(random(Key::MAX), ()).unwrap();
        }
    }
    println!("[DONE]");

    {
        let mut insert_store: Instance<Key, ()> = Instance::open(path.join("insert")).unwrap();
        let insert_variance = bench_function(
            || {},
            |()| {
                insert_store.insert(random(Key::MAX), ()).unwrap();
            },
        );
        println!(
            "random_insert: {:?}    ({} samples)",
            Duration::from_secs_f64(insert_variance.mean()),
            insert_variance.len(),
        );
    }

    {
        let search_store: Instance<Key, Vec<u8>> = Instance::open(path.join("search")).unwrap();
        let search_variance = bench_function(
            || {},
            |()| {
                search_store.search(random(Key::MAX)).unwrap();
            },
        );
        println!(
            "random_search: {:?}    ({} samples)",
            Duration::from_secs_f64(search_variance.mean()),
            search_variance.len(),
        );
    }
}
