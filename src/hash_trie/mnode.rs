#[allow(unused_imports)]
use super::{cnode::*, flag::*, lnode::{self, LNode}, snode::{self, SNode}, traits::*};
use alloc::{borrow::Cow, fmt::Debug, sync::Arc};

pub(super) enum FindResult<'a, V: Value> {
    NotFound,
    Found(&'a V),
}

pub(super) enum InsertResult<'a, B: Bits, V: Value, H: 'static> {
    Found(&'a V),
    InsertedC(CNode<B, V, H>),
    InsertedL(Arc<LNode<V>>),
}

pub(super) enum RemoveResult<'a, B: Bits, V: Value, H: 'static> {
    NotFound,
    RemovedC(CNode<B, V, H>, &'a V),
    RemovedL(Arc<LNode<V>>, &'a V),
    RemovedS(Arc<SNode<V>>, &'a V),
    RemovedZ(&'a V),
}

#[derive(Debug)]
pub(super) enum MNode <B: Bits, V: Value, H: 'static> {
    C(CNode<B, V, H>),
    L(Arc<LNode<V>>),
    S(Arc<SNode<V>>),
}

impl <B: Bits, V: Value, H: 'static> MNode<B, V, H> {
    pub(super) fn find<'a>(&'a self, value: &V, flag: Option<Flag<B>>) -> FindResult<'a, V> {
        match self {
            Self::C(cnode) => cnode.find(value, flag),
            Self::L(lnode) => lnode.find(value),
            Self::S(snode) => if *snode.get() == *value { FindResult::Found(snode.get()) } else { FindResult::NotFound },
        }
    }
    
    pub(super) fn remove<'a>(&'a self, value: &V, flag: Option<Flag<B>>) -> RemoveResult<'a, B, V, H> {
        match self {
            Self::C(cnode) => cnode.remove(value, flag),
            Self::L(lnode) => lnode.remove(value),
            Self::S(snode) => if *snode.get() == *value { RemoveResult::RemovedZ(snode.get()) } else { RemoveResult::NotFound },
        }
    }
}

impl <B: Bits, V: Value, H: HasherBv<B, V>> MNode<B, V, H> {
    pub(super) fn insert<'a>(&'a self, value: Cow<V>, flag: Option<Flag<B>>) -> InsertResult<'a, B, V, H> {
        match self {
            Self::C(cnode) => cnode.insert(value, flag),
            Self::L(lnode) => lnode::insert(&lnode, value, flag),
            Self::S(snode) => snode::insert(&snode, value, flag),
        }
    }
}

impl <B: Bits, V: Value, H: 'static> Clone for MNode<B, V, H> {
    fn clone(&self) -> Self {
        match self {
            Self::C(this) => Self::C((*this).clone()),
            Self::L(this) => Self::L((*this).clone()),
            Self::S(this) => Self::S((*this).clone()),
        }
    }
}

#[allow(unused_macros)]
macro_rules! assert_found_eq {
    ( $found:expr, $expected:expr ) => {
        match $found {
            FindResult::Found(reference) => assert_eq!(*reference, $expected),
            FindResult::NotFound => panic!()
        }
    };
}

#[allow(unused_macros)]
macro_rules! assert_found_none {
    ( $found:expr ) => {
        match $found {
            FindResult::Found(_reference) => panic!(),
            FindResult::NotFound => {}
        }
    };
}
