use alloc::{borrow::Cow, boxed::Box, collections::VecDeque, fmt::{Debug, Formatter}, vec::Vec};
use core::{mem, ptr};

pub trait BitContains {
    fn bit_contains(&self, bit: Self) -> Result<bool, ()>;
}
impl BitContains for u8 { fn bit_contains(&self, bit: Self) -> Result<bool, ()> {if bit.count_ones() == 1 {Ok((self & bit) != 0)} else {Err(())}} }
impl BitContains for u16 { fn bit_contains(&self, bit: Self) -> Result<bool, ()> {if bit.count_ones() == 1 {Ok((self & bit) != 0)} else {Err(())}} }
impl BitContains for u32 { fn bit_contains(&self, bit: Self) -> Result<bool, ()> {if bit.count_ones() == 1 {Ok((self & bit) != 0)} else {Err(())}} }
impl BitContains for u64 { fn bit_contains(&self, bit: Self) -> Result<bool, ()> {if bit.count_ones() == 1 {Ok((self & bit) != 0)} else {Err(())}} }
impl BitContains for u128 { fn bit_contains(&self, bit: Self) -> Result<bool, ()> {if bit.count_ones() == 1 {Ok((self & bit) != 0)} else {Err(())}} }
impl BitContains for usize { fn bit_contains(&self, bit: Self) -> Result<bool, ()> {if bit.count_ones() == 1 {Ok((self & bit) != 0)} else {Err(())}} }

pub trait BitIndex {
    fn bit_index(&self, bit: Self) -> Result<usize, ()>;
}
impl BitIndex for u8 { fn bit_index(&self, bit: Self) -> Result<usize, ()> {if bit.count_ones() == 1 && self & bit != 0 {Ok((self & (bit - 1)).count_ones() as usize)} else {Err(())}} }
impl BitIndex for u16 { fn bit_index(&self, bit: Self) -> Result<usize, ()> {if bit.count_ones() == 1 && self & bit != 0 {Ok((self & (bit - 1)).count_ones() as usize)} else {Err(())}} }
impl BitIndex for u32 { fn bit_index(&self, bit: Self) -> Result<usize, ()> {if bit.count_ones() == 1 && self & bit != 0 {Ok((self & (bit - 1)).count_ones() as usize)} else {Err(())}} }
impl BitIndex for u64 { fn bit_index(&self, bit: Self) -> Result<usize, ()> {if bit.count_ones() == 1 && self & bit != 0 {Ok((self & (bit - 1)).count_ones() as usize)} else {Err(())}} }
impl BitIndex for u128 { fn bit_index(&self, bit: Self) -> Result<usize, ()> {if bit.count_ones() == 1 && self & bit != 0 {Ok((self & (bit - 1)).count_ones() as usize)} else {Err(())}} }
impl BitIndex for usize { fn bit_index(&self, bit: Self) -> Result<usize, ()> {if bit.count_ones() == 1 && self & bit != 0 {Ok((self & (bit - 1)).count_ones() as usize)} else {Err(())}} }

pub trait BitInsert {
    fn bit_insert(&self, bit: Self) -> Result<Self, ()> where Self: Sized;
}
impl BitInsert for u8 { fn bit_insert(&self, bit: Self) -> Result<Self, ()> {if bit.count_ones() == 1 && self & bit == 0 {Ok(self | bit)} else {Err(())}} }
impl BitInsert for u16 { fn bit_insert(&self, bit: Self) -> Result<Self, ()> {if bit.count_ones() == 1 && self & bit == 0 {Ok(self | bit)} else {Err(())}} }
impl BitInsert for u32 { fn bit_insert(&self, bit: Self) -> Result<Self, ()> {if bit.count_ones() == 1 && self & bit == 0 {Ok(self | bit)} else {Err(())}} }
impl BitInsert for u64 { fn bit_insert(&self, bit: Self) -> Result<Self, ()> {if bit.count_ones() == 1 && self & bit == 0 {Ok(self | bit)} else {Err(())}} }
impl BitInsert for u128 { fn bit_insert(&self, bit: Self) -> Result<Self, ()> {if bit.count_ones() == 1 && self & bit == 0 {Ok(self | bit)} else {Err(())}} }
impl BitInsert for usize { fn bit_insert(&self, bit: Self) -> Result<Self, ()> {if bit.count_ones() == 1 && self & bit == 0 {Ok(self | bit)} else {Err(())}} }

