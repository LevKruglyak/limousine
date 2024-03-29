#![allow(unused)]

use std::time::Instant;

use limousine_core::Path;
use limousine_engine::prelude::*;
use rand::{thread_rng, Rng};
use rand_distr::Uniform;

create_hybrid_index! {
    name: CustomIndex,
    layout: [
        btree_top(),
        btree(fanout = 8),
        btree(fanout = 32, persist),
    ]
}

fn main() -> limousine_engine::Result<()> {
    let mut custom_index: CustomIndex<i32, i32> = CustomIndex::load("data/custom_index")?;
    let mut sled_index = sled::open("data/sled")?;

    let mut rng = thread_rng();
    let uniform = Uniform::new(0, i32::MAX);

    let start = Instant::now();
    for i in 0..100_000 {
        let key = rng.sample(uniform);
        let value = rng.sample(uniform);

        custom_index.insert(key, value)?;
    }
    println!("time to insert into `CustomIndex`: {:?}", start.elapsed());

    let start = Instant::now();
    for i in 0..100_000 {
        let key = rng.sample(uniform);
        let value = rng.sample(uniform);

        sled_index.insert(&key.to_le_bytes(), &value.to_le_bytes())?;
    }
    println!("time to insert into `sled`: {:?}", start.elapsed());

    let start = Instant::now();
    println!("search: {:?}", custom_index.search(60_000_000)?);
    println!("elapsed: {:?}", start.elapsed());

    println!("{:#?}", custom_index.store.stats());

    Ok(())

    // let num = 50_000_000;
    // println!("Inserting {} entries:", num);
    //
    // test_index::<Index1<K, V>>(num);
    // test_index::<Index2<K, V>>(num);
    //
    // use std::collections::BTreeMap;
    // let mut index: BTreeMap<u128, u128> = BTreeMap::new();
    //
    // let start = std::time::Instant::now();
    // for i in 0..num {
    //     index.insert(i, i * i);
    // }
    // println!("StdBTree took {:?} ms", start.elapsed().as_millis());
    //
    // let start = std::time::Instant::now();
    // for i in 0..num {
    //     assert_eq!(index.get(&i), Some(&(i * i)));
    // }
    // println!("StdBTree search took {:?} ms ", start.elapsed().as_millis());
}
