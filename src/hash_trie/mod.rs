mod flag;
mod traits;
#[macro_use]
mod mnode;
mod cnode;
mod lnode;
mod snode;

use cnode::*;
use flag::*;
use mnode::*;
use traits::*;

use alloc::{borrow::Cow, fmt::Debug};

#[derive(Clone, Debug)]
pub struct HashTrie <B: Bits, V: Value, H: HasherBv<B, V>> {
    root: MNode<B, V, H>,
}

impl <B: Bits, V: Value, H: HasherBv<B, V>> HashTrie<B, V, H> {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            root: MNode::C(CNode::<B, V, H>::default())
        }
    }

    fn singleton(mnode: MNode<B, V, H>) -> Self {
        Self {
            root: mnode
        }
    }

    #[allow(dead_code)]
    fn find(&self, value: &V) -> Result<&V, ()> {
        match self.root.find(value, Some(Flag::new(H::default().hash(value)))) {
            FindResult::NotFound => Err(()),
            FindResult::Found(found) => Ok(found)
        }
    }

    #[allow(dead_code)]
    fn insert(&self, value: Cow<V>) -> Result<Self, &V> {
        let flag = Flag::from(H::default().hash(value.as_ref()));
        match self.root.insert(value, Some(flag)) {
            InsertResult::Found(found) => Err(found),
            InsertResult::InsertedC(cnode) => Ok(Self::singleton(MNode::C(cnode))),
            InsertResult::InsertedL(lnode) => Ok(Self::singleton(MNode::L(lnode))),
        }
    }

    #[allow(dead_code)]
    fn remove(&self, value: &V) -> Result<(Self, Option<&V>), ()> {
        match self.root.remove(value, Some(Flag::from(H::default().hash(value)))) {
            RemoveResult::NotFound => Err(()),
            RemoveResult::RemovedC(cnode, removed) => Ok((Self::singleton(MNode::C(cnode)), Some(removed))),
            RemoveResult::RemovedL(lnode, removed) => Ok((Self::singleton(MNode::L(lnode)), Some(removed))),
            RemoveResult::RemovedS(snode, removed) => Ok((Self::singleton(MNode::S(snode)), Some(removed))),
            RemoveResult::RemovedZ(removed) => Ok((Self::default(), Some(removed)))
        }
    }
}

impl <B: Bits, V: Value, H: HasherBv<B, V>> Default for HashTrie<B, V, H> {
    fn default() -> Self {
        Self {
            root: MNode::<B, V, H>::C(CNode::<B, V, H>::default())
        }
    }
}
