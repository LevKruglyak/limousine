use std::{borrow::Borrow, ops::Bound};

use generational_arena::{Arena, Index};

use crate::{
    kv::{KeyBounded, StaticBounded},
    learned::generic::Segmentation,
    Address, Entry, Key, LinkedNode, NodeLayer,
};

use super::{pgm_model::LinearModel, pgm_node::PGMNode};

type Node<K, V, const EPSILON: usize> = PGMNode<K, V, EPSILON>;

pub struct MemoryPGMNode<K: Key, V, const EPSILON: usize, PA> {
    pub inner: Node<K, V, EPSILON>,
    pub next: Option<Index>,
    pub previous: Option<Index>,
    pub parent: Option<PA>,
}

impl<K: Key, V: Clone, const EPSILON: usize, PA> MemoryPGMNode<K, V, EPSILON, PA> {}

impl<K: StaticBounded + Key, V: Clone, const EPSILON: usize, PA> KeyBounded<K> for MemoryPGMNode<K, V, EPSILON, PA> {
    fn lower_bound(&self) -> &K {
        self.inner.borrow()
    }
}

// TODO: I think we need to make LinkedNode's be doubly linked
impl<K: StaticBounded + Key, V: 'static + Clone, const EPSILON: usize, PA: 'static> LinkedNode<K, Index, PA>
    for MemoryPGMNode<K, V, EPSILON, PA>
where
    PA: Address,
{
    fn next(&self) -> Option<Index> {
        self.next
    }

    fn parent(&self) -> Option<PA> {
        self.parent.clone()
    }

    fn set_parent(&mut self, parent: PA) {
        self.parent = Some(parent);
    }
}

/// ----------------------------------------
/// Layer Type
/// ----------------------------------------

pub struct MemoryPGMLayer<K: Key, V: Clone, const EPSILON: usize, PA> {
    pub arena: Arena<MemoryPGMNode<K, V, EPSILON, PA>>,
    pub head: Option<Index>,
}

impl<K: Key, V, const EPSILON: usize, PA> NodeLayer<K, Index, PA> for MemoryPGMLayer<K, V, EPSILON, PA>
where
    K: 'static + StaticBounded + Clone,
    V: 'static + Clone,
    PA: Address,
{
    type Node = MemoryPGMNode<K, V, EPSILON, PA>;

    fn deref(&self, ptr: Index) -> &Self::Node {
        self.arena.get(ptr).unwrap()
    }

    fn deref_mut(&mut self, ptr: Index) -> &mut Self::Node {
        self.arena.get_mut(ptr).unwrap()
    }

    unsafe fn deref_unsafe(&self, ptr: Index) -> *mut Self::Node {
        self.arena.get(ptr).unwrap() as *const Self::Node as *mut Self::Node
    }

    fn first(&self) -> Index {
        self.head.unwrap()
    }
}

