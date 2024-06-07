#[cfg(test)]
mod tests {
    use limousine_engine::prelude::*;
    use rand::{thread_rng, Rng};
    use rand_distr::Uniform;
    use tempfile::tempdir;

    type K = i128;
    type V = i128;

    fn test_persisted_kv_store<KV: PersistedKVStore<K, V>>() -> limousine_engine::Result<()> {
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();

        let mut rng = thread_rng();
        let key_dist = Uniform::new(K::MIN, K::MAX);
        let value_dist = Uniform::new(V::MIN, V::MAX);

        let num = 20_000;
        let keys: Vec<K> = (&mut rng)
            .sample_iter(key_dist)
            .filter(|&x| x < 0 as K || x > 10_000 as K) // we want to test for false positives as
            // well, so we hide some range from the distribution
            .take(num)
            .collect();

        let values: Vec<V> = (&mut rng).sample_iter(value_dist).take(num).collect();

        {
            let mut kv_store = KV::open(temp_path)?;

            // Test inserts
            for i in 0..num {
                kv_store.insert(keys[i], values[i])?;
            }

            // Test searches
            for i in 0..num {
                assert_eq!(kv_store.search(keys[i])?, Some(values[i]));
            }

            for key in 0..10_000 {
                assert_eq!(kv_store.search(key as K)?, None);
            }
        }

        let mut index = KV::open(temp_path)?;

        // Test searches were persisted
        for i in 0..num {
            assert_eq!(index.search(keys[i])?, Some(values[i]));
        }

        for key in 0..10_000 {
            assert_eq!(index.search(key as K)?, None);
        }

        // Test for insert now
        for key in 0..10_000 {
            index.insert(key, key as V * key as V)?;
        }

        // Search again
        for i in 0..num {
            assert_eq!(index.search(keys[i])?, Some(values[i]));
        }

        for key in 0..10_000 {
            assert_eq!(index.search(key as K)?, Some(key * key as V));
        }

        Ok(())
    }

