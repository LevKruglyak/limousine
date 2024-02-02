//! Helper struct to implement a gapped array to act as the base of data nodes
//! ASSUMPTIONS:
//! - Elements (T) are unique

pub struct GappedArray<T>
where
    T: Default + Copy + Clone + PartialEq + PartialOrd,
{
    pub bitmap: Box<[bool]>,
    pub data: Box<[T]>,
}

impl<T: Default + Copy + Clone + PartialEq + PartialOrd> GappedArray<T> {
    /// Creates an empty gapped array with the given size
    pub fn new(size: usize) -> Self {
        let bitmap_vec = vec![false; size];
        let data_vec = vec![T::default(); size];
        Self {
            bitmap: bitmap_vec.into_boxed_slice(),
            data: data_vec.into_boxed_slice(),
        }
    }

    /// The length of the gapped array (including gaps)
    pub const fn len(&self) -> usize {
        self.bitmap.len()
    }

    /// Helper function to implement next occupied and next free
    /// TODO: Bithacks to make faster
    fn next_ix_helper(&self, mut ix: usize, val: bool) -> Option<usize> {
        while ix < self.len() && self.bitmap[ix] != val {
            ix += 1
        }
        if ix < self.len() {
            Some(ix)
        } else {
            None
        }
    }

    /// Returns the next occupied slot in the range [ix, end]
    pub fn next_occupied_ix(&self, mut ix: usize) -> Option<usize> {
        self.next_ix_helper(ix, true)
    }

