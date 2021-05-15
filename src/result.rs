use crate::{node::{CNode, LNode, SNode}, traits::*};
use alloc::sync::Arc;

pub(crate) enum FindResult<'a, V: Value> {
    NotFound,
    Found(&'a V),
}

pub(crate) enum InsertResult<'a, B: Bits, V: Value, H: 'static> {
    Found(&'a V),
    InsertedC(CNode<B, V, H>, Option<&'a V>),
    InsertedL(Arc<LNode<V>>, Option<&'a V>),
    InsertedS(Arc<SNode<V>>, Option<&'a V>),
}

pub(crate) enum RemoveResult<'a, B: Bits, V: Value, H: 'static> {
    NotFound,
    RemovedC(CNode<B, V, H>, &'a V),
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
