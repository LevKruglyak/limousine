// Type dependence hierarchy

use std::ops::Bound;

use crate::iter::{Iter, IterMut};
use crate::traits::*;

/// A `NodeLayer` is has the interface of a linked list of key-bounded nodes which implement the
/// `Model` trait. It's assumed that a `NodeLayer` is always non-empty, and thus should always have
/// a `first` and `last` node.
pub trait NodeLayer<K, SA, PA>: 'static + Sized
where
    K: Copy,
    SA: Address,
    PA: Address,
{
    fn parent(&self, ptr: SA) -> Option<PA>;

    fn set_parent(&mut self, ptr: SA, parent: PA);

    /// Get the lower bound of a node. This could be overridden by some layers which might have a
    /// more optimal way of mapping the address to the lower bound.
    fn lower_bound(&self, ptr: SA) -> K;

    fn next(&self, ptr: SA) -> Option<SA>;

    /// First node in the current node layer
    fn first(&self) -> SA;

    /// Last node in the current node layer
    fn last(&self) -> SA;

    /// An immutable iterator over the layer, returning (Key, Address) pairs
    fn range(&self, start: Bound<SA>, end: Bound<SA>) -> Iter<'_, K, Self, SA, PA> {
        Iter::range(self, start, end)
    }

    /// An iterator over the layer, returning (Key, Address, ParentView) pairs, where parents
    /// can be modified by the ParentView struct
    fn range_mut(&mut self, start: Bound<SA>, end: Bound<SA>) -> IterMut<'_, K, Self, SA, PA> {
        IterMut::range(self, start, end)
    }
}

macro_rules! impl_node_layer {
    ($SA:ty, $PA:ty) => {
        fn parent(&self, ptr: $SA) -> Option<$PA> {
            self.inner.parent(ptr)
        }

        fn set_parent(&mut self, ptr: $SA, parent: $PA) {
            self.inner.set_parent(ptr, parent)
        }

        fn lower_bound(&self, ptr: $SA) -> K {
            self.inner.lower_bound(ptr)
        }

        fn next(&self, ptr: $SA) -> Option<$SA> {
            self.inner.next(ptr)
        }

        fn first(&self) -> $SA {
            self.inner.first()
        }

        fn last(&self) -> $SA {
            self.inner.last()
        }
    };
}

pub(crate) use impl_node_layer;
