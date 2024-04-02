use serde::{Deserialize, Serialize};
use trait_set::trait_set;

// Until `trait_alias` is stabilized, we have to use a macro
trait_set! {
    /// A simple address trait,
    pub trait Address = Eq + Clone + 'static;

    pub trait Persisted = Serialize + for<'de> Deserialize<'de> + Clone + Default + Eq + 'static;

    /// General key type, thread safe
    pub trait Key = Send + Sync + Default + StaticBounded + 'static;

    /// General value type, thread-safe
    pub trait Value = Send + Sync + Default + Clone + 'static;
}

pub trait KeyBounded<K> {
    fn lower_bound(&self) -> &K;
}

pub trait StaticBounded: 'static + Copy + Ord {
    fn min_ref() -> &'static Self;
}

macro_rules! impl_integer {
    ($($t:ty),+) => {
        $(
            impl StaticBounded for $t {
                fn min_ref() -> &'static Self {
                    static MIN: $t = <$t>::min_value();
                    &MIN
                }
            }

            impl KeyBounded<$t> for $t {
                fn lower_bound(&self) -> &$t {
                    Self::min_ref()
                }
            }
        )*
    }
}

impl_integer!(usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128);
