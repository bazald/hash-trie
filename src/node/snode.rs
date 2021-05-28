use crate::{flag::*, result::*, traits::*};
use super::{cnode::*, lnode::LNodeNext};
use alloc::{fmt::Debug, sync::Arc};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SNode<V: Value> {
    value: V,
}

impl <V: Value> SNode<V> {
    #[must_use]
    pub(super) fn new(value: V) -> Arc<Self> {
        Arc::new(Self {value})
    }

    pub(super) fn find<'a, K>(&'a self, key: &K) -> FindResult<'a, V> where V: PartialEq<K> {
        if self.value == *key {
            FindResult::Found(&self.value)
        }
        else {
            FindResult::NotFound
        }
    }
    
    pub(super) fn remove<'a, H: Hashword, F: Flagword<H>, M: 'static, K>(&'a self, key: &K) -> RemoveResult<'a, H, F, V, M> where V: PartialEq<K> {
        if self.value == *key {
            RemoveResult::RemovedZ(&self.value)
        }
        else {
            RemoveResult::NotFound
        }
    }

    pub(super) fn remove_from_snode<'a, K>(&'a self, key: &K) -> SNodeRemoveResult<'a, V> where V: PartialEq<K> {
        if self.value == *key {
            SNodeRemoveResult::RemovedZ(&self.value)
        }
        else {
            SNodeRemoveResult::NotFound
        }
    }

    pub(super) fn visit<Op>(&self, op: Op) where Op: Fn(&V) {
        op(&self.value);
    }

}

pub(super) fn insert<'a, H: Hashword, F: Flagword<H>, K: 'static, V: Value, C: AsRef<K> + Into<V>, M: HasherBv<H, V>>(this: &'a Arc<SNode<V>>, value: C, value_flag: Option<Flag<H, F>>, replace: bool) -> InsertResult<'a, H, F, V, M> where V: PartialEq<K>, <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    if this.value == *value.as_ref() {
        if replace {
            InsertResult::InsertedS(this.clone(), Some(&this.value))
        }
        else {
            InsertResult::Found(&this.value)
        }
    }
    else {
        lift_to_cnode_and_insert(LNodeNext::S(this.clone()), M::default().hash(&this.value), value.into(), value_flag)
    }
}

pub(super) fn transform<V: Value, ReduceT, Op>(this: &Arc<SNode<V>>, op: Op) -> SNodeTransformResult<V, ReduceT>
    where
    ReduceT: Default,
    Op: Fn(&V) -> (MapTransformResult<V>, ReduceT)
{
    let (t, r) = op(&this.value);
    match t {
        MapTransformResult::Unchanged => SNodeTransformResult::Unchanged(r),
        MapTransformResult::Changed(v) => SNodeTransformResult::S(SNode::new(v), r),
        MapTransformResult::Removed => SNodeTransformResult::Removed(r),
    }
}
