use crate::{bit_indexed_array::*, flag::*, result::*, traits::*};
use super::{lnode::{self, *}, mnode::*, snode::{self, *}};
use alloc::{boxed::Box, borrow::Cow, fmt::Debug, sync::Arc, vec::Vec};
use core::ptr;

#[derive(Debug)]
pub(crate) struct CNode <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> {
    nodes: Arc<dyn BitIndexedArray::<F, MNode<H, F, K, V, M>, usize>>,
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> CNode<H, F, K, V, M> {
    #[must_use]
    pub(super) fn new(nodes: Box<dyn BitIndexedArray::<F, MNode<H, F, K, V, M>, usize> + 'static>) -> Self {
        Self { nodes: nodes.into() }
    }
    
    #[must_use]
    pub(super) fn size(&self) -> usize {
        *self.nodes.extra()
    }
    
    pub(super) fn find<'a, L: Key>(&'a self, key: &L, flag: Option<Flag<H, F>>) -> FindResult<'a, K, V> where K: PartialEq<L>, <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
        match self.nodes.at(flag.as_ref().unwrap().flag.clone()) {
            Ok(node) => match node {
                MNode::C(cnode) => cnode.find(key, flag.unwrap().next()),
                MNode::L(lnode) => lnode.find(key),
                MNode::S(snode) => snode.find(key),
            },
            Err(_) => FindResult::NotFound
        }
    }

    pub(super) fn remove<'a, L: Key>(&'a self, key: &L, flag: Option<Flag<H, F>>) -> RemoveResult<'a, H, F, K, V, M> where K: PartialEq<L>, M: HasherBv<H, L>, <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
        match self.nodes.at(flag.as_ref().unwrap().flag.clone()) {
            Ok(node) => match node.remove(key, flag.as_ref().unwrap().next()) {
                RemoveResult::NotFound => RemoveResult::NotFound,
                RemoveResult::RemovedC(node, key, value) => {
                    if node.size() == 0 {
                        panic!()
                    }
                    RemoveResult::RemovedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::C(node)), Cow::Owned(self.size() - 1)).unwrap()), key, value)
                },
                RemoveResult::RemovedL(node, key, value) => {
                    RemoveResult::RemovedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::L(node)), Cow::Owned(self.size() - 1)).unwrap()), key, value)
                },
                RemoveResult::RemovedS(node, key, value) => {
                    RemoveResult::RemovedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::S(node)), Cow::Owned(self.size() - 1)).unwrap()), key, value)
                },
                RemoveResult::RemovedZ(key, value) => {
                    if self.size() == 1 {
                        RemoveResult::RemovedZ(key, value)
                    }
                    else {
                        RemoveResult::RemovedC(Self::new(self.nodes.removed(flag.unwrap().flag, Cow::Owned(self.size() - 1)).unwrap()), key, value)
                    }
                },
            },
            Err(_) => RemoveResult::NotFound
        }
    }
    
    pub(super) fn visit<Op: Clone>(&self, op: Op) where Op: Fn(&K, &V) {
        for node in self.nodes.as_ref() {
            match node {
                MNode::C(cnode) => cnode.visit(op.clone()),
                MNode::L(lnode) => lnode.visit(op.clone()),
                MNode::S(snode) => snode.visit(op.clone()),
            }
        }
    }

}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> CNode<H, F, K, V, M> {
    pub(super) fn insert<'a, L: Key + Into<K>, W: Into<V>>(&'a self, key: L, value: W, flag: Option<Flag<H, F>>, replace: bool) -> CNodeInsertResult<'a, H, F, K, V, M>
    where
        K: HashLike<L>,
        K: PartialEq<L>,
        M: HasherBv<H, L>,
        <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
    {
        match self.nodes.at(flag.as_ref().unwrap().flag.clone()) {
            Ok(node) => match node {
                MNode::C(cnode) => match cnode.insert(key, value, flag.as_ref().unwrap().next(), replace) {
                    CNodeInsertResult::Found(key, value) => CNodeInsertResult::Found(key, value),
                    CNodeInsertResult::InsertedC(cnode2, key, value, prev) => {
                        let size = self.size() + cnode2.size() - cnode.size();
                        CNodeInsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::C(cnode2)), Cow::Owned(size)).unwrap()), key, value, prev)
                    },
                },
                MNode::L(lnode) => match lnode::insert(&lnode, key, value, flag.as_ref().unwrap().next(), replace) {
                    LNodeInsertResult::Found(key, value) => CNodeInsertResult::Found(key, value),
                    LNodeInsertResult::InsertedC(cnode, key, value, prev) => CNodeInsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::C(cnode)), Cow::Owned(self.size() + 1)).unwrap()), key, value, prev),
                    LNodeInsertResult::InsertedL(lnode2, key, value, prev) => {
                        let size = self.size() + lnode2.size() - lnode.size();
                        CNodeInsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::L(lnode2)), Cow::Owned(size)).unwrap()), key, value, prev)
                    },
                },
                MNode::S(snode) => match snode::insert(&snode, key, value, flag.as_ref().unwrap().next(), replace) {
                    InsertResult::Found(key, value) => CNodeInsertResult::Found(key, value),
                    InsertResult::InsertedC(cnode, key, value, prev) => CNodeInsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::C(cnode)), Cow::Owned(self.size() + 1)).unwrap()), key, value, prev),
                    InsertResult::InsertedL(lnode, key, value, prev) => CNodeInsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::L(lnode)), Cow::Owned(self.size() + 1)).unwrap()), key, value, prev),
                    InsertResult::InsertedS(snode, key, value, prev) => CNodeInsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::S(snode)), Cow::Owned(self.size())).unwrap()), key, value, prev),
                },
            },
            Err(_) => {
                let snode = SNode::new(key.into(), value.into());
                let key: *const K = snode.key();
                let value: *const V = snode.value();
                CNodeInsertResult::InsertedC(Self::new(self.nodes.inserted(flag.unwrap().flag, Cow::Owned(MNode::S(snode)), Cow::Owned(self.size() + 1)).unwrap()), key, value, None)
            }
        }
    }
}

