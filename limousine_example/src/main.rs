#![allow(unused)]

use limousine_core::learned::pgm::viz::play_pgm;
use limousine_engine::prelude::*;

create_hybrid_index! {
    name: MyHybridIndex,
    layout: [
        btree_top(),
        btree(fanout = 12),
        btree(fanout = 12),
        btree(fanout = 1024),
        btree(fanout = 1024),
    ]
}

fn main() {
    play_pgm();

    //     let num = 10_000_000;

    //     let mut index: MyHybridIndex<u128, u128> = MyHybridIndex::empty();

    //     let start = std::time::Instant::now();
    //     for i in 0..num {
    //         index.insert(i, i * i);
    //     }
    //     println!("{:?} after {:?} ms", index.search(&10), start.elapsed());

    //     use std::collections::BTreeMap;
    //     let mut index: BTreeMap<u128, u128> = BTreeMap::new();

    //     let start = std::time::Instant::now();
    //     for i in 0..num {
    //         index.insert(i, i * i);
    //     }
    //     println!("{:?} after {:?} ms", index.get(&10), start.elapsed().as_millis());
}
