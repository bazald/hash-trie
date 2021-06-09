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
    
    pub(super) fn find<L: Key>(&self, key: &L, flag: Option<Flag<H, F>>) -> FindResult<K, V> where K: PartialEq<L>, <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
        match self.nodes.at(flag.as_ref().unwrap().flag.clone()) {
            Ok(node) => match node {
                MNode::C(cnode) => cnode.find(key, match flag {Some(flag) => flag.next(), None => None}),
                MNode::L(lnode) => lnode::find(lnode, key),
                MNode::S(snode) => snode::find(snode, key),
            },
            Err(_) => FindResult::NotFound
        }
    }

    pub(super) fn remove<L: Key>(&self, key: &L, flag: Option<Flag<H, F>>) -> RemoveResult<H, F, K, V, M> where K: PartialEq<L>, M: HasherBv<H, L>, <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
        match self.nodes.at(flag.as_ref().unwrap().flag.clone()) {
            Ok(node) => match node.remove(key, flag.as_ref().unwrap().next()) {
                RemoveResult::NotFound => RemoveResult::NotFound,
                RemoveResult::RemovedC(node, key_value) => {
                    if node.size() == 0 {
                        panic!()
                    }
                    RemoveResult::RemovedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::C(node)), Cow::Owned(self.size() - 1)).unwrap()), key_value)
                },
                RemoveResult::RemovedL(node, key_value) => {
                    RemoveResult::RemovedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::L(node)), Cow::Owned(self.size() - 1)).unwrap()), key_value)
                },
                RemoveResult::RemovedS(node, key_value) => {
                    RemoveResult::RemovedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::S(node)), Cow::Owned(self.size() - 1)).unwrap()), key_value)
                },
                RemoveResult::RemovedZ(key_value) => {
                    if self.size() == 1 {
                        RemoveResult::RemovedZ(key_value)
                    }
                    else {
                        RemoveResult::RemovedC(Self::new(self.nodes.removed(flag.unwrap().flag, Cow::Owned(self.size() - 1)).unwrap()), key_value)
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
    pub(super) fn insert<L: Key + Into<K>, W: Into<V>>(&self, key: L, value: W, flag: Option<Flag<H, F>>, replace: bool) -> InsertResult<H, F, K, V, M>
    where
        K: HashLike<L>,
        K: PartialEq<L>,
        M: HasherBv<H, L>,
        <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
    {
        match self.nodes.at(flag.as_ref().unwrap().flag.clone()) {
            Ok(node) => match node {
                MNode::C(cnode) => match cnode.insert(key, value, flag.as_ref().unwrap().next(), replace) {
                    InsertResult::Found(key_value) => InsertResult::Found(key_value),
                    InsertResult::InsertedC(cnode, key_value) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::C(cnode)), Cow::Owned(self.size() + 1)).unwrap()), key_value),
                    InsertResult::InsertedL(lnode, key_value) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::L(lnode)), Cow::Owned(self.size() + 1)).unwrap()), key_value),
                    InsertResult::InsertedS(_snode, _key_value) => panic!(),
                },
                MNode::L(lnode) => match lnode::insert(&lnode, key, value, flag.as_ref().unwrap().next(), replace) {
                    InsertResult::Found(key_value) => InsertResult::Found(key_value),
                    InsertResult::InsertedC(cnode, key_value) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::C(cnode)), Cow::Owned(self.size() + 1)).unwrap()), key_value),
                    InsertResult::InsertedL(lnode, key_value) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::L(lnode)), Cow::Owned(self.size() + 1)).unwrap()), key_value),
                    InsertResult::InsertedS(_snode, _key_value) => panic!(),
                },
                MNode::S(snode) => match snode::insert(&snode, key, value, flag.as_ref().unwrap().next(), replace) {
                    InsertResult::Found(key_value) => InsertResult::Found(key_value),
                    InsertResult::InsertedC(cnode, key_value) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::C(cnode)), Cow::Owned(self.size() + 1)).unwrap()), key_value),
                    InsertResult::InsertedL(lnode, key_value) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::L(lnode)), Cow::Owned(self.size() + 1)).unwrap()), key_value),
                    InsertResult::InsertedS(snode, key_value) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::S(snode)), Cow::Owned(self.size() + 1)).unwrap()), key_value),
                },
            },
            Err(_) => {
                let snode = SNode::new(key.into(), value.into());
                InsertResult::InsertedC(Self::new(self.nodes.inserted(flag.unwrap().flag, Cow::Owned(MNode::S(snode.clone())), Cow::Owned(self.size() + 1)).unwrap()), snode.clone().into())
            }
        }
    }
}

pub(super) fn transform<H: Hashword, F: Flagword<H>, K: Key, V: Value, S: Key, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, Op>(this: &CNode<H, F, K, V, M>, reduce_op: ReduceOp, op: Op) -> MNodeTransformResult<H, F, S, X, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
    Op: Fn(&K, &V) -> MapTransformResult<S, X, ReduceT> + Clone,
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
            match node.transform(reduce_op.clone(), op.clone()) {
                MNodeTransformResult::C(cnode, r) => {
                    size += cnode.size();
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(MNode::C(cnode));
                    reduced = reduce_op(&reduced, &r);
                },
                MNodeTransformResult::L(lnode, r) => {
                    size += lnode.size();
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(MNode::L(lnode));
                    reduced = reduce_op(&reduced, &r);
                },
                MNodeTransformResult::S(snode, r) => {
                    size += 1;
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(MNode::S(snode));
                    reduced = reduce_op(&reduced, &r);
                },
                MNodeTransformResult::Removed(r) => {
                    reduced = reduce_op(&reduced, &r);
                },
            }
        }
    }

    match values_t.len() {
        0 => MNodeTransformResult::Removed(reduced),
        1 => match values_t.pop().unwrap() {
            MNode::C(_cnode) => MNodeTransformResult::C(CNode::<H, F, S, X, M>::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
            MNode::L(lnode) => MNodeTransformResult::L(lnode, reduced),
            MNode::S(snode) => MNodeTransformResult::S(snode, reduced),
        },
        _ => MNodeTransformResult::C(CNode::<H, F, S, X, M>::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
    }
}

pub(crate) fn joint_transform<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, S: Key, X: Value, M: HasherBv<H, K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &MNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, S, X, M, ReduceT>
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
        MNode::<H, F, L, W, M>::C(cnode) => joint_transform_cnode(this, cnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::<H, F, L, W, M>::L(lnode) => joint_transform_lnode(this, lnode, reduce_op, both_op, left_op, right_op, depth),
        MNode::<H, F, L, W, M>::S(snode) => joint_transform_snode(this, snode, reduce_op, both_op, left_op, right_op, depth),
    }
}

