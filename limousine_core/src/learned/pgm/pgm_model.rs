//! This file defines the Model portion of the PGM, which is simply just a
//! linear approximator.
//! NOTE: We are making a simplification and forcing approximation lines
//! to pass through the origin, which slightly degrades performance

use crate::{
    component::Key,
    learned::generic::{ApproxPos, LearnedModel},
};
use std::borrow::Borrow;

/// A simple linear model for a key-rank segment of data.
#[derive(Copy, Clone, Debug)]
pub struct LinearModel<K, const EPSILON: usize> {
    /// Define the approximation line. See note at top of file about forcing
    /// approximations to pass through the origin.
    pub key: K,
    pub slope: f64,
    /// How many entries are indexed by this model. Not strictly needed but
    /// useful for debugging.
    pub size: usize,
}
impl<K: Key, const EPSILON: usize> LinearModel<K, EPSILON> {
    /// Construct a new model from the smallest key, slope, and size
    pub fn new(key: K, slope: f64, size: usize) -> Self {
        debug_assert!(slope.is_normal());
        Self { key, slope, size }
    }

    /// Construct a sentinel model which will sit at the end of a layer
    pub fn sentinel() -> Self {
        Self {
            key: K::max_value(),
            slope: 0.0,
            size: 0,
        }
    }
}

/// Functionality for borrowing a model as it's minimum key
impl<K: Key, const EPSILON: usize> Borrow<K> for LinearModel<K, EPSILON> {
    fn borrow(&self) -> &K {
        &self.key
    }
}
impl<K: Key, const EPSILON: usize> Borrow<K> for &LinearModel<K, EPSILON> {
    fn borrow(&self) -> &K {
        &self.key
    }
}
impl<K: Key, const EPSILON: usize> Borrow<K> for &mut LinearModel<K, EPSILON> {
    fn borrow(&self) -> &K {
        &self.key
    }
}

/// Actual approximation logic for linear models
impl<K: Key, const EPSILON: usize> LearnedModel<K> for LinearModel<K, EPSILON> {
    fn approximate(&self, key: &K) -> ApproxPos {
        let run = num::cast::<K, f64>(key.clone().saturating_sub(self.key)).unwrap();
        let pos = (run * self.slope).floor() as i64;
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
        let model: LinearModel<usize, EPS> = LinearModel::new(key, slope, 6);
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
