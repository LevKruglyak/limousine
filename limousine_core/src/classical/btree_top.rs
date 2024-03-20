use crate::component::*;
use crate::node_layer::NodeLayer;
use crate::traits::Address;
use std::collections::BTreeMap;
use std::ops::Bound;

/// A `TopComponent` implementation built around the BTreeMap implementation in the Rust standard
/// library.
pub struct BTreeTopComponent<K, X, A> {
    inner: BTreeMap<K, A>,
    _ph: std::marker::PhantomData<X>,
}

impl<K, X, Base, BA: Copy> TopComponent<K, Base, BA, ()> for BTreeTopComponent<K, X, BA>
where
    Base: NodeLayer<K, BA, ()>,
    K: Ord + Copy,
    BA: Address,
{
    fn search(&self, _: &Base, key: K) -> BA {
        *self.inner.range(..=key).next_back().unwrap().1
    }

    fn insert(&mut self, base: &mut Base, prop: PropagateInsert<K, BA, ()>) {
        match prop {
            PropagateInsert::Single(key, address, _parent) => {
                // TODO: figure out how to leverage parent?
                self.inner.insert(key, address);
                base.set_parent(address, ());
            }
            PropagateInsert::Replace { .. } => {
                unimplemented!()
            }
        }
    }
}

impl<K, X, Base, BA> TopComponentBuild<K, Base, BA, ()> for BTreeTopComponent<K, X, BA>
where
    Base: NodeLayer<K, BA, ()>,
    K: Ord + Copy,
    BA: Address + Copy,
{
    fn build(base: &mut Base) -> Self {
        let mut inner = BTreeMap::new();
        let mut iter = base.range_mut(Bound::Unbounded, Bound::Unbounded);

        while let Some((key, address, parent)) = iter.next() {
            inner.insert(key, address);
            parent.set(());
        }

        Self {
            inner,
            _ph: std::marker::PhantomData,
        }
    }
}
