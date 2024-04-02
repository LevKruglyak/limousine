use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};

use crate::entry::SortedArrayEntry;
use crate::SortedArray;

impl<K, V, const FANOUT: usize> Serialize for SortedArray<K, V, FANOUT>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len))?;
        for entry in self.entries() {
            seq.serialize_element(entry)?;
        }
        seq.end()
    }
}

struct SortedArrayDeserializer<K, V, const FANOUT: usize>(
    core::marker::PhantomData<(K, V, [(); FANOUT])>,
);

impl<'de, K, V, const FANOUT: usize> Visitor<'de> for SortedArrayDeserializer<K, V, FANOUT>
where
    K: Deserialize<'de> + Ord + Copy,
    V: Deserialize<'de>,
{
    type Value = SortedArray<K, V, FANOUT>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("A sequence of entries for SortedArray")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut map = SortedArray::<K, V, FANOUT>::empty();

        while let Some(entry) = seq.next_element::<SortedArrayEntry<K, V>>()? {
            if map.len() >= FANOUT {
                return Err(serde::de::Error::custom(
                    "SortedArray exceeded its capacity during deserialization",
                ));
            }
            map.insert(entry.key, entry.value);
        }

        Ok(map)
    }
}

impl<'de, K, V, const FANOUT: usize> Deserialize<'de> for SortedArray<K, V, FANOUT>
where
    K: Serialize + Deserialize<'de> + Ord + Copy,
    V: Serialize + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(SortedArrayDeserializer(core::marker::PhantomData))
    }
}
