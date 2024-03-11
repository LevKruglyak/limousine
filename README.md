# Limousine &emsp; 
[![Rust](https://github.com/LevKruglyak/limousine/actions/workflows/rust.yml/badge.svg)](https://github.com/LevKruglyak/limousine/actions/workflows/rust.yml)
[![Latest Version](https://img.shields.io/crates/v/limousine_engine.svg)](https://crates.io/crates/limousine_engine)

Learned indexes, which use statistical models to approximate the location of keys in an index, have been proven to be highly effective, both in terms of memory usage and performance. Nevertheless, they suffer from some unavoidable trade-offs when compared to the well-developed BTree design. In this experimental project, we want to map out a design space of *hybrid indexes*, which contain some classical, BTree layers, and some learned index layers. 

Supporting mutable hybrid indexes of this form which support efficient insertion and deletion is still the subject of ongoing research which we are working on. This crate serves as a fully-functioning prototype which can materialize a working design from a layout specification.

# Overview

***limousine_engine*** provides a procedural macro to automatically generate a hybrid index design:

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

To generate a design, we provide a name for the structure, and a layout description, which consists of a stack of components. These components can be either classical or learned. In this example, there is a large persisted BTree layer storing the base data, followed by two smaller BTree layers, a Piecewise Geometric Model layer, and everything above is an optimized in memory BTree.

We can then use these generated structs to perform queries:

```rust
// Load the first two layer of the index from memory
let index = ExampleHybridIndex::<i32, i32>::load("path_to_index")?;

// Range query
for (key, value) in index.range(&0, &100) {
    println!("found entry: {key:?} {value:?}");
}
```
