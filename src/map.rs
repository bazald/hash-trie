use crate::{results::*, transformations::*, traits::*, hash_trie::HashTrie, *};
use alloc::fmt::Debug;

/// `HashTrieMap` implements a hash map using a hash array mapped trie (HAMT).
/// 
/// # Example Usage
/// 
/// ```
/// use fnv::FnvHasher;
/// use hash_trie::{HashTrieMap, traits::HashLike};
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
/// let mut hash_map: HashTrieMap<u64, u32, String, String, FnvHasher> = HashTrieMap::new();
/// let hello = "Hello,";
/// let world = "world!,";
/// 
/// hash_map = hash_map.insert(Str::new(hello), world, false).unwrap().0;
/// 
/// // Inserting an already-inserted key returns references to the key and value in the map...
/// assert!(hash_map.insert(Str::new(hello), "?", false).map(|_| ())
///     .map_err(|key_value| *key_value.0 == hello && *key_value.1 == world).unwrap_err());
/// // ... unless you enable replacement.
/// assert!(hash_map.insert(Str::new(hello), "?", true).is_ok());
/// 
/// assert!(hash_map.find(&Str::new(hello)).map(|key_value| *key_value.0 == hello && *key_value.1 == world).unwrap());
/// 
/// match hash_map.remove(&Str::new(hello)) {
///     Ok((mutated, key, value)) => {
///         // Removing a key returns references to the key and
///         // value in the set in addition to the mutated map.
///         println!("Value stored in hash_map: ({}, {})", key, value);
///         hash_map = mutated;
///     },
///     Err(_) => panic!(),
/// }
/// ```
#[derive(Clone, Debug)]
#[must_use]
pub struct HashTrieMap <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    set: HashTrie<H, F, K, V, M>,
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> HashTrieMap<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    /// Get a new, empty HashTrieMap.
    pub fn new() -> Self {
        Self {
            set: HashTrie::<H, F, K, V, M>::new()
        }
    }

    /// Get the total number of entries in the map.
    pub fn size(&self) -> usize {
        self.set.size()
    }

    /// Search the HashTrieMap for the given key and return references if found, or `HashTrieError::NotFound` if not found.
    pub fn find<'a, L: Key + HashLike<K>>(&'a self, key: &L) -> Result<(&'a K, &'a V), HashTrieError> where K: PartialEq<L>, M: HasherBv<H, L> {
        self.set.find(key)
    }

    /// Search the HashTrieMap for the spot to insert the key and return both a mutated map and, if applicable, references to the replaced values.
    /// If found and replacement is disabled, references to the existing values are returned.
    #[allow(clippy::type_complexity)]
    pub fn insert<'a, L: Key + HashLike<K> + Into<K>, W: Into<V>>(&'a self, key: L, value: W, replace: bool) -> Result<(Self, *const K, *const V, Option<(&'a K, &'a V)>), (&'a K, &'a V)>
    where
        K: HashLike<L>,
        K: PartialEq<L>,
        M: HasherBv<H, L>
    {
        self.set.insert(key, value, replace).map(|(set, key, value, prev)| (Self {set}, key, value, prev))
    }

    /// Search the HashTrieMap for the given key to remove and return a mutated map, or `HashTrieError::NotFound` if not found.
    pub fn remove<'a, L: Key + HashLike<K>>(&'a self, key: &L) -> Result<(Self, &'a K, &'a V), HashTrieError> where K: PartialEq<L>, M: HasherBv<H, L> {
        self.set.remove(key).map(|(set, key, value)| (Self {set}, key, value))
    }

    /// Run an operation on each entry in the map.
    pub fn visit<Op: Clone>(&self, op: Op) where Op: Fn(&K, &V) {
        self.set.visit(|k,v| op(k, v));
    }

    /// Run a transform operation on each entry in the map. Returns the transformed map and a reduction of the secondary returns of the transform operations.
    pub fn transform<ReduceT, ReduceOp, Op>
        (&self, reduce_op: ReduceOp, op: MapTransform<ReduceT, Op>, par_strat: ParallelismStrategy) -> (Self, ReduceT)
        where
        Self: Sized,
        ReduceT: Clone + Default + Send + Sync,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
        Op: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync
    {
        let (set, reduced) = self.set.transform(reduce_op, op, par_strat);
        (Self{set}, reduced)
    }

    /// Run a transmute operation on each entry in the map. Returns the transmuted map and a reduction of the secondary returns of the transmute operations.
    pub unsafe fn transmute<S: Key + HashLike<K>, X: Value, ReduceT, ReduceOp, Op>
        (&self, reduce_op: ReduceOp, op: MapTransmute<ReduceT, Op>) -> (HashTrieMap<H, F, S, X, M>, ReduceT)
        where
        Self: Sized,
        ReduceT: Clone + Default + Send + Sync,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
        Op: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        K: HashLike<S>,
        K: PartialEq<S>,
        M: HasherBv<H, S>,
    {
        let (set, reduced) = self.set.transmute(reduce_op, op);
        (HashTrieMap{set}, reduced)
    }

    /// Run a transform operation on each entry or pair of entries in the maps. Returns the transformed map and a reduction of the secondary returns of the transmute operations. Can reuse nodes from either map.
    pub fn transform_with_transformed<ReduceT, ReduceOp, BothOp, LeftOp, RightOp>
        (&self, right: &Self, reduce_op: ReduceOp, both_op: MapJointTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>, par_strat: ParallelismStrategy) -> (Self, ReduceT)
        where
        Self: Sized,
        ReduceT: Clone + Default + Send + Sync,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
        BothOp: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone + Send + Sync,
        LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
        RightOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
    {
        let (set, reduced) = self.set.transform_with_transformed(&right.set, reduce_op, both_op, left_op, right_op, par_strat);
        (HashTrieMap{set}, reduced)
    }

    /// Run a transform/transmute operation on each entry or pair of entries in the maps. Returns the transmuted map and a reduction of the secondary returns of the transmute operations. Can reuse nodes from the transformed map. Like transform_with_transmuted but enforces identity transformations on keys.
    pub fn transform_with_transfuted<W: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>
        (&self, right: &HashTrieMap<H, F, K, W, M>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: SetTransmute<ReduceT, RightOp>, par_strat: ParallelismStrategy) -> (Self, ReduceT)
        where
        Self: Sized,
        ReduceT: Clone + Default + Send + Sync,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
        BothOp: Fn(&K, &V, &K, &W) -> MapTransformResult<V, ReduceT> + Clone,
        LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
        RightOp: Fn(&K, &W) -> SetTransmuteResult<V, ReduceT> + Clone + Send + Sync,
    {
        let (set, reduced) = unsafe {
            self.set.transform_with_transmuted(&right.set, reduce_op, both_op, left_op, match right_op {
                SetTransmute::Generic(g) => new_map_transmute_generic(move |k, w| match g(k, w) {
                    SetTransmuteResult::Transmuted(value, reduced) => MapTransmuteResult::Transmuted(k.clone(), value, reduced),
                    SetTransmuteResult::Removed(reduced) => MapTransmuteResult::Removed(reduced),
                }),
                SetTransmute::Removed(r) => MapTransmute::Removed(r),
            }, par_strat)
        };
        (HashTrieMap{set}, reduced)
    }

    /// Run a transform/transmute operation on each entry or pair of entries in the maps. Returns the transmuted map and a reduction of the secondary returns of the transmute operations. Can reuse nodes from the transformed map.
    pub unsafe fn transform_with_transmuted<L: Key + HashLike<K>, W: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>
        (&self, right: &HashTrieMap<H, F, L, W, M>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransform<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>, par_strat: ParallelismStrategy) -> (Self, ReduceT)
        where
        Self: Sized,
        ReduceT: Clone + Default + Send + Sync,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
        BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT> + Clone,
        LeftOp: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone + Send + Sync,
        RightOp: Fn(&L, &W) -> MapTransmuteResult<K, V, ReduceT> + Clone + Send + Sync,
        K: HashLike<L>,
        K: PartialEq<L>,
        L: HashLike<K>,
        L: PartialEq<K>,
        M: HasherBv<H, L>,
    {
        let (set, reduced) = self.set.transform_with_transmuted(&right.set, reduce_op, both_op, left_op, right_op, par_strat);
        (HashTrieMap{set}, reduced)
    }

    /// Run a transmute/transform operation on each entry or pair of entries in the maps. Returns the transmuted map and a reduction of the secondary returns of the transmute operations. Can reuse nodes from the transformed map. Like transmute_with_transformed but enforces identity transformations on keys.
    pub fn transfute_with_transformed<W: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>
        (&self, right: &HashTrieMap<H, F, K, W, M>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: SetTransmute<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>, par_strat: ParallelismStrategy) -> (HashTrieMap<H, F, K, W, M>, ReduceT)
        where
        Self: Sized,
        ReduceT: Clone + Default + Send + Sync,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
        BothOp: Fn(&K, &V, &K, &W) -> MapTransformResult<W, ReduceT> + Clone,
        LeftOp: Fn(&K, &V) -> SetTransmuteResult<W, ReduceT> + Clone + Send + Sync,
        RightOp: Fn(&K, &W) -> MapTransformResult<W, ReduceT> + Clone + Send + Sync,
    {
        let (set, reduced) = unsafe {
            self.set.transmute_with_transformed(&right.set, reduce_op, both_op, match left_op {
                SetTransmute::Generic(g) => new_map_transmute_generic(move |k, v| match g(k, v) {
                    SetTransmuteResult::Transmuted(value, reduced) => MapTransmuteResult::Transmuted(k.clone(), value, reduced),
                    SetTransmuteResult::Removed(reduced) => MapTransmuteResult::Removed(reduced),
                }),
                SetTransmute::Removed(r) => MapTransmute::Removed(r),
            }, right_op, par_strat)
        };
        (HashTrieMap{set}, reduced)
    }

    /// Run a transmute/transform operation on each entry or pair of entries in the maps. Returns the transmuted map and a reduction of the secondary returns of the transmute operations. Can reuse nodes from the transformed map.
    pub unsafe fn transmute_with_transformed<L: Key + HashLike<K>, W: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>
        (&self, right: &HashTrieMap<H, F, L, W, M>, reduce_op: ReduceOp, both_op: MapTransform<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransform<ReduceT, RightOp>, par_strat: ParallelismStrategy) -> (HashTrieMap<H, F, L, W, M>, ReduceT)
        where
        Self: Sized,
        ReduceT: Clone + Default + Send + Sync,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
        BothOp: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT> + Clone,
        LeftOp: Fn(&K, &V) -> MapTransmuteResult<L, W, ReduceT> + Clone + Send + Sync,
        RightOp: Fn(&L, &W) -> MapTransformResult<W, ReduceT> + Clone + Send + Sync,
        K: HashLike<L>,
        K: PartialEq<L>,
        L: HashLike<K>,
        L: PartialEq<K>,
        M: HasherBv<H, L>,
    {
        let (set, reduced) = self.set.transmute_with_transformed(&right.set, reduce_op, both_op, left_op, right_op, par_strat);
        (HashTrieMap{set}, reduced)
    }

    /// Run a transmute operation on each entry or pair of entries in the maps. Returns the transmuted map and a reduction of the secondary returns of the transmute operations.
    pub unsafe fn transmute_with_transmuted<L: Key + HashLike<K>, W: Value, S: Key + HashLike<K>, X: Value, ReduceT, ReduceOp, BothOp, LeftOp, RightOp>
        (&self, right: &HashTrieMap<H, F, L, W, M>, reduce_op: ReduceOp, both_op: MapTransmute<ReduceT, BothOp>, left_op: MapTransmute<ReduceT, LeftOp>, right_op: MapTransmute<ReduceT, RightOp>) -> (HashTrieMap<H, F, S, X, M>, ReduceT)
        where
        Self: Sized,
        ReduceT: Clone + Default + Send + Sync,
        ReduceOp: Fn(&ReduceT, &ReduceT) -> ReduceT + Clone + Send + Sync,
        BothOp: Fn(&K, &V, &L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        LeftOp: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone,
        RightOp: Fn(&L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone,
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
        let (set, reduced) = self.set.transmute_with_transmuted(&right.set, reduce_op, both_op, left_op, right_op);
        (HashTrieMap{set}, reduced)
    }

}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> Default for HashTrieMap<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn default() -> Self {
        Self::new()
    }
}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> Eq for HashTrieMap<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {}

