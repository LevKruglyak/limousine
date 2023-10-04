pub mod btree_top;
pub mod disk;
pub mod memory;

mod node;

pub use btree_top::*;
pub use disk::*;
pub use memory::*;
<<<<<<< HEAD
||||||| 84a7909

// -------------------------------------------------------
//                  Top Component
// -------------------------------------------------------

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
    fn search(&self, _: &Base, key: &K) -> BA {
        *self.inner.range(..=key).next_back().unwrap().1
    }

    fn insert(&mut self, base: &mut Base, prop: PropogateInsert<K, BA, ()>) {
        match prop {
            PropogateInsert::Single(key, address, _parent) => {
                // TODO: figure out how to leverage parent?
                self.inner.insert(key, address);
                base.deref_mut(address).set_parent(());
            }
            PropogateInsert::Replace { .. } => {
                unimplemented!()
                // self.inner.clear();
                //
                // for (key, address) in base.range(Bound::Unbounded, Bound::Unbounded) {
                //     self.inner.insert(key, address);
                // }
            }
        }
    }
}

impl<K, X, Base, BA> TopComponentInMemoryBuild<K, Base, BA, ()> for BTreeTopComponent<K, X, BA>
where
    Base: NodeLayer<K, BA, ()>,
    K: Ord + Copy,
    BA: Address + Copy,
{
    fn build(base: &mut Base) -> Self {
        let mut inner = BTreeMap::new();

        for view in base.mut_range(Bound::Unbounded, Bound::Unbounded) {
            inner.insert(view.key(), view.address());
            view.set_parent(());
        }

        Self {
            inner,
            _ph: std::marker::PhantomData,
        }
    }
}
=======

// -------------------------------------------------------
//                  Top Component
// -------------------------------------------------------
>>>>>>> 49533e6688fcf9413df1327f04baf6d9391e899b
