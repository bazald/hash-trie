use crate::{flag::*, transformations::*, results::*, traits::*, ParallelismStrategy};
use super::{cnode::{self}, lnode::{self, *}, mnode::*};
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

    pub(super) fn find<'a, L: Key>(&'a self, key: &L) -> FindResult<'a, K, V> where K: PartialEq<L> {
        if self.key == *key {
            FindResult::Found(&self.key, &self.value)
        }
        else {
            FindResult::NotFound
        }
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
            let key: *const K = snode.key();
            let value: *const V = snode.value();
            InsertResult::InsertedS(snode, key, value, Some((this.key(), this.value())))
        }
        else {
            InsertResult::Found(this.key(), this.value())
        }
    }
    else {
        lnode::lift_to_cnode_and_insert(LNodeNext::S(this.clone()), key, value, key_flag.unwrap()).into()
    }
}

pub(super) fn remove<'a, K: Key, V: Value, L: Key>(this: &'a Arc<SNode<K, V>>, key: &L) -> SNodeRemoveResult<'a, K, V> where K: PartialEq<L> {
    if this.key == *key {
        SNodeRemoveResult::RemovedZ(this.key(), this.value())
    }
    else {
        SNodeRemoveResult::NotFound
    }
}

pub(super) fn transform<K: Key, V: Value, ReduceT, Op>(this: &Arc<SNode<K, V>>, op: MapTransform<ReduceT, Op>) -> SNodeTransformResult<K, V, ReduceT>
    where
    ReduceT: Clone + Default + Send + Sync,
    Op: Fn(&K, &V) -> MapTransformResult<V, ReduceT>,
{
    match op.call(&this.key, &this.value) {
        MapTransformResult::Unchanged(reduced) => SNodeTransformResult::Unchanged(reduced),
        MapTransformResult::Transformed(value, reduced) => SNodeTransformResult::S(SNode::new(this.key.clone(), value), reduced),
        MapTransformResult::Removed(reduced) => SNodeTransformResult::Removed(reduced),
    }
}

pub(super) unsafe fn transmute<K: Key, V: Value, S: Key, X: Value, ReduceT, Op>(this: &Arc<SNode<K, V>>, op: MapTransmute<ReduceT, Op>) -> SNodeTransmuteResult<S, X, ReduceT>
    where
    ReduceT: Clone + Default + Send + Sync,
    Op: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT>,
    K: HashLike<S>,
    K: PartialEq<S>,
{
    op.call(&this.key, &this.value).into()
}