impl <H: Hashword, F: Flagword<H>, K: Key, V: Value, M: HasherBv<H, K>> PartialEq for HashTrieMap<H, F, K, V, M> where <F as core::convert::TryFrom<<H as core::ops::BitAnd>::Output>>::Error: core::fmt::Debug {
    fn eq(&self, other: &Self) -> bool {
        self.set == other.set
    }
}

#[cfg(test)]
mod tests {
    use crate::{*, results::*, transformations::{new_map_joint_transform_generic, new_map_transform_generic, new_map_transform_removed, new_map_transform_transmute_generic, new_map_transmute_generic, new_map_transmute_removed, new_map_transmute_transform_generic, new_map_transmute_transmute_generic}};
    use rand::Rng;
    
    #[test]
    fn map_transform() {
        let mut map = DefaultHashTrieMap::<i32, i32>::new();
        let mut squared = DefaultHashTrieMap::<i32, i32>::new();

        for i in 1..101 {
            map = map.insert(i, i, false).unwrap().0;
            squared = squared.insert(i, i * i, false).unwrap().0;
        }

        let removed = map.transform(|_,_| (), new_map_transform_removed(()), ParallelismStrategy::default_par());
        let tsquared = map.transform(|_,_| (), new_map_transform_generic(|_,v| MapTransformResult::Transformed(v * v, ())), ParallelismStrategy::default_par());

        assert_eq!(removed.0.size(), 0);

        for i in 1..101 {
            map.find(&i).unwrap();
            assert_eq!(i * i, *squared.find(&i).unwrap().1);
            assert_eq!(i * i, *tsquared.0.find(&i).unwrap().1);
        }
    }
    
