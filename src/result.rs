use crate::{node::{CNode, LNode, SNode}, traits::*};
use alloc::sync::Arc;

/// `BitError` enumerates possible error conditions when bitops are used "incorrectly."
#[derive(Debug)]
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

pub(crate) enum FindResult<'a, V: Value> {
    NotFound,
    Found(&'a V),
}

pub(crate) enum InsertResult<'a, H: Hashword, F: Flagword<H>, V: Value, M: 'static> {
    Found(&'a V),
    InsertedC(CNode<H, F, V, M>, Option<&'a V>),
    InsertedL(Arc<LNode<V>>, Option<&'a V>),
    InsertedS(Arc<SNode<V>>, Option<&'a V>),
}

pub(crate) enum RemoveResult<'a, H: Hashword, F: Flagword<H>, V: Value, M: 'static> {
    NotFound,
    RemovedC(CNode<H, F, V, M>, &'a V),
    RemovedL(Arc<LNode<V>>, &'a V),
    RemovedS(Arc<SNode<V>>, &'a V),
    RemovedZ(&'a V),
}

pub(crate) enum LNodeRemoveResult<'a, V: Value> {
    NotFound,
    RemovedL(Arc<LNode<V>>, &'a V),
    RemovedS(Arc<SNode<V>>, &'a V),
}

pub(crate) enum SNodeRemoveResult<'a, V: Value> {
    NotFound,
    RemovedZ(&'a V),
}

#[cfg(test)]
macro_rules! assert_found_eq {
    ( $found:expr, $expected:expr ) => {
        match $found {
            FindResult::Found(reference) => assert_eq!(*reference, $expected),
            FindResult::NotFound => panic!()
        }
    };
}

#[cfg(test)]
macro_rules! assert_found_none {
    ( $found:expr ) => {
        match $found {
            FindResult::Found(_reference) => panic!(),
            FindResult::NotFound => {}
        }
    };
}
