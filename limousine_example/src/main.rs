#![allow(unused)]

use std::{env::temp_dir, time::Instant};

use itertools::Itertools;
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
    let dir = tempfile::tempdir().unwrap();

    let num = 10_000_000;
    let res = 100;

    println!("CustomIndex insertion by `{:?}`", num / res);

    for j in 0..res {
        let mut custom_index: CustomIndex<u128, u128> =
            CustomIndex::load(dir.path().join("custom_index"))?;

        // let mut custom_index = sled::open(dir.path().join("sled"))?;

        let mut rng = thread_rng();
        let uniform = Uniform::new(0, u128::MAX);

        let count = num / res;
        let key_values = rng.sample_iter(uniform).take(count * 2).collect_vec();

        for i in 0..count {
            custom_index.insert(key_values[2 * i], key_values[2 * i + 1])?;
        }

        custom_index.insert(10_000_000_000 + j as u128, 25);

        // for i in 0..count {
        //     custom_index.insert(
        //         &key_values[2 * i].to_le_bytes(),
        //         &key_values[2 * i + 1].to_le_bytes(),
        //     )?;
        // }

        let start = Instant::now();
        for i in 0..(2 * count) {
            assert_ne!(custom_index.search(key_values[i])?, Some(u128::MAX));
        }

        // let start = Instant::now();
        // for i in 0..(2 * count) {
        //     assert_ne!(
        //         custom_index.get(&key_values[i].to_le_bytes())?,
        //         Some(sled::IVec::from(&u128::MAX.to_le_bytes()))
        //     );
        // }
        println!("{:?}", start.elapsed().as_millis());

        assert_eq!(custom_index.search(10_000_000_000 + j as u128)?, Some(25));
    }

    // println!("sled insertion by `{:?}`", num / res);
    // for i in 0..res {
    //     let mut custom_index = sled::open(dir.path().join("sled"))?;
    //
    //     let mut rng = thread_rng();
    //     let uniform = Uniform::new(0, u128::MAX);
    //
    //     let count = num / res;
    //     let key_values = rng.sample_iter(uniform).take(count * 2).collect_vec();
    //
    //     let start = Instant::now();
    //     for i in 0..count {
    //         custom_index.insert(
    //             &key_values[2 * i].to_le_bytes(),
    //             &key_values[2 * i + 1].to_be_bytes(),
    //         )?;
    //     }
    //
    //     println!("{:?}", start.elapsed().as_millis());
    // }

    // let mut sled_index = sled::open("data/sled")?;
    // let start = Instant::now();
    // for i in 0..100_000 {
    //     let key = rng.sample(uniform);
    //     let value = rng.sample(uniform);
    //
    //     sled_index.insert(&key.to_le_bytes(), &value.to_le_bytes())?;
    // }
    // println!("time to insert into `sled`: {:?}", start.elapsed());
    //
    // let start = Instant::now();
    // println!("search: {:?}", custom_index.search(60_000_000)?);
    // println!("elapsed: {:?}", start.elapsed());
    //
    // println!();
    // println!("{:#?}", custom_index.store.stats());
    //
    Ok(())
}
