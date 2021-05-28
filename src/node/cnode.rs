use crate::{bit_indexed_array::*, flag::*, result::*, traits::*};
use super::{lnode::{self, *}, mnode::*, snode::{self, *}};
use alloc::{boxed::Box, borrow::Cow, fmt::Debug, sync::Arc, vec::Vec};
use core::ptr;

#[derive(Debug)]
pub(crate) struct CNode <H: Hashword, F: Flagword<H>, V: Value, M: 'static> {
    nodes: Arc<dyn BitIndexedArray::<F, MNode<H, F, V, M>, usize>>,
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> CNode<H, F, V, M> {
    #[must_use]
    pub(super) fn new(nodes: Box<dyn BitIndexedArray::<F, MNode<H, F, V, M>, usize> + 'static>) -> Self {
        Self { nodes: nodes.into() }
    }
    
    #[must_use]
    pub(super) fn size(&self) -> usize {
        *self.nodes.extra()
    }
    
    pub(super) fn find<'a, K>(&'a self, key: &K, flag: Option<Flag<H, F>>) -> FindResult<'a, V> where V: PartialEq<K>, <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
        match self.nodes.at(flag.as_ref().unwrap().flag.clone()) {
            Ok(node) => match node {
                MNode::C(cnode) => cnode.find(key, match flag {Some(flag) => flag.next(), None => None}),
                MNode::L(lnode) => lnode.find(key),
                MNode::S(snode) => snode.find(key),
            },
            Err(_) => FindResult::NotFound
        }
    }

    pub(super) fn remove<'a, K>(&'a self, key: &K, flag: Option<Flag<H, F>>) -> RemoveResult<'a, H, F, V, M> where V: PartialEq<K>, <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
        match self.nodes.at(flag.as_ref().unwrap().flag.clone()) {
            Ok(node) => match node.remove(key, flag.as_ref().unwrap().next()) {
                RemoveResult::NotFound => RemoveResult::NotFound,
                RemoveResult::RemovedC(node, reference) => {
                    if node.size() == 0 {
                        panic!()
                    }
                    RemoveResult::RemovedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::C(node)), Cow::Owned(self.size() - 1)).unwrap()), reference)
                },
                RemoveResult::RemovedL(node, reference) => {
                    RemoveResult::RemovedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::L(node)), Cow::Owned(self.size() - 1)).unwrap()), reference)
                },
                RemoveResult::RemovedS(node, reference) => {
                    RemoveResult::RemovedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::S(node)), Cow::Owned(self.size() - 1)).unwrap()), reference)
                },
                RemoveResult::RemovedZ(reference) => {
                    if self.size() == 1 {
                        RemoveResult::RemovedZ(reference)
                    }
                    else {
                        RemoveResult::RemovedC(Self::new(self.nodes.removed(flag.unwrap().flag, Cow::Owned(self.size() - 1)).unwrap()), reference)
                    }
                },
            },
            Err(_) => RemoveResult::NotFound
        }
    }
    
    pub(super) fn visit<Op: Clone>(&self, op: Op) where Op: Fn(&V) {
        for node in self.nodes.as_ref() {
            match node {
                MNode::C(cnode) => cnode.visit(op.clone()),
                MNode::L(lnode) => lnode.visit(op.clone()),
                MNode::S(snode) => snode.visit(op.clone()),
            }
        }
    }

}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V>> CNode<H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    pub(super) fn insert<'a, K: 'static, C: AsRef<K> + Into<V>>(&'a self, value: C, flag: Option<Flag<H, F>>, replace: bool) -> InsertResult<'a, H, F, V, M> where V: PartialEq<K> {
        match self.nodes.at(flag.as_ref().unwrap().flag.clone()) {
            Ok(node) => match node {
                MNode::C(cnode) => match cnode.insert(value, flag.as_ref().unwrap().next(), replace) {
                    InsertResult::Found(reference) => InsertResult::Found(reference),
                    InsertResult::InsertedC(cnode, reference) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::C(cnode)), Cow::Owned(self.size() + 1)).unwrap()), reference),
                    InsertResult::InsertedL(lnode, reference) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::L(lnode)), Cow::Owned(self.size() + 1)).unwrap()), reference),
                    InsertResult::InsertedS(_snode, _reference) => panic!(),
                },
                MNode::L(lnode) => match lnode::insert(&lnode, value, flag.as_ref().unwrap().next(), replace) {
                    InsertResult::Found(reference) => InsertResult::Found(reference),
                    InsertResult::InsertedC(cnode, reference) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::C(cnode)), Cow::Owned(self.size() + 1)).unwrap()), reference),
                    InsertResult::InsertedL(lnode, reference) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::L(lnode)), Cow::Owned(self.size() + 1)).unwrap()), reference),
                    InsertResult::InsertedS(_snode, _reference) => panic!(),
                },
                MNode::S(snode) => match snode::insert(&snode, value, flag.as_ref().unwrap().next(), replace) {
                    InsertResult::Found(reference) => InsertResult::Found(reference),
                    InsertResult::InsertedC(cnode, reference) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::C(cnode)), Cow::Owned(self.size() + 1)).unwrap()), reference),
                    InsertResult::InsertedL(lnode, reference) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::L(lnode)), Cow::Owned(self.size() + 1)).unwrap()), reference),
                    InsertResult::InsertedS(snode, reference) => InsertResult::InsertedC(Self::new(self.nodes.updated(flag.unwrap().flag, Cow::Owned(MNode::S(snode)), Cow::Owned(self.size() + 1)).unwrap()), reference),
                },
            },
            Err(_) => InsertResult::InsertedC(Self::new(self.nodes.inserted(flag.unwrap().flag, Cow::Owned(MNode::S(SNode::new(value.into()))), Cow::Owned(self.size() + 1)).unwrap()), None)
        }
    }
}