pub(super) fn transform<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, Op>(this: &CNode<H, F, K, V, M>, reduce_op: ReduceOp, op: Op) -> MNodeTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    Op: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();
    let mut changed = false;

    for index in 0..<F>::max_ones() {
        if let Ok(node) = this.nodes.at_bit_index(index) {
            match node.transform(reduce_op.clone(), op.clone()) {
                MNodeTransformResult::Unchanged(r) => {
                    size += node.size();
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(node.clone());
                    reduced = reduce_op(&reduced, &r);
                },
                MNodeTransformResult::C(cnode, r) => {
                    size += cnode.size();
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(MNode::C(cnode));
                    reduced = reduce_op(&reduced, &r);
                    changed = true;
                },
                MNodeTransformResult::L(lnode, r) => {
                    size += lnode.size();
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(MNode::L(lnode));
                    reduced = reduce_op(&reduced, &r);
                    changed = true;
                },
                MNodeTransformResult::S(snode, r) => {
                    size += 1;
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(MNode::S(snode));
                    reduced = reduce_op(&reduced, &r);
                    changed = true;
                },
                MNodeTransformResult::Removed(r) => {
                    reduced = reduce_op(&reduced, &r);
                    changed = true;
                },
            }
        }
    }

    if changed {
        match values_t.len() {
            0 => MNodeTransformResult::Removed(reduced),
            1 => match values_t.pop().unwrap() {
                MNode::C(cnode) => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&[MNode::C(cnode)]), size).unwrap()), reduced),
                MNode::L(lnode) => MNodeTransformResult::L(lnode, reduced),
                MNode::S(snode) => MNodeTransformResult::S(snode, reduced),
            },
            _ => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
        }
    }
    else {
        MNodeTransformResult::Unchanged(reduced)
    }
}


