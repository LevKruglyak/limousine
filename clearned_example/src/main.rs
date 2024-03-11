use clearned::materialize_index;

materialize_index! {
    name: HybridIndex,
    layout: {
        0 => btree(5),
        _ => pgm(5),
    }
}

fn main() {
    let entries = (100..1_000_000)
        .map(|i| (2 * i, (i * 7895) % 32))
        .collect::<Vec<_>>();

    let index = HybridIndex::build_in_memory(entries.into_iter());

    for (key, value) in index.range(&0, &250) {
        println!("{key:?} {value:?}");
    }
}
