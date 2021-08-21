use crate::results::*;

/// `MapJointTransform` represents a transformation that can be run on a pair of maps.
#[derive(Clone)]
pub enum MapJointTransform<ReduceT, F> {
    /// `MapJointTransform::Generic` can store an arbitrary joint transformation. The other values should be preferred when applicable, since they can open up optimization potential.
    Generic(F, bool),
    /// `MapJointTransform::UnchangedLR` indicates the value of either side of the joint transformation should be reused in the new map.
    UnchangedLR(ReduceT),
    /// `MapJointTransform::UnchangedL` indicates the value of the left side only should be reused in the new map.
    UnchangedL(ReduceT),
    /// `MapJointTransform::UnchangedR` indicates the value of the right side only should be reused in the new map.
    UnchangedR(ReduceT),
    /// `MapJointTransform::Removed` indicates the value should be removed from the map.
    Removed(ReduceT),
}

impl <ReduceT: Clone, F: Clone> MapJointTransform<ReduceT, F> {
    /// Call the joint transformation and return a result.
    pub fn call<K, V>(&self, k: &K, v: &V, l: &K, w: &V) -> MapJointTransformResult<V, ReduceT> where F: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>
    {
        match self {
            Self::Generic(f, flipped) => {
                if *flipped {
                    f(l, w, k, v).flip()
                }
                else {
                    f(k, v, l, w)
                }
            },
            Self::UnchangedLR(r) => MapJointTransformResult::UnchangedLR(r.clone()),
            Self::UnchangedL(r) => MapJointTransformResult::UnchangedL(r.clone()),
            Self::UnchangedR(r) => MapJointTransformResult::UnchangedR(r.clone()),
            Self::Removed(r) => MapJointTransformResult::Removed(r.clone()),
        }
    }

    /// Flip the joint transformation so it can respond correctly to arguments in the reverse order.
    pub fn flip(&self) -> Self
    {
        match self {
            Self::Generic(f, flipped) => Self::Generic(f.clone(), !flipped),
            Self::UnchangedLR(r) => Self::UnchangedLR(r.clone()),
            Self::UnchangedL(r) => Self::UnchangedL(r.clone()),
            Self::UnchangedR(r) => Self::UnchangedR(r.clone()),
            Self::Removed(r) => Self::Removed(r.clone()),
        }
    }
}

/// Construct a generic joint transformation.
pub fn new_map_joint_transform_generic<K, V, ReduceT: Clone, F: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone>(f: F) -> MapJointTransform<ReduceT, F> {
    MapJointTransform::Generic(f, false)
}

/// Construct a left/right indifferent identity joint transformation.
pub fn new_map_joint_transform_unchangedlr<K, V, ReduceT: Clone>(r: ReduceT) -> MapJointTransform<ReduceT, fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>> {
    MapJointTransform::UnchangedLR(r)
}

/// Construct a left-identity joint transformation.
pub fn new_map_joint_transform_unchangedl<K, V, ReduceT: Clone>(r: ReduceT) -> MapJointTransform<ReduceT, fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>> {
    MapJointTransform::UnchangedL(r)
}

/// Construct a right-identity joint transformation.
pub fn new_map_joint_transform_unchangedr<K, V, ReduceT: Clone>(r: ReduceT) -> MapJointTransform<ReduceT, fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>> {
    MapJointTransform::UnchangedR(r)
}

/// Construct a null joint transformation.
pub fn new_map_joint_transform_removed<K, V, ReduceT: Clone>(r: ReduceT) -> MapJointTransform<ReduceT, fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>> {
    MapJointTransform::Removed(r)
}

/// `SetJointTransform` represents a transformation that can be run on a pair of sets.
#[derive(Clone)]
pub enum SetJointTransform<ReduceT, F> {
    /// `SetJointTransform::Generic` can store an arbitrary joint transformation. The other values should be preferred when applicable, since they can open up optimization potential.
    Generic(F, bool),
    /// `SetJointTransform::UnchangedLR` indicates the value of either side of the joint transformation should be reused in the new set.
    UnchangedLR(ReduceT),
    /// `SetJointTransform::UnchangedL` indicates the value of the left side should be reused in the new set.
    UnchangedL(ReduceT),
    /// `SetJointTransform::UnchangedR` indicates the value of the right side should be reused in the new set.
    UnchangedR(ReduceT),
    /// `SetJointTransform::Removed` indicates the value should be removed from the set.
    Removed(ReduceT),
}

