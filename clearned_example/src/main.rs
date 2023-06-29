use clearned::materialize_index;

materialize_index! {
    name: BTreeIndex,
    layout: {
        _ => pgm(5),
    }
}

fn main() {
    let entries = (1..1_000_000)
        .map(|i| (i, (i * 7895) % 32))
        .collect::<Vec<_>>();

    let index = BTreeIndex::build_in_memory(entries.into_iter());

    for (key, value) in index.range(&32, &64) {
        println!("{key:?} {value:?}");
    }
}
