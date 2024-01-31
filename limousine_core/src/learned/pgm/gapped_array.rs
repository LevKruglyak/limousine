//! Helper struct to implement a gapped array to act as the base of data nodes
//! ASSUMPTIONS:
//! - Elements (T) are unique

pub struct GappedArray<T>
where
    T: Default + Copy + Clone + PartialEq + PartialOrd,
{
    pub bitmap: Box<[bool]>,
    pub data: Box<[T]>,
    pub num_els: usize,
}

impl<T: Default + Copy + Clone + PartialEq + PartialOrd> GappedArray<T> {
    /// Creates an empty gapped array with the given size
    pub fn new(size: usize) -> Self {
        let bitmap_vec = vec![false; size];
        let data_vec = vec![T::default(); size];
        Self {
            bitmap: bitmap_vec.into_boxed_slice(),
            data: data_vec.into_boxed_slice(),
            num_els: 0,
        }
    }

    /// Helper function to implement next occupied and next free
    /// TODO: Bithacks to make faster
    fn next_ix_helper(&self, mut ix: usize, val: bool) -> Option<usize> {
        while ix < self.bitmap.len() && self.bitmap[ix] != val {
            ix += 1
        }
        if ix < self.bitmap.len() {
            Some(ix)
        } else {
            None
        }
    }

    /// Returns the next occupied slot in the range [ix, end]
    fn next_occupied_ix(&self, mut ix: usize) -> Option<usize> {
        self.next_ix_helper(ix, true)
    }

    /// Returns the next free slot in the range [ix, end]
    fn next_free_ix(&self, mut ix: usize) -> Option<usize> {
        self.next_ix_helper(ix, false)
    }

    /// Helper function to implement prev occupied and prev free
    /// TODO: Bithacks to make faster
    fn prev_ix_helper(&self, mut ix: usize, val: bool) -> Option<usize> {
        loop {
            if self.bitmap[ix] == val {
                return Some(ix);
            }
            if ix == 0 {
                return None;
            } else {
                ix -= 1;
            }
        }
    }

    /// Returns the previous occupied slot in the range [start, ix]
    fn prev_occupied_ix(&self, mut ix: usize) -> Option<usize> {
        self.prev_ix_helper(ix, true)
    }

    /// Returns the previous free slot in the range [start, ix]
    fn prev_free_ix(&self, mut ix: usize) -> Option<usize> {
        self.prev_ix_helper(ix, false)
    }

    /// Returns the smallest ix >= hint s.t. the element at this ix and all elements to the right are strictly greater than needle
    /// NOTE: If needle is larger than everything in the array, this returns bitmap.len()
    fn lub(&self, needle: &T, hint: Option<usize>) -> usize {
        let mut check = self.next_occupied_ix(hint.unwrap_or(0));
        while check.is_some() && self.data[check.unwrap()] < *needle {
            check = self.next_occupied_ix(check.unwrap() + 1);
        }
        match check {
            Some(ix) => ix,
            None => self.bitmap.len(),
        }
    }

    /// Returns the greatest ix <= hint s.t. the element at this ix and all elements to the left are less than or equal to needle.
    /// NOTE: If needle is smaller than everything in the array, this returns 0
    fn glb(&self, needle: &T, hint: Option<usize>) -> usize {
        let mut check = self.prev_occupied_ix(hint.unwrap_or(self.bitmap.len() - 1));
        while check.is_some() && self.data[check.unwrap()] > *needle {
            if check.unwrap() == 0 {
                return 0;
            }
            check = self.prev_occupied_ix(check.unwrap() - 1);
        }
        match check {
            Some(ix) => ix,
            None => 0,
        }
    }

    /// Helper function to check if a given ix is correct sorted position for an element
    pub fn is_ix_correct(&self, needle: &T, ix: usize) -> bool {
        let prev = if ix == 0 { None } else { self.prev_occupied_ix(ix - 1) };
        let next = self.next_occupied_ix(ix + 1);
        (prev.is_none() || self.data[prev.unwrap()] < *needle) && (next.is_none() || *needle < self.data[next.unwrap()])
    }

    /// If `needle` exists in the gapped array, it must exist at the index returned by this function
    /// If it does not exist and you wish to insert it, insert it _before_ the index returned
    /// Examples:
    /// - Inserting 3 into 2, 4, 6 would return 1
    /// - Inserting 1 into 2, 4, 6 would return 0
    /// - Inserting 7 into 2, 4, 6 would return 3
    pub fn search_helper(&self, needle: &T, hint: Option<usize>) -> usize {
        let lb = self.lub(needle, hint);
        if self.is_ix_correct(needle, lb) {
            return lb;
        }
        self.glb(needle, hint)
    }

    /// Search the gapped array for a specific value, using a starting hint
    /// TODO: Make exponential search
    pub fn search_with_hint(&mut self, needle: &T, hint: usize) -> Option<&T> {
        let ix = self.search_helper(needle, Some(hint));
        if self.bitmap[ix] && self.data[ix] == *needle {
            return Some(&self.data[ix]);
        } else {
            return None;
        }
    }

    /// Insert a specific value into the array with the given hint
    pub fn insert_with_hint(&mut self, value: T, hint: usize) -> Result<(), String> {
        let ix = self.search_helper(&value, Some(hint));
        let shift_left_ix = if ix == 0 { None } else { self.prev_free_ix(ix - 1) };
        let shift_right_ix = self.next_free_ix(ix + 1);
        match (shift_left_ix, shift_right_ix) {
            (Some(lix), Some(rix)) => {
                if lix.abs_diff(ix) < rix.abs_diff(ix) {
                    self.bitmap.copy_within(lix + 1..ix + 1, lix);
                    self.data.copy_within(lix + 1..ix + 1, lix);
                } else {
                    self.bitmap.copy_within(ix..rix, ix + 1);
                    self.data.copy_within(ix..rix, ix + 1);
                }
            }
            (Some(lix), None) => {
                self.bitmap.copy_within(lix + 1..ix + 1, lix);
                self.data.copy_within(lix + 1..ix + 1, lix);
            }
            (None, Some(rix)) => {
                self.bitmap.copy_within(ix..rix, ix + 1);
                self.data.copy_within(ix..rix, ix + 1);
            }
            (None, None) => return Err("Gapped array is full".to_string()),
        }
        self.bitmap[ix] = true;
        self.data[ix] = value;
        return Ok(());
    }
}