pub trait BitRemove {
    fn bit_remove(&self, bit: Self) -> Result<Self, ()> where Self: Sized;
}
impl BitRemove for u8 { fn bit_remove(&self, bit: Self) -> Result<Self, ()> {if bit.count_ones() == 1 && self & bit != 0 {Ok(self ^ bit)} else {Err(())}} }
impl BitRemove for u16 { fn bit_remove(&self, bit: Self) -> Result<Self, ()> {if bit.count_ones() == 1 && self & bit != 0 {Ok(self ^ bit)} else {Err(())}} }
impl BitRemove for u32 { fn bit_remove(&self, bit: Self) -> Result<Self, ()> {if bit.count_ones() == 1 && self & bit != 0 {Ok(self ^ bit)} else {Err(())}} }
impl BitRemove for u64 { fn bit_remove(&self, bit: Self) -> Result<Self, ()> {if bit.count_ones() == 1 && self & bit != 0 {Ok(self ^ bit)} else {Err(())}} }
impl BitRemove for u128 { fn bit_remove(&self, bit: Self) -> Result<Self, ()> {if bit.count_ones() == 1 && self & bit != 0 {Ok(self ^ bit)} else {Err(())}} }
impl BitRemove for usize { fn bit_remove(&self, bit: Self) -> Result<Self, ()> {if bit.count_ones() == 1 && self & bit != 0 {Ok(self ^ bit)} else {Err(())}} }

pub trait CountOnes {
    fn count_ones_t(&self) -> usize;
}
impl CountOnes for u8 { fn count_ones_t(&self) -> usize {self.count_ones() as usize} }
impl CountOnes for u16 { fn count_ones_t(&self) -> usize {self.count_ones() as usize} }
impl CountOnes for u32 { fn count_ones_t(&self) -> usize {self.count_ones() as usize} }
impl CountOnes for u64 { fn count_ones_t(&self) -> usize {self.count_ones() as usize} }
impl CountOnes for u128 { fn count_ones_t(&self) -> usize {self.count_ones() as usize} }
impl CountOnes for usize { fn count_ones_t(&self) -> usize {self.count_ones() as usize} }

pub trait NthBit {
    fn nth_bit(n: usize) -> Result<Self, ()> where Self: Sized;
}
impl NthBit for u8 { fn nth_bit(n: usize) -> Result<Self, ()> {if n < 8 {Ok(1_u8 << n)} else {Err(())}} }
impl NthBit for u16 { fn nth_bit(n: usize) -> Result<Self, ()> {if n < 16 {Ok(1_u16 << n)} else {Err(())}} }
impl NthBit for u32 { fn nth_bit(n: usize) -> Result<Self, ()> {if n < 32 {Ok(1_u32 << n)} else {Err(())}} }
impl NthBit for u64 { fn nth_bit(n: usize) -> Result<Self, ()> {if n < 64 {Ok(1_u64 << n)} else {Err(())}} }
impl NthBit for u128 { fn nth_bit(n: usize) -> Result<Self, ()> {if n < 128 {Ok(1_u128 << n)} else {Err(())}} }
impl NthBit for usize { fn nth_bit(n: usize) -> Result<Self, ()> {if n < 8 * mem::size_of::<usize>() {Ok(1_usize << n)} else {Err(())}} }

struct BitIndexedArrayImpl <B, V, E, const SIZE: usize> {
    bits: B,
    values: [V; SIZE],
    extra: E,
}

impl<B: CountOnes, V, E, const SIZE: usize> BitIndexedArrayImpl<B, V, E, SIZE> {
    fn new(bits: B, values: impl Into<VecDeque<V>>, extra: E) -> Result<Self, ()> {
        let mut values: VecDeque<V> = values.into();
        if bits.count_ones_t() != SIZE || values.len() != SIZE {
            return Err(());
        }
        let values = unsafe {
            #[allow(deprecated)]
            let mut building: [V; SIZE] = mem::uninitialized();
            for dest in building.iter_mut() {
                ptr::write(dest, values.pop_front().unwrap());
            }
            building
        };
        Ok(Self { bits, values, extra })
    }
}


