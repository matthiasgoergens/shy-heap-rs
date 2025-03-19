// Soft heaps based on pairing heaps.
// We do min-heaps by default.

use std::{collections::VecDeque, vec};

use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Pool<T> {
    pub item: T,
    pub count: usize,
}

impl<T> Pool<T> {
    pub fn new(item: T) -> Self {
        Pool { item, count: 0 }
    }

    pub fn delete_one(self) -> Option<Self> {
        self.count
            .checked_sub(1)
            .map(|count| Self { count, ..self })
    }

    pub fn merge(self, other: Self) -> Self {
        // We assume that self.item <= other.item.
        Self {
            item: other.item,
            count: self.count + other.count + 1,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Pairing<T> {
    pub key: Pool<T>,
    pub children: Vec<Pairing<T>>,
}

impl<T> From<Pool<T>> for Pairing<T> {
    fn from(key: Pool<T>) -> Self {
        Self {
            key,
            children: vec![],
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
}

impl<T: Ord> Pairing<T> {
    pub fn meld(self, other: Pairing<T>) -> Pairing<T> {
        let (mut a, b) = if self.key.item < other.key.item {
            (self, other)
        } else {
            (other, self)
        };
        a.children.push(b);
        a
    }

    pub fn insert(self, item: T) -> Self {
        self.meld(Self::new(item))
    }

    pub fn corrupt(self) -> Self {
        let Pairing { key, children } = self;
        match Self::merge_children(children) {
            None => Pairing::from(key),
            Some(pairing) => {
                assert!(key.item <= pairing.key.item);
                Pairing {
                    key: key.merge(pairing.key),
                    children: pairing.children,
                }
            }
        }
    }

    pub fn delete_min(self) -> Option<Self> {
        let Pairing { key, children } = self;
        match key.delete_one() {
            None => Self::merge_children(children),
            Some(key) => Some(Self { key, children }),
        }
    }

    pub fn merge_as_binary_tree<I: IntoIterator<Item = Self>>(items: I) -> Option<Self> {
        let items: Vec<_> = items
            .into_iter()
            .chunks(2)
            .into_iter()
            .filter_map(|pair| pair.reduce(Self::meld))
            .collect();
        if items.len() < 2 {
            items.into_iter().reduce(Self::meld)
        } else {
            Self::merge_as_binary_tree(items)
        }
    }

    pub fn merge_children(items: Vec<Self>) -> Option<Self> {
        Self::merge_as_binary_tree(
            items
                .into_iter()
                .chunks(CHUNKS)
                .into_iter()
                .map(Iterator::collect)
                .filter_map(|chunk: Vec<_>| {
                    // Only corrupt full chunks.
                    if chunk.len() < CHUNKS {
                        Self::merge_as_binary_tree(chunk)
                    } else {
                        Self::merge_as_binary_tree(chunk).map(Pairing::corrupt)
                    }
                }),
        )
    }

    pub fn check_heap_property(&self) -> bool {
        let Pairing { key, children } = self;
        children.iter().all(|child| key.item <= child.key.item)
            && children.iter().all(Self::check_heap_property)
    }
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
            } = pairing;
            todo.extend(children);
            items.push(item);
        }
        items
    }
}

/// This one controls the soft heap's 'epsilon' corruption behaviour.
// const EVERY: usize = 3;
pub const CHUNKS: usize = 8;
// Assert: inserts_so_far >= EPS * corrupted.
// Ie, at most 1/EPS * inserts_so_far of the heap is corrupted.
pub const EPS: usize = 3;

// Idea: look at my 'static visualisation' of sorting algorithms for various sequences of operations.
// Also: add tests etc.
// Also: actually use the soft pairing heap for my Schubert matroid.

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Heap<T> {
    pub root: Option<Pairing<T>>,
}

impl<T> Default for Heap<T> {
    fn default() -> Self {
        Self { root: None }
    }
}

impl<T: Ord> Heap<T> {
    pub fn insert(self, item: T) -> Self {
        match self.root {
            None => Self {
                root: Some(Pairing::new(item)),
            },
            Some(root) => Self {
                root: Some(root.insert(item)),
            },
        }
    }

    pub fn delete_min(self) -> Self {
        Self {
            root: self.root.and_then(Pairing::delete_min),
        }
    }
    pub fn count_corrupted(&self) -> usize {
        self.root.as_ref().map_or(0, Pairing::count_corrupted)
    }
    pub fn count_uncorrupted(&self) -> usize {
        self.root.as_ref().map_or(0, Pairing::count_uncorrupted)
    }
}

impl<T> From<Heap<T>> for Vec<T> {
    fn from(Heap { root }: Heap<T>) -> Self {
        root.map(Vec::from).unwrap_or_default()
    }
}
