use crate::{traits::*, hash_trie::HashTrie, *};
use alloc::fmt::Debug;

/// `HashTrieSet` implements a hash set using a hash array mapped trie (HAMT).
/// 
/// # Example Usage
/// 
/// ```
/// use fnv::FnvHasher;
/// use hash_trie::{HashTrieSet, traits::HashLike};
/// use std::string::String;
/// 
/// #[derive(Clone,Debug,Eq,Hash,PartialEq)]
/// struct Str<'a> {
///     s: &'a str
/// }
/// 
/// impl <'a> Str<'a> {
///     fn new(s: &'a str) -> Self {
///         Self {s}
///     }
/// }
/// 
/// impl <'a> Default for Str<'a> {
///     fn default() -> Self {
///         Self {s: ""}
///     }
/// }
/// impl <'a> From<Str<'a>> for String {
///     fn from(s: Str<'a>) -> String {
///         s.s.into()
///     }
/// }
/// impl <'a> PartialEq<Str<'a>> for String {
///     fn eq(&self, other: &Str<'a>) -> bool {
///         *self == other.s
///     }
/// }
/// impl <'a> HashLike<String> for Str<'a> {}
/// impl <'a> HashLike<Str<'a>> for String {}
/// unsafe impl <'a> Send for Str<'a> {}
/// unsafe impl <'a> Sync for Str<'a> {}
/// 
/// let mut hash_set: HashTrieSet<u64, u32, String, FnvHasher> = HashTrieSet::new();
/// let hello_world = "Hello, world!";
///
/// hash_set = hash_set.insert(Str::new(hello_world), false).unwrap().0;
/// 
/// // Inserting an already-inserted key returns a reference to the key in the set...
/// assert!(*hash_set.insert(Str::new(hello_world), false).map(|_| ()).unwrap_err() == hello_world);
/// // ... unless you enable replacement.
/// assert!(hash_set.insert(Str::new(hello_world), true).is_ok());
///
/// assert!(hash_set.find(&Str::new(hello_world)).map(|reference| *reference == hello_world).unwrap());
///
/// match hash_set.remove(&Str::new(hello_world)) {
///     Ok((mutated, reference)) => {
///         // Removing a key returns a reference to the key
///         // in the set in addition to the mutated set.
///         println!("Value stored in hash_set: {}", reference);
///         hash_set = mutated;
///     },
///     Err(_) => panic!(),
/// }
/// ```
#[derive(Clone, Debug)]
#[must_use]
pub struct HashTrieSet <H: Hashword, F: Flagword<H>, K: Key, M: HasherBv<H, K>> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    set: HashTrie<H, F, K, (), M>,
}