pub(super) unsafe fn transmute<H: Hashword, F: Flagword<H>, K: Key, V: Value, S: Key, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, Op>(this: &CNode<H, F, K, V, M>, reduce_op: ReduceOp, op: Op) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    Op: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone,
    K: HashLike<S>,
    K: PartialEq<S>,
    M: HasherBv<H, S>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();

    for index in 0..<F>::max_ones() {
        if let Ok(node) = this.nodes.at_bit_index(index) {
            match node.transmute(reduce_op.clone(), op.clone()) {
                MNodeTransmuteResult::C(cnode, r) => {
                    size += cnode.size();
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(MNode::C(cnode));
                    reduced = reduce_op(&reduced, &r);
                },
                MNodeTransmuteResult::L(lnode, r) => {
                    size += lnode.size();
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(MNode::L(lnode));
                    reduced = reduce_op(&reduced, &r);
                },
                MNodeTransmuteResult::S(snode, r) => {
                    size += 1;
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(MNode::S(snode));
                    reduced = reduce_op(&reduced, &r);
                },
                MNodeTransmuteResult::Removed(r) => {
                    reduced = reduce_op(&reduced, &r);
                },
            }
        }
    }

    assert_eq!(values_t.len(), bits_t.count_ones_t());

    match values_t.len() {
        0 => MNodeTransmuteResult::Removed(reduced),
        1 => match values_t.pop().unwrap() {
            MNode::C(cnode) => MNodeTransmuteResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&[MNode::C(cnode)]), size).unwrap()), reduced),
            MNode::L(lnode) => MNodeTransmuteResult::L(lnode, reduced),
            MNode::S(snode) => MNodeTransmuteResult::S(snode, reduced),
        },
        _ => MNodeTransmuteResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
    }
}

