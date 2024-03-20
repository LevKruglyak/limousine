// Adapted from: [idalloc](https://github.com/udoprog/idalloc)

use serde::{Deserialize, Serialize};

/// A type that can be used an allocator index.
pub trait ID: Copy {
    /// Allocate the initial, unallocated value.
    fn initial() -> Self;

    /// Get the index as a usize.
    fn as_usize(self) -> usize;

    /// Increment the index and return the incremented value.
    fn increment(self) -> Self;
}

macro_rules! impl_primitive_index {
    ($ty:ident) => {
        impl ID for $ty {
            #[inline(always)]
            fn initial() -> Self {
                0
            }

            #[inline(always)]
            fn as_usize(self) -> usize {
                self as usize
            }

            #[inline(always)]
            fn increment(self) -> Self {
                self + 1
            }
        }
    };
}

// TODO: NonMax optimization so that Option<ID> has same size as ID
impl_primitive_index!(u8);
impl_primitive_index!(u16);
impl_primitive_index!(u32);
impl_primitive_index!(u64);
impl_primitive_index!(u128);

/// A slab-based id allocator which can deal with automatic reclamation
#[derive(Clone, Serialize, Deserialize)]
pub struct IDAllocator<I> {
    data: Vec<Option<I>>,
    next: I,
}

#[allow(unused)]
impl<I> IDAllocator<I>
where
    I: ID,
{
    /// Construct a new slab allocator.
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            next: I::initial(),
        }
    }

    /// Allocate the next id.
    pub fn allocate(&mut self) -> I {
        let index = self.next;

        self.next = if let Some(entry) = self.data.get_mut(self.next.as_usize()) {
            entry.take().expect("IDAllocator found null index!")
        } else {
            self.data.push(None);
            self.next.increment()
        };

        index
    }

    /// Free the specified id.
    pub fn free(&mut self, index: I) -> bool {
        if let Some(entry) = self.data.get_mut(index.as_usize()) {
            if entry.is_none() {
                *entry = Some(self.next);
                self.next = index;
                return true;
            }
        }

        false
    }

    pub fn is_allocated(&self, index: I) -> bool {
        if let Some(entry) = self.data.get(index.as_usize()) {
            return entry.is_none();
        }

        false
    }
}

impl<I> Default for IDAllocator<I>
where
    I: ID,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_initial() {
        assert_eq!(u8::initial(), 0);
        assert_eq!(u32::initial(), 0);
        assert_eq!(u64::initial(), 0);
    }

    #[test]
    fn id_increment() {
        let id: u32 = 5;
        assert_eq!(id.increment(), 6);
    }

    // #[test]
    // #[should_panic(expected = "index `255` is out of bounds: 0-255")]
    // fn id_increment_panic() {
    //     let id: u8 = u8::none(); // this should be u8::MAX
    //     id.increment(); // this should panic
    // }

    #[test]
    fn allocate_ids() {
        let mut allocator = IDAllocator::<u8>::new();
        let id1 = allocator.allocate();
        let id2 = allocator.allocate();
        assert_eq!(id1, u8::initial()); // should be 0
        assert_eq!(id2, 1); // should be 1
    }

    #[test]
    fn free_and_reuse_id() {
        let mut allocator = IDAllocator::<u8>::new();
        let id1 = allocator.allocate();
        assert!(allocator.free(id1)); // should succeed
        let id2 = allocator.allocate(); // should reuse freed id
        assert_eq!(id1, id2);
    }

    #[test]
    fn free_invalid_id() {
        let mut allocator = IDAllocator::<u8>::new();
        assert!(!allocator.free(100)); // should fail, id was never allocated
    }

    #[test]
    fn is_allocated() {
        let mut allocator = IDAllocator::<u8>::new();
        let id = allocator.allocate();
        assert!(allocator.is_allocated(id)); // should be true
        assert!(!allocator.is_allocated(100)); // should be false, id was never allocated
    }

    #[test]
    fn multiple_allocations() {
        let mut allocator = IDAllocator::<u32>::new();
        let mut ids = Vec::new();

        // Allocate multiple IDs and ensure they are distinct
        for _ in 0..10 {
            let id = allocator.allocate();
            assert!(!ids.contains(&id)); // Ensure no duplicate IDs
            ids.push(id);
        }
    }

    #[test]
    fn allocation_after_free() {
        let mut allocator = IDAllocator::<u16>::new();
        let mut first_batch = (0..5).map(|_| allocator.allocate()).collect::<Vec<_>>();

        // Free some IDs
        for &id in &first_batch[1..3] {
            assert!(allocator.free(id));
        }

        // Allocate new IDs and check if freed IDs are reused
        let mut second_batch = (0..2).map(|_| allocator.allocate()).collect::<Vec<_>>();
        assert_eq!(first_batch[1..3].sort(), second_batch[..].sort());
    }

    #[test]
    fn freeing_unallocated_id() {
        let mut allocator = IDAllocator::<u8>::new();
        assert!(
            !allocator.free(10),
            "Freeing an unallocated ID should return false"
        );
    }

    #[test]
    fn allocation_fills_gaps_first() {
        let mut allocator = IDAllocator::<u32>::new();
        let id1 = allocator.allocate();
        let id2 = allocator.allocate();
        let id3 = allocator.allocate();

        // Free the second id to create a gap
        allocator.free(id2);

        // Next allocation should fill the gap
        let id4 = allocator.allocate();
        assert_eq!(
            id2, id4,
            "The allocator should reuse freed IDs before allocating new ones"
        );

        // Further allocation should not reuse other IDs
        let id5 = allocator.allocate();
        assert_ne!(id1, id5);
        assert_ne!(id3, id5);
    }

    #[test]
    fn continuous_allocation() {
        let mut allocator = IDAllocator::<u64>::new();

        // Continuously allocate and free
        for i in 0..100 {
            let id = allocator.allocate();
            assert_eq!(
                id.as_usize(),
                i as usize,
                "The allocator should provide continuous IDs when none are freed"
            );
        }
    }

    #[test]
    fn is_allocated_for_unallocated() {
        let allocator = IDAllocator::<u32>::new();
        assert!(
            !allocator.is_allocated(42),
            "Unallocated ID should return false for is_allocated"
        );
    }

    #[test]
    fn allocator_reuse_after_free_all() {
        let mut allocator = IDAllocator::<u16>::new();
        let ids = (0..100).map(|_| allocator.allocate()).collect::<Vec<_>>();

        for id in ids {
            assert!(allocator.free(id));
        }

        for i in (0..100).rev() {
            let new_id = allocator.allocate();
            assert_eq!(
                new_id, i,
                "Allocator should start from the beginning after all IDs are freed"
            );
        }
    }

    // #[test]
    // #[should_panic(expected = "IDAllocator found null index!")]
    // fn allocator_panic_on_corrupt_state() {
    //     let mut allocator = IDAllocator::<u32>::new();
    //     allocator.data.push(u32::none());
    //     allocator.allocate();
    // }
}