impl<K, V, const EPSILON: usize, PA> MemoryPGMLayer<K, V, EPSILON, PA>
where
    K: Clone + Key,
    V: 'static + Clone,
    PA: Address,
{
    /// Make an empty layer
    pub fn empty() -> Self {
        Self {
            arena: Arena::new(),
            head: None,
        }
    }

    /// Wipe this layer and rebuild it with the data in iter
    pub fn fill(&mut self, iter: impl Iterator<Item = Entry<K, V>> + Clone) {
        self.arena.clear();
        let blueprint = LinearModel::<K, EPSILON>::make_segmentation(iter);
        let mut ptr: Option<Index> = None;
        for (model, entries) in blueprint {
            let mut node = Node::from_model_n_vec(model, entries);
            let new_ptr = self.arena.insert(MemoryPGMNode {
                inner: node,
                next: None,
                previous: ptr,
                parent: None,
            });
            if self.head.is_none() {
                self.head = Some(new_ptr);
            }
            if ptr.is_some() {
                self.deref_mut(ptr.unwrap()).next = Some(new_ptr);
            }
            ptr = Some(new_ptr);
        }
    }

    /// Given the layer that is supposed to sit under this layer, fill this layer making sure
    /// to update the parents of the lower layer as needed
    pub fn fill_from_beneath<B>(&mut self, base: &mut B)
    where
        V: Address,
        B: NodeLayer<K, V, Index>,
    {
        // Just make two passes through the data for simplicity
        // First pass: build the layer
        let test = base.mut_range(Bound::Unbounded, Bound::Unbounded);
        let vec: Vec<Entry<K, V>> = test.map(|x| Entry::new(x.key(), x.address())).collect();
        self.fill(vec.into_iter());
        // Second pass: set parent pointer of base layer
        let mut parent = self.head;
        debug_assert!(parent.is_some());
        let mut next_parent = if parent.is_some() {
            self.deref(parent.unwrap()).next()
        } else {
            None
        };
        for view in base.mut_range(Bound::Unbounded, Bound::Unbounded) {
            if next_parent.is_none() || &view.key() < self.deref(next_parent.unwrap()).lower_bound() {
                view.set_parent(parent.unwrap());
            } else {
                parent = next_parent;
                next_parent = self.deref(parent.unwrap()).next();
                view.set_parent(parent.unwrap());
            }
        }
    }

    /// Assume that base B has had some potentially large continguous change.
    /// We will handle this by simply replacing all nodes in this layer who have a child participating in the change.
    /// `poison_head` is the address of the first node that needs to be replaced in this layer
    /// `poison_tail` is the address of the last node (INCLUSIVE) in that needs to be replaced in this layer
    /// `data_head` is the address of the first piece of data in the new node filling in the gap
    /// `data_tail` is the address of the last piece of data in the new node filling in the gap
    pub fn replace<B>(&mut self, base: &mut B, poison_head: Index, poison_tail: Index, data_head: V, data_tail: V)
    where
        V: Address,
        B: NodeLayer<K, V, Index>,
    {
        // AHHHHHH inefficient but (hopefully) correct
        // First let's construct a vector of all the things we're adding
        let mut finished = false;
        let mut bot_ptr = data_head.clone();
        let mut entries: Vec<Entry<K, V>> = vec![];
        while !finished {
            let node = base.deref(bot_ptr.clone());
            entries.push(Entry::new(node.lower_bound().clone(), bot_ptr.clone()));
            let next_bot_ptr = node.next();
            finished = (bot_ptr == data_tail) || next_bot_ptr.is_none();
            if next_bot_ptr.is_some() {
                bot_ptr = next_bot_ptr.unwrap();
            }
        }
        // println!("Replace is seeing {} entries", entries.len());
        // Then lets make the new chain
        let blueprint = LinearModel::<K, EPSILON>::make_segmentation(entries.into_iter());
        let mut chain_head: Option<Index> = None;
        let mut last_added_ptr: Option<Index> = None;
        for (model, entries) in blueprint {
            let new_inner = PGMNode::from_model_n_vec(model, entries);
            let new_ptr = self.arena.insert(MemoryPGMNode {
                inner: new_inner,
                next: None,
                previous: last_added_ptr,
                parent: None,
            });
            if chain_head.is_none() {
                chain_head = Some(new_ptr);
            }
            if last_added_ptr.is_some() {
                let node = self.deref_mut(last_added_ptr.unwrap());
                node.next = Some(new_ptr);
            }
            last_added_ptr = Some(new_ptr);
        }
        let chain_head = chain_head.unwrap();
        let chain_tail = last_added_ptr.unwrap();
        // We need to fix the linked list in this layer
        if self.head == Some(poison_head) {
            self.head = Some(chain_head);
        } else {
            let node = self.deref(poison_head);
            let previous_address = node.previous.unwrap();
            let mut prev_node = self.deref_mut(previous_address);
            prev_node.next = Some(chain_head);
        }
        let poison_tail_node = self.deref(poison_tail);
        let after_chain_address = poison_tail_node.next;
        let last_chain_node = self.deref_mut(chain_tail);
        last_chain_node.next = after_chain_address;
        // We need to clean up the linked list in this layer
        let mut finished = false;
        let mut bot_ptr = poison_head;
        while !finished {
            finished = bot_ptr == poison_tail;
            let next_address = self.deref(bot_ptr).next;
            self.arena.remove(bot_ptr);
            if next_address.is_some() {
                bot_ptr = next_address.unwrap();
            } else {
                break;
            }
        }
        // Finally we need to fix the parent pointers in the layer beneath us
        let mut finished = false;
        let mut top_ptr = chain_head;
        let mut bot_ptr = data_head;
        let mut entries: Vec<Entry<K, V>> = vec![];
        while !finished {
            let next_top_ptr = self.deref(top_ptr).next;
            if next_top_ptr == None {
                base.deref_mut(bot_ptr.clone()).set_parent(top_ptr);
            } else {
                let next_node = self.deref(next_top_ptr.unwrap());
                let bot_node = base.deref(bot_ptr.clone());
                if next_node.lower_bound() <= bot_node.lower_bound() {
                    top_ptr = next_top_ptr.unwrap();
                }
                base.deref_mut(bot_ptr.clone()).set_parent(top_ptr);
            }
            let node = base.deref(bot_ptr.clone());
            let next_bot_ptr = node.next();
            finished = (bot_ptr == data_tail) || next_bot_ptr.is_none();
            if next_bot_ptr.is_some() {
                bot_ptr = next_bot_ptr.unwrap();
            }
        }
    }
}

