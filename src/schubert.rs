// Schubert matroids.
use crate::pairing::Heap;

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

use std::cmp::{min, Reverse};
use std::collections::{BTreeSet, BinaryHeap};
use std::fmt::Debug;
use std::iter::repeat;

use itertools::{chain, enumerate, izip, Itertools};
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
pub fn dualise_buckets<T>(buckets: Buckets<T>) -> Buckets<Reverse<T>> {
    buckets
        .into_iter()
        .rev()
        .map(|Bucket { inserts, deletes }| Bucket {
            deletes: inserts.len().saturating_sub(deletes),
            inserts: inserts.into_iter().map(Reverse).collect(),
        })
        .collect()
}

pub fn normalise_buckets<T>(buckets: Buckets<T>) -> Buckets<T> {
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
    // This one is just so that dualising doesn't lose items.
    if !open_bucket.inserts.is_empty() {
        new_buckets.push(open_bucket);
    }
    new_buckets.reverse();
    new_buckets
}

#[allow(clippy::cast_sign_loss)]
pub fn operation() -> impl Strategy<Value = Operation<u32>> {
    any::<Option<u32>>().prop_map(|x| match x {
        Some(x) => Operation::Insert(x),
        None => Operation::DeleteMin,
    })
}

pub fn operations() -> impl Strategy<Value = Vec<Operation<u32>>> {
    proptest::collection::vec(operation(), 0..20_000).prop_map(compress_operations)
}

pub struct Ops(pub Vec<Operation<u32>>);
impl Debug for Ops {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for op in &self.0 {
            match op {
                Operation::Insert(x) => write!(f, "{} ", x)?,
                Operation::DeleteMin => write!(f, "_ ")?,
            }
        }
        Ok(())
    }
}