pub(crate) fn joint_transform_cnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, S: Key, W: Value, X: Value, M: HasherBv<H, K> + HasherBv<H, L>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &CNode<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, S, X, M, ReduceT>
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
    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();

    for index in 0..<F>::max_ones() {
        let transform_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            if let Ok(right) = right.nodes.at_bit_index(index) {
                node.joint_transform(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                node.transform(reduce_op.clone(), left_op.clone())
            }
        }
        else if let Ok(right) = right.nodes.at_bit_index(index) {
            right.transform(reduce_op.clone(), right_op.clone())
        }
        else {
            continue;
        };

        match transform_result {
            MNodeTransformResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
            },
        }
    }

    match values_t.len() {
        0 => MNodeTransformResult::Removed(reduced),
        1 => match values_t.pop().unwrap() {
            MNode::C(_cnode) => MNodeTransformResult::C(CNode::<H, F, S, X, M>::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
            MNode::L(lnode) => MNodeTransformResult::L(lnode, reduced),
            MNode::S(snode) => MNodeTransformResult::S(snode, reduced),
        },
        _ => MNodeTransformResult::C(CNode::<H, F, S, X, M>::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
    }
}

pub(crate) fn joint_transform_lnode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, S: Key, W: Value, X: Value, M: HasherBv<H, K> + HasherBv<H, L>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &Arc<LNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, S, X, M, ReduceT>
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
    let flag = Flag::<H, F>::new_at_depth(M::default().hash(right.key()), depth).unwrap().flag();

    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();

    for index in 0..<F>::max_ones() {
        let transform_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            if flag == F::nth_bit(index).unwrap() {
                node.joint_transform_lnode(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                node.transform(reduce_op.clone(), left_op.clone())
            }
        }
        else if flag == F::nth_bit(index).unwrap() {
            lnode::transform(right, reduce_op.clone(), right_op.clone()).into()
        }
        else {
            continue;
        };

        match transform_result {
            MNodeTransformResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
            },
        }
    }

    match values_t.len() {
        0 => MNodeTransformResult::Removed(reduced),
        1 => match values_t.pop().unwrap() {
            MNode::C(_cnode) => MNodeTransformResult::C(CNode::<H, F, S, X, M>::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
            MNode::L(lnode) => MNodeTransformResult::L(lnode, reduced),
            MNode::S(snode) => MNodeTransformResult::S(snode, reduced),
        },
        _ => MNodeTransformResult::C(CNode::<H, F, S, X, M>::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
    }
}

pub(crate) fn joint_transform_snode<H: Hashword, F: Flagword<H>, K: Key, V: Value, L: Key, W: Value, S: Key, X: Value, M: HasherBv<H, K> + HasherBv<H, L>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>(this: &CNode<H, F, K, V, M>, right: &Arc<SNode<L, W>>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp, depth: usize) -> MNodeTransformResult<H, F, S, X, M, ReduceT>
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
    let flag = Flag::<H, F>::new_at_depth(M::default().hash(right.key()), depth).unwrap().flag();

    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();

    for index in 0..<F>::max_ones() {
        let transform_result = if let Ok(node) = this.nodes.at_bit_index(index) {
            if flag == F::nth_bit(index).unwrap() {
                node.joint_transform_snode(right, reduce_op.clone(), both_op.clone(), left_op.clone(), right_op.clone(), depth + 1)
            }
            else {
                node.transform(reduce_op.clone(), left_op.clone())
            }
        }
        else if flag == F::nth_bit(index).unwrap() {
            snode::transform(right, right_op.clone()).into()
        }
        else {
            continue;
        };

        match transform_result {
            MNodeTransformResult::C(cnode, r) => {
                size += cnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::C(cnode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::L(lnode, r) => {
                size += lnode.size();
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::L(lnode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::S(snode, r) => {
                size += 1;
                bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                values_t.push(MNode::S(snode));
                reduced = reduce_op(&reduced, &r);
            },
            MNodeTransformResult::Removed(r) => {
                reduced = reduce_op(&reduced, &r);
            },
        }
    }

    match values_t.len() {
        0 => MNodeTransformResult::Removed(reduced),
        1 => match values_t.pop().unwrap() {
            MNode::C(_cnode) => MNodeTransformResult::C(CNode::<H, F, S, X, M>::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
            MNode::L(lnode) => MNodeTransformResult::L(lnode, reduced),
            MNode::S(snode) => MNodeTransformResult::S(snode, reduced),
        },
        _ => MNodeTransformResult::C(CNode::<H, F, S, X, M>::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
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
        CNode::<H, F, K, V, M>::new(default_bit_indexed_array())
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