#[cfg(test)]
mod pgm_layer_tests {
    use kdam::{tqdm, Bar, BarExt};
    use rand::{distributions::Uniform, Rng};

    use crate::learned::generic::Model;

    use super::*;

    /// It's easier to write tests if we fix these
    const EPSILON: usize = 8;
    type Key = usize;
    type Value = usize;

    /// Helper function to generate random entries
    fn generate_random_entries(num_entries: usize, lb: usize, ub: usize) -> Vec<Entry<Key, Value>> {
        let range = Uniform::from(lb..ub);
        let mut random_values: Vec<usize> = rand::thread_rng().sample_iter(&range).take(num_entries).collect();
        random_values.sort();
        random_values.dedup();
        let entries: Vec<Entry<Key, Value>> = random_values
            .into_iter()
            .enumerate()
            .map(|(ix, key)| Entry::new(key, ix))
            .collect();
        entries
    }

    /// Helper function to make a simple layer
    fn make_simple_layer(num_elements: usize) -> MemoryPGMLayer<Key, Value, EPSILON, Index> {
        let entries = generate_random_entries(num_elements, Key::MIN, Key::MAX);
        let mut layer = MemoryPGMLayer::<Key, Value, EPSILON, Index>::empty();
        layer.fill(entries.into_iter());
        layer
    }

    /// Helper function to make a base layer and a layer on top of it
    fn make_two_layers(
        num_elements: usize,
    ) -> (
        MemoryPGMLayer<Key, Value, EPSILON, Index>,
        MemoryPGMLayer<Key, Index, EPSILON, Index>,
    ) {
        let mut beneath = make_simple_layer(num_elements);
        let mut layer = MemoryPGMLayer::<Key, Index, EPSILON, Index>::empty();
        layer.fill_from_beneath::<MemoryPGMLayer<Key, Value, EPSILON, Index>>(&mut beneath);
        (beneath, layer)
    }