impl <H: Hashword, F: Flagword<H>, K: Key, M: HasherBv<H, K>> HashTrieSet<H, F, K, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    /// Get a new, empty HashTrieSet.
    pub fn new() -> Self {
        Self {
            set: HashTrie::new()
        }
    }

    /// Get the total number of entries in the set.
    pub fn size(&self) -> usize {
        self.set.size()
    }

    /// Search the HashTrieSet for the given key and return a reference if found, or `HashTrieError::NotFound` if not found.
    pub fn find<'a, L: Key + HashLike<K>>(&'a self, key: &L) -> Result<&'a K, HashTrieError> where K: PartialEq<L>, M: HasherBv<H, L> {
        self.set.find(key).map(|(key, _value)| key)
    }

    /// Search the HashTrieSet for the spot to insert the key and return both a mutated set and, if applicable, a reference to the replaced key.
    /// If found and replacement is disabled, a reference to the existing key is returned.
    pub fn insert<'a, L: Key + HashLike<K> + Into<K>>(&'a self, key: L, replace: bool) -> Result<(Self, *const K, Option<&'a K>), &'a K>
    where
        K: HashLike<L>,
        K: PartialEq<L>,
        M: HasherBv<H, L>
    {
        self.set.insert(key, (), replace).map(|(set, key, _value, prev)| (Self {set}, key, prev.map(|(k, _v)| k))).map_err(|(key, _value)| key)
    }

    /// Search the HashTrieSet for the given key to remove and return a mutated set, or `HashTrieError::NotFound` if not found.
    pub fn remove<'a, L: Key + HashLike<K>>(&'a self, key: &L) -> Result<(Self, &'a K), HashTrieError> where K: PartialEq<L>, M: HasherBv<H, L> {
        self.set.remove(key).map(|(set, key, _value)| (Self {set}, key))
    }

    /// Run an operation on each entry in the set.
    pub fn visit<Op: Clone>(&self, op: Op) where Op: Fn(&K) {
        self.set.visit(|key, _value| op(key));
    }

    /// Run a transform operation on each entry in the set. Returns the transformed set and a reduction of the secondary returns of the transform operations.
    pub fn transform<ReduceT, ReduceOp, Op>
        (&self, reduce_op: ReduceOp, op: Op) -> (Self, ReduceT)
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        Op: Fn(&K) -> SetTransformResult<ReduceT> + Clone,
    {
        let (set, reduced) = self.set.transform(reduce_op, |key, _value| match op(key) {
            SetTransformResult::Unchanged(reduced) => MapTransformResult::Unchanged(reduced),
            SetTransformResult::Removed(reduced) => MapTransformResult::Removed(reduced),
        });
        (Self{set}, reduced)
    }

    /// Run a transmute operation on each entry in the set. Returns the transmuted set and a reduction of the secondary returns of the transmute operations.
    pub unsafe fn transmute<S: Key + HashLike<S>, ReduceT, ReduceOp, Op>
        (&self, reduce_op: ReduceOp, op: Op) -> (HashTrieSet<H, F, S, M>, ReduceT)
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        Op: Fn(&K) -> SetTransmuteResult<S, ReduceT> + Clone,
        K: HashLike<S>,
        K: PartialEq<S>,
        M: HasherBv<H, S>,
    {
        let (set, reduced) = self.set.transmute(reduce_op, |key, _value| match op(key) {
            SetTransmuteResult::Transmuted(key, reduced) => MapTransmuteResult::Transmuted(key, (), reduced),
            SetTransmuteResult::Removed(reduced) => MapTransmuteResult::Removed(reduced),
        });
        (HashTrieSet{set}, reduced)
    }

    /// Run a transform operation on each entry or pair of entries in the sets. Returns the transformed set and a reduction of the secondary returns of the transmute operations. Can reuse nodes from either set.
    pub fn transform_with_transformed<ReduceT, ReduceOp, BothOp, LeftOp, RightOp>
        (&self, right: &Self, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp) -> (Self, ReduceT)
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        BothOp: Fn(&K, &K) -> SetJointTransformResult<ReduceT> + Clone,
        LeftOp: Fn(&K) -> SetTransformResult<ReduceT> + Clone,
        RightOp: Fn(&K) -> SetTransformResult<ReduceT> + Clone,
    {
        let (set, reduced) = self.set.transform_with_transformed(&right.set, reduce_op,
            |l,_,r,_| both_op(l, r).into(),
            |l,_| left_op(l).into(),
            |r,_| right_op(r).into());
        (HashTrieSet{set}, reduced)
    }

    /// Run a transform/transmute operation on each entry or pair of entries in the sets. Returns the transmuted set and a reduction of the secondary returns of the transmute operations. Can reuse nodes from the transformed set.
    pub unsafe fn transform_with_transmuted<L: Key + HashLike<K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>
        (&self, right: &HashTrieSet<H, F, L, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp) -> (Self, ReduceT)
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        BothOp: Fn(&K, &L) -> SetTransformResult<ReduceT> + Clone,
        LeftOp: Fn(&K) -> SetTransformResult<ReduceT> + Clone,
        RightOp: Fn(&L) -> SetTransmuteResult<K, ReduceT> + Clone,
        K: HashLike<L>,
        K: PartialEq<L>,
        L: HashLike<K>,
        L: PartialEq<K>,
        M: HasherBv<H, L>,
    {
        let (set, reduced) = self.set.transform_with_transmuted(&right.set, reduce_op,
            |l,_,r,_| both_op(l, r).into(),
            |l,_| left_op(l).into(),
            |r,_| match right_op(r) {
                SetTransmuteResult::Transmuted(key, reduced) => MapTransmuteResult::Transmuted(key, (), reduced),
                SetTransmuteResult::Removed(reduced) => MapTransmuteResult::Removed(reduced),
            });
        (HashTrieSet{set}, reduced)
    }

    /// Run a transmute/transform operation on each entry or pair of entries in the sets. Returns the transmuted set and a reduction of the secondary returns of the transmute operations. Can reuse nodes from the transformed set.
    pub unsafe fn transmute_with_transformed<L: Key + HashLike<K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>
        (&self, right: &HashTrieSet<H, F, L, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp) -> (HashTrieSet<H, F, L, M>, ReduceT)
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        BothOp: Fn(&K, &L) -> SetTransformResult<ReduceT> + Clone,
        LeftOp: Fn(&K) -> SetTransmuteResult<L, ReduceT> + Clone,
        RightOp: Fn(&L) -> SetTransformResult<ReduceT> + Clone,
        K: HashLike<L>,
        K: PartialEq<L>,
        L: HashLike<K>,
        L: PartialEq<K>,
        M: HasherBv<H, L>,
    {
        let (set, reduced) = self.set.transmute_with_transformed(&right.set, reduce_op,
            |l,_,r,_| both_op(l, r).into(),
            |l,_| match left_op(l) {
                SetTransmuteResult::Transmuted(key, reduced) => MapTransmuteResult::Transmuted(key, (), reduced),
                SetTransmuteResult::Removed(reduced) => MapTransmuteResult::Removed(reduced),
            },
            |r,_| right_op(r).into());
        (HashTrieSet{set}, reduced)
    }

    /// Run a transmute operation on each entry or pair of entries in the sets. Returns the transmuted set and a reduction of the secondary returns of the transmute operations.
    pub unsafe fn transmute_with_transmuted<L: Key + HashLike<K>, S: Key + HashLike<K>, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>
        (&self, right: &HashTrieSet<H, F, L, M>, reduce_op: ReduceOp, both_op: BothOp, left_op: LeftOp, right_op: RightOp) -> (HashTrieSet<H, F, S, M>, ReduceT)
        where
        Self: Sized,
        ReduceT: Default,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone,
        BothOp: Fn(&K, &L) -> SetTransmuteResult<S, ReduceT> + Clone,
        LeftOp: Fn(&K) -> SetTransmuteResult<S, ReduceT> + Clone,
        RightOp: Fn(&L) -> SetTransmuteResult<S, ReduceT> + Clone,
        K: HashLike<L>,
        K: PartialEq<L>,
        K: HashLike<S>,
        K: PartialEq<S>,
        L: HashLike<K>,
        L: PartialEq<K>,
        L: HashLike<S>,
        L: PartialEq<S>,
        M: HasherBv<H, L>,
        M: HasherBv<H, S>,
    {
        let (set, reduced) = self.set.transmute_with_transmuted(&right.set, reduce_op, |l,_,r,_| match both_op(l, r) {
            SetTransmuteResult::Transmuted(key, reduced) => MapTransmuteResult::Transmuted(key, (), reduced),
            SetTransmuteResult::Removed(reduced) => MapTransmuteResult::Removed(reduced),
        }, |l,_| match left_op(l) {
            SetTransmuteResult::Transmuted(key, reduced) => MapTransmuteResult::Transmuted(key, (), reduced),
            SetTransmuteResult::Removed(reduced) => MapTransmuteResult::Removed(reduced),
        }, |r,_| match right_op(r) {
            SetTransmuteResult::Transmuted(key, reduced) => MapTransmuteResult::Transmuted(key, (), reduced),
            SetTransmuteResult::Removed(reduced) => MapTransmuteResult::Removed(reduced),
        });
        (HashTrieSet{set}, reduced)
    }

}

