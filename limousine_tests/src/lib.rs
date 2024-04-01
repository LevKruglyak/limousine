#[cfg(test)]
mod tests {
    use limousine_engine::prelude::*;
    use rand::{thread_rng, Rng};
    use rand_distr::Uniform;
    use tempfile::tempdir;

    type K = i128;
    type V = i128;

    fn test_persisted_index<I: PersistedIndex<K, V>>() {
        let temp_dir = tempdir().expect("Failed to create a temporary directory");
        let temp_path = temp_dir.path();

        let mut rng = thread_rng();
        let key_dist = Uniform::new(K::MIN, K::MAX);
        let value_dist = Uniform::new(V::MIN, V::MAX);

        let num = 20_000;
        let keys: Vec<K> = (&mut rng)
            .sample_iter(key_dist)
            .filter(|&x| x < 0 as K || x > 10_000 as K) // we want to test for false positives as
            // well
            .take(num)
            .collect();

        let values: Vec<V> = (&mut rng).sample_iter(value_dist).take(num).collect();

        {
            let mut index = I::open(temp_path).expect("Failed to open index");

            // Test inserts
            for i in 0..num {
                index.insert(keys[i], values[i]).expect("Failed to insert");
            }

            // Test searches
            for i in 0..num {
                assert_eq!(
                    index.search(keys[i]).expect("Failed to search"),
                    Some(values[i])
                );
            }

            for key in 0..10_000 {
                assert_eq!(index.search(key as K).expect("Failed to search"), None);
            }
        }

        let mut index = I::open(temp_path).expect("Failed to open index");

        // Test searches were persisted
        for i in 0..num {
            assert_eq!(
                index.search(keys[i]).expect("Failed to search"),
                Some(values[i])
            );
        }

        for key in 0..10_000 {
            assert_eq!(index.search(key as K).expect("Failed to search"), None);
        }

        // Test for insert now
        for key in 0..10_000 {
            index.insert(key, key * key as V).expect("Failed to insert");
        }

        // Search again
        for i in 0..num {
            assert_eq!(
                index.search(keys[i]).expect("Failed to search"),
                Some(values[i])
            );
        }

        for key in 0..10_000 {
            assert_eq!(
                index.search(key as K).expect("Failed to search"),
                Some(key * key as V)
            );
        }
    }

    fn test_index<I: Index<K, V>>() {
        let mut rng = thread_rng();
        let key_dist = Uniform::new(K::MIN, K::MAX);
        let value_dist = Uniform::new(V::MIN, V::MAX);

        let num = 20_000;
        let keys: Vec<K> = (&mut rng)
            .sample_iter(key_dist)
            .filter(|&x| x < 0 as K || x > 10_000 as K) // we want to test for false positives as
            // well
            .take(num)
            .collect();

        let values: Vec<V> = (&mut rng).sample_iter(value_dist).take(num).collect();

        {
            let mut index = I::empty();

            // Test inserts
            for i in 0..num {
                index.insert(keys[i], values[i]);
            }

            // Test searches
            for i in 0..num {
                assert_eq!(index.search(keys[i]), Some(values[i]));
            }

            for key in 0..10_000 {
                assert_eq!(index.search(key as K), None);
            }
        }
    }

    #[test]
    fn test_persisted_index_1() {
        create_hybrid_index! {
            name: Index,
            layout: [
                btree_top(),
                btree(fanout = 64, persist),
            ]
        }

        test_persisted_index::<Index<K, V>>();
    }

    #[test]
    fn test_persisted_index_2() {
        create_hybrid_index! {
            name: Index,
            layout: [
                btree_top(),
                btree(fanout = 32, persist),
            ]
        }

        test_persisted_index::<Index<K, V>>();
    }

    #[test]
    fn test_persisted_index_3() {
        create_hybrid_index! {
            name: Index,
            layout: [
                btree_top(),
                btree(fanout = 32, persist),
                btree(fanout = 32, persist),
            ]
        }

        test_persisted_index::<Index<K, V>>();
    }

    #[test]
    fn test_persisted_index_4() {
        create_hybrid_index! {
            name: Index,
            layout: [
                btree_top(),
                btree(fanout = 8),
                btree(fanout = 8, persist),
                btree(fanout = 32, persist),
            ]
        }

        test_persisted_index::<Index<K, V>>();
    }

    #[test]
    fn test_persisted_index_5() {
        create_hybrid_index! {
            name: Index,
            layout: [
                btree_top(),
                btree(fanout = 8),
                btree(fanout = 8, persist),
                btree(fanout = 8, persist),
                btree(fanout = 32, persist),
            ]
        }

        test_persisted_index::<Index<K, V>>();
    }

    #[test]
    fn test_persisted_index_6() {
        create_hybrid_index! {
            name: Index,
            layout: [
                btree_top(),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 32, persist),
            ]
        }

        test_persisted_index::<Index<K, V>>();
    }

    #[test]
    fn test_persisted_index_7() {
        create_hybrid_index! {
            name: Index,
            layout: [
                btree_top(),
                btree(fanout = 8, persist),
                btree(fanout = 8, persist),
                btree(fanout = 8, persist),
                btree(fanout = 8, persist),
                btree(fanout = 8, persist),
                btree(fanout = 8, persist),
            ]
        }

        test_persisted_index::<Index<K, V>>();
    }

    #[test]
    fn test_index_1() {
        create_hybrid_index! {
            name: Index1,
            layout: [
                btree_top(),
                btree(fanout = 64),
            ]
        }

        test_index::<Index1<K, V>>();
    }

    #[test]
    fn test_index_2() {
        create_hybrid_index! {
            name: Index1,
            layout: [
                btree_top(),
                btree(fanout = 32),
            ]
        }

        test_index::<Index1<K, V>>();
    }

    #[test]
    fn test_index_3() {
        create_hybrid_index! {
            name: Index1,
            layout: [
                btree_top(),
                btree(fanout = 32),
                btree(fanout = 32),
            ]
        }

        test_index::<Index1<K, V>>();
    }

    #[test]
    fn test_index_4() {
        create_hybrid_index! {
            name: Index1,
            layout: [
                btree_top(),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 32),
            ]
        }

        test_index::<Index1<K, V>>();
    }

    #[test]
    fn test_index_5() {
        create_hybrid_index! {
            name: Index1,
            layout: [
                btree_top(),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 32),
            ]
        }

        test_index::<Index1<K, V>>();
    }

    #[test]
    fn test_index_6() {
        create_hybrid_index! {
            name: Index1,
            layout: [
                btree_top(),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 32),
            ]
        }

        test_index::<Index1<K, V>>();
    }

    #[test]
    fn test_index_7() {
        create_hybrid_index! {
            name: Index1,
            layout: [
                btree_top(),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 8),
            ]
        }

        test_index::<Index1<K, V>>();
    }
}
