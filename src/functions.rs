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
    pub fn new_generic<K, V>(f: F, flipped: bool) -> MapJointTransform<ReduceT, F> where F: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>
    {
        MapJointTransform::Generic(f, flipped)
    }

    pub fn new_unchangedlr<K, V>(r: ReduceT, _f: F) -> MapJointTransform<ReduceT, F> where F: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>
    {
        MapJointTransform::<ReduceT, F>::UnchangedLR(r)
    }

    pub fn new_unchangedl<K, V>(r: ReduceT, _f: F) -> MapJointTransform<ReduceT, F> where F: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>
    {
        MapJointTransform::<ReduceT, F>::UnchangedL(r)
    }

    pub fn new_unchangedr<K, V>(r: ReduceT, _f: F) -> MapJointTransform<ReduceT, F> where F: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>
    {
        MapJointTransform::<ReduceT, F>::UnchangedR(r)
    }

    pub fn new_removed<K, V>(r: ReduceT, _f: F) -> MapJointTransform<ReduceT, F> where F: Fn(&K, &V, &K, &V) -> MapJointTransformResult<V, ReduceT>
    {
        MapJointTransform::<ReduceT, F>::Removed(r)
    }

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

#[macro_export]
macro_rules! new_generic_map_joint_transform {
    ( $f:expr ) => {
        MapJointTransform::new_generic($f, false)
    };
}

#[macro_export]
macro_rules! new_unchangedlr_map_joint_transform {
    ( $r:expr ) => {
        MapJointTransform::new_unchanged($r, |_,_,_,_| MapJointTransformResult::UnchangedLR($r))
    };
}

#[macro_export]
macro_rules! new_unchangedl_map_joint_transform {
    ( $r:expr ) => {
        MapJointTransform::new_unchanged($r, |_,_,_,_| MapJointTransformResult::UnchangedL($r))
    };
}

#[macro_export]
macro_rules! new_unchangedr_map_joint_transform {
    ( $r:expr ) => {
        MapJointTransform::new_unchanged($r, |_,_,_,_| MapJointTransformResult::UnchangedR($r))
    };
}

#[macro_export]
macro_rules! new_removed_map_joint_transform {
    ( $r:expr ) => {
        MapJointTransform::new_removed($r, |_,_,_,_| MapJointTransformResult::Removed($r))
    };
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
    pub fn new_generic<K>(f: F, flipped: bool) -> SetJointTransform<ReduceT, F> where F: Fn(&K, &K) -> SetJointTransformResult<ReduceT>
    {
        SetJointTransform::Generic(f, flipped)
    }

    pub fn new_unchangedlr<K>(r: ReduceT, _f: F) -> SetJointTransform<ReduceT, F> where F: Fn(&K, &K) -> SetJointTransformResult<ReduceT>
    {
        SetJointTransform::<ReduceT, F>::UnchangedLR(r)
    }

    pub fn new_unchangedl<K>(r: ReduceT, _f: F) -> SetJointTransform<ReduceT, F> where F: Fn(&K, &K) -> SetJointTransformResult<ReduceT>
    {
        SetJointTransform::<ReduceT, F>::UnchangedL(r)
    }

    pub fn new_unchangedr<K>(r: ReduceT, _f: F) -> SetJointTransform<ReduceT, F> where F: Fn(&K, &K) -> SetJointTransformResult<ReduceT>
    {
        SetJointTransform::<ReduceT, F>::UnchangedR(r)
    }

    pub fn new_removed<K>(r: ReduceT, _f: F) -> SetJointTransform<ReduceT, F> where F: Fn(&K, &K) -> SetJointTransformResult<ReduceT>
    {
        SetJointTransform::<ReduceT, F>::Removed(r)
    }

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

#[macro_export]
macro_rules! new_generic_set_joint_transform {
    ( $f:expr ) => {
        SetJointTransform::new_generic($f, false)
    };
}

#[macro_export]
macro_rules! new_unchangedlr_set_joint_transform {
    ( $r:expr ) => {
        SetJointTransform::new_unchanged($r, |_,_| SetJointTransformResult::UnchangedLR($r))
    };
}

#[macro_export]
macro_rules! new_unchangedl_set_joint_transform {
    ( $r:expr ) => {
        SetJointTransform::new_unchanged($r, |_,_| SetJointTransformResult::UnchangedL($r))
    };
}

#[macro_export]
macro_rules! new_unchangedr_set_joint_transform {
    ( $r:expr ) => {
        SetJointTransform::new_unchanged($r, |_,_| SetJointTransformResult::UnchangedR($r))
    };
}

#[macro_export]
macro_rules! new_removed_set_joint_transform {
    ( $r:expr ) => {
        SetJointTransform::new_removed($r, |_,_| SetJointTransformResult::Removed($r))
    };
}

#[derive(Clone)]
pub enum MapTransform<ReduceT, F> {
    Generic(F),
    Unchanged(ReduceT),
    Removed(ReduceT),
}

impl <ReduceT: Clone, F> MapTransform<ReduceT, F> {
    pub fn new_generic<K, V>(f: F) -> MapTransform<ReduceT, F> where F: Fn(&K, &V) -> MapTransformResult<V, ReduceT>
    {
        MapTransform::Generic(f)
    }

    pub fn new_unchanged<K, V>(r: ReduceT, _f: F) -> MapTransform<ReduceT, F> where F: Fn(&K, &V) -> MapTransformResult<V, ReduceT>
    {
        MapTransform::<ReduceT, F>::Unchanged(r)
    }

    pub fn new_removed<K, V>(r: ReduceT, _f: F) -> MapTransform<ReduceT, F> where F: Fn(&K, &V) -> MapTransformResult<V, ReduceT>
    {
        MapTransform::<ReduceT, F>::Removed(r)
    }

    pub fn call<K, V>(&self, k: &K, v: &V) -> MapTransformResult<V, ReduceT> where F: Fn(&K, &V) -> MapTransformResult<V, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, v),
            Self::Unchanged(r) => MapTransformResult::Unchanged(r.clone()),
            Self::Removed(r) => MapTransformResult::Removed(r.clone()),
        }
    }
}

