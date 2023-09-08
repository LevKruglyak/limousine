#![allow(unused)]

use limousine_engine::prelude::*;

create_hybrid_index! {
    name: MyHybridIndex,
    layout: [
        btree_top(),
        btree(fanout = 64, persist),
        btree(fanout = 64, persist),
        btree(fanout = 64, persist),
        btree(fanout = 64, persist),
        btree(fanout = 16, persist),
        btree(fanout = 64, persist),
    ]
}

fn main() {
    let mut index = MyHybridIndex::empty();

    for i in 0..1_000 {
        index.insert(i, i * i);
    }

    println!("{:?}", index.search(&10));
}
