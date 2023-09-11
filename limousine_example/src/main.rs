#![allow(unused)]

use limousine_core::Entry;
use limousine_engine::prelude::*;

create_hybrid_index! {
    name: MyHybridIndex,
    layout: [
        btree_top(),
        pgm(epsilon = 10),
        btree(fanout = 4, persist)
    ]
}

fn main() {
    let mut index = MyHybridIndex::build((0..1000).map(|x| Entry::new(x, x * x)));

    println!("{:?}", index.search(&10));
}