pub(crate) fn transform_with_transformed<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &MNode<H, F, K, V, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeJointTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    RightOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    match right {
        MNode::C(cnode) => transform_with_transformed_cnode(this, cnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::L(lnode) => transform_with_transformed_lnode(this, lnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::S(snode) => transform_with_transformed_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) unsafe fn transform_with_transmuted<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    RightOp: Fn(&L, &W) -> MapTransmuteResult<K, V, ReduceT> + Clone,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    match right {
        MNode::C(cnode) => transform_with_transmuted_cnode(this, cnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::L(lnode) => transform_with_transmuted_lnode(this, lnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::S(snode) => transform_with_transmuted_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) unsafe fn transmute_with_transformed<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, L, W, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransmuteResult<L, W, ReduceT> + Clone,
    RightOp: Fn(&L, &W) -> MapTransformResult<W, ReduceT> + Clone,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    match right {
        MNode::C(cnode) => transmute_with_transformed_cnode(this, cnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::L(lnode) => transmute_with_transformed_lnode(this, lnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::S(snode) => transmute_with_transformed_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) unsafe fn transmute_with_transmuted<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, S: Key, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
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
    match right {
        MNode::C(cnode) => transmute_with_transmuted_cnode(this, cnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::L(lnode) => transmute_with_transmuted_lnode(this, lnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::S(snode) => transmute_with_transmuted_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) fn transform_with_transformed_cnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &CNode<H, F, K, V, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeJointTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    RightOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();
    let mut unchangedl = true;
    let mut unchangedr = true;

    for index in 0..<F>::max_ones() {
        let mut left = None;
        let mut right_node = None;

        let transform_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            left = Some(node);
            if let Ok(right) = right.nodes.at_bit_index(index) {
                right_node = Some(right);
                node.transform_with_transformed(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                match node.transform(reduce_op.clone(), left_op.clone()) {
                    MNodeTransformResult::Unchanged(reduced) => MNodeJointTransformResult::UnchangedL(reduced),
                    MNodeTransformResult::C(cnode, reduced) => MNodeJointTransformResult::C(cnode, reduced),
                    MNodeTransformResult::L(lnode, reduced) => MNodeJointTransformResult::L(lnode, reduced),
                    MNodeTransformResult::S(snode, reduced) => MNodeJointTransformResult::S(snode, reduced),
                    MNodeTransformResult::Removed(reduced) => MNodeJointTransformResult::Removed(reduced),
                }
            }
        }
        else if let Ok(right) = right.nodes.at_bit_index(index) {
            right_node = Some(right);
            match right.transform(reduce_op.clone(), right_op.clone()) {
                MNodeTransformResult::Unchanged(reduced) => MNodeJointTransformResult::UnchangedR(reduced),
                MNodeTransformResult::C(cnode, reduced) => MNodeJointTransformResult::C(cnode, reduced),
                MNodeTransformResult::L(lnode, reduced) => MNodeJointTransformResult::L(lnode, reduced),
                MNodeTransformResult::S(snode, reduced) => MNodeJointTransformResult::S(snode, reduced),
                MNodeTransformResult::Removed(reduced) => MNodeJointTransformResult::Removed(reduced),
            }
        }
        else {
            continue;
        };

        match transform_result {
            MNodeJointTransformResult::UnchangedLR(r) => {
                size += left.unwrap().size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(left.unwrap().clone());
                reduced = reduce_op(&reduced, &r);
            },
            MNodeJointTransformResult::UnchangedL(r) => {
                size += left.unwrap().size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(left.unwrap().clone());
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeJointTransformResult::UnchangedR(r) => {
                size += right_node.unwrap().size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(right_node.unwrap().clone());
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeJointTransformResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
                unchangedr = false;
            },
            MNodeJointTransformResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
                unchangedr = false;
            },
            MNodeJointTransformResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
                unchangedr = false;
            },
            MNodeJointTransformResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
                unchangedr = false;
            },
        }
    }

    if unchangedl {
        if unchangedr {
            MNodeJointTransformResult::UnchangedLR(reduced)
        }
        else {
            MNodeJointTransformResult::UnchangedL(reduced)
        }
    }
    else if unchangedr {
        MNodeJointTransformResult::UnchangedR(reduced)
    }
    else {
        match values_t.len() {
            0 => MNodeJointTransformResult::Removed(reduced),
            1 => match values_t.pop().unwrap() {
                MNode::C(_cnode) => MNodeJointTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
                MNode::L(lnode) => MNodeJointTransformResult::L(lnode, reduced),
                MNode::S(snode) => MNodeJointTransformResult::S(snode, reduced),
            },
            _ => MNodeJointTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
        }
    }
}

pub(crate) unsafe fn transform_with_transmuted_cnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &CNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    RightOp: Fn(&L, &W) -> MapTransmuteResult<K, V, ReduceT> + Clone,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();
    let mut unchangedl = true;

    for index in 0..<F>::max_ones() {
        let mut left = None;

        let transform_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            left = Some(node);
            if let Ok(right) = right.nodes.at_bit_index(index) {
                node.transform_with_transmuted(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                node.transform(reduce_op.clone(), left_op.clone())
            }
        }
        else if let Ok(right) = right.nodes.at_bit_index(index) {
            right.transmute(reduce_op.clone(), right_op.clone()).into()
        }
        else {
            continue;
        };

        match transform_result {
            MNodeTransformResult::Unchanged(r) => {
                size += left.unwrap().size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(left.unwrap().clone());
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeTransformResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeTransformResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeTransformResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
        }
    }

    if unchangedl {
        MNodeTransformResult::Unchanged(reduced)
    }
    else {
        match values_t.len() {
            0 => MNodeTransformResult::Removed(reduced),
            1 => match values_t.pop().unwrap() {
                MNode::C(_cnode) => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
                MNode::L(lnode) => MNodeTransformResult::L(lnode, reduced),
                MNode::S(snode) => MNodeTransformResult::S(snode, reduced),
            },
            _ => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
        }
    }
}

pub(crate) unsafe fn transmute_with_transformed_cnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &CNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, L, W, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransmuteResult<L, W, ReduceT> + Clone,
    RightOp: Fn(&L, &W) -> MapTransformResult<W, ReduceT> + Clone,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();
    let mut unchangedr = true;

    for index in 0..<F>::max_ones() {
        let mut right_node = None;

        let transform_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            if let Ok(right) = right.nodes.at_bit_index(index) {
                right_node = Some(right);
                node.transmute_with_transformed(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                node.transmute(reduce_op.clone(), left_op.clone()).into()
            }
        }
        else if let Ok(right) = right.nodes.at_bit_index(index) {
            right_node = Some(right);
            right.transform(reduce_op.clone(), right_op.clone())
        }
        else {
            continue;
        };

        match transform_result {
            MNodeTransformResult::Unchanged(r) => {
                size += right_node.unwrap().size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(right_node.unwrap().clone());
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeTransformResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeTransformResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeTransformResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
        }
    }

    if unchangedr {
        MNodeTransformResult::Unchanged(reduced)
    }
    else {
        match values_t.len() {
            0 => MNodeTransformResult::Removed(reduced),
            1 => match values_t.pop().unwrap() {
                MNode::C(_cnode) => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
                MNode::L(lnode) => MNodeTransformResult::L(lnode, reduced),
                MNode::S(snode) => MNodeTransformResult::S(snode, reduced),
            },
            _ => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
        }
    }
}

pub(crate) unsafe fn transmute_with_transmuted_cnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, S: Key, W: Value, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &CNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
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
    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();

    for index in 0..<F>::max_ones() {
        let transmute_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            if let Ok(right) = right.nodes.at_bit_index(index) {
                node.transmute_with_transmuted(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                node.transmute(reduce_op.clone(), left_op.clone())
            }
        }
        else if let Ok(right) = right.nodes.at_bit_index(index) {
            right.transmute(reduce_op.clone(), right_op.clone())
        }
        else {
            continue;
        };

        match transmute_result {
            MNodeTransmuteResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransmuteResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransmuteResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransmuteResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
            },
        }
    }

    match values_t.len() {
        0 => MNodeTransmuteResult::Removed(reduced),
        1 => match values_t.pop().unwrap() {
            MNode::C(_cnode) => MNodeTransmuteResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
            MNode::L(lnode) => MNodeTransmuteResult::L(lnode, reduced),
            MNode::S(snode) => MNodeTransmuteResult::S(snode, reduced),
        },
        _ => MNodeTransmuteResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
    }
}

pub(crate) fn transform_with_transformed_lnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &Arc<LNode<K, V>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeJointTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    RightOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let flag = Flag::<H, F>::new_at_depth(M::default().hash(right.key()), depth).unwrap().flag();

    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();
    let mut unchangedl = true;
    let mut unchangedr = true;

    for index in 0..<F>::max_ones() {
        let mut left = None;

        let transform_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            left = Some(node);
            if flag == F::nth_bit(index).unwrap() {
                node.transform_with_transformed_lnode(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                match node.transform(reduce_op.clone(), left_op.clone()) {
                    MNodeTransformResult::Unchanged(reduced) => MNodeJointTransformResult::UnchangedL(reduced),
                    MNodeTransformResult::C(cnode, reduced) => MNodeJointTransformResult::C(cnode, reduced),
                    MNodeTransformResult::L(lnode, reduced) => MNodeJointTransformResult::L(lnode, reduced),
                    MNodeTransformResult::S(snode, reduced) => MNodeJointTransformResult::S(snode, reduced),
                    MNodeTransformResult::Removed(reduced) => MNodeJointTransformResult::Removed(reduced),
                }
            }
        }
        else if flag == F::nth_bit(index).unwrap() {
            match lnode::transform(right, reduce_op.clone(), right_op.clone()) {
                LNodeTransformResult::Unchanged(reduced) => MNodeJointTransformResult::UnchangedR(reduced),
                LNodeTransformResult::L(lnode, reduced) => MNodeJointTransformResult::L(lnode, reduced),
                LNodeTransformResult::S(snode, reduced) => MNodeJointTransformResult::S(snode, reduced),
                LNodeTransformResult::Removed(reduced) => MNodeJointTransformResult::Removed(reduced),
            }
        }
        else {
            continue;
        };

        match transform_result {
            MNodeJointTransformResult::UnchangedLR(r) => {
                size += left.unwrap().size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(left.unwrap().clone());
                reduced = reduce_op(&reduced, &r);
            },
            MNodeJointTransformResult::UnchangedL(r) => {
                size += left.unwrap().size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(left.unwrap().clone());
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeJointTransformResult::UnchangedR(r) => {
                size += right.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(right.clone()));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeJointTransformResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
                unchangedr = false;
            },
            MNodeJointTransformResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
                unchangedr = false;
            },
            MNodeJointTransformResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
                unchangedr = false;
            },
            MNodeJointTransformResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
                unchangedr = false;
            },
        }
    }

    if unchangedl {
        if unchangedr {
            MNodeJointTransformResult::UnchangedLR(reduced)
        }
        else {
            MNodeJointTransformResult::UnchangedL(reduced)
        }
    }
    else if unchangedr {
        MNodeJointTransformResult::UnchangedR(reduced)
    }
    else {
        match values_t.len() {
            0 => MNodeJointTransformResult::Removed(reduced),
            1 => match values_t.pop().unwrap() {
                MNode::C(_cnode) => MNodeJointTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
                MNode::L(lnode) => MNodeJointTransformResult::L(lnode, reduced),
                MNode::S(snode) => MNodeJointTransformResult::S(snode, reduced),
            },
            _ => MNodeJointTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
        }
    }
}

