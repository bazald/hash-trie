use crate::bit_indexed_array::*;
use super::flag::*;
use alloc::fmt::Debug;
use core::{hash::{Hash, Hasher}, mem, ops::*};

pub trait Bits: AsUsize + BitAnd + BitContains + BitIndex + BitInsert + BitRemove + Clone + CountOnes + Debug + Default + From<<Self as BitAnd>::Output> + From<<Self as Shr<usize>>::Output> + LogB + MaskLogB + NthBit + PartialEq + Shr<usize> + Send + Sync + 'static {}
impl <B: AsUsize + BitAnd + BitContains + BitIndex + BitInsert + BitRemove + Clone + CountOnes + Debug + Default + From<<Self as BitAnd>::Output> + From<<Self as Shr<usize>>::Output> + LogB + MaskLogB + NthBit + PartialEq + Shr<usize> + Send + Sync + 'static> Bits
for B where B: AsUsize + BitAnd + BitContains + BitIndex + BitInsert + BitRemove + Clone + CountOnes + Debug + Default + From<<Self as BitAnd>::Output> + From<<Self as Shr<usize>>::Output> + LogB + MaskLogB + NthBit + PartialEq + Shr<usize> + Send + Sync + 'static {}

pub trait Value: Clone + Debug + Eq + PartialEq + Hash + Send + Sync + 'static {}
impl <T: Clone + Debug + Eq + PartialEq + Hash + Send + Sync + 'static> Value
for T where T: Clone + Debug + Eq + PartialEq + Hash + Send + Sync + 'static {}

pub trait HasherBv<B, V>: Default + 'static {
    fn hash(&self, value: &V) -> B;
}
impl <V: Default + 'static, H: Default + Hasher + 'static> HasherBv<u64, V> for H {
    fn hash(&self, value: &V) -> u64 {
        let mut hasher = H::default();
        hasher.write(unsafe { core::slice::from_raw_parts(value as *const V as *const u8, mem::size_of::<V>()) });
        hasher.finish()
    }
}
