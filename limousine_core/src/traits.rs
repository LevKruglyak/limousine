use num::PrimInt;
use serde::{Deserialize, Serialize};
use trait_set::trait_set;

// Until `trait_alias` is stabilized, we have to use a macro
trait_set! {
    /// A simple address trait,
    pub trait Address = Eq + Clone + 'static;

    pub trait DiskAddress = Address + Default + Serialize + for<'de> Deserialize<'de>;

    /// General value type, thread-safe
    pub trait Value = Send + Sync + Default + Copy + 'static;

    /// General key type, thread safe, and primitive integer type
    pub trait Key = Value + PrimInt + StaticBounded;
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
        )*
    }
}

impl_integer!(usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128);
