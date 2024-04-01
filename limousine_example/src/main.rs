#![allow(unused)]

use limousine_engine::prelude::*;

// Example of a persisted key-value store
create_hybrid_index! {
    name: Index1Disk,
    layout: [
        btree_top(),
        btree(fanout = 8),
        btree(fanout = 8, persist),
        btree(fanout = 64, persist),
    ]
}

// Example of an in-memory key-value store
create_hybrid_index! {
    name: Index1,
    layout: [
        btree_top(),
        btree(fanout = 4),
        btree(fanout = 4),
        btree(fanout = 4),
        btree(fanout = 64),
    ]
}

// Example of an in-memory key-value store, with layout provided in a file
create_hybrid_index! {
    name: Index2,
    path: "limousine_example/sample.layout"
}

fn main() -> limousine_engine::Result<()> {
    // Clear data directory
    std::fs::remove_dir_all("data")?;

    let num = 1_000_000;
    println!("Inserting {} entries:", num);

    let mut index: Index1Disk<u128, u128> = Index1Disk::open("data/index1")?;

    let start = std::time::Instant::now();
    for i in 0..num {
        index.insert(i, i * i)?;
    }
    println!(
        "[Persisted] Index1 took {:?} ms",
        start.elapsed().as_millis()
    );

    let mut index: Index1<u128, u128> = Index1::empty();

    let start = std::time::Instant::now();
    for i in 0..num {
        index.insert(i, i * i);
    }
    println!(
        "[In Memory] Index1 took {:?} ms",
        start.elapsed().as_millis()
    );

    let mut index: Index2<u128, u128> = Index2::empty();

    let start = std::time::Instant::now();
    for i in 0..num {
        index.insert(i, i * i);
    }
    println!(
        "[In Memory] Index2 took {:?} ms",
        start.elapsed().as_millis()
    );

    use std::collections::BTreeMap;
    let mut index: BTreeMap<u128, u128> = BTreeMap::new();

    let start = std::time::Instant::now();
    for i in 0..num {
        index.insert(i, i * i);
    }
    println!(
        "[In Memory] BTreeMap took {:?} ms",
        start.elapsed().as_millis()
    );

    Ok(())
}
