use std::{
    collections::HashMap,
    mem,
    ptr::{self, NonNull},
};

struct CacheNode<K, V> {
    key: mem::MaybeUninit<K>,
    value: mem::MaybeUninit<V>,
    prev: *mut CacheNode<K, V>,
    next: *mut CacheNode<K, V>,
}

/// A dynamically sized LRU cache with reference stability
pub struct StableLRUCache<K, V> {
    map: HashMap<K, NonNull<CacheNode<K, V>>>,

    head: *mut CacheNode<K, V>,
    tail: *mut CacheNode<K, V>,
}

impl<K, V> StableLRUCache<K, V> {
    pub fn new() -> Self {
        let cache = Self {
            map: HashMap::new(),
            head: Box::into_raw(Box::new(CacheNode::new_sigil())),
            tail: Box::into_raw(Box::new(CacheNode::new_sigil())),
        };

        unsafe {
            (*cache.head).next = cache.tail;
            (*cache.tail).prev = cache.head;
        }

        cache
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        unimplemented!()
    }

    pub fn get_or_put_fallible(
        &self,
        key: &K,
        put: impl Fn() -> crate::Result<Option<V>>,
    ) -> crate::Result<Option<&V>> {
        unimplemented!()
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        unimplemented!()
    }
}

impl<K, V> CacheNode<K, V> {
    pub(crate) fn new(key: K, value: V) -> Self {
        CacheNode {
            key: mem::MaybeUninit::new(key),
            value: mem::MaybeUninit::new(value),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }

    fn new_sigil() -> Self {
        CacheNode {
            key: mem::MaybeUninit::uninit(),
            value: mem::MaybeUninit::uninit(),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }
}
