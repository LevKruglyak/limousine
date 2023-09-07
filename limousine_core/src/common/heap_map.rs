/// `HeapMap` will be a constant-size, one-allocation associative container
/// backed by a boxed array
/// Why do we need it?
///     - Using buffers + merge granularity, when a model is retraine,
///       it will produce models with fixed sizes. However, these sizes
///       are unknowable at compile time, so we need a heap allocated
///       structure to deal with them
pub struct HeapMap<K, V> {
    pub len: usize,
    pub data: Box<[(K, V)]>,
}
