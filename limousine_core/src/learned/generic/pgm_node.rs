use crate::Key;

use super::{pgm::PGMSegmentation, ApproxPos, Model, PiecewiseModel};
use std::borrow::Borrow;

/// A simple linear model for a key-rank segment of data.
#[derive(Copy, Clone, Debug)]
pub struct LinearModel<K, const EPSILON: usize> {
    key: K,
    slope: f64,
    intercept: i32,
}

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
