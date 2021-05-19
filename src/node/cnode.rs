use crate::{bit_indexed_array::*, flag::*, result::*, traits::*};
use super::{lnode::{self, *}, mnode::*, snode::{self, *}};
use alloc::{boxed::Box, borrow::Cow, fmt::Debug, sync::Arc};

#[derive(Debug)]
pub(crate) struct CNode <H: Hashword, F: Flagword<H>, V: Value, M: 'static> {
    nodes: Arc<dyn BitIndexedArray::<F, MNode<H, F, V, M>, usize>>,
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: 'static> CNode<H, F, V, M> {
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

impl <H: Hashword, F: Flagword<H>, V: Value, M: 'static> Default for CNode<H, F, V, M> {
    fn default() -> Self {
        CNode::<H, F, V, M>::new(default_bit_indexed_array())
    }
}
