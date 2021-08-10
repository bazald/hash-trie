use crate::result::*;

#[derive(Clone)]
pub enum MapJointTransform<ReduceT, F> {
    Generic(F, bool),
    UnchangedLR(ReduceT),
    UnchangedL(ReduceT),
    UnchangedR(ReduceT),
    Removed(ReduceT),
}

impl <ReduceT: Clone, F: Clone> MapJointTransform<ReduceT, F> {
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

pub fn new_map_joint_transform_generic<K, V, ReduceT: Clone, F: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT> + Clone>(f: F) -> MapJointTransform<ReduceT, F> {
    MapJointTransform::Generic(f, false)
}

pub fn new_map_joint_transform_unchangedlr<K, V, ReduceT: Clone>(r: ReduceT) -> MapJointTransform<ReduceT, fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>> {
    MapJointTransform::UnchangedLR(r)
}

pub fn new_map_joint_transform_unchangedl<K, V, ReduceT: Clone>(r: ReduceT) -> MapJointTransform<ReduceT, fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>> {
    MapJointTransform::UnchangedL(r)
}

pub fn new_map_joint_transform_unchangedr<K, V, ReduceT: Clone>(r: ReduceT) -> MapJointTransform<ReduceT, fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>> {
    MapJointTransform::UnchangedR(r)
}

pub fn new_map_joint_transform_removed<K, V, ReduceT: Clone>(r: ReduceT) -> MapJointTransform<ReduceT, fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>> {
    MapJointTransform::Removed(r)
}

#[derive(Clone)]
pub enum SetJointTransform<ReduceT, F> {
    Generic(F, bool),
    UnchangedLR(ReduceT),
    UnchangedL(ReduceT),
    UnchangedR(ReduceT),
    Removed(ReduceT),
}

impl <ReduceT: Clone, F: Clone> SetJointTransform<ReduceT, F> {
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

pub fn new_set_joint_transform_generic<K, ReduceT: Clone, F: Fn(&K, &K) -> SetJointTransformResult<ReduceT> + Clone>(f: F) -> SetJointTransform<ReduceT, F> {
    SetJointTransform::Generic(f, false)
}

pub fn new_set_joint_transform_unchangedlr<K, ReduceT: Clone>(r: ReduceT) -> SetJointTransform<ReduceT, fn(&K, &K) -> SetJointTransformResult<ReduceT>> {
    SetJointTransform::UnchangedLR(r)
}

pub fn new_set_joint_transform_unchangedl<K, ReduceT: Clone>(r: ReduceT) -> SetJointTransform<ReduceT, fn(&K, &K) -> SetJointTransformResult<ReduceT>> {
    SetJointTransform::UnchangedL(r)
}

pub fn new_set_joint_transform_unchangedr<K, ReduceT: Clone>(r: ReduceT) -> SetJointTransform<ReduceT, fn(&K, &K) -> SetJointTransformResult<ReduceT>> {
    SetJointTransform::UnchangedR(r)
}

pub fn new_set_joint_transform_removed<K, ReduceT: Clone>(r: ReduceT) -> SetJointTransform<ReduceT, fn(&K, &K) -> SetJointTransformResult<ReduceT>> {
    SetJointTransform::Removed(r)
}

#[derive(Clone)]
pub enum MapTransform<ReduceT, F> {
    Generic(F),
    Unchanged(ReduceT),
    Removed(ReduceT),
}

impl <ReduceT: Clone, F> MapTransform<ReduceT, F> {
    pub fn call<K, V>(&self, k: &K, v: &V) -> MapTransformResult<V, ReduceT> where F: Fn(&K, &V) -> MapTransformResult<V, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, v),
            Self::Unchanged(r) => MapTransformResult::Unchanged(r.clone()),
            Self::Removed(r) => MapTransformResult::Removed(r.clone()),
        }
    }

    pub fn call_transform_transmute<K, V, L, W>(&self, k: &K, v: &V, l: &L, w: &W) -> MapTransformResult<V, ReduceT> where F: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, v, l, w),
            Self::Unchanged(r) => MapTransformResult::Unchanged(r.clone()),
            Self::Removed(r) => MapTransformResult::Removed(r.clone()),
        }
    }

    pub fn call_transmute_transform<K, V, L, W>(&self, k: &K, v: &V, l: &L, w: &W) -> MapTransformResult<W, ReduceT> where F: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, v, l, w),
            Self::Unchanged(r) => MapTransformResult::Unchanged(r.clone()),
            Self::Removed(r) => MapTransformResult::Removed(r.clone()),
        }
    }

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