    /// Returns the next free slot in the range [ix, end]
    pub fn next_free_ix(&self, mut ix: usize) -> Option<usize> {
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

    /// Returns the smallest ix s.t. the element at this ix and all elements to the right are strictly greater than needle
    /// NOTE: If needle is larger than everything in the array, this returns len()
    /// NOTE: Hint is just to help search speed. This must always return a correct result.
    fn lub(&self, needle: &T, hint: Option<usize>) -> usize {
        // First, move as far to the right as we can from hint
        let mut check = self.next_occupied_ix(hint.unwrap_or(self.len() / 2));
        while check.is_some() && self.data[check.unwrap()] < *needle {
            check = self.next_occupied_ix(check.unwrap() + 1);
        }
        // Then handle the edge case where this is the largest element
        if check.is_none() {
            let max = self.prev_occupied_ix(self.len() - 1);
            match max {
                Some(ix) => {
                    if self.data[ix] < *needle {
                        return self.len();
                    }
                }
                None => {
                    return self.len() / 2;
                }
            }
        }
        // Finally, to ensure correctness, move to the left if needed
        let before_ix = check.unwrap_or(self.len() - 1);
        if before_ix == 0 {
            return 0;
        }
        let mut before = self.prev_free_ix(before_ix - 1);
        let mut before = self.prev_occupied_ix(check.unwrap_or(self.len() - 1));
        while before.is_some() && *needle < self.data[before.unwrap()] {
            check = before;
            if before.unwrap() == 0 {
                return 0;
            }
            before = self.prev_occupied_ix(before.unwrap() - 1);
        }
        match check {
            Some(ix) => ix,
            None => self.len() / 2,
        }
    }

    /// Search the gapped array for a specific value, using a starting hint
    /// TODO: Make exponential search
    pub fn search_with_hint(&mut self, needle: &T, hint: usize) -> Option<&T> {
        let ix = self.lub(needle, Some(hint));
        if self.bitmap[ix] && self.data[ix] == *needle {
            return Some(&self.data[ix]);
        } else {
            return None;
        }
    }

    /// Insert a specific value into the array with the given hint
    pub fn insert_with_hint(&mut self, value: T, hint: usize) -> Result<(), String> {
        let ix = self.lub(&value, Some(hint));
        // Handle edge case where inserting at beginning
        if ix == 0 {
            let closest_ix = self.next_free_ix(0);
            match closest_ix {
                Some(rix) => {
                    self.bitmap.copy_within(0..rix, 1);
                    self.data.copy_within(0..rix, 1);
                    self.bitmap[0] = true;
                    self.data[0] = value;
                    return Ok(());
                }
                None => {
                    return Err("Gapped array is full".to_string());
                }
            }
        }
        // Handle edge case where inserting at end
        if ix >= self.len() {
            let closest_ix = self.prev_free_ix(self.len() - 1);
            match closest_ix {
                Some(lix) => {
                    self.bitmap.copy_within(lix + 1..self.len(), lix);
                    self.data.copy_within(lix + 1..self.len(), lix);
                    self.bitmap[self.len() - 1] = true;
                    self.data[self.len() - 1] = value;
                    return Ok(());
                }
                None => {
                    return Err("Gapped array is full".to_string());
                }
            }
        }
        // Inserting into the middle of the array
        let shift_left_ix = if ix == 0 { None } else { self.prev_free_ix(ix - 1) };
        let shift_right_ix = self.next_free_ix(ix + 1);
        match (shift_left_ix, shift_right_ix) {
            (Some(lix), Some(rix)) => {
                if lix.abs_diff(ix) < rix.abs_diff(ix) {
                    self.bitmap.copy_within(lix + 1..ix + 1, lix);
                    self.data.copy_within(lix + 1..ix + 1, lix);
                    self.bitmap[ix - 1] = true;
                    self.data[ix - 1] = value;
                } else {
                    self.bitmap.copy_within(ix..rix, ix + 1);
                    self.data.copy_within(ix..rix, ix + 1);
                    self.bitmap[ix] = true;
                    self.data[ix] = value;
                }
            }
            (Some(lix), None) => {
                self.bitmap.copy_within(lix + 1..ix, lix);
                self.data.copy_within(lix + 1..ix, lix);
                self.bitmap[ix - 1] = true;
                self.data[ix - 1] = value;
            }
            (None, Some(rix)) => {
                self.bitmap.copy_within(ix..rix, ix + 1);
                self.data.copy_within(ix..rix, ix + 1);
                self.bitmap[ix] = true;
                self.data[ix] = value;
            }
            (None, None) => return Err("Gapped array is full".to_string()),
        }
        return Ok(());
    }

    /// Called for the initial inserts. NOTE: This makes two assumptions:
    /// - The values themselves are monotonically increasing
    /// - The hints are monotonically non-decreasing
    /// If either of these assumptions break, bad stuff may happen (use regular insert)
    pub fn initial_model_based_insert(&mut self, value: T, hint: usize) -> Result<(), ()> {
        if !self.bitmap[hint] {
            self.bitmap[hint] = true;
            self.data[hint] = value;
            return Ok(());
        }
        match self.next_free_ix(hint + 1) {
            Some(free_ix) => {
                self.bitmap[free_ix] = true;
                self.data[free_ix] = value;
                return Ok(());
            }
            None => match self.prev_free_ix(self.len().saturating_sub(1)) {
                Some(free_ix) => {
                    self.bitmap.copy_within(free_ix + 1..self.len(), free_ix);
                    self.data.copy_within(free_ix + 1..self.len(), free_ix);
                    self.bitmap[self.len().saturating_sub(1)] = true;
                    self.data[self.len().saturating_sub(1)] = value;
                    return Ok(());
                }
                None => {
                    return Err(());
                }
            },
        }
    }
}

// pub struct ModelBasedGappedArrayBuilder<T>
// where
//     T: Default + Copy + Clone + PartialEq + PartialOrd,
// {
//     pub ga: GappedArray<T>,

//     pub bitmap: Box<[bool]>,
//     pub data: Box<[T]>,
// }

#[cfg(test)]
mod gapped_array_tests {
    use std::ops::Range;

    use super::*;
    use itertools::Itertools;
    use kdam::{tqdm, Bar, BarExt};

    fn print_gapped_array(ga: &GappedArray<i32>) {
        let mut line1 = String::new();
        let mut line2 = String::new();
        for ix in 0..ga.len() {
            line1 += &format!("{}", if ga.bitmap[ix] { 1 } else { 0 });
            line2 += &format!("{}", ga.data[ix]);
        }
        println!("{}", &line1);
        println!("{}", &line2);
    }

    fn fill_forward_with_hint(size: usize, hint: usize) {
        let mut ga = GappedArray::<i32>::new(size);
        for num in 0..size {
            let result = ga.insert_with_hint(num as i32, hint);
            assert!(result.is_ok());
        }
        for ix in 0..size {
            assert!(ga.bitmap[ix]);
            assert!(ga.data[ix] == ix as i32);
        }
    }

    #[test]
    fn fill_forward() {
        const SIZE: usize = 100;
        for hint in 0..SIZE {
            fill_forward_with_hint(SIZE, hint);
        }
    }

