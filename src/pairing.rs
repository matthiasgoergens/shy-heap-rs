// Soft heaps based on pairing heaps.
// We do min-heaps by default.

use std::{collections::VecDeque, iter::once};

use itertools::{chain, Itertools};

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

impl<const CORRUPT_EVERY_N: usize, T: Ord> Pairing<CORRUPT_EVERY_N, T> {
    #[must_use]
    pub fn meld(self, other: Pairing<CORRUPT_EVERY_N, T>) -> Pairing<CORRUPT_EVERY_N, T> {
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
    pub fn merge_children_pass_h(mut items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<Self> {
        // let start = previous_full_multiple(items.len(), CORRUPT_EVERY_N);
        let start = items.len().next_multiple_of(CORRUPT_EVERY_N).saturating_sub(CORRUPT_EVERY_N);
        assert_eq!(0, start % CORRUPT_EVERY_N);
        assert!(items.len() - start <= CORRUPT_EVERY_N);
        assert!(items.len() >= start);
        let last = Self::merge_many(items.drain(start..));
        let binding = items.into_iter().chunks(CORRUPT_EVERY_N);
        let chunked = binding
            .into_iter()
            .filter_map(Self::merge_many)
            .map(|a| a.corrupt(corrupted));
        Self::merge_many(chain!(chunked, once(last).flatten()))
    }

    #[must_use]
    pub fn merge_children_evenly_spaced(items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<Self> {
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

    // Corrupt all at the end.
    #[must_use]
    pub fn merge_children(items: Vec<Self>, corrupted: &mut Vec<T>) -> Option<Self> {
        let l = items.len().max(1);
        let mut d: VecDeque<_> = VecDeque::from(items);
        for c in 1.. {
            assert!(c <= l, "c: {c}, l: {l}");
            let next = match (d.pop_front(), d.pop_front()) {
                (Some(a), Some(b)) => a.meld(b),
                (a, _) => {
                    assert_eq!(c, l);
                    return a;
                },
            };
            d.push_back(if c % CORRUPT_EVERY_N == 0 {
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
