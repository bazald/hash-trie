use alloc::fmt::Debug;
use core::{hash::{Hash, Hasher}, mem, ops::*};

/// `BitError` enumerates possible error conditions when bitops are used "incorrectly."
#[derive(Debug)]
pub enum BitError {
    /// `BitError::CountNotEqualToOne` indicates a word representing a bit contains either 2 or more bits or 0 bits.
    CountNotEqualToOne,
    /// `BitError::Found` indicates a bit that is supposed to be absent is present.
    Found,
    /// `BitError::NotFound` indicates a bit that is supposed to be present is absent.
    NotFound,
    /// `BitError::Range` indicates an index exceeding the word size was used.
    Range,
}

macro_rules! bit_count_one {
    ( $bit:expr) => {
        if $bit.count_ones() != 1 {
            return Err(BitError::CountNotEqualToOne);
        }
    };
}

macro_rules! bit_found {
    ( $self:expr, $bit:expr) => {
        bit_count_one!($bit);
        if $self & $bit == 0 {
            return Err(BitError::NotFound);
        }
    };
}

macro_rules! bit_not_found {
    ( $self:expr, $bit:expr) => {
        bit_count_one!($bit);
        if $self & $bit == 1 {
            return Err(BitError::Found);
        }
    };
}

macro_rules! bit_in_range {
    ( $type:ty, $index:expr) => {
        if $index >= 8 * mem::size_of::<$type>() {
            return Err(BitError::Range);
        }
    };
}

/// `AsUsize` supports conversion to usize for values within the word's index range.
pub trait AsUsize {
    /// Simply cast or convert the value to a usize.
    fn as_usize(&self) -> usize;
}
impl AsUsize for u8 { fn as_usize(&self) -> usize {*self as usize} }
impl AsUsize for u16 { fn as_usize(&self) -> usize {*self as usize} }
impl AsUsize for u32 { fn as_usize(&self) -> usize {*self as usize} }
impl AsUsize for u64 { fn as_usize(&self) -> usize {*self as usize} }
impl AsUsize for u128 { fn as_usize(&self) -> usize {*self as usize} }
impl AsUsize for usize { fn as_usize(&self) -> usize {*self} }

/// `BitContains` supports testing to see if a bit is present in the word or not.
pub trait BitContains {
    /// Check whether the word contains the bit or return BitError::CountNotEqualToOne if the bit is invalid.
    /// 
    /// e.g. `0b110110.bit_contains(0b100) == true`
    fn bit_contains(&self, bit: Self) -> Result<bool, BitError>;
}
impl BitContains for u8 { fn bit_contains(&self, bit: Self) -> Result<bool, BitError> {bit_count_one!(bit); Ok((self & bit) != 0)} }
impl BitContains for u16 { fn bit_contains(&self, bit: Self) -> Result<bool, BitError> {bit_count_one!(bit); Ok((self & bit) != 0)} }
impl BitContains for u32 { fn bit_contains(&self, bit: Self) -> Result<bool, BitError> {bit_count_one!(bit); Ok((self & bit) != 0)} }
impl BitContains for u64 { fn bit_contains(&self, bit: Self) -> Result<bool, BitError> {bit_count_one!(bit); Ok((self & bit) != 0)} }
impl BitContains for u128 { fn bit_contains(&self, bit: Self) -> Result<bool, BitError> {bit_count_one!(bit); Ok((self & bit) != 0)} }
impl BitContains for usize { fn bit_contains(&self, bit: Self) -> Result<bool, BitError> {bit_count_one!(bit); Ok((self & bit) != 0)} }

/// `BitIndex` supports counting less significant 1s in the word (1s to the right of the bit).
pub trait BitIndex {
    /// Count less significant 1s in the word (1s to the right of the bit) or return either BitError::CountNotEqualToOne or BitError::NotFound.
    /// 
    /// e.g. `0b110110.bit_index(0b100) == 1`
    fn bit_index(&self, bit: Self) -> Result<usize, BitError>;
}
impl BitIndex for u8 { fn bit_index(&self, bit: Self) -> Result<usize, BitError> {bit_found!(self, bit); Ok((self & (bit - 1)).count_ones() as usize)} }
impl BitIndex for u16 { fn bit_index(&self, bit: Self) -> Result<usize, BitError> {bit_found!(self, bit); Ok((self & (bit - 1)).count_ones() as usize)} }
impl BitIndex for u32 { fn bit_index(&self, bit: Self) -> Result<usize, BitError> {bit_found!(self, bit); Ok((self & (bit - 1)).count_ones() as usize)} }
impl BitIndex for u64 { fn bit_index(&self, bit: Self) -> Result<usize, BitError> {bit_found!(self, bit); Ok((self & (bit - 1)).count_ones() as usize)} }
impl BitIndex for u128 { fn bit_index(&self, bit: Self) -> Result<usize, BitError> {bit_found!(self, bit); Ok((self & (bit - 1)).count_ones() as usize)} }
impl BitIndex for usize { fn bit_index(&self, bit: Self) -> Result<usize, BitError> {bit_found!(self, bit); Ok((self & (bit - 1)).count_ones() as usize)} }

