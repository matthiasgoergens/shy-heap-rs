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

use crate::pairing::{Heap, EPS};

pub fn simulate_pairing<T: Ord + std::fmt::Debug + Clone>(ops: Vec<Operation<T>>) -> Vec<T> {
    // This one will only return a subset of the items in the real result.
    // TODO: dualise, if subset is too small.
    // And rerun in a loop, without the subset.
    let mut pairing = Heap::default();
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
        assert!(
            inserts_so_far >= EPS * co,
            "{inserts_so_far} >= 3 * {co}; uncorrupted: {un}\n{pairing:?}"
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

// TODO: deal with dualising, and the necessary type wrangling.

// // result: definitely-in, definitely-out.
// pub fn linear<T: Ord + std::fmt::Debug + Clone>(ops: Vec<Operation<T>>) -> (Vec<T>, Vec<T>) {
//     // Bah, we need to deal with Reverse better.
//     // We can't just keep stacking it, alas, the compiler doesn't like that.
//     eprintln!("linear: {:?}", ops.len());
//     let mut in_heap = vec![];
//     let deletes = count_deletes(&ops);
//     if deletes == ops.len() {
//         (vec![], vec![])
//     } else if deletes * 3 < ops.len() {
//         let (left_ops, guaranteed_in) = simulate_pairing_2(ops);
//         in_heap.extend(guaranteed_in);

//         let (def_in, def_out) = linear(left_ops);

//         in_heap.extend(def_in);
//         (in_heap, def_out)
//     } else {
//         // here we need to dualise.
//         let ops = from_buckets(dualise_buckets(normalise_buckets(into_buckets(ops))));
//         let (def_in, def_out) = linear_d(ops);
//         (def_out, def_in)
//     }
// }

// result: definitely-in, definitely-out.
pub fn linear<T: Ord + std::fmt::Debug + Clone>(ops: Vec<Operation<T>>) -> Vec<T> {
    // Bah, we need to deal with Reverse better.
    // We can't just keep stacking it, alas, the compiler doesn't like that.
    eprintln!("linear: {:?}", ops.len());
    let inserts = count_inserts(&ops);
    let deletes = count_deletes(&ops);
    if ops.is_empty() {
        vec![]
    } else if deletes * 2 <= inserts {
        // primal
        let mut in_heap = vec![];
        let (left_ops, guaranteed_in) = simulate_pairing_2(ops);
        in_heap.extend(guaranteed_in);

        in_heap.extend(linear(left_ops));
        in_heap
    } else {
        // here we need to dualise.
        let ops = from_buckets(dualise_buckets(normalise_buckets(into_buckets(ops))));

        let (left_ops_d, _guaranteed_in_d) = simulate_pairing_2(ops);
        let left_ops: Vec<Operation<Reverse<Reverse<T>>>> =
            from_buckets(dualise_buckets(normalise_buckets(into_buckets(left_ops_d))));
        let left_ops: Vec<Operation<T>> = left_ops
            .into_iter()
            .map(|op| match op {
                Operation::Insert(Reverse(Reverse(x))) => Operation::Insert(x),
                Operation::DeleteMin => Operation::DeleteMin,
            })
            .collect::<Vec<_>>();
        linear(left_ops)
    }
}

// // result: definitely-in, definitely-out.
// pub fn linear_d<T: Ord + std::fmt::Debug + Clone>(
//     ops: Vec<Operation<Reverse<T>>>,
// ) -> (Vec<T>, Vec<T>) {
//     eprintln!("linear_d: {:?}", ops.len());
//     let mut in_heap = vec![];
//     let deletes = count_deletes(&ops);
//     if deletes == ops.len() {
//         (vec![], vec![])
//     } else if deletes * 3 < ops.len() {
//         let (left_ops, guaranteed_in) = simulate_pairing_2(ops);
//         in_heap.extend(guaranteed_in);

//         let (def_in, def_out) = linear_d(left_ops);

//         let mut in_heap = in_heap.into_iter().map(|Reverse(x)| x).collect::<Vec<_>>();
//         in_heap.extend(def_in);

//         (in_heap, def_out)
//     } else {
//         // here we need to dualise.
//         let ops = from_buckets(dualise_buckets(normalise_buckets(into_buckets(ops))));
//         let ops = ops
//             .into_iter()
//             .map(|op| match op {
//                 Operation::Insert(Reverse(Reverse(x))) => Operation::Insert(x),
//                 Operation::DeleteMin => Operation::DeleteMin,
//             })
//             .collect::<Vec<_>>();
//         let (def_in, def_out) = linear(ops);
//         (def_out, def_in)
//     }
// }

// This one will only return a subset of the items in the real result.
// TODO: dualise, if subset is too small.
// And rerun in a loop, without the subset.

/// The bool in the result mean 'definitely in the heap at the end'
pub fn simulate_pairing_2<T: Ord + std::fmt::Debug + Clone>(
    ops: Vec<Operation<T>>,
) -> (Vec<Operation<T>>, Vec<T>) {
    let mut ops_result = ops.clone().into_iter().map(Some).collect::<Vec<_>>();

    let ops = enumerate(ops)
        .map(|(i, op)| match op {
            Operation::Insert(x) => Operation::Insert((x, i)),
            Operation::DeleteMin => Operation::DeleteMin,
        })
        .collect::<Vec<_>>();

    let mut pairing = Heap::default();
    for op in ops {
        pairing = match op {
            Operation::Insert(x) => pairing.insert(x),
            Operation::DeleteMin => pairing.delete_min(),
        };
    }
    let left_over = Vec::from(pairing);
    // TODO: we could prettify this one a bit.
    let mut result = vec![];
    for (item, i) in left_over {
        assert_eq!(ops_result[i], Some(Operation::Insert(item.clone())));
        ops_result[i] = None;
        result.push(item);
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
            simulate_pairing(ops);
        }

        #[test]
        fn corruption(ops in full_ops(10_000)) {
            simulate_pairing(ops.0);
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
        fn test_via_pairing_heap(ops in operations()) {
            let mut naive = sim_naive(ops.clone());
            let mut pairing_in = linear(ops);

            naive.sort();
            pairing_in.sort();

            prop_assert_eq!(naive, pairing_in);
        }
    }
}
