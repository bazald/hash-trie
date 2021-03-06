use fnv::FnvHasher;
use hash_trie::{*, results::*, transformations::*};
use im::HashSet as ImHashSet;
use std::{collections::{hash_set::HashSet}, sync::{Arc, Mutex}, time::SystemTime, vec::Vec};
use rand::Rng;

#[tokio::main]
async fn main() {
    let (lefts, rights) = get_values();
    let mut xored = HashSet::new();

    println!("HashSet: {} µs", hash_set(&lefts, &rights, &mut xored));
    println!("ImHashSet: {} µs", im_hash_set(&lefts, &rights, &xored));
    println!("HashTrieSet transform with transformed: {} µs", hash_trie_set_transform_with_transformed(&lefts, &rights, &xored).await);
    println!("HashTrieSet transform with transmuted: {} µs", hash_trie_set_transform_with_transmuted(&lefts, &rights, &xored).await);
    println!("HashTrieSet transmute with transformed: {} µs", hash_trie_set_transmute_with_transformed(&lefts, &rights, &xored).await);
    println!("HashTrieSet transmute with transmuted: {} µs", hash_trie_set_transmute_with_transmuted(&lefts, &rights, &xored));
}

fn hash_set(lefts: &[i32], rights: &[i32], xored: &mut HashSet<i32>) -> u128 {
    let mut hash_set = HashSet::new();

    let t0 = SystemTime::now();

    for x in lefts {
        if !rights.contains(x) {
            hash_set.insert(*x);
        }
    }
    for x in rights {
        if !lefts.contains(x) {
            hash_set.insert(*x);
        }
    }

    let t1 = SystemTime::now();

    *xored = hash_set.clone();

    t1.duration_since(t0).unwrap().as_micros()
}

fn im_hash_set(lefts: &[i32], rights: &[i32], xored: &HashSet<i32>) -> u128 {
    let mut left = ImHashSet::new();
    for v in lefts {
        left = left.update(*v);
    }

    let mut right = ImHashSet::new();
    for v in rights {
        right = right.update(*v);
    }

    let t0 = SystemTime::now();

    let hash_set = left.difference(right);

    let t1 = SystemTime::now();

    let mut cmp = HashSet::new();
    for &k in &hash_set {
        cmp.insert(k);
    }
    assert_eq!(cmp, *xored);

    t1.duration_since(t0).unwrap().as_micros()
}

async fn hash_trie_set_transform_with_transformed(lefts: &[i32], rights: &[i32], xored: &HashSet<i32>) -> u128 {
    let mut left = HashTrieSet::<u64, u32, i32, FnvHasher>::new();
    for v in lefts {
        if let Ok(ht) = left.insert(*v, false) {
            left = ht.0;
        }
    }
    let mut right = HashTrieSet::<u64, u32, i32, FnvHasher>::new();
    for v in rights {
        if let Ok(ht) = right.insert(*v, false) {
            right = ht.0;
        }
    }

    let t0 = SystemTime::now();

    let hash_set = left.transform_with_transformed(
        &right, 
        |_,_| (), 
        new_set_joint_transform_removed(()),
        new_set_transform_unchanged(()),
        new_set_transform_unchanged(()),
    ParallelismStrategy::default_par()).await.0;

    let t1 = SystemTime::now();
    
    let cmp = Arc::new(Mutex::new(HashSet::new()));
    hash_set.visit(|&k| {cmp.lock().unwrap().insert(k);});
    assert_eq!(*cmp.lock().unwrap(), *xored);

    t1.duration_since(t0).unwrap().as_micros()
}

async fn hash_trie_set_transform_with_transmuted(lefts: &[i32], rights: &[i32], xored: &HashSet<i32>) -> u128 {
    let mut left = HashTrieSet::<u64, u32, i32, FnvHasher>::new();
    for v in lefts {
        if let Ok(ht) = left.insert(*v, false) {
            left = ht.0;
        }
    }
    let mut right = HashTrieSet::<u64, u32, i32, FnvHasher>::new();
    for v in rights {
        if let Ok(ht) = right.insert(*v, false) {
            right = ht.0;
        }
    }

    let t0 = SystemTime::now();

    let hash_set = unsafe { left.transform_with_transmuted(
        &right, 
        |_,_| (), 
        new_set_transform_transmute_removed(()),
        new_set_transform_unchanged(()),
        new_set_transmute_generic(|r| SetTransmuteResult::Transmuted(*r, ())),
    ParallelismStrategy::default_par()).await.0 };

    let t1 = SystemTime::now();
    
    let cmp = Arc::new(Mutex::new(HashSet::new()));
    hash_set.visit(|&k| {cmp.lock().unwrap().insert(k);});
    assert_eq!(*cmp.lock().unwrap(), *xored);

    t1.duration_since(t0).unwrap().as_micros()
}

async fn hash_trie_set_transmute_with_transformed(lefts: &[i32], rights: &[i32], xored: &HashSet<i32>) -> u128 {
    let mut left = HashTrieSet::<u64, u32, i32, FnvHasher>::new();
    for v in lefts {
        if let Ok(ht) = left.insert(*v, false) {
            left = ht.0;
        }
    }
    let mut right = HashTrieSet::<u64, u32, i32, FnvHasher>::new();
    for v in rights {
        if let Ok(ht) = right.insert(*v, false) {
            right = ht.0;
        }
    }

    let t0 = SystemTime::now();

    let hash_set = unsafe { left.transmute_with_transformed(
        &right, 
        |_,_| (), 
        new_set_transform_transmute_removed(()),
        new_set_transmute_generic(|l| SetTransmuteResult::Transmuted(*l, ())),
        new_set_transform_unchanged(()),
        ParallelismStrategy::default_par()).await.0 };

    let t1 = SystemTime::now();
    
    let cmp = Arc::new(Mutex::new(HashSet::new()));
    hash_set.visit(|&k| {cmp.lock().unwrap().insert(k);});
    assert_eq!(*cmp.lock().unwrap(), *xored);

    t1.duration_since(t0).unwrap().as_micros()
}

fn hash_trie_set_transmute_with_transmuted(lefts: &[i32], rights: &[i32], xored: &HashSet<i32>) -> u128 {
    let mut left = HashTrieSet::<u64, u32, i32, FnvHasher>::new();
    for v in lefts {
        if let Ok(ht) = left.insert(*v, false) {
            left = ht.0;
        }
    }
    let mut right = HashTrieSet::<u64, u32, i32, FnvHasher>::new();
    for v in rights {
        if let Ok(ht) = right.insert(*v, false) {
            right = ht.0;
        }
    }

    let t0 = SystemTime::now();

    let hash_set = unsafe {left.transmute_with_transmuted(
        &right, 
        |_,_| (), 
        new_set_transmute_transmute_removed(()),
        new_set_transmute_generic(|l| SetTransmuteResult::Transmuted(*l, ())),
        new_set_transmute_generic(|r| SetTransmuteResult::Transmuted(*r, ()))).0 };

    let t1 = SystemTime::now();
    
    let cmp = Arc::new(Mutex::new(HashSet::new()));
    hash_set.visit(|&k| {cmp.lock().unwrap().insert(k);});
    assert_eq!(*cmp.lock().unwrap(), *xored);

    t1.duration_since(t0).unwrap().as_micros()
}

fn get_values() -> (Vec<i32>, Vec<i32>) {
    let mut rng = rand::thread_rng();
    let lefts = (0..25000).map(|_| rng.gen_range(0..100000)).collect::<Vec<i32>>();
    let rights = (0..25000).map(|_| rng.gen_range(0..100000)).collect::<Vec<i32>>();
    (lefts, rights)
}
