macro_rules! impl_node_layer {
    ($SA:ty) => {
        fn deref(&self, ptr: $SA) -> &Self::Node {
            self.inner.deref(ptr)
        }

        fn deref_mut(&mut self, ptr: $SA) -> &mut Self::Node {
            self.inner.deref_mut(ptr)
        }

        unsafe fn deref_unsafe(&self, ptr: $SA) -> *mut Self::Node {
            self.inner.deref_unsafe(ptr)
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
