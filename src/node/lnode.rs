use crate::{ParallelismStrategy, flag::*, transformations::*, results::*, traits::*};
use super::{cnode::{self}, mnode::*, snode::{self, *}};
use alloc::{fmt::Debug, sync::*, vec::Vec};

#[derive(Clone, Debug)]
pub(crate) enum LNodeNext<K: Key, V: Value> {
    L(Arc<LNode<K, V>>),
    S(Arc<SNode<K, V>>),
}

impl <K: Key, V: Value> LNodeNext<K, V> {
    fn key(&self) -> &K {
        match self {
            LNodeNext::L(lnode) => lnode.key(),
            LNodeNext::S(snode) => snode.key(),
        }
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> From<LNodeNext<K, V>> for MNode<H, F, K, V, M> {
    fn from(other: LNodeNext<K, V>) -> Self {
        match other {
            LNodeNext::L(lnode) => MNode::L(lnode),
            LNodeNext::S(snode) => MNode::S(snode),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LNode<K: Key, V: Value> {
    key: K,
    value: V,
    next: LNodeNext<K, V>,
    size: usize,
}

impl<K: Key, V: Value> LNode<K, V> {
    #[must_use]
    pub(super) fn key(&self) -> &K {
        &self.key
    }

    #[must_use]
    pub(super) fn value(&self) -> &V {
        &self.value
    }

    #[must_use]
    pub(super) fn new(key: K, value: V, next: LNodeNext<K, V>) -> Arc<Self> {
        let size = 1 + match &next {
            LNodeNext::L(lnode) => lnode.size,
            LNodeNext::S(_snode) => 1,
        };
        Arc::new(Self {
            key,
            value,
            next,
            size,
        })
    }

    #[must_use]
    pub(super) fn size(&self) -> usize {
        self.size
    }

    pub(super) fn visit<Op>(&self, op: Op) where Op: Fn(&K, &V) {
        op(&self.key, &self.value);
        match &self.next {
            LNodeNext::L(lnode) => lnode.visit(op),
            LNodeNext::S(snode) => snode.visit(op),
        }
    }

    pub(super) fn find<'a, L: Key>(&'a self, key: &L) -> FindResult<'a, K, V> where K: PartialEq<L> {
        if self.key == *key {
            FindResult::Found(self.key(), self.value())
        }
        else {
            match &self.next {
                LNodeNext::L(lnode) => lnode.find(key),
                LNodeNext::S(snode) => snode.find(key),
            }
        }
    }
    
}

pub(super) fn insert<'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key + Into<K>, W: Into<V>, M: HasherBv<H, K>>(this: &'a Arc<LNode<K, V>>, key: L, value: W, key_flag: Option<Flag<H, F>>, replace: bool) -> LNodeInsertResult<'a, H, F, K, V, M>
where
    K: HashLike<L>,
    K: PartialEq<L>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    match this.find(&key) {
        FindResult::Found(k, v) => if replace {
            match remove_from_lnode(this, &key) {
                LNodeRemoveResult::RemovedL(lnode, k, v) => {
                    let lnode = LNode::new(key.into(), value.into(), LNodeNext::L(lnode));
                    let key: *const K = lnode.key();
                    let value: *const V = lnode.value();
                    LNodeInsertResult::InsertedL(lnode, key, value, Some((k, v)))
                },
                LNodeRemoveResult::RemovedS(snode, k, v) => {
                    let lnode = LNode::new(key.into(), value.into(), LNodeNext::S(snode));
                    let key: *const K = lnode.key();
                    let value: *const V = lnode.value();
                    LNodeInsertResult::InsertedL(lnode, key, value, Some((k, v)))
                },
                LNodeRemoveResult::NotFound => panic!(),
            }
        }
        else {
            LNodeInsertResult::Found(k, v)
        },
        FindResult::NotFound => lift_to_cnode_and_insert(LNodeNext::L(this.clone()), key, value, key_flag.unwrap())
    }
}

pub(super) fn remove<'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, M: HasherBv<H, K>>(this: &'a Arc<LNode<K, V>>, key: &L) -> RemoveResult<'a, H, F, K, V, M> where K: PartialEq<L> {
    match remove_from_lnode(this, key) {
        LNodeRemoveResult::NotFound => RemoveResult::NotFound,
        LNodeRemoveResult::RemovedL(lnode, key, value) => RemoveResult::RemovedL(lnode, key, value),
        LNodeRemoveResult::RemovedS(snode, key, value) => RemoveResult::RemovedS(snode, key, value),
    }
}

fn remove_from_lnode<'a, K: Key, V: Value, L: Key>(this: &'a Arc<LNode<K, V>>, key: &L) -> LNodeRemoveResult<'a, K, V> where K: PartialEq<L> {
    if this.key == *key {
        match &this.next {
            LNodeNext::L(lnode) => LNodeRemoveResult::RemovedL(lnode.clone(), this.key(), this.value()),
            LNodeNext::S(snode) => LNodeRemoveResult::RemovedS(snode.clone(), this.key(), this.value()),
        }
    }
    else {
        match &this.next {
            LNodeNext::L(lnode) => match remove_from_lnode(lnode, key) {
                LNodeRemoveResult::NotFound => LNodeRemoveResult::NotFound,
                LNodeRemoveResult::RemovedL(lnode, key, value) => LNodeRemoveResult::RemovedL(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::L(lnode)), key, value),
                LNodeRemoveResult::RemovedS(snode, key, value) => LNodeRemoveResult::RemovedL(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::S(snode)), key, value),
            },
            LNodeNext::S(snode) => match snode::remove(snode, key) {
                SNodeRemoveResult::NotFound => LNodeRemoveResult::NotFound,
                SNodeRemoveResult::RemovedZ(key, value) => LNodeRemoveResult::RemovedS(SNode::new(this.key.clone(), this.value.clone()), key, value),
            }
        }
    }
}

pub(super) fn transform<K: Key, V: Value, ReduceT, ReduceOp, Op>(this: &Arc<LNode<K, V>>, reduce_op: ReduceOp, op: MapTransform<ReduceT, Op>) -> LNodeTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    Op: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
{
    match op {
        MapTransform::Generic(_) => transform_impl(this, reduce_op, op),
        MapTransform::Unchanged(r) => LNodeTransformResult::Unchanged(r),
        MapTransform::Removed(r) => LNodeTransformResult::Removed(r),
    }
}

fn transform_impl<K: Key, V: Value, ReduceT, ReduceOp, Op>(this: &Arc<LNode<K, V>>, reduce_op: ReduceOp, op: MapTransform<ReduceT, Op>) -> LNodeTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    Op: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
{
    let next = match &this.next {
        LNodeNext::L(lnode) => transform(lnode, reduce_op.clone(), op.clone()),
        LNodeNext::S(snode) => snode::transform(snode, op.clone()).into(),
    };

    transform_result(this, op.call(&this.key, &this.value), next, reduce_op)
}

pub(super) unsafe fn transmute<K: Key, V: Value, S: Key, X: Value, ReduceT, ReduceOp, Op>(this: &Arc<LNode<K, V>>, reduce_op: ReduceOp, op: MapTransmute<ReduceT, Op>) -> LNodeTransmuteResult<S, X, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    Op: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone,
    K: HashLike<S>,
    K: PartialEq<S>,
{
    match op {
        MapTransmute::Generic(_) => transmute_impl(this, reduce_op, op),
        MapTransmute::Removed(r) => LNodeTransmuteResult::Removed(r),
    }
}