pub trait BitIndexedArray<B, V: Clone, E: Clone>: 'static {
    fn bits(&self) -> B;
    fn len(&self) -> usize;
    fn values(&self) -> &[V];
    fn extra(&self) -> &E;

    fn at(&self, bit: B) -> Result<&V, ()>;
    fn at_index(&self, index: usize) -> Result<&V, ()>;
    fn inserted(&self, bit: B, value: Cow<V>, extra: Cow<E>) -> Result<Box<dyn BitIndexedArray::<B, V, E>>, ()>;
    fn inserted_index(&self, index: usize, value: Cow<V>, extra: Cow<E>) -> Result<Box<dyn BitIndexedArray::<B, V, E>>, ()>;
    fn updated(&self, bit: B, value: Cow<V>, extra: Cow<E>) -> Result<Box<dyn BitIndexedArray::<B, V, E>>, ()>;
    fn updated_index(&self, index: usize, value: Cow<V>, extra: Cow<E>) -> Result<Box<dyn BitIndexedArray::<B, V, E>>, ()>;
    fn removed(&self, bit: B, extra: Cow<E>) -> Result<Box<dyn BitIndexedArray::<B, V, E>>, ()>;
    fn removed_index(&self, index: usize, extra: Cow<E>) -> Result<Box<dyn BitIndexedArray::<B, V, E>>, ()>;

    fn clone_impl(&self) -> Box<dyn BitIndexedArray::<B, V, E>>;
    
    fn iter(&'_ self) -> core::slice::Iter<'_, V>;
    fn iter_mut(&'_ mut self) -> core::slice::IterMut<'_, V>;
}

impl <B: BitContains + BitIndex + BitInsert + BitRemove + Clone + CountOnes + NthBit + PartialEq + 'static, V: Clone + 'static, E: Clone + Default + 'static, const SIZE: usize> BitIndexedArray<B, V, E> for BitIndexedArrayImpl<B, V, E, SIZE> {
    fn bits(&self) -> B {
        self.bits.clone()
    }

    fn len(&self) -> usize {
        SIZE
    }

    fn values(&self) -> &[V] {
        &self.values
    }

    fn extra(&self) -> &E {
        &self.extra
    }

    fn at(&self, bit: B) -> Result<&V, ()> {
        let index = self.bits.bit_index(bit)?;
        #[allow(unused_comparisons)]
        if index < SIZE {
            Ok(&self.values[index])
        }
        else {
            Err(())
        }
    }

    fn at_index(&self, index: usize) -> Result<&V, ()> {
        Ok(&self.values[self.bits.bit_index(B::nth_bit(index)?)?])
    }

    fn inserted(&self, bit: B, value: Cow<V>, extra: Cow<E>) -> Result<Box<dyn BitIndexedArray::<B, V, E>>, ()> {
        let bits = self.bits.bit_insert(bit.clone())?;
        let index = bits.bit_index(bit)?;
        let mut building = VecDeque::<V>::new();
        for i in 0..index {
            building.push_back(self.values[i].clone());
        }
        building.push_back(value.into_owned());
        for i in index..SIZE {
            building.push_back(self.values[i].clone());
        }
        new_bit_indexed_array(bits, building, extra.into_owned())
    }

    fn inserted_index(&self, index: usize, value: Cow<V>, extra: Cow<E>) -> Result<Box<dyn BitIndexedArray::<B, V, E>>, ()> {
        self.inserted(B::nth_bit(index)?, value, extra)
    }
    
    fn updated(&self, bit: B, value: Cow<V>, extra: Cow<E>) -> Result<Box<dyn BitIndexedArray::<B, V, E>>, ()> {
        if !self.bits.bit_contains(bit.clone())? {
            return Err(());
        }
        let index = self.bits.bit_index(bit)?;
        let mut building = VecDeque::<V>::new();
        for i in 0..index {
            building.push_back(self.values[i].clone());
        }
        building.push_back(value.into_owned());
        for i in index+1..SIZE {
            building.push_back(self.values[i].clone());
        }
        new_bit_indexed_array(self.bits.clone(), building, extra.into_owned())
    }

    fn updated_index(&self, index: usize, value: Cow<V>, extra: Cow<E>) -> Result<Box<dyn BitIndexedArray::<B, V, E>>, ()> {
        self.updated(B::nth_bit(index)?, value, extra)
    }
    
    fn removed(&self, bit: B, extra: Cow<E>) -> Result<Box<dyn BitIndexedArray::<B, V, E>>, ()> {
        let bits = self.bits.bit_remove(bit.clone())?;
        let index = self.bits.bit_index(bit)?;
        let mut building = VecDeque::<V>::new();
        for i in 0..index {
            building.push_back(self.values[i].clone());
        }
        for i in index+1..SIZE {
            building.push_back(self.values[i].clone());
        }
        new_bit_indexed_array(bits, building, extra.into_owned())
    }
    
    fn removed_index(&self, index: usize, extra: Cow<E>) -> Result<Box<dyn BitIndexedArray::<B, V, E>>, ()> {
        self.removed(B::nth_bit(index)?, extra)
    }
    
    fn clone_impl(&self) -> Box<dyn BitIndexedArray::<B, V, E>> {
        Box::new(Self {
            bits: self.bits.clone(),
            values: self.values.clone(),
            extra: self.extra.clone(),
        })
    }
    
    fn iter(&self) -> core::slice::Iter<V> {
        self.values.iter()
    }
    
    fn iter_mut(&mut self) -> core::slice::IterMut<V> {
        self.values.iter_mut()
    }
}