impl <H: Hashword, F: Flagword<H>, K: Key, M: HasherBv<H, K>> Default for HashTrieSet<H, F, K, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn default() -> Self {
        Self::new()
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, M: HasherBv<H, K>> Eq for HashTrieSet<H, F, K, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {}

impl <H: Hashword, F: Flagword<H>, K: Key, M: HasherBv<H, K>> PartialEq for HashTrieSet<H, F, K, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn eq(&self, other: &Self) -> bool {
        self.set == other.set
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use rand::Rng;
    
    #[test]
    fn set_transform() {
        let mut set = DefaultHashTrieSet::<i32>::new();

        for i in 1..101 {
            set = set.insert(i, false).unwrap().0;
        }

        let removed = set.transform(|_,_| (), |_| SetTransformResult::Removed(()));
        let summed = set.transform(|&l,&r| l + r, |&k| SetTransformResult::Removed(k));

        assert_eq!(removed.0.size(), 0);
        assert_eq!(summed.1, 5050);
    }

    #[test]
    fn set_transmute() {
        let mut set = DefaultHashTrieSet::<i32>::new();

        for i in 1..101 {
            set = set.insert(i, false).unwrap().0;
        }

        let removed = unsafe { set.transmute(|_,_| (), |_| SetTransmuteResult::Removed(())) };
        let summed = unsafe { set.transmute(|&l,&r| l + r, |&k| SetTransmuteResult::Removed(k)) };

        assert_eq!(removed.0.size(), 0);
        assert_eq!(summed.1, 5050);
    }
    
