use crate::{flag::*, traits::*, node::*, result::*, HashTrieError};
use alloc::{fmt::Debug};
use core::hash::Hash;

#[derive(Debug)]
pub(crate) struct HashTrie <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> {
    root: MNode<H, F, K, V, M>,
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> HashTrie<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            root: MNode::<H, F, K, V, M>::default()
        }
    }

    #[must_use]
    fn singleton(mnode: MNode<H, F, K, V, M>) -> Self {
        Self {
            root: mnode
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.root.size()
    }

    pub(crate) fn find<'a, L: Key + HashLike<K>>(&'a self, key: &L) -> Result<(&'a K, &'a V), HashTrieError> where K: PartialEq<L>, M: HasherBv<H, L>, <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
        match self.root.find(key, Some(Flag::new(M::default().hash(key)))) {
            FindResult::NotFound => Err(HashTrieError::NotFound),
            FindResult::Found(key, value) => Ok((key, value))
        }
    }

    pub(crate) fn insert<'a, L: Key + Into<K> + Hash + HashLike<K>, W: Into<V>>(&'a self, key: L, value: W, replace: bool) -> Result<(Self, *const K, *const V), (&'a K, &'a V)>
    where
        K: HashLike<L>,
        K: PartialEq<L>,
        M: HasherBv<H, L>
    {
        let flag = Flag::from(M::default().hash(&key));
        match self.root.insert(key, value, Some(flag), replace) {
            InsertResult::Found(key, value) => Err((key, value)),
            InsertResult::InsertedC(cnode, key, value) => Ok((Self::singleton(MNode::C(cnode)), key, value)),
            InsertResult::InsertedL(lnode, key, value) => Ok((Self::singleton(MNode::L(lnode)), key, value)),
            InsertResult::InsertedS(snode, key, value) => Ok((Self::singleton(MNode::S(snode)), key, value)),
        }
    }

    pub(crate) fn remove<'a, L: Key + HashLike<K>>(&'a self, key: &L) -> Result<(Self, &'a K, &'a V), HashTrieError> where K: PartialEq<L>, M: HasherBv<H, L> {
        match self.root.remove(key, Some(Flag::from(M::default().hash(key)))) {
            RemoveResult::NotFound => Err(HashTrieError::NotFound),
            RemoveResult::RemovedC(cnode, key, value) => Ok((Self::singleton(MNode::C(cnode)), key, value)),
            RemoveResult::RemovedL(lnode, key, value) => Ok((Self::singleton(MNode::L(lnode)), key, value)),
            RemoveResult::RemovedS(snode, key, value) => Ok((Self::singleton(MNode::S(snode)), key, value)),
            RemoveResult::RemovedZ(key, value) => Ok((Self::default(), key, value))
        }
    }
    
    pub(crate) fn visit<Op: Clone>(&self, op: Op) where Op: Fn(&K, &V) {
        self.root.visit(op);
    }

    pub(crate) fn transform<S: Key, X: Value, ReduceT, ReduceOp, Op>
        (&self, reduce_op: ReduceOp, op: Op) -> (HashTrie<H, F, S, X, M>, ReduceT)
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        Op: Fn(&K, &V) -> MapTransformResult<S, X, ReduceT> + Clone,
        K: HashLike<S>,
        K: PartialEq<S>,
        M: HasherBv<H, S>,
    {
        match self.root.transform(reduce_op, op) {
            MNodeTransformResult::C(cnode, reduced) => (HashTrie::<H, F, S, X, M>::singleton(MNode::C(cnode)), reduced),
            MNodeTransformResult::L(lnode, reduced) => (HashTrie::<H, F, S, X, M>::singleton(MNode::L(lnode)), reduced),
            MNodeTransformResult::S(snode, reduced) => (HashTrie::<H, F, S, X, M>::singleton(MNode::S(snode)), reduced),
            MNodeTransformResult::Removed(reduced) => (HashTrie::<H, F, S, X, M>::default(), reduced),
        }
    }

    pub(crate) fn joint_transform<L: Key, W: Value, S: Key, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>
        (&self, right: &HashTrie<H, F, L, W, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp) -> (HashTrie<H, F, S, X, M>, ReduceT)
        where
        Self: Sized,
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
    {
        match self.root.joint_transform(&right.root, reduce_op, both_op, left_op, right_op, 0) {
            MNodeTransformResult::C(cnode, reduced) => (HashTrie::<H, F, S, X, M>::singleton(MNode::C(cnode)), reduced),
            MNodeTransformResult::L(lnode, reduced) => (HashTrie::<H, F, S, X, M>::singleton(MNode::L(lnode)), reduced),
            MNodeTransformResult::S(snode, reduced) => (HashTrie::<H, F, S, X, M>::singleton(MNode::S(snode)), reduced),
            MNodeTransformResult::Removed(reduced) => (HashTrie::<H, F, S, X, M>::default(), reduced),
        }
    }

}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> Clone for HashTrie<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn clone(&self) -> Self {
        Self::singleton(self.root.clone())
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> Default for HashTrie<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn default() -> Self {
        Self::new()
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> Eq for HashTrie<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> PartialEq for HashTrie<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
}
