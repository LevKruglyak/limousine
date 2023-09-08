use crate::common::address::Address;
use crate::kv::KeyBounded;
use std::ops::{Bound, RangeBounds};
use std::path::Path;

// Type dependence hierarchy

pub trait LinkedNode<K>: 'static + KeyBounded<K> {
    // Address to the next node in the current component
    fn next(&self) -> Option<Address>;

    // Address to the parent node in the above component
    fn parent(&self) -> Option<Address>;
}

/// A `NodeLayer` is a linked list of key-bounded nodes which implement the `Node<K>` trait
pub trait NodeLayer<K>: 'static + Sized {
    /// Node type stored in the layer. Each node roughly represents a model in the hybrid index
    /// which indexes some finite/lower-bounded collection of `Keyed` elements.
    type Node: LinkedNode<K>;

    /// Immutable address dereference which returns a reference to a node.
    fn deref(&self, ptr: Address) -> &Self::Node;

    /// Mutable address dereference which returns a reference to a node.
    fn deref_mut(&mut self, ptr: Address) -> &mut Self::Node;

    /// Get the lower bound of a node. This could be overriden by some layers which might have a
    /// more optimal way of mapping the address to the lower bound.
    fn lower_bound(&self, ptr: Address) -> &K {
        self.deref(ptr).lower_bound()
    }

    /// First node in a
    fn first(&self) -> Address;

    fn range<'n>(&'n self, start: Bound<Address>, end: Bound<Address>) -> Iter<'n, K, Self> {
        Iter::range(&self, start, end)
    }
}

pub enum PropogateInsert<K> {
    /// Insert a single newly created node into the layer
    Single(K, Address),

    /// Rebuild the entire layer
    Rebuild,
}

pub trait TopComponent<K, Base>
where
    Base: NodeLayer<K>,
{
    fn search(&self, base: &Base, key: &K) -> Address;

    fn insert(&mut self, base: &Base, prop: PropogateInsert<K>);
}

pub trait TopComponentInMemoryBuild<K, Base>
where
    Base: NodeLayer<K>,
{
    fn build(base: &Base) -> Self;
}

pub trait InternalComponent<K, Base>
where
    Self: NodeLayer<K>,
    Base: NodeLayer<K>,
{
    fn search(&self, base: &Base, ptr: Address, key: &K) -> Address;

    fn insert<'n>(
        &'n mut self,
        base: &Base,
        ptr: Address,
        prop: PropogateInsert<K>,
    ) -> Option<PropogateInsert<K>>;
}

pub trait InternalComponentInMemoryBuild<K, Base>
where
    Base: NodeLayer<K>,
{
    fn build(base: &Base) -> Self;
}

pub trait InternalComponentDiskBuild<K, Base>
where
    Base: NodeLayer<K>,
{
    fn build(base: &Base, path: impl AsRef<Path>) -> Self;
}

pub trait BaseComponent<K, V, Base>
where
    Self: NodeLayer<K>,
    Base: NodeLayer<K>,
{
    fn insert(&mut self, ptr: Address, key: K, value: V) -> Option<PropogateInsert<K>>;

    fn search(&self, ptr: Address, key: &K) -> Option<&V>;
}

pub trait BaseComponentInMemoryBuild<K, V> {
    fn empty() -> Self;

    fn build(iter: impl Iterator<Item = (K, V)>) -> Self;
}

// ----------------------------------------
// Iterator Type
// ----------------------------------------

pub struct Iter<'n, K, N> {
    layer: &'n N,
    current: Option<Address>,
    end: Bound<Address>,
    _ph: std::marker::PhantomData<K>,
}

impl<'n, K, N: NodeLayer<K>> Iter<'n, K, N> {
    fn range(layer: &'n N, start: Bound<Address>, end: Bound<Address>) -> Self {
        match start {
            Bound::Excluded(start) => Self {
                layer,
                current: layer.deref(start).next(),
                end,
                _ph: std::marker::PhantomData,
            },

            Bound::Included(start) => Self {
                layer,
                current: Some(start.clone()),
                end,
                _ph: std::marker::PhantomData,
            },

            Bound::Unbounded => Self {
                layer,
                current: Some(layer.first()),
                end,
                _ph: std::marker::PhantomData,
            },
        }
    }
}

impl<'n, K, N: NodeLayer<K>> Iterator for Iter<'n, K, N>
where
    K: Copy,
{
    type Item = (K, Address);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.clone()?;

        match self.end {
            Bound::Excluded(end) => {
                if current == end {
                    return None;
                }
            }

            Bound::Included(end) => {
                if current == end {
                    self.current = None;
                }
            }

            _ => (),
        }

        // Advance pointer
        if let Some(current) = self.current {
            self.current = self.layer.deref(current).next();
        }

        return Some(((*self.layer.lower_bound(current)), current));
    }
}
