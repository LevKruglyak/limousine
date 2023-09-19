use crate::iter::{Iter, MutIter};
use crate::kv::KeyBounded;
use crate::Entry;
use std::ops::{Bound, RangeBounds};
use std::path::Path;
use trait_set::trait_set;

trait_set! {
    pub trait Address = Eq + Clone + 'static;
}

// Type dependence hierarchy

pub trait LinkedNode<K, SA, PA>: 'static + KeyBounded<K>
where
    SA: Address,
    PA: Address,
{
    // Address to the next node in the current component
    fn next(&self) -> Option<SA>;

    // Address to the parent node in the above component
    fn parent(&self) -> Option<PA>;

    fn set_parent(&mut self, parent: PA);
}

/// A `NodeLayer` is a linked list of key-bounded nodes which implement the `Node<K>` trait
pub trait NodeLayer<K, SA, PA>: 'static + Sized
where
    SA: Address,
    PA: Address,
{
    /// Node type stored in the layer. Each node roughly represents a model in the hybrid index
    /// which indexes some finite/lower-bounded collection of `Keyed` elements.
    type Node: LinkedNode<K, SA, PA>;

    /// Immutable address dereference which returns a reference to a node.
    fn deref(&self, ptr: SA) -> &Self::Node;

    /// Mutable address dereference which returns a reference to a node.
    fn deref_mut(&mut self, ptr: SA) -> &mut Self::Node;

    unsafe fn deref_unsafe(&self, ptr: SA) -> *mut Self::Node;

    /// Get the lower bound of a node. This could be overriden by some layers which might have a
    /// more optimal way of mapping the address to the lower bound.
    fn lower_bound(&self, ptr: SA) -> &K {
        self.deref(ptr).lower_bound()
    }

    /// First node in the current layer
    fn first(&self) -> SA;

    fn range<'n>(&'n self, start: Bound<SA>, end: Bound<SA>) -> Iter<'n, K, Self, SA, PA> {
        Iter::range(&self, start, end)
    }

    unsafe fn mut_range<'n>(
        &'n mut self,
        start: Bound<SA>,
        end: Bound<SA>,
    ) -> MutIter<'n, K, Self, SA, PA> {
        MutIter::range(self, start, end)
    }
}

pub enum PropogateInsert<K, SA, PA> {
    /// Insert a single newly created node into the layer
    Single(K, SA, PA),

    /// Rebuild the entire layer
    Replace(PA, PA),
}

pub trait TopComponent<K, Base, BA, SA>
where
    Base: NodeLayer<K, BA, SA>,
    BA: Address,
    SA: Address,
{
    fn search(&self, base: &Base, key: &K) -> BA;

    fn insert(&mut self, base: &mut Base, prop: PropogateInsert<K, BA, SA>);
}

pub trait TopComponentInMemoryBuild<K, Base, BA, SA>
where
    Base: NodeLayer<K, BA, SA>,
    BA: Address,
    SA: Address,
{
    fn build(base: &mut Base) -> Self;
}

pub trait InternalComponent<K, Base, BA, SA, PA>
where
    Self: NodeLayer<K, SA, PA>,
    Base: NodeLayer<K, BA, SA>,
    BA: Address,
    SA: Address,
    PA: Address,
{
    fn search(&self, base: &Base, ptr: SA, key: &K) -> BA;

    fn insert<'n>(
        &'n mut self,
        base: &mut Base,
        prop: PropogateInsert<K, BA, SA>,
    ) -> Option<PropogateInsert<K, SA, PA>>;
}

pub trait InternalComponentInMemoryBuild<K, Base, BA, SA, PA>
where
    Base: NodeLayer<K, BA, SA>,
    BA: Address,
    SA: Address,
    PA: Address,
{
    fn build(base: &mut Base) -> Self;
}

pub trait InternalComponentDiskBuild<K, Base, BA, SA, PA>
where
    Base: NodeLayer<K, BA, SA>,
    BA: Address,
    SA: Address,
    PA: Address,
{
    fn build(base: &Base, path: impl AsRef<Path>) -> Self;
}

pub trait BaseComponent<K, V, SA, PA>
where
    Self: NodeLayer<K, SA, PA>,
    SA: Address,
    PA: Address,
{
    fn insert(&mut self, ptr: SA, key: K, value: V) -> Option<PropogateInsert<K, SA, PA>>;

    fn search(&self, ptr: SA, key: &K) -> Option<&V>;
}

pub trait BaseComponentInMemoryBuild<K, V> {
    fn empty() -> Self;

    fn build(iter: impl Iterator<Item = Entry<K, V>>) -> Self;
}