/// `BitInsert` supports inserting a bit into the word.
pub trait BitInsert {
    /// Insert the bit into the word or return either BitError::CountNotEqualToOne or BitError::Found.
    /// 
    /// e.g. `0b10010.bit_insert(0b100) == 0b10110`
    fn bit_insert(&self, bit: Self) -> Result<Self, BitError> where Self: Sized;
}
impl BitInsert for u8 { fn bit_insert(&self, bit: Self) -> Result<Self, BitError> {bit_not_found!(self, bit); Ok(self | bit)} }
impl BitInsert for u16 { fn bit_insert(&self, bit: Self) -> Result<Self, BitError> {bit_not_found!(self, bit); Ok(self | bit)} }
impl BitInsert for u32 { fn bit_insert(&self, bit: Self) -> Result<Self, BitError> {bit_not_found!(self, bit); Ok(self | bit)} }
impl BitInsert for u64 { fn bit_insert(&self, bit: Self) -> Result<Self, BitError> {bit_not_found!(self, bit); Ok(self | bit)} }
impl BitInsert for u128 { fn bit_insert(&self, bit: Self) -> Result<Self, BitError> {bit_not_found!(self, bit); Ok(self | bit)} }
impl BitInsert for usize { fn bit_insert(&self, bit: Self) -> Result<Self, BitError> {bit_not_found!(self, bit); Ok(self | bit)} }

/// `BitRemove` supports removing a bit from the word.
pub trait BitRemove {
    /// Remove the bit from the word or return either BitError::CountNotEqualToOne or BitError::NotFound.
    /// 
    /// e.g. `0b10110.bit_remove(0b100) == 0b10010`
    fn bit_remove(&self, bit: Self) -> Result<Self, BitError> where Self: Sized;
}
impl BitRemove for u8 { fn bit_remove(&self, bit: Self) -> Result<Self, BitError> {bit_found!(self, bit); Ok(self ^ bit)} }
impl BitRemove for u16 { fn bit_remove(&self, bit: Self) -> Result<Self, BitError> {bit_found!(self, bit); Ok(self ^ bit)} }
impl BitRemove for u32 { fn bit_remove(&self, bit: Self) -> Result<Self, BitError> {bit_found!(self, bit); Ok(self ^ bit)} }
impl BitRemove for u64 { fn bit_remove(&self, bit: Self) -> Result<Self, BitError> {bit_found!(self, bit); Ok(self ^ bit)} }
impl BitRemove for u128 { fn bit_remove(&self, bit: Self) -> Result<Self, BitError> {bit_found!(self, bit); Ok(self ^ bit)} }
impl BitRemove for usize { fn bit_remove(&self, bit: Self) -> Result<Self, BitError> {bit_found!(self, bit); Ok(self ^ bit)} }

/// `CountOnes` supports counting 1s in the word. (i.e. a call to the standard `count_ones()` function)
pub trait CountOnes {
    /// Count the number of 1s in the word using `count_ones()`.
    /// 
    /// e.g. `0b10110.count_ones_t() == 3`
    fn count_ones_t(&self) -> usize;
}
impl CountOnes for u8 { fn count_ones_t(&self) -> usize {self.count_ones() as usize} }
impl CountOnes for u16 { fn count_ones_t(&self) -> usize {self.count_ones() as usize} }
impl CountOnes for u32 { fn count_ones_t(&self) -> usize {self.count_ones() as usize} }
impl CountOnes for u64 { fn count_ones_t(&self) -> usize {self.count_ones() as usize} }
impl CountOnes for u128 { fn count_ones_t(&self) -> usize {self.count_ones() as usize} }
impl CountOnes for usize { fn count_ones_t(&self) -> usize {self.count_ones() as usize} }

/// `LogB` provides log_2 of the word size.
pub trait LogB {
    /// Get the log_2 of the word size.
    /// 
    /// e.g. `u32::log_b() == 5`
    fn log_b() -> usize;
}
impl LogB for u8 { fn log_b() -> usize {3} }
impl LogB for u16 { fn log_b() -> usize {4} }
impl LogB for u32 { fn log_b() -> usize {5} }
impl LogB for u64 { fn log_b() -> usize {6} }
impl LogB for u128 { fn log_b() -> usize {7} }
impl LogB for usize {
    fn log_b() -> usize {
        match mem::size_of::<usize>() {
            1 => 3,
            2 => 4,
            4 => 5,
            8 => 6,
            16 => 7,
            _ => panic!()
        }
    }
}

