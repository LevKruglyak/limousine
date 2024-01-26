//! Helper struct to implement a gapped array to act as the base of data nodes

pub struct GappedArray<T>
where
    T: Default + Clone + PartialEq + PartialOrd,
{
    pub bitmap: Box<[bool]>,
    pub data: Box<[T]>,
    pub num_els: usize,
}

impl<T: Default + Clone + PartialEq + PartialOrd> GappedArray<T> {
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

    /// Returns the smallest ix >= hint s.t. all elements to the right  of this ix are smaller than needle.
    /// NOTE: If needle is larger than everything in the array, this returns bitmap.len()
    fn lub(&self, needle: &T, hint: usize) -> usize {
        panic!("NOT DONE")
    }

    /// Returns the greatest ix <= hint s.t. the element at this ix and all elements to the right are larger than needle.
    /// NOTE: If needle is smaller than everything in the array, this returns 0
    fn glb(&self, needle: &T, hint: usize) -> usize {
        panic!("NOT DONE")
    }

    /// Search the gapped array for a specific value, using a starting hint
    /// TODO: Make exponential search
    pub fn search_mut_with_hint(&mut self, needle: &T, hint: usize) -> Option<&mut T> {
        let ub = self.lub(needle, hint);
        if ub >= self.bitmap.len() {
            return None;
        }
        let lb = self.glb(needle, hint);
        if lb == 0 {
            return None;
        }
        // TODO: Searhc (lb, ub)
        // Likely should do linear if it's small, and binary search if it's not
        panic!("NOT DONE")
    }

    /// Insert a specific value into the array with the given hint
    pub fn insert_with_hint(&mut self, value: T, hint: usize) -> Result<(), String> {
        if (self.num_els >= self.data.len()) {
            return Err("Array full".to_string());
        }
        let lub = self.lub(&value, hint);
        let goal_ix = if lub > hint

        // First get the neighbors
        let next_ix = if hint < self.data.len() - 1 {
            self.next_occupied_ix(hint + 1)
        } else {
            None
        };
        let prev_ix = if hint > 0 {
            self.prev_occupied_ix(hint - 1)
        } else {
            None
        };
        // Then correct the ix until it's sorted order
        let mut ideal_ix = hint;
        let settled_right = next_ix.is_none() || value < self.data[next_ix.unwrap()];
        let settled_left = prev_ix.is_none() || self.data[prev_ix.unwrap()] < value;
        if !settled_right {
            loop {
                ideal_ix += 1;
            }
        } else if !settled_left {
        }
    }
}
