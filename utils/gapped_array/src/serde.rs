use std::mem::MaybeUninit;

use serde::{Deserialize, Serialize};

use crate::GappedEntryArray;

impl<K, V> Serialize for GappedEntryArray<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct GappedEntryArrayHelper<'a, K, V> {
            bitmap: &'a [bool],
            keys: &'a [K],
            vals: &'a [V],
            size: usize,
            capacity: usize,
        }

        let helper = GappedEntryArrayHelper {
            bitmap: &self.bitmap,
            keys: &self.keys,
            vals: &self.vals,
            size: self.size,
            capacity: self.capacity,
        };

        helper.serialize(serializer)
    }
}

impl<'de, K, V> Deserialize<'de> for GappedEntryArray<K, V>
where
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct GappedEntryArrayHelper<K, V> {
            bitmap: Vec<bool>,
            keys: Vec<K>,
            vals: Vec<V>,
            size: usize,
            capacity: usize,
        }

        let helper = GappedEntryArrayHelper::deserialize(deserializer)?;
        Ok(GappedEntryArray {
            bitmap: helper.bitmap,
            keys: helper.keys,
            vals: helper.vals,
            size: helper.size,
            capacity: helper.capacity,
        })
    }
}
