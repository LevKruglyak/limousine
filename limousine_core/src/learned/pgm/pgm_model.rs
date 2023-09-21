//! This file defines the Model portion of the PGM, which is simply just a linear approximator

use crate::{
    kv::Key,
    learned::generic::{ApproxPos, Model},
};

use std::borrow::Borrow;

/// A simple linear model for a key-rank segment of data.
/// K: The type of keys in the model
/// EPSILON: The maximum error bound for approximations into this model
#[derive(Copy, Clone, Debug)]
pub struct LinearModel<K, const EPSILON: usize> {
    // The smallest key indexed by this model
    pub key: K,
    /// Why don't we need an intercept?
    /// In our structures, each model will view the data its indexing as having offset 0
    /// This is because we need to "fracture" the underlying data representation so it's not looking at
    /// a huge layer, which would make inserts/updates/etc. nearly impossible.
    /// This implementation is not optimal w.r.t minimizing the number of segments, but it is almost certainly
    /// close and makes the actual segmentation algorithm + logic must simpler
    pub slope: f64,
}

/// Convenience methods on a LinearModel, most notably creation given key, slope, intercept
/// NOTE: the provided `key` must represent the smallest key indexed by this model
impl<K: Key, const EPSILON: usize> LinearModel<K, EPSILON> {
    pub fn new(key: K, slope: f64) -> Self {
        debug_assert!(slope.is_normal());
        Self { key, slope }
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
            self.slope * num::cast::<K, f64>(key.checked_sub(self.borrow()).unwrap_or(K::min_value())).unwrap(),
        )
        .unwrap();

        let pos = pos.max(0) as usize;

        ApproxPos {
            lo: pos.saturating_sub(EPSILON),
            hi: pos + EPSILON + 2,
        }
    }
}

/// Simple component with simple test(s)
#[cfg(test)]
mod pgm_model_tests {
    use super::*;

    #[test]
    fn pgm_model_basic() {
        const EPS: usize = 2;
        let key: usize = 10;
        let slope: f64 = 1.0;
        let slope_usize: usize = 1;
        let model: LinearModel<usize, EPS> = LinearModel::new(key, slope);
        for test in 20..1000 {
            let test: usize = test;
            let approx = model.approximate(&test);
            let expected_lo = (test - key) * slope_usize - EPS;
            let expected_hi = expected_lo + EPS * 2 + 2;
            assert!(approx.lo == expected_lo);
            assert!(approx.hi == expected_hi);
        }
    }
}
