use crate::component::{PropagateInsert, TopComponent};
use crate::node_layer::NodeLayer;
use crate::traits::Address;
use crate::Key;
use std::collections::BTreeMap;
use std::ops::Bound;

/// A `TopComponent` implementation built around the BTreeMap implementation in the Rust standard
/// library.
pub struct BTreeTopComponent<K, X, A> {
    pub inner: BTreeMap<K, A>,
    _ph: std::marker::PhantomData<X>,
}

impl<K, X, Base, BA: Copy> TopComponent<K, Base, BA, ()> for BTreeTopComponent<K, X, BA>
where
    Base: NodeLayer<K, BA, ()>,
    K: Key,
    BA: Address,
{
    fn search(&self, _: &Base, key: &K) -> BA {
        *self
            .inner
            .range(..=key)
            .next_back()
            .unwrap_or(self.inner.range(..).next().unwrap())
            .1
    }

    fn insert(&mut self, base: &mut Base, prop: PropagateInsert<K, BA, ()>) {
        match prop {
            PropagateInsert::Single(key, address, _) => {
                self.inner.insert(key, address);
                base.set_parent(address, ());
            }
            _ => unimplemented!(),
        }
    }

    fn build(base: &mut Base) -> Self {
        let mut inner = BTreeMap::new();
        let mut iter = base.range_mut(Bound::Unbounded, Bound::Unbounded);

        while let Some((key, address, parent)) = iter.next() {
            inner.insert(key.clone(), address);
            parent.set(());
        }

        Self {
            inner,
            _ph: std::marker::PhantomData,
        }
    }
}
