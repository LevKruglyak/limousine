macro_rules! impl_node_layer {
    ($SA:ty, $PA:ty) => {
        fn deref(&self, ptr: $SA) -> &Self::Node {
            self.inner.deref(ptr)
        }

        fn deref_mut(&mut self, ptr: $SA) -> &mut Self::Node {
            self.inner.deref_mut(ptr)
        }

        fn parent(&self, ptr: $SA) -> Option<$PA> {
            self.inner.parent(ptr)
        }

        fn set_parent(&mut self, ptr: $SA, parent: $PA) {
            self.inner.set_parent(ptr, parent)
        }

        unsafe fn set_parent_unsafe(&self, ptr: $SA, parent: $PA) {
            self.inner.set_parent_unsafe(ptr, parent)
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
