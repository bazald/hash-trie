use crate::{flag::*, result::*, traits::*};
use super::{cnode::*, lnode::{self, LNode}, snode::{self, SNode}};
use alloc::{fmt::Debug, sync::Arc};

#[derive(Debug)]
pub(crate) enum MNode <H: Hashword, F: Flagword<H>, V: Value, M: 'static> {
    C(CNode<H, F, V, M>),
    L(Arc<LNode<V>>),
    S(Arc<SNode<V>>),
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: 'static> MNode<H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    #[must_use]
    pub(super) fn size(&self) -> usize {
        match self {
            Self::C(cnode) => cnode.size(),
            Self::L(lnode) => lnode.size(),
            Self::S(_snode) => 1,
        }
    }

    pub(crate) fn find<'a, K>(&'a self, key: &K, flag: Option<Flag<H, F>>) -> FindResult<'a, V> where V: PartialEq<K> {
        match self {
            Self::C(cnode) => cnode.find(key, flag),
            Self::L(lnode) => lnode.find(key),
            Self::S(snode) => snode.find(key),
        }
    }
    
    pub(crate) fn remove<'a, K>(&'a self, key: &K, flag: Option<Flag<H, F>>) -> RemoveResult<'a, H, F, V, M> where V: PartialEq<K> {
        match self {
            Self::C(cnode) => cnode.remove(key, flag),
            Self::L(lnode) => lnode.remove(key),
            Self::S(snode) => snode.remove(key),
        }
    }
    
    pub(crate) fn visit<Op: Clone>(&self, op: Op) where Op: Fn(&'_ V) {
        match self {
            Self::C(cnode) => cnode.visit(op),
            Self::L(lnode) => lnode.visit(op),
            Self::S(snode) => snode.visit(op),
        }
    }

    pub(crate) fn transform<ReduceT, ReduceOp, Op>
        (&self, reduce_op: ReduceOp, op: Op) -> TransformResult<H, F, V, M, ReduceT>
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(ReduceT, ReduceT) -> ReduceT + Clone,
        Op: Fn(&V) -> (Option<V>, ReduceT) + Clone
    {
        match self {
            Self::C(cnode) => cnode.transform(reduce_op, op),
            Self::L(lnode) => match lnode.transform(reduce_op, op) {
                LNodeTransformResult::L(lnode, reduced) => TransformResult::L(lnode, reduced),
                LNodeTransformResult::S(snode, reduced) => TransformResult::S(snode, reduced),
                LNodeTransformResult::Z(reduced) => TransformResult::Z(reduced),
            },
            Self::S(snode) => match snode.transform(op) {
                SNodeTransformResult::S(snode, reduced) => TransformResult::S(snode, reduced),
                SNodeTransformResult::Z(reduced) => TransformResult::Z(reduced),
            },
        }
    }

}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> MNode<H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    pub(crate) fn insert<'a, K: 'static, C: AsRef<K> + Into<V>>(&'a self, value: C, flag: Option<Flag<H, F>>, replace: bool) -> InsertResult<'a, H, F, V, M> where V: PartialEq<K> {
        match self {
            Self::C(cnode) => cnode.insert(value, flag, replace),
            Self::L(lnode) => lnode::insert(&lnode, value, flag, replace),
            Self::S(snode) => snode::insert(&snode, value, flag, replace),
        }
    }
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: 'static> Clone for MNode<H, F, V, M> {
    fn clone(&self) -> Self {
        match self {
            Self::C(this) => Self::C((*this).clone()),
            Self::L(this) => Self::L((*this).clone()),
            Self::S(this) => Self::S((*this).clone()),
        }
    }
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: 'static> Default for MNode<H, F, V, M> {
    fn default() -> Self {
        Self::C(CNode::<H, F, V, M>::default())
    }
}
