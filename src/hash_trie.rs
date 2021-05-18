use crate::{flag::*, traits::*, node::*, result::*, HashTrieError};
use alloc::{fmt::Debug};
use core::hash::Hash;

#[derive(Debug)]
pub(crate) struct HashTrie <B: Bits, V: Value, H: HasherBv<B, V>> {
    root: MNode<B, V, H>,
}

impl <B: Bits, V: Value, H: HasherBv<B, V>> HashTrie<B, V, H> {
    pub(crate) fn new() -> Self {
        Self {
            root: MNode::<B, V, H>::default()
        }
    }

    fn singleton(mnode: MNode<B, V, H>) -> Self {
        Self {
            root: mnode
        }
    }

    pub(crate) fn find<K: Hash + HashLike<V>>(&self, key: &K) -> Result<&V, HashTrieError> where V: PartialEq<K>, H: HasherBv<B, K> {
        match self.root.find(key, Some(Flag::new(H::default().hash(key)))) {
            FindResult::NotFound => Err(HashTrieError::NotFound),
            FindResult::Found(found) => Ok(found)
        }
    }

    pub(crate) fn insert<'a, K: Hash + HashLike<V> + 'static, C: AsRef<K> + Into<V>>(&'a self, value: C, replace: bool) -> Result<(Self, Option<&V>), &V> where V: PartialEq<K>, H: HasherBv<B, K> {
        let flag = Flag::from(H::default().hash(value.as_ref()));
        match self.root.insert(value, Some(flag), replace) {
            InsertResult::Found(found) => Err(found),
            InsertResult::InsertedC(cnode, replaced) => Ok((Self::singleton(MNode::C(cnode)), replaced)),
            InsertResult::InsertedL(lnode, replaced) => Ok((Self::singleton(MNode::L(lnode)), replaced)),
            InsertResult::InsertedS(_snode, _replaced) => panic!(),
        }
    }

    pub(crate) fn remove<K: Hash + HashLike<V> + 'static>(&self, key: &K) -> Result<(Self, &V), HashTrieError> where V: PartialEq<K>, H: HasherBv<B, K> {
        match self.root.remove(key, Some(Flag::from(H::default().hash(key)))) {
            RemoveResult::NotFound => Err(HashTrieError::NotFound),
            RemoveResult::RemovedC(cnode, removed) => Ok((Self::singleton(MNode::C(cnode)), removed)),
            RemoveResult::RemovedL(lnode, removed) => Ok((Self::singleton(MNode::L(lnode)), removed)),
            RemoveResult::RemovedS(snode, removed) => Ok((Self::singleton(MNode::S(snode)), removed)),
            RemoveResult::RemovedZ(removed) => Ok((Self::default(), removed))
        }
    }
}

impl <B: Bits, V: Value, H: HasherBv<B, V>> Clone for HashTrie<B, V, H> {
    fn clone(&self) -> Self {
        Self::singleton(self.root.clone())
    }
}

impl <B: Bits, V: Value, H: HasherBv<B, V>> Default for HashTrie<B, V, H> {
    fn default() -> Self {
        Self::new()
    }
}
