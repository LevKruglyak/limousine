//! Helper struct to implement a gapped array to act as the base of data nodes
//! ASSUMPTIONS:
//! - Keys (T) are unique

use num::iter::Range;

#[derive(Debug)]
pub struct GappedKVArray<K, V>
where
    K: Default + Copy + Clone + PartialEq + PartialOrd + std::fmt::Debug,
    V: Default + Copy + Clone + PartialEq + PartialOrd + std::fmt::Debug,
{
    pub bitmap: Box<[bool]>,
    pub keys: Box<[K]>,
    pub vals: Box<[V]>,
}

impl<K, V> GappedKVArray<K, V>
where
    K: Default + Copy + Clone + PartialEq + PartialOrd + std::fmt::Debug,
    V: Default + Copy + Clone + PartialEq + PartialOrd + std::fmt::Debug,
{
    /// Creates an empty gapped array with the given size
    pub fn new(size: usize) -> Self {
        let bitmap_vec = vec![false; size];
        let keys_vec = vec![K::default(); size];
        let vals_vec = vec![V::default(); size];
        Self {
            bitmap: bitmap_vec.into_boxed_slice(),
            keys: keys_vec.into_boxed_slice(),
            vals: vals_vec.into_boxed_slice(),
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
            }
            ix -= 1;
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

    /// Returns the Some(ix) s.t. keys[ix] <= needle, and for all jx > ix, needle < keys[jx]
    /// Returns None if needle is smaller than everything in the array
    /// NOTE: Hint is just to help search speed. This must always return a correct result.
    fn price_is_right(&self, needle: &K, hint: Option<usize>) -> Option<usize> {
        // First, move as far to the right as we can from the hint
        let mut check = self.next_occupied_ix(hint.unwrap_or(self.len() / 2));
        while check.is_some() {
            let next = self.next_occupied_ix(check.unwrap() + 1);
            match next {
                Some(next_ix) => {
                    if *needle < self.keys[next_ix] {
                        break;
                    }
                    check = Some(next_ix);
                }
                None => break,
            }
        }
        // Make sure check is something if there is an element
        check = match check {
            Some(ix) => Some(ix),
            None => self.prev_occupied_ix(self.len() - 1),
        };
        // Then ensure correctness by moving left as far as we need to
        while check.is_some() {
            if self.keys[check.unwrap()] <= *needle {
                // Happy case where we got it right
                break;
            }
            if check.unwrap() == 0 {
                check = None;
                break;
            }
            check = self.prev_occupied_ix(check.unwrap() - 1);
        }
        check
    }

    /// Search the gapped array for a specific value, returning the price is right
    /// TODO: Make exponential search
    pub fn search_pir(&self, needle: &K, hint: Option<usize>) -> Option<&V> {
        match self.price_is_right(needle, hint) {
            Some(ix) => self.vals.get(ix),
            None => None,
        }
    }

    /// Search the gapped array for a specific value, using a starting hint
    /// TODO: Make exponential search
    pub fn search_exact(&self, needle: &K, hint: Option<usize>) -> Option<&V> {
        match self.price_is_right(needle, hint) {
            Some(ix) => {
                if self.keys[ix] == *needle {
                    self.vals.get(ix)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Helper function to copy within for all the needed arrays
    fn copy_within(&mut self, src: std::ops::Range<usize>, dest: usize) {
        self.bitmap.copy_within(src.clone(), dest);
        self.keys.copy_within(src.clone(), dest);
        self.vals.copy_within(src.clone(), dest);
    }

    /// Helper function to insert an entry into a given location
    fn insert_at(&mut self, pair: (K, V), ix: usize) {
        self.bitmap[ix] = true;
        self.keys[ix] = pair.0;
        self.vals[ix] = pair.1;
    }

    /// Insert a specific value into the array with the given hint
    pub fn insert_with_hint(&mut self, pair: (K, V), hint: usize) -> Result<(), String> {
        // TODO: We should do a better job covering the happy case where the guessed index is empty
        // ^ehhhh but also it's just a guess so we need to verify sorted... gets a bit hairy
        let maybe_ix = self.price_is_right(&pair.0, Some(hint));
        match maybe_ix {
            None => {
                // Edge case where inserting at the beginning
                let Some(closest_ix) = self.next_free_ix(0) else {
                    return Err("Gapped array is full".to_string());
                };
                self.copy_within(0..closest_ix, 1);
                self.insert_at(pair, 0);
                Ok(())
            }
            Some(mut ix) => {
                if ix + 1 == self.len() {
                    // Edge case where inserting at the end
                    let Some(closest_ix) = self.prev_free_ix(self.len() - 1) else {
                        return Err("Gapped array is full".to_string());
                    };
                    self.copy_within(closest_ix + 1..self.len(), closest_ix);
                    self.insert_at(pair, self.len() - 1);
                    Ok(())
                } else {
                    // We're doing a "normal" insert into the middle of the array
                    ix += 1; // Price-is-right quirk
                    if !self.bitmap[ix] {
                        // Easy win
                        self.insert_at(pair, ix);
                        return Ok(());
                    }
                    let shift_left_ix = self.prev_free_ix(ix - 1);
                    let shift_right_ix = self.next_free_ix(ix + 1);
                    match (shift_left_ix, shift_right_ix) {
                        (Some(lix), Some(rix)) => {
                            if lix.abs_diff(ix) < rix.abs_diff(ix) {
                                self.copy_within(lix + 1..ix + 1, lix);
                                self.insert_at(pair, ix - 1);
                                Ok(())
                            } else {
                                self.copy_within(ix..rix, ix + 1);
                                self.insert_at(pair, ix);
                                Ok(())
                            }
                        }
                        (Some(lix), None) => {
                            self.copy_within(lix + 1..ix + 1, lix);
                            self.insert_at(pair, ix - 1);
                            Ok(())
                        }
                        (None, Some(rix)) => {
                            self.copy_within(ix..rix, ix + 1);
                            self.insert_at(pair, ix);
                            Ok(())
                        }
                        _ => Err("Gapped array is full".to_string()),
                    }
                }
            }
        }
    }

    /// Called for the initial inserts. NOTE: This makes two assumptions:
    /// - The values themselves are monotonically increasing
    /// - The hints are monotonically non-decreasing
    /// If either of these assumptions break, bad stuff may happen (use regular insert)
    pub fn initial_model_based_insert(&mut self, pair: (K, V), hint: usize) -> Result<(), String> {
        if !self.bitmap[hint] {
            self.insert_at(pair, hint);
            return Ok(());
        }
        match self.next_free_ix(hint + 1) {
            Some(free_ix) => {
                self.insert_at(pair, free_ix);
                Ok(())
            }
            None => match self.prev_free_ix(self.len().saturating_sub(1)) {
                Some(free_ix) => {
                    self.copy_within(free_ix + 1..self.len(), free_ix);
                    self.insert_at(pair, self.len().saturating_sub(1));
                    Ok(())
                }
                None => Err("Gapped array is full".to_string()),
            },
        }
    }
}

#[cfg(test)]
mod gapped_array_tests {
    use std::ops::Range;

    use super::*;
    use itertools::Itertools;
    use kdam::{tqdm, Bar, BarExt};

    fn print_gapped_array(ga: &GappedKVArray<i32, i32>) {
        let mut line1 = String::new();
        let mut line2 = String::new();
        let mut line3 = String::new();
        for ix in 0..ga.len() {
            line1 += &format!("{}", if ga.bitmap[ix] { 1 } else { 0 });
            line2 += &format!("{}", ga.keys[ix]);
            line3 += &format!("{}", ga.vals[ix]);
        }
        println!("bitmap: {}", &line1);
        println!("keys: {}", &line2);
        println!("vals: {}", &line3);
    }

    fn fill_forward_with_hint(size: usize, hint: usize) {
        let mut ga = GappedKVArray::<i32, i32>::new(size);
        for num in 0..size {
            let result = ga.insert_with_hint((num as i32, num as i32), hint);
            assert!(result.is_ok());
            print_gapped_array(&ga);
        }
        for ix in 0..size {
            assert!(ga.bitmap[ix]);
            assert!(ga.keys[ix] == ix as i32);
        }
    }

    #[test]
    fn fill_forward() {
        const SIZE: usize = 10;
        for hint in 0..SIZE {
            fill_forward_with_hint(SIZE, hint);
        }
    }

    fn fill_backward_with_hint(size: usize, hint: usize) {
        let mut ga = GappedKVArray::<i32, i32>::new(size);
        for num in 0..size {
            let result = ga.insert_with_hint(((size - num - 1) as i32, (size - num - 1) as i32), hint);
            assert!(result.is_ok());
        }
        for ix in 0..size {
            assert!(ga.bitmap[ix]);
            assert!(ga.keys[ix] == ix as i32);
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
        let mut ga = GappedKVArray::<i32, i32>::new(perm.len());
        for (value, hint) in perm.iter().zip(hints.iter()) {
            assert!(ga
                .insert_with_hint((value.clone(), value.clone()), hint.clone())
                .is_ok());
        }
        for ix in 0..ga.len() {
            let good = ga.bitmap[ix] && ga.keys[ix] == ix as i32;
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
        let mut ga = GappedKVArray::<i32, i32>::new(perm.len());
        print_gapped_array(&ga);
        for (value, hint) in perm.iter().zip(hints.iter()) {
            assert!(ga
                .insert_with_hint((value.clone(), value.clone()), hint.clone())
                .is_ok());
            println!("");
            print_gapped_array(&ga);
        }
    }

    fn test_nondec_seq(items: &Vec<i32>, hints: &Vec<usize>) {
        let mut ga = GappedKVArray::<i32, i32>::new(items.len());
        for (value, hint) in items.iter().zip(hints.iter()) {
            assert!(ga
                .initial_model_based_insert((value.clone(), value.clone()), hint.clone())
                .is_ok());
        }
        for ix in 0..ga.len() {
            let good = ga.bitmap[ix] && ga.keys[ix] == ix as i32;
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
        let mut ga = GappedKVArray::<i32, i32>::new(perm.len());
        print_gapped_array(&ga);
        for (value, hint) in perm.iter().zip(hints.iter()) {
            assert!(ga
                .initial_model_based_insert((value.clone(), value.clone()), hint.clone())
                .is_ok());
            println!("");
            print_gapped_array(&ga);
        }
    }
}
