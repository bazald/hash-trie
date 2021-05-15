use crate::{flag::*, result::*, traits::*};
use super::{cnode::*, lnode::{self, LNode}, snode::{self, SNode}};
use alloc::{fmt::Debug, sync::Arc};

#[derive(Debug)]
pub(crate) enum MNode <B: Bits, V: Value, H: 'static> {
    C(CNode<B, V, H>),
    L(Arc<LNode<V>>),
    S(Arc<SNode<V>>),
}

impl <B: Bits, V: Value, H: 'static> MNode<B, V, H> {
    pub(super) fn size(&self) -> usize {
        match self {
            Self::C(cnode) => cnode.size(),
            Self::L(lnode) => lnode.size(),
            Self::S(_snode) => 1,
        }
    }

    pub(crate) fn find<'a, K>(&'a self, key: &K, flag: Option<Flag<B>>) -> FindResult<'a, V> where V: PartialEq<K> {
        match self {
            Self::C(cnode) => cnode.find(key, flag),
            Self::L(lnode) => lnode.find(key),
            Self::S(snode) => snode.find(key),
        }
    }
    
    pub(crate) fn remove<'a, K>(&'a self, key: &K, flag: Option<Flag<B>>) -> RemoveResult<'a, B, V, H> where V: PartialEq<K> {
        match self {
            Self::C(cnode) => cnode.remove(key, flag),
            Self::L(lnode) => lnode.remove(key),
            Self::S(snode) => snode.remove(key),
        }
    }
}

impl <B: Bits, V: Value, H: HasherBv<B, V>> MNode<B, V, H> {
    pub(crate) fn insert<'a, K: 'static, C: AsRef<K> + Into<V>>(&'a self, value: C, flag: Option<Flag<B>>, replace: bool) -> InsertResult<'a, B, V, H> where V: PartialEq<K> {
        match self {
            Self::C(cnode) => cnode.insert(value, flag, replace),
            Self::L(lnode) => lnode::insert(&lnode, value, flag, replace),
            Self::S(snode) => snode::insert(&snode, value, flag, replace),
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
