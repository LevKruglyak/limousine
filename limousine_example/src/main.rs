#![allow(unused)]

use limousine_engine::prelude::*;

// Example of a persisted key-value store
create_kv_store! {
    name: PersistedKVStore1,
    layout: [
        btree_top(),
        btree(fanout = 8),
        btree(fanout = 8, persist),
        btree(fanout = 64, persist),
    ]
}

// Example of an in-memory key-value store
create_kv_store! {
    name: KVStore1,
    layout: [
        btree_top(),
        btree(fanout = 4),
        btree(fanout = 4),
        btree(fanout = 4),
        btree(fanout = 64),
    ]
}

// Example of an in-memory key-value store, with layout provided in a file
create_kv_store! {
    name: KVStore2,
    path: "limousine_example/sample.layout"
}

fn main() -> limousine_engine::Result<()> {
    // Clear data directory
    std::fs::remove_dir_all("data")?;

    let num = 1_000_000;
    println!("Inserting {} entries:", num);

    let mut index: PersistedKVStore1<u128, u128> = PersistedKVStore1::open("data/index1")?;

    let start = std::time::Instant::now();
    for i in 0..num {
        index.insert(i, i * i)?;
    }
    println!(
        "[Persisted] KVStore1 took {:?} ms",
        start.elapsed().as_millis()
    );

    let mut index: KVStore1<u128, u128> = KVStore1::empty();

    let start = std::time::Instant::now();
    for i in 0..num {
        index.insert(i, i * i);
    }
    println!(
        "[In Memory] KVStore1 took {:?} ms",
        start.elapsed().as_millis()
    );

    let mut index: KVStore2<u128, u128> = KVStore2::empty();

    let start = std::time::Instant::now();
    for i in 0..num {
        index.insert(i, i * i);
    }
    println!(
        "[In Memory] KVStore2 took {:?} ms",
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