    fn test_kv_store<KV: KVStore<K, V>>() {
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
            let mut kv_store = KV::empty();

            // Test inserts
            for i in 0..num {
                kv_store.insert(keys[i], values[i]);
            }

            // Test searches
            for i in 0..num {
                assert_eq!(kv_store.search(keys[i]), Some(values[i]));
            }

            for key in 0..10_000 {
                assert_eq!(kv_store.search(key as K), None);
            }
        }
    }

    /// Same as test_kv_store, but instead of inserting elements one at a time,
    /// the index is built over the numbers
    fn test_kv_store_build<KV: KVStore<K, V>>() {
        let mut rng = thread_rng();
        let key_dist = Uniform::new(K::MIN, K::MAX);
        let value_dist = Uniform::new(V::MIN, V::MAX);

        let num = 20_000;
        let mut keys: Vec<K> = (&mut rng)
            .sample_iter(key_dist)
            .filter(|&x| x < 0 as K || x > 10_000 as K) // we want to test for false positives as
            // well
            .take(num)
            .collect();
        keys.sort();

        let values: Vec<V> = (&mut rng).sample_iter(value_dist).take(num).collect();

        {
            // Test build
            let kv_store = KV::build(keys.clone().into_iter().zip(values.clone().into_iter()));

            // Test searches
            for i in 0..num {
                assert_eq!(kv_store.search(keys[i]), Some(values[i]));
            }

            for key in 0..10_000 {
                assert_eq!(kv_store.search(key as K), None);
            }
        }
    }

    #[test]
    fn test_persisted_kv_store_1() -> limousine_engine::Result<()> {
        create_kv_store! {
            name: KVStore1,
            layout: [
                btree_top(),
                btree(fanout = 64, persist),
            ]
        }

        test_persisted_kv_store::<KVStore1<K, V>>()
    }

    #[test]
    fn test_persisted_kv_store_2() -> limousine_engine::Result<()> {
        create_kv_store! {
            name: KVStore1,
            layout: [
                btree_top(),
                btree(fanout = 32, persist),
            ]
        }

        test_persisted_kv_store::<KVStore1<K, V>>()
    }

    #[test]
    fn test_persisted_kv_store_3() -> limousine_engine::Result<()> {
        create_kv_store! {
            name: KVStore1,
            layout: [
                btree_top(),
                btree(fanout = 32, persist),
                btree(fanout = 32, persist),
            ]
        }

        test_persisted_kv_store::<KVStore1<K, V>>()
    }

    #[test]
    fn test_persisted_kv_store_4() -> limousine_engine::Result<()> {
        create_kv_store! {
            name: KVStore1,
            layout: [
                btree_top(),
                btree(fanout = 8),
                btree(fanout = 8, persist),
                btree(fanout = 32, persist),
            ]
        }

        test_persisted_kv_store::<KVStore1<K, V>>()
    }

    #[test]
    fn test_persisted_kv_store_5() -> limousine_engine::Result<()> {
        create_kv_store! {
            name: KVStore1,
            layout: [
                btree_top(),
                btree(fanout = 8),
                btree(fanout = 8, persist),
                btree(fanout = 8, persist),
                btree(fanout = 32, persist),
            ]
        }

        test_persisted_kv_store::<KVStore1<K, V>>()
    }

    #[test]
    fn test_persisted_kv_store_6() -> limousine_engine::Result<()> {
        create_kv_store! {
            name: KVStore1,
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

        test_persisted_kv_store::<KVStore1<K, V>>()
    }

    #[test]
    fn test_persisted_kv_store_7() -> limousine_engine::Result<()> {
        create_kv_store! {
            name: KVStore1,
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

        test_persisted_kv_store::<KVStore1<K, V>>()
    }

    #[test]
    fn test_kv_store_1() {
        create_kv_store! {
            name: KVStore1,
            layout: [
                btree_top(),
                btree(fanout = 64),
            ]
        }

        test_kv_store::<KVStore1<K, V>>();
    }

    #[test]
    fn test_kv_store_2() {
        create_kv_store! {
            name: KVStore1,
            layout: [
                btree_top(),
                btree(fanout = 32),
            ]
        }

        test_kv_store::<KVStore1<K, V>>();
    }

    #[test]
    fn test_kv_store_3() {
        create_kv_store! {
            name: KVStore1,
            layout: [
                btree_top(),
                btree(fanout = 32),
                btree(fanout = 32),
            ]
        }

        test_kv_store::<KVStore1<K, V>>();
    }

    #[test]
    fn test_kv_store_4() {
        create_kv_store! {
            name: KVStore1,
            layout: [
                btree_top(),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 32),
            ]
        }

        test_kv_store::<KVStore1<K, V>>();
    }

    #[test]
    fn test_kv_store_5() {
        create_kv_store! {
            name: KVStore1,
            layout: [
                btree_top(),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 8),
                btree(fanout = 32),
            ]
        }

        test_kv_store::<KVStore1<K, V>>();
    }

    #[test]
    fn test_kv_store_6() {
        create_kv_store! {
            name: KVStore1,
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

        test_kv_store::<KVStore1<K, V>>();
    }

    #[test]
    fn test_kv_store_7() {
        create_kv_store! {
            name: KVStore1,
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

        test_kv_store::<KVStore1<K, V>>();
    }

    #[test]

    fn test_pgm_store_3() {
        create_kv_store! {
            name: PGMStore1,
            layout: [
                btree_top(),
                pgm(epsilon = 8),
                pgm(epsilon = 8),
            ]
        }

        test_kv_store_build::<PGMStore1<K, V>>();
    }

    #[test]
    fn test_pgm_store_9() {
        create_kv_store! {
            name: PGMStore1,
            layout: [
                btree_top(),
                pgm(epsilon = 8),
                pgm(epsilon = 8),
                pgm(epsilon = 8),
                pgm(epsilon = 8),
                pgm(epsilon = 8),
                pgm(epsilon = 8),
                pgm(epsilon = 8),
                pgm(epsilon = 8),
            ]
        }

        test_kv_store_build::<PGMStore1<K, V>>();
    }
}
