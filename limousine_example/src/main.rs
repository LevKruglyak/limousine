#![allow(unused)]

use limousine_engine::prelude::*;

// create_hybrid_index! {
//     name: Index2,
//     path: "limousine_example/sample.layout"
// }
//
// type K = u128;
// type V = u128;
//
// fn test_index<T: Index<K, V>>(num: K) {
//     let name_vec: Vec<&str> = core::any::type_name::<T>().split("::").collect();
//     let name = name_vec.last().unwrap().split_once("<").unwrap().0;
//
//     let mut index: T = T::empty();
//
//     let start = std::time::Instant::now();
//     for i in 0..num {
//         index.insert(i, i * i);
//     }
//     println!(
//         "{} insertion took {:?} ms ",
//         name,
//         start.elapsed().as_millis()
//     );
//
//     let start = std::time::Instant::now();
//     for i in 0..num {
//         assert_eq!(index.search(i), Some(i * i));
//     }
//     println!("{} search took {:?} ms ", name, start.elapsed().as_millis());
// }

create_hybrid_index! {
    name: Index1,
    layout: [
        btree_top(),
        btree(fanout = 8, persist),
    ]
}

fn main() -> limousine_engine::Result<()> {
    let mut index: Index1<i32, i32> =
        Index1::build((0..100).map(|i| (i, i * i)), "Index1.lim_idx")?;

    for i in 0..100 {
        let result = index.search(i)?;
        println!("Search: {i:?} -> {result:?}");
    }

    // for i in 0..100 {
    //     println!("Insert: {i:?}");
    //     index.insert(i, i * i).expect("Failed to insert!");
    // }

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
