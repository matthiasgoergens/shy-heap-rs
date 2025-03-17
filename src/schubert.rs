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

// def merge_blocks(blocks: List[Block]) -> List[Block]:
// # Now we merge our blocks.
// new_blocks = []
// for block in blocks:
//     while True:
//         if len(block.pushes) > len(block.pops) or new_blocks == []:
//             if len(new_blocks) == 0:
//                 block.pops = block.pops[: len(block.pushes)]
//             new_blocks.append(block)
//             break
//         else:
//             last_block = new_blocks.pop()
//             block.pushes = last_block.pushes + block.pushes
//             block.pops = last_block.pops + block.pops
// return new_blocks

pub fn normalise_buckets<T>(buckets: Buckets<T>) -> Buckets<T> {
    let mut new_buckets = Vec::new();
    let mut buckets = buckets.into_iter();
    if let Some(mut last_bucket) = buckets.next() {
        for bucket in buckets {
            if bucket.inserts.len() > bucket.deletes {
                new_buckets.push(last_bucket);
                last_bucket = bucket;
            } else {
                last_bucket.inserts.extend(bucket.inserts);
                last_bucket.deletes += bucket.deletes;
            }
        }

        new_buckets.push(last_bucket);
    }
    new_buckets
}
