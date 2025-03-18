// Schubert matroids.

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operation<T> {
    Insert(T),
    DeleteMin,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Bucket<T> {
    pub inserts: Vec<T>,
    pub deletes: usize,
}

pub type Buckets<T> = Vec<Bucket<T>>;

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::Debug;

use itertools::{izip, Itertools};
use proptest::prelude::{any, Strategy};

pub fn sim_naive<T: Ord>(ops: Vec<Operation<T>>) -> Vec<T> {
    let mut h = BinaryHeap::new();
    for op in ops {
        match op {
            Operation::Insert(x) => {
                h.push(Reverse(x));
            }
            Operation::DeleteMin => {
                h.pop();
            }
        }
    }
    h.into_iter().map(|Reverse(x)| x).collect::<Vec<_>>()
}

pub fn into_buckets<T>(ops: Vec<Operation<T>>) -> Buckets<T> {
    ops.into_iter()
        .map(|op| match op {
            Operation::Insert(x) => Bucket {
                inserts: vec![x],
                deletes: 0,
            },
            Operation::DeleteMin => Bucket {
                inserts: vec![],
                deletes: 1,
            },
        })
        .collect::<Vec<Bucket<T>>>()
}

pub fn from_buckets<T>(buckets: Buckets<T>) -> Vec<Operation<T>> {
    buckets
        .into_iter()
        .flat_map(|bucket| {
            let mut ops = Vec::new();
            for x in bucket.inserts {
                ops.push(Operation::Insert(x));
            }
            for _ in 0..bucket.deletes {
                ops.push(Operation::DeleteMin);
            }
            ops
        })
        .collect::<Vec<Operation<T>>>()
}

/// Strictly speaking, this one only works for normalised buckets.
pub fn dualise_buckets<T: Ord>(buckets: Buckets<T>) -> Buckets<Reverse<T>> {
    buckets
        .into_iter()
        .rev()
        .map(|Bucket { inserts, deletes }| Bucket {
            deletes: inserts.len().saturating_sub(deletes),
            inserts: inserts.into_iter().map(Reverse).collect(),
        })
        .collect()
}

pub fn normalise_buckets<T: Debug>(buckets: Buckets<T>) -> Buckets<T> {
    let mut new_buckets = Vec::new();
    let mut open_bucket = Bucket {
        inserts: vec![],
        deletes: 0,
    };
    for mut bucket in buckets.into_iter().rev() {
        // combine buckets:
        bucket.inserts.extend(open_bucket.inserts);
        bucket.deletes += open_bucket.deletes;

        // Check if combined bucket is open:
        if bucket.inserts.len() <= bucket.deletes {
            open_bucket = bucket;
        } else {
            new_buckets.push(bucket);
            open_bucket = Bucket {
                inserts: vec![],
                deletes: 0,
            };
        }
    }
    new_buckets.reverse();
    new_buckets
}

#[allow(clippy::cast_sign_loss)]
pub fn operation() -> impl Strategy<Value = Operation<u32>> {
    any::<Option<u32>>().prop_map(|x| match x {
        None => Operation::DeleteMin,
        Some(x) => Operation::Insert(x),
    })
}

pub fn operations() -> impl Strategy<Value = Vec<Operation<u32>>> {
    proptest::collection::vec(operation(), 0..10_000).prop_map(compress_operations)
}

pub fn compress_operations<T: Ord>(ops: Vec<Operation<T>>) -> Vec<Operation<u32>> {
    izip!(ops, 0..)
        .sorted()
        .zip(0..)
        .map(|((op, i), o)| {
            (
                i,
                match op {
                    Operation::Insert(_) => Operation::Insert(o),
                    Operation::DeleteMin => Operation::DeleteMin,
                },
            )
        })
        .sorted()
        .map(|(_, op)| op)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    proptest! {

    #[test]
    fn test_simulate_via_buckets(ops in operations()) {
        let mut naive = sim_naive(ops.clone());
        let mut via_buckets = sim_naive(from_buckets(into_buckets(ops)));

        naive.sort();
        via_buckets.sort();

        prop_assert_eq!(naive, via_buckets);
    }

    #[test]
    fn test_simulate_via_buckets_normalised(ops in operations()) {
        let mut naive = sim_naive(ops.clone());
        let mut via_buckets = sim_naive(from_buckets(normalise_buckets(into_buckets(ops))));

        naive.sort();
        via_buckets.sort();

        prop_assert_eq!(naive, via_buckets);
    }
    }
}
