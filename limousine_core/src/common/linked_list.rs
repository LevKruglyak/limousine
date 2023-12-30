use std::ops::Bound;

use crate::common::bounded::KeyBounded;
use crate::{Address, Model, NodeLayer};
use generational_arena::Arena;

pub use generational_arena::Index;

/// A simple struct for storying connected nodes in a layer, abstracted to avoid dupe
/// N - Data that defines a node (for BTree this is Stackmap, for PGM this is data + model)
/// PA - Parent address type
/// NOTE: We never allow this to be empty
/// TODO: Solidify the sentinel requirement. First value added should be a sentinel.
pub struct LinkedList<N, PA> {
    pub arena: Arena<LinkedNode<N, PA>>,
    pub first: Index,
    pub last: Index,
}

pub struct LinkedNode<N, PA> {
    pub inner: N,
    next: Option<Index>,
    previous: Option<Index>,
    parent: Option<PA>,
}

impl<N, PA: Address> LinkedNode<N, PA> {
    pub fn new(node: N) -> Self {
        Self {
            inner: node,
            next: None,
            previous: None,
            parent: None,
        }
    }

    pub fn init(node: N, next: Option<Index>, previous: Option<Index>, parent: Option<PA>) -> Self {
        Self {
            inner: node,
            next,
            previous,
            parent,
        }
    }

    pub fn next(&self) -> Option<Index> {
        self.next
    }

    pub fn set_next(&mut self, next: Option<Index>) {
        self.next = next;
    }

    pub fn previous(&self) -> Option<Index> {
        self.previous
    }

    pub fn set_previous(&mut self, previous: Option<Index>) {
        self.previous = previous;
    }

    pub fn parent(&self) -> Option<PA> {
        self.parent.clone()
    }
}

impl<N, PA: Address> LinkedList<N, PA> {
    pub fn new(inner: N) -> Self {
        let mut arena = Arena::new();
        let ptr = arena.insert(LinkedNode::new(inner));

        LinkedList {
            arena,
            first: ptr,
            last: ptr,
        }
    }

    /// Append to the end of the list
    /// NOTE: Since these lists actually float around with a sentinel,
    /// not quite as simple as adding past tail (but almost that simple)
    pub fn append_before_sentinel(&mut self, inner: N) -> Index {
        self.insert_before(inner, self.last)
    }

    pub fn insert_after(&mut self, inner: N, ptr: Index) -> Index {
        let next_ptr = self.arena[ptr].next;

        let mut new_node = LinkedNode::new(inner);
        new_node.previous = Some(ptr);
        new_node.next = next_ptr;

        let new_node_ptr = self.arena.insert(new_node);
        self.arena[ptr].next = Some(new_node_ptr);

        if let Some(next_ptr) = next_ptr {
            self.arena[next_ptr].previous = Some(new_node_ptr);
        } else {
            self.last = new_node_ptr;
        }

        new_node_ptr
    }

    pub fn insert_before(&mut self, inner: N, ptr: Index) -> Index {
        let previous_ptr = self.arena[ptr].previous;

        let mut new_node = LinkedNode::new(inner);
        new_node.previous = previous_ptr;
        new_node.next = Some(ptr);

        let new_node_ptr = self.arena.insert(new_node);
        self.arena[ptr].previous = Some(new_node_ptr);

        if let Some(previous_ptr) = previous_ptr {
            self.arena[previous_ptr].next = Some(new_node_ptr);
        } else {
            self.first = new_node_ptr;
        }

        new_node_ptr
    }

    pub fn delete_subchain(&mut self, start: Bound<Index>, end: Bound<Index>) {
        // First translate the bounds
        let mut cur_ix: Index;
        match start {
            Bound::Included(ix) => {
                cur_ix = ix;
            }
            Bound::Excluded(ix) => {
                let next_ix = self.arena[ix].next();
                if next_ix.is_some() {
                    cur_ix = next_ix.unwrap();
                } else {
                    // Nothing to delete
                    return;
                }
            }
            Bound::Unbounded => {
                cur_ix = self.first;
            }
        }
        let sentinel_ix = match end {
            Bound::Included(ix) => self.arena[ix].next().unwrap(),
            Bound::Excluded(ix) => ix,
            Bound::Unbounded => self.last,
        };
        // Remember info that will let us tape the list back together
        let before = self.arena[cur_ix].previous();
        let mut after = self.arena[cur_ix].next().unwrap();
        // Do the deleting
        while cur_ix != sentinel_ix {
            after = self.arena[cur_ix].next().unwrap();
            let temp = self.arena[cur_ix].next().unwrap();
            self.arena.remove(cur_ix);
            cur_ix = temp;
        }
        // Retape
        match before {
            Some(ix) => {
                self.arena[ix].set_next(Some(after));
                self.arena[after].set_previous(Some(ix));
            }
            None => {
                self.first = after;
                self.arena[after].set_previous(None);
            }
        }
    }

