use clearned_core::{test::HybridIndex, ImmutableIndex};

fn main() {
    let entries = vec![
        (1, 10.01),
        (2, 20.0),
        (3, 30.0),
        (4, 40.0),
        (5, 50.0),
        (6, 60.0),
        (7, 70.0),
        (8, 80.0),
        (9, 10.0),
        (20, 20.05),
        (21, 20.0),
        (22, 30.0),
        (23, 40.0),
        (24, 50.0),
        (25, 60.0),
        (26, 70.0),
        (27, 80.0),
    ];

    let index = HybridIndex::build(entries.into_iter());

    for (key, value) in index.range(&-100, &1000) {
        println!("{key:?} {value:?}");
    }
}