#[allow(dead_code)]
pub fn default_bit_indexed_array<B: BitContains + BitIndex + BitInsert + BitRemove + Clone + CountOnes + Default + NthBit + PartialEq + 'static, V: Clone + 'static, E: Clone + Default + 'static>() -> Box<dyn BitIndexedArray<B, V, E>> {
    Box::new(BitIndexedArrayImpl::<B, V, E, 0>::default())
}

#[allow(dead_code)]
pub fn new_bit_indexed_array<B: BitContains + BitIndex + BitInsert + BitRemove + Clone + CountOnes + NthBit + PartialEq + 'static, V: Clone + 'static, E: Clone + Default + 'static>(bits: B, values: impl Into<VecDeque<V>>, extra: E) -> Result<Box<dyn BitIndexedArray<B, V, E>>, ()> {
    let values: VecDeque<V> = values.into();
    if bits.count_ones_t() != values.len() {
        return Err(());
    }
    match values.len() {
        0 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 0>::new(bits, values, extra).unwrap())),
        1 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 1>::new(bits, values, extra).unwrap())),
        2 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 2>::new(bits, values, extra).unwrap())),
        3 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 3>::new(bits, values, extra).unwrap())),
        4 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 4>::new(bits, values, extra).unwrap())),
        5 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 5>::new(bits, values, extra).unwrap())),
        6 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 6>::new(bits, values, extra).unwrap())),
        7 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 7>::new(bits, values, extra).unwrap())),
        8 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 8>::new(bits, values, extra).unwrap())),
        9 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 9>::new(bits, values, extra).unwrap())),
        10 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 10>::new(bits, values, extra).unwrap())),
        11 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 11>::new(bits, values, extra).unwrap())),
        12 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 12>::new(bits, values, extra).unwrap())),
        13 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 13>::new(bits, values, extra).unwrap())),
        14 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 14>::new(bits, values, extra).unwrap())),
        15 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 15>::new(bits, values, extra).unwrap())),
        16 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 16>::new(bits, values, extra).unwrap())),
        17 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 17>::new(bits, values, extra).unwrap())),
        18 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 18>::new(bits, values, extra).unwrap())),
        19 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 19>::new(bits, values, extra).unwrap())),
        20 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 20>::new(bits, values, extra).unwrap())),
        21 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 21>::new(bits, values, extra).unwrap())),
        22 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 22>::new(bits, values, extra).unwrap())),
        23 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 23>::new(bits, values, extra).unwrap())),
        24 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 24>::new(bits, values, extra).unwrap())),
        25 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 25>::new(bits, values, extra).unwrap())),
        26 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 26>::new(bits, values, extra).unwrap())),
        27 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 27>::new(bits, values, extra).unwrap())),
        28 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 28>::new(bits, values, extra).unwrap())),
        29 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 29>::new(bits, values, extra).unwrap())),
        30 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 30>::new(bits, values, extra).unwrap())),
        31 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 31>::new(bits, values, extra).unwrap())),
        32 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 32>::new(bits, values, extra).unwrap())),
        33 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 33>::new(bits, values, extra).unwrap())),
        34 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 34>::new(bits, values, extra).unwrap())),
        35 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 35>::new(bits, values, extra).unwrap())),
        36 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 36>::new(bits, values, extra).unwrap())),
        37 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 37>::new(bits, values, extra).unwrap())),
        38 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 38>::new(bits, values, extra).unwrap())),
        39 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 39>::new(bits, values, extra).unwrap())),
        40 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 40>::new(bits, values, extra).unwrap())),
        41 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 41>::new(bits, values, extra).unwrap())),
        42 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 42>::new(bits, values, extra).unwrap())),
        43 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 43>::new(bits, values, extra).unwrap())),
        44 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 44>::new(bits, values, extra).unwrap())),
        45 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 45>::new(bits, values, extra).unwrap())),
        46 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 46>::new(bits, values, extra).unwrap())),
        47 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 47>::new(bits, values, extra).unwrap())),
        48 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 48>::new(bits, values, extra).unwrap())),
        49 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 49>::new(bits, values, extra).unwrap())),
        50 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 50>::new(bits, values, extra).unwrap())),
        51 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 51>::new(bits, values, extra).unwrap())),
        52 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 52>::new(bits, values, extra).unwrap())),
        53 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 53>::new(bits, values, extra).unwrap())),
        54 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 54>::new(bits, values, extra).unwrap())),
        55 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 55>::new(bits, values, extra).unwrap())),
        56 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 56>::new(bits, values, extra).unwrap())),
        57 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 57>::new(bits, values, extra).unwrap())),
        58 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 58>::new(bits, values, extra).unwrap())),
        59 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 59>::new(bits, values, extra).unwrap())),
        60 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 60>::new(bits, values, extra).unwrap())),
        61 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 61>::new(bits, values, extra).unwrap())),
        62 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 62>::new(bits, values, extra).unwrap())),
        63 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 63>::new(bits, values, extra).unwrap())),
        64 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 64>::new(bits, values, extra).unwrap())),
        65 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 65>::new(bits, values, extra).unwrap())),
        66 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 66>::new(bits, values, extra).unwrap())),
        67 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 67>::new(bits, values, extra).unwrap())),
        68 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 68>::new(bits, values, extra).unwrap())),
        69 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 69>::new(bits, values, extra).unwrap())),
        70 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 70>::new(bits, values, extra).unwrap())),
        71 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 71>::new(bits, values, extra).unwrap())),
        72 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 72>::new(bits, values, extra).unwrap())),
        73 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 73>::new(bits, values, extra).unwrap())),
        74 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 74>::new(bits, values, extra).unwrap())),
        75 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 75>::new(bits, values, extra).unwrap())),
        76 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 76>::new(bits, values, extra).unwrap())),
        77 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 77>::new(bits, values, extra).unwrap())),
        78 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 78>::new(bits, values, extra).unwrap())),
        79 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 79>::new(bits, values, extra).unwrap())),
        80 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 80>::new(bits, values, extra).unwrap())),
        81 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 81>::new(bits, values, extra).unwrap())),
        82 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 82>::new(bits, values, extra).unwrap())),
        83 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 83>::new(bits, values, extra).unwrap())),
        84 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 84>::new(bits, values, extra).unwrap())),
        85 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 85>::new(bits, values, extra).unwrap())),
        86 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 86>::new(bits, values, extra).unwrap())),
        87 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 87>::new(bits, values, extra).unwrap())),
        88 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 88>::new(bits, values, extra).unwrap())),
        89 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 89>::new(bits, values, extra).unwrap())),
        90 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 90>::new(bits, values, extra).unwrap())),
        91 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 91>::new(bits, values, extra).unwrap())),
        92 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 92>::new(bits, values, extra).unwrap())),
        93 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 93>::new(bits, values, extra).unwrap())),
        94 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 94>::new(bits, values, extra).unwrap())),
        95 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 95>::new(bits, values, extra).unwrap())),
        96 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 96>::new(bits, values, extra).unwrap())),
        97 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 97>::new(bits, values, extra).unwrap())),
        98 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 98>::new(bits, values, extra).unwrap())),
        99 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 99>::new(bits, values, extra).unwrap())),
        100 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 100>::new(bits, values, extra).unwrap())),
        101 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 101>::new(bits, values, extra).unwrap())),
        102 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 102>::new(bits, values, extra).unwrap())),
        103 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 103>::new(bits, values, extra).unwrap())),
        104 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 104>::new(bits, values, extra).unwrap())),
        105 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 105>::new(bits, values, extra).unwrap())),
        106 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 106>::new(bits, values, extra).unwrap())),
        107 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 107>::new(bits, values, extra).unwrap())),
        108 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 108>::new(bits, values, extra).unwrap())),
        109 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 109>::new(bits, values, extra).unwrap())),
        110 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 110>::new(bits, values, extra).unwrap())),
        111 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 111>::new(bits, values, extra).unwrap())),
        112 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 112>::new(bits, values, extra).unwrap())),
        113 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 113>::new(bits, values, extra).unwrap())),
        114 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 114>::new(bits, values, extra).unwrap())),
        115 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 115>::new(bits, values, extra).unwrap())),
        116 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 116>::new(bits, values, extra).unwrap())),
        117 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 117>::new(bits, values, extra).unwrap())),
        118 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 118>::new(bits, values, extra).unwrap())),
        119 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 119>::new(bits, values, extra).unwrap())),
        120 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 120>::new(bits, values, extra).unwrap())),
        121 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 121>::new(bits, values, extra).unwrap())),
        122 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 122>::new(bits, values, extra).unwrap())),
        123 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 123>::new(bits, values, extra).unwrap())),
        124 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 124>::new(bits, values, extra).unwrap())),
        125 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 125>::new(bits, values, extra).unwrap())),
        126 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 126>::new(bits, values, extra).unwrap())),
        127 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 127>::new(bits, values, extra).unwrap())),
        128 => Ok(Box::new(BitIndexedArrayImpl::<B, V, E, 128>::new(bits, values, extra).unwrap())),
        _ => Err(())
    }
}