pub(crate) fn transform_with_transformed<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right: &MNode<H, F, K, V, M>, reduce_op: ReduceOp, both_op: MapJointTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>, depth: usize, par_strat: ParallelismStrategy) -> MNodeJointTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    match right {
        MNode::C(cnode) => cnode::transform_with_transformed_snode(cnode, this, reduce_op, both_op.flip(), right_op, left_op, depth, par_strat).flip(),
        MNode::L(lnode) => lnode::transform_with_transformed_snode(lnode, this, reduce_op, both_op.flip(), right_op, left_op, depth).flip(),
        MNode::S(snode) => transform_with_transformed_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) unsafe fn transform_with_transmuted<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>, depth: usize, par_strat: ParallelismStrategy) -> MNodeTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&L, &W) -> MapTransmuteResult<K, V, ReduceT> + Clone + Send + Sync,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    match right {
        MNode::C(cnode) => cnode::transmute_with_transformed_snode(cnode, this, reduce_op, both_op.flip_transform_transmute(), right_op, left_op, depth, par_strat),
        MNode::L(lnode) => lnode::transmute_with_transformed_snode(lnode, this, reduce_op, both_op.flip_transform_transmute(), right_op, left_op, depth),
        MNode::S(snode) => transform_with_transmuted_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) unsafe fn transmute_with_transformed<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>, depth: usize, par_strat: ParallelismStrategy) -> MNodeTransformResult<H, F, L, W, M, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransmuteResult<L, W, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&L, &W) -> MapTransformResult<W, ReduceT> + Clone + Send + Sync,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    match right {
        MNode::C(cnode) => cnode::transform_with_transmuted_snode(cnode, this, reduce_op, both_op.flip_transmute_transform(), right_op, left_op, depth, par_strat),
        MNode::L(lnode) => lnode::transform_with_transmuted_snode(lnode, this, reduce_op, both_op.flip_transmute_transform(), right_op, left_op, depth),
        MNode::S(snode) => transmute_with_transformed_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) fn transform_with_transformed_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right: &Arc<SNode<K, V>>, reduce_op: ReduceOp, both_op: MapJointTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>, depth: usize) -> MNodeJointTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let this_hash = M::default().hash(this.key());
    let right_hash = M::default().hash(right.key());

    if this_hash == right_hash {
        transform_with_transformed_snode_impl(this, right, reduce_op, both_op, left_op, right_op).into()
    }
    else {
        let left = transform(this, left_op);
        let r = transform(right, right_op);

        let left_flag = Flag::new_at_depth(this_hash, depth).unwrap();
        let right_flag = Flag::new_at_depth(right_hash, depth).unwrap();

        match left {
            SNodeTransformResult::Unchanged(lr) => match r {
                SNodeTransformResult::Unchanged(rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(MNode::S(this.clone()), left_flag, right.clone().into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::S(rsnode, rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(MNode::S(this.clone()), left_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeJointTransformResult::UnchangedL(reduce_op(&lr, &rr)),
            },
            SNodeTransformResult::S(lsnode, lr) => match r {
                SNodeTransformResult::Unchanged(rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), left_flag, right.clone().into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::S(rsnode, rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), left_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeJointTransformResult::S(lsnode, reduce_op(&lr, &rr)),
            },
            SNodeTransformResult::Removed(lr) => match r {
                SNodeTransformResult::Unchanged(rr) => MNodeJointTransformResult::UnchangedR(reduce_op(&lr, &rr)),
                SNodeTransformResult::S(rsnode, rr) => MNodeJointTransformResult::S(rsnode, reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeJointTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

pub(crate) fn transform_with_transformed_snode_impl<K: Key, V: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right: &Arc<SNode<K, V>>, reduce_op: ReduceOp, both_op: MapJointTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>) -> LNodeJointTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
{
    if *this.key() == *right.key() {
        match both_op.call(this.key(), this.value(), right.key(), right.value()) {
            MapJointTransformResult::UnchangedLR(reduced) => LNodeJointTransformResult::UnchangedLR(reduced),
            MapJointTransformResult::UnchangedL(reduced) => LNodeJointTransformResult::UnchangedL(reduced),
            MapJointTransformResult::UnchangedR(reduced) => LNodeJointTransformResult::UnchangedR(reduced),
            MapJointTransformResult::Transformed(value, reduced) => LNodeJointTransformResult::S(SNode::new(this.key().clone(), value), reduced),
            MapJointTransformResult::Removed(reduced) => LNodeJointTransformResult::Removed(reduced),
        }
    }
    else {
        let left = left_op.call(this.key(), this.value());
        let r = right_op.call(right.key(), right.value());

        match left {
            MapTransformResult::Unchanged(lr) => match r {
                MapTransformResult::Unchanged(rr) => LNodeJointTransformResult::L(LNode::new(right.key.clone(), right.value.clone(), LNodeNext::S(this.clone())), reduce_op(&lr, &rr)),
                MapTransformResult::Transformed(rv, rr) => LNodeJointTransformResult::L(LNode::new(right.key().clone(), rv, LNodeNext::S(this.clone())), reduce_op(&lr, &rr)),
                MapTransformResult::Removed(rr) => LNodeJointTransformResult::UnchangedL(reduce_op(&lr, &rr)),
            },
            MapTransformResult::Transformed(lv, lr) => match r {
                MapTransformResult::Unchanged(rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), lv, LNodeNext::S(right.clone())), reduce_op(&lr, &rr)),
                MapTransformResult::Transformed(rv, rr) => LNodeJointTransformResult::L(LNode::new(right.key.clone(), rv, LNodeNext::S(SNode::new(this.key().clone(), lv))), reduce_op(&lr, &rr)),
                MapTransformResult::Removed(rr) => LNodeJointTransformResult::S(SNode::new(this.key().clone(), lv), reduce_op(&lr, &rr)),
            },
            MapTransformResult::Removed(lr) => match r {
                MapTransformResult::Unchanged(rr) => LNodeJointTransformResult::UnchangedR(reduce_op(&lr, &rr)),
                MapTransformResult::Transformed(rv, rr) => LNodeJointTransformResult::S(SNode::new(right.key.clone(), rv), reduce_op(&lr, &rr)),
                MapTransformResult::Removed(rr) => LNodeJointTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

pub(crate) unsafe fn transform_with_transmuted_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>, depth: usize) -> MNodeTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&L, &W) -> MapTransmuteResult<K, V, ReduceT> + Clone + Send + Sync,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let this_hash = M::default().hash(this.key());
    let right_hash = M::default().hash(right.key());

    if this_hash == right_hash {
        transform_with_transmuted_snode_impl(this, right.key(), right.value(), reduce_op, both_op, left_op, right_op).into()
    }
    else {
        let left = transform(this, left_op);
        let right = transmute(right, right_op);

        let left_flag = Flag::new_at_depth(this_hash, depth).unwrap();
        let right_flag = Flag::new_at_depth(right_hash, depth).unwrap();

        match left {
            SNodeTransformResult::Unchanged(lr) => match right {
                SNodeTransmuteResult::S(rsnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(MNode::S(this.clone()), left_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransmuteResult::Removed(rr) => MNodeTransformResult::Unchanged(reduce_op(&lr, &rr)),
            },
            SNodeTransformResult::S(lsnode, lr) => match right {
                SNodeTransmuteResult::S(rsnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), left_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransmuteResult::Removed(rr) => MNodeTransformResult::S(lsnode, reduce_op(&lr, &rr)),
            },
            SNodeTransformResult::Removed(lr) => match right {
                SNodeTransmuteResult::S(rsnode, rr) => MNodeTransformResult::S(rsnode, reduce_op(&lr, &rr)),
                SNodeTransmuteResult::Removed(rr) => MNodeTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

pub(crate) unsafe fn transform_with_transmuted_snode_impl<K: Key, V: Value, L: Key, W: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right_key: &L, right_value: &W, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>) -> LNodeTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&L, &W) -> MapTransmuteResult<K, V, ReduceT> + Clone + Send + Sync,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
{
    if *this.key() == *right_key {
        match both_op.call_transform_transmute(this.key(), this.value(), right_key, right_value) {
            MapTransformResult::Unchanged(reduced) => LNodeTransformResult::Unchanged(reduced),
            MapTransformResult::Transformed(value, reduced) => LNodeTransformResult::S(SNode::new(this.key().clone(), value), reduced),
            MapTransformResult::Removed(reduced) => LNodeTransformResult::Removed(reduced),
        }
    }
    else {
        let left = left_op.call(this.key(), this.value());
        let right = right_op.call(right_key, right_value);

        match left {
            MapTransformResult::Unchanged(lr) => match right {
                MapTransmuteResult::Transmuted(rk, rv, rr) => LNodeTransformResult::L(LNode::new(rk, rv, LNodeNext::S(this.clone())), reduce_op(&lr, &rr)),
                MapTransmuteResult::Removed(rr) => LNodeTransformResult::Unchanged(reduce_op(&lr, &rr)),
            },
            MapTransformResult::Transformed(lv, lr) => match right {
                MapTransmuteResult::Transmuted(rk, rv, rr) => LNodeTransformResult::L(LNode::new(rk, rv, LNodeNext::S(SNode::new(this.key().clone(), lv))), reduce_op(&lr, &rr)),
                MapTransmuteResult::Removed(rr) => LNodeTransformResult::S(SNode::new(this.key().clone(), lv), reduce_op(&lr, &rr)),
            },
            MapTransformResult::Removed(lr) => match right {
                MapTransmuteResult::Transmuted(rk, rv, rr) => LNodeTransformResult::S(SNode::new(rk, rv), reduce_op(&lr, &rr)),
                MapTransmuteResult::Removed(rr) => LNodeTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

pub(crate) unsafe fn transmute_with_transformed_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>, depth: usize) -> MNodeTransformResult<H, F, L, W, M, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransmuteResult<L, W, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&L, &W) -> MapTransformResult<W, ReduceT> + Clone + Send + Sync,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let this_hash = M::default().hash(this.key());
    let right_hash = M::default().hash(right.key());

    if this_hash == right_hash {
        transmute_with_transformed_snode_impl(this, right, reduce_op, both_op, left_op, right_op).into()
    }
    else {
        let left = transmute(this, left_op);
        let r = transform(right, right_op);

        let left_flag = Flag::new_at_depth(this_hash, depth).unwrap();
        let right_flag = Flag::new_at_depth(right_hash, depth).unwrap();

        match left {
            SNodeTransmuteResult::S(lsnode, lr) => match r {
                SNodeTransformResult::Unchanged(rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), left_flag, right.clone().into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::S(rsnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), left_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeTransformResult::S(lsnode, reduce_op(&lr, &rr)),
            },
            SNodeTransmuteResult::Removed(lr) => match r {
                SNodeTransformResult::Unchanged(rr) => MNodeTransformResult::Unchanged(reduce_op(&lr, &rr)),
                SNodeTransformResult::S(rsnode, rr) => MNodeTransformResult::S(rsnode, reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

pub(crate) unsafe fn transmute_with_transformed_snode_impl<K: Key, V: Value, L: Key, W: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>) -> LNodeTransformResult<L, W, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransmuteResult<L, W, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&L, &W) -> MapTransformResult<W, ReduceT> + Clone + Send + Sync,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
{
    if *this.key() == *right.key() {
        match both_op.call_transmute_transform(this.key(), this.value(), right.key(), right.value()) {
            MapTransformResult::Unchanged(reduced) => LNodeTransformResult::Unchanged(reduced),
            MapTransformResult::Transformed(value, reduced) => LNodeTransformResult::S(SNode::new(right.key().clone(), value), reduced),
            MapTransformResult::Removed(reduced) => LNodeTransformResult::Removed(reduced),
        }
    }
    else {
        let left = left_op.call(this.key(), this.value());
        let r = right_op.call(right.key(), right.value());

        match left {
            MapTransmuteResult::Transmuted(lk, lv, lr) => match r {
                MapTransformResult::Unchanged(rr) => LNodeTransformResult::L(LNode::new(lk, lv, LNodeNext::S(right.clone())), reduce_op(&lr, &rr)),
                MapTransformResult::Transformed(rv, rr) => LNodeTransformResult::L(LNode::new(lk, lv, LNodeNext::S(SNode::new(right.key().clone(), rv))), reduce_op(&lr, &rr)),
                MapTransformResult::Removed(rr) => LNodeTransformResult::S(SNode::new(lk, lv), reduce_op(&lr, &rr)),
            },
            MapTransmuteResult::Removed(lr) => match r {
                MapTransformResult::Unchanged(rr) => LNodeTransformResult::Unchanged(reduce_op(&lr, &rr)),
                MapTransformResult::Transformed(rv, rr) => LNodeTransformResult::S(SNode::new(right.key().clone(), rv), reduce_op(&lr, &rr)),
                MapTransformResult::Removed(rr) => LNodeTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

pub(crate) unsafe fn transmute_with_transmuted<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, S: Key, W: Value, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: MapTransmute<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>, depth: usize) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
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
    match right {
        MNode::C(cnode) => cnode::transmute_with_transmuted_snode(cnode, this, reduce_op, both_op.flip(), right_op, left_op, depth),
        MNode::L(lnode) => lnode::transmute_with_transmuted_snode(lnode, this, reduce_op, both_op.flip(), right_op, left_op, depth),
        MNode::S(snode) => transmute_with_transmuted_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) unsafe fn transmute_with_transmuted_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, S: Key, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<SNode<K, V>>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: MapTransmute<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>, depth: usize) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
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
    let this_hash = M::default().hash(this.key());
    let right_hash = M::default().hash(right.key());

    if this_hash == right_hash {
        transmute_with_transmuted_values(&this.key, &this.value, &right.key, &right.value, reduce_op, both_op, left_op, right_op).into()
    }
    else {
        let this = transmute(this, left_op);
        let right = transmute(right, right_op);

        let this_flag = Flag::new_at_depth(this_hash, depth).unwrap();
        let right_flag = Flag::new_at_depth(right_hash, depth).unwrap();

        match this {
            SNodeTransmuteResult::S(lsnode, lr) => match right {
                SNodeTransmuteResult::S(rsnode, rr) => MNodeTransmuteResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransmuteResult::Removed(rr) => MNodeTransmuteResult::S(lsnode, reduce_op(&lr, &rr)),
            },
            SNodeTransmuteResult::Removed(lr) => match right {
                SNodeTransmuteResult::S(rsnode, rr) => MNodeTransmuteResult::S(rsnode, reduce_op(&lr, &rr)),
                SNodeTransmuteResult::Removed(rr) => MNodeTransmuteResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

pub(crate) unsafe fn transmute_with_transmuted_values<K: Key, V: Value, L: Key, W: Value, S: Key, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this_key: &K, this_value: &V, right_key: &L, right_value: &W, reduce_op: ReduceOp, both_op: MapTransmute<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>) -> LNodeTransmuteResult<S, X, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
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
{
    if this_key == right_key {
        both_op.call_transmute(this_key, this_value, right_key, right_value).into()
    }
    else {
        let lr = left_op.call(this_key, this_value);
        let rr = right_op.call(right_key, right_value);

        match lr {
            MapTransmuteResult::Transmuted(lk, lv, lr) => match rr {
                MapTransmuteResult::Transmuted(rk, rv, rr) => LNodeTransmuteResult::L(LNode::new(lk, lv, LNodeNext::S(SNode::new(rk, rv))), reduce_op(&lr, &rr)),
                MapTransmuteResult::Removed(rr) => LNodeTransmuteResult::S(SNode::new(lk, lv), reduce_op(&lr, &rr)),
            },
            MapTransmuteResult::Removed(lr) => match rr {
                MapTransmuteResult::Transmuted(rk, rv, rr) => LNodeTransmuteResult::S(SNode::new(rk, rv), reduce_op(&lr, &rr)),
                MapTransmuteResult::Removed(rr) => LNodeTransmuteResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> From<Arc<SNode<K, V>>> for MNode<H, F, K, V, M> {
    fn from(other: Arc<SNode<K, V>>) -> Self {
        MNode::S(other)
    }
}
