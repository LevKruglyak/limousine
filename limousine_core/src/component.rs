use crate::kv::KeyBounded;
use std::ops::RangeBounds;

/// A `NodeLayer` is an ordered collection of key-bounded nodes which implement the `Keyed` trait.
/// TODO: write more
pub trait NodeLayer<K>: 'static {
    /// Node type stored in the layer. Each node roughly represents a model in the hybrid index
    /// which indexes some finite/lower-bounded collection of `Keyed` elements.
    type Node: KeyBounded<K>;

    /// A valid address/reference/pointer to a `Node` within the layer.
    type Address: Clone + Eq;

    /// Immutable address dereference which returns a reference to a node.
    fn deref(&self, ptr: Self::Address) -> &Self::Node;

    /// Mutable address dereference which returns a reference to a node.
    fn deref_mut(&mut self, ptr: Self::Address) -> &mut Self::Node;

    /// Get the lower bound of a node. This could be overriden by some layers which might have a
    /// more optimal way of mapping the address to the lower bound.
    fn lower_bound(&self, ptr: Self::Address) -> &K {
        self.deref(ptr).lower_bound()
    }

    type Iter<'n>: Iterator<Item = (K, Self::Address)>
    where
        Self: 'n;

    /// Ordered iterator over all of the nodes in the layer. Functionally equivalent to calling
    /// ```self.range(None, None)```
    fn full_range<'n>(&'n self) -> Self::Iter<'n>;

    /// Ordered iterator over a (un)bounded slice of `Address`
    fn range<'n>(&'n self, range: impl RangeBounds<Self::Address>) -> Self::Iter<'n>;
}

pub enum PropogateInsert<'n, K, Base>
where
    Base: NodeLayer<K> + ?Sized,
{
    /// Insert a single newly created node into the layer
    Single(K, Base::Address),

    /// Rebuild the entire layer
    Rebuild(&'n Base),
}

pub trait TopComponent<K, Base>
where
    Base: NodeLayer<K>,
{
    fn search(&self, key: &K) -> Base::Address;

    fn insert(&mut self, prop: PropogateInsert<'_, K, Base>);

    fn size(&self) -> usize;
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
    fn search(&self, ptr: Self::Address, key: &K) -> Base::Address;

    fn insert<'n>(
        &'n mut self,
        ptr: Self::Address,
        prop: PropogateInsert<'_, K, Base>,
    ) -> Option<PropogateInsert<'n, K, Self>>;

    fn size(&self) -> usize;
}

pub trait InternalComponentInMemoryBuild<K, Base>
where
    Base: NodeLayer<K>,
{
    fn build(base: &Base) -> Self;
}

pub trait BaseComponent<K, V, Base>
where
    Self: NodeLayer<K>,
    Base: NodeLayer<K>,
{
    fn insert<'n>(
        &'n mut self,
        ptr: Self::Address,
        key: K,
        value: V,
    ) -> Option<PropogateInsert<'n, K, Self>>;

    fn get(&self, ptr: Self::Address, key: &K) -> Option<&V>;

    fn size(&self) -> usize;

    // type EntryIter<'n>: Iterator<Item = (&'n K, &'n V)>
    // where
    //     K: 'n,
    //     V: 'n,
    //     Self: 'n;
    //
    // fn range<'n>(&'n self, range: impl RangeBounds<(K, Self::Address)>) -> Self::EntryIter<'n>;
}

pub trait BaseComponentInMemoryBuild<K, V> {
    fn empty() -> Self;

    fn build(iter: impl Iterator<Item = (K, V)>) -> Self;
}