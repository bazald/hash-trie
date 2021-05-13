use crate::traits::*;
use alloc::fmt::Debug;
use core::{mem, ops::*};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Flag <B> {
    hash_value: B,
    depth: usize,
    pub(super) flag: B,
}

impl <B: AsUsize + BitAnd + Clone + From<<B as BitAnd>::Output> + From<<B as Shr<usize>>::Output> + LogB + MaskLogB + NthBit + Shr<usize>> Flag<B> {
    pub(crate) fn new(hash_value: B) -> Self {
        Flag {
            depth: 0,
            flag: B::nth_bit(B::from(hash_value.clone().bitand(B::mask_log_b())).as_usize()).unwrap(),
            hash_value,
        }
    }

    pub(crate) fn hash_value(&self) -> B {
        self.hash_value.clone()
    }

    pub(crate) fn depth(&self) -> usize {
        self.depth
    }

    pub(crate) fn flag(&self) -> B {
        self.flag.clone()
    }

    pub(crate) fn next(&self) -> Option<Flag<B>> {
        if self.depth * B::log_b() >= 8 * mem::size_of::<B>() {
            None
        }
        else {
            Some(Flag {
                hash_value: self.hash_value.clone(),
                depth: self.depth + 1,
                flag: B::nth_bit(B::from(B::from(self.hash_value.clone().shr((self.depth + 1) * B::log_b())).bitand(B::mask_log_b())).as_usize()).unwrap()
            })
        }
    }
}

impl <B: AsUsize + BitAnd + Clone + From<<B as BitAnd>::Output> + From<<B as Shr<usize>>::Output> + LogB + MaskLogB + NthBit + Shr<usize>> From<B> for Flag<B> {
    fn from(bits: B) -> Flag<B> {
        Self::new(bits)
    }
}