impl <B: Clone, V: Clone, E: Clone, const SIZE: usize> Clone for BitIndexedArrayImpl<B, V, E, SIZE> {
    fn clone(&self) -> Self {
        Self {
            bits: self.bits.clone(),
            values: self.values.clone(),
            extra: self.extra.clone(),
        }
    }
}

impl <B: Clone + 'static, V: Clone + 'static, E: Clone + 'static> Clone for Box<dyn BitIndexedArray<B, V, E>> {
    fn clone(&self) -> Self {
        self.clone_impl()
    }
}

impl <B: CountOnes + Default, V, E: Default> Default for BitIndexedArrayImpl<B, V, E, 0> {
    fn default() -> Self {
        Self::new(B::default(), Vec::new(), E::default()).unwrap()
    }
}

impl <B: Eq, V: Eq, E: Eq, const SIZE: usize> Eq for BitIndexedArrayImpl<B, V, E, SIZE> {}

impl <B: Eq + 'static, V: Clone + Eq + 'static, E: Clone + Eq + 'static> Eq for dyn BitIndexedArray<B, V, E> {}

impl <B: PartialEq, V: PartialEq, E: PartialEq, const SIZE: usize> PartialEq for BitIndexedArrayImpl<B, V, E, SIZE> {
    fn eq(&self, other: &Self) -> bool {
        if self.bits != other.bits || self.extra != other.extra {
            return false;
        }
        for i in 0..self.values.len() {
            if self.values[i] != other.values[i] {
                return false;
            }
        }
        true
    }
}

