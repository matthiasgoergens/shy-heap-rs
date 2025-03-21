// Schubert matroids.
use crate::pairing::SoftHeap;
use std::iter::repeat_with;
use std::option::Option;
use std::{cmp::Reverse, fmt::Debug};

use itertools::chain;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operation<T> {
    Insert(T),
    DeleteMin,
}

impl<T> Operation<T> {
    pub fn map<U, F>(self, f: F) -> Operation<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Insert(x) => Operation::Insert(f(x)),
            Self::DeleteMin => Operation::DeleteMin,
        }
    }

    pub const fn as_ref(&self) -> Operation<&T> {
        match *self {
            Self::Insert(ref x) => Operation::Insert(x),
            Self::DeleteMin => Operation::DeleteMin,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bucket<T> {
    pub inserts: Vec<T>,
    pub deletes: usize,
}

impl<T> Default for Bucket<T> {
    fn default() -> Self {
        Bucket {
            inserts: vec![],
            deletes: 0,
        }
    }
}

impl<T> Bucket<T> {
    pub fn extend(&mut self, other: Bucket<T>) {
        self.inserts.extend(other.inserts);
        self.deletes += other.deletes;
    }

    #[must_use]
    pub fn is_in_normal_form(&self) -> bool {
        self.deletes < self.inserts.len()
    }

    #[must_use]
    pub fn total_count(&self) -> usize {
        self.inserts.len() + self.deletes
    }
}

pub type Buckets<T> = Vec<Bucket<T>>;

impl<T> From<Operation<T>> for Bucket<T> {
    fn from(op: Operation<T>) -> Self {
        match op {
            Operation::Insert(x) => Bucket {
                inserts: vec![x],
                deletes: 0,
            },
            Operation::DeleteMin => Bucket {
                inserts: vec![],
                deletes: 1,
            },
        }
    }
}

#[must_use]
pub fn into_buckets<T>(ops: Vec<Operation<T>>) -> Buckets<T> {
    ops.into_iter()
        .map(Bucket::from)
        .collect::<Vec<Bucket<T>>>()
}

impl<T> IntoIterator for Bucket<T> {
    type Item = Operation<T>;
    type IntoIter = std::iter::Chain<
        std::iter::Map<std::vec::IntoIter<T>, fn(T) -> Operation<T>>,
        std::iter::Take<std::iter::RepeatWith<fn() -> Operation<T>>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        let insert_fn: fn(T) -> Operation<T> = Operation::Insert;
        let delete_fn: fn() -> Operation<T> = || Operation::DeleteMin;

        chain!(
            self.inserts.into_iter().map(insert_fn),
            repeat_with(delete_fn).take(self.deletes)
        )
    }
}

pub fn from_buckets<T>(buckets: Buckets<T>) -> Vec<Operation<T>> {
    buckets
        .into_iter()
        .flat_map(IntoIterator::into_iter)
        .collect::<Vec<Operation<T>>>()
}

/// Strictly speaking, this one only works for normalised buckets.
#[must_use]
pub fn dualise_buckets<T>(buckets: Buckets<T>) -> Buckets<Reverse<T>> {
    buckets
        .into_iter()
        .rev()
        .map(|Bucket { inserts, deletes }| Bucket {
            deletes: inserts.len().saturating_sub(deletes),
            inserts: inserts.into_iter().rev().map(Reverse).collect(),
        })
        .collect()
}

/// Normalises the buckets.
///
/// A list of buckets is in normal form, if all but the first bucket are in normal form.
///
/// A bucket is in normal form, if it contributes at least one new item on net.
/// That is, if it has more inserts than deletes.
///
/// If a bucket is _not_ in normal form, we can merge it with the previous bucket, without changing
/// the result of the operations.
///
/// This normal form is required to make dualisation work correctly.
#[must_use]
pub fn normalise_buckets<T>(buckets: Buckets<T>) -> Buckets<T> {
    let mut new_buckets = Vec::new();
    let mut open_bucket = Bucket::default();
    for mut bucket in buckets.into_iter().rev() {
        // combine buckets:
        bucket.extend(open_bucket);

        if bucket.is_in_normal_form() {
            new_buckets.push(bucket);
            open_bucket = Bucket::default();
        } else {
            open_bucket = bucket;
        }
    }
    // This one is just so that dualising doesn't lose items,
    // but also we don't want to add empty buckets.
    if !open_bucket.inserts.is_empty() {
        new_buckets.push(open_bucket);
    }
    new_buckets.reverse();
    new_buckets
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

#[must_use]
pub fn dualise_ops<T>(ops: Vec<Operation<T>>) -> Vec<Operation<Reverse<T>>> {
    from_buckets(dualise_buckets(normalise_buckets(into_buckets(ops))))
}

/// Dualise a dual.
///
/// Logically speaking, dualising is its own inverse.  But we need to fix up the types, because Rust
/// doesn't know that `Reverse<Reverse<T>>` is the same as `T`.
#[must_use]
pub fn undualise_ops<T>(ops: Vec<Operation<Reverse<T>>>) -> Vec<Operation<T>> {
    dualise_ops(ops)
        .into_iter()
        .map(|op| op.map(|Reverse(Reverse(x))| x))
        .collect::<Vec<_>>()
}

// result: definitely-in, definitely-out.
#[must_use]
pub fn linear<T: Ord + Debug + Clone>(ops: Vec<Operation<T>>) -> Vec<T> {
    const CHUNKS: usize = 8;
    let inserts = count_inserts(&ops);
    let deletes = count_deletes(&ops);
    if ops.is_empty() {
        vec![]
    } else if deletes * 2 <= inserts {
        // primal
        let (left_ops, guaranteed_in) = approximate_heap::<CHUNKS, _>(ops);
        chain!(guaranteed_in, linear(left_ops)).collect()
    } else {
        // here we need to dualise.
        let dual_ops = dualise_ops(ops);

        let (left_over_ops, _guaranteed_out) = approximate_heap::<CHUNKS, _>(dual_ops);
        linear(undualise_ops(left_over_ops))
    }
}

/// Processes operations iteratively, alternating between primal and dual approaches.
/// Returns a vector of elements that are definitely in the heap at the end.
///
/// # Panics
///
/// Panics if the operations list does not shrink by at least 1/6 of its size in each iteration.
/// That's the case, when the soft heap corruption guarantee is violated.
#[must_use]
pub fn linear_loop<T: Ord + Debug + Clone>(mut ops: Vec<Operation<T>>) -> Vec<T> {
    const CHUNKS: usize = 8;
    let mut result = vec![];

    while !ops.is_empty() {
        let inserts = count_inserts(&ops);
        let deletes = count_deletes(&ops);

        if deletes * 2 <= inserts {
            // primal
            let (left_ops, guaranteed_in) = approximate_heap::<CHUNKS, _>(ops);
            ops = left_ops;
            result.extend(guaranteed_in);
        } else {
            // here we need to dualise.
            let dual_ops = dualise_ops(ops);

            let (left_over_ops, _guaranteed_out) = approximate_heap::<CHUNKS, _>(dual_ops);
            ops = undualise_ops(left_over_ops);
        }
        assert!(ops.len() <= (inserts + deletes) * 5 / 6,);
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

// The bool in the result mean 'definitely in the heap at the end'

/// Approximates the heap operations in linear time using a soft heap
///
/// This function approximates heap operations (using a soft heap).
///
/// Given any sequence of operations `ops` we have:
/// ```notest
///     let (left_over_ops, guaranteed_survivors) = approximate_heap(ops);
///     precise_heap(left_over_ops) + guaranteed_survivors === precise_heap(ops)
/// ```
/// where (+) means multiset union.
///
/// You could trivially make this work out, by just returning the operations unchanged and zero
/// guaranteed survivors.  But the neat thing is that soft heaps gives us some guarantees.
///
/// Specifically for n inserts and k deletes, we have:
/// ```notest
///    corrupted <= epsilon * n
///    guaranteed_survivors := n - k - corrupted
///    guaranteed_survivors >= n * (1-epsilon) - k
/// ```
/// where epsilon is a function of CHUNKS. For CHUNKS=8, epsilon <= 1/6.
///
/// If you can get k <= n/2, then you can get `guaranteed_survivors` >= n * (1 - 1/6) - n/2 = n/3
#[must_use]
pub fn approximate_heap<const CHUNKS: usize, T: Ord + Debug + Clone>(
    ops: Vec<Operation<T>>,
) -> (Vec<Operation<T>>, Vec<T>) {
    // Wrap ops, so we can keep track of tombstones.
    let mut wrapped_ops: Vec<Operation<Option<T>>> =
        ops.into_iter().map(|op| op.map(Some)).collect();

    // Run the actual heap operations:
    let heap: SoftHeap<CHUNKS, &mut Option<T>> =
        wrapped_ops
            .iter_mut()
            .fold(SoftHeap::default(), |heap, op| match op {
                Operation::Insert(x) => heap.insert(x),
                Operation::DeleteMin => heap.delete_min(),
            });

    // Use the heap to collect guaranteed survivors from the sequence of operations,
    // and leave tombstones in their stead.
    let guaranteed_survivors: Vec<T> = Vec::from(heap)
        .into_iter()
        .filter_map(Option::take)
        .collect();
    // Clean up the tombstones, to get a clean vector of left over operations:
    let left_over_ops: Vec<Operation<T>> = wrapped_ops
        .into_iter()
        .filter_map(|op| match op {
            Operation::Insert(x) => x.map(Operation::Insert),
            Operation::DeleteMin => Some(Operation::DeleteMin),
        })
        .collect();

    (left_over_ops, guaranteed_survivors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::{izip, Itertools};
    use proptest::prelude::{any, Strategy};
    use proptest::prelude::{prop_assert_eq, proptest};
    use std::cmp::min;
    use std::collections::{BTreeSet, BinaryHeap};
    use std::iter::repeat;

    pub struct Ops(pub Vec<Operation<u32>>);

    impl Debug for Ops {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for op in &self.0 {
                match op {
                    Operation::Insert(x) => write!(f, "{x} ")?,
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

    #[must_use]
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

    #[must_use]
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

    #[must_use]
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

    #[must_use]
    /// Simulates the operations using a pairing heap and performs debug assertions.
    ///
    /// # Panics
    ///
    /// Panics if the number of insertions is less than `EPS * corrupted_elements`,
    /// where `corrupted_elements` is the count of corrupted elements in the heap.
    ///
    /// Ie when the soft heap corruption guarantee is violated.
    pub fn simulate_pairing_debug<T: Ord + std::fmt::Debug + Clone>(
        ops: Vec<Operation<T>>,
    ) -> Vec<T> {
        // CHUNKS>=8 and EPS = 6 seem to work.
        // Chunks>=6 and EPS=3 also seem to work.
        let mut pairing: SoftHeap<8, T> = SoftHeap::default();
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
            {
                const EPS: usize = 6;
                assert!(
                    inserts_so_far >= EPS * co,
                    "{inserts_so_far} >= {EPS} * {co}; uncorrupted: {un}\n{pairing:?}"
                );
            }
        }
        Vec::from(pairing)
    }

    proptest! {
        #[test]
        fn corruption_simple(ops in operations()) {
            let _ = simulate_pairing_debug(ops);
        }

        #[test]
        fn corruption(ops in full_ops(10_000)) {
            let _ = simulate_pairing_debug(ops.0);
        }


        #[test]
        fn test_simulate_via_buckets(ops in operations()) {
            let mut naive = sim_naive(ops.clone());
            let mut via_buckets = sim_naive(from_buckets(into_buckets(ops)));

            naive.sort_unstable();
            via_buckets.sort_unstable();

            prop_assert_eq!(naive, via_buckets);
        }

        #[test]
        fn test_simulate_via_buckets_normalised(ops in operations()) {
            let mut naive = sim_naive(ops.clone());
            let mut via_buckets = sim_naive(from_buckets(normalise_buckets(into_buckets(ops))));

            naive.sort_unstable();
            via_buckets.sort_unstable();

            prop_assert_eq!(naive, via_buckets);
        }


        #[test]
        fn test_simulate_dualised(ops in operations()) {
            let mut naive = sim_naive(ops.clone());
            let mut dualised = simulate_dualised(ops);

            naive.sort_unstable();
            dualised.sort_unstable();

            prop_assert_eq!(naive, dualised);
        }
        #[test]
        fn test_via_pairing_heap(ops in full_ops(10_000)) {
            let mut naive = sim_naive(ops.0.clone());
            let mut pairing_in = linear(ops.0.clone());

            naive.sort_unstable();
            pairing_in.sort_unstable();

            prop_assert_eq!(&naive, &pairing_in);
        }
        #[test]
        fn test_via_pairing_heap_loop(ops in full_ops(10_000)) {
            let mut naive = sim_naive(ops.0.clone());
            let mut pairing_in_2 = linear_loop(ops.0);

            naive.sort_unstable();
            pairing_in_2.sort_unstable();

            prop_assert_eq!(&naive, &pairing_in_2);
        }
    }
}
