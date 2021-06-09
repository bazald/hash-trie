use crate::{flag::*, result::*, traits::*};
use super::{cnode::{self, *}, lnode::{self, LNode}, snode::{self, SNode}};
use alloc::{fmt::Debug, sync::Arc};

#[derive(Debug)]
pub(crate) enum MNode <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> {
    C(CNode<H, F, K, V, M>),
    L(Arc<LNode<K, V>>),
    S(Arc<SNode<K, V>>),
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> MNode<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    #[must_use]
    pub(crate) fn size(&self) -> usize {
        match self {
            Self::C(cnode) => cnode.size(),
            Self::L(lnode) => lnode.size(),
            Self::S(_snode) => 1,
        }
    }

    pub(crate) fn find<L: Key>(&self, key: &L, flag: Option<Flag<H, F>>) -> FindResult<K, V>
    where
        K: PartialEq<L>
    {
        match self {
            Self::C(cnode) => cnode.find(key, flag),
            Self::L(lnode) => lnode::find(lnode, key),
            Self::S(snode) => snode::find(snode, key),
        }
    }

    pub(crate) fn insert<L: Key + Into<K>, W: Into<V>>(&self, key: L, value: W, flag: Option<Flag<H, F>>, replace: bool) -> InsertResult<H, F, K, V, M>
    where
        K: HashLike<L>,
        K: PartialEq<L>,
        M: HasherBv<H, L>,
    {
        match self {
            Self::C(cnode) => cnode.insert(key, value, flag, replace),
            Self::L(lnode) => lnode::insert(&lnode, key, value, flag, replace),
            Self::S(snode) => snode::insert(&snode, key, value, flag, replace),
        }
    }

    pub(crate) fn remove<L: Key>(&self, key: &L, flag: Option<Flag<H, F>>) -> RemoveResult<H, F, K, V, M> where K: PartialEq<L>, M: HasherBv<H, L> {
        match self {
            Self::C(cnode) => cnode.remove(key, flag),
            Self::L(lnode) => lnode::remove(lnode, key),
            Self::S(snode) => snode::remove(snode, key).into(),
        }
    }
    
    pub(crate) fn visit<Op: Clone>(&self, op: Op) where Op: Fn(&K, &V) {
        match self {
            Self::C(cnode) => cnode.visit(op),
            Self::L(lnode) => lnode.visit(op),
            Self::S(snode) => snode.visit(op),
        }
    }

    pub(crate) fn transform<S: Key, X: Value, ReduceT, ReduceOp, Op>(&self, reduce_op: ReduceOp, op: Op) -> MNodeTransformResult<H, F, S, X, M, ReduceT>
    where
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        Op: Fn(&K, &V) -> MapTransformResult<S, X, ReduceT> + Clone,
        K: HashLike<S>,
        K: PartialEq<S>,
        M: HasherBv<H, S>,
        <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
    {
        match self {
            MNode::<H, F, K, V, M>::C(cnode) => cnode::transform(cnode, reduce_op, op),
            MNode::<H, F, K, V, M>::L(lnode) => lnode::transform(lnode, reduce_op, op).into(),
            MNode::<H, F, K, V, M>::S(snode) => snode::transform(snode, op).into(),
        }
    }

    pub(crate) fn joint_transform<L: Key, W: Value, S: Key, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(&self, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, S, X, M, ReduceT>
    where
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<S, X, ReduceT> + Clone,
        LeftOp: Fn(&K, &V) -> MapTransformResult<S, X, ReduceT> + Clone,
        RightOp: Fn(&L, &W) -> MapTransformResult<S, X, ReduceT> + Clone,
        K: HashLike<L>,
        K: PartialEq<L>,
        K: HashLike<S>,
        K: PartialEq<S>,
        L: HashLike<K>,
        L: PartialEq<K>,
        L: HashLike<S>,
        L: PartialEq<S>,
        M: HasherBv<H, L>,
        M: HasherBv<H, S>,
        <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
    {
        match self {
            MNode::<H, F, K, V, M>::C(cnode) => cnode::joint_transform(cnode, right, reduce_op, both_op, left_op, right_op, depth),
            MNode::<H, F, K, V, M>::L(lnode) => lnode::joint_transform(lnode, right, reduce_op, both_op, left_op, right_op, depth),
            MNode::<H, F, K, V, M>::S(snode) => snode::joint_transform(snode, right, reduce_op, both_op, left_op, right_op, depth),
        }
    }

    pub(crate) fn joint_transform_lnode<L: Key, W: Value, S: Key, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(&self, right: &Arc<LNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, S, X, M, ReduceT>
    where
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<S, X, ReduceT> + Clone,
        LeftOp: Fn(&K, &V) -> MapTransformResult<S, X, ReduceT> + Clone,
        RightOp: Fn(&L, &W) -> MapTransformResult<S, X, ReduceT> + Clone,
        K: HashLike<L>,
        K: PartialEq<L>,
        K: HashLike<S>,
        K: PartialEq<S>,
        L: HashLike<K>,
        L: PartialEq<K>,
        L: HashLike<S>,
        L: PartialEq<S>,
        M: HasherBv<H, L>,
        M: HasherBv<H, S>,
        <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
    {
        match self {
            MNode::<H, F, K, V, M>::C(cnode) => cnode::joint_transform_lnode(cnode, right, reduce_op, both_op, left_op, right_op, depth),
            MNode::<H, F, K, V, M>::L(lnode) => lnode::joint_transform_lnode(lnode, right, reduce_op, both_op, left_op, right_op, depth),
            MNode::<H, F, K, V, M>::S(snode) => lnode::joint_transform_snode(right, snode, reduce_op, |k,v,l,w| both_op(l, w, k, v), right_op, left_op, depth),
        }
    }

    pub(crate) fn joint_transform_snode<L: Key, W: Value, S: Key, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(&self, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, S, X, M, ReduceT>
    where
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<S, X, ReduceT> + Clone,
        LeftOp: Fn(&K, &V) -> MapTransformResult<S, X, ReduceT> + Clone,
        RightOp: Fn(&L, &W) -> MapTransformResult<S, X, ReduceT> + Clone,
        K: HashLike<L>,
        K: PartialEq<L>,
        K: HashLike<S>,
        K: PartialEq<S>,
        L: HashLike<K>,
        L: PartialEq<K>,
        L: HashLike<S>,
        L: PartialEq<S>,
        M: HasherBv<H, L>,
        M: HasherBv<H, S>,
        <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
    {
        match self {
            MNode::<H, F, K, V, M>::C(cnode) => cnode::joint_transform_snode(cnode, right, reduce_op, both_op, left_op, right_op, depth),
            MNode::<H, F, K, V, M>::L(lnode) => lnode::joint_transform_snode(lnode, right, reduce_op, both_op, left_op, right_op, depth),
            MNode::<H, F, K, V, M>::S(snode) => snode::joint_transform_snode(snode, right, reduce_op, both_op, left_op, right_op, depth),
        }
    }

}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> Clone for MNode<H, F, K, V, M> {
    fn clone(&self) -> Self {
        match self {
            Self::C(this) => Self::C((*this).clone()),
            Self::L(this) => Self::L((*this).clone()),
            Self::S(this) => Self::S((*this).clone()),
        }
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> Default for MNode<H, F, K, V, M> {
    fn default() -> Self {
        Self::C(CNode::<H, F, K, V, M>::default())
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> Eq for MNode<H, F, K, V, M> {}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> PartialEq for MNode<H, F, K, V, M> {
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