impl <B: PartialEq + 'static, V: Clone + PartialEq + 'static, E: Clone + PartialEq + 'static> PartialEq for dyn BitIndexedArray<B, V, E> {
    fn eq(&self, other: &Self) -> bool {
        self.bits() == other.bits() && self.values() == other.values() && *self.extra() == *other.extra()
    }
}

impl <'a, B, V, E, const SIZE: usize> IntoIterator for &'a BitIndexedArrayImpl<B, V, E, SIZE> {
    type Item = &'a V;
    type IntoIter = core::slice::Iter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

impl <'a, B, V, E, const SIZE: usize> IntoIterator for &'a mut BitIndexedArrayImpl<B, V, E, SIZE> {
    type Item = &'a mut V;
    type IntoIter = core::slice::IterMut<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter_mut()
    }
}

impl <'a, B: 'static, V: Clone + 'static, E: Clone + 'static> IntoIterator for &'a dyn BitIndexedArray<B, V, E> {
    type Item = &'a V;
    type IntoIter = core::slice::Iter<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl <'a, B: 'static, V: Clone + 'static, E: Clone + 'static> IntoIterator for &'a mut dyn BitIndexedArray<B, V, E> {
    type Item = &'a mut V;
    type IntoIter = core::slice::IterMut<'a, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl <B: Debug, V: Debug, E: Debug, const SIZE: usize> Debug for BitIndexedArrayImpl<B, V, E, SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
        write!(f, "BitIndexedArrayImpl {{ bits: {:?}, values: {:?}, extra: {:?} }}", self.bits, self.values, self.extra)
    }
}

