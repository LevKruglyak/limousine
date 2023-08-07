// use btree_slab::BTreeMap;
use limousine_core::classical::*;
use limousine_core::*;
use std::collections::BTreeMap;

type Key = i32;
type Value = i32;

type Component0 = BTreeBaseComponent<Key, Value, 16>;
type Component1 = BTreeInternalComponent<Key, Component0, 16>;
type Component2 = BTreeInternalComponent<Key, Component1, 16>;
type Component3 = BTreeTopComponent<Key, Component2>;

pub struct TestIndex {
    component3: Component3,
    component2: Component2,
    component1: Component1,
    component0: Component0,
}

impl TestIndex {
    fn search(&mut self, key: &Key) -> Option<&Value> {
        let search3 = self.component3.search_top(&key);
        let search2 = self.component2.search_internal(&key, search3);
        let search1 = self.component1.search_internal(&key, search2);
        let search0 = self.component0.search_base(&key, search1);

        search0
    }

    fn insert(&mut self, key: Key, value: Value) -> Option<Value> {
        // Search stage
        let search3 = self.component3.search_top(&key);
        let search2 = self.component2.search_internal(&key, search3);
        let search1 = self.component1.search_internal(&key, search2);
        let search0 = self.component0.search_base(&key, search1);

        // If value already exists, return
        if let Some(value) = search0 {
            return Some(*value);
        }

        // Insert stage
        let (key, value) = self.component0.insert_base(key, value, search1)?;
        let (key, value) = self.component1.insert_internal(key, value, search2)?;
        let (key, value) = self.component2.insert_internal(key, value, search3)?;
        self.component3.insert_top(key, value);

        None
    }

    fn new() -> Self {
        let component0 = Component0::new_base();
        let component1 = Component1::new_internal(&component0);
        let component2 = Component2::new_internal(&component1);
        let component3 = Component3::new_top(&component2);

        Self {
            component3,
            component2,
            component1,
            component0,
        }
    }
}
fn main() {
    let num_trials = 10_000_000;

    {
        let mut index = TestIndex::new();
        let start = std::time::Instant::now();
        for i in 0..num_trials {
            index.insert(i * 100, 100);
        }
        println!("custom insert: {:?}", start.elapsed());

        let start = std::time::Instant::now();
        for i in 0..num_trials {
            assert_eq!(index.search(&(i * 50)).is_some(), i % 2 == 0);
        }
        println!("custom search: {:?}", start.elapsed());
    }

    {
        let mut index: BTreeMap<Key, Value> = BTreeMap::new();
        let start = std::time::Instant::now();
        for i in 0..num_trials {
            index.insert(i * 100, 100);
        }
        println!("std insert: {:?}", start.elapsed());

        let start = std::time::Instant::now();
        for i in 0..num_trials {
            assert_eq!(index.get(&(i * 50)).is_some(), i % 2 == 0);
        }
        println!("std search: {:?}", start.elapsed());
    }
}
