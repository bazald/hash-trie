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
