use crate::{Address, LinkedNode, NodeLayer};
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

pub struct MutIter<'n, K, N, SA, PA> {
    layer: &'n mut N,
    current: Option<SA>,
    end: Bound<SA>,
    _ph: std::marker::PhantomData<(K, PA)>,
}

impl<'n, K, SA, PA, N: NodeLayer<K, SA, PA>> MutIter<'n, K, N, SA, PA>
where
    SA: Address,
    PA: Address,
{
    pub fn range(layer: &'n mut N, start: Bound<SA>, end: Bound<SA>) -> Self {
        match start {
            Bound::Excluded(start) => Self {
                current: layer.deref(start).next(),
                layer,
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
                current: Some(layer.first()),
                layer,
                end,
                _ph: std::marker::PhantomData,
            },
        }
    }
}

impl<'n, K, SA, PA, N: NodeLayer<K, SA, PA>> Iterator for MutIter<'n, K, N, SA, PA>
where
    K: Copy,
    SA: Address,
    PA: Address,
{
    type Item = (K, SA, *mut N::Node);

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

        return Some((
            (*self.layer.lower_bound(current.clone())),
            current.clone(),
            unsafe { self.layer.deref_unsafe(current.clone()) },
        ));
    }
}
