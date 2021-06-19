use crate::traits::*;
use alloc::fmt::Debug;

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
    pub(crate) fn new_at_depth(hash_value: H, depth: usize) -> Option<Self> {
        if depth * F::log_b() >= 8 * <H>::max_ones() {
            None
        }
        else {
            Some(Flag {
                hash_value: hash_value.clone(),
                depth: depth,
                flag: F::nth_bit(F::try_from(H::from(hash_value.shr(depth * F::log_b())).bitand(F::mask_log_b())).unwrap().as_usize()).unwrap()
            })
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
    pub(crate) fn next(&self) -> Option<Self> {
        Self::new_at_depth(self.hash_value().clone(), self.depth + 1)
    }
}

impl <H: Hashword, F: Flagword<H>> From<H> for Flag<H, F> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn from(bits: H) -> Flag<H, F> {
        Self::new(bits)
    }
}
