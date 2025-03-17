// Schubert matroids.

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operation<T> {
    Insert(T),
    DeleteMin,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bucket<T> {
    pub inserts: Vec<T>,
    pub deletes: isize,
}

pub type Buckets<T> = Vec<Bucket<T>>;

use std::cmp::Reverse;
use std::collections::BinaryHeap;

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

pub fn bucket<T>(ops: Vec<Operation<T>>) -> Buckets<T> {
    // let mut buckets = vec![];
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
