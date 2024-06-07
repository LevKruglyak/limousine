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
    io::{stdout, Write},
    path::PathBuf,
    time::Instant,
};

use limousine_engine::prelude::*;
use sled::IVec;

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

fn main() {
    println!("[DONE]");

    #[cfg(not(feature = "instance"))]
    eprintln!("WARNING: running in instance mode, this should only happen during development!");

    let value_size: usize = str::parse(std::env!("VALUE_SIZE")).unwrap();
    let size: usize = str::parse(std::env!("SIZE")).unwrap();

    let path: PathBuf = PathBuf::from(std::env!("STORE_PATH"));
    let path = path.join("data");
    if !path.exists() {
        std::fs::create_dir_all(path.clone()).unwrap();
    }

    let layout = std::fs::read_to_string(".layout").unwrap();

    println!();
    println!("Running benchmark with parameters:");
    println!("    Key Type: {} bytes", Key::BITS / 8);
    println!("    Value Size: {} bytes", value_size);
    println!("    Size: {} entries", size);
    println!("    Layout: {}", layout);
    println!("    Path: {}", path.to_str().unwrap());
    println!();

    print!("Loading stores...    ");
    stdout().flush().unwrap();

    {
        std::fs::remove_dir_all(path.clone()).unwrap();

        let store_path = path.join("store");
        let store_sled_path = path.join("store_sled");

        let mut store: Instance<Key, Vec<u8>> =
            Instance::open(store_path.clone().join("store")).unwrap();
        let store_sled = sled::open(store_sled_path.clone()).unwrap();

        let start = Instant::now();
        for key in 0..size {
            let value = vec![0; value_size];
            store.insert(key as Key, value).unwrap();
        }
        let store_end = start.elapsed();

        let start = Instant::now();
        for key in 0..size {
            let value = IVec::from(vec![0; value_size]);
            store_sled.insert(&key.to_le_bytes(), &value).unwrap();
        }
        let store_sled_end = start.elapsed();

        drop(store);
        drop(store_sled);

        let store_sled_size = fs_extra::dir::get_size(store_sled_path).unwrap();
        let store_size = fs_extra::dir::get_size(store_path).unwrap();

        println!("[DONE]");
        println!();

        println!(
            "size:    {:.2e} bytes        baseline improvement: {:.2}x",
            store_size as f64,
            store_sled_size as f64 / store_size as f64
        );

        println!(
            "insert:  {:.2e} ops/sec      baseline improvement: {:.2}x",
            size as f64 / store_end.as_secs_f64(),
            store_sled_end.as_secs_f64() / store_end.as_secs_f64()
        );
    }

    {
        let store: Instance<Key, Vec<u8>> = Instance::open(path.join("store")).unwrap();
        let store_sled = sled::open(path.join("store_sled")).unwrap();

        let start = Instant::now();
        for key in 0..size {
            black_box(store.search(key as Key).unwrap());
        }
        let store_end = start.elapsed();

        let start = Instant::now();
        for key in 0..size {
            black_box(store_sled.get(&key.to_le_bytes()).unwrap());
        }
        let store_sled_end = start.elapsed();

        println!(
            "search:  {:.2e} ops/sec      baseline improvement: {:.2}x",
            size as f64 / store_end.as_secs_f64(),
            store_sled_end.as_secs_f64() / store_end.as_secs_f64(),
        );
    }

    std::fs::remove_file(".config").unwrap();
    std::fs::remove_file(".layout").unwrap();
}
