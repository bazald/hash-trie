use crate::{flag::*, traits::*, node::*, result::*, HashTrieError};
use alloc::{fmt::Debug};
use core::hash::Hash;

#[derive(Debug)]
pub(crate) struct HashTrie <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> {
    root: MNode<H, F, V, M>,
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> HashTrie<H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            root: MNode::<H, F, V, M>::default()
        }
    }

    #[must_use]
    fn singleton(mnode: MNode<H, F, V, M>) -> Self {
        Self {
            root: mnode
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.root.size()
    }

    pub(crate) fn find<K: Hash + HashLike<V>>(&self, key: &K) -> Result<&V, HashTrieError> where V: PartialEq<K>, M: HasherBv<H, K>, <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
        match self.root.find(key, Some(Flag::new(M::default().hash(key)))) {
            FindResult::NotFound => Err(HashTrieError::NotFound),
            FindResult::Found(found) => Ok(found)
        }
    }

    pub(crate) fn insert<'a, K: Hash + HashLike<V> + 'static, C: AsRef<K> + Into<V>>(&'a self, value: C, replace: bool) -> Result<(Self, Option<&V>), &V> where V: PartialEq<K>, M: HasherBv<H, K> {
        let flag = Flag::from(M::default().hash(value.as_ref()));
        match self.root.insert(value, Some(flag), replace) {
            InsertResult::Found(found) => Err(found),
            InsertResult::InsertedC(cnode, replaced) => Ok((Self::singleton(MNode::C(cnode)), replaced)),
            InsertResult::InsertedL(lnode, replaced) => Ok((Self::singleton(MNode::L(lnode)), replaced)),
            InsertResult::InsertedS(_snode, _replaced) => panic!(),
        }
    }

    pub(crate) fn remove<K: Hash + HashLike<V> + 'static>(&self, key: &K) -> Result<(Self, &V), HashTrieError> where V: PartialEq<K>, M: HasherBv<H, K> {
        match self.root.remove(key, Some(Flag::from(M::default().hash(key)))) {
            RemoveResult::NotFound => Err(HashTrieError::NotFound),
            RemoveResult::RemovedC(cnode, removed) => Ok((Self::singleton(MNode::C(cnode)), removed)),
            RemoveResult::RemovedL(lnode, removed) => Ok((Self::singleton(MNode::L(lnode)), removed)),
            RemoveResult::RemovedS(snode, removed) => Ok((Self::singleton(MNode::S(snode)), removed)),
            RemoveResult::RemovedZ(removed) => Ok((Self::default(), removed))
        }
    }
    
    pub(crate) fn visit<Op: Clone>(&self, op: Op) where Op: Fn(&V) {
        self.root.visit(op);
    }

    pub(crate) fn transform<ReduceT, ReduceOp, Op>
        (&self, reduce_op: ReduceOp, op: Op) -> (Self, ReduceT)
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(ReduceT, ReduceT) -> ReduceT + Clone,
        Op: Fn(&V) -> (MapTransformResult<V>, ReduceT) + Clone
    {
        match self.root.transform(reduce_op, op) {
            MNodeTransformResult::Unchanged(reduced) => (Self::singleton(self.root.clone()), reduced),
            MNodeTransformResult::C(cnode, reduced) => (Self::singleton(MNode::C(cnode)), reduced),
            MNodeTransformResult::L(lnode, reduced) => (Self::singleton(MNode::L(lnode)), reduced),
            MNodeTransformResult::S(snode, reduced) => (Self::singleton(MNode::S(snode)), reduced),
            MNodeTransformResult::Removed(reduced) => (Self::default(), reduced),
        }
    }

}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> Clone for HashTrie<H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn clone(&self) -> Self {
        Self::singleton(self.root.clone())
    }
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> Default for HashTrie<H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn default() -> Self {
        Self::new()
    }
}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> Eq for HashTrie<H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {}

impl <H: Hashword, F: Flagword<H>, V: Value, M: HasherBv<H, V> + 'static> PartialEq for HashTrie<H, F, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
}