pub fn full_ops(n: u32) -> impl Strategy<Value = Ops> {
    let l = (0..n, 0..n)
        .prop_map(|(n, k)| {
            let k = min(n, k) as usize;
            chain!(
                repeat(Operation::DeleteMin).take(k),
                (0..n).map(Operation::Insert)
            )
            .collect::<Vec<Operation<u32>>>()
        })
        .prop_shuffle();
    (l, 0..10 * n)
        .prop_map(|(mut ops, n)| {
            ops.truncate(n as usize);
            ops
        })
        .prop_map(Ops)
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

pub fn simulate_dualised<T: Ord + std::fmt::Debug + Clone>(ops: Vec<Operation<T>>) -> Vec<T> {
    // only works for all ops being different, ie uniquelified.
    // We can fix that later.

    let original_ops = ops.clone();

    let buckets = into_buckets(ops);
    let buckets = normalise_buckets(buckets);
    let buckets = dualise_buckets(buckets);
    let ops = from_buckets(buckets);
    let result = sim_naive(ops);
    let result = result
        .into_iter()
        .map(|Reverse(x)| x)
        .collect::<BTreeSet<_>>();

    // You can do this one via indices and direct lookups, so you don't have to compare keys.
    // That's important for getting our O(n) comparisons.
    original_ops
        .into_iter()
        .filter_map(|op| match op {
            Operation::Insert(x) if !result.contains(&x) => Some(x),
            _ => None,
        })
        .collect()
}

pub fn simulate_pairing_debug<T: Ord + std::fmt::Debug + Clone>(ops: Vec<Operation<T>>) -> Vec<T> {
    // CHUNKS>=8 and EPS = 6 seem to work.
    // Chunks>=6 and EPS=3 also seem to work.
    let mut pairing: Heap<6, T> = Heap::default();
    let mut inserts_so_far = 0;
    for op in ops {
        pairing = match op {
            Operation::Insert(x) => {
                inserts_so_far += 1;
                pairing.insert(x)
            }
            Operation::DeleteMin => pairing.delete_min(),
        };
        let un = pairing.count_uncorrupted();
        let co = pairing.count_corrupted();
        // With a bit of care, we should be able to guarantee a relationship between uncorrupted * epsilon >= corrupted,
        // in our setting, because we do not allow removal of arbitrary elements.  We only allow removal of the smallest,
        // and corruption can not travel downwards, in some sense, and only delete_min introduced new corruption.
        // TODO: btw, we thought of tracking _information_ and proving something about that as an invariant.
        // 'information' measures given the structure of the heap and the heap property, how many different permutations
        // of the items are compatible with what we know.
        // A very flat heap has lots of possible permutations.
        // A very deep heap has very few possible permutations.  In the extreme of a linked list structure, only one possibility.

        // How does corruption play into this measure of information?
        const EPS: usize = 3;
        assert!(
            inserts_so_far >= EPS * co,
            "{inserts_so_far} >= {EPS} * {co}; uncorrupted: {un}\n{pairing:?}"
        );
    }
    Vec::from(pairing)
}

pub fn count_deletes<T>(ops: &[Operation<T>]) -> usize {
    ops.iter()
        .filter(|op| matches!(op, Operation::DeleteMin))
        .count()
}

pub fn count_inserts<T>(ops: &[Operation<T>]) -> usize {
    ops.iter()
        .filter(|op| matches!(op, Operation::Insert(_)))
        .count()
}

pub fn dualise_ops<T>(ops: Vec<Operation<T>>) -> Vec<Operation<Reverse<T>>> {
    from_buckets(dualise_buckets(normalise_buckets(into_buckets(ops))))
}

pub fn undualise_ops<T>(ops: Vec<Operation<Reverse<T>>>) -> Vec<Operation<T>> {
    dualise_ops(ops)
        .into_iter()
        .map(|op| match op {
            Operation::Insert(Reverse(Reverse(x))) => Operation::Insert(x),
            Operation::DeleteMin => Operation::DeleteMin,
        })
        .collect::<Vec<_>>()
}

// result: definitely-in, definitely-out.
pub fn linear<T: Ord + std::fmt::Debug + Clone>(ops: Vec<Operation<T>>) -> Vec<T> {
    const CHUNKS: usize = 8;
    let inserts = count_inserts(&ops);
    let deletes = count_deletes(&ops);
    if ops.is_empty() {
        vec![]
    } else if deletes * 2 <= inserts {
        // primal
        let (left_ops, guaranteed_in) = simulate_pairing::<CHUNKS, _>(ops);
        chain!(guaranteed_in, linear(left_ops)).collect()
    } else {
        // here we need to dualise.
        let dual_ops = dualise_ops(ops);

        let (left_over_ops, _guaranteed_out) = simulate_pairing::<CHUNKS, _>(dual_ops);
        linear(undualise_ops(left_over_ops))
    }
}

pub fn linear_loop<T: Ord + std::fmt::Debug + Clone>(mut ops: Vec<Operation<T>>) -> Vec<T> {
    const CHUNKS: usize = 8;
    let mut result = vec![];

    while !ops.is_empty() {
        let inserts = count_inserts(&ops);
        let deletes = count_deletes(&ops);

        if deletes * 2 <= inserts {
            // primal
            let (left_ops, guaranteed_in) = simulate_pairing::<CHUNKS, _>(ops);
            ops = left_ops;
            result.extend(guaranteed_in);
        } else {
            // here we need to dualise.
            let dual_ops = dualise_ops(ops);

            let (left_over_ops, _guaranteed_out) = simulate_pairing::<CHUNKS, _>(dual_ops);
            ops = undualise_ops(left_over_ops);
        }
        assert!(
            ops.len() <= (inserts + deletes) * 5 / 6,
        );
    }
    result
}

// inserts >= 3 * corrupted
// uncorrupted = inserts - deleted - corrupted
// deleted <= inserts / 2
// corrupted <= inserts / 3
// uncorrupted >= inserts - (inserts / 2) - (inserts / 3)
// uncorrupted >= inserts / 6

// Well, the above holds for primal.  For dual we have remove deletes instead.
// inserts <= deletes / 2
// deletes' = inserts - deletes
// Now we have 1/6 uncorrupted to be kept.  And that results in losing at least 1/6 of deletes.

/// The bool in the result mean 'definitely in the heap at the end'
pub fn simulate_pairing<const CHUNKS: usize, T: Ord + std::fmt::Debug + Clone>(
    ops: Vec<Operation<T>>,
) -> (Vec<Operation<T>>, Vec<T>) {
    let ops_extended = enumerate(&ops)
        .map(|(i, op)| match op {
            Operation::Insert(x) => Operation::Insert((x, i)),
            Operation::DeleteMin => Operation::DeleteMin,
        })
        .collect::<Vec<_>>();

    let mut pairing: Heap<CHUNKS, (&T, usize)> = Heap::default();
    for op in ops_extended {
        pairing = match op {
            Operation::Insert(x) => pairing.insert(x),
            Operation::DeleteMin => pairing.delete_min(),
        };
    }
    let left_over: Vec<usize> = Vec::from(pairing)
        .into_iter()
        .map(|(_x, i)| i)
        .collect::<Vec<_>>();

    // TODO: we could prettify this one a bit.
    let mut ops_result = ops.into_iter().map(Some).collect::<Vec<_>>();
    let mut result = vec![];
    for i in left_over {
        let op = ops_result[i].take();
        if let Some(Operation::Insert(x)) = op {
            result.push(x);
        } else {
            unreachable!();
        }
    }
    (ops_result.into_iter().flatten().collect::<Vec<_>>(), result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn corruption_simple(ops in operations()) {
            simulate_pairing_debug(ops);
        }

        #[test]
        fn corruption(ops in full_ops(10_000)) {
            simulate_pairing_debug(ops.0);
        }


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


        #[test]
        fn test_simulate_dualised(ops in operations()) {
            let mut naive = sim_naive(ops.clone());
            let mut dualised = simulate_dualised(ops);

            naive.sort();
            dualised.sort();

            prop_assert_eq!(naive, dualised);
        }
        #[test]
        fn test_via_pairing_heap(ops in full_ops(10_000)) {
            let mut naive = sim_naive(ops.0.clone());
            let mut pairing_in = linear(ops.0.clone());

            naive.sort();
            pairing_in.sort();

            prop_assert_eq!(&naive, &pairing_in);
        }
        #[test]
        fn test_via_pairing_heap_loop(ops in full_ops(10_000)) {
            let mut naive = sim_naive(ops.0.clone());
            let mut pairing_in_2 = linear_loop(ops.0);

            naive.sort();
            pairing_in_2.sort();

            prop_assert_eq!(&naive, &pairing_in_2);
        }
    }
}
