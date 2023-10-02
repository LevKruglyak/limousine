use crate::{Address, Model, NodeLayer};
use std::mem::MaybeUninit;
use std::ops::{Bound, RangeBounds};

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
    K: Copy,
{
    pub fn range(layer: &'n N, start: Bound<SA>, end: Bound<SA>) -> Self {
        match start {
            Bound::Excluded(start) => Self {
                layer,
                current: layer.deref(start).next(),
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
    K: Copy,
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
            self.current = self.layer.deref(current).next();
        }

        return Some(((*self.layer.lower_bound(current.clone())), current.clone()));
    }
}

pub struct MutNodeView<'n, K, N, SA, PA> {
    layer: &'n N,
    current: Option<SA>,
    _ph: std::marker::PhantomData<(K, PA)>,
}

impl<'n, K, SA, PA, N: NodeLayer<K, SA, PA>> Clone for MutNodeView<'n, K, N, SA, PA>
where
    N: NodeLayer<K, SA, PA>,
    SA: Address + Clone,
    PA: Address,
    K: Copy,
{
    fn clone(&self) -> Self {
        Self {
            layer: self.layer,
            current: self.current.clone(),
            _ph: std::marker::PhantomData,
        }
    }
}

impl<'n, K, SA, PA, N: NodeLayer<K, SA, PA>> MutNodeView<'n, K, N, SA, PA>
where
    K: Copy,
    N: NodeLayer<K, SA, PA>,
    SA: Address,
    PA: Address,
{
    fn new(layer: &'n mut N) -> Self {
        Self {
            layer,
            current: None,
            _ph: std::marker::PhantomData,
        }
    }

    fn set_current(&mut self, current: SA) {
        self.current = Some(current);
    }

    pub fn key(&self) -> K {
        let current = self.current.clone().unwrap();
        self.layer.lower_bound(current).clone()
    }

    pub fn address(&self) -> SA {
        self.current.clone().unwrap()
    }

    pub fn set_parent(&self, parent: PA) {
        let current = self.current.clone().unwrap();
        unsafe { self.layer.deref_unsafe(current).as_mut().unwrap() }.set_parent(parent)
    }
}

pub struct MutIter<'n, K, N, SA, PA> {
    view: MutNodeView<'n, K, N, SA, PA>,
    current: Option<SA>,
    end: Bound<SA>,
    _ph: std::marker::PhantomData<(K, PA)>,
}

impl<'n, K, SA, PA, N: NodeLayer<K, SA, PA>> MutIter<'n, K, N, SA, PA>
where
    K: Copy,
    SA: Address,
    PA: Address,
{
    pub fn range(layer: &'n mut N, start: Bound<SA>, end: Bound<SA>) -> Self {
        match start {
            Bound::Excluded(start) => Self {
                current: layer.deref(start).next(),
                view: MutNodeView::new(layer),
                end,
                _ph: std::marker::PhantomData,
            },

            Bound::Included(start) => Self {
                view: MutNodeView::new(layer),
                current: Some(start.clone()),
                end,
                _ph: std::marker::PhantomData,
            },

            Bound::Unbounded => Self {
                current: Some(layer.first()),
                view: MutNodeView::new(layer),
                end,
                _ph: std::marker::PhantomData,
            },
        }
    }
}

impl<'n, K, SA, PA, N: NodeLayer<K, SA, PA>> Iterator for MutIter<'n, K, N, SA, PA>
where
    K: Copy + 'n,
    SA: Address,
    PA: Address,
{
    type Item = MutNodeView<'n, K, N, SA, PA>;

    fn next(&mut self) -> Option<(Self::Item)> {
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
            self.current = self.view.layer.deref(current).next();
        }

        self.view.set_current(current);
        Some(self.view.clone())
    }
}
