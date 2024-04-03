use std::ops::Bound;

use crate::{node_layer::NodeLayer, traits::Address};

// ----------------------------------------
// Iterator Type
// ----------------------------------------

pub struct Iter<'n, K, N, SA, PA> {
    layer: &'n N,
    current: Option<SA>,
    end: Bound<SA>,
    _ph: std::marker::PhantomData<(K, PA)>,
}

impl<'n, K, SA, PA, N: NodeLayer<K, SA, PA>> Iter<'n, K, N, SA, PA>
where
    SA: Address,
    PA: Address,
{
    pub fn range(layer: &'n N, start: Bound<SA>, end: Bound<SA>) -> Self {
        match start {
            Bound::Excluded(start) => Self {
                layer,
                current: layer.next(start),
                end,
                _ph: std::marker::PhantomData,
            },

            Bound::Included(start) => Self {
                layer,
                current: Some(start.clone()),
                end,
                _ph: std::marker::PhantomData,
            },

            Bound::Unbounded => Self {
                layer,
                current: Some(layer.first()),
                end,
                _ph: std::marker::PhantomData,
            },
        }
    }
}

impl<'n, K, SA, PA, N: NodeLayer<K, SA, PA>> Iterator for Iter<'n, K, N, SA, PA>
where
    SA: Address,
    PA: Address,
{
    type Item = (K, SA);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.clone()?;

        match self.end.clone() {
            Bound::Excluded(end) => {
                if current == end {
                    return None;
                }
            }

            Bound::Included(end) => {
                if current == end {
                    self.current = None;
                }
            }

            _ => (),
        }

        // Advance pointer
        if let Some(current) = self.current.clone() {
            self.current = self.layer.next(current);
        }

        Some(((self.layer.lower_bound(current.clone())), current))
    }
}

// ----------------------------------------
// Mutable Iterator Type
// ----------------------------------------

pub struct IterMut<'n, K, N, SA, PA> {
    layer: &'n mut N,
    current: Option<SA>,
    end: Bound<SA>,
    _ph: std::marker::PhantomData<(K, PA)>,
}

impl<'n, K, SA, PA, N: NodeLayer<K, SA, PA>> IterMut<'n, K, N, SA, PA>
where
    K: Clone,
    SA: Address,
    PA: Address,
{
    pub fn range(layer: &'n mut N, start: Bound<SA>, end: Bound<SA>) -> Self {
        match start {
            Bound::Excluded(start) => {
                let current = layer.next(start);

                Self {
                    layer,
                    current,
                    end,
                    _ph: std::marker::PhantomData,
                }
            }

            Bound::Included(start) => Self {
                layer,
                current: Some(start.clone()),
                end,
                _ph: std::marker::PhantomData,
            },

            Bound::Unbounded => {
                let current = Some(layer.first());

                Self {
                    layer,
                    current,
                    end,
                    _ph: std::marker::PhantomData,
                }
            }
        }
    }

    #[allow(clippy::type_complexity)]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<(K, SA, IterMutParentView<'_, K, N, SA, PA>)> {
        let current = self.current.clone()?;

        match self.end.clone() {
            Bound::Excluded(end) => {
                if current == end {
                    return None;
                }
            }

            Bound::Included(end) => {
                if current == end {
                    self.current = None;
                }
            }

            _ => (),
        }

        // Advance pointer
        if let Some(current) = self.current.clone() {
            self.current = self.layer.next(current);
        }

        let key = self.layer.lower_bound(current.clone()).clone();
        let current = current.clone();
        let parent = IterMutParentView {
            layer: self.layer,
            address: current.clone(),
            _ph: std::marker::PhantomData,
        };

        Some((key, current, parent))
    }
}

pub struct IterMutParentView<'n, K, N, SA, PA> {
    layer: &'n mut N,
    address: SA,
    _ph: std::marker::PhantomData<(K, PA)>,
}

impl<'n, K, SA, PA, N: NodeLayer<K, SA, PA>> IterMutParentView<'n, K, N, SA, PA>
where
    SA: Address,
    PA: Address,
{
    pub fn set(self, parent: PA) {
        self.layer.set_parent(self.address.clone(), parent);
    }
}
