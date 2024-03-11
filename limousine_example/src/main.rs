#![allow(unused)]

use limousine_engine::prelude::*;

create_hybrid_index! {
    name: MyHybridIndex,
    layout: [
        btree_top(),
        btree(fanout = 8),
        btree(fanout = 8),
        btree(fanout = 8, persist),
        btree(fanout = 16, persist),
        btree(fanout = 32, persist),
    ]
}

fn main() {
    let num = 10_000_000;

    let mut index: MyHybridIndex<u128, u128> = MyHybridIndex::empty();

    let start = std::time::Instant::now();
    for i in 0..num {
        index.insert(i, i * i);
    }
    println!(
        "{:?} after {:?} ms",
        index.search(&10),
        start.elapsed().as_millis()
    );

    use std::collections::BTreeMap;
    let mut index: BTreeMap<u128, u128> = BTreeMap::new();

    let start = std::time::Instant::now();
    for i in 0..num {
        index.insert(i, i * i);
    }
    println!(
        "{:?} after {:?} ms",
        index.get(&10),
        start.elapsed().as_millis()
    );
}
