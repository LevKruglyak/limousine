use crate::iter::{Iter, MutIter};
use crate::kv::KeyBounded;
use std::ops::{Bound, RangeBounds};
use std::path::Path;
use trait_set::trait_set;

trait_set! {
    pub trait Address = Eq + Clone + 'static;
}

// Type dependence hierarchy

/// A `LinkedNode` is a model in a `NodeLayer`, representing a set of entries above a
/// lower bound. In addition to storing a pointer to its neighbor, it also stores a
/// pointer to its parent, which is in a different layer.
///
/// In order to avoid circular type dependencies during composition, it is generic over
/// its own address type, as well as its parent type. (SA, PA respectively)
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

/// A `NodeLayer` is a linked list of key-bounded nodes which implement the `LinkedNode<K>` trait.
pub trait NodeLayer<K, SA, PA>: 'static + Sized
where
    K: Copy,
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

    /// First node in the current node layer
    fn first(&self) -> SA;

    /// An immutable iterator over the layer, returning (Key, Address) pairs
    fn range<'n>(&'n self, start: Bound<SA>, end: Bound<SA>) -> Iter<'n, K, Self, SA, PA> {
        Iter::range(&self, start, end)
    }

    /// A mutable iterator over the layer, returning MutNodeView objects, which have methods to
    /// access the lower bound (Key) and address, as well as interior mutability to change the
    /// parent of the underlying node. This is useful during building, since a layer cannot know
    /// its own parents until the parents themselves are built.
    fn mut_range<'n>(
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

    /// Rebuild a region of the layer completely
    Replace(PA, PA),
}

pub trait TopComponent<K, Base, BA, SA>
where
    Base: NodeLayer<K, BA, SA>,
    BA: Address,
    SA: Address,
    K: Copy,
{
    fn search(&self, base: &Base, key: &K) -> BA;

    fn insert(&mut self, base: &mut Base, prop: PropogateInsert<K, BA, SA>);
}

pub trait TopComponentInMemoryBuild<K, Base, BA, SA>
where
    Base: NodeLayer<K, BA, SA>,
    BA: Address,
    SA: Address,
    K: Copy,
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
    K: Copy,
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
    K: Copy,
{
    fn build(base: &mut Base) -> Self;
}

pub trait InternalComponentDiskBuild<K, Base, BA, SA, PA>
where
    Base: NodeLayer<K, BA, SA>,
    BA: Address,
    SA: Address,
    PA: Address,
    K: Copy,
{
    fn build(base: &Base, path: impl AsRef<Path>) -> Self;
}

pub trait BaseComponent<K, V, SA, PA>
where
    Self: NodeLayer<K, SA, PA>,
    SA: Address,
    PA: Address,
    K: Copy,
{
    fn insert(&mut self, ptr: SA, key: K, value: V) -> Option<PropogateInsert<K, SA, PA>>;

    fn search(&self, ptr: SA, key: &K) -> Option<&V>;
}

pub trait BaseComponentInMemoryBuild<K, V> {
    fn empty() -> Self;

    fn build(iter: impl Iterator<Item = (K, V)>) -> Self;
}
