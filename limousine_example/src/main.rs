#![allow(unused)]

use limousine_engine::prelude::*;

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

create_hybrid_index! {
    name: Index2,
    path: "limousine_example/sample.layout"
}

fn main() {
    let num = 10_000_000;
    println!("Inserting {} entries:", num);

    let mut index: Index1<u128, u128> = Index1::empty();

    let start = std::time::Instant::now();
    for i in 0..num {
        index.insert(i, i * i);
    }
    println!("Index1 took {:?} ms", start.elapsed().as_millis());

    let mut index: Index2<u128, u128> = Index2::empty();

    let start = std::time::Instant::now();
    for i in 0..num {
        index.insert(i, i * i);
    }
    println!("Index2 took {:?} ms", start.elapsed().as_millis());

    use std::collections::BTreeMap;
    let mut index: BTreeMap<u128, u128> = BTreeMap::new();

    let start = std::time::Instant::now();
    for i in 0..num {
        index.insert(i, i * i);
    }
    println!("StdBTree took {:?} ms", start.elapsed().as_millis());
}
