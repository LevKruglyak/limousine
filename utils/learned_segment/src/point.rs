use std::ops::Sub;

use num::PrimInt;

#[derive(Clone)]
pub struct Point<K: PrimInt> {
    x: K,
    y: i32,
}

impl<K: PrimInt> Point<K> {
    pub fn new(x: K, y: i32) -> Self {
        Self { x, y }
    }

    /// Slope of the line connecting (0,0) to this point.
    pub fn slope(self) -> f64 {
        let run = num::cast::<K, f64>(self.x).unwrap();
        (self.y as f64) / run
    }
}

impl<K: PrimInt> Sub<Self> for Point<K> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Point::new(self.x.saturating_sub(rhs.x), self.y.saturating_sub(rhs.y))
    }
}
