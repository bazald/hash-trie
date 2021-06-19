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
    InsertedC(CNode<H, F, K, V, M>, *const K, *const V, Option<(&'a K, &'a V)>),
    InsertedL(Arc<LNode<K, V>>, *const K, *const V, Option<(&'a K, &'a V)>),
    InsertedS(Arc<SNode<K, V>>, *const K, *const V, Option<(&'a K, &'a V)>),
}

#[must_use]
pub(crate) enum CNodeInsertResult<'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> {
    Found(&'a K, &'a V),
    InsertedC(CNode<H, F, K, V, M>, *const K, *const V, Option<(&'a K, &'a V)>),
}

impl <'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> From<CNodeInsertResult<'a, H, F, K, V, M>> for InsertResult<'a, H, F, K, V, M> {
    fn from(other: CNodeInsertResult<'a, H, F, K, V, M>) -> Self {
        match other {
            CNodeInsertResult::Found(key, value) => InsertResult::Found(key, value),
            CNodeInsertResult::InsertedC(lnode, key, value, prev) => InsertResult::InsertedC(lnode, key, value, prev),
        }
    }
}

#[must_use]
pub(crate) enum LNodeInsertResult<'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> {
    Found(&'a K, &'a V),
    InsertedC(CNode<H, F, K, V, M>, *const K, *const V, Option<(&'a K, &'a V)>),
    InsertedL(Arc<LNode<K, V>>, *const K, *const V, Option<(&'a K, &'a V)>),
}

