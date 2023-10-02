use super::Segmentation;
use crate::kv::*;
use crate::learned::generic::*;
use num::{Bounded, CheckedSub, Num, NumCast};

/// A simple linear model for a key-rank segment of data.
#[derive(Copy, Clone, Debug)]
pub struct LinearModel<K, const EPSILON: usize> {
    pub key: K,
    pub slope: f64,
    pub intercept: i32,
}

impl<K, const EPSILON: usize> LinearModel<K, EPSILON> {
    pub fn new(key: K, slope: f64, intercept: i32) -> Self {
        debug_assert!(slope.is_normal());
        Self {
            key,
            slope,
            intercept,
        }
    }

    /// Create a segment which always approximates the intercept
    pub fn intercept(n: usize) -> Self
    where
        K: Bounded,
    {
        Self {
            key: K::min_value(),
            slope: 0.0,
            intercept: n as i32,
        }
    }
}

impl<K, const EPSILON: usize> KeyBounded<K> for LinearModel<K, EPSILON> {
    fn lower_bound(&self) -> &K {
        &self.key
    }
}

impl<K, const EPSILON: usize> Model<K> for LinearModel<K, EPSILON>
where
    K: CheckedSub + NumCast + Bounded + 'static,
{
    fn approximate(&self, key: &K) -> ApproxPos {
        // To support generic floats, we need all these shenanigans
        // TODO: check on godbolt that this is optimized away
        let pos = num::cast::<f64, i64>(
            self.slope
                * num::cast::<K, f64>(
                    key.checked_sub(self.lower_bound())
                        .unwrap_or(K::min_value()),
                )
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

/// The optimal linear segmentation algorithm, translated
/// almost directly from the official PGMIndex repository
pub struct PGMSegmentation;

impl<K, const EPSILON: usize> Segmentation<K, LinearModel<K, EPSILON>> for PGMSegmentation
where
    K: Key,
{
    fn make_segmentation(
        key_ranks: impl Iterator<Item = (K, usize)>,
    ) -> Vec<LinearModel<K, EPSILON>> {
        let key_ranks: Vec<(K, usize)> = key_ranks.collect();

        let in_fun = |i: usize| {
            let x: K = key_ranks[i].0;
            // Here there is an adjustment for inputs with duplicate keys: at the end
            // of a run of duplicate keys equal to x=first[i] such that
            // x+1!=first[i+1], we map the values x+1,...,first[i+1]-1 to their
            // correct rank i
            let flag = i > 0
                && i + 1 < key_ranks.len()
                && x == key_ranks[i - 1].0
                && x != key_ranks[i + 1].0
                && x + K::one() != key_ranks[i + 1].0;

            if flag {
                (x + K::one(), i)
            } else {
                (x, i)
            }
        };

        let mut segments: Vec<LinearModel<K, EPSILON>> = vec![];
        let mut model = OptimalPiecewiseLinearModel::<K, EPSILON>::new();

        let mut p = in_fun(0);
        model.add_point(p);

        // TODO: fix the off-by-one error here
        for i in 1..(key_ranks.len() - 1) {
            let next_p = in_fun(i);
            if next_p.1 == p.1 {
                continue;
            }

            p = next_p;
            if !model.add_point(p) {
                let seg = model.get_segment();
                segments.push(seg.into());
                model.add_point(p);
            }
        }

        let seg = model.get_segment();
        segments.push(seg.into());

        segments
    }
}

type KeySigned = i128;

fn key_cast<K: Key>(key: K) -> KeySigned {
    num::cast::<K, KeySigned>(key).unwrap()
}

/// A slope of key-rank pairs
#[derive(Clone, Copy)]
struct Slope {
    dx: KeySigned,
    dy: i128,
}

/// Represents a key-rank pair
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point<K: Key> {
    x: K,
    y: usize,
}

impl<K: Key> Default for Point<K> {
    fn default() -> Self {
        Self { x: K::zero(), y: 0 }
    }
}

#[derive(Debug)]
struct CanonicalSegment<K: Key> {
    rectangle: [Point<K>; 4],
    first: K,
}

struct OptimalPiecewiseLinearModel<K: Key, const EPSILON: usize> {
    lower: Vec<Point<K>>,
    upper: Vec<Point<K>>,
    first_x: K,
    last_x: K,
    lower_start: usize,
    upper_start: usize,
    points_in_hull: usize,
    rectangle: [Point<K>; 4],
}

impl<K: Key, const EPSILON: usize> OptimalPiecewiseLinearModel<K, EPSILON> {
    fn new() -> Self {
        Self {
            lower: Vec::with_capacity(1 << 16),
            upper: Vec::with_capacity(1 << 16),
            first_x: K::min_value(),
            last_x: K::min_value(),
            lower_start: 0,
            upper_start: 0,
            points_in_hull: 0,
            rectangle: [Default::default(); 4],
        }
    }

    fn add_point(&mut self, pair: (K, usize)) -> bool {
        let x = pair.0;
        let y = pair.1;

        if self.points_in_hull > 0 && x <= self.last_x {
            // println!("Points must be increasing by x. {x:?} {:?}", self.last_x);
            return false;
        }

        self.last_x = x;

        let p1 = Point {
            x,
            y: y.saturating_add(EPSILON),
        };
        let p2 = Point {
            x,
            y: y.saturating_sub(EPSILON),
        };

        if self.points_in_hull == 0 {
            self.first_x = x;
            self.rectangle[0] = p1;
            self.rectangle[1] = p2;
            self.upper.clear();
            self.lower.clear();
            self.upper.push(p1);
            self.lower.push(p2);
            self.upper_start = 0;
            self.lower_start = 0;
            self.points_in_hull += 1;
            return true;
        }

        if self.points_in_hull == 1 {
            self.rectangle[2] = p2;
            self.rectangle[3] = p1;
            self.upper.push(p1);
            self.lower.push(p2);
            self.points_in_hull += 1;
            return true;
        }

        let slope1 = self.rectangle[2] - self.rectangle[0];
        let slope2 = self.rectangle[3] - self.rectangle[1];
        let outside_line1 = p1 - self.rectangle[2] < slope1;
        let outside_line2 = p2 - self.rectangle[3] > slope2;

        if outside_line1 || outside_line2 {
            self.points_in_hull = 0;
            return false;
        }

        if p1 - self.rectangle[1] < slope2 {
            // Find extreme slope
            let mut min = self.lower[self.lower_start] - p1;
            let mut min_i = self.lower_start;
            for i in (self.lower_start + 1)..self.lower.len() {
                let val = self.lower[i] - p1;
                if val > min {
                    break;
                }
                min = val;
                min_i = i;
            }

            self.rectangle[1] = self.lower[min_i];
            self.rectangle[3] = p1;
            self.lower_start = min_i;

            // Hull update
            let mut end = self.upper.len();
            while end >= self.upper_start + 2
                && cross(self.upper[end - 2], self.upper[end - 1], p1) <= 0 as KeySigned
            {
                end -= 1;
            }

            self.upper.resize(end, Default::default());
            self.upper.push(p1);
        }

        if p2 - self.rectangle[0] > slope1 {
            // Find extreme slope
            let mut max = self.upper[self.upper_start] - p2;
            let mut max_i = self.upper_start;
            for i in (self.upper_start + 1)..self.upper.len() {
                let val = self.upper[i] - p2;
                if val < max {
                    break;
                }
                max = val;
                max_i = i;
            }

            self.rectangle[0] = self.upper[max_i];
            self.rectangle[2] = p2;
            self.upper_start = max_i;

            // Hull update
            let mut end = self.lower.len();
            while end >= self.lower_start + 2
                && cross(self.lower[end - 2], self.lower[end - 1], p2) >= 0 as KeySigned
            {
                end -= 1;
            }

            self.lower.resize(end, Default::default());
            self.lower.push(p2);
        }

        self.points_in_hull += 1;
        true
    }

    fn get_segment(&self) -> CanonicalSegment<K> {
        if self.points_in_hull == 1 {
            return CanonicalSegment::diagonal(self.rectangle[0], self.rectangle[1], self.first_x);
        }

        CanonicalSegment::new(self.rectangle, self.first_x)
    }

    // fn reset(&mut self) {
    //     self.points_in_hull = 0;
    //     self.lower.clear();
    //     self.upper.clear();
    // }
}

fn cross<K: Key>(o: Point<K>, a: Point<K>, b: Point<K>) -> KeySigned {
    let oa = a - o;
    let ob = b - o;
    oa.dx * (ob.dy as KeySigned) - (oa.dy as KeySigned) * ob.dx
}

impl<K: Key, const EPSILON: usize> Into<LinearModel<K, EPSILON>> for CanonicalSegment<K> {
    fn into(self) -> LinearModel<K, EPSILON> {
        let (cs_slope, cs_intercept) = self.get_floating_point_segment(self.first);
        LinearModel::new(self.first, cs_slope, cs_intercept as i32)
    }
}

impl<K: Key> CanonicalSegment<K> {
    fn diagonal(p0: Point<K>, p1: Point<K>, first: K) -> Self {
        Self {
            rectangle: [p0, p1, p0, p1],
            first,
        }
    }

    fn new(rectangle: [Point<K>; 4], first: K) -> Self {
        Self { rectangle, first }
    }

    fn one_point(&self) -> bool {
        self.rectangle[0] == self.rectangle[2] && self.rectangle[1] == self.rectangle[3]
    }

    // fn get_intersection(&self) -> (f64, f64) {
    //     let p0 = self.rectangle[0];
    //     let p1 = self.rectangle[1];
    //     let p2 = self.rectangle[2];
    //     let p3 = self.rectangle[3];
    //     let slope1 = p2 - p0;
    //     let slope2 = p3 - p1;
    //
    //     if self.one_point() || slope1 == slope2 {
    //         return (num::cast::<K, f64>(p0.x).unwrap(), p0.y as f64);
    //     }
    //
    //     let p0p1 = p1 - p0;
    //     let a = slope1.dx * slope2.dy - slope1.dy * slope2.dx;
    //     let b = (p0p1.dx * slope2.dy - p0p1.dy * slope2.dx) as f64 / (a as f64);
    //     let i_x = num::cast::<K, f64>(p0.x).unwrap() + b * slope1.dx as f64;
    //     let i_y = p0.y as f64 + b * slope1.dy as f64;
    //     (i_x, i_y)
    // }

    fn get_floating_point_segment(&self, origin: K) -> (f64, i128) {
        if self.one_point() {
            return (
                0.0,
                (self.rectangle[0].y as i128 + self.rectangle[1].y as i128) / 2,
            );
        }

        // Integral verson of the method (rounding version)
        let slope = self.rectangle[3] - self.rectangle[1];
        let intercept_n = slope.dy * key_cast(origin - self.rectangle[1].x);
        let intercept_d = slope.dx;
        let rounding_term = (if (intercept_n < 0) ^ (intercept_d < 0) {
            -1
        } else {
            1
        }) * intercept_d
            / 2;
        let intercept = (intercept_n + rounding_term) / intercept_d + self.rectangle[1].y as i128;

        (slope.slope(), intercept)
    }

    // fn get_slope_range(&self) -> (f64, f64) {
    //     if self.one_point() {
    //         return (0.0, 1.0);
    //     }
    //
    //     let min_slope = (self.rectangle[2] - self.rectangle[0]).slope();
    //     let max_slope = (self.rectangle[3] - self.rectangle[1]).slope();
    //     (min_slope, max_slope)
    // }
}

impl Slope {
    fn slope(&self) -> f64 {
        self.dy as f64 / self.dx as f64
    }
}

impl PartialEq for Slope {
    fn eq(&self, other: &Self) -> bool {
        self.dy * (other.dx as i128) == (self.dx as i128) * other.dy
    }
}

impl PartialOrd for Slope {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.dy * (other.dx as i128)).partial_cmp(&((self.dx as i128) * other.dy))
    }
}

impl<K: Key> std::ops::Sub for Point<K> {
    type Output = Slope;

    fn sub(self, rhs: Self) -> Self::Output {
        Slope {
            dx: key_cast(self.x) - key_cast(rhs.x),
            dy: self.y as i128 - rhs.y as i128,
        }
    }
}