pub fn new_map_transform_generic<K, V, ReduceT: Clone, F: Fn(&K, &V) -> MapTransformResult<V, ReduceT> + Clone>(f: F) -> MapTransform<ReduceT, F> {
    MapTransform::Generic(f)
}

pub fn new_map_transform_unchanged<K, V, ReduceT: Clone>(r: ReduceT) -> MapTransform<ReduceT, fn(&K, &V) -> MapTransformResult<V, ReduceT>> {
    MapTransform::Unchanged(r)
}

pub fn new_map_transform_removed<K, V, ReduceT: Clone>(r: ReduceT) -> MapTransform<ReduceT, fn(&K, &V) -> MapTransformResult<V, ReduceT>> {
    MapTransform::Removed(r)
}

pub fn new_map_transform_transmute_generic<K, V, L, W, ReduceT: Clone, F: Fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT> + Clone>(f: F) -> MapTransform<ReduceT, F> {
    MapTransform::Generic(f)
}

pub fn new_map_transform_transmute_unchanged<K, V, L, W, ReduceT: Clone>(r: ReduceT) -> MapTransform<ReduceT, fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT>> {
    MapTransform::Unchanged(r)
}

pub fn new_map_transform_transmute_removed<K, V, L, W, ReduceT: Clone>(r: ReduceT) -> MapTransform<ReduceT, fn(&K, &V, &L, &W) -> MapTransformResult<V, ReduceT>> {
    MapTransform::Removed(r)
}

pub fn new_map_transmute_transform_generic<K, V, L, W, ReduceT: Clone, F: Fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT> + Clone>(f: F) -> MapTransform<ReduceT, F> {
    MapTransform::Generic(f)
}

pub fn new_map_transmute_transform_unchanged<K, V, L, W, ReduceT: Clone>(r: ReduceT) -> MapTransform<ReduceT, fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT>> {
    MapTransform::Unchanged(r)
}

pub fn new_map_transmute_transform_removed<K, V, L, W, ReduceT: Clone>(r: ReduceT) -> MapTransform<ReduceT, fn(&K, &V, &L, &W) -> MapTransformResult<W, ReduceT>> {
    MapTransform::Removed(r)
}

#[derive(Clone)]
pub enum SetTransform<ReduceT, F> {
    Generic(F),
    Unchanged(ReduceT),
    Removed(ReduceT),
}

impl <ReduceT: Clone, F> SetTransform<ReduceT, F> {
    pub fn call<K>(&self, k: &K) -> SetTransformResult<ReduceT> where F: Fn(&K) -> SetTransformResult<ReduceT>
    {
        match self {
            Self::Generic(f) => f(k),
            Self::Unchanged(r) => SetTransformResult::Unchanged(r.clone()),
            Self::Removed(r) => SetTransformResult::Removed(r.clone()),
        }
    }

    pub fn call_transmute<K, L>(&self, k: &K, l: &L) -> SetTransformResult<ReduceT> where F: Fn(&K, &L) -> SetTransformResult<ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, l),
            Self::Unchanged(r) => SetTransformResult::Unchanged(r.clone()),
            Self::Removed(r) => SetTransformResult::Removed(r.clone()),
        }
    }

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

pub fn new_set_transform_generic<K, ReduceT: Clone, F: Fn(&K) -> SetTransformResult<ReduceT> + Clone>(f: F) -> SetTransform<ReduceT, F> {
    SetTransform::Generic(f)
}

pub fn new_set_transform_unchanged<K, ReduceT: Clone>(r: ReduceT) -> SetTransform<ReduceT, fn(&K) -> SetTransformResult<ReduceT>> {
    SetTransform::Unchanged(r)
}

pub fn new_set_transform_removed<K, ReduceT: Clone>(r: ReduceT) -> SetTransform<ReduceT, fn(&K) -> SetTransformResult<ReduceT>> {
    SetTransform::Removed(r)
}

pub fn new_set_transform_transmute_generic<K, L, ReduceT: Clone, F: Fn(&K, &L) -> SetTransformResult<ReduceT> + Clone>(f: F) -> SetTransform<ReduceT, F> {
    SetTransform::Generic(f)
}

