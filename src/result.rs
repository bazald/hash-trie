use crate::{node::{CNode, LNode, SNode}, traits::*, *};
use alloc::sync::Arc;

/// `BitError` enumerates possible error conditions when bitops are used "incorrectly."
#[derive(Debug)]
#[must_use]
pub enum BitError {
    /// `BitError::CountNotEqualToOne` indicates a word representing a bit contains either 2 or more bits or 0 bits.
    CountNotEqualToOne,
    /// `BitError::Found` indicates a bit that is supposed to be absent is present.
    Found,
    /// `BitError::NotFound` indicates a bit that is supposed to be present is absent.
    NotFound,
    /// `BitError::Range` indicates an index exceeding the word size was used.
    Range,
}

/// The only error you'll find is `NotFound`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HashTrieError {
    /// The value was not found.
    NotFound,
}

#[must_use]
pub(crate) enum FindResult<'a, K: Key, V: Value> {
    NotFound,
    Found(&'a K, &'a V),
}

#[must_use]
pub(crate) enum InsertResult<'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> {
    Found(&'a K, &'a V),
    InsertedC(CNode<H, F, K, V, M>, *const K, *const V),
    InsertedL(Arc<LNode<K, V>>, *const K, *const V),
    InsertedS(Arc<SNode<K, V>>, *const K, *const V),
}

#[must_use]
pub(crate) enum LNodeInsertResult<'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> {
    Found(&'a K, &'a V),
    InsertedC(CNode<H, F, K, V, M>, *const K, *const V),
    InsertedL(Arc<LNode<K, V>>, *const K, *const V),
}

impl <'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> From<LNodeInsertResult<'a, H, F, K, V, M>> for InsertResult<'a, H, F, K, V, M> {
    fn from(other: LNodeInsertResult<'a, H, F, K, V, M>) -> Self {
        match other {
            LNodeInsertResult::Found(key, value) => InsertResult::Found(key, value),
            LNodeInsertResult::InsertedC(lnode, key, value) => InsertResult::InsertedC(lnode, key, value),
            LNodeInsertResult::InsertedL(snode, key, value) => InsertResult::InsertedL(snode, key, value),
        }
    }
}

#[must_use]
pub(crate) enum RemoveResult<'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> {
    NotFound,
    RemovedC(CNode<H, F, K, V, M>, &'a K, &'a V),
    RemovedL(Arc<LNode<K, V>>, &'a K, &'a V),
    RemovedS(Arc<SNode<K, V>>, &'a K, &'a V),
    RemovedZ(&'a K, &'a V),
}

#[must_use]
pub(crate) enum LNodeRemoveResult<'a, K: Key, V: Value> {
    NotFound,
    RemovedL(Arc<LNode<K, V>>, &'a K, &'a V),
    RemovedS(Arc<SNode<K, V>>, &'a K, &'a V),
}

impl <'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> From<LNodeRemoveResult<'a, K, V>> for RemoveResult<'a, H, F, K, V, M> {
    fn from(other: LNodeRemoveResult<'a, K, V>) -> Self {
        match other {
            LNodeRemoveResult::NotFound => RemoveResult::NotFound,
            LNodeRemoveResult::RemovedL(lnode, key, value) => RemoveResult::RemovedL(lnode, key, value),
            LNodeRemoveResult::RemovedS(snode, key, value) => RemoveResult::RemovedS(snode, key, value),
        }
    }
}

#[must_use]
pub(crate) enum SNodeRemoveResult<'a, K: Key, V: Value> {
    NotFound,
    RemovedZ(&'a K, &'a V),
}

impl <'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> From<SNodeRemoveResult<'a, K, V>> for RemoveResult<'a, H, F, K, V, M> {
    fn from(other: SNodeRemoveResult<'a, K, V>) -> Self {
        match other {
            SNodeRemoveResult::NotFound => RemoveResult::NotFound,
            SNodeRemoveResult::RemovedZ(key, value) => RemoveResult::RemovedZ(key, value),
        }
    }
}

/// MapTransformResult is the result of a transform operation on a single entry of a map.
#[must_use]
pub enum MapTransformResult<K, V, ReduceT> {
    /// The values has been transformed.
    Transformed(K, V, ReduceT),
    /// The key-value pair was removed.
    Removed(ReduceT),
}

