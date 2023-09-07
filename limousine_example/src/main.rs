#![allow(unused)]

use average::*;
use egui::DragValue;
use itertools::Itertools;
use itertools::Unique;
use limousine_core::classical::*;
use limousine_core::component::*;
use limousine_core::learned::*;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use rand_distr::Uniform;
use std::time::Instant;

use limousine_engine::prelude::*;

create_hybrid_index! {
    name: MyHybridIndex,
    layout: [
        btree_top(),
        btree(fanout = 8),
        btree(fanout = 8),
        btree(fanout = 8),
        btree(fanout = 16),
        btree(fanout = 32),
    ]
}

fn main() {
    let mut index = MyHybridIndex::empty();

    for i in 0..1_000 {
        index.insert(i, i * i);
    }

    println!("{:?}", index.search(&10));
}