impl <ReduceT: Clone, F: Clone> SetJointTransform<ReduceT, F> {
    /// Call the joint transformation and return a result.
    pub fn call<K>(&self, k: &K, l: &K) -> SetJointTransformResult<ReduceT> where F: Fn(&K, &K) -> SetJointTransformResult<ReduceT>
    {
        match self {
            Self::Generic(f, flipped) => {
                if *flipped {
                    f(l, k).flip()
                }
                else {
                    f(k, l)
                }
            },
            Self::UnchangedLR(r) => SetJointTransformResult::UnchangedLR(r.clone()),
            Self::UnchangedL(r) => SetJointTransformResult::UnchangedL(r.clone()),
            Self::UnchangedR(r) => SetJointTransformResult::UnchangedR(r.clone()),
            Self::Removed(r) => SetJointTransformResult::Removed(r.clone()),
        }
    }

    /// Flip the joint transformation so it can respond correctly to arguments in the reverse order.
    pub fn flip(&self) -> Self
    {
        match self {
            Self::Generic(f, flipped) => Self::Generic(f.clone(), !flipped),
            Self::UnchangedLR(r) => Self::UnchangedLR(r.clone()),
            Self::UnchangedL(r) => Self::UnchangedL(r.clone()),
            Self::UnchangedR(r) => Self::UnchangedR(r.clone()),
            Self::Removed(r) => Self::Removed(r.clone()),
        }
    }
}

/// Construct a generic joint transformation.
pub fn new_set_joint_transform_generic<K, ReduceT: Clone, F: Fn(&K, &K) -> SetJointTransformResult<ReduceT> + Clone>(f: F) -> SetJointTransform<ReduceT, F> {
    SetJointTransform::Generic(f, false)
}

/// Construct a left/right indifferent identity joint transformation.
pub fn new_set_joint_transform_unchangedlr<K, ReduceT: Clone>(r: ReduceT) -> SetJointTransform<ReduceT, fn(&K, &K) -> SetJointTransformResult<ReduceT>> {
    SetJointTransform::UnchangedLR(r)
}

/// Construct a left-identity joint transformation.
pub fn new_set_joint_transform_unchangedl<K, ReduceT: Clone>(r: ReduceT) -> SetJointTransform<ReduceT, fn(&K, &K) -> SetJointTransformResult<ReduceT>> {
    SetJointTransform::UnchangedL(r)
}

/// Construct a right-identity joint transformation.
pub fn new_set_joint_transform_unchangedr<K, ReduceT: Clone>(r: ReduceT) -> SetJointTransform<ReduceT, fn(&K, &K) -> SetJointTransformResult<ReduceT>> {
    SetJointTransform::UnchangedR(r)
}

/// Construct a null joint transformation.
pub fn new_set_joint_transform_removed<K, ReduceT: Clone>(r: ReduceT) -> SetJointTransform<ReduceT, fn(&K, &K) -> SetJointTransformResult<ReduceT>> {
    SetJointTransform::Removed(r)
}

/// `MapTransform` represents a transformation that can be run on a map.
#[derive(Clone)]
pub enum MapTransform<ReduceT, F> {
    /// `MapTransform::Generic` can store an arbitrary transformation. The other values should be preferred when applicable, since they can open up optimization potential.
    Generic(F),
    /// `MapTransform::UnchangedL` indicates the original value should be reused in the new map.
    Unchanged(ReduceT),
    /// `MapTransform::Removed` indicates the value should be removed from the map.
    Removed(ReduceT),
}

