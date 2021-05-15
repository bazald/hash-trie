use crate::{flag::*, result::*, traits::*};
use super::{cnode::*, snode::*};
use alloc::{fmt::Debug, sync::*};

#[derive(Clone, Debug)]
pub(super) enum LNodeNext<V: Value> {
    L(Arc<LNode<V>>),
    S(Arc<SNode<V>>),
}

#[derive(Clone, Debug)]
pub(crate) struct LNode<V: Value> {
    value: V,
    next: LNodeNext<V>,
    size: usize,
}

impl<V: Value> LNode<V> {
    pub(super) fn new(value: V, next: LNodeNext<V>) -> Arc<Self> {
        let size = 1 + match &next {
            LNodeNext::L(lnode) => lnode.size,
            LNodeNext::S(_snode) => 1,
        };
        Arc::new(Self {
            value,
            next,
            size,
        })
    }

    pub(super) fn size(&self) -> usize {
        self.size
    }

    pub(super) fn find<'a, K>(&'a self, key: &K) -> FindResult<'a, V> where V: PartialEq<K> {
        if self.value == *key {
            FindResult::Found(&self.value)
        }
        else {
            match &self.next {
                LNodeNext::L(lnode) => lnode.find(key),
                LNodeNext::S(snode) => snode.find(key),
            }
        }
    }

    pub(super) fn remove<'a, B: Bits, K, H: 'static>(&'a self, key: &K) -> RemoveResult<'a, B, V, H> where V: PartialEq<K> {
        match self.remove_from_lnode(key) {
            LNodeRemoveResult::NotFound => RemoveResult::NotFound,
            LNodeRemoveResult::RemovedL(lnode, reference) => RemoveResult::RemovedL(lnode, reference),
            LNodeRemoveResult::RemovedS(snode, reference) => RemoveResult::RemovedS(snode, reference),
        }
    }
    
    fn remove_from_lnode<'a, K>(&'a self, key: &K) -> LNodeRemoveResult<'a, V> where V: PartialEq<K> {
        if self.value == *key {
            match &self.next {
                LNodeNext::L(lnode) => LNodeRemoveResult::RemovedL(lnode.clone(), &self.value),
                LNodeNext::S(snode) => LNodeRemoveResult::RemovedS(snode.clone(), &self.value),
            }
        }
        else {
            match &self.next {
                LNodeNext::L(lnode) => match lnode.remove_from_lnode(key) {
                    LNodeRemoveResult::NotFound => LNodeRemoveResult::NotFound,
                    LNodeRemoveResult::RemovedL(lnode, reference) => LNodeRemoveResult::RemovedL(LNode::new(self.value.clone(), LNodeNext::L(lnode)), reference),
                    LNodeRemoveResult::RemovedS(snode, reference) => LNodeRemoveResult::RemovedL(LNode::new(self.value.clone(), LNodeNext::S(snode)), reference),
                },
                LNodeNext::S(snode) => match snode.remove_from_snode(key) {
                    SNodeRemoveResult::NotFound => LNodeRemoveResult::NotFound,
                    SNodeRemoveResult::RemovedZ(reference) => LNodeRemoveResult::RemovedS(SNode::new(self.value.clone()), reference),
                }
            }
        }
    }
}

pub(super) fn insert<'a, B: Bits, K: 'static, V: Value, C: AsRef<K> + Into<V>, H: HasherBv<B, V>>(this: &'a Arc<LNode<V>>, value: C, value_flag: Option<Flag<B>>, replace: bool) -> InsertResult<'a, B, V, H> where V: PartialEq<K> {
    match this.find(value.as_ref()) {
        FindResult::Found(reference) => if replace {
            match this.remove_from_lnode(value.as_ref()) {
                LNodeRemoveResult::RemovedL(lnode, reference) => InsertResult::InsertedL(LNode::new(value.into(), LNodeNext::L(lnode)), Some(reference)),
                LNodeRemoveResult::RemovedS(snode, reference) => InsertResult::InsertedL(LNode::new(value.into(), LNodeNext::S(snode)), Some(reference)),
                LNodeRemoveResult::NotFound => panic!(),
            }
        }
        else {
            InsertResult::Found(reference)
        },
        FindResult::NotFound => {
            lift_to_cnode_and_insert(LNodeNext::L(this.clone()), H::default().hash(&this.value), value.into(), value_flag)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    macro_rules! lnode {
        ( $value:expr, $snode:expr ) => {
            {
                LNode::new($value, LNodeNext::S(SNode::new($snode)))
            }
        };
        ( $value:expr, $($rest:expr),+ ) => {
            {
                LNode::new($value, LNodeNext::L(lnode!($($rest),*)))
            }
        };
    }

    #[test]
    fn lnode_insert_3() {
        let node = lnode!(3, 2, 1);
        assert_eq!(node.size, 3);
        assert_found_eq!(node.find(&1), 1);
        assert_found_eq!(node.find(&2), 2);
        assert_found_eq!(node.find(&3), 3);
        assert_found_none!(node.find(&4));
    }

}