pub(crate) unsafe fn transform_with_transmuted_lnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &Arc<LNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    RightOp: Fn(&L, &W) -> MapTransmuteResult<K, V, ReduceT> + Clone,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let flag = Flag::<H, F>::new_at_depth(M::default().hash(right.key()), depth).unwrap().flag();

    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();
    let mut unchangedl = true;

    for index in 0..<F>::max_ones() {
        let mut left = None;

        let transform_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            left = Some(node);
            if flag == F::nth_bit(index).unwrap() {
                node.transform_with_transmuted_lnode(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                node.transform(reduce_op.clone(), left_op.clone())
            }
        }
        else if flag == F::nth_bit(index).unwrap() {
            lnode::transmute(right, reduce_op.clone(), right_op.clone()).into()
        }
        else {
            continue;
        };

        match transform_result {
            MNodeTransformResult::Unchanged(r) => {
                size += left.unwrap().size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(left.unwrap().clone());
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeTransformResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeTransformResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeTransformResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeTransformResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
        }
    }

    if unchangedl {
        MNodeTransformResult::Unchanged(reduced)
    }
    else {
        match values_t.len() {
            0 => MNodeTransformResult::Removed(reduced),
            1 => match values_t.pop().unwrap() {
                MNode::C(_cnode) => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
                MNode::L(lnode) => MNodeTransformResult::L(lnode, reduced),
                MNode::S(snode) => MNodeTransformResult::S(snode, reduced),
            },
            _ => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
        }
    }
}