/// `MaskLogB` provides a mask of 1s equal to the log_2 of the word size.
pub trait MaskLogB {
    /// Get the mask, length log_2 of the word size.
    /// 
    /// e.g. `u32::mask_log_b() == 0b11111`
    fn mask_log_b() -> Self;
}
impl MaskLogB for u8 { fn mask_log_b() -> Self {0b111} }
impl MaskLogB for u16 { fn mask_log_b() -> Self {0b1111} }
impl MaskLogB for u32 { fn mask_log_b() -> Self {0b11111} }
impl MaskLogB for u64 { fn mask_log_b() -> Self {0b111111} }
impl MaskLogB for u128 { fn mask_log_b() -> Self {0b1111111} }
impl MaskLogB for usize {
    fn mask_log_b() -> usize {
        match mem::size_of::<usize>() {
            1 => 0b111,
            2 => 0b1111,
            4 => 0b11111,
            8 => 0b111111,
            16 => 0b1111111,
            _ => panic!()
        }
    }
}

/// `NthBit` provides a word with only the nth bit set to 1.
pub trait NthBit {
    /// Get the nth bit of the given word size.
    /// 
    /// e.g. `u32::nth_bit(4) == 0b10000`
    fn nth_bit(n: usize) -> Result<Self, BitError> where Self: Sized;
}
impl NthBit for u8 { fn nth_bit(n: usize) -> Result<Self, BitError> {bit_in_range!(u8, n); Ok(1_u8 << n)} }
impl NthBit for u16 { fn nth_bit(n: usize) -> Result<Self, BitError> {bit_in_range!(u16, n); Ok(1_u16 << n)} }
impl NthBit for u32 { fn nth_bit(n: usize) -> Result<Self, BitError> {bit_in_range!(u32, n); Ok(1_u32 << n)} }
impl NthBit for u64 { fn nth_bit(n: usize) -> Result<Self, BitError> {bit_in_range!(u64, n); Ok(1_u64 << n)} }
impl NthBit for u128 { fn nth_bit(n: usize) -> Result<Self, BitError> {bit_in_range!(u128, n); Ok(1_u128 << n)} }
impl NthBit for usize { fn nth_bit(n: usize) -> Result<Self, BitError> {bit_in_range!(usize, n); Ok(1_usize << n)} }

/// `Bits` lists the requirements on the hash value for the hash array mapped trie to function.
pub trait Bits: AsUsize + BitAnd + BitContains + BitIndex + BitInsert + BitRemove + Clone + CountOnes + Debug + Default + From<<Self as BitAnd>::Output> + From<<Self as Shr<usize>>::Output> + LogB + MaskLogB + NthBit + PartialEq + Shr<usize> + Send + Sync + 'static {}
impl <B: AsUsize + BitAnd + BitContains + BitIndex + BitInsert + BitRemove + Clone + CountOnes + Debug + Default + From<<Self as BitAnd>::Output> + From<<Self as Shr<usize>>::Output> + LogB + MaskLogB + NthBit + PartialEq + Shr<usize> + Send + Sync + 'static> Bits
for B where B: AsUsize + BitAnd + BitContains + BitIndex + BitInsert + BitRemove + Clone + CountOnes + Debug + Default + From<<Self as BitAnd>::Output> + From<<Self as Shr<usize>>::Output> + LogB + MaskLogB + NthBit + PartialEq + Shr<usize> + Send + Sync + 'static {}

/// `Value` lists the requirements on the value type for the hash array mapped trie to function.
pub trait Value: Clone + Debug + Eq + PartialEq + Hash + Send + Sync + 'static {}
impl <T: Clone + Debug + Eq + PartialEq + Hash + Send + Sync + 'static> Value
for T where T: Clone + Debug + Eq + PartialEq + Hash + Send + Sync + 'static {}

/// `HashLike` provides a means to assert that two types will hash identically.
pub trait HashLike<T> {}
impl <T> HashLike<T> for T {}

/// `HasherBv` provides a generalization of the Hasher trait to support different word sizes for the hash values.
pub trait HasherBv<B, V>: Default + 'static {
    fn hash(&self, value: &V) -> B;
}
macro_rules! hasher_bv_impl {
    ( $type:ty ) => {
        impl <V: Default + Hash + 'static, H: Default + Hasher + 'static> HasherBv<$type, V> for H {
            fn hash(&self, value: &V) -> $type {
                let mut hasher = H::default();
                value.hash(&mut hasher);
                hasher.finish() as $type
            }
        }
    };
}
hasher_bv_impl!(u8);
hasher_bv_impl!(u16);
hasher_bv_impl!(u32);
hasher_bv_impl!(u64);