    #[test]
    fn set_joint_transformations() {
        let mut seta = DefaultHashTrieSet::<i32>::new();
        let mut setb = DefaultHashTrieSet::<i32>::new();

        let mut rng = rand::thread_rng();
        let a = (0..25000).map(|_| rng.gen_range(0..100000)).collect::<Vec<i32>>();
        let b = (0..25000).map(|_| rng.gen_range(0..100000)).collect::<Vec<i32>>();
        for i in a {
            seta = seta.insert(i, true).unwrap().0;
        }
        for i in b {
            setb = setb.insert(i, true).unwrap().0;
        }

        let ff = seta.transform_with_transformed(&setb, |l,r| -> i32 {l.wrapping_add(*r)},
            |l,r| SetJointTransformResult::Removed(l.wrapping_mul(*r)), |l| SetTransformResult::Unchanged(*l), |r| SetTransformResult::Unchanged(*r));
        let fm = unsafe { seta.transform_with_transmuted(&setb, |l,r| -> i32 {l.wrapping_add(*r)},
            |l,r| SetTransformResult::Removed(l.wrapping_mul(*r)), |l| SetTransformResult::Unchanged(*l), |r| SetTransmuteResult::Transmuted(*r, *r)) };
        let mf = unsafe { seta.transmute_with_transformed(&setb, |l,r| -> i32 {l.wrapping_add(*r)},
            |l,r| SetTransformResult::Removed(l.wrapping_mul(*r)), |l| SetTransmuteResult::Transmuted(*l, *l), |r| SetTransformResult::Unchanged(*r)) };
        let mm = unsafe { seta.transmute_with_transmuted(&setb, |l,r| -> i32 {l.wrapping_add(*r)},
            |l,r| SetTransmuteResult::Removed(l.wrapping_mul(*r)), |l| SetTransmuteResult::Transmuted(*l, *l), |r| SetTransmuteResult::Transmuted(*r, *r)) };

        assert_eq!(ff.1, fm.1);
        assert_eq!(ff.1, mf.1);
        assert_eq!(ff.1, mm.1);

        let ffx = ff.0.transform(|l,r| -> i32 {l.wrapping_add(*r)}, |k| SetTransformResult::Removed(*k));
        let fmx = fm.0.transform(|l,r| -> i32 {l.wrapping_add(*r)}, |k| SetTransformResult::Removed(*k));
        let mfx = mf.0.transform(|l,r| -> i32 {l.wrapping_add(*r)}, |k| SetTransformResult::Removed(*k));
        let mmx = mm.0.transform(|l,r| -> i32 {l.wrapping_add(*r)}, |k| SetTransformResult::Removed(*k));

        assert_eq!(ffx.1, fmx.1);
        assert_eq!(ffx.1, mfx.1);
        assert_eq!(ffx.1, mmx.1);
    }

}