impl <B: Debug + 'static, V: Clone + Debug + 'static, E: Clone + Debug + 'static> Debug for dyn BitIndexedArray<B, V, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
        write!(f, "BitIndexedArray {{ bits: {:?}, values: {:?}, extra: {:?} }}", self.bits(), self.values(), self.extra())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::println;

    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    struct Zst {}
    
    #[test]
    fn bit_indexed_array_sizes_differ() {
        assert!(mem::size_of::<BitIndexedArrayImpl::<u64,usize,Zst,0>>() < mem::size_of::<BitIndexedArrayImpl::<u64,usize,Zst,1>>());
        assert!(mem::size_of::<BitIndexedArrayImpl::<u64,usize,Zst,1>>() < mem::size_of::<BitIndexedArrayImpl::<u64,usize,Zst,2>>());
        assert!(mem::size_of::<BitIndexedArrayImpl::<u64,usize,Zst,126>>() < mem::size_of::<BitIndexedArrayImpl::<u64,usize,Zst,127>>());
        assert!(mem::size_of::<BitIndexedArrayImpl::<u64,usize,Zst,127>>() < mem::size_of::<BitIndexedArrayImpl::<u64,usize,Zst,128>>());
    }
    
    #[test]
    fn bit_indexed_array_insert() {
        let mut bia: Box<dyn BitIndexedArray<_,_,_>> = Box::new(BitIndexedArrayImpl::<_,_,_,2>::new(0b101_u64, vec!(13, 42), Zst{}).unwrap());
        bia = bia.as_ref().inserted(0b10, Cow::Owned(3), Cow::Owned(Zst{})).unwrap();
        assert_eq!(bia.as_ref().len(), 3);
        assert_eq!(*bia.as_ref().at_index(0).unwrap(), 13);
        assert_eq!(*bia.as_ref().at_index(1).unwrap(), 3);
        assert_eq!(*bia.as_ref().at_index(2).unwrap(), 42);
        assert!(bia.as_ref().at_index(3).is_err());
        assert_eq!(*bia.as_ref().at(0b1).unwrap(), 13);
        assert_eq!(*bia.as_ref().at(0b10).unwrap(), 3);
        assert_eq!(*bia.as_ref().at(0b100).unwrap(), 42);
        assert!(bia.as_ref().at(0b1000).is_err());
        println!("Array: {:?}", bia.as_ref());
    }

    #[test]
    fn bit_indexed_array_insert_reinsert_failure() {
        let bia: Box<dyn BitIndexedArray<_,_,_>> = Box::new(BitIndexedArrayImpl::<_,_,_,2>::new(0b101_u64, vec!(13, 42), Zst{}).unwrap());
        assert!(bia.as_ref().inserted(0b100, Cow::Owned(3), Cow::Owned(Zst{})).is_err());
    }

    #[test]
    fn bit_indexed_array_insert_multibit_failure() {
        let bia: Box<dyn BitIndexedArray<_,_,_>> = Box::new(BitIndexedArrayImpl::<_,_,_,2>::new(0b101_u64, vec!(13, 42), Zst{}).unwrap());
        assert!(bia.as_ref().inserted(0b1010, Cow::Owned(3), Cow::Owned(Zst{})).is_err());
    }

    #[test]
    fn bit_indexed_array_update() {
        let mut bia: Box<dyn BitIndexedArray<_,_,_>> = Box::new(BitIndexedArrayImpl::<_,_,_,3>::new(0b1101_u64, vec!(13, 42, 8), Zst{}).unwrap());
        bia = bia.as_ref().updated(0b1000, Cow::Owned(11), Cow::Owned(Zst{})).unwrap();
        assert_eq!(bia.as_ref().len(), 3);
        assert_eq!(*bia.as_ref().at_index(0).unwrap(), 13);
        assert!(bia.as_ref().at_index(1).is_err());
        assert_eq!(*bia.as_ref().at_index(2).unwrap(), 42);
        assert_eq!(*bia.as_ref().at_index(3).unwrap(), 11);
        assert!(bia.as_ref().at_index(4).is_err());
        assert_eq!(*bia.as_ref().at(0b1).unwrap(), 13);
        assert!(bia.as_ref().at(0b10).is_err());
        assert_eq!(*bia.as_ref().at(0b100).unwrap(), 42);
        assert_eq!(*bia.as_ref().at(0b1000).unwrap(), 11);
        assert!(bia.as_ref().at(0b10000).is_err());
        println!("Array: {:?}", bia.as_ref());
    }

    #[test]
    fn bit_indexed_array_update_absent_failure() {
        let bia: Box<dyn BitIndexedArray<_,_,_>> = Box::new(BitIndexedArrayImpl::<_,_,_,2>::new(0b101_u64, vec!(13, 42), Zst{}).unwrap());
        assert!(bia.as_ref().updated(0b10, Cow::Owned(3), Cow::Owned(Zst{})).is_err());
    }

    #[test]
    fn bit_indexed_array_update_multibit_failure() {
        let bia: Box<dyn BitIndexedArray<_,_,_>> = Box::new(BitIndexedArrayImpl::<_,_,_,2>::new(0b101_u64, vec!(13, 42), Zst{}).unwrap());
        assert!(bia.as_ref().updated(0b101, Cow::Owned(3), Cow::Owned(Zst{})).is_err());
    }

    #[test]
    fn bit_indexed_array_update_index() {
        let mut bia: Box<dyn BitIndexedArray<_,_,_>> = Box::new(BitIndexedArrayImpl::<_,_,_,3>::new(0b1101_u64, vec!(13, 42, 8), Zst{}).unwrap());
        bia = bia.as_ref().updated_index(2, Cow::Owned(11), Cow::Owned(Zst{})).unwrap();
        assert_eq!(bia.as_ref().len(), 3);
        assert_eq!(*bia.as_ref().at_index(0).unwrap(), 13);
        assert!(bia.as_ref().at_index(1).is_err());
        assert_eq!(*bia.as_ref().at_index(2).unwrap(), 11);
        assert_eq!(*bia.as_ref().at_index(3).unwrap(), 8);
        assert!(bia.as_ref().at_index(4).is_err());
        assert_eq!(*bia.as_ref().at(0b1).unwrap(), 13);
        assert!(bia.as_ref().at(0b10).is_err());
        assert_eq!(*bia.as_ref().at(0b100).unwrap(), 11);
        assert_eq!(*bia.as_ref().at(0b1000).unwrap(), 8);
        assert!(bia.as_ref().at(0b10000).is_err());
        println!("Array: {:?}", bia.as_ref());
    }

    #[test]
    fn bit_indexed_array_update_absent_index_failure() {
        let bia: Box<dyn BitIndexedArray<_,_,_>> = Box::new(BitIndexedArrayImpl::<_,_,_,2>::new(0b101_u64, vec!(13, 42), Zst{}).unwrap());
        assert!(bia.as_ref().updated_index(1, Cow::Owned(3), Cow::Owned(Zst{})).is_err());
    }

    #[test]
    fn bit_indexed_array_remove() {
        let mut bia: Box<dyn BitIndexedArray<_,_,_>> = Box::new(BitIndexedArrayImpl::<_,_,_,3>::new(0b1101_u64, vec!(13, 42, 8), Zst{}).unwrap());
        bia = bia.as_ref().removed(0b1000, Cow::Owned(Zst{})).unwrap();
        assert_eq!(bia.as_ref().len(), 2);
        assert_eq!(*bia.as_ref().at_index(0).unwrap(), 13);
        assert!(bia.as_ref().at_index(1).is_err());
        assert_eq!(*bia.as_ref().at_index(2).unwrap(), 42);
        assert!(bia.as_ref().at_index(3).is_err());
        assert!(bia.as_ref().at_index(4).is_err());
        assert_eq!(*bia.as_ref().at(0b1).unwrap(), 13);
        assert!(bia.as_ref().at(0b10).is_err());
        assert_eq!(*bia.as_ref().at(0b100).unwrap(), 42);
        assert!(bia.as_ref().at(0b1000).is_err());
        println!("Array: {:?}", bia.as_ref());
    }

    #[test]
    fn bit_indexed_array_remove_absent_failure() {
        let bia: Box<dyn BitIndexedArray<_,_,_>> = Box::new(BitIndexedArrayImpl::<_,_,_,3>::new(0b1101_u64, vec!(13, 42, 8), Zst{}).unwrap());
        assert!(bia.as_ref().removed(0b10, Cow::Owned(Zst{})).is_err());
    }

    #[test]
    fn bit_indexed_array_remove_multibit_failure() {
        let bia: Box<dyn BitIndexedArray<_,_,_>> = Box::new(BitIndexedArrayImpl::<_,_,_,3>::new(0b1101_u64, vec!(13, 42, 8), Zst{}).unwrap());
        assert!(bia.as_ref().removed(0b101, Cow::Owned(Zst{})).is_err());
    }

}