impl <ReduceT: Clone, F> MapTransform<ReduceT, F> {
    /// Call the transformation and return a result.
    pub fn call<K, V>(&self, k: &K, v: &V) -> MapTransformResult<V, ReduceT> where F: Fn(&K, &V) -> MapTransformResult<V, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, v),
            Self::Unchanged(r) => MapTransformResult::Unchanged(r.clone()),
            Self::Removed(r) => MapTransformResult::Removed(r.clone()),
        }
    }

    /// Call the joint transformation/transmutation and return a result.
    pub fn call_transform_transmute<K, V, L, W>(&self, k: &K, v: &V, l: &L, w: &W) -> MapTransformResult<V, ReduceT> where F: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, v, l, w),
            Self::Unchanged(r) => MapTransformResult::Unchanged(r.clone()),
            Self::Removed(r) => MapTransformResult::Removed(r.clone()),
        }
    }

    /// Call the joint transmutation/transformation and return a result.
    pub fn call_transmute_transform<K, V, L, W>(&self, k: &K, v: &V, l: &L, w: &W) -> MapTransformResult<W, ReduceT> where F: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, v, l, w),
            Self::Unchanged(r) => MapTransformResult::Unchanged(r.clone()),
            Self::Removed(r) => MapTransformResult::Removed(r.clone()),
        }
    }

    /// Flip the joint transformation/transmutation so it can respond correctly to arguments in the reverse order.
    pub fn flip_transform_transmute<K, V, L, W>(&self) -> MapTransform<ReduceT, impl Fn(&L, &W, &K, &V) -> MapTransformResult<V, ReduceT> + Clone>
    where F: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT> + Clone
    {
        match self {
            Self::Generic(f) => {
                let g = f.clone();
                MapTransform::Generic(move |l: &_, w: &_, k: &_, v: &_| g(k, v, l, w))
            },
            Self::Unchanged(r) => MapTransform::Unchanged(r.clone()),
            Self::Removed(r) => MapTransform::Removed(r.clone()),
        }
    }

    /// Flip the joint transmutation/transformation so it can respond correctly to arguments in the reverse order.
    pub fn flip_transmute_transform<K, V, L, W>(&self) -> MapTransform<ReduceT, impl Fn(&L, &W, &K, &V) -> MapTransformResult<W, ReduceT> + Clone>
    where F: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT> + Clone
    {
        match self {
            Self::Generic(f) => {
                let g = f.clone();
                MapTransform::Generic(move |l: &_, w: &_, k: &_, v: &_| g(k, v, l, w))
            },
            Self::Unchanged(r) => MapTransform::Unchanged(r.clone()),
            Self::Removed(r) => MapTransform::Removed(r.clone()),
        }
    }
}

/// Construct a generic transformation.
pub fn new_map_transform_generic<K, V, ReduceT: Clone, F: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone>(f: F) -> MapTransform<ReduceT, F> {
    MapTransform::Generic(f)
}

/// Construct an identity joint transformation.
pub fn new_map_transform_unchanged<K, V, ReduceT: Clone>(r: ReduceT) -> MapTransform<ReduceT, fn(&K, &V) -> MapTransformResult<V, ReduceT>> {
    MapTransform::Unchanged(r)
}

/// Construct a null transformation.
pub fn new_map_transform_removed<K, V, ReduceT: Clone>(r: ReduceT) -> MapTransform<ReduceT, fn(&K, &V) -> MapTransformResult<V, ReduceT>> {
    MapTransform::Removed(r)
}

/// Construct a generic joint transformation/transmutation.
pub fn new_map_transform_transmute_generic<K, V, L, W, ReduceT: Clone, F: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT> + Clone>(f: F) -> MapTransform<ReduceT, F> {
    MapTransform::Generic(f)
}

/// Construct a left-identity joint transformation/transmutation.
pub fn new_map_transform_transmute_unchanged<K, V, L, W, ReduceT: Clone>(r: ReduceT) -> MapTransform<ReduceT, fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT>> {
    MapTransform::Unchanged(r)
}

/// Construct a null joint transformation/transmutation.
pub fn new_map_transform_transmute_removed<K, V, L, W, ReduceT: Clone>(r: ReduceT) -> MapTransform<ReduceT, fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT>> {
    MapTransform::Removed(r)
}

/// Construct a generic joint transmutation/transformation.
pub fn new_map_transmute_transform_generic<K, V, L, W, ReduceT: Clone, F: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT> + Clone>(f: F) -> MapTransform<ReduceT, F> {
    MapTransform::Generic(f)
}

/// Construct a right-identity joint transmutation/transformation.
pub fn new_map_transmute_transform_unchanged<K, V, L, W, ReduceT: Clone>(r: ReduceT) -> MapTransform<ReduceT, fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT>> {
    MapTransform::Unchanged(r)
}

/// Construct a null joint transmutation/transformation.
pub fn new_map_transmute_transform_removed<K, V, L, W, ReduceT: Clone>(r: ReduceT) -> MapTransform<ReduceT, fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT>> {
    MapTransform::Removed(r)
}

/// `SetTransform` represents a transformation that can be run on a set.
#[derive(Clone)]
pub enum SetTransform<ReduceT, F> {
    /// `SetTransform::Generic` can store an arbitrary transformation. The other values should be preferred when applicable, since they can open up optimization potential.
    Generic(F),
    /// `SetTransform::UnchangedL` indicates the original value should be reused in the new set.
    Unchanged(ReduceT),
    /// `SetTransform::Removed` indicates the value should be removed from the set.
    Removed(ReduceT),
}

