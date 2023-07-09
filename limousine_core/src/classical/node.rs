use bytemuck::{Pod, Zeroable};
use num::Bounded;
use std::{borrow::Borrow, mem::MaybeUninit};

use crate::{
    entry::Entry,
    search::{lower_bound, OptimalSearch, Search},
    Key,
};

#[derive(Copy)]
#[repr(C)]
pub struct BTreeNode<K, V, const FANOUT: usize> {
    key_ptrs: [MaybeUninit<Entry<K, V>>; FANOUT],
    min: K,
    len: usize,
}

// SAFETY: this is safe by the `Zeroable` rules, but we want to avoid
// dependencies on unstable features
unsafe impl<K, V, const FANOUT: usize> Zeroable for BTreeNode<K, V, FANOUT> {}

// SAFETY: this violates the padding rule of `Pod`, so transmuting this
// into any other `Pod` type would lead to a UB violation: specifically
// treating uninitialized data as initialized data. We only need this type
// to be `Pod` to persist to a file, and it is used internally so this isn't
// a big issue.
unsafe impl<K: Copy + 'static, V: Copy + 'static, const FANOUT: usize> Pod
    for BTreeNode<K, V, FANOUT>
{
}

impl<K: Bounded, V, const FANOUT: usize> Borrow<K> for BTreeNode<K, V, FANOUT> {
    fn borrow(&self) -> &K {
        &self.min
    }
}

impl<K, V, const FANOUT: usize> Clone for BTreeNode<K, V, FANOUT>
where
    K: Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        let mut key_ptrs =
            unsafe { MaybeUninit::<[MaybeUninit<Entry<K, V>>; FANOUT]>::uninit().assume_init() };

        for i in 0..self.len {
            let key_ptr = unsafe { self.key_ptrs[i].assume_init_ref() };
            key_ptrs[i] = MaybeUninit::new(key_ptr.clone());
        }

        Self {
            key_ptrs,
            min: self.min.clone(),
            len: self.len,
        }
    }
}

impl<K: Bounded + Copy, V, const FANOUT: usize> BTreeNode<K, V, FANOUT> {
    /// Create an empty `BTreeNode`
    pub fn empty() -> Self {
        Self {
            key_ptrs: unsafe {
                MaybeUninit::<[MaybeUninit<Entry<K, V>>; FANOUT]>::uninit().assume_init()
            },
            min: K::min_value(),
            len: 0,
        }
    }

    pub fn push(&mut self, key_ptr: Entry<K, V>) {
        if self.len == 0 {
            self.min = key_ptr.key;
        }

        debug_assert!(self.len < FANOUT, "Tried to push into a full BTreeNode.");

        self.key_ptrs[self.len] = MaybeUninit::new(key_ptr);
        self.len += 1;
    }

    /// Borrow a slice view into the entries stored in the `MergePage`
    pub fn entries(&self) -> &[Entry<K, V>] {
        // SAFETY: `len` must be strictly less than `F`
        debug_assert!(self.len <= FANOUT);
        let slice = unsafe { self.key_ptrs.get_unchecked(..usize::from(self.len)) };

        // SAFETY: feature `maybe_uninit_slice`
        unsafe { &*(slice as *const [MaybeUninit<Entry<K, V>>] as *const [Entry<K, V>]) }
    }
}

impl<K, V: Default, const FANOUT: usize> BTreeNode<K, V, FANOUT>
where
    K: Key,
    V: Clone,
{
    /// Returns the `ptr` corresponding to the desired key
    pub fn search(&self, key: &K) -> V {
        if self.entries().len() == 0 {
            return V::default();
        }

        self.entries()[lower_bound(OptimalSearch::search_by_key(self.entries(), key))]
            .value
            .clone()
    }
}