/// SetTransformResult is the result of a transform operation on a single entry of a set.
#[must_use]
pub enum SetTransformResult<K, ReduceT> {
    /// The value has been transformed.
    Transformed(K, ReduceT),
    /// The value was removed.
    Removed(ReduceT),
}

#[must_use]
pub(crate) enum MNodeTransformResult<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> {
    C(CNode<H, F, K, V, M>, ReduceT),
    L(Arc<LNode<K, V>>, ReduceT),
    S(Arc<SNode<K, V>>, ReduceT),
    Removed(ReduceT),
}

#[must_use]
pub(crate) enum LNodeTransformResult<K: Key, V: Value, ReduceT> {
    L(Arc<LNode<K, V>>, ReduceT),
    S(Arc<SNode<K, V>>, ReduceT),
    Removed(ReduceT),
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> From<LNodeTransformResult<K, V, ReduceT>> for MNodeTransformResult<H, F, K, V, M, ReduceT> {
    fn from(other: LNodeTransformResult<K, V, ReduceT>) -> Self {
        match other {
            LNodeTransformResult::L(lnode, reduced) => MNodeTransformResult::L(lnode, reduced),
            LNodeTransformResult::S(snode, reduced) => MNodeTransformResult::S(snode, reduced),
            LNodeTransformResult::Removed(reduced) => MNodeTransformResult::Removed(reduced),
        }
    }
}

#[must_use]
pub(crate) enum SNodeTransformResult<K: Key, V: Value, ReduceT> {
    S(Arc<SNode<K, V>>, ReduceT),
    Removed(ReduceT),
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> From<SNodeTransformResult<K, V, ReduceT>> for MNodeTransformResult<H, F, K, V, M, ReduceT> {
    fn from(other: SNodeTransformResult<K, V, ReduceT>) -> Self {
        match other {
            SNodeTransformResult::S(snode, reduced) => MNodeTransformResult::S(snode, reduced),
            SNodeTransformResult::Removed(reduced) => MNodeTransformResult::Removed(reduced),
        }
    }
}

impl <K: Key, V: Value, ReduceT> From<SNodeTransformResult<K, V, ReduceT>> for LNodeTransformResult<K, V, ReduceT> {
    fn from(other: SNodeTransformResult<K, V, ReduceT>) -> Self {
        match other {
            SNodeTransformResult::S(snode, reduced) => LNodeTransformResult::S(snode, reduced),
            SNodeTransformResult::Removed(reduced) => LNodeTransformResult::Removed(reduced),
        }
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> From<MapTransformResult<K, V, ReduceT>> for MNodeTransformResult<H, F, K, V, M, ReduceT> {
    fn from(other: MapTransformResult<K, V, ReduceT>) -> Self {
        match other {
            MapTransformResult::Transformed(key, value, reduced) => MNodeTransformResult::S(SNode::new(key, value), reduced),
            MapTransformResult::Removed(reduced) => MNodeTransformResult::Removed(reduced),
        }
    }
}

impl <K: Key, V: Value, ReduceT> From<MapTransformResult<K, V, ReduceT>> for LNodeTransformResult<K, V, ReduceT> {
    fn from(other: MapTransformResult<K, V, ReduceT>) -> Self {
        match other {
            MapTransformResult::Transformed(key, value, reduced) => LNodeTransformResult::S(SNode::new(key, value), reduced),
            MapTransformResult::Removed(reduced) => LNodeTransformResult::Removed(reduced),
        }
    }
}

impl <K: Key, V: Value, ReduceT> From<MapTransformResult<K, V, ReduceT>> for SNodeTransformResult<K, V, ReduceT> {
    fn from(other: MapTransformResult<K, V, ReduceT>) -> Self {
        match other {
            MapTransformResult::Transformed(key, value, reduced) => SNodeTransformResult::S(SNode::new(key, value), reduced),
            MapTransformResult::Removed(reduced) => SNodeTransformResult::Removed(reduced),
        }
    }
}

#[cfg(test)]
macro_rules! assert_found_eq {
    ( $found:expr, $expected:expr ) => {
        match $found {
            FindResult::Found(key, value) => {
                assert_eq!(*key, $expected.0);
                assert_eq!(*value, $expected.1);
            },
            FindResult::NotFound => panic!()
        }
    };
}

#[cfg(test)]
macro_rules! assert_found_none {
    ( $found:expr ) => {
        match $found {
            FindResult::Found(_key, _value) => panic!(),
            FindResult::NotFound => {}
        }
    };
}
