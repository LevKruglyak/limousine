#![no_std]

//! A collection of algorithms for searching within slices.
//!
//! This module provides different search strategies and utilities to work with sorted slices.
//! Currently, it supports binary and linear search algorithms, as well as an optimal search
//! algorithm which picks between binary and linear searches depending on the size of the slice.
#![deny(missing_docs)]

/// Returns the index of the smallest element greater than or equal to the search
/// key.
///
/// # Example
/// ```
/// use slice_search::*;
///
/// let array = [0, 1, 2, 3, 4, 6];
///
/// let search_result = BinarySearch::search(&array[..], &5);
/// assert_eq!(upper_bound(search_result, 6), 5);
///
/// let search_result = BinarySearch::search(&array[..], &10);
/// assert_eq!(upper_bound(search_result, 6), 6);
/// ```
pub fn upper_bound(search: Result<usize, usize>, cap: usize) -> usize {
    match search {
        Ok(index) => index,
        Err(index) => index.min(cap),
    }
}

/// Returns the index of the largest element less than or equal to the search
/// key.
///
/// # Example
/// ```
/// use slice_search::*;
///
/// let array = [0, 1, 2, 3, 4, 6];
///
/// let search_result = BinarySearch::search(&array[..], &5);
///
/// assert_eq!(lower_bound(search_result), 4);
/// ```
pub fn lower_bound(search: Result<usize, usize>) -> usize {
    match search {
        Ok(index) => index,
        Err(index) => index.saturating_sub(1),
    }
}

/// Offsets a search result by a constant factor. This is useful when searching
/// on a slice which is a sub-slice of a larger slice.
///
/// # Example
/// ```
/// use slice_search::*;
///
/// let array = [0, 1, 2, 3, 4, 6];
///
/// let search_result = BinarySearch::search(&array[2..5], &5);
///
/// assert_eq!(lower_bound(offset(search_result, 2)), 4);
/// assert_eq!(upper_bound(offset(search_result, 2), 6), 5);
/// ```
pub fn offset(search: Result<usize, usize>, offset: usize) -> Result<usize, usize> {
    match search {
        Ok(index) => Ok(index + offset),
        Err(index) => Err(index + offset),
    }
}

use core::borrow::Borrow;

/// An algorithm for searching a sorted slice, e.g. Binary or Linear
pub trait Search {
    /// Search a slice of `T` by comparing with a given value of `T`
    ///
    /// If the value is found then `Result::Ok` is returned, containing the index of
    /// the matching element. If there are multiple matches, then any one of the matches
    /// could be returned. The index is chosen deterministically, but is subject to change
    /// in future versions of Rust. If the value is not found then `Result::Err` is returned,
    /// containing the index where a matching element could be inserted while maintaining sorted order.
    ///
    /// This method assumes that the given slice is sorted.
    ///
    /// # Example
    ///
    /// ```
    /// use slice_search::*;
    ///
    /// let slice = [1, 2, 3, 5, 8];
    /// let result = BinarySearch::search(&slice, &3);
    /// assert_eq!(result, Ok(2));
    ///
    /// let result = BinarySearch::search(&slice, &6);
    /// assert_eq!(result, Err(4));
    /// ```
    fn search<T: Ord + Clone>(slice: &[T], x: &T) -> Result<usize, usize> {
        Self::search_by_key(slice, x)
    }

    /// Search a slice of `T`, where `T: Borrow<K>` and comparing to a given
    /// value of `K`, using the `Borrow<K>` trait like a key extraction
    /// function.
    ///
    /// If the value is found then `Result::Ok` is returned, containing the index of
    /// the matching element. If there are multiple matches, then any one of the matches
    /// could be returned. The index is chosen deterministically, but is subject to change
    /// in future versions of Rust. If the value is not found then `Result::Err` is returned,
    /// containing the index where a matching element could be inserted while maintaining sorted order.
    ///
    /// This method assumes that the given slice is sorted.
    ///
    /// ```
    /// use slice_search::*;
    ///
    /// struct Object {
    ///     key: i32,
    /// }
    ///
    /// impl core::borrow::Borrow<i32> for Object {
    ///     fn borrow(&self) -> &i32 {
    ///         &self.key
    ///     }
    /// }
    ///
    /// let slice = [
    ///     Object { key: 1 },
    ///     Object { key: 3 },
    ///     Object { key: 5 }
    /// ];
    ///
    /// let result = BinarySearch::search_by_key(&slice, &3);
    /// assert_eq!(result, Ok(1));
    /// ```
    fn search_by_key<K: Ord + Clone, T: Borrow<K>>(slice: &[T], x: &K) -> Result<usize, usize>;
}

/// Performs a binary search on a slice, with computational complexity `O(log n)`
/// However, for small searches, a linear search may be faster.
pub struct BinarySearch;

impl Search for BinarySearch {
    fn search_by_key<K: Ord + Clone, T: Borrow<K>>(slice: &[T], x: &K) -> Result<usize, usize> {
        slice.binary_search_by_key(x, |x| x.borrow().clone())
    }
}

/// Performs a simple linear search on a slice, with computational complexity `O(n)`
pub struct LinearSearch;

impl Search for LinearSearch {
    fn search_by_key<K: Ord, T: Borrow<K>>(slice: &[T], x: &K) -> Result<usize, usize> {
        let mut index = 0;
        let size = slice.len();

        while index < size && unsafe { slice.get_unchecked(index).borrow() } < x {
            index += 1;
        }

        if index >= size {
            Err(size)
        } else if unsafe { slice.get_unchecked(index).borrow() } == x {
            Ok(index)
        } else {
            Err(index)
        }
    }
}

const BINARY_SEARCH_CUTOFF: usize = 1024;

/// Chooses between binary and linear search depending on the size of the slice to search
pub struct OptimalSearch;

impl Search for OptimalSearch {
    fn search_by_key<K: Ord + Clone, T: Borrow<K>>(slice: &[T], x: &K) -> Result<usize, usize> {
        if slice.len() * core::mem::size_of::<K>() > BINARY_SEARCH_CUTOFF {
            BinarySearch::search_by_key(slice, x)
        } else {
            LinearSearch::search_by_key(slice, x)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upper_bound() {
        assert_eq!(upper_bound(Ok(3), 5), 3);
        assert_eq!(upper_bound(Err(3), 5), 3);
        assert_eq!(upper_bound(Err(7), 5), 5);
    }

    #[test]
    fn test_lower_bound() {
        assert_eq!(lower_bound(Ok(3)), 3);
        assert_eq!(lower_bound(Err(3)), 2);
        assert_eq!(lower_bound(Err(0)), 0);
    }

    #[test]
    fn binary_linear_search() {
        let array = [1, 2, 3, 4, 7, 10, 24, 55, 56, 57, 100];
        for i in -10..110 {
            assert_eq!(
                BinarySearch::search(&array[..], &i),
                LinearSearch::search(&array[..], &i)
            );
        }
    }

    #[test]
    fn binary_optimal_search() {
        let array = [1, 2, 3, 4, 7, 10, 24, 55, 56, 57, 100];

        for i in 0..1_000 {
            assert_eq!(
                BinarySearch::search(&array[..], &i),
                OptimalSearch::search(&array[..], &i)
            );
        }
    }
}
