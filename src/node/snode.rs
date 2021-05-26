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

    pub(super) fn visit<Op>(&self, op: Op) where Op: Fn(&'_ V) {
        op(&self.value);
    }

    pub(super) fn transform<ReduceT, Op>
        (&self, op: Op) -> SNodeTransformResult<V, ReduceT>
        where
        Self: Sized,
        ReduceT: Default,
        Op: Fn(&V) -> (Option<V>, ReduceT)
    {
        let (v, r) = op(&self.value);
        match v {
            Some(v) => SNodeTransformResult::S(Self::new(v), r),
            None => SNodeTransformResult::Z(r),
        }
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
