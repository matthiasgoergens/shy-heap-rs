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

use proptest::prelude::{any, Just, Strategy};
use proptest::prop_oneof;

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

#[allow(clippy::cast_sign_loss)]
pub fn operation() -> impl Strategy<Value = Operation<u32>> {
    any::<Option<u32>>().prop_map(|x| match x {
        None => Operation::DeleteMin,
        Some(x) => Operation::Insert(x),
    })
}

// TODO: later add a way to 'uniquelify' the inserts, and to 'compress' them down to only use small integers, like we have in Python.
pub fn operations() -> impl Strategy<Value = Vec<Operation<u32>>> {
    proptest::collection::vec(operation(), 0..1000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    proptest! {
    // TODO: use proptest to create an arbitrary sequence of operations.
    // Then run them via sim_naive, via from_buckets . into_buckets and via from_buckets . normalise_buckets . into_buckets.
    // Check that the results are the same, up to reordering.


        #[test]
        fn test_simulate_three_ways(ops in operations()) {
            let mut naive = sim_naive(ops.clone());
            let buckets = into_buckets(ops.clone());
            let normalised = normalise_buckets(buckets.clone());
            let normalised_naive = sim_naive(from_buckets(normalised.clone()));
            let buckets_normalised = normalise_buckets(buckets.clone());
            let normalised_buckets_naive = sim_naive(from_buckets(buckets_normalised.clone()));
            prop_assert_eq!(naive.clone(), normalised_naive);
            prop_assert_eq!(naive, normalised_buckets_naive);
        }
    }
}

// This is all just for testing proptest, to remind me how it worked.
fn parse_date(s: &str) -> Option<(u32, u32, u32)> {
    if 10 != s.len() {
        return None;
    }

    // NEW: Ignore non-ASCII strings so we don't need to deal with Unicode.
    if !s.is_ascii() {
        return None;
    }

    if "-" != &s[4..5] || "-" != &s[7..8] {
        return None;
    }

    let year = &s[0..4];
    let month = &s[5..7];
    let day = &s[8..10];

    year.parse::<u32>().ok().and_then(|y| {
        month
            .parse::<u32>()
            .ok()
            .and_then(|m| day.parse::<u32>().ok().map(|d| (y, m, d)))
    })
}

#[cfg(test)]
mod test_tests {
    use super::*;

    #[test]
    fn exploration() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
    use proptest::prelude::*;
    proptest! {
            #[test]
            fn doesnt_crash(s in "\\PC*") {
                parse_date(&s);
            }
            #[test]
            fn parses_all_valid_dates(s in "[0-9]{4}-[0-9]{2}-[0-9]{2}") {
                parse_date(&s).unwrap();
            }
        #[test]
        fn parses_date_back_to_original(y in 0u32..10000,
                                        m in 1u32..13, d in 1u32..32) {
            let (y2, m2, d2) = parse_date(
                &format!("{:04}-{:02}-{:02}", y, m, d)).unwrap();
            // prop_assert_eq! is basically the same as assert_eq!, but doesn't
            // cause a bunch of panic messages to be printed on intermediate
            // test failures. Which one to use is largely a matter of taste.
            prop_assert_eq!((y, m, d), (y2, m2, d2));
        }
    }
}