pub(crate) unsafe fn transmute_with_transformed_lnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &Arc<LNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, L, W, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransmuteResult<L, W, ReduceT> + Clone,
    RightOp: Fn(&L, &W) -> MapTransformResult<W, ReduceT> + Clone,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let flag = Flag::<H, F>::new_at_depth(M::default().hash(right.key()), depth).unwrap().flag();

    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();
    let mut unchangedr = true;

    for index in 0..<F>::max_ones() {
        let transform_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            if flag == F::nth_bit(index).unwrap() {
                node.transmute_with_transformed_lnode(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                node.transmute(reduce_op.clone(), left_op.clone()).into()
            }
        }
        else if flag == F::nth_bit(index).unwrap() {
            lnode::transform(right, reduce_op.clone(), right_op.clone()).into()
        }
        else {
            continue;
        };

        match transform_result {
            MNodeTransformResult::Unchanged(r) => {
                size += right.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(right.clone()));
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeTransformResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeTransformResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeTransformResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeTransformResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
        }
    }

    if unchangedr {
        MNodeTransformResult::Unchanged(reduced)
    }
    else {
        match values_t.len() {
            0 => MNodeTransformResult::Removed(reduced),
            1 => match values_t.pop().unwrap() {
                MNode::C(_cnode) => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
                MNode::L(lnode) => MNodeTransformResult::L(lnode, reduced),
                MNode::S(snode) => MNodeTransformResult::S(snode, reduced),
            },
            _ => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
        }
    }
}

pub(crate) unsafe fn transmute_with_transmuted_lnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, S: Key, W: Value, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &Arc<LNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
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
    let flag = Flag::<H, F>::new_at_depth(M::default().hash(right.key()), depth).unwrap().flag();

    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();

    for index in 0..<F>::max_ones() {
        let transmute_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            if flag == F::nth_bit(index).unwrap() {
                node.transmute_with_transmuted_lnode(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                node.transmute(reduce_op.clone(), left_op.clone())
            }
        }
        else if flag == F::nth_bit(index).unwrap() {
            lnode::transmute(right, reduce_op.clone(), right_op.clone()).into()
        }
        else {
            continue;
        };

        match transmute_result {
            MNodeTransmuteResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransmuteResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransmuteResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransmuteResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
            },
        }
    }

    match values_t.len() {
        0 => MNodeTransmuteResult::Removed(reduced),
        1 => match values_t.pop().unwrap() {
            MNode::C(_cnode) => MNodeTransmuteResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
            MNode::L(lnode) => MNodeTransmuteResult::L(lnode, reduced),
            MNode::S(snode) => MNodeTransmuteResult::S(snode, reduced),
        },
        _ => MNodeTransmuteResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
    }
}

