use std::borrow::Borrow;

/// Returns the `upper bound` index from a search result
#[allow(unused)]
pub fn upper_bound(search: Result<usize, usize>, cap: usize) -> usize {
    match search {
        Ok(index) => index,
        Err(index) => index.min(cap),
    }
}

/// Returns the `lower bound` index from a search result
#[allow(unused)]
pub fn lower_bound(search: Result<usize, usize>) -> usize {
    match search {
        Ok(index) => index,
        Err(index) => index.saturating_sub(1),
    }
}

/// Some algorithm for searching a slice, e.g. Binary or Linear
pub trait Search {
    /// Same as calling `search_by_key` when `T` == `K`.
    fn search<T: Ord + Copy>(slice: &[T], x: &T) -> Result<usize, usize> {
        Self::search_by_key(slice, x)
    }

    /// If the value is found in the slice, will return `Ok(index)` with the
    /// index of the found value. If it is not found, will return `Err(index)`
    /// with the index of where the value should be inserted to maintain the
    /// sorted order.
    ///
    /// This method assumes that the given slice is sorted, and contains no duplicates
    fn search_by_key<K: Ord + Copy, T: Borrow<K>>(slice: &[T], x: &K) -> Result<usize, usize>;

    /// Same as calling `search`, but adjusts for the case when
    /// slice's index 0 is actually offset in some larger slice
    fn search_with_offset<T: Ord + Copy>(
        slice: &[T],
        x: &T,
        offset: usize,
    ) -> Result<usize, usize> {
        Self::search_by_key(slice, x)
            .map(|x| x + offset)
            .map_err(|x| x + offset)
    }

    /// Same as calling `search_by_key`, but adjusts for the case
    /// when the slice's index 0 is actually offset in some larger slice
    fn search_by_key_with_offset<K: Ord + Copy, T: Borrow<K>>(
        slice: &[T],
        x: &K,
        offset: usize,
    ) -> Result<usize, usize> {
        Self::search_by_key(slice, x)
            .map(|x| x + offset)
            .map_err(|x| x + offset)
    }
}

/// Performs a binary search on a slice, with computational complexity `O(log n)`
/// However, for small searches, a linear search may be faster.
pub struct BinarySearch;

impl Search for BinarySearch {
    fn search_by_key<K: Ord + Copy, T: Borrow<K>>(slice: &[T], x: &K) -> Result<usize, usize> {
        slice.binary_search_by_key(x, |x| *x.borrow())
    }
}

/// Performs a simple linear search on a slice, with computational complexity `O(n)`
pub struct LinearSearch;

impl Search for LinearSearch {
    fn search_by_key<K: Ord + Copy, T: Borrow<K>>(slice: &[T], x: &K) -> Result<usize, usize> {
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

/// If a slice has less than `LINEAR_SEARCH_THRESHOLD` bytes, use a linear search
/// TODO: experimentally determine
const LINEAR_SEARCH_THRESHOLD: usize = 8 * 64;

/// Decides whether to use a linear or binary search based on some experimentally determined
/// threshold on the size of the given slice.
pub struct OptimalSearch;

impl Search for OptimalSearch {
    fn search_by_key<K: Ord + Copy, T: Borrow<K>>(slice: &[T], x: &K) -> Result<usize, usize> {
        if slice.len() * std::mem::size_of::<T>() > LINEAR_SEARCH_THRESHOLD {
            BinarySearch::search_by_key(slice, x)
        } else {
            LinearSearch::search_by_key(slice, x)
        }
    }
}

#[cfg(test)]
mod search_tests {
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
    fn linear_search() {
        let array = [1, 2, 3, 4, 7, 10, 24, 55, 56, 57, 100];
        for i in -10..110 {
            assert_eq!(
                BinarySearch::search(&array[..], &i),
                LinearSearch::search(&array[..], &i)
            );
        }
    }

    #[test]
    fn optimal_search() {
        let array = [1, 2, 3, 4, 7, 10, 24, 55, 56, 57, 100];
        for i in -10..110 {
            assert_eq!(
                BinarySearch::search(&array[..], &i),
                OptimalSearch::search(&array[..], &i)
            );
        }
    }
}
