// Schubert matroids.
use crate::pairing::SoftHeap;
use std::option::Option;
use std::{cmp::Reverse, fmt::Debug};

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
    from_wrapped_ops(dualise_wrapped_ops(to_wrapped_ops(ops)))
}

/// This is equivalent to the formulation that we can create a nested by matroid by starting with an empty matroid
/// and repeatedly either adding a co-loop or a free extension.
///
/// Coloop === `has_delete` is false
/// Free extension === `has_delete` is true
pub struct WrappedOp<T> {
    pub item: T,
    pub has_delete: bool,
}

impl<T> WrappedOp<T> {
    pub fn map<U, F>(self, f: F) -> WrappedOp<U>
    where
        F: FnOnce(T) -> U,
    {
        WrappedOp {
            item: f(self.item),
            has_delete: self.has_delete,
        }
    }
}

#[must_use]
pub fn to_wrapped_ops<T>(ops: Vec<Operation<T>>) -> Vec<WrappedOp<T>> {
    let mut excess_deletes: usize = 0;
    let mut new_ops = vec![];
    for op in ops.into_iter().rev() {
        match op {
            Operation::Insert(item) => {
                new_ops.push(WrappedOp {
                    item,
                    has_delete: excess_deletes > 0,
                });
                excess_deletes = excess_deletes.saturating_sub(1);
            }
            Operation::DeleteMin => {
                excess_deletes += 1;
            }
        }
    }
    new_ops.reverse();
    new_ops
}

#[must_use]
pub fn from_wrapped_ops<T>(ops: Vec<WrappedOp<T>>) -> Vec<Operation<T>> {
    ops.into_iter()
        .flat_map(|WrappedOp { item, has_delete }| {
            if has_delete {
                vec![Operation::Insert(item), Operation::DeleteMin]
            } else {
                vec![Operation::Insert(item)]
            }
        })
        .collect()
}

#[must_use]
pub fn dualise_wrapped_ops<T>(ops: Vec<WrappedOp<T>>) -> Vec<WrappedOp<Reverse<T>>> {
    ops.into_iter()
        .rev()
        .map(|WrappedOp { item, has_delete }| WrappedOp {
            item: Reverse(item),
            has_delete: !has_delete,
        })
        .collect()
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

#[must_use]
pub fn normalise_ops<T>(ops: Vec<Operation<T>>) -> Vec<Operation<T>> {
    from_wrapped_ops(to_wrapped_ops(ops))
}

/// Processes operations iteratively, alternating between primal and dual approaches.
/// Returns a vector of elements that are definitely in the heap at the end.
///
/// # Panics
///
/// Panics if the operations list does not shrink by at least 1/6 of its size in each iteration.
/// That's the case, when the soft heap corruption guarantee is violated.
#[must_use]
pub fn linear_loop<T: Ord + Debug + Clone>(ops: Vec<Operation<T>>) -> Vec<T> {
    const CHUNKS: usize = 8;

    // Normalising is not necessary, it just helps makes our debug asserts cleaner.
    // Normalising removes eg leading deletes, before anything has been inserted.
    let mut ops = normalise_ops(ops);
    let mut result = vec![];

    while !ops.is_empty() {
        let inserts = count_inserts(&ops);
        let deletes = count_deletes(&ops);

        if deletes * 2 <= inserts {
            // primal
            let (left_ops, guaranteed_in) = approximate_heap::<CHUNKS, _>(ops);
            ops = left_ops;
            result.extend(guaranteed_in);
            assert!(count_inserts(&ops) <= inserts * 2 / 3);
            assert!(count_inserts(&ops) <= inserts / 6 + deletes);
            assert!(count_deletes(&ops) == deletes);
        } else {
            // here we need to dualise.
            let dual_ops = dualise_ops(ops);
            let (left_over_ops, _guaranteed_out) = approximate_heap::<CHUNKS, _>(dual_ops);
            ops = undualise_ops(left_over_ops);
        }
        debug_assert!(count_inserts(&ops) <= inserts * 2 / 3);
        debug_assert!(count_deletes(&ops) <= count_inserts(&ops));
    }
    result
}

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
    // Clean up the tombstones to get a clean vector of left-over operations:
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
    use itertools::{chain, izip, Itertools};
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
        let ops = dualise_ops(ops);

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
        fn test_simulate_normalised_ops(ops in operations()) {
            let mut naive = sim_naive(ops.clone());
            let mut via_wrapped = sim_naive(normalise_ops(ops));

            naive.sort_unstable();
            via_wrapped.sort_unstable();

            prop_assert_eq!(naive, via_wrapped);
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
        fn test_via_pairing_heap_loop(ops in full_ops(10_000)) {
            let mut naive = sim_naive(ops.0.clone());
            let mut pairing_in_2 = linear_loop(ops.0);

            naive.sort_unstable();
            pairing_in_2.sort_unstable();

            prop_assert_eq!(&naive, &pairing_in_2);
        }
    }
}
