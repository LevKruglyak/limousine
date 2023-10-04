# Limousine &emsp; 
[![Rust](https://github.com/LevKruglyak/limousine/actions/workflows/rust.yml/badge.svg)](https://github.com/LevKruglyak/limousine/actions/workflows/rust.yml)
[![Latest Version](https://img.shields.io/crates/v/limousine_engine.svg)](https://crates.io/crates/limousine_engine)

**Limousine** is an exploration into the world of hybrid indexes. Traditional indexes, like BTrees, have been optimized for decades, offering consistent performance for both inserts and reads. On the other hand, learned indexes, which leverage statistical models to approximate the locations of keys, bring massive benefits in memory usage and read performance. However, they come with their own set of trade-offs; most notably there isn't a canonical or efficient algorithm for performing inserts.

This project experiments with hybrid indexes — a combination of traditional BTree layers and learned index layers. The goal is to harness the strengths of both indexing methods, in addition to improving the state of the art for learned index insertion. While developing efficient and mutable hybrid indexes is an active area of research, this crate offers a fully-functioning prototype, capable of turning a layout specification into a working design.

Most of our work with learned indexes was inspired by [PGM Index](https://github.com/gvinciguerra/PGM-index).

# Overview

***limousine_engine*** offers a procedural macro that auto-generates a hybrid index design:

```rust
use limousine_engine::prelude::*;

create_hybrid_index! {
    name: ExampleHybridIndex,
    layout: [
        btree_top(),
        pgm(epsilon = 4),
        btree(fanout = 32),
        btree(fanout = 32),
        btree(fanout = 1024, persist),
    ]
}
```

To create a hybrid index, specify a name and a layout. The layout is a stack of components that can be classical or learned. In this example, we use a large persisted BTree layer for base data storage, followed by two smaller BTree layers, a Piecewise Geometric Model (PGM) layer, and an optimized in-memory BTree at the top.

Once the index is generated, you can run queries:

```rust
// Load the first two layer of the index from memory
let index = ExampleHybridIndex::<i32, i32>::load("path_to_index")?;

// Range query
for (key, value) in index.range(&0, &100) {
    println!("found entry: {key:?} {value:?}");
}
```
