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
            Self::L(lnode) => lnode.find(key),
            Self::S(snode) => snode.find(key),
        }
    }

    pub(crate) fn insert<L: Key + Into<K>, W: Into<V>>(&self, key: L, value: W, flag: Option<Flag<H, F>>, replace: bool) -> InsertResult<H, F, K, V, M>
    where
        K: HashLike<L>,
        K: PartialEq<L>,
        M: HasherBv<H, L>,
    {
        match self {
            Self::C(cnode) => cnode.insert(key, value, flag, replace).into(),
            Self::L(lnode) => lnode::insert(&lnode, key, value, flag, replace).into(),
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

    pub(crate) fn transform<ReduceT, ReduceOp, Op>(&self, reduce_op: ReduceOp, op: Op) -> MNodeTransformResult<H, F, K, V, M, ReduceT>
    where
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        Op: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
        <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
    {
        match self {
            Self::C(cnode) => cnode::transform(cnode, reduce_op, op),
            Self::L(lnode) => lnode::transform(lnode, reduce_op, op).into(),
            Self::S(snode) => snode::transform(snode, op).into(),
        }
    }

    pub(crate) fn transmute<S: Key, X: Value, ReduceT, ReduceOp, Op>(&self, reduce_op: ReduceOp, op: Op) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
    where
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        Op: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        K: HashLike<S>,
        K: PartialEq<S>,
        M: HasherBv<H, S>,
        <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
    {
        match self {
            MNode::C(cnode) => cnode::transmute(cnode, reduce_op, op),
            MNode::L(lnode) => lnode::transmute(lnode, reduce_op, op).into(),
            MNode::S(snode) => snode::transmute(snode, op).into(),
        }
    }

    pub(crate) fn joint_transmute<L: Key, W: Value, S: Key, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(&self, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
    where
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        BothOp: Fn(&K, &V, &L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        LeftOp: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        RightOp: Fn(&L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone,
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
            MNode::C(cnode) => cnode::joint_transmute(cnode, right, reduce_op, both_op, left_op, right_op, depth),
            MNode::L(lnode) => lnode::joint_transmute(lnode, right, reduce_op, both_op, left_op, right_op, depth),
            MNode::S(snode) => snode::joint_transmute(snode, right, reduce_op, both_op, left_op, right_op, depth),
        }
    }

    pub(crate) fn joint_transmute_lnode<L: Key, W: Value, S: Key, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(&self, right: &Arc<LNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
    where
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        BothOp: Fn(&K, &V, &L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        LeftOp: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        RightOp: Fn(&L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone,
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
            MNode::C(cnode) => cnode::joint_transmute_lnode(cnode, right, reduce_op, both_op, left_op, right_op, depth),
            MNode::L(lnode) => lnode::joint_transmute_lnode(lnode, right, reduce_op, both_op, left_op, right_op, depth),
            MNode::S(snode) => lnode::joint_transmute_snode(right, snode, reduce_op, |k,v,l,w| both_op(l, w, k, v), right_op, left_op, depth),
        }
    }

    pub(crate) fn joint_transmute_snode<L: Key, W: Value, S: Key, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(&self, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
    where
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        BothOp: Fn(&K, &V, &L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        LeftOp: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        RightOp: Fn(&L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone,
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
            MNode::C(cnode) => cnode::joint_transmute_snode(cnode, right, reduce_op, both_op, left_op, right_op, depth),
            MNode::L(lnode) => lnode::joint_transmute_snode(lnode, right, reduce_op, both_op, left_op, right_op, depth),
            MNode::S(snode) => snode::joint_transmute_snode(snode, right, reduce_op, both_op, left_op, right_op, depth),
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
        Self::C(CNode::default())
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
