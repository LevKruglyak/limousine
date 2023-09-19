//! This file defines the Model portion of the PGM, which is simply just a linear approximator

use crate::{
    kv::Key,
    learned::generic::{ApproxPos, Model},
};

use std::borrow::Borrow;

use super::pgm_segmentation::PGMSegmentation;

/// A simple linear model for a key-rank segment of data.
/// K: The type of keys in the model
/// EPSILON: The maximum error bound for approximations into this model
#[derive(Copy, Clone, Debug)]
pub struct LinearModel<K, const EPSILON: usize> {
    key: K,
    slope: f64,
    intercept: i32,
}

/// Convenience methods on a LinearModel, most notably creation given key, slope, intercept
/// NOTE: the provided `key` must represent the smallest key indexed by this model
impl<K: Key, const EPSILON: usize> LinearModel<K, EPSILON> {
    pub fn new(key: K, slope: f64, intercept: i32) -> Self {
        debug_assert!(slope.is_normal());
        Self {
            key,
            slope,
            intercept,
        }
    }
}

/// Allows us to use a model as a key, which should represent the smallest key indexed by this model
impl<K: Key, const EPSILON: usize> Borrow<K> for LinearModel<K, EPSILON> {
    fn borrow(&self) -> &K {
        &self.key
    }
}

/// Implement LinearModel as a Model, meaning we can use it to approximate
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

#[cfg(test)]
mod pgm_model_tests {
    use super::*;

    #[test]
    fn pgm_model_basic() {
        const EPS: usize = 2;
        let key: usize = 10;
        let slope: f64 = 1.0;
        let slope_usize: usize = 1;
        let intercept: i32 = 6;
        let model: LinearModel<usize, EPS> = LinearModel::new(key, slope, intercept);
        for test in 20..1000 {
            let test: usize = test;
            let approx = model.approximate(&test);
            let expected_lo = (test - key) * slope_usize + (intercept as usize) - EPS;
            let expected_hi = expected_lo + EPS * 2 + 2;
            assert!(approx.lo == expected_lo);
            assert!(approx.hi == expected_hi);
        }
    }
}