pub fn new_set_transform_transmute_unchanged<K, L, ReduceT: Clone>(r: ReduceT) -> SetTransform<ReduceT, fn(&K, &L) -> SetTransformResult<ReduceT>> {
    SetTransform::Unchanged(r)
}

pub fn new_set_transform_transmute_removed<K, L, ReduceT: Clone>(r: ReduceT) -> SetTransform<ReduceT, fn(&K, &L) -> SetTransformResult<ReduceT>> {
    SetTransform::Removed(r)
}

#[derive(Clone)]
pub enum MapTransmute<ReduceT, F> {
    Generic(F),
    Removed(ReduceT),
}

impl <ReduceT: Clone, F> MapTransmute<ReduceT, F> {
    pub fn call<K, V, S, X>(&self, k: &K, v: &V) -> MapTransmuteResult<S, X, ReduceT> where F: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, v),
            Self::Removed(r) => MapTransmuteResult::Removed(r.clone()),
        }
    }

    pub fn call_transmute<K, V, L, W, S, X>(&self, k: &K, v: &V, l: &L, w: &W) -> MapTransmuteResult<S, X, ReduceT> where F: Fn(&K, &V, &L, &W) -> MapTransmuteResult<S, X, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, v, l, w),
            Self::Removed(r) => MapTransmuteResult::Removed(r.clone()),
        }
    }

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

pub fn new_map_transmute_generic<K, V, S, X, ReduceT: Clone, F: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT> + Clone>(f: F) -> MapTransmute<ReduceT, F> {
    MapTransmute::Generic(f)
}

pub fn new_map_transmute_removed<K, V, S, X, ReduceT: Clone>(r: ReduceT) -> MapTransmute<ReduceT, fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT>> {
    MapTransmute::Removed(r)
}

pub fn new_map_transmute_transmute_generic<K, V, L, W, S, X, ReduceT: Clone, F: Fn(&K, &V, &L, &W) -> MapTransmuteResult<S, X, ReduceT> + Clone>(f: F) -> MapTransmute<ReduceT, F> {
    MapTransmute::Generic(f)
}

pub fn new_map_transmute_transmute_removed<K, V, L, W, S, X, ReduceT: Clone>(r: ReduceT) -> MapTransmute<ReduceT, fn(&K, &V, &L, &W) -> MapTransmuteResult<S, X, ReduceT>> {
    MapTransmute::Removed(r)
}

#[derive(Clone)]
pub enum SetTransmute<ReduceT, F> {
    Generic(F),
    Removed(ReduceT),
}

impl <ReduceT: Clone, F> SetTransmute<ReduceT, F> {
    pub fn call<K, S>(&self, k: &K) -> SetTransmuteResult<S, ReduceT> where F: Fn(&K) -> SetTransmuteResult<S, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k),
            Self::Removed(r) => SetTransmuteResult::Removed(r.clone()),
        }
    }

    pub fn call_transmute<K, L, S>(&self, k: &K, l: &L) -> SetTransmuteResult<S, ReduceT> where F: Fn(&K, &L) -> SetTransmuteResult<S, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, l),
            Self::Removed(r) => SetTransmuteResult::Removed(r.clone()),
        }
    }

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

pub fn new_set_transmute_generic<K, S, ReduceT: Clone, F: Fn(&K) -> SetTransmuteResult<S, ReduceT> + Clone>(f: F) -> SetTransmute<ReduceT, F> {
    SetTransmute::Generic(f)
}

pub fn new_set_transmute_removed<K, S, ReduceT: Clone>(r: ReduceT) -> SetTransmute<ReduceT, fn(&K) -> SetTransmuteResult<S, ReduceT>> {
    SetTransmute::Removed(r)
}

pub fn new_set_transmute_transmute_generic<K, L, S, ReduceT: Clone, F: Fn(&K, &L) -> SetTransmuteResult<S, ReduceT> + Clone>(f: F) -> SetTransmute<ReduceT, F> {
    SetTransmute::Generic(f)
}

pub fn new_set_transmute_transmute_removed<K, L, S, ReduceT: Clone>(r: ReduceT) -> SetTransmute<ReduceT, fn(&K, &L) -> SetTransmuteResult<S, ReduceT>> {
    SetTransmute::Removed(r)
}