impl <ReduceT: Clone, F> SetTransform<ReduceT, F> {
    /// Call the transformation and return a result.
    pub fn call<K>(&self, k: &K) -> SetTransformResult<ReduceT> where F: Fn(&K) -> SetTransformResult<ReduceT>
    {
        match self {
            Self::Generic(f) => f(k),
            Self::Unchanged(r) => SetTransformResult::Unchanged(r.clone()),
            Self::Removed(r) => SetTransformResult::Removed(r.clone()),
        }
    }

    /// Call the transmutation and return a result.
    pub fn call_transmute<K, L>(&self, k: &K, l: &L) -> SetTransformResult<ReduceT> where F: Fn(&K, &L) -> SetTransformResult<ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, l),
            Self::Unchanged(r) => SetTransformResult::Unchanged(r.clone()),
            Self::Removed(r) => SetTransformResult::Removed(r.clone()),
        }
    }

    /// Flip the joint transformation/transmutation so it can respond correctly to arguments in the reverse order.
    pub fn flip_transmute<K, L>(&self) -> SetTransform<ReduceT, impl Fn(&L, &K) -> SetTransformResult<ReduceT> + Clone>
    where F: Fn(&K, &L) -> SetTransformResult<ReduceT> + Clone
    {
        match self {
            Self::Generic(f) => {
                let g = f.clone();
                SetTransform::Generic(move |l: &_, k: &_| g(k, l))
            },
            Self::Unchanged(r) => SetTransform::Unchanged(r.clone()),
            Self::Removed(r) => SetTransform::Removed(r.clone()),
        }
    }
}

/// Construct a generic transformation.
pub fn new_set_transform_generic<K, ReduceT: Clone, F: Fn(&K) -> SetTransformResult<ReduceT> + Clone>(f: F) -> SetTransform<ReduceT, F> {
    SetTransform::Generic(f)
}

/// Construct an identity transformation.
pub fn new_set_transform_unchanged<K, ReduceT: Clone>(r: ReduceT) -> SetTransform<ReduceT, fn(&K) -> SetTransformResult<ReduceT>> {
    SetTransform::Unchanged(r)
}

/// Construct a null transformation.
pub fn new_set_transform_removed<K, ReduceT: Clone>(r: ReduceT) -> SetTransform<ReduceT, fn(&K) -> SetTransformResult<ReduceT>> {
    SetTransform::Removed(r)
}

/// Construct a generic joint transformation/transmutation.
pub fn new_set_transform_transmute_generic<K, L, ReduceT: Clone, F: Fn(&K, &L) -> SetTransformResult<ReduceT> + Clone>(f: F) -> SetTransform<ReduceT, F> {
    SetTransform::Generic(f)
}

/// Construct a left/right identity joint transformation/transmutation.
pub fn new_set_transform_transmute_unchanged<K, L, ReduceT: Clone>(r: ReduceT) -> SetTransform<ReduceT, fn(&K, &L) -> SetTransformResult<ReduceT>> {
    SetTransform::Unchanged(r)
}

/// Construct a generic joint transformation/transmutation.
pub fn new_set_transform_transmute_removed<K, L, ReduceT: Clone>(r: ReduceT) -> SetTransform<ReduceT, fn(&K, &L) -> SetTransformResult<ReduceT>> {
    SetTransform::Removed(r)
}

/// `MapTransmute` represents a transmutation that can be run on a map.
#[derive(Clone)]
pub enum MapTransmute<ReduceT, F> {
    /// `MapTransmute::Generic` can store an arbitrary transmutation.
    Generic(F),
    /// `MapTransmute::Removed` indicates the value should be removed from the map.
    Removed(ReduceT),
}

