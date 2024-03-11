use rand::{thread_rng, Rng};
use std::{sync::Arc, time::Duration};

use limousine_engine::prelude::*;

limousine_macros::create_immutable_hybrid_index! {
    name: ExampleHybridIndex,
    layout: {
        0 | 1 => btree(32),
        _ => pgm(8),
    }
}

fn main() {
    // Generate 1_000_000 gibberish entries
    let entries = (100..1_000_000)
        .map(|i| (2 * i, (i * 7895) % 32))
        .collect::<Vec<_>>();

    // Build index in memory
    let index = Arc::new(ExampleHybridIndex::build_in_memory(entries.into_iter()));
    let index1 = index.clone();
    let index2 = index.clone();

    let handle1 = std::thread::spawn(move || {
        let mut rng = thread_rng();

        for (key, value) in index1.range(&0, &250) {
            println!("thread1: {key:?} {value:?}");
            std::thread::sleep(Duration::from_millis(rng.gen_range(0..=500)));
        }
    });

    let handle2 = std::thread::spawn(move || {
        let mut rng = thread_rng();

        for (key, value) in index2.range(&300, &350) {
            println!("thread2: {key:?} {value:?}");
            std::thread::sleep(Duration::from_millis(rng.gen_range(0..=500)));
        }
    });

    handle1.join().expect("thread1 error");
    handle2.join().expect("thread2 error");
}