    pub fn replace(&mut self, poison_head: Index, poison_tail: Index, mut new_data: impl Iterator<Item = N>) {
        // Add in the first new node
        let mut new_ptr = self.insert_before(new_data.next().unwrap(), poison_head);
        // Now we delete everything that was poisoned
        self.delete_subchain(Bound::Included(poison_head), Bound::Included(poison_tail));
        // Add in the rest
        for node in new_data {
            new_ptr = self.insert_after(node, new_ptr);
        }
    }

    pub fn clear(&mut self, inner: N) -> Index {
        self.arena.clear();
        let ptr = self.arena.insert(LinkedNode::new(inner));

        self.first = ptr;
        self.last = ptr;
        ptr
    }

    pub fn parent(&self, ptr: Index) -> Option<PA>
    where
        PA: Address,
    {
        self.arena[ptr].parent.clone()
    }

    pub fn len(&self) -> usize {
        self.arena.len()
    }
}

// ----------------------------------------
// Common implementations
// ----------------------------------------

impl<K, N, PA> KeyBounded<K> for LinkedNode<N, PA>
where
    N: KeyBounded<K>,
{
    fn lower_bound(&self) -> &K {
        self.inner.lower_bound()
    }
}

impl<K, N, PA> Model<K, Index, PA> for LinkedNode<N, PA>
where
    N: KeyBounded<K> + 'static,
    PA: Address,
{
    fn next(&self) -> Option<Index> {
        self.next
    }

    fn previous(&self) -> Option<Index> {
        self.previous
    }

    fn parent(&self) -> Option<PA> {
        self.parent.clone()
    }

    fn set_parent(&mut self, parent: PA) {
        self.parent = Some(parent);
    }
}

impl<N, PA> std::ops::Index<Index> for LinkedList<N, PA> {
    type Output = N;

    fn index(&self, index: Index) -> &Self::Output {
        &self.arena[index].inner
    }
}

impl<N, PA> std::ops::IndexMut<Index> for LinkedList<N, PA> {
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        &mut self.arena[index].inner
    }
}

impl<K, N, PA> NodeLayer<K, Index, PA> for LinkedList<N, PA>
where
    K: Copy,
    N: KeyBounded<K> + 'static,
    PA: Address,
{
    type Node = LinkedNode<N, PA>;

    fn deref(&self, ptr: Index) -> &Self::Node {
        &self.arena[ptr]
    }

    fn deref_mut(&mut self, ptr: Index) -> &mut Self::Node {
        &mut self.arena[ptr]
    }

    unsafe fn deref_unsafe(&self, ptr: Index) -> *mut Self::Node {
        self.arena.get(ptr).unwrap() as *const Self::Node as *mut Self::Node
    }

    fn first(&self) -> Index {
        self.first
    }

    fn last(&self) -> Index {
        self.last
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linked_list_new() {
        let list: LinkedList<i32, ()> = LinkedList::new(1);

        assert_eq!(list.len(), 1);
        assert_eq!(list[list.first], 1);
        assert_eq!(list.first, list.last);
    }

    #[test]
    fn test_linked_list_insert_after() {
        let mut list: LinkedList<i32, ()> = LinkedList::new(1);

        let first_ptr = list.first;
        let second_ptr = list.insert_after(2, first_ptr);

        assert_eq!(list.arena[first_ptr].next, Some(second_ptr));
        assert_eq!(list.arena[second_ptr].previous, Some(first_ptr));
        assert_eq!(list.last, second_ptr);
    }

    #[test]
    fn test_linked_list_insert_before() {
        let mut list: LinkedList<i32, ()> = LinkedList::new(1);

        let first_ptr = list.first;
        let zero_ptr = list.insert_before(0, first_ptr);

        assert_eq!(list.arena[first_ptr].previous, Some(zero_ptr));
        assert_eq!(list.arena[zero_ptr].next, Some(first_ptr));
        assert_eq!(list.first, zero_ptr);
    }

    #[test]
    fn test_linked_list_clear() {
        let mut list: LinkedList<i32, ()> = LinkedList::new(1);
        list.insert_after(2, list.first);

        assert_eq!(list.arena.len(), 2);

        list.clear(5);

        assert_eq!(list.len(), 1);
        assert_eq!(list[list.first], 5);
        assert_eq!(list.first, list.last);
    }

    #[test]
    fn test_linked_node_new() {
        let node: LinkedNode<i32, ()> = LinkedNode::new(10);

        assert_eq!(node.inner, 10);
        assert_eq!(node.next, None);
        assert_eq!(node.previous, None);
        assert_eq!(node.parent, None);
    }

    #[test]
    fn test_linked_append_before_sentinel() {
        let mut list: LinkedList<i32, ()> = LinkedList::new(100);

        let new_head_ptr = list.append_before_sentinel(2);
        let sentinel_ptr = list.last;

        assert_eq!(list.arena[new_head_ptr].next, Some(sentinel_ptr));
        assert_eq!(list.arena[sentinel_ptr].previous, Some(new_head_ptr));
        assert_eq!(list.first, new_head_ptr);
    }
}
