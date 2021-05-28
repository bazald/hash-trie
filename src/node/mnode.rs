use crate::{flag::*, result::*, traits::*};
use super::{cnode::{self, *}, lnode::{self, LNode}, snode::{self, SNode}};
use alloc::{fmt::Debug, sync::Arc};

#[derive(Debug)]
pub(crate) enum MNode <H: Hashword, F: Flagword<H>, V: Value, M: 'static> {
    C(CNode<H, F, V, M>),
    L(Arc<LNode<V>>),
    S(Arc<SNode<V>>),
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> MNode<H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    #[must_use]
    pub(crate) fn size(&self) -> usize {
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
    
    pub(crate) fn visit<Op: Clone>(&self, op: Op) where Op: Fn(&V) {
        match self {
            Self::C(cnode) => cnode.visit(op),
            Self::L(lnode) => lnode.visit(op),
            Self::S(snode) => snode.visit(op),
        }
    }

    pub(crate) fn transform<ReduceT, ReduceOp, Op>(&self, reduce_op: ReduceOp, op: Op) -> MNodeTransformResult<H, F, V, M, ReduceT>
    where
        ReduceT: Default,
        ReduceOp: Fn(ReduceT, ReduceT) -> ReduceT + Clone,
        Op: Fn(&V) -> (MapTransformResult<V>, ReduceT) + Clone,
        <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
    {
        match self {
            MNode::<H, F, V, M>::C(cnode) => cnode::transform(cnode, reduce_op, op),
            MNode::<H, F, V, M>::L(lnode) => match lnode::transform(lnode, reduce_op, op) {
                LNodeTransformResult::Unchanged(reduced) => MNodeTransformResult::Unchanged(reduced),
                LNodeTransformResult::L(lnode, reduced) => MNodeTransformResult::L(lnode, reduced),
                LNodeTransformResult::S(snode, reduced) => MNodeTransformResult::S(snode, reduced),
                LNodeTransformResult::Removed(reduced) => MNodeTransformResult::Removed(reduced),
            },
            MNode::<H, F, V, M>::S(snode) => match snode::transform(snode, op) {
                SNodeTransformResult::Unchanged(reduced) => MNodeTransformResult::Unchanged(reduced),
                SNodeTransformResult::S(snode, reduced) => MNodeTransformResult::S(snode, reduced),
                SNodeTransformResult::Removed(reduced) => MNodeTransformResult::Removed(reduced),
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

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> Default for MNode<H, F, V, M> {
    fn default() -> Self {
        Self::C(CNode::<H, F, V, M>::default())
    }
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> Eq for MNode<H, F, V, M> {}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> PartialEq for MNode<H, F, V, M> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            MNode::C(cnode) => if let MNode::C(other) = other {
                cnode == other
            }
            else {
                false
            },
            MNode::L(lnode) => if let MNode::L(other) = other {
                Arc::ptr_eq(&lnode, &other)
            }
            else {
                false
            },
            MNode::S(snode) => if let MNode::S(other) = other {
                Arc::ptr_eq(&snode, &other)
            }
            else {
                false
            },
        }
    }
}