    fn fill_backward_with_hint(size: usize, hint: usize) {
        let mut ga = GappedArray::<i32>::new(size);
        for num in 0..size {
            let result = ga.insert_with_hint((size - num - 1) as i32, hint);
            assert!(result.is_ok());
        }
        for ix in 0..size {
            assert!(ga.bitmap[ix]);
            assert!(ga.data[ix] == ix as i32);
        }
    }

    #[test]
    fn fill_backward() {
        const SIZE: usize = 100;
        for hint in 0..SIZE {
            fill_backward_with_hint(SIZE, hint);
        }
    }

    fn get_all_possible_hints(size: usize, num_hints: usize) -> Vec<Vec<usize>> {
        if num_hints == 0 {
            return vec![];
        }
        if num_hints == 1 {
            return (0..size).into_iter().map(|val| vec![val]).collect();
        }
        let mut result: Vec<Vec<usize>> = vec![];
        for first_val in 0..size {
            let tails = get_all_possible_hints(size, num_hints - 1);
            for tail in tails {
                let mut new_thing = vec![first_val];
                new_thing.extend(tail.into_iter());
                result.push(new_thing);
            }
        }
        result
    }

    fn test_perm_with_hints(perm: &Vec<i32>, hints: &Vec<usize>) {
        let mut ga = GappedArray::<i32>::new(perm.len());
        for (value, hint) in perm.iter().zip(hints.iter()) {
            assert!(ga.insert_with_hint(value.clone(), hint.clone()).is_ok());
        }
        for ix in 0..ga.len() {
            let good = ga.bitmap[ix] && ga.data[ix] == ix as i32;
            if !good {
                println!("Perm: {:?}", perm);
                println!("Hints: {:?}", hints);
                print_gapped_array(&ga);
            }
            assert!(good);
        }
    }

    #[test]
    fn permutation_test() {
        const SIZE: usize = 6;
        let items: Vec<i32> = (0..SIZE).into_iter().map(|val| val as i32).collect();
        let perms: Vec<Vec<i32>> = items.into_iter().permutations(SIZE).collect();
        let hints = get_all_possible_hints(SIZE, SIZE);
        let mut pb = tqdm!(total = perms.len() * hints.len());
        for perm in perms.iter() {
            for hints in hints.iter() {
                test_perm_with_hints(perm, hints);
                pb.update(1);
            }
        }
    }

    #[test]
    fn debug_gapped() {
        let perm = vec![1, 2, 0, 3, 4];
        let hints = vec![0, 0, 3, 0, 0];
        let mut ga = GappedArray::<i32>::new(perm.len());
        print_gapped_array(&ga);
        for (value, hint) in perm.iter().zip(hints.iter()) {
            assert!(ga.insert_with_hint(value.clone(), hint.clone()).is_ok());
            println!("");
            print_gapped_array(&ga);
        }
    }

    fn test_nondec_seq(items: &Vec<i32>, hints: &Vec<usize>) {
        let mut ga = GappedArray::<i32>::new(items.len());
        for (value, hint) in items.iter().zip(hints.iter()) {
            assert!(ga.initial_model_based_insert(value.clone(), hint.clone()).is_ok());
        }
        for ix in 0..ga.len() {
            let good = ga.bitmap[ix] && ga.data[ix] == ix as i32;
            if !good {
                println!("Items: {:?}", items);
                println!("Hints: {:?}", hints);
                print_gapped_array(&ga);
            }
            assert!(good);
        }
    }

    #[test]
    fn initial_inserts() {
        const SIZE: usize = 6;
        let items: Vec<i32> = (0..SIZE).into_iter().map(|val| val as i32).collect();
        let mut sequences = get_all_possible_hints(SIZE, SIZE);
        sequences.retain(|seq| {
            let mut last: Option<usize> = None;
            for thing in seq.iter() {
                if last.is_some() && *thing < last.unwrap() {
                    return false;
                }
                last = Some(*thing);
            }
            return true;
        });
        for seq in sequences {
            test_nondec_seq(&items, &seq);
        }
    }

    #[test]
    fn debug_initial_gapped() {
        let perm = vec![0, 1, 2, 3, 4, 5];
        let hints = vec![0, 0, 0, 4, 4, 4];
        let mut ga = GappedArray::<i32>::new(perm.len());
        print_gapped_array(&ga);
        for (value, hint) in perm.iter().zip(hints.iter()) {
            assert!(ga.initial_model_based_insert(value.clone(), hint.clone()).is_ok());
            println!("");
            print_gapped_array(&ga);
        }
    }
}
