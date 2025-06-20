// Soft heaps based on pairing heaps.
// We do min-heaps by default.

use std::{collections::VecDeque, iter::once};

use itertools::{chain, enumerate, Itertools};
use rand::seq::SliceRandom;

use crate::tools::previous_full_multiple;

// use crate::tools::previous_full_multiple;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Pool<T> {
    pub item: T,
    pub count: usize,
}

impl<T> Pool<T> {
    pub fn new(item: T) -> Self {
        Pool { item, count: 0 }
    }

    pub fn delete_one(self) -> Result<Self, T> {
        match self.count.checked_sub(1) {
            Some(count) => Ok(Self {
                count,
                item: self.item,
            }),
            None => Err(self.item),
        }
    }

    #[must_use]
    pub fn add_to_pool(mut self, count: usize) -> Self {
        self.count += count;
        self
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Pairing<const CORRUPT_EVERY_N: usize, T> {
    pub key: Pool<T>,
    pub children: Vec<Pairing<CORRUPT_EVERY_N, T>>,
}

impl<const CORRUPT_EVERY_N: usize, T> From<Pool<T>> for Pairing<CORRUPT_EVERY_N, T> {
    fn from(key: Pool<T>) -> Self {
        Self {
            key,
            children: vec![],
        }
    }
}

impl<const CORRUPT_EVERY_N: usize, T> Pairing<CORRUPT_EVERY_N, T> {
    pub fn new(item: T) -> Self {
        Self::from(Pool::new(item))
    }

    pub fn count_corrupted(&self) -> usize {
        self.key.count
            + self
                .children
                .iter()
                .map(Pairing::count_corrupted)
                .sum::<usize>()
    }

    pub fn count_uncorrupted(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(Pairing::count_uncorrupted)
            .sum::<usize>()
    }
}

const BOUND: usize = 2;

impl<const CORRUPT_EVERY_N: usize, T: Ord> Pairing<CORRUPT_EVERY_N, T> {
    #[must_use]
    pub fn meld(self, other: Self) -> Self {
        let (mut a, b) = if self.key.item < other.key.item {
            (self, other)
        } else {
            (other, self)
        };
        a.children.push(b);
        a
    }

    #[must_use]
    pub fn bounded_meld(self, other: Self) -> Self {
        let (mut a, b) = if self.key.item < other.key.item {
            (self, other)
        } else {
            (other, self)
        };
        a.children.push(b);
        Pairing {
            key: a.key,
            children: Self::merge_many_bound(a.children),
        }
    }

    #[must_use]
    pub fn merge_many_bound(mut items: Vec<Self>) -> Vec<Self> {
        if items.len() <= BOUND {
            return items;
        }
        assert_eq!(items.len(), BOUND + 1);

        // let c = items.pop();
        // let b = items.pop();
        // let a = items.pop();
        // chain!(a, Self::bounded_meld_option(b, c)).collect::<Vec<_>>()

        //

        // OK, the swap doesn't work.  Because it just builds two big lists,
        // it doesn't do pairing to limit the depth.

        // let c = items.pop();
        // let b = items.pop();
        // let a = items.pop();

        // chain!(Self::bounded_meld_option(b, c), a).collect::<Vec<_>>()

        // a b c
        // => c (a | b)
        // c (a | b) d
        // => d (c | (a | b))
        // d (c | (a | b)) e
        // => e (d | (c | (b | a)))

        // vs
        // a b c
        // => (b | c) a
        // (b | c) a d
        // => (a | d) (b | c)
        // (a | d) (b | c) e
        // => (b | c | e) (a | d)
        // (b | c | e) (a | d) f
        // => (a | d | f) (b | c | e)

        // OK, the above works better, I think?

        // // The two below are equivalent for BOUND==2 only;
        // // a b c -> a | (b | c)
        items.reverse();
        items
            .into_iter()
            .reduce(Self::bounded_meld)
            .into_iter()
            .collect()

        // // while items.len() > 1 {
        // //     // items.reverse();
        // //     // If we have more than BOUND items, we need to merge them.
        // //     let chunked = items.into_iter().chunks(2);
        // //     items = chunked
        // //         .into_iter()
        // //         .filter_map(|chunk| chunk.reduce(Self::bounded_meld))
        // //         .collect();
        // // }
        // // items
    }

    // This copy and paste job could be avoided with an extra parameter.
    #[must_use]
    pub fn merge_many_bound1(items: Vec<Self>) -> Option<Self> {
        // This only works for BOUND==2.  (At least my analysis does.)
        let items = Self::merge_many_bound(items);
        items.into_iter().reduce(Self::bounded_meld)

        // items.reverse();
        // The two below are equivalent for BOUND==2 only;

        // while items.len() > 1 {
        //     let chunked = items.into_iter().chunks(2);
        //     items = chunked
        //         .into_iter()
        //         .filter_map(|chunk| chunk.reduce(Self::bounded_meld))
        //         .collect();
        // }
        // items.into_iter().next()
        // items.into_iter().reduce(Self::bounded_meld)
    }

    // #[must_use]
    // pub fn merge_children_bounded(items: Vec<Self>, _corrupted: &mut Vec<T>) -> Option<Self> {
    //     // TODO: introduce corruption later.
    //     Self::merge_many_bound1(items)
    // }

    #[must_use]
    pub fn meld_option(me: Option<Self>, other: Option<Self>) -> Option<Self> {
        match (me, other) {
            (me, None) => me,
            (None, other) => other,
            (Some(a), Some(b)) => Some(a.meld(b)),
        }
    }

    // #[must_use]
    // pub fn bounded_meld_option(me: Option<Self>, other: Option<Self>) -> Option<Self> {
    //     match (me, other) {
    //         (me, None) => me,
    //         (None, other) => other,
    //         (Some(a), Some(b)) => Some(a.bounded_meld(b)),
    //     }
    // }

    #[must_use]
    pub fn insert(self, item: T) -> Self {
        self.meld(Self::new(item))
    }

    // #[must_use]
    // pub fn bounded_insert(self, item: T) -> Self {
    //     self.bounded_meld(Self::new(item))
    // }

    /// Corrupts the heap by pooling the top two elements.
    ///
    /// # Panics
    ///
    /// Panics if the heap property is violated (when the key's item is greater than
    /// the merged pairing's key item).
    #[must_use]
    pub fn corrupt(self, corrupted: &mut Vec<T>) -> Self {
        let Pairing { key, children } = self;
        match Self::merge_children(children, corrupted) {
            None => Pairing::from(key),
            Some(pairing) => {
                assert!(key.item <= pairing.key.item);
                corrupted.push(key.item);
                Pairing {
                    key: pairing.key.add_to_pool(key.count + 1),
                    children: pairing.children,
                }
            }
        }
    }

    pub fn heavy_delete_min(self) -> (Option<Self>, Pool<T>, Vec<T>) {
        let mut corrupted = vec![];
        let Pairing { key, children } = self;
        (
            Self::merge_children(children, &mut corrupted),
            key,
            corrupted,
        )
    }

    pub fn delete_min(self) -> (Option<Self>, Option<T>, Vec<T>) {
        let mut corrupted = vec![];
        let Pairing { key, children } = self;

        let (a, b) = match key.delete_one() {
            Err(item) => (Self::merge_children(children, &mut corrupted), Some(item)),
            Ok(key) => (Some(Self { key, children }), None),
        };
        (a, b, corrupted)
    }

    #[must_use]
    pub fn merge_many<I>(items: I) -> Option<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let mut d: VecDeque<_> = items.into_iter().collect();
        loop {
            match (d.pop_front(), d.pop_front()) {
                (Some(a), Some(b)) => d.push_back(a.meld(b)),
                (a, _) => return a,
            }
        }
    }

    pub fn merge_children_multi_pass_binary(
        items: Vec<Self>,
        corrupted: &mut Vec<T>,
    ) -> Option<Self> {
        let mut digits: Vec<Option<Self>> = vec![];

        // Add items one by one by simulating incrementing a binary number.
        // We do this work in the merge_children of delete-min,
        // but it's actually paid for by the inserts' O(1) charge already.
        // So we don't need to corrupt here.
        for item in items {
            let mut carry = item;
            // Make sure that we always have one trailing zero, ie trailing None,
            // so that standard carry logic works.
            if let Some(None) = digits.last() {
            } else {
                digits.push(None);
            }
            for digit in &mut digits {
                match digit.take() {
                    None => {
                        *digit = Some(carry);
                        break;
                    }
                    Some(digit_item) => {
                        carry = carry.meld(digit_item);
                    }
                }
            }
        }
        // Roll up our counter.
        let mut digits = digits.into_iter().flatten();
        let mut carry = digits.next()?;
        let digits = enumerate(digits);
        for (i, digit) in digits {
            carry = carry.meld(digit);
            if (i.saturating_sub(BOUND) + 1) % CORRUPT_EVERY_N == 0 {
                // After BOUND, corrupt every CORRUPT_EVERY_Nth item.
                carry = carry.corrupt(corrupted);
            }
        }
        Some(carry)
    }

    /// Merges the list of children of a (former) node into one node.
    ///
    /// See 'A Nearly-Tight Analysis of Multipass Pairing Heaps"
    /// by Corwin Sinnamon and Robert E. Tarjan.
    /// <https://epubs.siam.org/doi/epdf/10.1137/1.9781611977554.ch23>
    ///
    /// The paper explains why multipass (like here) does give O(log n)
    /// delete-min.
    ///
    /// (Originally O(log n) delete-min was only proven for the two-pass
    /// variant.)
    #[must_use]
    pub fn merge_children_pass_h_last_no_corruption(
        mut items: Vec<Self>,
        corrupted: &mut Vec<T>,
    ) -> Option<Self> {
        // let start0 = previous_full_multiple(items.len(), CORRUPT_EVERY_N);
        let start = items
            .len()
            .next_multiple_of(CORRUPT_EVERY_N)
            .saturating_sub(CORRUPT_EVERY_N);
        // assert!(start >= start0, "start: {start}, start0: {start0}, items.len(): {}", items.len());
        assert_eq!(0, start % CORRUPT_EVERY_N);
        assert!(items.len() - start < CORRUPT_EVERY_N);
        // assert!(items.len() >= 0);
        let last = Self::merge_many(items.drain(start..));
        let binding = items.into_iter().chunks(CORRUPT_EVERY_N);
        let chunked = binding
            .into_iter()
            .filter_map(Self::merge_many)
            .map(|a| a.corrupt(corrupted));
        Self::merge_many(chain!(chunked, once(last).flatten()))
    }

    pub fn merge_children_pass_h(mut items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<Self> {
        let end = items.len() % CORRUPT_EVERY_N;
        let start = items.len() - end;
        let start0 = previous_full_multiple(items.len(), CORRUPT_EVERY_N);
        // let start = items
        //     .len()
        //     .next_multiple_of(CORRUPT_EVERY_N)
        //     .saturating_sub(CORRUPT_EVERY_N);
        assert!(
            start >= start0,
            "start: {start}, start0: {start0}, items.len(): {}",
            items.len()
        );
        assert_eq!(0, start % CORRUPT_EVERY_N);
        assert!(items.len() - start < CORRUPT_EVERY_N);
        // assert!(items.len() >= 0);
        let last = Self::merge_many(items.drain(start..));
        let binding = items.into_iter().chunks(CORRUPT_EVERY_N);
        let chunked = binding
            .into_iter()
            .filter_map(Self::merge_many)
            .map(|a| a.corrupt(corrupted));
        Self::merge_many(chain!(chunked, once(last).flatten()))
    }

    pub fn merge_children_pass_h_queue(items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<Self> {
        let mut queue: VecDeque<_> = VecDeque::from(items);
        let mut free: usize = 1;
        loop {
            if queue.len() <= free * CORRUPT_EVERY_N {
                return Self::merge_many(queue);
            }
            if let Some(Pairing {
                key: Pool { item, count },
                mut children,
            }) = Self::merge_many(queue.drain(..CORRUPT_EVERY_N))
            {
                free += 1;
                corrupted.push(item);
                // children.sort();
                children.shuffle(&mut rand::rng());
                // ch
                // children.reverse();
                if let Some(first) = children.first_mut() {
                    first.key.count += count + 1;
                } else {
                    unreachable!(
                        "We should always have at least one child after merging a full chunk."
                    );
                }
                children.shuffle(&mut rand::rng());
                queue.extend(children);
            } else {
                unreachable!("We should have have a non-empty heap after merging a full chunk.");
            }
        }
    }

    pub fn merge_children_pass_h_queue_power_of_n(
        items: Vec<Self>,
        corrupted: &mut Vec<T>,
    ) -> Option<Self> {
        let mut queue: VecDeque<_> = VecDeque::from(items);
        loop {
            if queue.len() < CORRUPT_EVERY_N {
                return Self::merge_many(queue);
            }
            if let Some(Pairing {
                key: Pool { item, count },
                mut children,
            }) = Self::merge_many(queue.drain(..CORRUPT_EVERY_N))
            {
                const POWER: usize = 10;

                corrupted.push(item);
                // children.sort();
                children.shuffle(&mut rand::rng());

                let mix = children.split_off(children.len().saturating_sub(POWER));
                if let Some(r) = Self::merge_many(mix) {
                    children.push(r);
                } else {
                    unreachable!("We should have a non-empty heap after merging a chunk.");
                }

                if let Some(last) = children.last_mut() {
                    last.key.count += count + 1;
                } else {
                    unreachable!(
                        "We should always have at least one child after merging a full chunk."
                    );
                }
                children.shuffle(&mut rand::rng());
                queue.extend(children);
            } else {
                unreachable!("We should have have a non-empty heap after merging a full chunk.");
            }
        }
    }

    pub fn merge_children_pass_h_queue_simple(
        items: Vec<Self>,
        corrupted: &mut Vec<T>,
    ) -> Option<Self> {
        let mut queue: VecDeque<Pairing<CORRUPT_EVERY_N, T>> = VecDeque::from(items);
        loop {
            if queue.len() <= CORRUPT_EVERY_N {
                return Self::merge_many(queue);
            }
            if let Some(p) = Self::merge_many(queue.drain(..CORRUPT_EVERY_N)) {
                queue.push_back(p.corrupt(corrupted));
            } else {
                unreachable!("We should have have a non-empty heap after merging a full chunk.");
            }
        }
    }

    #[must_use]
    // multi-pass, evenly distributed corruption
    pub fn merge_children_evenly(items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<Self> {
        // let mut d: VecDeque<_> = items
        //     .into_iter()
        //     .chunks(2)
        //     .into_iter()
        //     .filter_map(|chunk| chunk.reduce(Self::meld))
        //     .collect();
        let mut d: VecDeque<_> = VecDeque::from(items);
        for c in 1.. {
            let next = match (d.pop_front(), d.pop_front()) {
                (Some(a), Some(b)) => a.meld(b),
                (a, _) => return a,
            };
            d.push_back(if c % CORRUPT_EVERY_N == 0 {
                next.corrupt(corrupted)
            } else {
                next
            });
        }
        unreachable!()
    }

    #[must_use]
    pub fn merge_children_two_pass(items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<Self> {
        items
            .into_iter()
            .chunks(2)
            .into_iter()
            .filter_map(|chunk| chunk.reduce(Self::meld))
            .chunks(CORRUPT_EVERY_N)
            .into_iter()
            .fold(None, |acc, item| {
                chain!(acc.map(|acc| acc.corrupt(corrupted)), item).reduce(Self::meld)
            })
    }

    #[must_use]
    pub fn merge_children_two_pass_grouped(
        items: Vec<Self>,
        corrupted: &mut Vec<T>,
    ) -> Option<Self> {
        let mut queue: VecDeque<_> = VecDeque::from(items);
        loop {
            if queue.len() < CORRUPT_EVERY_N {
                return Self::merge_many(queue);
            }
            if let Some(p) = Self::merge_many(queue.drain(..CORRUPT_EVERY_N)) {
                queue.push_front(p.corrupt(corrupted));
            } else {
                unreachable!("We should have have a non-empty heap after merging a full chunk.");
            }
        }
    }

    #[must_use]
    pub fn merge_children_two_pass_grouped_last(
        items: Vec<Self>,
        corrupted: &mut Vec<T>,
    ) -> Option<Self> {
        let mut queue: VecDeque<_> = VecDeque::from(items);
        loop {
            if queue.len() < CORRUPT_EVERY_N {
                return Self::merge_many(queue);
            }
            if let Some(p) = Self::merge_many(queue.drain(..CORRUPT_EVERY_N)) {
                queue.push_back(p.corrupt(corrupted));
            } else {
                unreachable!("We should have have a non-empty heap after merging a full chunk.");
            }
        }
    }

    #[must_use]
    pub fn merge_children_multi_grouped(items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<Self> {
        const EVERY_SECOND_LAYER: usize = 4;
        let binding = items.into_iter().chunks(CORRUPT_EVERY_N);
        let mut queue: VecDeque<_> = binding.into_iter().filter_map(Self::merge_many).collect();
        loop {
            if queue.len() < EVERY_SECOND_LAYER {
                return Self::merge_many(queue);
            }
            let res = Self::merge_many(queue.drain(..EVERY_SECOND_LAYER))
                .expect("We should have have a non-empty heap after merging a full chunk.")
                .corrupt(corrupted);
            queue.push_back(res);
            // if let Some(p) = Self::merge_many(queue.drain(..EVERY_SECOND_LAYER)) {
            //     // queue.push_back(p.corrupt(corrupted));
            //     queue.push_back(p.corrupt(corrupted));
            // } else {
            //     unreachable!("We should have have a non-empty heap after merging a full chunk.");
            // }
        }
    }

    #[must_use]
    pub fn merge_children_multi_grouped_less_grace(
        items: Vec<Self>,
        corrupted: &mut Vec<T>,
    ) -> Option<Self> {
        const EVERY_SECOND_LAYER: usize = 4;
        let binding = items.into_iter().chunks(CORRUPT_EVERY_N);
        let mut queue: VecDeque<_> = binding.into_iter().filter_map(Self::merge_many).collect();
        loop {
            if queue.len() < EVERY_SECOND_LAYER {
                return Self::merge_many(queue);
            };

            if let Some(p) = Self::merge_many(
                queue
                    .drain(..EVERY_SECOND_LAYER.min(queue.len()))
                    .map(|p| p.corrupt(corrupted)),
            ) {
                // queue.push_back(p.corrupt(corrupted));
                queue.push_front(p);
            } else {
                unreachable!("We should have have a non-empty heap after merging a full chunk.");
            }
        }
    }

    #[must_use]
    // This one does not work!  Leads to 100% corruption.
    // Probably for the same reason it leads to amortised O(n) delete-min
    pub fn merge_children_one_pass(items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<Self> {
        items
            .into_iter()
            // .chunks(2)
            // .into_iter()
            // .filter_map(|chunk| chunk.reduce(Self::meld))
            .chunks(CORRUPT_EVERY_N)
            .into_iter()
            .fold(None, |acc, item| {
                chain!(acc.map(|acc| acc.corrupt(corrupted)), item).reduce(Self::meld)
            })
    }

    #[must_use]
    // This one does not work!  Leads to 100% corruption.
    pub fn merge_children_two_pass_degree(
        items: Vec<Self>,
        corrupted: &mut Vec<T>,
    ) -> Option<Self> {
        items
            .into_iter()
            .chunks(2)
            .into_iter()
            .filter_map(|chunk| chunk.reduce(Self::meld))
            .reduce(|a, b| {
                let acc = a.meld(b);
                if acc.children.len() > CORRUPT_EVERY_N {
                    acc.corrupt(corrupted)
                } else {
                    acc
                }
            })
    }

    pub fn merge_children(items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<Self> {
        // This one doesn't actually do any corruption
        // Self::merge_children_bounded(items, corrupted)
        Self::merge_children_multi_pass_binary(items, corrupted)

        // This one seems to work, but it has a higher corruption rate for our parameter.
        // Self::merge_children_evenly(items, corrupted)

        // These ones work:
        // Self::merge_children_pass_h(items, corrupted)
        // Self::merge_children_pass_h_queue_simple(items, corrupted)
        // Self::merge_children_two_pass(items, corrupted)
        // Self::merge_children_two_pass_grouped(items, corrupted)
        // Self::merge_children_two_pass_grouped_last(items, corrupted)

        // Self::merge_children_multi_grouped(items, corrupted)
        // Self::merge_children_multi_grouped_less_grace(items, corrupted)

        // These ones should maybe work, but doesn't:
        // Self::merge_children_pass_h_queue(items, corrupted)
        // Self::merge_children_pass_h_queue_power_of_n(items, corrupted)

        // // These two do not work!
        // Self::merge_children_two_pass_degree(items, corrupted)
        // Self::merge_children_one_pass(items, corrupted)
        // Self::merge_children_at_end(items, corrupted)
    }

    // Corrupt all at the end.
    #[must_use]
    pub fn merge_children_at_end(items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<Self> {
        let l = items.len().max(1);
        let mut d: VecDeque<_> = VecDeque::from(items);
        // Total = l-1 comparisons.
        // So we need floor(l / EVERY) corruptions at the end.
        let end = l - l / CORRUPT_EVERY_N;
        assert!(end <= l, "end: {end}, l: {l}");
        assert!((l - end) * CORRUPT_EVERY_N <= l);
        for c in 1.. {
            assert!(c <= l, "c: {c}, l: {l}");
            let next = match (d.pop_front(), d.pop_front()) {
                (Some(a), Some(b)) => a.meld(b),
                (a, _) => {
                    assert_eq!(c, l);
                    return a;
                }
            };
            d.push_back(if c > end {
                next.corrupt(corrupted)
            } else {
                next
            });
        }
        unreachable!()
    }

    pub fn check_heap_property(&self) -> bool {
        let Pairing { key, children } = self;
        children.iter().all(|child| key.item <= child.key.item)
            && children.iter().all(Self::check_heap_property)
    }
}

// Get all non-corrupted elements still in the heap.
impl<const CORRUPT_EVERY_N: usize, T> From<Pairing<CORRUPT_EVERY_N, T>> for Vec<T> {
    fn from(pairing: Pairing<CORRUPT_EVERY_N, T>) -> Self {
        // Pre-order traversal.
        let mut items = vec![];
        let mut todo = VecDeque::from([pairing]);
        while let Some(pairing) = todo.pop_front() {
            let Pairing {
                key: Pool { item, count: _ },
                children,
            } = pairing;
            todo.extend(children);
            items.push(item);
        }
        items
    }
}

// Idea: look at my 'static visualisation' of sorting algorithms for various sequences of operations.
// Also: add tests etc.
// Also: actually use the soft pairing heap for my Schubert matroid.

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct SoftHeap<const CORRUPT_EVERY_N: usize, T> {
    pub root: Option<Pairing<CORRUPT_EVERY_N, T>>,
    pub size: usize,
    pub corrupted: usize,
}

impl<const CORRUPT_EVERY_N: usize, T> Default for SoftHeap<CORRUPT_EVERY_N, T> {
    fn default() -> Self {
        Self {
            root: None,
            size: 0,
            corrupted: 0,
        }
    }
}

impl<const CORRUPT_EVERY_N: usize, T> SoftHeap<CORRUPT_EVERY_N, T> {
    #[must_use]
    pub fn singleton(item: T) -> Self {
        Self {
            root: Some(Pairing::new(item)),
            size: 1,
            corrupted: 0,
        }
    }
}

impl<const CORRUPT_EVERY_N: usize, T: Ord> SoftHeap<CORRUPT_EVERY_N, T> {
    #[must_use]
    pub fn insert(self, item: T) -> Self {
        match self.root {
            None => Self {
                root: Some(Pairing::new(item)),
                size: 1,
                corrupted: 0,
            },
            Some(root) => Self {
                root: Some(root.insert(item)),
                size: self.size + 1,
                corrupted: self.corrupted,
            },
        }
    }

    #[must_use]
    pub fn meld(self, other: Self) -> Self {
        let root = Pairing::meld_option(self.root, other.root);
        Self {
            root,
            size: self.size + other.size,
            corrupted: self.corrupted + other.corrupted,
        }
    }

    // #[must_use]
    // pub fn bounded_meld(self, other: Self) -> Self {
    //     let root = Pairing::bounded_meld_option(self.root, other.root);
    //     Self {
    //         root,
    //         size: self.size + other.size,
    //         corrupted: self.corrupted + other.corrupted,
    //     }
    // }

    #[must_use]
    pub fn heavy_delete_min(self) -> (Self, Option<T>, Vec<T>) {
        match self.root {
            None => (Self::default(), None, vec![]),
            Some(root) => {
                let (root, pool, corrupted) = root.heavy_delete_min();
                (
                    Self {
                        root,
                        size: self.size - pool.count - 1,
                        corrupted: self.corrupted + corrupted.len() - pool.count,
                    },
                    Some(pool.item),
                    corrupted,
                )
            }
        }
    }

    #[must_use]
    pub fn delete_min(self) -> (Self, Option<T>, Vec<T>) {
        // TODO: simplify.
        match self.root {
            None => (Self::default(), None, vec![]),
            Some(root) => {
                let (root, item, corrupted) = root.delete_min();
                (
                    Self {
                        root,
                        size: self.size - 1,
                        corrupted: self.corrupted + corrupted.len() - usize::from(item.is_none()),
                    },
                    item,
                    corrupted,
                )
            }
        }
    }
}
impl<const CORRUPT_EVERY_N: usize, T> SoftHeap<CORRUPT_EVERY_N, T> {
    pub fn count_children(&self) -> usize {
        self.root.as_ref().map_or(0, |r| r.children.len())
    }
    pub fn count_corrupted(&self) -> usize {
        debug_assert_eq!(
            self.corrupted,
            self.root.as_ref().map_or(0, Pairing::count_corrupted)
        );
        self.corrupted
    }
    pub fn count_uncorrupted(&self) -> usize {
        debug_assert_eq!(
            self.root.as_ref().map_or(0, Pairing::count_uncorrupted),
            self.size - self.corrupted
        );
        self.size - self.count_corrupted()
    }
    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.size, self.count_uncorrupted() + self.count_corrupted());
        debug_assert_eq!(self.size > 0, self.root.is_some());
        self.root.is_none()
    }
}

impl<const CORRUPT_EVERY_N: usize, T> From<SoftHeap<CORRUPT_EVERY_N, T>> for Vec<T> {
    fn from(SoftHeap { root, .. }: SoftHeap<CORRUPT_EVERY_N, T>) -> Self {
        root.map(Vec::from).unwrap_or_default()
    }
}
