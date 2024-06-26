use std::ops::Bound;

use crate::{
    classical::node::BTreeNode,
    common::{
        list::boundary_disk::BoundaryDiskList,
        storage::{GlobalStore, StoreID},
    },
    impl_node_layer, Address, Key, KeyBounded, NodeLayer, Persisted,
};

pub struct BoundaryDiskBTreeLayer<K, V, const FANOUT: usize, PA>
where
    K: Persisted + Ord,
    V: Persisted,
{
    inner: BoundaryDiskList<BTreeNode<K, V, FANOUT>, PA>,
}

impl<K, V, const FANOUT: usize, PA> BoundaryDiskBTreeLayer<K, V, FANOUT, PA>
where
    K: Persisted + Key,
    V: Persisted,
    PA: Address,
{
    pub fn load(store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self> {
        Ok(Self {
            inner: BoundaryDiskList::load(store, ident)?,
        })
    }

    pub fn fill(&mut self, iter: impl Iterator<Item = (K, V)>) -> crate::Result<()>
    where
        K: Clone + Ord,
    {
        if let Some(mut ptr) = self.inner.is_empty()? {
            for (key, address) in iter {
                // If node too full, carry over to next
                if self.inner.get_node(ptr)?.unwrap().is_half_full() {
                    ptr = self.inner.insert_after(BTreeNode::empty(), ptr)?;
                }

                self.insert_into_node(key, &address, ptr)?;
            }
        }

        Ok(())
    }

    pub fn fill_with_parent<B: NodeLayer<K, V, StoreID>>(
        &mut self,
        base: &mut B,
    ) -> crate::Result<()> {
        if let Some(mut ptr) = self.inner.is_empty()? {
            let mut iter = base.range_mut(Bound::Unbounded, Bound::Unbounded);

            while let Some((key, address, parent)) = iter.next() {
                // If node too full, carry over to next
                if self.inner.get_node(ptr)?.unwrap().is_half_full() {
                    ptr = self.inner.insert_after(BTreeNode::empty(), ptr)?;
                }

                self.insert_into_node(key.clone(), &address, ptr)?;
                parent.set(ptr);
            }
        }

        Ok(())
    }

    fn insert_into_node(&mut self, key: K, value: &V, ptr: StoreID) -> crate::Result<Option<V>> {
        self.inner
            .transform_node(ptr, |node| node.insert(key.clone(), value.clone()))
    }

    pub fn get_node(&self, ptr: StoreID) -> crate::Result<BTreeNode<K, V, FANOUT>> {
        self.inner.get_node(ptr).map(|node| node.unwrap())
    }

    pub fn insert(
        &mut self,
        key: K,
        value: V,
        ptr: StoreID,
    ) -> crate::Result<Option<(K, StoreID, PA)>> {
        if self.inner.get_node(ptr)?.unwrap().is_full() {
            let parent = self.inner.parent(ptr).unwrap();

            // Split
            let (split_point, new_node) = self.inner.transform_node(ptr, BTreeNode::split)?;
            let new_node_ptr = self.inner.insert_after(new_node, ptr)?;

            // Insert into the right node
            if key < split_point {
                self.insert_into_node(key, &value, ptr)?;
            } else {
                self.insert_into_node(key, &value, new_node_ptr)?;
            }

            return Ok(Some((
                self.inner
                    .get_node(new_node_ptr)?
                    .unwrap()
                    .lower_bound()
                    .clone(),
                new_node_ptr,
                parent,
            )));
        } else {
            self.insert_into_node(key, &value, ptr)?;
        }

        Ok(None)
    }

    pub fn insert_with_parent<B: NodeLayer<K, V, StoreID>>(
        &mut self,
        key: K,
        value: V,
        base: &mut B,
        ptr: StoreID,
    ) -> crate::Result<Option<(K, StoreID, PA)>> {
        if self.inner.get_node(ptr)?.unwrap().is_full() {
            let parent = self.inner.parent(ptr).unwrap();

            // Split
            let (split_point, new_node) = self.inner.transform_node(ptr, BTreeNode::split)?;
            let new_node_ptr = self.inner.insert_after(new_node, ptr)?;

            // Update all of the parents for the split node
            for entry in self.inner.get_node(new_node_ptr)?.unwrap().entries() {
                base.set_parent(entry.value.clone(), new_node_ptr)
            }

            // Insert into the right node
            if key < split_point {
                self.insert_into_node(key, &value, ptr)?;
                base.set_parent(value, ptr);
            } else {
                self.insert_into_node(key, &value, new_node_ptr)?;
                base.set_parent(value, new_node_ptr);
            }

            return Ok(Some((
                self.inner
                    .get_node(new_node_ptr)?
                    .unwrap()
                    .lower_bound()
                    .clone(),
                new_node_ptr,
                parent,
            )));
        } else {
            self.insert_into_node(key, &value, ptr)?;
            base.set_parent(value, ptr);
        }

        Ok(None)
    }
}

impl<K, V, const FANOUT: usize, PA> NodeLayer<K, StoreID, PA>
    for BoundaryDiskBTreeLayer<K, V, FANOUT, PA>
where
    K: Persisted + Key,
    V: Persisted + Eq,
    PA: Address,
{
    impl_node_layer!(StoreID, PA);
}