impl <'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> From<LNodeInsertResult<'a, H, F, K, V, M>> for InsertResult<'a, H, F, K, V, M> {
    fn from(other: LNodeInsertResult<'a, H, F, K, V, M>) -> Self {
        match other {
            LNodeInsertResult::Found(key, value) => InsertResult::Found(key, value),
            LNodeInsertResult::InsertedC(lnode, key, value, prev) => InsertResult::InsertedC(lnode, key, value, prev),
            LNodeInsertResult::InsertedL(snode, key, value, prev) => InsertResult::InsertedL(snode, key, value, prev),
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

/// SetTransformResult is the result of a transform operation on a single entry of a set.
#[must_use]
pub enum SetTransformResult<ReduceT> {
    /// The value is unchanged.
    Unchanged(ReduceT),
    /// The value was removed.
    Removed(ReduceT),
}

/// SetJointTransformResult is the result of a transform operation on a pair of entries of a set.
#[must_use]
pub enum SetJointTransformResult<ReduceT> {
    /// The value is unchanged from both left and right.
    UnchangedLR(ReduceT),
    /// The value is unchanged from the left.
    UnchangedL(ReduceT),
    /// The value is unchanged from the right.
    UnchangedR(ReduceT),
    /// The value was removed.
    Removed(ReduceT),
}

/// SetTransmuteResult is the result of a transmute operation on a single entry or pair of entries of a set.
#[must_use]
pub enum SetTransmuteResult<K, ReduceT> {
    /// The value has been transmuted.
    Transmuted(K, ReduceT),
    /// The value was removed.
    Removed(ReduceT),
}

/// MapTransformResult is the result of a transform operation on a single entry of a map.
#[must_use]
pub enum MapTransformResult<V, ReduceT> {
    /// The value is unchanged.
    Unchanged(ReduceT),
    /// The value has been transformed.
    Transformed(V, ReduceT),
    /// The key-value pair was removed.
    Removed(ReduceT),
}

/// MapJointTransformResult is the result of a transform operation on a pair of entries of a map.
#[must_use]
pub enum MapJointTransformResult<V, ReduceT> {
    /// The value is unchanged from both the left and right.
    UnchangedLR(ReduceT),
    /// The value is unchanged from the left.
    UnchangedL(ReduceT),
    /// The value is unchanged from the right.
    UnchangedR(ReduceT),
    /// The value has been transformed.
    Transformed(V, ReduceT),
    /// The key-value pair was removed.
    Removed(ReduceT),
}

/// MapTransmuteResult is the result of a transmute operation on a single entry of a map or a pair of entries of a map.
#[must_use]
pub enum MapTransmuteResult<K, V, ReduceT> {
    /// The value has been transmuted.
    Transmuted(K, V, ReduceT),
    /// The key-value pair was removed.
    Removed(ReduceT),
}

#[must_use]
pub(crate) enum MNodeTransformResult<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> {
    Unchanged(ReduceT),
    C(CNode<H, F, K, V, M>, ReduceT),
    L(Arc<LNode<K, V>>, ReduceT),
    S(Arc<SNode<K, V>>, ReduceT),
    Removed(ReduceT),
}

#[must_use]
pub(crate) enum MNodeJointTransformResult<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> {
    UnchangedLR(ReduceT),
    UnchangedL(ReduceT),
    UnchangedR(ReduceT),
    C(CNode<H, F, K, V, M>, ReduceT),
    L(Arc<LNode<K, V>>, ReduceT),
    S(Arc<SNode<K, V>>, ReduceT),
    Removed(ReduceT),
}

#[must_use]
pub(crate) enum MNodeTransmuteResult<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> {
    C(CNode<H, F, K, V, M>, ReduceT),
    L(Arc<LNode<K, V>>, ReduceT),
    S(Arc<SNode<K, V>>, ReduceT),
    Removed(ReduceT),
}

#[must_use]
pub(crate) enum LNodeTransformResult<K: Key, V: Value, ReduceT> {
    Unchanged(ReduceT),
    L(Arc<LNode<K, V>>, ReduceT),
    S(Arc<SNode<K, V>>, ReduceT),
    Removed(ReduceT),
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> From<LNodeTransformResult<K, V, ReduceT>> for MNodeTransformResult<H, F, K, V, M, ReduceT> {
    fn from(other: LNodeTransformResult<K, V, ReduceT>) -> Self {
        match other {
            LNodeTransformResult::Unchanged(reduced) => MNodeTransformResult::Unchanged(reduced),
            LNodeTransformResult::L(lnode, reduced) => MNodeTransformResult::L(lnode, reduced),
            LNodeTransformResult::S(snode, reduced) => MNodeTransformResult::S(snode, reduced),
            LNodeTransformResult::Removed(reduced) => MNodeTransformResult::Removed(reduced),
        }
    }
}

#[must_use]
pub(crate) enum LNodeJointTransformResult<K: Key, V: Value, ReduceT> {
    UnchangedLR(ReduceT),
    UnchangedL(ReduceT),
    UnchangedR(ReduceT),
    L(Arc<LNode<K, V>>, ReduceT),
    S(Arc<SNode<K, V>>, ReduceT),
    Removed(ReduceT),
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> From<LNodeJointTransformResult<K, V, ReduceT>> for MNodeJointTransformResult<H, F, K, V, M, ReduceT> {
    fn from(other: LNodeJointTransformResult<K, V, ReduceT>) -> Self {
        match other {
            LNodeJointTransformResult::UnchangedLR(reduced) => MNodeJointTransformResult::UnchangedLR(reduced),
            LNodeJointTransformResult::UnchangedL(reduced) => MNodeJointTransformResult::UnchangedL(reduced),
            LNodeJointTransformResult::UnchangedR(reduced) => MNodeJointTransformResult::UnchangedR(reduced),
            LNodeJointTransformResult::L(lnode, reduced) => MNodeJointTransformResult::L(lnode, reduced),
            LNodeJointTransformResult::S(snode, reduced) => MNodeJointTransformResult::S(snode, reduced),
            LNodeJointTransformResult::Removed(reduced) => MNodeJointTransformResult::Removed(reduced),
        }
    }
}

#[must_use]
pub(crate) enum LNodeTransmuteResult<K: Key, V: Value, ReduceT> {
    L(Arc<LNode<K, V>>, ReduceT),
    S(Arc<SNode<K, V>>, ReduceT),
    Removed(ReduceT),
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> From<LNodeTransmuteResult<K, V, ReduceT>> for MNodeTransmuteResult<H, F, K, V, M, ReduceT> {
    fn from(other: LNodeTransmuteResult<K, V, ReduceT>) -> Self {
        match other {
            LNodeTransmuteResult::L(lnode, reduced) => MNodeTransmuteResult::L(lnode, reduced),
            LNodeTransmuteResult::S(snode, reduced) => MNodeTransmuteResult::S(snode, reduced),
            LNodeTransmuteResult::Removed(reduced) => MNodeTransmuteResult::Removed(reduced),
        }
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> From<MapTransmuteResult<K, V, ReduceT>> for MNodeTransmuteResult<H, F, K, V, M, ReduceT> {
    fn from(other: MapTransmuteResult<K, V, ReduceT>) -> Self {
        match other {
            MapTransmuteResult::Transmuted(key, value, reduced) => MNodeTransmuteResult::S(SNode::new(key, value), reduced),
            MapTransmuteResult::Removed(reduced) => MNodeTransmuteResult::Removed(reduced),
        }
    }
}

impl <K: Key, V: Value, ReduceT> From<MapTransmuteResult<K, V, ReduceT>> for LNodeTransmuteResult<K, V, ReduceT> {
    fn from(other: MapTransmuteResult<K, V, ReduceT>) -> Self {
        match other {
            MapTransmuteResult::Transmuted(key, value, reduced) => LNodeTransmuteResult::S(SNode::new(key, value), reduced),
            MapTransmuteResult::Removed(reduced) => LNodeTransmuteResult::Removed(reduced),
        }
    }
}

#[must_use]
pub(crate) enum SNodeTransformResult<K: Key, V: Value, ReduceT> {
    Unchanged(ReduceT),
    S(Arc<SNode<K, V>>, ReduceT),
    Removed(ReduceT),
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> From<SNodeTransformResult<K, V, ReduceT>> for MNodeTransformResult<H, F, K, V, M, ReduceT> {
    fn from(other: SNodeTransformResult<K, V, ReduceT>) -> Self {
        match other {
            SNodeTransformResult::Unchanged(reduced) => MNodeTransformResult::Unchanged(reduced),
            SNodeTransformResult::S(snode, reduced) => MNodeTransformResult::S(snode, reduced),
            SNodeTransformResult::Removed(reduced) => MNodeTransformResult::Removed(reduced),
        }
    }
}

impl <K: Key, V: Value, ReduceT> From<SNodeTransformResult<K, V, ReduceT>> for LNodeTransformResult<K, V, ReduceT> {
    fn from(other: SNodeTransformResult<K, V, ReduceT>) -> Self {
        match other {
            SNodeTransformResult::Unchanged(reduced) => LNodeTransformResult::Unchanged(reduced),
            SNodeTransformResult::S(snode, reduced) => LNodeTransformResult::S(snode, reduced),
            SNodeTransformResult::Removed(reduced) => LNodeTransformResult::Removed(reduced),
        }
    }
}

#[must_use]
pub(crate) enum SNodeJointTransformResult<K: Key, V: Value, ReduceT> {
    UnchangedLR(ReduceT),
    UnchangedL(ReduceT),
    UnchangedR(ReduceT),
    S(Arc<SNode<K, V>>, ReduceT),
    Removed(ReduceT),
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> From<SNodeJointTransformResult<K, V, ReduceT>> for MNodeJointTransformResult<H, F, K, V, M, ReduceT> {
    fn from(other: SNodeJointTransformResult<K, V, ReduceT>) -> Self {
        match other {
            SNodeJointTransformResult::UnchangedLR(reduced) => MNodeJointTransformResult::UnchangedLR(reduced),
            SNodeJointTransformResult::UnchangedL(reduced) => MNodeJointTransformResult::UnchangedL(reduced),
            SNodeJointTransformResult::UnchangedR(reduced) => MNodeJointTransformResult::UnchangedR(reduced),
            SNodeJointTransformResult::S(snode, reduced) => MNodeJointTransformResult::S(snode, reduced),
            SNodeJointTransformResult::Removed(reduced) => MNodeJointTransformResult::Removed(reduced),
        }
    }
}

impl <K: Key, V: Value, ReduceT> From<SNodeJointTransformResult<K, V, ReduceT>> for LNodeJointTransformResult<K, V, ReduceT> {
    fn from(other: SNodeJointTransformResult<K, V, ReduceT>) -> Self {
        match other {
            SNodeJointTransformResult::UnchangedLR(reduced) => LNodeJointTransformResult::UnchangedLR(reduced),
            SNodeJointTransformResult::UnchangedL(reduced) => LNodeJointTransformResult::UnchangedL(reduced),
            SNodeJointTransformResult::UnchangedR(reduced) => LNodeJointTransformResult::UnchangedR(reduced),
            SNodeJointTransformResult::S(snode, reduced) => LNodeJointTransformResult::S(snode, reduced),
            SNodeJointTransformResult::Removed(reduced) => LNodeJointTransformResult::Removed(reduced),
        }
    }
}

#[must_use]
pub(crate) enum SNodeTransmuteResult<K: Key, V: Value, ReduceT> {
    S(Arc<SNode<K, V>>, ReduceT),
    Removed(ReduceT),
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static, ReduceT> From<SNodeTransmuteResult<K, V, ReduceT>> for MNodeTransmuteResult<H, F, K, V, M, ReduceT> {
    fn from(other: SNodeTransmuteResult<K, V, ReduceT>) -> Self {
        match other {
            SNodeTransmuteResult::S(snode, reduced) => MNodeTransmuteResult::S(snode, reduced),
            SNodeTransmuteResult::Removed(reduced) => MNodeTransmuteResult::Removed(reduced),
        }
    }
}

impl <K: Key, V: Value, ReduceT> From<SNodeTransmuteResult<K, V, ReduceT>> for LNodeTransmuteResult<K, V, ReduceT> {
    fn from(other: SNodeTransmuteResult<K, V, ReduceT>) -> Self {
        match other {
            SNodeTransmuteResult::S(snode, reduced) => LNodeTransmuteResult::S(snode, reduced),
            SNodeTransmuteResult::Removed(reduced) => LNodeTransmuteResult::Removed(reduced),
        }
    }
}

impl <K: Key, V: Value, ReduceT> From<MapTransmuteResult<K, V, ReduceT>> for SNodeTransmuteResult<K, V, ReduceT> {
    fn from(other: MapTransmuteResult<K, V, ReduceT>) -> Self {
        match other {
            MapTransmuteResult::Transmuted(key, value, reduced) => SNodeTransmuteResult::S(SNode::new(key, value), reduced),
            MapTransmuteResult::Removed(reduced) => SNodeTransmuteResult::Removed(reduced),
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