impl <ReduceT: Clone, F> MapTransmute<ReduceT, F> {
    /// Call the transmutation and return a result.
    pub fn call<K, V, S, X>(&self, k: &K, v: &V) -> MapTransmuteResult<S, X, ReduceT> where F: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, v),
            Self::Removed(r) => MapTransmuteResult::Removed(r.clone()),
        }
    }

    /// Call the joint transmutation and return a result.
    pub fn call_transmute<K, V, L, W, S, X>(&self, k: &K, v: &V, l: &L, w: &W) -> MapTransmuteResult<S, X, ReduceT> where F: Fn(&K, &V, &L, &W) -> MapTransmuteResult<S, X, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, v, l, w),
            Self::Removed(r) => MapTransmuteResult::Removed(r.clone()),
        }
    }

    /// Flip the joint transmutation so it can respond correctly to arguments in the reverse order.
    pub fn flip<K, V, L, W, S, X>(&self) -> MapTransmute<ReduceT, impl Fn(&L, &W, &K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone>
    where F: Fn(&K, &V, &L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone
    {
        match self {
            Self::Generic(f) => {
                let g = f.clone();
                MapTransmute::Generic(move |l: &_, w: &_, k: &_, v: &_| g(k, v, l, w))
            },
            Self::Removed(r) => MapTransmute::Removed(r.clone()),
        }
    }
}

/// Construct a generic transmutation.
pub fn new_map_transmute_generic<K, V, S, X, ReduceT: Clone, F: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone>(f: F) -> MapTransmute<ReduceT, F> {
    MapTransmute::Generic(f)
}

/// Construct a null transmutation.
pub fn new_map_transmute_removed<K, V, S, X, ReduceT: Clone>(r: ReduceT) -> MapTransmute<ReduceT, fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT>> {
    MapTransmute::Removed(r)
}

/// Construct a generic joint transmutation.
pub fn new_map_transmute_transmute_generic<K, V, L, W, S, X, ReduceT: Clone, F: Fn(&K, &V, &L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone>(f: F) -> MapTransmute<ReduceT, F> {
    MapTransmute::Generic(f)
}

/// Construct a null joint transmutation.
pub fn new_map_transmute_transmute_removed<K, V, L, W, S, X, ReduceT: Clone>(r: ReduceT) -> MapTransmute<ReduceT, fn(&K, &V, &L, &W) -> MapTransmuteResult<S, X, ReduceT>> {
    MapTransmute::Removed(r)
}

/// `SetTransmute` represents a transmutation that can be run on a set.
#[derive(Clone)]
pub enum SetTransmute<ReduceT, F> {
    /// `SetTransmute::Generic` can store an arbitrary transmutation.
    Generic(F),
    /// `SetTransmute::Removed` indicates the value should be removed from the set.
    Removed(ReduceT),
}

impl <ReduceT: Clone, F> SetTransmute<ReduceT, F> {
    /// Call the transmutation and return a result.
    pub fn call<K, S>(&self, k: &K) -> SetTransmuteResult<S, ReduceT> where F: Fn(&K) -> SetTransmuteResult<S, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k),
            Self::Removed(r) => SetTransmuteResult::Removed(r.clone()),
        }
    }

    /// Call the joint transmutation and return a result.
    pub fn call_transmute<K, L, S>(&self, k: &K, l: &L) -> SetTransmuteResult<S, ReduceT> where F: Fn(&K, &L) -> SetTransmuteResult<S, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, l),
            Self::Removed(r) => SetTransmuteResult::Removed(r.clone()),
        }
    }

    /// Flip the joint transmutation so it can respond correctly to arguments in the reverse order.
    pub fn flip<K, L, S>(&self) -> SetTransmute<ReduceT, impl Fn(&L, &K) -> SetTransmuteResult<S, ReduceT> + Clone>
    where F: Fn(&K, &L) -> SetTransmuteResult<S, ReduceT> + Clone
    {
        match self {
            Self::Generic(f) => {
                let g = f.clone();
                SetTransmute::Generic(move |l: &_, k: &_| g(k, l))
            },
            Self::Removed(r) => SetTransmute::Removed(r.clone()),
        }
    }
}

/// Construct a generic transmutation.
pub fn new_set_transmute_generic<K, S, ReduceT: Clone, F: Fn(&K) -> SetTransmuteResult<S, ReduceT> + Clone>(f: F) -> SetTransmute<ReduceT, F> {
    SetTransmute::Generic(f)
}

/// Construct a null transmutation.
pub fn new_set_transmute_removed<K, S, ReduceT: Clone>(r: ReduceT) -> SetTransmute<ReduceT, fn(&K) -> SetTransmuteResult<S, ReduceT>> {
    SetTransmute::Removed(r)
}

/// Construct a generic joint transmutation.
pub fn new_set_transmute_transmute_generic<K, L, S, ReduceT: Clone, F: Fn(&K, &L) -> SetTransmuteResult<S, ReduceT> + Clone>(f: F) -> SetTransmute<ReduceT, F> {
    SetTransmute::Generic(f)
}

/// Construct a null joint transmutation.
pub fn new_set_transmute_transmute_removed<K, L, S, ReduceT: Clone>(r: ReduceT) -> SetTransmute<ReduceT, fn(&K, &L) -> SetTransmuteResult<S, ReduceT>> {
    SetTransmute::Removed(r)
}