pub(super) fn transform<H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V>, ReduceT, ReduceOp, Op>(this: &CNode<H, F, V, M>, reduce_op: ReduceOp, op: Op) -> MNodeTransformResult<H, F, V, M, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(ReduceT, ReduceT) -> ReduceT + Clone,
    Op: Fn(&V) -> (MapTransformResult<V>, ReduceT) + Clone,
    <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug
{
    let mut changed = false;
    let mut size = 0;
    let mut bits_t = F::default();
    let mut values_t = Vec::default();
    let mut reduced = ReduceT::default();

    for index in 0..<F>::max_ones() {
        if let Ok(node) = this.nodes.at_bit_index(index) {
            match node.transform(reduce_op.clone(), op.clone()) {
                MNodeTransformResult::Unchanged(r) => {
                    size += node.size();
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(node.clone());
                    reduced = reduce_op(reduced, r);
                },
                MNodeTransformResult::C(cnode, r) => {
                    changed = true;
                    size += cnode.size();
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(MNode::C(cnode));
                    reduced = reduce_op(reduced, r);
                },
                MNodeTransformResult::L(lnode, r) => {
                    changed = true;
                    size += lnode.size();
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(MNode::L(lnode));
                    reduced = reduce_op(reduced, r);
                },
                MNodeTransformResult::S(snode, r) => {
                    changed = true;
                    size += 1;
                    bits_t = bits_t.bit_insert(<F>::nth_bit(index).unwrap()).unwrap();
                    values_t.push(MNode::S(snode));
                    reduced = reduce_op(reduced, r);
                },
                MNodeTransformResult::Removed(r) => {
                    changed = true;
                    reduced = reduce_op(reduced, r);
                },
            }
        }
    }

    if changed {
        match values_t.len() {
            0 => MNodeTransformResult::Removed(reduced),
            1 => match values_t.pop().unwrap() {
                MNode::C(_cnode) => MNodeTransformResult::C(CNode::<H, F, V, M>::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
                MNode::L(lnode) => MNodeTransformResult::L(lnode, reduced),
                MNode::S(snode) => MNodeTransformResult::S(snode, reduced),
            },
            _ => MNodeTransformResult::C(CNode::<H, F, V, M>::new(new_bit_indexed_array(bits_t, BitIndexedArrayVec::new(&values_t), size).unwrap()), reduced),
        }
    }
    else {
        MNodeTransformResult::Unchanged(reduced)
    }
}

pub(super) fn lift_to_cnode_and_insert<'a, H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V>>(this: LNodeNext::<V>, this_hash: H, value: V, value_flag: Option<Flag<H, F>>) -> InsertResult<'a, H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    if value_flag.as_ref().unwrap().hash_value() == this_hash {
        return InsertResult::InsertedL(LNode::new(value, this), None);
    }

    let this_mnode = match this {
        LNodeNext::L(lnode) => MNode::L(lnode),
        LNodeNext::S(snode) => MNode::S(snode),
    };

    let value_flag = value_flag.unwrap();
    let mut this_flag = Flag::from(this_hash);
    while this_flag.depth() != value_flag.depth() {
        this_flag = this_flag.next().unwrap();
    }

    InsertResult::InsertedC(lift_to_cnode_and_insert_recursion(this_mnode, this_flag, SNode::new(value), value_flag), None)
}

#[must_use]
fn lift_to_cnode_and_insert_recursion<H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V>>(this: MNode<H, F, V, M>, this_flag: Flag<H, F>, snode: Arc<SNode<V>>, snode_flag: Flag<H, F>) -> CNode<H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    let size = this.size() + 1;
    if this_flag.flag() == snode_flag.flag() {
        CNode::new(new_bit_indexed_array(this_flag.flag(), BitIndexedArrayVec::new(&[MNode::C(lift_to_cnode_and_insert_recursion(this, this_flag.next().unwrap(), snode, snode_flag.next().unwrap()))]), size).unwrap())
    }
    else  {
        let flags = this_flag.flag().bit_insert(snode_flag.flag()).unwrap();
        let values = if flags.bit_index(this_flag.flag).unwrap() == 0 {
            vec!(this, MNode::S(snode))
        } else {
            vec!(MNode::S(snode), this)
        };
        CNode::new(new_bit_indexed_array(flags, BitIndexedArrayVec::new(&values), size).unwrap())
    }
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: 'static> Clone for CNode<H, F, V, M> {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone()
        }
    }
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> Default for CNode<H, F, V, M> {
    fn default() -> Self {
        CNode::<H, F, V, M>::new(default_bit_indexed_array())
    }
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> Eq for CNode<H, F, V, M> {}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> PartialEq for CNode<H, F, V, M> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(Arc::as_ptr(&self.nodes) as *const u8, Arc::as_ptr(&other.nodes) as *const u8)
    }
}
