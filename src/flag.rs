use crate::traits::*;
use alloc::fmt::Debug;
use core::mem;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Flag <H, F> {
    hash_value: H,
    depth: usize,
    pub(super) flag: F,
}

impl <H: Hashword, F: Flagword<H>> Flag<H, F> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    #[must_use]
    pub(crate) fn new(hash_value: H) -> Self {
        Flag {
            depth: 0,
            flag: F::nth_bit(F::try_from(hash_value.clone().bitand(F::mask_log_b())).unwrap().as_usize()).unwrap(),
            hash_value,
        }
    }

    #[must_use]
    pub(crate) fn hash_value(&self) -> H {
        self.hash_value.clone()
    }

    #[must_use]
    pub(crate) fn depth(&self) -> usize {
        self.depth
    }

    #[must_use]
    pub(crate) fn flag(&self) -> F {
        self.flag.clone()
    }

    #[must_use]
    pub(crate) fn next(&self) -> Option<Flag<H, F>> {
        if self.depth * F::log_b() >= 8 * mem::size_of::<H>() {
            None
        }
        else {
            Some(Flag {
                hash_value: self.hash_value.clone(),
                depth: self.depth + 1,
                flag: F::nth_bit(F::try_from(H::from(self.hash_value.clone().shr((self.depth + 1) * F::log_b())).bitand(F::mask_log_b())).unwrap().as_usize()).unwrap()
            })
        }
    }
}

impl <H: Hashword, F: Flagword<H>> From<H> for Flag<H, F> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn from(bits: H) -> Flag<H, F> {
        Self::new(bits)
    }
}