pub(crate) fn transform_with_transformed_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &Arc<SNode<K, V>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeJointTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    RightOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let flag = Flag::<H, F>::new_at_depth(M::default().hash(right.key()), depth).unwrap().flag();

    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();
    let mut unchangedl = true;
    let mut unchangedr = true;

    for index in 0..<F>::max_ones() {
        let mut left = None;

        let transform_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            left = Some(node);
            if flag == F::nth_bit(index).unwrap() {
                node.transform_with_transformed_snode(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                match node.transform(reduce_op.clone(), left_op.clone()) {
                    MNodeTransformResult::Unchanged(reduced) => MNodeJointTransformResult::UnchangedL(reduced),
                    MNodeTransformResult::C(cnode, reduced) => MNodeJointTransformResult::C(cnode, reduced),
                    MNodeTransformResult::L(lnode, reduced) => MNodeJointTransformResult::L(lnode, reduced),
                    MNodeTransformResult::S(snode, reduced) => MNodeJointTransformResult::S(snode, reduced),
                    MNodeTransformResult::Removed(reduced) => MNodeJointTransformResult::Removed(reduced),
                }
            }
        }
        else if flag == F::nth_bit(index).unwrap() {
            match snode::transform(right, right_op.clone()) {
                SNodeTransformResult::Unchanged(reduced) => MNodeJointTransformResult::UnchangedR(reduced),
                SNodeTransformResult::S(snode, reduced) => MNodeJointTransformResult::S(snode, reduced),
                SNodeTransformResult::Removed(reduced) => MNodeJointTransformResult::Removed(reduced),
            }
        }
        else {
            continue;
        };

        match transform_result {
            MNodeJointTransformResult::UnchangedLR(r) => {
                size += left.unwrap().size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(left.unwrap().clone());
                reduced = reduce_op(&reduced, &r);
            },
            MNodeJointTransformResult::UnchangedL(r) => {
                size += left.unwrap().size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(left.unwrap().clone());
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeJointTransformResult::UnchangedR(r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(right.clone()));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeJointTransformResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
                unchangedr = false;
            },
            MNodeJointTransformResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
                unchangedr = false;
            },
            MNodeJointTransformResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
                unchangedr = false;
            },
            MNodeJointTransformResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
                unchangedr = false;
            },
        }
    }

    if unchangedl {
        if unchangedr {
            MNodeJointTransformResult::UnchangedLR(reduced)
        }
        else {
            MNodeJointTransformResult::UnchangedL(reduced)
        }
    }
    else if unchangedr {
        MNodeJointTransformResult::UnchangedR(reduced)
    }
    else {
        match values_t.len() {
            0 => MNodeJointTransformResult::Removed(reduced),
            1 => match values_t.pop().unwrap() {
                MNode::C(_cnode) => MNodeJointTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
                MNode::L(lnode) => MNodeJointTransformResult::L(lnode, reduced),
                MNode::S(snode) => MNodeJointTransformResult::S(snode, reduced),
            },
            _ => MNodeJointTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
        }
    }
}

pub(crate) unsafe fn transform_with_transmuted_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, K, V, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone,
    RightOp: Fn(&L, &W) -> MapTransmuteResult<K, V, ReduceT> + Clone,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let flag = Flag::<H, F>::new_at_depth(M::default().hash(right.key()), depth).unwrap().flag();

    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();
    let mut unchangedl = true;

    for index in 0..<F>::max_ones() {
        let mut left = None;

        let transform_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            left = Some(node);
            if flag == F::nth_bit(index).unwrap() {
                node.transform_with_transmuted_snode(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                node.transform(reduce_op.clone(), left_op.clone())
            }
        }
        else if flag == F::nth_bit(index).unwrap() {
            snode::transmute(right, right_op.clone()).into()
        }
        else {
            continue;
        };

        match transform_result {
            MNodeTransformResult::Unchanged(r) => {
                size += left.unwrap().size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(left.unwrap().clone());
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeTransformResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeTransformResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
            MNodeTransformResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
                unchangedl = false;
            },
        }
    }

    if unchangedl {
        MNodeTransformResult::Unchanged(reduced)
    }
    else {
        match values_t.len() {
            0 => MNodeTransformResult::Removed(reduced),
            1 => match values_t.pop().unwrap() {
                MNode::C(_cnode) => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
                MNode::L(lnode) => MNodeTransformResult::L(lnode, reduced),
                MNode::S(snode) => MNodeTransformResult::S(snode, reduced),
            },
            _ => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
        }
    }
}

