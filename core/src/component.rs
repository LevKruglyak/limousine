use crate::common::storage::GlobalStore;
use crate::node_layer::NodeLayer;
use crate::traits::*;

pub enum PropagateInsert<K, SA, PA> {
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
    K: Key,
{
    fn search(&self, base: &Base, key: &K) -> BA;

    fn insert(&mut self, base: &mut Base, prop: PropagateInsert<K, BA, SA>);

    fn build(base: &mut Base) -> Self;
}

pub trait InternalComponent<K, Base, BA, SA, PA>
where
    Self: NodeLayer<K, SA, PA>,
    Base: NodeLayer<K, BA, SA>,
    BA: Address,
    SA: Address,
    PA: Address,
    K: Key,
{
    fn search(&self, base: &Base, ptr: SA, key: &K) -> BA;

    fn insert(
        &mut self,
        base: &mut Base,
        prop: PropagateInsert<K, BA, SA>,
    ) -> Option<PropagateInsert<K, SA, PA>>;

    fn build(base: &mut Base) -> Self;
}

pub trait BoundaryDiskInternalComponent<K, Base, BA, SA, PA>
where
    Self: NodeLayer<K, SA, PA> + Sized,
    Base: NodeLayer<K, BA, SA>,
    BA: Persisted + Address,
    SA: Persisted + Address,
    PA: Address,
    K: Key,
{
    fn search(&self, base: &Base, ptr: SA, key: &K) -> crate::Result<BA>;

    fn insert(
        &mut self,
        base: &mut Base,
        prop: PropagateInsert<K, BA, SA>,
    ) -> crate::Result<Option<PropagateInsert<K, SA, PA>>>;

    fn load(base: &mut Base, store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self>;
}

pub trait DeepDiskInternalComponent<K, Base, BA, SA, PA>
where
    Self: NodeLayer<K, SA, PA> + Sized,
    Base: NodeLayer<K, BA, SA>,
    BA: Persisted + Address,
    SA: Persisted + Address,
    PA: Persisted + Address,
    K: Key,
{
    fn search(&self, base: &Base, ptr: SA, key: &K) -> crate::Result<BA>;

    fn insert(
        &mut self,
        base: &mut Base,
        prop: PropagateInsert<K, BA, SA>,
    ) -> crate::Result<Option<PropagateInsert<K, SA, PA>>>;

    fn load(base: &mut Base, store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self>;
}

pub trait BaseComponent<K, V, SA, PA>
where
    Self: NodeLayer<K, SA, PA>,
    SA: Address,
    PA: Address,
    K: Key,
{
    fn insert(&mut self, ptr: SA, key: K, value: V) -> Option<PropagateInsert<K, SA, PA>>;

    fn search(&self, ptr: SA, key: &K) -> Option<V>;

    fn empty() -> Self;

    fn build(iter: impl Iterator<Item = (K, V)>) -> Self;
}

pub trait BoundaryDiskBaseComponent<K, V, SA, PA>
where
    Self: NodeLayer<K, SA, PA> + Sized,
    SA: Persisted + Address,
    PA: Address,
    K: Key,
{
    fn insert(
        &mut self,
        ptr: SA,
        key: K,
        value: V,
    ) -> crate::Result<Option<PropagateInsert<K, SA, PA>>>;

    fn search(&self, ptr: SA, key: &K) -> crate::Result<Option<V>>;

    fn load(store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self>;
}

pub trait DeepDiskBaseComponent<K, V, SA, PA>
where
    Self: NodeLayer<K, SA, PA> + Sized,
    SA: Persisted + Address,
    PA: Persisted + Address,
    K: Key,
{
    fn insert(
        &mut self,
        ptr: SA,
        key: K,
        value: V,
    ) -> crate::Result<Option<PropagateInsert<K, SA, PA>>>;

    fn search(&self, ptr: SA, key: &K) -> crate::Result<Option<V>>;

    fn load(store: &mut GlobalStore, ident: impl ToString) -> crate::Result<Self>;
}
