use crate::{node::{CNode, LNode, LNodeNext, SNode}, traits::*, *};
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

#[derive(Clone, Debug)]
pub struct KeyRef<K: Key, V: Value> {
    lsnode: LNodeNext<K, V>,
}

impl <K: Key, V: Value> KeyRef<K, V> {
    pub(crate) fn new(lsnode: LNodeNext<K, V>) -> Self {
        Self {
            lsnode
        }
    }

    pub fn get(&self) -> &K {
        self.lsnode.key()
    }
}

#[derive(Clone, Debug)]
pub struct KeyValueRef<K: Key, V: Value> {
    lsnode: LNodeNext<K, V>,
}

impl <K: Key, V: Value> KeyValueRef<K, V> {
    pub(crate) fn new(lsnode: LNodeNext<K, V>) -> Self {
        Self {
            lsnode
        }
    }

    pub fn key(&self) -> &K {
        self.lsnode.key()
    }

    pub fn value(&self) -> &V {
        self.lsnode.value()
    }
}

impl <K: Key, V: Value> From<KeyValueRef<K, V>> for KeyRef<K, V> {
    fn from(other: KeyValueRef<K, V>) -> Self {
        KeyRef::new(other.lsnode)
    }
}

impl <K: Key, V: Value> From<Arc<LNode<K, V>>> for KeyValueRef<K, V> {
    fn from(other: Arc<LNode<K, V>>) -> Self {
        KeyValueRef::new(LNodeNext::L(other))
    }
}

impl <K: Key, V: Value> From<Arc<SNode<K, V>>> for KeyValueRef<K, V> {
    fn from(other: Arc<SNode<K, V>>) -> Self {
        KeyValueRef::new(LNodeNext::S(other))
    }
}

#[must_use]
pub(crate) enum FindResult<K: Key, V: Value> {
    NotFound,
    Found(KeyValueRef<K, V>),
}

#[must_use]
pub(crate) enum InsertResult<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> {
    Found(KeyValueRef<K, V>),
    InsertedC(CNode<H, F, K, V, M>, KeyValueRef<K, V>),
    InsertedL(Arc<LNode<K, V>>, KeyValueRef<K, V>),
    InsertedS(Arc<SNode<K, V>>, KeyValueRef<K, V>),
}

#[must_use]
pub(crate) enum RemoveResult<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> {
    NotFound,
    RemovedC(CNode<H, F, K, V, M>, KeyValueRef<K, V>),
    RemovedL(Arc<LNode<K, V>>, KeyValueRef<K, V>),
    RemovedS(Arc<SNode<K, V>>, KeyValueRef<K, V>),
    RemovedZ(KeyValueRef<K, V>),
}

#[must_use]
pub(crate) enum LNodeRemoveResult<K: Key, V: Value> {
    NotFound,
    RemovedL(Arc<LNode<K, V>>, KeyValueRef<K, V>),
    RemovedS(Arc<SNode<K, V>>, KeyValueRef<K, V>),
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> From<LNodeRemoveResult<K, V>> for RemoveResult<H, F, K, V, M> {
    fn from(other: LNodeRemoveResult<K, V>) -> Self {
        match other {
            LNodeRemoveResult::NotFound => RemoveResult::NotFound,
            LNodeRemoveResult::RemovedL(lnode, key_value) => RemoveResult::RemovedL(lnode, key_value),
            LNodeRemoveResult::RemovedS(snode, key_value) => RemoveResult::RemovedS(snode, key_value),
        }
    }
}

#[must_use]
pub(crate) enum SNodeRemoveResult<K: Key, V: Value> {
    NotFound,
    RemovedZ(KeyValueRef<K, V>),
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> From<SNodeRemoveResult<K, V>> for RemoveResult<H, F, K, V, M> {
    fn from(other: SNodeRemoveResult<K, V>) -> Self {
        match other {
            SNodeRemoveResult::NotFound => RemoveResult::NotFound,
            SNodeRemoveResult::RemovedZ(key_value) => RemoveResult::RemovedZ(key_value),
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
            FindResult::Found(key_value) => {
                assert_eq!(*key_value.key(), $expected.0);
                assert_eq!(*key_value.value(), $expected.1);
            },
            FindResult::NotFound => panic!()
        }
    };
}

#[cfg(test)]
macro_rules! assert_found_none {
    ( $found:expr ) => {
        match $found {
            FindResult::Found(_key_value) => panic!(),
            FindResult::NotFound => {}
        }
    };
}
