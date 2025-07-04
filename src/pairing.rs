// Soft heaps based on pairing heaps.
// We do min-heaps by default.

use itertools::{chain, Itertools};
use std::ops::Add;
use std::{collections::VecDeque, mem};

use crate::witness_set::{Witnessed, WitnessedSet};

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
pub struct UnboundWitnessed<T> {
    pub pairing: Pairing<T>,
    pub to_be_witnessed: WitnessedSet<T>,
}

impl<T> From<Pairing<T>> for UnboundWitnessed<T> {
    fn from(pairing: Pairing<T>) -> Self {
        Self {
            pairing,
            to_be_witnessed: WitnessedSet::default(),
        }
    }
}

impl<T: Ord> UnboundWitnessed<T> {
    #[must_use]
    pub fn extract(me: Option<Self>) -> (Option<Pairing<T>>, Vec<T>) {
        if let Some(UnboundWitnessed {
            pairing,
            to_be_witnessed: witnessed,
        }) = me
        {
            (Some(pairing), Vec::from(witnessed))
        } else {
            (None, vec![])
        }
    }

    #[must_use]
    pub fn meld(self, other: Self) -> Self {
        let (mut a, b) = if self.pairing.key.item <= other.pairing.key.item {
            (self, other)
        } else {
            (other, self)
        };
        a.pairing.witnessed.extend(b.to_be_witnessed);
        a.pairing.children.push(b.pairing);
        a
    }

    #[must_use]
    pub fn pop_min(
        self,
        corrupt_every_n: usize,
    ) -> (Option<Pairing<T>>, Option<T>, WitnessedSet<T>) {
        let UnboundWitnessed {
            pairing:
                Pairing {
                    key,
                    children,
                    witnessed,
                },
            mut to_be_witnessed,
        } = self;
        let (new_me, deleted_item) = match key.delete_one() {
            Ok(key) => (
                Some(Pairing {
                    key,
                    witnessed,
                    children,
                }),
                None,
            ),
            Err(item) => (
                if let Some(UnboundWitnessed {
                    to_be_witnessed: tbc,
                    pairing,
                }) = Self::merge_children(corrupt_every_n, children)
                {
                    // This might need to change?  TODO: we always need to do this.
                    to_be_witnessed.extend(witnessed);
                    to_be_witnessed.extend(tbc);
                    Some(pairing)
                } else {
                    to_be_witnessed.extend(witnessed);
                    None
                },
                Some(item),
            ),
        };

        (new_me, deleted_item, to_be_witnessed)
    }

    pub fn heavy_pop_min(
        self,
        corrupt_every_n: usize,
    ) -> (Option<Pairing<T>>, Pool<T>, WitnessedSet<T>) {
        let UnboundWitnessed {
            pairing:
                Pairing {
                    key,
                    children,
                    witnessed,
                },
            mut to_be_witnessed,
        } = self;
        to_be_witnessed.extend(witnessed);

        let new_me = Self::merge_children(corrupt_every_n, children).map(
            |UnboundWitnessed {
                 to_be_witnessed: tbc,
                 pairing,
             }| {
                to_be_witnessed.extend(tbc);
                pairing
            },
        );
        (new_me, key, to_be_witnessed)
    }

    #[must_use]
    pub fn merge_many(items: impl IntoIterator<Item = Self>) -> Option<Self> {
        let mut d: VecDeque<_> = items.into_iter().collect();
        loop {
            match (d.pop_front(), d.pop_front()) {
                (Some(a), Some(b)) => d.push_back(a.meld(b)),
                (a, _) => return a,
            }
        }
    }

    #[must_use]
    pub fn merge_children_pass_h(corrupt_every_n: usize, items: Vec<Pairing<T>>) -> Option<Self> {
        let mut items = items.into_iter().map(Self::from).collect::<Vec<_>>();

        let start = items
            .len()
            .add(1)
            .next_multiple_of(corrupt_every_n)
            .saturating_sub(corrupt_every_n);

        assert_eq!(0, start % corrupt_every_n);
        assert!(items.len() - start <= corrupt_every_n);

        let last = Self::merge_many(items.drain(start..));
        let binding = items.into_iter().chunks(corrupt_every_n);
        let chunked = binding
            .into_iter()
            .filter_map(Self::merge_many)
            .map(|x| x.corrupt(corrupt_every_n));
        Self::merge_many(chain!(chunked, last.into_iter()))
    }

    #[must_use]
    pub fn merge_children(corrupt_every_n: usize, children: Vec<Pairing<T>>) -> Option<Self> {
        Self::merge_children_pass_h(corrupt_every_n, children)
    }

