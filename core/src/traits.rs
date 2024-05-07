use lazy_static::lazy_static;
use num::PrimInt;
use serde::{Deserialize, Serialize};
use trait_set::trait_set;

// Until `trait_alias` is stabilized, we have to use a macro
trait_set! {
    /// A simple address trait,
    pub trait Address = Eq + Clone + 'static;

    /// General trait for types which are serialized to disk
    pub trait Persisted = Serialize + for<'de> Deserialize<'de> + Clone + Default + Eq + 'static;

    /// General key type
    pub trait Key = PrimInt + Clone + StaticBounded + 'static ;

    /// General value type
    pub trait Value = Clone + 'static;
}

pub trait KeyBounded<K> {
    fn lower_bound(&self) -> &K;
}

pub trait StaticBounded: Ord + 'static {
    fn min_ref() -> &'static Self;

    fn max_ref() -> &'static Self;
}

macro_rules! impl_integer {
    ($($t:ty),+) => {
        $(
            impl StaticBounded for $t {
                fn min_ref() -> &'static Self {
                    static MIN: $t = <$t>::min_value();
                    &MIN
                }

                fn max_ref() -> &'static Self {
                    static MAX: $t = <$t>::max_value();
                    &MAX
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

lazy_static! {
    static ref MIN_STRING: String = "".to_string();
    static ref MAX_STRING: String = "".to_string();
}

impl StaticBounded for String {
    fn min_ref() -> &'static Self {
        &MIN_STRING
    }

    fn max_ref() -> &'static Self {
        &MAX_STRING
    }
}
