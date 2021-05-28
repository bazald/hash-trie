use crate::{flag::*, result::*, traits::*};
use super::{cnode::*, snode::{self, *}};
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
    #[must_use]
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

    #[must_use]
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

    pub(super) fn remove<'a, H: Hashword, F: Flagword<H>, M: 'static, K>(&'a self, key: &K) -> RemoveResult<'a, H, F, V, M> where V: PartialEq<K> {
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
    
    pub(super) fn visit<Op>(&self, op: Op) where Op: Fn(&V) {
        op(&self.value);
        match &self.next {
            LNodeNext::L(lnode) => lnode.visit(op),
            LNodeNext::S(snode) => snode.visit(op),
        }
    }

}

pub(super) fn insert<'a, H: Hashword, F: Flagword<H>, K: 'static, V: Value, C: AsRef<K> + Into<V>, M: HasherBv<H, V>>(this: &'a Arc<LNode<V>>, value: C, value_flag: Option<Flag<H, F>>, replace: bool) -> InsertResult<'a, H, F, V, M> where V: PartialEq<K>, <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
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
            lift_to_cnode_and_insert(LNodeNext::L(this.clone()), M::default().hash(&this.value), value.into(), value_flag)
        }
    }
}

pub(super) fn transform<V: Value, ReduceT, ReduceOp, Op>(this: &Arc<LNode<V>>, reduce_op: ReduceOp, op: Op) -> LNodeTransformResult<V, ReduceT>
where
    ReduceT: Default,
    ReduceOp: Fn(ReduceT, ReduceT) -> ReduceT + Clone,
    Op: Fn(&V) -> (MapTransformResult<V>, ReduceT) + Clone
{
    let next = match &this.next {
        LNodeNext::L(lnode) => transform(&lnode, reduce_op.clone(), op.clone()),
        LNodeNext::S(snode) => match snode::transform(&snode, op.clone()) {
            SNodeTransformResult::Unchanged(r) => LNodeTransformResult::Unchanged(r),
            SNodeTransformResult::S(snode, r) => LNodeTransformResult::S(snode, r),
            SNodeTransformResult::Removed(r) => LNodeTransformResult::Removed(r),
        },
    };

    let (t, r) = op(&this.value);

    match t {
        MapTransformResult::Unchanged => match next {
            LNodeTransformResult::Unchanged(rn) => LNodeTransformResult::Unchanged(reduce_op(r, rn)),
            LNodeTransformResult::L(lnode, rn) => LNodeTransformResult::L(LNode::new(this.value.clone(), LNodeNext::L(lnode)), reduce_op(r, rn)),
            LNodeTransformResult::S(snode, rn) => LNodeTransformResult::L(LNode::new(this.value.clone(), LNodeNext::S(snode)), reduce_op(r, rn)),
            LNodeTransformResult::Removed(rn) => LNodeTransformResult::S(SNode::new(this.value.clone()), reduce_op(r, rn)),
        },
        MapTransformResult::Changed(v) => match next {
            LNodeTransformResult::Unchanged(rn) => LNodeTransformResult::L(LNode::new(v, this.next.clone()), reduce_op(r, rn)),
            LNodeTransformResult::L(lnode, rn) => LNodeTransformResult::L(LNode::new(v, LNodeNext::L(lnode)), reduce_op(r, rn)),
            LNodeTransformResult::S(snode, rn) => LNodeTransformResult::L(LNode::new(v, LNodeNext::S(snode)), reduce_op(r, rn)),
            LNodeTransformResult::Removed(rn) => LNodeTransformResult::S(SNode::new(v), reduce_op(r, rn)),
        },
        MapTransformResult::Removed => match next {
            LNodeTransformResult::Unchanged(rn) => match &this.next {
                LNodeNext::L(lnode) => LNodeTransformResult::L(lnode.clone(), reduce_op(r, rn)),
                LNodeNext::S(snode) => LNodeTransformResult::S(snode.clone(), reduce_op(r, rn)),
            },
            LNodeTransformResult::L(lnode, rn) => LNodeTransformResult::L(lnode, reduce_op(r, rn)),
            LNodeTransformResult::S(snode, rn) => LNodeTransformResult::S(snode, reduce_op(r, rn)),
            LNodeTransformResult::Removed(rn) => LNodeTransformResult::Removed(reduce_op(r, rn)),
        },
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
