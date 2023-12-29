use trait_set::trait_set;

pub trait KeyBounded<K> {
    fn lower_bound(&self) -> &K;
}

pub trait StaticBounded: 'static + Copy + Ord {
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
        )*
    }
}

impl_integer!(usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128);
