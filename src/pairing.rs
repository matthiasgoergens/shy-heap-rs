// Soft heaps based on pairing heaps.
// We do min-heaps by default.

use std::{collections::VecDeque, vec};

use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Pool<T> {
    pub item: T,
    pub count: isize,
}

impl<T> Pool<T> {
    pub fn new(item: T) -> Self {
        Pool { item, count: 0 }
    }

    pub fn pop(self) -> (Option<T>, Option<Self>) {
        assert!(self.count >= 0);
        if self.count <= 0 {
            (Some(self.item), None)
        } else {
            (
                None,
                Some(Self {
                    count: self.count - 1,
                    ..self
                }),
            )
        }
    }

    pub fn merge(self, other: Self) -> Self {
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

    pub fn pop_min(self) -> (Option<T>, Option<Self>) {
        let Pairing { key, children } = self;
        let (popped, remainder) = key.pop();
        (
            popped,
            match remainder {
                None => Self::merge_pairs(children),
                Some(key) => Some(Self { key, children }),
            },
        )
    }

    pub fn sift_min(self) -> Self {
        let Pairing { key, children } = self;
        match Self::merge_pairs(children) {
            None => Pairing::from(key),
            Some(pairing) => {
                assert!(key.item <= pairing.key.item);
                Pairing {
                    key: key.merge(pairing.key),
                    ..pairing
                }
            }
        }
    }

    pub fn delete_min(self) -> Option<Self> {
        let (_, children) = self.pop_min();
        children
    }

    pub fn merge_pairs(items: Vec<Self>) -> Option<Self> {
        // We can probably also just chunk M items at a time (via a reduce),
        // sift every full chunk (or every but the last chunk?) and then reduce everything once (without any sifting).
        // Instead of our tree based approach.
        let mut items = items;
        loop {
            for _ in 0..EVERY {
                items = items
                    .into_iter()
                    .chunks(2)
                    .into_iter()
                    .filter_map(|pair| pair.reduce(Self::meld))
                    .collect();
                if items.len() < 2 {
                    return items.into_iter().reduce(Self::meld);
                }
            }
            // This is potentially arbitrarily recursive.
            items = items.into_iter().map(Self::sift_min).collect();
        }
    }

    pub fn merge_pairs_flat(items: Vec<Self>) -> Option<Self> {
        items
            .into_iter()
            .chunks(CHUNKS)
            .into_iter()
            .filter_map(|chunk| chunk.reduce(Self::meld))
            .reduce(|a, b| a.meld(b).sift_min())
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
                key: Pool { item, count },
                children,
            } = pairing;
            assert!(count >= 0);
            todo.extend(children);
            items.push(item);
        }
        items
    }
}

/// This one controls the soft heap's 'epsilon' corruption behaviour.
const EVERY: usize = 3;
const CHUNKS: usize = 4;
