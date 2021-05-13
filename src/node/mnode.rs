use crate::{flag::*, result::*, traits::*};
use super::{cnode::*, lnode::{self, LNode}, snode::{self, SNode}};
use alloc::{borrow::Cow, fmt::Debug, sync::Arc};

#[derive(Debug)]
pub(crate) enum MNode <B: Bits, V: Value, H: 'static> {
    C(CNode<B, V, H>),
    L(Arc<LNode<V>>),
    S(Arc<SNode<V>>),
}

impl <B: Bits, V: Value, H: 'static> MNode<B, V, H> {
    pub(crate) fn find<'a>(&'a self, value: &V, flag: Option<Flag<B>>) -> FindResult<'a, V> {
        match self {
            Self::C(cnode) => cnode.find(value, flag),
            Self::L(lnode) => lnode.find(value),
            Self::S(snode) => if *snode.get() == *value { FindResult::Found(snode.get()) } else { FindResult::NotFound },
        }
    }
    
    pub(crate) fn remove<'a>(&'a self, value: &V, flag: Option<Flag<B>>) -> RemoveResult<'a, B, V, H> {
        match self {
            Self::C(cnode) => cnode.remove(value, flag),
            Self::L(lnode) => lnode.remove(value),
            Self::S(snode) => if *snode.get() == *value { RemoveResult::RemovedZ(snode.get()) } else { RemoveResult::NotFound },
        }
    }
}

impl <B: Bits, V: Value, H: HasherBv<B, V>> MNode<B, V, H> {
    pub(crate) fn insert<'a>(&'a self, value: Cow<V>, flag: Option<Flag<B>>) -> InsertResult<'a, B, V, H> {
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

impl <B: Bits, V: Value, H: 'static> Default for MNode<B, V, H> {
    fn default() -> Self {
        Self::C(CNode::<B, V, H>::default())
    }
}
