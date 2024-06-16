# Limousine [WIP]

[![Rust](https://github.com/LevKruglyak/limousine/actions/workflows/rust.yml/badge.svg)](https://github.com/LevKruglyak/limousine/actions/workflows/rust.yml)
[![Latest Version](https://img.shields.io/crates/v/limousine_engine.svg)](https://crates.io/crates/limousine_engine)

`limousine` is an exploration into the world of hybrid key-value stores. Traditional indices 
-- such as BTrees -- have been optimized for decades, offering consistent performance for both 
inserts and reads. On the other hand, learned indices, which leverage statistical models to 
approximate the locations of keys, bring massive benefits in memory usage and read performance. 
However, these newer structures come with their own set of trade-offs; most notably there isn't 
a canonical or efficient algorithm for performing inserts.

This project experiments with hybrid key-value stores -- data structures which consist of a 
combination of traditional BTree nodes and learned, statistical model nodes. The goal is to harness
the strengths of both indexing methods, in addition to improving the state of the art for learned 
index insertion. While developing efficient and mutable hybrid indexes is an active area of research, 
this crate offers a fully-functioning prototype, capable of turning a layout specification into a 
working design.

Most of our work with learned indexes was inspired by 
[PGM Index](https://github.com/gvinciguerra/PGM-index).

This is mostly a prototype project. Although the generated key-value stores are functional and 
fairly efficient, they lack many features such as efficient removal of entries, batch inserts, 
multithreaded insertion, transactions, etc. Eventually, we hope that we are able to implement these 
features, however there are a variety of challenges associated with dynamic code generation of novel 
data structures and this is still an active area of research.

# Overview

`limousine_engine` provides a procedural macro to automatically
generate an hybrid key-value store design consisting of both
classical and learned components.

**As of the current version, learned components are not yet fully
supported.**

```rust
use limousine_engine::prelude::*;

create_kv_store! {
    name: ExampleStore,
    layout: [
        btree_top(),
        pgm(epsilon = 8),
        pgm(epsilon = 8),
        btree(fanout = 32),
        btree(fanout = 32, persist),
        btree(fanout = 64, persist)   
    ]
}
```

To generate a design, we provide a name for the structure and a
layout description which consists of a stack of components. For
instance in the above example, the key-value store consists of
a base layer of on-disk BTree nodes of fanout 64, underneath a  
layer of on on-disk BTree nodes with fanout 32, underneath an
in-memory layer of BTree nodes with fanout 32. On top of this, we
have two in-memory PGM learned layers with epsilon parameters of 8,
and a tiny in-memory BTree as a top layer.

**Since learned components are not yet fully supported, the above example
will not compile. To get a working key-value store in the current version,
we should only use BTree components.**

```rust
use limousine_engine::prelude::*;

create_kv_store! {
    name: ExampleStore,
    layout: [
        btree_top(),
        btree(fanout = 8),
        btree(fanout = 8),
        btree(fanout = 32),
        btree(fanout = 32, persist),
        btree(fanout = 64, persist)   
    ]
}
```

We can then use these generated data structures to perform queries:

```rust
// Load the first two layer of the index from memory
let index: ExampleStore<u128, u128> = ExampleStore::open("data/index")?;

index.insert(10, 50)?;
index.insert(20, 60)?;
index.insert(30, 70)?;
index.insert(40, 80)?;

assert_eq!(index.search(10)?, Some(50));
```
