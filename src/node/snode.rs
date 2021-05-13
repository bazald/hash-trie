use crate::{flag::*, result::*, traits::*};
use super::{cnode::*, lnode::LNodeNext};
use alloc::{borrow::Cow, fmt::Debug, sync::Arc};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SNode<V: Value> {
    value: V,
}

impl <V: Value> SNode<V> {
    pub(super) fn new(value: V) -> Arc<Self> {
        Arc::new(Self {value})
    }

    pub(super) fn get(&self) -> &V {
        &self.value
    }

    pub(super) fn find<'a>(&'a self, value: &V) -> FindResult<'a, V> {
        if self.value == *value {
            FindResult::Found(&self.value)
        }
        else {
            FindResult::NotFound
        }
    }

    pub(super) fn remove<'a>(&'a self, value: &V) -> SNodeRemoveResult<'a, V> {
        if self.value == *value {
            SNodeRemoveResult::RemovedZ(&self.value)
        }
        else {
            SNodeRemoveResult::NotFound
        }
    }
}

pub(super) fn insert<'a, B: Bits, V: Value, H: HasherBv<B, V>>(this: &'a Arc<SNode<V>>, value: Cow<V>, value_flag: Option<Flag<B>>) -> InsertResult<'a, B, V, H> {
    if this.value == *value {
        InsertResult::Found(&this.value)
    }
    else {
        lift_to_cnode_and_insert(LNodeNext::S(this.clone()), H::default().hash(&this.value), value, value_flag)
    }
}
