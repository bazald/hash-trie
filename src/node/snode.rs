use crate::{flag::*, result::*, traits::*};
use super::{cnode::self, lnode::{self, *}, mnode::*};
use alloc::{fmt::Debug, sync::Arc};

#[derive(Clone, Debug)]
pub(crate) struct SNode<K: Key, V: Value> {
    key: K,
    value: V,
}

impl <K: Key, V: Value> SNode<K, V> {
    #[must_use]
    pub(super) fn key(&self) -> &K {
        &self.key
    }

    #[must_use]
    pub(super) fn value(&self) -> &V {
        &self.value
    }

    #[must_use]
    pub(crate) fn new(key: K, value: V) -> Arc<Self> {
        Arc::new(Self {key, value})
    }

    pub(super) fn visit<Op>(&self, op: Op) where Op: Fn(&K, &V) {
        op(&self.key, &self.value);
    }

}

pub(super) fn find<K: Key, V: Value, L: Key>(this: &Arc<SNode<K, V>>, key: &L) -> FindResult<K, V> where K: PartialEq<L> {
    if this.key == *key {
        FindResult::Found(this.clone().into())
    }
    else {
        FindResult::NotFound
    }
}

pub(super) fn insert<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key + Into<K>, W: Into<V>, M: HasherBv<H, K>>(this: &Arc<SNode<K, V>>, key: L, value: W, key_flag: Option<Flag<H, F>>, replace: bool) -> InsertResult<H, F, K, V, M>
where
    K: HashLike<L>,
    K: PartialEq<L>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    if this.key == key {
        if replace {
            let snode = SNode::new(key.into(), value.into());
            InsertResult::InsertedS(snode.clone(), snode.into())
        }
        else {
            InsertResult::Found(this.clone().into())
        }
    }
    else {
        lnode::lift_to_cnode_and_insert(LNodeNext::S(this.clone()), key, value, key_flag.unwrap())
    }
}

pub(super) fn remove<K: Key, V: Value, L: Key>(this: &Arc<SNode<K, V>>, key: &L) -> SNodeRemoveResult<K, V> where K: PartialEq<L> {
    if this.key == *key {
        SNodeRemoveResult::RemovedZ(this.clone().into())
    }
    else {
        SNodeRemoveResult::NotFound
    }
}

pub(super) fn transform<K: Key, V: Value, S: Key, X: Value, ReduceT, Op>(this: &Arc<SNode<K, V>>, op: Op) -> SNodeTransformResult<S, X, ReduceT>
    where
    ReduceT: Default,
    Op: Fn(&K, &V) -> MapTransformResult<S, X, ReduceT>,
    K: HashLike<S>,
    K: PartialEq<S>,
{
    op(&this.key, &this.value).into()
}

pub(crate) fn joint_transform<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, S: Key, W: Value, X: Value, M: HasherBv<H, K> + HasherBv<H, L>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, S, X, M, ReduceT>
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
    match right {
        MNode::<H, F, L, W, M>::C(cnode) => cnode::joint_transform_snode(cnode, this, |a,b| reduce_op(b, a), |k,v,l,w| both_op(l, w, k, v), right_op, left_op, depth),
        MNode::<H, F, L, W, M>::L(lnode) => lnode::joint_transform_snode(lnode, this, |a,b| reduce_op(b, a), |k,v,l,w| both_op(l, w, k, v), right_op, left_op, depth),
        MNode::<H, F, L, W, M>::S(snode) => joint_transform_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) fn joint_transform_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, S: Key, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, S, X, M, ReduceT>
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
    let this_hash = M::default().hash(this.key());
    let right_hash = M::default().hash(right.key());

    if this_hash == right_hash {
        joint_transform_values(&this.key, &this.value, &right.key, &right.value, reduce_op, both_op, left_op, right_op).into()
    }
    else {
        let this = transform(this, left_op);
        let right = transform(right, right_op);

        let this_flag = Flag::new_at_depth(this_hash, depth).unwrap();
        let right_flag = Flag::new_at_depth(right_hash, depth).unwrap();

        match this {
            SNodeTransformResult::S(lsnode, lr) => match right {
                SNodeTransformResult::S(rsnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeTransformResult::S(lsnode, reduce_op(&lr, &rr)),
            },
            SNodeTransformResult::Removed(lr) => match right {
                SNodeTransformResult::S(rsnode, rr) => MNodeTransformResult::S(rsnode, reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

pub(crate) fn joint_transform_values<K: Key, V: Value, L: Key, W: Value, S: Key, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this_key: &K, this_value: &V, right_key: &L, right_value: &W, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp) -> LNodeTransformResult<S, X, ReduceT>
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
{
    if this_key == right_key {
        both_op(this_key, this_value, right_key, right_value).into()
    }
    else {
        let lr = left_op(this_key, this_value);
        let rr = right_op(right_key, right_value);

        match lr {
            MapTransformResult::Transformed(lk, lv, lr) => match rr {
                MapTransformResult::Transformed(rk, rv, rr) => LNodeTransformResult::L(LNode::new(lk, lv, LNodeNext::S(SNode::new(rk, rv))), reduce_op(&lr, &rr)),
                MapTransformResult::Removed(rr) => LNodeTransformResult::S(SNode::new(lk.clone(), lv), reduce_op(&lr, &rr)),
            },
            MapTransformResult::Removed(lr) => match rr {
                MapTransformResult::Transformed(rk, rv, rr) => LNodeTransformResult::S(SNode::new(rk, rv), reduce_op(&lr, &rr)),
                MapTransformResult::Removed(rr) => LNodeTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> From<Arc<SNode<K, V>>> for MNode<H, F, K, V, M> {
    fn from(other: Arc<SNode<K, V>>) -> Self {
        MNode::S(other)
    }
}
