use clearned::materialize_index;

materialize_index! {
    name: HybridIndex,
    layout: {
        0 => btree(32),
        1 => btree(4),
        _ => pgm(4),
    }
}

fn main() {
    // Generate 1_000_000 gibberish entries
    let entries = (100..1_000_000)
        .map(|i| (2 * i, (i * 7895) % 32))
        .collect::<Vec<_>>();

    // Build index in memory
    let index = HybridIndex::build_in_memory(entries.into_iter());

    // Print a range
    for (key, value) in index.range(&0, &250) {
        println!("{key:?} {value:?}");
    }
}