    /// Helper function to generate a random replace
    /// NOTE: Has the side-effect of actually deleting + replacing stuff in the base layer
    /// NOTE: DOES NOT do anything to the top layer
    /// Returns: The poison head, poison tail, data_head, data_tail (confusing bc they all have the same type)
    fn generate_fake_replace(
        beneath: &mut MemoryPGMLayer<Key, Value, EPSILON, Index>,
        above: &MemoryPGMLayer<Key, Index, EPSILON, Index>,
    ) -> (Index, Index, Index, Index) {
        // NOTE: Yes this is inefficient but goal is to be comprehensible
        // First get the number of nodes in the beneath layer
        let mut bot_ptr = beneath.head;
        let mut num_bot_nodes: usize = 0;
        while bot_ptr.is_some() {
            let mem_node = beneath.deref(bot_ptr.unwrap());
            num_bot_nodes += 1;
            bot_ptr = mem_node.next;
        }

        // Then pick a random node to start replacing at, a random number of elements to replace, and a random number of new elements to train on
        let start_replace_ix: usize = rand::thread_rng().gen_range(0..(num_bot_nodes - 2)); // 2 arbitrary
        let mut num_replace: usize = rand::thread_rng().gen_range(2..(num_bot_nodes / 10)); // 10 is arbitrary
        num_replace = num_replace.min(num_bot_nodes - start_replace_ix);
        let num_new = rand::thread_rng().gen_range(100..1000); // _everything_ arbitrary

        // Translate the first replacing node and last replace
        // NOTE: These exist in the _bottom_ layer
        let mut start_replace_address: Option<Index> = None;
        let mut end_replace_address: Option<Index> = None;
        let mut bot_ptr = beneath.head;
        let mut ix: usize = 0;
        while bot_ptr.is_some() {
            if ix == start_replace_ix {
                start_replace_address = bot_ptr;
            }
            if ix == start_replace_ix + num_replace - 1 {
                end_replace_address = bot_ptr;
            }
            let mem_node = beneath.deref(bot_ptr.unwrap());
            bot_ptr = mem_node.next;
            ix += 1;
        }
        assert!(start_replace_address.is_some());
        assert!(end_replace_address.is_some());
        let start_replace_address = start_replace_address.unwrap();
        let end_replace_address = end_replace_address.unwrap();

        // Get the poison bounds
        let start_replace_node = beneath.deref(start_replace_address);
        let end_replace_node = beneath.deref(end_replace_address);
        let poison_head = start_replace_node.parent.unwrap();
        let poison_tail = end_replace_node.parent.unwrap();

        // Generate linked-list of new entries
        let min_new = start_replace_node.lower_bound().clone();
        let max_new = end_replace_node.lower_bound() + 1;
        let new_entries = generate_random_entries(num_new, min_new, max_new);
        let blueprint = LinearModel::<Key, EPSILON>::make_segmentation(new_entries.into_iter());
        let mut first_added: Option<Index> = None;
        let mut last_added: Option<Index> = None;
        let mut num_gen = 0;
        for (model, entries) in blueprint {
            let new_address = beneath.arena.insert(MemoryPGMNode {
                inner: PGMNode::from_model_n_vec(model, entries),
                next: None,
                previous: last_added,
                parent: None,
            });
            if first_added.is_none() {
                first_added = Some(new_address);
            }
            if last_added.is_some() {
                let mut node = beneath.deref_mut(last_added.unwrap());
                node.next = Some(new_address);
            }
            last_added = Some(new_address);
            num_gen += 1;
        }
        let first_added = first_added.unwrap();
        let last_added = last_added.unwrap();

        // Find data_start
        let mut data_start_address = above.deref(poison_head).inner.data[0].value;
        if data_start_address == start_replace_address {
            // This is the edge case where the leftmost bound of our replace is the very first thing in our poison head
            // In this case we need to pass in the address into the new nodes.
            data_start_address = first_added;
        }

        // Fix the linked list
        let start_replace_node = beneath.deref(start_replace_address);
        let end_replace_node = beneath.deref(end_replace_address);
        let prev_addr = start_replace_node.previous.clone();
        let next_addr = end_replace_node.next;
        if Some(start_replace_address) == beneath.head {
            beneath.head = Some(first_added);
        } else {
            let prev_node = beneath.deref_mut(prev_addr.unwrap());
            prev_node.next = Some(first_added);
        }
        let last_new_node = beneath.deref_mut(last_added);
        last_new_node.next = next_addr;

        // Calculate debug value
        let mut start_count = 0;
        let mut test_ptr = data_start_address;
        while test_ptr != first_added {
            let node = beneath.deref(test_ptr);
            test_ptr = node.next.unwrap();
            start_count += 1;
        }

        // Find data_end
        let mut data_end_address = end_replace_address;
        let mut next_data_end_address = beneath.deref(data_end_address).next;
        let mut end_count = 0;
        while next_data_end_address.is_some() {
            let node = beneath.deref(next_data_end_address.unwrap());
            if node.parent != Some(poison_tail) {
                break;
            }
            end_count += 1;
            data_end_address = next_data_end_address.unwrap();
            next_data_end_address = beneath.deref(data_end_address).next;
        }

        // Sanity
        // println!(
        //     "Replace should be seeing {} + {} + {} = {} entries",
        //     start_count,
        //     num_gen,
        //     end_count,
        //     start_count + num_gen + end_count
        // );

        // Clean-up the deleted elements
        let mut finished = false;
        let mut bot_ptr = start_replace_address;
        while !finished {
            finished = bot_ptr == end_replace_address;
            let next_address = beneath.deref(bot_ptr).next;
            beneath.arena.remove(bot_ptr);
            if next_address.is_some() {
                bot_ptr = next_address.unwrap();
            }
        }

        (poison_head, poison_tail, data_start_address, data_end_address)
    }

