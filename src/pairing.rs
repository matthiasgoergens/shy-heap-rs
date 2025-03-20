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

    #[must_use]
    pub fn delete_one(self) -> Option<Self> {
        self.count
            .checked_sub(1)
            .map(|count| Self { count, ..self })
    }

    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        // We assume that self.item <= other.item.
        Self {
            item: other.item,
            count: self.count + other.count + 1,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Pairing<const CHUNKS: usize, T> {
    pub key: Pool<T>,
    pub children: Vec<Pairing<CHUNKS, T>>,
}

impl<const CHUNKS: usize, T> From<Pool<T>> for Pairing<CHUNKS, T> {
    fn from(key: Pool<T>) -> Self {
        Self {
            key,
            children: vec![],
        }
    }
}

impl<const CHUNKS: usize, T> Pairing<CHUNKS, T> {
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

impl<const CHUNKS: usize, T: Ord> Pairing<CHUNKS, T> {
    #[must_use]
    pub fn meld(self, other: Pairing<CHUNKS, T>) -> Pairing<CHUNKS, T> {
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

    /// This variant might be better in practice than `merge_children`,
    /// but it's harder to analyse in theory.
    pub fn merge_children_multipass(items: Vec<Self>) -> Option<Self> {
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

    pub fn merge_children(items: Vec<Self>) -> Option<Self> {
        // Conventional two-pass strategy, but with bigger chunks and corruption.

        let first_pass: Vec<_> = items
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
            })
            .collect::<Vec<_>>();

        // We need to reverse here, to implement the conventional two-pass strategy.
        first_pass.into_iter().rev().reduce(Self::meld)
    }

    pub fn check_heap_property(&self) -> bool {
        let Pairing { key, children } = self;
        children.iter().all(|child| key.item <= child.key.item)
            && children.iter().all(Self::check_heap_property)
    }
}

// Get all non-corrupted elements still in the heap.
impl<const CHUNKS: usize, T> From<Pairing<CHUNKS, T>> for Vec<T> {
    fn from(pairing: Pairing<CHUNKS, T>) -> Self {
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
pub struct Heap<const CHUNKS: usize, T> {
    pub root: Option<Pairing<CHUNKS, T>>,
}

impl<const CHUNKS: usize, T> Default for Heap<CHUNKS, T> {
    fn default() -> Self {
        Self { root: None }
    }
}

impl<const CHUNKS: usize, T: Ord> Heap<CHUNKS, T> {
    #[must_use]
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

    #[must_use]
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

impl<const CHUNKS: usize, T> From<Heap<CHUNKS, T>> for Vec<T> {
    fn from(Heap { root }: Heap<CHUNKS, T>) -> Self {
        root.map(Vec::from).unwrap_or_default()
    }
}
