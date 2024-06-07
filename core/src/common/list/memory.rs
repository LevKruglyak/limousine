use generational_arena::Arena;

use crate::{
    node_layer::NodeLayer,
    traits::{Address, KeyBounded},
};

pub type ArenaID = generational_arena::Index;

pub struct MemoryList<N, PA> {
    arena: Arena<(MemoryNode<N>, Option<PA>)>,
    first: ArenaID,
    last: ArenaID,
}

#[derive(Default)]
pub struct MemoryNode<N> {
    pub inner: N,
    next: Option<ArenaID>,
    previous: Option<ArenaID>,
}

impl<N> MemoryNode<N> {
    fn new(node: N) -> Self {
        Self {
            inner: node,
            next: None,
            previous: None,
        }
    }
}

impl<N, PA> MemoryList<N, PA>
where
    N: Default,
{
    pub fn empty() -> Self {
        let mut arena = Arena::new();
        let ptr = arena.insert((Default::default(), None));

        MemoryList {
            arena,
            first: ptr,
            last: ptr,
        }
    }

    #[must_use]
    pub fn insert_after(&mut self, node: N, ptr: ArenaID) -> ArenaID {
        let next_ptr = self.arena[ptr].0.next;

        let mut new_node = MemoryNode::new(node);
        new_node.previous = Some(ptr);
        new_node.next = next_ptr;

        let new_node_ptr = self.arena.insert((new_node, None));
        self.arena[ptr].0.next = Some(new_node_ptr);

        if let Some(next_ptr) = next_ptr {
            self.arena[next_ptr].0.previous = Some(new_node_ptr);
        } else {
            self.last = new_node_ptr;
        }

        new_node_ptr
    }

    #[allow(unused)]
    #[must_use]
    pub fn insert_before(&mut self, node: N, ptr: ArenaID) -> ArenaID {
        let previous_ptr = self.arena[ptr].0.previous;

        let mut new_node = MemoryNode::new(node);
        new_node.previous = previous_ptr;
        new_node.next = Some(ptr);

        let new_node_ptr = self.arena.insert((new_node, None));
        self.arena[ptr].0.previous = Some(new_node_ptr);

        if let Some(previous_ptr) = previous_ptr {
            self.arena[previous_ptr].0.next = Some(new_node_ptr);
        } else {
            self.first = new_node_ptr;
        }

        new_node_ptr
    }

    #[must_use]
    pub fn clear(&mut self) -> ArenaID {
        self.arena.clear();
        let ptr = self
            .arena
            .insert((MemoryNode::new(Default::default()), None));

        self.first = ptr;
        self.last = ptr;
        ptr
    }

    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.arena.len()
    }
}

// ----------------------------------------
// Common implementations
// ----------------------------------------

impl<K, N> KeyBounded<K> for MemoryNode<N>
where
    N: KeyBounded<K>,
{
    fn lower_bound(&self) -> &K {
        self.inner.lower_bound()
    }
}

impl<N> AsRef<MemoryNode<N>> for &MemoryNode<N> {
    fn as_ref(&self) -> &MemoryNode<N> {
        self
    }
}

impl<N, PA> std::ops::Index<ArenaID> for MemoryList<N, PA> {
    type Output = N;

    fn index(&self, index: ArenaID) -> &Self::Output {
        &self.arena[index].0.inner
    }
}

impl<N, PA> std::ops::IndexMut<ArenaID> for MemoryList<N, PA> {
    fn index_mut(&mut self, index: ArenaID) -> &mut Self::Output {
        &mut self.arena[index].0.inner
    }
}

impl<K, N, PA> NodeLayer<K, ArenaID, PA> for MemoryList<N, PA>
where
    K: Clone,
    N: KeyBounded<K>,
    PA: Address,
{
    fn parent(&self, ptr: ArenaID) -> Option<PA> {
        self.arena[ptr].1.clone()
    }

    fn set_parent(&mut self, ptr: ArenaID, parent: PA) {
        self.arena[ptr].1 = Some(parent);
    }

    fn lower_bound(&self, ptr: ArenaID) -> K {
        self.arena[ptr].0.lower_bound().clone()
    }

    fn next(&self, ptr: ArenaID) -> Option<ArenaID> {
        self.arena[ptr].0.next
    }

    fn prev(&self, ptr: ArenaID) -> Option<ArenaID> {
        self.arena[ptr].0.previous
    }

    fn first(&self) -> ArenaID {
        self.first
    }

    fn last(&self) -> ArenaID {
        self.last
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linked_list_new() {
        let list: MemoryList<i32, ()> = MemoryList::empty();

        assert_eq!(list.len(), 1);
        assert_eq!(list[list.first], Default::default());
        assert_eq!(list.first, list.last);
    }

    #[test]
    fn linked_list_insert_after() {
        let mut list: MemoryList<u32, ()> = MemoryList::empty();

        let first_ptr = list.first;
        let second_ptr = list.insert_after(2, first_ptr);

        assert_eq!(list.arena[first_ptr].0.next, Some(second_ptr));
        assert_eq!(list.arena[second_ptr].0.previous, Some(first_ptr));
        assert_eq!(list.last, second_ptr);
    }

    #[test]
    fn linked_list_insert_before() {
        let mut list: MemoryList<u32, ()> = MemoryList::empty();

        let first_ptr = list.first;
        let zero_ptr = list.insert_before(0, first_ptr);

        assert_eq!(list.arena[first_ptr].0.previous, Some(zero_ptr));
        assert_eq!(list.arena[zero_ptr].0.next, Some(first_ptr));
        assert_eq!(list.first, zero_ptr);
    }

    #[test]
    fn test_linked_list_clear() {
        let mut list: MemoryList<i32, ()> = MemoryList::empty();
        list.insert_after(2, list.first);

        assert_eq!(list.arena.len(), 2);

        list.clear();

        assert_eq!(list.len(), 1);
        assert_eq!(list[list.first], Default::default());
        assert_eq!(list.first, list.last);
    }

    #[test]
    fn linked_node_new() {
        let node: MemoryNode<i32> = MemoryNode::new(10);

        assert_eq!(node.inner, 10);
        assert_eq!(node.next, None);
        assert_eq!(node.previous, None);
    }
}