    /// Helper function to ensure a memory node has a model that works
    fn test_mem_node_model<V: Clone + 'static>(mem_node: &MemoryPGMNode<Key, V, EPSILON, Index>) {
        let node = &mem_node.inner;
        for (ix, entry) in node.entries().iter().enumerate() {
            let pred_ix = node.approximate(&entry.key);
            assert!(pred_ix.lo <= ix && ix < pred_ix.hi);
        }
    }

    /// Helper function to make sure a layer is normal
    fn test_is_layer_normal<V: Clone + 'static>(
        layer: &MemoryPGMLayer<Key, V, EPSILON, Index>,
        size_hint: Option<usize>,
    ) {
        // First lets check that the total size of all nodes in layer is what we expect
        let mut ptr = layer.head;
        let mut seen: usize = 0;
        while ptr.is_some() {
            let mem_node = layer.deref(ptr.unwrap());
            seen += mem_node.inner.data.len();
            ptr = mem_node.next;
        }
        if size_hint.is_some() {
            assert!(seen == size_hint.unwrap());
        } else {
            assert!(seen > 0);
        }
        // Then for each node lets check that all its entries are well-approximated
        let mut ptr = layer.head;
        let mut seen: usize = 0;
        while ptr.is_some() {
            let mem_node = layer.deref(ptr.unwrap());
            test_mem_node_model(mem_node);
            ptr = mem_node.next;
        }
    }

    /// Helper function to make sure a pair of layers is normal
    fn test_are_layers_normal(
        beneath: &MemoryPGMLayer<Key, Value, EPSILON, Index>,
        above: &MemoryPGMLayer<Key, Index, EPSILON, Index>,
    ) {
        // First lets check that every node in the bottom layer has a parent, and that that parent has a key <= our key,
        // and that the next parent either doesn't exist or has a key > our key
        let mut bot_ptr = beneath.head;
        while bot_ptr.is_some() {
            let mem_node = beneath.deref(bot_ptr.unwrap());
            assert!(mem_node.parent.is_some());
            let parent_node = above.deref(mem_node.parent.unwrap());
            assert!(mem_node.lower_bound() <= mem_node.lower_bound());
            if parent_node.next.is_some() {
                let uncle_node = above.deref(parent_node.next.unwrap());
                assert!(mem_node.lower_bound() < uncle_node.lower_bound());
            }
            bot_ptr = mem_node.next;
        }
        // Now we'll try indexing into the lower level through the first level
        let mut bot_ptr = beneath.head;
        while bot_ptr.is_some() {
            let mem_node = beneath.deref(bot_ptr.unwrap());
            let entries = mem_node.inner.entries();
            let parent_node = above.deref(mem_node.parent.unwrap());
            for entry in entries {
                let mut approx = parent_node.inner.approximate(&entry.key);
                approx.hi = approx.hi.min(parent_node.inner.entries().len());
                let mut found = false;
                for ix in approx.lo..approx.hi {
                    let value = parent_node.inner.entries()[ix].value;
                    if value == bot_ptr.unwrap() {
                        found = true;
                        break;
                    }
                }
                assert!(found);
            }
            bot_ptr = mem_node.next;
        }
    }

    /// This tests the basic functionalities of fill as nothing more than a bunch of wrappers.
    /// Specifically, given an iterator over entries, we should be able to build a layer, which is a
    /// connected list of nodes, with accurate models and data.
    #[test]
    fn basic_fill() {
        let size: usize = 10_000;
        let layer = make_simple_layer(size);
        test_is_layer_normal::<Value>(&layer, Some(size));
    }

    /// This test the functionality of building a layer on top of a layer below it.
    #[test]
    fn fill_from_beneath() {
        let beneath_size: usize = 100_000;
        let mut beneath = make_simple_layer(beneath_size);
        let mut layer = MemoryPGMLayer::<Key, Index, EPSILON, Index>::empty();
        layer.fill_from_beneath::<MemoryPGMLayer<Key, Value, EPSILON, Index>>(&mut beneath);
        test_are_layers_normal(&beneath, &layer);
    }

    /// Runs a single trial of our replacement correctness test
    fn test_replace_trial(num_elements: usize) {
        let (mut beneath, mut above) = make_two_layers(num_elements);
        let (poison_head, poison_tail, data_head, data_tail) = generate_fake_replace(&mut beneath, &above);
        // Let's make sure that the beneath layer is still normal
        test_is_layer_normal(&beneath, None);
        // Magic!
        above.replace(&mut beneath, poison_head, poison_tail, data_head, data_tail);
        // Magic?
        test_are_layers_normal(&beneath, &above);
    }

    #[test]
    fn test_replace() {
        let num_elements: usize = 1_000_000;
        let num_trials: usize = 10;
        let mut pb = tqdm!(total = num_trials);
        for _ in 0..num_trials {
            pb.update(1);
            test_replace_trial(num_elements);
        }
    }
}