#[macro_export]
macro_rules! new_generic_map_transform {
    ( $f:expr ) => {
        MapTransform::new_generic($f)
    };
}

#[macro_export]
macro_rules! new_unchanged_map_transform {
    ( $r:expr ) => {
        MapTransform::new_unchanged($r, |_,_| MapTransformResult::Unchanged($r))
    };
}

#[macro_export]
macro_rules! new_removed_map_transform {
    ( $r:expr ) => {
        MapTransform::new_removed($r, |_,_| MapTransformResult::Removed($r))
    };
}

#[derive(Clone)]
pub enum SetTransform<ReduceT, F> {
    Generic(F),
    Unchanged(ReduceT),
    Removed(ReduceT),
}

impl <ReduceT: Clone, F> SetTransform<ReduceT, F> {
    pub fn new_generic<K>(f: F) -> SetTransform<ReduceT, F> where F: Fn(&K) -> SetTransformResult<ReduceT>
    {
        SetTransform::Generic(f)
    }

    pub fn new_unchanged<K>(r: ReduceT, _f: F) -> SetTransform<ReduceT, F> where F: Fn(&K) -> SetTransformResult<ReduceT>
    {
        SetTransform::<ReduceT, F>::Unchanged(r)
    }

    pub fn new_removed<K>(r: ReduceT, _f: F) -> SetTransform<ReduceT, F> where F: Fn(&K) -> SetTransformResult<ReduceT>
    {
        SetTransform::<ReduceT, F>::Removed(r)
    }

    pub fn call<K>(&self, k: &K) -> SetTransformResult<ReduceT> where F: Fn(&K) -> SetTransformResult<ReduceT>
    {
        match self {
            Self::Generic(f) => f(k),
            Self::Unchanged(r) => SetTransformResult::Unchanged(r.clone()),
            Self::Removed(r) => SetTransformResult::Removed(r.clone()),
        }
    }
}

#[macro_export]
macro_rules! new_generic_set_transform {
    ( $f:expr ) => {
        SetTransform::new_generic($f)
    };
}

#[macro_export]
macro_rules! new_unchanged_set_transform {
    ( $r:expr ) => {
        SetTransform::new_unchanged($r, |_| SetTransformResult::Unchanged($r))
    };
}

#[macro_export]
macro_rules! new_removed_set_transform {
    ( $r:expr ) => {
        SetTransform::new_removed($r, |_| SetTransformResult::Removed($r))
    };
}

#[derive(Clone)]
pub enum MapTransmute<ReduceT, F> {
    Generic(F),
    Removed(ReduceT),
}

impl <ReduceT: Clone, F> MapTransmute<ReduceT, F> {
    pub fn new_generic<K, V, S, X>(f: F) -> MapTransmute<ReduceT, F> where F: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT>
    {
        MapTransmute::Generic(f)
    }

    pub fn new_removed<K, V, S, X>(r: ReduceT, _f: F) -> MapTransmute<ReduceT, F> where F: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT>
    {
        MapTransmute::<ReduceT, F>::Removed(r)
    }

    pub fn call<K, V, S, X>(&self, k: &K, v: &V) -> MapTransmuteResult<S, X, ReduceT> where F: Fn(&K, &V) -> MapTransmuteResult<S, X, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k, v),
            Self::Removed(r) => MapTransmuteResult::Removed(r.clone()),
        }
    }
}

#[macro_export]
macro_rules! new_generic_map_transmute {
    ( $f:expr ) => {
        MapTransmute::new_generic($f)
    };
}

#[macro_export]
macro_rules! new_removed_map_transmute {
    ( $r:expr ) => {
        MapTransmute::new_removed($r, |_,_| MapTransmuteResult::Removed($r))
    };
}

#[derive(Clone)]
pub enum SetTransmute<ReduceT, F> {
    Generic(F),
    Removed(ReduceT),
}

impl <ReduceT: Clone, F> SetTransmute<ReduceT, F> {
    pub fn new_generic<K, S>(f: F) -> SetTransmute<ReduceT, F> where F: Fn(&K) -> SetTransmuteResult<S, ReduceT>
    {
        SetTransmute::Generic(f)
    }

    pub fn new_removed<K, S>(r: ReduceT, _f: F) -> SetTransmute<ReduceT, F> where F: Fn(&K) -> SetTransmuteResult<S, ReduceT>
    {
        SetTransmute::<ReduceT, F>::Removed(r)
    }

    pub fn call<K, S>(&self, k: &K) -> SetTransmuteResult<S, ReduceT> where F: Fn(&K) -> SetTransmuteResult<S, ReduceT>
    {
        match self {
            Self::Generic(f) => f(k),
            Self::Removed(r) => SetTransmuteResult::Removed(r.clone()),
        }
    }
}

#[macro_export]
macro_rules! new_generic_set_transmute {
    ( $f:expr ) => {
        SetTransmute::new_generic($f)
    };
}

#[macro_export]
macro_rules! new_removed_set_transmute {
    ( $r:expr ) => {
        SetTransmute::new_removed($r, |_| SetTransmuteResult::Removed($r))
    };
}
