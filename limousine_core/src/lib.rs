#![allow(unused, dead_code)]

use crate::kv::{Key, Value};
use core::marker::PhantomData;
use num::PrimInt;
use std::cell::Ref;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::{borrow::Borrow, fmt::Debug, ptr::NonNull};
use trait_set::trait_set;

pub mod classical;
mod common;
// mod learned;
mod kv;

/// Some collection of key-bounded nodes which can be indexed
pub trait NodeLayer<K: Key>: 'static {
    /// Node type
    type Node: Borrow<K>;

    type NodeRef: Clone;

    fn node_ref(&self, ptr: Self::NodeRef) -> &Self::Node;

    type NodeIter<'n>: Iterator<Item = Self::NodeRef>;

    fn iter<'n>(&'n self) -> Self::NodeIter<'n>;
}

// -------------------------------------------------------
//                  Components
// -------------------------------------------------------

pub trait TopComponent<K: Key, B: NodeLayer<K>> {
    fn new_top(base: &B) -> Self;

    fn search_top(&self, key: &K) -> B::NodeRef;

    fn insert_top(&mut self, key: K, value: B::NodeRef);
}

pub trait InternalComponent<K: Key, B: NodeLayer<K>>: NodeLayer<K> {
    fn new_internal(base: &B) -> Self;

    fn search_internal(&self, key: &K, ptr: Self::NodeRef) -> B::NodeRef;

    fn insert_internal(
        &mut self,
        key: K,
        value: B::NodeRef,
        ptr: Self::NodeRef,
    ) -> Option<(K, Self::NodeRef)>;
}

pub trait BaseComponent<K: Key, V: Value>: NodeLayer<K> {
    fn new_base() -> Self;

    fn search_base(&self, key: &K, ptr: Self::NodeRef) -> Option<&V>;

    fn insert_base(&mut self, key: K, value: V, ptr: Self::NodeRef) -> Option<(K, Self::NodeRef)>;
}