    #[must_use]
    pub fn corrupt(self, corrupt_every_n: usize) -> Self {
        // TODO(Matthias): this is like a heavy pop-min, so we should unify?  Maybe..
        let Self {
            pairing:
                Pairing {
                    key,
                    children,
                    witnessed,
                },
            to_be_witnessed: mut tbw_c,
        } = self;
        if let Some(Self {
            pairing,
            mut to_be_witnessed,
        }) = Self::merge_children(corrupt_every_n, children)
        {
            // assert!(key.item <= pairing.key.item);
            tbw_c.extend(witnessed);
            tbw_c.add_child(Witnessed::singleton(key.item));
            to_be_witnessed.extend(tbw_c);
            Self {
                to_be_witnessed,
                pairing: Pairing {
                    key: pairing.key.add_to_pool(key.count + 1),
                    children: pairing.children,
                    witnessed: pairing.witnessed,
                },
            }
        } else {
            unreachable!(
                "This should never happen, we should always have at least one child to corrupt."
            );
            // Well, we have no children, so we can't actually corrupt anything.
            // This shouldn't happen.
            // Self {
            //     pairing: Pairing::from(key),
            //     to_be_witnessed: tbw_c,
            // }
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Pairing<T> {
    pub key: Pool<T>,
    pub witnessed: WitnessedSet<T>,
    pub children: Vec<Pairing<T>>,
}

impl<T> From<Pool<T>> for Pairing<T> {
    fn from(key: Pool<T>) -> Self {
        Self {
            key,
            children: vec![],
            witnessed: WitnessedSet::default(),
        }
    }
}

impl<T> Pairing<T> {
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

    pub fn count_delayed_corruption(&self) -> usize {
        self.witnessed.count
            + self
                .children
                .iter()
                .map(Pairing::count_delayed_corruption)
                .sum::<usize>()
    }
}

// const BOUND: usize = 2;
// TODO: let's worry about whether we really need Clone later.  For now, this is simplest.
impl<T: Ord + Clone> Pairing<T> {
    #[must_use]
    pub fn merge_children_late(_items: Vec<Self>) -> (Option<Self>, WitnessedSet<T>) {
        todo!()
        // // This should give =< 1/3 corruption.
        // let mut total_work: usize = items.len();

        // if let Some(pairing) = Self::merge_many(items) {
        //     const FIXED_CORRUPT_EVERY_N: usize = 4;
        //     let mut work_discharged = CORRUPT_EVERY_N;

        //     let mut candidates: Vec<T> = vec![];
        //     let root = RefCell::new(pairing);
        //     let mut inner_heap: SoftHeap<4, RefCell<Pairing<T>>> =
        //         SoftHeap::singleton(root.clone());
        //     while work_discharged < total_work {
        //         let (new_heap, mut nodes) = inner_heap.pop_min_combined();
        //         candidates.extend(nodes.iter().map(|node| node.borrow().key.item.clone()));
        //         inner_heap = new_heap;
        //         for node in &mut nodes {
        //             if node.borrow().children.len() > 2 {
        //                 todo!();
        //                 // let my_children = mem::take(&mut node.borrow_mut().children);
        //                 // total_work += my_children.len();
        //                 // node.borrow_mut()
        //                 //     .children
        //                 //     .extend(Self::merge_many(my_children));
        //             }
        //             // inner_heap.extend(node.borrow_mut().children.iter_mut().map(RefCell::new));
        //             for child in node.borrow_mut().children.iter_mut() {
        //                 inner_heap.insert(RefCell::new(child));
        //             }
        //         }

        //         work_discharged += FIXED_CORRUPT_EVERY_N;
        //     }
        //     return todo!();
        // }
        // // We have no items, so we can't merge anything.
        // (None, WitnessedSet::default())
    }
}

impl<T: Ord> Pairing<T> {
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
    pub fn insert(self, item: T) -> Self {
        self.meld(Self::new(item))
    }

    #[must_use]
    pub fn meld_option(me: Option<Self>, other: Option<Self>) -> Option<Self> {
        match (me, other) {
            (me, None) => me,
            (None, other) => other,
            (Some(a), Some(b)) => Some(a.meld(b)),
        }
    }

    /*
        #[must_use]
        pub fn meld_witnessed(me: (Self, WitnessedSet<T>), other: (Self, WitnessedSet<T>)) -> (Self, WitnessedSet<T>) {
            let ((mut a, a_w), (b, b_w)) = if me.0.key.item < other.0.key.item {
                (me, other)
            } else {
                (other, me)
            };
            a.witnessed.extend(b_w);
            a.children.push(b);
            (a, a_w)
        }


        /// Corrupts the heap by pooling the top two elements.
        ///
        /// # Panics
        ///
        /// Panics if the heap property is violated (when the key's item is greater than
        /// the merged pairing's key item).
        #[must_use]
        pub fn corrupt(self) -> UnboundWitnessed<T> {
            let Pairing { key, children , witnessed} = self;
            match Self::merge_children(children) {
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
    */

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

    /*
    pub fn merge_children_multi_pass_binary_implicit(
        mut items: Vec<Self>,
        corrupted: &mut Vec<T>,
    ) -> Option<Self> {
        let mut counter: usize = 0;
        while items.len() > 1 {
            let len = items.len();
            let binding = items.into_iter().chunks(2);
            let chunked = binding
                .into_iter()
                .filter_map(|chunk| chunk.reduce(Self::meld));
            items = chunked.collect();
            if len % 4 == 3 {
                // Odd to even!
                assert!(len % 2 == 1);
                assert!(items.len() % 2 == 0);
                counter += 1;
                if counter > BOUND && (counter.saturating_sub(BOUND) % CORRUPT_EVERY_N == 0) {
                    // After BOUND, corrupt every CORRUPT_EVERY_Nth item.
                    if let Some(last) = items.pop() {
                        items.push(last.corrupt(corrupted));
                    }
                }
            } else {
                // Assert: not odd to even.
                assert!((len % 2 == 0) || (items.len() % 2 == 1));
            }
        }
        items.into_iter().next()
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

    // /// Merges the list of children of a (former) node into one node.
    // ///
    // /// See 'A Nearly-Tight Analysis of Multipass Pairing Heaps"
    // /// by Corwin Sinnamon and Robert E. Tarjan.
    // /// <https://epubs.siam.org/doi/epdf/10.1137/1.9781611977554.ch23>
    // ///
    // /// The paper explains why multipass (like here) does give O(log n)
    // /// delete-min.
    // ///
    // /// (Originally O(log n) delete-min was only proven for the two-pass
    // /// variant.)
    // #[must_use]
    // pub fn merge_children_pass_h_last_no_corruption(
    //     mut items: Vec<Self>,
    //     corrupted: &mut Vec<T>,
    // ) -> Option<Self> {
    //     // let start0 = previous_full_multiple(items.len(), CORRUPT_EVERY_N);
    //     let start = items
    //         .len()
    //         .next_multiple_of(CORRUPT_EVERY_N)
    //         .saturating_sub(CORRUPT_EVERY_N);
    //     // assert!(start >= start0, "start: {start}, start0: {start0}, items.len(): {}", items.len());
    //     assert_eq!(0, start % CORRUPT_EVERY_N);
    //     assert!(items.len() - start < CORRUPT_EVERY_N);
    //     // assert!(items.len() >= 0);
    //     let last = Self::merge_many(items.drain(start..));
    //     let binding = items.into_iter().chunks(CORRUPT_EVERY_N);
    //     let chunked = binding
    //         .into_iter()
    //         .filter_map(Self::merge_many)
    //         .map(|a| a.corrupt(corrupted));
    //     Self::merge_many(chain!(chunked, once(last).flatten()))
    // }

    pub fn merge_children_pass_h(mut items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<UnboundWitnessed<T>> {
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
        let mut queue: VecDeque<Pairing<T>> = VecDeque::from(items);
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
            }

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

    pub fn merge_children(items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<UnboundWitnessed<T>> {
        // These two work, really well:
        // Self::merge_children_multi_pass_binary(items, corrupted)
        // Self::merge_children_multi_pass_binary_implicit(items, corrupted)

        // This one doesn't actually do any corruption
        // Self::merge_children_bounded(items, corrupted)

        // This one seems to work, but it has a higher corruption rate for our parameter.
        // Self::merge_children_evenly(items, corrupted)

        // These ones work:
        Self::merge_children_pass_h(items,)
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
    */
}

// Get all non-corrupted elements still in the heap.
impl<T> From<Pairing<T>> for Vec<T> {
    fn from(pairing: Pairing<T>) -> Self {
        // Pre-order traversal.
        let mut items = vec![];
        let mut todo = VecDeque::from([pairing]);
        while let Some(pairing) = todo.pop_front() {
            let Pairing {
                key: Pool { item, count: _ },
                children,
                witnessed: _,
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
pub struct SoftHeap<T> {
    pub root: Option<Pairing<T>>,
    pub size: usize,
    pub corrupted: usize,
    pub corrupt_every_n: usize,
}

impl<T> SoftHeap<T> {
    #[must_use]
    pub fn singleton(corrupt_every_n: usize, item: T) -> Self {
        Self {
            root: Some(Pairing::new(item)),
            size: 1,
            corrupted: 0,
            corrupt_every_n,
        }
    }
    #[must_use]
    pub fn new(corrupt_every_n: usize) -> Self {
        Self {
            root: None,
            size: 0,
            corrupted: 0,
            corrupt_every_n,
        }
    }
}

impl<T: Ord> SoftHeap<T> {
    #[must_use]
    pub fn insert(self, item: T) -> Self {
        match self.root {
            None => Self {
                root: Some(Pairing::new(item)),
                size: 1,
                ..self
            },
            Some(root) => Self {
                root: Some(root.insert(item)),
                size: self.size + 1,
                ..self
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
            corrupt_every_n: self.corrupt_every_n,
        }
    }

    #[must_use]
    pub fn heavy_pop_min(self) -> (Self, Option<T>, Vec<T>) {
        match self.root {
            None => (self, None, vec![]),
            Some(root) => {
                let (root, pool, corrupted) =
                    UnboundWitnessed::from(root).heavy_pop_min(self.corrupt_every_n);
                (
                    Self {
                        root,
                        size: self.size - pool.count - 1,
                        corrupted: self.corrupted + corrupted.count - pool.count,
                        corrupt_every_n: self.corrupt_every_n,
                    },
                    Some(pool.item),
                    Vec::from(corrupted),
                )
            }
        }
    }

    #[must_use]
    pub fn pop_min_combined(self) -> (Self, Vec<T>) {
        let (me, item, mut corrupted) = self.pop_min();
        corrupted.extend(item);
        (me, corrupted)
    }

    #[must_use]
    pub fn pop_min(self) -> (Self, Option<T>, Vec<T>) {
        // TODO: simplify.
        match self.root {
            None => (self, None, vec![]),
            Some(root) => {
                let (root, item, corrupted) =
                    UnboundWitnessed::from(root).pop_min(self.corrupt_every_n);
                (
                    Self {
                        root,
                        size: self.size - 1,
                        corrupted: self.corrupted + corrupted.count - usize::from(item.is_none()),
                        corrupt_every_n: self.corrupt_every_n,
                    },
                    item,
                    Vec::from(corrupted),
                )
            }
        }
    }
}
impl<T> SoftHeap<T> {
    pub fn count_delayed_corruption(&self) -> usize {
        self.root
            .as_ref()
            .map_or(0, Pairing::count_delayed_corruption)
    }

    pub fn count_children(&self) -> usize {
        self.root.as_ref().map_or(0, |r| r.children.len())
    }
    pub fn count_corrupted(&self) -> usize {
        // debug_assert_eq!(
        //     self.corrupted,
        //     self.root.as_ref().map_or(0, Pairing::count_corrupted)
        // );
        self.corrupted
    }
    pub fn count_uncorrupted(&self) -> usize {
        // debug_assert_eq!(
        //     self.root.as_ref().map_or(0, Pairing::count_uncorrupted),
        //     self.size - self.corrupted
        // );
        self.size - self.count_corrupted()
    }
    pub fn is_empty(&self) -> bool {
        // debug_assert_eq!(self.size, self.count_uncorrupted() + self.count_corrupted());
        debug_assert_eq!(self.size > 0, self.root.is_some());
        self.root.is_none()
    }
}

impl<T> From<SoftHeap<T>> for Vec<T> {
    fn from(SoftHeap { root, .. }: SoftHeap<T>) -> Self {
        root.map(Vec::from).unwrap_or_default()
    }
}

impl<T: Ord> Extend<T> for SoftHeap<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        // TODO(Matthias): do we really need to replace self?  Can we do this with some borrowing?
        let mut me = mem::replace(self, Self::new(self.corrupt_every_n));
        for item in iter {
            me = me.insert(item);
        }
        *self = me;
    }
}
pub struct LateHeap<T> {
    pub root: Option<Pairing<T>>,
    pub size: usize,
    pub corrupted: usize,
}

impl<T> Default for LateHeap<T> {
    fn default() -> Self {
        Self {
            root: None,
            size: 0,
            corrupted: 0,
        }
    }
}

impl<T: Ord> LateHeap<T> {
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
    #[must_use]
    pub fn pop_min(self) -> (Self, Option<T>, Vec<T>) {
        // TODO: simplify.
        match self.root {
            None => (Self::default(), None, vec![]),
            Some(_root) => {
                todo!()
            }
        }
    }
    #[must_use]
    pub fn heavy_pop_min(self) -> (Self, Option<T>, Vec<T>) {
        match self.root {
            None => (Self::default(), None, vec![]),
            Some(_root) => {
                todo!()
            }
        }
    }
}