pub(crate) unsafe fn transmute_with_transformed_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, L, W, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT> + Clone,
    LeftOp: Fn(&K, &V) -> MapTransmuteResult<L, W, ReduceT> + Clone,
    RightOp: Fn(&L, &W) -> MapTransformResult<W, ReduceT> + Clone,
    K: HashLike<L>,
    K: PartialEq<L>,
    L: HashLike<K>,
    L: PartialEq<K>,
    M: HasherBv<H, L>,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let flag = Flag::<H, F>::new_at_depth(M::default().hash(right.key()), depth).unwrap().flag();

    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();
    let mut unchangedr = true;

    for index in 0..<F>::max_ones() {
        let transform_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            if flag == F::nth_bit(index).unwrap() {
                node.transmute_with_transformed_snode(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                node.transmute(reduce_op.clone(), left_op.clone()).into()
            }
        }
        else if flag == F::nth_bit(index).unwrap() {
            snode::transform(right, right_op.clone()).into()
        }
        else {
            continue;
        };

        match transform_result {
            MNodeTransformResult::Unchanged(r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(right.clone()));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeTransformResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeTransformResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
            MNodeTransformResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
                unchangedr = false;
            },
        }
    }

    if unchangedr {
        MNodeTransformResult::Unchanged(reduced)
    }
    else {
        match values_t.len() {
            0 => MNodeTransformResult::Removed(reduced),
            1 => match values_t.pop().unwrap() {
                MNode::C(_cnode) => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
                MNode::L(lnode) => MNodeTransformResult::L(lnode, reduced),
                MNode::S(snode) => MNodeTransformResult::S(snode, reduced),
            },
            _ => MNodeTransformResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
        }
    }
}

pub(crate) unsafe fn transmute_with_transmuted_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, S: Key, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransmuteResult<H, F, S, X, M, ReduceT>
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
    let flag = Flag::<H, F>::new_at_depth(M::default().hash(right.key()), depth).unwrap().flag();

    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();

    for index in 0..<F>::max_ones() {
        let transmute_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            if flag == F::nth_bit(index).unwrap() {
                node.transmute_with_transmuted_snode(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                node.transmute(reduce_op.clone(), left_op.clone())
            }
        }
        else if flag == F::nth_bit(index).unwrap() {
            snode::transmute(right, right_op.clone()).into()
        }
        else {
            continue;
        };

        match transmute_result {
            MNodeTransmuteResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransmuteResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransmuteResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransmuteResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
            },
        }
    }

    match values_t.len() {
        0 => MNodeTransmuteResult::Removed(reduced),
        1 => match values_t.pop().unwrap() {
            MNode::C(_cnode) => MNodeTransmuteResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
            MNode::L(lnode) => MNodeTransmuteResult::L(lnode, reduced),
            MNode::S(snode) => MNodeTransmuteResult::S(snode, reduced),
        },
        _ => MNodeTransmuteResult::C(CNode::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
    }
}

#[must_use]
pub(super) fn lift_to_cnode_and_insert<H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>>(this: MNode<H, F, K, V, M>, this_flag: Flag<H, F>, right: MNode<H, F, K, V, M>, right_flag: Flag<H, F>) -> CNode<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    let size = this.size() + right.size();
    if this_flag.flag() == right_flag.flag() {
        CNode::new(new_bit_indexed_array(this_flag.flag(), BitIndexedArrayVec::new(&[MNode::C(lift_to_cnode_and_insert(this, this_flag.next().unwrap(), right, right_flag.next().unwrap()))]), size).unwrap())
    }
    else  {
        let flags = this_flag.flag().bit_insert(right_flag.flag()).unwrap();
        let values = if flags.bit_index(this_flag.flag).unwrap() == 0 {
            vec!(this, right)
        } else {
            vec!(right, this)
        };
        CNode::new(new_bit_indexed_array(flags, BitIndexedArrayVec::new(&values), size).unwrap())
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: 'static> Clone for CNode<H, F, K, V, M> {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone()
        }
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> Default for CNode<H, F, K, V, M> {
    fn default() -> Self {
        CNode::new(default_bit_indexed_array())
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> Eq for CNode<H, F, K, V, M> {}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> PartialEq for CNode<H, F, K, V, M> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(Arc::as_ptr(&self.nodes) as *const u8, Arc::as_ptr(&other.nodes) as *const u8)
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> From<CNode<H, F, K, V, M>> for MNode<H, F, K, V, M> {
    fn from(other: CNode<H, F, K, V, M>) -> Self {
        MNode::C(other)
    }
}