    #[test]
    fn map_transmute() {
        let mut map = DefaultHashTrieMap::<i32, i32>::new();
        let mut squared = DefaultHashTrieMap::<i32, i32>::new();

        for i in 1..101 {
            map = map.insert(i, i, false).unwrap().0;
            squared = squared.insert(i, i * i, false).unwrap().0;
        }

        let removed: (DefaultHashTrieMap::<i32, i32>, ()) = unsafe { map.transmute(|_,_| (), new_map_transmute_removed(())) };
        let tsquared = unsafe { map.transmute(|_,_| (), new_map_transmute_generic(|k,v| MapTransmuteResult::Transmuted(*k, v * v, ()))) };

        assert_eq!(removed.0.size(), 0);

        for i in 1..101 {
            map.find(&i).unwrap();
            assert_eq!(i * i, *squared.find(&i).unwrap().1);
            assert_eq!(i * i, *tsquared.0.find(&i).unwrap().1);
        }
    }
    
    #[test]
    fn map_joint_transformations() {
        let mut mapa = DefaultHashTrieMap::<i32, i32>::new();
        let mut mapb = DefaultHashTrieMap::<i32, i32>::new();

        let mut rng = rand::thread_rng();
        let a = (0..25000).map(|_| rng.gen_range(0..100000)).collect::<Vec<i32>>();
        let b = (0..25000).map(|_| rng.gen_range(0..100000)).collect::<Vec<i32>>();
        for i in a {
            mapa = mapa.insert(i, i, true).unwrap().0;
        }
        for i in b {
            mapb = mapb.insert(i, i, true).unwrap().0;
        }

        let ff = mapa.transform_with_transformed(&mapb, |l,r| -> i32 {l.wrapping_add(*r)},
        new_map_joint_transform_generic(|_,v:&i32,_,w| MapJointTransformResult::Removed(v.wrapping_mul(*w))), new_map_transform_generic(|_, v: &i32| MapTransformResult::Unchanged(*v)), new_map_transform_generic(|_,v| MapTransformResult::Unchanged(*v)), ParallelismStrategy::default_par());
        let fm = unsafe { mapa.transform_with_transmuted(&mapb, |l,r| -> i32 {l.wrapping_add(*r)},
            new_map_transform_transmute_generic(|_,v:&i32,_,w| MapTransformResult::Removed(v.wrapping_mul(*w))), new_map_transform_generic(|_,v| MapTransformResult::Unchanged(*v)), new_map_transmute_generic(|k,v| MapTransmuteResult::Transmuted(*k, *v, *v)), ParallelismStrategy::default_par()) };
        let mf = unsafe { mapa.transmute_with_transformed(&mapb, |l,r| -> i32 {l.wrapping_add(*r)},
            new_map_transmute_transform_generic(|_,v:&i32,_,w| MapTransformResult::Removed(v.wrapping_mul(*w))), new_map_transmute_generic(|k,v| MapTransmuteResult::Transmuted(*k, *v, *v)), new_map_transform_generic(|_,v| MapTransformResult::Unchanged(*v)), ParallelismStrategy::default_par()) };
        let mm = unsafe { mapa.transmute_with_transmuted(&mapb, |l,r| -> i32 {l.wrapping_add(*r)},
            new_map_transmute_transmute_generic(|_,v:&i32,_,w| MapTransmuteResult::Removed(v.wrapping_mul(*w))), new_map_transmute_generic(|k,v| MapTransmuteResult::Transmuted(*k, *v, *v)), new_map_transmute_generic(|k,v| MapTransmuteResult::Transmuted(*k, *v, *v))) };

        assert_eq!(ff.1, fm.1);
        assert_eq!(ff.1, mf.1);
        assert_eq!(ff.1, mm.1);

        let ffx = ff.0.transform(|l,r| -> i32 {l.wrapping_add(*r)}, new_map_transform_generic(|_,v| MapTransformResult::Removed(*v)), ParallelismStrategy::default_par());
        let fmx = fm.0.transform(|l,r| -> i32 {l.wrapping_add(*r)}, new_map_transform_generic(|_,v| MapTransformResult::Removed(*v)), ParallelismStrategy::default_par());
        let mfx = mf.0.transform(|l,r| -> i32 {l.wrapping_add(*r)}, new_map_transform_generic(|_,v| MapTransformResult::Removed(*v)), ParallelismStrategy::default_par());
        let mmx = mm.0.transform(|l,r| -> i32 {l.wrapping_add(*r)}, new_map_transform_generic(|_,v| MapTransformResult::Removed(*v)), ParallelismStrategy::default_par());

        assert_eq!(ffx.1, fmx.1);
        assert_eq!(ffx.1, mfx.1);
        assert_eq!(ffx.1, mmx.1);
    }

}