unsafe fn transmute_impl<K: Key, V: Value, S: Key, X: Value, ReduceT, ReduceOp, Op>(this: &Arc<LNode<K, V>>, reduce_op: ReduceOp, op: MapTransmute<ReduceT, Op>) -> LNodeTransmuteResult<S, X, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    Op: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone,
    K: HashLike<S>,
    K: PartialEq<S>,
{
    let next = match &this.next {
        LNodeNext::L(lnode) => transmute(lnode, reduce_op.clone(), op.clone()),
        LNodeNext::S(snode) => snode::transmute(snode, op.clone()).into(),
    };

    transmute_result(op.call(&this.key, &this.value), next, reduce_op)
}

pub(crate) async fn transform_with_transformed<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &MNode<H, F, K, V, M>, reduce_op: ReduceOp, both_op: MapJointTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>, depth: usize, par_strat: ParallelismStrategy) -> MNodeJointTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone + Send + Sync,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    match right {
        MNode::C(cnode) => cnode::transform_with_transformed_lnode(cnode, this, reduce_op, both_op.flip(), right_op, left_op, depth, par_strat).await.flip(),
        MNode::L(lnode) => transform_with_transformed_lnode(this, lnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::S(snode) => transform_with_transformed_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) unsafe fn transform_with_transmuted<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>, depth: usize) -> MNodeTransformResult<H, F, K, V, M, ReduceT>
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
        MNode::C(cnode) => cnode::transmute_with_transformed_lnode(cnode, this, reduce_op, both_op.flip_transform_transmute(), right_op, left_op, depth),
        MNode::L(lnode) => transform_with_transmuted_lnode(this, lnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::S(snode) => transform_with_transmuted_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) async unsafe fn transmute_with_transformed<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>, depth: usize, par_strat: ParallelismStrategy) -> MNodeTransformResult<H, F, L, W, M, ReduceT>
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
        MNode::C(cnode) => cnode::transform_with_transmuted_lnode(cnode, this, reduce_op, both_op.flip_transmute_transform(), right_op, left_op, depth, par_strat).await,
        MNode::L(lnode) => transform_with_transmuted_lnode(lnode, this, reduce_op, both_op.flip_transmute_transform(), right_op, left_op, depth),
        MNode::S(snode) => transmute_with_transformed_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) unsafe fn transmute_with_transmuted<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, S: Key, W: Value, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: MapTransmute<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>, depth: usize) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
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
        MNode::C(cnode) => cnode::transmute_with_transmuted_lnode(cnode, this, reduce_op, both_op.flip(), right_op, left_op, depth),
        MNode::L(lnode) => transmute_with_transmuted_lnode(this, lnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::S(snode) => transmute_with_transmuted_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) fn transform_with_transformed_lnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &Arc<LNode<K, V>>, reduce_op: ReduceOp, both_op: MapJointTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>, depth: usize) -> MNodeJointTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone + Send + Sync,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let this_hash = M::default().hash(this.key());
    let right_hash = M::default().hash(right.key());

    if this_hash == right_hash {
        let mut rights = vec!((&right.key, &right.value));
        let mut r = right;
        loop {
            match &r.next {
                LNodeNext::L(lnode) => {
                    rights.push((&lnode.key, &lnode.value));
                    r = right;
                },
                LNodeNext::S(snode) => {
                    rights.push((snode.key(), snode.value()));
                    break;
                }
            }
        }
    
        let mut next = transform_with_transformed_lnode_impl(this, &mut rights, reduce_op.clone(), both_op, left_op, right_op.clone());
    
        for (rk, rv) in rights {
            next = match right_op.call(rk, rv) {
                MapTransformResult::Unchanged(rr) => match next {
                    LNodeJointTransformResult::UnchangedLR(rn) | LNodeJointTransformResult::UnchangedL(rn) => LNodeJointTransformResult::L(LNode::new(rk.clone(), rv.clone(), LNodeNext::L(this.clone())), reduce_op(&rr, &rn)),
                    LNodeJointTransformResult::UnchangedR(_rn) => panic!(), // Too difficult to really optimize over both sides for the time being
                    LNodeJointTransformResult::L(lnode, rn) => LNodeJointTransformResult::L(LNode::new(rk.clone(), rv.clone(), LNodeNext::L(lnode)), reduce_op(&rr, &rn)),
                    LNodeJointTransformResult::S(snode, rn) => LNodeJointTransformResult::L(LNode::new(rk.clone(), rv.clone(), LNodeNext::S(snode)), reduce_op(&rr, &rn)),
                    LNodeJointTransformResult::Removed(rn) => LNodeJointTransformResult::S(SNode::new(rk.clone(), rv.clone()), reduce_op(&rr, &rn)),
                },
                MapTransformResult::Transformed(rv, rr) => match next {
                    LNodeJointTransformResult::UnchangedLR(rn) | LNodeJointTransformResult::UnchangedL(rn) => LNodeJointTransformResult::L(LNode::new(rk.clone(), rv, LNodeNext::L(this.clone())), reduce_op(&rr, &rn)),
                    LNodeJointTransformResult::UnchangedR(_rn) => panic!(), // Too difficult to really optimize over both sides for the time being
                    LNodeJointTransformResult::L(lnode, rn) => LNodeJointTransformResult::L(LNode::new(rk.clone(), rv, LNodeNext::L(lnode)), reduce_op(&rr, &rn)),
                    LNodeJointTransformResult::S(snode, rn) => LNodeJointTransformResult::L(LNode::new(rk.clone(), rv, LNodeNext::S(snode)), reduce_op(&rr, &rn)),
                    LNodeJointTransformResult::Removed(rn) => LNodeJointTransformResult::S(SNode::new(rk.clone(), rv), reduce_op(&rr, &rn)),
                },
                MapTransformResult::Removed(rr) => match next {
                    LNodeJointTransformResult::UnchangedLR(rn) | LNodeJointTransformResult::UnchangedL(rn) => LNodeJointTransformResult::UnchangedL(reduce_op(&rr, &rn)),
                    LNodeJointTransformResult::UnchangedR(_rn) => panic!(), // Too difficult to really optimize over both sides for the time being
                    LNodeJointTransformResult::L(lnode, rn) => LNodeJointTransformResult::L(lnode, reduce_op(&rr, &rn)),
                    LNodeJointTransformResult::S(snode, rn) => LNodeJointTransformResult::S(snode, reduce_op(&rr, &rn)),
                    LNodeJointTransformResult::Removed(rn) => LNodeJointTransformResult::Removed(reduce_op(&rr, &rn)),
                },
            };
        }
    
        next.into()
    }
    else {
        let left = transform(this, reduce_op.clone(), left_op);
        let right_result = transform(right, reduce_op.clone(), right_op);

        let this_flag = Flag::new_at_depth(this_hash, depth).unwrap();
        let right_flag = Flag::new_at_depth(right_hash, depth).unwrap();

        match left {
            LNodeTransformResult::Unchanged(lr) => match right_result {
                LNodeTransformResult::Unchanged(rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(this.clone().into(), this_flag, right.clone().into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransformResult::L(rlnode, rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(this.clone().into(), this_flag, rlnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransformResult::S(rsnode, rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(this.clone().into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransformResult::Removed(rr) => MNodeJointTransformResult::UnchangedL(reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::L(llnode, lr) => match right_result {
                LNodeTransformResult::Unchanged(rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, right.clone().into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransformResult::L(rlnode, rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, rlnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransformResult::S(rsnode, rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransformResult::Removed(rr) => MNodeJointTransformResult::L(llnode, reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::S(lsnode, lr) => match right_result {
                LNodeTransformResult::Unchanged(rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, right.clone().into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransformResult::L(rlnode, rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, rlnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransformResult::S(rsnode, rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransformResult::Removed(rr) => MNodeJointTransformResult::S(lsnode, reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::Removed(lr) => match right_result {
                LNodeTransformResult::Unchanged(rr) => MNodeJointTransformResult::UnchangedR(reduce_op(&lr, &rr)),
                LNodeTransformResult::L(rlnode, rr) => MNodeJointTransformResult::L(rlnode, reduce_op(&lr, &rr)),
                LNodeTransformResult::S(rsnode, rr) => MNodeJointTransformResult::S(rsnode, reduce_op(&lr, &rr)),
                LNodeTransformResult::Removed(rr) => MNodeJointTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

fn transform_with_transformed_lnode_impl<K: Key, V: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, rights: &mut Vec<(&K, &V)>, reduce_op: ReduceOp, both_op: MapJointTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>) -> LNodeJointTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone + Send + Sync,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
{
    let mut next_node = None;

    let next = match &this.next {
        LNodeNext::L(lnode) => transform_with_transformed_lnode_impl(lnode, rights, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone()),
        LNodeNext::S(snode) => {
            match rights.iter().position(|key_value| *snode.key() == *key_value.0).map(|index| rights.swap_remove(index)) {
                Some((right_key, right_value)) => {
                    next_node = Some((right_key, right_value));
                    snode::transform_with_transformed_snode_impl(snode, &SNode::new(right_key.clone(), right_value.clone()), reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone())
                },
                None => match snode::transform(snode, left_op.clone()) {
                    SNodeTransformResult::Unchanged(reduced) => LNodeJointTransformResult::UnchangedL(reduced),
                    SNodeTransformResult::S(snode, reduced) => LNodeJointTransformResult::S(snode, reduced),
                    SNodeTransformResult::Removed(reduced) => LNodeJointTransformResult::Removed(reduced),
                },
            }
        },
    };

    let result = match rights.iter().position(|key_value| this.key == *key_value.0).map(|index| rights.swap_remove(index)) {
        Some((right_key, right_value)) => both_op.call(&this.key, &this.value, right_key, right_value),
        None => match left_op.call(&this.key, &this.value) {
            MapTransformResult::Unchanged(reduced) => MapJointTransformResult::UnchangedL(reduced),
            MapTransformResult::Transformed(value, reduced) => MapJointTransformResult::Transformed(value, reduced),
            MapTransformResult::Removed(reduced) => MapJointTransformResult::Removed(reduced),
        },
    };

    joint_transform_with_joint_transformed_result(this, result, next_node, next, reduce_op)
}

pub(crate) unsafe fn transform_with_transmuted_lnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &Arc<LNode<L, W>>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>, depth: usize) -> MNodeTransformResult<H, F, K, V, M, ReduceT>
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
        let mut rights = vec!((&right.key, &right.value));
        let mut r = right;
        loop {
            match &r.next {
                LNodeNext::L(lnode) => {
                    rights.push((&lnode.key, &lnode.value));
                    r = right;
                },
                LNodeNext::S(snode) => {
                    rights.push((snode.key(), snode.value()));
                    break;
                }
            }
        }
    
        let mut next = transform_with_transmuted_lnode_impl(this, &mut rights, reduce_op.clone(), both_op, left_op, right_op.clone());
    
        for (rk, rv) in rights {
            next = match right_op.call(rk, rv) {
                MapTransmuteResult::Transmuted(rk, rv, rr) => match next {
                    LNodeTransformResult::Unchanged(rn) => LNodeTransformResult::L(LNode::new(rk, rv, LNodeNext::L(this.clone())), reduce_op(&rr, &rn)),
                    LNodeTransformResult::L(lnode, rn) => LNodeTransformResult::L(LNode::new(rk, rv, LNodeNext::L(lnode)), reduce_op(&rr, &rn)),
                    LNodeTransformResult::S(snode, rn) => LNodeTransformResult::L(LNode::new(rk, rv, LNodeNext::S(snode)), reduce_op(&rr, &rn)),
                    LNodeTransformResult::Removed(rn) => LNodeTransformResult::S(SNode::new(rk, rv), reduce_op(&rr, &rn)),
                },
                MapTransmuteResult::Removed(rr) => match next {
                    LNodeTransformResult::Unchanged(rn) => LNodeTransformResult::Unchanged(reduce_op(&rr, &rn)),
                    LNodeTransformResult::L(lnode, rn) => LNodeTransformResult::L(lnode, reduce_op(&rr, &rn)),
                    LNodeTransformResult::S(snode, rn) => LNodeTransformResult::S(snode, reduce_op(&rr, &rn)),
                    LNodeTransformResult::Removed(rn) => LNodeTransformResult::Removed(reduce_op(&rr, &rn)),
                },
            };
        }
    
        next.into()
    }
    else {
        let left = transform(this, reduce_op.clone(), left_op);
        let right = transmute(right, reduce_op.clone(), right_op);

        let this_flag = Flag::new_at_depth(this_hash, depth).unwrap();
        let right_flag = Flag::new_at_depth(right_hash, depth).unwrap();

        match left {
            LNodeTransformResult::Unchanged(lr) => match right {
                LNodeTransmuteResult::L(rlnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(this.clone().into(), this_flag, rlnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransmuteResult::S(rsnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(this.clone().into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransmuteResult::Removed(rr) => MNodeTransformResult::Unchanged(reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::L(llnode, lr) => match right {
                LNodeTransmuteResult::L(rlnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, rlnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransmuteResult::S(rsnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransmuteResult::Removed(rr) => MNodeTransformResult::L(llnode, reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::S(lsnode, lr) => match right {
                LNodeTransmuteResult::L(rlnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, rlnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransmuteResult::S(rsnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransmuteResult::Removed(rr) => MNodeTransformResult::S(lsnode, reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::Removed(lr) => match right {
                LNodeTransmuteResult::L(rlnode, rr) => MNodeTransformResult::L(rlnode, reduce_op(&lr, &rr)),
                LNodeTransmuteResult::S(rsnode, rr) => MNodeTransformResult::S(rsnode, reduce_op(&lr, &rr)),
                LNodeTransmuteResult::Removed(rr) => MNodeTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

unsafe fn transform_with_transmuted_lnode_impl<K: Key, V: Value, L: Key, W: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, rights: &mut Vec<(&L, &W)>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>) -> LNodeTransformResult<K, V, ReduceT>
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
    let next = match &this.next {
        LNodeNext::L(lnode) => transform_with_transmuted_lnode_impl(lnode, rights, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone()),
        LNodeNext::S(snode) => {
            match rights.iter().position(|key_value| *snode.key() == *key_value.0).map(|index| rights.swap_remove(index)) {
                Some((right_key, right_value)) => snode::transform_with_transmuted_snode_impl(snode, right_key, right_value, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone()),
                None => snode::transform(snode, left_op.clone()).into(),
            }
        },
    };

    let result = match rights.iter().position(|key_value| this.key == *key_value.0).map(|index| rights.swap_remove(index)) {
        Some((right_key, right_value)) => both_op.call_transform_transmute(&this.key, &this.value, right_key, right_value),
        None => left_op.call(&this.key, &this.value),
    };

    transform_result(this, result, next, reduce_op)
}

pub(crate) unsafe fn transmute_with_transmuted_lnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, S: Key, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &Arc<LNode<L, W>>, reduce_op: ReduceOp, both_op: MapTransmute<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>, depth: usize) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
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
        let mut rights = vec!((&right.key, &right.value));
        let mut r = right;
        loop {
            match &r.next {
                LNodeNext::L(lnode) => {
                    rights.push((&lnode.key, &lnode.value));
                    r = right;
                },
                LNodeNext::S(snode) => {
                    rights.push((snode.key(), snode.value()));
                    break;
                }
            }
        }
    
        let mut next = transmute_with_transmuted_lnode_impl(this, &mut rights, reduce_op.clone(), both_op, left_op, right_op.clone());
    
        for (rk, rv) in rights {
            next = match right_op.call(rk, rv) {
                MapTransmuteResult::Transmuted(rk, rv, rr) => match next {
                    LNodeTransmuteResult::L(lnode, rn) => LNodeTransmuteResult::L(LNode::new(rk, rv, LNodeNext::L(lnode)), reduce_op(&rr, &rn)),
                    LNodeTransmuteResult::S(snode, rn) => LNodeTransmuteResult::L(LNode::new(rk, rv, LNodeNext::S(snode)), reduce_op(&rr, &rn)),
                    LNodeTransmuteResult::Removed(rn) => LNodeTransmuteResult::S(SNode::new(rk, rv), reduce_op(&rr, &rn)),
                },
                MapTransmuteResult::Removed(rr) => match next {
                    LNodeTransmuteResult::L(lnode, rn) => LNodeTransmuteResult::L(lnode, reduce_op(&rr, &rn)),
                    LNodeTransmuteResult::S(snode, rn) => LNodeTransmuteResult::S(snode, reduce_op(&rr, &rn)),
                    LNodeTransmuteResult::Removed(rn) => LNodeTransmuteResult::Removed(reduce_op(&rr, &rn)),
                },
            };
        }
    
        next.into()
    }
    else {
        let this = transmute(this, reduce_op.clone(), left_op);
        let right = transmute(right, reduce_op.clone(), right_op);

        let this_flag = Flag::new_at_depth(this_hash, depth).unwrap();
        let right_flag = Flag::new_at_depth(right_hash, depth).unwrap();

        match this {
            LNodeTransmuteResult::L(llnode, lr) => match right {
                LNodeTransmuteResult::L(rlnode, rr) => MNodeTransmuteResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, rlnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransmuteResult::S(rsnode, rr) => MNodeTransmuteResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransmuteResult::Removed(rr) => MNodeTransmuteResult::L(llnode, reduce_op(&lr, &rr)),
            },
            LNodeTransmuteResult::S(lsnode, lr) => match right {
                LNodeTransmuteResult::L(rlnode, rr) => MNodeTransmuteResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, rlnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransmuteResult::S(rsnode, rr) => MNodeTransmuteResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                LNodeTransmuteResult::Removed(rr) => MNodeTransmuteResult::S(lsnode, reduce_op(&lr, &rr)),
            },
            LNodeTransmuteResult::Removed(lr) => match right {
                LNodeTransmuteResult::L(rlnode, rr) => MNodeTransmuteResult::L(rlnode, reduce_op(&lr, &rr)),
                LNodeTransmuteResult::S(rsnode, rr) => MNodeTransmuteResult::S(rsnode, reduce_op(&lr, &rr)),
                LNodeTransmuteResult::Removed(rr) => MNodeTransmuteResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

unsafe fn transmute_with_transmuted_lnode_impl<K: Key, V: Value, L: Key, W: Value, S: Key, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, rights: &mut Vec<(&L, &W)>, reduce_op: ReduceOp, both_op: MapTransmute<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>) -> LNodeTransmuteResult<S, X, ReduceT>
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
    let next = match &this.next {
        LNodeNext::L(lnode) => transmute_with_transmuted_lnode_impl(lnode, rights, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone()),
        LNodeNext::S(snode) => {
            match rights.iter().position(|key_value| *snode.key() == *key_value.0).map(|index| rights.swap_remove(index)) {
                Some((right_key, right_value)) => snode::transmute_with_transmuted_values(snode.key(), snode.value(), right_key, right_value, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone()),
                None => snode::transmute(snode, left_op.clone()).into(),
            }
        },
    };

    let result = match rights.iter().position(|key_value| this.key == *key_value.0).map(|index| rights.swap_remove(index)) {
        Some((right_key, right_value)) => both_op.call_transmute(&this.key, &this.value, right_key, right_value),
        None => left_op.call(&this.key, &this.value),
    };

    transmute_result(result, next, reduce_op)
}

pub(crate) fn transform_with_transformed_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &Arc<SNode<K, V>>, reduce_op: ReduceOp, both_op: MapJointTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>, depth: usize) -> MNodeJointTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone + Send + Sync,
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
        let left = transform(this, reduce_op.clone(), left_op);
        let r = snode::transform(right, right_op);

        let this_flag = Flag::new_at_depth(this_hash, depth).unwrap();
        let right_flag = Flag::new_at_depth(right_hash, depth).unwrap();

        match left {
            LNodeTransformResult::Unchanged(lr) => match r {
                SNodeTransformResult::Unchanged(rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(this.clone().into(), this_flag, right.clone().into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::S(rsnode, rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(this.clone().into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeJointTransformResult::UnchangedL(reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::L(llnode, lr) => match r {
                SNodeTransformResult::Unchanged(rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, right.clone().into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::S(rsnode, rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeJointTransformResult::L(llnode, reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::S(lsnode, lr) => match r {
                SNodeTransformResult::Unchanged(rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, right.clone().into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::S(rsnode, rr) => MNodeJointTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeJointTransformResult::S(lsnode, reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::Removed(lr) => match r {
                SNodeTransformResult::Unchanged(rr) => MNodeJointTransformResult::UnchangedR(reduce_op(&lr, &rr)),
                SNodeTransformResult::S(rsnode, rr) => MNodeJointTransformResult::S(rsnode, reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeJointTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

pub(crate) fn transform_with_transformed_snode_impl<K: Key, V: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &Arc<SNode<K, V>>, reduce_op: ReduceOp, both_op: MapJointTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>) -> LNodeJointTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
    BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone + Send + Sync,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    RightOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
{
    if this.key == *right.key() {
        let next = match &this.next {
            LNodeNext::L(lnode) => transform(lnode, reduce_op.clone(), left_op),
            LNodeNext::S(snode) => snode::transform(snode, left_op).into(),
        };

        joint_transform_with_transformed_result(this, both_op.call(this.key(), this.value(), right.key(), right.value()), right, next, reduce_op)
    }
    else {
        let next = match &this.next {
            LNodeNext::L(lnode) => transform_with_transformed_snode_impl(lnode, right, reduce_op.clone(), both_op, left_op.clone(), right_op),
            LNodeNext::S(snode) => snode::transform_with_transformed_snode_impl(snode, right, reduce_op.clone(), both_op, left_op.clone(), right_op),
        };

        transform_with_joint_transformed_result(this, left_op.call(this.key(), this.value()), right, next, reduce_op)
    }
}

pub(crate) unsafe fn transform_with_transmuted_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>, depth: usize) -> MNodeTransformResult<H, F, K, V, M, ReduceT>
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
        transform_with_transmuted_snode_impl(this, right, reduce_op, both_op, left_op, right_op).into()
    }
    else {
        let left = transform(this, reduce_op.clone(), left_op);
        let right = snode::transmute(right, right_op);

        let this_flag = Flag::new_at_depth(this_hash, depth).unwrap();
        let right_flag = Flag::new_at_depth(right_hash, depth).unwrap();

        match left {
            LNodeTransformResult::Unchanged(lr) => match right {
                SNodeTransmuteResult::S(rsnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(this.clone().into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransmuteResult::Removed(rr) => MNodeTransformResult::Unchanged(reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::L(llnode, lr) => match right {
                SNodeTransmuteResult::S(rsnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransmuteResult::Removed(rr) => MNodeTransformResult::L(llnode, reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::S(lsnode, lr) => match right {
                SNodeTransmuteResult::S(rsnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransmuteResult::Removed(rr) => MNodeTransformResult::S(lsnode, reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::Removed(lr) => match right {
                SNodeTransmuteResult::S(rsnode, rr) => MNodeTransformResult::S(rsnode, reduce_op(&lr, &rr)),
                SNodeTransmuteResult::Removed(rr) => MNodeTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

pub(crate) unsafe fn transform_with_transmuted_snode_impl<K: Key, V: Value, L: Key, W: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>) -> LNodeTransformResult<K, V, ReduceT>
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
    if this.key == *right.key() {
        let next = match &this.next {
            LNodeNext::L(lnode) => transform(lnode, reduce_op.clone(), left_op),
            LNodeNext::S(snode) => snode::transform(snode, left_op).into(),
        };

        transform_with_transformed_result(this, both_op.call_transform_transmute(this.key(), this.value(), right.key(), right.value()), next, reduce_op)
    }
    else {
        let next = match &this.next {
            LNodeNext::L(lnode) => transform_with_transmuted_snode_impl(lnode, right, reduce_op.clone(), both_op, left_op.clone(), right_op),
            LNodeNext::S(snode) => snode::transform_with_transmuted_snode_impl(snode, right.key(), right.value(), reduce_op.clone(), both_op, left_op.clone(), right_op),
        };

        transform_with_transformed_result(this, left_op.call(this.key(), this.value()), next, reduce_op)
    }
}

pub(crate) unsafe fn transmute_with_transformed_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>, depth: usize) -> MNodeTransformResult<H, F, L, W, M, ReduceT>
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
        let this = transmute(this, reduce_op.clone(), left_op);
        let r = snode::transform(right, right_op);

        let this_flag = Flag::new_at_depth(this_hash, depth).unwrap();
        let right_flag = Flag::new_at_depth(right_hash, depth).unwrap();

        match this {
            LNodeTransmuteResult::L(llnode, lr) => match r {
                SNodeTransformResult::Unchanged(rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, right.clone().into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::S(rsnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeTransformResult::L(llnode, reduce_op(&lr, &rr)),
            },
            LNodeTransmuteResult::S(lsnode, lr) => match r {
                SNodeTransformResult::Unchanged(rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, right.clone().into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::S(rsnode, rr) => MNodeTransformResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeTransformResult::S(lsnode, reduce_op(&lr, &rr)),
            },
            LNodeTransmuteResult::Removed(lr) => match r {
                SNodeTransformResult::Unchanged(rr) => MNodeTransformResult::Unchanged(reduce_op(&lr, &rr)),
                SNodeTransformResult::S(rsnode, rr) => MNodeTransformResult::S(rsnode, reduce_op(&lr, &rr)),
                SNodeTransformResult::Removed(rr) => MNodeTransformResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

pub(crate) unsafe fn transmute_with_transformed_snode_impl<K: Key, V: Value, L: Key, W: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>) -> LNodeTransformResult<L, W, ReduceT>
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
    if this.key == *right.key() {
        let next = match &this.next {
            LNodeNext::L(lnode) => transmute(lnode, reduce_op.clone(), left_op),
            LNodeNext::S(snode) => snode::transmute(snode, left_op).into(),
        };

        transform_with_transmuted_result(right, both_op.call_transmute_transform(this.key(), this.value(), right.key(), right.value()), next, reduce_op)
    }
    else {
        let next = match &this.next {
            LNodeNext::L(lnode) => transmute_with_transformed_snode_impl(lnode, right, reduce_op.clone(), both_op, left_op.clone(), right_op),
            LNodeNext::S(snode) => snode::transmute_with_transformed_snode_impl(snode, right, reduce_op.clone(), both_op, left_op.clone(), right_op),
        };

        transmute_with_transformed_result(right, left_op.call(this.key(), this.value()), next, reduce_op)
    }
}

pub(crate) unsafe fn transmute_with_transmuted_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, S: Key, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: MapTransmute<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>, depth: usize) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
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
        transmute_with_transmuted_snode_impl(this, right, reduce_op, both_op, left_op, right_op).into()
    }
    else {
        let this = transmute(this, reduce_op.clone(), left_op);
        let right = snode::transmute(right, right_op);

        let this_flag = Flag::new_at_depth(this_hash, depth).unwrap();
        let right_flag = Flag::new_at_depth(right_hash, depth).unwrap();

        match this {
            LNodeTransmuteResult::L(llnode, lr) => match right {
                SNodeTransmuteResult::S(rsnode, rr) => MNodeTransmuteResult::C(cnode::lift_to_cnode_and_insert(llnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransmuteResult::Removed(rr) => MNodeTransmuteResult::L(llnode, reduce_op(&lr, &rr)),
            },
            LNodeTransmuteResult::S(lsnode, lr) => match right {
                SNodeTransmuteResult::S(rsnode, rr) => MNodeTransmuteResult::C(cnode::lift_to_cnode_and_insert(lsnode.into(), this_flag, rsnode.into(), right_flag), reduce_op(&lr, &rr)),
                SNodeTransmuteResult::Removed(rr) => MNodeTransmuteResult::S(lsnode, reduce_op(&lr, &rr)),
            },
            LNodeTransmuteResult::Removed(lr) => match right {
                SNodeTransmuteResult::S(rsnode, rr) => MNodeTransmuteResult::S(rsnode, reduce_op(&lr, &rr)),
                SNodeTransmuteResult::Removed(rr) => MNodeTransmuteResult::Removed(reduce_op(&lr, &rr)),
            },
        }
    }
}

pub(crate) unsafe fn transmute_with_transmuted_snode_impl<K: Key, V: Value, L: Key, S: Key, W: Value, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &Arc<LNode<K, V>>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: MapTransmute<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>) -> LNodeTransmuteResult<S, X, ReduceT>
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
    if this.key == *right.key() {
        let next = match &this.next {
            LNodeNext::L(lnode) => transmute(lnode, reduce_op.clone(), left_op),
            LNodeNext::S(snode) => snode::transmute(snode, left_op).into(),
        };
        let this = both_op.call_transmute(this.key(), this.value(), right.key(), right.value());

        transmute_result(this, next, reduce_op)
    }
    else {
        let next = match &this.next {
            LNodeNext::L(lnode) => transmute_with_transmuted_snode_impl(lnode, right, reduce_op.clone(), both_op, left_op.clone(), right_op),
            LNodeNext::S(snode) => snode::transmute_with_transmuted_values(snode.key(), snode.value(), right.key(), right.value(), reduce_op.clone(), both_op, left_op.clone(), right_op),
        };
        let this = left_op.call(this.key(), this.value());

        transmute_result(this, next, reduce_op)
    }
}

pub(super) fn joint_transform_with_joint_transformed_result<K: Key, V: Value, ReduceT, ReduceOp>(this: &Arc<LNode<K, V>>, result: MapJointTransformResult<V, ReduceT>, that: Option<(&K, &V)>, next: LNodeJointTransformResult<K, V, ReduceT>, reduce_op: ReduceOp) -> LNodeJointTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync
{
    match result {
        MapJointTransformResult::UnchangedLR(lr) => match next {
            LNodeJointTransformResult::UnchangedLR(rr) => LNodeJointTransformResult::UnchangedLR(reduce_op(&lr, &rr)),
            LNodeJointTransformResult::UnchangedL(rr) => LNodeJointTransformResult::UnchangedL(reduce_op(&lr, &rr)),
            LNodeJointTransformResult::UnchangedR(rr) => LNodeJointTransformResult::UnchangedR(reduce_op(&lr, &rr)),
            LNodeJointTransformResult::L(lnode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::S(snode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::Removed(rr) => LNodeJointTransformResult::S(SNode::new(this.key.clone(), this.value.clone()), reduce_op(&lr, &rr)),
        },
        MapJointTransformResult::UnchangedL(lr) => match next {
            LNodeJointTransformResult::UnchangedLR(rr) | LNodeJointTransformResult::UnchangedL(rr) => LNodeJointTransformResult::UnchangedL(reduce_op(&lr, &rr)),
            LNodeJointTransformResult::UnchangedR(rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::S(SNode::new(that.as_ref().unwrap().0.clone(), that.as_ref().unwrap().1.clone()))), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::L(lnode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::S(snode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::Removed(rr) => LNodeJointTransformResult::S(SNode::new(this.key.clone(), this.value.clone()), reduce_op(&lr, &rr)),
        },
        MapJointTransformResult::UnchangedR(lr) => match next {
            LNodeJointTransformResult::UnchangedLR(rr) | LNodeJointTransformResult::UnchangedR(rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::S(SNode::new(that.as_ref().unwrap().0.clone(), that.as_ref().unwrap().1.clone()))), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::UnchangedL(rr) => LNodeJointTransformResult::L(LNode::new(that.as_ref().unwrap().0.clone(), that.as_ref().unwrap().1.clone(), this.next.clone()), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::L(lnode, rr) => LNodeJointTransformResult::L(LNode::new(that.as_ref().unwrap().0.clone(), that.as_ref().unwrap().1.clone(), LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::S(snode, rr) => LNodeJointTransformResult::L(LNode::new(that.as_ref().unwrap().0.clone(), that.as_ref().unwrap().1.clone(), LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::Removed(rr) => LNodeJointTransformResult::UnchangedR(reduce_op(&lr, &rr)),
        },
        MapJointTransformResult::Transformed(lv, lr) => match next {
            LNodeJointTransformResult::UnchangedLR(rr) | LNodeJointTransformResult::UnchangedL(rr) => LNodeJointTransformResult::L(LNode::new(this.key().clone(), lv, this.next.clone()), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::UnchangedR(rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::S(SNode::new(that.as_ref().unwrap().0.clone(), that.as_ref().unwrap().1.clone()))), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::L(lnode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), lv, LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::S(snode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), lv, LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::Removed(rr) => LNodeJointTransformResult::S(SNode::new(this.key.clone(), lv), reduce_op(&lr, &rr)),
        },
        MapJointTransformResult::Removed(lr) => match next {
            LNodeJointTransformResult::UnchangedLR(rr) | LNodeJointTransformResult::UnchangedL(rr) => match &this.next {
                LNodeNext::L(lnode) => LNodeJointTransformResult::L(lnode.clone(), reduce_op(&lr, &rr)),
                LNodeNext::S(snode) => LNodeJointTransformResult::S(snode.clone(), reduce_op(&lr, &rr)),
            },
            LNodeJointTransformResult::UnchangedR(rr) => LNodeJointTransformResult::S(SNode::new(that.as_ref().unwrap().0.clone(), that.as_ref().unwrap().1.clone()), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::L(lnode, rr) => LNodeJointTransformResult::L(lnode, reduce_op(&lr, &rr)),
            LNodeJointTransformResult::S(snode, rr) => LNodeJointTransformResult::S(snode, reduce_op(&lr, &rr)),
            LNodeJointTransformResult::Removed(rr) => LNodeJointTransformResult::Removed(reduce_op(&lr, &rr)),
        },
    }
}

pub(super) fn joint_transform_with_transformed_result<K: Key, V: Value, ReduceT, ReduceOp>(this: &Arc<LNode<K, V>>, result: MapJointTransformResult<V, ReduceT>, that: &Arc<SNode<K, V>>, next: LNodeTransformResult<K, V, ReduceT>, reduce_op: ReduceOp) -> LNodeJointTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync
{
    match result {
        MapJointTransformResult::UnchangedLR(lr) | MapJointTransformResult::UnchangedL(lr) => match next {
            LNodeTransformResult::Unchanged(rr) => LNodeJointTransformResult::UnchangedL(reduce_op(&lr, &rr)),
            LNodeTransformResult::L(lnode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::S(snode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::Removed(rr) => LNodeJointTransformResult::S(SNode::new(this.key.clone(), this.value.clone()), reduce_op(&lr, &rr)),
        },
        MapJointTransformResult::UnchangedR(lr) => match next {
            LNodeTransformResult::Unchanged(rr) => LNodeJointTransformResult::L(LNode::new(that.key().clone(), that.value().clone(), this.next.clone()), reduce_op(&lr, &rr)),
            LNodeTransformResult::L(lnode, rr) => LNodeJointTransformResult::L(LNode::new(that.key().clone(), that.value().clone(), LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::S(snode, rr) => LNodeJointTransformResult::L(LNode::new(that.key().clone(), that.value().clone(), LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::Removed(rr) => LNodeJointTransformResult::UnchangedR(reduce_op(&lr, &rr)),
        },
        MapJointTransformResult::Transformed(lv, lr) => match next {
            LNodeTransformResult::Unchanged(rr) => LNodeJointTransformResult::L(LNode::new(this.key().clone(), lv, this.next.clone()), reduce_op(&lr, &rr)),
            LNodeTransformResult::L(lnode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), lv, LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::S(snode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), lv, LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::Removed(rr) => LNodeJointTransformResult::S(SNode::new(this.key.clone(), lv), reduce_op(&lr, &rr)),
        },
        MapJointTransformResult::Removed(lr) => match next {
            LNodeTransformResult::Unchanged(rr) => match &this.next {
                LNodeNext::L(lnode) => LNodeJointTransformResult::L(lnode.clone(), reduce_op(&lr, &rr)),
                LNodeNext::S(snode) => LNodeJointTransformResult::S(snode.clone(), reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::L(lnode, rr) => LNodeJointTransformResult::L(lnode, reduce_op(&lr, &rr)),
            LNodeTransformResult::S(snode, rr) => LNodeJointTransformResult::S(snode, reduce_op(&lr, &rr)),
            LNodeTransformResult::Removed(rr) => LNodeJointTransformResult::Removed(reduce_op(&lr, &rr)),
        },
    }
}

pub(super) fn transform_with_transformed_result<K: Key, V: Value, ReduceT, ReduceOp>(this: &Arc<LNode<K, V>>, result: MapTransformResult<V, ReduceT>, next: LNodeTransformResult<K, V, ReduceT>, reduce_op: ReduceOp) -> LNodeTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync
{
    match result {
        MapTransformResult::Unchanged(lr) => match next {
            LNodeTransformResult::Unchanged(rr) => LNodeTransformResult::Unchanged(reduce_op(&lr, &rr)),
            LNodeTransformResult::L(lnode, rr) => LNodeTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::S(snode, rr) => LNodeTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::Removed(rr) => LNodeTransformResult::S(SNode::new(this.key.clone(), this.value.clone()), reduce_op(&lr, &rr)),
        },
        MapTransformResult::Transformed(lv, lr) => match next {
            LNodeTransformResult::Unchanged(rr) => LNodeTransformResult::L(LNode::new(this.key().clone(), lv, this.next.clone()), reduce_op(&lr, &rr)),
            LNodeTransformResult::L(lnode, rr) => LNodeTransformResult::L(LNode::new(this.key.clone(), lv, LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::S(snode, rr) => LNodeTransformResult::L(LNode::new(this.key.clone(), lv, LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::Removed(rr) => LNodeTransformResult::S(SNode::new(this.key.clone(), lv), reduce_op(&lr, &rr)),
        },
        MapTransformResult::Removed(lr) => match next {
            LNodeTransformResult::Unchanged(rr) => match &this.next {
                LNodeNext::L(lnode) => LNodeTransformResult::L(lnode.clone(), reduce_op(&lr, &rr)),
                LNodeNext::S(snode) => LNodeTransformResult::S(snode.clone(), reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::L(lnode, rr) => LNodeTransformResult::L(lnode, reduce_op(&lr, &rr)),
            LNodeTransformResult::S(snode, rr) => LNodeTransformResult::S(snode, reduce_op(&lr, &rr)),
            LNodeTransformResult::Removed(rr) => LNodeTransformResult::Removed(reduce_op(&lr, &rr)),
        },
    }
}

pub(super) fn transform_with_joint_transformed_result<K: Key, V: Value, ReduceT, ReduceOp>(this: &Arc<LNode<K, V>>, result: MapTransformResult<V, ReduceT>, that: &Arc<SNode<K, V>>, next: LNodeJointTransformResult<K, V, ReduceT>, reduce_op: ReduceOp) -> LNodeJointTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync
{
    match result {
        MapTransformResult::Unchanged(lr) => match next {
            LNodeJointTransformResult::UnchangedLR(rr) | LNodeJointTransformResult::UnchangedL(rr) => LNodeJointTransformResult::UnchangedL(reduce_op(&lr, &rr)),
            LNodeJointTransformResult::UnchangedR(rr) => LNodeJointTransformResult::L(LNode::new(that.key().clone(), that.value().clone(), LNodeNext::L(this.clone())), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::L(lnode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::S(snode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::Removed(rr) => LNodeJointTransformResult::S(SNode::new(this.key.clone(), this.value.clone()), reduce_op(&lr, &rr)),
        },
        MapTransformResult::Transformed(lv, lr) => match next {
            LNodeJointTransformResult::UnchangedLR(rr) | LNodeJointTransformResult::UnchangedL(rr) => LNodeJointTransformResult::L(LNode::new(this.key().clone(), lv, this.next.clone()), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::UnchangedR(rr) => LNodeJointTransformResult::L(LNode::new(this.key().clone(), lv, LNodeNext::S(that.clone())), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::L(lnode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), lv, LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::S(snode, rr) => LNodeJointTransformResult::L(LNode::new(this.key.clone(), lv, LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeJointTransformResult::Removed(rr) => LNodeJointTransformResult::S(SNode::new(this.key.clone(), lv), reduce_op(&lr, &rr)),
        },
        MapTransformResult::Removed(lr) => match next {
            LNodeJointTransformResult::UnchangedLR(rr) | LNodeJointTransformResult::UnchangedL(rr) => match &this.next {
                LNodeNext::L(lnode) => LNodeJointTransformResult::L(lnode.clone(), reduce_op(&lr, &rr)),
                LNodeNext::S(snode) => LNodeJointTransformResult::S(snode.clone(), reduce_op(&lr, &rr)),
            },
            LNodeJointTransformResult::UnchangedR(rr) => LNodeJointTransformResult::UnchangedR(reduce_op(&lr, &rr)),
            LNodeJointTransformResult::L(lnode, rr) => LNodeJointTransformResult::L(lnode, reduce_op(&lr, &rr)),
            LNodeJointTransformResult::S(snode, rr) => LNodeJointTransformResult::S(snode, reduce_op(&lr, &rr)),
            LNodeJointTransformResult::Removed(rr) => LNodeJointTransformResult::Removed(reduce_op(&lr, &rr)),
        },
    }
}

pub(super) fn transform_with_transmuted_result<K: Key, V: Value, ReduceT, ReduceOp>(this: &Arc<SNode<K, V>>, result: MapTransformResult<V, ReduceT>, next: LNodeTransmuteResult<K, V, ReduceT>, reduce_op: ReduceOp) -> LNodeTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync
{
    match result {
        MapTransformResult::Unchanged(lr) => match next {
            LNodeTransmuteResult::L(lnode, rr) => LNodeTransformResult::L(LNode::new(this.key().clone(), this.value().clone(), LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeTransmuteResult::S(snode, rr) => LNodeTransformResult::L(LNode::new(this.key().clone(), this.value().clone(), LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeTransmuteResult::Removed(rr) => LNodeTransformResult::S(SNode::new(this.key().clone(), this.value().clone()), reduce_op(&lr, &rr)),
        },
        MapTransformResult::Transformed(lv, lr) => match next {
            LNodeTransmuteResult::L(lnode, rr) => LNodeTransformResult::L(LNode::new(this.key().clone(), lv, LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeTransmuteResult::S(snode, rr) => LNodeTransformResult::L(LNode::new(this.key().clone(), lv, LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeTransmuteResult::Removed(rr) => LNodeTransformResult::S(SNode::new(this.key().clone(), lv), reduce_op(&lr, &rr)),
        },
        MapTransformResult::Removed(lr) => match next {
            LNodeTransmuteResult::L(lnode, rr) => LNodeTransformResult::L(lnode, reduce_op(&lr, &rr)),
            LNodeTransmuteResult::S(snode, rr) => LNodeTransformResult::S(snode, reduce_op(&lr, &rr)),
            LNodeTransmuteResult::Removed(rr) => LNodeTransformResult::Removed(reduce_op(&lr, &rr)),
        },
    }
}

pub(super) fn transmute_with_transformed_result<K: Key, V: Value, ReduceT, ReduceOp>(this: &Arc<SNode<K, V>>, result: MapTransmuteResult<K, V, ReduceT>, next: LNodeTransformResult<K, V, ReduceT>, reduce_op: ReduceOp) -> LNodeTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync
{
    match result {
        MapTransmuteResult::Transmuted(lk, lv, lr) => match next {
            LNodeTransformResult::Unchanged(rr) => LNodeTransformResult::L(LNode::new(lk, lv, LNodeNext::S(this.clone())), reduce_op(&lr, &rr)),
            LNodeTransformResult::L(lnode, rr) => LNodeTransformResult::L(LNode::new(lk, lv, LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::S(snode, rr) => LNodeTransformResult::L(LNode::new(lk, lv, LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::Removed(rr) => LNodeTransformResult::S(SNode::new(lk, lv), reduce_op(&lr, &rr)),
        },
        MapTransmuteResult::Removed(lr) => match next {
            LNodeTransformResult::Unchanged(rr) => LNodeTransformResult::S(SNode::new(this.key().clone(), this.value().clone()), reduce_op(&lr, &rr)),
            LNodeTransformResult::L(lnode, rr) => LNodeTransformResult::L(lnode, reduce_op(&lr, &rr)),
            LNodeTransformResult::S(snode, rr) => LNodeTransformResult::S(snode, reduce_op(&lr, &rr)),
            LNodeTransformResult::Removed(rr) => LNodeTransformResult::Removed(reduce_op(&lr, &rr)),
        },
    }
}

pub(super) fn transform_result<K: Key, V: Value, ReduceT, ReduceOp>(this: &Arc<LNode<K, V>>, result: MapTransformResult<V, ReduceT>, next: LNodeTransformResult<K, V, ReduceT>, reduce_op: ReduceOp) -> LNodeTransformResult<K, V, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync
{
    match result {
        MapTransformResult::Unchanged(lr) => match next {
            LNodeTransformResult::Unchanged(rr) => LNodeTransformResult::Unchanged(reduce_op(&lr, &rr)),
            LNodeTransformResult::L(lnode, rr) => LNodeTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::S(snode, rr) => LNodeTransformResult::L(LNode::new(this.key.clone(), this.value.clone(), LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::Removed(rr) => LNodeTransformResult::S(SNode::new(this.key.clone(), this.value.clone()), reduce_op(&lr, &rr)),
        },
        MapTransformResult::Transformed(lv, lr) => match next {
            LNodeTransformResult::Unchanged(rr) => LNodeTransformResult::L(LNode::new(this.key.clone(), lv, this.next.clone()), reduce_op(&lr, &rr)),
            LNodeTransformResult::L(lnode, rr) => LNodeTransformResult::L(LNode::new(this.key.clone(), lv, LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::S(snode, rr) => LNodeTransformResult::L(LNode::new(this.key.clone(), lv, LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeTransformResult::Removed(rr) => LNodeTransformResult::S(SNode::new(this.key.clone(), lv), reduce_op(&lr, &rr)),
        },
        MapTransformResult::Removed(lr) => match next {
            LNodeTransformResult::Unchanged(rr) => match &this.next {
                LNodeNext::L(lnode) => LNodeTransformResult::L(lnode.clone(), reduce_op(&lr, &rr)),
                LNodeNext::S(snode) => LNodeTransformResult::S(snode.clone(), reduce_op(&lr, &rr)),
            },
            LNodeTransformResult::L(lnode, rr) => LNodeTransformResult::L(lnode, reduce_op(&lr, &rr)),
            LNodeTransformResult::S(snode, rr) => LNodeTransformResult::S(snode, reduce_op(&lr, &rr)),
            LNodeTransformResult::Removed(rr) => LNodeTransformResult::Removed(reduce_op(&lr, &rr)),
        },
    }
}

pub(super) unsafe fn transmute_result<S: Key, X: Value, ReduceT, ReduceOp>(result: MapTransmuteResult<S, X, ReduceT>, next: LNodeTransmuteResult<S, X, ReduceT>, reduce_op: ReduceOp) -> LNodeTransmuteResult<S, X, ReduceT>
where
    ReduceT: Clone + Default + Send + Sync,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync
{
    match result {
        MapTransmuteResult::Transmuted(lk, lv, lr) => match next {
            LNodeTransmuteResult::L(lnode, rr) => LNodeTransmuteResult::L(LNode::new(lk, lv, LNodeNext::L(lnode)), reduce_op(&lr, &rr)),
            LNodeTransmuteResult::S(snode, rr) => LNodeTransmuteResult::L(LNode::new(lk, lv, LNodeNext::S(snode)), reduce_op(&lr, &rr)),
            LNodeTransmuteResult::Removed(rr) => LNodeTransmuteResult::S(SNode::new(lk, lv), reduce_op(&lr, &rr)),
        },
        MapTransmuteResult::Removed(lr) => match next {
            LNodeTransmuteResult::L(lnode, rr) => LNodeTransmuteResult::L(lnode, reduce_op(&lr, &rr)),
            LNodeTransmuteResult::S(snode, rr) => LNodeTransmuteResult::S(snode, reduce_op(&lr, &rr)),
            LNodeTransmuteResult::Removed(rr) => LNodeTransmuteResult::Removed(reduce_op(&lr, &rr)),
        },
    }
}

#[must_use]
pub(super) fn lift_to_cnode_and_insert<'a, H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key + Into<K>, W: Into<V>, M: HasherBv<H, K>>(this: LNodeNext<K, V>, key: L, value: W, key_flag: Flag<H, F>) -> LNodeInsertResult<'a, H, F, K, V, M>
where
    K: HashLike<L>,
    K: PartialEq<L>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let this_hash = M::default().hash(this.key());
    if this_hash == key_flag.hash_value() {
        let lnode = LNode::new(key.into(), value.into(), this);
        let key: *const K = lnode.key();
        let value: *const V = lnode.value();
        LNodeInsertResult::InsertedL(lnode, key, value, None)
    }
    else {
        let this_flag = Flag::new_at_depth(this_hash, key_flag.depth()).unwrap();

        let snode = SNode::new(key.into(), value.into());
        let key: *const K = snode.key();
        let value: *const V = snode.value();
        LNodeInsertResult::InsertedC(cnode::lift_to_cnode_and_insert(this.into(), this_flag, snode.into(), key_flag), key, value, None)
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> From<Arc<LNode<K, V>>> for MNode<H, F, K, V, M> {
    fn from(other: Arc<LNode<K, V>>) -> Self {
        MNode::L(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    macro_rules! lnode {
        ( $keyvalue1:expr, $keyvalue2:expr ) => {
            {
                LNode::new($keyvalue1.0, $keyvalue1.1, LNodeNext::S(SNode::new($keyvalue2.0, $keyvalue2.1)))
            }
        };
        ( $keyvalue:expr, $($rest:expr),+ ) => {
            {
                LNode::new($keyvalue.0, $keyvalue.1, LNodeNext::L(lnode!($($rest),*)))
            }
        };
    }

    #[test]
    fn lnode_insert_3() {
        let node = lnode!((3, ()), (2, ()), (1, ()));
        assert_eq!(node.size, 3);
        assert_found_eq!(node.find(&1), (1, ()));
        assert_found_eq!(node.find(&2), (2, ()));
        assert_found_eq!(node.find(&3), (3, ()));
        assert_found_none!(node.find(&4));
    }

}
