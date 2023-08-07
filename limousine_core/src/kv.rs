use num::PrimInt;
use std::fmt::Debug;
use trait_set::trait_set;

pub trait StaticBounded: 'static {
    fn min_ref() -> &'static Self;
}

// Until `trait_alias` is stabilized, we have to use a macro
trait_set! {
    /// General value type, thread-safe
    pub trait Value = Send + Sync + Debug + Default + Copy + 'static;

    /// General key type, thread safe, and primitive integer type
    pub trait Key = Value + PrimInt + StaticBounded;
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
