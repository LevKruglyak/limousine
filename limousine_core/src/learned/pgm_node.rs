use std::borrow::Borrow;

use bytemuck::{Pod, Zeroable};

use crate::{ApproxPos, Key};

use super::{pgm::PGMSegmentation, Model, PiecewiseModel};

/// A simple linear model for a key-rank segment of data.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct LinearModel<K, const EPSILON: usize> {
    key: K,
    slope: f64,
    intercept: i32,
}

// SAFETY: this is safe by the `Zeroable` rules, but we want to avoid
// dependencies on unstable features
unsafe impl<K, const EPSILON: usize> Zeroable for LinearModel<K, EPSILON> {}

// SAFETY: this violates the padding rule of `Pod`, so transmuting this
// into any other `Pod` type would lead to a UB violation: specifically
// treating uninitialized data as initialized data. We only need this type
// to be `Pod` to persist to a file, and it is used internally so this isn't
// a big issue.
unsafe impl<K: Copy + 'static, const EPSILON: usize> Pod for LinearModel<K, EPSILON> {}

impl<K: Key, const EPSILON: usize> LinearModel<K, EPSILON> {
    pub fn new(key: K, slope: f64, intercept: i32) -> Self {
        debug_assert!(slope.is_normal());
        Self {
            key,
            slope,
            intercept,
        }
    }

    /// Create a segment which always approximates the intercept
    pub fn intercept(n: usize) -> Self {
        Self {
            key: K::max_value(),
            slope: 0.0,
            intercept: n as i32,
        }
    }
}

impl<K: Key, const EPSILON: usize> Borrow<K> for LinearModel<K, EPSILON> {
    fn borrow(&self) -> &K {
        &self.key
    }
}

impl<K: Key, const EPSILON: usize> Model<K> for LinearModel<K, EPSILON> {
    fn approximate(&self, key: &K) -> ApproxPos {
        // To support generic floats, we need all these shenanigans
        // TODO: check on godbolt that this is optimized away
        let pos = num::cast::<f64, i64>(
            self.slope
                * num::cast::<K, f64>(key.checked_sub(self.borrow()).unwrap_or(K::min_value()))
                    .unwrap(),
        )
        .unwrap()
            + (self.intercept as i64);

        let pos = pos.max(0) as usize;

        ApproxPos {
            lo: pos.saturating_sub(EPSILON),
            hi: pos + EPSILON + 2,
        }
    }
}

/// A `PGMLayer` is an `InternalLayer` consisting of linear models with `EPSILON`
/// controlled error, and build by a `PGM` segmentation algorithm.
pub type PGMLayer<K, const EPSILON: usize> =
    PiecewiseModel<K, LinearModel<K, EPSILON>, PGMSegmentation>;
