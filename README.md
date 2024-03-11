# Limousine &emsp; 
[![Rust](https://github.com/LevKruglyak/limousine/actions/workflows/rust.yml/badge.svg)](https://github.com/LevKruglyak/limousine/actions/workflows/rust.yml)
[![Latest Version](https://img.shields.io/crates/v/limousine_engine.svg)](https://crates.io/crates/limousine_engine)

Learned indexes, which use statistical models to approximate the location of keys in an index, have been proven to be highly effective, both in terms of memory usage and performance. Nevertheless, they suffer from some unavoidable trade-offs when compared to the well-developed BTree design. In this project, we want to map out a design space of *hybrid indexes*, which contain some classical, BTree layers and some learned index layers. 

Supporting (mutable) hybrid indexes which support efficient insertion and deletion is still the subject of ongoing research which we are working on. This crate serves as a preliminary example of the techniques we will use for the final verson.

# Overview

***limousine_engine*** provides a procedural macro to automatically generate an (immutable) hybrid index design:

```rust
use limousine_engine::prelude::*;

limousine_macros::create_immutable_hybrid_index! {
    name: ExampleHybridIndex,
    layout: {
        0 | 1 => btree(16),
        _ => pgm(4),
    }
}
```

To generate a design, we provide a name for the structure, and a layout description, which resembles the syntax of a Rust match expression. In this example, the first two layers are BTree layers with a fanout of 16, and the rest of the layers are PGM layers with an epsilon parameter of 4. All of this is parsed and generated into a static implementation at compile time by the procedural macro. We can also generate efficient pure designs using this approach:

```rust
use limousine_engine::prelude::*;

create_immutable_hybrid_index! {
    name: BTreeIndex,
    layout: {
        _ => btree(16),
    }
}

create_immutable_hybrid_index! {
    name: PGMIndex,
    layout: {
        _ => pgm(4),
    }
}
```

We can then use these generated structs to perform queries:

```rust
// Load the first two layer of the index from memory
let index = ExampleHybridIndex::<i32, i32>::load("path_to_index", 2)?;

// Range query
for (key, value) in index.range(&0, &100) {
    println!("found entry: {key:?} {value:?}");
}
```
