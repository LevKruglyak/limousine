//! Defines the segmentation algorithm for PGM layers.

use super::pgm_model::LinearModel;
use crate::learned::generic::*;
use crate::{common::entry::Entry, kv::*};
use generational_arena::{Arena, Index};
use num::{Bounded, CheckedSub, Num, NumCast};

/// The optimal linear segmentation algorithm, translated
/// almost directly from the official PGMIndex repository
#[derive(Clone)]
pub struct PGMSegmentation;
/*
impl<K, V, const EPSILON: usize> Segmentation<K, V, LinearModel<K, EPSILON>> for PGMSegmentation
where
    K: Key + Clone,
    V: Clone,
{
    /// Given an iterator over entries, constructs the piecewise nodes and returns them as a Vector
    fn make_segmentation(
        mut data: impl Iterator<Item = Entry<K, V>>,
        arena: &mut Arena<PiecewiseNode<K, V, LinearModel<K, EPSILON>>>,
    ) -> Index {
        // Helpful variables and lambda functions for reading/writing
        let mut rank: usize = 0;
        let mut in_fun = || {
            let val = data.next();
            rank += 1;
            match val {
                Some(entry) => Some((rank - 1, entry)),
                None => None,
            }
        };
        let mut head_node_id: Option<Index> = None;
        let mut last_node_id: Option<Index> = None;
        let mut model = OptimalPiecewiseLinearModel::<K, EPSILON>::new();
        let mut entries: Vec<Entry<K, V>> = vec![];
        let mut out_fun = |new_model: OptimalPiecewiseLinearModel<K, EPSILON>,
                           new_entries: Vec<Entry<K, V>>| {
            let seg = new_model.get_segment();
            let node = PiecewiseNode {
                model: seg.into(),
                data: new_entries,
                next: None,
            };
            let new_id = arena.insert(node);
            match last_node_id {
                Some(last_id) => {
                    let last_node = arena.get_mut(last_id);
                    if last_node.is_some() {
                        last_node.unwrap().next = Some(new_id);
                    }
                }
                None => {
                    head_node_id = Some(new_id);
                }
            };
            last_node_id = Some(new_id);
        };

        let Some((rank, entry)) = in_fun() else { panic!() };
        model.add_point((entry.key, rank));
        entries.push(entry);

        let mut next_pair = in_fun();

        while next_pair.is_some() {
            let (rank, entry) = next_pair.unwrap();
            if !model.add_point((entry.key, rank)) {
                out_fun(model.clone(), entries.clone());
                entries.clear();
                model.add_point((entry.key, rank));
            }
            entries.push(entry);
            next_pair = in_fun();
        }
        if model.points_in_hull > 0 {
            out_fun(model, entries.clone());
        }

        head_node_id.unwrap()
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

#[derive(Clone)]
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
            panic!("Points must be increasing by x. {x:?} {:?}", self.last_x);
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

#[cfg(test)]
mod pgm_tests {
    use super::*;
    use crate::learned::generic::Model;

    /// Tests that when we provide a huge list of numbers that are
    /// perfectly linear, the segmentation algorithm returns one
    /// segment with slope basically 1
    #[test]
    fn test_one_big_linear() {
        let mut inp: Vec<Entry<i32, i32>> = Vec::new();
        for i in 0..100_000 {
            inp.push(Entry::new(i, i));
        }
        let mut arena: Arena<PiecewiseNode<i32, i32, LinearModel<i32, 2>>> = Arena::new();
        let head = PGMSegmentation::make_segmentation(inp.into_iter(), &mut arena);
        assert!(arena.len() == 1);
        let node = arena.get(head).unwrap();
        let guess_100 = node.model.approximate(&100);
        println!("{:?}", node.model);
        assert!(guess_100.lo == 98);
        assert!(guess_100.hi == 104);
    }

    /// Tests that when we provide step-like series of points,
    /// we get a corresponding number of models with near-zero slope
    #[test]
    fn test_step_function() {
        let mut inp: Vec<Entry<i32, i32>> = Vec::new();
        const STEPS: usize = 1_000;
        const STEP_SIZE: usize = 10_000;
        const POINTS_PER_STEP: usize = 100;
        for step in 0..STEPS {
            for point in 0..POINTS_PER_STEP {
                let key = step * STEP_SIZE + point;
                inp.push(Entry::new((step * STEP_SIZE + point) as i32, point as i32));
            }
        }
        let mut arena: Arena<PiecewiseNode<i32, i32, LinearModel<i32, 2>>> = Arena::new();
        let nodes = PGMSegmentation::make_segmentation(inp.into_iter(), &mut arena);
        assert!(arena.len() == STEPS);
    }
}
 */
